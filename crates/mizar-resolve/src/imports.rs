//! Semantic import graph construction.
//!
//! This module takes canonical import candidates and builds the deterministic
//! accepted acyclic graph used by later import, name, and symbol resolution
//! tasks. Surface syntax collection, alias binding, relative-path
//! interpretation, and unresolved import recovery feed this graph layer in
//! follow-on tasks.

use crate::module_index::{IndexedModuleId, ModuleIndexInput, ModuleIndexProviderError};
use crate::resolved_ast::ModuleId;
use mizar_session::SourceRange;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// Canonical import candidates for one source module.
///
/// Callers must pass an explicit empty candidate set for a zero-import module
/// that should participate in graph ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleImportCandidates {
    module: ModuleId,
    imports: Vec<ImportEdgeCandidate>,
}

impl ModuleImportCandidates {
    /// Creates a candidate set for one module.
    #[must_use]
    pub fn new(module: ModuleId, imports: Vec<ImportEdgeCandidate>) -> Self {
        Self { module, imports }
    }

    /// Returns the source module.
    #[must_use]
    pub const fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Returns canonical import-edge candidates in source collection order.
    #[must_use]
    pub fn imports(&self) -> &[ImportEdgeCandidate] {
        &self.imports
    }
}

/// A canonical import edge candidate before graph deduplication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEdgeCandidate {
    target: ModuleId,
    range: SourceRange,
    ordinal: usize,
}

impl ImportEdgeCandidate {
    /// Creates a canonical import edge candidate.
    #[must_use]
    pub const fn new(target: ModuleId, range: SourceRange, ordinal: usize) -> Self {
        Self {
            target,
            range,
            ordinal,
        }
    }

    /// Returns the target module.
    #[must_use]
    pub const fn target(&self) -> &ModuleId {
        &self.target
    }

    /// Returns the source range that introduced this candidate.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source-order ordinal for this candidate.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// Builds import graphs against the resolver module-index input.
#[derive(Clone, Copy)]
pub struct ImportGraphBuilder<'a> {
    module_index: ModuleIndexInput<'a>,
}

impl<'a> ImportGraphBuilder<'a> {
    /// Creates a graph builder backed by the resolver module-index input.
    #[must_use]
    pub const fn new(module_index: ModuleIndexInput<'a>) -> Self {
        Self { module_index }
    }

    /// Builds a deterministic import graph and rejects cyclic components.
    ///
    /// Inputs must already contain canonical module identities. Unknown source
    /// or target modules are invalid builder inputs and are reported as build
    /// errors; semantic unresolved-import recovery is a higher-level concern for
    /// the alias/path recovery task.
    pub fn build(
        self,
        modules: &[ModuleImportCandidates],
    ) -> Result<ImportGraphResolution, ImportGraphBuildError> {
        let mut nodes = BTreeSet::<ModuleId>::new();
        let mut canonical_edges = BTreeMap::<(ModuleId, ModuleId), ImportGraphEdge>::new();

        for module in modules {
            self.ensure_source_module(module.module())?;
            nodes.insert(module.module().clone());
            for candidate in module.imports() {
                self.ensure_target_module(module.module(), candidate)?;
                nodes.insert(candidate.target().clone());
                let edge = ImportGraphEdge::new(
                    module.module().clone(),
                    candidate.target().clone(),
                    candidate.range(),
                    candidate.ordinal(),
                );
                canonical_edges
                    .entry((edge.source().clone(), edge.target().clone()))
                    .and_modify(|existing| {
                        if edge_provenance_cmp(&edge, existing).is_lt() {
                            *existing = edge.clone();
                        }
                    })
                    .or_insert(edge);
            }
        }

        let edges = canonical_edges.into_values().collect::<Vec<_>>();
        let cycles = detect_cycles(&nodes, &edges);
        let cyclic_nodes = cycles
            .iter()
            .flat_map(|cycle| cycle.modules().iter().cloned())
            .collect::<BTreeSet<_>>();
        let accepted_nodes = nodes
            .into_iter()
            .filter(|node| !cyclic_nodes.contains(node))
            .collect::<BTreeSet<_>>();
        let accepted_edges = edges
            .iter()
            .filter(|edge| {
                !cyclic_nodes.contains(edge.source()) && !cyclic_nodes.contains(edge.target())
            })
            .cloned()
            .collect::<Vec<_>>();
        let topological_order =
            dependency_first_topological_order(&accepted_nodes, &accepted_edges);
        let graph = ImportGraph::new(
            accepted_nodes.into_iter().collect(),
            accepted_edges,
            topological_order,
        );

        Ok(ImportGraphResolution::new(graph, cycles))
    }

