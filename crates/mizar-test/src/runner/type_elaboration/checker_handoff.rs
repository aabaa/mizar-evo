use std::collections::BTreeMap;

use mizar_checker::binding_env::{
    BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
    BindingContextOwner, BindingContextRecovery, BindingContextTable, BindingDiagnosticTable,
    BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingStatus, BindingTable,
    BindingTypeSite,
};
use mizar_checker::cluster_trace::ClusterFactTable;
use mizar_checker::overload_resolution::{
    CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
    OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
    OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
    TemplateExpansionOutput,
};
use mizar_checker::resolved_typed_ast::{
    ExprId, ExpressionMetadataInput, ResolvedNodeKindHint, ResolvedNodeKindHintKind,
    ResolvedTypedAst, ResolvedTypedAstInputs, ResolvedTypedNodeId, ResolvedTypedNodeKind,
    SourceNodeRole, StatementSemanticInput, StatementSemanticInputs,
};
use mizar_checker::type_checker::{
    CheckedStatementOwner, DeclarationCheckingOutput, DeclarationKind, DeclarationStatus,
    FormulaInput, FormulaKind, ModeExpansion, SourceReserveDeclarationBridge, TermFormulaChecker,
    TermFormulaInferenceOutput,
};
use mizar_checker::typed_ast::{
    CoercionTable, InitialObligationTable, LocalTypeContextId, LocalTypeContextTable,
    NodeRecoveryState, TypeDiagnosticTable, TypeEntryId, TypeFactTable, TypeStatus, TypeTable,
    TypedArenaBuilder, TypedAst, TypedAstParts, TypedNode, TypedNodeId, TypedNodeLinks,
    TypedSiteRef, TypingState,
};
use mizar_core::{
    binder_normalization::{NormalizedVarClass, NormalizedVarSort},
    core_ir::{CoreSourceRef, CoreVarId, CoreVarRole},
    elaborator::{
        CheckerOwnedProvenance, CoreBinderSeed, CoreContextInput, CoreVariableSeed,
        ResolvedTypedAstSummary, prepare_core_context,
    },
};
use mizar_resolve::env::{NamespacePath, SymbolEnv, SymbolKind};
use mizar_resolve::resolved_ast::{ModuleId as ResolverModuleId, SymbolId as ResolverSymbolId};
use mizar_session::SourceAnchor;
use mizar_syntax::SurfaceAst;

use super::source_formula::{SourceFormulaStatement, extract_source_contradiction_formula};
#[cfg(test)]
use super::source_reserve::SourceReserveExtraction;

pub(in crate::runner) fn source_module_binding_env(
    ast: &SurfaceAst,
    module: ResolverModuleId,
) -> Result<BindingEnv, mizar_checker::binding_env::BindingEnvError> {
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
    debug_assert_eq!(context, BindingContextId::new(0));
    BindingEnv::try_new(BindingEnvParts {
        source_id: ast.source_id,
        module_id: module,
        contexts,
        bindings: BindingTable::new(),
        diagnostics: BindingDiagnosticTable::new(),
    })
}

const CONTRADICTION_FORMULA_NODE: TypedNodeId = TypedNodeId::new(0);
const CONTRADICTION_THEOREM_NODE: TypedNodeId = TypedNodeId::new(1);
const CONTRADICTION_ROOT_NODE: TypedNodeId = TypedNodeId::new(2);
const CONTRADICTION_OWNER: &str = "SourceDerivedContradictionConstantBoundary";

