#[cfg(test)]
use mizar_checker::binding_env::BindingEnv;
use mizar_checker::{
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationInput, SourceTypeArgument,
        SourceTypeArgumentInput, SourceTypeExpressionId, SourceTypeExpressionInput,
        SourceTypeHandoffInput, SourceTypeHead, SourceTypeProducer,
    },
    type_checker::{SourceReserveBindingInput, SourceReserveDeclarationBridge, TypeHeadInput},
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedArenaBuilder, TypedAst,
        TypedAstParts, TypedNode, TypedNodeId, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    env::{SymbolEnv, SymbolKind},
    resolved_ast::{ModuleId, SemanticOrigin, SymbolId},
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_ast::{
        exact_compilation_item_list, qualified_symbol_spelling, structural_child_ids,
        subtree_has_recovery,
    },
};

const PENDING_KEY: &str =
    "type_elaboration.checker.source_type_application.semantic_dependencies_pending";
const INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.source_type_application.invalid_payload";

const EXPECTED_TOKENS: &[&str] = &[
    "import",
    "parser",
    ".",
    "type_fixtures",
    ";",
    "definition",
    "mode",
    "ElementDef",
    ":",
    "Element",
    "of",
    "p1",
    "is",
    "set",
    ";",
    "struct",
    "PositionalStruct",
    "of",
    "p2",
    "where",
    "field",
    "carrier1",
    "->",
    "set",
    ";",
    "end",
    ";",
    "mode",
    "MatrixDef",
    ":",
    "Matrix",
    "over",
    "p3",
    "is",
    "set",
    ";",
    "mode",
    "FamilyDef",
    ":",
    "Family",
    "[",
    "p4",
    "]",
    "is",
    "set",
    ";",
    "struct",
    "BracketStruct",
    "[",
    "p5",
    "]",
    "where",
    "field",
    "carrier2",
    "->",
    "set",
    ";",
    "end",
    ";",
    "end",
    ";",
    "reserve",
    "a",
    "for",
    "set",
    ";",
    "reserve",
    "o",
    "for",
    "object",
    ";",
    "reserve",
    "e",
    "for",
    "Element",
    "of",
    "a",
    ";",
    "reserve",
    "s",
    "for",
    "PositionalStruct",
    "of",
    "a",
    ";",
    "reserve",
    "v",
    "for",
    "Matrix",
    "over",
    "a",
    ";",
    "reserve",
    "f",
    "for",
    "Family",
    "[",
    "set",
    "]",
    ";",
    "reserve",
    "q",
    "for",
    "Family",
    "[",
    "a",
    "qua",
    "set",
    "]",
    ";",
    "reserve",
    "b",
    "for",
    "BracketStruct",
    "[",
    "set",
    "]",
    ";",
    "reserve",
    "m",
    "for",
    "TypeCaseMode",
    ";",
    "reserve",
    "t",
    "for",
    "TypeCaseStruct",
    ";",
];

#[derive(Debug)]
pub(in crate::runner) struct SourceTypeRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
    #[cfg(test)]
    pub(in crate::runner) binding_env: BindingEnv,
}

pub(in crate::runner) fn source_type_application_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    source_type_application_output(ast, module, symbols).map(|result| match result {
        Ok(output) => {
            debug_assert_eq!(
                output.typed_ast.source_type(),
                output.resolved.source_type()
            );
            vec![PENDING_KEY.to_owned()]
        }
        Err(_) => vec![INVALID_PAYLOAD_KEY.to_owned()],
    })
}

