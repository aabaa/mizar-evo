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
        SourceAtomicFormulaHandoff, SourceAtomicFormulaHandoffInput, SourceAtomicFormulaId,
        SourceAtomicFormulaInput, SourceAtomicFormulaKind, SourceAtomicFormulaProducer,
        SourceAtomicFormulaRecovery, SourceAtomicRequestInput, SourceAtomicRequestKind,
        SourceAtomicTermTarget,
    },
    source_composite_formula::{
        SourceBinderTypeHead, SourceBinderTypeSiteId, SourceBinderTypeSiteInput,
        SourceCompositeFormulaHandoffInput, SourceCompositeFormulaId, SourceCompositeFormulaInput,
        SourceCompositeFormulaKind, SourceCompositeFormulaProducer, SourceCompositeFormulaRecovery,
        SourceFormulaEdgeInput, SourceFormulaEdgeRole, SourceFormulaRequestInput,
        SourceFormulaRequestKind, SourceFormulaRootInput, SourceFormulaRootOwnership,
        SourceFormulaWrapperInput, SourceQuantifierBinderId, SourceQuantifierBinderInput,
    },
    source_formula_composition::{
        SourceConditionFormulaCompositionHandoffInput, SourceConditionFormulaCompositionProducer,
        SourceConditionFormulaEdgeInput, SourceFormulaAtomicEdgeId, SourceFormulaAtomicEdgeInput,
        SourceFormulaAtomicEdgeRole, SourceFormulaCompositionHandoffInput,
        SourceFormulaCompositionProducer, SourcePredicateChainCompositionHandoffInput,
        SourcePredicateChainCompositionProducer, SourcePredicateChainConjunctionInput,
        SourcePredicateChainNegationInput, SourceQuantifierBoundUseInput,
    },
    source_set_term::SourceSetConditionId,
    source_term::{
        SourceNumericTypeRequestInput, SourcePrimaryTermHandoff, SourcePrimaryTermHandoffInput,
        SourcePrimaryTermId, SourcePrimaryTermInput, SourcePrimaryTermKind,
        SourcePrimaryTermProducer, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceId,
        SourcePrimaryTermReferenceInput, SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
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
use mizar_syntax::{SurfaceAst, SurfaceNodeKind};

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_atomic_formula::source_atomic_formula_output_with_source,
    source_formula::{
        SourceFormulaConnectiveGrouping, SourceFormulaNestedQuantifierPayload,
        SourceFormulaQuantifierBoundUse, extract_source_formula_connective_grouping,
        extract_source_formula_nested_quantifier_payload,
        extract_source_formula_quantifier_bound_use,
        extract_source_imported_predicate_chain_formula,
    },
    source_set_term::conditioned_source_set_term_output,
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";
const PAYLOAD_EXTRACTION_GAP_KEY: &str =
    "type_elaboration.external_dependency.ast_payload_extraction";

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

#[derive(Debug)]
pub(in crate::runner) struct SourceConditionFormulaCompositionRouteInputs {
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) atomic: SourceAtomicFormulaHandoffInput,
    pub(in crate::runner) composition: SourceConditionFormulaCompositionHandoffInput,
}

#[derive(Debug)]
pub(in crate::runner) struct SourcePredicateChainCompositionRouteInputs {
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: Option<SourcePrimaryTermHandoff>,
    pub(in crate::runner) atomic: Option<SourceAtomicFormulaHandoff>,
    pub(in crate::runner) composition: SourcePredicateChainCompositionHandoffInput,
}