#[derive(Debug)]
pub(in crate::runner) struct SourceContradictionHandoff {
    pub(in crate::runner) owner: CheckedStatementOwner,
    pub(in crate::runner) term_formula: TermFormulaInferenceOutput,
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

pub(in crate::runner) fn assemble_source_contradiction_checker_handoff(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceContradictionHandoff, String> {
    let payload = extract_source_contradiction_formula(ast)
        .ok_or_else(|| "missing exact source contradiction formula".to_owned())?;
    if ast
        .node(payload.owner_site)
        .is_none_or(|node| node.range != payload.owner_range)
        || ast
            .root()
            .and_then(|root| ast.node(root))
            .is_none_or(|root| root.range != payload.root_range)
    {
        return Err("source contradiction owner/root provenance mismatch".to_owned());
    }

    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, CONTRADICTION_OWNER)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return Err("source contradiction requires exactly one resolver theorem owner".to_owned());
    };
    let owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner.symbol().clone(),
        ast.source_id,
        &module,
    )
    .map_err(|error| error.to_string())?;

    let binding_env =
        source_module_binding_env(ast, module.clone()).map_err(|error| error.to_string())?;
    let typed_ast = assemble_source_contradiction_typed_ast(ast, &module, &payload)?;
    let formula = FormulaInput::new(
        payload.formula_site,
        BindingContextId::new(0),
        payload.formula_range,
        FormulaKind::Contradiction,
    )
    .with_recovery(payload.formula_recovery);
    let term_formula = TermFormulaChecker::default()
        .try_infer(symbols, &binding_env, [], [formula])
        .map_err(|error| error.to_string())?;
    let formula = term_formula
        .formulas()
        .iter()
        .next()
        .map(|(id, _)| id)
        .ok_or_else(|| "source contradiction checker produced no formula".to_owned())?;
    let resolved = assemble_source_contradiction_resolved_typed_ast(
        &typed_ast,
        &owner,
        &binding_env,
        &term_formula,
        vec![StatementSemanticInput {
            owner: owner.symbol().clone(),
            owner_node: CONTRADICTION_THEOREM_NODE,
            formula,
            formula_node: CONTRADICTION_FORMULA_NODE,
        }],
    )?;

    Ok(SourceContradictionHandoff {
        owner,
        term_formula,
        typed_ast,
        resolved,
    })
}

pub(in crate::runner) fn assert_source_contradiction_handoff(
    handoff: &SourceContradictionHandoff,
) -> Result<(), String> {
    if handoff.typed_ast.nodes().len() != 3
        || handoff.resolved.nodes().len() != 3
        || handoff.resolved.statement_semantics().len() != 1
        || handoff.resolved.checked_formulas() != handoff.term_formula.formulas()
    {
        return Err("source contradiction final handoff count mismatch".to_owned());
    }
    let statement = handoff
        .resolved
        .statement_semantics()
        .get(mizar_checker::resolved_typed_ast::StatementSemanticId::new(
            0,
        ))
        .ok_or_else(|| "missing source contradiction final statement".to_owned())?;
    if statement.owner != *handoff.owner.symbol()
        || statement.owner_origin != *handoff.owner.origin()
        || handoff
            .typed_ast
            .nodes()
            .node(statement.owner_node)
            .is_none()
        || handoff
            .typed_ast
            .nodes()
            .node(statement.formula_node)
            .is_none()
        || handoff
            .resolved
            .checked_formulas()
            .get(statement.formula)
            .is_none()
    {
        return Err("source contradiction final identity mismatch".to_owned());
    }
    Ok(())
}

fn assemble_source_contradiction_typed_ast(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    payload: &SourceFormulaStatement,
) -> Result<TypedAst, String> {
    let mut builder = TypedArenaBuilder::new();
    let formula = builder
        .push(
            TypedNode::new(
                "source.formula.contradiction",
                SourceAnchor::Range(payload.formula_range),
            )
            .with_recovery(payload.formula_recovery)
            .with_typing(TypingState::Successful),
        )
        .map_err(|error| error.to_string())?;
    if formula != CONTRADICTION_FORMULA_NODE {
        return Err("source contradiction formula node id mismatch".to_owned());
    }
    let theorem = builder
        .push(
            TypedNode::new(
                "source.statement.theorem",
                SourceAnchor::Range(payload.owner_range),
            )
            .with_children(vec![formula])
            .with_recovery(NodeRecoveryState::Normal)
            .with_typing(TypingState::Successful),
        )
        .map_err(|error| error.to_string())?;
    if theorem != CONTRADICTION_THEOREM_NODE {
        return Err("source contradiction theorem node id mismatch".to_owned());
    }
    let root = builder
        .push(
            TypedNode::new("source.module", SourceAnchor::Range(payload.root_range))
                .with_children(vec![theorem])
                .with_recovery(NodeRecoveryState::Normal)
                .with_typing(TypingState::Successful),
        )
        .map_err(|error| error.to_string())?;
    if root != CONTRADICTION_ROOT_NODE {
        return Err("source contradiction root node id mismatch".to_owned());
    }

    let nodes = builder
        .finish(Some(root))
        .map_err(|error| error.to_string())?;
    TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module.clone(),
        resolved_root: None,
        nodes,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}

