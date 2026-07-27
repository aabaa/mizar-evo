//! Syntax-free transport for source set, comprehension, choice, and `qua` terms.

use crate::{
    binding_env::{BindingContextId, BindingEnv},
    source_application::{
        SourceFunctorApplicationHandoff, SourceFunctorApplicationId, SourceFunctorArgumentTarget,
    },
    source_structure::{SourceStructureHandoff, SourceStructureTarget, SourceStructureTermId},
    source_term::{SourcePrimaryTermHandoff, SourcePrimaryTermId},
    typed_ast::{NodeRecoveryState, TypedArena, TypedSiteRef},
};
use mizar_resolve::resolved_ast::ModuleId;
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

dense_id!(SourceSetTermId);
dense_id!(SourceSetWrapperId);
dense_id!(SourceSetGeneratorId);
dense_id!(SourceSetTypeSiteId);
dense_id!(SourceSetEdgeId);
dense_id!(SourceSetRequestId);

/// Complete syntax-free input for one source/module set-term transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetTermHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub terms: Vec<SourceSetTermInput>,
    pub wrappers: Vec<SourceSetWrapperInput>,
    pub generators: Vec<SourceSetGeneratorInput>,
    pub type_sites: Vec<SourceSetTypeSiteInput>,
    pub edges: Vec<SourceSetEdgeInput>,
    pub requests: Vec<SourceSetRequestInput>,
}

/// One set-family term occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetTermInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceSetTermRecovery,
    pub spelling: String,
    pub kind: SourceSetTermKind,
}

/// One transparent parenthesized set-family wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetWrapperInput {
    pub term: SourceSetTermId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceSetTermRecovery,
    pub spelling: String,
}

/// One written comprehension generator declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetGeneratorInput {
    pub term: SourceSetTermId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub context: BindingContextId,
    pub recovery: SourceSetTermRecovery,
    pub type_site: SourceSetTypeSiteId,
}

/// One bare builtin target-type occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetTypeSiteInput {
    pub owner: SourceSetTypeOwner,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub head_site: TypedSiteRef,
    pub head_range: SourceRange,
    pub head_spelling: String,
    pub context: BindingContextId,
    pub recovery: SourceSetTermRecovery,
    pub head: SourceSetTypeHead,
}

/// One ordered child association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetEdgeInput {
    pub term: SourceSetTermId,
    pub ordinal: usize,
    pub role: SourceSetEdgeRole,
    pub target: SourceSetTarget,
}

/// One unresolved set-family dependency request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetRequestInput {
    pub term: SourceSetTermId,
    pub ordinal: usize,
    pub kind: SourceSetRequestKind,
    pub generator: Option<SourceSetGeneratorId>,
    pub type_site: Option<SourceSetTypeSiteId>,
}

/// Source set-family shape admitted by Task 255.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceSetTermKind {
    Enumeration,
    Comprehension,
    Choice,
    Qua,
}

/// Recovery state retained at the source-set-term boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceSetTermRecovery {
    Normal,
    Degraded,
}

/// Owner of one bare builtin target-type site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceSetTypeOwner {
    Generator(SourceSetGeneratorId),
    Term {
        term: SourceSetTermId,
        role: SourceSetTypeRole,
    },
}

/// Term-owned target-type role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceSetTypeRole {
    ChoiceTarget,
    QuaTarget,
}

/// Bare builtin target head retained by Task 255.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceSetTypeHead {
    BuiltinSet,
    BuiltinObject,
}

/// Source role of one set-family child edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceSetEdgeRole {
    EnumerationElement,
    ComprehensionMapper,
    QuaBase,
}

/// Cross-family target owned by one set-family child occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceSetTarget {
    Primary(SourcePrimaryTermId),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
    SetTerm(SourceSetTermId),
}

/// Unresolved set-family dependency request kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceSetRequestKind {
    ResultType,
    GeneratorSethood,
    ChoiceNonempty,
    QuaWidening,
}

/// Immutable validated source-set-term handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetTermHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    primary_term_fingerprint: String,
    application_fingerprint: Option<String>,
    structure_fingerprint: Option<String>,
    terms: SourceSetTermTable,
    wrappers: SourceSetWrapperTable,
    generators: SourceSetGeneratorTable,
    type_sites: SourceSetTypeSiteTable,
    edges: SourceSetEdgeTable,
    requests: SourceSetRequestTable,
}

impl SourceSetTermHandoff {
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

    pub fn structure_fingerprint(&self) -> Option<&str> {
        self.structure_fingerprint.as_deref()
    }

    pub const fn terms(&self) -> &SourceSetTermTable {
        &self.terms
    }

    pub const fn wrappers(&self) -> &SourceSetWrapperTable {
        &self.wrappers
    }

    pub const fn generators(&self) -> &SourceSetGeneratorTable {
        &self.generators
    }

    pub const fn type_sites(&self) -> &SourceSetTypeSiteTable {
        &self.type_sites
    }

    pub const fn edges(&self) -> &SourceSetEdgeTable {
        &self.edges
    }

    pub const fn requests(&self) -> &SourceSetRequestTable {
        &self.requests
    }

    /// Stable, source-ordered representation used as the dependency fingerprint.
    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-set-term-debug-v1\n");
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
        let _ = writeln!(
            output,
            "structure-fingerprint: {:?}",
            self.structure_fingerprint
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
        for (id, generator) in self.generators.iter() {
            let _ = writeln!(
                output,
                "generator#{} term={} ordinal={} range={}..{} site={} context={} recovery={} spelling={:?} type_site={}",
                id.index(),
                generator.term.index(),
                generator.ordinal,
                generator.source_range.start,
                generator.source_range.end,
                generator.site.node().index(),
                generator.context.index(),
                recovery_key(generator.recovery),
                generator.spelling,
                generator.type_site.index(),
            );
        }
        for (id, type_site) in self.type_sites.iter() {
            let _ = write!(output, "type-site#{} owner=", id.index(),);
            write_type_owner(&mut output, type_site.owner);
            let _ = writeln!(
                output,
                " range={}..{} site={} spelling={:?} head_range={}..{} head_site={} head_spelling={:?} context={} recovery={} head={}",
                type_site.source_range.start,
                type_site.source_range.end,
                type_site.site.node().index(),
                type_site.spelling,
                type_site.head_range.start,
                type_site.head_range.end,
                type_site.head_site.node().index(),
                type_site.head_spelling,
                type_site.context.index(),
                recovery_key(type_site.recovery),
                type_head_key(type_site.head),
            );
        }
        for (id, edge) in self.edges.iter() {
            let _ = write!(
                output,
                "edge#{} term={} ordinal={} role={} target=",
                id.index(),
                edge.term.index(),
                edge.ordinal,
                edge_role_key(edge.role),
            );
            write_target(&mut output, edge.target);
            output.push('\n');
        }
        for (id, request) in self.requests.iter() {
            let _ = write!(
                output,
                "request#{} term={} ordinal={} kind={} generator=",
                id.index(),
                request.term.index(),
                request.ordinal,
                request_kind_key(request.kind),
            );
            write_optional_id(
                &mut output,
                request.generator.map(SourceSetGeneratorId::index),
            );
            output.push_str(" type_site=");
            write_optional_id(
                &mut output,
                request.type_site.map(SourceSetTypeSiteId::index),
            );
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
        structures: Option<&SourceStructureHandoff>,
        arena: &TypedArena,
    ) -> Result<(), SourceSetTermError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.primary_term_fingerprint != primary_terms.debug_text()
        {
            return Err(SourceSetTermError::PrimaryDependencyMismatch);
        }
        primary_terms
            .validate_installation(source_id, module_id, arena)
            .map_err(|_| SourceSetTermError::PrimaryDependencyMismatch)?;
        if let Some(applications) = applications {
            applications
                .validate_installation(source_id, module_id, primary_terms)
                .map_err(|_| SourceSetTermError::ApplicationDependencyMismatch)?;
        }
        if let Some(structures) = structures {
            structures
                .validate_installation(source_id, module_id, primary_terms, applications, arena)
                .map_err(|_| SourceSetTermError::StructureDependencyMismatch)?;
        }
        let input = self.to_input();
        validate_payload(&input, None, primary_terms, applications, structures, arena)?;
        let uses_applications = input
            .edges
            .iter()
            .any(|edge| matches!(edge.target, SourceSetTarget::Application(_)));
        let uses_structures = input
            .edges
            .iter()
            .any(|edge| matches!(edge.target, SourceSetTarget::Structure(_)));
        let expected_application =
            uses_applications.then(|| applications.expect("dependency validated").debug_text());
        let expected_structure =
            uses_structures.then(|| structures.expect("dependency validated").debug_text());
        if self.application_fingerprint != expected_application {
            return Err(SourceSetTermError::ApplicationDependencyMismatch);
        }
        if self.structure_fingerprint != expected_structure {
            return Err(SourceSetTermError::StructureDependencyMismatch);
        }
        Ok(())
    }

    fn to_input(&self) -> SourceSetTermHandoffInput {
        SourceSetTermHandoffInput {
            source_id: self.source_id,
            module_id: self.module_id.clone(),
            terms: self
                .terms
                .iter()
                .map(|(_, row)| SourceSetTermInput {
                    site: row.site.clone(),
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling.clone(),
                    kind: row.kind,
                })
                .collect(),
            wrappers: self
                .wrappers
                .iter()
                .map(|(_, row)| SourceSetWrapperInput {
                    term: row.term,
                    ordinal: row.ordinal,
                    site: row.site.clone(),
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling.clone(),
                })
                .collect(),
            generators: self
                .generators
                .iter()
                .map(|(_, row)| SourceSetGeneratorInput {
                    term: row.term,
                    ordinal: row.ordinal,
                    site: row.site.clone(),
                    source_range: row.source_range,
                    spelling: row.spelling.clone(),
                    context: row.context,
                    recovery: row.recovery,
                    type_site: row.type_site,
                })
                .collect(),
            type_sites: self
                .type_sites
                .iter()
                .map(|(_, row)| SourceSetTypeSiteInput {
                    owner: row.owner,
                    site: row.site.clone(),
                    source_range: row.source_range,
                    spelling: row.spelling.clone(),
                    head_site: row.head_site.clone(),
                    head_range: row.head_range,
                    head_spelling: row.head_spelling.clone(),
                    context: row.context,
                    recovery: row.recovery,
                    head: row.head,
                })
                .collect(),
            edges: self
                .edges
                .iter()
                .map(|(_, row)| SourceSetEdgeInput {
                    term: row.term,
                    ordinal: row.ordinal,
                    role: row.role,
                    target: row.target,
                })
                .collect(),
            requests: self
                .requests
                .iter()
                .map(|(_, row)| SourceSetRequestInput {
                    term: row.term,
                    ordinal: row.ordinal,
                    kind: row.kind,
                    generator: row.generator,
                    type_site: row.type_site,
                })
                .collect(),
        }
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

table!(SourceSetTermTable, SourceSetTerm, SourceSetTermId);
table!(SourceSetWrapperTable, SourceSetWrapper, SourceSetWrapperId);
table!(
    SourceSetGeneratorTable,
    SourceSetGenerator,
    SourceSetGeneratorId
);
table!(
    SourceSetTypeSiteTable,
    SourceSetTypeSite,
    SourceSetTypeSiteId
);
table!(SourceSetEdgeTable, SourceSetEdge, SourceSetEdgeId);
table!(SourceSetRequestTable, SourceSetRequest, SourceSetRequestId);

/// One validated set-family term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetTerm {
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourceSetTermRecovery,
    spelling: String,
    kind: SourceSetTermKind,
}

impl SourceSetTerm {
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
    pub const fn recovery(&self) -> SourceSetTermRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn kind(&self) -> SourceSetTermKind {
        self.kind
    }
}

/// One validated transparent set-family wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetWrapper {
    term: SourceSetTermId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceSetTermRecovery,
    spelling: String,
}

impl SourceSetWrapper {
    pub const fn term(&self) -> SourceSetTermId {
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
    pub const fn recovery(&self) -> SourceSetTermRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

/// One validated comprehension generator declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetGenerator {
    term: SourceSetTermId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    context: BindingContextId,
    recovery: SourceSetTermRecovery,
    type_site: SourceSetTypeSiteId,
}

impl SourceSetGenerator {
    pub const fn term(&self) -> SourceSetTermId {
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
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub const fn recovery(&self) -> SourceSetTermRecovery {
        self.recovery
    }
    pub const fn type_site(&self) -> SourceSetTypeSiteId {
        self.type_site
    }
}

/// One validated bare builtin target-type occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetTypeSite {
    owner: SourceSetTypeOwner,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    head_site: TypedSiteRef,
    head_range: SourceRange,
    head_spelling: String,
    context: BindingContextId,
    recovery: SourceSetTermRecovery,
    head: SourceSetTypeHead,
}

impl SourceSetTypeSite {
    pub const fn owner(&self) -> SourceSetTypeOwner {
        self.owner
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
    pub const fn head_site(&self) -> &TypedSiteRef {
        &self.head_site
    }
    pub const fn head_range(&self) -> SourceRange {
        self.head_range
    }
    pub fn head_spelling(&self) -> &str {
        &self.head_spelling
    }
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub const fn recovery(&self) -> SourceSetTermRecovery {
        self.recovery
    }
    pub const fn head(&self) -> SourceSetTypeHead {
        self.head
    }
}

/// One validated ordered child edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetEdge {
    term: SourceSetTermId,
    ordinal: usize,
    role: SourceSetEdgeRole,
    target: SourceSetTarget,
}

impl SourceSetEdge {
    pub const fn term(&self) -> SourceSetTermId {
        self.term
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn role(&self) -> SourceSetEdgeRole {
        self.role
    }
    pub const fn target(&self) -> SourceSetTarget {
        self.target
    }
}

/// One validated unresolved set-family request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSetRequest {
    term: SourceSetTermId,
    ordinal: usize,
    kind: SourceSetRequestKind,
    generator: Option<SourceSetGeneratorId>,
    type_site: Option<SourceSetTypeSiteId>,
}

impl SourceSetRequest {
    pub const fn term(&self) -> SourceSetTermId {
        self.term
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceSetRequestKind {
        self.kind
    }
    pub const fn generator(&self) -> Option<SourceSetGeneratorId> {
        self.generator
    }
    pub const fn type_site(&self) -> Option<SourceSetTypeSiteId> {
        self.type_site
    }
}

/// Atomically validates and constructs source-set-term handoffs.
pub struct SourceSetTermProducer;

impl SourceSetTermProducer {
    pub fn build(
        input: SourceSetTermHandoffInput,
        bindings: &BindingEnv,
        primary_terms: &SourcePrimaryTermHandoff,
        applications: Option<&SourceFunctorApplicationHandoff>,
        structures: Option<&SourceStructureHandoff>,
        arena: &TypedArena,
    ) -> Result<SourceSetTermHandoff, SourceSetTermError> {
        validate_input(
            &input,
            bindings,
            primary_terms,
            applications,
            structures,
            arena,
        )?;

        let uses_applications = input
            .edges
            .iter()
            .any(|edge| matches!(edge.target, SourceSetTarget::Application(_)));
        let uses_structures = input
            .edges
            .iter()
            .any(|edge| matches!(edge.target, SourceSetTarget::Structure(_)));
        let primary_term_fingerprint = primary_terms.debug_text();
        let application_fingerprint =
            uses_applications.then(|| applications.expect("dependency validated").debug_text());
        let structure_fingerprint =
            uses_structures.then(|| structures.expect("dependency validated").debug_text());

        Ok(SourceSetTermHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            primary_term_fingerprint,
            application_fingerprint,
            structure_fingerprint,
            terms: SourceSetTermTable {
                rows: input
                    .terms
                    .into_iter()
                    .map(|row| SourceSetTerm {
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        context: row.context,
                        recovery: row.recovery,
                        spelling: row.spelling,
                        kind: row.kind,
                    })
                    .collect(),
            },
            wrappers: SourceSetWrapperTable {
                rows: input
                    .wrappers
                    .into_iter()
                    .map(|row| SourceSetWrapper {
                        term: row.term,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        context: row.context,
                        recovery: row.recovery,
                        spelling: row.spelling,
                    })
                    .collect(),
            },
            generators: SourceSetGeneratorTable {
                rows: input
                    .generators
                    .into_iter()
                    .map(|row| SourceSetGenerator {
                        term: row.term,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        context: row.context,
                        recovery: row.recovery,
                        type_site: row.type_site,
                    })
                    .collect(),
            },
            type_sites: SourceSetTypeSiteTable {
                rows: input
                    .type_sites
                    .into_iter()
                    .map(|row| SourceSetTypeSite {
                        owner: row.owner,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        head_site: row.head_site,
                        head_range: row.head_range,
                        head_spelling: row.head_spelling,
                        context: row.context,
                        recovery: row.recovery,
                        head: row.head,
                    })
                    .collect(),
            },
            edges: SourceSetEdgeTable {
                rows: input
                    .edges
                    .into_iter()
                    .map(|row| SourceSetEdge {
                        term: row.term,
                        ordinal: row.ordinal,
                        role: row.role,
                        target: row.target,
                    })
                    .collect(),
            },
            requests: SourceSetRequestTable {
                rows: input
                    .requests
                    .into_iter()
                    .map(|row| SourceSetRequest {
                        term: row.term,
                        ordinal: row.ordinal,
                        kind: row.kind,
                        generator: row.generator,
                        type_site: row.type_site,
                    })
                    .collect(),
            },
        })
    }
}

/// Atomic Task-255 producer failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceSetTermError {
    EnvironmentMismatch,
    PrimaryDependencyMismatch,
    ApplicationDependencyMismatch,
    StructureDependencyMismatch,
    InvalidTerm { term: SourceSetTermId },
    InvalidWrapper { wrapper: SourceSetWrapperId },
    InvalidGenerator { generator: SourceSetGeneratorId },
    InvalidTypeSite { type_site: SourceSetTypeSiteId },
    InvalidEdge { edge: SourceSetEdgeId },
    InvalidRequest { request: SourceSetRequestId },
    DuplicateSite,
    ReorderedTerm { term: SourceSetTermId },
    ReorderedWrapper { wrapper: SourceSetWrapperId },
    ReorderedGenerator { generator: SourceSetGeneratorId },
    ReorderedTypeSite { type_site: SourceSetTypeSiteId },
    ReorderedEdge { edge: SourceSetEdgeId },
    ReorderedRequest { request: SourceSetRequestId },
    MultipleParents { term: SourceSetTermId },
    DuplicateTarget { edge: SourceSetEdgeId },
    OverlappingChildren { term: SourceSetTermId },
}