pub(in crate::runner) fn source_formula_composition_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    if let Some(result) = source_predicate_chain_composition_output_with_mutation_impl(
        ast,
        module.clone(),
        symbols,
        source_text,
        |_| {},
    ) {
        return match result {
            Ok(output)
                if output.typed_ast.source_context().is_none()
                    && output.typed_ast.source_term().is_some()
                    && output.typed_ast.source_application().is_none()
                    && output.typed_ast.source_structure().is_none()
                    && output.typed_ast.source_set_term().is_none()
                    && output.typed_ast.source_atomic_formula().is_some()
                    && output.typed_ast.source_composite_formula().is_none()
                    && output.typed_ast.source_formula_composition().is_none()
                    && output
                        .typed_ast
                        .source_condition_formula_composition()
                        .is_none()
                    && output
                        .typed_ast
                        .source_predicate_chain_composition()
                        .is_some()
                    && output.typed_ast.source_predicate_chain_composition()
                        == output.resolved.source_predicate_chain_composition() =>
            {
                Some(Vec::new())
            }
            Ok(_) | Err(_) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
        };
    }
    if let Some(result) = source_condition_formula_composition_output_with_mutation_impl(
        ast,
        module.clone(),
        symbols,
        source_text,
        |_| {},
    ) {
        return match result {
            Ok(output)
                if output.typed_ast.source_context().is_none()
                    && output.typed_ast.source_term().is_some()
                    && output.typed_ast.source_application().is_some()
                    && output.typed_ast.source_set_term().is_some()
                    && output.typed_ast.source_atomic_formula().is_some()
                    && output
                        .typed_ast
                        .source_condition_formula_composition()
                        .is_some()
                    && output.typed_ast.source_condition_formula_composition()
                        == output.resolved.source_condition_formula_composition() =>
            {
                Some(vec![PAYLOAD_EXTRACTION_GAP_KEY.to_owned()])
            }
            Ok(_) | Err(_) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
        };
    }
    match source_formula_composition_output_with_source(ast, module, symbols, source_text) {
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
pub(in crate::runner) fn source_predicate_chain_composition_output_with_source(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_predicate_chain_composition_output_with_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        |_| {},
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_predicate_chain_composition_output_with_source_and_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourcePredicateChainCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_predicate_chain_composition_output_with_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_formula_composition_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_formula_composition_output_with_mutation(ast, module, symbols, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_formula_composition_output_with_source(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_formula_composition_output_with_mutation_impl(
        ast,
        module,
        symbols,
        Some(source_text),
        |_| {},
    )
}

#[cfg(not(test))]
pub(in crate::runner) fn source_formula_composition_output_with_source(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_formula_composition_output_with_mutation_impl(
        ast,
        module,
        symbols,
        Some(source_text),
        |_| {},
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_formula_composition_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_formula_composition_output_with_mutation_impl(ast, module, symbols, None, mutate)
}

#[cfg(test)]
pub(in crate::runner) fn source_formula_composition_output_with_source_and_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_formula_composition_output_with_mutation_impl(
        ast,
        module,
        symbols,
        Some(source_text),
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_condition_formula_composition_output_with_source(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_condition_formula_composition_output_with_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        |_| {},
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_condition_formula_composition_output_with_source_and_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceConditionFormulaCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    source_condition_formula_composition_output_with_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

fn source_predicate_chain_composition_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourcePredicateChainCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    extract_source_imported_predicate_chain_formula(ast, &module, symbols, source_text)?;
    let lower =
        source_atomic_formula_output_with_source(ast, module.clone(), symbols, source_text)?;
    Some(lower.and_then(|lower| build_task_257c3_output(ast, module, lower.typed_ast, mutate)))
}

fn build_task_257c3_output(
    ast: &SurfaceAst,
    module: ModuleId,
    lower: TypedAst,
    mutate: impl FnOnce(&mut SourcePredicateChainCompositionRouteInputs),
) -> Result<SourceFormulaCompositionRouteOutput, String> {
    let primary = lower
        .source_term()
        .cloned()
        .ok_or_else(|| "Task257C3 lost Task252 handoff".to_owned())?;
    let atomic = lower
        .source_atomic_formula()
        .cloned()
        .ok_or_else(|| "Task257C3 lost Task256 handoff".to_owned())?;
    let mut inputs =
        SourcePredicateChainCompositionRouteInputs {
            arena: lower.nodes().clone(),
            primary: Some(primary),
            atomic: Some(atomic),
            composition: SourcePredicateChainCompositionHandoffInput {
                source_id: ast.source_id,
                module_id: module.clone(),
                conjunctions: vec![SourcePredicateChainConjunctionInput {
                    formula: SourceAtomicFormulaId::new(0),
                    ordinal: 0,
                    left_segment:
                        mizar_checker::source_atomic_formula::SourcePredicateSegmentId::new(0),
                    right_segment:
                        mizar_checker::source_atomic_formula::SourcePredicateSegmentId::new(1),
                    boundary: SourceAtomicEdgeId::new(1),
                }],
                negations: vec![SourcePredicateChainNegationInput {
                    formula: SourceAtomicFormulaId::new(0),
                    ordinal: 0,
                    segment: mizar_checker::source_atomic_formula::SourcePredicateSegmentId::new(1),
                }],
            },
        };
    mutate(&mut inputs);
    let primary = inputs
        .primary
        .ok_or_else(|| "Task257C3 lost mutable Task252 dependency".to_owned())?;
    let atomic = inputs
        .atomic
        .ok_or_else(|| "Task257C3 lost mutable Task256 dependency".to_owned())?;
    let composition = SourcePredicateChainCompositionProducer::build(
        inputs.composition,
        &primary,
        &atomic,
        &inputs.arena,
    )
    .map_err(|error| error.to_string())?;

    let typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: inputs.arena,
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
    .with_source_predicate_chain_composition(composition)
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.formula.predicate-chain-composition"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_context().is_some()
        || typed_ast.source_application().is_some()
        || typed_ast.source_structure().is_some()
        || typed_ast.source_set_term().is_some()
        || typed_ast.source_composite_formula().is_some()
        || typed_ast.source_formula_composition().is_some()
        || typed_ast.source_condition_formula_composition().is_some()
        || typed_ast.source_predicate_chain_composition().is_none()
        || typed_ast.source_term() != resolved.source_term()
        || typed_ast.source_atomic_formula() != resolved.source_atomic_formula()
        || typed_ast.source_predicate_chain_composition()
            != resolved.source_predicate_chain_composition()
        || !typed_ast.contexts().is_empty()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.collection_candidates().is_empty()
        || !resolved.expanded_candidates().is_empty()
        || !resolved.template_expansions().is_empty()
        || !resolved.viable_candidates().is_empty()
        || !resolved.viability_decisions().is_empty()
        || !resolved.specificity_graphs().is_empty()
        || !resolved.resolved_overloads().is_empty()
        || !resolved.inserted_coercions().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
        || !resolved.checked_formulas().is_empty()
        || !resolved.statement_semantics().is_empty()
        || !resolved.checked_proofs().is_empty()
        || !resolved.checked_proof_nodes().is_empty()
        || !resolved.checked_terminal_goals().is_empty()
    {
        return Err("Task257C3 immutable final handoff mismatch".to_owned());
    }
    Ok(SourceFormulaCompositionRouteOutput {
        typed_ast,
        resolved,
    })
}

fn source_condition_formula_composition_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceConditionFormulaCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    let lower = conditioned_source_set_term_output(ast, module.clone(), symbols, source_text)?;
    Some(
        lower.and_then(|lower| {
            build_task_257c2_output(ast, module, symbols, lower.typed_ast, mutate)
        }),
    )
}

fn build_task_257c2_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    lower: TypedAst,
    mutate: impl FnOnce(&mut SourceConditionFormulaCompositionRouteInputs),
) -> Result<SourceFormulaCompositionRouteOutput, String> {
    if symbols.module_id() != &module {
        return Err("Task257C2 symbol module mismatch".to_owned());
    }
    let equality_nodes = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            matches!(node.kind, SurfaceNodeKind::BuiltinPredicateApplication)
                && node.range.start == 177
                && node.range.end == 182
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [equality_node] = equality_nodes.as_slice() else {
        return Err("Task257C2 requires one exact inner equality".to_owned());
    };
    let mut nodes = lower
        .nodes()
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let equality = nodes
        .get_mut(*equality_node)
        .ok_or_else(|| "Task257C2 equality arena node disappeared".to_owned())?;
    equality.kind = "source.formula.atomic.equality".into();
    equality.recovery = NodeRecoveryState::Normal;
    let arena =
        TypedArena::try_new(lower.nodes().root(), nodes).map_err(|error| error.to_string())?;

    let primary = lower
        .source_term()
        .cloned()
        .ok_or_else(|| "Task257C2 lost Task252 handoff".to_owned())?;
    let application = lower
        .source_application()
        .cloned()
        .ok_or_else(|| "Task257C2 lost Task253 handoff".to_owned())?;
    let set = lower
        .source_set_term()
        .cloned()
        .ok_or_else(|| "Task257C2 lost Task255 handoff".to_owned())?;
    let binding_env = super::checker_handoff::source_module_binding_env(ast, module.clone())
        .map_err(|error| error.to_string())?;
    let mut inputs = SourceConditionFormulaCompositionRouteInputs {
        arena,
        atomic: conditioned_equality_input(ast, module.clone(), *equality_node)?,
        composition: SourceConditionFormulaCompositionHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            edges: vec![SourceConditionFormulaEdgeInput {
                condition: SourceSetConditionId::new(0),
                ordinal: 0,
                formula: SourceAtomicFormulaId::new(0),
            }],
        },
    };
    mutate(&mut inputs);
    let atomic = SourceAtomicFormulaProducer::build(
        inputs.atomic,
        &binding_env,
        symbols,
        &primary,
        Some(&application),
        None,
        Some(&set),
        &inputs.arena,
    )
    .map_err(|error| error.to_string())?;
    let composition = SourceConditionFormulaCompositionProducer::build(
        inputs.composition,
        &primary,
        &application,
        &set,
        &atomic,
        &inputs.arena,
    )
    .map_err(|error| error.to_string())?;

    let typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: inputs.arena,
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
    .with_source_application(application)
    .map_err(|error| error.to_string())?
    .with_source_set_term(set)
    .map_err(|error| error.to_string())?
    .with_source_atomic_formula(atomic)
    .map_err(|error| error.to_string())?
    .with_source_condition_formula_composition(composition)
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.formula.condition-composition"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_context().is_some()
        || typed_ast.source_condition_formula_composition().is_none()
        || typed_ast.source_condition_formula_composition()
            != resolved.source_condition_formula_composition()
        || typed_ast.source_term() != resolved.source_term()
        || typed_ast.source_application() != resolved.source_application()
        || typed_ast.source_set_term() != resolved.source_set_term()
        || typed_ast.source_atomic_formula() != resolved.source_atomic_formula()
        || typed_ast.source_composite_formula().is_some()
        || typed_ast.source_formula_composition().is_some()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
    {
        return Err("Task257C2 immutable final handoff mismatch".to_owned());
    }
    Ok(SourceFormulaCompositionRouteOutput {
        typed_ast,
        resolved,
    })
}

fn conditioned_equality_input(
    ast: &SurfaceAst,
    module: ModuleId,
    equality_node: usize,
) -> Result<SourceAtomicFormulaHandoffInput, String> {
    let equality = ast
        .nodes()
        .get(equality_node)
        .ok_or_else(|| "Task257C2 equality node disappeared".to_owned())?;
    let operand_ranges = equality
        .children
        .iter()
        .filter_map(|child| {
            ast.node(*child).and_then(|node| {
                matches!(node.kind, SurfaceNodeKind::TermExpression).then_some(node.range)
            })
        })
        .collect::<Vec<_>>();
    let [left_range, right_range] = operand_ranges.as_slice() else {
        let children = equality
            .children
            .iter()
            .filter_map(|child| {
                ast.node(*child)
                    .map(|node| (child.index(), &node.kind, node.range))
            })
            .collect::<Vec<_>>();
        return Err(format!("Task257C2 equality operands changed: {children:?}"));
    };
    if (left_range.start, left_range.end) != (177, 178)
        || (right_range.start, right_range.end) != (181, 182)
    {
        return Err("Task257C2 equality operand ranges changed".to_owned());
    }
    let left = SourcePrimaryTermId::new(2);
    let right = SourcePrimaryTermId::new(3);
    Ok(SourceAtomicFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        formulas: vec![SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(TypedNodeId::new(equality_node)),
            source_range: equality.range,
            source_ordinal: 0,
            context: BindingContextId::new(0),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "3 = 4".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        }],
        wrappers: Vec::new(),
        predicate_segments: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges: vec![
            SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                target: SourceAtomicTermTarget::Primary(left),
            },
            SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 1,
                role: SourceAtomicEdgeRole::BuiltinRightOperand,
                target: SourceAtomicTermTarget::Primary(right),
            },
        ],
        requests: (0..2)
            .map(|index| SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: index,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(index)),
                candidate: None,
                type_site: None,
                attribute: None,
            })
            .collect(),
    })
}

