//! Syntax-free source type-expression application handoff.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextLayer, BindingContextOwner, BindingContextRecovery,
        BindingEnv, BindingId, BindingKind, BindingRecoveryState, BindingStatus, BindingTypeSite,
    },
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

dense_id!(SourceTypeApplicationId);
dense_id!(SourceTypeExpressionId);
dense_id!(SourceTypeArgumentId);

/// Syntax-free inputs for one complete source type-expression transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub applications: Vec<SourceTypeApplicationInput>,
    pub expressions: Vec<SourceTypeExpressionInput>,
    pub arguments: Vec<SourceTypeArgumentInput>,
}

/// One top-level binding-to-source-type application input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeApplicationInput {
    pub binding: BindingId,
    pub source_ordinal: usize,
    pub root: SourceTypeExpressionId,
}

/// One flat source type-expression input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeExpressionInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub head_site: TypedSiteRef,
    pub head_range: SourceRange,
    pub head_spelling: String,
    pub form: SourceTypeApplicationForm,
    pub head: SourceTypeHead,
    pub recovery: NodeRecoveryState,
}

/// One ordered flat source type argument input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeArgumentInput {
    pub parent: SourceTypeExpressionId,
    pub ordinal: usize,
    pub argument: SourceTypeArgument,
}

/// Source-written type application form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceTypeApplicationForm {
    Bare,
    Of,
    Over,
    Bracket,
}

/// Source-written type head authenticated by built-in identity or `SymbolEnv`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceTypeHead {
    BuiltinSet,
    BuiltinObject,
    Symbol {
        symbol: SymbolId,
        contribution: SourceContributionId,
    },
}

/// Source-written argument payload. Term and qua sites intentionally carry no
/// checker `BindingId`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceTypeArgument {
    TermSite {
        site: TypedSiteRef,
        source_range: SourceRange,
        spelling: String,
        recovery: NodeRecoveryState,
        provenance: SemanticOrigin,
    },
    TypeSite {
        expression: SourceTypeExpressionId,
    },
    QuaSite {
        site: TypedSiteRef,
        source_range: SourceRange,
        spelling: String,
        recovery: NodeRecoveryState,
        provenance: SemanticOrigin,
        radix: Vec<SourceTypeExpressionId>,
    },
}

/// Immutable validated source type-expression handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeApplicationHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    applications: SourceTypeApplicationTable,
    expressions: SourceTypeExpressionTable,
    arguments: SourceTypeArgumentTable,
}

