//! Syntax-free source attribute-chain handoff.

use crate::{
    binding_env::{BindingEnv, BindingTypeSite},
    source_type::{SourceTypeApplicationHandoff, SourceTypeExpressionId},
    typed_ast::{NodeRecoveryState, TypedArena, TypedSiteRef},
};
use mizar_resolve::{
    env::{
        ContributionKind, ExportStatus, SourceContributionId, SymbolEnv, SymbolKind, Visibility,
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

dense_id!(SourceAttributeChainId);
dense_id!(SourceAttributeId);
dense_id!(SourceAttributeQualifierId);
dense_id!(SourceAttributeArgumentGroupId);
dense_id!(SourceAttributeArgumentId);

/// Complete flat input for one source/module attribute transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub chains: Vec<SourceAttributeChainInput>,
    pub attributes: Vec<SourceAttributeInput>,
    pub qualifiers: Vec<SourceAttributeQualifierInput>,
    pub argument_groups: Vec<SourceAttributeArgumentGroupInput>,
    pub arguments: Vec<SourceAttributeArgumentInput>,
}

/// One nonempty attribute chain attached to a Task-249 expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeChainInput {
    pub expression: SourceTypeExpressionId,
    pub source_ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub recovery: NodeRecoveryState,
}

/// One source-written attribute occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeInput {
    pub chain: SourceAttributeChainId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub target_site: TypedSiteRef,
    pub target_range: SourceRange,
    pub target_spelling: String,
    pub recovery: NodeRecoveryState,
    pub symbol: SymbolId,
    pub contribution: SourceContributionId,
    pub polarity: SourceAttributePolarityInput,
}

/// Source-written attribute polarity and optional exact `non` occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceAttributePolarityInput {
    Positive,
    Negative {
        non_site: TypedSiteRef,
        non_range: SourceRange,
        non_spelling: String,
        non_recovery: NodeRecoveryState,
    },
}

/// One resolver-authenticated written `structure-name.` qualifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeQualifierInput {
    pub attribute: SourceAttributeId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub recovery: NodeRecoveryState,
    pub structure: SymbolId,
    pub contribution: SourceContributionId,
}

/// Source-written attribute argument-group family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAttributeArgumentGroupKind {
    Prefix,
    ParenthesizedArgumentList,
}

/// Source-written prefix punctuation form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAttributePrefixForm {
    Single,
    Parenthesized,
}

/// One prefix or parenthesized argument-list occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeArgumentGroupInput {
    pub attribute: SourceAttributeId,
    pub ordinal: usize,
    pub kind: SourceAttributeArgumentGroupKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub recovery: NodeRecoveryState,
    pub prefix_form: Option<SourceAttributePrefixForm>,
    pub hyphen_range: Option<SourceRange>,
    pub hyphen_spelling: Option<String>,
    pub open_range: Option<SourceRange>,
    pub open_spelling: Option<String>,
    pub close_range: Option<SourceRange>,
    pub close_spelling: Option<String>,
    pub comma_ranges: Vec<SourceRange>,
    pub comma_spellings: Vec<String>,
}

/// Source-written actual family. No variant carries a selected binding or type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAttributeActualKind {
    PrefixIdentifier,
    PrefixNumeral,
    TermSite,
}

/// One dense source-ordered attribute actual.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeArgumentInput {
    pub group: SourceAttributeArgumentGroupId,
    pub ordinal: usize,
    pub kind: SourceAttributeActualKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub recovery: NodeRecoveryState,
    pub provenance: SemanticOrigin,
}

/// Immutable validated source-attribute table bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    chains: SourceAttributeChainTable,
    attributes: SourceAttributeTable,
    qualifiers: SourceAttributeQualifierTable,
    argument_groups: SourceAttributeArgumentGroupTable,
    arguments: SourceAttributeArgumentTable,
}

impl SourceAttributeHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn chains(&self) -> &SourceAttributeChainTable {
        &self.chains
    }

    pub const fn attributes(&self) -> &SourceAttributeTable {
        &self.attributes
    }

    pub const fn qualifiers(&self) -> &SourceAttributeQualifierTable {
        &self.qualifiers
    }

    pub const fn argument_groups(&self) -> &SourceAttributeArgumentGroupTable {
        &self.argument_groups
    }

    pub const fn arguments(&self) -> &SourceAttributeArgumentTable {
        &self.arguments
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-attribute-debug-v1\n");
        output.push_str("module: ");
        output.push_str(self.module_id.path().as_str());
        output.push('\n');
        for (id, row) in self.chains.iter() {
            let _ = write!(
                output,
                "chain#{} expression={} ordinal={} range={}..{} site=",
                id.index(),
                row.expression.index(),
                row.source_ordinal,
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " recovery={} spelling={:?}",
                recovery_key(row.recovery),
                row.spelling
            );
        }
        for (id, row) in self.attributes.iter() {
            let _ = write!(
                output,
                "attribute#{} chain={} ordinal={} range={}..{} site=",
                id.index(),
                row.chain.index(),
                row.ordinal,
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = write!(
                output,
                " target_range={}..{} target_site=",
                row.target_range.start, row.target_range.end
            );
            write_site(&mut output, &row.target_site);
            let _ = write!(
                output,
                " symbol={:?} contribution={} recovery={} spelling={:?} target_spelling={:?} ",
                row.symbol,
                row.contribution.index(),
                recovery_key(row.recovery),
                row.spelling,
                row.target_spelling
            );
            match &row.polarity {
                SourceAttributePolarityInput::Positive => output.push_str("polarity=positive\n"),
                SourceAttributePolarityInput::Negative {
                    non_site,
                    non_range,
                    non_spelling,
                    non_recovery,
                } => {
                    let _ = write!(
                        output,
                        "polarity=negative non_range={}..{} non_site=",
                        non_range.start, non_range.end
                    );
                    write_site(&mut output, non_site);
                    let _ = writeln!(
                        output,
                        " non_recovery={} non_spelling={:?}",
                        recovery_key(*non_recovery),
                        non_spelling
                    );
                }
            }
        }
        for (id, row) in self.qualifiers.iter() {
            let _ = write!(
                output,
                "qualifier#{} attribute={} range={}..{} site=",
                id.index(),
                row.attribute.index(),
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " structure={:?} contribution={} recovery={} spelling={:?}",
                row.structure,
                row.contribution.index(),
                recovery_key(row.recovery),
                row.spelling
            );
        }
        for (id, row) in self.argument_groups.iter() {
            let _ = write!(
                output,
                "group#{} attribute={} ordinal={} kind={} range={}..{} site=",
                id.index(),
                row.attribute.index(),
                row.ordinal,
                group_kind_key(row.kind),
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " prefix_form={} recovery={} spelling={:?} hyphen={:?}/{:?} open={:?}/{:?} close={:?}/{:?} commas={:?}/{:?}",
                row.prefix_form.map_or("none", prefix_form_key),
                recovery_key(row.recovery),
                row.spelling,
                row.hyphen_range,
                row.hyphen_spelling,
                row.open_range,
                row.open_spelling,
                row.close_range,
                row.close_spelling,
                row.comma_ranges,
                row.comma_spellings
            );
        }
        for (id, row) in self.arguments.iter() {
            let _ = write!(
                output,
                "actual#{} group={} ordinal={} kind={} range={}..{} site=",
                id.index(),
                row.group.index(),
                row.ordinal,
                actual_kind_key(row.kind),
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " recovery={} spelling={:?} origin={:?}",
                recovery_key(row.recovery),
                row.spelling,
                row.provenance
            );
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        source_type: &SourceTypeApplicationHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceAttributeError> {
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceAttributeError::EnvironmentMismatch);
        }
        for (id, chain) in self.chains.iter() {
            let expression = source_type
                .expressions()
                .get(chain.expression)
                .ok_or(SourceAttributeError::InvalidChain { chain: id })?;
            if chain.site != *expression.site()
                || chain.source_range != expression.source_range()
                || chain.spelling != expression.spelling()
                || chain.recovery != expression.recovery()
            {
                return Err(SourceAttributeError::InvalidChain { chain: id });
            }
            validate_site(
                &chain.site,
                chain.source_range,
                chain.recovery,
                arena,
                SourceAttributeError::InvalidChainSite { chain: id },
            )?;
        }
        for (id, attribute) in self.attributes.iter() {
            validate_attribute_sites(id, attribute, arena)?;
        }
        for (id, qualifier) in self.qualifiers.iter() {
            validate_site(
                &qualifier.site,
                qualifier.source_range,
                qualifier.recovery,
                arena,
                SourceAttributeError::InvalidQualifierSite { qualifier: id },
            )?;
        }
        for (id, group) in self.argument_groups.iter() {
            validate_site(
                &group.site,
                group.source_range,
                group.recovery,
                arena,
                SourceAttributeError::InvalidArgumentGroupSite { group: id },
            )?;
        }
        for (id, argument) in self.arguments.iter() {
            validate_site(
                &argument.site,
                argument.source_range,
                argument.recovery,
                arena,
                SourceAttributeError::InvalidArgumentSite { argument: id },
            )?;
        }
        Ok(())
    }
}

macro_rules! table {
    ($table:ident, $row:ident, $id:ident) => {
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $table {
            entries: Vec<$row>,
        }

        impl $table {
            pub fn get(&self, id: $id) -> Option<&$row> {
                self.entries.get(id.index())
            }

            pub fn iter(&self) -> impl Iterator<Item = ($id, &$row)> {
                self.entries.iter().map(|entry| (entry.id, entry))
            }

            pub const fn len(&self) -> usize {
                self.entries.len()
            }

            pub const fn is_empty(&self) -> bool {
                self.entries.is_empty()
            }
        }
    };
}