impl fmt::Display for SourceSetTermError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("source set-term environment identity mismatch")
            }
            Self::PrimaryDependencyMismatch => {
                formatter.write_str("source set-term primary dependency mismatch")
            }
            Self::ApplicationDependencyMismatch => {
                formatter.write_str("source set-term application dependency mismatch")
            }
            Self::StructureDependencyMismatch => {
                formatter.write_str("source set-term structure dependency mismatch")
            }
            Self::InvalidTerm { term } => {
                write!(formatter, "source set term {} is invalid", term.index())
            }
            Self::InvalidWrapper { wrapper } => {
                write!(
                    formatter,
                    "source set wrapper {} is invalid",
                    wrapper.index()
                )
            }
            Self::InvalidGenerator { generator } => write!(
                formatter,
                "source set generator {} is invalid",
                generator.index()
            ),
            Self::InvalidTypeSite { type_site } => write!(
                formatter,
                "source set type site {} is invalid",
                type_site.index()
            ),
            Self::InvalidEdge { edge } => {
                write!(formatter, "source set edge {} is invalid", edge.index())
            }
            Self::InvalidRequest { request } => write!(
                formatter,
                "source set request {} is invalid",
                request.index()
            ),
            Self::DuplicateSite => formatter.write_str("source set term repeats a typed site"),
            Self::ReorderedTerm { term } => write!(
                formatter,
                "source set term {} is out of source pre-order",
                term.index()
            ),
            Self::ReorderedWrapper { wrapper } => write!(
                formatter,
                "source set wrapper {} is out of order",
                wrapper.index()
            ),
            Self::ReorderedGenerator { generator } => write!(
                formatter,
                "source set generator {} is out of order",
                generator.index()
            ),
            Self::ReorderedTypeSite { type_site } => write!(
                formatter,
                "source set type site {} is out of source order",
                type_site.index()
            ),
            Self::ReorderedEdge { edge } => {
                write!(
                    formatter,
                    "source set edge {} is out of order",
                    edge.index()
                )
            }
            Self::ReorderedRequest { request } => write!(
                formatter,
                "source set request {} is out of order",
                request.index()
            ),
            Self::MultipleParents { term } => write!(
                formatter,
                "source set term {} has multiple parents",
                term.index()
            ),
            Self::DuplicateTarget { edge } => write!(
                formatter,
                "source set edge {} repeats an owned occurrence",
                edge.index()
            ),
            Self::OverlappingChildren { term } => write!(
                formatter,
                "source set term {} has overlapping children",
                term.index()
            ),
        }
    }
}

impl Error for SourceSetTermError {}

