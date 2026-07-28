use std::collections::BTreeMap;

use mizar_checker::{
    binding_env::{BindingContextId, BindingEnv, BindingId},
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
    source_statement::{
        SourceStatementCandidateFactInput, SourceStatementCandidateFactKind,
        SourceStatementContextId, SourceStatementContextInput, SourceStatementFormulaTarget,
        SourceStatementHandoffInput, SourceStatementId, SourceStatementInput,
        SourceStatementInputFactInput, SourceStatementInputFactKind, SourceStatementKind,
        SourceStatementProducer, SourceStatementRecovery, SourceTheoremOwnerId,
        SourceTheoremOwnerInput, SourceTheoremRole, SourceTheoremStatus,
    },
    source_term::{SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermReferenceId},
    type_checker::{CheckedStatementOwner, FormulaKind},
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
        TypeFactTable, TypeTable, TypedArena, TypedAst, TypedAstParts, TypedSiteRef,
    },
};
use mizar_resolve::{
    env::{ExportStatus, NamespacePath, SymbolEnv, SymbolEnvIndexes, SymbolKind, Visibility},
    labels::{LabelProjection, LabelProjectionData, LabelResolver},
    resolved_ast::{LabelKind, LabelOriginPath, ModuleId},
};
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceNodeKind};

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_ast::{
        direct_token_texts, structural_child_ids, subtree_has_recovery, surface_nodes_with_kind,
        surface_site,
    },
    source_formula::{
        SourceReservedVariableBinaryFormula, SourceReservedVariableBinaryFormulaConfig,
        SourceReservedVariableBuiltinType, extract_source_reserved_variable_binary_formula,
    },
    source_term::source_term_parts_for_roots,
};

pub(in crate::runner) const SOURCE_STATEMENT_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementReservedVariableEqualitySmoke: x = x;\n",
);

const SOURCE_STATEMENT_LABEL: &str = "FormulaStatementReservedVariableEqualitySmoke";
const SOURCE_STATEMENT_SPELLING: &str =
    "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;";
const SOURCE_STATEMENT_CONFIG: SourceReservedVariableBinaryFormulaConfig =
    SourceReservedVariableBinaryFormulaConfig {
        label: SOURCE_STATEMENT_LABEL,
        operator: "=",
        formula_kind: FormulaKind::Equality,
        invalid_payload_key: "type_elaboration.checker.typed_ast_invalid",
        reserve_item_count: 1,
        binding_spellings: &["x"],
        binding_types: &[SourceReservedVariableBuiltinType::Set],
        binding_source_mode_spellings: &[None],
        mode_definitions: &[],
        left_binding_index: 0,
        right_binding_index: 0,
        require_shared_type_range: false,
        require_distinct_type_ranges: false,
        left_result_role: "source.statement.left",
        right_result_role: "source.statement.right",
        left_expected_role: None,
        right_expected_role: None,
    };

#[derive(Debug)]
pub(in crate::runner) struct SourceStatementExtraction {
    pub(in crate::runner) payload: SourceReservedVariableBinaryFormula,
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) label_spelling: String,
    pub(in crate::runner) statement_spelling: String,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementRouteInputs {
    pub(in crate::runner) binding_env: BindingEnv,
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: SourcePrimaryTermHandoff,
    pub(in crate::runner) atomic: SourceAtomicFormulaHandoff,
    pub(in crate::runner) statement: SourceStatementHandoffInput,
}

#[derive(Debug)]
pub(in crate::runner) struct SourceStatementRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
    pub(in crate::runner) left_lookup_ordinal: usize,
    pub(in crate::runner) right_lookup_ordinal: usize,
}

pub(in crate::runner) fn source_statement_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_statement_output_with_source(ast, module, symbols, source_text) {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_statement().is_some()
                && output.typed_ast.source_statement() == output.resolved.source_statement()
                && output.left_lookup_ordinal == 1
                && output.right_lookup_ordinal == 2 =>
        {
            Some(Vec::new())
        }
        Some(Ok(_)) | Some(Err(_)) => Some(vec![
            "type_elaboration.checker.typed_ast_invalid".to_owned(),
        ]),
    }
}

pub(in crate::runner) fn extract_source_reserved_variable_theorem_statement(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<SourceStatementExtraction> {
    if source_text != SOURCE_STATEMENT_TEXT
        || source_text.len() != 81
        || !source_text.ends_with('\n')
    {
        return None;
    }
    let payload = extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_STATEMENT_CONFIG,
    )?;
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let [(theorem_id, theorem)] = theorem_items.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, theorem)
        || theorem.range.start != 19
        || theorem.range.end != 80
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_LABEL, ":", ";"]
    {
        return None;
    }
    let theorem_children = structural_child_ids(ast, theorem);
    let [formula_expression_id] = theorem_children.as_slice() else {
        return None;
    };
    let formula_expression = ast.node(*formula_expression_id)?;
    let formula_children = structural_child_ids(ast, formula_expression);
    if !matches!(formula_expression.kind, SurfaceNodeKind::FormulaExpression)
        || formula_children.len() != 1
        || formula_children[0].index() != payload.formula_site.node().index()
    {
        return None;
    }
    let label = theorem
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .find(|child| child.token_text() == Some(SOURCE_STATEMENT_LABEL))?;
    if label.range.start != 27 || label.range.end != 72 {
        return None;
    }
    Some(SourceStatementExtraction {
        payload,
        theorem_site: surface_site(*theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        label_spelling: SOURCE_STATEMENT_LABEL.to_owned(),
        statement_spelling: SOURCE_STATEMENT_SPELLING.to_owned(),
    })
}

