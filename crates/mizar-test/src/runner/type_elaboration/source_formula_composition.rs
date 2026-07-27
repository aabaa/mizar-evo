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
    source_atomic_formula::{
        SourceAtomicEdgeId, SourceAtomicEdgeInput, SourceAtomicEdgeRole,
        SourceAtomicFormulaHandoffInput, SourceAtomicFormulaId, SourceAtomicFormulaInput,
        SourceAtomicFormulaKind, SourceAtomicFormulaProducer, SourceAtomicFormulaRecovery,
        SourceAtomicRequestInput, SourceAtomicRequestKind, SourceAtomicTermTarget,
    },
    source_composite_formula::{
        SourceBinderTypeHead, SourceBinderTypeSiteId, SourceBinderTypeSiteInput,
        SourceCompositeFormulaHandoffInput, SourceCompositeFormulaId, SourceCompositeFormulaInput,
        SourceCompositeFormulaKind, SourceCompositeFormulaProducer, SourceCompositeFormulaRecovery,
        SourceFormulaRequestInput, SourceFormulaRequestKind, SourceFormulaRootInput,
        SourceFormulaRootOwnership, SourceQuantifierBinderId, SourceQuantifierBinderInput,
    },
    source_formula_composition::{
        SourceFormulaAtomicEdgeId, SourceFormulaAtomicEdgeInput, SourceFormulaAtomicEdgeRole,
        SourceFormulaCompositionHandoffInput, SourceFormulaCompositionProducer,
        SourceQuantifierBoundUseInput,
    },
    source_term::{
        SourcePrimaryTermHandoffInput, SourcePrimaryTermId, SourcePrimaryTermInput,
        SourcePrimaryTermKind, SourcePrimaryTermProducer, SourcePrimaryTermRecovery,
        SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceInput,
        SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
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
use mizar_session::SourceAnchor;
use mizar_syntax::SurfaceAst;

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_formula::{
        SourceFormulaQuantifierBoundUse, extract_source_formula_quantifier_bound_use,
    },
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";

#[derive(Debug)]
pub(in crate::runner) struct SourceFormulaCompositionRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug)]
pub(in crate::runner) struct SourceFormulaCompositionRouteInputs {
    pub(in crate::runner) primary: SourcePrimaryTermHandoffInput,
    pub(in crate::runner) atomic: SourceAtomicFormulaHandoffInput,
    pub(in crate::runner) composite: SourceCompositeFormulaHandoffInput,
    pub(in crate::runner) composition: SourceFormulaCompositionHandoffInput,
}

pub(in crate::runner) fn source_formula_composition_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    match source_formula_composition_output(ast, module, symbols) {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_context().is_none()
                && output.typed_ast.source_term().is_some()
                && output.typed_ast.source_atomic_formula().is_some()
                && output.typed_ast.source_composite_formula().is_some()
                && output.typed_ast.source_formula_composition().is_some()
                && output.typed_ast.source_formula_composition()
                    == output.resolved.source_formula_composition() =>
        {
            Some(Vec::new())
        }
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_formula_composition_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_formula_composition_output_with_mutation(ast, module, symbols, |_| {})
}

#[cfg(not(test))]
fn source_formula_composition_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_formula_composition_output_with_mutation(ast, module, symbols, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_formula_composition_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_formula_composition_output_with_mutation_impl(ast, module, symbols, mutate)
}

#[cfg(not(test))]
fn source_formula_composition_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_formula_composition_output_with_mutation_impl(ast, module, symbols, mutate)
}

