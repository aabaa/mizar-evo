use std::collections::BTreeMap;

use mizar_checker::{
    binding_env::{
        BindingContextDraft, BindingContextId, BindingContextLayer, BindingContextOwner,
        BindingContextRecovery, BindingContextTable, BindingDiagnosticClass,
        BindingDiagnosticDraft, BindingDiagnosticRecovery, BindingDiagnosticSeverity,
        BindingDiagnosticTable, BindingEnv, BindingEnvParts, BindingId, BindingTable,
    },
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_composite_formula::{
        SourceBinderTypeHead, SourceBinderTypeSiteId, SourceBinderTypeSiteInput,
        SourceCompositeFormulaHandoffInput, SourceCompositeFormulaId, SourceCompositeFormulaInput,
        SourceCompositeFormulaKind, SourceCompositeFormulaProducer, SourceCompositeFormulaRecovery,
        SourceFormulaEdgeInput, SourceFormulaEdgeRole, SourceFormulaRequestInput,
        SourceFormulaRequestKind, SourceFormulaRootInput, SourceFormulaRootOwnership,
        SourceQuantifierBinderId, SourceQuantifierBinderInput,
    },
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedArenaBuilder, TypedAst,
        TypedAstParts, TypedNode, TypedNodeId, TypedSiteRef,
    },
};
use mizar_resolve::{
    env::SymbolEnv,
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::ModuleId,
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::SurfaceAst;

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_formula::{
        SourceFormulaConnectiveQuantifier, extract_source_formula_connective_quantifier,
    },
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";

#[derive(Debug)]
pub(in crate::runner) struct SourceCompositeFormulaRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

/// Runs the bounded Task-257A transport before the pre-existing semantic route.
pub(in crate::runner) fn source_composite_formula_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    match source_composite_formula_output(ast, module, symbols) {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_context().is_none()
                && output.typed_ast.source_composite_formula().is_some()
                && output.typed_ast.source_composite_formula()
                    == output.resolved.source_composite_formula() =>
        {
            None
        }
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_composite_formula_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceCompositeFormulaRouteOutput, String>> {
    source_composite_formula_output_with_mutation(ast, module, symbols, |_| {})
}

#[cfg(not(test))]
fn source_composite_formula_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceCompositeFormulaRouteOutput, String>> {
    source_composite_formula_output_with_mutation(ast, module, symbols, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_composite_formula_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceCompositeFormulaHandoffInput),
) -> Option<Result<SourceCompositeFormulaRouteOutput, String>> {
    source_composite_formula_output_with_mutation_impl(ast, module, symbols, mutate)
}

#[cfg(not(test))]
fn source_composite_formula_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceCompositeFormulaHandoffInput),
) -> Option<Result<SourceCompositeFormulaRouteOutput, String>> {
    source_composite_formula_output_with_mutation_impl(ast, module, symbols, mutate)
}

fn source_composite_formula_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceCompositeFormulaHandoffInput),
) -> Option<Result<SourceCompositeFormulaRouteOutput, String>> {
    let payload = extract_source_formula_connective_quantifier(ast, &module, symbols)?;
    Some(build_output(ast, module, symbols, payload, mutate))
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    payload: SourceFormulaConnectiveQuantifier,
    mutate: impl FnOnce(&mut SourceCompositeFormulaHandoffInput),
) -> Result<SourceCompositeFormulaRouteOutput, String> {
    if symbols.module_id() != &module {
        return Err("Task257A symbol module mismatch".to_owned());
    }
    let arena = typed_arena(ast, &payload)?;
    let mut input = handoff_input(ast, module.clone(), &payload);
    mutate(&mut input);
    let base = module_shell(ast, module.clone())?;
    let bindings = SourceCompositeFormulaProducer::extend_bindings(&input, &base, &arena)
        .map_err(|error| error.to_string())?;
    let handoff = SourceCompositeFormulaProducer::build(input, &bindings, &arena)
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
    .with_source_composite_formula(handoff)
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.formula.composite"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_context().is_some()
        || typed_ast.source_composite_formula().is_none()
        || typed_ast.source_composite_formula() != resolved.source_composite_formula()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
    {
        return Err("Task257A immutable final handoff mismatch".to_owned());
    }
    Ok(SourceCompositeFormulaRouteOutput {
        typed_ast,
        resolved,
    })
}

