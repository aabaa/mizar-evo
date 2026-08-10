//! Neutral transport from resolver-owned template type-parameter links to an
//! existing typed arena.

use crate::typed_ast::{NodeRecoveryState, TypedAst, TypedNode, TypedNodeId};
use mizar_resolve::{
    names::{
        FraenkelGeneratorVariableBindingId, FraenkelGeneratorVariableSourceCollection,
        FraenkelGeneratorVariableUseLink, FraenkelGeneratorVariableUseRole,
        TemplateTypeParameterBindingId, TemplateTypeParameterSourceCollection,
    },
    resolved_ast::{ModuleId, ResolvedNodeId},
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::{
    error::Error,
    fmt::{self, Write as _},
};

/// Stable dense id for a resolver-link association.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceTemplateTypeParameterAssociationId(usize);

impl SourceTemplateTypeParameterAssociationId {
    /// Creates an id from its deterministic table index.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based table index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// One exact resolver-to-typed association for a template type parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateTypeParameterAssociation {
    binding: TemplateTypeParameterBindingId,
    definition_block: TypedNodeId,
    parameter: TypedNodeId,
    binder: TypedNodeId,
    type_head: TypedNodeId,
    identifier: TypedNodeId,
    parameter_range: SourceRange,
    type_head_range: SourceRange,
    parameter_source_ordinal: usize,
    type_head_source_ordinal: usize,
}

impl SourceTemplateTypeParameterAssociation {
    /// Returns the resolver-owned declaration binding.
    #[must_use]
    pub const fn binding(&self) -> TemplateTypeParameterBindingId {
        self.binding
    }

    /// Returns the typed definition-block node.
    #[must_use]
    pub const fn definition_block(&self) -> TypedNodeId {
        self.definition_block
    }

    /// Returns the typed template-parameter node.
    #[must_use]
    pub const fn parameter(&self) -> TypedNodeId {
        self.parameter
    }

    /// Returns the typed declaration identifier node.
    #[must_use]
    pub const fn binder(&self) -> TypedNodeId {
        self.binder
    }

    /// Returns the typed generator type-head node.
    #[must_use]
    pub const fn type_head(&self) -> TypedNodeId {
        self.type_head
    }

    /// Returns the typed generator identifier node.
    #[must_use]
    pub const fn identifier(&self) -> TypedNodeId {
        self.identifier
    }

    /// Returns the resolver-owned template-parameter range.
    #[must_use]
    pub const fn parameter_range(&self) -> SourceRange {
        self.parameter_range
    }

    /// Returns the resolver-owned generator type-head range.
    #[must_use]
    pub const fn type_head_range(&self) -> SourceRange {
        self.type_head_range
    }

    /// Returns the resolver-owned template-parameter source ordinal.
    #[must_use]
    pub const fn parameter_source_ordinal(&self) -> usize {
        self.parameter_source_ordinal
    }

    /// Returns the resolver-owned generator type-head source ordinal.
    #[must_use]
    pub const fn type_head_source_ordinal(&self) -> usize {
        self.type_head_source_ordinal
    }
}

/// Immutable dense associations in resolver generator-link order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateTypeParameterAssociationTable {
    rows: Vec<SourceTemplateTypeParameterAssociation>,
}

impl SourceTemplateTypeParameterAssociationTable {
    /// Returns the association with `id`, when present.
    #[must_use]
    pub fn get(
        &self,
        id: SourceTemplateTypeParameterAssociationId,
    ) -> Option<&SourceTemplateTypeParameterAssociation> {
        self.rows.get(id.index())
    }

    /// Iterates associations in resolver generator-link order.
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceTemplateTypeParameterAssociationId,
            &SourceTemplateTypeParameterAssociation,
        ),
    > {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceTemplateTypeParameterAssociationId::new(index), row))
    }

    /// Returns the number of associations.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns whether no associations were collected.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Complete neutral handoff for one source/module typed arena.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateTypeParameterAssociationHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    associations: SourceTemplateTypeParameterAssociationTable,
}

impl SourceTemplateTypeParameterAssociationHandoff {
    /// Returns the source represented by this handoff.
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// Returns the canonical module represented by this handoff.
    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    /// Returns the immutable resolver-to-typed associations.
    #[must_use]
    pub const fn associations(&self) -> &SourceTemplateTypeParameterAssociationTable {
        &self.associations
    }

    /// Renders deterministic debug-only association provenance.
    #[must_use]
    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-template-type-parameter-association-debug-v1\n");
        let _ = writeln!(
            output,
            "module: {}::{}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str()
        );
        for (id, row) in self.associations.iter() {
            let _ = writeln!(
                output,
                "association#{} binding={} definition_block={} parameter={} binder={} type_head={} identifier={} parameter_range={}..{} type_head_range={}..{} parameter_ordinal={} type_head_ordinal={}",
                id.index(),
                row.binding.index(),
                row.definition_block.index(),
                row.parameter.index(),
                row.binder.index(),
                row.type_head.index(),
                row.identifier.index(),
                row.parameter_range.start,
                row.parameter_range.end,
                row.type_head_range.start,
                row.type_head_range.end,
                row.parameter_source_ordinal,
                row.type_head_source_ordinal,
            );
        }
        output
    }
}

/// A failed resolver-to-typed structural association.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceTemplateTypeParameterAssociationError {
    /// The resolver collection and typed AST do not describe one environment.
    EnvironmentMismatch,
    /// One resolver generator link has no exact valid typed association.
    InvalidAssociation {
        /// The dense resolver-link association that failed validation.
        association: SourceTemplateTypeParameterAssociationId,
    },
}

impl fmt::Display for SourceTemplateTypeParameterAssociationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => formatter
                .write_str("source template type-parameter association environment mismatch"),
            Self::InvalidAssociation { association } => write!(
                formatter,
                "source template type-parameter association {} is invalid",
                association.index()
            ),
        }
    }
}

impl Error for SourceTemplateTypeParameterAssociationError {}

/// Builds neutral associations from resolver-owned template type-parameter links.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceTemplateTypeParameterAssociationProducer;

impl SourceTemplateTypeParameterAssociationProducer {
    /// Validates exact resolver-to-typed structural associations.
    pub fn build(
        collection: &TemplateTypeParameterSourceCollection,
        typed_ast: &TypedAst,
    ) -> Result<
        SourceTemplateTypeParameterAssociationHandoff,
        SourceTemplateTypeParameterAssociationError,
    > {
        if collection.source_id() != typed_ast.source_id()
            || collection.module() != typed_ast.module_id()
        {
            return Err(SourceTemplateTypeParameterAssociationError::EnvironmentMismatch);
        }

        let mut rows = Vec::with_capacity(collection.generator_links().len());
        for (index, link) in collection.generator_links().iter().enumerate() {
            let association = SourceTemplateTypeParameterAssociationId::new(index);
            let Some(binding) = collection.bindings().get(link.binding()) else {
                return Err(invalid_association(association));
            };
            if binding.definition_block() != link.definition_block() {
                return Err(invalid_association(association));
            }

            let Some(definition_block) = exact_resolved_node(typed_ast, binding.definition_block())
            else {
                return Err(invalid_association(association));
            };
            let Some(parameter) = exact_resolved_node(typed_ast, binding.parameter()) else {
                return Err(invalid_association(association));
            };
            let Some(binder) = exact_resolved_node(typed_ast, binding.binder()) else {
                return Err(invalid_association(association));
            };
            let Some(type_head) = exact_resolved_node(typed_ast, link.type_head()) else {
                return Err(invalid_association(association));
            };
            let Some(identifier) = exact_resolved_node(typed_ast, link.identifier()) else {
                return Err(invalid_association(association));
            };

            let Some(definition_node) = typed_ast.nodes().node(definition_block) else {
                return Err(invalid_association(association));
            };
            let Some(parameter_node) = typed_ast.nodes().node(parameter) else {
                return Err(invalid_association(association));
            };
            let Some(binder_node) = typed_ast.nodes().node(binder) else {
                return Err(invalid_association(association));
            };
            let Some(type_head_node) = typed_ast.nodes().node(type_head) else {
                return Err(invalid_association(association));
            };
            let Some(identifier_node) = typed_ast.nodes().node(identifier) else {
                return Err(invalid_association(association));
            };

            if !normal_recovery([
                definition_node,
                parameter_node,
                binder_node,
                type_head_node,
                identifier_node,
            ]) || !exact_kinds(
                definition_node,
                parameter_node,
                binder_node,
                type_head_node,
                identifier_node,
            ) {
                return Err(invalid_association(association));
            }

            let Some(definition_range) = range_anchor(definition_node, typed_ast.source_id())
            else {
                return Err(invalid_association(association));
            };
            let Some(parameter_range) = range_anchor(parameter_node, typed_ast.source_id()) else {
                return Err(invalid_association(association));
            };
            let Some(binder_range) = range_anchor(binder_node, typed_ast.source_id()) else {
                return Err(invalid_association(association));
            };
            let Some(type_head_range) = range_anchor(type_head_node, typed_ast.source_id()) else {
                return Err(invalid_association(association));
            };
            let Some(identifier_range) = range_anchor(identifier_node, typed_ast.source_id())
            else {
                return Err(invalid_association(association));
            };
            if parameter_range != binding.source_range() || type_head_range != link.source_range() {
                return Err(invalid_association(association));
            }
            if !contains_range(parameter_range, binder_range)
                || !contains_range(definition_range, parameter_range)
                || !contains_range(definition_range, type_head_range)
                || !contains_range(type_head_range, identifier_range)
            {
                return Err(invalid_association(association));
            }
            if !definition_node.children.contains(&parameter)
                || !parameter_node.children.contains(&binder)
                || !type_head_node.children.contains(&identifier)
            {
                return Err(invalid_association(association));
            }

            rows.push(SourceTemplateTypeParameterAssociation {
                binding: link.binding(),
                definition_block,
                parameter,
                binder,
                type_head,
                identifier,
                parameter_range: binding.source_range(),
                type_head_range: link.source_range(),
                parameter_source_ordinal: binding.source_ordinal(),
                type_head_source_ordinal: link.source_ordinal(),
            });
        }

        Ok(SourceTemplateTypeParameterAssociationHandoff {
            source_id: collection.source_id(),
            module_id: collection.module().clone(),
            associations: SourceTemplateTypeParameterAssociationTable { rows },
        })
    }
}

fn invalid_association(
    association: SourceTemplateTypeParameterAssociationId,
) -> SourceTemplateTypeParameterAssociationError {
    SourceTemplateTypeParameterAssociationError::InvalidAssociation { association }
}

fn exact_resolved_node(typed_ast: &TypedAst, resolved_node: ResolvedNodeId) -> Option<TypedNodeId> {
    let mut match_id = None;
    for (typed_id, node) in typed_ast.nodes().iter() {
        if node.resolved_node == Some(resolved_node) && match_id.replace(typed_id).is_some() {
            return None;
        }
    }
    match_id
}

fn normal_recovery(nodes: [&TypedNode; 5]) -> bool {
    nodes
        .iter()
        .all(|node| node.recovery == NodeRecoveryState::Normal)
}

fn exact_kinds(
    definition_block: &TypedNode,
    parameter: &TypedNode,
    binder: &TypedNode,
    type_head: &TypedNode,
    identifier: &TypedNode,
) -> bool {
    definition_block.kind.as_str() == "DefinitionBlockItem"
        && parameter.kind.as_str() == "TemplateParameter"
        && identifier_kind(binder.kind.as_str())
        && type_head.kind.as_str() == "TypeHead"
        && identifier_kind(identifier.kind.as_str())
}

fn identifier_kind(kind: &str) -> bool {
    kind == "Identifier"
}

fn range_anchor(node: &TypedNode, source_id: SourceId) -> Option<SourceRange> {
    let SourceAnchor::Range(range) = node.anchor else {
        return None;
    };
    (range.source_id == source_id && range.start < range.end).then_some(range)
}

fn contains_range(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start <= child.start && child.end <= parent.end
}

/// Stable dense id for one template/Fraenkel structural composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceTemplateFraenkelStructuralCompositionId(usize);

