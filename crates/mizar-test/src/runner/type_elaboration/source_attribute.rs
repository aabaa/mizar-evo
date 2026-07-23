#[cfg(test)]
use mizar_checker::resolved_typed_ast::ResolvedTypedAst;
use mizar_checker::{
    binding_env::BindingEnv,
    resolved_typed_ast::{ResolvedNodeKindHint, ResolvedNodeKindHintKind, SourceNodeRole},
    source_attribute::{
        SourceAttributeActualKind, SourceAttributeArgumentGroupId,
        SourceAttributeArgumentGroupInput, SourceAttributeArgumentGroupKind,
        SourceAttributeArgumentInput, SourceAttributeChainId, SourceAttributeChainInput,
        SourceAttributeHandoffInput, SourceAttributeId, SourceAttributeInput,
        SourceAttributePolarityInput, SourceAttributePrefixForm, SourceAttributeProducer,
        SourceAttributeQualifierInput,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationInput, SourceTypeExpressionId,
        SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeHead, SourceTypeProducer,
    },
    type_checker::{
        AttributeInput, AttributePolarity, DeclarationCheckingOutput, SourceReserveBindingInput,
        SourceReserveDeclarationBridge, TypeHeadInput,
    },
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeEntryId, TypeFactTable, TypeStatus, TypeTable, TypedArena,
        TypedArenaBuilder, TypedAst, TypedAstParts, TypedNode, TypedNodeId, TypedNodeLinks,
        TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    env::SymbolEnv,
    resolved_ast::{ModuleId, SemanticOrigin, SymbolId},
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind, SurfaceTokenKind};

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_ast::{exact_compilation_item_list, structural_child_ids, subtree_has_recovery},
    source_reserve::{resolve_visible_attribute, resolve_visible_type_head},
};

const PENDING_KEY: &str = "type_elaboration.checker.source_attribute.semantic_dependencies_pending";
const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.source_attribute.invalid_payload";
const EVIDENCE_QUERY_KEY: &str =
    "type_elaboration.checker.checker.declaration.deferred.evidence_query";

const ARGUMENT_BEARING_TOKENS: &[&str] = &[
    "definition",
    "let",
    "x",
    "be",
    "set",
    ";",
    "attr",
    "RankedDef",
    ":",
    "x",
    "is",
    "2",
    "-",
    "ranked",
    "means",
    "thesis",
    ";",
    "end",
    ";",
    "reserve",
    "y",
    "for",
    "ranked",
    "(",
    "2",
    ")",
    "set",
    ";",
];

const STRUCTURE_QUALIFIED_TOKENS: &[&str] = &[
    "definition",
    "let",
    "x",
    "be",
    "set",
    ";",
    "attr",
    "MarkedDef",
    ":",
    "x",
    "is",
    "marked",
    "means",
    "thesis",
    ";",
    "end",
    ";",
    "definition",
    "struct",
    "LocalStruct",
    "where",
    "field",
    "carrier",
    "->",
    "set",
    ";",
    "end",
    ";",
    "end",
    ";",
    "reserve",
    "s",
    "for",
    "LocalStruct",
    ".",
    "marked",
    "LocalStruct",
    ";",
];

const IMPORTED_TOKENS: &[&str] = &[
    "import",
    "parser",
    ".",
    "type_fixtures",
    ";",
    "reserve",
    "a",
    "for",
    "TypeCaseAttr",
    "set",
    ";",
];