pub(in crate::runner) fn source_statement_output_with_source(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_output_with_source_and_mutation_impl(ast, module, symbols, source_text, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_output_with_source_and_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_output_with_source_and_mutation_impl(ast, module, symbols, source_text, mutate)
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_source_reserved_variable_theorem_statement(
        ast,
        module.clone(),
        symbols,
        source_text,
    )?;
    let symbols = match enrich_source_statement_resolver_env(&module, symbols, &extracted) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_source_reserved_variable_theorem_statement(
        ast,
        module.clone(),
        symbols,
        source_text,
    )?;
    let symbols = match enrich_source_statement_resolver_env(&module, symbols, &extracted) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

fn enrich_source_statement_resolver_env(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementExtraction,
) -> Result<SymbolEnv, String> {
    if symbols.module_id() != module {
        return Err("Task258A label resolver module mismatch".to_owned());
    }
    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, &extracted.label_spelling)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return Err("Task258A label resolver requires one theorem projection".to_owned());
    };
    let contribution = symbols
        .contributions()
        .get(owner.contribution())
        .ok_or_else(|| "Task258A label resolver contribution is missing".to_owned())?;
    let existing = symbols.labels().by_contribution(owner.contribution());
    if existing.len() == 1
        && symbols.labels().len() == 1
        && contribution.effects().labels() == [existing[0].origin_path().clone()]
    {
        return Ok(symbols.clone());
    }
    if !symbols.labels().is_empty() || !contribution.effects().labels().is_empty() {
        return Err("Task258A label resolver input is inconsistent".to_owned());
    }

    let origin_path = LabelOriginPath::new(format!(
        "{}::{}::theorem::{}",
        module.package().as_str(),
        module.path().as_str(),
        extracted.label_spelling,
    ));
    let projection = LabelProjection::current_module(
        LabelProjectionData {
            origin_path: origin_path.clone(),
            module: module.clone(),
            namespace: namespace.clone(),
            primary_spelling: extracted.label_spelling.clone(),
            kind: LabelKind::Theorem,
            declaration_range: extracted.label_range,
            origin: owner.origin().clone(),
            contribution: owner.contribution(),
        },
        2,
    )
    .with_visibility(Visibility::Public)
    .with_export_status(ExportStatus::Exported);
    let resolved = LabelResolver::new(&[projection]).resolve(module, &namespace, &[]);
    if !resolved.diagnostics().is_empty()
        || !resolved.table().is_empty()
        || !resolved.ids().is_empty()
        || resolved.index().len() != 1
    {
        return Err("Task258A label resolver result mismatch".to_owned());
    }

    let mut contributions = symbols.contributions().clone();
    contributions.add_label(owner.contribution(), origin_path);
    Ok(SymbolEnv::new(
        module.clone(),
        SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbols.symbols().clone(),
            labels: resolved.index().clone(),
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions,
            module_summaries: symbols.module_summaries().clone(),
        },
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementExtraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env(module, symbols, extracted)
}

fn build_source_statement_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementExtraction,
    mutate: impl FnOnce(&mut SourceStatementRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    if symbols.module_id() != &module {
        return Err("Task258A symbol module mismatch".to_owned());
    }
    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, &extracted.label_spelling)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner_entry] = owners.as_slice() else {
        return Err("Task258A requires one exact resolver theorem owner".to_owned());
    };
    let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner_entry.symbol().clone(),
        ast.source_id,
        &module,
    )
    .map_err(|error| error.to_string())?;
    if checked_owner.source_range() != extracted.theorem_range {
        return Err("Task258A resolver theorem range mismatch".to_owned());
    }
    if extracted.label_range.start != 27
        || extracted.label_range.end != 72
        || extracted.label_range.source_id != extracted.theorem_range.source_id
        || extracted.label_range.start < extracted.theorem_range.start
        || extracted.label_range.end > extracted.theorem_range.end
    {
        return Err("Task258A resolver label range mismatch".to_owned());
    }

    let binding_env = extracted
        .payload
        .reserve
        .bridge
        .prepare_binding_env(symbols)
        .map_err(|error| error.to_string())?;
    let mut owned_node_kinds = BTreeMap::new();
    owned_node_kinds.insert(
        extracted.theorem_site.node().index(),
        "source.statement.theorem",
    );
    owned_node_kinds.insert(
        extracted.payload.formula_site.node().index(),
        "source.formula.atomic.equality",
    );
    let parts = source_term_parts_for_roots(
        ast,
        module.clone(),
        &binding_env,
        [
            extracted.payload.left_site.node().index(),
            extracted.payload.right_site.node().index(),
        ],
        BindingContextId::new(0),
        &owned_node_kinds,
    )?;
    let primary = parts.handoff;
    let arena = parts.arena;
    let atomic_input = atomic_input(ast, module.clone(), &extracted);
    let atomic = SourceAtomicFormulaProducer::build(
        atomic_input,
        &binding_env,
        symbols,
        &primary,
        None,
        None,
        None,
        &arena,
    )
    .map_err(|error| error.to_string())?;
    let statement = statement_input(
        ast,
        module.clone(),
        owner_entry.symbol().clone(),
        owner_entry.contribution(),
        &extracted,
    );
    let mut inputs = SourceStatementRouteInputs {
        binding_env,
        arena,
        primary,
        atomic,
        statement,
    };
    mutate(&mut inputs);
    let statement = SourceStatementProducer::build(
        inputs.statement,
        symbols,
        &inputs.binding_env,
        &inputs.primary,
        &inputs.atomic,
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
    .with_source_term(inputs.primary)
    .map_err(|error| error.to_string())?
    .with_source_atomic_formula(inputs.atomic)
    .map_err(|error| error.to_string())?
    .with_source_statement(statement)
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.statement.transport"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_statement().is_none()
        || typed_ast.source_statement() != resolved.source_statement()
        || typed_ast.source_term() != resolved.source_term()
        || typed_ast.source_atomic_formula() != resolved.source_atomic_formula()
        || typed_ast.source_context().is_some()
        || typed_ast.source_type().is_some()
        || typed_ast.source_attribute().is_some()
        || typed_ast.source_evidence().is_some()
        || typed_ast.source_application().is_some()
        || typed_ast.source_structure().is_some()
        || typed_ast.source_set_term().is_some()
        || typed_ast.source_composite_formula().is_some()
        || typed_ast.source_formula_composition().is_some()
        || typed_ast.source_condition_formula_composition().is_some()
        || typed_ast.source_predicate_chain_composition().is_some()
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
        return Err("Task258A immutable final handoff mismatch".to_owned());
    }
    Ok(SourceStatementRouteOutput {
        typed_ast,
        resolved,
        left_lookup_ordinal: extracted.payload.left_lookup_ordinal,
        right_lookup_ordinal: extracted.payload.right_lookup_ordinal,
    })
}