impl SourceTemplateFraenkelStructuralCompositionId {
    /// Creates an id from its deterministic generator-binding source order.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based composition-table index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// One exact structural composition of a template association and Fraenkel binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateFraenkelStructuralComposition {
    template_association: SourceTemplateTypeParameterAssociationId,
    template_binding: TemplateTypeParameterBindingId,
    generator_binding: FraenkelGeneratorVariableBindingId,
    definition_block: TypedNodeId,
    parameter: TypedNodeId,
    template_binder: TypedNodeId,
    type_head: TypedNodeId,
    template_identifier: TypedNodeId,
    functor_definition: TypedNodeId,
    comprehension: TypedNodeId,
    segment: TypedNodeId,
    generator_binder: TypedNodeId,
    type_expression: TypedNodeId,
    mapper_role_owner: TypedNodeId,
    mapper_term_reference: TypedNodeId,
    mapper_identifier: TypedNodeId,
    first_condition_role_owner: TypedNodeId,
    first_condition_term_reference: TypedNodeId,
    first_condition_identifier: TypedNodeId,
    second_condition_role_owner: TypedNodeId,
    second_condition_term_reference: TypedNodeId,
    second_condition_identifier: TypedNodeId,
    mapper_source_ordinal: usize,
    mapper_role_source_ordinal: usize,
    first_condition_source_ordinal: usize,
    first_condition_role_source_ordinal: usize,
    second_condition_source_ordinal: usize,
    second_condition_role_source_ordinal: usize,
}

impl SourceTemplateFraenkelStructuralComposition {
    /// Returns the consumed template association.
    #[must_use]
    pub const fn template_association(&self) -> SourceTemplateTypeParameterAssociationId {
        self.template_association
    }

    /// Returns the resolver-owned template declaration binding.
    #[must_use]
    pub const fn template_binding(&self) -> TemplateTypeParameterBindingId {
        self.template_binding
    }

    /// Returns the resolver-owned Fraenkel generator binding.
    #[must_use]
    pub const fn generator_binding(&self) -> FraenkelGeneratorVariableBindingId {
        self.generator_binding
    }

    /// Returns the typed definition-block node.
    #[must_use]
    pub const fn definition_block(&self) -> TypedNodeId {
        self.definition_block
    }

    /// Returns the typed template-parameter node.
    #[must_use]
    pub const fn parameter(&self) -> TypedNodeId {
        self.parameter
    }

    /// Returns the typed template-binder identifier node.
    #[must_use]
    pub const fn template_binder(&self) -> TypedNodeId {
        self.template_binder
    }

    /// Returns the typed generator type-head node.
    #[must_use]
    pub const fn type_head(&self) -> TypedNodeId {
        self.type_head
    }

    /// Returns the typed template type-head identifier node.
    #[must_use]
    pub const fn template_identifier(&self) -> TypedNodeId {
        self.template_identifier
    }

    /// Returns the typed functor-definition node.
    #[must_use]
    pub const fn functor_definition(&self) -> TypedNodeId {
        self.functor_definition
    }

    /// Returns the typed set-comprehension node.
    #[must_use]
    pub const fn comprehension(&self) -> TypedNodeId {
        self.comprehension
    }

    /// Returns the typed generator segment node.
    #[must_use]
    pub const fn segment(&self) -> TypedNodeId {
        self.segment
    }

    /// Returns the typed generator-binder identifier node.
    #[must_use]
    pub const fn generator_binder(&self) -> TypedNodeId {
        self.generator_binder
    }

    /// Returns the typed generator type-expression node.
    #[must_use]
    pub const fn type_expression(&self) -> TypedNodeId {
        self.type_expression
    }

    /// Returns the typed mapper term-expression owner.
    #[must_use]
    pub const fn mapper_role_owner(&self) -> TypedNodeId {
        self.mapper_role_owner
    }

    /// Returns the typed mapper term-reference node.
    #[must_use]
    pub const fn mapper_term_reference(&self) -> TypedNodeId {
        self.mapper_term_reference
    }

    /// Returns the typed mapper identifier node.
    #[must_use]
    pub const fn mapper_identifier(&self) -> TypedNodeId {
        self.mapper_identifier
    }

    /// Returns the typed first-condition formula owner.
    #[must_use]
    pub const fn first_condition_role_owner(&self) -> TypedNodeId {
        self.first_condition_role_owner
    }

    /// Returns the typed first-condition term-reference node.
    #[must_use]
    pub const fn first_condition_term_reference(&self) -> TypedNodeId {
        self.first_condition_term_reference
    }

    /// Returns the typed first-condition identifier node.
    #[must_use]
    pub const fn first_condition_identifier(&self) -> TypedNodeId {
        self.first_condition_identifier
    }

    /// Returns the typed second-condition formula owner.
    #[must_use]
    pub const fn second_condition_role_owner(&self) -> TypedNodeId {
        self.second_condition_role_owner
    }

    /// Returns the typed second-condition term-reference node.
    #[must_use]
    pub const fn second_condition_term_reference(&self) -> TypedNodeId {
        self.second_condition_term_reference
    }

    /// Returns the typed second-condition identifier node.
    #[must_use]
    pub const fn second_condition_identifier(&self) -> TypedNodeId {
        self.second_condition_identifier
    }

    /// Returns the mapper use's global source-order ordinal.
    #[must_use]
    pub const fn mapper_source_ordinal(&self) -> usize {
        self.mapper_source_ordinal
    }

    /// Returns the mapper use's mapper-role source-order ordinal.
    #[must_use]
    pub const fn mapper_role_source_ordinal(&self) -> usize {
        self.mapper_role_source_ordinal
    }

    /// Returns the first condition use's global source-order ordinal.
    #[must_use]
    pub const fn first_condition_source_ordinal(&self) -> usize {
        self.first_condition_source_ordinal
    }

    /// Returns the first condition use's condition-role source-order ordinal.
    #[must_use]
    pub const fn first_condition_role_source_ordinal(&self) -> usize {
        self.first_condition_role_source_ordinal
    }

    /// Returns the second condition use's global source-order ordinal.
    #[must_use]
    pub const fn second_condition_source_ordinal(&self) -> usize {
        self.second_condition_source_ordinal
    }

    /// Returns the second condition use's condition-role source-order ordinal.
    #[must_use]
    pub const fn second_condition_role_source_ordinal(&self) -> usize {
        self.second_condition_role_source_ordinal
    }
}

/// Immutable dense template/Fraenkel structural compositions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateFraenkelStructuralCompositionTable {
    rows: Vec<SourceTemplateFraenkelStructuralComposition>,
}

impl SourceTemplateFraenkelStructuralCompositionTable {
    /// Returns the composition with `id`, when present.
    #[must_use]
    pub fn get(
        &self,
        id: SourceTemplateFraenkelStructuralCompositionId,
    ) -> Option<&SourceTemplateFraenkelStructuralComposition> {
        self.rows.get(id.index())
    }

    /// Iterates compositions in generator-binding source order.
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceTemplateFraenkelStructuralCompositionId,
            &SourceTemplateFraenkelStructuralComposition,
        ),
    > {
        self.rows.iter().enumerate().map(|(index, row)| {
            (
                SourceTemplateFraenkelStructuralCompositionId::new(index),
                row,
            )
        })
    }

    /// Returns the number of compositions.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns whether no compositions were built.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Complete neutral composition handoff for one source/module typed arena.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTemplateFraenkelStructuralCompositionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    compositions: SourceTemplateFraenkelStructuralCompositionTable,
}

impl SourceTemplateFraenkelStructuralCompositionHandoff {
    /// Returns the source represented by this handoff.
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// Returns the canonical module represented by this handoff.
    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    /// Returns the immutable structural compositions.
    #[must_use]
    pub const fn compositions(&self) -> &SourceTemplateFraenkelStructuralCompositionTable {
        &self.compositions
    }

    /// Renders deterministic debug-only structural provenance.
    #[must_use]
    pub fn debug_text(&self) -> String {
        format!(
            "source-template-fraenkel-structural-composition-v1|module={}.{}|compositions={}|uses={}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.compositions.len(),
            self.compositions.len() * 3,
        )
    }
}

/// A failed template/Fraenkel structural composition.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceTemplateFraenkelStructuralCompositionError {
    /// The source/module inputs and typed AST do not describe one environment.
    EnvironmentMismatch,
    /// A template association failed immutable structural revalidation.
    InvalidTemplateAssociation {
        /// The dense template association that failed validation.
        association: SourceTemplateTypeParameterAssociationId,
    },
    /// A Fraenkel generator binding failed immutable structural revalidation.
    InvalidGeneratorBinding {
        /// The dense generator binding that failed validation.
        binding: FraenkelGeneratorVariableBindingId,
    },
    /// A Fraenkel generator use failed immutable structural revalidation.
    InvalidGeneratorUse {
        /// The source-order generator-use index that failed validation.
        use_index: usize,
    },
    /// A complete generator candidate has no one-to-one structural composition.
    InvalidComposition {
        /// The dense generator-binding-order composition that failed.
        composition: SourceTemplateFraenkelStructuralCompositionId,
    },
    /// A valid template association was not consumed by any generator candidate.
    UnmatchedTemplateAssociation {
        /// The lowest dense unmatched template association.
        association: SourceTemplateTypeParameterAssociationId,
    },
}

impl fmt::Display for SourceTemplateFraenkelStructuralCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => formatter
                .write_str("source template/Fraenkel structural composition environment mismatch"),
            Self::InvalidTemplateAssociation { association } => write!(
                formatter,
                "source template/Fraenkel structural composition template association {} is invalid",
                association.index()
            ),
            Self::InvalidGeneratorBinding { binding } => write!(
                formatter,
                "source template/Fraenkel structural composition generator binding {} is invalid",
                binding.index()
            ),
            Self::InvalidGeneratorUse { use_index } => write!(
                formatter,
                "source template/Fraenkel structural composition generator use {} is invalid",
                use_index
            ),
            Self::InvalidComposition { composition } => write!(
                formatter,
                "source template/Fraenkel structural composition {} is invalid",
                composition.index()
            ),
            Self::UnmatchedTemplateAssociation { association } => write!(
                formatter,
                "source template/Fraenkel structural composition template association {} is unmatched",
                association.index()
            ),
        }
    }
}

impl Error for SourceTemplateFraenkelStructuralCompositionError {}

/// Builds neutral structural compositions from completed template and Fraenkel handoffs.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceTemplateFraenkelStructuralCompositionProducer;

impl SourceTemplateFraenkelStructuralCompositionProducer {
    /// Validates exact template/Fraenkel structural compositions.
    pub fn build(
        template: &SourceTemplateTypeParameterAssociationHandoff,
        generators: &FraenkelGeneratorVariableSourceCollection,
        typed_ast: &TypedAst,
    ) -> Result<
        SourceTemplateFraenkelStructuralCompositionHandoff,
        SourceTemplateFraenkelStructuralCompositionError,
    > {
        if template.source_id() != generators.source_id()
            || template.source_id() != typed_ast.source_id()
            || template.module_id() != generators.module()
            || template.module_id() != typed_ast.module_id()
        {
            return Err(SourceTemplateFraenkelStructuralCompositionError::EnvironmentMismatch);
        }

        let mut template_profiles = Vec::with_capacity(template.associations().len());
        for (association, row) in template.associations().iter() {
            let Some(profile) = validate_template_association(typed_ast, association, row) else {
                return Err(
                    SourceTemplateFraenkelStructuralCompositionError::InvalidTemplateAssociation {
                        association,
                    },
                );
            };
            template_profiles.push(profile);
        }

        let mut binding_profiles = Vec::with_capacity(generators.bindings().len());
        for (binding, row) in generators.bindings().iter() {
            let Some(profile) = validate_generator_binding(typed_ast, binding, row) else {
                return Err(
                    SourceTemplateFraenkelStructuralCompositionError::InvalidGeneratorBinding {
                        binding,
                    },
                );
            };
            binding_profiles.push(profile);
        }

        let mut use_profiles = Vec::with_capacity(generators.uses().len());
        let mut next_mapper_role_ordinal = 0;
        let mut next_condition_role_ordinal = 0;
        for (use_index, link) in generators.uses().iter().enumerate() {
            let expected_role_ordinal = match link.role() {
                FraenkelGeneratorVariableUseRole::Mapper => {
                    let ordinal = next_mapper_role_ordinal;
                    next_mapper_role_ordinal += 1;
                    ordinal
                }
                FraenkelGeneratorVariableUseRole::Condition => {
                    let ordinal = next_condition_role_ordinal;
                    next_condition_role_ordinal += 1;
                    ordinal
                }
                _ => return Err(invalid_generator_use(use_index)),
            };
            let Some(profile) = validate_generator_use(
                typed_ast,
                generators,
                &binding_profiles,
                use_index,
                expected_role_ordinal,
                link,
            ) else {
                return Err(invalid_generator_use(use_index));
            };
            use_profiles.push(profile);
        }

        let rows = compose_structural_rows(
            typed_ast,
            &template_profiles,
            &binding_profiles,
            &use_profiles,
        )?;

        Ok(SourceTemplateFraenkelStructuralCompositionHandoff {
            source_id: template.source_id(),
            module_id: template.module_id().clone(),
            compositions: SourceTemplateFraenkelStructuralCompositionTable { rows },
        })
    }
}

