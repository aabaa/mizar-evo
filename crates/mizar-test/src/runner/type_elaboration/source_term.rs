use std::collections::BTreeMap;

use mizar_checker::{
    binding_env::{BindingContextId, BindingEnv, BindingId, BindingKind},
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_term::{
        SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermId,
        SourcePrimaryTermInput, SourcePrimaryTermKind, SourcePrimaryTermProducer,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceInput, SourcePrimaryTermReferenceRole,
        SourcePrimaryTermRole,
    },
    type_checker::FormulaKind,
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedArenaBuilder, TypedAst,
        TypedAstParts, TypedNode, TypedNodeId, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{env::SymbolEnv, resolved_ast::ModuleId};
use mizar_session::SourceAnchor;
use mizar_syntax::{SurfaceAst, SurfaceNodeKind};

use super::{
    binary_routes::extract_source_reserved_variable_equality,
    checker_handoff::{assemble_empty_resolved_typed_ast, source_module_binding_env},
    parenthesized_routes::extract_source_parenthesized_reserved_variable_equality,
    source_ast::{direct_token_texts, structural_child_ids, subtree_has_recovery},
    source_formula::{SourceParenthesizedOperandSide, extract_source_builtin_binary_term_formula},
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";

#[derive(Debug)]
pub(in crate::runner) struct SourceTermRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

/// Runs the bounded Task-252 transport on its exact real selectors without
/// replacing the legacy route that owns the case's existing detail keys.
pub(in crate::runner) fn source_term_transport_error_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    match source_term_output(ast, module, symbols) {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_term().is_some()
                && output.typed_ast.source_term() == output.resolved.source_term() =>
        {
            None
        }
        Some(Ok(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
        Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceTermRouteOutput, String>> {
    source_term_output_with_mutation(ast, module, symbols, |_| {})
}

#[cfg(not(test))]
fn source_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceTermRouteOutput, String>> {
    source_term_output_with_mutation(ast, module, symbols, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_term_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourcePrimaryTermHandoffInput),
) -> Option<Result<SourceTermRouteOutput, String>> {
    source_term_output_with_mutation_impl(ast, module, symbols, mutate)
}

#[cfg(not(test))]
fn source_term_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourcePrimaryTermHandoffInput),
) -> Option<Result<SourceTermRouteOutput, String>> {
    source_term_output_with_mutation_impl(ast, module, symbols, mutate)
}

fn source_term_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourcePrimaryTermHandoffInput),
) -> Option<Result<SourceTermRouteOutput, String>> {
    let extracted = if let Some(payload) = extract_source_builtin_binary_term_formula(ast) {
        if payload.formula_kind != FormulaKind::Equality {
            return None;
        }
        let binding_env = match source_module_binding_env(ast, module.clone()) {
            Ok(binding_env) => binding_env,
            Err(error) => return Some(Err(error.to_string())),
        };
        let roots = [
            payload.left_site.node().index(),
            payload.right_site.node().index(),
        ];
        match extract_roots(ast, roots, &binding_env, ReferenceClassification::Variable) {
            Ok(terms) => Some((binding_env, terms)),
            Err(error) => return Some(Err(error)),
        }
    } else if let Some(payload) =
        extract_source_parenthesized_reserved_variable_equality(ast, module.clone(), symbols)
    {
        if payload.wrapper_side != SourceParenthesizedOperandSide::Left {
            return None;
        }
        let binding_env = match payload.formula.reserve.bridge.prepare_binding_env(symbols) {
            Ok(binding_env) => binding_env,
            Err(error) => return Some(Err(error)),
        };
        let roots = [
            payload.wrapper_site.node().index(),
            payload.formula.right_site.node().index(),
        ];
        match extract_roots(ast, roots, &binding_env, ReferenceClassification::Variable) {
            Ok(terms) => Some((binding_env, terms)),
            Err(error) => return Some(Err(error)),
        }
    } else if let Some(payload) =
        extract_source_reserved_variable_equality(ast, module.clone(), symbols)
    {
        let binding_env = match payload.reserve.bridge.prepare_binding_env(symbols) {
            Ok(binding_env) => binding_env,
            Err(error) => return Some(Err(error)),
        };
        let roots = [
            payload.left_site.node().index(),
            payload.right_site.node().index(),
        ];
        match extract_roots(ast, roots, &binding_env, ReferenceClassification::Variable) {
            Ok(terms) => Some((binding_env, terms)),
            Err(error) => return Some(Err(error)),
        }
    } else {
        None
    }?;

    Some(build_output(ast, module, extracted.0, extracted.1, mutate))
}

#[cfg(test)]
pub(in crate::runner) fn synthetic_source_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
) -> Result<SourceTermRouteOutput, String> {
    if binding_env.source_id() != ast.source_id || binding_env.module_id() != &module {
        return Err("synthetic source-term binding environment identity mismatch".to_owned());
    }
    let roots = outermost_term_expression_roots(ast);
    let terms = extract_roots(
        ast,
        roots,
        &binding_env,
        ReferenceClassification::FromBindingKind,
    )?;
    build_output(ast, module, binding_env, terms, |_| {})
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    terms: Vec<ExtractedTerm>,
    mutate: impl FnOnce(&mut SourcePrimaryTermHandoffInput),
) -> Result<SourceTermRouteOutput, String> {
    let arena = surface_indexed_arena(ast, &terms)?;
    let mut input = handoff_input(ast, module.clone(), &binding_env, &terms)?;
    mutate(&mut input);
    let handoff = SourcePrimaryTermProducer::build(input, &binding_env, &arena)
        .map_err(|error| error.to_string())?;
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
    .with_source_term(handoff)
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.term.surface"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_term().is_none()
        || resolved.source_term() != typed_ast.source_term()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
    {
        return Err("source primary-term immutable final handoff mismatch".to_owned());
    }
    Ok(SourceTermRouteOutput {
        typed_ast,
        resolved,
    })
}