fn assemble_source_contradiction_resolved_typed_ast(
    typed_ast: &TypedAst,
    owner: &CheckedStatementOwner,
    binding_env: &BindingEnv,
    term_formula: &TermFormulaInferenceOutput,
    rows: Vec<StatementSemanticInput>,
) -> Result<ResolvedTypedAst, String> {
    let cluster_facts = ClusterFactTable::new();
    let overload_collection = OverloadCollectionOutput::collect(
        Vec::<OverloadSiteInput>::new(),
        Vec::<OverloadCandidateInput>::new(),
    );
    let template_expansion = TemplateExpansionOutput::expand(&overload_collection);
    let viability = CandidateViabilityOutput::filter(
        &template_expansion,
        Vec::<CandidateViabilityInput>::new(),
    );
    let specificity =
        SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
    let overload_selection =
        OverloadSelectionOutput::resolve(&specificity, Vec::<OverloadSiteResolutionInput>::new());
    let node_hints = [
        (CONTRADICTION_FORMULA_NODE, "source.formula.contradiction"),
        (CONTRADICTION_THEOREM_NODE, "source.statement.theorem"),
        (CONTRADICTION_ROOT_NODE, "source.module"),
    ]
    .into_iter()
    .map(|(typed_node, role)| ResolvedNodeKindHint {
        typed_node,
        kind: ResolvedNodeKindHintKind::SourcePreserved {
            role: SourceNodeRole::new(role),
        },
    })
    .collect();

    ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
        typed_ast,
        cluster_facts: &cluster_facts,
        overload_collection: &overload_collection,
        template_expansion: &template_expansion,
        viability: &viability,
        specificity: &specificity,
        overload_selection: &overload_selection,
        expressions: Vec::new(),
        node_hints,
        statement_semantics: Some(StatementSemanticInputs {
            owner,
            binding_env,
            term_formula,
            rows,
        }),
    })
    .map_err(|error| error.to_string())
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(in crate::runner) enum SourceContradictionHandoffCorruption {
    MissingRow,
    DuplicateRow,
    InvalidFormula,
    WrongOwnerNode,
}

#[cfg(test)]
pub(in crate::runner) fn source_contradiction_handoff_corruption_error(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
    corruption: SourceContradictionHandoffCorruption,
) -> Result<String, String> {
    let handoff = assemble_source_contradiction_checker_handoff(ast, module.clone(), symbols)?;
    let binding_env = source_module_binding_env(ast, module).map_err(|error| error.to_string())?;
    let statement = handoff
        .resolved
        .statement_semantics()
        .get(mizar_checker::resolved_typed_ast::StatementSemanticId::new(
            0,
        ))
        .ok_or_else(|| "missing source contradiction statement for corruption".to_owned())?;
    let valid = StatementSemanticInput {
        owner: statement.owner.clone(),
        owner_node: statement.owner_node,
        formula: statement.formula,
        formula_node: statement.formula_node,
    };
    let rows = match corruption {
        SourceContradictionHandoffCorruption::MissingRow => Vec::new(),
        SourceContradictionHandoffCorruption::DuplicateRow => vec![valid.clone(), valid],
        SourceContradictionHandoffCorruption::InvalidFormula => vec![StatementSemanticInput {
            formula: mizar_checker::type_checker::CheckedFormulaId::new(1),
            ..valid
        }],
        SourceContradictionHandoffCorruption::WrongOwnerNode => vec![StatementSemanticInput {
            owner_node: CONTRADICTION_FORMULA_NODE,
            ..valid
        }],
    };
    match assemble_source_contradiction_resolved_typed_ast(
        &handoff.typed_ast,
        &handoff.owner,
        &binding_env,
        &handoff.term_formula,
        rows,
    ) {
        Ok(_) => Err("source contradiction corruption unexpectedly assembled".to_owned()),
        Err(error) => Ok(error),
    }
}