table!(
    SourceAttributeChainTable,
    SourceAttributeChain,
    SourceAttributeChainId
);
table!(SourceAttributeTable, SourceAttribute, SourceAttributeId);
table!(
    SourceAttributeQualifierTable,
    SourceAttributeQualifier,
    SourceAttributeQualifierId
);
table!(
    SourceAttributeArgumentGroupTable,
    SourceAttributeArgumentGroup,
    SourceAttributeArgumentGroupId
);
table!(
    SourceAttributeArgumentTable,
    SourceAttributeArgument,
    SourceAttributeArgumentId
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeChain {
    id: SourceAttributeChainId,
    expression: SourceTypeExpressionId,
    source_ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    recovery: NodeRecoveryState,
}

impl SourceAttributeChain {
    pub const fn id(&self) -> SourceAttributeChainId {
        self.id
    }
    pub const fn expression(&self) -> SourceTypeExpressionId {
        self.expression
    }
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
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
    pub const fn recovery(&self) -> NodeRecoveryState {
        self.recovery
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttribute {
    id: SourceAttributeId,
    chain: SourceAttributeChainId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    target_site: TypedSiteRef,
    target_range: SourceRange,
    target_spelling: String,
    recovery: NodeRecoveryState,
    symbol: SymbolId,
    contribution: SourceContributionId,
    polarity: SourceAttributePolarityInput,
}

impl SourceAttribute {
    pub const fn id(&self) -> SourceAttributeId {
        self.id
    }
    pub const fn chain(&self) -> SourceAttributeChainId {
        self.chain
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
    pub const fn target_site(&self) -> &TypedSiteRef {
        &self.target_site
    }
    pub const fn target_range(&self) -> SourceRange {
        self.target_range
    }
    pub fn target_spelling(&self) -> &str {
        &self.target_spelling
    }
    pub const fn recovery(&self) -> NodeRecoveryState {
        self.recovery
    }
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
    pub const fn polarity(&self) -> &SourceAttributePolarityInput {
        &self.polarity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeQualifier {
    id: SourceAttributeQualifierId,
    attribute: SourceAttributeId,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    recovery: NodeRecoveryState,
    structure: SymbolId,
    contribution: SourceContributionId,
}

impl SourceAttributeQualifier {
    pub const fn id(&self) -> SourceAttributeQualifierId {
        self.id
    }
    pub const fn attribute(&self) -> SourceAttributeId {
        self.attribute
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
    pub const fn recovery(&self) -> NodeRecoveryState {
        self.recovery
    }
    pub const fn structure(&self) -> &SymbolId {
        &self.structure
    }
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeArgumentGroup {
    id: SourceAttributeArgumentGroupId,
    attribute: SourceAttributeId,
    ordinal: usize,
    kind: SourceAttributeArgumentGroupKind,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    recovery: NodeRecoveryState,
    prefix_form: Option<SourceAttributePrefixForm>,
    hyphen_range: Option<SourceRange>,
    hyphen_spelling: Option<String>,
    open_range: Option<SourceRange>,
    open_spelling: Option<String>,
    close_range: Option<SourceRange>,
    close_spelling: Option<String>,
    comma_ranges: Vec<SourceRange>,
    comma_spellings: Vec<String>,
}

impl SourceAttributeArgumentGroup {
    pub const fn id(&self) -> SourceAttributeArgumentGroupId {
        self.id
    }
    pub const fn attribute(&self) -> SourceAttributeId {
        self.attribute
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceAttributeArgumentGroupKind {
        self.kind
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
    pub const fn recovery(&self) -> NodeRecoveryState {
        self.recovery
    }
    pub const fn prefix_form(&self) -> Option<SourceAttributePrefixForm> {
        self.prefix_form
    }
    pub const fn hyphen_range(&self) -> Option<SourceRange> {
        self.hyphen_range
    }
    pub fn hyphen_spelling(&self) -> Option<&str> {
        self.hyphen_spelling.as_deref()
    }
    pub const fn open_range(&self) -> Option<SourceRange> {
        self.open_range
    }
    pub fn open_spelling(&self) -> Option<&str> {
        self.open_spelling.as_deref()
    }
    pub const fn close_range(&self) -> Option<SourceRange> {
        self.close_range
    }
    pub fn close_spelling(&self) -> Option<&str> {
        self.close_spelling.as_deref()
    }
    pub fn comma_ranges(&self) -> &[SourceRange] {
        &self.comma_ranges
    }
    pub fn comma_spellings(&self) -> &[String] {
        &self.comma_spellings
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeArgument {
    id: SourceAttributeArgumentId,
    group: SourceAttributeArgumentGroupId,
    ordinal: usize,
    kind: SourceAttributeActualKind,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    recovery: NodeRecoveryState,
    provenance: SemanticOrigin,
}

impl SourceAttributeArgument {
    pub const fn id(&self) -> SourceAttributeArgumentId {
        self.id
    }
    pub const fn group(&self) -> SourceAttributeArgumentGroupId {
        self.group
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceAttributeActualKind {
        self.kind
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
    pub const fn recovery(&self) -> NodeRecoveryState {
        self.recovery
    }
    pub const fn provenance(&self) -> &SemanticOrigin {
        &self.provenance
    }
}

/// Validates and transactionally constructs source-attribute handoffs.
pub struct SourceAttributeProducer;

impl SourceAttributeProducer {
    pub fn build(
        input: SourceAttributeHandoffInput,
        source_type: &SourceTypeApplicationHandoff,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<SourceAttributeHandoff, SourceAttributeError> {
        validate_input(&input, source_type, bindings, symbols, arena)?;
        Ok(SourceAttributeHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            chains: SourceAttributeChainTable {
                entries: input
                    .chains
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceAttributeChain {
                        id: SourceAttributeChainId::new(index),
                        expression: row.expression,
                        source_ordinal: row.source_ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        recovery: row.recovery,
                    })
                    .collect(),
            },
            attributes: SourceAttributeTable {
                entries: input
                    .attributes
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceAttribute {
                        id: SourceAttributeId::new(index),
                        chain: row.chain,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        target_site: row.target_site,
                        target_range: row.target_range,
                        target_spelling: row.target_spelling,
                        recovery: row.recovery,
                        symbol: row.symbol,
                        contribution: row.contribution,
                        polarity: row.polarity,
                    })
                    .collect(),
            },
            qualifiers: SourceAttributeQualifierTable {
                entries: input
                    .qualifiers
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceAttributeQualifier {
                        id: SourceAttributeQualifierId::new(index),
                        attribute: row.attribute,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        recovery: row.recovery,
                        structure: row.structure,
                        contribution: row.contribution,
                    })
                    .collect(),
            },
            argument_groups: SourceAttributeArgumentGroupTable {
                entries: input
                    .argument_groups
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceAttributeArgumentGroup {
                        id: SourceAttributeArgumentGroupId::new(index),
                        attribute: row.attribute,
                        ordinal: row.ordinal,
                        kind: row.kind,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        recovery: row.recovery,
                        prefix_form: row.prefix_form,
                        hyphen_range: row.hyphen_range,
                        hyphen_spelling: row.hyphen_spelling,
                        open_range: row.open_range,
                        open_spelling: row.open_spelling,
                        close_range: row.close_range,
                        close_spelling: row.close_spelling,
                        comma_ranges: row.comma_ranges,
                        comma_spellings: row.comma_spellings,
                    })
                    .collect(),
            },
            arguments: SourceAttributeArgumentTable {
                entries: input
                    .arguments
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceAttributeArgument {
                        id: SourceAttributeArgumentId::new(index),
                        group: row.group,
                        ordinal: row.ordinal,
                        kind: row.kind,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        recovery: row.recovery,
                        provenance: row.provenance,
                    })
                    .collect(),
            },
        })
    }
}

/// Transaction validation failures. No failure publishes a partial handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceAttributeError {
    EmptyChains,
    EmptyAttributes,
    EnvironmentMismatch,
    InvalidSourceType,
    InvalidChain {
        chain: SourceAttributeChainId,
    },
    InvalidChainSite {
        chain: SourceAttributeChainId,
    },
    InvalidAttribute {
        attribute: SourceAttributeId,
    },
    InvalidAttributeSite {
        attribute: SourceAttributeId,
    },
    InvalidAttributeSymbol {
        attribute: SourceAttributeId,
    },
    InvalidPolarity {
        attribute: SourceAttributeId,
    },
    InvalidQualifier {
        qualifier: SourceAttributeQualifierId,
    },
    InvalidQualifierSite {
        qualifier: SourceAttributeQualifierId,
    },
    InvalidQualifierSymbol {
        qualifier: SourceAttributeQualifierId,
    },
    InvalidArgumentGroup {
        group: SourceAttributeArgumentGroupId,
    },
    InvalidArgumentGroupSite {
        group: SourceAttributeArgumentGroupId,
    },
    InvalidPunctuation {
        group: SourceAttributeArgumentGroupId,
    },
    InvalidArgument {
        argument: SourceAttributeArgumentId,
    },
    InvalidArgumentSite {
        argument: SourceAttributeArgumentId,
    },
    InvalidArgumentProvenance {
        argument: SourceAttributeArgumentId,
    },
    ReorderedAttribute {
        attribute: SourceAttributeId,
    },
    ReorderedQualifier {
        qualifier: SourceAttributeQualifierId,
    },
    ReorderedArgumentGroup {
        group: SourceAttributeArgumentGroupId,
    },
    ReorderedArgument {
        argument: SourceAttributeArgumentId,
    },
    DuplicateSite,
    PartialBundle,
}

impl fmt::Display for SourceAttributeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChains => formatter.write_str("source attribute input has no chains"),
            Self::EmptyAttributes => {
                formatter.write_str("source attribute input has no attributes")
            }
            Self::EnvironmentMismatch => {
                formatter.write_str("source attribute environment identity mismatch")
            }
            Self::InvalidSourceType => {
                formatter.write_str("source attribute dependency handoff is invalid")
            }
            Self::InvalidChain { chain } => write!(
                formatter,
                "source attribute chain {} is invalid",
                chain.index()
            ),
            Self::InvalidChainSite { chain } => write!(
                formatter,
                "source attribute chain {} has an invalid typed site",
                chain.index()
            ),
            Self::InvalidAttribute { attribute } => write!(
                formatter,
                "source attribute {} is invalid",
                attribute.index()
            ),
            Self::InvalidAttributeSite { attribute } => write!(
                formatter,
                "source attribute {} has an invalid typed site",
                attribute.index()
            ),
            Self::InvalidAttributeSymbol { attribute } => write!(
                formatter,
                "source attribute {} has unauthenticated symbol provenance",
                attribute.index()
            ),
            Self::InvalidPolarity { attribute } => write!(
                formatter,
                "source attribute {} has invalid polarity",
                attribute.index()
            ),
            Self::InvalidQualifier { qualifier } => write!(
                formatter,
                "source attribute qualifier {} is invalid",
                qualifier.index()
            ),
            Self::InvalidQualifierSite { qualifier } => write!(
                formatter,
                "source attribute qualifier {} has an invalid typed site",
                qualifier.index()
            ),
            Self::InvalidQualifierSymbol { qualifier } => write!(
                formatter,
                "source attribute qualifier {} has unauthenticated structure provenance",
                qualifier.index()
            ),
            Self::InvalidArgumentGroup { group } => write!(
                formatter,
                "source attribute argument group {} is invalid",
                group.index()
            ),
            Self::InvalidArgumentGroupSite { group } => write!(
                formatter,
                "source attribute argument group {} has an invalid typed site",
                group.index()
            ),
            Self::InvalidPunctuation { group } => write!(
                formatter,
                "source attribute argument group {} has invalid punctuation",
                group.index()
            ),
            Self::InvalidArgument { argument } => write!(
                formatter,
                "source attribute actual {} is invalid",
                argument.index()
            ),
            Self::InvalidArgumentSite { argument } => write!(
                formatter,
                "source attribute actual {} has an invalid typed site",
                argument.index()
            ),
            Self::InvalidArgumentProvenance { argument } => write!(
                formatter,
                "source attribute actual {} has invalid provenance",
                argument.index()
            ),
            Self::ReorderedAttribute { attribute } => write!(
                formatter,
                "source attribute {} is out of canonical order",
                attribute.index()
            ),
            Self::ReorderedQualifier { qualifier } => write!(
                formatter,
                "source attribute qualifier {} is out of canonical order",
                qualifier.index()
            ),
            Self::ReorderedArgumentGroup { group } => write!(
                formatter,
                "source attribute argument group {} is out of canonical order",
                group.index()
            ),
            Self::ReorderedArgument { argument } => write!(
                formatter,
                "source attribute actual {} is out of canonical order",
                argument.index()
            ),
            Self::DuplicateSite => {
                formatter.write_str("source attribute input repeats a typed site")
            }
            Self::PartialBundle => {
                formatter.write_str("source attribute input is a partial bundle")
            }
        }
    }
}

impl Error for SourceAttributeError {}

fn validate_input(
    input: &SourceAttributeHandoffInput,
    source_type: &SourceTypeApplicationHandoff,
    bindings: &BindingEnv,
    symbols: &SymbolEnv,
    arena: &TypedArena,
) -> Result<(), SourceAttributeError> {
    if input.chains.is_empty() {
        return Err(SourceAttributeError::EmptyChains);
    }
    if input.attributes.is_empty() {
        return Err(SourceAttributeError::EmptyAttributes);
    }
    if source_type.source_id() != input.source_id
        || source_type.module_id() != &input.module_id
        || bindings.source_id() != input.source_id
        || bindings.module_id() != &input.module_id
        || symbols.module_id() != &input.module_id
    {
        return Err(SourceAttributeError::EnvironmentMismatch);
    }
    source_type
        .validate_installation(input.source_id, &input.module_id, arena)
        .map_err(|_| SourceAttributeError::InvalidSourceType)?;
    authenticate_source_type(source_type, bindings)?;

    let mut sites = BTreeSet::new();
    validate_chains(input, source_type, arena, &mut sites)?;
    let attribute_counts = validate_attributes(input, symbols, arena, &mut sites)?;
    if attribute_counts.contains(&0) {
        return Err(SourceAttributeError::PartialBundle);
    }
    let qualifiers = validate_qualifiers(input, symbols, arena, &mut sites)?;
    let group_actual_counts = count_group_actuals(input)?;
    let group_kinds = validate_groups(input, &group_actual_counts, arena, &mut sites)?;
    validate_arguments(input, &group_kinds, arena, &mut sites)?;
    validate_argument_punctuation_order(input)?;
    validate_component_order(input, &qualifiers)?;
    validate_compositional_spellings(input)?;
    Ok(())
}

fn authenticate_source_type(
    source_type: &SourceTypeApplicationHandoff,
    bindings: &BindingEnv,
) -> Result<(), SourceAttributeError> {
    if source_type.applications().len() != bindings.bindings().len()
        || !bindings.diagnostics().is_empty()
    {
        return Err(SourceAttributeError::InvalidSourceType);
    }
    for (_, application) in source_type.applications().iter() {
        let expression = source_type
            .expressions()
            .get(application.root())
            .ok_or(SourceAttributeError::InvalidSourceType)?;
        let binding = bindings
            .bindings()
            .get(application.binding())
            .ok_or(SourceAttributeError::InvalidSourceType)?;
        if binding.id != application.binding()
            || binding.visible_after_ordinal != application.source_ordinal()
            || binding.type_site != BindingTypeSite::Source(expression.source_range())
        {
            return Err(SourceAttributeError::InvalidSourceType);
        }
    }
    Ok(())
}

fn validate_chains(
    input: &SourceAttributeHandoffInput,
    source_type: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceAttributeError> {
    let mut previous_expression = None;
    let mut previous_range = None;
    for (index, chain) in input.chains.iter().enumerate() {
        let id = SourceAttributeChainId::new(index);
        let expression = source_type
            .expressions()
            .get(chain.expression)
            .ok_or(SourceAttributeError::InvalidChain { chain: id })?;
        if chain.source_ordinal != index
            || previous_expression.is_some_and(|previous| chain.expression <= previous)
            || previous_range.is_some_and(|range: SourceRange| range.end > chain.source_range.start)
            || chain.site != *expression.site()
            || chain.source_range != expression.source_range()
            || chain.spelling != expression.spelling()
            || chain.recovery != expression.recovery()
            || chain.recovery != NodeRecoveryState::Normal
        {
            return Err(SourceAttributeError::InvalidChain { chain: id });
        }
        validate_site(
            &chain.site,
            chain.source_range,
            chain.recovery,
            arena,
            SourceAttributeError::InvalidChainSite { chain: id },
        )?;
        if !sites.insert(chain.site.clone()) {
            return Err(SourceAttributeError::DuplicateSite);
        }
        previous_expression = Some(chain.expression);
        previous_range = Some(chain.source_range);
    }
    Ok(())
}

fn validate_attributes(
    input: &SourceAttributeHandoffInput,
    symbols: &SymbolEnv,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<usize>, SourceAttributeError> {
    let mut counts = vec![0; input.chains.len()];
    let mut expected_chain = 0;
    let mut expected_ordinal = 0;
    let mut previous_range = None;
    for (index, attribute) in input.attributes.iter().enumerate() {
        let id = SourceAttributeId::new(index);
        let Some(chain) = input.chains.get(attribute.chain.index()) else {
            return Err(SourceAttributeError::InvalidAttribute { attribute: id });
        };
        if attribute.chain.index() < expected_chain {
            return Err(SourceAttributeError::ReorderedAttribute { attribute: id });
        }
        if attribute.chain.index() > expected_chain {
            if attribute.chain.index() != expected_chain + 1 {
                return Err(SourceAttributeError::ReorderedAttribute { attribute: id });
            }
            expected_chain = attribute.chain.index();
            expected_ordinal = 0;
            previous_range = None;
        }
        if attribute.ordinal != expected_ordinal
            || !valid_range(input.source_id, attribute.source_range)
            || !valid_range(input.source_id, attribute.target_range)
            || !range_contains(chain.source_range, attribute.source_range)
            || !range_contains(attribute.source_range, attribute.target_range)
            || previous_range
                .is_some_and(|range: SourceRange| range.end > attribute.source_range.start)
            || attribute.spelling.trim().is_empty()
            || attribute.target_spelling.trim().is_empty()
            || attribute.site == attribute.target_site
            || attribute.recovery != NodeRecoveryState::Normal
        {
            return Err(SourceAttributeError::InvalidAttribute { attribute: id });
        }
        validate_attribute_sites_input(id, attribute, arena)?;
        for site in [&attribute.site, &attribute.target_site] {
            if !sites.insert(site.clone()) {
                return Err(SourceAttributeError::DuplicateSite);
            }
        }
        validate_symbol(
            input,
            symbols,
            &attribute.symbol,
            attribute.contribution,
            SymbolKind::Attribute,
            &attribute.target_spelling,
            attribute.target_range,
        )
        .map_err(|_| SourceAttributeError::InvalidAttributeSymbol { attribute: id })?;
        validate_polarity(input, id, attribute, arena, sites)?;
        counts[attribute.chain.index()] += 1;
        expected_ordinal += 1;
        previous_range = Some(attribute.source_range);
    }
    Ok(counts)
}

fn validate_attribute_sites_input(
    id: SourceAttributeId,
    attribute: &SourceAttributeInput,
    arena: &TypedArena,
) -> Result<(), SourceAttributeError> {
    validate_site(
        &attribute.site,
        attribute.source_range,
        attribute.recovery,
        arena,
        SourceAttributeError::InvalidAttributeSite { attribute: id },
    )?;
    validate_site(
        &attribute.target_site,
        attribute.target_range,
        attribute.recovery,
        arena,
        SourceAttributeError::InvalidAttributeSite { attribute: id },
    )
}

fn validate_attribute_sites(
    id: SourceAttributeId,
    attribute: &SourceAttribute,
    arena: &TypedArena,
) -> Result<(), SourceAttributeError> {
    validate_site(
        &attribute.site,
        attribute.source_range,
        attribute.recovery,
        arena,
        SourceAttributeError::InvalidAttributeSite { attribute: id },
    )?;
    validate_site(
        &attribute.target_site,
        attribute.target_range,
        attribute.recovery,
        arena,
        SourceAttributeError::InvalidAttributeSite { attribute: id },
    )?;
    if let SourceAttributePolarityInput::Negative {
        non_site,
        non_range,
        non_recovery,
        ..
    } = &attribute.polarity
    {
        validate_site(
            non_site,
            *non_range,
            *non_recovery,
            arena,
            SourceAttributeError::InvalidAttributeSite { attribute: id },
        )?;
    }
    Ok(())
}

fn validate_polarity(
    input: &SourceAttributeHandoffInput,
    id: SourceAttributeId,
    attribute: &SourceAttributeInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceAttributeError> {
    let SourceAttributePolarityInput::Negative {
        non_site,
        non_range,
        non_spelling,
        non_recovery,
    } = &attribute.polarity
    else {
        return Ok(());
    };
    if !valid_range(input.source_id, *non_range)
        || !range_contains(attribute.source_range, *non_range)
        || non_range.end > attribute.target_range.start
        || non_spelling != "non"
        || *non_recovery != attribute.recovery
        || *non_recovery != NodeRecoveryState::Normal
        || !sites.insert(non_site.clone())
    {
        return Err(SourceAttributeError::InvalidPolarity { attribute: id });
    }
    validate_site(
        non_site,
        *non_range,
        *non_recovery,
        arena,
        SourceAttributeError::InvalidPolarity { attribute: id },
    )
}

fn validate_qualifiers(
    input: &SourceAttributeHandoffInput,
    symbols: &SymbolEnv,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Option<SourceAttributeQualifierId>>, SourceAttributeError> {
    let mut by_attribute = vec![None; input.attributes.len()];
    let mut previous_attribute = None;
    for (index, qualifier) in input.qualifiers.iter().enumerate() {
        let id = SourceAttributeQualifierId::new(index);
        let Some(attribute) = input.attributes.get(qualifier.attribute.index()) else {
            return Err(SourceAttributeError::InvalidQualifier { qualifier: id });
        };
        if previous_attribute.is_some_and(|previous| qualifier.attribute <= previous)
            || by_attribute[qualifier.attribute.index()]
                .replace(id)
                .is_some()
        {
            return Err(SourceAttributeError::ReorderedQualifier { qualifier: id });
        }
        let Some(structure_spelling) = qualifier.spelling.strip_suffix('.') else {
            return Err(SourceAttributeError::InvalidQualifier { qualifier: id });
        };
        if structure_spelling.trim().is_empty()
            || !valid_range(input.source_id, qualifier.source_range)
            || !range_contains(attribute.source_range, qualifier.source_range)
            || qualifier.source_range.end > attribute.target_range.start
            || qualifier.recovery != NodeRecoveryState::Normal
            || !sites.insert(qualifier.site.clone())
        {
            return Err(SourceAttributeError::InvalidQualifier { qualifier: id });
        }
        validate_site(
            &qualifier.site,
            qualifier.source_range,
            qualifier.recovery,
            arena,
            SourceAttributeError::InvalidQualifierSite { qualifier: id },
        )?;
        validate_symbol(
            input,
            symbols,
            &qualifier.structure,
            qualifier.contribution,
            SymbolKind::Structure,
            structure_spelling,
            qualifier.source_range,
        )
        .map_err(|_| SourceAttributeError::InvalidQualifierSymbol { qualifier: id })?;
        previous_attribute = Some(qualifier.attribute);
    }
    Ok(by_attribute)
}

fn count_group_actuals(
    input: &SourceAttributeHandoffInput,
) -> Result<Vec<usize>, SourceAttributeError> {
    let mut counts = vec![0; input.argument_groups.len()];
    for (index, argument) in input.arguments.iter().enumerate() {
        let id = SourceAttributeArgumentId::new(index);
        let Some(count) = counts.get_mut(argument.group.index()) else {
            return Err(SourceAttributeError::InvalidArgument { argument: id });
        };
        *count += 1;
    }
    Ok(counts)
}

fn validate_groups(
    input: &SourceAttributeHandoffInput,
    actual_counts: &[usize],
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<SourceAttributeArgumentGroupKind>, SourceAttributeError> {
    let mut kinds = Vec::with_capacity(input.argument_groups.len());
    let mut expected_attribute = 0;
    let mut expected_ordinal = 0;
    let mut seen_prefix = false;
    let mut seen_list = false;
    for (index, group) in input.argument_groups.iter().enumerate() {
        let id = SourceAttributeArgumentGroupId::new(index);
        let Some(attribute) = input.attributes.get(group.attribute.index()) else {
            return Err(SourceAttributeError::InvalidArgumentGroup { group: id });
        };
        if group.attribute.index() < expected_attribute {
            return Err(SourceAttributeError::ReorderedArgumentGroup { group: id });
        }
        if group.attribute.index() > expected_attribute {
            expected_attribute = group.attribute.index();
            expected_ordinal = 0;
            seen_prefix = false;
            seen_list = false;
        }
        if group.ordinal != expected_ordinal
            || !valid_range(input.source_id, group.source_range)
            || !range_contains(attribute.source_range, group.source_range)
            || group.spelling.trim().is_empty()
            || group.recovery != NodeRecoveryState::Normal
            || !sites.insert(group.site.clone())
        {
            return Err(SourceAttributeError::InvalidArgumentGroup { group: id });
        }
        match group.kind {
            SourceAttributeArgumentGroupKind::Prefix => {
                if seen_prefix || seen_list || group.source_range.end > attribute.target_range.start
                {
                    return Err(SourceAttributeError::InvalidArgumentGroup { group: id });
                }
                seen_prefix = true;
            }
            SourceAttributeArgumentGroupKind::ParenthesizedArgumentList => {
                if seen_list || group.source_range.start < attribute.target_range.end {
                    return Err(SourceAttributeError::InvalidArgumentGroup { group: id });
                }
                seen_list = true;
            }
        }
        validate_site(
            &group.site,
            group.source_range,
            group.recovery,
            arena,
            SourceAttributeError::InvalidArgumentGroupSite { group: id },
        )?;
        validate_group_punctuation(input.source_id, id, group, actual_counts[index])?;
        kinds.push(group.kind);
        expected_ordinal += 1;
    }
    Ok(kinds)
}

fn validate_group_punctuation(
    source_id: SourceId,
    id: SourceAttributeArgumentGroupId,
    group: &SourceAttributeArgumentGroupInput,
    actual_count: usize,
) -> Result<(), SourceAttributeError> {
    let punctuation = || SourceAttributeError::InvalidPunctuation { group: id };
    if actual_count == 0
        || group.comma_ranges.len() != group.comma_spellings.len()
        || group.comma_ranges.len() != actual_count.saturating_sub(1)
        || group.comma_spellings.iter().any(|spelling| spelling != ",")
        || group
            .comma_ranges
            .iter()
            .any(|range| !valid_punctuation(source_id, *range, group.source_range))
    {
        return Err(punctuation());
    }
    match (group.kind, group.prefix_form) {
        (SourceAttributeArgumentGroupKind::Prefix, Some(SourceAttributePrefixForm::Single)) => {
            if actual_count != 1
                || !valid_token_pair(
                    source_id,
                    group.source_range,
                    group.hyphen_range,
                    group.hyphen_spelling.as_deref(),
                    "-",
                )
                || group.open_range.is_some()
                || group.open_spelling.is_some()
                || group.close_range.is_some()
                || group.close_spelling.is_some()
                || !group.comma_ranges.is_empty()
            {
                return Err(punctuation());
            }
        }
        (
            SourceAttributeArgumentGroupKind::Prefix,
            Some(SourceAttributePrefixForm::Parenthesized),
        ) => {
            if !valid_token_pair(
                source_id,
                group.source_range,
                group.hyphen_range,
                group.hyphen_spelling.as_deref(),
                "-",
            ) || !valid_parentheses(source_id, group)
            {
                return Err(punctuation());
            }
        }
        (SourceAttributeArgumentGroupKind::ParenthesizedArgumentList, None) => {
            if group.hyphen_range.is_some()
                || group.hyphen_spelling.is_some()
                || !valid_parentheses(source_id, group)
            {
                return Err(punctuation());
            }
        }
        _ => return Err(punctuation()),
    }
    Ok(())
}

fn valid_parentheses(source_id: SourceId, group: &SourceAttributeArgumentGroupInput) -> bool {
    valid_token_pair(
        source_id,
        group.source_range,
        group.open_range,
        group.open_spelling.as_deref(),
        "(",
    ) && valid_token_pair(
        source_id,
        group.source_range,
        group.close_range,
        group.close_spelling.as_deref(),
        ")",
    ) && group
        .open_range
        .zip(group.close_range)
        .is_some_and(|(open, close)| open.end <= close.start)
}

fn valid_token_pair(
    source_id: SourceId,
    parent: SourceRange,
    range: Option<SourceRange>,
    spelling: Option<&str>,
    expected: &str,
) -> bool {
    range.is_some_and(|range| valid_punctuation(source_id, range, parent))
        && spelling == Some(expected)
}

fn valid_punctuation(source_id: SourceId, range: SourceRange, parent: SourceRange) -> bool {
    valid_range(source_id, range) && range.end == range.start + 1 && range_contains(parent, range)
}

fn validate_arguments(
    input: &SourceAttributeHandoffInput,
    group_kinds: &[SourceAttributeArgumentGroupKind],
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceAttributeError> {
    let mut expected_group = 0;
    let mut expected_ordinal = 0;
    let mut previous_range = None;
    for (index, argument) in input.arguments.iter().enumerate() {
        let id = SourceAttributeArgumentId::new(index);
        let Some(group) = input.argument_groups.get(argument.group.index()) else {
            return Err(SourceAttributeError::InvalidArgument { argument: id });
        };
        if argument.group.index() < expected_group {
            return Err(SourceAttributeError::ReorderedArgument { argument: id });
        }
        if argument.group.index() > expected_group {
            if argument.group.index() != expected_group + 1 {
                return Err(SourceAttributeError::ReorderedArgument { argument: id });
            }
            expected_group = argument.group.index();
            expected_ordinal = 0;
            previous_range = None;
        }
        if argument.ordinal != expected_ordinal
            || !valid_range(input.source_id, argument.source_range)
            || !range_contains(group.source_range, argument.source_range)
            || previous_range
                .is_some_and(|range: SourceRange| range.end > argument.source_range.start)
            || argument.spelling.trim().is_empty()
            || argument.recovery != NodeRecoveryState::Normal
            || !sites.insert(argument.site.clone())
        {
            return Err(SourceAttributeError::InvalidArgument { argument: id });
        }
        match group_kinds[argument.group.index()] {
            SourceAttributeArgumentGroupKind::Prefix
                if !matches!(
                    argument.kind,
                    SourceAttributeActualKind::PrefixIdentifier
                        | SourceAttributeActualKind::PrefixNumeral
                ) =>
            {
                return Err(SourceAttributeError::InvalidArgument { argument: id });
            }
            SourceAttributeArgumentGroupKind::ParenthesizedArgumentList
                if argument.kind != SourceAttributeActualKind::TermSite =>
            {
                return Err(SourceAttributeError::InvalidArgument { argument: id });
            }
            _ => {}
        }
        if (argument.kind == SourceAttributeActualKind::PrefixIdentifier
            && !valid_prefix_identifier(&argument.spelling))
            || (argument.kind == SourceAttributeActualKind::PrefixNumeral
                && !argument.spelling.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return Err(SourceAttributeError::InvalidArgument { argument: id });
        }
        validate_site(
            &argument.site,
            argument.source_range,
            argument.recovery,
            arena,
            SourceAttributeError::InvalidArgumentSite { argument: id },
        )?;
        let group_index = u32::try_from(argument.group.index())
            .map_err(|_| SourceAttributeError::InvalidArgumentProvenance { argument: id })?;
        let ordinal = u32::try_from(argument.ordinal)
            .map_err(|_| SourceAttributeError::InvalidArgumentProvenance { argument: id })?;
        if argument.provenance.source_id() != input.source_id
            || argument.provenance.module_id() != &input.module_id
            || argument.provenance.anchor() != &SourceAnchor::Range(argument.source_range)
            || argument.provenance.structural_path() != [group_index, ordinal]
            || argument.provenance.import_edge().is_some()
            || argument.provenance.is_recovered()
        {
            return Err(SourceAttributeError::InvalidArgumentProvenance { argument: id });
        }
        expected_ordinal += 1;
        previous_range = Some(argument.source_range);
    }
    Ok(())
}

fn validate_argument_punctuation_order(
    input: &SourceAttributeHandoffInput,
) -> Result<(), SourceAttributeError> {
    for (group_index, group) in input.argument_groups.iter().enumerate() {
        let group_id = SourceAttributeArgumentGroupId::new(group_index);
        let arguments = input
            .arguments
            .iter()
            .filter(|argument| argument.group == group_id)
            .collect::<Vec<_>>();
        let invalid = || SourceAttributeError::InvalidPunctuation { group: group_id };
        match (group.kind, group.prefix_form) {
            (SourceAttributeArgumentGroupKind::Prefix, Some(SourceAttributePrefixForm::Single)) => {
                if arguments[0].source_range.end
                    > group.hyphen_range.expect("validated punctuation").start
                {
                    return Err(invalid());
                }
            }
            (
                SourceAttributeArgumentGroupKind::Prefix,
                Some(SourceAttributePrefixForm::Parenthesized),
            )
            | (SourceAttributeArgumentGroupKind::ParenthesizedArgumentList, None) => {
                let open = group.open_range.expect("validated punctuation");
                let close = group.close_range.expect("validated punctuation");
                if open.end > arguments[0].source_range.start
                    || arguments
                        .last()
                        .is_some_and(|argument| argument.source_range.end > close.start)
                {
                    return Err(invalid());
                }
                for (index, comma) in group.comma_ranges.iter().enumerate() {
                    if arguments[index].source_range.end > comma.start
                        || comma.end > arguments[index + 1].source_range.start
                    {
                        return Err(invalid());
                    }
                }
                if group.kind == SourceAttributeArgumentGroupKind::Prefix
                    && close.end > group.hyphen_range.expect("validated punctuation").start
                {
                    return Err(invalid());
                }
            }
            _ => return Err(invalid()),
        }
    }
    Ok(())
}

fn validate_component_order(
    input: &SourceAttributeHandoffInput,
    qualifiers: &[Option<SourceAttributeQualifierId>],
) -> Result<(), SourceAttributeError> {
    for (attribute_index, attribute) in input.attributes.iter().enumerate() {
        let prefix = input.argument_groups.iter().find(|group| {
            group.attribute.index() == attribute_index
                && group.kind == SourceAttributeArgumentGroupKind::Prefix
        });
        let qualifier = qualifiers[attribute_index].and_then(|id| input.qualifiers.get(id.index()));
        let earliest_after_non = prefix
            .map(|group| group.source_range.start)
            .or_else(|| qualifier.map(|row| row.source_range.start))
            .unwrap_or(attribute.target_range.start);
        if let SourceAttributePolarityInput::Negative { non_range, .. } = &attribute.polarity
            && non_range.end > earliest_after_non
        {
            return Err(SourceAttributeError::InvalidPolarity {
                attribute: SourceAttributeId::new(attribute_index),
            });
        }
        if let (Some(prefix), Some(qualifier)) = (prefix, qualifier)
            && prefix.source_range.end > qualifier.source_range.start
        {
            return Err(SourceAttributeError::InvalidArgumentGroup {
                group: SourceAttributeArgumentGroupId::new(
                    input
                        .argument_groups
                        .iter()
                        .position(|candidate| std::ptr::eq(candidate, prefix))
                        .expect("borrowed from input"),
                ),
            });
        }
    }
    Ok(())
}

fn validate_compositional_spellings(
    input: &SourceAttributeHandoffInput,
) -> Result<(), SourceAttributeError> {
    for (group_index, group) in input.argument_groups.iter().enumerate() {
        let group_id = SourceAttributeArgumentGroupId::new(group_index);
        let arguments = input
            .arguments
            .iter()
            .filter(|argument| argument.group == group_id)
            .collect::<Vec<_>>();
        let mut expected = Vec::new();
        match (group.kind, group.prefix_form) {
            (SourceAttributeArgumentGroupKind::Prefix, Some(SourceAttributePrefixForm::Single)) => {
                extend_spelling_atoms(&mut expected, &arguments[0].spelling);
                extend_spelling_atoms(
                    &mut expected,
                    group.hyphen_spelling.as_deref().expect("validated hyphen"),
                );
            }
            (
                SourceAttributeArgumentGroupKind::Prefix,
                Some(SourceAttributePrefixForm::Parenthesized),
            )
            | (SourceAttributeArgumentGroupKind::ParenthesizedArgumentList, None) => {
                extend_spelling_atoms(
                    &mut expected,
                    group.open_spelling.as_deref().expect("validated open"),
                );
                for (index, argument) in arguments.iter().enumerate() {
                    extend_spelling_atoms(&mut expected, &argument.spelling);
                    if let Some(comma) = group.comma_spellings.get(index) {
                        extend_spelling_atoms(&mut expected, comma);
                    }
                }
                extend_spelling_atoms(
                    &mut expected,
                    group.close_spelling.as_deref().expect("validated close"),
                );
                if group.kind == SourceAttributeArgumentGroupKind::Prefix {
                    extend_spelling_atoms(
                        &mut expected,
                        group.hyphen_spelling.as_deref().expect("validated hyphen"),
                    );
                }
            }
            _ => {
                return Err(SourceAttributeError::InvalidArgumentGroup { group: group_id });
            }
        }
        if spelling_atoms(&group.spelling) != expected {
            return Err(SourceAttributeError::InvalidArgumentGroup { group: group_id });
        }
    }

    for (attribute_index, attribute) in input.attributes.iter().enumerate() {
        let attribute_id = SourceAttributeId::new(attribute_index);
        let mut expected = Vec::new();
        if let SourceAttributePolarityInput::Negative { non_spelling, .. } = &attribute.polarity {
            extend_spelling_atoms(&mut expected, non_spelling);
        }
        if let Some(prefix) = input.argument_groups.iter().find(|group| {
            group.attribute == attribute_id
                && group.kind == SourceAttributeArgumentGroupKind::Prefix
        }) {
            extend_spelling_atoms(&mut expected, &prefix.spelling);
        }
        if let Some(qualifier) = input
            .qualifiers
            .iter()
            .find(|qualifier| qualifier.attribute == attribute_id)
        {
            extend_spelling_atoms(&mut expected, &qualifier.spelling);
        }
        extend_spelling_atoms(&mut expected, &attribute.target_spelling);
        if let Some(argument_list) = input.argument_groups.iter().find(|group| {
            group.attribute == attribute_id
                && group.kind == SourceAttributeArgumentGroupKind::ParenthesizedArgumentList
        }) {
            extend_spelling_atoms(&mut expected, &argument_list.spelling);
        }
        if spelling_atoms(&attribute.spelling) != expected {
            return Err(SourceAttributeError::InvalidAttribute {
                attribute: attribute_id,
            });
        }
    }

    for (chain_index, chain) in input.chains.iter().enumerate() {
        let chain_id = SourceAttributeChainId::new(chain_index);
        let mut expected_prefix = Vec::new();
        let mut first_attribute = None;
        for (attribute_index, attribute) in input.attributes.iter().enumerate() {
            if attribute.chain != chain_id {
                continue;
            }
            first_attribute.get_or_insert(SourceAttributeId::new(attribute_index));
            extend_spelling_atoms(&mut expected_prefix, &attribute.spelling);
        }
        if !spelling_atoms(&chain.spelling).starts_with(&expected_prefix) {
            return Err(SourceAttributeError::InvalidAttribute {
                attribute: first_attribute.expect("validated nonempty chain"),
            });
        }
    }
    Ok(())
}

fn spelling_atoms(spelling: &str) -> Vec<String> {
    let mut atoms = Vec::new();
    extend_spelling_atoms(&mut atoms, spelling);
    atoms
}

fn extend_spelling_atoms(atoms: &mut Vec<String>, spelling: &str) {
    let mut word = String::new();
    for character in spelling.chars() {
        if character.is_alphanumeric() || character == '_' {
            word.push(character);
            continue;
        }
        if !word.is_empty() {
            atoms.push(std::mem::take(&mut word));
        }
        if !character.is_whitespace() {
            atoms.push(character.to_string());
        }
    }
    if !word.is_empty() {
        atoms.push(word);
    }
}

fn validate_symbol(
    input: &SourceAttributeHandoffInput,
    symbols: &SymbolEnv,
    symbol: &SymbolId,
    contribution_id: SourceContributionId,
    role: SymbolKind,
    spelling: &str,
    use_range: SourceRange,
) -> Result<(), ()> {
    let entry = symbols.symbols().get(symbol).ok_or(())?;
    let contribution = symbols.contributions().get(contribution_id).ok_or(())?;
    if entry.contribution() != contribution_id
        || entry.kind() != role
        || entry.primary_spelling() != spelling
        || entry.namespace().as_str() != input.module_id.path().as_str()
        || contribution.module() != symbol.module()
        || !contribution.effects().symbols().contains(symbol)
        || entry.origin().is_recovered()
    {
        return Err(());
    }
    if symbol.module() == &input.module_id {
        let origin_range = source_range(entry.origin().anchor()).ok_or(())?;
        if !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == input.source_id
        ) || contribution.module() != &input.module_id
            || entry.origin().source_id() != input.source_id
            || entry.origin().module_id() != &input.module_id
            || entry.origin().import_edge().is_some()
            || !valid_range(input.source_id, origin_range)
            || origin_range.end > use_range.start
        {
            return Err(());
        }
    } else {
        let contribution_range = source_range(contribution.anchor()).ok_or(())?;
        let import_is_authenticated = contribution.effects().imports().iter().any(|import| {
            symbols
                .imports()
                .get(*import)
                .and_then(|entry| entry.module())
                == Some(symbol.module())
        });
        if !matches!(
            contribution.kind(),
            ContributionKind::ImportedSource { source_id } if *source_id == input.source_id
        ) || !imported_access_is_visible(entry.visibility(), entry.export_status())
            || !valid_range(input.source_id, contribution_range)
            || contribution_range.end > use_range.start
            || entry.origin().module_id() != symbol.module()
            || !import_is_authenticated
        {
            return Err(());
        }
    }
    Ok(())
}

fn imported_access_is_visible(visibility: Visibility, export_status: ExportStatus) -> bool {
    visibility == Visibility::Public
        && matches!(
            export_status,
            ExportStatus::Exported | ExportStatus::ReExported
        )
}

fn validate_site(
    site: &TypedSiteRef,
    range: SourceRange,
    recovery: NodeRecoveryState,
    arena: &TypedArena,
    error: SourceAttributeError,
) -> Result<(), SourceAttributeError> {
    let Some(node) = arena.node(site.node()) else {
        return Err(error);
    };
    let Some(anchor) = source_range(&node.anchor) else {
        return Err(error);
    };
    if node.recovery != recovery || !range_contains(anchor, range) {
        return Err(error);
    }
    Ok(())
}

fn valid_prefix_identifier(spelling: &str) -> bool {
    let mut chars = spelling.chars();
    chars
        .next()
        .is_some_and(|character| character.is_alphabetic() || character == '_')
        && chars.all(|character| character.is_alphanumeric() || character == '_')
}

fn valid_range(source_id: SourceId, range: SourceRange) -> bool {
    range.source_id == source_id && range.start < range.end
}

fn range_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start <= child.start && child.end <= parent.end
}

fn source_range(anchor: &SourceAnchor) -> Option<SourceRange> {
    match anchor {
        SourceAnchor::Range(range) => Some(*range),
        SourceAnchor::Point { .. } | SourceAnchor::Generated(_) | _ => None,
    }
}

fn write_site(output: &mut String, site: &TypedSiteRef) {
    match site {
        TypedSiteRef::Node(node) => {
            let _ = write!(output, "node:{}", node.index());
        }
        TypedSiteRef::Role { node, role } => {
            let _ = write!(output, "role:{}:{:?}", node.index(), role);
        }
    }
}

fn recovery_key(recovery: NodeRecoveryState) -> &'static str {
    match recovery {
        NodeRecoveryState::Normal => "normal",
        NodeRecoveryState::Recovered => "recovered",
        NodeRecoveryState::Degraded => "degraded",
    }
}

fn group_kind_key(kind: SourceAttributeArgumentGroupKind) -> &'static str {
    match kind {
        SourceAttributeArgumentGroupKind::Prefix => "prefix",
        SourceAttributeArgumentGroupKind::ParenthesizedArgumentList => "argument-list",
    }
}

fn prefix_form_key(form: SourceAttributePrefixForm) -> &'static str {
    match form {
        SourceAttributePrefixForm::Single => "single",
        SourceAttributePrefixForm::Parenthesized => "parenthesized",
    }
}

fn actual_kind_key(kind: SourceAttributeActualKind) -> &'static str {
    match kind {
        SourceAttributeActualKind::PrefixIdentifier => "prefix-identifier",
        SourceAttributeActualKind::PrefixNumeral => "prefix-numeral",
        SourceAttributeActualKind::TermSite => "term-site",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
            BindingContextOwner, BindingContextRecovery, BindingContextTable,
            BindingDiagnosticTable, BindingDraft, BindingEnvParts, BindingId, BindingKind,
            BindingRecoveryState, BindingStatus, BindingTable, CapturedFreeVariables,
        },
        source_type::{
            SourceTypeApplicationForm, SourceTypeApplicationInput, SourceTypeExpressionInput,
            SourceTypeHandoffInput, SourceTypeHead, SourceTypeProducer,
        },
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactTable, TypeRole, TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode,
            TypedNodeId,
        },
    };
    use mizar_resolve::{
        env::{NamespacePath, SymbolEntry, SymbolEnvIndexes},
        resolved_ast::{FullyQualifiedName, LocalSymbolId},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        source_type: SourceTypeApplicationHandoff,
        bindings: BindingEnv,
        symbols: SymbolEnv,
        input: SourceAttributeHandoffInput,
        arena: TypedArena,
        ranked: SymbolId,
        empty: SymbolId,
        ranked_contribution: SourceContributionId,
        empty_contribution: SourceContributionId,
    }

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "c4".repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn other_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "c4".repeat(32)
        ))
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        allocator.next_source_id(snapshot).expect("first source");
        allocator.next_source_id(snapshot).expect("second source")
    }

    fn module(path: &str) -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new(path))
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn role(node: usize, role: &str) -> TypedSiteRef {
        TypedSiteRef::Role {
            node: TypedNodeId::new(node),
            role: TypeRole::new(role),
        }
    }

    fn binding_env(source: SourceId, module: &ModuleId, type_range: SourceRange) -> BindingEnv {
        binding_env_for_ranges(source, module, &[type_range])
    }

    fn binding_env_for_ranges(
        source: SourceId,
        module: &ModuleId,
        type_ranges: &[SourceRange],
    ) -> BindingEnv {
        let binding_ids = (0..type_ranges.len())
            .map(BindingId::new)
            .collect::<Vec<_>>();
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: binding_ids.clone(),
            visible_bindings: binding_ids,
            recovery: BindingContextRecovery::Normal,
        });
        let mut bindings = BindingTable::new();
        for (ordinal, type_range) in type_ranges.iter().enumerate() {
            let declaration_range = range(source, 5 + ordinal * 2, 6 + ordinal * 2);
            let spelling = format!("x{ordinal}");
            bindings.insert(BindingDraft {
                spelling: spelling.clone(),
                kind: BindingKind::ReservedVariable,
                identity: BinderIdentity::ReservedVariable {
                    spelling,
                    declaration_range,
                },
                owner_context: BindingContextId::new(0),
                declaration_range,
                visible_after_ordinal: ordinal,
                type_site: BindingTypeSite::Source(*type_range),
                status: BindingStatus::Reserved,
                captured: CapturedFreeVariables::default(),
                diagnostics: Vec::new(),
                recovery: BindingRecoveryState::Normal,
            });
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding env")
    }

    fn local_symbol(
        indexes: &mut SymbolEnvIndexes,
        source: SourceId,
        module: &ModuleId,
        spelling: &str,
        kind: SymbolKind,
        ordinal: usize,
    ) -> (SymbolId, SourceContributionId) {
        let origin_range = range(source, ordinal, ordinal + 1);
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(origin_range),
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("{spelling}/{ordinal}")),
            FullyQualifiedName::new(format!("{}::{spelling}/{ordinal}", module.path().as_str())),
        );
        indexes.symbols.insert(SymbolEntry::new(
            symbol.clone(),
            kind,
            NamespacePath::new(module.path().as_str()),
            spelling,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(origin_range),
                vec![ordinal as u32],
            ),
            contribution,
        ));
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        (symbol, contribution)
    }

    fn imported_attribute_env(
        fixture: &Fixture,
        visibility: Visibility,
        export_status: ExportStatus,
    ) -> (SymbolEnv, SymbolId, SourceContributionId) {
        let dependency = module("dependency.attributes");
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            dependency.clone(),
            ContributionKind::ImportedSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 2)),
        );
        let symbol = SymbolId::new(
            dependency.clone(),
            LocalSymbolId::new("empty/imported"),
            FullyQualifiedName::new("dependency.attributes::empty"),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Attribute,
                NamespacePath::new(fixture.module.path().as_str()),
                "empty",
                SemanticOrigin::new(
                    fixture.source,
                    dependency,
                    SourceAnchor::Range(range(fixture.source, 1, 2)),
                    vec![0],
                ),
                contribution,
            )
            .with_visibility(visibility)
            .with_export_status(export_status),
        );
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        (
            SymbolEnv::new(fixture.module.clone(), indexes),
            symbol,
            contribution,
        )
    }

    fn source_type(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        arena: &TypedArena,
    ) -> SourceTypeApplicationHandoff {
        source_type_with_spelling(
            source,
            module,
            bindings,
            arena,
            "p-ranked non Carrier.empty(x) set",
        )
    }

    fn source_type_with_spelling(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        arena: &TypedArena,
        spelling: &str,
    ) -> SourceTypeApplicationHandoff {
        SourceTypeProducer::build(
            SourceTypeHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![SourceTypeApplicationInput {
                    binding: BindingId::new(0),
                    source_ordinal: 0,
                    root: SourceTypeExpressionId::new(0),
                }],
                expressions: vec![SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: role(0, "source-type"),
                    source_range: range(source, 10, 100),
                    spelling: spelling.to_owned(),
                    head_site: role(1, "type-head"),
                    head_range: range(source, 90, 93),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                }],
                arguments: Vec::new(),
            },
            bindings,
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            arena,
        )
        .expect("source type")
    }

    fn arena_for(input: &SourceAttributeHandoffInput) -> TypedArena {
        let mut anchors = vec![None::<SourceRange>; 14];
        let mut record = |site: &TypedSiteRef, value: SourceRange| {
            let entry = &mut anchors[site.node().index()];
            match entry {
                Some(range) => {
                    range.start = range.start.min(value.start);
                    range.end = range.end.max(value.end);
                }
                None => *entry = Some(value),
            }
        };
        record(&role(0, "source-type"), range(input.source_id, 10, 100));
        record(&role(1, "type-head"), range(input.source_id, 90, 93));
        record(&role(13, "type-head-1"), range(input.source_id, 117, 120));
        for chain in &input.chains {
            record(&chain.site, chain.source_range);
        }
        for attribute in &input.attributes {
            record(&attribute.site, attribute.source_range);
            record(&attribute.target_site, attribute.target_range);
            if let SourceAttributePolarityInput::Negative {
                non_site,
                non_range,
                ..
            } = &attribute.polarity
            {
                record(non_site, *non_range);
            }
        }
        for qualifier in &input.qualifiers {
            record(&qualifier.site, qualifier.source_range);
        }
        for group in &input.argument_groups {
            record(&group.site, group.source_range);
        }
        for argument in &input.arguments {
            record(&argument.site, argument.source_range);
        }
        TypedArena::try_new(
            None,
            anchors
                .into_iter()
                .enumerate()
                .map(|(index, anchor)| {
                    TypedNode::new(
                        format!("source-attribute-test-{index}"),
                        SourceAnchor::Range(anchor.unwrap_or_else(|| range(input.source_id, 1, 2))),
                    )
                })
                .collect(),
        )
        .expect("arena")
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = module("source.attribute");
        let mut indexes = SymbolEnvIndexes::default();
        let (ranked, ranked_contribution) = local_symbol(
            &mut indexes,
            source,
            &module,
            "ranked",
            SymbolKind::Attribute,
            1,
        );
        let (empty, empty_contribution) = local_symbol(
            &mut indexes,
            source,
            &module,
            "empty",
            SymbolKind::Attribute,
            2,
        );
        let (structure, structure_contribution) = local_symbol(
            &mut indexes,
            source,
            &module,
            "Carrier",
            SymbolKind::Structure,
            3,
        );
        let symbols = SymbolEnv::new(module.clone(), indexes);
        let input = SourceAttributeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            chains: vec![SourceAttributeChainInput {
                expression: SourceTypeExpressionId::new(0),
                source_ordinal: 0,
                site: role(0, "source-type"),
                source_range: range(source, 10, 100),
                spelling: "p-ranked non Carrier.empty(x) set".to_owned(),
                recovery: NodeRecoveryState::Normal,
            }],
            attributes: vec![
                SourceAttributeInput {
                    chain: SourceAttributeChainId::new(0),
                    ordinal: 0,
                    site: role(2, "attribute-0"),
                    source_range: range(source, 10, 18),
                    spelling: "p-ranked".to_owned(),
                    target_site: role(3, "target-0"),
                    target_range: range(source, 12, 18),
                    target_spelling: "ranked".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    symbol: ranked.clone(),
                    contribution: ranked_contribution,
                    polarity: SourceAttributePolarityInput::Positive,
                },
                SourceAttributeInput {
                    chain: SourceAttributeChainId::new(0),
                    ordinal: 1,
                    site: role(6, "attribute-1"),
                    source_range: range(source, 30, 50),
                    spelling: "non Carrier.empty(x)".to_owned(),
                    target_site: role(9, "target-1"),
                    target_range: range(source, 42, 47),
                    target_spelling: "empty".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    symbol: empty.clone(),
                    contribution: empty_contribution,
                    polarity: SourceAttributePolarityInput::Negative {
                        non_site: role(7, "non-1"),
                        non_range: range(source, 30, 33),
                        non_spelling: "non".to_owned(),
                        non_recovery: NodeRecoveryState::Normal,
                    },
                },
            ],
            qualifiers: vec![SourceAttributeQualifierInput {
                attribute: SourceAttributeId::new(1),
                site: role(8, "qualifier-1"),
                source_range: range(source, 34, 42),
                spelling: "Carrier.".to_owned(),
                recovery: NodeRecoveryState::Normal,
                structure: structure.clone(),
                contribution: structure_contribution,
            }],
            argument_groups: vec![
                SourceAttributeArgumentGroupInput {
                    attribute: SourceAttributeId::new(0),
                    ordinal: 0,
                    kind: SourceAttributeArgumentGroupKind::Prefix,
                    site: role(4, "prefix-0"),
                    source_range: range(source, 10, 12),
                    spelling: "p-".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    prefix_form: Some(SourceAttributePrefixForm::Single),
                    hyphen_range: Some(range(source, 11, 12)),
                    hyphen_spelling: Some("-".to_owned()),
                    open_range: None,
                    open_spelling: None,
                    close_range: None,
                    close_spelling: None,
                    comma_ranges: Vec::new(),
                    comma_spellings: Vec::new(),
                },
                SourceAttributeArgumentGroupInput {
                    attribute: SourceAttributeId::new(1),
                    ordinal: 0,
                    kind: SourceAttributeArgumentGroupKind::ParenthesizedArgumentList,
                    site: role(10, "list-1"),
                    source_range: range(source, 47, 50),
                    spelling: "(x)".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    prefix_form: None,
                    hyphen_range: None,
                    hyphen_spelling: None,
                    open_range: Some(range(source, 47, 48)),
                    open_spelling: Some("(".to_owned()),
                    close_range: Some(range(source, 49, 50)),
                    close_spelling: Some(")".to_owned()),
                    comma_ranges: Vec::new(),
                    comma_spellings: Vec::new(),
                },
            ],
            arguments: vec![
                SourceAttributeArgumentInput {
                    group: SourceAttributeArgumentGroupId::new(0),
                    ordinal: 0,
                    kind: SourceAttributeActualKind::PrefixIdentifier,
                    site: role(5, "actual-0-0"),
                    source_range: range(source, 10, 11),
                    spelling: "p".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    provenance: SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, 10, 11)),
                        vec![0, 0],
                    ),
                },
                SourceAttributeArgumentInput {
                    group: SourceAttributeArgumentGroupId::new(1),
                    ordinal: 0,
                    kind: SourceAttributeActualKind::TermSite,
                    site: role(11, "actual-1-0"),
                    source_range: range(source, 48, 49),
                    spelling: "x".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    provenance: SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, 48, 49)),
                        vec![1, 0],
                    ),
                },
            ],
        };
        let arena = arena_for(&input);
        let bindings = binding_env(source, &module, range(source, 10, 100));
        let source_type = source_type(source, &module, &bindings, &arena);
        Fixture {
            source,
            module,
            source_type,
            bindings,
            symbols,
            input,
            arena,
            ranked,
            empty,
            ranked_contribution,
            empty_contribution,
        }
    }

    fn build(fixture: &Fixture) -> Result<SourceAttributeHandoff, SourceAttributeError> {
        SourceAttributeProducer::build(
            fixture.input.clone(),
            &fixture.source_type,
            &fixture.bindings,
            &fixture.symbols,
            &fixture.arena,
        )
    }

    fn imported_only_fixture(
        fixture: &Fixture,
        visibility: Visibility,
        export_status: ExportStatus,
    ) -> Fixture {
        let (symbols, imported, contribution) =
            imported_attribute_env(fixture, visibility, export_status);
        let mut imported_fixture = fixture.clone();
        imported_fixture.symbols = symbols;
        imported_fixture.input.attributes.remove(0);
        imported_fixture.input.attributes[0].ordinal = 0;
        imported_fixture.input.attributes[0].symbol = imported;
        imported_fixture.input.attributes[0].contribution = contribution;
        imported_fixture.input.qualifiers[0].attribute = SourceAttributeId::new(0);
        imported_fixture.input.argument_groups.remove(0);
        imported_fixture.input.argument_groups[0].attribute = SourceAttributeId::new(0);
        imported_fixture.input.arguments.remove(0);
        imported_fixture.input.arguments[0].group = SourceAttributeArgumentGroupId::new(0);
        imported_fixture.input.arguments[0].provenance = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 48, 49)),
            vec![0, 0],
        );
        imported_fixture
    }

    fn refresh_arena(fixture: &mut Fixture) {
        fixture.arena = arena_for(&fixture.input);
    }

    fn empty_typed_parts(fixture: &Fixture) -> TypedAstParts {
        TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: Some(fixture.source_type.clone()),
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

    #[test]
    fn complete_chain_is_dense_immutable_and_deterministic() {
        let fixture = fixture();
        let first = build(&fixture).expect("handoff");
        let second = build(&fixture).expect("handoff");
        assert_eq!(first, second);
        assert_eq!(first.chains().len(), 1);
        assert_eq!(first.attributes().len(), 2);
        assert_eq!(first.qualifiers().len(), 1);
        assert_eq!(first.argument_groups().len(), 2);
        assert_eq!(first.arguments().len(), 2);
        assert_eq!(
            first
                .chains()
                .get(SourceAttributeChainId::new(0))
                .map(SourceAttributeChain::expression),
            Some(SourceTypeExpressionId::new(0))
        );
        assert_eq!(
            first
                .attributes()
                .get(SourceAttributeId::new(1))
                .map(SourceAttribute::symbol),
            Some(&fixture.empty)
        );
        assert_eq!(first.debug_text(), second.debug_text());
        assert!(
            first
                .debug_text()
                .starts_with("source-attribute-debug-v1\n")
        );
    }

    #[test]
    fn parenthesized_prefix_retains_ordered_identifier_and_numeral_actuals() {
        let mut fixture = fixture();
        let group = &mut fixture.input.argument_groups[0];
        group.source_range = range(fixture.source, 10, 16);
        group.spelling = "(q,2)-".to_owned();
        group.prefix_form = Some(SourceAttributePrefixForm::Parenthesized);
        group.hyphen_range = Some(range(fixture.source, 15, 16));
        group.open_range = Some(range(fixture.source, 10, 11));
        group.open_spelling = Some("(".to_owned());
        group.close_range = Some(range(fixture.source, 14, 15));
        group.close_spelling = Some(")".to_owned());
        group.comma_ranges = vec![range(fixture.source, 12, 13)];
        group.comma_spellings = vec![",".to_owned()];
        let attribute = &mut fixture.input.attributes[0];
        attribute.source_range = range(fixture.source, 10, 22);
        attribute.spelling = "(q,2)-ranked".to_owned();
        attribute.target_range = range(fixture.source, 16, 22);
        fixture.input.arguments[0].source_range = range(fixture.source, 11, 12);
        fixture.input.arguments[0].spelling = "q".to_owned();
        fixture.input.arguments[0].provenance = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 11, 12)),
            vec![0, 0],
        );
        fixture.input.arguments.insert(
            1,
            SourceAttributeArgumentInput {
                group: SourceAttributeArgumentGroupId::new(0),
                ordinal: 1,
                kind: SourceAttributeActualKind::PrefixNumeral,
                site: role(11, "actual-0-1"),
                source_range: range(fixture.source, 13, 14),
                spelling: "2".to_owned(),
                recovery: NodeRecoveryState::Normal,
                provenance: SemanticOrigin::new(
                    fixture.source,
                    fixture.module.clone(),
                    SourceAnchor::Range(range(fixture.source, 13, 14)),
                    vec![0, 1],
                ),
            },
        );
        fixture.input.arguments[2].site = role(11, "actual-1-0");
        refresh_arena(&mut fixture);
        let chain_spelling = "(q,2)-ranked non Carrier.empty(x) set";
        fixture.input.chains[0].spelling = chain_spelling.to_owned();
        fixture.source_type = source_type_with_spelling(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            chain_spelling,
        );

        let handoff = build(&fixture).expect("parenthesized prefix");
        assert_eq!(handoff.arguments().len(), 3);
        assert_eq!(
            handoff
                .argument_groups()
                .get(SourceAttributeArgumentGroupId::new(0))
                .map(SourceAttributeArgumentGroup::prefix_form),
            Some(Some(SourceAttributePrefixForm::Parenthesized))
        );
        assert_eq!(
            handoff
                .arguments()
                .get(SourceAttributeArgumentId::new(1))
                .map(SourceAttributeArgument::kind),
            Some(SourceAttributeActualKind::PrefixNumeral)
        );
    }

    #[test]
    fn chain_attribute_and_polarity_corruption_fail_closed() {
        let fixture = fixture();
        let valid = build(&fixture).expect("baseline");

        let mut corruptions = Vec::new();
        let mut missing_chain = fixture.clone();
        missing_chain.input.chains.clear();
        corruptions.push(missing_chain);
        let mut dangling_expression = fixture.clone();
        dangling_expression.input.chains[0].expression = SourceTypeExpressionId::new(99);
        corruptions.push(dangling_expression);
        let mut wrong_chain_spelling = fixture.clone();
        wrong_chain_spelling.input.chains[0].spelling.push('!');
        corruptions.push(wrong_chain_spelling);
        let mut wrong_chain_ordinal = fixture.clone();
        wrong_chain_ordinal.input.chains[0].source_ordinal = 1;
        corruptions.push(wrong_chain_ordinal);
        let mut wrong_attribute_ordinal = fixture.clone();
        wrong_attribute_ordinal.input.attributes[1].ordinal = 0;
        corruptions.push(wrong_attribute_ordinal);
        let mut dangling_chain = fixture.clone();
        dangling_chain.input.attributes[0].chain = SourceAttributeChainId::new(9);
        corruptions.push(dangling_chain);
        let mut wrong_target = fixture.clone();
        wrong_target.input.attributes[0].target_range = range(fixture.source, 9, 18);
        corruptions.push(wrong_target);
        let mut recovered = fixture.clone();
        recovered.input.attributes[0].recovery = NodeRecoveryState::Recovered;
        corruptions.push(recovered);
        let mut wrong_non = fixture.clone();
        let SourceAttributePolarityInput::Negative { non_spelling, .. } =
            &mut wrong_non.input.attributes[1].polarity
        else {
            panic!("negative");
        };
        *non_spelling = "not".to_owned();
        corruptions.push(wrong_non);
        let mut duplicate_site = fixture.clone();
        duplicate_site.input.attributes[1].target_site =
            duplicate_site.input.attributes[0].target_site.clone();
        corruptions.push(duplicate_site);

        for corrupted in corruptions {
            assert!(build(&corrupted).is_err());
        }
        assert_eq!(build(&fixture), Ok(valid));
    }

    #[test]
    fn chain_without_any_owned_attribute_is_a_partial_bundle() {
        let mut partial = fixture();
        partial.input.chains.push(SourceAttributeChainInput {
            expression: SourceTypeExpressionId::new(1),
            source_ordinal: 1,
            site: role(12, "source-type-1"),
            source_range: range(partial.source, 110, 120),
            spelling: "set".to_owned(),
            recovery: NodeRecoveryState::Normal,
        });
        partial.arena = arena_for(&partial.input);
        partial.bindings = binding_env_for_ranges(
            partial.source,
            &partial.module,
            &[
                range(partial.source, 10, 100),
                range(partial.source, 110, 120),
            ],
        );
        partial.source_type = SourceTypeProducer::build(
            SourceTypeHandoffInput {
                source_id: partial.source,
                module_id: partial.module.clone(),
                applications: vec![
                    SourceTypeApplicationInput {
                        binding: BindingId::new(0),
                        source_ordinal: 0,
                        root: SourceTypeExpressionId::new(0),
                    },
                    SourceTypeApplicationInput {
                        binding: BindingId::new(1),
                        source_ordinal: 1,
                        root: SourceTypeExpressionId::new(1),
                    },
                ],
                expressions: vec![
                    SourceTypeExpressionInput {
                        source_id: partial.source,
                        module_id: partial.module.clone(),
                        site: role(0, "source-type"),
                        source_range: range(partial.source, 10, 100),
                        spelling: "p-ranked non Carrier.empty(x) set".to_owned(),
                        head_site: role(1, "type-head"),
                        head_range: range(partial.source, 90, 93),
                        head_spelling: "set".to_owned(),
                        form: SourceTypeApplicationForm::Bare,
                        head: SourceTypeHead::BuiltinSet,
                        recovery: NodeRecoveryState::Normal,
                    },
                    SourceTypeExpressionInput {
                        source_id: partial.source,
                        module_id: partial.module.clone(),
                        site: role(12, "source-type-1"),
                        source_range: range(partial.source, 110, 120),
                        spelling: "set".to_owned(),
                        head_site: role(13, "type-head-1"),
                        head_range: range(partial.source, 117, 120),
                        head_spelling: "set".to_owned(),
                        form: SourceTypeApplicationForm::Bare,
                        head: SourceTypeHead::BuiltinSet,
                        recovery: NodeRecoveryState::Normal,
                    },
                ],
                arguments: Vec::new(),
            },
            &partial.bindings,
            &partial.symbols,
            &partial.arena,
        )
        .expect("two-expression Task-249 dependency");

        assert_eq!(build(&partial), Err(SourceAttributeError::PartialBundle));
    }

    #[test]
    fn symbol_role_local_order_and_missing_import_closure_are_authenticated() {
        let fixture = fixture();

        let mut wrong_contribution = fixture.clone();
        wrong_contribution.input.attributes[0].contribution = fixture.empty_contribution;
        assert!(matches!(
            build(&wrong_contribution),
            Err(SourceAttributeError::InvalidAttributeSymbol { .. })
        ));

        let mut wrong_role = fixture.clone();
        wrong_role.input.qualifiers[0].structure = fixture.ranked.clone();
        wrong_role.input.qualifiers[0].contribution = fixture.ranked_contribution;
        assert!(matches!(
            build(&wrong_role),
            Err(SourceAttributeError::InvalidQualifierSymbol { .. })
        ));

        let mut after_use_indexes = SymbolEnvIndexes::default();
        let (late, late_contribution) = local_symbol(
            &mut after_use_indexes,
            fixture.source,
            &fixture.module,
            "ranked",
            SymbolKind::Attribute,
            20,
        );
        let mut late_fixture = fixture.clone();
        late_fixture.symbols = SymbolEnv::new(fixture.module.clone(), after_use_indexes);
        late_fixture.input.attributes.truncate(1);
        late_fixture.input.qualifiers.clear();
        late_fixture.input.argument_groups.truncate(1);
        late_fixture.input.arguments.truncate(1);
        late_fixture.input.attributes[0].symbol = late;
        late_fixture.input.attributes[0].contribution = late_contribution;
        assert!(matches!(
            build(&late_fixture),
            Err(SourceAttributeError::InvalidAttributeSymbol { .. })
        ));

        let no_closure =
            imported_only_fixture(&fixture, Visibility::Public, ExportStatus::Exported);
        assert!(matches!(
            build(&no_closure),
            Err(SourceAttributeError::InvalidAttributeSymbol { .. })
        ));
    }

    #[test]
    fn imported_visibility_and_export_status_are_independently_gated() {
        assert!(imported_access_is_visible(
            Visibility::Public,
            ExportStatus::Exported
        ));
        assert!(imported_access_is_visible(
            Visibility::Public,
            ExportStatus::ReExported
        ));
        assert!(!imported_access_is_visible(
            Visibility::Private,
            ExportStatus::Exported
        ));
        assert!(!imported_access_is_visible(
            Visibility::Public,
            ExportStatus::LocalOnly
        ));
    }

    #[test]
    fn qualifier_group_punctuation_and_actual_corruption_are_rejected() {
        let fixture = fixture();

        let mut wrong_qualifier = fixture.clone();
        wrong_qualifier.input.qualifiers[0].spelling = "Carrier".to_owned();
        assert!(matches!(
            build(&wrong_qualifier),
            Err(SourceAttributeError::InvalidQualifier { .. })
        ));

        let mut duplicate_qualifier = fixture.clone();
        duplicate_qualifier
            .input
            .qualifiers
            .push(duplicate_qualifier.input.qualifiers[0].clone());
        assert!(matches!(
            build(&duplicate_qualifier),
            Err(SourceAttributeError::ReorderedQualifier { .. })
        ));

        let mut wrong_kind = fixture.clone();
        wrong_kind.input.argument_groups[0].kind =
            SourceAttributeArgumentGroupKind::ParenthesizedArgumentList;
        assert!(build(&wrong_kind).is_err());

        let mut missing_hyphen = fixture.clone();
        missing_hyphen.input.argument_groups[0].hyphen_range = None;
        assert!(matches!(
            build(&missing_hyphen),
            Err(SourceAttributeError::InvalidPunctuation { .. })
        ));

        let mut wrong_close = fixture.clone();
        wrong_close.input.argument_groups[1].close_spelling = Some("]".to_owned());
        assert!(matches!(
            build(&wrong_close),
            Err(SourceAttributeError::InvalidPunctuation { .. })
        ));

        let mut dangling_group = fixture.clone();
        dangling_group.input.arguments[0].group = SourceAttributeArgumentGroupId::new(99);
        assert!(matches!(
            build(&dangling_group),
            Err(SourceAttributeError::InvalidArgument { .. })
        ));

        let mut wrong_actual_kind = fixture.clone();
        wrong_actual_kind.input.arguments[1].kind = SourceAttributeActualKind::PrefixIdentifier;
        assert!(matches!(
            build(&wrong_actual_kind),
            Err(SourceAttributeError::InvalidArgument { .. })
        ));

        let mut term_prefix = fixture.clone();
        term_prefix.input.arguments[0].kind = SourceAttributeActualKind::TermSite;
        assert!(matches!(
            build(&term_prefix),
            Err(SourceAttributeError::InvalidArgument { .. })
        ));

        let mut stale_group_spelling = fixture.clone();
        stale_group_spelling.input.argument_groups[0].spelling = "q-".to_owned();
        assert!(matches!(
            build(&stale_group_spelling),
            Err(SourceAttributeError::InvalidArgumentGroup { .. })
        ));

        let mut stale_actual_spelling = fixture.clone();
        stale_actual_spelling.input.arguments[0].spelling = "q".to_owned();
        assert!(matches!(
            build(&stale_actual_spelling),
            Err(SourceAttributeError::InvalidArgumentGroup { .. })
        ));

        let mut stale_attribute_spelling = fixture.clone();
        stale_attribute_spelling.input.attributes[0].spelling = "q-ranked".to_owned();
        assert!(matches!(
            build(&stale_attribute_spelling),
            Err(SourceAttributeError::InvalidAttribute { .. })
        ));

        let mut coordinated_stale_spellings = fixture.clone();
        coordinated_stale_spellings.input.arguments[0].spelling = "q".to_owned();
        coordinated_stale_spellings.input.argument_groups[0].spelling = "q-".to_owned();
        coordinated_stale_spellings.input.attributes[0].spelling = "q-ranked".to_owned();
        assert!(matches!(
            build(&coordinated_stale_spellings),
            Err(SourceAttributeError::InvalidAttribute { .. })
        ));

        let mut wrong_origin = fixture.clone();
        wrong_origin.input.arguments[0].provenance = SemanticOrigin::new(
            other_source_id(),
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 10, 11)),
            vec![0, 0],
        );
        assert!(matches!(
            build(&wrong_origin),
            Err(SourceAttributeError::InvalidArgumentProvenance { .. })
        ));

        let mut wrong_order = fixture.clone();
        wrong_order.input.arguments.swap(0, 1);
        assert!(matches!(
            build(&wrong_order),
            Err(SourceAttributeError::ReorderedArgument { .. })
        ));
    }

    #[test]
    fn environment_site_spelling_cardinality_and_origin_matrix_is_rejected() {
        let fixture = fixture();

        let mut wrong_symbol_environment = fixture.clone();
        wrong_symbol_environment.symbols =
            SymbolEnv::new(module("wrong.environment"), SymbolEnvIndexes::default());
        assert_eq!(
            build(&wrong_symbol_environment),
            Err(SourceAttributeError::EnvironmentMismatch)
        );

        let mut wrong_binding_environment = fixture.clone();
        wrong_binding_environment.bindings = binding_env(
            other_source_id(),
            &fixture.module,
            range(other_source_id(), 10, 100),
        );
        assert_eq!(
            build(&wrong_binding_environment),
            Err(SourceAttributeError::EnvironmentMismatch)
        );

        let mut wrong_target_spelling = fixture.clone();
        wrong_target_spelling.input.attributes[0].target_spelling = "rankedX".to_owned();
        assert!(matches!(
            build(&wrong_target_spelling),
            Err(SourceAttributeError::InvalidAttributeSymbol { .. })
        ));

        let mut wrong_source_range = fixture.clone();
        wrong_source_range.input.attributes[0].source_range = range(other_source_id(), 10, 18);
        assert!(matches!(
            build(&wrong_source_range),
            Err(SourceAttributeError::InvalidAttribute { .. })
        ));

        let mut missing_site = fixture.clone();
        missing_site.input.attributes[0].target_site = role(99, "missing");
        assert!(matches!(
            build(&missing_site),
            Err(SourceAttributeError::InvalidAttributeSite { .. })
        ));

        let mut empty_group = fixture.clone();
        empty_group.input.arguments.remove(0);
        empty_group.input.arguments[0].group = SourceAttributeArgumentGroupId::new(1);
        assert!(matches!(
            build(&empty_group),
            Err(SourceAttributeError::InvalidPunctuation { .. })
        ));

        let mut duplicate_group = fixture.clone();
        let mut second_prefix = duplicate_group.input.argument_groups[0].clone();
        second_prefix.ordinal = 1;
        duplicate_group
            .input
            .argument_groups
            .insert(1, second_prefix);
        duplicate_group.input.arguments[1].group = SourceAttributeArgumentGroupId::new(2);
        duplicate_group.input.arguments[1].provenance = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 48, 49)),
            vec![2, 0],
        );
        assert!(matches!(
            build(&duplicate_group),
            Err(SourceAttributeError::InvalidArgumentGroup { .. })
        ));

        let mut recovered_origin = fixture.clone();
        recovered_origin.input.arguments[0].provenance = recovered_origin.input.arguments[0]
            .provenance
            .clone()
            .recovered();
        assert!(matches!(
            build(&recovered_origin),
            Err(SourceAttributeError::InvalidArgumentProvenance { .. })
        ));
    }

    #[test]
    fn typed_ast_owns_handoff_and_legacy_debug_is_conditional() {
        let fixture = fixture();
        let handoff = build(&fixture).expect("handoff");

        let legacy = TypedAst::try_new(empty_typed_parts(&fixture)).expect("legacy typed AST");
        assert!(legacy.source_attribute().is_none());
        assert!(!legacy.debug_text().contains("source-attribute-debug-v1"));

        let mut parts = empty_typed_parts(&fixture);
        parts.source_attribute = Some(handoff.clone());
        let typed = TypedAst::try_new(parts).expect("typed AST with source attributes");
        assert_eq!(typed.source_attribute(), Some(&handoff));
        assert!(typed.debug_text().contains("source-attribute-debug-v1"));

        let mut missing_dependency = empty_typed_parts(&fixture);
        missing_dependency.source_type = None;
        missing_dependency.source_attribute = Some(handoff);
        assert_eq!(
            TypedAst::try_new(missing_dependency),
            Err(TypedAstError::InvalidSourceAttribute)
        );
    }

    #[test]
    fn public_module_stays_syntax_free_and_legacy_attribute_input_is_untouched() {
        let source = include_str!("source_attribute.rs");
        let production = &source[..source.find("#[cfg(test)]").expect("test module")];
        for forbidden in [
            concat!("mizar", "_syntax"),
            concat!("Surface", "Ast"),
            concat!("Surface", "NodeId"),
            concat!("Syntax", "Kind"),
            concat!("evidence", "_request"),
            concat!("cluster", "_fact"),
        ] {
            assert!(
                !production.contains(forbidden),
                "source attribute handoff leaked forbidden boundary token {forbidden}"
            );
        }
    }
}
