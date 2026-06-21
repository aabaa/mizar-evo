use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Done,
}

pub(super) fn validate_resolved_ast(
    source_id: SourceId,
    module_id: &ModuleId,
    nodes: &ResolvedArena,
    name_refs: &NameRefTable,
    label_refs: &LabelRefTable,
    imports: &ResolvedImports,
) -> Result<(), ResolvedAstError> {
    if nodes.node(nodes.root()).is_none() {
        return Err(ResolvedArenaError::InvalidRoot { root: nodes.root() }.into());
    }

    for (node_id, node) in nodes.iter() {
        if node.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, node.origin().anchor())?;
        if node.origin().module_id() != module_id {
            return Err(ResolvedAstError::NodeModuleMismatch { node: node_id });
        }
        validate_origin_import_edge(node.origin(), imports)?;
        if node.recovery() != recovery_from_origin(node.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        if let Some(key) = node.reference_key() {
            validate_node_reference_key(node_id, key, name_refs, label_refs, imports)?;
        }
    }

    for (_, entry) in name_refs.iter() {
        if nodes.node(entry.site().node()).is_none() {
            return Err(ResolvedAstError::InvalidNameReferenceSite {
                node: entry.site().node(),
            });
        }
        validate_source_range(source_id, entry.site().range())?;
        if entry.origin().module_id() != module_id {
            return Err(ResolvedAstError::OriginModuleMismatch);
        }
        if entry.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, entry.origin().anchor())?;
        validate_origin_import_edge(entry.origin(), imports)?;
        if entry.recovery() != recovery_from_origin(entry.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        validate_name_resolution(
            source_id,
            nodes,
            imports,
            entry.origin(),
            entry.resolution(),
        )?;
    }

    for (_, entry) in label_refs.iter() {
        if nodes.node(entry.site().node()).is_none() {
            return Err(ResolvedAstError::InvalidLabelReferenceSite {
                node: entry.site().node(),
            });
        }
        validate_source_range(source_id, entry.site().range())?;
        if entry.origin().module_id() != module_id {
            return Err(ResolvedAstError::OriginModuleMismatch);
        }
        if entry.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, entry.origin().anchor())?;
        validate_origin_import_edge(entry.origin(), imports)?;
        if entry.recovery() != recovery_from_origin(entry.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        validate_label_resolution(source_id, entry.resolution())?;
    }

    for (_, import) in imports.imports() {
        if nodes.node(import.owner()).is_none() {
            return Err(ResolvedAstError::InvalidDirectiveOwner {
                node: import.owner(),
            });
        }
        validate_source_range(source_id, import.range())?;
        if import.origin().module_id() != module_id {
            return Err(ResolvedAstError::OriginModuleMismatch);
        }
        if import.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, import.origin().anchor())?;
        validate_origin_import_edge(import.origin(), imports)?;
        if import.recovery() != recovery_from_origin(import.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        validate_import_resolution(source_id, import.resolution())?;
    }
    for (_, export) in imports.exports() {
        if nodes.node(export.owner()).is_none() {
            return Err(ResolvedAstError::InvalidDirectiveOwner {
                node: export.owner(),
            });
        }
        validate_source_range(source_id, export.range())?;
        if export.origin().module_id() != module_id {
            return Err(ResolvedAstError::OriginModuleMismatch);
        }
        if export.origin().source_id() != source_id {
            return Err(ResolvedAstError::OriginSourceMismatch);
        }
        validate_source_anchor(source_id, export.origin().anchor())?;
        validate_origin_import_edge(export.origin(), imports)?;
        if export.recovery() != recovery_from_origin(export.origin()) {
            return Err(ResolvedAstError::RecoveryOriginMismatch);
        }
        validate_export_target(source_id, export.target())?;
    }

    Ok(())
}

fn validate_origin_import_edge(
    origin: &SemanticOrigin,
    imports: &ResolvedImports,
) -> Result<(), ResolvedAstError> {
    if let Some(import) = origin.import_edge()
        && imports.import(import).is_none()
    {
        return Err(ResolvedAstError::InvalidImportEdge { import });
    }
    Ok(())
}

fn validate_node_reference_key(
    node_id: ResolvedNodeId,
    key: NodeReferenceKey,
    name_refs: &NameRefTable,
    label_refs: &LabelRefTable,
    imports: &ResolvedImports,
) -> Result<(), ResolvedAstError> {
    match key {
        NodeReferenceKey::Name(id) => {
            let Some(entry) = name_refs.get(id) else {
                return Err(ResolvedAstError::InvalidNodeReferenceKey { node: node_id, key });
            };
            if entry.site().node() != node_id {
                return Err(ResolvedAstError::NodeReferenceSiteMismatch { node: node_id, key });
            }
        }
        NodeReferenceKey::Label(id) => {
            let Some(entry) = label_refs.get(id) else {
                return Err(ResolvedAstError::InvalidNodeReferenceKey { node: node_id, key });
            };
            if entry.site().node() != node_id {
                return Err(ResolvedAstError::NodeReferenceSiteMismatch { node: node_id, key });
            }
        }
        NodeReferenceKey::Import(id) => {
            let Some(import) = imports.import(id) else {
                return Err(ResolvedAstError::InvalidNodeReferenceKey { node: node_id, key });
            };
            if import.owner() != node_id {
                return Err(ResolvedAstError::NodeReferenceSiteMismatch { node: node_id, key });
            }
        }
        NodeReferenceKey::Export(id) => {
            let Some(export) = imports.export(id) else {
                return Err(ResolvedAstError::InvalidNodeReferenceKey { node: node_id, key });
            };
            if export.owner() != node_id {
                return Err(ResolvedAstError::NodeReferenceSiteMismatch { node: node_id, key });
            }
        }
    }
    Ok(())
}