fn source_formula_composition_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: Option<&str>,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Option<Result<SourceFormulaCompositionRouteOutput, String>> {
    if let Some(payload) = source_text.and_then(|source_text| {
        extract_source_formula_nested_quantifier_payload(ast, &module, symbols, source_text)
    }) {
        return Some(build_task_257b3_output(
            ast, module, symbols, payload, mutate,
        ));
    }
    if let Some(payload) = extract_source_formula_quantifier_bound_use(ast, &module, symbols) {
        return Some(build_task_257b1_output(
            ast, module, symbols, payload, mutate,
        ));
    }
    let payload = extract_source_formula_connective_grouping(ast, &module, symbols)?;
    Some(build_task_257b2_output(
        ast, module, symbols, payload, mutate,
    ))
}

fn build_task_257b1_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    payload: SourceFormulaQuantifierBoundUse,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Result<SourceFormulaCompositionRouteOutput, String> {
    let arena = typed_arena(ast, &payload)?;
    let inputs = route_inputs(ast, module.clone(), &payload);
    let base = module_shell(ast, module.clone())?;
    build_output(ast, module, symbols, arena, base, inputs, mutate)
}

fn build_task_257b2_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    payload: SourceFormulaConnectiveGrouping,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Result<SourceFormulaCompositionRouteOutput, String> {
    let arena = task_257b2_typed_arena(ast, &payload)?;
    let inputs = task_257b2_route_inputs(ast, module.clone(), &payload)?;
    let base = module_shell(ast, module.clone())?;
    build_output(ast, module, symbols, arena, base, inputs, mutate)
}

