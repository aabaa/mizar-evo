//! Syntax-free direct transport for parser-origin template occurrences.

use crate::typed_ast::{NodeRecoveryState, TypedArena, TypedNodeId};
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

dense_id!(SourceTemplateParameterId);
dense_id!(SourceTemplateLociId);
dense_id!(SourceTemplateLocusId);
dense_id!(SourceTemplateArgumentsId);
dense_id!(SourceTemplateArgumentId);

/// Complete direct-parser input for one source/module template profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub parameters: Vec<SourceTemplateParameterInput>,
    pub loci_groups: Vec<SourceTemplateLociInput>,
    pub loci: Vec<SourceTemplateLocusInput>,
    pub argument_groups: Vec<SourceTemplateArgumentsInput>,
    pub arguments: Vec<SourceTemplateArgumentInput>,
}

/// One parser-origin template parameter site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateParameterInput {
    pub site: TypedNodeId,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceTemplateRecovery,
    pub parent: TypedNodeId,
    pub parent_kind: SourceTemplateParentKind,
    pub kind: SourceTemplateParameterKind,
}

/// One parser-origin template loci-group site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateLociInput {
    pub site: TypedNodeId,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceTemplateRecovery,
    pub parent: TypedNodeId,
    pub parent_kind: SourceTemplateParentKind,
}

/// One parser-origin template locus site within a loci group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateLocusInput {
    pub loci: SourceTemplateLociId,
    pub ordinal: usize,
    pub site: TypedNodeId,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceTemplateRecovery,
}

/// One parser-origin template argument-group site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateArgumentsInput {
    pub site: TypedNodeId,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceTemplateRecovery,
    pub parent: TypedNodeId,
    pub parent_kind: SourceTemplateParentKind,
}

/// One parser-origin template argument site within an argument group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateArgumentInput {
    pub arguments: SourceTemplateArgumentsId,
    pub ordinal: usize,
    pub site: TypedNodeId,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceTemplateRecovery,
}

/// Parser recovery state carried without diagnostic interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceTemplateRecovery {
    Normal,
    Recovered,
}

/// Provenance-only kind of a direct parser parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceTemplateParentKind {
    DefinitionBlockItem,
    PredicatePattern,
    FunctorPattern,
    PredicateHead,
    TermReference,
}

/// Provenance-only parser kind of a direct template parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceTemplateParameterKind {
    AbstractTypeSyntax,
    TypedValueSyntax,
}

/// One validated template parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateParameter {
    id: SourceTemplateParameterId,
    site: TypedNodeId,
    source_range: SourceRange,
    source_ordinal: usize,
    recovery: SourceTemplateRecovery,
    parent: TypedNodeId,
    parent_kind: SourceTemplateParentKind,
    kind: SourceTemplateParameterKind,
}

impl SourceTemplateParameter {
    pub const fn id(&self) -> SourceTemplateParameterId {
        self.id
    }

    pub const fn site(&self) -> TypedNodeId {
        self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn recovery(&self) -> SourceTemplateRecovery {
        self.recovery
    }

    pub const fn parent(&self) -> TypedNodeId {
        self.parent
    }

    pub const fn parent_kind(&self) -> SourceTemplateParentKind {
        self.parent_kind
    }

    pub const fn kind(&self) -> SourceTemplateParameterKind {
        self.kind
    }
}

/// Immutable dense template-parameter table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateParameterTable {
    rows: Vec<SourceTemplateParameter>,
}

impl SourceTemplateParameterTable {
    pub fn get(&self, id: SourceTemplateParameterId) -> Option<&SourceTemplateParameter> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceTemplateParameterId, &SourceTemplateParameter)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceTemplateParameterId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated template loci group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateLoci {
    id: SourceTemplateLociId,
    site: TypedNodeId,
    source_range: SourceRange,
    source_ordinal: usize,
    recovery: SourceTemplateRecovery,
    parent: TypedNodeId,
    parent_kind: SourceTemplateParentKind,
}

impl SourceTemplateLoci {
    pub const fn id(&self) -> SourceTemplateLociId {
        self.id
    }

    pub const fn site(&self) -> TypedNodeId {
        self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn recovery(&self) -> SourceTemplateRecovery {
        self.recovery
    }

    pub const fn parent(&self) -> TypedNodeId {
        self.parent
    }

    pub const fn parent_kind(&self) -> SourceTemplateParentKind {
        self.parent_kind
    }
}

/// Immutable dense template-loci table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateLociTable {
    rows: Vec<SourceTemplateLoci>,
}

impl SourceTemplateLociTable {
    pub fn get(&self, id: SourceTemplateLociId) -> Option<&SourceTemplateLoci> {
        self.rows.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceTemplateLociId, &SourceTemplateLoci)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceTemplateLociId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated template locus within a loci group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateLocus {
    id: SourceTemplateLocusId,
    loci: SourceTemplateLociId,
    ordinal: usize,
    site: TypedNodeId,
    source_range: SourceRange,
    source_ordinal: usize,
    recovery: SourceTemplateRecovery,
}

impl SourceTemplateLocus {
    pub const fn id(&self) -> SourceTemplateLocusId {
        self.id
    }