pub(in crate::runner) fn source_type_application_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceTypeRouteOutput, String>> {
    if ast.token_texts() != EXPECTED_TOKENS {
        return None;
    }
    Some(build_output(ast, module, symbols))
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceTypeRouteOutput, String> {
    if symbols.module_id() != &module {
        return Err("source-type symbol environment module mismatch".to_owned());
    }
    let item_list = exact_compilation_item_list(ast)
        .ok_or_else(|| "source-type compilation item list is not exact".to_owned())?;
    let item_ids = structural_child_ids(ast, item_list);
    if item_ids.len() != 12 {
        return Err("source-type fixture requires import, definition, and ten reserves".to_owned());
    }
    let reserve_ids = &item_ids[2..];
    if reserve_ids.iter().any(|id| {
        ast.node(*id).is_none_or(|node| {
            !matches!(node.kind, SurfaceNodeKind::ReserveItem) || subtree_has_recovery(ast, node)
        })
    }) {
        return Err("source-type fixture reserve items are not exact".to_owned());
    }

    let arena = typed_arena_from_surface(ast)?;
    let mut expressions = Vec::new();
    let mut arguments = Vec::new();
    let mut applications = Vec::new();
    let mut reserve_bindings = Vec::new();

    for (ordinal, reserve_id) in reserve_ids.iter().copied().enumerate() {
        let reserve = ast
            .node(reserve_id)
            .ok_or_else(|| format!("source-type reserve {ordinal} disappeared"))?;
        let segment_ids = structural_child_ids(ast, reserve);
        let [segment_id] = segment_ids.as_slice() else {
            return Err(format!(
                "source-type reserve {ordinal} requires one segment"
            ));
        };
        let segment = ast
            .node(*segment_id)
            .ok_or_else(|| format!("source-type reserve {ordinal} segment disappeared"))?;
        if !matches!(segment.kind, SurfaceNodeKind::ReserveSegment)
            || subtree_has_recovery(ast, segment)
        {
            return Err(format!(
                "source-type reserve {ordinal} segment is not exact"
            ));
        }
        let type_ids = structural_child_ids(ast, segment);
        let [type_id] = type_ids.as_slice() else {
            return Err(format!(
                "source-type reserve {ordinal} requires one type expression"
            ));
        };
        let type_node = ast
            .node(*type_id)
            .ok_or_else(|| format!("source-type reserve {ordinal} type disappeared"))?;
        let binding = reserve_binding(ast, segment, type_node, ordinal)?;
        let root = extract_expression(
            ast,
            *type_id,
            &module,
            symbols,
            &mut expressions,
            &mut arguments,
        )?;
        let head = legacy_head(&expressions[root.index()].head);
        reserve_bindings.push(SourceReserveBindingInput::new(
            binding.0,
            binding.1,
            type_node.range,
            source_text(ast, type_node)?,
            head,
        ));
        applications.push(SourceTypeApplicationInput {
            binding: mizar_checker::binding_env::BindingId::new(ordinal),
            source_ordinal: ordinal,
            root,
        });
    }

    let root_range = ast
        .root()
        .and_then(|id| ast.node(id))
        .map(|node| node.range)
        .ok_or_else(|| "source-type root disappeared".to_owned())?;
    let bridge = SourceReserveDeclarationBridge::new(
        ast.source_id,
        module.clone(),
        root_range,
        reserve_bindings,
    )?;
    let binding_env = bridge.prepare_binding_env(symbols)?;
    let handoff = SourceTypeProducer::build(
        SourceTypeHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            applications,
            expressions,
            arguments,
        },
        &binding_env,
        symbols,
        &arena,
    )
    .map_err(|error| error.to_string())?;

    let typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module,
        resolved_root: None,
        source_context: None,
        source_type: Some(handoff),
        source_attribute: None,
        nodes: arena,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.type.surface"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_type().is_none()
        || resolved.source_type() != typed_ast.source_type()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
        || !resolved.checked_formulas().is_empty()
        || !resolved.statement_semantics().is_empty()
        || !resolved.checked_proofs().is_empty()
    {
        return Err("source-type final handoff invariant failed".to_owned());
    }
    Ok(SourceTypeRouteOutput {
        typed_ast,
        resolved,
        #[cfg(test)]
        binding_env,
    })
}

fn reserve_binding(
    ast: &SurfaceAst,
    segment: &SurfaceNode,
    type_node: &SurfaceNode,
    ordinal: usize,
) -> Result<(String, SourceRange), String> {
    let mut tokens = segment
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .filter_map(|node| node.token_text().map(|text| (text, node.range)));
    let (spelling, range) = tokens
        .next()
        .ok_or_else(|| format!("source-type reserve {ordinal} has no binding token"))?;
    let (keyword, _) = tokens
        .next()
        .ok_or_else(|| format!("source-type reserve {ordinal} has no `for` token"))?;
    if keyword != "for"
        || tokens.next().is_some()
        || spelling.is_empty()
        || range.end > type_node.range.start
    {
        return Err(format!(
            "source-type reserve {ordinal} binding shape is not exact"
        ));
    }
    Ok((spelling.to_owned(), range))
}