fn build_task_257b3_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    payload: SourceFormulaNestedQuantifierPayload,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Result<SourceFormulaCompositionRouteOutput, String> {
    let arena = task_257b3_typed_arena(ast, &payload)?;
    let base = payload
        .reserve
        .bridge
        .prepare_binding_env(symbols)
        .map_err(|error| format!("Task257B3 reserve base: {error}"))?;
    let inputs = task_257b3_route_inputs(ast, module.clone(), &payload)?;
    build_output(ast, module, symbols, arena, base, inputs, mutate)
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    arena: TypedArena,
    base: BindingEnv,
    mut inputs: SourceFormulaCompositionRouteInputs,
    mutate: impl FnOnce(&mut SourceFormulaCompositionRouteInputs),
) -> Result<SourceFormulaCompositionRouteOutput, String> {
    if symbols.module_id() != &module {
        return Err("Task257 formula-composition symbol module mismatch".to_owned());
    }
    mutate(&mut inputs);

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
        return Err("Task257 formula-composition immutable final handoff mismatch".to_owned());
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
        predicate_segments: Vec::new(),
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

fn task_257b2_route_inputs(
    ast: &SurfaceAst,
    module: ModuleId,
    payload: &SourceFormulaConnectiveGrouping,
) -> Result<SourceFormulaCompositionRouteInputs, String> {
    if payload.formula_sites.len() != 8
        || payload.formula_ranges.len() != 8
        || payload.wrapper_sites.len() != 6
        || payload.wrapper_ranges.len() != 6
        || payload.equality_sites.len() != 8
        || payload.equality_ranges.len() != 8
        || payload.numeral_sites.len() != 16
        || payload.numeral_ranges.len() != 16
        || payload.numeral_spellings.len() != 16
    {
        return Err("Task257B2 extracted aggregate changed".to_owned());
    }
    let context0 = BindingContextId::new(0);
    let context1 = BindingContextId::new(1);
    let formula_specs = [
        (SourceCompositeFormulaKind::Universal, "for holds"),
        (SourceCompositeFormulaKind::Biconditional, "iff"),
        (SourceCompositeFormulaKind::Disjunction, "or"),
        (SourceCompositeFormulaKind::RepeatedConjunction, "& ... &"),
        (SourceCompositeFormulaKind::RepeatedDisjunction, "or ... or"),
        (SourceCompositeFormulaKind::Disjunction, "or"),
        (SourceCompositeFormulaKind::Conjunction, "&"),
        (SourceCompositeFormulaKind::Disjunction, "or"),
    ];
    let formulas = formula_specs
        .into_iter()
        .enumerate()
        .map(|(index, (kind, spelling))| SourceCompositeFormulaInput {
            site: payload.formula_sites[index].clone(),
            source_range: payload.formula_ranges[index],
            source_ordinal: index,
            context: if index == 0 { context0 } else { context1 },
            recovery: SourceCompositeFormulaRecovery::Normal,
            spelling: spelling.to_owned(),
            kind,
        })
        .collect();
    let wrapper_owners = [2, 3, 4, 5, 6, 7];
    let wrapper_spellings = [
        "( or )",
        "( & ... & )",
        "( or ... or )",
        "( or )",
        "( & )",
        "( or )",
    ];
    let wrappers = wrapper_owners
        .into_iter()
        .zip(wrapper_spellings)
        .enumerate()
        .map(|(index, (formula, spelling))| SourceFormulaWrapperInput {
            formula: SourceCompositeFormulaId::new(formula),
            ordinal: 0,
            site: payload.wrapper_sites[index].clone(),
            source_range: payload.wrapper_ranges[index],
            context: context1,
            recovery: SourceCompositeFormulaRecovery::Normal,
            spelling: spelling.to_owned(),
        })
        .collect();
    let mut requests = vec![
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
    ];
    requests.extend((1..8).map(|formula| SourceFormulaRequestInput {
        formula: SourceCompositeFormulaId::new(formula),
        ordinal: 0,
        kind: SourceFormulaRequestKind::ConnectiveSemantics,
        binder: None,
        type_site: None,
    }));
    let composite = SourceCompositeFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        formulas,
        wrappers,
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
        edges: [
            (0, 0, SourceFormulaEdgeRole::UniversalBody, 1),
            (1, 0, SourceFormulaEdgeRole::BiconditionalLeft, 2),
            (1, 1, SourceFormulaEdgeRole::BiconditionalRight, 5),
            (2, 0, SourceFormulaEdgeRole::DisjunctionLeft, 3),
            (2, 1, SourceFormulaEdgeRole::DisjunctionRight, 4),
            (5, 0, SourceFormulaEdgeRole::DisjunctionLeft, 6),
            (5, 1, SourceFormulaEdgeRole::DisjunctionRight, 7),
        ]
        .into_iter()
        .map(|(parent, ordinal, role, child)| SourceFormulaEdgeInput {
            parent: SourceCompositeFormulaId::new(parent),
            ordinal,
            role,
            child: SourceCompositeFormulaId::new(child),
        })
        .collect(),
        requests,
    };

    let terms = (0..16)
        .map(|index| SourcePrimaryTermInput {
            site: payload.numeral_sites[index].clone(),
            source_range: payload.numeral_ranges[index],
            source_ordinal: index,
            context: context1,
            recovery: SourcePrimaryTermRecovery::Normal,
            spelling: payload.numeral_spellings[index].clone(),
            kind: SourcePrimaryTermKind::Numeral,
            role: SourcePrimaryTermRole::Value,
            parent: None,
        })
        .collect::<Vec<_>>();
    let numeric_type_requests = terms
        .iter()
        .enumerate()
        .map(|(index, term)| SourceNumericTypeRequestInput {
            term: SourcePrimaryTermId::new(index),
            owner: term.site.clone(),
            source_range: term.source_range,
            spelling: term.spelling.clone(),
            request_ordinal: index,
        })
        .collect();
    let primary = SourcePrimaryTermHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms,
        references: Vec::new(),
        numeric_type_requests,
    };

    let equality_spellings = [
        "0 = 0", "0 = 3", "0 = 0", "0 = 3", "0 = 0", "0 = 0", "0 = 0", "0 = 0",
    ];
    let atomic_formulas = equality_spellings
        .into_iter()
        .enumerate()
        .map(|(index, spelling)| SourceAtomicFormulaInput {
            site: payload.equality_sites[index].clone(),
            source_range: payload.equality_ranges[index],
            source_ordinal: index,
            context: context1,
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: spelling.to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        })
        .collect();
    let atomic_edges = (0..8)
        .flat_map(|formula| {
            [
                SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(formula),
                    ordinal: 0,
                    role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(formula * 2)),
                },
                SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(formula),
                    ordinal: 1,
                    role: SourceAtomicEdgeRole::BuiltinRightOperand,
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                        formula * 2 + 1,
                    )),
                },
            ]
        })
        .collect::<Vec<_>>();
    let atomic_requests = atomic_edges
        .iter()
        .enumerate()
        .map(|(index, edge)| SourceAtomicRequestInput {
            formula: edge.formula,
            ordinal: edge.ordinal,
            kind: SourceAtomicRequestKind::OperandExpectedType,
            edge: Some(SourceAtomicEdgeId::new(index)),
            candidate: None,
            type_site: None,
            attribute: None,
        })
        .collect();
    let atomic = SourceAtomicFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        formulas: atomic_formulas,
        wrappers: Vec::new(),
        predicate_segments: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges: atomic_edges,
        requests: atomic_requests,
    };
    let composition = SourceFormulaCompositionHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        atomic_edges: [
            (3, 0, SourceFormulaAtomicEdgeRole::ConjunctionLeft, 0),
            (3, 1, SourceFormulaAtomicEdgeRole::ConjunctionRight, 1),
            (4, 0, SourceFormulaAtomicEdgeRole::DisjunctionLeft, 2),
            (4, 1, SourceFormulaAtomicEdgeRole::DisjunctionRight, 3),
            (6, 0, SourceFormulaAtomicEdgeRole::ConjunctionLeft, 4),
            (6, 1, SourceFormulaAtomicEdgeRole::ConjunctionRight, 5),
            (7, 0, SourceFormulaAtomicEdgeRole::DisjunctionLeft, 6),
            (7, 1, SourceFormulaAtomicEdgeRole::DisjunctionRight, 7),
        ]
        .into_iter()
        .map(
            |(formula, ordinal, role, child)| SourceFormulaAtomicEdgeInput {
                formula: SourceCompositeFormulaId::new(formula),
                ordinal,
                role,
                child: SourceAtomicFormulaId::new(child),
            },
        )
        .collect(),
        bound_uses: Vec::new(),
    };
    Ok(SourceFormulaCompositionRouteInputs {
        primary,
        atomic,
        composite,
        composition,
    })
}