fn validate_name_resolution(
    source_id: SourceId,
    nodes: &ResolvedArena,
    imports: &ResolvedImports,
    origin: &SemanticOrigin,
    resolution: &NameResolution,
) -> Result<(), ResolvedAstError> {
    match resolution {
        NameResolution::Resolved(symbol) => {
            validate_source_range(source_id, symbol.range())?;
            if let Some(import) = symbol.import() {
                validate_import_edge(import, imports)?;
            }
            if symbol.import() != origin.import_edge() {
                return Err(ResolvedAstError::ImportProvenanceMismatch {
                    symbol_import: symbol.import(),
                    origin_import: origin.import_edge(),
                });
            }
        }
        NameResolution::ResolvedBuiltin(builtin) => {
            validate_source_range(source_id, builtin.range())?
        }
        NameResolution::DeferredSelector(selector) => {
            validate_source_range(source_id, selector.range())?;
            if nodes.node(selector.base()).is_none() {
                return Err(ResolvedAstError::InvalidDeferredSelectorBase {
                    node: selector.base(),
                });
            }
        }
        NameResolution::Ambiguous(ambiguous) => {
            validate_source_range(source_id, ambiguous.range())?;
        }
        NameResolution::Unresolved(unresolved) => {
            validate_source_range(source_id, unresolved.range())?
        }
    }
    Ok(())
}

fn validate_label_resolution(
    source_id: SourceId,
    resolution: &LabelResolution,
) -> Result<(), ResolvedAstError> {
    match resolution {
        LabelResolution::Resolved(label) => validate_source_range(source_id, label.range())?,
        LabelResolution::Ambiguous(ambiguous) => {
            validate_source_range(source_id, ambiguous.range())?;
        }
        LabelResolution::Unresolved(unresolved) => {
            validate_source_range(source_id, unresolved.range())?
        }
    }
    Ok(())
}

fn validate_import_resolution(
    source_id: SourceId,
    resolution: &ImportResolution,
) -> Result<(), ResolvedAstError> {
    match resolution {
        ImportResolution::Resolved(_) => {}
        ImportResolution::Ambiguous(_) => {}
        ImportResolution::Unresolved(unresolved) => {
            validate_source_range(source_id, unresolved.range())?;
        }
    }
    Ok(())
}

fn validate_export_target(
    source_id: SourceId,
    target: &ExportTarget,
) -> Result<(), ResolvedAstError> {
    match target {
        ExportTarget::Module(_) | ExportTarget::ImportAlias { .. } | ExportTarget::Symbol(_) => {}
        ExportTarget::Unresolved(unresolved) => {
            validate_source_range(source_id, unresolved.range())?
        }
    }
    Ok(())
}

fn validate_import_edge(
    import: ResolvedImportId,
    imports: &ResolvedImports,
) -> Result<(), ResolvedAstError> {
    if imports.import(import).is_none() {
        return Err(ResolvedAstError::InvalidImportEdge { import });
    }
    Ok(())
}

fn validate_source_range(source_id: SourceId, range: SourceRange) -> Result<(), ResolvedAstError> {
    if range.source_id != source_id {
        return Err(ResolvedAstError::PayloadSourceMismatch);
    }
    Ok(())
}

fn validate_source_anchor(
    source_id: SourceId,
    anchor: &SourceAnchor,
) -> Result<(), ResolvedAstError> {
    if let Some(anchor_source_id) = source_anchor_id(anchor)
        && anchor_source_id != source_id
    {
        return Err(ResolvedAstError::PayloadSourceMismatch);
    }
    Ok(())
}

fn source_anchor_id(anchor: &SourceAnchor) -> Option<SourceId> {
    match anchor {
        SourceAnchor::Range(range) => Some(range.source_id),
        SourceAnchor::Point { source_id, .. } => Some(*source_id),
        SourceAnchor::Generated(origin) => generated_span_anchor_id(origin.anchor()),
        _ => None,
    }
}

const fn generated_span_anchor_id(anchor: GeneratedSpanAnchor) -> Option<SourceId> {
    match anchor {
        GeneratedSpanAnchor::Range(range) => Some(range.source_id),
        GeneratedSpanAnchor::Point { source_id, .. } => Some(source_id),
        _ => None,
    }
}

pub(super) fn validate_nodes(nodes: &[ResolvedNode]) -> Result<(), ResolvedArenaError> {
    for (index, node) in nodes.iter().enumerate() {
        let node_id = ResolvedNodeId::new(index);
        for child in node.children() {
            if child.index() >= nodes.len() {
                return Err(ResolvedArenaError::InvalidChild {
                    node: node_id,
                    child: *child,
                });
            }
        }
    }

    let mut states = vec![None; nodes.len()];
    for index in 0..nodes.len() {
        visit_node(index, nodes, &mut states)?;
    }
    Ok(())
}

fn visit_node(
    index: usize,
    nodes: &[ResolvedNode],
    states: &mut [Option<VisitState>],
) -> Result<(), ResolvedArenaError> {
    match states[index] {
        Some(VisitState::Done) => return Ok(()),
        Some(VisitState::Visiting) => {
            return Err(ResolvedArenaError::Cycle {
                node: ResolvedNodeId::new(index),
            });
        }
        None => {}
    }

    states[index] = Some(VisitState::Visiting);
    for child in nodes[index].children() {
        visit_node(child.index(), nodes, states)?;
    }
    states[index] = Some(VisitState::Done);
    Ok(())
}

pub(super) const fn range_key(range: SourceRange) -> (usize, usize) {
    (range.start, range.end)
}

pub(super) const fn recovery_from_origin(origin: &SemanticOrigin) -> RecoveryState {
    if origin.recovered {
        RecoveryState::Recovered
    } else {
        RecoveryState::Normal
    }
}