fn extract_expression(
    ast: &SurfaceAst,
    expression_id: SurfaceNodeId,
    module: &ModuleId,
    symbols: &SymbolEnv,
    expressions: &mut Vec<SourceTypeExpressionInput>,
    arguments: &mut Vec<SourceTypeArgumentInput>,
) -> Result<SourceTypeExpressionId, String> {
    let expression = ast
        .node(expression_id)
        .ok_or_else(|| "source-type expression disappeared".to_owned())?;
    if !matches!(expression.kind, SurfaceNodeKind::TypeExpression)
        || subtree_has_recovery(ast, expression)
    {
        return Err("source-type expression is recovered or has the wrong kind".to_owned());
    }
    let children = structural_child_ids(ast, expression);
    let [head_id] = children.as_slice() else {
        return Err("source-type expression requires exactly one head".to_owned());
    };
    let head_node = ast
        .node(*head_id)
        .ok_or_else(|| "source-type head disappeared".to_owned())?;
    let parsed_head = extract_head(ast, head_node, module, symbols)?;
    let id = SourceTypeExpressionId::new(expressions.len());
    expressions.push(SourceTypeExpressionInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        site: TypedSiteRef::Node(TypedNodeId::new(expression_id.index())),
        source_range: expression.range,
        spelling: source_text(ast, expression)?,
        head_site: TypedSiteRef::Node(TypedNodeId::new(parsed_head.site.index())),
        head_range: parsed_head.range,
        head_spelling: parsed_head.spelling,
        form: parsed_head.form,
        head: parsed_head.head,
        recovery: NodeRecoveryState::Normal,
    });

    let Some(type_arguments_id) = parsed_head.arguments else {
        return Ok(id);
    };
    let type_arguments = ast
        .node(type_arguments_id)
        .ok_or_else(|| "source-type argument list disappeared".to_owned())?;
    for (ordinal, argument_id) in structural_child_ids(ast, type_arguments)
        .into_iter()
        .enumerate()
    {
        let argument = ast
            .node(argument_id)
            .ok_or_else(|| "source-type argument disappeared".to_owned())?;
        let kind = match parsed_head.form {
            SourceTypeApplicationForm::Of | SourceTypeApplicationForm::Over => {
                if !matches!(argument.kind, SurfaceNodeKind::TermExpression)
                    || subtree_has_recovery(ast, argument)
                {
                    return Err("source-type term argument is not exact".to_owned());
                }
                SourceTypeArgument::TermSite {
                    site: TypedSiteRef::Node(TypedNodeId::new(argument_id.index())),
                    source_range: argument.range,
                    spelling: source_text(ast, argument)?,
                    recovery: NodeRecoveryState::Normal,
                    provenance: semantic_origin(
                        ast.source_id,
                        module,
                        argument.range,
                        id,
                        ordinal,
                    )?,
                }
            }
            SourceTypeApplicationForm::Bracket
                if matches!(argument.kind, SurfaceNodeKind::TypeExpression) =>
            {
                let child =
                    extract_expression(ast, argument_id, module, symbols, expressions, arguments)?;
                SourceTypeArgument::TypeSite { expression: child }
            }
            SourceTypeApplicationForm::Bracket
                if matches!(argument.kind, SurfaceNodeKind::TermExpression) =>
            {
                extract_qua_argument(
                    ast,
                    argument,
                    id,
                    ordinal,
                    module,
                    symbols,
                    expressions,
                    arguments,
                )?
            }
            _ => return Err("source-type argument has the wrong parent form".to_owned()),
        };
        arguments.push(SourceTypeArgumentInput {
            parent: id,
            ordinal,
            argument: kind,
        });
    }
    Ok(id)
}

#[allow(clippy::too_many_arguments)] // Rationale: keep each syntax-owned qua projection input and output table explicit at this private boundary.
fn extract_qua_argument(
    ast: &SurfaceAst,
    term: &SurfaceNode,
    parent: SourceTypeExpressionId,
    ordinal: usize,
    module: &ModuleId,
    symbols: &SymbolEnv,
    expressions: &mut Vec<SourceTypeExpressionInput>,
    arguments: &mut Vec<SourceTypeArgumentInput>,
) -> Result<SourceTypeArgument, String> {
    let term_children = structural_child_ids(ast, term);
    let [qua_id] = term_children.as_slice() else {
        return Err("source-type bracket term is not one qua expression".to_owned());
    };
    let qua = ast
        .node(*qua_id)
        .ok_or_else(|| "source-type qua expression disappeared".to_owned())?;
    if !matches!(qua.kind, SurfaceNodeKind::QuaExpression) || subtree_has_recovery(ast, qua) {
        return Err("source-type bracket term is not an exact qua expression".to_owned());
    }
    let qua_children = structural_child_ids(ast, qua);
    let [identifier_id, radix_id] = qua_children.as_slice() else {
        return Err("source-type qua expression requires identifier and radix".to_owned());
    };
    let identifier = ast
        .node(*identifier_id)
        .ok_or_else(|| "source-type qua identifier disappeared".to_owned())?;
    if !matches!(identifier.kind, SurfaceNodeKind::TermReference) {
        return Err("source-type qua base is not an identifier reference".to_owned());
    }
    let radix = extract_expression(ast, *radix_id, module, symbols, expressions, arguments)?;
    Ok(SourceTypeArgument::QuaSite {
        site: TypedSiteRef::Node(TypedNodeId::new(identifier_id.index())),
        source_range: identifier.range,
        spelling: source_text(ast, identifier)?,
        recovery: NodeRecoveryState::Normal,
        provenance: semantic_origin(ast.source_id, module, identifier.range, parent, ordinal)?,
        radix: vec![radix],
    })
}