fn task_257b3_route_inputs(
    ast: &SurfaceAst,
    module: ModuleId,
    payload: &SourceFormulaNestedQuantifierPayload,
) -> Result<SourceFormulaCompositionRouteInputs, String> {
    if payload.formula_sites.len() != 3
        || payload.formula_ranges.len() != 3
        || payload.binder_segment_sites.len() != 3
        || payload.binder_segment_ranges.len() != 3
        || payload.binder_identifier_sites.len() != 3
        || payload.binder_identifier_ranges.len() != 3
        || payload.binder_type_sites.len() != 3
        || payload.binder_type_ranges.len() != 3
        || payload.binder_type_head_sites.len() != 3
        || payload.binder_type_head_ranges.len() != 3
        || payload.equality_sites.len() != 3
        || payload.equality_ranges.len() != 3
        || payload.term_sites.len() != 6
        || payload.term_ranges.len() != 6
        || payload.term_spellings.as_slice() != ["x", "x", "r", "y", "x", "r"]
    {
        return Err("Task257B3 extracted aggregate changed".to_owned());
    }
    let contexts = [
        BindingContextId::new(0),
        BindingContextId::new(1),
        BindingContextId::new(2),
        BindingContextId::new(3),
    ];
    let formulas = [
        (SourceCompositeFormulaKind::Universal, "for st"),
        (SourceCompositeFormulaKind::Existential, "ex st"),
        (SourceCompositeFormulaKind::Universal, "for st holds"),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (kind, spelling))| SourceCompositeFormulaInput {
        site: payload.formula_sites[index].clone(),
        source_range: payload.formula_ranges[index],
        source_ordinal: index,
        context: contexts[index],
        recovery: SourceCompositeFormulaRecovery::Normal,
        spelling: spelling.to_owned(),
        kind,
    })
    .collect();
    let binder_spellings = [("x being", "x"), ("y being", "y"), ("r", "r")];
    let binder_scopes = [vec![0], vec![0, 0], vec![0, 0, 0]];
    let binders = binder_spellings
        .into_iter()
        .zip(binder_scopes)
        .enumerate()
        .map(
            |(index, ((segment_spelling, identifier_spelling), scope))| {
                SourceQuantifierBinderInput {
                    formula: SourceCompositeFormulaId::new(index),
                    ordinal: 0,
                    segment_site: payload.binder_segment_sites[index].clone(),
                    segment_range: payload.binder_segment_ranges[index],
                    segment_spelling: segment_spelling.to_owned(),
                    identifier_site: payload.binder_identifier_sites[index].clone(),
                    identifier_range: payload.binder_identifier_ranges[index],
                    identifier_spelling: identifier_spelling.to_owned(),
                    local: LocalTermBinding::new(
                        identifier_spelling,
                        LocalTermScope::new(scope),
                        payload.binder_identifier_ranges[index],
                        index + 1,
                    ),
                    binding: BindingId::new(index + 1),
                    body_context: contexts[index + 1],
                    type_site: SourceBinderTypeSiteId::new(index),
                    recovery: SourceCompositeFormulaRecovery::Normal,
                }
            },
        )
        .collect();
    let type_sites = (0..3)
        .map(|index| SourceBinderTypeSiteInput {
            binder: SourceQuantifierBinderId::new(index),
            site: payload.binder_type_sites[index].clone(),
            source_range: payload.binder_type_ranges[index],
            spelling: "set".to_owned(),
            head_site: payload.binder_type_head_sites[index].clone(),
            head_range: payload.binder_type_head_ranges[index],
            head_spelling: "set".to_owned(),
            context: if index == 1 { contexts[1] } else { contexts[0] },
            recovery: SourceCompositeFormulaRecovery::Normal,
            head: SourceBinderTypeHead::BuiltinSet,
        })
        .collect();
    let requests = (0..3)
        .flat_map(|formula| {
            [
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(formula),
                    ordinal: 0,
                    kind: SourceFormulaRequestKind::QuantifierSemantics,
                    binder: None,
                    type_site: None,
                },
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(formula),
                    ordinal: 1,
                    kind: SourceFormulaRequestKind::BinderType,
                    binder: Some(SourceQuantifierBinderId::new(formula)),
                    type_site: Some(SourceBinderTypeSiteId::new(formula)),
                },
            ]
        })
        .collect();
    let composite = SourceCompositeFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        formulas,
        wrappers: Vec::new(),
        roots: vec![SourceFormulaRootInput {
            formula: SourceCompositeFormulaId::new(0),
            ordinal: 0,
            ownership: SourceFormulaRootOwnership::UnassignedStatement,
        }],
        binders,
        type_sites,
        edges: vec![
            SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                role: SourceFormulaEdgeRole::UniversalBody,
                child: SourceCompositeFormulaId::new(1),
            },
            SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(1),
                ordinal: 0,
                role: SourceFormulaEdgeRole::ExistentialBody,
                child: SourceCompositeFormulaId::new(2),
            },
        ],
        requests,
    };

    let term_contexts = [
        contexts[1],
        contexts[1],
        contexts[3],
        contexts[3],
        contexts[3],
        contexts[3],
    ];
    let terms = (0..6)
        .map(|index| SourcePrimaryTermInput {
            site: payload.term_sites[index].clone(),
            source_range: payload.term_ranges[index],
            source_ordinal: index,
            context: term_contexts[index],
            recovery: SourcePrimaryTermRecovery::Normal,
            spelling: payload.term_spellings[index].clone(),
            kind: SourcePrimaryTermKind::VariableReference,
            role: SourcePrimaryTermRole::Value,
            parent: None,
        })
        .collect();
    let reference_bindings = [1, 1, 3, 2, 1, 3];
    let references = reference_bindings
        .into_iter()
        .enumerate()
        .map(|(index, binding)| SourcePrimaryTermReferenceInput {
            term: SourcePrimaryTermId::new(index),
            binding: BindingId::new(binding),
            role: SourcePrimaryTermReferenceRole::Variable,
        })
        .collect();
    let primary = SourcePrimaryTermHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms,
        references,
        numeric_type_requests: Vec::new(),
    };

    let equality_spellings = ["x = x", "r = y", "x = r"];
    let equality_contexts = [contexts[1], contexts[3], contexts[3]];
    let atomic_formulas = equality_spellings
        .into_iter()
        .enumerate()
        .map(|(index, spelling)| SourceAtomicFormulaInput {
            site: payload.equality_sites[index].clone(),
            source_range: payload.equality_ranges[index],
            source_ordinal: index,
            context: equality_contexts[index],
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: spelling.to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        })
        .collect();
    let atomic_edges = (0..3)
        .flat_map(|formula| {
            [
                SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(formula),
                    ordinal: 0,
                    role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(formula * 2)),
                },
                SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(formula),
                    ordinal: 1,
                    role: SourceAtomicEdgeRole::BuiltinRightOperand,
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                        formula * 2 + 1,
                    )),
                },
            ]
        })
        .collect::<Vec<_>>();
    let atomic_requests = atomic_edges
        .iter()
        .enumerate()
        .map(|(index, edge)| SourceAtomicRequestInput {
            formula: edge.formula,
            ordinal: edge.ordinal,
            kind: SourceAtomicRequestKind::OperandExpectedType,
            edge: Some(SourceAtomicEdgeId::new(index)),
            candidate: None,
            type_site: None,
            attribute: None,
        })
        .collect();
    let atomic = SourceAtomicFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        formulas: atomic_formulas,
        wrappers: Vec::new(),
        predicate_segments: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges: atomic_edges,
        requests: atomic_requests,
    };
    let atomic_edges = [
        (0, 0, SourceFormulaAtomicEdgeRole::UniversalRestriction, 0),
        (2, 0, SourceFormulaAtomicEdgeRole::UniversalRestriction, 1),
        (2, 1, SourceFormulaAtomicEdgeRole::UniversalBody, 2),
    ]
    .into_iter()
    .map(
        |(formula, ordinal, role, child)| SourceFormulaAtomicEdgeInput {
            formula: SourceCompositeFormulaId::new(formula),
            ordinal,
            role,
            child: SourceAtomicFormulaId::new(child),
        },
    )
    .collect();
    let binder_ids = [0, 0, 2, 1, 0, 2];
    let binder_ordinals = [0, 1, 0, 0, 2, 1];
    let owning_edges = [0, 0, 1, 1, 2, 2];
    let bound_uses = (0..6)
        .map(|index| SourceQuantifierBoundUseInput {
            binder: SourceQuantifierBinderId::new(binder_ids[index]),
            ordinal: binder_ordinals[index],
            body_edge: SourceFormulaAtomicEdgeId::new(owning_edges[index]),
            term: SourcePrimaryTermId::new(index),
            reference: SourcePrimaryTermReferenceId::new(index),
        })
        .collect();
    let composition = SourceFormulaCompositionHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        atomic_edges,
        bound_uses,
    };
    Ok(SourceFormulaCompositionRouteInputs {
        primary,
        atomic,
        composite,
        composition,
    })
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