    pub const fn loci(&self) -> SourceTemplateLociId {
        self.loci
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn site(&self) -> TypedNodeId {
        self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn recovery(&self) -> SourceTemplateRecovery {
        self.recovery
    }
}

/// Immutable dense template-locus table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateLocusTable {
    rows: Vec<SourceTemplateLocus>,
}

impl SourceTemplateLocusTable {
    pub fn get(&self, id: SourceTemplateLocusId) -> Option<&SourceTemplateLocus> {
        self.rows.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceTemplateLocusId, &SourceTemplateLocus)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceTemplateLocusId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated template argument group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateArguments {
    id: SourceTemplateArgumentsId,
    site: TypedNodeId,
    source_range: SourceRange,
    source_ordinal: usize,
    recovery: SourceTemplateRecovery,
    parent: TypedNodeId,
    parent_kind: SourceTemplateParentKind,
}

impl SourceTemplateArguments {
    pub const fn id(&self) -> SourceTemplateArgumentsId {
        self.id
    }

    pub const fn site(&self) -> TypedNodeId {
        self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn recovery(&self) -> SourceTemplateRecovery {
        self.recovery
    }

    pub const fn parent(&self) -> TypedNodeId {
        self.parent
    }

    pub const fn parent_kind(&self) -> SourceTemplateParentKind {
        self.parent_kind
    }
}

/// Immutable dense template-argument-group table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateArgumentsTable {
    rows: Vec<SourceTemplateArguments>,
}

impl SourceTemplateArgumentsTable {
    pub fn get(&self, id: SourceTemplateArgumentsId) -> Option<&SourceTemplateArguments> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceTemplateArgumentsId, &SourceTemplateArguments)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceTemplateArgumentsId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated template argument within an argument group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateArgument {
    id: SourceTemplateArgumentId,
    arguments: SourceTemplateArgumentsId,
    ordinal: usize,
    site: TypedNodeId,
    source_range: SourceRange,
    source_ordinal: usize,
    recovery: SourceTemplateRecovery,
}

impl SourceTemplateArgument {
    pub const fn id(&self) -> SourceTemplateArgumentId {
        self.id
    }

    pub const fn arguments(&self) -> SourceTemplateArgumentsId {
        self.arguments
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn site(&self) -> TypedNodeId {
        self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn recovery(&self) -> SourceTemplateRecovery {
        self.recovery
    }
}

/// Immutable dense template-argument table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateArgumentTable {
    rows: Vec<SourceTemplateArgument>,
}

impl SourceTemplateArgumentTable {
    pub fn get(&self, id: SourceTemplateArgumentId) -> Option<&SourceTemplateArgument> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceTemplateArgumentId, &SourceTemplateArgument)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceTemplateArgumentId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Immutable direct parser-origin template transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    parameters: SourceTemplateParameterTable,
    loci_groups: SourceTemplateLociTable,
    loci: SourceTemplateLocusTable,
    argument_groups: SourceTemplateArgumentsTable,
    arguments: SourceTemplateArgumentTable,
}

impl SourceTemplateHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn parameters(&self) -> &SourceTemplateParameterTable {
        &self.parameters
    }

    pub const fn loci_groups(&self) -> &SourceTemplateLociTable {
        &self.loci_groups
    }

    pub const fn loci(&self) -> &SourceTemplateLocusTable {
        &self.loci
    }

    pub const fn argument_groups(&self) -> &SourceTemplateArgumentsTable {
        &self.argument_groups
    }

    pub const fn arguments(&self) -> &SourceTemplateArgumentTable {
        &self.arguments
    }

    /// Stable debug rendering of direct syntax provenance only.
    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-template-debug-v1\n");
        let _ = writeln!(
            output,
            "module: {}::{}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str()
        );
        for (id, row) in self.parameters.iter() {
            let _ = writeln!(
                output,
                "parameter#{} site={} range={}..{} ordinal={} recovery={} parent={} parent_kind={} kind={}",
                id.index(),
                row.site.index(),
                row.source_range.start,
                row.source_range.end,
                row.source_ordinal,
                recovery_key(row.recovery),
                row.parent.index(),
                parent_kind_key(row.parent_kind),
                parameter_kind_key(row.kind),
            );
        }
        for (id, row) in self.loci_groups.iter() {
            let _ = writeln!(
                output,
                "loci#{} site={} range={}..{} ordinal={} recovery={} parent={} parent_kind={}",
                id.index(),
                row.site.index(),
                row.source_range.start,
                row.source_range.end,
                row.source_ordinal,
                recovery_key(row.recovery),
                row.parent.index(),
                parent_kind_key(row.parent_kind),
            );
        }
        for (id, row) in self.loci.iter() {
            let _ = writeln!(
                output,
                "locus#{} loci={} ordinal={} site={} range={}..{} source_ordinal={} recovery={}",
                id.index(),
                row.loci.index(),
                row.ordinal,
                row.site.index(),
                row.source_range.start,
                row.source_range.end,
                row.source_ordinal,
                recovery_key(row.recovery),
            );
        }
        for (id, row) in self.argument_groups.iter() {
            let _ = writeln!(
                output,
                "arguments#{} site={} range={}..{} ordinal={} recovery={} parent={} parent_kind={}",
                id.index(),
                row.site.index(),
                row.source_range.start,
                row.source_range.end,
                row.source_ordinal,
                recovery_key(row.recovery),
                row.parent.index(),
                parent_kind_key(row.parent_kind),
            );
        }
        for (id, row) in self.arguments.iter() {
            let _ = writeln!(
                output,
                "argument#{} arguments={} ordinal={} site={} range={}..{} source_ordinal={} recovery={}",
                id.index(),
                row.arguments.index(),
                row.ordinal,
                row.site.index(),
                row.source_range.start,
                row.source_range.end,
                row.source_ordinal,
                recovery_key(row.recovery),
            );
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceTemplateError> {
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceTemplateError::EnvironmentMismatch);
        }
        SourceTemplateProducer::build(
            SourceTemplateHandoffInput {
                source_id: self.source_id,
                module_id: self.module_id.clone(),
                parameters: self
                    .parameters
                    .iter()
                    .map(|(_, row)| SourceTemplateParameterInput {
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                        parent: row.parent,
                        parent_kind: row.parent_kind,
                        kind: row.kind,
                    })
                    .collect(),
                loci_groups: self
                    .loci_groups
                    .iter()
                    .map(|(_, row)| SourceTemplateLociInput {
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                        parent: row.parent,
                        parent_kind: row.parent_kind,
                    })
                    .collect(),
                loci: self
                    .loci
                    .iter()
                    .map(|(_, row)| SourceTemplateLocusInput {
                        loci: row.loci,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                    })
                    .collect(),
                argument_groups: self
                    .argument_groups
                    .iter()
                    .map(|(_, row)| SourceTemplateArgumentsInput {
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                        parent: row.parent,
                        parent_kind: row.parent_kind,
                    })
                    .collect(),
                arguments: self
                    .arguments
                    .iter()
                    .map(|(_, row)| SourceTemplateArgumentInput {
                        arguments: row.arguments,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                    })
                    .collect(),
            },
            arena,
        )
        .map(|_| ())
    }
}

/// Direct parser-origin template transport failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceTemplateError {
    EnvironmentMismatch,
    InvalidParameter {
        parameter: SourceTemplateParameterId,
    },
    InvalidLoci {
        loci: SourceTemplateLociId,
    },
    InvalidLocus {
        locus: SourceTemplateLocusId,
    },
    InvalidArguments {
        arguments: SourceTemplateArgumentsId,
    },
    InvalidArgument {
        argument: SourceTemplateArgumentId,
    },
    DuplicateTemplateSite,
    ReorderedParameter {
        parameter: SourceTemplateParameterId,
    },
    ReorderedLoci {
        loci: SourceTemplateLociId,
    },
    ReorderedLocus {
        locus: SourceTemplateLocusId,
    },
    ReorderedArguments {
        arguments: SourceTemplateArgumentsId,
    },
    ReorderedArgument {
        argument: SourceTemplateArgumentId,
    },
}