#[derive(Debug)]
pub(in crate::runner) struct SourceReserveHandoff {
    pub(in crate::runner) binding_env: BindingEnv,
    pub(in crate::runner) declarations: DeclarationCheckingOutput,
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

pub(in crate::runner) fn assemble_source_reserve_checker_handoff(
    symbols: &SymbolEnv,
    source_reserve: &SourceReserveDeclarationBridge,
    mode_expansions: BTreeMap<ResolverSymbolId, ModeExpansion>,
) -> Result<SourceReserveHandoff, String> {
    let (binding_env, declarations) = source_reserve
        .check_with_mode_expansions(symbols, mode_expansions)
        .map_err(|error| error.to_string())?
        .into_parts();
    let typed_ast = assemble_source_reserve_typed_ast(source_reserve, &declarations)?;
    let resolved = assemble_source_reserve_resolved_typed_ast(&typed_ast, source_reserve)
        .map_err(|error| error.to_string())?;

    Ok(SourceReserveHandoff {
        binding_env,
        declarations,
        typed_ast,
        resolved,
    })
}

fn assemble_source_reserve_resolved_typed_ast(
    typed_ast: &TypedAst,
    source_reserve: &SourceReserveDeclarationBridge,
) -> Result<ResolvedTypedAst, String> {
    let cluster_facts = ClusterFactTable::new();
    let overload_collection = OverloadCollectionOutput::collect(
        Vec::<OverloadSiteInput>::new(),
        Vec::<OverloadCandidateInput>::new(),
    );
    let template_expansion = TemplateExpansionOutput::expand(&overload_collection);
    let viability = CandidateViabilityOutput::filter(
        &template_expansion,
        Vec::<CandidateViabilityInput>::new(),
    );
    let specificity =
        SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
    let overload_selection =
        OverloadSelectionOutput::resolve(&specificity, Vec::<OverloadSiteResolutionInput>::new());
    let expressions = source_reserve
        .bindings()
        .iter()
        .enumerate()
        .map(|(index, _)| ExpressionMetadataInput {
            expr: ExprId::new(format!("source.reserve.declaration.{index}")),
            typed_site: TypedSiteRef::Node(source_reserve.declaration_node(index)),
            local_context: Some(LocalTypeContextId::new(0)),
            cluster_facts: Vec::new(),
        })
        .collect();
    let mut node_hints = Vec::new();
    for (index, _) in source_reserve.bindings().iter().enumerate() {
        node_hints.push(ResolvedNodeKindHint {
            typed_node: source_reserve.type_node(index),
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.reserve.type_expression"),
            },
        });
        node_hints.push(ResolvedNodeKindHint {
            typed_node: source_reserve.declaration_node(index),
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.reserve.declaration"),
            },
        });
    }
    node_hints.push(ResolvedNodeKindHint {
        typed_node: source_reserve.root_node(),
        kind: ResolvedNodeKindHintKind::SourcePreserved {
            role: SourceNodeRole::new("source.reserve.module"),
        },
    });

    ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
        typed_ast,
        cluster_facts: &cluster_facts,
        overload_collection: &overload_collection,
        template_expansion: &template_expansion,
        viability: &viability,
        specificity: &specificity,
        overload_selection: &overload_selection,
        expressions,
        node_hints,
        statement_semantics: None,
    })
    .map_err(|error| error.to_string())
}