fn task_257b2_typed_arena(
    ast: &SurfaceAst,
    payload: &SourceFormulaConnectiveGrouping,
) -> Result<TypedArena, String> {
    let formula_kinds = [
        "source.formula.composite.universal",
        "source.formula.composite.biconditional",
        "source.formula.composite.disjunction",
        "source.formula.composite.repeated-conjunction",
        "source.formula.composite.repeated-disjunction",
        "source.formula.composite.disjunction",
        "source.formula.composite.conjunction",
        "source.formula.composite.disjunction",
    ];
    let mut kinds = BTreeMap::new();
    for (site, kind) in payload.formula_sites.iter().zip(formula_kinds) {
        kinds.insert(site.node().index(), kind);
    }
    for site in &payload.wrapper_sites {
        kinds.insert(site.node().index(), "source.formula.parenthesized");
    }
    for (site, kind) in [
        (
            &payload.binder_segment_site,
            "source.formula.quantifier-binder",
        ),
        (
            &payload.binder_identifier_site,
            "source.formula.quantifier-binder",
        ),
        (&payload.binder_type_site, "source.formula.binder-type"),
        (
            &payload.binder_type_head_site,
            "source.formula.binder-type-head",
        ),
    ] {
        kinds.insert(site.node().index(), kind);
    }
    for site in &payload.equality_sites {
        kinds.insert(site.node().index(), "source.formula.atomic.equality");
    }
    for site in &payload.numeral_sites {
        kinds.insert(site.node().index(), "source.term.numeral");
    }
    if kinds.len() != 42 {
        return Err("Task257B2 source roles alias typed sites".to_owned());
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
            return Err("Task257B2 typed-arena identity changed".to_owned());
        }
    }
    if !kinds.is_empty() {
        return Err("Task257B2 source site disappeared".to_owned());
    }
    builder.finish(None).map_err(|error| error.to_string())
}