#[derive(Debug, Clone, Copy)]
enum ReferenceClassification {
    Variable,
    #[cfg(test)]
    FromBindingKind,
}

#[derive(Debug, Clone)]
struct ExtractedTerm {
    node: TypedNodeId,
    source_range: mizar_session::SourceRange,
    spelling: String,
    kind: SourcePrimaryTermKind,
    role: SourcePrimaryTermRole,
    parent: Option<SourcePrimaryTermId>,
}

fn extract_roots(
    ast: &SurfaceAst,
    roots: impl IntoIterator<Item = usize>,
    binding_env: &BindingEnv,
    classification: ReferenceClassification,
) -> Result<Vec<ExtractedTerm>, String> {
    let mut roots = roots.into_iter().collect::<Vec<_>>();
    roots.sort_by_key(|root| {
        ast.nodes()
            .get(*root)
            .map(|node| (node.range.start, node.range.end))
            .unwrap_or((usize::MAX, usize::MAX))
    });
    roots.dedup();

    let mut terms = Vec::new();
    for root in roots {
        let checkpoint = terms.len();
        if !collect_eligible_term(ast, root, None, binding_env, classification, &mut terms)? {
            terms.truncate(checkpoint);
        }
    }
    Ok(terms)
}

fn collect_eligible_term(
    ast: &SurfaceAst,
    id: usize,
    parent: Option<SourcePrimaryTermId>,
    binding_env: &BindingEnv,
    classification: ReferenceClassification,
    terms: &mut Vec<ExtractedTerm>,
) -> Result<bool, String> {
    let Some(node) = ast.nodes().get(id) else {
        return Ok(false);
    };
    if subtree_has_recovery(ast, node) {
        return Ok(false);
    }

    match node.kind {
        SurfaceNodeKind::TermExpression => {
            if !direct_token_texts(ast, node).is_empty() {
                return Ok(false);
            }
            let children = structural_child_ids(ast, node);
            let [child] = children.as_slice() else {
                return Ok(false);
            };
            collect_eligible_term(
                ast,
                child.index(),
                parent,
                binding_env,
                classification,
                terms,
            )
        }
        SurfaceNodeKind::TermReference => {
            if !structural_child_ids(ast, node).is_empty() {
                return Ok(false);
            }
            let tokens = direct_token_texts(ast, node);
            let [spelling] = tokens.as_slice() else {
                return Ok(false);
            };
            if spelling.is_empty() {
                return Ok(false);
            }
            let kind = reference_kind(binding_env, spelling, classification)?;
            terms.push(ExtractedTerm {
                node: TypedNodeId::new(id),
                source_range: node.range,
                spelling: spelling.clone(),
                kind,
                role: SourcePrimaryTermRole::Value,
                parent,
            });
            Ok(true)
        }
        SurfaceNodeKind::ItTerm => {
            if !structural_child_ids(ast, node).is_empty()
                || direct_token_texts(ast, node).as_slice() != ["it"]
            {
                return Ok(false);
            }
            terms.push(ExtractedTerm {
                node: TypedNodeId::new(id),
                source_range: node.range,
                spelling: "it".to_owned(),
                kind: SourcePrimaryTermKind::It,
                role: SourcePrimaryTermRole::CurrentDefinitionResult,
                parent,
            });
            Ok(true)
        }
        SurfaceNodeKind::NumeralTerm => {
            if !structural_child_ids(ast, node).is_empty() {
                return Ok(false);
            }
            let tokens = direct_token_texts(ast, node);
            let [spelling] = tokens.as_slice() else {
                return Ok(false);
            };
            if spelling.is_empty() || !spelling.bytes().all(|byte| byte.is_ascii_digit()) {
                return Ok(false);
            }
            terms.push(ExtractedTerm {
                node: TypedNodeId::new(id),
                source_range: node.range,
                spelling: spelling.clone(),
                kind: SourcePrimaryTermKind::Numeral,
                role: SourcePrimaryTermRole::Value,
                parent,
            });
            Ok(true)
        }
        SurfaceNodeKind::ParenthesizedTerm => {
            if direct_token_texts(ast, node).as_slice() != ["(", ")"] {
                return Ok(false);
            }
            let children = structural_child_ids(ast, node);
            let [child] = children.as_slice() else {
                return Ok(false);
            };
            let checkpoint = terms.len();
            let wrapper = SourcePrimaryTermId::new(checkpoint);
            terms.push(ExtractedTerm {
                node: TypedNodeId::new(id),
                source_range: node.range,
                spelling: String::new(),
                kind: SourcePrimaryTermKind::Parenthesized,
                role: SourcePrimaryTermRole::Value,
                parent,
            });
            if !collect_eligible_term(
                ast,
                child.index(),
                Some(wrapper),
                binding_env,
                classification,
                terms,
            )? || terms.len() < checkpoint + 2
            {
                terms.truncate(checkpoint);
                return Ok(false);
            }
            terms[checkpoint].spelling = format!("( {} )", terms[checkpoint + 1].spelling);
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn reference_kind(
    _binding_env: &BindingEnv,
    _spelling: &str,
    classification: ReferenceClassification,
) -> Result<SourcePrimaryTermKind, String> {
    match classification {
        ReferenceClassification::Variable => Ok(SourcePrimaryTermKind::VariableReference),
        #[cfg(test)]
        ReferenceClassification::FromBindingKind => {
            reference_kind_from_binding(_binding_env, _spelling)
        }
    }
}

#[cfg(test)]
fn reference_kind_from_binding(
    binding_env: &BindingEnv,
    spelling: &str,
) -> Result<SourcePrimaryTermKind, String> {
    let mut kinds = binding_env
        .bindings()
        .iter()
        .filter_map(|(_, binding)| (binding.spelling == spelling).then_some(binding.kind))
        .collect::<Vec<_>>();
    kinds.sort();
    kinds.dedup();
    match kinds.as_slice() {
        [BindingKind::LocalAbbreviation] => Ok(SourcePrimaryTermKind::ConstantReference),
        [BindingKind::ReservedVariable]
        | [BindingKind::LetBinding]
        | [BindingKind::QuantifierBinder]
        | [BindingKind::DefinitionParameter] => Ok(SourcePrimaryTermKind::VariableReference),
        [] => Err(format!(
            "source primary-term reference `{spelling}` has no authenticated binding"
        )),
        _ => Err(format!(
            "source primary-term reference `{spelling}` has no unique Task-252 role"
        )),
    }
}

fn handoff_input(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: &BindingEnv,
    extracted: &[ExtractedTerm],
) -> Result<SourcePrimaryTermHandoffInput, String> {
    let terms = extracted
        .iter()
        .enumerate()
        .map(|(source_ordinal, term)| SourcePrimaryTermInput {
            site: TypedSiteRef::Node(term.node),
            source_range: term.source_range,
            source_ordinal,
            context: BindingContextId::new(0),
            recovery: SourcePrimaryTermRecovery::Normal,
            spelling: term.spelling.clone(),
            kind: term.kind,
            role: term.role,
            parent: term.parent,
        })
        .collect::<Vec<_>>();
    let mut references = Vec::new();
    let mut numeric_type_requests = Vec::new();
    for (index, term) in extracted.iter().enumerate() {
        match term.kind {
            SourcePrimaryTermKind::VariableReference | SourcePrimaryTermKind::ConstantReference => {
                let role = if term.kind == SourcePrimaryTermKind::VariableReference {
                    SourcePrimaryTermReferenceRole::Variable
                } else {
                    SourcePrimaryTermReferenceRole::LocalConstant
                };
                references.push(SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(index),
                    binding: unique_binding(binding_env, &term.spelling, role)?,
                    role,
                });
            }
            SourcePrimaryTermKind::Numeral => {
                numeric_type_requests.push(SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(index),
                    owner: TypedSiteRef::Node(term.node),
                    source_range: term.source_range,
                    spelling: term.spelling.clone(),
                    request_ordinal: numeric_type_requests.len(),
                });
            }
            SourcePrimaryTermKind::It | SourcePrimaryTermKind::Parenthesized => {}
            _ => return Err("source primary-term extractor saw an unowned kind".to_owned()),
        }
    }
    Ok(SourcePrimaryTermHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        terms,
        references,
        numeric_type_requests,
    })
}