pub(in crate::runner) fn assert_source_reserve_handoff(
    handoff: &SourceReserveHandoff,
    source_reserve: &SourceReserveDeclarationBridge,
) -> Result<(), String> {
    let expected_bindings = source_reserve.bindings().len();
    let expected_nodes = expected_bindings * 2 + 1;
    if handoff.resolved.nodes().len() != expected_nodes
        || handoff.resolved.expr_metadata().len() != expected_bindings
        || handoff.declarations.declarations().len() != expected_bindings
    {
        return Err("resolved source reserve count mismatch".to_owned());
    }
    let module_context = handoff
        .binding_env
        .contexts()
        .get(source_reserve.module_context())
        .ok_or_else(|| "missing source reserve module binding context".to_owned())?;
    let expected_binding_ids = (0..expected_bindings)
        .map(BindingId::new)
        .collect::<Vec<_>>();
    if module_context.bindings != expected_binding_ids
        || module_context.visible_bindings != expected_binding_ids
    {
        return Err("source reserve module binding context mismatch".to_owned());
    }
    if handoff.declarations.contexts().len() != 1
        || handoff
            .declarations
            .contexts()
            .get(LocalTypeContextId::new(0))
            .is_none()
    {
        return Err("source reserve local context missing".to_owned());
    }

    for (index, source_binding) in source_reserve.bindings().iter().enumerate() {
        let binding = handoff
            .binding_env
            .bindings()
            .get(BindingId::new(index))
            .ok_or_else(|| format!("missing source reserve binding {index}"))?;
        if binding.spelling != source_binding.spelling
            || binding.kind != BindingKind::ReservedVariable
            || binding.owner_context != source_reserve.module_context()
            || binding.declaration_range != source_binding.binding_range
            || binding.visible_after_ordinal != index
            || binding.type_site != BindingTypeSite::Source(source_binding.type_range)
            || binding.status != BindingStatus::Reserved
        {
            return Err(format!("source reserve binding {index} metadata mismatch"));
        }
        match &binding.identity {
            BinderIdentity::ReservedVariable {
                spelling,
                declaration_range,
            } if spelling == &source_binding.spelling
                && *declaration_range == source_binding.binding_range => {}
            _ => {
                return Err(format!("source reserve binding {index} identity mismatch"));
            }
        }

        let type_node_id = source_reserve.type_node(index);
        let declaration_node_id = source_reserve.declaration_node(index);
        let type_node = handoff
            .resolved
            .nodes()
            .node(ResolvedTypedNodeId::new(type_node_id.index()))
            .ok_or_else(|| format!("missing resolved type node {index}"))?;
        if type_node.source_range != source_binding.type_range {
            return Err(format!("resolved type node {index} source range mismatch"));
        }
        match &type_node.kind {
            ResolvedTypedNodeKind::SourcePreserved { role }
                if role.as_str() == "source.reserve.type_expression" => {}
            _ => return Err(format!("resolved type node {index} source role mismatch")),
        }
        if type_node.final_type.is_none() {
            return Err(format!(
                "resolved type node {index} is missing a final type"
            ));
        }

        let declaration = handoff
            .declarations
            .declarations()
            .iter()
            .map(|(_, declaration)| declaration)
            .find(|declaration| declaration.binding == BindingId::new(index))
            .ok_or_else(|| format!("missing checked declaration {index}"))?;
        if declaration.site != TypedSiteRef::Node(declaration_node_id)
            || declaration.type_site != Some(TypedSiteRef::Node(type_node_id))
            || declaration.type_entry.is_none()
            || declaration.kind != DeclarationKind::ReservedVariable
            || declaration.status != DeclarationStatus::Checked
            || !declaration.deferred.is_empty()
        {
            return Err(format!("checked declaration {index} site mismatch"));
        }
        let typed_declaration = handoff
            .typed_ast
            .nodes()
            .node(declaration_node_id)
            .ok_or_else(|| format!("missing typed declaration node {index}"))?;
        if typed_declaration.links.type_entry != declaration.type_entry
            || typed_declaration.links.context != Some(LocalTypeContextId::new(0))
        {
            return Err(format!("typed declaration node {index} links mismatch"));
        }
        let declaration_node = handoff
            .resolved
            .nodes()
            .node(ResolvedTypedNodeId::new(declaration_node_id.index()))
            .ok_or_else(|| format!("missing resolved declaration node {index}"))?;
        if declaration_node.source_range != source_binding.binding_range {
            return Err(format!(
                "resolved declaration node {index} source range mismatch"
            ));
        }
        match &declaration_node.kind {
            ResolvedTypedNodeKind::SourcePreserved { role }
                if role.as_str() == "source.reserve.declaration" => {}
            _ => return Err(format!("resolved declaration node {index} role mismatch")),
        }
        if declaration_node.final_type.is_none() {
            return Err(format!(
                "resolved declaration node {index} is missing a final type"
            ));
        }
        let expr = ExprId::new(format!("source.reserve.declaration.{index}"));
        let metadata = handoff
            .resolved
            .expr_metadata()
            .get_by_expr(&expr)
            .ok_or_else(|| format!("missing expression metadata {}", expr.as_str()))?;
        if metadata.source_range != source_binding.binding_range {
            return Err(format!(
                "expression metadata {} source range mismatch",
                expr.as_str()
            ));
        }
        if metadata.final_type.is_none() {
            return Err(format!(
                "expression metadata {} is missing a final type",
                expr.as_str()
            ));
        }
    }
    if !handoff.resolved.diagnostics().is_empty() {
        return Err("resolved typed AST produced diagnostics".to_owned());
    }
    Ok(())
}