fn module_shell(ast: &SurfaceAst, module: ModuleId) -> Result<BindingEnv, String> {
    let mut contexts = BindingContextTable::new();
    let context = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: Vec::new(),
        visible_bindings: Vec::new(),
        recovery: BindingContextRecovery::Normal,
    });
    if context != BindingContextId::new(0) {
        return Err("Task257A module context identity changed".to_owned());
    }
    let mut diagnostics = BindingDiagnosticTable::new();
    for message_key in [
        "checker.binding.external.local_bindings",
        "checker.binding.external.use_site_scope",
        "checker.binding.external.reserve_payload",
        "checker.binding.external.closure_payload",
    ] {
        diagnostics.insert(BindingDiagnosticDraft {
            source_range: None,
            class: BindingDiagnosticClass::ExternalDependencyGap,
            severity: BindingDiagnosticSeverity::Note,
            message_key: message_key.to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        });
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: ast.source_id,
        module_id: module,
        contexts,
        bindings: BindingTable::new(),
        diagnostics,
    })
    .map_err(|error| error.to_string())
}

fn typed_arena(
    ast: &SurfaceAst,
    payload: &SourceFormulaConnectiveQuantifier,
) -> Result<TypedArena, String> {
    let mut kinds = BTreeMap::from([
        (
            payload.implication_site.node().index(),
            "source.formula.composite.implication",
        ),
        (
            payload.premise_constant_site.node().index(),
            "source.formula.constant.contradiction",
        ),
        (
            payload.quantified_site.node().index(),
            "source.formula.composite.universal",
        ),
        (
            payload.negation_site.node().index(),
            "source.formula.composite.negation",
        ),
        (
            payload.body_constant_site.node().index(),
            "source.formula.constant.contradiction",
        ),
        (
            payload.binder_segment_site.node().index(),
            "source.formula.quantifier-binder",
        ),
        (
            payload.binder_identifier_site.node().index(),
            "source.formula.quantifier-binder",
        ),
        (
            payload.binder_type_site.node().index(),
            "source.formula.binder-type",
        ),
        (
            payload.binder_type_head_site.node().index(),
            "source.formula.binder-type-head",
        ),
    ]);
    if kinds.len() != 9 {
        return Err("Task257A source roles alias typed sites".to_owned());
    }
    let mut builder = TypedArenaBuilder::new();
    for (index, node) in ast.nodes().iter().enumerate() {
        let key = kinds
            .remove(&index)
            .unwrap_or("source.formula.composite.unowned");
        let pushed = builder
            .push(
                TypedNode::new(key, SourceAnchor::Range(node.range))
                    .with_recovery(NodeRecoveryState::Normal),
            )
            .map_err(|error| error.to_string())?;
        if pushed != TypedNodeId::new(index) {
            return Err("Task257A typed-arena identity changed".to_owned());
        }
    }
    if !kinds.is_empty() {
        return Err("Task257A source site disappeared".to_owned());
    }
    builder.finish(None).map_err(|error| error.to_string())
}