fn compose_structural_rows(
    typed_ast: &TypedAst,
    template_profiles: &[TemplateStructuralProfile],
    binding_profiles: &[GeneratorBindingStructuralProfile],
    use_profiles: &[GeneratorUseStructuralProfile],
) -> Result<
    Vec<SourceTemplateFraenkelStructuralComposition>,
    SourceTemplateFraenkelStructuralCompositionError,
> {
    let mut consumed = vec![false; template_profiles.len()];
    let mut rows = Vec::with_capacity(binding_profiles.len());
    for (index, binding) in binding_profiles.iter().enumerate() {
        let composition = SourceTemplateFraenkelStructuralCompositionId::new(index);
        let matching = template_profiles
            .iter()
            .enumerate()
            .filter_map(|(profile_index, template)| {
                (template.definition_block == binding.definition_block
                    && template.type_head == binding.type_head)
                    .then_some(profile_index)
            })
            .collect::<Vec<_>>();
        let [template_index] = matching.as_slice() else {
            return Err(invalid_composition(composition));
        };
        if consumed[*template_index] {
            return Err(invalid_composition(composition));
        }

        let mut mapper = None;
        let mut conditions = Vec::new();
        for profile in use_profiles
            .iter()
            .filter(|profile| profile.binding == binding.id)
        {
            match profile.role {
                FraenkelGeneratorVariableUseRole::Mapper if mapper.replace(profile).is_none() => {}
                FraenkelGeneratorVariableUseRole::Condition => conditions.push(profile),
                _ => return Err(invalid_composition(composition)),
            }
        }
        let Some(mapper) = mapper else {
            return Err(invalid_composition(composition));
        };
        let [first_condition, second_condition] = conditions.as_slice() else {
            return Err(invalid_composition(composition));
        };
        if mapper.source_ordinal >= first_condition.source_ordinal
            || first_condition.source_ordinal >= second_condition.source_ordinal
        {
            return Err(invalid_composition(composition));
        }

        let template = &template_profiles[*template_index];
        if !has_direct_edge(typed_ast, template.definition_block, template.parameter)
            || !has_direct_edge(
                typed_ast,
                template.definition_block,
                binding.functor_definition,
            )
        {
            return Err(invalid_composition(composition));
        }
        consumed[*template_index] = true;
        rows.push(SourceTemplateFraenkelStructuralComposition {
            template_association: template.association,
            template_binding: template.binding,
            generator_binding: binding.id,
            definition_block: template.definition_block,
            parameter: template.parameter,
            template_binder: template.binder,
            type_head: template.type_head,
            template_identifier: template.identifier,
            functor_definition: binding.functor_definition,
            comprehension: binding.comprehension,
            segment: binding.segment,
            generator_binder: binding.binder,
            type_expression: binding.type_expression,
            mapper_role_owner: mapper.role_owner,
            mapper_term_reference: mapper.term_reference,
            mapper_identifier: mapper.identifier,
            first_condition_role_owner: first_condition.role_owner,
            first_condition_term_reference: first_condition.term_reference,
            first_condition_identifier: first_condition.identifier,
            second_condition_role_owner: second_condition.role_owner,
            second_condition_term_reference: second_condition.term_reference,
            second_condition_identifier: second_condition.identifier,
            mapper_source_ordinal: mapper.source_ordinal,
            mapper_role_source_ordinal: mapper.role_source_ordinal,
            first_condition_source_ordinal: first_condition.source_ordinal,
            first_condition_role_source_ordinal: first_condition.role_source_ordinal,
            second_condition_source_ordinal: second_condition.source_ordinal,
            second_condition_role_source_ordinal: second_condition.role_source_ordinal,
        });
    }

    if let Some((association, _)) = template_profiles
        .iter()
        .enumerate()
        .find(|(index, _)| !consumed[*index])
    {
        return Err(
            SourceTemplateFraenkelStructuralCompositionError::UnmatchedTemplateAssociation {
                association: SourceTemplateTypeParameterAssociationId::new(association),
            },
        );
    }

    Ok(rows)
}

#[derive(Clone, Copy)]
struct TemplateStructuralProfile {
    association: SourceTemplateTypeParameterAssociationId,
    binding: TemplateTypeParameterBindingId,
    definition_block: TypedNodeId,
    parameter: TypedNodeId,
    binder: TypedNodeId,
    type_head: TypedNodeId,
    identifier: TypedNodeId,
}

#[derive(Clone, Copy)]
struct GeneratorBindingStructuralProfile {
    id: FraenkelGeneratorVariableBindingId,
    definition_block: TypedNodeId,
    functor_definition: TypedNodeId,
    comprehension: TypedNodeId,
    segment: TypedNodeId,
    binder: TypedNodeId,
    type_expression: TypedNodeId,
    type_head: TypedNodeId,
}

#[derive(Clone, Copy)]
struct GeneratorUseStructuralProfile {
    binding: FraenkelGeneratorVariableBindingId,
    role: FraenkelGeneratorVariableUseRole,
    role_owner: TypedNodeId,
    term_reference: TypedNodeId,
    identifier: TypedNodeId,
    source_ordinal: usize,
    role_source_ordinal: usize,
}

fn validate_template_association(
    typed_ast: &TypedAst,
    association: SourceTemplateTypeParameterAssociationId,
    row: &SourceTemplateTypeParameterAssociation,
) -> Option<TemplateStructuralProfile> {
    let definition_block = exact_stored_typed_node(typed_ast, row.definition_block())?;
    let parameter = exact_stored_typed_node(typed_ast, row.parameter())?;
    let binder = exact_stored_typed_node(typed_ast, row.binder())?;
    let type_head = exact_stored_typed_node(typed_ast, row.type_head())?;
    let identifier = exact_stored_typed_node(typed_ast, row.identifier())?;
    let nodes = [definition_block, parameter, binder, type_head, identifier];
    let node_refs = nodes.map(|(_, node)| node);
    if !all_normal(&node_refs)
        || !all_exact_kinds(
            &node_refs,
            [
                "DefinitionBlockItem",
                "TemplateParameter",
                "Identifier",
                "TypeHead",
                "Identifier",
            ],
        )
    {
        return None;
    }
    let definition_range = range_anchor(definition_block.1, typed_ast.source_id())?;
    let parameter_range = range_anchor(parameter.1, typed_ast.source_id())?;
    let binder_range = range_anchor(binder.1, typed_ast.source_id())?;
    let type_head_range = range_anchor(type_head.1, typed_ast.source_id())?;
    let identifier_range = range_anchor(identifier.1, typed_ast.source_id())?;
    if parameter_range != row.parameter_range()
        || type_head_range != row.type_head_range()
        || !contains_range(definition_range, parameter_range)
        || !contains_range(parameter_range, binder_range)
        || !contains_range(definition_range, type_head_range)
        || !contains_range(type_head_range, identifier_range)
        || !has_direct_edge(typed_ast, definition_block.0, parameter.0)
        || !has_direct_edge(typed_ast, parameter.0, binder.0)
        || !has_direct_edge(typed_ast, type_head.0, identifier.0)
    {
        return None;
    }
    Some(TemplateStructuralProfile {
        association,
        binding: row.binding(),
        definition_block: definition_block.0,
        parameter: parameter.0,
        binder: binder.0,
        type_head: type_head.0,
        identifier: identifier.0,
    })
}

fn validate_generator_binding(
    typed_ast: &TypedAst,
    id: FraenkelGeneratorVariableBindingId,
    binding: &mizar_resolve::names::FraenkelGeneratorVariableBinding,
) -> Option<GeneratorBindingStructuralProfile> {
    if binding.source_ordinal() != id.index() {
        return None;
    }
    let definition_block = exact_resolved_node(typed_ast, binding.definition_block())?;
    let functor_definition = exact_resolved_node(typed_ast, binding.functor_definition())?;
    let comprehension = exact_resolved_node(typed_ast, binding.comprehension())?;
    let segment = exact_resolved_node(typed_ast, binding.segment())?;
    let binder = exact_resolved_node(typed_ast, binding.binder())?;
    let term_definiens =
        exact_direct_child_with_kind(typed_ast, functor_definition, "TermDefiniens")?;
    let term_expression =
        exact_direct_child_with_kind(typed_ast, term_definiens, "TermExpression")?;
    if !has_direct_edge(typed_ast, term_expression, comprehension) {
        return None;
    }
    let type_expression = exact_direct_child_with_kind(typed_ast, segment, "TypeExpression")?;
    let type_head = exact_direct_child_with_kind(typed_ast, type_expression, "TypeHead")?;
    let nodes = [
        typed_ast.nodes().node(definition_block)?,
        typed_ast.nodes().node(functor_definition)?,
        typed_ast.nodes().node(comprehension)?,
        typed_ast.nodes().node(segment)?,
        typed_ast.nodes().node(binder)?,
        typed_ast.nodes().node(term_definiens)?,
        typed_ast.nodes().node(term_expression)?,
        typed_ast.nodes().node(type_expression)?,
        typed_ast.nodes().node(type_head)?,
    ];
    if !all_normal(&nodes)
        || !all_exact_kinds(
            &nodes,
            [
                "DefinitionBlockItem",
                "FunctorDefinition",
                "SetComprehension",
                "ComprehensionVariableSegment",
                "Identifier",
                "TermDefiniens",
                "TermExpression",
                "TypeExpression",
                "TypeHead",
            ],
        )
    {
        return None;
    }
    let definition_range = range_anchor(nodes[0], typed_ast.source_id())?;
    let functor_range = range_anchor(nodes[1], typed_ast.source_id())?;
    let comprehension_range = range_anchor(nodes[2], typed_ast.source_id())?;
    let segment_range = range_anchor(nodes[3], typed_ast.source_id())?;
    let binder_range = range_anchor(nodes[4], typed_ast.source_id())?;
    let term_definiens_range = range_anchor(nodes[5], typed_ast.source_id())?;
    let term_expression_range = range_anchor(nodes[6], typed_ast.source_id())?;
    let type_expression_range = range_anchor(nodes[7], typed_ast.source_id())?;
    let type_head_range = range_anchor(nodes[8], typed_ast.source_id())?;
    if segment_range != binding.segment_range()
        || binder_range != binding.binder_range()
        || !contains_range(definition_range, functor_range)
        || !contains_range(functor_range, term_definiens_range)
        || !contains_range(term_definiens_range, term_expression_range)
        || !contains_range(term_expression_range, comprehension_range)
        || !contains_range(comprehension_range, segment_range)
        || !contains_range(segment_range, binder_range)
        || !contains_range(segment_range, type_expression_range)
        || !contains_range(type_expression_range, type_head_range)
        || !has_direct_edge(typed_ast, definition_block, functor_definition)
        || !has_direct_edge(typed_ast, functor_definition, term_definiens)
        || !has_direct_edge(typed_ast, term_definiens, term_expression)
        || !has_direct_edge(typed_ast, term_expression, comprehension)
        || !has_direct_edge(typed_ast, comprehension, segment)
        || !has_direct_edge(typed_ast, segment, binder)
        || !has_direct_edge(typed_ast, segment, type_expression)
        || !has_direct_edge(typed_ast, type_expression, type_head)
    {
        return None;
    }
    Some(GeneratorBindingStructuralProfile {
        id,
        definition_block,
        functor_definition,
        comprehension,
        segment,
        binder,
        type_expression,
        type_head,
    })
}