impl fmt::Display for SourceTemplateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("source template environment mismatch")
            }
            Self::InvalidParameter { parameter } => {
                write!(
                    formatter,
                    "source template parameter {} is invalid",
                    parameter.index()
                )
            }
            Self::InvalidLoci { loci } => {
                write!(
                    formatter,
                    "source template loci group {} is invalid",
                    loci.index()
                )
            }
            Self::InvalidLocus { locus } => {
                write!(
                    formatter,
                    "source template locus {} is invalid",
                    locus.index()
                )
            }
            Self::InvalidArguments { arguments } => write!(
                formatter,
                "source template arguments group {} is invalid",
                arguments.index()
            ),
            Self::InvalidArgument { argument } => write!(
                formatter,
                "source template argument {} is invalid",
                argument.index()
            ),
            Self::DuplicateTemplateSite => {
                formatter.write_str("source template sites must be unique")
            }
            Self::ReorderedParameter { parameter } => write!(
                formatter,
                "source template parameter {} is out of source order",
                parameter.index()
            ),
            Self::ReorderedLoci { loci } => write!(
                formatter,
                "source template loci group {} is out of source order",
                loci.index()
            ),
            Self::ReorderedLocus { locus } => write!(
                formatter,
                "source template locus {} is out of source order",
                locus.index()
            ),
            Self::ReorderedArguments { arguments } => write!(
                formatter,
                "source template arguments group {} is out of source order",
                arguments.index()
            ),
            Self::ReorderedArgument { argument } => write!(
                formatter,
                "source template argument {} is out of source order",
                argument.index()
            ),
        }
    }
}

impl Error for SourceTemplateError {}

/// Validates and constructs direct parser-origin template transport.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceTemplateProducer;

impl SourceTemplateProducer {
    pub fn build(
        input: SourceTemplateHandoffInput,
        arena: &TypedArena,
    ) -> Result<SourceTemplateHandoff, SourceTemplateError> {
        validate_environment(&input)?;
        validate_parameters(&input, arena)?;
        validate_loci(&input, arena)?;
        validate_locus(&input, arena)?;
        validate_arguments(&input, arena)?;
        validate_argument(&input, arena)?;
        validate_unique_sites(&input)?;
        validate_parameter_order(&input)?;
        validate_loci_order(&input)?;
        validate_locus_order(&input)?;
        validate_arguments_order(&input)?;
        validate_argument_order(&input)?;

        Ok(SourceTemplateHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            parameters: SourceTemplateParameterTable {
                rows: input
                    .parameters
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceTemplateParameter {
                        id: SourceTemplateParameterId::new(index),
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                        parent: row.parent,
                        parent_kind: row.parent_kind,
                        kind: row.kind,
                    })
                    .collect(),
            },
            loci_groups: SourceTemplateLociTable {
                rows: input
                    .loci_groups
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceTemplateLoci {
                        id: SourceTemplateLociId::new(index),
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                        parent: row.parent,
                        parent_kind: row.parent_kind,
                    })
                    .collect(),
            },
            loci: SourceTemplateLocusTable {
                rows: input
                    .loci
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceTemplateLocus {
                        id: SourceTemplateLocusId::new(index),
                        loci: row.loci,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                    })
                    .collect(),
            },
            argument_groups: SourceTemplateArgumentsTable {
                rows: input
                    .argument_groups
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceTemplateArguments {
                        id: SourceTemplateArgumentsId::new(index),
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                        parent: row.parent,
                        parent_kind: row.parent_kind,
                    })
                    .collect(),
            },
            arguments: SourceTemplateArgumentTable {
                rows: input
                    .arguments
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceTemplateArgument {
                        id: SourceTemplateArgumentId::new(index),
                        arguments: row.arguments,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                    })
                    .collect(),
            },
        })
    }
}