fn source_formula_composition_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    let payload = extract_source_formula_quantifier_bound_use(ast, &module, symbols)?;
    Some(build_output(ast, module, symbols, payload, mutate))
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    payload: SourceFormulaQuantifierBoundUse,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Result<SourceFormulaCompositionRouteOutput, String> {
    if symbols.module_id() != &module {
        return Err("Task257B1 symbol module mismatch".to_owned());
    }
    let arena = typed_arena(ast, &payload)?;
    let mut inputs = route_inputs(ast, module.clone(), &payload);
    mutate(&mut inputs);

    let base = module_shell(ast, module.clone())?;
    let bindings =
        SourceCompositeFormulaProducer::extend_bindings(&inputs.composite, &base, &arena)
            .map_err(|error| error.to_string())?;
    let composite = SourceCompositeFormulaProducer::build(inputs.composite, &bindings, &arena)
        .map_err(|error| error.to_string())?;
    let primary = SourcePrimaryTermProducer::build(inputs.primary, &bindings, &arena)
        .map_err(|error| error.to_string())?;
    let atomic = SourceAtomicFormulaProducer::build(
        inputs.atomic,
        &bindings,
        symbols,
        &primary,
        None,
        None,
        None,
        &arena,
    )
    .map_err(|error| error.to_string())?;
    let composition = SourceFormulaCompositionProducer::build(
        inputs.composition,
        &primary,
        &atomic,
        &composite,
        &arena,
    )
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
    .with_source_term(primary)
    .map_err(|error| error.to_string())?
    .with_source_atomic_formula(atomic)
    .map_err(|error| error.to_string())?
    .with_source_formula_composition(composite, composition)
    .map_err(|error| error.to_string())?;

    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.formula.composition"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_context().is_some()
        || typed_ast.source_formula_composition().is_none()
        || typed_ast.source_formula_composition() != resolved.source_formula_composition()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
    {
        return Err("Task257B1 immutable final handoff mismatch".to_owned());
    }
    Ok(SourceFormulaCompositionRouteOutput {
        typed_ast,
        resolved,
    })
}

fn route_inputs(
    ast: &SurfaceAst,
    module: ModuleId,
    payload: &SourceFormulaQuantifierBoundUse,
) -> SourceFormulaCompositionRouteInputs {
    let context0 = BindingContextId::new(0);
    let context1 = BindingContextId::new(1);
    let composite = SourceCompositeFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        formulas: vec![SourceCompositeFormulaInput {
            site: payload.quantified_site.clone(),
            source_range: payload.quantified_range,
            source_ordinal: 0,
            context: context0,
            recovery: SourceCompositeFormulaRecovery::Normal,
            spelling: "for holds".to_owned(),
            kind: SourceCompositeFormulaKind::Universal,
        }],
        wrappers: Vec::new(),
        roots: vec![SourceFormulaRootInput {
            formula: SourceCompositeFormulaId::new(0),
            ordinal: 0,
            ownership: SourceFormulaRootOwnership::UnassignedStatement,
        }],
        binders: vec![SourceQuantifierBinderInput {
            formula: SourceCompositeFormulaId::new(0),
            ordinal: 0,
            segment_site: payload.binder_segment_site.clone(),
            segment_range: payload.binder_segment_range,
            segment_spelling: "x being".to_owned(),
            identifier_site: payload.binder_identifier_site.clone(),
            identifier_range: payload.binder_identifier_range,
            identifier_spelling: "x".to_owned(),
            local: LocalTermBinding::new(
                "x",
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
            spelling: "set".to_owned(),
            head_site: payload.binder_type_head_site.clone(),
            head_range: payload.binder_type_head_range,
            head_spelling: "set".to_owned(),
            context: context0,
            recovery: SourceCompositeFormulaRecovery::Normal,
            head: SourceBinderTypeHead::BuiltinSet,
        }],
        edges: Vec::new(),
        requests: vec![
            SourceFormulaRequestInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                kind: SourceFormulaRequestKind::QuantifierSemantics,
                binder: None,
                type_site: None,
            },
            SourceFormulaRequestInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 1,
                kind: SourceFormulaRequestKind::BinderType,
                binder: Some(SourceQuantifierBinderId::new(0)),
                type_site: Some(SourceBinderTypeSiteId::new(0)),
            },
        ],
    };
    let primary = SourcePrimaryTermHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms: vec![
            variable_term(payload.left_site.clone(), payload.left_range, 0, context1),
            variable_term(payload.right_site.clone(), payload.right_range, 1, context1),
        ],
        references: vec![
            SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(0),
                binding: BindingId::new(0),
                role: SourcePrimaryTermReferenceRole::Variable,
            },
            SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(1),
                binding: BindingId::new(0),
                role: SourcePrimaryTermReferenceRole::Variable,
            },
        ],
        numeric_type_requests: Vec::new(),
    };
    let atomic = SourceAtomicFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        formulas: vec![SourceAtomicFormulaInput {
            site: payload.equality_site.clone(),
            source_range: payload.equality_range,
            source_ordinal: 0,
            context: context1,
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = x".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        }],
        wrappers: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges: vec![
            SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
            },
            SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 1,
                role: SourceAtomicEdgeRole::BuiltinRightOperand,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
            },
        ],
        requests: vec![
            SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(0)),
                candidate: None,
                type_site: None,
                attribute: None,
            },
            SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 1,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(1)),
                candidate: None,
                type_site: None,
                attribute: None,
            },
        ],
    };
    let composition = SourceFormulaCompositionHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        atomic_edges: vec![SourceFormulaAtomicEdgeInput {
            formula: SourceCompositeFormulaId::new(0),
            ordinal: 0,
            role: SourceFormulaAtomicEdgeRole::UniversalBody,
            child: SourceAtomicFormulaId::new(0),
        }],
        bound_uses: vec![
            SourceQuantifierBoundUseInput {
                binder: SourceQuantifierBinderId::new(0),
                ordinal: 0,
                body_edge: SourceFormulaAtomicEdgeId::new(0),
                term: SourcePrimaryTermId::new(0),
                reference: SourcePrimaryTermReferenceId::new(0),
            },
            SourceQuantifierBoundUseInput {
                binder: SourceQuantifierBinderId::new(0),
                ordinal: 1,
                body_edge: SourceFormulaAtomicEdgeId::new(0),
                term: SourcePrimaryTermId::new(1),
                reference: SourcePrimaryTermReferenceId::new(1),
            },
        ],
    };
    SourceFormulaCompositionRouteInputs {
        primary,
        atomic,
        composite,
        composition,
    }
}