    fn ensure_source_module(&self, module: &ModuleId) -> Result<(), ImportGraphBuildError> {
        self.module_index
            .module(&indexed_module_id(module))
            .map(|_| ())
            .map_err(|lookup| ImportGraphBuildError::UnknownSourceModule {
                module: Box::new(module.clone()),
                lookup: Box::new(lookup),
            })
    }

    fn ensure_target_module(
        &self,
        source: &ModuleId,
        candidate: &ImportEdgeCandidate,
    ) -> Result<(), ImportGraphBuildError> {
        self.module_index
            .module(&indexed_module_id(candidate.target()))
            .map(|_| ())
            .map_err(|lookup| ImportGraphBuildError::UnknownTargetModule {
                source: Box::new(source.clone()),
                target: Box::new(candidate.target().clone()),
                range: candidate.range(),
                ordinal: candidate.ordinal(),
                lookup: Box::new(lookup),
            })
    }
}

/// Completed import graph construction result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportGraphResolution {
    graph: ImportGraph,
    cycles: Vec<ImportCycle>,
}

impl ImportGraphResolution {
    fn new(graph: ImportGraph, mut cycles: Vec<ImportCycle>) -> Self {
        cycles.sort_by(import_cycle_cmp);
        Self { graph, cycles }
    }

    /// Returns the accepted acyclic graph portion.
    #[must_use]
    pub const fn graph(&self) -> &ImportGraph {
        &self.graph
    }

    /// Returns rejected cycles in deterministic order.
    #[must_use]
    pub fn cycles(&self) -> &[ImportCycle] {
        &self.cycles
    }

    /// Returns whether any cycles were rejected.
    #[must_use]
    pub const fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }
}

/// Accepted acyclic import graph portion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportGraph {
    nodes: Vec<ModuleId>,
    edges: Vec<ImportGraphEdge>,
    topological_order: Vec<ModuleId>,
}

impl ImportGraph {
    fn new(
        mut nodes: Vec<ModuleId>,
        mut edges: Vec<ImportGraphEdge>,
        topological_order: Vec<ModuleId>,
    ) -> Self {
        nodes.sort();
        edges.sort_by(import_edge_cmp);
        Self {
            nodes,
            edges,
            topological_order,
        }
    }

    /// Returns accepted graph nodes in canonical order.
    #[must_use]
    pub fn nodes(&self) -> &[ModuleId] {
        &self.nodes
    }

    /// Returns accepted graph edges in deterministic canonical order.
    #[must_use]
    pub fn edges(&self) -> &[ImportGraphEdge] {
        &self.edges
    }

    /// Returns dependency-first topological order for accepted graph nodes.
    #[must_use]
    pub fn topological_order(&self) -> &[ModuleId] {
        &self.topological_order
    }
}

/// A canonical import graph edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportGraphEdge {
    source: ModuleId,
    target: ModuleId,
    range: SourceRange,
    ordinal: usize,
}

impl ImportGraphEdge {
    fn new(source: ModuleId, target: ModuleId, range: SourceRange, ordinal: usize) -> Self {
        Self {
            source,
            target,
            range,
            ordinal,
        }
    }

    /// Returns the importing module.
    #[must_use]
    pub const fn source(&self) -> &ModuleId {
        &self.source
    }

    /// Returns the imported module.
    #[must_use]
    pub const fn target(&self) -> &ModuleId {
        &self.target
    }

    /// Returns the source range for the retained edge provenance.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the source-order ordinal for the retained edge provenance.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// A rejected import cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportCycle {
    modules: Vec<ModuleId>,
    edges: Vec<ImportGraphEdge>,
}