fn validate_environment(input: &SourceTemplateHandoffInput) -> Result<(), SourceTemplateError> {
    let valid = input
        .parameters
        .iter()
        .map(|row| row.source_range)
        .chain(input.loci_groups.iter().map(|row| row.source_range))
        .chain(input.loci.iter().map(|row| row.source_range))
        .chain(input.argument_groups.iter().map(|row| row.source_range))
        .chain(input.arguments.iter().map(|row| row.source_range))
        .all(|range| valid_range(input.source_id, range));
    if valid {
        Ok(())
    } else {
        Err(SourceTemplateError::EnvironmentMismatch)
    }
}

fn validate_parameters(
    input: &SourceTemplateHandoffInput,
    arena: &TypedArena,
) -> Result<(), SourceTemplateError> {
    if input.parameters.len() != 2 {
        return Err(SourceTemplateError::InvalidParameter {
            parameter: SourceTemplateParameterId::new(input.parameters.len().min(1)),
        });
    }
    for (index, row) in input.parameters.iter().enumerate() {
        let id = SourceTemplateParameterId::new(index);
        let expected_kind = if index == 0 {
            SourceTemplateParameterKind::AbstractTypeSyntax
        } else {
            SourceTemplateParameterKind::TypedValueSyntax
        };
        if row.parent_kind != SourceTemplateParentKind::DefinitionBlockItem
            || row.kind != expected_kind
            || !valid_parent_site(row.parent, row.parent_kind, row.site, arena)
            || !valid_site(row.site, row.source_range, row.recovery, arena)
            || arena
                .node(row.site)
                .is_none_or(|node| node.kind.as_str() != parameter_kind_key(row.kind))
        {
            return Err(SourceTemplateError::InvalidParameter { parameter: id });
        }
    }
    Ok(())
}

fn validate_loci(
    input: &SourceTemplateHandoffInput,
    arena: &TypedArena,
) -> Result<(), SourceTemplateError> {
    if input.loci_groups.len() != 2 {
        return Err(SourceTemplateError::InvalidLoci {
            loci: SourceTemplateLociId::new(input.loci_groups.len().min(1)),
        });
    }
    for (index, row) in input.loci_groups.iter().enumerate() {
        let id = SourceTemplateLociId::new(index);
        let expected_parent_kind = if index == 0 {
            SourceTemplateParentKind::PredicatePattern
        } else {
            SourceTemplateParentKind::FunctorPattern
        };
        if row.parent_kind != expected_parent_kind
            || !valid_parent_site(row.parent, row.parent_kind, row.site, arena)
            || !valid_site(row.site, row.source_range, row.recovery, arena)
        {
            return Err(SourceTemplateError::InvalidLoci { loci: id });
        }
    }
    Ok(())
}

fn validate_locus(
    input: &SourceTemplateHandoffInput,
    arena: &TypedArena,
) -> Result<(), SourceTemplateError> {
    if input.loci.len() != 2 {
        return Err(SourceTemplateError::InvalidLocus {
            locus: SourceTemplateLocusId::new(input.loci.len().min(1)),
        });
    }
    for (index, row) in input.loci.iter().enumerate() {
        let id = SourceTemplateLocusId::new(index);
        let Some(group) = input.loci_groups.get(row.loci.index()) else {
            return Err(SourceTemplateError::InvalidLocus { locus: id });
        };
        if row.loci.index() != index
            || !valid_site(row.site, row.source_range, row.recovery, arena)
            || !arena
                .node(group.site)
                .is_some_and(|node| node.children.contains(&row.site))
        {
            return Err(SourceTemplateError::InvalidLocus { locus: id });
        }
    }
    Ok(())
}

fn validate_arguments(
    input: &SourceTemplateHandoffInput,
    arena: &TypedArena,
) -> Result<(), SourceTemplateError> {
    if input.argument_groups.len() != 2 {
        return Err(SourceTemplateError::InvalidArguments {
            arguments: SourceTemplateArgumentsId::new(input.argument_groups.len().min(1)),
        });
    }
    for (index, row) in input.argument_groups.iter().enumerate() {
        let id = SourceTemplateArgumentsId::new(index);
        let expected_parent_kind = if index == 0 {
            SourceTemplateParentKind::PredicateHead
        } else {
            SourceTemplateParentKind::TermReference
        };
        if row.parent_kind != expected_parent_kind
            || !valid_parent_site(row.parent, row.parent_kind, row.site, arena)
            || !valid_site(row.site, row.source_range, row.recovery, arena)
        {
            return Err(SourceTemplateError::InvalidArguments { arguments: id });
        }
    }
    Ok(())
}

fn validate_argument(
    input: &SourceTemplateHandoffInput,
    arena: &TypedArena,
) -> Result<(), SourceTemplateError> {
    if input.arguments.len() != 2 {
        return Err(SourceTemplateError::InvalidArgument {
            argument: SourceTemplateArgumentId::new(input.arguments.len().min(1)),
        });
    }
    for (index, row) in input.arguments.iter().enumerate() {
        let id = SourceTemplateArgumentId::new(index);
        let Some(group) = input.argument_groups.get(row.arguments.index()) else {
            return Err(SourceTemplateError::InvalidArgument { argument: id });
        };
        if row.arguments.index() != index
            || !valid_site(row.site, row.source_range, row.recovery, arena)
            || !arena
                .node(group.site)
                .is_some_and(|node| node.children.contains(&row.site))
        {
            return Err(SourceTemplateError::InvalidArgument { argument: id });
        }
    }
    Ok(())
}