fn unique_binding(
    binding_env: &BindingEnv,
    spelling: &str,
    role: SourcePrimaryTermReferenceRole,
) -> Result<BindingId, String> {
    let matches_role = |kind| match role {
        SourcePrimaryTermReferenceRole::Variable => matches!(
            kind,
            BindingKind::ReservedVariable
                | BindingKind::LetBinding
                | BindingKind::QuantifierBinder
                | BindingKind::DefinitionParameter
        ),
        SourcePrimaryTermReferenceRole::LocalConstant => kind == BindingKind::LocalAbbreviation,
        _ => false,
    };
    let bindings = binding_env
        .bindings()
        .iter()
        .filter_map(|(id, binding)| {
            (binding.spelling == spelling && matches_role(binding.kind)).then_some(id)
        })
        .collect::<Vec<_>>();
    match bindings.as_slice() {
        [binding] => Ok(*binding),
        _ => Err(format!(
            "source primary-term reference `{spelling}` requires one authenticated binding"
        )),
    }
}

fn surface_indexed_arena(ast: &SurfaceAst, terms: &[ExtractedTerm]) -> Result<TypedArena, String> {
    let kinds = terms
        .iter()
        .map(|term| (term.node.index(), typed_kind_key(term.kind)))
        .collect::<BTreeMap<_, _>>();
    let mut builder = TypedArenaBuilder::new();
    for (index, node) in ast.nodes().iter().enumerate() {
        let kind = kinds
            .get(&index)
            .copied()
            .unwrap_or("source.surface.unowned");
        let children = node
            .children
            .iter()
            .map(|child| TypedNodeId::new(child.index()))
            .collect();
        let typed = builder
            .push(
                TypedNode::new(kind, SourceAnchor::Range(node.range))
                    .with_children(children)
                    .with_typing(TypingState::Unknown)
                    .with_recovery(if node.recovered {
                        NodeRecoveryState::Recovered
                    } else {
                        NodeRecoveryState::Normal
                    }),
            )
            .map_err(|error| error.to_string())?;
        if typed.index() != index {
            return Err("surface-indexed typed arena lost node identity".to_owned());
        }
    }
    builder
        .finish(ast.root().map(|root| TypedNodeId::new(root.index())))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
fn outermost_term_expression_roots(ast: &SurfaceAst) -> Vec<usize> {
    let mut parent = vec![None; ast.nodes().len()];
    for (parent_index, node) in ast.nodes().iter().enumerate() {
        for child in &node.children {
            parent[child.index()] = Some(parent_index);
        }
    }
    let mut roots = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| matches!(node.kind, SurfaceNodeKind::TermExpression))
        .filter(|(index, _)| {
            let mut cursor = parent[*index];
            while let Some(ancestor) = cursor {
                if matches!(ast.nodes()[ancestor].kind, SurfaceNodeKind::TermExpression) {
                    return false;
                }
                cursor = parent[ancestor];
            }
            true
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    roots.sort_by_key(|root| {
        let range = ast.nodes()[*root].range;
        (range.start, range.end)
    });
    roots
}

fn typed_kind_key(kind: SourcePrimaryTermKind) -> &'static str {
    match kind {
        SourcePrimaryTermKind::VariableReference => "source.term.variable-reference",
        SourcePrimaryTermKind::ConstantReference => "source.term.constant-reference",
        SourcePrimaryTermKind::It => "source.term.it",
        SourcePrimaryTermKind::Numeral => "source.term.numeral",
        SourcePrimaryTermKind::Parenthesized => "source.term.parenthesized",
        _ => "source.surface.unowned",
    }
}