impl ImportCycle {
    fn new(mut modules: Vec<ModuleId>, mut edges: Vec<ImportGraphEdge>) -> Self {
        modules.sort();
        edges.sort_by(cycle_edge_cmp);
        Self { modules, edges }
    }

    /// Returns cyclic modules in canonical order.
    #[must_use]
    pub fn modules(&self) -> &[ModuleId] {
        &self.modules
    }

    /// Returns internal cycle edges in deterministic order.
    #[must_use]
    pub fn edges(&self) -> &[ImportGraphEdge] {
        &self.edges
    }
}

/// Build error for canonical import graph construction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportGraphBuildError {
    /// Source module was not present in the module index.
    UnknownSourceModule {
        /// Missing source module.
        module: Box<ModuleId>,
        /// Underlying module-index lookup error.
        lookup: Box<ModuleIndexProviderError>,
    },
    /// Target module was not present in the module index.
    UnknownTargetModule {
        /// Source module that requested the import.
        source: Box<ModuleId>,
        /// Missing target module.
        target: Box<ModuleId>,
        /// Source range for the import candidate.
        range: SourceRange,
        /// Source-order ordinal for the import candidate.
        ordinal: usize,
        /// Underlying module-index lookup error.
        lookup: Box<ModuleIndexProviderError>,
    },
}

fn indexed_module_id(module: &ModuleId) -> IndexedModuleId {
    IndexedModuleId::new(module.package().clone(), module.path().clone())
}

fn detect_cycles(nodes: &BTreeSet<ModuleId>, edges: &[ImportGraphEdge]) -> Vec<ImportCycle> {
    let adjacency = adjacency(nodes, edges);
    let self_edges = edges
        .iter()
        .filter(|edge| edge.source() == edge.target())
        .map(|edge| edge.source().clone())
        .collect::<BTreeSet<_>>();
    let components = strongly_connected_components(nodes, &adjacency);
    components
        .into_iter()
        .filter(|component| component.len() > 1 || self_edges.contains(&component[0]))
        .map(|component| {
            let component_nodes = component.iter().cloned().collect::<BTreeSet<_>>();
            let cycle_edges = edges
                .iter()
                .filter(|edge| {
                    component_nodes.contains(edge.source())
                        && component_nodes.contains(edge.target())
                })
                .cloned()
                .collect();
            ImportCycle::new(component, cycle_edges)
        })
        .collect()
}

