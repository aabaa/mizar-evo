//! Original compilation syntax capture. Endpoint callbacks remain compiler-owned;
//! structural construction is not authentication of their semantic bindings.

use mizar_frontend::orchestration::FrontendOutput;
use mizar_frontend::preprocess::LexicalSourceMap;
use mizar_session::{Hash, LoadingMap, SourceId, SourceOrigin, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind, SurfaceNodeView};

use crate::{
    module_summary::ModuleSummaryIdentity,
    verified_artifact::{VerifiedArtifact, VerifiedExport},
};

/// Complete publication-bound export identity supplied by the compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectByEndpoint {
    pub module: ModuleSummaryIdentity,
    pub publication_hash: Hash,
    pub export: VerifiedExport,
}

/// Compiler classification of one original syntactic target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DirectByTarget {
    Theorem(Box<DirectByEndpoint>),
    /// A positively resolved proof-step or other non-theorem reference.
    NonTheorem,
    /// Includes unsupported syntax and absent/ambiguous compiler bindings.
    Unmapped,
}

/// Explicit examination of a theorem's complete original body and proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectByExamination {
    pub owner: Option<DirectByEndpoint>,
    pub body_range: SourceRange,
    pub proof_range: Option<SourceRange>,
    pub complete: bool,
}

/// One target occurrence; repeated references are intentionally retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectByOccurrence {
    pub citing: Option<DirectByEndpoint>,
    pub by_ordinal: usize,
    pub target_ordinal: usize,
    pub by_range: SourceRange,
    pub target_range: SourceRange,
    pub target: DirectByTarget,
}

/// Immutable structural handoff. A consumer must separately authenticate source,
/// publication, endpoint identities and the entire declared finite scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectByProjection {
    module: ModuleSummaryIdentity,
    publication_hash: Hash,
    captured_source: (SourceId, SourceOrigin, Hash),
    loading_map: Option<LoadingMap>,
    lexical_source_map: LexicalSourceMap,
    examinations: Vec<DirectByExamination>,
    occurrences: Vec<DirectByOccurrence>,
    complete: bool,
}

impl DirectByProjection {
    pub fn module(&self) -> &ModuleSummaryIdentity {
        &self.module
    }
    pub fn publication_hash(&self) -> Hash {
        self.publication_hash
    }
    pub fn source_hash(&self) -> Hash {
        self.captured_source.2
    }
    pub fn captured_source(&self) -> &(SourceId, SourceOrigin, Hash) {
        &self.captured_source
    }
    pub fn loading_map(&self) -> Option<&LoadingMap> {
        self.loading_map.as_ref()
    }
    pub fn lexical_source_map(&self) -> &LexicalSourceMap {
        &self.lexical_source_map
    }
    pub fn examinations(&self) -> &[DirectByExamination] {
        &self.examinations
    }
    pub fn occurrences(&self) -> &[DirectByOccurrence] {
        &self.occurrences
    }
    pub fn complete(&self) -> bool {
        self.complete
    }
}