impl SourceTypeApplicationHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn applications(&self) -> &SourceTypeApplicationTable {
        &self.applications
    }

    pub const fn expressions(&self) -> &SourceTypeExpressionTable {
        &self.expressions
    }

    pub const fn arguments(&self) -> &SourceTypeArgumentTable {
        &self.arguments
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-type-application-debug-v1\n");
        output.push_str("module: ");
        output.push_str(self.module_id.path().as_str());
        output.push('\n');
        for (id, application) in self.applications.iter() {
            let _ = writeln!(
                output,
                "application#{} binding={} ordinal={} root={}",
                id.index(),
                application.binding.index(),
                application.source_ordinal,
                application.root.index(),
            );
        }
        for (id, expression) in self.expressions.iter() {
            let _ = write!(
                output,
                "expression#{} form={} range={}..{} site=",
                id.index(),
                form_key(expression.form),
                expression.source_range.start,
                expression.source_range.end,
            );
            write_site(&mut output, &expression.site);
            output.push_str(" head=");
            write_head(&mut output, &expression.head);
            output.push_str(" head_range=");
            let _ = write!(
                output,
                "{}..{} head_site=",
                expression.head_range.start, expression.head_range.end
            );
            write_site(&mut output, &expression.head_site);
            let _ = writeln!(
                output,
                " recovery={} spelling={:?} head_spelling={:?}",
                recovery_key(expression.recovery),
                expression.spelling,
                expression.head_spelling,
            );
        }
        for (id, argument) in self.arguments.iter() {
            let _ = write!(
                output,
                "argument#{} parent={} ordinal={} ",
                id.index(),
                argument.parent.index(),
                argument.ordinal,
            );
            write_argument(&mut output, &argument.argument);
            output.push('\n');
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceTypeError> {
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceTypeError::EnvironmentMismatch);
        }
        for (id, expression) in self.expressions.iter() {
            validate_arena_site(
                id,
                &expression.site,
                expression.source_range,
                expression.recovery,
                arena,
            )?;
            validate_arena_site(
                id,
                &expression.head_site,
                expression.head_range,
                expression.recovery,
                arena,
            )?;
        }
        for (id, argument) in self.arguments.iter() {
            match &argument.argument {
                SourceTypeArgument::TermSite {
                    site,
                    source_range,
                    recovery,
                    ..
                }
                | SourceTypeArgument::QuaSite {
                    site,
                    source_range,
                    recovery,
                    ..
                } => validate_argument_arena_site(id, site, *source_range, *recovery, arena)?,
                SourceTypeArgument::TypeSite { .. } => {}
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceTypeApplicationTable {
    entries: Vec<SourceTypeApplication>,
}

impl SourceTypeApplicationTable {
    pub fn get(&self, id: SourceTypeApplicationId) -> Option<&SourceTypeApplication> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceTypeApplicationId, &SourceTypeApplication)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeApplication {
    id: SourceTypeApplicationId,
    binding: BindingId,
    source_ordinal: usize,
    root: SourceTypeExpressionId,
}

impl SourceTypeApplication {
    pub const fn id(&self) -> SourceTypeApplicationId {
        self.id
    }

    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn root(&self) -> SourceTypeExpressionId {
        self.root
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceTypeExpressionTable {
    entries: Vec<SourceTypeExpression>,
}

impl SourceTypeExpressionTable {
    pub fn get(&self, id: SourceTypeExpressionId) -> Option<&SourceTypeExpression> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceTypeExpressionId, &SourceTypeExpression)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeExpression {
    id: SourceTypeExpressionId,
    source_id: SourceId,
    module_id: ModuleId,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    head_site: TypedSiteRef,
    head_range: SourceRange,
    head_spelling: String,
    form: SourceTypeApplicationForm,
    head: SourceTypeHead,
    recovery: NodeRecoveryState,
}

impl SourceTypeExpression {
    pub const fn id(&self) -> SourceTypeExpressionId {
        self.id
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
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

    pub const fn form(&self) -> SourceTypeApplicationForm {
        self.form
    }

    pub const fn head(&self) -> &SourceTypeHead {
        &self.head
    }

    pub const fn recovery(&self) -> NodeRecoveryState {
        self.recovery
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceTypeArgumentTable {
    entries: Vec<SourceTypeArgumentRow>,
}

impl SourceTypeArgumentTable {
    pub fn get(&self, id: SourceTypeArgumentId) -> Option<&SourceTypeArgumentRow> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceTypeArgumentId, &SourceTypeArgumentRow)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeArgumentRow {
    id: SourceTypeArgumentId,
    parent: SourceTypeExpressionId,
    ordinal: usize,
    argument: SourceTypeArgument,
}

impl SourceTypeArgumentRow {
    pub const fn id(&self) -> SourceTypeArgumentId {
        self.id
    }

    pub const fn parent(&self) -> SourceTypeExpressionId {
        self.parent
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn argument(&self) -> &SourceTypeArgument {
        &self.argument
    }
}

/// Validates and transactionally constructs source type-expression handoffs.
pub struct SourceTypeProducer;

impl SourceTypeProducer {
    pub fn build(
        input: SourceTypeHandoffInput,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError> {
        validate_input(&input, bindings, symbols, arena)?;
        Ok(SourceTypeApplicationHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            applications: SourceTypeApplicationTable {
                entries: input
                    .applications
                    .into_iter()
                    .enumerate()
                    .map(|(index, input)| SourceTypeApplication {
                        id: SourceTypeApplicationId::new(index),
                        binding: input.binding,
                        source_ordinal: input.source_ordinal,
                        root: input.root,
                    })
                    .collect(),
            },
            expressions: SourceTypeExpressionTable {
                entries: input
                    .expressions
                    .into_iter()
                    .enumerate()
                    .map(|(index, input)| SourceTypeExpression {
                        id: SourceTypeExpressionId::new(index),
                        source_id: input.source_id,
                        module_id: input.module_id,
                        site: input.site,
                        source_range: input.source_range,
                        spelling: input.spelling,
                        head_site: input.head_site,
                        head_range: input.head_range,
                        head_spelling: input.head_spelling,
                        form: input.form,
                        head: input.head,
                        recovery: input.recovery,
                    })
                    .collect(),
            },
            arguments: SourceTypeArgumentTable {
                entries: input
                    .arguments
                    .into_iter()
                    .enumerate()
                    .map(|(index, input)| SourceTypeArgumentRow {
                        id: SourceTypeArgumentId::new(index),
                        parent: input.parent,
                        ordinal: input.ordinal,
                        argument: input.argument,
                    })
                    .collect(),
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceTypeError {
    EmptyApplications,
    EmptyExpressions,
    EnvironmentMismatch,
    BindingCardinalityMismatch,
    InvalidApplication {
        application: SourceTypeApplicationId,
    },
    InvalidBinding {
        application: SourceTypeApplicationId,
    },
    InvalidExpression {
        expression: SourceTypeExpressionId,
    },
    InvalidHead {
        expression: SourceTypeExpressionId,
    },
    InvalidSymbolHead {
        expression: SourceTypeExpressionId,
    },
    InvalidExpressionSite {
        expression: SourceTypeExpressionId,
    },
    InvalidArgument {
        argument: SourceTypeArgumentId,
    },
    InvalidArgumentSite {
        argument: SourceTypeArgumentId,
    },
    InvalidProvenance {
        argument: SourceTypeArgumentId,
    },
    DuplicateSite,
    ReorderedArgument {
        argument: SourceTypeArgumentId,
    },
    DanglingChild {
        argument: SourceTypeArgumentId,
        child: SourceTypeExpressionId,
    },
    DuplicateChild {
        argument: SourceTypeArgumentId,
        child: SourceTypeExpressionId,
    },
    MultipleParents {
        child: SourceTypeExpressionId,
    },
    RootHasParent {
        root: SourceTypeExpressionId,
    },
    ForwardParent {
        parent: SourceTypeExpressionId,
        child: SourceTypeExpressionId,
    },
    Cycle {
        expression: SourceTypeExpressionId,
    },
    UnreachableExpression {
        expression: SourceTypeExpressionId,
    },
    WrongApplicationForm {
        expression: SourceTypeExpressionId,
    },
    ChildOutsideParent {
        parent: SourceTypeExpressionId,
        child: SourceTypeExpressionId,
    },
    OverlappingSiblings {
        parent: SourceTypeExpressionId,
    },
    OverlappingApplications {
        application: SourceTypeApplicationId,
    },
}

impl fmt::Display for SourceTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyApplications => formatter.write_str("source type input has no applications"),
            Self::EmptyExpressions => formatter.write_str("source type input has no expressions"),
            Self::EnvironmentMismatch => {
                formatter.write_str("source type input environment identity mismatch")
            }
            Self::BindingCardinalityMismatch => {
                formatter.write_str("source type application and binding cardinalities differ")
            }
            Self::InvalidApplication { application } => write!(
                formatter,
                "source type application {} is invalid",
                application.index()
            ),
            Self::InvalidBinding { application } => write!(
                formatter,
                "source type application {} has an invalid binding",
                application.index()
            ),
            Self::InvalidExpression { expression } => write!(
                formatter,
                "source type expression {} is invalid",
                expression.index()
            ),
            Self::InvalidHead { expression } => write!(
                formatter,
                "source type expression {} has an invalid head",
                expression.index()
            ),
            Self::InvalidSymbolHead { expression } => write!(
                formatter,
                "source type expression {} has an unauthenticated symbol head",
                expression.index()
            ),
            Self::InvalidExpressionSite { expression } => write!(
                formatter,
                "source type expression {} has an invalid typed site",
                expression.index()
            ),
            Self::InvalidArgument { argument } => write!(
                formatter,
                "source type argument {} is invalid",
                argument.index()
            ),
            Self::InvalidArgumentSite { argument } => write!(
                formatter,
                "source type argument {} has an invalid typed site",
                argument.index()
            ),
            Self::InvalidProvenance { argument } => write!(
                formatter,
                "source type argument {} has invalid provenance",
                argument.index()
            ),
            Self::DuplicateSite => formatter.write_str("source type input repeats a typed site"),
            Self::ReorderedArgument { argument } => write!(
                formatter,
                "source type argument {} is out of canonical order",
                argument.index()
            ),
            Self::DanglingChild { argument, child } => write!(
                formatter,
                "source type argument {} references missing expression {}",
                argument.index(),
                child.index()
            ),
            Self::DuplicateChild { argument, child } => write!(
                formatter,
                "source type argument {} repeats expression {}",
                argument.index(),
                child.index()
            ),
            Self::MultipleParents { child } => write!(
                formatter,
                "source type expression {} has multiple parents",
                child.index()
            ),
            Self::RootHasParent { root } => write!(
                formatter,
                "source type root expression {} also has a parent",
                root.index()
            ),
            Self::ForwardParent { parent, child } => write!(
                formatter,
                "source type parent {} does not precede child {}",
                parent.index(),
                child.index()
            ),
            Self::Cycle { expression } => write!(
                formatter,
                "source type expression {} participates in a cycle",
                expression.index()
            ),
            Self::UnreachableExpression { expression } => write!(
                formatter,
                "source type expression {} is unreachable",
                expression.index()
            ),
            Self::WrongApplicationForm { expression } => write!(
                formatter,
                "source type expression {} has arguments incompatible with its form",
                expression.index()
            ),
            Self::ChildOutsideParent { parent, child } => write!(
                formatter,
                "source type expression {} does not contain child {}",
                parent.index(),
                child.index()
            ),
            Self::OverlappingSiblings { parent } => write!(
                formatter,
                "source type expression {} has overlapping argument siblings",
                parent.index()
            ),
            Self::OverlappingApplications { application } => write!(
                formatter,
                "source type application {} overlaps its predecessor",
                application.index()
            ),
        }
    }
}

impl Error for SourceTypeError {}

fn validate_input(
    input: &SourceTypeHandoffInput,
    bindings: &BindingEnv,
    symbols: &SymbolEnv,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if input.applications.is_empty() {
        return Err(SourceTypeError::EmptyApplications);
    }
    if input.expressions.is_empty() {
        return Err(SourceTypeError::EmptyExpressions);
    }
    if bindings.source_id() != input.source_id
        || bindings.module_id() != &input.module_id
        || symbols.module_id() != &input.module_id
    {
        return Err(SourceTypeError::EnvironmentMismatch);
    }
    if bindings.bindings().len() != input.applications.len() {
        return Err(SourceTypeError::BindingCardinalityMismatch);
    }

    let mut sites = BTreeSet::new();
    for (index, expression) in input.expressions.iter().enumerate() {
        let id = SourceTypeExpressionId::new(index);
        validate_expression(input, id, expression, symbols, arena)?;
        if !sites.insert(expression.site.clone()) || !sites.insert(expression.head_site.clone()) {
            return Err(SourceTypeError::DuplicateSite);
        }
    }

    let roots = validate_applications(input, bindings)?;
    let validated_arguments = validate_arguments(input, arena, &mut sites)?;
    validate_graph(
        input,
        &roots,
        &validated_arguments.parents,
        &validated_arguments.children,
    )?;
    validate_forms(input)?;
    validate_sibling_ranges(input, &validated_arguments.spans)?;
    Ok(())
}

fn validate_applications(
    input: &SourceTypeHandoffInput,
    bindings: &BindingEnv,
) -> Result<BTreeSet<SourceTypeExpressionId>, SourceTypeError> {
    if !bindings.diagnostics().is_empty() {
        return Err(SourceTypeError::BindingCardinalityMismatch);
    }

    let mut roots = BTreeSet::new();
    let mut previous_root = None;
    let mut previous_range = None;
    for (index, application) in input.applications.iter().enumerate() {
        let id = SourceTypeApplicationId::new(index);
        let Some(root) = input.expressions.get(application.root.index()) else {
            return Err(SourceTypeError::InvalidApplication { application: id });
        };
        if application.source_ordinal != index
            || application.binding != BindingId::new(index)
            || previous_root.is_some_and(|previous| application.root <= previous)
            || !roots.insert(application.root)
        {
            return Err(SourceTypeError::InvalidApplication { application: id });
        }
        if previous_range.is_some_and(|range: SourceRange| range.end > root.source_range.start) {
            return Err(SourceTypeError::OverlappingApplications { application: id });
        }
        let Some(binding) = bindings.bindings().get(application.binding) else {
            return Err(SourceTypeError::InvalidBinding { application: id });
        };
        let Some(context) = bindings.contexts().get(binding.owner_context) else {
            return Err(SourceTypeError::InvalidBinding { application: id });
        };
        let identity_matches = match (&binding.kind, &binding.identity, binding.status) {
            (
                BindingKind::ReservedVariable,
                BinderIdentity::ReservedVariable {
                    spelling,
                    declaration_range,
                },
                BindingStatus::Reserved,
            ) => {
                spelling == &binding.spelling
                    && declaration_range == &binding.declaration_range
                    && matches!(context.owner, BindingContextOwner::Module)
                    && context.layer == BindingContextLayer::Module
            }
            (
                BindingKind::DefinitionParameter,
                BinderIdentity::ResolverLocal {
                    scope,
                    ordinal,
                    declaration_range,
                },
                BindingStatus::Active,
            ) => {
                *ordinal == binding.visible_after_ordinal
                    && declaration_range == &binding.declaration_range
                    && matches!(context.owner, BindingContextOwner::DeclarationShell(_))
                    && context.layer == BindingContextLayer::Declaration
                    && context.parent.is_some()
                    && context.lexical_scope.as_ref() == Some(scope)
            }
            _ => false,
        };
        if binding.id != application.binding
            || binding.visible_after_ordinal != application.source_ordinal
            || binding.recovery != BindingRecoveryState::Normal
            || !binding.diagnostics.is_empty()
            || !binding.captured.identities().is_empty()
            || !identity_matches
            || context.recovery != BindingContextRecovery::Normal
            || !context.bindings.contains(&application.binding)
            || !context.visible_bindings.contains(&application.binding)
            || binding.declaration_range.source_id != input.source_id
            || binding.declaration_range.start >= binding.declaration_range.end
            || binding.type_site != BindingTypeSite::Source(root.source_range)
        {
            return Err(SourceTypeError::InvalidBinding { application: id });
        }
        previous_root = Some(application.root);
        previous_range = Some(root.source_range);
    }
    Ok(roots)
}

fn validate_expression(
    input: &SourceTypeHandoffInput,
    id: SourceTypeExpressionId,
    expression: &SourceTypeExpressionInput,
    symbols: &SymbolEnv,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if expression.source_id != input.source_id
        || expression.module_id != input.module_id
        || !valid_range(input.source_id, expression.source_range)
        || !valid_range(input.source_id, expression.head_range)
        || !range_contains(expression.source_range, expression.head_range)
        || expression.spelling.trim().is_empty()
        || expression.head_spelling.trim().is_empty()
        || expression.site == expression.head_site
    {
        return Err(SourceTypeError::InvalidExpression { expression: id });
    }
    validate_arena_site(
        id,
        &expression.site,
        expression.source_range,
        expression.recovery,
        arena,
    )?;
    validate_arena_site(
        id,
        &expression.head_site,
        expression.head_range,
        expression.recovery,
        arena,
    )?;
    match &expression.head {
        SourceTypeHead::BuiltinSet if expression.head_spelling == "set" => Ok(()),
        SourceTypeHead::BuiltinObject if expression.head_spelling == "object" => Ok(()),
        SourceTypeHead::BuiltinSet | SourceTypeHead::BuiltinObject => {
            Err(SourceTypeError::InvalidHead { expression: id })
        }
        SourceTypeHead::Symbol {
            symbol,
            contribution,
        } => validate_symbol_head(input, id, expression, symbol, *contribution, symbols),
    }
}

fn validate_symbol_head(
    input: &SourceTypeHandoffInput,
    id: SourceTypeExpressionId,
    expression: &SourceTypeExpressionInput,
    symbol: &SymbolId,
    contribution_id: SourceContributionId,
    symbols: &SymbolEnv,
) -> Result<(), SourceTypeError> {
    let invalid = || SourceTypeError::InvalidSymbolHead { expression: id };
    let entry = symbols.symbols().get(symbol).ok_or_else(invalid)?;
    let contribution = symbols
        .contributions()
        .get(contribution_id)
        .ok_or_else(invalid)?;
    if entry.contribution() != contribution_id
        || !matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure)
        || !symbol_spelling_matches_form(
            entry.primary_spelling(),
            &expression.head_spelling,
            expression.form,
        )
        || entry.namespace().as_str() != input.module_id.path().as_str()
        || contribution.module() != symbol.module()
        || !contribution.effects().symbols().contains(symbol)
        || entry.origin().is_recovered()
    {
        return Err(invalid());
    }

    if symbol.module() == &input.module_id {
        let origin_range = source_range(entry.origin().anchor()).ok_or_else(invalid)?;
        if contribution.module() != &input.module_id
            || !matches!(
                contribution.kind(),
                ContributionKind::LocalSource { source_id } if *source_id == input.source_id
            )
            || entry.origin().source_id() != input.source_id
            || entry.origin().module_id() != &input.module_id
            || entry.origin().import_edge().is_some()
            || !valid_range(input.source_id, origin_range)
            || origin_range.end > expression.head_range.start
        {
            return Err(invalid());
        }
    } else {
        let contribution_range = source_range(contribution.anchor()).ok_or_else(invalid)?;
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
        ) || entry.visibility() != Visibility::Public
            || !matches!(
                entry.export_status(),
                ExportStatus::Exported | ExportStatus::ReExported
            )
            || !valid_range(input.source_id, contribution_range)
            || contribution_range.end > expression.head_range.start
            || entry.origin().module_id() != symbol.module()
            || !import_is_authenticated
        {
            return Err(invalid());
        }
    }
    Ok(())
}

fn symbol_spelling_matches_form(
    primary: &str,
    head: &str,
    form: SourceTypeApplicationForm,
) -> bool {
    match form {
        SourceTypeApplicationForm::Bare => primary == head,
        SourceTypeApplicationForm::Of => primary
            .strip_prefix(head)
            .is_some_and(|suffix| suffix.starts_with(" of ") && suffix.len() > " of ".len()),
        SourceTypeApplicationForm::Over => primary
            .strip_prefix(head)
            .is_some_and(|suffix| suffix.starts_with(" over ") && suffix.len() > " over ".len()),
        SourceTypeApplicationForm::Bracket => primary.strip_prefix(head).is_some_and(|suffix| {
            suffix.starts_with(" [ ") && suffix.ends_with(" ]") && suffix.len() > " [  ]".len()
        }),
    }
}

type ArgumentSpans = Vec<Vec<(usize, SourceRange)>>;

struct ValidatedArguments {
    parents: Vec<Option<SourceTypeExpressionId>>,
    children: Vec<Vec<SourceTypeExpressionId>>,
    spans: ArgumentSpans,
}

fn validate_arguments(
    input: &SourceTypeHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<ValidatedArguments, SourceTypeError> {
    let mut parents = vec![None; input.expressions.len()];
    let mut children = vec![Vec::new(); input.expressions.len()];
    let mut spans = vec![Vec::new(); input.expressions.len()];
    let mut expected_parent = 0;
    let mut expected_ordinal = 0;
    for (index, argument) in input.arguments.iter().enumerate() {
        let id = SourceTypeArgumentId::new(index);
        let Some(parent) = input.expressions.get(argument.parent.index()) else {
            return Err(SourceTypeError::InvalidArgument { argument: id });
        };
        if argument.parent.index() < expected_parent {
            return Err(SourceTypeError::ReorderedArgument { argument: id });
        }
        if argument.parent.index() > expected_parent {
            expected_parent = argument.parent.index();
            expected_ordinal = 0;
        }
        if argument.ordinal != expected_ordinal {
            return Err(SourceTypeError::ReorderedArgument { argument: id });
        }
        expected_ordinal += 1;
        let span = match &argument.argument {
            SourceTypeArgument::TermSite {
                site,
                source_range,
                spelling,
                recovery,
                provenance,
            } => {
                validate_source_argument(
                    input,
                    id,
                    argument,
                    site,
                    *source_range,
                    spelling,
                    *recovery,
                    provenance,
                    arena,
                    sites,
                )?;
                *source_range
            }
            SourceTypeArgument::TypeSite { expression } => {
                add_child(
                    input,
                    id,
                    argument.parent,
                    *expression,
                    &mut parents,
                    &mut children,
                )?;
                input.expressions[expression.index()].source_range
            }
            SourceTypeArgument::QuaSite {
                site,
                source_range,
                spelling,
                recovery,
                provenance,
                radix,
            } => {
                validate_source_argument(
                    input,
                    id,
                    argument,
                    site,
                    *source_range,
                    spelling,
                    *recovery,
                    provenance,
                    arena,
                    sites,
                )?;
                if radix.is_empty() {
                    return Err(SourceTypeError::InvalidArgument { argument: id });
                }
                let mut span = *source_range;
                let mut unique = BTreeSet::new();
                for expression in radix {
                    if !unique.insert(*expression) {
                        return Err(SourceTypeError::DuplicateChild {
                            argument: id,
                            child: *expression,
                        });
                    }
                    add_child(
                        input,
                        id,
                        argument.parent,
                        *expression,
                        &mut parents,
                        &mut children,
                    )?;
                    let child_range = input.expressions[expression.index()].source_range;
                    if span.end > child_range.start {
                        return Err(SourceTypeError::OverlappingSiblings {
                            parent: argument.parent,
                        });
                    }
                    span.end = child_range.end;
                }
                span
            }
        };
        if !range_contains(parent.source_range, span) {
            return Err(SourceTypeError::InvalidArgument { argument: id });
        }
        spans[argument.parent.index()].push((argument.ordinal, span));
    }
    Ok(ValidatedArguments {
        parents,
        children,
        spans,
    })
}

#[allow(clippy::too_many_arguments)] // Rationale: keep every source-site invariant explicit at the validation boundary.
fn validate_source_argument(
    input: &SourceTypeHandoffInput,
    id: SourceTypeArgumentId,
    argument: &SourceTypeArgumentInput,
    site: &TypedSiteRef,
    range: SourceRange,
    spelling: &str,
    recovery: NodeRecoveryState,
    provenance: &SemanticOrigin,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceTypeError> {
    if !valid_range(input.source_id, range)
        || spelling.trim().is_empty()
        || !sites.insert(site.clone())
    {
        return Err(SourceTypeError::InvalidArgument { argument: id });
    }
    validate_argument_arena_site(id, site, range, recovery, arena)?;
    let parent = u32::try_from(argument.parent.index())
        .map_err(|_| SourceTypeError::InvalidProvenance { argument: id })?;
    let ordinal = u32::try_from(argument.ordinal)
        .map_err(|_| SourceTypeError::InvalidProvenance { argument: id })?;
    if provenance.source_id() != input.source_id
        || provenance.module_id() != &input.module_id
        || provenance.anchor() != &SourceAnchor::Range(range)
        || provenance.structural_path() != [parent, ordinal]
        || provenance.import_edge().is_some()
        || provenance.is_recovered() != !matches!(recovery, NodeRecoveryState::Normal)
    {
        return Err(SourceTypeError::InvalidProvenance { argument: id });
    }
    Ok(())
}

fn add_child(
    input: &SourceTypeHandoffInput,
    argument: SourceTypeArgumentId,
    parent: SourceTypeExpressionId,
    child: SourceTypeExpressionId,
    parents: &mut [Option<SourceTypeExpressionId>],
    children: &mut [Vec<SourceTypeExpressionId>],
) -> Result<(), SourceTypeError> {
    if input.expressions.get(child.index()).is_none() {
        return Err(SourceTypeError::DanglingChild { argument, child });
    }
    if children[parent.index()].contains(&child) {
        return Err(SourceTypeError::DuplicateChild { argument, child });
    }
    if parents[child.index()].replace(parent).is_some() {
        return Err(SourceTypeError::MultipleParents { child });
    }
    children[parent.index()].push(child);
    Ok(())
}

fn validate_graph(
    input: &SourceTypeHandoffInput,
    roots: &BTreeSet<SourceTypeExpressionId>,
    parents: &[Option<SourceTypeExpressionId>],
    children: &[Vec<SourceTypeExpressionId>],
) -> Result<(), SourceTypeError> {
    for root in roots {
        if parents[root.index()].is_some() {
            return Err(SourceTypeError::RootHasParent { root: *root });
        }
    }
    validate_acyclic(children)?;
    for (parent_index, child_ids) in children.iter().enumerate() {
        let parent = SourceTypeExpressionId::new(parent_index);
        for child in child_ids {
            if parent >= *child {
                return Err(SourceTypeError::ForwardParent {
                    parent,
                    child: *child,
                });
            }
            if !range_contains(
                input.expressions[parent.index()].source_range,
                input.expressions[child.index()].source_range,
            ) {
                return Err(SourceTypeError::ChildOutsideParent {
                    parent,
                    child: *child,
                });
            }
        }
    }
    let mut reachable = vec![false; input.expressions.len()];
    let mut stack = roots.iter().copied().collect::<Vec<_>>();
    while let Some(expression) = stack.pop() {
        if std::mem::replace(&mut reachable[expression.index()], true) {
            continue;
        }
        stack.extend(children[expression.index()].iter().copied());
    }
    for (index, is_reachable) in reachable.into_iter().enumerate() {
        if !is_reachable {
            return Err(SourceTypeError::UnreachableExpression {
                expression: SourceTypeExpressionId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_acyclic(children: &[Vec<SourceTypeExpressionId>]) -> Result<(), SourceTypeError> {
    let mut states = vec![0_u8; children.len()];
    for start in 0..children.len() {
        if states[start] != 0 {
            continue;
        }
        states[start] = 1;
        let mut stack = vec![(SourceTypeExpressionId::new(start), 0_usize)];
        while let Some((expression, next_child)) = stack.last_mut() {
            let Some(child) = children[expression.index()].get(*next_child).copied() else {
                states[expression.index()] = 2;
                stack.pop();
                continue;
            };
            *next_child += 1;
            match states[child.index()] {
                0 => {
                    states[child.index()] = 1;
                    stack.push((child, 0));
                }
                1 => return Err(SourceTypeError::Cycle { expression: child }),
                2 => {}
                _ => unreachable!("source-type traversal state is internal"),
            }
        }
    }
    Ok(())
}

fn validate_forms(input: &SourceTypeHandoffInput) -> Result<(), SourceTypeError> {
    let mut arguments = vec![Vec::new(); input.expressions.len()];
    for argument in &input.arguments {
        arguments[argument.parent.index()].push(&argument.argument);
    }
    for (index, expression) in input.expressions.iter().enumerate() {
        let valid = match expression.form {
            SourceTypeApplicationForm::Bare => arguments[index].is_empty(),
            SourceTypeApplicationForm::Of | SourceTypeApplicationForm::Over => {
                !arguments[index].is_empty()
                    && arguments[index]
                        .iter()
                        .all(|argument| matches!(argument, SourceTypeArgument::TermSite { .. }))
            }
            SourceTypeApplicationForm::Bracket => {
                !arguments[index].is_empty()
                    && arguments[index].iter().all(|argument| {
                        matches!(
                            argument,
                            SourceTypeArgument::TypeSite { .. }
                                | SourceTypeArgument::QuaSite { .. }
                        )
                    })
            }
        };
        if !valid {
            return Err(SourceTypeError::WrongApplicationForm {
                expression: SourceTypeExpressionId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_sibling_ranges(
    input: &SourceTypeHandoffInput,
    spans: &ArgumentSpans,
) -> Result<(), SourceTypeError> {
    for (parent_index, ranges) in spans.iter().enumerate() {
        let mut previous = None;
        for (ordinal, range) in ranges {
            if previous.is_some_and(|previous: SourceRange| previous.end > range.start) {
                return Err(SourceTypeError::OverlappingSiblings {
                    parent: SourceTypeExpressionId::new(parent_index),
                });
            }
            if *ordinal >= input.arguments.len() {
                return Err(SourceTypeError::OverlappingSiblings {
                    parent: SourceTypeExpressionId::new(parent_index),
                });
            }
            previous = Some(*range);
        }
    }
    Ok(())
}

fn validate_arena_site(
    expression: SourceTypeExpressionId,
    site: &TypedSiteRef,
    range: SourceRange,
    recovery: NodeRecoveryState,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    let Some(node) = arena.node(site.node()) else {
        return Err(SourceTypeError::InvalidExpressionSite { expression });
    };
    let Some(anchor) = source_range(&node.anchor) else {
        return Err(SourceTypeError::InvalidExpressionSite { expression });
    };
    if node.recovery != recovery || !range_contains(anchor, range) {
        return Err(SourceTypeError::InvalidExpressionSite { expression });
    }
    Ok(())
}

fn validate_argument_arena_site(
    argument: SourceTypeArgumentId,
    site: &TypedSiteRef,
    range: SourceRange,
    recovery: NodeRecoveryState,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    let Some(node) = arena.node(site.node()) else {
        return Err(SourceTypeError::InvalidArgumentSite { argument });
    };
    let Some(anchor) = source_range(&node.anchor) else {
        return Err(SourceTypeError::InvalidArgumentSite { argument });
    };
    if node.recovery != recovery || !range_contains(anchor, range) {
        return Err(SourceTypeError::InvalidArgumentSite { argument });
    }
    Ok(())
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

fn form_key(form: SourceTypeApplicationForm) -> &'static str {
    match form {
        SourceTypeApplicationForm::Bare => "bare",
        SourceTypeApplicationForm::Of => "of",
        SourceTypeApplicationForm::Over => "over",
        SourceTypeApplicationForm::Bracket => "bracket",
    }
}

fn recovery_key(recovery: NodeRecoveryState) -> &'static str {
    match recovery {
        NodeRecoveryState::Normal => "normal",
        NodeRecoveryState::Recovered => "recovered",
        NodeRecoveryState::Degraded => "degraded",
    }
}

fn write_site(output: &mut String, site: &TypedSiteRef) {
    match site {
        TypedSiteRef::Node(node) => {
            let _ = write!(output, "node:{}", node.index());
        }
        TypedSiteRef::Role { node, role } => {
            let _ = write!(output, "node:{}:role:{:?}", node.index(), role.as_str());
        }
    }
}

fn write_head(output: &mut String, head: &SourceTypeHead) {
    match head {
        SourceTypeHead::BuiltinSet => output.push_str("builtin:set"),
        SourceTypeHead::BuiltinObject => output.push_str("builtin:object"),
        SourceTypeHead::Symbol {
            symbol,
            contribution,
        } => {
            let _ = write!(
                output,
                "symbol:{}:contribution:{}",
                symbol.fqn().as_str(),
                contribution.index()
            );
        }
    }
}

fn write_argument(output: &mut String, argument: &SourceTypeArgument) {
    match argument {
        SourceTypeArgument::TermSite {
            site,
            source_range,
            spelling,
            recovery,
            ..
        } => {
            output.push_str("term site=");
            write_site(output, site);
            let _ = write!(
                output,
                " range={}..{} recovery={} spelling={:?}",
                source_range.start,
                source_range.end,
                recovery_key(*recovery),
                spelling,
            );
        }
        SourceTypeArgument::TypeSite { expression } => {
            let _ = write!(output, "type expression={}", expression.index());
        }
        SourceTypeArgument::QuaSite {
            site,
            source_range,
            spelling,
            recovery,
            radix,
            ..
        } => {
            output.push_str("qua site=");
            write_site(output, site);
            let _ = write!(
                output,
                " range={}..{} recovery={} spelling={:?} radix=",
                source_range.start,
                source_range.end,
                recovery_key(*recovery),
                spelling,
            );
            for (index, expression) in radix.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                let _ = write!(output, "{}", expression.index());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BindingContextDraft, BindingContextId, BindingContextTable, BindingDiagnosticClass,
            BindingDiagnosticDraft, BindingDiagnosticRecovery, BindingDiagnosticSeverity,
            BindingDiagnosticTable, BindingDraft, BindingEnvParts, BindingTable,
            CapturedFreeVariables,
        },
        type_checker::{SourceReserveBindingInput, SourceReserveDeclarationBridge, TypeHeadInput},
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactTable, TypeRole, TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode,
        },
    };
    use mizar_resolve::{
        env::{NamespacePath, SymbolEntry, SymbolEnvIndexes},
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
        input: SourceTypeHandoffInput,
        bindings: BindingEnv,
        symbols: SymbolEnv,
        arena: TypedArena,
    }

    fn source_id() -> SourceId {
        source_id_for("a7")
    }

    fn other_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "a7".repeat(32)
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

    fn role(node: usize, value: &str) -> TypedSiteRef {
        TypedSiteRef::Role {
            node: crate::typed_ast::TypedNodeId::new(node),
            role: TypeRole::new(value),
        }
    }

    fn bare_expression(
        source: SourceId,
        module: &ModuleId,
        node: usize,
        start: usize,
        end: usize,
        head: &str,
    ) -> SourceTypeExpressionInput {
        SourceTypeExpressionInput {
            source_id: source,
            module_id: module.clone(),
            site: role(node, &format!("expression-{node}")),
            source_range: range(source, start, end),
            spelling: head.to_owned(),
            head_site: role(node, &format!("head-{node}")),
            head_range: range(source, start, start + head.len()),
            head_spelling: head.to_owned(),
            form: SourceTypeApplicationForm::Bare,
            head: match head {
                "set" => SourceTypeHead::BuiltinSet,
                "object" => SourceTypeHead::BuiltinObject,
                _ => panic!("test helper only admits builtins"),
            },
            recovery: NodeRecoveryState::Normal,
        }
    }

    fn binding_env(source: SourceId, module: &ModuleId, type_ranges: &[SourceRange]) -> BindingEnv {
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
        for (index, type_range) in type_ranges.iter().enumerate() {
            let declaration_range = range(source, index * 2, index * 2 + 1);
            let spelling = format!("x{index}");
            bindings.insert(BindingDraft {
                spelling: spelling.clone(),
                kind: BindingKind::ReservedVariable,
                identity: BinderIdentity::ReservedVariable {
                    spelling,
                    declaration_range,
                },
                owner_context: BindingContextId::new(0),
                declaration_range,
                visible_after_ordinal: index,
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

    fn arena_for(input: &SourceTypeHandoffInput) -> TypedArena {
        let max_node = input
            .expressions
            .iter()
            .flat_map(|expression| [expression.site.node(), expression.head_site.node()])
            .chain(
                input
                    .arguments
                    .iter()
                    .filter_map(|argument| match &argument.argument {
                        SourceTypeArgument::TermSite { site, .. }
                        | SourceTypeArgument::QuaSite { site, .. } => Some(site.node()),
                        SourceTypeArgument::TypeSite { .. } => None,
                    }),
            )
            .map(|node| node.index())
            .max()
            .expect("at least one site");
        let mut anchors = vec![None::<(SourceRange, NodeRecoveryState)>; max_node + 1];
        let mut record = |site: &TypedSiteRef, range: SourceRange, recovery: NodeRecoveryState| {
            let entry = &mut anchors[site.node().index()];
            match entry {
                Some((anchor, existing_recovery)) => {
                    assert_eq!(*existing_recovery, recovery);
                    anchor.start = anchor.start.min(range.start);
                    anchor.end = anchor.end.max(range.end);
                }
                None => *entry = Some((range, recovery)),
            }
        };
        for expression in &input.expressions {
            record(
                &expression.site,
                expression.source_range,
                expression.recovery,
            );
            record(
                &expression.head_site,
                expression.head_range,
                expression.recovery,
            );
        }
        for argument in &input.arguments {
            match &argument.argument {
                SourceTypeArgument::TermSite {
                    site,
                    source_range,
                    recovery,
                    ..
                }
                | SourceTypeArgument::QuaSite {
                    site,
                    source_range,
                    recovery,
                    ..
                } => record(site, *source_range, *recovery),
                SourceTypeArgument::TypeSite { .. } => {}
            }
        }
        TypedArena::try_new(
            None,
            anchors
                .into_iter()
                .enumerate()
                .map(|(index, value)| {
                    let (anchor, recovery) = value.unwrap_or_else(|| {
                        (range(input.source_id, 0, 1), NodeRecoveryState::Normal)
                    });
                    TypedNode::new(
                        format!("source-type-test-{index}"),
                        SourceAnchor::Range(anchor),
                    )
                    .with_recovery(recovery)
                })
                .collect(),
        )
        .expect("arena")
    }

    fn refresh(fixture: &mut Fixture) {
        let root_ranges = fixture
            .input
            .applications
            .iter()
            .map(|application| fixture.input.expressions[application.root.index()].source_range)
            .collect::<Vec<_>>();
        fixture.bindings = binding_env(fixture.source, &fixture.module, &root_ranges);
        fixture.arena = arena_for(&fixture.input);
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = module("source.type");
        let mut root = bare_expression(source, &module, 0, 10, 90, "set");
        root.form = SourceTypeApplicationForm::Bracket;
        root.spelling = "set[object, qua x set]".to_owned();
        let expressions = vec![
            root,
            bare_expression(source, &module, 1, 20, 30, "object"),
            bare_expression(source, &module, 2, 50, 60, "set"),
        ];
        let arguments = vec![
            SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(0),
                ordinal: 0,
                argument: SourceTypeArgument::TypeSite {
                    expression: SourceTypeExpressionId::new(1),
                },
            },
            SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(0),
                ordinal: 1,
                argument: SourceTypeArgument::QuaSite {
                    site: role(3, "qua-0-1"),
                    source_range: range(source, 40, 41),
                    spelling: "x".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    provenance: SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, 40, 41)),
                        vec![0, 1],
                    ),
                    radix: vec![SourceTypeExpressionId::new(2)],
                },
            },
        ];
        let input = SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            }],
            expressions,
            arguments,
        };
        let bindings = binding_env(source, &module, &[input.expressions[0].source_range]);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let arena = arena_for(&input);
        Fixture {
            source,
            module,
            input,
            bindings,
            symbols,
            arena,
        }
    }

    fn term_fixture(form: SourceTypeApplicationForm) -> Fixture {
        let mut fixture = fixture();
        fixture.input.expressions.truncate(1);
        fixture.input.expressions[0].form = form;
        fixture.input.arguments = vec![SourceTypeArgumentInput {
            parent: SourceTypeExpressionId::new(0),
            ordinal: 0,
            argument: SourceTypeArgument::TermSite {
                site: role(3, "term-0-0"),
                source_range: range(fixture.source, 20, 21),
                spelling: "x".to_owned(),
                recovery: NodeRecoveryState::Normal,
                provenance: SemanticOrigin::new(
                    fixture.source,
                    fixture.module.clone(),
                    SourceAnchor::Range(range(fixture.source, 20, 21)),
                    vec![0, 0],
                ),
            },
        }];
        refresh(&mut fixture);
        fixture
    }

    fn two_root_fixture() -> Fixture {
        let mut fixture = fixture();
        fixture.input.expressions = vec![
            bare_expression(fixture.source, &fixture.module, 0, 10, 20, "set"),
            bare_expression(fixture.source, &fixture.module, 1, 30, 40, "object"),
        ];
        fixture.input.applications = vec![
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
        ];
        fixture.input.arguments.clear();
        refresh(&mut fixture);
        fixture
    }

    #[derive(Clone, Copy, Debug)]
    enum BindingMutation {
        GlobalDiagnostic,
        Recovery,
        Diagnostic,
        Capture,
        WrongIdentity,
        WrongStatus,
        WrongOrdinal,
        ContextRecovery,
        MissingMembership,
        MissingVisibility,
        EmptyDeclarationRange,
        WrongTypeSite,
    }

    fn binding_env_with_mutation(fixture: &Fixture, mutation: BindingMutation) -> BindingEnv {
        let declaration_range = range(fixture.source, 1, 2);
        let mut diagnostics = BindingDiagnosticTable::new();
        let diagnostic = matches!(
            mutation,
            BindingMutation::GlobalDiagnostic | BindingMutation::Diagnostic
        )
        .then(|| {
            diagnostics.insert(BindingDiagnosticDraft {
                source_range: Some(declaration_range),
                class: BindingDiagnosticClass::ExternalDependencyGap,
                severity: BindingDiagnosticSeverity::Note,
                message_key: "source_type.test.binding_diagnostic".to_owned(),
                recovery: BindingDiagnosticRecovery::Normal,
            })
        });
        let mut binding = BindingDraft {
            spelling: "x0".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x0".to_owned(),
                declaration_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(fixture.input.expressions[0].source_range),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        };
        let mut context = BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        };
        match mutation {
            BindingMutation::GlobalDiagnostic => {}
            BindingMutation::Recovery => binding.recovery = BindingRecoveryState::Recovered,
            BindingMutation::Diagnostic => {
                binding.diagnostics = vec![diagnostic.expect("diagnostic")]
            }
            BindingMutation::Capture => {
                binding.captured = CapturedFreeVariables::new(vec![binding.identity.clone()]);
            }
            BindingMutation::WrongIdentity => {
                binding.kind = BindingKind::QuantifierBinder;
                binding.identity = BinderIdentity::ResolverLocal {
                    scope: LocalTermScope::new(vec![0]),
                    ordinal: 0,
                    declaration_range,
                };
                binding.status = BindingStatus::Active;
            }
            BindingMutation::WrongStatus => binding.status = BindingStatus::Active,
            BindingMutation::WrongOrdinal => binding.visible_after_ordinal = 1,
            BindingMutation::ContextRecovery => {
                context.recovery = BindingContextRecovery::Recovered
            }
            BindingMutation::MissingMembership => {
                context.bindings.clear();
                context.visible_bindings.clear();
            }
            BindingMutation::MissingVisibility => context.visible_bindings.clear(),
            BindingMutation::EmptyDeclarationRange => {
                binding.declaration_range = range(fixture.source, 1, 1);
                binding.identity = BinderIdentity::ReservedVariable {
                    spelling: "x0".to_owned(),
                    declaration_range: binding.declaration_range,
                };
            }
            BindingMutation::WrongTypeSite => {
                binding.type_site = BindingTypeSite::Source(range(fixture.source, 10, 19))
            }
        }
        let mut contexts = BindingContextTable::new();
        contexts.insert(context);
        let mut bindings = BindingTable::new();
        bindings.insert(binding);
        BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings,
            diagnostics,
        })
        .expect("corrupt source-type binding env must remain structurally valid upstream")
    }

    fn binding_env_with_identity(
        fixture: &Fixture,
        source_id: SourceId,
        module_id: ModuleId,
    ) -> BindingEnv {
        let mut contexts = BindingContextTable::new();
        for (_, context) in fixture.bindings.contexts().iter() {
            contexts.insert(BindingContextDraft {
                owner: context.owner.clone(),
                parent: context.parent,
                layer: context.layer,
                lexical_scope: context.lexical_scope.clone(),
                bindings: context.bindings.clone(),
                visible_bindings: context.visible_bindings.clone(),
                recovery: context.recovery,
            });
        }
        let mut bindings = BindingTable::new();
        for (_, binding) in fixture.bindings.bindings().iter() {
            let mut declaration_range = binding.declaration_range;
            declaration_range.source_id = source_id;
            let mut identity = binding.identity.clone();
            match &mut identity {
                BinderIdentity::ResolverLocal {
                    declaration_range, ..
                }
                | BinderIdentity::ReservedVariable {
                    declaration_range, ..
                } => declaration_range.source_id = source_id,
                _ => {}
            }
            let mut type_site = binding.type_site.clone();
            if let BindingTypeSite::Source(range) = &mut type_site {
                range.source_id = source_id;
            }
            bindings.insert(BindingDraft {
                spelling: binding.spelling.clone(),
                kind: binding.kind,
                identity,
                owner_context: binding.owner_context,
                declaration_range,
                visible_after_ordinal: binding.visible_after_ordinal,
                type_site,
                status: binding.status,
                captured: binding.captured.clone(),
                diagnostics: binding.diagnostics.clone(),
                recovery: binding.recovery,
            });
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id,
            module_id,
            contexts,
            bindings,
            diagnostics: fixture.bindings.diagnostics().clone(),
        })
        .expect("environment identity corruption must remain structurally valid upstream")
    }

    fn build(fixture: &Fixture) -> Result<SourceTypeApplicationHandoff, SourceTypeError> {
        SourceTypeProducer::build(
            fixture.input.clone(),
            &fixture.bindings,
            &fixture.symbols,
            &fixture.arena,
        )
    }

    fn install_symbol(
        fixture: &mut Fixture,
        imported: bool,
        exported: bool,
        valid_signature: bool,
    ) {
        let symbol_module = if imported {
            module("dependency.types")
        } else {
            fixture.module.clone()
        };
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            symbol_module.clone(),
            if imported {
                ContributionKind::ImportedSource {
                    source_id: fixture.source,
                }
            } else {
                ContributionKind::LocalSource {
                    source_id: fixture.source,
                }
            },
            SourceAnchor::Range(range(fixture.source, 1, 5)),
        );
        let symbol = SymbolId::new(
            symbol_module.clone(),
            LocalSymbolId::new("mode"),
            FullyQualifiedName::new(format!("{}::ModeT", symbol_module.path().as_str())),
        );
        let mut entry = SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Mode,
            NamespacePath::new(fixture.module.path().as_str()),
            if valid_signature {
                "ModeT [ p ]"
            } else {
                "ModeTX [ p ]"
            },
            SemanticOrigin::new(
                fixture.source,
                symbol_module,
                SourceAnchor::Range(range(fixture.source, 1, 5)),
                vec![0],
            ),
            contribution,
        );
        if imported && exported {
            entry = entry
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported);
        }
        indexes.symbols.insert(entry);
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        fixture.symbols = SymbolEnv::new(fixture.module.clone(), indexes);
        fixture.input.expressions[0].head = SourceTypeHead::Symbol {
            symbol,
            contribution,
        };
        fixture.input.expressions[0].head_spelling = "ModeT".to_owned();
        fixture.input.expressions[0].head_range = range(fixture.source, 10, 15);
        refresh(fixture);
    }

    #[derive(Clone, Copy, Debug)]
    enum LocalSymbolMutation {
        EntryContribution,
        Kind,
        Signature,
        Namespace,
        ContributionModule,
        ContributionKind,
        ContributionSource,
        MissingEffect,
        OriginSource,
        OriginModule,
        OriginAnchor,
        OriginAfterUse,
        OriginRecovery,
    }

    fn install_local_symbol_mutation(fixture: &mut Fixture, mutation: LocalSymbolMutation) {
        let wrong_module = module("wrong.symbol");
        let contribution_module = if matches!(mutation, LocalSymbolMutation::ContributionModule) {
            wrong_module.clone()
        } else {
            fixture.module.clone()
        };
        let contribution_kind = match mutation {
            LocalSymbolMutation::ContributionKind => ContributionKind::ImportedSource {
                source_id: fixture.source,
            },
            LocalSymbolMutation::ContributionSource => ContributionKind::LocalSource {
                source_id: other_source_id(),
            },
            _ => ContributionKind::LocalSource {
                source_id: fixture.source,
            },
        };
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            contribution_module,
            contribution_kind,
            SourceAnchor::Range(range(fixture.source, 1, 5)),
        );
        let alternate_contribution = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 5)),
        );
        let symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("mode"),
            FullyQualifiedName::new(format!("{}::ModeT", fixture.module.path().as_str())),
        );
        let origin_source = if matches!(mutation, LocalSymbolMutation::OriginSource) {
            other_source_id()
        } else {
            fixture.source
        };
        let origin_module = if matches!(mutation, LocalSymbolMutation::OriginModule) {
            wrong_module
        } else {
            fixture.module.clone()
        };
        let origin_anchor = match mutation {
            LocalSymbolMutation::OriginAnchor => SourceAnchor::Point {
                source_id: fixture.source,
                offset: 1,
            },
            LocalSymbolMutation::OriginAfterUse => {
                SourceAnchor::Range(range(fixture.source, 20, 25))
            }
            _ => SourceAnchor::Range(range(fixture.source, 1, 5)),
        };
        let mut origin = SemanticOrigin::new(origin_source, origin_module, origin_anchor, vec![0]);
        if matches!(mutation, LocalSymbolMutation::OriginRecovery) {
            origin = origin.recovered();
        }
        let entry_contribution = if matches!(mutation, LocalSymbolMutation::EntryContribution) {
            alternate_contribution
        } else {
            contribution
        };
        let entry = SymbolEntry::new(
            symbol.clone(),
            if matches!(mutation, LocalSymbolMutation::Kind) {
                SymbolKind::Predicate
            } else {
                SymbolKind::Mode
            },
            if matches!(mutation, LocalSymbolMutation::Namespace) {
                NamespacePath::new("wrong.namespace")
            } else {
                NamespacePath::new(fixture.module.path().as_str())
            },
            if matches!(mutation, LocalSymbolMutation::Signature) {
                "ModeTX [ p ]"
            } else {
                "ModeT [ p ]"
            },
            origin,
            entry_contribution,
        );
        indexes.symbols.insert(entry);
        if !matches!(mutation, LocalSymbolMutation::MissingEffect) {
            indexes
                .contributions
                .add_symbol(contribution, symbol.clone());
            if entry_contribution != contribution {
                indexes
                    .contributions
                    .add_symbol(entry_contribution, symbol.clone());
            }
        }
        fixture.symbols = SymbolEnv::new(fixture.module.clone(), indexes);
        fixture.input.expressions[0].head = SourceTypeHead::Symbol {
            symbol,
            contribution,
        };
        fixture.input.expressions[0].head_spelling = "ModeT".to_owned();
        fixture.input.expressions[0].head_range = range(fixture.source, 10, 15);
        refresh(fixture);
    }

    #[test]
    fn flat_bracket_handoff_is_dense_immutable_and_deterministic() {
        let fixture = fixture();
        let first = build(&fixture).expect("handoff");
        let second = build(&fixture).expect("handoff");
        assert_eq!(first, second);
        assert_eq!(first.applications().len(), 1);
        assert_eq!(first.expressions().len(), 3);
        assert_eq!(first.arguments().len(), 2);
        assert_eq!(
            first
                .applications()
                .get(SourceTypeApplicationId::new(0))
                .map(SourceTypeApplication::root),
            Some(SourceTypeExpressionId::new(0))
        );
        assert_eq!(first.debug_text(), second.debug_text());
        assert!(
            first
                .debug_text()
                .starts_with("source-type-application-debug-v1\n")
        );
    }

    #[test]
    fn of_and_over_term_sites_preserve_provenance_without_binding_ids() {
        for form in [
            SourceTypeApplicationForm::Of,
            SourceTypeApplicationForm::Over,
        ] {
            let mut fixture = fixture();
            fixture.input.expressions.truncate(1);
            fixture.input.expressions[0].form = form;
            fixture.input.arguments = vec![SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(0),
                ordinal: 0,
                argument: SourceTypeArgument::TermSite {
                    site: role(3, "term-0-0"),
                    source_range: range(fixture.source, 20, 21),
                    spelling: "x".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    provenance: SemanticOrigin::new(
                        fixture.source,
                        fixture.module.clone(),
                        SourceAnchor::Range(range(fixture.source, 20, 21)),
                        vec![0, 0],
                    ),
                },
            }];
            refresh(&mut fixture);
            let handoff = build(&fixture).expect("term handoff");
            assert!(matches!(
                handoff
                    .arguments()
                    .get(SourceTypeArgumentId::new(0))
                    .expect("argument")
                    .argument(),
                SourceTypeArgument::TermSite { .. }
            ));
        }
    }

    #[test]
    fn local_and_imported_mode_heads_are_authenticated() {
        let mut local = fixture();
        install_symbol(&mut local, false, false, true);
        build(&local).expect("local mode");

        let mut imported = fixture();
        install_symbol(&mut imported, true, false, true);
        assert!(matches!(
            build(&imported),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));

        let mut wrong_signature = fixture();
        install_symbol(&mut wrong_signature, false, false, false);
        assert!(matches!(
            build(&wrong_signature),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));

        let mut wrong_form = fixture();
        install_symbol(&mut wrong_form, false, false, true);
        wrong_form.input.expressions[0].form = SourceTypeApplicationForm::Of;
        assert!(matches!(
            build(&wrong_form),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));

        let mut missing_import = fixture();
        install_symbol(&mut missing_import, true, true, true);
        assert!(matches!(
            build(&missing_import),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));
    }

    #[test]
    fn local_symbol_identity_contribution_and_provenance_matrix_is_rejected() {
        for mutation in [
            LocalSymbolMutation::EntryContribution,
            LocalSymbolMutation::Kind,
            LocalSymbolMutation::Signature,
            LocalSymbolMutation::Namespace,
            LocalSymbolMutation::ContributionModule,
            LocalSymbolMutation::ContributionKind,
            LocalSymbolMutation::ContributionSource,
            LocalSymbolMutation::MissingEffect,
            LocalSymbolMutation::OriginSource,
            LocalSymbolMutation::OriginModule,
            LocalSymbolMutation::OriginAnchor,
            LocalSymbolMutation::OriginAfterUse,
            LocalSymbolMutation::OriginRecovery,
        ] {
            let mut fixture = fixture();
            install_local_symbol_mutation(&mut fixture, mutation);
            assert!(
                matches!(
                    build(&fixture),
                    Err(SourceTypeError::InvalidSymbolHead { .. })
                ),
                "local symbol mutation {mutation:?} was accepted"
            );
        }

        let mut missing_symbol = fixture();
        install_symbol(&mut missing_symbol, false, false, true);
        let contribution = match &missing_symbol.input.expressions[0].head {
            SourceTypeHead::Symbol { contribution, .. } => *contribution,
            _ => unreachable!(),
        };
        missing_symbol.input.expressions[0].head = SourceTypeHead::Symbol {
            symbol: SymbolId::new(
                missing_symbol.module.clone(),
                LocalSymbolId::new("missing"),
                FullyQualifiedName::new("source.type::Missing"),
            ),
            contribution,
        };
        assert!(matches!(
            build(&missing_symbol),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));
    }

    #[test]
    fn environment_and_binding_drift_are_rejected_transactionally() {
        let mut wrong_environment = fixture();
        wrong_environment.symbols = SymbolEnv::new(module("wrong"), SymbolEnvIndexes::default());
        assert_eq!(
            build(&wrong_environment),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let mut wrong_binding_source = fixture();
        wrong_binding_source.bindings = binding_env_with_identity(
            &wrong_binding_source,
            other_source_id(),
            wrong_binding_source.module.clone(),
        );
        assert_eq!(
            build(&wrong_binding_source),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let mut wrong_binding_module = fixture();
        wrong_binding_module.bindings = binding_env_with_identity(
            &wrong_binding_module,
            wrong_binding_module.source,
            module("wrong.binding"),
        );
        assert_eq!(
            build(&wrong_binding_module),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let mut wrong_ordinal = fixture();
        wrong_ordinal.input.applications[0].source_ordinal = 1;
        assert!(matches!(
            build(&wrong_ordinal),
            Err(SourceTypeError::InvalidApplication { .. })
        ));

        let mut stale_binding = fixture();
        stale_binding.input.expressions[0].source_range.end -= 1;
        stale_binding.arena = arena_for(&stale_binding.input);
        assert!(matches!(
            build(&stale_binding),
            Err(SourceTypeError::InvalidBinding { .. })
        ));
    }

    #[test]
    fn empty_cardinality_and_application_identity_corruptions_are_rejected() {
        let mut empty_applications = fixture();
        empty_applications.input.applications.clear();
        assert_eq!(
            build(&empty_applications),
            Err(SourceTypeError::EmptyApplications)
        );

        let mut empty_expressions = fixture();
        empty_expressions.input.expressions.clear();
        assert_eq!(
            build(&empty_expressions),
            Err(SourceTypeError::EmptyExpressions)
        );

        let mut cardinality = fixture();
        cardinality.bindings = binding_env(
            cardinality.source,
            &cardinality.module,
            &[
                cardinality.input.expressions[0].source_range,
                cardinality.input.expressions[0].source_range,
            ],
        );
        assert_eq!(
            build(&cardinality),
            Err(SourceTypeError::BindingCardinalityMismatch)
        );

        let mut dangling_root = fixture();
        dangling_root.input.applications[0].root = SourceTypeExpressionId::new(99);
        assert!(matches!(
            build(&dangling_root),
            Err(SourceTypeError::InvalidApplication { .. })
        ));

        let mut duplicate_root = two_root_fixture();
        duplicate_root.input.applications[1].root = SourceTypeExpressionId::new(0);
        assert!(matches!(
            build(&duplicate_root),
            Err(SourceTypeError::InvalidApplication { .. })
        ));

        let mut duplicate_binding = two_root_fixture();
        duplicate_binding.input.applications[1].binding = BindingId::new(0);
        assert!(matches!(
            build(&duplicate_binding),
            Err(SourceTypeError::InvalidApplication { .. })
        ));

        let mut non_monotonic = two_root_fixture();
        non_monotonic.input.applications[0].root = SourceTypeExpressionId::new(1);
        non_monotonic.input.applications[1].root = SourceTypeExpressionId::new(0);
        refresh(&mut non_monotonic);
        assert!(matches!(
            build(&non_monotonic),
            Err(SourceTypeError::InvalidApplication { .. })
        ));
    }

    #[test]
    fn every_checker_specific_binding_invariant_is_rejected() {
        for mutation in [
            BindingMutation::GlobalDiagnostic,
            BindingMutation::Recovery,
            BindingMutation::Diagnostic,
            BindingMutation::Capture,
            BindingMutation::WrongIdentity,
            BindingMutation::WrongStatus,
            BindingMutation::WrongOrdinal,
            BindingMutation::ContextRecovery,
            BindingMutation::MissingMembership,
            BindingMutation::MissingVisibility,
            BindingMutation::EmptyDeclarationRange,
            BindingMutation::WrongTypeSite,
        ] {
            let mut mutated = fixture();
            mutated.bindings = binding_env_with_mutation(&mutated, mutation);
            assert!(
                matches!(
                    build(&mutated),
                    Err(SourceTypeError::BindingCardinalityMismatch)
                        | Err(SourceTypeError::InvalidBinding { .. })
                ),
                "binding mutation was accepted"
            );
        }
    }

    #[test]
    fn expression_identity_range_spelling_and_site_matrix_is_rejected() {
        let mut expression_source = fixture();
        expression_source.input.expressions[0].source_id = other_source_id();
        assert!(matches!(
            build(&expression_source),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut expression_module = fixture();
        expression_module.input.expressions[0].module_id = module("wrong.expression");
        assert!(matches!(
            build(&expression_module),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut source_range_source = fixture();
        source_range_source.input.expressions[0]
            .source_range
            .source_id = other_source_id();
        assert!(matches!(
            build(&source_range_source),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut empty_source_range = fixture();
        empty_source_range.input.expressions[0].source_range.end =
            empty_source_range.input.expressions[0].source_range.start;
        assert!(matches!(
            build(&empty_source_range),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut head_range_source = fixture();
        head_range_source.input.expressions[0].head_range.source_id = other_source_id();
        assert!(matches!(
            build(&head_range_source),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut empty_head_range = fixture();
        empty_head_range.input.expressions[0].head_range.end =
            empty_head_range.input.expressions[0].head_range.start;
        assert!(matches!(
            build(&empty_head_range),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut empty_head_spelling = fixture();
        empty_head_spelling.input.expressions[0]
            .head_spelling
            .clear();
        assert!(matches!(
            build(&empty_head_spelling),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut same_role_site = fixture();
        same_role_site.input.expressions[0].head_site =
            same_role_site.input.expressions[0].site.clone();
        assert!(matches!(
            build(&same_role_site),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut duplicate_site = fixture();
        duplicate_site.input.expressions[1].site = duplicate_site.input.expressions[0].site.clone();
        assert_eq!(build(&duplicate_site), Err(SourceTypeError::DuplicateSite));

        let mut missing_head_site = fixture();
        missing_head_site.input.expressions[0].head_site = role(99, "missing-head");
        assert!(matches!(
            build(&missing_head_site),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));

        let mut wrong_head_anchor = fixture();
        wrong_head_anchor.input.expressions[0].head_site = role(1, "wrong-head-anchor");
        assert!(matches!(
            build(&wrong_head_anchor),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));

        let mut wrong_object_head = fixture();
        wrong_object_head.input.expressions[1].head = SourceTypeHead::BuiltinSet;
        assert!(matches!(
            build(&wrong_object_head),
            Err(SourceTypeError::InvalidHead { .. })
        ));
    }

    #[test]
    fn argument_site_range_spelling_and_provenance_matrix_is_rejected() {
        let mut dangling_parent = term_fixture(SourceTypeApplicationForm::Of);
        dangling_parent.input.arguments[0].parent = SourceTypeExpressionId::new(99);
        assert!(matches!(
            build(&dangling_parent),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut wrong_range_source = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite {
            ref mut source_range,
            ..
        } = wrong_range_source.input.arguments[0].argument
        else {
            unreachable!()
        };
        source_range.source_id = other_source_id();
        assert!(matches!(
            build(&wrong_range_source),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut empty_range = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite {
            ref mut source_range,
            ..
        } = empty_range.input.arguments[0].argument
        else {
            unreachable!()
        };
        source_range.end = source_range.start;
        assert!(matches!(
            build(&empty_range),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut empty_spelling = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite {
            ref mut spelling, ..
        } = empty_spelling.input.arguments[0].argument
        else {
            unreachable!()
        };
        spelling.clear();
        assert!(matches!(
            build(&empty_spelling),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut duplicate_site = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite { ref mut site, .. } =
            duplicate_site.input.arguments[0].argument
        else {
            unreachable!()
        };
        *site = duplicate_site.input.expressions[0].site.clone();
        assert!(matches!(
            build(&duplicate_site),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut missing_site = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite { ref mut site, .. } =
            missing_site.input.arguments[0].argument
        else {
            unreachable!()
        };
        *site = role(99, "missing-term");
        assert!(matches!(
            build(&missing_site),
            Err(SourceTypeError::InvalidArgumentSite { .. })
        ));

        let mut wrong_anchor = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite { ref mut site, .. } =
            wrong_anchor.input.arguments[0].argument
        else {
            unreachable!()
        };
        *site = role(1, "wrong-term-anchor");
        assert!(matches!(
            build(&wrong_anchor),
            Err(SourceTypeError::InvalidArgumentSite { .. })
        ));

        let mut recovery_mismatch = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite {
            ref mut recovery,
            ref mut provenance,
            ..
        } = recovery_mismatch.input.arguments[0].argument
        else {
            unreachable!()
        };
        *recovery = NodeRecoveryState::Recovered;
        *provenance = provenance.clone().recovered();
        assert!(matches!(
            build(&recovery_mismatch),
            Err(SourceTypeError::InvalidArgumentSite { .. })
        ));

        for mutation in 0..5 {
            let mut invalid = term_fixture(SourceTypeApplicationForm::Of);
            let SourceTypeArgument::TermSite {
                provenance: actual,
                source_range,
                ..
            } = &mut invalid.input.arguments[0].argument
            else {
                unreachable!()
            };
            let mut provenance = SemanticOrigin::new(
                invalid.source,
                invalid.module.clone(),
                SourceAnchor::Range(*source_range),
                vec![0, 0],
            );
            match mutation {
                0 => {
                    let source = other_source_id();
                    provenance = SemanticOrigin::new(
                        source,
                        invalid.module.clone(),
                        SourceAnchor::Range(range(source, source_range.start, source_range.end)),
                        vec![0, 0],
                    );
                }
                1 => {
                    provenance = SemanticOrigin::new(
                        invalid.source,
                        module("wrong.provenance"),
                        SourceAnchor::Range(*source_range),
                        vec![0, 0],
                    );
                }
                2 => {
                    provenance = SemanticOrigin::new(
                        invalid.source,
                        invalid.module.clone(),
                        SourceAnchor::Range(range(
                            invalid.source,
                            source_range.start + 1,
                            source_range.end + 1,
                        )),
                        vec![0, 0],
                    );
                }
                3 => {
                    provenance = SemanticOrigin::new(
                        invalid.source,
                        invalid.module.clone(),
                        SourceAnchor::Range(*source_range),
                        vec![0, 1],
                    );
                }
                4 => provenance = provenance.recovered(),
                _ => unreachable!(),
            }
            *actual = provenance;
            assert!(matches!(
                build(&invalid),
                Err(SourceTypeError::InvalidProvenance { .. })
            ));
        }

        let mut empty_radix = fixture();
        let SourceTypeArgument::QuaSite { radix, .. } =
            &mut empty_radix.input.arguments[1].argument
        else {
            unreachable!()
        };
        radix.clear();
        assert!(matches!(
            build(&empty_radix),
            Err(SourceTypeError::InvalidArgument { .. })
        ));
    }

    #[test]
    fn every_application_form_rejects_wrong_argument_shapes() {
        let bare_with_term = term_fixture(SourceTypeApplicationForm::Bare);
        assert!(matches!(
            validate_forms(&bare_with_term.input),
            Err(SourceTypeError::WrongApplicationForm { .. })
        ));

        for form in [
            SourceTypeApplicationForm::Of,
            SourceTypeApplicationForm::Over,
            SourceTypeApplicationForm::Bracket,
        ] {
            let mut empty = fixture();
            empty.input.expressions.truncate(1);
            empty.input.expressions[0].form = form;
            empty.input.arguments.clear();
            assert!(matches!(
                validate_forms(&empty.input),
                Err(SourceTypeError::WrongApplicationForm { .. })
            ));
        }

        for form in [
            SourceTypeApplicationForm::Of,
            SourceTypeApplicationForm::Over,
        ] {
            let mut type_argument = fixture();
            type_argument.input.expressions[0].form = form;
            assert!(matches!(
                validate_forms(&type_argument.input),
                Err(SourceTypeError::WrongApplicationForm { .. })
            ));
        }

        let bracket_with_term = term_fixture(SourceTypeApplicationForm::Bracket);
        assert!(matches!(
            validate_forms(&bracket_with_term.input),
            Err(SourceTypeError::WrongApplicationForm { .. })
        ));
    }

    #[test]
    fn expression_spelling_ranges_heads_and_arena_sites_are_rejected_on_drift() {
        let mut empty_spelling = fixture();
        empty_spelling.input.expressions[0].spelling.clear();
        assert!(matches!(
            build(&empty_spelling),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut wrong_builtin = fixture();
        wrong_builtin.input.expressions[0].head_spelling = "SET".to_owned();
        assert!(matches!(
            build(&wrong_builtin),
            Err(SourceTypeError::InvalidHead { .. })
        ));

        let mut head_outside = fixture();
        head_outside.input.expressions[0].head_range = range(head_outside.source, 91, 94);
        assert!(matches!(
            build(&head_outside),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut missing_site = fixture();
        missing_site.input.expressions[0].site = role(99, "missing");
        assert!(matches!(
            build(&missing_site),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));

        let mut recovery_drift = fixture();
        let mut nodes = recovery_drift
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[0].recovery = NodeRecoveryState::Recovered;
        recovery_drift.arena = TypedArena::try_new(None, nodes).expect("arena");
        assert!(matches!(
            build(&recovery_drift),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));
    }

    #[test]
    fn argument_order_site_provenance_and_form_are_enforced() {
        let mut wrong_order = fixture();
        wrong_order.input.arguments[1].ordinal = 2;
        assert!(matches!(
            build(&wrong_order),
            Err(SourceTypeError::ReorderedArgument { .. })
        ));

        let mut decreasing_parent = fixture();
        decreasing_parent.input.arguments[0].parent = SourceTypeExpressionId::new(1);
        decreasing_parent.input.arguments[1].parent = SourceTypeExpressionId::new(0);
        decreasing_parent.input.arguments[1].ordinal = 0;
        assert!(matches!(
            build(&decreasing_parent),
            Err(SourceTypeError::ReorderedArgument { .. })
        ));

        let mut wrong_provenance = fixture();
        let SourceTypeArgument::QuaSite {
            ref mut provenance, ..
        } = wrong_provenance.input.arguments[1].argument
        else {
            unreachable!()
        };
        *provenance = SemanticOrigin::new(
            wrong_provenance.source,
            wrong_provenance.module.clone(),
            SourceAnchor::Range(range(wrong_provenance.source, 40, 41)),
            vec![0, 0],
        );
        assert!(matches!(
            build(&wrong_provenance),
            Err(SourceTypeError::InvalidProvenance { .. })
        ));

        let mut wrong_form = fixture();
        wrong_form.input.expressions[0].form = SourceTypeApplicationForm::Bare;
        assert!(matches!(
            build(&wrong_form),
            Err(SourceTypeError::WrongApplicationForm { .. })
        ));

        let mut overlapping = fixture();
        let SourceTypeArgument::QuaSite {
            ref mut source_range,
            ..
        } = overlapping.input.arguments[1].argument
        else {
            unreachable!()
        };
        *source_range = range(overlapping.source, 25, 26);
        overlapping.arena = arena_for(&overlapping.input);
        assert!(matches!(
            build(&overlapping),
            Err(SourceTypeError::InvalidProvenance { .. })
                | Err(SourceTypeError::OverlappingSiblings { .. })
        ));
    }

    #[test]
    fn dangling_duplicate_and_multiple_parent_children_are_rejected() {
        let mut dangling = fixture();
        dangling.input.arguments[0].argument = SourceTypeArgument::TypeSite {
            expression: SourceTypeExpressionId::new(99),
        };
        assert!(matches!(
            build(&dangling),
            Err(SourceTypeError::DanglingChild { .. })
        ));

        let mut duplicate = fixture();
        let SourceTypeArgument::QuaSite { radix, .. } = &mut duplicate.input.arguments[1].argument
        else {
            unreachable!()
        };
        *radix = vec![SourceTypeExpressionId::new(1)];
        assert!(matches!(
            build(&duplicate),
            Err(SourceTypeError::DuplicateChild { .. })
        ));

        let mut multiple = fixture();
        multiple.input.expressions[1].form = SourceTypeApplicationForm::Bracket;
        multiple.input.arguments.push(SourceTypeArgumentInput {
            parent: SourceTypeExpressionId::new(1),
            ordinal: 0,
            argument: SourceTypeArgument::TypeSite {
                expression: SourceTypeExpressionId::new(2),
            },
        });
        assert!(matches!(
            build(&multiple),
            Err(SourceTypeError::MultipleParents { .. })
        ));
    }

    #[test]
    fn cycles_forward_parents_and_unreachable_expressions_are_rejected() {
        let mut cycle = fixture();
        cycle.input.expressions[1].form = SourceTypeApplicationForm::Bracket;
        cycle.input.expressions[2].form = SourceTypeApplicationForm::Bracket;
        cycle.input.expressions[1].source_range = range(cycle.source, 20, 70);
        cycle.input.expressions[2].source_range = range(cycle.source, 20, 70);
        cycle.input.arguments = vec![
            SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(1),
                ordinal: 0,
                argument: SourceTypeArgument::TypeSite {
                    expression: SourceTypeExpressionId::new(2),
                },
            },
            SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(2),
                ordinal: 0,
                argument: SourceTypeArgument::TypeSite {
                    expression: SourceTypeExpressionId::new(1),
                },
            },
        ];
        refresh(&mut cycle);
        assert!(matches!(build(&cycle), Err(SourceTypeError::Cycle { .. })));

        let mut forward = fixture();
        forward.input.expressions = vec![
            bare_expression(forward.source, &forward.module, 1, 20, 30, "object"),
            {
                let mut parent = bare_expression(forward.source, &forward.module, 0, 10, 90, "set");
                parent.form = SourceTypeApplicationForm::Bracket;
                parent
            },
        ];
        forward.input.applications[0].root = SourceTypeExpressionId::new(1);
        forward.input.arguments = vec![SourceTypeArgumentInput {
            parent: SourceTypeExpressionId::new(1),
            ordinal: 0,
            argument: SourceTypeArgument::TypeSite {
                expression: SourceTypeExpressionId::new(0),
            },
        }];
        refresh(&mut forward);
        assert!(matches!(
            build(&forward),
            Err(SourceTypeError::ForwardParent { .. })
        ));

        let mut unreachable = fixture();
        unreachable.input.arguments.clear();
        unreachable.input.expressions[0].form = SourceTypeApplicationForm::Bare;
        refresh(&mut unreachable);
        assert!(matches!(
            build(&unreachable),
            Err(SourceTypeError::UnreachableExpression { .. })
        ));
    }

    #[test]
    fn root_parent_conflict_is_rejected_before_general_graph_traversal() {
        let fixture = two_root_fixture();
        let roots = BTreeSet::from([
            SourceTypeExpressionId::new(0),
            SourceTypeExpressionId::new(1),
        ]);
        let parents = vec![None, Some(SourceTypeExpressionId::new(0))];
        let children = vec![vec![SourceTypeExpressionId::new(1)], Vec::new()];
        assert_eq!(
            validate_graph(&fixture.input, &roots, &parents, &children),
            Err(SourceTypeError::RootHasParent {
                root: SourceTypeExpressionId::new(1),
            })
        );
    }

    #[test]
    fn deep_forward_graph_is_validated_iteratively_without_stack_growth() {
        const DEPTH: usize = 10_000;

        let source = source_id();
        let module = module("source.type.deep");
        let mut expressions = Vec::with_capacity(DEPTH);
        let mut arguments = Vec::with_capacity(DEPTH - 1);
        for index in 0..DEPTH {
            let start = index + 1;
            let end = DEPTH * 3 - index;
            let mut expression = bare_expression(source, &module, index, start, end, "set");
            if index + 1 < DEPTH {
                expression.form = SourceTypeApplicationForm::Bracket;
                arguments.push(SourceTypeArgumentInput {
                    parent: SourceTypeExpressionId::new(index),
                    ordinal: 0,
                    argument: SourceTypeArgument::TypeSite {
                        expression: SourceTypeExpressionId::new(index + 1),
                    },
                });
            }
            expressions.push(expression);
        }
        let input = SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            }],
            expressions,
            arguments,
        };
        let bindings = binding_env(source, &module, &[input.expressions[0].source_range]);
        let symbols = SymbolEnv::new(module, SymbolEnvIndexes::default());
        let arena = arena_for(&input);
        let handoff = SourceTypeProducer::build(input, &bindings, &symbols, &arena)
            .expect("deep forward source-type graph");
        assert_eq!(handoff.expressions().len(), DEPTH);
        assert_eq!(handoff.arguments().len(), DEPTH - 1);
    }

    #[test]
    fn parent_sibling_and_top_level_ranges_are_enforced() {
        let mut outside = fixture();
        outside.input.expressions[1].source_range = range(outside.source, 95, 105);
        outside.input.expressions[1].head_range = range(outside.source, 95, 101);
        refresh(&mut outside);
        assert!(matches!(
            build(&outside),
            Err(SourceTypeError::InvalidArgument { .. })
                | Err(SourceTypeError::ChildOutsideParent { .. })
        ));

        let mut siblings = fixture();
        let SourceTypeArgument::QuaSite {
            source_range,
            provenance,
            ..
        } = &mut siblings.input.arguments[1].argument
        else {
            unreachable!()
        };
        *source_range = range(siblings.source, 25, 26);
        *provenance = SemanticOrigin::new(
            siblings.source,
            siblings.module.clone(),
            SourceAnchor::Range(*source_range),
            vec![0, 1],
        );
        refresh(&mut siblings);
        assert!(matches!(
            build(&siblings),
            Err(SourceTypeError::OverlappingSiblings { .. })
        ));

        let mut top_level = fixture();
        top_level.input.expressions = vec![
            bare_expression(top_level.source, &top_level.module, 0, 10, 30, "set"),
            bare_expression(top_level.source, &top_level.module, 1, 20, 40, "object"),
        ];
        top_level.input.applications = vec![
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
        ];
        top_level.input.arguments.clear();
        refresh(&mut top_level);
        assert!(matches!(
            build(&top_level),
            Err(SourceTypeError::OverlappingApplications { .. })
        ));
    }

    #[test]
    fn typed_ast_installation_rechecks_the_actual_arena() {
        let fixture = fixture();
        let handoff = build(&fixture).expect("handoff");
        handoff
            .validate_installation(fixture.source, &fixture.module, &fixture.arena)
            .expect("same arena");
        let empty = TypedArena::try_new(None, Vec::new()).expect("empty arena");
        assert!(matches!(
            handoff.validate_installation(fixture.source, &fixture.module, &empty),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));
        assert_eq!(
            handoff.validate_installation(fixture.source, &module("wrong"), &fixture.arena),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let typed = TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: Some(handoff.clone()),
            nodes: fixture.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("TypedAst source type ownership");
        assert_eq!(typed.source_type(), Some(&handoff));

        assert_eq!(
            TypedAst::try_new(TypedAstParts {
                source_id: fixture.source,
                module_id: fixture.module,
                resolved_root: None,
                source_context: None,
                source_type: Some(handoff),
                nodes: empty,
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: InitialObligationTable::new(),
                diagnostics: TypeDiagnosticTable::new(),
            }),
            Err(TypedAstError::InvalidSourceType)
        );
    }

    #[test]
    fn legacy_input_only_seam_returns_the_real_binding_env() {
        let source = source_id();
        let module = module("legacy.reserve");
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let bridge = SourceReserveDeclarationBridge::new(
            source,
            module,
            range(source, 0, 20),
            vec![SourceReserveBindingInput::new(
                "x",
                range(source, 1, 2),
                range(source, 10, 13),
                "set",
                TypeHeadInput::BuiltinSet,
            )],
        )
        .expect("bridge");
        let bindings = bridge.prepare_binding_env(&symbols).expect("binding env");
        assert_eq!(bindings.bindings().len(), 1);
        assert_eq!(
            bindings
                .bindings()
                .get(BindingId::new(0))
                .map(|entry| &entry.type_site),
            Some(&BindingTypeSite::Source(range(source, 10, 13)))
        );
        assert!(bindings.diagnostics().is_empty());
        let checked = bridge.check(&symbols).expect("legacy semantic bridge");
        assert_eq!(checked.binding_env(), &bindings);
    }

    #[test]
    fn generated_declaration_context_is_not_authenticated() {
        let source = source_id();
        let module = module("source.context");
        let expressions = vec![
            bare_expression(source, &module, 0, 10, 13, "set"),
            bare_expression(source, &module, 1, 30, 33, "set"),
        ];
        let input = SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
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
            expressions,
            arguments: Vec::new(),
        };
        let reserve_range = range(source, 1, 2);
        let parameter_range = range(source, 20, 21);
        let local_scope = LocalTermScope::new(vec![1]);
        let mut bindings = BindingTable::new();
        bindings.insert(BindingDraft {
            spelling: "r".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "r".to_owned(),
                declaration_range: reserve_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range: reserve_range,
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(input.expressions[0].source_range),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::DefinitionParameter,
            identity: BinderIdentity::ResolverLocal {
                scope: local_scope.clone(),
                ordinal: 1,
                declaration_range: parameter_range,
            },
            owner_context: BindingContextId::new(1),
            declaration_range: parameter_range,
            visible_after_ordinal: 1,
            type_site: BindingTypeSite::Source(input.expressions[1].source_range),
            status: BindingStatus::Active,
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
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("definition".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Declaration,
            lexical_scope: Some(local_scope),
            bindings: vec![BindingId::new(1)],
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: BindingContextRecovery::Normal,
        });
        let bindings = BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task248-shaped binding environment");
        let symbols = SymbolEnv::new(module, SymbolEnvIndexes::default());
        let arena = arena_for(&input);
        assert!(matches!(
            SourceTypeProducer::build(input, &bindings, &symbols, &arena),
            Err(SourceTypeError::InvalidBinding { .. })
        ));
    }

    #[test]
    fn production_boundary_stays_syntax_free_and_has_no_semantic_result_payloads() {
        let source = include_str!("source_type.rs");
        for forbidden in [
            concat!("mizar", "_syntax"),
            concat!("Surface", "NodeId"),
            concat!("Normalized", "Type"),
            concat!("Declaration", "CheckingOutput"),
            concat!("Accepted", "Fact"),
            concat!("Proof", "Context"),
        ] {
            assert!(
                !source[..source.find("#[cfg(test)]").expect("test module")].contains(forbidden),
                "source type handoff exposes {forbidden}"
            );
        }
    }
}
