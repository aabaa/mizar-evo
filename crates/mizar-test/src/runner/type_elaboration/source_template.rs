//! Test-only parser-origin template transport for Task 277A.
//!
//! This leaf deliberately has no production caller.  It preserves the frozen
//! parser surface profile as checker-owned neutral transport data and does not
//! identify a template target, actual kind, substitution, or semantic result.

use mizar_checker::{
    resolved_typed_ast::{ResolvedNodeKindHint, ResolvedNodeKindHintKind, SourceNodeRole},
    source_template::{
        SourceTemplateArgumentInput, SourceTemplateArgumentsId, SourceTemplateArgumentsInput,
        SourceTemplateError, SourceTemplateHandoff, SourceTemplateHandoffInput,
        SourceTemplateLociId, SourceTemplateLociInput, SourceTemplateLocusInput,
        SourceTemplateParameterInput, SourceTemplateParameterKind, SourceTemplateParentKind,
        SourceTemplateProducer, SourceTemplateRecovery,
    },
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedArenaBuilder, TypedAst,
        TypedAstParts, TypedNode, TypedNodeId, TypingState,
    },
};
use mizar_resolve::resolved_ast::ModuleId;
use mizar_session::SourceAnchor;
use mizar_syntax::SurfaceAst;

use super::checker_handoff::assemble_empty_resolved_typed_ast;

pub(in crate::runner) const SOURCE_TEMPLATE_TEXT: &str =
    include_str!("../../../../../tests/miz/pass/parser/pass_parser_template_arguments_001.miz");

#[derive(Debug)]
pub(in crate::runner) struct SourceTemplateRouteOutput {
    pub(in crate::runner) handoff: SourceTemplateHandoff,
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: mizar_checker::resolved_typed_ast::ResolvedTypedAst,
}

pub(in crate::runner) fn source_template_output(
    ast: &SurfaceAst,
    module: ModuleId,
    source_text: &str,
) -> Option<Result<SourceTemplateRouteOutput, String>> {
    source_template_output_with_mutation(ast, module, source_text, |_| {})
}

pub(in crate::runner) fn source_template_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceTemplateHandoffInput),
) -> Option<Result<SourceTemplateRouteOutput, String>> {
    if !has_exact_source_template_profile(ast, source_text) {
        return None;
    }
    Some(build_source_template_output(ast, module, mutate))
}

fn build_source_template_output(
    ast: &SurfaceAst,
    module: ModuleId,
    mutate: impl FnOnce(&mut SourceTemplateHandoffInput),
) -> Result<SourceTemplateRouteOutput, String> {
    let arena = all_surface_arena(ast)?;
    let mut input = source_template_input(ast, module.clone());
    mutate(&mut input);
    let handoff = SourceTemplateProducer::build(input, &arena).map_err(format_template_error)?;
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
    .map_err(|error| error.to_string())?
    .with_source_template(handoff.clone())
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.template.surface"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_template() != Some(&handoff) || resolved.source_template() != Some(&handoff)
    {
        return Err("Task277A template handoff did not survive neutral ownership".to_owned());
    }
    Ok(SourceTemplateRouteOutput {
        handoff,
        typed_ast,
        resolved,
    })
}

fn source_template_input(ast: &SurfaceAst, module_id: ModuleId) -> SourceTemplateHandoffInput {
    SourceTemplateHandoffInput {
        source_id: ast.source_id,
        module_id,
        parameters: vec![
            SourceTemplateParameterInput {
                site: node(60),
                source_range: range(ast, 60),
                source_ordinal: 0,
                recovery: SourceTemplateRecovery::Normal,
                parent: node(112),
                parent_kind: SourceTemplateParentKind::DefinitionBlockItem,
                kind: SourceTemplateParameterKind::AbstractTypeSyntax,
            },
            SourceTemplateParameterInput {
                site: node(63),
                source_range: range(ast, 63),
                source_ordinal: 1,
                recovery: SourceTemplateRecovery::Normal,
                parent: node(112),
                parent_kind: SourceTemplateParentKind::DefinitionBlockItem,
                kind: SourceTemplateParameterKind::TypedValueSyntax,
            },
        ],
        loci_groups: vec![
            SourceTemplateLociInput {
                site: node(65),
                source_range: range(ast, 65),
                source_ordinal: 0,
                recovery: SourceTemplateRecovery::Normal,
                parent: node(66),
                parent_kind: SourceTemplateParentKind::PredicatePattern,
            },
            SourceTemplateLociInput {
                site: node(76),
                source_range: range(ast, 76),
                source_ordinal: 1,
                recovery: SourceTemplateRecovery::Normal,
                parent: node(77),
                parent_kind: SourceTemplateParentKind::FunctorPattern,
            },
        ],
        loci: vec![
            SourceTemplateLocusInput {
                loci: SourceTemplateLociId::new(0),
                ordinal: 0,
                site: node(64),
                source_range: range(ast, 64),
                source_ordinal: 0,
                recovery: SourceTemplateRecovery::Normal,
            },
            SourceTemplateLocusInput {
                loci: SourceTemplateLociId::new(1),
                ordinal: 0,
                site: node(75),
                source_range: range(ast, 75),
                source_ordinal: 1,
                recovery: SourceTemplateRecovery::Normal,
            },
        ],
        argument_groups: vec![
            SourceTemplateArgumentsInput {
                site: node(91),
                source_range: range(ast, 91),
                source_ordinal: 0,
                recovery: SourceTemplateRecovery::Normal,
                parent: node(92),
                parent_kind: SourceTemplateParentKind::PredicateHead,
            },
            SourceTemplateArgumentsInput {
                site: node(100),
                source_range: range(ast, 100),
                source_ordinal: 1,
                recovery: SourceTemplateRecovery::Normal,
                parent: node(101),
                parent_kind: SourceTemplateParentKind::TermReference,
            },
        ],
        arguments: vec![
            SourceTemplateArgumentInput {
                arguments: SourceTemplateArgumentsId::new(0),
                ordinal: 0,
                site: node(90),
                source_range: range(ast, 90),
                source_ordinal: 0,
                recovery: SourceTemplateRecovery::Normal,
            },
            SourceTemplateArgumentInput {
                arguments: SourceTemplateArgumentsId::new(1),
                ordinal: 0,
                site: node(99),
                source_range: range(ast, 99),
                source_ordinal: 1,
                recovery: SourceTemplateRecovery::Normal,
            },
        ],
    }
}