struct ParsedHead {
    site: SurfaceNodeId,
    range: SourceRange,
    spelling: String,
    form: SourceTypeApplicationForm,
    head: SourceTypeHead,
    arguments: Option<SurfaceNodeId>,
}

fn extract_head(
    ast: &SurfaceAst,
    head_node: &SurfaceNode,
    module: &ModuleId,
    symbols: &SymbolEnv,
) -> Result<ParsedHead, String> {
    if !matches!(head_node.kind, SurfaceNodeKind::TypeHead) || subtree_has_recovery(ast, head_node)
    {
        return Err("source-type head is recovered or has the wrong kind".to_owned());
    }
    let structural = structural_child_ids(ast, head_node);
    if structural.is_empty() {
        let [token_id] = head_node.children.as_slice() else {
            return Err("source-type builtin head requires one token".to_owned());
        };
        let token = ast
            .node(*token_id)
            .ok_or_else(|| "source-type builtin token disappeared".to_owned())?;
        let spelling = token
            .token_text()
            .ok_or_else(|| "source-type builtin head is not a token".to_owned())?;
        let head = match spelling {
            "set" => SourceTypeHead::BuiltinSet,
            "object" => SourceTypeHead::BuiltinObject,
            _ => return Err("source-type builtin head is unsupported".to_owned()),
        };
        return Ok(ParsedHead {
            site: *token_id,
            range: token.range,
            spelling: spelling.to_owned(),
            form: SourceTypeApplicationForm::Bare,
            head,
            arguments: None,
        });
    }
    let (symbol_id, arguments) = match structural.as_slice() {
        [symbol] => (*symbol, None),
        [symbol, arguments] => (*symbol, Some(*arguments)),
        _ => return Err("source-type symbol head has an unsupported shape".to_owned()),
    };
    let symbol_node = ast
        .node(symbol_id)
        .ok_or_else(|| "source-type symbol node disappeared".to_owned())?;
    let spelling = qualified_symbol_spelling(ast, symbol_node)
        .map_err(|()| "source-type symbol spelling is invalid".to_owned())?;
    let form = match arguments {
        None => SourceTypeApplicationForm::Bare,
        Some(arguments_id) => {
            let argument_node = ast
                .node(arguments_id)
                .ok_or_else(|| "source-type argument list disappeared".to_owned())?;
            match argument_node
                .children
                .iter()
                .filter_map(|id| ast.node(*id))
                .find_map(SurfaceNode::token_text)
            {
                Some("of") => SourceTypeApplicationForm::Of,
                Some("over") => SourceTypeApplicationForm::Over,
                Some("[") => SourceTypeApplicationForm::Bracket,
                _ => return Err("source-type argument form is unsupported".to_owned()),
            }
        }
    };
    let symbol = resolve_source_type_head(symbols, module, &spelling, form).ok_or_else(|| {
        let candidates = symbols
            .symbols()
            .iter()
            .filter(|entry| {
                matches!(
                    entry.kind(),
                    mizar_resolve::env::SymbolKind::Mode
                        | mizar_resolve::env::SymbolKind::Structure
                )
            })
            .map(|entry| {
                format!(
                    "{}:{}:{:?}",
                    entry.symbol().module().path().as_str(),
                    entry.primary_spelling(),
                    entry.kind()
                )
            })
            .collect::<Vec<_>>();
        format!("source-type symbol `{spelling}` is not uniquely visible: {candidates:?}")
    })?;
    let entry = symbols
        .symbols()
        .get(&symbol)
        .ok_or_else(|| format!("source-type symbol `{spelling}` disappeared"))?;
    Ok(ParsedHead {
        site: symbol_id,
        range: symbol_node.range,
        spelling,
        form,
        head: SourceTypeHead::Symbol {
            symbol,
            contribution: entry.contribution(),
        },
        arguments,
    })
}