fn validate_input(
    input: &SourceSetTermHandoffInput,
    bindings: &BindingEnv,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceSetTermError> {
    if bindings.source_id() != input.source_id
        || bindings.module_id() != &input.module_id
        || primary_terms.source_id() != input.source_id
        || primary_terms.module_id() != &input.module_id
    {
        return Err(SourceSetTermError::EnvironmentMismatch);
    }
    primary_terms
        .validate_installation(input.source_id, &input.module_id, arena)
        .map_err(|_| SourceSetTermError::PrimaryDependencyMismatch)?;
    if input.terms.is_empty()
        && (!input.wrappers.is_empty()
            || !input.generators.is_empty()
            || !input.type_sites.is_empty()
            || !input.edges.is_empty()
            || !input.requests.is_empty())
    {
        return Err(SourceSetTermError::EnvironmentMismatch);
    }

    if let Some(applications) = applications {
        if applications.source_id() != input.source_id
            || applications.module_id() != &input.module_id
            || applications.primary_term_fingerprint() != primary_terms.debug_text()
        {
            return Err(SourceSetTermError::ApplicationDependencyMismatch);
        }
        applications
            .validate_installation(input.source_id, &input.module_id, primary_terms)
            .map_err(|_| SourceSetTermError::ApplicationDependencyMismatch)?;
    }
    if let Some(structures) = structures {
        if structures.source_id() != input.source_id
            || structures.module_id() != &input.module_id
            || structures.primary_term_fingerprint() != primary_terms.debug_text()
        {
            return Err(SourceSetTermError::StructureDependencyMismatch);
        }
        structures
            .validate_installation(
                input.source_id,
                &input.module_id,
                primary_terms,
                applications,
                arena,
            )
            .map_err(|_| SourceSetTermError::StructureDependencyMismatch)?;
    }

    let uses_applications = input
        .edges
        .iter()
        .any(|edge| matches!(edge.target, SourceSetTarget::Application(_)));
    let uses_structures = input
        .edges
        .iter()
        .any(|edge| matches!(edge.target, SourceSetTarget::Structure(_)));
    if uses_applications && applications.is_none() {
        return Err(SourceSetTermError::ApplicationDependencyMismatch);
    }
    if uses_structures && structures.is_none() {
        return Err(SourceSetTermError::StructureDependencyMismatch);
    }

    validate_payload(
        input,
        Some(bindings),
        primary_terms,
        applications,
        structures,
        arena,
    )
}

fn validate_payload(
    input: &SourceSetTermHandoffInput,
    bindings: Option<&BindingEnv>,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceSetTermError> {
    let uses_applications = input
        .edges
        .iter()
        .any(|edge| matches!(edge.target, SourceSetTarget::Application(_)));
    let uses_structures = input
        .edges
        .iter()
        .any(|edge| matches!(edge.target, SourceSetTarget::Structure(_)));
    if uses_applications && applications.is_none() {
        return Err(SourceSetTermError::ApplicationDependencyMismatch);
    }
    if uses_structures && structures.is_none() {
        return Err(SourceSetTermError::StructureDependencyMismatch);
    }

    let mut sites = BTreeSet::new();
    validate_terms(input, bindings, arena, &mut sites)?;
    let wrapper_groups = validate_wrappers(input, arena, &mut sites)?;
    let effective = input_effective_occurrences(input, &wrapper_groups);
    validate_term_preorder(input, &effective)?;
    let generator_groups = validate_generators(input, arena, &mut sites)?;
    validate_type_sites(input, &generator_groups, arena, &mut sites)?;
    validate_cross_family_relationships(
        input,
        primary_terms,
        applications,
        structures,
        &effective,
    )?;
    let edge_groups = validate_edges(
        input,
        primary_terms,
        applications,
        structures,
        &effective,
        &mut sites,
    )?;
    validate_set_term_ownership(input, &effective)?;
    validate_shapes_and_spelling(
        input,
        primary_terms,
        applications,
        structures,
        &effective,
        &generator_groups,
        &edge_groups,
    )?;
    validate_requests(input, &generator_groups)?;
    Ok(())
}

fn validate_terms(
    input: &SourceSetTermHandoffInput,
    bindings: Option<&BindingEnv>,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceSetTermError> {
    for (index, term) in input.terms.iter().enumerate() {
        let id = SourceSetTermId::new(index);
        if term.source_ordinal != index
            || bindings.is_some_and(|bindings| bindings.contexts().get(term.context).is_none())
            || !valid_range(input.source_id, term.source_range)
            || !canonical_spelling(&term.spelling)
        {
            return Err(SourceSetTermError::InvalidTerm { term: id });
        }
        validate_arena_site(
            &term.site,
            term.source_range,
            term_kind_node_key(term.kind),
            term.recovery,
            arena,
        )
        .map_err(|()| SourceSetTermError::InvalidTerm { term: id })?;
        if !sites.insert(term.site.clone()) {
            return Err(SourceSetTermError::DuplicateSite);
        }
    }
    Ok(())
}

fn validate_wrappers(
    input: &SourceSetTermHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceSetTermError> {
    let groups = grouped_rows(
        input.terms.len(),
        &input.wrappers,
        |row| row.term,
        |row| row.ordinal,
        |index| SourceSetTermError::ReorderedWrapper {
            wrapper: SourceSetWrapperId::new(index),
        },
    )?;
    for (term_index, group) in groups.iter().enumerate() {
        let term_id = SourceSetTermId::new(term_index);
        let term = &input.terms[term_index];
        let mut contained_range = term.source_range;
        let mut contained_spelling = term.spelling.as_str();
        for wrapper_index in group.iter().rev().copied() {
            let id = SourceSetWrapperId::new(wrapper_index);
            let wrapper = &input.wrappers[wrapper_index];
            if wrapper.term != term_id
                || wrapper.context != term.context
                || !valid_range(input.source_id, wrapper.source_range)
                || !strictly_contains(wrapper.source_range, contained_range)
                || wrapper.spelling != format!("( {contained_spelling} )")
            {
                return Err(SourceSetTermError::InvalidWrapper { wrapper: id });
            }
            validate_arena_site(
                &wrapper.site,
                wrapper.source_range,
                "source.term.set.parenthesized",
                wrapper.recovery,
                arena,
            )
            .map_err(|()| SourceSetTermError::InvalidWrapper { wrapper: id })?;
            if !sites.insert(wrapper.site.clone()) {
                return Err(SourceSetTermError::DuplicateSite);
            }
            contained_range = wrapper.source_range;
            contained_spelling = &wrapper.spelling;
        }
    }
    Ok(groups)
}

fn validate_term_preorder(
    input: &SourceSetTermHandoffInput,
    effective: &[EffectiveOccurrence],
) -> Result<(), SourceSetTermError> {
    for right in 1..input.terms.len() {
        for left in 0..right {
            let left_range = effective[left].range;
            let right_range = effective[right].range;
            if right_range.start < left_range.start
                || (ranges_overlap(left_range, right_range)
                    && !properly_contains(left_range, right_range))
            {
                return Err(SourceSetTermError::ReorderedTerm {
                    term: SourceSetTermId::new(right),
                });
            }
        }
    }
    Ok(())
}

fn validate_generators(
    input: &SourceSetTermHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceSetTermError> {
    let groups = grouped_rows(
        input.terms.len(),
        &input.generators,
        |row| row.term,
        |row| row.ordinal,
        |index| SourceSetTermError::ReorderedGenerator {
            generator: SourceSetGeneratorId::new(index),
        },
    )?;
    for (term_index, group) in groups.iter().enumerate() {
        let term = &input.terms[term_index];
        if (term.kind == SourceSetTermKind::Comprehension) == group.is_empty() {
            return Err(SourceSetTermError::InvalidTerm {
                term: SourceSetTermId::new(term_index),
            });
        }
        let mut previous_range = None;
        for generator_index in group.iter().copied() {
            let id = SourceSetGeneratorId::new(generator_index);
            let generator = &input.generators[generator_index];
            if generator.term != SourceSetTermId::new(term_index)
                || generator.context != term.context
                || !valid_range(input.source_id, generator.source_range)
                || !properly_contains(term.source_range, generator.source_range)
                || !identifier_spelling(&generator.spelling)
                || previous_range.is_some_and(|previous: SourceRange| {
                    previous.end > generator.source_range.start
                })
            {
                return Err(SourceSetTermError::InvalidGenerator { generator: id });
            }
            validate_arena_site(
                &generator.site,
                generator.source_range,
                "source.term.set.comprehension-generator",
                generator.recovery,
                arena,
            )
            .map_err(|()| SourceSetTermError::InvalidGenerator { generator: id })?;
            if !sites.insert(generator.site.clone()) {
                return Err(SourceSetTermError::DuplicateSite);
            }
            previous_range = Some(generator.source_range);
        }
    }
    Ok(groups)
}

fn validate_type_sites(
    input: &SourceSetTermHandoffInput,
    generator_groups: &[Vec<usize>],
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceSetTermError> {
    let mut generator_owners = BTreeSet::new();
    let mut term_owners = BTreeSet::new();
    let mut previous_range = None;
    for (index, type_site) in input.type_sites.iter().enumerate() {
        let id = SourceSetTypeSiteId::new(index);
        let (term_id, expected_context, valid_owner) = match type_site.owner {
            SourceSetTypeOwner::Generator(generator_id) => {
                let Some(generator) = input.generators.get(generator_id.index()) else {
                    return Err(SourceSetTermError::InvalidTypeSite { type_site: id });
                };
                (
                    generator.term,
                    generator.context,
                    generator.type_site == id && generator_owners.insert(generator_id),
                )
            }
            SourceSetTypeOwner::Term { term, role } => {
                let Some(owner) = input.terms.get(term.index()) else {
                    return Err(SourceSetTermError::InvalidTypeSite { type_site: id });
                };
                let kind_matches = matches!(
                    (owner.kind, role),
                    (SourceSetTermKind::Choice, SourceSetTypeRole::ChoiceTarget)
                        | (SourceSetTermKind::Qua, SourceSetTypeRole::QuaTarget)
                );
                (
                    term,
                    owner.context,
                    kind_matches && term_owners.insert((term, role)),
                )
            }
        };
        let Some(term) = input.terms.get(term_id.index()) else {
            return Err(SourceSetTermError::InvalidTypeSite { type_site: id });
        };
        let expected_spelling = type_head_spelling(type_site.head);
        if !valid_owner
            || type_site.context != expected_context
            || !valid_range(input.source_id, type_site.source_range)
            || !valid_range(input.source_id, type_site.head_range)
            || !properly_contains(term.source_range, type_site.source_range)
            || type_site.source_range != type_site.head_range
            || type_site.spelling != expected_spelling
            || type_site.head_spelling != expected_spelling
        {
            return Err(SourceSetTermError::InvalidTypeSite { type_site: id });
        }
        if previous_range
            .is_some_and(|previous: SourceRange| previous.end > type_site.source_range.start)
        {
            return Err(SourceSetTermError::ReorderedTypeSite { type_site: id });
        }
        validate_arena_site(
            &type_site.site,
            type_site.source_range,
            "source.term.set.target-type",
            type_site.recovery,
            arena,
        )
        .map_err(|()| SourceSetTermError::InvalidTypeSite { type_site: id })?;
        validate_arena_site(
            &type_site.head_site,
            type_site.head_range,
            "source.term.set.target-type-head",
            type_site.recovery,
            arena,
        )
        .map_err(|()| SourceSetTermError::InvalidTypeSite { type_site: id })?;
        if !sites.insert(type_site.site.clone()) || !sites.insert(type_site.head_site.clone()) {
            return Err(SourceSetTermError::DuplicateSite);
        }
        previous_range = Some(type_site.source_range);
    }

    for (generator_index, generator) in input.generators.iter().enumerate() {
        let id = SourceSetGeneratorId::new(generator_index);
        let Some(type_site) = input.type_sites.get(generator.type_site.index()) else {
            return Err(SourceSetTermError::InvalidGenerator { generator: id });
        };
        if type_site.owner != SourceSetTypeOwner::Generator(id)
            || generator.source_range.end > type_site.source_range.start
        {
            return Err(SourceSetTermError::InvalidGenerator { generator: id });
        }
    }
    for group in generator_groups {
        for pair in group.windows(2) {
            let current = &input.generators[pair[0]];
            let next = &input.generators[pair[1]];
            let current_type = &input.type_sites[current.type_site.index()];
            if current_type.source_range.end > next.source_range.start {
                return Err(SourceSetTermError::InvalidGenerator {
                    generator: SourceSetGeneratorId::new(pair[1]),
                });
            }
        }
    }
    for (term_index, term) in input.terms.iter().enumerate() {
        let term_id = SourceSetTermId::new(term_index);
        let expected = match term.kind {
            SourceSetTermKind::Choice => Some(SourceSetTypeRole::ChoiceTarget),
            SourceSetTermKind::Qua => Some(SourceSetTypeRole::QuaTarget),
            SourceSetTermKind::Enumeration | SourceSetTermKind::Comprehension => None,
        };
        if expected.is_some_and(|role| !term_owners.contains(&(term_id, role))) {
            return Err(SourceSetTermError::InvalidTerm { term: term_id });
        }
        if expected.is_none()
            && input.type_sites.iter().any(|site| {
                matches!(
                    site.owner,
                    SourceSetTypeOwner::Term { term, .. } if term == term_id
                )
            })
        {
            return Err(SourceSetTermError::InvalidTerm { term: term_id });
        }
        if term.kind == SourceSetTermKind::Comprehension && generator_groups[term_index].is_empty()
        {
            return Err(SourceSetTermError::InvalidTerm { term: term_id });
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct EffectiveOccurrence {
    range: SourceRange,
    site: TypedSiteRef,
    spelling: String,
}

#[derive(Debug, Clone)]
struct TargetOccurrence {
    target: SourceSetTarget,
    range: SourceRange,
    site: TypedSiteRef,
    spelling: String,
    context: BindingContextId,
}

fn validate_cross_family_relationships(
    input: &SourceSetTermHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    effective: &[EffectiveOccurrence],
) -> Result<(), SourceSetTermError> {
    let application_primaries = applications
        .map(application_argument_primary_ids)
        .unwrap_or_default();
    let structure_primaries = structures.map(structure_primary_ids).unwrap_or_default();
    for (id, primary) in primary_terms.terms().iter() {
        if primary.parent().is_some()
            || application_primaries.contains(&id)
            || structure_primaries.contains(&id)
        {
            continue;
        }
        validate_external_range_against_terms(
            input,
            effective,
            primary.source_range(),
            SourceSetTermError::PrimaryDependencyMismatch,
        )?;
    }
    if let Some(applications) = applications {
        for (id, _) in applications.applications().iter() {
            let Some(occurrence) = application_effective_occurrence(applications, id) else {
                return Err(SourceSetTermError::ApplicationDependencyMismatch);
            };
            validate_external_range_against_terms(
                input,
                effective,
                occurrence.range,
                SourceSetTermError::ApplicationDependencyMismatch,
            )?;
        }
    }
    if let Some(structures) = structures {
        for (id, _) in structures.terms().iter() {
            let Some(occurrence) = structure_effective_occurrence(structures, id) else {
                return Err(SourceSetTermError::StructureDependencyMismatch);
            };
            validate_external_range_against_terms(
                input,
                effective,
                occurrence.range,
                SourceSetTermError::StructureDependencyMismatch,
            )?;
        }
    }
    Ok(())
}

fn validate_external_range_against_terms(
    input: &SourceSetTermHandoffInput,
    effective: &[EffectiveOccurrence],
    external: SourceRange,
    error: SourceSetTermError,
) -> Result<(), SourceSetTermError> {
    for (index, term) in input.terms.iter().enumerate() {
        let effective_range = effective[index].range;
        if ranges_overlap(effective_range, external)
            && !properly_contains(term.source_range, external)
        {
            return Err(error.clone());
        }
    }
    Ok(())
}

fn validate_edges(
    input: &SourceSetTermHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    effective: &[EffectiveOccurrence],
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceSetTermError> {
    let groups = grouped_rows(
        input.terms.len(),
        &input.edges,
        |row| row.term,
        |row| row.ordinal,
        |index| SourceSetTermError::ReorderedEdge {
            edge: SourceSetEdgeId::new(index),
        },
    )?;
    let mut owned_targets = BTreeSet::new();
    for (term_index, group) in groups.iter().enumerate() {
        let term_id = SourceSetTermId::new(term_index);
        let candidates = direct_targets(
            input,
            primary_terms,
            applications,
            structures,
            effective,
            term_index,
        )?;
        if candidates.len() != group.len() {
            return Err(SourceSetTermError::InvalidTerm { term: term_id });
        }
        let mut previous_range = None;
        for (ordinal, edge_index) in group.iter().copied().enumerate() {
            let id = SourceSetEdgeId::new(edge_index);
            let edge = &input.edges[edge_index];
            let candidate = &candidates[ordinal];
            if edge.target != candidate.target {
                return Err(SourceSetTermError::InvalidEdge { edge: id });
            }
            if !owned_targets.insert(edge.target) {
                return Err(SourceSetTermError::DuplicateTarget { edge: id });
            }
            if previous_range
                .is_some_and(|previous: SourceRange| previous.end > candidate.range.start)
            {
                return Err(SourceSetTermError::OverlappingChildren { term: term_id });
            }
            if !matches!(candidate.target, SourceSetTarget::SetTerm(_))
                && !sites.insert(candidate.site.clone())
            {
                return Err(SourceSetTermError::DuplicateTarget { edge: id });
            }
            previous_range = Some(candidate.range);
        }
    }
    Ok(groups)
}

fn direct_targets(
    input: &SourceSetTermHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    effective: &[EffectiveOccurrence],
    term_index: usize,
) -> Result<Vec<TargetOccurrence>, SourceSetTermError> {
    let parent = &input.terms[term_index];
    let application_owned_primaries = applications
        .map(application_argument_primary_ids)
        .unwrap_or_default();
    let structure_owned_primaries = structures.map(structure_primary_ids).unwrap_or_default();
    let structure_owned_applications = structures
        .map(structure_application_ids)
        .unwrap_or_default();
    let mut candidates = Vec::new();

    for (id, primary) in primary_terms.terms().iter() {
        if primary.parent().is_some()
            || application_owned_primaries.contains(&id)
            || structure_owned_primaries.contains(&id)
            || !properly_contains(parent.source_range, primary.source_range())
        {
            continue;
        }
        if primary.context() != parent.context {
            return Err(SourceSetTermError::PrimaryDependencyMismatch);
        }
        candidates.push(TargetOccurrence {
            target: SourceSetTarget::Primary(id),
            range: primary.source_range(),
            site: primary.site().clone(),
            spelling: primary.spelling().to_owned(),
            context: primary.context(),
        });
    }

    if let Some(applications) = applications {
        for id in application_root_ids(applications) {
            if structure_owned_applications.contains(&id) {
                continue;
            }
            let application = applications
                .applications()
                .get(id)
                .ok_or(SourceSetTermError::ApplicationDependencyMismatch)?;
            let occurrence = application_effective_occurrence(applications, id)
                .ok_or(SourceSetTermError::ApplicationDependencyMismatch)?;
            if !properly_contains(parent.source_range, occurrence.range) {
                continue;
            }
            if application.context() != parent.context {
                return Err(SourceSetTermError::ApplicationDependencyMismatch);
            }
            candidates.push(TargetOccurrence {
                target: SourceSetTarget::Application(id),
                range: occurrence.range,
                site: occurrence.site,
                spelling: occurrence.spelling,
                context: application.context(),
            });
        }
    }

    if let Some(structures) = structures {
        for id in structure_root_ids(structures) {
            let structure = structures
                .terms()
                .get(id)
                .ok_or(SourceSetTermError::StructureDependencyMismatch)?;
            let occurrence = structure_effective_occurrence(structures, id)
                .ok_or(SourceSetTermError::StructureDependencyMismatch)?;
            if !properly_contains(parent.source_range, occurrence.range) {
                continue;
            }
            if structure.context() != parent.context {
                return Err(SourceSetTermError::StructureDependencyMismatch);
            }
            candidates.push(TargetOccurrence {
                target: SourceSetTarget::Structure(id),
                range: occurrence.range,
                site: occurrence.site,
                spelling: occurrence.spelling,
                context: structure.context(),
            });
        }
    }

    for (child_index, child) in input.terms.iter().enumerate() {
        if child_index == term_index
            || !properly_contains(parent.source_range, effective[child_index].range)
        {
            continue;
        }
        if child.context != parent.context {
            return Err(SourceSetTermError::InvalidTerm {
                term: SourceSetTermId::new(child_index),
            });
        }
        candidates.push(TargetOccurrence {
            target: SourceSetTarget::SetTerm(SourceSetTermId::new(child_index)),
            range: effective[child_index].range,
            site: effective[child_index].site.clone(),
            spelling: effective[child_index].spelling.clone(),
            context: child.context,
        });
    }

    let all = candidates.clone();
    candidates.retain(|candidate| {
        !all.iter().any(|container| {
            container.target != candidate.target
                && properly_contains(container.range, candidate.range)
        })
    });
    candidates
        .sort_by_key(|candidate| (candidate.range.start, candidate.range.end, candidate.target));
    if candidates.windows(2).any(|pair| {
        ranges_overlap(pair[0].range, pair[1].range)
            || pair[0].context != parent.context
            || pair[1].context != parent.context
    }) {
        return Err(SourceSetTermError::OverlappingChildren {
            term: SourceSetTermId::new(term_index),
        });
    }
    Ok(candidates)
}

fn validate_set_term_ownership(
    input: &SourceSetTermHandoffInput,
    effective: &[EffectiveOccurrence],
) -> Result<(), SourceSetTermError> {
    let mut target_counts = vec![0usize; input.terms.len()];
    for (edge_index, edge) in input.edges.iter().enumerate() {
        if let SourceSetTarget::SetTerm(term) = edge.target {
            let Some(count) = target_counts.get_mut(term.index()) else {
                return Err(SourceSetTermError::InvalidEdge {
                    edge: SourceSetEdgeId::new(edge_index),
                });
            };
            *count += 1;
            if *count > 1 {
                return Err(SourceSetTermError::MultipleParents { term });
            }
        }
    }
    for index in 0..input.terms.len() {
        let nested = (0..index).any(|parent| {
            properly_contains(input.terms[parent].source_range, effective[index].range)
        });
        if target_counts[index] != usize::from(nested) {
            return Err(SourceSetTermError::InvalidTerm {
                term: SourceSetTermId::new(index),
            });
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Rationale: keep cross-family spelling inputs explicit at the frozen validation boundary.
fn validate_shapes_and_spelling(
    input: &SourceSetTermHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    effective: &[EffectiveOccurrence],
    generator_groups: &[Vec<usize>],
    edge_groups: &[Vec<usize>],
) -> Result<(), SourceSetTermError> {
    for (term_index, term) in input.terms.iter().enumerate() {
        let term_id = SourceSetTermId::new(term_index);
        let edges = &edge_groups[term_index];
        let targets = edges
            .iter()
            .map(|edge| {
                target_occurrence(
                    input,
                    primary_terms,
                    applications,
                    structures,
                    effective,
                    input.edges[*edge].target,
                )
                .ok_or(SourceSetTermError::InvalidEdge {
                    edge: SourceSetEdgeId::new(*edge),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let generators = &generator_groups[term_index];
        let (expected_spelling, positions_valid) = match term.kind {
            SourceSetTermKind::Enumeration => {
                if !generators.is_empty()
                    || edges.iter().any(|edge| {
                        input.edges[*edge].role != SourceSetEdgeRole::EnumerationElement
                    })
                {
                    return Err(SourceSetTermError::InvalidTerm { term: term_id });
                }
                let spelling = if targets.is_empty() {
                    "{ }".to_owned()
                } else {
                    format!(
                        "{{ {} }}",
                        targets
                            .iter()
                            .map(|target| target.spelling.as_str())
                            .collect::<Vec<_>>()
                            .join(" , ")
                    )
                };
                let valid = targets.first().is_none_or(|first| {
                    term.source_range.start < first.range.start
                        && targets
                            .last()
                            .is_some_and(|last| last.range.end < term.source_range.end)
                });
                (spelling, valid)
            }
            SourceSetTermKind::Comprehension => {
                if edges.len() != 1
                    || input.edges[edges[0]].role != SourceSetEdgeRole::ComprehensionMapper
                    || generators.is_empty()
                {
                    return Err(SourceSetTermError::InvalidTerm { term: term_id });
                }
                let fragments = generators
                    .iter()
                    .map(|generator_index| {
                        let generator = &input.generators[*generator_index];
                        let type_site = input.type_sites.get(generator.type_site.index()).ok_or(
                            SourceSetTermError::InvalidGenerator {
                                generator: SourceSetGeneratorId::new(*generator_index),
                            },
                        )?;
                        Ok(format!("{} is {}", generator.spelling, type_site.spelling))
                    })
                    .collect::<Result<Vec<_>, SourceSetTermError>>()?;
                let first_generator = &input.generators[generators[0]];
                let final_generator = &input.generators[*generators.last().expect("nonempty")];
                let final_type = input
                    .type_sites
                    .get(final_generator.type_site.index())
                    .ok_or(SourceSetTermError::InvalidGenerator {
                        generator: SourceSetGeneratorId::new(*generators.last().expect("nonempty")),
                    })?;
                (
                    format!(
                        "{{ {} where {} }}",
                        targets[0].spelling,
                        fragments.join(" , ")
                    ),
                    term.source_range.start < targets[0].range.start
                        && targets[0].range.end <= first_generator.source_range.start
                        && final_type.source_range.end < term.source_range.end,
                )
            }
            SourceSetTermKind::Choice => {
                if !edges.is_empty() || !generators.is_empty() {
                    return Err(SourceSetTermError::InvalidTerm { term: term_id });
                }
                let type_site = term_type_site(input, term_id, SourceSetTypeRole::ChoiceTarget)
                    .ok_or(SourceSetTermError::InvalidTerm { term: term_id })?;
                (
                    format!("the {}", type_site.spelling),
                    term.source_range.start < type_site.source_range.start
                        && term.source_range.end == type_site.source_range.end,
                )
            }
            SourceSetTermKind::Qua => {
                if edges.len() != 1
                    || input.edges[edges[0]].role != SourceSetEdgeRole::QuaBase
                    || !generators.is_empty()
                {
                    return Err(SourceSetTermError::InvalidTerm { term: term_id });
                }
                let type_site = term_type_site(input, term_id, SourceSetTypeRole::QuaTarget)
                    .ok_or(SourceSetTermError::InvalidTerm { term: term_id })?;
                (
                    format!("{} qua {}", targets[0].spelling, type_site.spelling),
                    term.source_range.start == targets[0].range.start
                        && targets[0].range.end <= type_site.source_range.start
                        && term.source_range.end == type_site.source_range.end,
                )
            }
        };
        if !positions_valid || term.spelling != expected_spelling {
            return Err(SourceSetTermError::InvalidTerm { term: term_id });
        }
    }
    Ok(())
}

fn validate_requests(
    input: &SourceSetTermHandoffInput,
    generator_groups: &[Vec<usize>],
) -> Result<(), SourceSetTermError> {
    let groups = grouped_rows(
        input.terms.len(),
        &input.requests,
        |row| row.term,
        |row| row.ordinal,
        |index| SourceSetTermError::ReorderedRequest {
            request: SourceSetRequestId::new(index),
        },
    )?;
    for (term_index, requests) in groups.iter().enumerate() {
        let term_id = SourceSetTermId::new(term_index);
        let term = &input.terms[term_index];
        let generators = &generator_groups[term_index];
        let expected_len = match term.kind {
            SourceSetTermKind::Enumeration => 1,
            SourceSetTermKind::Comprehension => generators.len() + 1,
            SourceSetTermKind::Choice | SourceSetTermKind::Qua => 2,
        };
        if requests.len() != expected_len {
            return Err(SourceSetTermError::InvalidRequest {
                request: SourceSetRequestId::new(
                    requests.first().copied().unwrap_or(input.requests.len()),
                ),
            });
        }
        let mut ordinal = 0;
        if term.kind == SourceSetTermKind::Comprehension {
            for generator_index in generators {
                let generator_id = SourceSetGeneratorId::new(*generator_index);
                let generator = &input.generators[*generator_index];
                let request_index = requests[ordinal];
                let request = &input.requests[request_index];
                if request.kind != SourceSetRequestKind::GeneratorSethood
                    || request.generator != Some(generator_id)
                    || request.type_site != Some(generator.type_site)
                {
                    return Err(SourceSetTermError::InvalidRequest {
                        request: SourceSetRequestId::new(request_index),
                    });
                }
                ordinal += 1;
            }
        } else if matches!(
            term.kind,
            SourceSetTermKind::Choice | SourceSetTermKind::Qua
        ) {
            let (kind, role) = match term.kind {
                SourceSetTermKind::Choice => (
                    SourceSetRequestKind::ChoiceNonempty,
                    SourceSetTypeRole::ChoiceTarget,
                ),
                SourceSetTermKind::Qua => (
                    SourceSetRequestKind::QuaWidening,
                    SourceSetTypeRole::QuaTarget,
                ),
                SourceSetTermKind::Enumeration | SourceSetTermKind::Comprehension => unreachable!(),
            };
            let type_site = input
                .type_sites
                .iter()
                .enumerate()
                .find_map(|(index, site)| {
                    (site.owner
                        == SourceSetTypeOwner::Term {
                            term: term_id,
                            role,
                        })
                    .then_some(SourceSetTypeSiteId::new(index))
                })
                .ok_or(SourceSetTermError::InvalidTerm { term: term_id })?;
            let request_index = requests[ordinal];
            let request = &input.requests[request_index];
            if request.kind != kind
                || request.generator.is_some()
                || request.type_site != Some(type_site)
            {
                return Err(SourceSetTermError::InvalidRequest {
                    request: SourceSetRequestId::new(request_index),
                });
            }
            ordinal += 1;
        }
        let request_index = requests[ordinal];
        let request = &input.requests[request_index];
        if request.kind != SourceSetRequestKind::ResultType
            || request.generator.is_some()
            || request.type_site.is_some()
        {
            return Err(SourceSetTermError::InvalidRequest {
                request: SourceSetRequestId::new(request_index),
            });
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
) -> Result<Vec<Vec<usize>>, SourceSetTermError>
where
    FTerm: Fn(&T) -> SourceSetTermId,
    FOrdinal: Fn(&T) -> usize,
    FError: Fn(usize) -> SourceSetTermError,
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
    input: &SourceSetTermHandoffInput,
    wrapper_groups: &[Vec<usize>],
) -> Vec<EffectiveOccurrence> {
    input
        .terms
        .iter()
        .enumerate()
        .map(|(index, term)| {
            wrapper_groups[index].first().map_or_else(
                || EffectiveOccurrence {
                    range: term.source_range,
                    site: term.site.clone(),
                    spelling: term.spelling.clone(),
                },
                |wrapper| {
                    let wrapper = &input.wrappers[*wrapper];
                    EffectiveOccurrence {
                        range: wrapper.source_range,
                        site: wrapper.site.clone(),
                        spelling: wrapper.spelling.clone(),
                    }
                },
            )
        })
        .collect()
}

fn target_occurrence(
    input: &SourceSetTermHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    effective: &[EffectiveOccurrence],
    target: SourceSetTarget,
) -> Option<TargetOccurrence> {
    match target {
        SourceSetTarget::Primary(id) => {
            let primary = primary_terms.terms().get(id)?;
            Some(TargetOccurrence {
                target,
                range: primary.source_range(),
                site: primary.site().clone(),
                spelling: primary.spelling().to_owned(),
                context: primary.context(),
            })
        }
        SourceSetTarget::Application(id) => {
            let applications = applications?;
            let application = applications.applications().get(id)?;
            let occurrence = application_effective_occurrence(applications, id)?;
            Some(TargetOccurrence {
                target,
                range: occurrence.range,
                site: occurrence.site,
                spelling: occurrence.spelling,
                context: application.context(),
            })
        }
        SourceSetTarget::Structure(id) => {
            let structures = structures?;
            let structure = structures.terms().get(id)?;
            let occurrence = structure_effective_occurrence(structures, id)?;
            Some(TargetOccurrence {
                target,
                range: occurrence.range,
                site: occurrence.site,
                spelling: occurrence.spelling,
                context: structure.context(),
            })
        }
        SourceSetTarget::SetTerm(id) => {
            let term = input.terms.get(id.index())?;
            let occurrence = effective.get(id.index())?;
            Some(TargetOccurrence {
                target,
                range: occurrence.range,
                site: occurrence.site.clone(),
                spelling: occurrence.spelling.clone(),
                context: term.context,
            })
        }
    }
}

fn term_type_site(
    input: &SourceSetTermHandoffInput,
    term: SourceSetTermId,
    role: SourceSetTypeRole,
) -> Option<&SourceSetTypeSiteInput> {
    input
        .type_sites
        .iter()
        .find(|site| site.owner == SourceSetTypeOwner::Term { term, role })
}

fn application_root_ids(
    handoff: &SourceFunctorApplicationHandoff,
) -> BTreeSet<SourceFunctorApplicationId> {
    let nested = handoff
        .arguments()
        .iter()
        .filter_map(|(_, argument)| match argument.target() {
            SourceFunctorArgumentTarget::Application(id) => Some(id),
            SourceFunctorArgumentTarget::Primary(_) => None,
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
            SourceFunctorArgumentTarget::Primary(id) => Some(id),
            SourceFunctorArgumentTarget::Application(_) => None,
        })
        .collect()
}

fn structure_root_ids(handoff: &SourceStructureHandoff) -> BTreeSet<SourceStructureTermId> {
    let nested = handoff
        .edges()
        .iter()
        .filter_map(|(_, edge)| match edge.target() {
            SourceStructureTarget::Structure(id) => Some(id),
            SourceStructureTarget::Primary(_) | SourceStructureTarget::Application(_) => None,
        })
        .collect::<BTreeSet<_>>();
    handoff
        .terms()
        .iter()
        .map(|(id, _)| id)
        .filter(|id| !nested.contains(id))
        .collect()
}

fn structure_primary_ids(handoff: &SourceStructureHandoff) -> BTreeSet<SourcePrimaryTermId> {
    handoff
        .edges()
        .iter()
        .filter_map(|(_, edge)| match edge.target() {
            SourceStructureTarget::Primary(id) => Some(id),
            SourceStructureTarget::Application(_) | SourceStructureTarget::Structure(_) => None,
        })
        .collect()
}

fn structure_application_ids(
    handoff: &SourceStructureHandoff,
) -> BTreeSet<SourceFunctorApplicationId> {
    handoff
        .edges()
        .iter()
        .filter_map(|(_, edge)| match edge.target() {
            SourceStructureTarget::Application(id) => Some(id),
            SourceStructureTarget::Primary(_) | SourceStructureTarget::Structure(_) => None,
        })
        .collect()
}

fn application_effective_occurrence(
    handoff: &SourceFunctorApplicationHandoff,
    id: SourceFunctorApplicationId,
) -> Option<EffectiveOccurrence> {
    let application = handoff.applications().get(id)?;
    Some(
        handoff
            .wrappers()
            .iter()
            .find(|(_, wrapper)| wrapper.application() == id && wrapper.ordinal() == 0)
            .map_or_else(
                || EffectiveOccurrence {
                    range: application.source_range(),
                    site: application.site().clone(),
                    spelling: application.spelling().to_owned(),
                },
                |(_, wrapper)| EffectiveOccurrence {
                    range: wrapper.source_range(),
                    site: wrapper.site().clone(),
                    spelling: wrapper.spelling().to_owned(),
                },
            ),
    )
}

fn structure_effective_occurrence(
    handoff: &SourceStructureHandoff,
    id: SourceStructureTermId,
) -> Option<EffectiveOccurrence> {
    let term = handoff.terms().get(id)?;
    Some(
        handoff
            .wrappers()
            .iter()
            .find(|(_, wrapper)| wrapper.term() == id && wrapper.ordinal() == 0)
            .map_or_else(
                || EffectiveOccurrence {
                    range: term.source_range(),
                    site: term.site().clone(),
                    spelling: term.spelling().to_owned(),
                },
                |(_, wrapper)| EffectiveOccurrence {
                    range: wrapper.source_range(),
                    site: wrapper.site().clone(),
                    spelling: wrapper.spelling().to_owned(),
                },
            ),
    )
}

fn validate_arena_site(
    site: &TypedSiteRef,
    source_range: SourceRange,
    kind: &str,
    recovery: SourceSetTermRecovery,
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

fn ranges_overlap(left: SourceRange, right: SourceRange) -> bool {
    left.source_id == right.source_id && left.start < right.end && right.start < left.end
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

fn recovery_matches(recovery: SourceSetTermRecovery, node_recovery: NodeRecoveryState) -> bool {
    match recovery {
        SourceSetTermRecovery::Normal => node_recovery == NodeRecoveryState::Normal,
        SourceSetTermRecovery::Degraded => matches!(
            node_recovery,
            NodeRecoveryState::Recovered | NodeRecoveryState::Degraded
        ),
    }
}

fn term_kind_node_key(kind: SourceSetTermKind) -> &'static str {
    match kind {
        SourceSetTermKind::Enumeration => "source.term.set.enumeration",
        SourceSetTermKind::Comprehension => "source.term.set.comprehension",
        SourceSetTermKind::Choice => "source.term.set.choice",
        SourceSetTermKind::Qua => "source.term.set.qua",
    }
}

fn term_kind_key(kind: SourceSetTermKind) -> &'static str {
    match kind {
        SourceSetTermKind::Enumeration => "enumeration",
        SourceSetTermKind::Comprehension => "comprehension",
        SourceSetTermKind::Choice => "choice",
        SourceSetTermKind::Qua => "qua",
    }
}

fn recovery_key(recovery: SourceSetTermRecovery) -> &'static str {
    match recovery {
        SourceSetTermRecovery::Normal => "normal",
        SourceSetTermRecovery::Degraded => "degraded",
    }
}

fn type_head_spelling(head: SourceSetTypeHead) -> &'static str {
    match head {
        SourceSetTypeHead::BuiltinSet => "set",
        SourceSetTypeHead::BuiltinObject => "object",
    }
}

fn type_head_key(head: SourceSetTypeHead) -> &'static str {
    match head {
        SourceSetTypeHead::BuiltinSet => "builtin-set",
        SourceSetTypeHead::BuiltinObject => "builtin-object",
    }
}

fn edge_role_key(role: SourceSetEdgeRole) -> &'static str {
    match role {
        SourceSetEdgeRole::EnumerationElement => "enumeration-element",
        SourceSetEdgeRole::ComprehensionMapper => "comprehension-mapper",
        SourceSetEdgeRole::QuaBase => "qua-base",
    }
}

fn request_kind_key(kind: SourceSetRequestKind) -> &'static str {
    match kind {
        SourceSetRequestKind::ResultType => "result-type",
        SourceSetRequestKind::GeneratorSethood => "generator-sethood",
        SourceSetRequestKind::ChoiceNonempty => "choice-nonempty",
        SourceSetRequestKind::QuaWidening => "qua-widening",
    }
}

fn write_type_owner(output: &mut String, owner: SourceSetTypeOwner) {
    match owner {
        SourceSetTypeOwner::Generator(id) => {
            let _ = write!(output, "generator({})", id.index());
        }
        SourceSetTypeOwner::Term { term, role } => {
            let role = match role {
                SourceSetTypeRole::ChoiceTarget => "choice",
                SourceSetTypeRole::QuaTarget => "qua",
            };
            let _ = write!(output, "term({},{role})", term.index());
        }
    }
}

fn write_target(output: &mut String, target: SourceSetTarget) {
    match target {
        SourceSetTarget::Primary(id) => {
            let _ = write!(output, "primary({})", id.index());
        }
        SourceSetTarget::Application(id) => {
            let _ = write!(output, "application({})", id.index());
        }
        SourceSetTarget::Structure(id) => {
            let _ = write!(output, "structure({})", id.index());
        }
        SourceSetTarget::SetTerm(id) => {
            let _ = write!(output, "set-term({})", id.index());
        }
    }
}

fn write_optional_id(output: &mut String, id: Option<usize>) {
    if let Some(id) = id {
        let _ = write!(output, "{id}");
    } else {
        output.push('-');
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
            SourceFunctorApplicationForm, SourceFunctorApplicationHandoff,
            SourceFunctorApplicationHandoffInput, SourceFunctorApplicationInput,
            SourceFunctorApplicationKind, SourceFunctorApplicationProducer,
            SourceFunctorApplicationRecovery, SourceFunctorArgumentInput,
            SourceFunctorArgumentTarget, SourceFunctorHeadSite,
        },
        source_structure::{
            SourceStructureEdgeInput, SourceStructureEdgeRole, SourceStructureHandoffInput,
            SourceStructureMemberId, SourceStructureMemberInput, SourceStructureMemberRole,
            SourceStructureProducer, SourceStructureRecovery, SourceStructureRequestInput,
            SourceStructureRequestKind, SourceStructureTarget, SourceStructureTermInput,
            SourceStructureTermKind,
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
    use mizar_resolve::env::{SymbolEnv, SymbolEnvIndexes};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        input: SourceSetTermHandoffInput,
        bindings: BindingEnv,
        primary: SourcePrimaryTermHandoff,
        arena: TypedArena,
    }

    impl Fixture {
        fn build(&self) -> Result<SourceSetTermHandoff, SourceSetTermError> {
            SourceSetTermProducer::build(
                self.input.clone(),
                &self.bindings,
                &self.primary,
                None,
                None,
                &self.arena,
            )
        }
    }

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "e5".repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("source.set.term"))
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
            owner: BindingContextOwner::Generated("set-target-child".to_owned()),
            parent: Some(BindingContextId::new(0)),
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
        .expect("binding environment with two valid contexts")
    }

    fn arena_nodes(source: SourceId) -> Vec<TypedNode> {
        vec![
            TypedNode::new(
                "source.term.numeral",
                SourceAnchor::Range(range(source, 12, 13)),
            ),
            TypedNode::new(
                "source.term.numeral",
                SourceAnchor::Range(range(source, 20, 21)),
            ),
            TypedNode::new(
                "source.term.numeral",
                SourceAnchor::Range(range(source, 32, 33)),
            ),
            TypedNode::new(
                "source.term.numeral",
                SourceAnchor::Range(range(source, 90, 91)),
            ),
            TypedNode::new(
                "source.term.set.enumeration",
                SourceAnchor::Range(range(source, 10, 25)),
            ),
            TypedNode::new(
                "source.term.set.comprehension",
                SourceAnchor::Range(range(source, 30, 70)),
            ),
            TypedNode::new(
                "source.term.set.choice",
                SourceAnchor::Range(range(source, 80, 87)),
            ),
            TypedNode::new(
                "source.term.set.qua",
                SourceAnchor::Range(range(source, 90, 99)),
            ),
            TypedNode::new(
                "source.term.set.comprehension-generator",
                SourceAnchor::Range(range(source, 40, 52)),
            ),
            TypedNode::new(
                "source.term.set.target-type",
                SourceAnchor::Range(range(source, 56, 59)),
            ),
            TypedNode::new(
                "source.term.set.target-type-head",
                SourceAnchor::Range(range(source, 56, 59)),
            ),
            TypedNode::new(
                "source.term.set.target-type",
                SourceAnchor::Range(range(source, 84, 87)),
            ),
            TypedNode::new(
                "source.term.set.target-type-head",
                SourceAnchor::Range(range(source, 84, 87)),
            ),
            TypedNode::new(
                "source.term.set.target-type",
                SourceAnchor::Range(range(source, 96, 99)),
            ),
            TypedNode::new(
                "source.term.set.target-type-head",
                SourceAnchor::Range(range(source, 96, 99)),
            ),
            TypedNode::new(
                "source.term.set.parenthesized",
                SourceAnchor::Range(range(source, 8, 27)),
            ),
            TypedNode::new(
                "source.term.functor-application.inline",
                SourceAnchor::Range(range(source, 11, 22)),
            ),
            TypedNode::new(
                "source.term.functor-head.single",
                SourceAnchor::Range(range(source, 11, 12)),
            ),
            TypedNode::new(
                "source.term.functor-application.inline",
                SourceAnchor::Range(range(source, 110, 125)),
            ),
            TypedNode::new(
                "source.term.functor-head.single",
                SourceAnchor::Range(range(source, 110, 119)),
            ),
            TypedNode::new(
                "source.term.structure.selector",
                SourceAnchor::Range(range(source, 12, 18)),
            ),
            TypedNode::new(
                "source.term.structure.member.selector",
                SourceAnchor::Range(range(source, 14, 18)),
            ),
            TypedNode::new(
                "source.term.set.parenthesized",
                SourceAnchor::Range(range(source, 6, 29)),
            ),
            TypedNode::new(
                "source.term.set.enumeration",
                SourceAnchor::Range(range(source, 130, 133)),
            ),
        ]
    }

    fn primary_handoff(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        arena: &TypedArena,
    ) -> SourcePrimaryTermHandoff {
        let occurrences = [
            (0, 12, 13, "1"),
            (1, 20, 21, "2"),
            (2, 32, 33, "3"),
            (3, 90, 91, "4"),
        ];
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

    fn empty_primary_handoff(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        arena: &TypedArena,
    ) -> SourcePrimaryTermHandoff {
        SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: Vec::new(),
                references: Vec::new(),
                numeric_type_requests: Vec::new(),
            },
            bindings,
            arena,
        )
        .expect("empty primary handoff")
    }

    fn typed_ast_for(source: SourceId, module: &ModuleId, arena: &TypedArena) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("typed ast")
    }

    fn typed_ast(fixture: &Fixture) -> TypedAst {
        typed_ast_for(fixture.source, &fixture.module, &fixture.arena)
    }

    fn empty_application(fixture: &Fixture) -> SourceFunctorApplicationHandoff {
        SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                applications: Vec::new(),
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: Vec::new(),
                type_requests: Vec::new(),
            },
            &SymbolEnv::new(fixture.module.clone(), SymbolEnvIndexes::default()),
            &fixture.bindings,
            &fixture.primary,
            &fixture.arena,
        )
        .expect("empty application handoff")
    }

    fn overlapping_application(fixture: &Fixture) -> SourceFunctorApplicationHandoff {
        SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(16),
                    source_range: range(fixture.source, 11, 22),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "inline255 ( 1 , 2 )".to_owned(),
                    kind: SourceFunctorApplicationKind::Inline,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(17),
                        source_range: range(fixture.source, 11, 12),
                        spelling: "inline255".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: vec![
                    SourceFunctorArgumentInput {
                        application: SourceFunctorApplicationId::new(0),
                        ordinal: 0,
                        target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0)),
                    },
                    SourceFunctorArgumentInput {
                        application: SourceFunctorApplicationId::new(0),
                        ordinal: 1,
                        target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(1)),
                    },
                ],
                type_requests: Vec::new(),
            },
            &SymbolEnv::new(fixture.module.clone(), SymbolEnvIndexes::default()),
            &fixture.bindings,
            &fixture.primary,
            &fixture.arena,
        )
        .expect("overlapping application handoff")
    }

    fn empty_structure(fixture: &Fixture) -> SourceStructureHandoff {
        SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                terms: Vec::new(),
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: Vec::new(),
                field_updates: Vec::new(),
                edges: Vec::new(),
                requests: Vec::new(),
            },
            &SymbolEnv::new(fixture.module.clone(), SymbolEnvIndexes::default()),
            &fixture.bindings,
            &fixture.primary,
            None,
            &fixture.arena,
        )
        .expect("empty structure handoff")
    }

    fn overlapping_structure(fixture: &Fixture) -> SourceStructureHandoff {
        SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                terms: vec![SourceStructureTermInput {
                    site: node(20),
                    source_range: range(fixture.source, 12, 18),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "1 . carrier".to_owned(),
                    kind: SourceStructureTermKind::SelectorAccess,
                }],
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: vec![SourceStructureMemberInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    site: node(21),
                    source_range: range(fixture.source, 14, 18),
                    spelling: "carrier".to_owned(),
                    role: SourceStructureMemberRole::Selector,
                    parent: None,
                }],
                field_updates: Vec::new(),
                edges: vec![SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::SelectorBase,
                    member: None,
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                requests: vec![
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                ],
            },
            &SymbolEnv::new(fixture.module.clone(), SymbolEnvIndexes::default()),
            &fixture.bindings,
            &fixture.primary,
            None,
            &fixture.arena,
        )
        .expect("overlapping structure handoff")
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = module_id();
        let bindings = binding_env(source, &module);
        let arena = TypedArena::try_new(None, arena_nodes(source)).expect("arena");
        let primary = primary_handoff(source, &module, &bindings, &arena);
        let input = SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![
                SourceSetTermInput {
                    site: node(4),
                    source_range: range(source, 10, 25),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "{ 1 , 2 }".to_owned(),
                    kind: SourceSetTermKind::Enumeration,
                },
                SourceSetTermInput {
                    site: node(5),
                    source_range: range(source, 30, 70),
                    source_ordinal: 1,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "{ 3 where candidate255 is set }".to_owned(),
                    kind: SourceSetTermKind::Comprehension,
                },
                SourceSetTermInput {
                    site: node(6),
                    source_range: range(source, 80, 87),
                    source_ordinal: 2,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "the set".to_owned(),
                    kind: SourceSetTermKind::Choice,
                },
                SourceSetTermInput {
                    site: node(7),
                    source_range: range(source, 90, 99),
                    source_ordinal: 3,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "4 qua set".to_owned(),
                    kind: SourceSetTermKind::Qua,
                },
            ],
            wrappers: Vec::new(),
            generators: vec![SourceSetGeneratorInput {
                term: SourceSetTermId::new(1),
                ordinal: 0,
                site: node(8),
                source_range: range(source, 40, 52),
                spelling: "candidate255".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                type_site: SourceSetTypeSiteId::new(0),
            }],
            type_sites: vec![
                SourceSetTypeSiteInput {
                    owner: SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0)),
                    site: node(9),
                    source_range: range(source, 56, 59),
                    spelling: "set".to_owned(),
                    head_site: node(10),
                    head_range: range(source, 56, 59),
                    head_spelling: "set".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    head: SourceSetTypeHead::BuiltinSet,
                },
                SourceSetTypeSiteInput {
                    owner: SourceSetTypeOwner::Term {
                        term: SourceSetTermId::new(2),
                        role: SourceSetTypeRole::ChoiceTarget,
                    },
                    site: node(11),
                    source_range: range(source, 84, 87),
                    spelling: "set".to_owned(),
                    head_site: node(12),
                    head_range: range(source, 84, 87),
                    head_spelling: "set".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    head: SourceSetTypeHead::BuiltinSet,
                },
                SourceSetTypeSiteInput {
                    owner: SourceSetTypeOwner::Term {
                        term: SourceSetTermId::new(3),
                        role: SourceSetTypeRole::QuaTarget,
                    },
                    site: node(13),
                    source_range: range(source, 96, 99),
                    spelling: "set".to_owned(),
                    head_site: node(14),
                    head_range: range(source, 96, 99),
                    head_spelling: "set".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    head: SourceSetTypeHead::BuiltinSet,
                },
            ],
            edges: vec![
                SourceSetEdgeInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    role: SourceSetEdgeRole::EnumerationElement,
                    target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
                },
                SourceSetEdgeInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 1,
                    role: SourceSetEdgeRole::EnumerationElement,
                    target: SourceSetTarget::Primary(SourcePrimaryTermId::new(1)),
                },
                SourceSetEdgeInput {
                    term: SourceSetTermId::new(1),
                    ordinal: 0,
                    role: SourceSetEdgeRole::ComprehensionMapper,
                    target: SourceSetTarget::Primary(SourcePrimaryTermId::new(2)),
                },
                SourceSetEdgeInput {
                    term: SourceSetTermId::new(3),
                    ordinal: 0,
                    role: SourceSetEdgeRole::QuaBase,
                    target: SourceSetTarget::Primary(SourcePrimaryTermId::new(3)),
                },
            ],
            requests: vec![
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(1),
                    ordinal: 0,
                    kind: SourceSetRequestKind::GeneratorSethood,
                    generator: Some(SourceSetGeneratorId::new(0)),
                    type_site: Some(SourceSetTypeSiteId::new(0)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(1),
                    ordinal: 1,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(2),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ChoiceNonempty,
                    generator: None,
                    type_site: Some(SourceSetTypeSiteId::new(1)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(2),
                    ordinal: 1,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(3),
                    ordinal: 0,
                    kind: SourceSetRequestKind::QuaWidening,
                    generator: None,
                    type_site: Some(SourceSetTypeSiteId::new(2)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(3),
                    ordinal: 1,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
            ],
        };
        Fixture {
            source,
            module,
            input,
            bindings,
            primary,
            arena,
        }
    }

    #[test]
    fn exact_four_shape_transaction_is_dense_deterministic_and_installable() {
        let fixture = fixture();
        let first = fixture.build().expect("valid transaction");
        let second = fixture.build().expect("deterministic transaction");
        assert_eq!(first, second);
        assert_eq!(first.terms().len(), 4);
        assert_eq!(first.wrappers().len(), 0);
        assert_eq!(first.generators().len(), 1);
        assert_eq!(first.type_sites().len(), 3);
        assert_eq!(first.edges().len(), 4);
        assert_eq!(first.requests().len(), 7);
        assert_eq!(first.application_fingerprint(), None);
        assert_eq!(first.structure_fingerprint(), None);
        assert_eq!(
            first.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert!(
            first
                .debug_text()
                .contains("request#6 term=3 ordinal=1 kind=result-type")
        );
        first
            .validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                None,
                None,
                &fixture.arena,
            )
            .expect("installable");
    }

    #[test]
    fn typed_ast_requires_primary_and_installs_set_terms_once() {
        let fixture = fixture();
        let handoff = fixture.build().expect("handoff");
        assert_eq!(
            typed_ast(&fixture)
                .with_source_set_term(handoff.clone())
                .expect_err("primary dependency is mandatory"),
            TypedAstError::InvalidSourceSetTerm
        );

        let ast = typed_ast(&fixture)
            .with_source_term(fixture.primary.clone())
            .expect("primary")
            .with_source_set_term(handoff.clone())
            .expect("set-term");
        assert_eq!(
            ast.with_source_set_term(handoff)
                .expect_err("one-shot installation"),
            TypedAstError::InvalidSourceSetTerm
        );
    }

    #[test]
    fn typed_ast_revalidates_set_terms_for_later_application_installation() {
        let fixture = fixture();
        let handoff = fixture.build().expect("set handoff without applications");
        let unrelated = empty_application(&fixture);

        typed_ast(&fixture)
            .with_source_term(fixture.primary.clone())
            .expect("primary")
            .with_source_set_term(handoff.clone())
            .expect("set first")
            .with_source_application(unrelated.clone())
            .expect("later unrelated application");
        typed_ast(&fixture)
            .with_source_term(fixture.primary.clone())
            .expect("primary")
            .with_source_application(unrelated)
            .expect("unrelated application first")
            .with_source_set_term(handoff.clone())
            .expect("set after unrelated application");

        let overlapping = overlapping_application(&fixture);
        assert_eq!(
            typed_ast(&fixture)
                .with_source_term(fixture.primary.clone())
                .expect("primary")
                .with_source_set_term(handoff)
                .expect("set first")
                .with_source_application(overlapping.clone())
                .expect_err("later ownership-changing application must reject"),
            TypedAstError::InvalidSourceApplication
        );

        let direct = SourceSetTermProducer::build(
            single_enumeration_input(
                fixture.source,
                &fixture.module,
                node(4),
                range(fixture.source, 10, 25),
                "{ inline255 ( 1 , 2 ) }",
                SourceSetTarget::Application(SourceFunctorApplicationId::new(0)),
            ),
            &fixture.bindings,
            &fixture.primary,
            Some(&overlapping),
            None,
            &fixture.arena,
        )
        .expect("application-owned set handoff");
        assert_eq!(
            typed_ast(&fixture)
                .with_source_term(fixture.primary.clone())
                .expect("primary")
                .with_source_set_term(direct.clone())
                .expect_err("fingerprinted application must already be installed"),
            TypedAstError::InvalidSourceSetTerm
        );
        typed_ast(&fixture)
            .with_source_term(fixture.primary.clone())
            .expect("primary")
            .with_source_application(overlapping.clone())
            .expect("application first")
            .with_source_set_term(direct.clone())
            .expect("fingerprinted set after application");

        let mut mismatched = direct;
        mismatched.application_fingerprint = Some("stale-application-fingerprint".to_owned());
        assert_eq!(
            mismatched.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                Some(&overlapping),
                None,
                &fixture.arena,
            ),
            Err(SourceSetTermError::ApplicationDependencyMismatch)
        );
    }

    #[test]
    fn typed_ast_revalidates_set_terms_for_later_structure_installation() {
        let fixture = fixture();
        let handoff = fixture.build().expect("set handoff without structures");
        let unrelated = empty_structure(&fixture);

        typed_ast(&fixture)
            .with_source_term(fixture.primary.clone())
            .expect("primary")
            .with_source_set_term(handoff.clone())
            .expect("set first")
            .with_source_structure(unrelated.clone())
            .expect("later unrelated structure");
        typed_ast(&fixture)
            .with_source_term(fixture.primary.clone())
            .expect("primary")
            .with_source_structure(unrelated)
            .expect("unrelated structure first")
            .with_source_set_term(handoff.clone())
            .expect("set after unrelated structure");

        let overlapping = overlapping_structure(&fixture);
        assert_eq!(
            typed_ast(&fixture)
                .with_source_term(fixture.primary.clone())
                .expect("primary")
                .with_source_set_term(handoff.clone())
                .expect("set first")
                .with_source_structure(overlapping.clone())
                .expect_err("later ownership-changing structure must reject"),
            TypedAstError::InvalidSourceStructure
        );
        assert_eq!(
            typed_ast(&fixture)
                .with_source_term(fixture.primary.clone())
                .expect("primary")
                .with_source_structure(overlapping)
                .expect("structure first")
                .with_source_set_term(handoff)
                .expect_err("set must see the installed structure partition"),
            TypedAstError::InvalidSourceSetTerm
        );
    }

    #[test]
    fn zero_element_enumeration_and_multiple_generators_are_nonvacuous() {
        let baseline = fixture();
        let empty = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: baseline.source,
                module_id: baseline.module.clone(),
                terms: vec![SourceSetTermInput {
                    site: node(23),
                    source_range: range(baseline.source, 130, 133),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "{ }".to_owned(),
                    kind: SourceSetTermKind::Enumeration,
                }],
                wrappers: Vec::new(),
                generators: Vec::new(),
                type_sites: Vec::new(),
                edges: Vec::new(),
                requests: vec![SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                }],
            },
            &baseline.bindings,
            &baseline.primary,
            None,
            None,
            &baseline.arena,
        )
        .expect("zero-element enumeration");
        assert!(empty.edges().is_empty());
        assert_eq!(
            empty
                .terms()
                .get(SourceSetTermId::new(0))
                .unwrap()
                .spelling(),
            "{ }"
        );

        let source = source_id();
        let module = module_id();
        let bindings = binding_env(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 12, 13)),
                ),
                TypedNode::new(
                    "source.term.set.comprehension",
                    SourceAnchor::Range(range(source, 10, 60)),
                ),
                TypedNode::new(
                    "source.term.set.comprehension-generator",
                    SourceAnchor::Range(range(source, 20, 25)),
                ),
                TypedNode::new(
                    "source.term.set.target-type",
                    SourceAnchor::Range(range(source, 28, 31)),
                ),
                TypedNode::new(
                    "source.term.set.target-type-head",
                    SourceAnchor::Range(range(source, 28, 31)),
                ),
                TypedNode::new(
                    "source.term.set.comprehension-generator",
                    SourceAnchor::Range(range(source, 36, 40)),
                ),
                TypedNode::new(
                    "source.term.set.target-type",
                    SourceAnchor::Range(range(source, 44, 50)),
                ),
                TypedNode::new(
                    "source.term.set.target-type-head",
                    SourceAnchor::Range(range(source, 44, 50)),
                ),
            ],
        )
        .expect("many-generator arena");
        let primary = primary_handoff_from(source, &module, &bindings, &arena, &[(0, 12, 13, "1")]);
        let input = SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceSetTermInput {
                site: node(1),
                source_range: range(source, 10, 60),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "{ 1 where alpha is set , beta is object }".to_owned(),
                kind: SourceSetTermKind::Comprehension,
            }],
            wrappers: Vec::new(),
            generators: vec![
                SourceSetGeneratorInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    site: node(2),
                    source_range: range(source, 20, 25),
                    spelling: "alpha".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    type_site: SourceSetTypeSiteId::new(0),
                },
                SourceSetGeneratorInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 1,
                    site: node(5),
                    source_range: range(source, 36, 40),
                    spelling: "beta".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    type_site: SourceSetTypeSiteId::new(1),
                },
            ],
            type_sites: vec![
                SourceSetTypeSiteInput {
                    owner: SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0)),
                    site: node(3),
                    source_range: range(source, 28, 31),
                    spelling: "set".to_owned(),
                    head_site: node(4),
                    head_range: range(source, 28, 31),
                    head_spelling: "set".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    head: SourceSetTypeHead::BuiltinSet,
                },
                SourceSetTypeSiteInput {
                    owner: SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(1)),
                    site: node(6),
                    source_range: range(source, 44, 50),
                    spelling: "object".to_owned(),
                    head_site: node(7),
                    head_range: range(source, 44, 50),
                    head_spelling: "object".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    head: SourceSetTypeHead::BuiltinObject,
                },
            ],
            edges: vec![SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::ComprehensionMapper,
                target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
            }],
            requests: vec![
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::GeneratorSethood,
                    generator: Some(SourceSetGeneratorId::new(0)),
                    type_site: Some(SourceSetTypeSiteId::new(0)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 1,
                    kind: SourceSetRequestKind::GeneratorSethood,
                    generator: Some(SourceSetGeneratorId::new(1)),
                    type_site: Some(SourceSetTypeSiteId::new(1)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 2,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
            ],
        };
        let many =
            SourceSetTermProducer::build(input.clone(), &bindings, &primary, None, None, &arena)
                .expect("multiple independent generators");
        assert_eq!(many.generators().len(), 2);
        assert_eq!(many.requests().len(), 3);

        let mut zero = input;
        zero.generators.clear();
        zero.type_sites.clear();
        zero.requests = vec![SourceSetRequestInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            kind: SourceSetRequestKind::ResultType,
            generator: None,
            type_site: None,
        }];
        assert_eq!(
            SourceSetTermProducer::build(zero, &bindings, &primary, None, None, &arena),
            Err(SourceSetTermError::InvalidTerm {
                term: SourceSetTermId::new(0)
            })
        );
    }

    #[test]
    fn nested_independent_comprehension_and_left_associated_qua_are_preserved() {
        let source = source_id();
        let module = module_id();
        let bindings = binding_env(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 25, 26)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 10, 60)),
                ),
                TypedNode::new(
                    "source.term.set.comprehension",
                    SourceAnchor::Range(range(source, 20, 50)),
                ),
                TypedNode::new(
                    "source.term.set.comprehension-generator",
                    SourceAnchor::Range(range(source, 32, 33)),
                ),
                TypedNode::new(
                    "source.term.set.target-type",
                    SourceAnchor::Range(range(source, 36, 39)),
                ),
                TypedNode::new(
                    "source.term.set.target-type-head",
                    SourceAnchor::Range(range(source, 36, 39)),
                ),
            ],
        )
        .expect("nested comprehension arena");
        let primary = primary_handoff_from(source, &module, &bindings, &arena, &[(0, 25, 26, "1")]);
        let nested = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![
                    SourceSetTermInput {
                        site: node(1),
                        source_range: range(source, 10, 60),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        spelling: "{ { 1 where x is set } }".to_owned(),
                        kind: SourceSetTermKind::Enumeration,
                    },
                    SourceSetTermInput {
                        site: node(2),
                        source_range: range(source, 20, 50),
                        source_ordinal: 1,
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        spelling: "{ 1 where x is set }".to_owned(),
                        kind: SourceSetTermKind::Comprehension,
                    },
                ],
                wrappers: Vec::new(),
                generators: vec![SourceSetGeneratorInput {
                    term: SourceSetTermId::new(1),
                    ordinal: 0,
                    site: node(3),
                    source_range: range(source, 32, 33),
                    spelling: "x".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    type_site: SourceSetTypeSiteId::new(0),
                }],
                type_sites: vec![SourceSetTypeSiteInput {
                    owner: SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0)),
                    site: node(4),
                    source_range: range(source, 36, 39),
                    spelling: "set".to_owned(),
                    head_site: node(5),
                    head_range: range(source, 36, 39),
                    head_spelling: "set".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    head: SourceSetTypeHead::BuiltinSet,
                }],
                edges: vec![
                    SourceSetEdgeInput {
                        term: SourceSetTermId::new(0),
                        ordinal: 0,
                        role: SourceSetEdgeRole::EnumerationElement,
                        target: SourceSetTarget::SetTerm(SourceSetTermId::new(1)),
                    },
                    SourceSetEdgeInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 0,
                        role: SourceSetEdgeRole::ComprehensionMapper,
                        target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
                    },
                ],
                requests: vec![
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(0),
                        ordinal: 0,
                        kind: SourceSetRequestKind::ResultType,
                        generator: None,
                        type_site: None,
                    },
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 0,
                        kind: SourceSetRequestKind::GeneratorSethood,
                        generator: Some(SourceSetGeneratorId::new(0)),
                        type_site: Some(SourceSetTypeSiteId::new(0)),
                    },
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 1,
                        kind: SourceSetRequestKind::ResultType,
                        generator: None,
                        type_site: None,
                    },
                ],
            },
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("nested independent comprehension");
        assert!(matches!(
            nested.edges().get(SourceSetEdgeId::new(0)).unwrap().target(),
            SourceSetTarget::SetTerm(id) if id == SourceSetTermId::new(1)
        ));

        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 10, 11)),
                ),
                TypedNode::new(
                    "source.term.set.qua",
                    SourceAnchor::Range(range(source, 10, 42)),
                ),
                TypedNode::new(
                    "source.term.set.qua",
                    SourceAnchor::Range(range(source, 10, 23)),
                ),
                TypedNode::new(
                    "source.term.set.target-type",
                    SourceAnchor::Range(range(source, 20, 23)),
                ),
                TypedNode::new(
                    "source.term.set.target-type-head",
                    SourceAnchor::Range(range(source, 20, 23)),
                ),
                TypedNode::new(
                    "source.term.set.target-type",
                    SourceAnchor::Range(range(source, 36, 42)),
                ),
                TypedNode::new(
                    "source.term.set.target-type-head",
                    SourceAnchor::Range(range(source, 36, 42)),
                ),
            ],
        )
        .expect("qua arena");
        let primary = primary_handoff_from(source, &module, &bindings, &arena, &[(0, 10, 11, "1")]);
        let qua = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![
                    SourceSetTermInput {
                        site: node(1),
                        source_range: range(source, 10, 42),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        spelling: "1 qua set qua object".to_owned(),
                        kind: SourceSetTermKind::Qua,
                    },
                    SourceSetTermInput {
                        site: node(2),
                        source_range: range(source, 10, 23),
                        source_ordinal: 1,
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        spelling: "1 qua set".to_owned(),
                        kind: SourceSetTermKind::Qua,
                    },
                ],
                wrappers: Vec::new(),
                generators: Vec::new(),
                type_sites: vec![
                    SourceSetTypeSiteInput {
                        owner: SourceSetTypeOwner::Term {
                            term: SourceSetTermId::new(1),
                            role: SourceSetTypeRole::QuaTarget,
                        },
                        site: node(3),
                        source_range: range(source, 20, 23),
                        spelling: "set".to_owned(),
                        head_site: node(4),
                        head_range: range(source, 20, 23),
                        head_spelling: "set".to_owned(),
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        head: SourceSetTypeHead::BuiltinSet,
                    },
                    SourceSetTypeSiteInput {
                        owner: SourceSetTypeOwner::Term {
                            term: SourceSetTermId::new(0),
                            role: SourceSetTypeRole::QuaTarget,
                        },
                        site: node(5),
                        source_range: range(source, 36, 42),
                        spelling: "object".to_owned(),
                        head_site: node(6),
                        head_range: range(source, 36, 42),
                        head_spelling: "object".to_owned(),
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        head: SourceSetTypeHead::BuiltinObject,
                    },
                ],
                edges: vec![
                    SourceSetEdgeInput {
                        term: SourceSetTermId::new(0),
                        ordinal: 0,
                        role: SourceSetEdgeRole::QuaBase,
                        target: SourceSetTarget::SetTerm(SourceSetTermId::new(1)),
                    },
                    SourceSetEdgeInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 0,
                        role: SourceSetEdgeRole::QuaBase,
                        target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
                    },
                ],
                requests: vec![
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(0),
                        ordinal: 0,
                        kind: SourceSetRequestKind::QuaWidening,
                        generator: None,
                        type_site: Some(SourceSetTypeSiteId::new(1)),
                    },
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(0),
                        ordinal: 1,
                        kind: SourceSetRequestKind::ResultType,
                        generator: None,
                        type_site: None,
                    },
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 0,
                        kind: SourceSetRequestKind::QuaWidening,
                        generator: None,
                        type_site: Some(SourceSetTypeSiteId::new(0)),
                    },
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 1,
                        kind: SourceSetRequestKind::ResultType,
                        generator: None,
                        type_site: None,
                    },
                ],
            },
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("left-associated qua");
        assert_eq!(
            qua.terms().get(SourceSetTermId::new(0)).unwrap().spelling(),
            "1 qua set qua object"
        );
        assert!(matches!(
            qua.edges().get(SourceSetEdgeId::new(0)).unwrap().target(),
            SourceSetTarget::SetTerm(id) if id == SourceSetTermId::new(1)
        ));
    }

    #[test]
    fn transparent_wrapper_and_degraded_transport_are_authenticated() {
        let mut fixture = fixture();
        fixture.input.wrappers.push(SourceSetWrapperInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: node(22),
            source_range: range(fixture.source, 6, 29),
            context: BindingContextId::new(0),
            recovery: SourceSetTermRecovery::Normal,
            spelling: "( ( { 1 , 2 } ) )".to_owned(),
        });
        fixture.input.wrappers.push(SourceSetWrapperInput {
            term: SourceSetTermId::new(0),
            ordinal: 1,
            site: node(15),
            source_range: range(fixture.source, 8, 27),
            context: BindingContextId::new(0),
            recovery: SourceSetTermRecovery::Normal,
            spelling: "( { 1 , 2 } )".to_owned(),
        });
        let wrapped = fixture.build().expect("wrapper");
        assert_eq!(wrapped.wrappers().len(), 2);
        assert_eq!(
            wrapped
                .wrappers()
                .get(SourceSetWrapperId::new(0))
                .unwrap()
                .term(),
            SourceSetTermId::new(0)
        );
        assert_eq!(
            wrapped
                .wrappers()
                .get(SourceSetWrapperId::new(1))
                .unwrap()
                .ordinal(),
            1
        );

        let mut nodes = arena_nodes(fixture.source);
        for index in [4, 5, 8, 9, 10, 15, 22] {
            nodes[index] = nodes[index]
                .clone()
                .with_recovery(NodeRecoveryState::Recovered);
        }
        fixture.arena = TypedArena::try_new(None, nodes).expect("degraded arena");
        fixture.primary = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
        );
        fixture.input.terms[0].recovery = SourceSetTermRecovery::Degraded;
        fixture.input.terms[1].recovery = SourceSetTermRecovery::Degraded;
        fixture.input.wrappers[0].recovery = SourceSetTermRecovery::Degraded;
        fixture.input.wrappers[1].recovery = SourceSetTermRecovery::Degraded;
        fixture.input.generators[0].recovery = SourceSetTermRecovery::Degraded;
        fixture.input.type_sites[0].recovery = SourceSetTermRecovery::Degraded;
        fixture
            .build()
            .expect("degraded term/wrapper/generator/type rows");
    }

    #[test]
    fn row_shape_and_request_corruption_reject_atomically() {
        let baseline = fixture();

        let mut corrupt = baseline.clone();
        corrupt.input.terms[0].spelling = "{ 2 , 1 }".to_owned();
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTerm {
                term
            }) if term == SourceSetTermId::new(0)
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.generators[0].type_site = SourceSetTypeSiteId::new(1);
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTypeSite { .. })
                | Err(SourceSetTermError::InvalidGenerator { .. })
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.type_sites[0].head = SourceSetTypeHead::BuiltinObject;
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTypeSite { .. })
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.edges[2].role = SourceSetEdgeRole::QuaBase;
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTerm {
                term
            }) if term == SourceSetTermId::new(1)
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.requests[1].type_site = Some(SourceSetTypeSiteId::new(2));
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidRequest { .. })
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.edges[0].target =
            SourceSetTarget::Application(SourceFunctorApplicationId::new(0));
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::ApplicationDependencyMismatch)
        );
    }

    #[test]
    fn range_spelling_recovery_context_owner_and_cardinality_corruptions_reject() {
        let baseline = fixture();

        let mut corrupt = baseline.clone();
        corrupt.input.terms[0].source_range.end = corrupt.input.terms[0].source_range.start;
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTerm { term })
                if term == SourceSetTermId::new(0)
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.terms[0].spelling = " { 1 , 2 }".to_owned();
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTerm { term })
                if term == SourceSetTermId::new(0)
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.terms[0].recovery = SourceSetTermRecovery::Degraded;
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTerm { term })
                if term == SourceSetTermId::new(0)
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.terms[0].context = BindingContextId::new(1);
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTerm { term })
                if term == SourceSetTermId::new(0)
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.type_sites[1].owner = SourceSetTypeOwner::Term {
            term: SourceSetTermId::new(3),
            role: SourceSetTypeRole::QuaTarget,
        };
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTypeSite { type_site })
                if type_site == SourceSetTypeSiteId::new(1)
        ));

        let mut corrupt = baseline.clone();
        corrupt.input.edges.pop();
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidTerm { term })
                if term == SourceSetTermId::new(3)
        ));

        let mut corrupt = baseline;
        corrupt.input.requests.pop();
        assert!(matches!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidRequest { .. })
        ));
    }

    #[test]
    fn table_order_corruptions_report_the_owned_dense_row() {
        let baseline = fixture();

        let mut corrupt = baseline.clone();
        corrupt.input.terms.swap(0, 1);
        corrupt.input.terms[0].source_ordinal = 0;
        corrupt.input.terms[1].source_ordinal = 1;
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::ReorderedTerm {
                term: SourceSetTermId::new(1),
            })
        );

        let mut corrupt = baseline.clone();
        corrupt.input.wrappers.push(SourceSetWrapperInput {
            term: SourceSetTermId::new(0),
            ordinal: 1,
            site: node(15),
            source_range: range(corrupt.source, 8, 27),
            context: BindingContextId::new(0),
            recovery: SourceSetTermRecovery::Normal,
            spelling: "( { 1 , 2 } )".to_owned(),
        });
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::ReorderedWrapper {
                wrapper: SourceSetWrapperId::new(0),
            })
        );

        let mut corrupt = baseline.clone();
        corrupt.input.generators[0].ordinal = 1;
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::ReorderedGenerator {
                generator: SourceSetGeneratorId::new(0),
            })
        );

        let mut corrupt = baseline.clone();
        corrupt.input.type_sites.swap(0, 1);
        corrupt.input.generators[0].type_site = SourceSetTypeSiteId::new(1);
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::ReorderedTypeSite {
                type_site: SourceSetTypeSiteId::new(1),
            })
        );

        let mut corrupt = baseline.clone();
        corrupt.input.edges[0].ordinal = 1;
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::ReorderedEdge {
                edge: SourceSetEdgeId::new(0),
            })
        );

        let mut corrupt = baseline;
        corrupt.input.requests[0].ordinal = 1;
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::ReorderedRequest {
                request: SourceSetRequestId::new(0),
            })
        );
    }

    #[test]
    fn dense_extra_rows_reach_per_kind_cardinality_without_sparse_ids() {
        let mut baseline = fixture();
        let mut nodes = arena_nodes(baseline.source);
        nodes.extend([
            TypedNode::new(
                "source.term.set.parenthesized",
                SourceAnchor::Range(range(baseline.source, 4, 30)),
            ),
            TypedNode::new(
                "source.term.set.comprehension-generator",
                SourceAnchor::Range(range(baseline.source, 14, 15)),
            ),
            TypedNode::new(
                "source.term.set.target-type",
                SourceAnchor::Range(range(baseline.source, 16, 18)),
            ),
            TypedNode::new(
                "source.term.set.target-type-head",
                SourceAnchor::Range(range(baseline.source, 16, 18)),
            ),
            TypedNode::new(
                "source.term.set.target-type",
                SourceAnchor::Range(range(baseline.source, 84, 87)),
            ),
            TypedNode::new(
                "source.term.set.target-type-head",
                SourceAnchor::Range(range(baseline.source, 84, 87)),
            ),
        ]);
        baseline.arena = TypedArena::try_new(None, nodes).expect("extra-row arena");
        baseline.primary = primary_handoff(
            baseline.source,
            &baseline.module,
            &baseline.bindings,
            &baseline.arena,
        );

        let mut wrappers = baseline.clone();
        wrappers.input.wrappers = vec![
            SourceSetWrapperInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(24),
                source_range: range(wrappers.source, 4, 30),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "( ( ( { 1 , 2 } ) ) )".to_owned(),
            },
            SourceSetWrapperInput {
                term: SourceSetTermId::new(0),
                ordinal: 1,
                site: node(22),
                source_range: range(wrappers.source, 6, 29),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "( ( { 1 , 2 } ) )".to_owned(),
            },
            SourceSetWrapperInput {
                term: SourceSetTermId::new(0),
                ordinal: 2,
                site: node(15),
                source_range: range(wrappers.source, 8, 27),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "( { 1 , 2 } )".to_owned(),
            },
        ];
        let wrapped = wrappers.build().expect("three dense transparent wrappers");
        assert_eq!(wrapped.wrappers().len(), 3);
        assert_eq!(
            wrapped
                .wrappers()
                .get(SourceSetWrapperId::new(2))
                .expect("dense wrapper 2")
                .ordinal(),
            2
        );

        let mut generator = baseline.clone();
        generator.input.generators[0].type_site = SourceSetTypeSiteId::new(1);
        generator.input.type_sites[0].owner =
            SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(1));
        generator.input.generators.insert(
            0,
            SourceSetGeneratorInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(25),
                source_range: range(generator.source, 14, 15),
                spelling: "extra255".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                type_site: SourceSetTypeSiteId::new(0),
            },
        );
        generator.input.type_sites.insert(
            0,
            SourceSetTypeSiteInput {
                owner: SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0)),
                site: node(26),
                source_range: range(generator.source, 16, 18),
                spelling: "set".to_owned(),
                head_site: node(27),
                head_range: range(generator.source, 16, 18),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                head: SourceSetTypeHead::BuiltinSet,
            },
        );
        generator.input.requests[1].generator = Some(SourceSetGeneratorId::new(1));
        generator.input.requests[1].type_site = Some(SourceSetTypeSiteId::new(1));
        generator.input.requests[3].type_site = Some(SourceSetTypeSiteId::new(2));
        generator.input.requests[5].type_site = Some(SourceSetTypeSiteId::new(3));
        assert_eq!(generator.input.generators.len(), 2);
        assert_eq!(generator.input.type_sites.len(), 4);
        assert_eq!(
            generator.build(),
            Err(SourceSetTermError::InvalidTerm {
                term: SourceSetTermId::new(0),
            })
        );

        let mut type_site = baseline.clone();
        type_site.input.type_sites.insert(
            2,
            SourceSetTypeSiteInput {
                owner: SourceSetTypeOwner::Term {
                    term: SourceSetTermId::new(2),
                    role: SourceSetTypeRole::ChoiceTarget,
                },
                site: node(28),
                source_range: range(type_site.source, 84, 87),
                spelling: "set".to_owned(),
                head_site: node(29),
                head_range: range(type_site.source, 84, 87),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                head: SourceSetTypeHead::BuiltinSet,
            },
        );
        type_site.input.requests[5].type_site = Some(SourceSetTypeSiteId::new(3));
        assert_eq!(type_site.input.type_sites.len(), 4);
        assert_eq!(
            type_site.build(),
            Err(SourceSetTermError::InvalidTypeSite {
                type_site: SourceSetTypeSiteId::new(2),
            })
        );

        let mut edge = baseline.clone();
        edge.input.edges.insert(
            2,
            SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 2,
                role: SourceSetEdgeRole::EnumerationElement,
                target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
            },
        );
        assert_eq!(edge.input.edges.len(), 5);
        assert_eq!(
            edge.build(),
            Err(SourceSetTermError::InvalidTerm {
                term: SourceSetTermId::new(0),
            })
        );

        let mut request = baseline;
        request.input.requests.insert(
            1,
            SourceSetRequestInput {
                term: SourceSetTermId::new(0),
                ordinal: 1,
                kind: SourceSetRequestKind::ResultType,
                generator: None,
                type_site: None,
            },
        );
        assert_eq!(request.input.requests.len(), 8);
        assert_eq!(
            request.build(),
            Err(SourceSetTermError::InvalidRequest {
                request: SourceSetRequestId::new(0),
            })
        );
    }

    #[test]
    fn wrapper_generator_and_type_site_rows_reject_local_transport_drift() {
        let baseline = fixture();

        let wrapped = |fixture: &Fixture| {
            let mut fixture = fixture.clone();
            fixture.input.wrappers.push(SourceSetWrapperInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(15),
                source_range: range(fixture.source, 8, 27),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "( { 1 , 2 } )".to_owned(),
            });
            fixture
        };

        let mut corrupt = wrapped(&baseline);
        corrupt.input.wrappers[0].source_range = range(corrupt.source, 9, 26);
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidWrapper {
                wrapper: SourceSetWrapperId::new(0),
            })
        );

        let mut corrupt = wrapped(&baseline);
        corrupt.input.wrappers[0].spelling = "( { 2 , 1 } )".to_owned();
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidWrapper {
                wrapper: SourceSetWrapperId::new(0),
            })
        );

        let mut corrupt = wrapped(&baseline);
        corrupt.input.wrappers[0].recovery = SourceSetTermRecovery::Degraded;
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidWrapper {
                wrapper: SourceSetWrapperId::new(0),
            })
        );

        let mut corrupt = wrapped(&baseline);
        corrupt.input.wrappers[0].context = BindingContextId::new(1);
        assert_eq!(
            corrupt.build(),
            Err(SourceSetTermError::InvalidWrapper {
                wrapper: SourceSetWrapperId::new(0),
            })
        );

        for mutate in [
            |row: &mut SourceSetGeneratorInput, source| {
                row.source_range = range(source, 29, 52);
            },
            |row: &mut SourceSetGeneratorInput, _| {
                row.spelling = "candidate-255".to_owned();
            },
            |row: &mut SourceSetGeneratorInput, _| {
                row.recovery = SourceSetTermRecovery::Degraded;
            },
            |row: &mut SourceSetGeneratorInput, _| {
                row.context = BindingContextId::new(1);
            },
        ] {
            let mut corrupt = baseline.clone();
            mutate(&mut corrupt.input.generators[0], corrupt.source);
            assert_eq!(
                corrupt.build(),
                Err(SourceSetTermError::InvalidGenerator {
                    generator: SourceSetGeneratorId::new(0),
                })
            );
        }

        for mutate in [
            |row: &mut SourceSetTypeSiteInput, source| {
                row.source_range = range(source, 57, 59);
            },
            |row: &mut SourceSetTypeSiteInput, _| {
                row.spelling = "object".to_owned();
            },
            |row: &mut SourceSetTypeSiteInput, _| {
                row.recovery = SourceSetTermRecovery::Degraded;
            },
            |row: &mut SourceSetTypeSiteInput, _| {
                row.context = BindingContextId::new(1);
            },
        ] {
            let mut corrupt = baseline.clone();
            mutate(&mut corrupt.input.type_sites[0], corrupt.source);
            assert_eq!(
                corrupt.build(),
                Err(SourceSetTermError::InvalidTypeSite {
                    type_site: SourceSetTypeSiteId::new(0),
                })
            );
        }
    }

    #[test]
    fn duplicate_overlap_cycle_multi_parent_and_cross_family_root_corruptions_reject() {
        let baseline = fixture();

        let mut duplicate_site = baseline.clone();
        duplicate_site.input.terms[1] = duplicate_site.input.terms[0].clone();
        duplicate_site.input.terms[1].source_ordinal = 1;
        assert_eq!(
            duplicate_site.build(),
            Err(SourceSetTermError::DuplicateSite)
        );

        let mut duplicate_target = baseline.clone();
        duplicate_target.input.edges[1].target =
            SourceSetTarget::Primary(SourcePrimaryTermId::new(0));
        assert!(matches!(
            duplicate_target.build(),
            Err(SourceSetTermError::InvalidEdge { edge })
                if edge == SourceSetEdgeId::new(1)
        ));

        let mut overlapping = baseline.clone();
        overlapping.input.terms[1].source_range = range(overlapping.source, 15, 40);
        let mut nodes = arena_nodes(overlapping.source);
        nodes[5].anchor = SourceAnchor::Range(range(overlapping.source, 15, 40));
        overlapping.arena = TypedArena::try_new(None, nodes).expect("overlap arena");
        assert_eq!(
            overlapping.build(),
            Err(SourceSetTermError::ReorderedTerm {
                term: SourceSetTermId::new(1),
            })
        );

        let mut cycle = baseline.clone();
        cycle.input.edges[0].target = SourceSetTarget::SetTerm(SourceSetTermId::new(0));
        assert!(matches!(
            cycle.build(),
            Err(SourceSetTermError::InvalidEdge { edge })
                if edge == SourceSetEdgeId::new(0)
        ));

        let mut multiple_parents = baseline.clone();
        multiple_parents.input.edges[0].target = SourceSetTarget::SetTerm(SourceSetTermId::new(1));
        multiple_parents.input.edges[1].target = SourceSetTarget::SetTerm(SourceSetTermId::new(1));
        assert_eq!(
            multiple_parents.build(),
            Err(SourceSetTermError::InvalidEdge {
                edge: SourceSetEdgeId::new(0),
            })
        );

        let mut missing_application = baseline.clone();
        missing_application.input.edges[0].target =
            SourceSetTarget::Application(SourceFunctorApplicationId::new(0));
        assert_eq!(
            missing_application.build(),
            Err(SourceSetTermError::ApplicationDependencyMismatch)
        );

        let mut missing_structure = baseline;
        missing_structure.input.edges[0].target =
            SourceSetTarget::Structure(SourceStructureTermId::new(0));
        assert_eq!(
            missing_structure.build(),
            Err(SourceSetTermError::StructureDependencyMismatch)
        );
    }

    #[test]
    fn real_cross_context_targets_reach_each_owned_family_check() {
        let source = source_id();
        let module = module_id();
        let bindings = binding_env_with_two_contexts(source, &module);
        assert_eq!(bindings.contexts().len(), 2);

        let primary_arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 20, 21)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 10, 30)),
                ),
            ],
        )
        .expect("primary cross-context arena");
        let primary = single_primary_handoff_in_context(
            source,
            &module,
            &bindings,
            &primary_arena,
            (0, 20, 21),
            BindingContextId::new(1),
        );
        assert_eq!(primary.terms().len(), 1);
        assert_eq!(
            SourceSetTermProducer::build(
                single_enumeration_input(
                    source,
                    &module,
                    node(1),
                    range(source, 10, 30),
                    "{ 1 }",
                    SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
                ),
                &bindings,
                &primary,
                None,
                None,
                &primary_arena,
            ),
            Err(SourceSetTermError::PrimaryDependencyMismatch)
        );

        let nested_arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 10, 50)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 20, 30)),
                ),
            ],
        )
        .expect("nested cross-context arena");
        let empty_primary = empty_primary_handoff(source, &module, &bindings, &nested_arena);
        let nested_input = SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![
                SourceSetTermInput {
                    site: node(0),
                    source_range: range(source, 10, 50),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "{ { } }".to_owned(),
                    kind: SourceSetTermKind::Enumeration,
                },
                SourceSetTermInput {
                    site: node(1),
                    source_range: range(source, 20, 30),
                    source_ordinal: 1,
                    context: BindingContextId::new(1),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "{ }".to_owned(),
                    kind: SourceSetTermKind::Enumeration,
                },
            ],
            wrappers: Vec::new(),
            generators: Vec::new(),
            type_sites: Vec::new(),
            edges: vec![SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::EnumerationElement,
                target: SourceSetTarget::SetTerm(SourceSetTermId::new(1)),
            }],
            requests: vec![
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(1),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
            ],
        };
        assert_eq!(
            SourceSetTermProducer::build(
                nested_input,
                &bindings,
                &empty_primary,
                None,
                None,
                &nested_arena,
            ),
            Err(SourceSetTermError::InvalidTerm {
                term: SourceSetTermId::new(1),
            })
        );

        let application_arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 18, 19)),
                ),
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 10, 22)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 10, 11)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 8, 25)),
                ),
            ],
        )
        .expect("application cross-context arena");
        let application_primary = single_primary_handoff_in_context(
            source,
            &module,
            &bindings,
            &application_arena,
            (0, 18, 19),
            BindingContextId::new(1),
        );
        let applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(1),
                    source_range: range(source, 10, 22),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "f ( 1 )".to_owned(),
                    kind: SourceFunctorApplicationKind::Inline,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(2),
                        source_range: range(source, 10, 11),
                        spelling: "f".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: vec![SourceFunctorArgumentInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal: 0,
                    target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                type_requests: Vec::new(),
            },
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &bindings,
            &application_primary,
            &application_arena,
        )
        .expect("valid context-one application");
        assert_eq!(applications.applications().len(), 1);
        assert_eq!(
            SourceSetTermProducer::build(
                single_enumeration_input(
                    source,
                    &module,
                    node(3),
                    range(source, 8, 25),
                    "{ f ( 1 ) }",
                    SourceSetTarget::Application(SourceFunctorApplicationId::new(0)),
                ),
                &bindings,
                &application_primary,
                Some(&applications),
                None,
                &application_arena,
            ),
            Err(SourceSetTermError::ApplicationDependencyMismatch)
        );

        let structure_arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 11, 12)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(source, 10, 20)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(source, 15, 19)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 8, 23)),
                ),
            ],
        )
        .expect("structure cross-context arena");
        let structure_primary = single_primary_handoff_in_context(
            source,
            &module,
            &bindings,
            &structure_arena,
            (0, 11, 12),
            BindingContextId::new(1),
        );
        let structures = SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourceStructureTermInput {
                    site: node(1),
                    source_range: range(source, 10, 20),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "1 . carrier".to_owned(),
                    kind: SourceStructureTermKind::SelectorAccess,
                }],
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: vec![SourceStructureMemberInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    site: node(2),
                    source_range: range(source, 15, 19),
                    spelling: "carrier".to_owned(),
                    role: SourceStructureMemberRole::Selector,
                    parent: None,
                }],
                field_updates: Vec::new(),
                edges: vec![SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::SelectorBase,
                    member: None,
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                requests: vec![
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                ],
            },
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &bindings,
            &structure_primary,
            None,
            &structure_arena,
        )
        .expect("valid context-one structure");
        assert_eq!(structures.terms().len(), 1);
        assert_eq!(
            SourceSetTermProducer::build(
                single_enumeration_input(
                    source,
                    &module,
                    node(3),
                    range(source, 8, 23),
                    "{ 1 . carrier }",
                    SourceSetTarget::Structure(SourceStructureTermId::new(0)),
                ),
                &bindings,
                &structure_primary,
                None,
                Some(&structures),
                &structure_arena,
            ),
            Err(SourceSetTermError::StructureDependencyMismatch)
        );
    }

    #[test]
    fn supplied_non_root_application_and_structure_targets_are_rejected() {
        let source = source_id();
        let module = module_id();
        let bindings = binding_env(source, &module);

        let application_arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 18, 19)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 24, 25)),
                ),
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 10, 28)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 10, 11)),
                ),
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 14, 22)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 14, 15)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 8, 30)),
                ),
            ],
        )
        .expect("nested application arena");
        let application_primary = primary_handoff_from(
            source,
            &module,
            &bindings,
            &application_arena,
            &[(0, 18, 19, "1"), (1, 24, 25, "2")],
        );
        let applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![
                    SourceFunctorApplicationInput {
                        site: node(2),
                        source_range: range(source, 10, 28),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourceFunctorApplicationRecovery::Normal,
                        spelling: "f ( g ( 1 ) , 2 )".to_owned(),
                        kind: SourceFunctorApplicationKind::Inline,
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
                        context: BindingContextId::new(0),
                        recovery: SourceFunctorApplicationRecovery::Normal,
                        spelling: "g ( 1 )".to_owned(),
                        kind: SourceFunctorApplicationKind::Inline,
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
                candidates: Vec::new(),
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
                type_requests: Vec::new(),
            },
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &bindings,
            &application_primary,
            &application_arena,
        )
        .expect("valid nested application handoff");
        assert_eq!(applications.applications().len(), 2);
        assert_eq!(
            application_root_ids(&applications),
            BTreeSet::from([SourceFunctorApplicationId::new(0)])
        );
        assert_eq!(
            SourceSetTermProducer::build(
                single_enumeration_input(
                    source,
                    &module,
                    node(6),
                    range(source, 8, 30),
                    "{ g ( 1 ) }",
                    SourceSetTarget::Application(SourceFunctorApplicationId::new(1)),
                ),
                &bindings,
                &application_primary,
                Some(&applications),
                None,
                &application_arena,
            ),
            Err(SourceSetTermError::InvalidEdge {
                edge: SourceSetEdgeId::new(0),
            })
        );

        let structure_arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 12, 13)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(source, 10, 40)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(source, 30, 35)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(source, 10, 25)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(source, 18, 23)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 8, 42)),
                ),
            ],
        )
        .expect("nested structure arena");
        let structure_primary = primary_handoff_from(
            source,
            &module,
            &bindings,
            &structure_arena,
            &[(0, 12, 13, "1")],
        );
        let structures = SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![
                    SourceStructureTermInput {
                        site: node(1),
                        source_range: range(source, 10, 40),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourceStructureRecovery::Normal,
                        spelling: "1 . inner . outer".to_owned(),
                        kind: SourceStructureTermKind::SelectorAccess,
                    },
                    SourceStructureTermInput {
                        site: node(3),
                        source_range: range(source, 10, 25),
                        source_ordinal: 1,
                        context: BindingContextId::new(0),
                        recovery: SourceStructureRecovery::Normal,
                        spelling: "1 . inner".to_owned(),
                        kind: SourceStructureTermKind::SelectorAccess,
                    },
                ],
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: vec![
                    SourceStructureMemberInput {
                        term: SourceStructureTermId::new(0),
                        ordinal: 0,
                        site: node(2),
                        source_range: range(source, 30, 35),
                        spelling: "outer".to_owned(),
                        role: SourceStructureMemberRole::Selector,
                        parent: None,
                    },
                    SourceStructureMemberInput {
                        term: SourceStructureTermId::new(1),
                        ordinal: 0,
                        site: node(4),
                        source_range: range(source, 18, 23),
                        spelling: "inner".to_owned(),
                        role: SourceStructureMemberRole::Selector,
                        parent: None,
                    },
                ],
                field_updates: Vec::new(),
                edges: vec![
                    SourceStructureEdgeInput {
                        term: SourceStructureTermId::new(0),
                        ordinal: 0,
                        role: SourceStructureEdgeRole::SelectorBase,
                        member: None,
                        target: SourceStructureTarget::Structure(SourceStructureTermId::new(1)),
                    },
                    SourceStructureEdgeInput {
                        term: SourceStructureTermId::new(1),
                        ordinal: 0,
                        role: SourceStructureEdgeRole::SelectorBase,
                        member: None,
                        target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
                    },
                ],
                requests: vec![
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(1),
                        member: Some(SourceStructureMemberId::new(1)),
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(1),
                        member: Some(SourceStructureMemberId::new(1)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(1),
                        member: None,
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                ],
            },
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &bindings,
            &structure_primary,
            None,
            &structure_arena,
        )
        .expect("valid nested structure handoff");
        assert_eq!(structures.terms().len(), 2);
        assert_eq!(
            structure_root_ids(&structures),
            BTreeSet::from([SourceStructureTermId::new(0)])
        );
        assert_eq!(
            SourceSetTermProducer::build(
                single_enumeration_input(
                    source,
                    &module,
                    node(5),
                    range(source, 8, 42),
                    "{ 1 . inner }",
                    SourceSetTarget::Structure(SourceStructureTermId::new(1)),
                ),
                &bindings,
                &structure_primary,
                None,
                Some(&structures),
                &structure_arena,
            ),
            Err(SourceSetTermError::InvalidEdge {
                edge: SourceSetEdgeId::new(0),
            })
        );
    }

    #[test]
    fn arena_role_and_installation_corruption_are_rejected() {
        let baseline = fixture();
        let handoff = baseline.build().expect("handoff");
        for (index, wrong_key, expected_class) in [
            (4, "source.term.set.choice", "term"),
            (5, "source.term.set.enumeration", "term"),
            (6, "source.term.set.qua", "term"),
            (7, "source.term.set.choice", "term"),
            (8, "source.term.set.target-type", "generator"),
            (9, "source.term.set.target-type-head", "type"),
            (10, "source.term.set.target-type", "type"),
        ] {
            let mut nodes = arena_nodes(baseline.source);
            nodes[index].kind = wrong_key.into();
            let wrong_arena = TypedArena::try_new(None, nodes).expect("wrong arena");
            let error = SourceSetTermProducer::build(
                baseline.input.clone(),
                &baseline.bindings,
                &baseline.primary,
                None,
                None,
                &wrong_arena,
            )
            .expect_err("cross-role arena key must reject");
            assert!(
                matches!(
                    (&error, expected_class),
                    (SourceSetTermError::InvalidTerm { .. }, "term")
                        | (SourceSetTermError::InvalidGenerator { .. }, "generator")
                        | (SourceSetTermError::InvalidTypeSite { .. }, "type")
                ),
                "wrong error for key index {index}: {error:?}"
            );
        }

        let mut wrapped = baseline.clone();
        wrapped.input.wrappers.push(SourceSetWrapperInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: node(15),
            source_range: range(baseline.source, 8, 27),
            context: BindingContextId::new(0),
            recovery: SourceSetTermRecovery::Normal,
            spelling: "( { 1 , 2 } )".to_owned(),
        });
        let mut nodes = arena_nodes(baseline.source);
        nodes[15].kind = "source.term.set.enumeration".into();
        wrapped.arena = TypedArena::try_new(None, nodes).expect("wrapper-key arena");
        wrapped.primary = primary_handoff(
            wrapped.source,
            &wrapped.module,
            &wrapped.bindings,
            &wrapped.arena,
        );
        assert!(matches!(
            wrapped.build(),
            Err(SourceSetTermError::InvalidWrapper { .. })
        ));

        let mut nodes = arena_nodes(baseline.source);
        nodes[4].kind = "source.term.set.choice".into();
        let wrong_arena = TypedArena::try_new(None, nodes).expect("wrong install arena");
        assert!(matches!(
            handoff.validate_installation(
                baseline.source,
                &baseline.module,
                &baseline.primary,
                None,
                None,
                &wrong_arena,
            ),
            Err(SourceSetTermError::InvalidTerm { .. })
        ));
    }

    #[test]
    fn nested_set_term_uses_nearest_family_effective_wrapper() {
        let source = source_id();
        let module = module_id();
        let bindings = binding_env(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 10, 50)),
                ),
                TypedNode::new(
                    "source.term.set.choice",
                    SourceAnchor::Range(range(source, 20, 27)),
                ),
                TypedNode::new(
                    "source.term.set.target-type",
                    SourceAnchor::Range(range(source, 24, 27)),
                ),
                TypedNode::new(
                    "source.term.set.target-type-head",
                    SourceAnchor::Range(range(source, 24, 27)),
                ),
                TypedNode::new(
                    "source.term.set.parenthesized",
                    SourceAnchor::Range(range(source, 18, 29)),
                ),
            ],
        )
        .expect("arena");
        let primary = empty_primary_handoff(source, &module, &bindings, &arena);
        let handoff = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![
                    SourceSetTermInput {
                        site: node(0),
                        source_range: range(source, 10, 50),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        spelling: "{ ( the set ) }".to_owned(),
                        kind: SourceSetTermKind::Enumeration,
                    },
                    SourceSetTermInput {
                        site: node(1),
                        source_range: range(source, 20, 27),
                        source_ordinal: 1,
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        spelling: "the set".to_owned(),
                        kind: SourceSetTermKind::Choice,
                    },
                ],
                wrappers: vec![SourceSetWrapperInput {
                    term: SourceSetTermId::new(1),
                    ordinal: 0,
                    site: node(4),
                    source_range: range(source, 18, 29),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "( the set )".to_owned(),
                }],
                generators: Vec::new(),
                type_sites: vec![SourceSetTypeSiteInput {
                    owner: SourceSetTypeOwner::Term {
                        term: SourceSetTermId::new(1),
                        role: SourceSetTypeRole::ChoiceTarget,
                    },
                    site: node(2),
                    source_range: range(source, 24, 27),
                    spelling: "set".to_owned(),
                    head_site: node(3),
                    head_range: range(source, 24, 27),
                    head_spelling: "set".to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    head: SourceSetTypeHead::BuiltinSet,
                }],
                edges: vec![SourceSetEdgeInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    role: SourceSetEdgeRole::EnumerationElement,
                    target: SourceSetTarget::SetTerm(SourceSetTermId::new(1)),
                }],
                requests: vec![
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(0),
                        ordinal: 0,
                        kind: SourceSetRequestKind::ResultType,
                        generator: None,
                        type_site: None,
                    },
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 0,
                        kind: SourceSetRequestKind::ChoiceNonempty,
                        generator: None,
                        type_site: Some(SourceSetTypeSiteId::new(0)),
                    },
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 1,
                        kind: SourceSetRequestKind::ResultType,
                        generator: None,
                        type_site: None,
                    },
                ],
            },
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("nested set term");
        assert!(matches!(
            handoff.edges().get(SourceSetEdgeId::new(0)).unwrap().target(),
            SourceSetTarget::SetTerm(id) if id == SourceSetTermId::new(1)
        ));
    }

    #[test]
    fn root_application_target_drives_optional_fingerprint() {
        let source = source_id();
        let module = module_id();
        let bindings = binding_env(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 12, 27)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 12, 21)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 10, 30)),
                ),
            ],
        )
        .expect("arena");
        let primary = empty_primary_handoff(source, &module, &bindings, &arena);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(0),
                    source_range: range(source, 12, 27),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "inline255 ( )".to_owned(),
                    kind: SourceFunctorApplicationKind::Inline,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(1),
                        source_range: range(source, 12, 21),
                        spelling: "inline255".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: Vec::new(),
                type_requests: Vec::new(),
            },
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("application");
        let handoff = SourceSetTermProducer::build(
            single_enumeration_input(
                source,
                &module,
                node(2),
                range(source, 10, 30),
                "{ inline255 ( ) }",
                SourceSetTarget::Application(SourceFunctorApplicationId::new(0)),
            ),
            &bindings,
            &primary,
            Some(&applications),
            None,
            &arena,
        )
        .expect("application target");
        assert_eq!(
            handoff.application_fingerprint(),
            Some(applications.debug_text().as_str())
        );
        assert_eq!(handoff.structure_fingerprint(), None);
    }

    #[test]
    fn root_structure_target_owns_its_primary_descendant() {
        let source = source_id();
        let module = module_id();
        let bindings = binding_env(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 14, 15)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(source, 14, 25)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(source, 20, 25)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 10, 28)),
                ),
            ],
        )
        .expect("arena");
        let primary = primary_handoff_from(source, &module, &bindings, &arena, &[(0, 14, 15, "1")]);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let structures = SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourceStructureTermInput {
                    site: node(1),
                    source_range: range(source, 14, 25),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "1 . carrier".to_owned(),
                    kind: SourceStructureTermKind::SelectorAccess,
                }],
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: vec![SourceStructureMemberInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    site: node(2),
                    source_range: range(source, 20, 25),
                    spelling: "carrier".to_owned(),
                    role: SourceStructureMemberRole::Selector,
                    parent: None,
                }],
                field_updates: Vec::new(),
                edges: vec![SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::SelectorBase,
                    member: None,
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                requests: vec![
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                ],
            },
            &symbols,
            &bindings,
            &primary,
            None,
            &arena,
        )
        .expect("structure");
        let handoff = SourceSetTermProducer::build(
            single_enumeration_input(
                source,
                &module,
                node(3),
                range(source, 10, 28),
                "{ 1 . carrier }",
                SourceSetTarget::Structure(SourceStructureTermId::new(0)),
            ),
            &bindings,
            &primary,
            None,
            Some(&structures),
            &arena,
        )
        .expect("structure target");
        assert_eq!(
            handoff.structure_fingerprint(),
            Some(structures.debug_text().as_str())
        );
        assert_eq!(handoff.application_fingerprint(), None);
        assert_eq!(
            typed_ast_for(source, &module, &arena)
                .with_source_term(primary.clone())
                .expect("primary")
                .with_source_set_term(handoff.clone())
                .expect_err("fingerprinted structure must already be installed"),
            TypedAstError::InvalidSourceSetTerm
        );
        typed_ast_for(source, &module, &arena)
            .with_source_term(primary.clone())
            .expect("primary")
            .with_source_structure(structures.clone())
            .expect("structure first")
            .with_source_set_term(handoff.clone())
            .expect("fingerprinted set after structure");

        let mut mismatched = handoff;
        mismatched.structure_fingerprint = Some("stale-structure-fingerprint".to_owned());
        assert_eq!(
            mismatched.validate_installation(
                source,
                &module,
                &primary,
                None,
                Some(&structures),
                &arena,
            ),
            Err(SourceSetTermError::StructureDependencyMismatch)
        );
    }

    fn single_enumeration_input(
        source: SourceId,
        module: &ModuleId,
        site: TypedSiteRef,
        source_range: SourceRange,
        spelling: &str,
        target: SourceSetTarget,
    ) -> SourceSetTermHandoffInput {
        SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceSetTermInput {
                site,
                source_range,
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                spelling: spelling.to_owned(),
                kind: SourceSetTermKind::Enumeration,
            }],
            wrappers: Vec::new(),
            generators: Vec::new(),
            type_sites: Vec::new(),
            edges: vec![SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::EnumerationElement,
                target,
            }],
            requests: vec![SourceSetRequestInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                kind: SourceSetRequestKind::ResultType,
                generator: None,
                type_site: None,
            }],
        }
    }

    fn primary_handoff_from(
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

    fn single_primary_handoff_in_context(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        arena: &TypedArena,
        occurrence: (usize, usize, usize),
        context: BindingContextId,
    ) -> SourcePrimaryTermHandoff {
        let (site, start, end) = occurrence;
        SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourcePrimaryTermInput {
                    site: node(site),
                    source_range: range(source, start, end),
                    source_ordinal: 0,
                    context,
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "1".to_owned(),
                    kind: SourcePrimaryTermKind::Numeral,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }],
                references: Vec::new(),
                numeric_type_requests: vec![SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(0),
                    owner: node(site),
                    source_range: range(source, start, end),
                    spelling: "1".to_owned(),
                    request_ordinal: 0,
                }],
            },
            bindings,
            arena,
        )
        .expect("single primary handoff in selected context")
    }
}