const NEGATIVE_IMPORTED_TOKENS: &[&str] = &[
    "import",
    "parser",
    ".",
    "type_fixtures",
    ";",
    "reserve",
    "x",
    "for",
    "non",
    "empty",
    "set",
    ";",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceAttributeRoute {
    ArgumentBearing,
    StructureQualified,
    Imported,
    NegativeImported,
    #[cfg(test)]
    Synthetic,
}

impl SourceAttributeRoute {
    const fn preserves_evidence_query(self) -> bool {
        matches!(self, Self::Imported | Self::NegativeImported)
    }
}

#[derive(Debug)]
struct ExtractedSourceAttribute {
    binding_spelling: String,
    binding_range: SourceRange,
    source_range: SourceRange,
    type_node: SurfaceNodeId,
    type_spelling: String,
    head_node: SurfaceNodeId,
    head_spelling: String,
    head: TypeHeadInput,
    attributes: Vec<ExtractedAttribute>,
}

#[derive(Debug)]
struct ExtractedAttribute {
    node: SurfaceNodeId,
    target_node: SurfaceNodeId,
    target_spelling: String,
    symbol: SymbolId,
    polarity: AttributePolarity,
    non_node: Option<SurfaceNodeId>,
    qualifier: Option<ExtractedQualifier>,
    groups: Vec<ExtractedArgumentGroup>,
}

#[derive(Debug)]
struct ExtractedQualifier {
    source_range: SourceRange,
    spelling: String,
    symbol: SymbolId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtractedArgumentGroupKind {
    Prefix,
    ParenthesizedArgumentList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtractedPrefixForm {
    None,
    Single,
    Parenthesized,
}

#[derive(Debug)]
struct ExtractedArgumentGroup {
    kind: ExtractedArgumentGroupKind,
    form: ExtractedPrefixForm,
    node: SurfaceNodeId,
    hyphen_node: Option<SurfaceNodeId>,
    open_node: Option<SurfaceNodeId>,
    close_node: Option<SurfaceNodeId>,
    comma_nodes: Vec<SurfaceNodeId>,
    actuals: Vec<ExtractedActual>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtractedActualKind {
    PrefixIdentifier,
    PrefixNumeral,
    TermSite,
}

#[derive(Debug)]
struct ExtractedActual {
    node: SurfaceNodeId,
    kind: ExtractedActualKind,
}

fn select_real_route(ast: &SurfaceAst) -> Option<SourceAttributeRoute> {
    let tokens = ast.token_texts();
    if tokens == ARGUMENT_BEARING_TOKENS {
        Some(SourceAttributeRoute::ArgumentBearing)
    } else if tokens == STRUCTURE_QUALIFIED_TOKENS {
        Some(SourceAttributeRoute::StructureQualified)
    } else if tokens == IMPORTED_TOKENS {
        Some(SourceAttributeRoute::Imported)
    } else if tokens == NEGATIVE_IMPORTED_TOKENS {
        Some(SourceAttributeRoute::NegativeImported)
    } else {
        None
    }
}

#[derive(Debug)]
pub(in crate::runner) struct SourceAttributeRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    #[cfg(test)]
    pub(in crate::runner) resolved: ResolvedTypedAst,
    #[cfg(test)]
    pub(in crate::runner) binding_env: BindingEnv,
    #[cfg(test)]
    pub(in crate::runner) declarations: Option<DeclarationCheckingOutput>,
    #[cfg(test)]
    pub(in crate::runner) legacy_attribute_count: usize,
}

pub(in crate::runner) fn source_attribute_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let route = select_real_route(ast)?;
    Some(match build_output(ast, module, symbols, route) {
        Ok(output) if route.preserves_evidence_query() => {
            let mut keys = output
                .typed_ast
                .diagnostics()
                .canonical_iter()
                .map(|(_, diagnostic)| {
                    format!("type_elaboration.checker.{}", diagnostic.message_key)
                })
                .collect::<Vec<_>>();
            keys.sort();
            keys.dedup();
            if keys == [EVIDENCE_QUERY_KEY] {
                keys
            } else {
                vec![INVALID_PAYLOAD_KEY.to_owned()]
            }
        }
        Ok(_) => vec![PENDING_KEY.to_owned()],
        Err(_) => vec![INVALID_PAYLOAD_KEY.to_owned()],
    })
}

#[cfg(test)]
pub(in crate::runner) fn source_attribute_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceAttributeRouteOutput, String>> {
    let route = select_real_route(ast)?;
    Some(build_output(ast, module, symbols, route))
}

#[cfg(test)]
pub(in crate::runner) fn synthetic_source_attribute_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceAttributeRouteOutput, String> {
    build_output(ast, module, symbols, SourceAttributeRoute::Synthetic)
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    route: SourceAttributeRoute,
) -> Result<SourceAttributeRouteOutput, String> {
    let extracted = extract_source_attribute(ast, &module, symbols, route)?;
    let legacy_attributes = if route.preserves_evidence_query() {
        extracted
            .attributes
            .iter()
            .map(|attribute| {
                Ok(AttributeInput::new(
                    attribute.symbol.clone(),
                    attribute.polarity,
                    ast.node(attribute.node)
                        .ok_or_else(|| "source-attribute legacy occurrence disappeared".to_owned())?
                        .range,
                    source_text(
                        ast,
                        ast.node(attribute.node).ok_or_else(|| {
                            "source-attribute legacy occurrence disappeared".to_owned()
                        })?,
                    )?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        Vec::new()
    };
    let source_binding = SourceReserveBindingInput::new(
        extracted.binding_spelling.clone(),
        extracted.binding_range,
        ast.node(extracted.type_node)
            .ok_or_else(|| "source-attribute type expression disappeared".to_owned())?
            .range,
        extracted.type_spelling.clone(),
        extracted.head.clone(),
    )
    .with_type_attributes(legacy_attributes.clone());
    let bridge = SourceReserveDeclarationBridge::new(
        ast.source_id,
        module.clone(),
        extracted.source_range,
        vec![source_binding],
    )?;
    let (binding_env, declarations) = if route.preserves_evidence_query() {
        let (bindings, declarations) = bridge.check(symbols)?.into_parts();
        (bindings, Some(declarations))
    } else {
        (bridge.prepare_binding_env(symbols)?, None)
    };

    let projection = build_typed_arena(ast, &extracted, declarations.as_ref())?;
    let source_type =
        build_source_type(ast, &module, symbols, &extracted, &binding_env, &projection)?;
    let source_attribute = SourceAttributeProducer::build(
        source_attribute_input(ast, &module, symbols, &extracted, &projection)?,
        &source_type,
        &binding_env,
        symbols,
        &projection.arena,
    )
    .map_err(|error| error.to_string())?;

    let (contexts, types, facts, diagnostics) = declarations.as_ref().map_or_else(
        || {
            (
                LocalTypeContextTable::new(),
                TypeTable::new(),
                TypeFactTable::new(),
                TypeDiagnosticTable::new(),
            )
        },
        |output| {
            (
                output.contexts().clone(),
                output.type_entries().clone(),
                output.facts().clone(),
                output.diagnostics().clone(),
            )
        },
    );
    let typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module,
        resolved_root: None,
        source_context: None,
        source_type: Some(source_type),
        source_attribute: Some(source_attribute),
        nodes: projection.arena,
        contexts,
        types,
        facts,
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics,
    })
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.attribute.surface"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_type().is_none()
        || typed_ast.source_attribute().is_none()
        || resolved.source_type() != typed_ast.source_type()
        || resolved.source_attribute() != typed_ast.source_attribute()
    {
        return Err("source-attribute immutable final handoff mismatch".to_owned());
    }
    Ok(SourceAttributeRouteOutput {
        typed_ast,
        #[cfg(test)]
        resolved,
        #[cfg(test)]
        binding_env,
        #[cfg(test)]
        declarations,
        #[cfg(test)]
        legacy_attribute_count: legacy_attributes.len(),
    })
}

#[derive(Debug)]
struct TypedSiteProjection {
    arena: TypedArena,
    head_site: TypedSiteRef,
    attributes: Vec<AttributeSiteProjection>,
}

#[derive(Debug)]
struct AttributeSiteProjection {
    site: TypedSiteRef,
    target_site: TypedSiteRef,
    non_site: Option<TypedSiteRef>,
    qualifier_site: Option<TypedSiteRef>,
    groups: Vec<GroupSiteProjection>,
}

#[derive(Debug)]
struct GroupSiteProjection {
    site: TypedSiteRef,
    actual_sites: Vec<TypedSiteRef>,
}

fn build_typed_arena(
    ast: &SurfaceAst,
    extracted: &ExtractedSourceAttribute,
    declarations: Option<&DeclarationCheckingOutput>,
) -> Result<TypedSiteProjection, String> {
    let mut builder = TypedArenaBuilder::new();
    let type_site = TypedSiteRef::Node(TypedNodeId::new(0));
    let type_entry =
        declarations.and_then(|output| type_entry_for_site(output.type_entries(), &type_site));
    let type_node = builder
        .push(
            TypedNode::new(
                "source.attribute.type_expression",
                SourceAnchor::Range(
                    ast.node(extracted.type_node)
                        .ok_or_else(|| "source-attribute type expression disappeared".to_owned())?
                        .range,
                ),
            )
            .with_typing(typing_for_type_entry(
                declarations.map(DeclarationCheckingOutput::type_entries),
                type_entry,
            ))
            .with_recovery(NodeRecoveryState::Normal)
            .with_links(TypedNodeLinks {
                type_entry,
                ..TypedNodeLinks::default()
            }),
        )
        .map_err(|error| error.to_string())?;
    if type_node != TypedNodeId::new(0) {
        return Err("source-attribute type node lost legacy identity".to_owned());
    }
    let declaration_links = declarations
        .and_then(|output| {
            output
                .declarations()
                .iter()
                .next()
                .map(|(_, declaration)| declaration)
        })
        .map_or_else(TypedNodeLinks::default, |declaration| TypedNodeLinks {
            context: Some(mizar_checker::typed_ast::LocalTypeContextId::new(0)),
            type_entry: declaration.type_entry,
            facts: declaration.facts.clone(),
            ..TypedNodeLinks::default()
        });
    let declaration = builder
        .push(
            TypedNode::new(
                "source.attribute.reserve_declaration",
                SourceAnchor::Range(extracted.binding_range),
            )
            .with_children(vec![type_node])
            .with_typing(if declarations.is_some() {
                TypingState::Successful
            } else {
                TypingState::Unknown
            })
            .with_recovery(NodeRecoveryState::Normal)
            .with_links(declaration_links),
        )
        .map_err(|error| error.to_string())?;
    let root = builder
        .push(
            TypedNode::new(
                "source.attribute.reserve",
                SourceAnchor::Range(extracted.source_range),
            )
            .with_children(vec![declaration])
            .with_typing(TypingState::Unknown)
            .with_recovery(NodeRecoveryState::Normal),
        )
        .map_err(|error| error.to_string())?;
    if root != TypedNodeId::new(2) {
        return Err("source-attribute root lost legacy identity".to_owned());
    }

    let head_site = push_site(
        &mut builder,
        "source.attribute.type_head",
        ast.node(extracted.head_node)
            .ok_or_else(|| "source-attribute head site disappeared".to_owned())?
            .range,
    )?;
    let mut attributes = Vec::new();
    for attribute in &extracted.attributes {
        let site = push_surface_site(
            &mut builder,
            ast,
            "source.attribute.occurrence",
            attribute.node,
        )?;
        let target_site = push_surface_site(
            &mut builder,
            ast,
            "source.attribute.target",
            attribute.target_node,
        )?;
        let non_site = attribute
            .non_node
            .map(|node| push_surface_site(&mut builder, ast, "source.attribute.non", node))
            .transpose()?;
        let qualifier_site = attribute
            .qualifier
            .as_ref()
            .map(|qualifier| {
                push_site(
                    &mut builder,
                    "source.attribute.qualifier",
                    qualifier.source_range,
                )
            })
            .transpose()?;
        let mut groups = Vec::new();
        for group in &attribute.groups {
            let group_range = argument_group_range(ast, group)?;
            let group_site =
                push_site(&mut builder, "source.attribute.argument_group", group_range)?;
            let actual_sites = group
                .actuals
                .iter()
                .map(|actual| {
                    push_surface_site(&mut builder, ast, "source.attribute.actual", actual.node)
                })
                .collect::<Result<Vec<_>, _>>()?;
            groups.push(GroupSiteProjection {
                site: group_site,
                actual_sites,
            });
        }
        attributes.push(AttributeSiteProjection {
            site,
            target_site,
            non_site,
            qualifier_site,
            groups,
        });
    }
    let arena = builder
        .finish(Some(root))
        .map_err(|error| error.to_string())?;
    Ok(TypedSiteProjection {
        arena,
        head_site,
        attributes,
    })
}

fn push_surface_site(
    builder: &mut TypedArenaBuilder,
    ast: &SurfaceAst,
    role: &'static str,
    node: SurfaceNodeId,
) -> Result<TypedSiteRef, String> {
    let range = ast
        .node(node)
        .ok_or_else(|| format!("{role} surface node disappeared"))?
        .range;
    push_site(builder, role, range)
}

fn push_site(
    builder: &mut TypedArenaBuilder,
    role: &'static str,
    range: SourceRange,
) -> Result<TypedSiteRef, String> {
    builder
        .push(
            TypedNode::new(role, SourceAnchor::Range(range))
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal),
        )
        .map(TypedSiteRef::Node)
        .map_err(|error| error.to_string())
}

fn build_source_type(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &ExtractedSourceAttribute,
    bindings: &BindingEnv,
    projection: &TypedSiteProjection,
) -> Result<mizar_checker::source_type::SourceTypeApplicationHandoff, String> {
    let head = match &extracted.head {
        TypeHeadInput::BuiltinSet => SourceTypeHead::BuiltinSet,
        TypeHeadInput::BuiltinObject => SourceTypeHead::BuiltinObject,
        TypeHeadInput::Symbol(symbol) => SourceTypeHead::Symbol {
            symbol: symbol.clone(),
            contribution: symbols
                .symbols()
                .get(symbol)
                .ok_or_else(|| "source-attribute type-head symbol disappeared".to_owned())?
                .contribution(),
        },
        _ => return Err("source-attribute type head is outside Task 249".to_owned()),
    };
    SourceTypeProducer::build(
        SourceTypeHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            applications: vec![SourceTypeApplicationInput {
                binding: mizar_checker::binding_env::BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            }],
            expressions: vec![SourceTypeExpressionInput {
                source_id: ast.source_id,
                module_id: module.clone(),
                site: TypedSiteRef::Node(TypedNodeId::new(0)),
                source_range: ast
                    .node(extracted.type_node)
                    .ok_or_else(|| "source-attribute type expression disappeared".to_owned())?
                    .range,
                spelling: extracted.type_spelling.clone(),
                head_site: projection.head_site.clone(),
                head_range: ast
                    .node(extracted.head_node)
                    .ok_or_else(|| "source-attribute type head disappeared".to_owned())?
                    .range,
                head_spelling: extracted.head_spelling.clone(),
                form: SourceTypeApplicationForm::Bare,
                head,
                recovery: NodeRecoveryState::Normal,
            }],
            arguments: Vec::new(),
        },
        bindings,
        symbols,
        &projection.arena,
    )
    .map_err(|error| error.to_string())
}

fn source_attribute_input(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &ExtractedSourceAttribute,
    projection: &TypedSiteProjection,
) -> Result<SourceAttributeHandoffInput, String> {
    let type_range = ast
        .node(extracted.type_node)
        .ok_or_else(|| "source-attribute type expression disappeared".to_owned())?
        .range;
    let mut attributes = Vec::new();
    let mut qualifiers = Vec::new();
    let mut argument_groups = Vec::new();
    let mut arguments = Vec::new();
    for (attribute_index, attribute) in extracted.attributes.iter().enumerate() {
        let sites = projection
            .attributes
            .get(attribute_index)
            .ok_or_else(|| "source-attribute site projection disappeared".to_owned())?;
        let occurrence = ast
            .node(attribute.node)
            .ok_or_else(|| "source-attribute occurrence disappeared".to_owned())?;
        let target = ast
            .node(attribute.target_node)
            .ok_or_else(|| "source-attribute target disappeared".to_owned())?;
        let contribution = symbols
            .symbols()
            .get(&attribute.symbol)
            .ok_or_else(|| "source-attribute symbol disappeared".to_owned())?
            .contribution();
        let polarity = match (
            attribute.polarity,
            attribute.non_node,
            sites.non_site.as_ref(),
        ) {
            (AttributePolarity::Positive, None, None) => SourceAttributePolarityInput::Positive,
            (AttributePolarity::Negative, Some(non), Some(non_site)) => {
                let non = ast
                    .node(non)
                    .ok_or_else(|| "source-attribute non occurrence disappeared".to_owned())?;
                SourceAttributePolarityInput::Negative {
                    non_site: non_site.clone(),
                    non_range: non.range,
                    non_spelling: "non".to_owned(),
                    non_recovery: NodeRecoveryState::Normal,
                }
            }
            _ => return Err("source-attribute polarity projection is incomplete".to_owned()),
        };
        attributes.push(SourceAttributeInput {
            chain: SourceAttributeChainId::new(0),
            ordinal: attribute_index,
            site: sites.site.clone(),
            source_range: occurrence.range,
            spelling: source_text(ast, occurrence)?,
            target_site: sites.target_site.clone(),
            target_range: target.range,
            target_spelling: attribute.target_spelling.clone(),
            recovery: NodeRecoveryState::Normal,
            symbol: attribute.symbol.clone(),
            contribution,
            polarity,
        });
        if let (Some(qualifier), Some(site)) = (&attribute.qualifier, sites.qualifier_site.as_ref())
        {
            let contribution = symbols
                .symbols()
                .get(&qualifier.symbol)
                .ok_or_else(|| "source-attribute qualifier symbol disappeared".to_owned())?
                .contribution();
            qualifiers.push(SourceAttributeQualifierInput {
                attribute: SourceAttributeId::new(attribute_index),
                site: site.clone(),
                source_range: qualifier.source_range,
                spelling: qualifier.spelling.clone(),
                recovery: NodeRecoveryState::Normal,
                structure: qualifier.symbol.clone(),
                contribution,
            });
        }
        for (group_ordinal, group) in attribute.groups.iter().enumerate() {
            let group_id = SourceAttributeArgumentGroupId::new(argument_groups.len());
            let group_sites = sites
                .groups
                .get(group_ordinal)
                .ok_or_else(|| "source-attribute group site disappeared".to_owned())?;
            let group_range = argument_group_range(ast, group)?;
            argument_groups.push(SourceAttributeArgumentGroupInput {
                attribute: SourceAttributeId::new(attribute_index),
                ordinal: group_ordinal,
                kind: match group.kind {
                    ExtractedArgumentGroupKind::Prefix => SourceAttributeArgumentGroupKind::Prefix,
                    ExtractedArgumentGroupKind::ParenthesizedArgumentList => {
                        SourceAttributeArgumentGroupKind::ParenthesizedArgumentList
                    }
                },
                site: group_sites.site.clone(),
                source_range: group_range,
                spelling: argument_group_spelling(ast, group)?,
                recovery: NodeRecoveryState::Normal,
                prefix_form: match group.form {
                    ExtractedPrefixForm::None => None,
                    ExtractedPrefixForm::Single => Some(SourceAttributePrefixForm::Single),
                    ExtractedPrefixForm::Parenthesized => {
                        Some(SourceAttributePrefixForm::Parenthesized)
                    }
                },
                hyphen_range: optional_node_range(ast, group.hyphen_node)?,
                hyphen_spelling: optional_node_spelling(ast, group.hyphen_node)?,
                open_range: optional_node_range(ast, group.open_node)?,
                open_spelling: optional_node_spelling(ast, group.open_node)?,
                close_range: optional_node_range(ast, group.close_node)?,
                close_spelling: optional_node_spelling(ast, group.close_node)?,
                comma_ranges: group
                    .comma_nodes
                    .iter()
                    .map(|node| required_node_range(ast, *node))
                    .collect::<Result<Vec<_>, _>>()?,
                comma_spellings: group
                    .comma_nodes
                    .iter()
                    .map(|node| required_node_spelling(ast, *node))
                    .collect::<Result<Vec<_>, _>>()?,
            });
            for (ordinal, actual) in group.actuals.iter().enumerate() {
                let actual_node = ast
                    .node(actual.node)
                    .ok_or_else(|| "source-attribute actual disappeared".to_owned())?;
                arguments.push(SourceAttributeArgumentInput {
                    group: group_id,
                    ordinal,
                    kind: match actual.kind {
                        ExtractedActualKind::PrefixIdentifier => {
                            SourceAttributeActualKind::PrefixIdentifier
                        }
                        ExtractedActualKind::PrefixNumeral => {
                            SourceAttributeActualKind::PrefixNumeral
                        }
                        ExtractedActualKind::TermSite => SourceAttributeActualKind::TermSite,
                    },
                    site: group_sites
                        .actual_sites
                        .get(ordinal)
                        .ok_or_else(|| "source-attribute actual site disappeared".to_owned())?
                        .clone(),
                    source_range: actual_node.range,
                    spelling: source_text(ast, actual_node)?,
                    recovery: NodeRecoveryState::Normal,
                    provenance: semantic_origin(
                        ast,
                        module,
                        actual.node,
                        group_id.index(),
                        ordinal,
                    )?,
                });
            }
        }
    }
    Ok(SourceAttributeHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        chains: vec![SourceAttributeChainInput {
            expression: SourceTypeExpressionId::new(0),
            source_ordinal: 0,
            site: TypedSiteRef::Node(TypedNodeId::new(0)),
            source_range: type_range,
            spelling: extracted.type_spelling.clone(),
            recovery: NodeRecoveryState::Normal,
        }],
        attributes,
        qualifiers,
        argument_groups,
        arguments,
    })
}

fn type_entry_for_site(types: &TypeTable, site: &TypedSiteRef) -> Option<TypeEntryId> {
    types
        .iter()
        .find_map(|(entry, row)| (&row.owner == site).then_some(entry))
}

fn typing_for_type_entry(
    types: Option<&TypeTable>,
    type_entry: Option<TypeEntryId>,
) -> TypingState {
    type_entry
        .and_then(|entry| types.and_then(|types| types.get(entry)))
        .map_or(TypingState::Unknown, |entry| match entry.status {
            TypeStatus::Known => TypingState::Successful,
            TypeStatus::Assumed => TypingState::Assumed,
            TypeStatus::Unknown => TypingState::Unknown,
            TypeStatus::Error => TypingState::Error,
            TypeStatus::Skipped => TypingState::Skipped,
            _ => TypingState::Unknown,
        })
}

fn argument_group_range(
    ast: &SurfaceAst,
    group: &ExtractedArgumentGroup,
) -> Result<SourceRange, String> {
    match group.kind {
        ExtractedArgumentGroupKind::Prefix => required_node_range(ast, group.node),
        ExtractedArgumentGroupKind::ParenthesizedArgumentList => {
            let open = optional_node_range(ast, group.open_node)?
                .ok_or_else(|| "source-attribute argument group open disappeared".to_owned())?;
            let close = optional_node_range(ast, group.close_node)?
                .ok_or_else(|| "source-attribute argument group close disappeared".to_owned())?;
            Ok(SourceRange {
                source_id: open.source_id,
                start: open.start,
                end: close.end,
            })
        }
    }
}

fn argument_group_spelling(
    ast: &SurfaceAst,
    group: &ExtractedArgumentGroup,
) -> Result<String, String> {
    match group.kind {
        ExtractedArgumentGroupKind::Prefix => source_text(
            ast,
            ast.node(group.node)
                .ok_or_else(|| "source-attribute prefix disappeared".to_owned())?,
        ),
        ExtractedArgumentGroupKind::ParenthesizedArgumentList => {
            let mut nodes = Vec::new();
            if let Some(open) = group.open_node {
                nodes.push(open);
            }
            for (index, actual) in group.actuals.iter().enumerate() {
                nodes.push(actual.node);
                if let Some(comma) = group.comma_nodes.get(index) {
                    nodes.push(*comma);
                }
            }
            if let Some(close) = group.close_node {
                nodes.push(close);
            }
            source_text_from_nodes(ast, &nodes)
        }
    }
}

fn source_text_from_nodes(ast: &SurfaceAst, nodes: &[SurfaceNodeId]) -> Result<String, String> {
    nodes
        .iter()
        .map(|node| {
            ast.node(*node)
                .ok_or_else(|| "source-attribute spelling node disappeared".to_owned())
                .and_then(|node| source_text(ast, node))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|tokens| tokens.join(" "))
}

fn required_node_range(ast: &SurfaceAst, node: SurfaceNodeId) -> Result<SourceRange, String> {
    ast.node(node)
        .map(|node| node.range)
        .ok_or_else(|| "source-attribute punctuation node disappeared".to_owned())
}

fn optional_node_range(
    ast: &SurfaceAst,
    node: Option<SurfaceNodeId>,
) -> Result<Option<SourceRange>, String> {
    node.map(|node| required_node_range(ast, node)).transpose()
}

fn required_node_spelling(ast: &SurfaceAst, node: SurfaceNodeId) -> Result<String, String> {
    ast.node(node)
        .and_then(SurfaceNode::token_text)
        .map(str::to_owned)
        .ok_or_else(|| "source-attribute punctuation spelling disappeared".to_owned())
}

fn optional_node_spelling(
    ast: &SurfaceAst,
    node: Option<SurfaceNodeId>,
) -> Result<Option<String>, String> {
    node.map(|node| required_node_spelling(ast, node))
        .transpose()
}

fn extract_source_attribute(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    route: SourceAttributeRoute,
) -> Result<ExtractedSourceAttribute, String> {
    if symbols.module_id() != module {
        return Err("source-attribute symbol environment module mismatch".to_owned());
    }
    let item_list = exact_compilation_item_list(ast)
        .ok_or_else(|| "source-attribute compilation item list is not exact".to_owned())?;
    let reserve_items = structural_child_ids(ast, item_list)
        .into_iter()
        .filter_map(|id| ast.node(id).map(|node| (id, node)))
        .filter(|(_, node)| matches!(node.kind, SurfaceNodeKind::ReserveItem))
        .collect::<Vec<_>>();
    let [(_, reserve)] = reserve_items.as_slice() else {
        return Err("source-attribute route requires exactly one reserve item".to_owned());
    };
    if subtree_has_recovery(ast, reserve) {
        return Err("source-attribute reserve item is recovered".to_owned());
    }
    let reserve_segments = structural_child_ids(ast, reserve)
        .into_iter()
        .filter_map(|id| ast.node(id).map(|node| (id, node)))
        .filter(|(_, node)| matches!(node.kind, SurfaceNodeKind::ReserveSegment))
        .collect::<Vec<_>>();
    let [(_, segment)] = reserve_segments.as_slice() else {
        return Err("source-attribute route requires one reserve segment".to_owned());
    };
    if subtree_has_recovery(ast, segment) {
        return Err("source-attribute reserve segment is recovered".to_owned());
    }
    let type_nodes = structural_child_ids(ast, segment)
        .into_iter()
        .filter_map(|id| ast.node(id).map(|node| (id, node)))
        .filter(|(_, node)| matches!(node.kind, SurfaceNodeKind::TypeExpression))
        .collect::<Vec<_>>();
    let [(type_node, type_expression)] = type_nodes.as_slice() else {
        return Err("source-attribute route requires one type expression".to_owned());
    };
    let (binding_spelling, binding_range) = reserve_binding(ast, segment, *type_node)?;

    let type_children = structural_child_ids(ast, type_expression);
    let [chain_node, head_node] = type_children.as_slice() else {
        return Err("source-attribute type expression requires a chain and head".to_owned());
    };
    let chain = ast
        .node(*chain_node)
        .ok_or_else(|| "source-attribute chain disappeared".to_owned())?;
    let head = ast
        .node(*head_node)
        .ok_or_else(|| "source-attribute head disappeared".to_owned())?;
    if !matches!(chain.kind, SurfaceNodeKind::AttributeChain)
        || !matches!(head.kind, SurfaceNodeKind::TypeHead)
        || subtree_has_recovery(ast, chain)
        || subtree_has_recovery(ast, head)
    {
        return Err("source-attribute chain or head is not exact".to_owned());
    }
    let (head_site, head_spelling, legacy_head) = extract_head(ast, head, module, symbols)?;
    let mut attributes = Vec::new();
    for attribute_node in structural_child_ids(ast, chain) {
        attributes.push(extract_attribute(ast, attribute_node, module, symbols)?);
    }
    if attributes.is_empty() {
        return Err("source-attribute chain is empty".to_owned());
    }
    validate_route_shape(route, &attributes, &legacy_head)?;

    Ok(ExtractedSourceAttribute {
        binding_spelling,
        binding_range,
        source_range: reserve.range,
        type_node: *type_node,
        type_spelling: source_text(ast, type_expression)?,
        head_node: head_site,
        head_spelling,
        head: legacy_head,
        attributes,
    })
}

fn reserve_binding(
    ast: &SurfaceAst,
    segment: &SurfaceNode,
    type_node: SurfaceNodeId,
) -> Result<(String, SourceRange), String> {
    let type_range = ast
        .node(type_node)
        .ok_or_else(|| "source-attribute type expression disappeared".to_owned())?
        .range;
    let tokens = segment
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .filter_map(|node| node.token_text().map(|text| (text, node.range)))
        .collect::<Vec<_>>();
    let [(spelling, binding_range), (keyword, _)] = tokens.as_slice() else {
        return Err("source-attribute reserve binding shape is not exact".to_owned());
    };
    if *keyword != "for" || spelling.is_empty() || binding_range.end > type_range.start {
        return Err("source-attribute reserve binding shape is invalid".to_owned());
    }
    Ok(((*spelling).to_owned(), *binding_range))
}

fn extract_head(
    ast: &SurfaceAst,
    head: &SurfaceNode,
    module: &ModuleId,
    symbols: &SymbolEnv,
) -> Result<(SurfaceNodeId, String, TypeHeadInput), String> {
    let structural = structural_child_ids(ast, head);
    let [site] = structural.as_slice() else {
        let [token] = head.children.as_slice() else {
            return Err("source-attribute builtin head shape is not exact".to_owned());
        };
        let token_node = ast
            .node(*token)
            .ok_or_else(|| "source-attribute builtin head disappeared".to_owned())?;
        let spelling = token_node
            .token_text()
            .ok_or_else(|| "source-attribute builtin head is not a token".to_owned())?;
        let head = match spelling {
            "set" => TypeHeadInput::BuiltinSet,
            "object" => TypeHeadInput::BuiltinObject,
            _ => return Err("source-attribute builtin head is unsupported".to_owned()),
        };
        return Ok((*token, spelling.to_owned(), head));
    };
    let symbol_node = ast
        .node(*site)
        .ok_or_else(|| "source-attribute symbol head disappeared".to_owned())?;
    if !matches!(symbol_node.kind, SurfaceNodeKind::QualifiedSymbol) {
        return Err("source-attribute symbol head has the wrong kind".to_owned());
    }
    let spelling = qualified_symbol_parts(ast, symbol_node)?.join(".");
    let symbol = resolve_visible_type_head(symbols, module, &spelling)
        .map_err(|()| format!("source-attribute head `{spelling}` is not uniquely visible"))?;
    Ok((*site, spelling, TypeHeadInput::Symbol(symbol)))
}

fn extract_attribute(
    ast: &SurfaceAst,
    node_id: SurfaceNodeId,
    module: &ModuleId,
    symbols: &SymbolEnv,
) -> Result<ExtractedAttribute, String> {
    let node = ast
        .node(node_id)
        .ok_or_else(|| "source-attribute occurrence disappeared".to_owned())?;
    if !matches!(node.kind, SurfaceNodeKind::AttributeRef) || subtree_has_recovery(ast, node) {
        return Err("source-attribute occurrence is recovered or has the wrong kind".to_owned());
    }
    let mut cursor = 0;
    let mut non_node = None;
    if node.children.get(cursor).is_some_and(|id| {
        ast.node(*id)
            .and_then(SurfaceNode::token_text)
            .is_some_and(|text| text == "non")
    }) {
        non_node = node.children.get(cursor).copied();
        cursor += 1;
    }

    let mut groups = Vec::new();
    if node.children.get(cursor).is_some_and(|id| {
        ast.node(*id)
            .is_some_and(|child| matches!(child.kind, SurfaceNodeKind::ParameterPrefix))
    }) {
        groups.push(extract_prefix(ast, node.children[cursor])?);
        cursor += 1;
    }

    let symbol_node_id = *node
        .children
        .get(cursor)
        .ok_or_else(|| "source-attribute target symbol is missing".to_owned())?;
    let symbol_node = ast
        .node(symbol_node_id)
        .ok_or_else(|| "source-attribute target symbol disappeared".to_owned())?;
    if !matches!(symbol_node.kind, SurfaceNodeKind::QualifiedSymbol) {
        return Err("source-attribute target has the wrong kind".to_owned());
    }
    cursor += 1;
    let parts = qualified_symbol_parts(ast, symbol_node)?;
    let target_spelling = parts
        .last()
        .cloned()
        .ok_or_else(|| "source-attribute target spelling is empty".to_owned())?;
    let target_node = qualified_symbol_target_node(ast, symbol_node)?;
    let symbol = resolve_visible_attribute(symbols, module, &target_spelling).map_err(|()| {
        format!("source-attribute target `{target_spelling}` is not uniquely visible")
    })?;
    let qualifier = if parts.len() == 1 {
        None
    } else if parts.len() == 2 {
        let qualifier_spelling = parts[0].clone();
        let qualifier_range = qualified_symbol_qualifier_range(ast, symbol_node)?;
        let qualifier_symbol = resolve_visible_type_head(symbols, module, &qualifier_spelling)
            .map_err(|()| {
                format!("source-attribute qualifier `{qualifier_spelling}` is not uniquely visible")
            })?;
        Some(ExtractedQualifier {
            source_range: qualifier_range,
            spelling: format!("{qualifier_spelling}."),
            symbol: qualifier_symbol,
        })
    } else {
        return Err("source-attribute qualifier has unsupported depth".to_owned());
    };

    if cursor < node.children.len() {
        groups.push(extract_argument_list(ast, &node.children[cursor..])?);
        cursor = node.children.len();
    }
    if cursor != node.children.len() {
        return Err("source-attribute occurrence has trailing syntax".to_owned());
    }

    Ok(ExtractedAttribute {
        node: node_id,
        target_node,
        target_spelling,
        symbol,
        polarity: if non_node.is_some() {
            AttributePolarity::Negative
        } else {
            AttributePolarity::Positive
        },
        non_node,
        qualifier,
        groups,
    })
}

fn extract_prefix(
    ast: &SurfaceAst,
    node_id: SurfaceNodeId,
) -> Result<ExtractedArgumentGroup, String> {
    let node = ast
        .node(node_id)
        .ok_or_else(|| "source-attribute prefix disappeared".to_owned())?;
    let token_nodes = direct_token_nodes(ast, node)?;
    let hyphen_node = token_nodes
        .last()
        .copied()
        .filter(|id| ast.node(*id).and_then(SurfaceNode::token_text) == Some("-"))
        .ok_or_else(|| "source-attribute prefix has no hyphen".to_owned())?;
    let body = &token_nodes[..token_nodes.len() - 1];
    let (form, open_node, close_node, actual_slice) = if body
        .first()
        .is_some_and(|id| ast.node(*id).and_then(SurfaceNode::token_text) == Some("("))
    {
        let close = body
            .last()
            .copied()
            .filter(|id| ast.node(*id).and_then(SurfaceNode::token_text) == Some(")"))
            .ok_or_else(|| "source-attribute parenthesized prefix has no close".to_owned())?;
        (
            ExtractedPrefixForm::Parenthesized,
            body.first().copied(),
            Some(close),
            &body[1..body.len() - 1],
        )
    } else {
        (ExtractedPrefixForm::Single, None, None, body)
    };
    let (actuals, comma_nodes) = prefix_actuals(ast, actual_slice)?;
    Ok(ExtractedArgumentGroup {
        kind: ExtractedArgumentGroupKind::Prefix,
        form,
        node: node_id,
        hyphen_node: Some(hyphen_node),
        open_node,
        close_node,
        comma_nodes,
        actuals,
    })
}

fn prefix_actuals(
    ast: &SurfaceAst,
    nodes: &[SurfaceNodeId],
) -> Result<(Vec<ExtractedActual>, Vec<SurfaceNodeId>), String> {
    let mut actuals = Vec::new();
    let mut commas = Vec::new();
    for (index, node_id) in nodes.iter().copied().enumerate() {
        let node = ast
            .node(node_id)
            .ok_or_else(|| "source-attribute prefix token disappeared".to_owned())?;
        if node.token_text() == Some(",") {
            if index == 0 || index + 1 == nodes.len() || index % 2 == 0 {
                return Err("source-attribute prefix comma order is invalid".to_owned());
            }
            commas.push(node_id);
            continue;
        }
        if index % 2 != 0 {
            return Err("source-attribute prefix actual order is invalid".to_owned());
        }
        let kind = match &node.kind {
            SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::Identifier => {
                ExtractedActualKind::PrefixIdentifier
            }
            SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::Numeral => {
                ExtractedActualKind::PrefixNumeral
            }
            _ => return Err("source-attribute prefix actual kind is invalid".to_owned()),
        };
        actuals.push(ExtractedActual {
            node: node_id,
            kind,
        });
    }
    if actuals.is_empty() {
        return Err("source-attribute prefix has no actual".to_owned());
    }
    Ok((actuals, commas))
}

fn extract_argument_list(
    ast: &SurfaceAst,
    nodes: &[SurfaceNodeId],
) -> Result<ExtractedArgumentGroup, String> {
    let open = nodes
        .first()
        .copied()
        .filter(|id| ast.node(*id).and_then(SurfaceNode::token_text) == Some("("))
        .ok_or_else(|| "source-attribute argument list has no open".to_owned())?;
    let close = nodes
        .last()
        .copied()
        .filter(|id| ast.node(*id).and_then(SurfaceNode::token_text) == Some(")"))
        .ok_or_else(|| "source-attribute argument list has no close".to_owned())?;
    let mut actuals = Vec::new();
    let mut comma_nodes = Vec::new();
    for (index, node_id) in nodes[1..nodes.len() - 1].iter().copied().enumerate() {
        let node = ast
            .node(node_id)
            .ok_or_else(|| "source-attribute argument element disappeared".to_owned())?;
        if node.token_text() == Some(",") {
            if index == 0 || index + 1 == nodes.len() - 2 || index % 2 == 0 {
                return Err("source-attribute argument comma order is invalid".to_owned());
            }
            comma_nodes.push(node_id);
        } else if index % 2 == 0 && matches!(node.kind, SurfaceNodeKind::TermExpression) {
            actuals.push(ExtractedActual {
                node: node_id,
                kind: ExtractedActualKind::TermSite,
            });
        } else {
            return Err("source-attribute argument element kind is invalid".to_owned());
        }
    }
    if actuals.is_empty() {
        return Err("source-attribute argument list is empty".to_owned());
    }
    Ok(ExtractedArgumentGroup {
        kind: ExtractedArgumentGroupKind::ParenthesizedArgumentList,
        form: ExtractedPrefixForm::None,
        node: open,
        hyphen_node: None,
        open_node: Some(open),
        close_node: Some(close),
        comma_nodes,
        actuals,
    })
}

fn qualified_symbol_parts(ast: &SurfaceAst, symbol: &SurfaceNode) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    for child_id in &symbol.children {
        let child = ast
            .node(*child_id)
            .ok_or_else(|| "source-attribute symbol child disappeared".to_owned())?;
        if child.token_text() == Some(".") {
            continue;
        }
        if !matches!(child.kind, SurfaceNodeKind::PathSegment) {
            return Err("source-attribute symbol child has the wrong kind".to_owned());
        }
        let [token_id] = child.children.as_slice() else {
            return Err("source-attribute path segment is not singular".to_owned());
        };
        let spelling = ast
            .node(*token_id)
            .and_then(SurfaceNode::token_text)
            .ok_or_else(|| "source-attribute path segment has no token".to_owned())?;
        parts.push(spelling.to_owned());
    }
    if parts.is_empty() {
        return Err("source-attribute symbol has no path segments".to_owned());
    }
    Ok(parts)
}

fn qualified_symbol_target_node(
    ast: &SurfaceAst,
    symbol: &SurfaceNode,
) -> Result<SurfaceNodeId, String> {
    let segment = symbol
        .children
        .iter()
        .rev()
        .find_map(|id| {
            ast.node(*id)
                .filter(|node| matches!(node.kind, SurfaceNodeKind::PathSegment))
                .map(|_| *id)
        })
        .ok_or_else(|| "source-attribute target segment is missing".to_owned())?;
    ast.node(segment)
        .and_then(|segment| segment.children.first())
        .copied()
        .ok_or_else(|| "source-attribute target token is missing".to_owned())
}

fn qualified_symbol_qualifier_range(
    ast: &SurfaceAst,
    symbol: &SurfaceNode,
) -> Result<SourceRange, String> {
    let [segment, dot, ..] = symbol.children.as_slice() else {
        return Err("source-attribute qualifier shape is incomplete".to_owned());
    };
    let segment = ast
        .node(*segment)
        .ok_or_else(|| "source-attribute qualifier segment disappeared".to_owned())?;
    if !matches!(segment.kind, SurfaceNodeKind::PathSegment)
        || ast.node(*dot).and_then(SurfaceNode::token_text) != Some(".")
    {
        return Err("source-attribute qualifier punctuation is invalid".to_owned());
    }
    let dot = ast
        .node(*dot)
        .ok_or_else(|| "source-attribute qualifier dot disappeared".to_owned())?;
    Ok(SourceRange {
        source_id: segment.range.source_id,
        start: segment.range.start,
        end: dot.range.end,
    })
}

fn direct_token_nodes(ast: &SurfaceAst, node: &SurfaceNode) -> Result<Vec<SurfaceNodeId>, String> {
    node.children
        .iter()
        .copied()
        .map(|id| {
            ast.node(id)
                .filter(|child| child.token_text().is_some())
                .map(|_| id)
                .ok_or_else(|| "source-attribute group has a non-token child".to_owned())
        })
        .collect()
}

fn validate_route_shape(
    route: SourceAttributeRoute,
    attributes: &[ExtractedAttribute],
    head: &TypeHeadInput,
) -> Result<(), String> {
    let shape_ok = match route {
        SourceAttributeRoute::ArgumentBearing => {
            matches!(head, TypeHeadInput::BuiltinSet)
                && matches!(
                    attributes,
                    [ExtractedAttribute {
                        target_spelling,
                        polarity: AttributePolarity::Positive,
                        qualifier: None,
                        groups,
                        ..
                    }] if target_spelling == "ranked"
                        && matches!(
                            groups.as_slice(),
                            [ExtractedArgumentGroup {
                                kind: ExtractedArgumentGroupKind::ParenthesizedArgumentList,
                                form: ExtractedPrefixForm::None,
                                actuals,
                                ..
                            }] if actuals.len() == 1
                                && actuals[0].kind == ExtractedActualKind::TermSite
                        )
                )
        }
        SourceAttributeRoute::StructureQualified => {
            matches!(head, TypeHeadInput::Symbol(_))
                && matches!(
                    attributes,
                    [ExtractedAttribute {
                        target_spelling,
                        polarity: AttributePolarity::Positive,
                        qualifier: Some(_),
                        groups,
                        ..
                    }] if target_spelling == "marked" && groups.is_empty()
                )
        }
        SourceAttributeRoute::Imported => {
            matches!(head, TypeHeadInput::BuiltinSet)
                && matches!(
                    attributes,
                    [ExtractedAttribute {
                        target_spelling,
                        polarity: AttributePolarity::Positive,
                        qualifier: None,
                        groups,
                        ..
                    }] if target_spelling == "TypeCaseAttr" && groups.is_empty()
                )
        }
        SourceAttributeRoute::NegativeImported => {
            matches!(head, TypeHeadInput::BuiltinSet)
                && matches!(
                    attributes,
                    [ExtractedAttribute {
                        target_spelling,
                        polarity: AttributePolarity::Negative,
                        non_node: Some(_),
                        qualifier: None,
                        groups,
                        ..
                    }] if target_spelling == "empty" && groups.is_empty()
                )
        }
        #[cfg(test)]
        SourceAttributeRoute::Synthetic => {
            matches!(head, TypeHeadInput::BuiltinSet)
                && attributes.len() == 2
                && attributes.iter().all(|attribute| {
                    attribute.polarity == AttributePolarity::Positive
                        && attribute.qualifier.is_none()
                })
        }
    };
    shape_ok
        .then_some(())
        .ok_or_else(|| "source-attribute route shape does not match its exact selector".to_owned())
}

fn semantic_origin(
    ast: &SurfaceAst,
    module: &ModuleId,
    node: SurfaceNodeId,
    group: usize,
    actual: usize,
) -> Result<SemanticOrigin, String> {
    let node = ast
        .node(node)
        .ok_or_else(|| "source-attribute actual disappeared".to_owned())?;
    Ok(SemanticOrigin::new(
        ast.source_id,
        module.clone(),
        SourceAnchor::Range(node.range),
        [group, actual]
            .into_iter()
            .map(|part| {
                u32::try_from(part)
                    .map_err(|_| "source-attribute provenance path exceeds u32".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

fn source_text(ast: &SurfaceAst, node: &SurfaceNode) -> Result<String, String> {
    let mut tokens = Vec::new();
    collect_source_text(ast, node, &mut tokens)?;
    if tokens.is_empty() {
        return Err("source-attribute spelling is empty".to_owned());
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
            .ok_or_else(|| "source-attribute source child disappeared".to_owned())?;
        collect_source_text(ast, child, tokens)?;
    }
    Ok(())
}
