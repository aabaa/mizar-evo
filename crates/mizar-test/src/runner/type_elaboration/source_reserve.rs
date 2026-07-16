use mizar_checker::type_checker::{AttributeInput, AttributePolarity, TypeHeadInput};
use mizar_resolve::env::{ContributionKind, NamespacePath, SymbolEnv, SymbolKind};
use mizar_resolve::resolved_ast::ModuleId as ResolverModuleId;
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeKind, SurfaceTokenKind};

use super::source_ast::{qualified_symbol_spelling, subtree_has_recovery};

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

pub(in crate::runner) fn source_reserve_symbol_head_kind(
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

pub(in crate::runner) fn is_imported_fixture_reserve_attribute(
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

pub(in crate::runner) fn imported_fixture_reserve_attribute_spelling<'a>(
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