/// Capture original retained frontend syntax, never evidence or reparsed text.
/// The caller owns compilation lineage and supplies semantic endpoint bindings;
/// passing callbacks does not authenticate them. Wrong source binding refuses
/// capture. Unmapped owners/targets and recovered syntax retain incomplete scope.
pub fn project_frontend_direct_by(
    frontend: &FrontendOutput<SurfaceAst>,
    artifact: &VerifiedArtifact,
    publication_hash: Hash,
    mut bind_owner: impl FnMut(SurfaceNodeId) -> Option<VerifiedExport>,
    mut bind_target: impl FnMut(SurfaceNodeId, SurfaceNodeId) -> DirectByTarget,
) -> Option<DirectByProjection> {
    let source = &frontend.source;
    let captured = (source.source_id, source.origin.clone(), source.source_hash);
    let ast = frontend.ast.as_ref()?;
    if artifact.provenance.captured_source.as_ref() != Some(&captured)
        || artifact.source_hash != source.source_hash
        || artifact.source_file != source.normalized_path.as_str()
        || artifact.module.package_id != source.package_id.as_str()
        || artifact.module.module_path != source.module_path.as_str()
        || artifact.module.language_edition != source.edition.as_str()
        || ast.source_id != source.source_id
        || frontend.preprocessed.source_id != source.source_id
        || frontend.preprocessed.source_map.source_id != source.source_id
        || source
            .loading_map
            .as_ref()
            .is_some_and(|map| map.source_id != source.source_id)
    {
        return None;
    }
    let mut view = DirectByProjection {
        module: artifact.module.clone(),
        publication_hash,
        captured_source: captured,
        loading_map: source.loading_map.clone(),
        lexical_source_map: frontend.preprocessed.source_map.clone(),
        examinations: Vec::new(),
        occurrences: Vec::new(),
        complete: frontend.diagnostics.is_empty(),
    };
    // Root also retains the flat token stream beside the structural tree.
    let mut pending = ast
        .root_view()?
        .child_views()
        .filter(|node| matches!(node.kind(), SurfaceNodeKind::CompilationUnit))
        .collect::<Vec<_>>();
    view.complete &= pending.len() == 1;
    let mut visited = std::collections::BTreeSet::from([ast.root_view()?.id()]);
    while let Some(node) = pending.pop() {
        if !visited.insert(node.id()) || !valid_node(node, source.source_id) {
            view.complete = false;
            continue;
        }
        if matches!(
            node.kind(),
            SurfaceNodeKind::TheoremItem | SurfaceNodeKind::LemmaItem
        ) {
            let owner = bind_owner(node.id()).map(|export| DirectByEndpoint {
                module: artifact.module.clone(),
                publication_hash,
                export,
            });
            let proof = node
                .child_views()
                .find(|child| matches!(child.kind(), SurfaceNodeKind::ProofBlock));
            let mut complete = owner.is_some() && frontend.diagnostics.is_empty();
            let mut body = vec![node];
            let mut body_visited = std::collections::BTreeSet::new();
            let mut by_ordinal = 0;
            while let Some(part) = body.pop() {
                if !body_visited.insert(part.id()) || !valid_node(part, source.source_id) {
                    complete = false;
                    continue;
                }
                if matches!(part.kind(), SurfaceNodeKind::JustificationClause)
                    && part
                        .child_views()
                        .next()
                        .and_then(|child| child.as_token())
                        .is_some_and(|token| token.text.as_ref() == "by")
                {
                    let list = part
                        .child_views()
                        .find(|child| matches!(child.kind(), SurfaceNodeKind::ReferenceList));
                    if let Some(list) = list {
                        let mut targets = 0;
                        for target in list
                            .child_views()
                            .filter(|child| child.as_token().is_none())
                        {
                            // Grouped items are individual citations (spec16.5.2).
                            let items =
                                if matches!(target.kind(), SurfaceNodeKind::GroupedReference) {
                                    target
                                        .child_views()
                                        .filter(|item| {
                                            matches!(
                                                item.kind(),
                                                SurfaceNodeKind::GroupedReferenceItem
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                } else {
                                    vec![target]
                                };
                            for target in items {
                                let binding = bind_target(node.id(), target.id());
                                if matches!(binding, DirectByTarget::Unmapped) {
                                    complete = false;
                                }
                                view.occurrences.push(DirectByOccurrence {
                                    citing: owner.clone(),
                                    by_ordinal,
                                    target_ordinal: targets,
                                    by_range: part.range(),
                                    target_range: target.range(),
                                    target: binding,
                                });
                                targets += 1;
                            }
                        }
                        complete &= targets != 0;
                    } else {
                        complete = false;
                    }
                    by_ordinal += 1;
                }
                let children = part.child_views().collect::<Vec<_>>();
                body.extend(children.into_iter().rev());
            }
            view.complete &= complete;
            visited.extend(body_visited);
            view.examinations.push(DirectByExamination {
                owner,
                body_range: node.range(),
                proof_range: proof.map(|proof| proof.range()),
                complete,
            });
        } else {
            let children = node.child_views().collect::<Vec<_>>();
            pending.extend(children.into_iter().rev());
        }
    }
    view.complete &= ast.node_views().all(|node| visited.contains(&node.id()));
    Some(view)
}

fn valid_node(node: SurfaceNodeView<'_>, source_id: SourceId) -> bool {
    let range = node.range();
    let children = node.child_views().collect::<Vec<_>>();
    !node.is_recovered()
        && range.source_id == source_id
        && range.start <= range.end
        && children.iter().all(|child| {
            let child = child.range();
            child.source_id == source_id && child.start >= range.start && child.end <= range.end
        })
        && children
            .windows(2)
            .all(|pair| pair[0].range().end <= pair[1].range().start)
}