fn task_257b3_typed_arena(
    ast: &SurfaceAst,
    payload: &SourceFormulaNestedQuantifierPayload,
) -> Result<TypedArena, String> {
    let mut kinds = BTreeMap::new();
    for (site, kind) in payload.formula_sites.iter().zip([
        "source.formula.composite.universal",
        "source.formula.composite.existential",
        "source.formula.composite.universal",
    ]) {
        kinds.insert(site.node().index(), kind);
    }
    for site in payload
        .binder_segment_sites
        .iter()
        .chain(&payload.binder_identifier_sites)
    {
        kinds.insert(site.node().index(), "source.formula.quantifier-binder");
    }
    for site in &payload.binder_type_sites {
        kinds.insert(site.node().index(), "source.formula.binder-type");
    }
    for site in &payload.binder_type_head_sites {
        kinds.insert(site.node().index(), "source.formula.binder-type-head");
    }
    for site in &payload.equality_sites {
        kinds.insert(site.node().index(), "source.formula.atomic.equality");
    }
    for site in &payload.term_sites {
        kinds.insert(site.node().index(), "source.term.variable-reference");
    }
    if kinds.len() != 24 {
        return Err("Task257B3 source roles alias typed sites".to_owned());
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
            return Err("Task257B3 typed-arena identity changed".to_owned());
        }
    }
    if !kinds.is_empty() {
        return Err("Task257B3 source site disappeared".to_owned());
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
        return Err("Task257 formula-composition module context identity changed".to_owned());
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