fn resolve_source_type_head(
    symbols: &SymbolEnv,
    module: &ModuleId,
    spelling: &str,
    form: SourceTypeApplicationForm,
) -> Option<SymbolId> {
    let mut local = Vec::new();
    let mut imported = Vec::new();
    for entry in symbols.symbols().iter().filter(|entry| {
        matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure)
            && entry.namespace().as_str() == module.path().as_str()
            && source_type_signature_matches(entry.primary_spelling(), spelling, form)
    }) {
        if entry.symbol().module() == module {
            local.push(entry.symbol().clone());
        } else {
            imported.push(entry.symbol().clone());
        }
    }
    match local.as_slice() {
        [symbol] => Some(symbol.clone()),
        [] => match imported.as_slice() {
            [symbol] => Some(symbol.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn source_type_signature_matches(
    primary_spelling: &str,
    head_spelling: &str,
    form: SourceTypeApplicationForm,
) -> bool {
    match form {
        SourceTypeApplicationForm::Bare => primary_spelling == head_spelling,
        SourceTypeApplicationForm::Of => primary_spelling
            .strip_prefix(head_spelling)
            .is_some_and(|suffix| suffix.starts_with(" of ") && suffix.len() > 4),
        SourceTypeApplicationForm::Over => primary_spelling
            .strip_prefix(head_spelling)
            .is_some_and(|suffix| suffix.starts_with(" over ") && suffix.len() > 6),
        SourceTypeApplicationForm::Bracket => primary_spelling
            .strip_prefix(head_spelling)
            .is_some_and(|suffix| {
                suffix.starts_with(" [ ") && suffix.ends_with(" ]") && suffix.len() > 5
            }),
        _ => false,
    }
}

fn legacy_head(head: &SourceTypeHead) -> TypeHeadInput {
    match head {
        SourceTypeHead::BuiltinSet => TypeHeadInput::BuiltinSet,
        SourceTypeHead::BuiltinObject => TypeHeadInput::BuiltinObject,
        SourceTypeHead::Symbol { symbol, .. } => TypeHeadInput::Symbol(symbol.clone()),
        _ => unreachable!("Task249 source-type head is frozen to three variants"),
    }
}

fn semantic_origin(
    source_id: mizar_session::SourceId,
    module: &ModuleId,
    range: SourceRange,
    parent: SourceTypeExpressionId,
    ordinal: usize,
) -> Result<SemanticOrigin, String> {
    let parent = u32::try_from(parent.index())
        .map_err(|_| "source-type parent id exceeds provenance width".to_owned())?;
    let ordinal = u32::try_from(ordinal)
        .map_err(|_| "source-type argument ordinal exceeds provenance width".to_owned())?;
    Ok(SemanticOrigin::new(
        source_id,
        module.clone(),
        SourceAnchor::Range(range),
        vec![parent, ordinal],
    ))
}

fn typed_arena_from_surface(ast: &SurfaceAst) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    for (index, node) in ast.nodes().iter().enumerate() {
        if node.children.iter().any(|child| child.index() >= index) {
            return Err("source-type surface tree is not child-before-parent ordered".to_owned());
        }
        let recovery = if node.recovered {
            NodeRecoveryState::Recovered
        } else {
            NodeRecoveryState::Normal
        };
        let id = builder
            .push(
                TypedNode::new("source.type.surface", SourceAnchor::Range(node.range))
                    .with_children(
                        node.children
                            .iter()
                            .map(|child| TypedNodeId::new(child.index()))
                            .collect(),
                    )
                    .with_typing(TypingState::Unknown)
                    .with_recovery(recovery),
            )
            .map_err(|error| error.to_string())?;
        if id.index() != index {
            return Err("source-type typed arena lost dense source order".to_owned());
        }
    }
    let root = ast
        .root()
        .map(|id| TypedNodeId::new(id.index()))
        .ok_or_else(|| "source-type surface AST has no root".to_owned())?;
    builder
        .finish(Some(root))
        .map_err(|error| error.to_string())
}

fn source_text(ast: &SurfaceAst, node: &SurfaceNode) -> Result<String, String> {
    let mut tokens = Vec::new();
    collect_source_text(ast, node, &mut tokens)?;
    if tokens.is_empty() {
        return Err("source-type source spelling is empty".to_owned());
    }
    Ok(tokens.join(" "))
}

fn collect_source_text<'a>(
    ast: &'a SurfaceAst,
    node: &'a SurfaceNode,
    tokens: &mut Vec<&'a str>,
) -> Result<(), String> {
    if let Some(token) = node.token_text() {
        tokens.push(token);
        return Ok(());
    }
    for child in &node.children {
        let child = ast
            .node(*child)
            .ok_or_else(|| "source-type source child disappeared".to_owned())?;
        collect_source_text(ast, child, tokens)?;
    }
    Ok(())
}