fn variable_term(
    site: TypedSiteRef,
    source_range: mizar_session::SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
) -> SourcePrimaryTermInput {
    SourcePrimaryTermInput {
        site,
        source_range,
        source_ordinal,
        context,
        recovery: SourcePrimaryTermRecovery::Normal,
        spelling: "x".to_owned(),
        kind: SourcePrimaryTermKind::VariableReference,
        role: SourcePrimaryTermRole::Value,
        parent: None,
    }
}

fn typed_arena(
    ast: &SurfaceAst,
    payload: &SourceFormulaQuantifierBoundUse,
) -> Result<TypedArena, String> {
    let mut kinds = BTreeMap::from([
        (
            payload.quantified_site.node().index(),
            "source.formula.composite.universal",
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
        (
            payload.equality_site.node().index(),
            "source.formula.atomic.equality",
        ),
        (
            payload.left_site.node().index(),
            "source.term.variable-reference",
        ),
        (
            payload.right_site.node().index(),
            "source.term.variable-reference",
        ),
    ]);
    if kinds.len() != 8 {
        return Err("Task257B1 source roles alias typed sites".to_owned());
    }
    let mut builder = TypedArenaBuilder::new();
    for (index, node) in ast.nodes().iter().enumerate() {
        let key = kinds
            .remove(&index)
            .unwrap_or("source.formula.composition.unowned");
        let pushed = builder
            .push(
                TypedNode::new(key, SourceAnchor::Range(node.range))
                    .with_recovery(NodeRecoveryState::Normal),
            )
            .map_err(|error| error.to_string())?;
        if pushed != TypedNodeId::new(index) {
            return Err("Task257B1 typed-arena identity changed".to_owned());
        }
    }
    if !kinds.is_empty() {
        return Err("Task257B1 source site disappeared".to_owned());
    }
    builder.finish(None).map_err(|error| error.to_string())
}

fn module_shell(ast: &SurfaceAst, module: ModuleId) -> Result<BindingEnv, String> {
    let mut contexts = BindingContextTable::new();
    if contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: Vec::new(),
        visible_bindings: Vec::new(),
        recovery: BindingContextRecovery::Normal,
    }) != BindingContextId::new(0)
    {
        return Err("Task257B1 module context identity changed".to_owned());
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