fn all_surface_arena(ast: &SurfaceAst) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    for (index, surface) in ast.nodes().iter().enumerate() {
        let typed = builder
            .push(
                TypedNode::new(
                    surface_typed_kind(index, surface),
                    SourceAnchor::Range(surface.range),
                )
                .with_children(
                    surface
                        .children
                        .iter()
                        .map(|child| TypedNodeId::new(child.index()))
                        .collect(),
                )
                .with_typing(TypingState::Unknown)
                .with_recovery(if surface.recovered {
                    NodeRecoveryState::Recovered
                } else {
                    NodeRecoveryState::Normal
                }),
            )
            .map_err(|error| error.to_string())?;
        if typed.index() != index {
            return Err("Task277A all-surface typed arena is no longer dense".to_owned());
        }
    }
    builder
        .finish(ast.root().map(|root| TypedNodeId::new(root.index())))
        .map_err(|error| error.to_string())
}

fn has_exact_source_template_profile(ast: &SurfaceAst, source_text: &str) -> bool {
    source_text == SOURCE_TEMPLATE_TEXT
        && source_text.len() == 207
        && ast.nodes().len() == 116
        && ast.root().is_some_and(|root| root.index() == 115)
        && ast
            .root()
            .and_then(|root| ast.node(root))
            .is_some_and(|root| root.range.start == 0 && root.range.end == 206)
        && ast.nodes().iter().all(|surface| !surface.recovered)
        && exact_node(ast, 60, 13, 27, "TemplateParameter")
        && exact_node(ast, 63, 30, 41, "TemplateParameter")
        && exact_node(ast, 64, 76, 77, "TemplateLocus")
        && exact_node(ast, 65, 75, 78, "TemplateLoci")
        && exact_node_kind(ast, 66, "PredicatePattern")
        && exact_node(ast, 75, 121, 122, "TemplateLocus")
        && exact_node(ast, 76, 120, 123, "TemplateLoci")
        && exact_node_kind(ast, 77, "FunctorPattern")
        && exact_node(ast, 90, 179, 180, "TemplateArgument")
        && exact_node(ast, 91, 178, 181, "TemplateArguments")
        && exact_node_kind(ast, 92, "PredicateHead")
        && exact_node(ast, 99, 191, 192, "TemplateArgument")
        && exact_node(ast, 100, 190, 193, "TemplateArguments")
        && exact_node_kind(ast, 101, "TermReference")
        && exact_node_kind(ast, 112, "DefinitionBlockItem")
        && has_direct_child(ast, 112, 60)
        && has_direct_child(ast, 112, 63)
        && has_direct_child(ast, 66, 65)
        && has_direct_child(ast, 65, 64)
        && has_direct_child(ast, 77, 76)
        && has_direct_child(ast, 76, 75)
        && has_direct_child(ast, 92, 91)
        && has_direct_child(ast, 91, 90)
        && has_direct_child(ast, 101, 100)
        && has_direct_child(ast, 100, 99)
}

fn exact_node(ast: &SurfaceAst, index: usize, start: usize, end: usize, kind: &str) -> bool {
    ast.nodes().get(index).is_some_and(|node| {
        node.range.start == start
            && node.range.end == end
            && format!("{:?}", node.kind) == kind
            && !node.recovered
    })
}

fn exact_node_kind(ast: &SurfaceAst, index: usize, kind: &str) -> bool {
    ast.nodes()
        .get(index)
        .is_some_and(|node| format!("{:?}", node.kind) == kind && !node.recovered)
}

fn has_direct_child(ast: &SurfaceAst, parent: usize, child: usize) -> bool {
    ast.nodes()
        .get(parent)
        .is_some_and(|node| node.children.iter().any(|id| id.index() == child))
}

fn surface_typed_kind(index: usize, surface: &mizar_syntax::SurfaceNode) -> String {
    match index {
        60 => "AbstractTypeSyntax".to_owned(),
        63 => "TypedValueSyntax".to_owned(),
        _ => format!("{:?}", surface.kind),
    }
}

fn node(index: usize) -> TypedNodeId {
    TypedNodeId::new(index)
}

fn range(ast: &SurfaceAst, index: usize) -> mizar_session::SourceRange {
    ast.nodes()[index].range
}

fn format_template_error(error: SourceTemplateError) -> String {
    error.to_string()
}
