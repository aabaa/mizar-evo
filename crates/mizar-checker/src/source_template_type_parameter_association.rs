//! Neutral transport from resolver-owned template type-parameter links to an
//! existing typed arena.

use crate::typed_ast::{NodeRecoveryState, TypedAst, TypedNode, TypedNodeId};
use mizar_resolve::{
    names::{TemplateTypeParameterBindingId, TemplateTypeParameterSourceCollection},
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
        TypeFactTable, TypeTable, TypedArena, TypedAstParts,
    };
    use mizar_resolve::{
        names::TemplateTypeParameterSourceCollector, resolved_ast::SurfaceResolvedArena,
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
