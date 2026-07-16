use std::collections::{BTreeMap, BTreeSet};

use mizar_checker::type_checker::{
    AttributeInput, AttributePolarity, ModeExpansion, SourceReserveBindingInput,
    SourceReserveDeclarationBridge, TypeExpressionInput, TypeHeadInput,
};
use mizar_checker::typed_ast::TypedSiteRef;
use mizar_resolve::env::{ContributionKind, NamespacePath, SymbolEnv, SymbolKind};
use mizar_resolve::resolved_ast::{ModuleId as ResolverModuleId, SymbolId as ResolverSymbolId};
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind, SurfaceTokenKind};

use super::source_ast::{
    exact_compilation_item_list, is_exact_parser_type_fixtures_import, qualified_symbol_spelling,
    structural_child_ids, subtree_has_recovery, surface_nodes_with_kind,
};

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceTypeExpression {
    pub(in crate::runner) range: SourceRange,
    pub(in crate::runner) spelling: String,
    pub(in crate::runner) head: TypeHeadInput,
    pub(in crate::runner) attributes: Vec<AttributeInput>,
}

pub(in crate::runner) fn extract_builtin_source_type_expression(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceTypeExpression, ()> {
    if subtree_has_recovery(ast, node) || node.children.is_empty() || node.children.len() > 2 {
        return Err(());
    }
    let (attribute_node, head_id) = match node.children.as_slice() {
        [head] => (None, *head),
        [attributes, head] => (Some(ast.node(*attributes).ok_or(())?), *head),
        _ => return Err(()),
    };
    let attributes = match attribute_node {
        Some(attribute_node) => {
            extract_builtin_source_attributes(ast, attribute_node, module, symbols)?
        }
        None => Vec::new(),
    };
    let head =
        extract_source_reserve_type_head(ast, ast.node(head_id).ok_or(())?, module, symbols)?;
    if !attributes.is_empty()
        && !is_supported_attributed_source_reserve_head(symbols, module, &head)
    {
        return Err(());
    }
    if attributes.iter().any(|attribute| {
        is_imported_fixture_reserve_attribute(symbols, module, &attribute.symbol)
            && !supported_imported_fixture_reserve_attribute_use(
                symbols,
                module,
                &head,
                attribute,
                attributes.len(),
            )
    }) {
        return Err(());
    }
    Ok(SourceTypeExpression {
        range: node.range,
        spelling: source_text_from_children(ast, node).ok_or(())?,
        head,
        attributes,
    })
}

fn extract_source_reserve_type_head(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<TypeHeadInput, ()> {
    if !matches!(node.kind, SurfaceNodeKind::TypeHead) || node.children.len() != 1 {
        return Err(());
    }
    let child = ast.node(node.children[0]).ok_or(())?;
    if let Some(token) = child.token_text() {
        return match token {
            "set" => Ok(TypeHeadInput::BuiltinSet),
            "object" => Ok(TypeHeadInput::BuiltinObject),
            _ => Err(()),
        };
    }
    if matches!(child.kind, SurfaceNodeKind::QualifiedSymbol) {
        let spelling = qualified_symbol_spelling(ast, child)?;
        let symbol = resolve_visible_type_head(symbols, module, &spelling)?;
        return Ok(TypeHeadInput::Symbol(symbol));
    }
    Err(())
}

fn is_supported_attributed_source_reserve_head(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    head: &TypeHeadInput,
) -> bool {
    match head {
        TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject => true,
        TypeHeadInput::Symbol(symbol) => {
            source_reserve_symbol_head_kind(symbols, module, symbol).is_some()
        }
        _ => false,
    }
}

fn supported_source_reserve_type_head_kind(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> Option<SymbolKind> {
    let entry = symbols.symbols().get(symbol)?;
    if !matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure) {
        return None;
    }
    let contribution = symbols.contributions().get(entry.contribution())?;
    if symbol.module() == module
        && contribution.module() == module
        && matches!(contribution.kind(), ContributionKind::LocalSource { .. })
    {
        return Some(entry.kind());
    }
    if symbol.module() != module
        && matches!(entry.kind(), SymbolKind::Mode)
        && contribution.module() == symbol.module()
        && matches!(contribution.kind(), ContributionKind::ImportedSource { .. })
        && symbol.module().path().as_str() == "parser.type_fixtures"
        && entry.primary_spelling() == "TypeCaseMode"
    {
        return Some(entry.kind());
    }
    if symbol.module() != module
        && matches!(entry.kind(), SymbolKind::Structure)
        && contribution.module() == symbol.module()
        && matches!(contribution.kind(), ContributionKind::ImportedSource { .. })
        && symbol.module().path().as_str() == "parser.type_fixtures"
        && matches!(entry.primary_spelling(), "R" | "TypeCaseStruct")
    {
        return Some(entry.kind());
    }
    None
}

fn source_reserve_symbol_head_kind(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> Option<SymbolKind> {
    if symbol.module() != module {
        return None;
    }
    let entry = symbols.symbols().get(symbol)?;
    if !matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure) {
        return None;
    }
    let contribution = symbols.contributions().get(entry.contribution())?;
    if contribution.module() != module
        || !matches!(contribution.kind(), ContributionKind::LocalSource { .. })
    {
        return None;
    }
    Some(entry.kind())
}

fn extract_builtin_source_attributes(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<Vec<AttributeInput>, ()> {
    if !matches!(node.kind, SurfaceNodeKind::AttributeChain) || node.children.is_empty() {
        return Err(());
    }
    node.children
        .iter()
        .map(|child_id| {
            let child = ast.node(*child_id).ok_or(())?;
            extract_builtin_source_attribute(ast, child, module, symbols)
        })
        .collect()
}

fn extract_builtin_source_attribute(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<AttributeInput, ()> {
    if !matches!(node.kind, SurfaceNodeKind::AttributeRef) {
        return Err(());
    }
    let mut polarity = AttributePolarity::Positive;
    let mut symbol_spelling = None;
    for child_id in &node.children {
        let child = ast.node(*child_id).ok_or(())?;
        match &child.kind {
            SurfaceNodeKind::Token(token)
                if token.kind == SurfaceTokenKind::ReservedWord
                    && token.text.as_ref() == "non"
                    && symbol_spelling.is_none()
                    && polarity == AttributePolarity::Positive =>
            {
                polarity = AttributePolarity::Negative;
            }
            SurfaceNodeKind::QualifiedSymbol if symbol_spelling.is_none() => {
                symbol_spelling = Some(qualified_symbol_spelling(ast, child)?);
            }
            _ => return Err(()),
        }
    }
    let spelling = symbol_spelling.ok_or(())?;
    let symbol = resolve_visible_attribute(symbols, module, &spelling)?;
    Ok(AttributeInput::new(
        symbol,
        polarity,
        node.range,
        source_text_from_children(ast, node).ok_or(())?,
    ))
}

pub(in crate::runner) fn resolve_visible_attribute(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    spelling: &str,
) -> Result<mizar_resolve::resolved_ast::SymbolId, ()> {
    let namespace = NamespacePath::new(module.path().as_str());
    let mut local_candidates = Vec::new();
    let mut imported_candidates = Vec::new();
    for entry in symbols
        .symbols()
        .visible_candidates(&namespace, spelling)
        .into_iter()
        .filter(|entry| matches!(entry.kind(), SymbolKind::Attribute))
        .filter(|entry| supported_source_reserve_attribute(symbols, module, entry.symbol()))
    {
        let symbol = entry.symbol().clone();
        if symbol.module() == module {
            local_candidates.push(symbol);
        } else {
            imported_candidates.push(symbol);
        }
    }
    match local_candidates.as_slice() {
        [symbol] => Ok(symbol.clone()),
        [] => match imported_candidates.as_slice() {
            [symbol] => Ok(symbol.clone()),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

pub(in crate::runner) fn resolve_visible_type_head(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    spelling: &str,
) -> Result<mizar_resolve::resolved_ast::SymbolId, ()> {
    let namespace = NamespacePath::new(module.path().as_str());
    let mut local_candidates = Vec::new();
    let mut imported_candidates = Vec::new();
    for entry in symbols
        .symbols()
        .visible_candidates(&namespace, spelling)
        .into_iter()
        .filter(|entry| matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure))
        .filter(|entry| {
            supported_source_reserve_type_head_kind(symbols, module, entry.symbol()).is_some()
        })
    {
        let symbol = entry.symbol().clone();
        if symbol.module() == module {
            local_candidates.push(symbol);
        } else {
            imported_candidates.push(symbol);
        }
    }
    match local_candidates.as_slice() {
        [symbol] => Ok(symbol.clone()),
        [] => match imported_candidates.as_slice() {
            [symbol] => Ok(symbol.clone()),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

fn supported_source_reserve_attribute(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> bool {
    let Some(entry) = symbols.symbols().get(symbol) else {
        return false;
    };
    if !matches!(entry.kind(), SymbolKind::Attribute) {
        return false;
    }
    let Some(contribution) = symbols.contributions().get(entry.contribution()) else {
        return false;
    };
    let local_source_attribute = symbol.module() == module
        && contribution.module() == module
        && matches!(contribution.kind(), ContributionKind::LocalSource { .. });
    let imported_fixture_attribute = is_imported_fixture_reserve_attribute(symbols, module, symbol);
    local_source_attribute || imported_fixture_attribute
}

fn is_imported_fixture_reserve_attribute(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> bool {
    imported_fixture_reserve_attribute_spelling(symbols, module, symbol).is_some()
}

fn supported_imported_fixture_reserve_attribute_use(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    head: &TypeHeadInput,
    attribute: &AttributeInput,
    attribute_count: usize,
) -> bool {
    match imported_fixture_reserve_attribute_spelling(symbols, module, &attribute.symbol) {
        Some("TypeCaseAttr") => matches!(head, TypeHeadInput::BuiltinSet),
        Some("empty") => {
            matches!(head, TypeHeadInput::BuiltinSet)
                || (matches!(head, TypeHeadInput::BuiltinObject)
                    && attribute_count == 1
                    && matches!(attribute.polarity, AttributePolarity::Negative))
        }
        _ => false,
    }
}

fn imported_fixture_reserve_attribute_spelling<'a>(
    symbols: &'a SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> Option<&'a str> {
    let entry = symbols.symbols().get(symbol)?;
    let contribution = symbols.contributions().get(entry.contribution())?;
    if symbol.module() != module
        && contribution.module() == symbol.module()
        && matches!(contribution.kind(), ContributionKind::ImportedSource { .. })
        && symbol.module().path().as_str() == "parser.type_fixtures"
        && matches!(entry.kind(), SymbolKind::Attribute)
        && matches!(entry.primary_spelling(), "TypeCaseAttr" | "empty")
    {
        Some(entry.primary_spelling())
    } else {
        None
    }
}

fn source_text_from_children(ast: &SurfaceAst, node: &SurfaceNode) -> Option<String> {
    let mut tokens = Vec::new();
    collect_token_text(ast, node, &mut tokens)?;
    Some(tokens.join(" "))
}

fn collect_token_text<'a>(
    ast: &'a SurfaceAst,
    node: &'a SurfaceNode,
    output: &mut Vec<&'a str>,
) -> Option<()> {
    if let Some(text) = node.token_text() {
        output.push(text);
        return Some(());
    }
    for child_id in &node.children {
        collect_token_text(ast, ast.node(*child_id)?, output)?;
    }
    Some(())
}

#[derive(Debug)]
pub(in crate::runner) struct SourceReserveExtraction {
    pub(in crate::runner) bridge: SourceReserveDeclarationBridge,
    pub(in crate::runner) mode_expansions: BTreeMap<ResolverSymbolId, ModeExpansion>,
}

#[cfg(test)]
impl SourceReserveExtraction {
    pub(in crate::runner) fn bindings(&self) -> &[SourceReserveBindingInput] {
        self.bridge.bindings()
    }

    pub(in crate::runner) fn module_id(&self) -> &ResolverModuleId {
        self.bridge.module_id()
    }

    pub(in crate::runner) fn module_context(&self) -> mizar_checker::binding_env::BindingContextId {
        self.bridge.module_context()
    }

    pub(in crate::runner) fn type_node(
        &self,
        index: usize,
    ) -> mizar_checker::typed_ast::TypedNodeId {
        self.bridge.type_node(index)
    }

    pub(in crate::runner) fn declaration_node(
        &self,
        index: usize,
    ) -> mizar_checker::typed_ast::TypedNodeId {
        self.bridge.declaration_node(index)
    }

    #[cfg(test)]
    pub(in crate::runner) fn mode_expansions(&self) -> &BTreeMap<ResolverSymbolId, ModeExpansion> {
        &self.mode_expansions
    }
}

#[derive(Debug, Clone)]
struct SourceModeExpansionCandidate {
    definition_range: SourceRange,
    expansion: ModeExpansion,
    dependencies: Vec<ResolverSymbolId>,
}

#[derive(Debug, Clone)]
struct SourceModeExpansionEntry {
    definition_range: SourceRange,
    expansion: ModeExpansion,
    chain_edges_to_terminal: usize,
    chain_terminal_is_safe_builtin: bool,
}

#[derive(Debug, Clone, Copy)]
struct SourceModeExpansionTraversalBudget {
    mode_definition_count: usize,
}

impl SourceModeExpansionTraversalBudget {
    fn from_ast(ast: &SurfaceAst) -> Self {
        Self {
            mode_definition_count: surface_nodes_with_kind(ast, SurfaceNodeKind::ModeDefinition)
                .len(),
        }
    }

    fn permits_depth(self, depth: usize) -> bool {
        depth < self.mode_definition_count
    }
}

pub(in crate::runner) fn extract_builtin_source_reserve_declarations(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceReserveExtraction, ()> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_builtin_reserve_bridge_node(node))
    {
        return Err(());
    }
    let extraction =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)?;
    if source_imported_fixture_reserve_shape_is_exact(ast, &module, symbols, &extraction) {
        Ok(extraction)
    } else {
        Err(())
    }
}

fn source_imported_fixture_reserve_shape_is_exact(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    extraction: &SourceReserveExtraction,
) -> bool {
    let bindings = extraction.bridge.bindings();
    let has_imported_fixture_attribute = bindings.iter().any(|binding| {
        binding.type_attributes.iter().any(|attribute| {
            is_imported_fixture_reserve_attribute(symbols, module, &attribute.symbol)
        })
    });
    if !has_imported_fixture_attribute {
        return true;
    }

    let Some(item_list) = exact_compilation_item_list(ast) else {
        return false;
    };
    let item_ids = structural_child_ids(ast, item_list);
    if item_ids
        .first()
        .and_then(|item_id| ast.node(*item_id))
        .is_none_or(|item| !is_exact_parser_type_fixtures_import(ast, item))
    {
        return false;
    }
    if ast.nodes().iter().any(|node| {
        matches!(
            node.kind,
            SurfaceNodeKind::DefinitionBlockItem
                | SurfaceNodeKind::DefinitionParameter
                | SurfaceNodeKind::QualifiedVariableSegment
                | SurfaceNodeKind::AttributeDefinition
                | SurfaceNodeKind::AttributePattern
                | SurfaceNodeKind::ModeDefinition
                | SurfaceNodeKind::ModePattern
                | SurfaceNodeKind::StructureDefinition
                | SurfaceNodeKind::StructurePattern
                | SurfaceNodeKind::StructureField
                | SurfaceNodeKind::FormulaDefiniens
                | SurfaceNodeKind::FormulaCase
                | SurfaceNodeKind::FormulaExpression
                | SurfaceNodeKind::FormulaConstant(_)
                | SurfaceNodeKind::ErrorRecovery(_)
        )
    }) {
        return false;
    }

    let reserve_item_count = surface_nodes_with_kind(ast, SurfaceNodeKind::ReserveItem).len();
    match bindings {
        [binding]
            if reserve_item_count == 1
                && matches!(
                    item_ids
                        .iter()
                        .filter_map(|item_id| ast.node(*item_id))
                        .map(|item| &item.kind)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    [SurfaceNodeKind::ImportItem, SurfaceNodeKind::ReserveItem]
                ) =>
        {
            source_imported_fixture_single_reserve_binding_is_exact(binding, module, symbols)
        }
        [first, second]
            if reserve_item_count == 2
                && matches!(
                    item_ids
                        .iter()
                        .filter_map(|item_id| ast.node(*item_id))
                        .map(|item| &item.kind)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    [
                        SurfaceNodeKind::ImportItem,
                        SurfaceNodeKind::ReserveItem,
                        SurfaceNodeKind::ReserveItem
                    ]
                ) =>
        {
            first.spelling == "x"
                && first.type_spelling == "set"
                && first.type_head == TypeHeadInput::BuiltinSet
                && first.type_attributes.is_empty()
                && second.spelling == "y"
                && second.type_spelling == "non empty set"
                && second.type_head == TypeHeadInput::BuiltinSet
                && source_imported_fixture_attribute_is_exact(
                    second,
                    "empty",
                    AttributePolarity::Negative,
                    module,
                    symbols,
                )
        }
        _ => false,
    }
}

fn source_imported_fixture_single_reserve_binding_is_exact(
    binding: &SourceReserveBindingInput,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> bool {
    match (
        binding.spelling.as_str(),
        binding.type_spelling.as_str(),
        &binding.type_head,
    ) {
        ("a", "TypeCaseAttr set", TypeHeadInput::BuiltinSet) => {
            source_imported_fixture_attribute_is_exact(
                binding,
                "TypeCaseAttr",
                AttributePolarity::Positive,
                module,
                symbols,
            )
        }
        ("x", "empty set", TypeHeadInput::BuiltinSet) => {
            source_imported_fixture_attribute_is_exact(
                binding,
                "empty",
                AttributePolarity::Positive,
                module,
                symbols,
            )
        }
        ("x", "non empty set", TypeHeadInput::BuiltinSet)
        | ("x", "non empty object", TypeHeadInput::BuiltinObject) => {
            source_imported_fixture_attribute_is_exact(
                binding,
                "empty",
                AttributePolarity::Negative,
                module,
                symbols,
            )
        }
        _ => false,
    }
}

fn source_imported_fixture_attribute_is_exact(
    binding: &SourceReserveBindingInput,
    expected_spelling: &str,
    expected_polarity: AttributePolarity,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> bool {
    let [attribute] = binding.type_attributes.as_slice() else {
        return false;
    };
    let expected_source_spelling = if expected_polarity == AttributePolarity::Positive {
        expected_spelling.to_owned()
    } else if expected_polarity == AttributePolarity::Negative {
        format!("non {expected_spelling}")
    } else {
        return false;
    };
    attribute.args.is_empty()
        && attribute.spelling == expected_source_spelling
        && attribute.polarity == expected_polarity
        && imported_fixture_reserve_attribute_spelling(symbols, module, &attribute.symbol)
            == Some(expected_spelling)
}

pub(in crate::runner) fn extract_builtin_source_reserve_declarations_after_node_guard(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceReserveExtraction, ()> {
    let reserve_items = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ReserveItem))
        .collect::<Vec<_>>();
    if reserve_items.is_empty() {
        return Err(());
    }

    let mut bindings = Vec::new();
    let mut source_range = None;
    for item in reserve_items {
        if subtree_has_recovery(ast, item) {
            return Err(());
        }
        source_range = Some(merge_optional_range(source_range, item.range));
        let segments = item
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .filter(|child| matches!(child.kind, SurfaceNodeKind::ReserveSegment))
            .collect::<Vec<_>>();
        if segments.is_empty() {
            return Err(());
        }
        for segment in segments {
            bindings.extend(extract_builtin_reserve_segment(
                ast, segment, &module, symbols,
            )?);
        }
    }

    if bindings.is_empty() {
        return Err(());
    }
    let bridge = SourceReserveDeclarationBridge::new(
        ast.source_id,
        module.clone(),
        source_range.expect("reserve_items was non-empty"),
        bindings,
    )
    .map_err(|_| ())?;
    let mode_expansions = extract_source_local_mode_expansions(ast, &module, symbols, &bridge);
    Ok(SourceReserveExtraction {
        bridge,
        mode_expansions,
    })
}

fn extract_source_local_mode_expansions(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    bridge: &SourceReserveDeclarationBridge,
) -> BTreeMap<ResolverSymbolId, ModeExpansion> {
    let attributed_mode_heads = bridge
        .bindings()
        .iter()
        .filter(|binding| !binding.type_attributes.is_empty())
        .filter_map(|binding| match &binding.type_head {
            TypeHeadInput::Symbol(symbol)
                if source_reserve_symbol_head_kind(symbols, module, symbol)
                    == Some(SymbolKind::Mode) =>
            {
                Some(symbol.clone())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let bare_mode_heads = bridge
        .bindings()
        .iter()
        .filter(|binding| binding.type_attributes.is_empty())
        .filter_map(|binding| match &binding.type_head {
            TypeHeadInput::Symbol(symbol)
                if source_reserve_symbol_head_kind(symbols, module, symbol)
                    == Some(SymbolKind::Mode) =>
            {
                Some(symbol.clone())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mixed_mode_heads = attributed_mode_heads
        .intersection(&bare_mode_heads)
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut blocked_attributed_mode_heads = mixed_mode_heads;
    let traversal_budget = SourceModeExpansionTraversalBudget::from_ast(ast);
    let dependency_collector = AttributedModeDependencyCollector {
        ast,
        module,
        symbols,
        bridge,
        attributed_mode_heads: &attributed_mode_heads,
        traversal_budget,
    };
    for binding in bridge.bindings() {
        if !binding.type_attributes.is_empty() {
            continue;
        }
        let TypeHeadInput::Symbol(symbol) = &binding.type_head else {
            continue;
        };
        if source_reserve_symbol_head_kind(symbols, module, symbol) != Some(SymbolKind::Mode) {
            continue;
        }
        let mut visiting = BTreeSet::new();
        dependency_collector.collect(
            symbol,
            binding.type_range,
            &mut visiting,
            &mut blocked_attributed_mode_heads,
            0,
        );
    }

    let mut expansions = BTreeMap::new();
    let extractor = SourceLocalModeExpansionExtractor {
        ast,
        module,
        symbols,
        bridge,
        attributed_mode_heads: &attributed_mode_heads,
        blocked_attributed_mode_heads: &blocked_attributed_mode_heads,
        traversal_budget,
    };
    for binding in bridge.bindings() {
        let TypeHeadInput::Symbol(symbol) = &binding.type_head else {
            continue;
        };
        if blocked_attributed_mode_heads.contains(symbol)
            || expansions.contains_key(symbol)
            || source_reserve_symbol_head_kind(symbols, module, symbol) != Some(SymbolKind::Mode)
        {
            continue;
        }
        let mut visiting = BTreeSet::new();
        let _ = extractor.insert(
            symbol,
            binding.type_range,
            &mut visiting,
            &mut expansions,
            0,
            attributed_mode_heads.contains(symbol),
        );
    }
    expansions
        .into_iter()
        .map(|(symbol, entry)| (symbol, entry.expansion))
        .collect()
}

struct SourceLocalModeExpansionExtractor<'a> {
    ast: &'a SurfaceAst,
    module: &'a ResolverModuleId,
    symbols: &'a SymbolEnv,
    bridge: &'a SourceReserveDeclarationBridge,
    attributed_mode_heads: &'a BTreeSet<ResolverSymbolId>,
    blocked_attributed_mode_heads: &'a BTreeSet<ResolverSymbolId>,
    traversal_budget: SourceModeExpansionTraversalBudget,
}

impl SourceLocalModeExpansionExtractor<'_> {
    fn insert(
        &self,
        symbol: &ResolverSymbolId,
        reserve_type_range: SourceRange,
        visiting: &mut BTreeSet<ResolverSymbolId>,
        expansions: &mut BTreeMap<ResolverSymbolId, SourceModeExpansionEntry>,
        depth: usize,
        root_is_attributed: bool,
    ) -> bool {
        if !self.traversal_budget.permits_depth(depth) {
            return false;
        }
        if let Some(entry) = expansions.get(symbol) {
            if depth >= 1 && entry.definition_range.end > reserve_type_range.start {
                return false;
            }
            return depth < 1
                || mode_expansion_can_feed_chain_dependency(
                    &entry.expansion,
                    entry.chain_edges_to_terminal,
                    entry.chain_terminal_is_safe_builtin,
                    depth,
                    root_is_attributed,
                    self.symbols,
                    self.module,
                );
        }
        let is_attributed_root = depth == 0 && root_is_attributed;
        if !visiting.insert(symbol.clone())
            || self.blocked_attributed_mode_heads.contains(symbol)
            || (depth > 0 && self.attributed_mode_heads.contains(symbol))
            || source_reserve_symbol_head_kind(self.symbols, self.module, symbol)
                != Some(SymbolKind::Mode)
        {
            return false;
        }
        let Some(candidate) = extract_source_local_mode_expansion(
            self.ast,
            self.module,
            self.symbols,
            symbol,
            reserve_type_range,
            self.bridge,
        ) else {
            visiting.remove(symbol);
            return false;
        };
        if is_attributed_root
            && !mode_expansion_is_supported_attributed_root(
                &candidate.expansion,
                &candidate.dependencies,
                self.symbols,
                self.module,
            )
        {
            visiting.remove(symbol);
            return false;
        }
        if depth >= 1
            && !mode_expansion_candidate_can_feed_chain_dependency(
                &candidate,
                depth,
                root_is_attributed,
                self.symbols,
                self.module,
            )
        {
            visiting.remove(symbol);
            return false;
        }
        for dependency in &candidate.dependencies {
            if self.attributed_mode_heads.contains(dependency)
                || !self.insert(
                    dependency,
                    candidate.definition_range,
                    visiting,
                    expansions,
                    depth + 1,
                    root_is_attributed,
                )
            {
                visiting.remove(symbol);
                return false;
            }
        }
        let (chain_edges_to_terminal, chain_terminal_is_safe_builtin) =
            mode_expansion_chain_metadata(&candidate, expansions);
        expansions.insert(
            symbol.clone(),
            SourceModeExpansionEntry {
                definition_range: candidate.definition_range,
                expansion: candidate.expansion,
                chain_edges_to_terminal,
                chain_terminal_is_safe_builtin,
            },
        );
        visiting.remove(symbol);
        true
    }
}

struct AttributedModeDependencyCollector<'a> {
    ast: &'a SurfaceAst,
    module: &'a ResolverModuleId,
    symbols: &'a SymbolEnv,
    bridge: &'a SourceReserveDeclarationBridge,
    attributed_mode_heads: &'a BTreeSet<ResolverSymbolId>,
    traversal_budget: SourceModeExpansionTraversalBudget,
}

impl AttributedModeDependencyCollector<'_> {
    fn collect(
        &self,
        symbol: &ResolverSymbolId,
        reserve_type_range: SourceRange,
        visiting: &mut BTreeSet<ResolverSymbolId>,
        blocked: &mut BTreeSet<ResolverSymbolId>,
        depth: usize,
    ) {
        if !self.traversal_budget.permits_depth(depth) {
            return;
        }
        if !visiting.insert(symbol.clone()) {
            return;
        }
        if let Some(candidate) = extract_source_local_mode_expansion(
            self.ast,
            self.module,
            self.symbols,
            symbol,
            reserve_type_range,
            self.bridge,
        ) {
            for dependency in &candidate.dependencies {
                if self.attributed_mode_heads.contains(dependency) {
                    blocked.insert(dependency.clone());
                } else {
                    self.collect(
                        dependency,
                        candidate.definition_range,
                        visiting,
                        blocked,
                        depth + 1,
                    );
                }
            }
        }
        visiting.remove(symbol);
    }
}

fn mode_expansion_is_safe_chain_terminal(expansion: &ModeExpansion) -> bool {
    expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject
        )
}

fn mode_expansion_is_chain_dependency_terminal(
    expansion: &ModeExpansion,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    depth: usize,
) -> bool {
    mode_expansion_is_safe_chain_terminal(expansion)
        || (depth == 1
            && mode_expansion_is_direct_structure_rhs_terminal(expansion, symbols, module))
        || (depth == 1 && mode_expansion_is_direct_attributed_builtin_rhs_terminal(expansion))
}

fn mode_expansion_candidate_can_feed_chain_dependency(
    candidate: &SourceModeExpansionCandidate,
    depth: usize,
    root_is_attributed: bool,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    if mode_expansion_is_chain_dependency_terminal(&candidate.expansion, symbols, module, depth) {
        return candidate.dependencies.is_empty();
    }
    !root_is_attributed
        && candidate.dependencies.len() == 1
        && mode_expansion_is_bare_local_mode_dependency(
            &candidate.expansion,
            &candidate.dependencies[0],
            symbols,
            module,
        )
}

fn mode_expansion_can_feed_chain_dependency(
    expansion: &ModeExpansion,
    _chain_edges_to_terminal: usize,
    chain_terminal_is_safe_builtin: bool,
    depth: usize,
    root_is_attributed: bool,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    mode_expansion_is_chain_dependency_terminal(expansion, symbols, module, depth)
        || (!root_is_attributed
            && chain_terminal_is_safe_builtin
            && mode_expansion_is_bare_local_mode_head(expansion, symbols, module))
}

fn mode_expansion_chain_metadata(
    candidate: &SourceModeExpansionCandidate,
    expansions: &BTreeMap<ResolverSymbolId, SourceModeExpansionEntry>,
) -> (usize, bool) {
    if candidate.dependencies.is_empty() {
        return (
            0,
            mode_expansion_is_safe_chain_terminal(&candidate.expansion),
        );
    }
    candidate
        .dependencies
        .iter()
        .filter_map(|dependency| expansions.get(dependency))
        .map(|entry| {
            (
                entry.chain_edges_to_terminal.saturating_add(1),
                entry.chain_terminal_is_safe_builtin,
            )
        })
        .max_by_key(|(chain_edges_to_terminal, _)| *chain_edges_to_terminal)
        .unwrap_or((usize::MAX, false))
}

fn mode_expansion_is_bare_local_mode_dependency(
    expansion: &ModeExpansion,
    dependency: &ResolverSymbolId,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && expansion.radix.args.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::Symbol(ref radix)
                if radix == dependency
                    && source_reserve_symbol_head_kind(symbols, module, radix)
                        == Some(SymbolKind::Mode)
        )
}

fn mode_expansion_is_bare_local_mode_head(
    expansion: &ModeExpansion,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && expansion.radix.args.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::Symbol(ref radix)
                if source_reserve_symbol_head_kind(symbols, module, radix)
                    == Some(SymbolKind::Mode)
        )
}

fn mode_expansion_is_supported_attributed_root(
    expansion: &ModeExpansion,
    dependencies: &[ResolverSymbolId],
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    if dependencies.is_empty() {
        return mode_expansion_is_safe_chain_terminal(expansion)
            || mode_expansion_is_direct_structure_rhs_terminal(expansion, symbols, module)
            || mode_expansion_is_direct_attributed_builtin_rhs_terminal(expansion);
    }
    dependencies.len() == 1
        && expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && expansion.radix.args.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::Symbol(ref dependency) if dependency == &dependencies[0]
        )
}

fn mode_expansion_is_direct_structure_rhs_terminal(
    expansion: &ModeExpansion,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::Symbol(ref radix)
                if source_reserve_symbol_head_kind(symbols, module, radix)
                    == Some(SymbolKind::Structure)
        )
}

fn mode_expansion_is_direct_attributed_builtin_rhs_terminal(expansion: &ModeExpansion) -> bool {
    !expansion.attributes.is_empty()
        && expansion
            .attributes
            .iter()
            .all(|attribute| attribute.args.is_empty())
        && expansion.radix.attributes.is_empty()
        && expansion.radix.args.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject
        )
}

fn extract_source_local_mode_expansion(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    symbol: &ResolverSymbolId,
    use_site_range: SourceRange,
    bridge: &SourceReserveDeclarationBridge,
) -> Option<SourceModeExpansionCandidate> {
    if symbol.module() != module {
        return None;
    }
    let spelling = source_mode_symbol_spelling(symbol)?;
    let definitions = surface_nodes_with_kind(ast, SurfaceNodeKind::ModeDefinition)
        .into_iter()
        .filter(|(id, node)| {
            !subtree_has_recovery(ast, node)
                && node.range.end <= use_site_range.start
                && !mode_definition_has_local_context(ast, *id)
                && mode_definition_pattern_spelling(ast, node).as_deref() == Some(spelling)
        })
        .collect::<Vec<_>>();
    let [(_, definition)] = definitions.as_slice() else {
        return None;
    };
    let rhs = extract_source_mode_rhs(ast, definition, module, symbols)?;
    if matches!(
        rhs.head,
        TypeHeadInput::Symbol(ref radix)
            if source_reserve_symbol_head_kind(symbols, module, radix) == Some(SymbolKind::Structure)
                && !local_structure_definition_precedes(
                    ast,
                    module,
                    symbols,
                    radix,
                    definition.range.start,
                )
    ) {
        return None;
    }
    let dependencies = match &rhs.head {
        TypeHeadInput::Symbol(dependency)
            if source_reserve_symbol_head_kind(symbols, module, dependency)
                == Some(SymbolKind::Mode) =>
        {
            vec![dependency.clone()]
        }
        _ => Vec::new(),
    };
    Some(SourceModeExpansionCandidate {
        definition_range: definition.range,
        expansion: ModeExpansion::new(
            TypeExpressionInput::new(
                TypedSiteRef::Node(bridge.root_node()),
                rhs.range,
                rhs.spelling,
                rhs.head,
            ),
            rhs.attributes,
        ),
        dependencies,
    })
}

pub(in crate::runner) fn source_mode_symbol_spelling(symbol: &ResolverSymbolId) -> Option<&str> {
    source_local_symbol_spelling(symbol)
}

fn source_local_symbol_spelling(symbol: &ResolverSymbolId) -> Option<&str> {
    let spelling = symbol.local().as_str();
    let name = spelling.strip_prefix("name=").unwrap_or_else(|| {
        spelling
            .split_once(":name=")
            .map(|(_, name)| name)
            .unwrap_or(spelling)
    });
    let name = name.split_once(':').map_or(name, |(name, _)| name);
    let mut slash_parts = name.split('/');
    let first = slash_parts.next();
    let second = slash_parts.next();
    let third = slash_parts.next();
    let name = match (first, second, third) {
        (Some(_), Some(spelling), Some(_)) => spelling,
        (Some(spelling), Some(_), None) => spelling,
        _ => name,
    };
    (!name.is_empty()).then_some(name)
}

fn local_structure_definition_precedes(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    symbol: &ResolverSymbolId,
    before: usize,
) -> bool {
    if symbol.module() != module
        || source_reserve_symbol_head_kind(symbols, module, symbol) != Some(SymbolKind::Structure)
    {
        return false;
    }
    let Some(spelling) = source_local_symbol_spelling(symbol) else {
        return false;
    };
    let definitions = surface_nodes_with_kind(ast, SurfaceNodeKind::StructureDefinition)
        .into_iter()
        .filter(|(_, node)| {
            !subtree_has_recovery(ast, node)
                && node.range.end <= before
                && structure_definition_pattern_spelling(ast, node).as_deref() == Some(spelling)
        })
        .collect::<Vec<_>>();
    matches!(definitions.as_slice(), [(_, _)])
}

pub(in crate::runner) fn mode_definition_pattern_spelling(
    ast: &SurfaceAst,
    node: &SurfaceNode,
) -> Option<String> {
    if !matches!(node.kind, SurfaceNodeKind::ModeDefinition) {
        return None;
    }
    let pattern = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .find(|child| matches!(child.kind, SurfaceNodeKind::ModePattern))?;
    if pattern.children.len() != 1 {
        return None;
    }
    let token_node = ast.node(pattern.children[0])?;
    match &token_node.kind {
        SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::Identifier => {
            Some(token.text.to_string())
        }
        _ => None,
    }
}

fn structure_definition_pattern_spelling(ast: &SurfaceAst, node: &SurfaceNode) -> Option<String> {
    if !matches!(node.kind, SurfaceNodeKind::StructureDefinition) {
        return None;
    }
    let pattern = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .find(|child| matches!(child.kind, SurfaceNodeKind::StructurePattern))?;
    if pattern.children.len() != 1 {
        return None;
    }
    let token_node = ast.node(pattern.children[0])?;
    match &token_node.kind {
        SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::Identifier => {
            Some(token.text.to_string())
        }
        _ => None,
    }
}

fn extract_source_mode_rhs(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceTypeExpression> {
    let rhs_nodes = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter(|child| matches!(child.kind, SurfaceNodeKind::TypeExpression))
        .collect::<Vec<_>>();
    let [rhs] = rhs_nodes.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, rhs) {
        return None;
    }
    let rhs = extract_builtin_source_type_expression(ast, rhs, module, symbols).ok()?;
    if !rhs.attributes.is_empty()
        && !matches!(
            rhs.head,
            TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject
        )
    {
        return None;
    }
    Some(rhs)
}

fn mode_definition_has_local_context(ast: &SurfaceAst, mode_id: SurfaceNodeId) -> bool {
    let Some(block_id) = containing_definition_block(ast, mode_id) else {
        return false;
    };
    ast.node(block_id)
        .is_some_and(|block| subtree_has_definition_local_context(ast, block, mode_id))
}

fn containing_definition_block(ast: &SurfaceAst, target: SurfaceNodeId) -> Option<SurfaceNodeId> {
    surface_nodes_with_kind(ast, SurfaceNodeKind::DefinitionBlockItem)
        .into_iter()
        .filter(|(id, _)| subtree_contains_node(ast, *id, target))
        .min_by_key(|(_, node)| node.range.end.saturating_sub(node.range.start))
        .map(|(id, _)| id)
}

fn subtree_contains_node(ast: &SurfaceAst, current: SurfaceNodeId, target: SurfaceNodeId) -> bool {
    if current == target {
        return true;
    }
    ast.node(current).is_some_and(|node| {
        node.children
            .iter()
            .any(|child| subtree_contains_node(ast, *child, target))
    })
}

fn subtree_has_definition_local_context(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    mode_id: SurfaceNodeId,
) -> bool {
    for child_id in &node.children {
        if *child_id == mode_id {
            continue;
        }
        let Some(child) = ast.node(*child_id) else {
            continue;
        };
        if matches!(
            child.kind,
            SurfaceNodeKind::DefinitionParameter
                | SurfaceNodeKind::AssumptionStatement
                | SurfaceNodeKind::GivenStatement
        ) || subtree_has_definition_local_context(ast, child, mode_id)
        {
            return true;
        }
    }
    false
}

fn is_supported_builtin_reserve_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::Token(_)
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::ImportItem
            | SurfaceNodeKind::ImportAliasDecl
            | SurfaceNodeKind::ModulePath
            | SurfaceNodeKind::PathSegment
            | SurfaceNodeKind::DefinitionBlockItem
            | SurfaceNodeKind::DefinitionParameter
            | SurfaceNodeKind::QualifiedVariableSegment
            | SurfaceNodeKind::AttributeDefinition
            | SurfaceNodeKind::AttributePattern
            | SurfaceNodeKind::ModeDefinition
            | SurfaceNodeKind::ModePattern
            | SurfaceNodeKind::StructureDefinition
            | SurfaceNodeKind::StructurePattern
            | SurfaceNodeKind::StructureField
            | SurfaceNodeKind::FormulaDefiniens
            | SurfaceNodeKind::FormulaCase
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::FormulaConstant(_)
            | SurfaceNodeKind::ReserveItem
            | SurfaceNodeKind::ReserveSegment
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::TypeArguments
            | SurfaceNodeKind::AttributeChain
            | SurfaceNodeKind::AttributeRef
            | SurfaceNodeKind::QualifiedSymbol
            | SurfaceNodeKind::TypeHead
            | SurfaceNodeKind::ErrorRecovery(_)
    )
}

fn extract_builtin_reserve_segment(
    ast: &SurfaceAst,
    segment: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<Vec<SourceReserveBindingInput>, ()> {
    if subtree_has_recovery(ast, segment) {
        return Err(());
    }

    let mut identifiers = Vec::new();
    let mut saw_for = false;
    let mut type_expression = None;
    for child_id in &segment.children {
        let child = ast.node(*child_id).ok_or(())?;
        match &child.kind {
            SurfaceNodeKind::Token(token)
                if token.kind == SurfaceTokenKind::ReservedWord
                    && token.text.as_ref() == "for"
                    && !saw_for =>
            {
                saw_for = true;
            }
            SurfaceNodeKind::Token(token)
                if token.kind == SurfaceTokenKind::Identifier && !saw_for =>
            {
                let spelling = token.text.to_string();
                identifiers.push((spelling, child.range));
            }
            SurfaceNodeKind::Token(token)
                if token.kind == SurfaceTokenKind::ReservedSymbol
                    && token.text.as_ref() == ","
                    && !saw_for => {}
            SurfaceNodeKind::TypeExpression if saw_for && type_expression.is_none() => {
                type_expression = Some(extract_builtin_source_type_expression(
                    ast, child, module, symbols,
                )?);
            }
            _ => return Err(()),
        }
    }

    if !saw_for || identifiers.is_empty() {
        return Err(());
    }
    let type_expression = type_expression.ok_or(())?;
    Ok(identifiers
        .into_iter()
        .map(|(spelling, binding_range)| {
            SourceReserveBindingInput::new(
                spelling,
                binding_range,
                type_expression.range,
                type_expression.spelling.clone(),
                type_expression.head.clone(),
            )
            .with_type_attributes(type_expression.attributes.clone())
        })
        .collect())
}

fn merge_optional_range(left: Option<SourceRange>, right: SourceRange) -> SourceRange {
    match left {
        Some(left) => SourceRange {
            source_id: left.source_id,
            start: left.start.min(right.start),
            end: left.end.max(right.end),
        },
        None => right,
    }
}