fn adjacency(
    nodes: &BTreeSet<ModuleId>,
    edges: &[ImportGraphEdge],
) -> BTreeMap<ModuleId, Vec<ModuleId>> {
    let mut adjacency = nodes
        .iter()
        .map(|node| (node.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in edges {
        adjacency
            .entry(edge.source().clone())
            .or_default()
            .insert(edge.target().clone());
    }
    adjacency
        .into_iter()
        .map(|(node, targets)| (node, targets.into_iter().collect()))
        .collect()
}

fn strongly_connected_components(
    nodes: &BTreeSet<ModuleId>,
    adjacency: &BTreeMap<ModuleId, Vec<ModuleId>>,
) -> Vec<Vec<ModuleId>> {
    let mut state = TarjanState::default();
    for node in nodes {
        if !state.indices.contains_key(node) {
            strong_connect(node.clone(), adjacency, &mut state);
        }
    }
    state.components
}

#[derive(Default)]
struct TarjanState {
    next_index: usize,
    indices: BTreeMap<ModuleId, usize>,
    lowlinks: BTreeMap<ModuleId, usize>,
    stack: Vec<ModuleId>,
    on_stack: BTreeSet<ModuleId>,
    components: Vec<Vec<ModuleId>>,
}

fn strong_connect(
    node: ModuleId,
    adjacency: &BTreeMap<ModuleId, Vec<ModuleId>>,
    state: &mut TarjanState,
) {
    let index = state.next_index;
    state.next_index += 1;
    state.indices.insert(node.clone(), index);
    state.lowlinks.insert(node.clone(), index);
    state.stack.push(node.clone());
    state.on_stack.insert(node.clone());

    if let Some(targets) = adjacency.get(&node) {
        for target in targets {
            if !state.indices.contains_key(target) {
                strong_connect(target.clone(), adjacency, state);
                let target_lowlink = state.lowlinks[target];
                let node_lowlink = state
                    .lowlinks
                    .get_mut(&node)
                    .expect("node lowlink must exist after recursive visit");
                *node_lowlink = (*node_lowlink).min(target_lowlink);
            } else if state.on_stack.contains(target) {
                let target_index = state.indices[target];
                let node_lowlink = state
                    .lowlinks
                    .get_mut(&node)
                    .expect("node lowlink must exist for stack target");
                *node_lowlink = (*node_lowlink).min(target_index);
            }
        }
    }

    if state.lowlinks[&node] == state.indices[&node] {
        let mut component = Vec::new();
        loop {
            let stacked = state
                .stack
                .pop()
                .expect("root node must be present on Tarjan stack");
            state.on_stack.remove(&stacked);
            let is_root = stacked == node;
            component.push(stacked);
            if is_root {
                break;
            }
        }
        component.sort();
        state.components.push(component);
    }
}

fn dependency_first_topological_order(
    nodes: &BTreeSet<ModuleId>,
    edges: &[ImportGraphEdge],
) -> Vec<ModuleId> {
    let mut dependency_counts = nodes
        .iter()
        .map(|node| (node.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut dependents_by_dependency = BTreeMap::<ModuleId, BTreeSet<ModuleId>>::new();

    for edge in edges {
        if let Some(count) = dependency_counts.get_mut(edge.source()) {
            *count += 1;
        }
        dependents_by_dependency
            .entry(edge.target().clone())
            .or_default()
            .insert(edge.source().clone());
    }

    let mut ready = dependency_counts
        .iter()
        .filter_map(|(node, count)| (*count == 0).then_some(node.clone()))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(nodes.len());

    while let Some(node) = ready.pop_first() {
        order.push(node.clone());
        if let Some(dependents) = dependents_by_dependency.get(&node) {
            for dependent in dependents {
                let count = dependency_counts
                    .get_mut(dependent)
                    .expect("dependent must be present in accepted node set");
                *count -= 1;
                if *count == 0 {
                    ready.insert(dependent.clone());
                }
            }
        }
    }

    order
}

fn import_edge_cmp(left: &ImportGraphEdge, right: &ImportGraphEdge) -> Ordering {
    left.source()
        .cmp(right.source())
        .then_with(|| left.target().cmp(right.target()))
        .then_with(|| edge_provenance_cmp(left, right))
}

fn edge_provenance_cmp(left: &ImportGraphEdge, right: &ImportGraphEdge) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
}

fn import_cycle_cmp(left: &ImportCycle, right: &ImportCycle) -> Ordering {
    first_cycle_edge(left)
        .and_then(|left_edge| {
            first_cycle_edge(right).map(|right_edge| cycle_edge_cmp(left_edge, right_edge))
        })
        .unwrap_or_else(|| left.edges().len().cmp(&right.edges().len()))
        .then_with(|| left.modules().cmp(right.modules()))
        .then_with(|| edge_slice_cmp(left.edges(), right.edges(), cycle_edge_cmp))
}

fn first_cycle_edge(cycle: &ImportCycle) -> Option<&ImportGraphEdge> {
    cycle.edges().first()
}

const fn range_key(range: SourceRange) -> (usize, usize) {
    (range.start, range.end)
}

fn cycle_edge_cmp(left: &ImportGraphEdge, right: &ImportGraphEdge) -> Ordering {
    range_key(left.range())
        .cmp(&range_key(right.range()))
        .then_with(|| left.source().cmp(right.source()))
        .then_with(|| left.target().cmp(right.target()))
        .then_with(|| left.ordinal().cmp(&right.ordinal()))
}

fn edge_slice_cmp(
    left: &[ImportGraphEdge],
    right: &[ImportGraphEdge],
    cmp: fn(&ImportGraphEdge, &ImportGraphEdge) -> Ordering,
) -> Ordering {
    for (left_edge, right_edge) in left.iter().zip(right.iter()) {
        let ordering = cmp(left_edge, right_edge);
        if !ordering.is_eq() {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_index::{
        DependencyModuleSummaryRef, ModuleIndexLocation, PackageIndexEntry,
        WorkspaceStubModuleIndexProvider,
    };
    use mizar_build::module_index::PackageIndexSource;
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceId,
    };
    use semver::Version;

    #[test]
    fn acyclic_fixture_builds_expected_graph_and_dependency_first_order() {
        let provider = fixture_provider();
        let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&[
                module_candidates(
                    module_id("app", "main"),
                    vec![
                        candidate("dep", "logic", 20, 24, 2),
                        candidate("app", "util", 10, 14, 0),
                        candidate("dep", "logic", 30, 34, 1),
                        candidate("dep", "logic", 25, 29, 1),
                    ],
                ),
                module_candidates(
                    module_id("app", "util"),
                    vec![candidate("dep", "logic", 40, 44, 0)],
                ),
                module_candidates(module_id("dep", "logic"), Vec::new()),
            ])
            .unwrap();

        assert!(!graph.has_cycles());
        assert_eq!(
            module_paths(graph.graph().nodes()),
            vec!["app:main", "app:util", "dep:logic"]
        );
        assert_eq!(
            edge_paths(graph.graph().edges()),
            vec![
                ("app:main", "app:util", 0, (10, 14)),
                ("app:main", "dep:logic", 1, (25, 29)),
                ("app:util", "dep:logic", 0, (40, 44)),
            ]
        );
        assert_eq!(
            module_paths(graph.graph().topological_order()),
            vec!["dep:logic", "app:util", "app:main"]
        );
    }

    #[test]
    fn independent_acyclic_components_use_canonical_ready_ties() {
        let provider = fixture_provider();
        let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&[
                module_candidates(
                    module_id("app", "main"),
                    vec![candidate("app", "util", 10, 14, 0)],
                ),
                module_candidates(
                    module_id("app", "alpha"),
                    vec![candidate("dep", "logic", 20, 24, 0)],
                ),
                module_candidates(module_id("app", "util"), Vec::new()),
                module_candidates(module_id("app", "beta"), Vec::new()),
                module_candidates(module_id("dep", "logic"), Vec::new()),
            ])
            .unwrap();

        assert_eq!(
            module_paths(graph.graph().topological_order()),
            vec!["app:beta", "app:util", "app:main", "dep:logic", "app:alpha",]
        );
    }

    #[test]
    fn cycle_fixture_is_rejected_deterministically() {
        let provider = fixture_provider();
        let first = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&cycle_fixture_one())
            .unwrap();
        let second = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&cycle_fixture_two())
            .unwrap();

        assert_eq!(first, second);
        assert!(first.has_cycles());
        assert_eq!(first.cycles().len(), 1);
        assert_eq!(
            module_paths(first.cycles()[0].modules()),
            vec!["app:main", "app:util"]
        );
        assert_eq!(
            edge_paths(first.cycles()[0].edges()),
            vec![
                ("app:main", "app:util", 0, (10, 14)),
                ("app:util", "app:main", 0, (20, 24)),
            ]
        );
        assert_eq!(
            module_paths(first.graph().topological_order()),
            vec!["dep:logic", "app:facade"]
        );
        assert_eq!(
            edge_paths(first.graph().edges()),
            vec![("app:facade", "dep:logic", 0, (30, 34))]
        );
    }

    #[test]
    fn self_cycle_is_rejected_deterministically() {
        let provider = fixture_provider();
        let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&[module_candidates(
                module_id("app", "main"),
                vec![candidate("app", "main", 50, 54, 0)],
            )])
            .unwrap();

        assert!(graph.has_cycles());
        assert_eq!(module_paths(graph.cycles()[0].modules()), vec!["app:main"]);
        assert!(graph.graph().nodes().is_empty());
        assert!(graph.graph().edges().is_empty());
        assert!(graph.graph().topological_order().is_empty());
    }

    #[test]
    fn independent_cycles_sort_by_source_provenance() {
        let provider = fixture_provider();
        let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&[
                module_candidates(
                    module_id("app", "alpha"),
                    vec![candidate("app", "beta", 100, 104, 0)],
                ),
                module_candidates(
                    module_id("app", "beta"),
                    vec![candidate("app", "alpha", 110, 114, 0)],
                ),
                module_candidates(
                    module_id("app", "yankee"),
                    vec![candidate("app", "zulu", 10, 14, 0)],
                ),
                module_candidates(
                    module_id("app", "zulu"),
                    vec![candidate("app", "yankee", 20, 24, 0)],
                ),
            ])
            .unwrap();

        assert_eq!(graph.cycles().len(), 2);
        assert_eq!(
            module_paths(graph.cycles()[0].modules()),
            vec!["app:yankee", "app:zulu"]
        );
        assert_eq!(
            module_paths(graph.cycles()[1].modules()),
            vec!["app:alpha", "app:beta"]
        );
    }

    #[test]
    fn unknown_modules_are_rejected_before_graph_publication() {
        let provider = fixture_provider();
        let builder = ImportGraphBuilder::new(ModuleIndexInput::new(&provider));

        let unknown_source = builder
            .build(&[module_candidates(module_id("missing", "main"), Vec::new())])
            .unwrap_err();
        match unknown_source {
            ImportGraphBuildError::UnknownSourceModule { module, lookup } => {
                assert_eq!(module_key(&module), "missing:main");
                match *lookup {
                    ModuleIndexProviderError::UnknownModule { module } => {
                        assert_eq!(module.package.as_str(), "missing");
                        assert_eq!(module.path.as_str(), "main");
                    }
                    other => panic!("expected unknown source module lookup, got {other:?}"),
                }
            }
            other => panic!("expected unknown source module, got {other:?}"),
        }

        let unknown_target = builder
            .build(&[module_candidates(
                module_id("app", "main"),
                vec![candidate("missing", "dep", 60, 64, 0)],
            )])
            .unwrap_err();
        match unknown_target {
            ImportGraphBuildError::UnknownTargetModule {
                source,
                target,
                range: actual_range,
                ordinal,
                lookup,
            } => {
                assert_eq!(module_key(&source), "app:main");
                assert_eq!(module_key(&target), "missing:dep");
                assert_eq!(actual_range, range(60, 64));
                assert_eq!(ordinal, 0);
                match *lookup {
                    ModuleIndexProviderError::UnknownModule { module } => {
                        assert_eq!(module.package.as_str(), "missing");
                        assert_eq!(module.path.as_str(), "dep");
                    }
                    other => panic!("expected unknown target module lookup, got {other:?}"),
                }
            }
            other => panic!("expected unknown target module, got {other:?}"),
        }
    }

    fn cycle_fixture_one() -> Vec<ModuleImportCandidates> {
        vec![
            module_candidates(
                module_id("app", "main"),
                vec![candidate("app", "util", 10, 14, 0)],
            ),
            module_candidates(
                module_id("app", "util"),
                vec![candidate("app", "main", 20, 24, 0)],
            ),
            module_candidates(
                module_id("app", "facade"),
                vec![candidate("dep", "logic", 30, 34, 0)],
            ),
            module_candidates(module_id("dep", "logic"), Vec::new()),
        ]
    }

    fn cycle_fixture_two() -> Vec<ModuleImportCandidates> {
        vec![
            module_candidates(module_id("dep", "logic"), Vec::new()),
            module_candidates(
                module_id("app", "facade"),
                vec![candidate("dep", "logic", 30, 34, 0)],
            ),
            module_candidates(
                module_id("app", "util"),
                vec![candidate("app", "main", 20, 24, 0)],
            ),
            module_candidates(
                module_id("app", "main"),
                vec![candidate("app", "util", 10, 14, 0)],
            ),
        ]
    }

    fn module_candidates(
        module: ModuleId,
        imports: Vec<ImportEdgeCandidate>,
    ) -> ModuleImportCandidates {
        ModuleImportCandidates::new(module, imports)
    }

    fn candidate(
        package: &str,
        path: &str,
        start: usize,
        end: usize,
        ordinal: usize,
    ) -> ImportEdgeCandidate {
        ImportEdgeCandidate::new(module_id(package, path), range(start, end), ordinal)
    }

    fn fixture_provider() -> WorkspaceStubModuleIndexProvider {
        WorkspaceStubModuleIndexProvider::new(
            vec![package("app"), package("dep")],
            Vec::new(),
            vec![
                workspace_module("app", "main"),
                workspace_module("app", "util"),
                workspace_module("app", "facade"),
                workspace_module("app", "alpha"),
                workspace_module("app", "beta"),
                workspace_module("app", "yankee"),
                workspace_module("app", "zulu"),
                dependency_module("dep", "logic"),
            ],
            vec![DependencyModuleSummaryRef {
                module: indexed_module("dep", "logic"),
                artifact: "dep.logic.summary".to_owned(),
                content_hash: Hash::from_bytes([9; Hash::BYTE_LEN]),
            }],
        )
    }

    fn package(package_id: &str) -> PackageIndexEntry {
        PackageIndexEntry {
            package_id: PackageId::new(package_id),
            version: Version::new(0, 1, 0),
            edition: Edition::new("2026"),
            source: PackageIndexSource::Workspace {
                package_root: format!("/workspace/{package_id}"),
                source_root: format!("/workspace/{package_id}/src"),
                manifest_path: format!("/workspace/{package_id}/mizar.toml"),
            },
            dependencies: Vec::new(),
        }
    }

    fn workspace_module(package_id: &str, path: &str) -> crate::module_index::ModuleIndexEntry {
        crate::module_index::ModuleIndexEntry {
            module: indexed_module(package_id, path),
            package_id: PackageId::new(package_id),
            module_path: ModulePath::new(path),
            location: ModuleIndexLocation::WorkspaceFile {
                source_root: format!("/workspace/{package_id}/src"),
                normalized_path: format!("/workspace/{package_id}/src/{path}.miz"),
                source_relative_path: format!("{path}.miz"),
            },
            edition: Edition::new("2026"),
        }
    }

    fn dependency_module(package_id: &str, path: &str) -> crate::module_index::ModuleIndexEntry {
        crate::module_index::ModuleIndexEntry {
            module: indexed_module(package_id, path),
            package_id: PackageId::new(package_id),
            module_path: ModulePath::new(path),
            location: ModuleIndexLocation::DependencySummary {
                artifact: format!("{package_id}.{path}.summary"),
                content_hash: Hash::from_bytes([9; Hash::BYTE_LEN]),
            },
            edition: Edition::new("2026"),
        }
    }

    fn indexed_module(package_id: &str, path: &str) -> IndexedModuleId {
        IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path))
    }

    fn module_id(package: &str, path: &str) -> ModuleId {
        ModuleId::new(PackageId::new(package), ModulePath::new(path))
    }

    fn module_paths(modules: &[ModuleId]) -> Vec<String> {
        modules
            .iter()
            .map(|module| format!("{}:{}", module.package().as_str(), module.path().as_str()))
            .collect()
    }

    fn edge_paths(edges: &[ImportGraphEdge]) -> Vec<(&str, &str, usize, (usize, usize))> {
        edges
            .iter()
            .map(|edge| {
                (
                    module_key(edge.source()),
                    module_key(edge.target()),
                    edge.ordinal(),
                    range_key(edge.range()),
                )
            })
            .collect()
    }

    fn module_key(module: &ModuleId) -> &str {
        match (module.package().as_str(), module.path().as_str()) {
            ("app", "main") => "app:main",
            ("app", "util") => "app:util",
            ("app", "facade") => "app:facade",
            ("app", "alpha") => "app:alpha",
            ("app", "beta") => "app:beta",
            ("app", "yankee") => "app:yankee",
            ("app", "zulu") => "app:zulu",
            ("dep", "logic") => "dep:logic",
            ("missing", "dep") => "missing:dep",
            ("missing", "main") => "missing:main",
            _ => "unknown",
        }
    }

    fn source_id() -> SourceId {
        let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "03".repeat(Hash::BYTE_LEN)
        ))
        .unwrap();
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id)
            .unwrap()
    }

    fn range(start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: source_id(),
            start,
            end,
        }
    }
}