fn validate_generator_use(
    typed_ast: &TypedAst,
    generators: &FraenkelGeneratorVariableSourceCollection,
    bindings: &[GeneratorBindingStructuralProfile],
    use_index: usize,
    expected_role_ordinal: usize,
    link: &FraenkelGeneratorVariableUseLink,
) -> Option<GeneratorUseStructuralProfile> {
    if link.source_ordinal() != use_index || link.role_source_ordinal() != expected_role_ordinal {
        return None;
    }
    let binding_row = generators.bindings().get(link.binding())?;
    let binding = bindings
        .iter()
        .find(|profile| profile.id == link.binding())?;
    if link.definition_block() != binding_row.definition_block()
        || link.functor_definition() != binding_row.functor_definition()
        || link.comprehension() != binding_row.comprehension()
    {
        return None;
    }
    let definition_block = exact_resolved_node(typed_ast, link.definition_block())?;
    let functor_definition = exact_resolved_node(typed_ast, link.functor_definition())?;
    let comprehension = exact_resolved_node(typed_ast, link.comprehension())?;
    if definition_block != binding.definition_block
        || functor_definition != binding.functor_definition
        || comprehension != binding.comprehension
    {
        return None;
    }
    let role_owner = exact_resolved_node(typed_ast, link.role_owner())?;
    let term_reference = exact_resolved_node(typed_ast, link.term_reference())?;
    let identifier = exact_resolved_node(typed_ast, link.identifier())?;
    let nodes = [
        typed_ast.nodes().node(role_owner)?,
        typed_ast.nodes().node(term_reference)?,
        typed_ast.nodes().node(identifier)?,
    ];
    if !all_normal(&nodes) || !all_exact_kinds(&nodes[1..], ["TermReference", "Identifier"]) {
        return None;
    }
    let role_range = range_anchor(nodes[0], typed_ast.source_id())?;
    let reference_range = range_anchor(nodes[1], typed_ast.source_id())?;
    let identifier_range = range_anchor(nodes[2], typed_ast.source_id())?;
    if identifier_range != link.identifier_range()
        || !contains_range(role_range, reference_range)
        || !contains_range(reference_range, identifier_range)
        || !has_direct_edge(typed_ast, term_reference, identifier)
    {
        return None;
    }
    match link.role() {
        FraenkelGeneratorVariableUseRole::Mapper => {
            if nodes[0].kind.as_str() != "TermExpression"
                || !has_direct_edge(typed_ast, binding.comprehension, role_owner)
                || !has_direct_edge(typed_ast, role_owner, term_reference)
                || !contains_range(
                    range_anchor(
                        typed_ast.nodes().node(binding.comprehension)?,
                        typed_ast.source_id(),
                    )?,
                    role_range,
                )
            {
                return None;
            }
        }
        FraenkelGeneratorVariableUseRole::Condition => {
            if nodes[0].kind.as_str() != "FormulaExpression"
                || !has_direct_edge(typed_ast, binding.comprehension, role_owner)
                || !contains_range(
                    range_anchor(
                        typed_ast.nodes().node(binding.comprehension)?,
                        typed_ast.source_id(),
                    )?,
                    role_range,
                )
            {
                return None;
            }
            let prefix = exact_direct_child_with_kind(typed_ast, role_owner, "PrefixFormula(Not)")?;
            let predicate =
                exact_direct_child_with_kind(typed_ast, prefix, "BuiltinPredicateApplication")?;
            let condition_term = exact_direct_child_with_kind_containing(
                typed_ast,
                predicate,
                "TermExpression",
                term_reference,
            )?;
            let prefix_node = typed_ast.nodes().node(prefix)?;
            let predicate_node = typed_ast.nodes().node(predicate)?;
            let condition_term_node = typed_ast.nodes().node(condition_term)?;
            if !all_normal(&[prefix_node, predicate_node, condition_term_node])
                || !has_direct_edge(typed_ast, predicate, condition_term)
                || !has_direct_edge(typed_ast, condition_term, term_reference)
                || !contains_range(
                    role_range,
                    range_anchor(prefix_node, typed_ast.source_id())?,
                )
                || !contains_range(
                    range_anchor(prefix_node, typed_ast.source_id())?,
                    range_anchor(predicate_node, typed_ast.source_id())?,
                )
                || !contains_range(
                    range_anchor(predicate_node, typed_ast.source_id())?,
                    range_anchor(condition_term_node, typed_ast.source_id())?,
                )
                || !contains_range(
                    range_anchor(condition_term_node, typed_ast.source_id())?,
                    reference_range,
                )
            {
                return None;
            }
        }
        _ => return None,
    }
    Some(GeneratorUseStructuralProfile {
        binding: link.binding(),
        role: link.role(),
        role_owner,
        term_reference,
        identifier,
        source_ordinal: link.source_ordinal(),
        role_source_ordinal: link.role_source_ordinal(),
    })
}

fn exact_stored_typed_node(
    typed_ast: &TypedAst,
    id: TypedNodeId,
) -> Option<(TypedNodeId, &TypedNode)> {
    let node = typed_ast.nodes().node(id)?;
    let resolved = node.resolved_node?;
    (exact_resolved_node(typed_ast, resolved) == Some(id)).then_some((id, node))
}

fn all_normal(nodes: &[&TypedNode]) -> bool {
    nodes
        .iter()
        .all(|node| node.recovery == NodeRecoveryState::Normal)
}

fn all_exact_kinds<const N: usize>(nodes: &[&TypedNode], expected: [&str; N]) -> bool {
    nodes.len() == N
        && nodes
            .iter()
            .zip(expected)
            .all(|(node, kind)| node.kind.as_str() == kind)
}

fn exact_direct_child_with_kind(
    typed_ast: &TypedAst,
    parent: TypedNodeId,
    kind: &str,
) -> Option<TypedNodeId> {
    let parent = typed_ast.nodes().node(parent)?;
    let mut matches = parent.children.iter().filter_map(|child| {
        let node = typed_ast.nodes().node(*child)?;
        (node.kind.as_str() == kind).then_some(*child)
    });
    let child = matches.next()?;
    (matches.next().is_none()
        && typed_ast
            .nodes()
            .node(child)
            .is_some_and(|node| node.recovery == NodeRecoveryState::Normal))
    .then_some(child)
}

fn exact_direct_child_with_kind_containing(
    typed_ast: &TypedAst,
    parent: TypedNodeId,
    kind: &str,
    descendant: TypedNodeId,
) -> Option<TypedNodeId> {
    let parent = typed_ast.nodes().node(parent)?;
    let mut matches = parent.children.iter().filter_map(|child| {
        let node = typed_ast.nodes().node(*child)?;
        (node.kind.as_str() == kind && node.children.contains(&descendant)).then_some(*child)
    });
    let child = matches.next()?;
    (matches.next().is_none()
        && typed_ast
            .nodes()
            .node(child)
            .is_some_and(|node| node.recovery == NodeRecoveryState::Normal))
    .then_some(child)
}

fn has_direct_edge(typed_ast: &TypedAst, parent: TypedNodeId, child: TypedNodeId) -> bool {
    typed_ast
        .nodes()
        .node(parent)
        .is_some_and(|node| node.children.contains(&child))
}

fn invalid_generator_use(use_index: usize) -> SourceTemplateFraenkelStructuralCompositionError {
    SourceTemplateFraenkelStructuralCompositionError::InvalidGeneratorUse { use_index }
}