fn atomic_input(
    ast: &SurfaceAst,
    module: ModuleId,
    extracted: &SourceStatementExtraction,
) -> SourceAtomicFormulaHandoffInput {
    SourceAtomicFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        formulas: vec![SourceAtomicFormulaInput {
            site: extracted.payload.formula_site.clone(),
            source_range: extracted.payload.formula_range,
            source_ordinal: 0,
            context: BindingContextId::new(0),
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
        requests: (0..2)
            .map(|ordinal| SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(ordinal)),
                candidate: None,
                type_site: None,
                attribute: None,
            })
            .collect(),
    }
}

fn statement_input(
    ast: &SurfaceAst,
    module: ModuleId,
    symbol: mizar_resolve::resolved_ast::SymbolId,
    contribution: mizar_resolve::env::SourceContributionId,
    extracted: &SourceStatementExtraction,
) -> SourceStatementHandoffInput {
    SourceStatementHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        owners: vec![SourceTheoremOwnerInput {
            symbol,
            contribution,
            site: extracted.theorem_site.clone(),
            source_range: extracted.theorem_range,
            spelling: extracted.label_spelling.clone(),
            role: SourceTheoremRole::Theorem,
            status: SourceTheoremStatus::Unmodified,
            recovery: SourceStatementRecovery::Normal,
        }],
        statements: vec![SourceStatementInput {
            owner: SourceTheoremOwnerId::new(0),
            context: SourceStatementContextId::new(0),
            formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0)),
            site: extracted.theorem_site.clone(),
            source_range: extracted.theorem_range,
            source_ordinal: 0,
            spelling: extracted.statement_spelling.clone(),
            kind: SourceStatementKind::TheoremProposition,
            recovery: SourceStatementRecovery::Normal,
        }],
        contexts: vec![SourceStatementContextInput {
            statement: SourceStatementId::new(0),
            binding_context: BindingContextId::new(0),
            source_range: extracted.theorem_range,
            visible_bindings: vec![BindingId::new(0)],
        }],
        input_facts: vec![SourceStatementInputFactInput {
            statement: SourceStatementId::new(0),
            context: SourceStatementContextId::new(0),
            ordinal: 0,
            kind: SourceStatementInputFactKind::ReservedTypeGuard,
            binding: BindingId::new(0),
            uses: vec![
                SourcePrimaryTermReferenceId::new(0),
                SourcePrimaryTermReferenceId::new(1),
            ],
        }],
        candidate_facts: vec![SourceStatementCandidateFactInput {
            statement: SourceStatementId::new(0),
            context: SourceStatementContextId::new(0),
            ordinal: 0,
            kind: SourceStatementCandidateFactKind::UnverifiedProposition,
            formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0)),
        }],
    }
}