fn handoff_input(
    ast: &SurfaceAst,
    module: ModuleId,
    payload: &SourceFormulaConnectiveQuantifier,
) -> SourceCompositeFormulaHandoffInput {
    let context0 = BindingContextId::new(0);
    let context1 = BindingContextId::new(1);
    let formula = |site: TypedSiteRef,
                   source_range: SourceRange,
                   source_ordinal: usize,
                   context: BindingContextId,
                   spelling: &str,
                   kind: SourceCompositeFormulaKind| {
        SourceCompositeFormulaInput {
            site,
            source_range,
            source_ordinal,
            context,
            recovery: SourceCompositeFormulaRecovery::Normal,
            spelling: spelling.to_owned(),
            kind,
        }
    };
    SourceCompositeFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        formulas: vec![
            formula(
                payload.implication_site.clone(),
                payload.implication_range,
                0,
                context0,
                "implies",
                SourceCompositeFormulaKind::Implication,
            ),
            formula(
                payload.premise_constant_site.clone(),
                payload.premise_constant_range,
                1,
                context0,
                "contradiction",
                SourceCompositeFormulaKind::Contradiction,
            ),
            formula(
                payload.quantified_site.clone(),
                payload.quantified_range,
                2,
                context0,
                "for holds",
                SourceCompositeFormulaKind::Universal,
            ),
            formula(
                payload.negation_site.clone(),
                payload.negation_range,
                3,
                context1,
                "not",
                SourceCompositeFormulaKind::Negation,
            ),
            formula(
                payload.body_constant_site.clone(),
                payload.body_constant_range,
                4,
                context1,
                "contradiction",
                SourceCompositeFormulaKind::Contradiction,
            ),
        ],
        wrappers: Vec::new(),
        roots: vec![SourceFormulaRootInput {
            formula: SourceCompositeFormulaId::new(0),
            ordinal: 0,
            ownership: SourceFormulaRootOwnership::UnassignedStatement,
        }],
        binders: vec![SourceQuantifierBinderInput {
            formula: SourceCompositeFormulaId::new(2),
            ordinal: 0,
            segment_site: payload.binder_segment_site.clone(),
            segment_range: payload.binder_segment_range,
            segment_spelling: "x being".to_owned(),
            identifier_site: payload.binder_identifier_site.clone(),
            identifier_range: payload.binder_identifier_range,
            identifier_spelling: payload.binder_identifier_spelling.clone(),
            local: LocalTermBinding::new(
                payload.binder_identifier_spelling.clone(),
                LocalTermScope::new(vec![0]),
                payload.binder_identifier_range,
                0,
            ),
            binding: BindingId::new(0),
            body_context: context1,
            type_site: SourceBinderTypeSiteId::new(0),
            recovery: SourceCompositeFormulaRecovery::Normal,
        }],
        type_sites: vec![SourceBinderTypeSiteInput {
            binder: SourceQuantifierBinderId::new(0),
            site: payload.binder_type_site.clone(),
            source_range: payload.binder_type_range,
            spelling: payload.binder_type_spelling.clone(),
            head_site: payload.binder_type_head_site.clone(),
            head_range: payload.binder_type_head_range,
            head_spelling: payload.binder_type_head_spelling.clone(),
            context: context0,
            recovery: SourceCompositeFormulaRecovery::Normal,
            head: SourceBinderTypeHead::BuiltinSet,
        }],
        edges: vec![
            SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                role: SourceFormulaEdgeRole::ImplicationLeft,
                child: SourceCompositeFormulaId::new(1),
            },
            SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(0),
                ordinal: 1,
                role: SourceFormulaEdgeRole::ImplicationRight,
                child: SourceCompositeFormulaId::new(2),
            },
            SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(2),
                ordinal: 0,
                role: SourceFormulaEdgeRole::UniversalBody,
                child: SourceCompositeFormulaId::new(3),
            },
            SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(3),
                ordinal: 0,
                role: SourceFormulaEdgeRole::NegatedFormula,
                child: SourceCompositeFormulaId::new(4),
            },
        ],
        requests: vec![
            request(0, 0, SourceFormulaRequestKind::ConnectiveSemantics),
            request(1, 0, SourceFormulaRequestKind::ConstantSemantics),
            request(2, 0, SourceFormulaRequestKind::QuantifierSemantics),
            SourceFormulaRequestInput {
                formula: SourceCompositeFormulaId::new(2),
                ordinal: 1,
                kind: SourceFormulaRequestKind::BinderType,
                binder: Some(SourceQuantifierBinderId::new(0)),
                type_site: Some(SourceBinderTypeSiteId::new(0)),
            },
            request(3, 0, SourceFormulaRequestKind::NegationSemantics),
            request(4, 0, SourceFormulaRequestKind::ConstantSemantics),
        ],
    }
}

fn request(
    formula: usize,
    ordinal: usize,
    kind: SourceFormulaRequestKind,
) -> SourceFormulaRequestInput {
    SourceFormulaRequestInput {
        formula: SourceCompositeFormulaId::new(formula),
        ordinal,
        kind,
        binder: None,
        type_site: None,
    }
}