pub(in crate::runner) fn assert_source_reserve_core_summary_readiness(
    handoff: &SourceReserveHandoff,
) -> Result<(), String> {
    let summary = ResolvedTypedAstSummary::from_ast(&handoff.resolved);
    if summary.source_id() != handoff.resolved.source_id() {
        return Err("resolved typed AST summary source mismatch".to_owned());
    }
    if summary.module_id() != handoff.resolved.module_id() {
        return Err("resolved typed AST summary module mismatch".to_owned());
    }
    if !summary.checker_sites().is_empty() {
        return Err("resolved typed AST summary produced checker sites".to_owned());
    }
    Ok(())
}

pub(in crate::runner) fn assert_source_reserve_core_context_readiness(
    handoff: &SourceReserveHandoff,
    source_reserve: &SourceReserveDeclarationBridge,
) -> Result<(), String> {
    let summary = ResolvedTypedAstSummary::from_ast(&handoff.resolved);
    let mut input = CoreContextInput::new(summary);

    for (index, source_binding) in source_reserve.bindings().iter().enumerate() {
        let binding_id = BindingId::new(index);
        let binding = handoff
            .binding_env
            .bindings()
            .get(binding_id)
            .ok_or_else(|| format!("missing source reserve binding {index}"))?;
        if binding.kind != BindingKind::ReservedVariable
            || binding.declaration_range != source_binding.binding_range
            || binding.status != BindingStatus::Reserved
        {
            return Err(format!("source reserve binding {index} is not core-ready"));
        }

        let var = CoreVarId::new(binding_id.index());
        let provenance = CheckerOwnedProvenance::checker(format!("source.reserve.binding.{index}"));
        let source = CoreSourceRef::direct(binding.declaration_range)
            .with_provenance(provenance.as_slice().to_vec());
        input.variable_seeds.push(CoreVariableSeed::new(
            var,
            NormalizedVarClass::Free,
            CoreVarRole::new("reserved-variable"),
            NormalizedVarSort::Term,
            provenance.clone(),
        ));
        input
            .binder_seeds
            .push(CoreBinderSeed::new(var, source, provenance));
    }

    let context = prepare_core_context(input).map_err(|error| error.to_string())?;
    if context.source_id() != handoff.resolved.source_id() {
        return Err("core context source mismatch".to_owned());
    }
    if context.module_id() != handoff.resolved.module_id() {
        return Err("core context module mismatch".to_owned());
    }
    if !context.item_registry().items().is_empty()
        || !context.diagnostics().is_empty()
        || !context.worklist().entries().is_empty()
    {
        return Err("core context promoted unsupported work".to_owned());
    }
    if context.binder_sources().iter().count() != source_reserve.bindings().len()
        || context.binder_context().free_variables.len() != source_reserve.bindings().len()
    {
        return Err("core context binding count mismatch".to_owned());
    }

    for (index, source_binding) in source_reserve.bindings().iter().enumerate() {
        let var = CoreVarId::new(index);
        let binder_source = context
            .binder_sources()
            .get(var)
            .ok_or_else(|| format!("missing core binder source {index}"))?;
        if binder_source.source.anchor != CoreSourceRef::direct(source_binding.binding_range).anchor
        {
            return Err(format!("core binder source {index} range mismatch"));
        }
        if binder_source.provenance.as_slice().is_empty() {
            return Err(format!("core binder source {index} provenance missing"));
        }
        if context.binder_context().variable_roles.get(&var)
            != Some(&CoreVarRole::new("reserved-variable"))
            || context.binder_context().variable_sorts.get(&var) != Some(&NormalizedVarSort::Term)
            || !matches!(context.binder_type_facts().get(&var), Some(facts) if facts.is_empty())
        {
            return Err(format!("core binder {index} metadata mismatch"));
        }
    }

    Ok(())
}

#[cfg(test)]
pub(in crate::runner) fn assemble_source_checker_handoff(
    symbols: &SymbolEnv,
    source_reserve: &SourceReserveExtraction,
) -> Result<SourceReserveHandoff, String> {
    let handoff = assemble_source_reserve_checker_handoff(
        symbols,
        &source_reserve.bridge,
        source_reserve.mode_expansions.clone(),
    )?;
    assert_source_reserve_handoff(&handoff, &source_reserve.bridge)?;
    assert_source_reserve_core_summary_readiness(&handoff)?;
    assert_source_reserve_core_context_readiness(&handoff, &source_reserve.bridge)?;
    Ok(handoff)
}