fn validate_unique_sites(input: &SourceTemplateHandoffInput) -> Result<(), SourceTemplateError> {
    let mut sites = BTreeSet::new();
    let unique = input
        .parameters
        .iter()
        .map(|row| row.site)
        .chain(input.loci_groups.iter().map(|row| row.site))
        .chain(input.loci.iter().map(|row| row.site))
        .chain(input.argument_groups.iter().map(|row| row.site))
        .chain(input.arguments.iter().map(|row| row.site))
        .all(|site| sites.insert(site));
    if unique {
        Ok(())
    } else {
        Err(SourceTemplateError::DuplicateTemplateSite)
    }
}

fn validate_parameter_order(input: &SourceTemplateHandoffInput) -> Result<(), SourceTemplateError> {
    for (index, row) in input.parameters.iter().enumerate() {
        if row.source_ordinal != index
            || !range_precedes(index, row.source_range, &input.parameters, |row| {
                row.source_range
            })
        {
            return Err(SourceTemplateError::ReorderedParameter {
                parameter: SourceTemplateParameterId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_loci_order(input: &SourceTemplateHandoffInput) -> Result<(), SourceTemplateError> {
    for (index, row) in input.loci_groups.iter().enumerate() {
        if row.source_ordinal != index
            || !range_precedes(index, row.source_range, &input.loci_groups, |row| {
                row.source_range
            })
        {
            return Err(SourceTemplateError::ReorderedLoci {
                loci: SourceTemplateLociId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_locus_order(input: &SourceTemplateHandoffInput) -> Result<(), SourceTemplateError> {
    let mut next_ordinal = vec![0; input.loci_groups.len()];
    let mut previous_group = 0;
    for (index, row) in input.loci.iter().enumerate() {
        let group = row.loci.index();
        if row.source_ordinal != index
            || group < previous_group
            || row.ordinal != next_ordinal[group]
            || !range_precedes(index, row.source_range, &input.loci, |row| row.source_range)
        {
            return Err(SourceTemplateError::ReorderedLocus {
                locus: SourceTemplateLocusId::new(index),
            });
        }
        next_ordinal[group] += 1;
        previous_group = group;
    }
    Ok(())
}

fn validate_arguments_order(input: &SourceTemplateHandoffInput) -> Result<(), SourceTemplateError> {
    for (index, row) in input.argument_groups.iter().enumerate() {
        if row.source_ordinal != index
            || !range_precedes(index, row.source_range, &input.argument_groups, |row| {
                row.source_range
            })
        {
            return Err(SourceTemplateError::ReorderedArguments {
                arguments: SourceTemplateArgumentsId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_argument_order(input: &SourceTemplateHandoffInput) -> Result<(), SourceTemplateError> {
    let mut next_ordinal = vec![0; input.argument_groups.len()];
    let mut previous_group = 0;
    for (index, row) in input.arguments.iter().enumerate() {
        let group = row.arguments.index();
        if row.source_ordinal != index
            || group < previous_group
            || row.ordinal != next_ordinal[group]
            || !range_precedes(index, row.source_range, &input.arguments, |row| {
                row.source_range
            })
        {
            return Err(SourceTemplateError::ReorderedArgument {
                argument: SourceTemplateArgumentId::new(index),
            });
        }
        next_ordinal[group] += 1;
        previous_group = group;
    }
    Ok(())
}

fn valid_range(source_id: SourceId, range: SourceRange) -> bool {
    range.source_id == source_id
}

fn valid_site(
    site: TypedNodeId,
    source_range: SourceRange,
    recovery: SourceTemplateRecovery,
    arena: &TypedArena,
) -> bool {
    arena.node(site).is_some_and(|node| {
        source_range.start < source_range.end
            && node.anchor == SourceAnchor::Range(source_range)
            && recovery_matches(recovery, node.recovery)
    })
}

fn valid_parent_site(
    parent: TypedNodeId,
    parent_kind: SourceTemplateParentKind,
    child: TypedNodeId,
    arena: &TypedArena,
) -> bool {
    arena.node(parent).is_some_and(|node| {
        node.kind.as_str() == parent_kind_key(parent_kind) && node.children.contains(&child)
    })
}

fn recovery_matches(recovery: SourceTemplateRecovery, actual: NodeRecoveryState) -> bool {
    matches!(
        (recovery, actual),
        (SourceTemplateRecovery::Normal, NodeRecoveryState::Normal)
            | (
                SourceTemplateRecovery::Recovered,
                NodeRecoveryState::Recovered
            )
    )
}

fn range_precedes<T>(
    index: usize,
    current: SourceRange,
    rows: &[T],
    range: impl Fn(&T) -> SourceRange,
) -> bool {
    index == 0 || range(&rows[index - 1]).end <= current.start
}

fn recovery_key(recovery: SourceTemplateRecovery) -> &'static str {
    match recovery {
        SourceTemplateRecovery::Normal => "normal",
        SourceTemplateRecovery::Recovered => "recovered",
    }
}

fn parent_kind_key(kind: SourceTemplateParentKind) -> &'static str {
    match kind {
        SourceTemplateParentKind::DefinitionBlockItem => "DefinitionBlockItem",
        SourceTemplateParentKind::PredicatePattern => "PredicatePattern",
        SourceTemplateParentKind::FunctorPattern => "FunctorPattern",
        SourceTemplateParentKind::PredicateHead => "PredicateHead",
        SourceTemplateParentKind::TermReference => "TermReference",
    }
}

fn parameter_kind_key(kind: SourceTemplateParameterKind) -> &'static str {
    match kind {
        SourceTemplateParameterKind::AbstractTypeSyntax => "AbstractTypeSyntax",
        SourceTemplateParameterKind::TypedValueSyntax => "TypedValueSyntax",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_ast::{TypedArenaBuilder, TypedNode};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        input: SourceTemplateHandoffInput,
        arena: TypedArena,
    }

    fn fixture() -> Fixture {
        fixture_with_shared_loci_edge(false)
    }

    fn fixture_with_shared_loci_edge(shared_loci_edge: bool) -> Fixture {
        let snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .expect("snapshot");
        let source = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source");
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task277a"));
        let mut nodes = TypedArenaBuilder::new();

        let parameter_type = push_node(&mut nodes, source, "AbstractTypeSyntax", 13, 27, vec![]);
        let parameter_value = push_node(&mut nodes, source, "TypedValueSyntax", 30, 41, vec![]);
        let definition = push_node(
            &mut nodes,
            source,
            "DefinitionBlockItem",
            0,
            50,
            vec![parameter_type, parameter_value],
        );
        let locus_one = push_node(&mut nodes, source, "Identifier", 76, 77, vec![]);
        let loci_one = push_node(&mut nodes, source, "TemplateLoci", 75, 78, vec![locus_one]);
        let predicate_pattern = push_node(
            &mut nodes,
            source,
            "PredicatePattern",
            60,
            90,
            vec![loci_one],
        );
        let locus_two = push_node(&mut nodes, source, "Identifier", 121, 122, vec![]);
        let loci_two = push_node(
            &mut nodes,
            source,
            "TemplateLoci",
            120,
            123,
            vec![locus_two],
        );
        let functor_pattern = push_node(
            &mut nodes,
            source,
            "FunctorPattern",
            100,
            140,
            if shared_loci_edge {
                vec![loci_two, loci_one]
            } else {
                vec![loci_two]
            },
        );
        let argument_one = push_node(&mut nodes, source, "Term", 179, 180, vec![]);
        let arguments_one = push_node(
            &mut nodes,
            source,
            "TemplateArguments",
            178,
            181,
            vec![argument_one],
        );
        let predicate_head = push_node(
            &mut nodes,
            source,
            "PredicateHead",
            170,
            185,
            vec![arguments_one],
        );
        let argument_two = push_node(&mut nodes, source, "Term", 191, 192, vec![]);
        let arguments_two = push_node(
            &mut nodes,
            source,
            "TemplateArguments",
            190,
            193,
            vec![argument_two],
        );
        let term_reference = push_node(
            &mut nodes,
            source,
            "TermReference",
            188,
            200,
            vec![arguments_two],
        );
        let arena = nodes.finish(Some(term_reference)).expect("arena");
        let input = SourceTemplateHandoffInput {
            source_id: source,
            module_id: module.clone(),
            parameters: vec![
                SourceTemplateParameterInput {
                    site: parameter_type,
                    source_range: range(source, 13, 27),
                    source_ordinal: 0,
                    recovery: SourceTemplateRecovery::Normal,
                    parent: definition,
                    parent_kind: SourceTemplateParentKind::DefinitionBlockItem,
                    kind: SourceTemplateParameterKind::AbstractTypeSyntax,
                },
                SourceTemplateParameterInput {
                    site: parameter_value,
                    source_range: range(source, 30, 41),
                    source_ordinal: 1,
                    recovery: SourceTemplateRecovery::Normal,
                    parent: definition,
                    parent_kind: SourceTemplateParentKind::DefinitionBlockItem,
                    kind: SourceTemplateParameterKind::TypedValueSyntax,
                },
            ],
            loci_groups: vec![
                SourceTemplateLociInput {
                    site: loci_one,
                    source_range: range(source, 75, 78),
                    source_ordinal: 0,
                    recovery: SourceTemplateRecovery::Normal,
                    parent: predicate_pattern,
                    parent_kind: SourceTemplateParentKind::PredicatePattern,
                },
                SourceTemplateLociInput {
                    site: loci_two,
                    source_range: range(source, 120, 123),
                    source_ordinal: 1,
                    recovery: SourceTemplateRecovery::Normal,
                    parent: functor_pattern,
                    parent_kind: SourceTemplateParentKind::FunctorPattern,
                },
            ],
            loci: vec![
                SourceTemplateLocusInput {
                    loci: SourceTemplateLociId::new(0),
                    ordinal: 0,
                    site: locus_one,
                    source_range: range(source, 76, 77),
                    source_ordinal: 0,
                    recovery: SourceTemplateRecovery::Normal,
                },
                SourceTemplateLocusInput {
                    loci: SourceTemplateLociId::new(1),
                    ordinal: 0,
                    site: locus_two,
                    source_range: range(source, 121, 122),
                    source_ordinal: 1,
                    recovery: SourceTemplateRecovery::Normal,
                },
            ],
            argument_groups: vec![
                SourceTemplateArgumentsInput {
                    site: arguments_one,
                    source_range: range(source, 178, 181),
                    source_ordinal: 0,
                    recovery: SourceTemplateRecovery::Normal,
                    parent: predicate_head,
                    parent_kind: SourceTemplateParentKind::PredicateHead,
                },
                SourceTemplateArgumentsInput {
                    site: arguments_two,
                    source_range: range(source, 190, 193),
                    source_ordinal: 1,
                    recovery: SourceTemplateRecovery::Normal,
                    parent: term_reference,
                    parent_kind: SourceTemplateParentKind::TermReference,
                },
            ],
            arguments: vec![
                SourceTemplateArgumentInput {
                    arguments: SourceTemplateArgumentsId::new(0),
                    ordinal: 0,
                    site: argument_one,
                    source_range: range(source, 179, 180),
                    source_ordinal: 0,
                    recovery: SourceTemplateRecovery::Normal,
                },
                SourceTemplateArgumentInput {
                    arguments: SourceTemplateArgumentsId::new(1),
                    ordinal: 0,
                    site: argument_two,
                    source_range: range(source, 191, 192),
                    source_ordinal: 1,
                    recovery: SourceTemplateRecovery::Normal,
                },
            ],
        };
        Fixture {
            source,
            module,
            input,
            arena,
        }
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn push_node(
        nodes: &mut TypedArenaBuilder,
        source: SourceId,
        kind: &str,
        start: usize,
        end: usize,
        children: Vec<TypedNodeId>,
    ) -> TypedNodeId {
        nodes
            .push(
                TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                    .with_children(children),
            )
            .expect("node")
    }

    #[test]
    fn task277a_producer_preserves_exact_direct_template_profile() {
        let fixture = fixture();
        let handoff =
            SourceTemplateProducer::build(fixture.input, &fixture.arena).expect("handoff");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.parameters().len(), 2);
        assert_eq!(handoff.loci_groups().len(), 2);
        assert_eq!(handoff.loci().len(), 2);
        assert_eq!(handoff.argument_groups().len(), 2);
        assert_eq!(handoff.arguments().len(), 2);
        assert_eq!(
            handoff
                .parameters()
                .get(SourceTemplateParameterId::new(0))
                .unwrap()
                .kind(),
            SourceTemplateParameterKind::AbstractTypeSyntax
        );
        assert_eq!(
            handoff
                .parameters()
                .get(SourceTemplateParameterId::new(1))
                .unwrap()
                .kind(),
            SourceTemplateParameterKind::TypedValueSyntax
        );
        assert!(handoff.debug_text().contains("source-template-debug-v1"));
    }

    #[test]
    fn task277a_rejects_invalid_parent_kind_and_direct_edge() {
        let fixture = fixture();
        let mut wrong_kind = fixture.input.clone();
        wrong_kind.loci_groups[0].parent_kind = SourceTemplateParentKind::TermReference;
        assert!(matches!(
            SourceTemplateProducer::build(wrong_kind, &fixture.arena),
            Err(SourceTemplateError::InvalidLoci { loci })
                if loci == SourceTemplateLociId::new(0)
        ));
        let mut invalid_locus = fixture.input.clone();
        invalid_locus.loci[0].loci = SourceTemplateLociId::new(1);
        assert!(matches!(
            SourceTemplateProducer::build(invalid_locus, &fixture.arena),
            Err(SourceTemplateError::InvalidLocus { locus })
                if locus == SourceTemplateLocusId::new(0)
        ));
        let mut invalid_arguments = fixture.input.clone();
        invalid_arguments.argument_groups[0].parent_kind =
            SourceTemplateParentKind::DefinitionBlockItem;
        assert!(matches!(
            SourceTemplateProducer::build(invalid_arguments, &fixture.arena),
            Err(SourceTemplateError::InvalidArguments { arguments })
                if arguments == SourceTemplateArgumentsId::new(0)
        ));
        let mut detached = fixture.input;
        detached.arguments[0].site = TypedNodeId::new(0);
        detached.arguments[0].source_range = range(fixture.source, 13, 27);
        assert!(matches!(
            SourceTemplateProducer::build(detached, &fixture.arena),
            Err(SourceTemplateError::InvalidArgument { argument })
                if argument == SourceTemplateArgumentId::new(0)
        ));
    }

    #[test]
    fn task277a_rejects_group_order_and_duplicate_sites() {
        let fixture = fixture_with_shared_loci_edge(true);
        let mut duplicate = fixture.input.clone();
        duplicate.loci_groups[1].site = duplicate.loci_groups[0].site;
        duplicate.loci_groups[1].source_range = duplicate.loci_groups[0].source_range;
        duplicate.loci[1].site = duplicate.loci[0].site;
        duplicate.loci[1].source_range = duplicate.loci[0].source_range;
        assert!(matches!(
            SourceTemplateProducer::build(duplicate, &fixture.arena),
            Err(SourceTemplateError::DuplicateTemplateSite)
        ));
        let mut reordered_parameter = fixture.input.clone();
        reordered_parameter.parameters[1].source_ordinal = 0;
        assert!(matches!(
            SourceTemplateProducer::build(reordered_parameter, &fixture.arena),
            Err(SourceTemplateError::ReorderedParameter { parameter })
                if parameter == SourceTemplateParameterId::new(1)
        ));
        let mut reordered_loci = fixture.input.clone();
        reordered_loci.loci_groups[1].source_ordinal = 0;
        assert!(matches!(
            SourceTemplateProducer::build(reordered_loci, &fixture.arena),
            Err(SourceTemplateError::ReorderedLoci { loci })
                if loci == SourceTemplateLociId::new(1)
        ));
        let mut reordered_locus = fixture.input.clone();
        reordered_locus.loci[1].source_ordinal = 0;
        assert!(matches!(
            SourceTemplateProducer::build(reordered_locus, &fixture.arena),
            Err(SourceTemplateError::ReorderedLocus { locus })
                if locus == SourceTemplateLocusId::new(1)
        ));
        let mut reordered_arguments = fixture.input.clone();
        reordered_arguments.argument_groups[1].source_ordinal = 0;
        assert!(matches!(
            SourceTemplateProducer::build(reordered_arguments, &fixture.arena),
            Err(SourceTemplateError::ReorderedArguments { arguments })
                if arguments == SourceTemplateArgumentsId::new(1)
        ));
        let mut reordered_argument = fixture.input;
        reordered_argument.arguments[1].source_ordinal = 0;
        assert!(matches!(
            SourceTemplateProducer::build(reordered_argument, &fixture.arena),
            Err(SourceTemplateError::ReorderedArgument { argument })
                if argument == SourceTemplateArgumentId::new(1)
        ));
    }

    #[test]
    fn task277a_rejects_environment_recovery_range_and_duplicate_installation() {
        let fixture = fixture();
        let mut environment = fixture.input.clone();
        let other_snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        let _ = allocator
            .next_source_id(other_snapshot)
            .expect("first source");
        let other_source = allocator
            .next_source_id(other_snapshot)
            .expect("second source");
        environment.parameters[0].source_range.source_id = other_source;
        assert!(matches!(
            SourceTemplateProducer::build(environment, &fixture.arena),
            Err(SourceTemplateError::EnvironmentMismatch)
        ));
        let mut recovery = fixture.input.clone();
        recovery.parameters[0].recovery = SourceTemplateRecovery::Recovered;
        assert!(matches!(
            SourceTemplateProducer::build(recovery, &fixture.arena),
            Err(SourceTemplateError::InvalidParameter { parameter })
                if parameter == SourceTemplateParameterId::new(0)
        ));
        for (start, end) in [(27, 27), (28, 27)] {
            let mut invalid_range = fixture.input.clone();
            invalid_range.parameters[0].source_range.start = start;
            invalid_range.parameters[0].source_range.end = end;
            assert!(matches!(
                SourceTemplateProducer::build(invalid_range, &fixture.arena),
                Err(SourceTemplateError::InvalidParameter { .. })
            ));
        }
        let handoff =
            SourceTemplateProducer::build(fixture.input, &fixture.arena).expect("handoff");
        assert!(
            handoff
                .validate_installation(fixture.source, &fixture.module, &fixture.arena)
                .is_ok()
        );
        let typed_ast = crate::typed_ast::TypedAst::try_new(crate::typed_ast::TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: fixture.arena,
            contexts: crate::typed_ast::LocalTypeContextTable::new(),
            types: crate::typed_ast::TypeTable::new(),
            facts: crate::typed_ast::TypeFactTable::new(),
            coercions: crate::typed_ast::CoercionTable::new(),
            initial_obligations: crate::typed_ast::InitialObligationTable::new(),
            diagnostics: crate::typed_ast::TypeDiagnosticTable::new(),
        })
        .expect("typed AST")
        .with_source_template(handoff.clone())
        .expect("first source template installation");
        assert_eq!(typed_ast.source_template(), Some(&handoff));
        assert!(matches!(
            typed_ast.with_source_template(handoff),
            Err(crate::typed_ast::TypedAstError::InvalidSourceTemplate)
        ));

        let precedence_fixture = fixture_with_shared_loci_edge(true);
        let input_with_faults = |first_error: usize| {
            let mut input = precedence_fixture.input.clone();
            if first_error == 0 {
                input.parameters[0].source_range.source_id = other_source;
            }
            if first_error <= 1 {
                input.parameters[0].recovery = SourceTemplateRecovery::Recovered;
            }
            if first_error <= 2 {
                input.loci_groups[0].parent_kind = SourceTemplateParentKind::TermReference;
            }
            if first_error <= 3 {
                input.loci[0].loci = SourceTemplateLociId::new(1);
            }
            if first_error <= 4 {
                input.argument_groups[0].parent_kind =
                    SourceTemplateParentKind::DefinitionBlockItem;
            }
            if first_error <= 5 {
                input.arguments[0].arguments = SourceTemplateArgumentsId::new(1);
            }
            if first_error <= 6 {
                input.loci_groups[1].site = input.loci_groups[0].site;
                input.loci_groups[1].source_range = input.loci_groups[0].source_range;
                input.loci[1].site = input.loci[0].site;
                input.loci[1].source_range = input.loci[0].source_range;
            }
            if first_error <= 7 {
                input.parameters[1].source_ordinal = 0;
            }
            if first_error <= 8 {
                input.loci_groups[1].source_ordinal = 0;
            }
            if first_error <= 9 {
                input.loci[1].source_ordinal = 0;
            }
            if first_error <= 10 {
                input.argument_groups[1].source_ordinal = 0;
            }
            if first_error <= 11 {
                input.arguments[1].source_ordinal = 0;
            }
            input
        };
        let expected_precedence = [
            SourceTemplateError::EnvironmentMismatch,
            SourceTemplateError::InvalidParameter {
                parameter: SourceTemplateParameterId::new(0),
            },
            SourceTemplateError::InvalidLoci {
                loci: SourceTemplateLociId::new(0),
            },
            SourceTemplateError::InvalidLocus {
                locus: SourceTemplateLocusId::new(0),
            },
            SourceTemplateError::InvalidArguments {
                arguments: SourceTemplateArgumentsId::new(0),
            },
            SourceTemplateError::InvalidArgument {
                argument: SourceTemplateArgumentId::new(0),
            },
            SourceTemplateError::DuplicateTemplateSite,
            SourceTemplateError::ReorderedParameter {
                parameter: SourceTemplateParameterId::new(1),
            },
            SourceTemplateError::ReorderedLoci {
                loci: SourceTemplateLociId::new(1),
            },
            SourceTemplateError::ReorderedLocus {
                locus: SourceTemplateLocusId::new(1),
            },
            SourceTemplateError::ReorderedArguments {
                arguments: SourceTemplateArgumentsId::new(1),
            },
            SourceTemplateError::ReorderedArgument {
                argument: SourceTemplateArgumentId::new(1),
            },
        ];
        for (first_error, expected) in expected_precedence.into_iter().enumerate() {
            assert_eq!(
                SourceTemplateProducer::build(
                    input_with_faults(first_error),
                    &precedence_fixture.arena,
                )
                .expect_err("Task277A precedence fault must reject"),
                expected,
            );
        }
    }
}