fn invalid_composition(
    composition: SourceTemplateFraenkelStructuralCompositionId,
) -> SourceTemplateFraenkelStructuralCompositionError {
    SourceTemplateFraenkelStructuralCompositionError::InvalidComposition { composition }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
        TypeFactTable, TypeTable, TypedArena, TypedArenaBuilder, TypedAstParts,
    };
    use mizar_resolve::{
        names::{FraenkelGeneratorVariableSourceCollector, TemplateTypeParameterSourceCollector},
        resolved_ast::SurfaceResolvedArena,
    };
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator as _,
    };
    use mizar_syntax as syntax;

    #[test]
    fn task277bl_maps_exact_resolver_association_to_typed_nodes() {
        let fixture = fixture();
        let handoff = SourceTemplateTypeParameterAssociationProducer::build(
            &fixture.collection,
            &fixture.typed_ast,
        )
        .expect("Task277B-L exact resolver association must build");

        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.associations().len(), 1);
        assert!(!handoff.associations().is_empty());
        let association = handoff
            .associations()
            .get(SourceTemplateTypeParameterAssociationId::new(0))
            .expect("Task277B-L association row");
        assert_eq!(
            association.binding(),
            TemplateTypeParameterBindingId::new(0)
        );
        assert_eq!(association.definition_block(), TypedNodeId::new(4));
        assert_eq!(association.parameter(), TypedNodeId::new(1));
        assert_eq!(association.binder(), TypedNodeId::new(0));
        assert_eq!(association.type_head(), TypedNodeId::new(3));
        assert_eq!(association.identifier(), TypedNodeId::new(2));
        assert_eq!(association.parameter_range(), range(fixture.source, 11, 25));
        assert_eq!(association.type_head_range(), range(fixture.source, 48, 49));
        assert_eq!(association.parameter_source_ordinal(), 0);
        assert_eq!(association.type_head_source_ordinal(), 0);
        assert_eq!(
            handoff
                .associations()
                .iter()
                .map(|(id, row)| (id, row.binding()))
                .collect::<Vec<_>>(),
            vec![(
                SourceTemplateTypeParameterAssociationId::new(0),
                TemplateTypeParameterBindingId::new(0),
            )]
        );
        assert!(handoff.debug_text().contains("association#0 binding=0"));
    }

    #[test]
    fn task277bl_rejects_environment_missing_and_ambiguous_resolved_nodes() {
        let fixture = fixture();
        let other_source = other_source();
        let mut malformed_source_nodes = fixture.nodes.clone();
        malformed_source_nodes[1].kind = "Other".into();
        let wrong_source = typed_ast_from_nodes(
            other_source,
            fixture.module.clone(),
            nodes_for_source(malformed_source_nodes, other_source),
        );
        assert!(matches!(
            SourceTemplateTypeParameterAssociationProducer::build(
                &fixture.collection,
                &wrong_source
            ),
            Err(SourceTemplateTypeParameterAssociationError::EnvironmentMismatch)
        ));
        let mut malformed_module_nodes = fixture.nodes.clone();
        malformed_module_nodes[1].kind = "Other".into();
        let wrong_module = typed_ast_from_nodes(
            fixture.source,
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("other")),
            malformed_module_nodes,
        );
        assert!(matches!(
            SourceTemplateTypeParameterAssociationProducer::build(
                &fixture.collection,
                &wrong_module
            ),
            Err(SourceTemplateTypeParameterAssociationError::EnvironmentMismatch)
        ));

        for (node_index, resolved_node) in association_sites(&fixture) {
            let mut missing_nodes = fixture.nodes.clone();
            missing_nodes[node_index].resolved_node = None;
            let missing =
                typed_ast_from_nodes(fixture.source, fixture.module.clone(), missing_nodes);
            assert_invalid_at(
                &fixture.collection,
                &missing,
                SourceTemplateTypeParameterAssociationId::new(0),
            );

            let mut ambiguous_nodes = fixture.nodes.clone();
            ambiguous_nodes.push(
                TypedNode::new(
                    "Unrelated",
                    SourceAnchor::Range(range(fixture.source, 60, 61)),
                )
                .with_resolved_node(resolved_node),
            );
            let ambiguous =
                typed_ast_from_nodes(fixture.source, fixture.module.clone(), ambiguous_nodes);
            assert_invalid_at(
                &fixture.collection,
                &ambiguous,
                SourceTemplateTypeParameterAssociationId::new(0),
            );
        }
    }

    #[test]
    fn task277bl_rejects_kind_range_recovery_and_direct_edge_corruption() {
        let fixture = fixture();
        for (node_index, _) in association_sites(&fixture) {
            let invalid_kind = mutated_typed_ast(&fixture, |nodes| {
                nodes[node_index].kind = "Other".into();
            });
            assert_invalid_at(
                &fixture.collection,
                &invalid_kind,
                SourceTemplateTypeParameterAssociationId::new(0),
            );
            let recovered = mutated_typed_ast(&fixture, |nodes| {
                nodes[node_index].recovery = NodeRecoveryState::Recovered;
            });
            assert_invalid_at(
                &fixture.collection,
                &recovered,
                SourceTemplateTypeParameterAssociationId::new(0),
            );
        }
        for node_index in [0, 2] {
            for forged_kind in [
                "Token(SurfaceToken { kind: Identifier,",
                "Identifier trailing data",
            ] {
                let forged_identifier = mutated_typed_ast(&fixture, |nodes| {
                    nodes[node_index].kind = forged_kind.into();
                });
                assert_invalid_at(
                    &fixture.collection,
                    &forged_identifier,
                    SourceTemplateTypeParameterAssociationId::new(0),
                );
            }
        }

        for node_index in 0..5 {
            let non_range = mutated_typed_ast(&fixture, |nodes| {
                nodes[node_index].anchor = SourceAnchor::Point {
                    source_id: fixture.source,
                    offset: 0,
                };
            });
            assert_invalid_at(
                &fixture.collection,
                &non_range,
                SourceTemplateTypeParameterAssociationId::new(0),
            );
            let mut wrong_source_nodes = fixture.nodes.clone();
            wrong_source_nodes[node_index].anchor =
                SourceAnchor::Range(range(other_source(), 1, 2));
            assert!(matches!(
                try_typed_ast_from_nodes(
                    fixture.source,
                    fixture.module.clone(),
                    wrong_source_nodes
                ),
                Err(crate::typed_ast::TypedAstError::PayloadSourceMismatch)
            ));
            let empty = mutated_typed_ast(&fixture, |nodes| {
                nodes[node_index].anchor = SourceAnchor::Range(range(fixture.source, 1, 1));
            });
            assert_invalid_at(
                &fixture.collection,
                &empty,
                SourceTemplateTypeParameterAssociationId::new(0),
            );
        }

        for node_index in [1, 3] {
            let range_mismatch = mutated_typed_ast(&fixture, |nodes| {
                nodes[node_index].anchor = SourceAnchor::Range(range(fixture.source, 30, 31));
            });
            assert_invalid_at(
                &fixture.collection,
                &range_mismatch,
                SourceTemplateTypeParameterAssociationId::new(0),
            );
        }

        let binder_not_contained = mutated_typed_ast(&fixture, |nodes| {
            nodes[0].anchor = SourceAnchor::Range(range(fixture.source, 26, 27));
        });
        assert_invalid_at(
            &fixture.collection,
            &binder_not_contained,
            SourceTemplateTypeParameterAssociationId::new(0),
        );
        let definition_excludes_parameter = mutated_typed_ast(&fixture, |nodes| {
            nodes[4].anchor = SourceAnchor::Range(range(fixture.source, 30, 60));
        });
        assert_invalid_at(
            &fixture.collection,
            &definition_excludes_parameter,
            SourceTemplateTypeParameterAssociationId::new(0),
        );
        let definition_excludes_type_head = mutated_typed_ast(&fixture, |nodes| {
            nodes[4].anchor = SourceAnchor::Range(range(fixture.source, 0, 30));
        });
        assert_invalid_at(
            &fixture.collection,
            &definition_excludes_type_head,
            SourceTemplateTypeParameterAssociationId::new(0),
        );
        let type_head_excludes_identifier = mutated_typed_ast(&fixture, |nodes| {
            nodes[2].anchor = SourceAnchor::Range(range(fixture.source, 49, 50));
        });
        assert_invalid_at(
            &fixture.collection,
            &type_head_excludes_identifier,
            SourceTemplateTypeParameterAssociationId::new(0),
        );
        for (parent, child) in [
            (TypedNodeId::new(4), TypedNodeId::new(1)),
            (TypedNodeId::new(1), TypedNodeId::new(0)),
            (TypedNodeId::new(3), TypedNodeId::new(2)),
        ] {
            let detached = mutated_typed_ast(&fixture, |nodes| {
                nodes[parent.index()].children.retain(|id| *id != child);
            });
            assert_invalid_at(
                &fixture.collection,
                &detached,
                SourceTemplateTypeParameterAssociationId::new(0),
            );
        }
    }

    #[test]
    fn task277bl_rebuilds_deterministically_without_mutating_typed_ast() {
        let fixture = fixture();
        let before = fixture.typed_ast.clone();
        let first = SourceTemplateTypeParameterAssociationProducer::build(
            &fixture.collection,
            &fixture.typed_ast,
        )
        .expect("first Task277B-L build");
        let second = SourceTemplateTypeParameterAssociationProducer::build(
            &fixture.collection,
            &fixture.typed_ast,
        )
        .expect("second Task277B-L build");

        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(fixture.typed_ast, before);

        let (empty_collection, empty_typed_ast) = empty_profile();
        let empty = SourceTemplateTypeParameterAssociationProducer::build(
            &empty_collection,
            &empty_typed_ast,
        )
        .expect("Task277B-L valid empty collection");
        assert!(empty.associations().is_empty());
        assert_eq!(empty.associations().len(), 0);
        assert!(
            empty
                .associations()
                .get(SourceTemplateTypeParameterAssociationId::new(0))
                .is_none()
        );

        let multi = multi_link_fixture();
        let multi_before = multi.typed_ast.clone();
        let first_multi = SourceTemplateTypeParameterAssociationProducer::build(
            &multi.collection,
            &multi.typed_ast,
        )
        .expect("first Task277B-L multi-link build");
        let second_multi = SourceTemplateTypeParameterAssociationProducer::build(
            &multi.collection,
            &multi.typed_ast,
        )
        .expect("second Task277B-L multi-link build");
        assert_eq!(first_multi, second_multi);
        assert_eq!(multi.typed_ast, multi_before);
        assert_eq!(multi.collection.bindings().len(), 2);
        assert_eq!(
            multi
                .collection
                .bindings()
                .iter()
                .map(|(id, binding)| (id.index(), binding.source_ordinal()))
                .collect::<Vec<_>>(),
            vec![(0, 0), (1, 1)]
        );
        assert_eq!(first_multi.associations().len(), 2);
        assert_eq!(
            first_multi
                .associations()
                .iter()
                .map(|(id, row)| (
                    id.index(),
                    row.binding().index(),
                    row.parameter_source_ordinal(),
                    row.type_head_source_ordinal()
                ))
                .collect::<Vec<_>>(),
            vec![(0, 1, 1, 0), (1, 0, 0, 1)]
        );
        assert!(
            first_multi
                .associations()
                .get(SourceTemplateTypeParameterAssociationId::new(2))
                .is_none()
        );
        let corrupt_second_link = mutated_typed_ast(&multi, |nodes| {
            nodes[3].recovery = NodeRecoveryState::Recovered;
        });
        assert_invalid_at(
            &multi.collection,
            &corrupt_second_link,
            SourceTemplateTypeParameterAssociationId::new(1),
        );
    }

    #[test]
    fn task277c_composes_exact_template_fraenkel_structural_handoff() {
        let fixture = structural_fixture();
        let handoff = SourceTemplateFraenkelStructuralCompositionProducer::build(
            &fixture.template,
            &fixture.generators,
            &fixture.typed_ast,
        )
        .expect("Task277C exact structural composition must build");

        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.compositions().len(), 1);
        assert!(!handoff.compositions().is_empty());
        let composition_rows = handoff.compositions().iter().collect::<Vec<_>>();
        let [(composition_id, row)] = composition_rows.as_slice() else {
            panic!("Task277C fixture must produce one composition");
        };
        assert_eq!(composition_id.index(), 0);
        assert_eq!(row.template_association().index(), 0);
        assert_eq!(row.template_binding().index(), 0);
        assert_eq!(row.generator_binding().index(), 0);

        let association = fixture
            .template
            .associations()
            .get(SourceTemplateTypeParameterAssociationId::new(0))
            .expect("Task277C template association");
        let binding = fixture
            .generators
            .bindings()
            .get(FraenkelGeneratorVariableBindingId::new(0))
            .expect("Task277C generator binding");
        let uses = fixture.generators.uses().iter().collect::<Vec<_>>();
        assert_eq!(uses.len(), 3);
        assert_eq!(binding.spelling(), "x");
        assert_eq!(row.definition_block(), association.definition_block());
        assert_eq!(row.parameter(), association.parameter());
        assert_eq!(row.template_binder(), association.binder());
        assert_eq!(row.type_head(), association.type_head());
        assert_eq!(row.template_identifier(), association.identifier());
        assert_eq!(
            row.functor_definition(),
            typed_for_resolved(&fixture.typed_ast, binding.functor_definition())
        );
        assert_eq!(
            row.comprehension(),
            typed_for_resolved(&fixture.typed_ast, binding.comprehension())
        );
        assert_eq!(
            row.segment(),
            typed_for_resolved(&fixture.typed_ast, binding.segment())
        );
        assert_eq!(
            row.generator_binder(),
            typed_for_resolved(&fixture.typed_ast, binding.binder())
        );
        assert_eq!(
            row.mapper_role_owner(),
            typed_for_resolved(&fixture.typed_ast, uses[0].role_owner())
        );
        assert_eq!(
            row.mapper_term_reference(),
            typed_for_resolved(&fixture.typed_ast, uses[0].term_reference())
        );
        assert_eq!(
            row.mapper_identifier(),
            typed_for_resolved(&fixture.typed_ast, uses[0].identifier())
        );
        assert_eq!(
            (
                row.mapper_source_ordinal(),
                row.mapper_role_source_ordinal(),
                row.first_condition_source_ordinal(),
                row.first_condition_role_source_ordinal(),
                row.second_condition_source_ordinal(),
                row.second_condition_role_source_ordinal(),
            ),
            (0, 0, 1, 0, 2, 1)
        );
        assert_eq!(
            handoff.debug_text(),
            "source-template-fraenkel-structural-composition-v1|module=pkg.templates|compositions=1|uses=3"
        );
        assert!(
            handoff
                .compositions()
                .get(SourceTemplateFraenkelStructuralCompositionId::new(1))
                .is_none()
        );
    }

    #[test]
    fn task277c_rejects_environment_missing_and_ambiguous_resolved_nodes() {
        let fixture = structural_fixture();
        let wrong_environment = structural_typed_ast_from_nodes(
            &fixture,
            other_source(),
            fixture.module.clone(),
            nodes_for_source(fixture.nodes.clone(), other_source()),
        );
        assert_eq!(
            compose(
                &fixture,
                &fixture.template,
                &fixture.generators,
                &wrong_environment
            ),
            Err(SourceTemplateFraenkelStructuralCompositionError::EnvironmentMismatch)
        );
        let wrong_module = structural_typed_ast_from_nodes(
            &fixture,
            fixture.source,
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("other")),
            fixture.nodes.clone(),
        );
        assert_eq!(
            compose(
                &fixture,
                &fixture.template,
                &fixture.generators,
                &wrong_module
            ),
            Err(SourceTemplateFraenkelStructuralCompositionError::EnvironmentMismatch)
        );

        let association = fixture
            .template
            .associations()
            .get(SourceTemplateTypeParameterAssociationId::new(0))
            .expect("Task277C template association");
        let binding = fixture
            .generators
            .bindings()
            .get(FraenkelGeneratorVariableBindingId::new(0))
            .expect("Task277C generator binding");
        let mapper = fixture
            .generators
            .uses()
            .get(0)
            .expect("Task277C mapper use");

        let mut template_and_lower_faults = fixture.nodes.clone();
        template_and_lower_faults[association.parameter().index()].resolved_node = None;
        template_and_lower_faults[binding.segment().index()].resolved_node = None;
        template_and_lower_faults
            [typed_for_resolved(&fixture.typed_ast, mapper.identifier()).index()]
        .resolved_node = None;
        assert_invalid_template(&fixture, template_and_lower_faults);

        let mut binding_and_use_faults = fixture.nodes.clone();
        binding_and_use_faults[binding.segment().index()].resolved_node = None;
        binding_and_use_faults
            [typed_for_resolved(&fixture.typed_ast, mapper.identifier()).index()]
        .resolved_node = None;
        assert_invalid_binding(&fixture, binding_and_use_faults);

        let mut missing_use = fixture.nodes.clone();
        missing_use[typed_for_resolved(&fixture.typed_ast, mapper.identifier()).index()]
            .resolved_node = None;
        assert_invalid_use(&fixture, missing_use, 0);

        let mut ambiguous_template = fixture.nodes.clone();
        ambiguous_template.push(
            TypedNode::new(
                "Unrelated",
                SourceAnchor::Range(range(fixture.source, 151, 152)),
            )
            .with_resolved_node(
                fixture
                    .typed_ast
                    .nodes()
                    .node(association.parameter())
                    .and_then(|node| node.resolved_node)
                    .expect("Task277C association provenance"),
            ),
        );
        assert_invalid_template(&fixture, ambiguous_template);

        let mut ambiguous_binding = fixture.nodes.clone();
        ambiguous_binding.push(
            TypedNode::new(
                "Unrelated",
                SourceAnchor::Range(range(fixture.source, 152, 153)),
            )
            .with_resolved_node(binding.segment()),
        );
        assert_invalid_binding(&fixture, ambiguous_binding);

        let mut ambiguous_use = fixture.nodes.clone();
        ambiguous_use.push(
            TypedNode::new(
                "Unrelated",
                SourceAnchor::Range(range(fixture.source, 153, 154)),
            )
            .with_resolved_node(mapper.identifier()),
        );
        assert_invalid_use(&fixture, ambiguous_use, 0);
    }

    #[test]
    fn task277c_rejects_recovery_kind_range_edge_and_provenance_corruption() {
        let fixture = structural_fixture();
        let binding = fixture
            .generators
            .bindings()
            .get(FraenkelGeneratorVariableBindingId::new(0))
            .expect("Task277C generator binding");
        let mapper = fixture
            .generators
            .uses()
            .get(0)
            .expect("Task277C mapper use");
        let first_condition = fixture
            .generators
            .uses()
            .get(1)
            .expect("Task277C first condition use");

        let mut recovered_type_expression = fixture.nodes.clone();
        let segment = typed_for_resolved(&fixture.typed_ast, binding.segment());
        let type_expression =
            exact_direct_child_with_kind(&fixture.typed_ast, segment, "TypeExpression")
                .expect("Task277C type expression");
        let recovered_duplicate = TypedNodeId::new(recovered_type_expression.len());
        recovered_type_expression.push(
            TypedNode::new(
                "TypeExpression",
                SourceAnchor::Range(range(fixture.source, 75, 76)),
            )
            .with_recovery(NodeRecoveryState::Recovered),
        );
        recovered_type_expression[segment.index()]
            .children
            .push(recovered_duplicate);
        assert_invalid_binding(&fixture, recovered_type_expression);

        let mut forged_kind = fixture.nodes.clone();
        forged_kind[typed_for_resolved(&fixture.typed_ast, mapper.term_reference()).index()].kind =
            "TermReferenceSpoof".into();
        assert_invalid_use(&fixture, forged_kind, 0);

        let mut condition_outside_comprehension = fixture.nodes.clone();
        condition_outside_comprehension
            [typed_for_resolved(&fixture.typed_ast, first_condition.role_owner()).index()]
        .anchor = SourceAnchor::Range(range(fixture.source, 130, 131));
        assert_invalid_use(&fixture, condition_outside_comprehension, 1);

        let mut detached_mapper_reference = fixture.nodes.clone();
        let mapper_owner = typed_for_resolved(&fixture.typed_ast, mapper.role_owner());
        let mapper_reference = typed_for_resolved(&fixture.typed_ast, mapper.term_reference());
        detached_mapper_reference[mapper_owner.index()]
            .children
            .retain(|child| *child != mapper_reference);
        assert_invalid_use(&fixture, detached_mapper_reference, 0);

        let mut wrong_provenance = fixture.nodes.clone();
        wrong_provenance
            [typed_for_resolved(&fixture.typed_ast, first_condition.identifier()).index()]
        .resolved_node = None;
        assert_invalid_use(&fixture, wrong_provenance, 1);

        let mut foreign_reference_range = fixture.nodes.clone();
        foreign_reference_range
            [typed_for_resolved(&fixture.typed_ast, first_condition.term_reference()).index()]
        .anchor = SourceAnchor::Range(range(fixture.source, 130, 131));
        assert_invalid_use(&fixture, foreign_reference_range, 1);

        let mut template_range = fixture.nodes.clone();
        let association = fixture
            .template
            .associations()
            .get(SourceTemplateTypeParameterAssociationId::new(0))
            .expect("Task277C template association");
        template_range[association.type_head().index()].anchor =
            SourceAnchor::Range(range(fixture.source, 74, 75));
        assert_invalid_template(&fixture, template_range);

        assert_eq!(
            type_expression,
            exact_direct_child_with_kind(&fixture.typed_ast, segment, "TypeExpression",)
                .expect("Task277C original type expression remains unique")
        );
    }

    #[test]
    fn task277c_rebuilds_deterministically_without_mutating_typed_ast() {
        let fixture = structural_fixture();
        let before = fixture.typed_ast.clone();
        let first = compose(
            &fixture,
            &fixture.template,
            &fixture.generators,
            &fixture.typed_ast,
        )
        .expect("first Task277C build");
        let second = compose(
            &fixture,
            &fixture.template,
            &fixture.generators,
            &fixture.typed_ast,
        )
        .expect("second Task277C build");
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(fixture.typed_ast, before);

        let (empty_template, empty_generators, empty_typed) = empty_structural_inputs();
        let empty = SourceTemplateFraenkelStructuralCompositionProducer::build(
            &empty_template,
            &empty_generators,
            &empty_typed,
        )
        .expect("Task277C empty/empty input is valid");
        assert!(empty.compositions().is_empty());
        assert_eq!(empty.compositions().len(), 0);

        let no_template = SourceTemplateTypeParameterAssociationHandoff {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            associations: SourceTemplateTypeParameterAssociationTable { rows: Vec::new() },
        };
        assert_eq!(
            compose(
                &fixture,
                &no_template,
                &fixture.generators,
                &fixture.typed_ast
            ),
            Err(
                SourceTemplateFraenkelStructuralCompositionError::InvalidComposition {
                    composition: SourceTemplateFraenkelStructuralCompositionId::new(0),
                }
            )
        );

        let mut same_range_foreign_type_head = fixture.nodes.clone();
        let binding = fixture
            .generators
            .bindings()
            .get(FraenkelGeneratorVariableBindingId::new(0))
            .expect("Task277C generator binding");
        let segment = typed_for_resolved(&fixture.typed_ast, binding.segment());
        let type_expression =
            exact_direct_child_with_kind(&fixture.typed_ast, segment, "TypeExpression")
                .expect("Task277C type expression");
        let original_type_head =
            exact_direct_child_with_kind(&fixture.typed_ast, type_expression, "TypeHead")
                .expect("Task277C type head");
        let type_head_range = fixture
            .typed_ast
            .nodes()
            .node(original_type_head)
            .and_then(|node| range_anchor(node, fixture.source))
            .expect("Task277C type-head range");
        let foreign_identifier = TypedNodeId::new(same_range_foreign_type_head.len());
        same_range_foreign_type_head.push(TypedNode::new(
            "Identifier",
            SourceAnchor::Range(type_head_range),
        ));
        let foreign_type_head = TypedNodeId::new(same_range_foreign_type_head.len());
        same_range_foreign_type_head.push(
            TypedNode::new("TypeHead", SourceAnchor::Range(type_head_range))
                .with_children(vec![foreign_identifier]),
        );
        same_range_foreign_type_head[type_expression.index()].children = vec![foreign_type_head];
        assert_eq!(
            compose(
                &fixture,
                &fixture.template,
                &fixture.generators,
                &structural_typed_ast_from_nodes(
                    &fixture,
                    fixture.source,
                    fixture.module.clone(),
                    same_range_foreign_type_head,
                ),
            ),
            Err(
                SourceTemplateFraenkelStructuralCompositionError::InvalidComposition {
                    composition: SourceTemplateFraenkelStructuralCompositionId::new(0),
                }
            )
        );

        let mut multiple_matches = fixture.template.clone();
        multiple_matches
            .associations
            .rows
            .push(multiple_matches.associations.rows[0].clone());
        assert_eq!(
            compose(
                &fixture,
                &multiple_matches,
                &fixture.generators,
                &fixture.typed_ast,
            ),
            Err(
                SourceTemplateFraenkelStructuralCompositionError::InvalidComposition {
                    composition: SourceTemplateFraenkelStructuralCompositionId::new(0),
                }
            )
        );

        let template_profiles = fixture
            .template
            .associations()
            .iter()
            .map(|(id, row)| validate_template_association(&fixture.typed_ast, id, row))
            .collect::<Option<Vec<_>>>()
            .expect("Task277C template profiles");
        let generator_profiles = fixture
            .generators
            .bindings()
            .iter()
            .map(|(id, row)| validate_generator_binding(&fixture.typed_ast, id, row))
            .collect::<Option<Vec<_>>>()
            .expect("Task277C generator profiles");
        let use_profiles = fixture
            .generators
            .uses()
            .iter()
            .enumerate()
            .map(|(index, link)| {
                validate_generator_use(
                    &fixture.typed_ast,
                    &fixture.generators,
                    &generator_profiles,
                    index,
                    link.role_source_ordinal(),
                    link,
                )
            })
            .collect::<Option<Vec<_>>>()
            .expect("Task277C use profiles");
        let mut reused_binding = generator_profiles[0];
        reused_binding.id = FraenkelGeneratorVariableBindingId::new(1);
        let mut reused_uses = use_profiles.clone();
        reused_uses.extend(
            use_profiles
                .iter()
                .map(|profile| GeneratorUseStructuralProfile {
                    binding: FraenkelGeneratorVariableBindingId::new(1),
                    ..*profile
                }),
        );
        assert_eq!(
            compose_structural_rows(
                &fixture.typed_ast,
                &template_profiles,
                &[generator_profiles[0], reused_binding],
                &reused_uses,
            ),
            Err(
                SourceTemplateFraenkelStructuralCompositionError::InvalidComposition {
                    composition: SourceTemplateFraenkelStructuralCompositionId::new(1),
                }
            )
        );

        let empty_generators = empty_generators_for(fixture.source, &fixture.module);
        assert_eq!(
            compose(
                &fixture,
                &fixture.template,
                &empty_generators,
                &fixture.typed_ast,
            ),
            Err(
                SourceTemplateFraenkelStructuralCompositionError::UnmatchedTemplateAssociation {
                    association: SourceTemplateTypeParameterAssociationId::new(0),
                }
            )
        );
    }

    #[derive(Clone)]
    struct StructuralFixture {
        source: SourceId,
        module: ModuleId,
        template: SourceTemplateTypeParameterAssociationHandoff,
        generators: FraenkelGeneratorVariableSourceCollection,
        nodes: Vec<TypedNode>,
        typed_ast: TypedAst,
        root: TypedNodeId,
    }

    fn structural_fixture() -> StructuralFixture {
        let source = source_id();
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("templates"));
        let ast = structural_ast(source);
        let resolved = SurfaceResolvedArena::lower(&ast, &module).expect("Task277C resolver arena");
        let template_collection =
            TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
                .expect("Task277C template collector validation")
                .collect()
                .expect("Task277C template collection");
        let generators = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
            .expect("Task277C generator collector validation")
            .collect()
            .expect("Task277C generator collection");
        assert_eq!(template_collection.generator_links().len(), 1);
        assert_eq!(generators.bindings().len(), 1);
        assert_eq!(generators.uses().len(), 3);
        let (nodes, typed_ast, root) = structural_typed_profile(&ast, module.clone(), &resolved);
        let template =
            SourceTemplateTypeParameterAssociationProducer::build(&template_collection, &typed_ast)
                .expect("Task277C template handoff");
        StructuralFixture {
            source,
            module,
            template,
            generators,
            nodes,
            typed_ast,
            root,
        }
    }

    fn compose(
        _fixture: &StructuralFixture,
        template: &SourceTemplateTypeParameterAssociationHandoff,
        generators: &FraenkelGeneratorVariableSourceCollection,
        typed_ast: &TypedAst,
    ) -> Result<
        SourceTemplateFraenkelStructuralCompositionHandoff,
        SourceTemplateFraenkelStructuralCompositionError,
    > {
        SourceTemplateFraenkelStructuralCompositionProducer::build(template, generators, typed_ast)
    }

    fn assert_invalid_template(fixture: &StructuralFixture, nodes: Vec<TypedNode>) {
        assert!(matches!(
            compose(
                fixture,
                &fixture.template,
                &fixture.generators,
                &structural_typed_ast_from_nodes(
                    fixture,
                    fixture.source,
                    fixture.module.clone(),
                    nodes,
                ),
            ),
            Err(SourceTemplateFraenkelStructuralCompositionError::InvalidTemplateAssociation {
                association
            }) if association == SourceTemplateTypeParameterAssociationId::new(0)
        ));
    }

    fn assert_invalid_binding(fixture: &StructuralFixture, nodes: Vec<TypedNode>) {
        assert!(matches!(
            compose(
                fixture,
                &fixture.template,
                &fixture.generators,
                &structural_typed_ast_from_nodes(
                    fixture,
                    fixture.source,
                    fixture.module.clone(),
                    nodes,
                ),
            ),
            Err(SourceTemplateFraenkelStructuralCompositionError::InvalidGeneratorBinding {
                binding
            }) if binding == FraenkelGeneratorVariableBindingId::new(0)
        ));
    }

    fn assert_invalid_use(fixture: &StructuralFixture, nodes: Vec<TypedNode>, use_index: usize) {
        assert_eq!(
            compose(
                fixture,
                &fixture.template,
                &fixture.generators,
                &structural_typed_ast_from_nodes(
                    fixture,
                    fixture.source,
                    fixture.module.clone(),
                    nodes,
                ),
            ),
            Err(
                SourceTemplateFraenkelStructuralCompositionError::InvalidGeneratorUse { use_index }
            )
        );
    }

    fn typed_for_resolved(typed_ast: &TypedAst, resolved: ResolvedNodeId) -> TypedNodeId {
        exact_resolved_node(typed_ast, resolved).expect("Task277C unique typed resolver mapping")
    }

    fn structural_typed_profile(
        ast: &syntax::SurfaceAst,
        module: ModuleId,
        resolved: &SurfaceResolvedArena,
    ) -> (Vec<TypedNode>, TypedAst, TypedNodeId) {
        let mut builder = TypedArenaBuilder::new();
        let mut typed_by_surface = std::collections::BTreeMap::new();
        let mut nodes = Vec::with_capacity(ast.nodes().len());
        for view in ast.node_views() {
            let surface = ast.node(view.id()).expect("Task277C surface node");
            let resolved_node = resolved
                .resolved_node_for(view.id())
                .expect("Task277C resolver mapping");
            let node = TypedNode::new(
                structural_typed_kind(surface),
                SourceAnchor::Range(surface.range),
            )
            .with_children(
                surface
                    .children
                    .iter()
                    .map(|child| {
                        *typed_by_surface
                            .get(child)
                            .expect("Task277C child typed before parent")
                    })
                    .collect(),
            )
            .with_recovery(if surface.recovered {
                NodeRecoveryState::Recovered
            } else {
                NodeRecoveryState::Normal
            })
            .with_resolved_node(resolved_node);
            let typed = builder
                .push(node.clone())
                .expect("Task277C typed arena node");
            assert!(typed_by_surface.insert(view.id(), typed).is_none());
            nodes.push(node);
        }
        let root = ast
            .root()
            .and_then(|surface| typed_by_surface.get(&surface).copied())
            .expect("Task277C typed root");
        let arena = builder.finish(Some(root)).expect("Task277C typed arena");
        let typed_ast = TypedAst::try_new(TypedAstParts {
            source_id: ast.source_id,
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
        .expect("Task277C typed AST");
        (nodes, typed_ast, root)
    }

    fn structural_typed_ast_from_nodes(
        fixture: &StructuralFixture,
        source: SourceId,
        module: ModuleId,
        nodes: Vec<TypedNode>,
    ) -> TypedAst {
        let arena =
            TypedArena::try_new(Some(fixture.root), nodes).expect("Task277C mutated typed arena");
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
        .expect("Task277C mutated typed AST")
    }

    fn structural_typed_kind(surface: &syntax::SurfaceNode) -> String {
        match &surface.kind {
            syntax::SurfaceNodeKind::Token(token)
                if token.kind == syntax::SurfaceTokenKind::Identifier =>
            {
                "Identifier".to_owned()
            }
            _ => format!("{:?}", surface.kind),
        }
    }

    fn empty_structural_inputs() -> (
        SourceTemplateTypeParameterAssociationHandoff,
        FraenkelGeneratorVariableSourceCollection,
        TypedAst,
    ) {
        let source = source_id();
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("empty"));
        let mut builder = syntax::SurfaceAstBuilder::new(source);
        let root = builder.add_node(
            syntax::SurfaceNodeKind::Root,
            range(source, 0, 1),
            Vec::new(),
        );
        let ast = builder.finish(Some(root), None);
        let resolved = SurfaceResolvedArena::lower(&ast, &module).expect("Task277C empty resolver");
        let templates = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
            .expect("Task277C empty template collector")
            .collect()
            .expect("Task277C empty template collection");
        let generators = empty_generators_for(source, &module);
        let nodes = vec![TypedNode::new(
            "Root",
            SourceAnchor::Range(range(source, 0, 1)),
        )];
        let typed_ast = typed_ast_from_nodes(source, module, nodes);
        let template =
            SourceTemplateTypeParameterAssociationProducer::build(&templates, &typed_ast)
                .expect("Task277C empty template handoff");
        (template, generators, typed_ast)
    }

    fn empty_generators_for(
        source: SourceId,
        module: &ModuleId,
    ) -> FraenkelGeneratorVariableSourceCollection {
        let mut builder = syntax::SurfaceAstBuilder::new(source);
        let root = builder.add_node(
            syntax::SurfaceNodeKind::Root,
            range(source, 0, 1),
            Vec::new(),
        );
        let ast = builder.finish(Some(root), None);
        let resolved =
            SurfaceResolvedArena::lower(&ast, module).expect("Task277C empty generator resolver");
        FraenkelGeneratorVariableSourceCollector::new(&ast, module, &resolved)
            .expect("Task277C empty generator collector")
            .collect()
            .expect("Task277C empty generator collection")
    }

    fn structural_ast(source: SourceId) -> syntax::SurfaceAst {
        let mut builder = syntax::SurfaceAstBuilder::new(source);
        let definition = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "definition",
            range(source, 0, 10),
        );
        let parameter = direct_type_parameter(&mut builder, source, 11, "T");
        let func = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "func",
            range(source, 30, 34),
        );
        let open = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "{",
            range(source, 70, 71),
        );
        let mapper_identifier = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 72, 73),
        );
        let mapper_reference = builder.add_node(
            syntax::SurfaceNodeKind::TermReference,
            range(source, 72, 73),
            vec![mapper_identifier],
        );
        let mapper = builder.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 72, 73),
            vec![mapper_reference],
        );
        let where_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "where",
            range(source, 74, 79),
        );
        let generator_binder = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 80, 81),
        );
        let is_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "is",
            range(source, 82, 84),
        );
        let template_identifier = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "T",
            range(source, 85, 86),
        );
        let type_head = builder.add_node(
            syntax::SurfaceNodeKind::TypeHead,
            range(source, 85, 86),
            vec![template_identifier],
        );
        let type_expression = builder.add_node(
            syntax::SurfaceNodeKind::TypeExpression,
            range(source, 85, 86),
            vec![type_head],
        );
        let segment = builder.add_node(
            syntax::SurfaceNodeKind::ComprehensionVariableSegment,
            range(source, 80, 86),
            vec![generator_binder, is_keyword, type_expression],
        );
        let colon = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            ":",
            range(source, 87, 88),
        );
        let not_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "not",
            range(source, 89, 92),
        );
        let first_identifier = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 93, 94),
        );
        let first_reference = builder.add_node(
            syntax::SurfaceNodeKind::TermReference,
            range(source, 93, 94),
            vec![first_identifier],
        );
        let first_term = builder.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 93, 94),
            vec![first_reference],
        );
        let membership = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "in",
            range(source, 95, 97),
        );
        let second_identifier = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 98, 99),
        );
        let second_reference = builder.add_node(
            syntax::SurfaceNodeKind::TermReference,
            range(source, 98, 99),
            vec![second_identifier],
        );
        let second_term = builder.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 98, 99),
            vec![second_reference],
        );
        let predicate = builder.add_node(
            syntax::SurfaceNodeKind::BuiltinPredicateApplication,
            range(source, 93, 99),
            vec![first_term, membership, second_term],
        );
        let prefix = builder.add_node(
            syntax::SurfaceNodeKind::PrefixFormula(syntax::SurfaceFormulaPrefixOperator::Not),
            range(source, 89, 99),
            vec![not_keyword, predicate],
        );
        let condition = builder.add_node(
            syntax::SurfaceNodeKind::FormulaExpression,
            range(source, 89, 99),
            vec![prefix],
        );
        let close = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "}",
            range(source, 108, 109),
        );
        let comprehension = builder.add_node(
            syntax::SurfaceNodeKind::SetComprehension,
            range(source, 70, 109),
            vec![
                open,
                mapper,
                where_keyword,
                segment,
                colon,
                condition,
                close,
            ],
        );
        let term_expression = builder.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 70, 109),
            vec![comprehension],
        );
        let definiens = builder.add_node(
            syntax::SurfaceNodeKind::TermDefiniens,
            range(source, 70, 109),
            vec![term_expression],
        );
        let functor = builder.add_node(
            syntax::SurfaceNodeKind::FunctorDefinition,
            range(source, 30, 115),
            vec![func, definiens],
        );
        let definition_block = builder.add_node(
            syntax::SurfaceNodeKind::DefinitionBlockItem,
            range(source, 0, 120),
            vec![definition, parameter, functor],
        );
        let root = builder.add_node(
            syntax::SurfaceNodeKind::Root,
            range(source, 0, 120),
            vec![definition_block],
        );
        builder.finish(Some(root), None)
    }

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        collection: TemplateTypeParameterSourceCollection,
        nodes: Vec<TypedNode>,
        typed_ast: TypedAst,
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("templates"));
        let ast = identity_ast(source);
        fixture_from_ast(source, module, ast)
    }

    fn multi_link_fixture() -> Fixture {
        let source = source_id();
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("templates"));
        let ast = multi_link_ast(source);
        let resolved =
            SurfaceResolvedArena::lower(&ast, &module).expect("Task277B-L multi resolver arena");
        let collection = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
            .expect("Task277B-L multi collector validation")
            .collect()
            .expect("Task277B-L multi collection");
        let binding_t = collection
            .bindings()
            .get(TemplateTypeParameterBindingId::new(0))
            .expect("Task277B-L T binding");
        let binding_u = collection
            .bindings()
            .get(TemplateTypeParameterBindingId::new(1))
            .expect("Task277B-L U binding");
        let link_u = collection
            .generator_links()
            .get(0)
            .expect("Task277B-L U generator link");
        let link_t = collection
            .generator_links()
            .get(1)
            .expect("Task277B-L T generator link");
        assert_eq!(
            (link_u.binding(), link_t.binding()),
            (
                TemplateTypeParameterBindingId::new(1),
                TemplateTypeParameterBindingId::new(0)
            )
        );
        let nodes = vec![
            TypedNode::new("Identifier", SourceAnchor::Range(range(source, 30, 31)))
                .with_resolved_node(binding_u.binder()),
            TypedNode::new(
                "TemplateParameter",
                SourceAnchor::Range(binding_u.source_range()),
            )
            .with_children(vec![TypedNodeId::new(0)])
            .with_resolved_node(binding_u.parameter()),
            TypedNode::new("Identifier", SourceAnchor::Range(link_t.source_range()))
                .with_resolved_node(link_t.identifier()),
            TypedNode::new("TypeHead", SourceAnchor::Range(link_t.source_range()))
                .with_children(vec![TypedNodeId::new(2)])
                .with_resolved_node(link_t.type_head()),
            TypedNode::new("Identifier", SourceAnchor::Range(range(source, 15, 16)))
                .with_resolved_node(binding_t.binder()),
            TypedNode::new(
                "TemplateParameter",
                SourceAnchor::Range(binding_t.source_range()),
            )
            .with_children(vec![TypedNodeId::new(4)])
            .with_resolved_node(binding_t.parameter()),
            TypedNode::new("Identifier", SourceAnchor::Range(link_u.source_range()))
                .with_resolved_node(link_u.identifier()),
            TypedNode::new("TypeHead", SourceAnchor::Range(link_u.source_range()))
                .with_children(vec![TypedNodeId::new(6)])
                .with_resolved_node(link_u.type_head()),
            TypedNode::new(
                "DefinitionBlockItem",
                SourceAnchor::Range(range(source, 0, 120)),
            )
            .with_children(vec![
                TypedNodeId::new(5),
                TypedNodeId::new(1),
                TypedNodeId::new(3),
                TypedNodeId::new(7),
            ])
            .with_resolved_node(binding_t.definition_block()),
        ];
        let typed_ast = typed_ast_from_nodes(source, module.clone(), nodes.clone());
        Fixture {
            source,
            module,
            collection,
            nodes,
            typed_ast,
        }
    }

    fn fixture_from_ast(source: SourceId, module: ModuleId, ast: syntax::SurfaceAst) -> Fixture {
        let resolved =
            SurfaceResolvedArena::lower(&ast, &module).expect("Task277B-L resolver arena");
        let collection = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
            .expect("Task277B-L collector validation")
            .collect()
            .expect("Task277B-L collection");
        let binding = collection
            .bindings()
            .get(TemplateTypeParameterBindingId::new(0))
            .expect("Task277B-L binding");
        let mut nodes = vec![
            TypedNode::new("Identifier", SourceAnchor::Range(range(source, 15, 16)))
                .with_resolved_node(binding.binder()),
            TypedNode::new(
                "TemplateParameter",
                SourceAnchor::Range(binding.source_range()),
            )
            .with_children(vec![TypedNodeId::new(0)])
            .with_resolved_node(binding.parameter()),
        ];
        let mut type_heads = Vec::new();
        for link in collection.generator_links().iter() {
            let identifier = TypedNodeId::new(nodes.len());
            nodes.push(
                TypedNode::new("Identifier", SourceAnchor::Range(link.source_range()))
                    .with_resolved_node(link.identifier()),
            );
            let type_head = TypedNodeId::new(nodes.len());
            nodes.push(
                TypedNode::new("TypeHead", SourceAnchor::Range(link.source_range()))
                    .with_children(vec![identifier])
                    .with_resolved_node(link.type_head()),
            );
            type_heads.push(type_head);
        }
        let mut definition_children = vec![TypedNodeId::new(1)];
        definition_children.extend(type_heads);
        nodes.push(
            TypedNode::new(
                "DefinitionBlockItem",
                SourceAnchor::Range(range(source, 0, 200)),
            )
            .with_children(definition_children)
            .with_resolved_node(binding.definition_block()),
        );
        let typed_ast = typed_ast_from_nodes(source, module.clone(), nodes.clone());
        Fixture {
            source,
            module,
            collection,
            nodes,
            typed_ast,
        }
    }

    fn typed_ast_from_nodes(source: SourceId, module: ModuleId, nodes: Vec<TypedNode>) -> TypedAst {
        try_typed_ast_from_nodes(source, module, nodes).expect("Task277B-L typed AST")
    }

    fn try_typed_ast_from_nodes(
        source: SourceId,
        module: ModuleId,
        nodes: Vec<TypedNode>,
    ) -> Result<TypedAst, crate::typed_ast::TypedAstError> {
        let root = nodes.len().checked_sub(1).map(TypedNodeId::new);
        let arena = TypedArena::try_new(root, nodes).expect("Task277B-L typed arena");
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
    }

    fn nodes_for_source(mut nodes: Vec<TypedNode>, source: SourceId) -> Vec<TypedNode> {
        for node in &mut nodes {
            if let SourceAnchor::Range(mut anchor) = node.anchor {
                anchor.source_id = source;
                node.anchor = SourceAnchor::Range(anchor);
            }
        }
        nodes
    }

    fn mutated_typed_ast(fixture: &Fixture, mutate: impl FnOnce(&mut Vec<TypedNode>)) -> TypedAst {
        let mut nodes = fixture.nodes.clone();
        mutate(&mut nodes);
        typed_ast_from_nodes(fixture.source, fixture.module.clone(), nodes)
    }

    fn association_sites(fixture: &Fixture) -> [(usize, ResolvedNodeId); 5] {
        let binding = fixture
            .collection
            .bindings()
            .get(TemplateTypeParameterBindingId::new(0))
            .expect("binding");
        let link = fixture
            .collection
            .generator_links()
            .get(0)
            .expect("generator link");
        [
            (4, binding.definition_block()),
            (1, binding.parameter()),
            (0, binding.binder()),
            (3, link.type_head()),
            (2, link.identifier()),
        ]
    }

    fn assert_invalid_at(
        collection: &TemplateTypeParameterSourceCollection,
        typed_ast: &TypedAst,
        expected: SourceTemplateTypeParameterAssociationId,
    ) {
        assert!(matches!(
            SourceTemplateTypeParameterAssociationProducer::build(collection, typed_ast),
            Err(SourceTemplateTypeParameterAssociationError::InvalidAssociation { association })
                if association == expected
        ));
    }

    fn empty_profile() -> (TemplateTypeParameterSourceCollection, TypedAst) {
        let source = source_id();
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("empty"));
        let mut builder = syntax::SurfaceAstBuilder::new(source);
        let root = builder.add_node(
            syntax::SurfaceNodeKind::Root,
            range(source, 0, 1),
            Vec::new(),
        );
        let ast = builder.finish(Some(root), None);
        let resolved =
            SurfaceResolvedArena::lower(&ast, &module).expect("Task277B-L empty resolver arena");
        let collection = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
            .expect("Task277B-L empty collector validation")
            .collect()
            .expect("Task277B-L empty collection");
        let typed_ast = typed_ast_from_nodes(
            source,
            module,
            vec![TypedNode::new(
                "Root",
                SourceAnchor::Range(range(source, 0, 1)),
            )],
        );
        (collection, typed_ast)
    }

    fn identity_ast(source: SourceId) -> syntax::SurfaceAst {
        let mut builder = syntax::SurfaceAstBuilder::new(source);
        let definition = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "definition",
            range(source, 0, 10),
        );
        let parameter = direct_type_parameter(&mut builder, source, 11, "T");
        let generator = generator(&mut builder, source, 30, 48, "T");
        let functor = builder.add_node(
            syntax::SurfaceNodeKind::FunctorDefinition,
            range(source, 30, 60),
            vec![generator],
        );
        let definition_block = builder.add_node(
            syntax::SurfaceNodeKind::DefinitionBlockItem,
            range(source, 0, 62),
            vec![definition, parameter, functor],
        );
        let root = builder.add_node(
            syntax::SurfaceNodeKind::Root,
            range(source, 0, 62),
            vec![definition_block],
        );
        builder.finish(Some(root), None)
    }

    fn multi_link_ast(source: SourceId) -> syntax::SurfaceAst {
        let mut builder = syntax::SurfaceAstBuilder::new(source);
        let definition = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "definition",
            range(source, 0, 10),
        );
        let parameter_t = direct_type_parameter(&mut builder, source, 11, "T");
        let parameter_u = direct_type_parameter(&mut builder, source, 26, "U");
        // The source order of R1 links is U then T. The typed fixture below
        // deliberately orders its matched sites differently.
        let first_generator = generator(&mut builder, source, 50, 68, "U");
        let second_generator = generator(&mut builder, source, 85, 103, "T");
        let functor = builder.add_node(
            syntax::SurfaceNodeKind::FunctorDefinition,
            range(source, 50, 115),
            vec![first_generator, second_generator],
        );
        let definition_block = builder.add_node(
            syntax::SurfaceNodeKind::DefinitionBlockItem,
            range(source, 0, 120),
            vec![definition, parameter_t, parameter_u, functor],
        );
        let root = builder.add_node(
            syntax::SurfaceNodeKind::Root,
            range(source, 0, 120),
            vec![definition_block],
        );
        builder.finish(Some(root), None)
    }

    fn direct_type_parameter(
        builder: &mut syntax::SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        spelling: &str,
    ) -> syntax::SurfaceBuilderNodeId {
        let let_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "let",
            range(source, start, start + 3),
        );
        let binder_start = start + 4;
        let binder = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            spelling,
            range(source, binder_start, binder_start + spelling.len()),
        );
        let be_start = binder_start + spelling.len() + 1;
        let be_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "be",
            range(source, be_start, be_start + 2),
        );
        let type_start = be_start + 3;
        let type_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "type",
            range(source, type_start, type_start + 4),
        );
        let semicolon = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source, type_start + 4, type_start + 5),
        );
        builder.add_node(
            syntax::SurfaceNodeKind::TemplateParameter,
            range(source, start, type_start + 5),
            vec![let_keyword, binder, be_keyword, type_keyword, semicolon],
        )
    }

    fn generator(
        builder: &mut syntax::SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        type_start: usize,
        spelling: &str,
    ) -> syntax::SurfaceBuilderNodeId {
        let opener = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "{",
            range(source, start, start + 1),
        );
        let mapper = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, start + 2, start + 3),
        );
        let where_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "where",
            range(source, start + 4, start + 9),
        );
        let generator = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, type_start - 8, type_start - 7),
        );
        let is_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "is",
            range(source, type_start - 6, type_start - 4),
        );
        let identifier = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            spelling,
            range(source, type_start, type_start + spelling.len()),
        );
        let type_head = builder.add_node(
            syntax::SurfaceNodeKind::TypeHead,
            range(source, type_start, type_start + spelling.len()),
            vec![identifier],
        );
        let type_expression = builder.add_node(
            syntax::SurfaceNodeKind::TypeExpression,
            range(source, type_start, type_start + spelling.len()),
            vec![type_head],
        );
        let segment = builder.add_node(
            syntax::SurfaceNodeKind::ComprehensionVariableSegment,
            range(source, type_start - 8, type_start + spelling.len()),
            vec![generator, is_keyword, type_expression],
        );
        let closer = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "}",
            range(
                source,
                type_start + spelling.len() + 1,
                type_start + spelling.len() + 2,
            ),
        );
        builder.add_node(
            syntax::SurfaceNodeKind::SetComprehension,
            range(source, start, type_start + spelling.len() + 2),
            vec![opener, mapper, where_keyword, segment, closer],
        )
    }

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "27".repeat(Hash::BYTE_LEN)
        ))
        .expect("Task277B-L snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("Task277B-L source")
    }

    fn other_source() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "28".repeat(Hash::BYTE_LEN)
        ))
        .expect("Task277B-L other snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        let _ = allocator
            .next_source_id(snapshot)
            .expect("Task277B-L first other source");
        allocator
            .next_source_id(snapshot)
            .expect("Task277B-L second other source")
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }
}