fn assemble_source_reserve_typed_ast(
    source_reserve: &SourceReserveDeclarationBridge,
    output: &DeclarationCheckingOutput,
) -> Result<TypedAst, String> {
    if source_reserve.bindings().is_empty() {
        return Err("source reserve bridge produced no bindings".to_owned());
    }
    let declarations_by_binding = output
        .declarations()
        .iter()
        .map(|(_, declaration)| (declaration.binding, declaration))
        .collect::<BTreeMap<_, _>>();
    let mut builder = TypedArenaBuilder::new();
    let mut declaration_nodes = Vec::new();
    for (index, source_binding) in source_reserve.bindings().iter().enumerate() {
        let type_node_id = source_reserve.type_node(index);
        let type_site = TypedSiteRef::Node(type_node_id);
        let type_entry = type_entry_for_site(output.type_entries(), &type_site);
        let pushed = builder
            .push(
                TypedNode::new(
                    "source.reserve.type_expression",
                    SourceAnchor::Range(source_binding.type_range),
                )
                .with_recovery(NodeRecoveryState::Normal)
                .with_typing(typing_for_type_entry(output.type_entries(), type_entry))
                .with_links(TypedNodeLinks {
                    type_entry,
                    ..TypedNodeLinks::default()
                }),
            )
            .map_err(|error| error.to_string())?;
        if pushed != type_node_id {
            return Err(format!("source reserve type node {index} id mismatch"));
        }

        let declaration = declarations_by_binding
            .get(&BindingId::new(index))
            .ok_or_else(|| format!("missing checked source reserve declaration {index}"))?;
        let declaration_node_id = source_reserve.declaration_node(index);
        let declaration_type_entry = declaration.type_entry;
        let pushed = builder
            .push(
                TypedNode::new(
                    "source.reserve.declaration",
                    SourceAnchor::Range(source_binding.binding_range),
                )
                .with_children(vec![type_node_id])
                .with_recovery(NodeRecoveryState::Normal)
                .with_typing(typing_for_type_entry(
                    output.type_entries(),
                    declaration_type_entry,
                ))
                .with_links(TypedNodeLinks {
                    context: Some(LocalTypeContextId::new(0)),
                    type_entry: declaration_type_entry,
                    facts: declaration.facts.clone(),
                    ..TypedNodeLinks::default()
                }),
            )
            .map_err(|error| error.to_string())?;
        if pushed != declaration_node_id {
            return Err(format!(
                "source reserve declaration node {index} id mismatch"
            ));
        }
        declaration_nodes.push(declaration_node_id);
    }

    let root = builder
        .push(
            TypedNode::new(
                "source.reserve.module",
                SourceAnchor::Range(source_reserve.source_range()),
            )
            .with_children(declaration_nodes)
            .with_recovery(NodeRecoveryState::Normal)
            .with_typing(TypingState::Successful)
            .with_links(TypedNodeLinks {
                context: Some(LocalTypeContextId::new(0)),
                ..TypedNodeLinks::default()
            }),
        )
        .map_err(|error| error.to_string())?;
    let nodes = builder
        .finish(Some(root))
        .map_err(|error| error.to_string())?;
    TypedAst::try_new(TypedAstParts {
        source_id: source_reserve.source_id(),
        module_id: source_reserve.module_id().clone(),
        resolved_root: None,
        nodes,
        contexts: output.contexts().clone(),
        types: output.type_entries().clone(),
        facts: output.facts().clone(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: output.diagnostics().clone(),
    })
    .map_err(|error| error.to_string())
}

fn type_entry_for_site(types: &TypeTable, site: &TypedSiteRef) -> Option<TypeEntryId> {
    types
        .iter()
        .find_map(|(entry_id, entry)| (&entry.owner == site).then_some(entry_id))
}

fn typing_for_type_entry(types: &TypeTable, type_entry: Option<TypeEntryId>) -> TypingState {
    type_entry
        .and_then(|entry_id| types.get(entry_id))
        .map_or(TypingState::Unknown, |entry| match entry.status {
            TypeStatus::Known => TypingState::Successful,
            TypeStatus::Assumed => TypingState::Assumed,
            TypeStatus::Unknown => TypingState::Unknown,
            TypeStatus::Error => TypingState::Error,
            TypeStatus::Skipped => TypingState::Skipped,
            _ => TypingState::Unknown,
        })
}
