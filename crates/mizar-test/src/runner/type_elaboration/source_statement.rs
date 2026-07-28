use std::collections::BTreeMap;

use mizar_checker::{
    binding_env::{
        BindingContextDraft, BindingContextId, BindingContextLayer, BindingContextOwner,
        BindingContextRecovery, BindingEnv, BindingEnvParts, BindingId,
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
    source_statement::{
        SourceStatementCandidateFactId, SourceStatementCandidateFactInput,
        SourceStatementCandidateFactKind, SourceStatementCitationInput,
        SourceStatementCitationKind, SourceStatementContextId, SourceStatementContextInput,
        SourceStatementFormulaTarget, SourceStatementHandoffInput, SourceStatementId,
        SourceStatementInput, SourceStatementInputFactInput, SourceStatementInputFactKind,
        SourceStatementKind, SourceStatementLabelId, SourceStatementLabelInput,
        SourceStatementLabelKind, SourceStatementProducer, SourceStatementRecovery,
        SourceStatementReferenceHandoffInput, SourceStatementReferenceProducer,
        SourceStatementWitnessHandoffInput, SourceStatementWitnessInput,
        SourceStatementWitnessKind, SourceStatementWitnessNameId, SourceStatementWitnessNameInput,
        SourceStatementWitnessProducer, SourceStatementWitnessTermTarget, SourceTheoremOwnerId,
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
    labels::{
        LabelProjection, LabelProjectionData, LabelReferenceCandidate, LabelResolver,
        LabelScopePath,
    },
    names::LocalTermScope,
    resolved_ast::{
        LabelKind, LabelOriginPath, ModuleId, NameRefTable, NodeReferenceKey, NodeResolutionState,
        RecoveryState, ReferenceSite, ResolvedArena, ResolvedArenaBuilder, ResolvedAst,
        ResolvedImports, ResolvedNode, SemanticOrigin,
    },
};
use mizar_session::{SourceAnchor, SourceRange};
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
    source_reserve::extract_builtin_source_reserve_declarations_after_node_guard,
    source_term::{source_term_parts_for_context_roots, source_term_parts_for_roots},
};

pub(in crate::runner) const SOURCE_STATEMENT_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementReservedVariableEqualitySmoke: x = x;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B1_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementNestedContextSmoke: x = x proof\n",
    "  A: x = x proof\n",
    "    thus x = x;\n",
    "  end;\n",
    "  thus x = x by A;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B2_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementSingleAssumptionSmoke: x = x proof\n",
    "  assume x = x;\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementSingleWitnessSmoke: x = x proof\n",
    "  take x;\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3N_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementNamedWitnessSmoke: x = x proof\n",
    "  take y = x;\n",
    "  thus x = x;\n",
    "end;\n",
);

const SOURCE_STATEMENT_LABEL: &str = "FormulaStatementReservedVariableEqualitySmoke";
const SOURCE_STATEMENT_SPELLING: &str =
    "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;";
const SOURCE_STATEMENT_B1_LABEL: &str = "FormulaStatementNestedContextSmoke";
const SOURCE_STATEMENT_B1_SPELLINGS: [&str; 4] = [
    "theorem FormulaStatementNestedContextSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x by A ; end ;",
    "A : x = x proof thus x = x ; end ;",
    "thus x = x ;",
    "thus x = x by A ;",
];
const SOURCE_STATEMENT_B2_LABEL: &str = "FormulaStatementSingleAssumptionSmoke";
const SOURCE_STATEMENT_B2_SPELLINGS: [&str; 3] = [
    "theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ; thus x = x ; end ;",
    "assume x = x ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3_LABEL: &str = "FormulaStatementSingleWitnessSmoke";
const SOURCE_STATEMENT_B3_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementSingleWitnessSmoke : x = x proof take x ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3N_LABEL: &str = "FormulaStatementNamedWitnessSmoke";
const SOURCE_STATEMENT_B3N_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementNamedWitnessSmoke : x = x proof take y = x ; thus x = x ; end ;",
    "thus x = x ;",
];
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
pub(in crate::runner) struct SourceStatementB1Extraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 4],
    pub(in crate::runner) statement_ranges: [SourceRange; 4],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 4],
    pub(in crate::runner) formula_ranges: [SourceRange; 4],
    pub(in crate::runner) term_sites: [TypedSiteRef; 8],
    pub(in crate::runner) term_ranges: [SourceRange; 8],
    pub(in crate::runner) proof_ranges: [SourceRange; 2],
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB2Extraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 3],
    pub(in crate::runner) statement_ranges: [SourceRange; 3],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 3],
    pub(in crate::runner) formula_ranges: [SourceRange; 3],
    pub(in crate::runner) term_sites: [TypedSiteRef; 6],
    pub(in crate::runner) term_ranges: [SourceRange; 6],
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3Extraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 5],
    pub(in crate::runner) term_ranges: [SourceRange; 5],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3NExtraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 5],
    pub(in crate::runner) term_ranges: [SourceRange; 5],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) name_site: TypedSiteRef,
    pub(in crate::runner) name_range: SourceRange,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
struct SourceStatementWitnessExtraction {
    theorem_site: TypedSiteRef,
    theorem_range: SourceRange,
    label_range: SourceRange,
    statement_sites: [TypedSiteRef; 2],
    statement_ranges: [SourceRange; 2],
    formula_sites: [TypedSiteRef; 2],
    formula_ranges: [SourceRange; 2],
    term_sites: [TypedSiteRef; 5],
    term_ranges: [SourceRange; 5],
    take_site: TypedSiteRef,
    take_range: SourceRange,
    witness_site: TypedSiteRef,
    witness_range: SourceRange,
    name: Option<(TypedSiteRef, SourceRange)>,
    proof_range: SourceRange,
    label: &'static str,
    spellings: &'static [&'static str; 2],
    task: &'static str,
    node_count: usize,
    root: usize,
}

impl From<SourceStatementB3Extraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3Extraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites,
            term_ranges: extracted.term_ranges,
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witness_site: extracted.witness_site,
            witness_range: extracted.witness_range,
            name: None,
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3_LABEL,
            spellings: &SOURCE_STATEMENT_B3_SPELLINGS,
            task: "Task258B3",
            node_count: 49,
            root: 48,
        }
    }
}

impl From<SourceStatementB3NExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3NExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites,
            term_ranges: extracted.term_ranges,
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witness_site: extracted.witness_site,
            witness_range: extracted.witness_range,
            name: Some((extracted.name_site, extracted.name_range)),
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3N_LABEL,
            spellings: &SOURCE_STATEMENT_B3N_SPELLINGS,
            task: "Task258B3N",
            node_count: 51,
            root: 50,
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementRouteInputs {
    pub(in crate::runner) binding_env: BindingEnv,
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: SourcePrimaryTermHandoff,
    pub(in crate::runner) atomic: SourceAtomicFormulaHandoff,
    pub(in crate::runner) statement: SourceStatementHandoffInput,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB1RouteInputs {
    pub(in crate::runner) binding_env: BindingEnv,
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: SourcePrimaryTermHandoff,
    pub(in crate::runner) atomic: SourceAtomicFormulaHandoff,
    pub(in crate::runner) statement: SourceStatementHandoffInput,
    pub(in crate::runner) resolver_ast: ResolvedAst,
    pub(in crate::runner) projection: LabelProjection,
    pub(in crate::runner) reference: LabelReferenceCandidate,
    pub(in crate::runner) resolution: mizar_resolve::labels::LabelResolutionResult,
    pub(in crate::runner) reference_input: SourceStatementReferenceHandoffInput,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3RouteInputs {
    pub(in crate::runner) binding_env: BindingEnv,
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: SourcePrimaryTermHandoff,
    pub(in crate::runner) atomic: SourceAtomicFormulaHandoff,
    pub(in crate::runner) statement: SourceStatementHandoffInput,
    pub(in crate::runner) witness: SourceStatementWitnessHandoffInput,
}

pub(in crate::runner) type SourceStatementB3NRouteInputs = SourceStatementB3RouteInputs;

#[derive(Debug)]
pub(in crate::runner) struct SourceStatementRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
    pub(in crate::runner) left_lookup_ordinal: usize,
    pub(in crate::runner) right_lookup_ordinal: usize,
    pub(in crate::runner) reference_use_ordinals: Vec<usize>,
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
                && ((output.typed_ast.source_statement_references().is_none()
                    && output.typed_ast.source_statement_witnesses().is_none()
                    && output
                        .typed_ast
                        .source_statement()
                        .is_some_and(|statement| statement.statements().len() == 1)
                    && output.left_lookup_ordinal == 1
                    && output.right_lookup_ordinal == 2
                    && output.reference_use_ordinals == [1, 2])
                    || (output.typed_ast.source_statement_references().is_none()
                        && output.typed_ast.source_statement_witnesses().is_none()
                        && output
                            .typed_ast
                            .source_statement()
                            .is_some_and(|statement| statement.statements().len() == 3)
                        && output.left_lookup_ordinal == 1
                        && output.right_lookup_ordinal == 1
                        && output.reference_use_ordinals == [1; 6])
                    || (output.typed_ast.source_statement_references().is_some()
                        && output.typed_ast.source_statement_witnesses().is_none()
                        && output.left_lookup_ordinal == 1
                        && output.right_lookup_ordinal == 1
                        && output.reference_use_ordinals == [1; 8])
                    || (output.typed_ast.source_statement_references().is_none()
                        && output.typed_ast.source_statement_witnesses().is_some()
                        && output.typed_ast.source_statement_witnesses()
                            == output.resolved.source_statement_witnesses()
                        && output
                            .typed_ast
                            .source_statement()
                            .is_some_and(|statement| statement.statements().len() == 2)
                        && output.left_lookup_ordinal == 1
                        && output.right_lookup_ordinal == 1
                        && output.reference_use_ordinals == [1; 5])) =>
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

pub(in crate::runner) fn extract_nested_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB1Extraction> {
    if source_text != SOURCE_STATEMENT_B1_TEXT
        || source_text.len() != 139
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 77
        || ast.root()?.index() != 76
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
    {
        return None;
    }
    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    if item_children.len() != 2 {
        return None;
    }
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 138)?;
    if item_children[1] != theorem_id
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B1_LABEL, ":", ";"]
    {
        return None;
    }
    let (compact_id, _) = exact_surface_node(ast, SurfaceNodeKind::CompactStatement, 77, 114)?;
    let (nested_conclusion_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 96, 107)?;
    let (outer_conclusion_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 117, 133)?;
    let (outer_proof_id, _) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 69, 137)?;
    let (nested_proof_id, _) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 86, 113)?;
    let (justification_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::JustificationClause, 128, 132)?;
    let (reference_id, reference) = exact_surface_node(ast, SurfaceNodeKind::Reference, 131, 132)?;
    if reference_id.index() != 68
        || direct_token_texts(ast, reference).as_slice() != ["A"]
        || !surface_is_descendant(ast, theorem_id, outer_proof_id)
        || !surface_is_descendant(ast, outer_proof_id, compact_id)
        || !surface_is_descendant(ast, compact_id, nested_proof_id)
        || !surface_is_descendant(ast, nested_proof_id, nested_conclusion_id)
        || !surface_is_descendant(ast, outer_proof_id, outer_conclusion_id)
        || surface_is_descendant(ast, nested_proof_id, outer_conclusion_id)
        || !surface_is_descendant(ast, outer_conclusion_id, justification_id)
        || !surface_is_descendant(ast, justification_id, reference_id)
    {
        return None;
    }
    let label = ast.nodes().get(12)?;
    if label.range.start != 77 || label.range.end != 78 || label.token_text() != Some("A") {
        return None;
    }
    let statement_ids = [
        theorem_id,
        compact_id,
        nested_conclusion_id,
        outer_conclusion_id,
    ];
    let formula_ranges = [(63, 68), (80, 85), (101, 106), (122, 127)];
    let mut formula_ids = Vec::new();
    for (start, end) in formula_ranges {
        let (id, formula) = exact_surface_node(
            ast,
            SurfaceNodeKind::BuiltinPredicateApplication,
            start,
            end,
        )?;
        if direct_token_texts(ast, formula).as_slice() != ["="] {
            return None;
        }
        formula_ids.push(id);
    }
    let term_ranges = [
        (63, 64),
        (67, 68),
        (80, 81),
        (84, 85),
        (101, 102),
        (105, 106),
        (122, 123),
        (126, 127),
    ];
    let mut term_sites = Vec::new();
    for (start, end) in term_ranges {
        let (id, term) = exact_surface_node(ast, SurfaceNodeKind::TermReference, start, end)?;
        if direct_token_texts(ast, term).as_slice() != ["x"] {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    for index in 0..4 {
        if !surface_is_descendant(ast, statement_ids[index], formula_ids[index]) {
            return None;
        }
    }
    let formula_ids: [mizar_syntax::SurfaceNodeId; 4] = formula_ids.try_into().ok()?;
    Some(SourceStatementB1Extraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: range(ast.source_id, 27, 61),
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [
            range(ast.source_id, 19, 138),
            range(ast.source_id, 77, 114),
            range(ast.source_id, 96, 107),
            range(ast.source_id, 117, 133),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: term_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        proof_ranges: [range(ast.source_id, 69, 137), range(ast.source_id, 86, 113)],
    })
}

pub(in crate::runner) fn extract_single_assumption_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB2Extraction> {
    if source_text != SOURCE_STATEMENT_B2_TEXT
        || source_text.len() != 113
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 55
        || ast.root()?.index() != 54
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
    {
        return None;
    }
    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 112)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 72, 111)?;
    let (assumption_id, assumption) =
        exact_surface_node(ast, SurfaceNodeKind::AssumptionStatement, 80, 93)?;
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 96, 107)?;
    if item_children.len() != 2
        || item_children[0].index() != 27
        || item_children[1] != theorem_id
        || theorem_id.index() != 51
        || proof_id.index() != 50
        || assumption_id.index() != 41
        || conclusion_id.index() != 49
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B2_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, assumption).as_slice() != ["assume", ";"]
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, assumption_id)
        || !surface_is_descendant(ast, proof_id, conclusion_id)
        || surface_is_descendant(ast, assumption_id, conclusion_id)
        || surface_is_descendant(ast, conclusion_id, assumption_id)
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::AssumptionStatement).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement).len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::CompactStatement).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::Reference).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::JustificationClause).is_empty()
    {
        return None;
    }
    let label = ast.nodes().get(6)?;
    if label.range.start != 27
        || label.range.end != 64
        || label.token_text() != Some(SOURCE_STATEMENT_B2_LABEL)
    {
        return None;
    }
    let statement_ids = [theorem_id, assumption_id, conclusion_id];
    let formula_ranges = [(66, 71), (87, 92), (101, 106)];
    let expected_formula_ids = [32, 38, 46];
    let mut formula_ids = Vec::new();
    for (index, (start, end)) in formula_ranges.into_iter().enumerate() {
        let (id, formula) = exact_surface_node(
            ast,
            SurfaceNodeKind::BuiltinPredicateApplication,
            start,
            end,
        )?;
        if id.index() != expected_formula_ids[index]
            || direct_token_texts(ast, formula).as_slice() != ["="]
            || !surface_is_descendant(ast, statement_ids[index], id)
        {
            return None;
        }
        formula_ids.push(id);
    }
    let term_ranges = [
        (66, 67),
        (70, 71),
        (87, 88),
        (91, 92),
        (101, 102),
        (105, 106),
    ];
    let expected_term_ids = [28, 30, 34, 36, 42, 44];
    let mut term_sites = Vec::new();
    for (index, (start, end)) in term_ranges.into_iter().enumerate() {
        let (id, term) = exact_surface_node(ast, SurfaceNodeKind::TermReference, start, end)?;
        if id.index() != expected_term_ids[index]
            || direct_token_texts(ast, term).as_slice() != ["x"]
        {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let formula_ids: [mizar_syntax::SurfaceNodeId; 3] = formula_ids.try_into().ok()?;
    Some(SourceStatementB2Extraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [
            range(ast.source_id, 19, 112),
            range(ast.source_id, 80, 93),
            range(ast.source_id, 96, 107),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: term_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        proof_range: proof.range,
    })
}

pub(in crate::runner) fn extract_single_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3Extraction> {
    if source_text != SOURCE_STATEMENT_B3_TEXT
        || source_text.len() != 104
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 49
        || ast.root()?.index() != 48
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
    {
        return None;
    }
    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 103)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 69, 102)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 77, 84)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 82, 83)?;
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 87, 98)?;
    if item_children.len() != 2
        || item_children[0].index() != 25
        || item_children[1] != theorem_id
        || theorem_id.index() != 45
        || proof_id.index() != 44
        || take_id.index() != 35
        || witness_id.index() != 34
        || conclusion_id.index() != 43
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ";"]
        || !direct_token_texts(ast, witness).is_empty()
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, proof_id, conclusion_id)
        || surface_is_descendant(ast, conclusion_id, take_id)
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::TakeStatement).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::Witness).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement).len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::CompactStatement).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::Reference).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::JustificationClause).is_empty()
    {
        return None;
    }
    let label = ast.nodes().get(6)?;
    if label.range.start != 27
        || label.range.end != 61
        || label.token_text() != Some(SOURCE_STATEMENT_B3_LABEL)
    {
        return None;
    }
    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(63, 68), (92, 97)];
    let expected_formula_ids = [30, 40];
    let mut formula_ids = Vec::new();
    for (index, (start, end)) in formula_ranges.into_iter().enumerate() {
        let (id, formula) = exact_surface_node(
            ast,
            SurfaceNodeKind::BuiltinPredicateApplication,
            start,
            end,
        )?;
        if id.index() != expected_formula_ids[index]
            || direct_token_texts(ast, formula).as_slice() != ["="]
            || !surface_is_descendant(ast, statement_ids[index], id)
        {
            return None;
        }
        formula_ids.push(id);
    }
    let term_ranges = [(63, 64), (67, 68), (82, 83), (92, 93), (96, 97)];
    let expected_term_ids = [26, 28, 32, 36, 38];
    let mut term_sites = Vec::new();
    for (index, (start, end)) in term_ranges.into_iter().enumerate() {
        let (id, term) = exact_surface_node(ast, SurfaceNodeKind::TermReference, start, end)?;
        if id.index() != expected_term_ids[index]
            || direct_token_texts(ast, term).as_slice() != ["x"]
        {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3Extraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [range(ast.source_id, 19, 103), range(ast.source_id, 87, 98)],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: term_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        proof_range: proof.range,
    })
}

pub(in crate::runner) fn extract_named_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3NExtraction> {
    if source_text != SOURCE_STATEMENT_B3N_TEXT
        || source_text.len() != 107
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 51
        || ast.root()?.index() != 50
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
    {
        return None;
    }
    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 106)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 68, 105)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 76, 87)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 81, 86)?;
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 90, 101)?;
    if item_children.len() != 2
        || item_children[0].index() != 27
        || item_children[1] != theorem_id
        || theorem_id.index() != 47
        || proof_id.index() != 46
        || take_id.index() != 37
        || witness_id.index() != 36
        || conclusion_id.index() != 45
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3N_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ";"]
        || direct_token_texts(ast, witness).as_slice() != ["y", "="]
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, proof_id, conclusion_id)
        || surface_is_descendant(ast, conclusion_id, take_id)
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::TakeStatement).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::Witness).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement).len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::CompactStatement).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::Reference).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::JustificationClause).is_empty()
    {
        return None;
    }
    let label = ast.nodes().get(6)?;
    let name_id = *witness.children.first()?;
    let name = ast.node(name_id)?;
    if label.range.start != 27
        || label.range.end != 60
        || label.token_text() != Some(SOURCE_STATEMENT_B3N_LABEL)
        || name.range.start != 81
        || name.range.end != 82
        || name_id.index() != 13
        || name.token_text() != Some("y")
        || !surface_is_descendant(ast, witness_id, name_id)
    {
        return None;
    }
    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(62, 67), (95, 100)];
    let expected_formula_ids = [32, 42];
    let mut formula_ids = Vec::new();
    for (index, (start, end)) in formula_ranges.into_iter().enumerate() {
        let (id, formula) = exact_surface_node(
            ast,
            SurfaceNodeKind::BuiltinPredicateApplication,
            start,
            end,
        )?;
        if id.index() != expected_formula_ids[index]
            || direct_token_texts(ast, formula).as_slice() != ["="]
            || !surface_is_descendant(ast, statement_ids[index], id)
        {
            return None;
        }
        formula_ids.push(id);
    }
    let term_ranges = [(62, 63), (66, 67), (85, 86), (95, 96), (99, 100)];
    let expected_term_ids = [28, 30, 34, 38, 40];
    let mut term_sites = Vec::new();
    for (index, (start, end)) in term_ranges.into_iter().enumerate() {
        let (id, term) = exact_surface_node(ast, SurfaceNodeKind::TermReference, start, end)?;
        if id.index() != expected_term_ids[index]
            || direct_token_texts(ast, term).as_slice() != ["x"]
        {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3NExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [range(ast.source_id, 19, 106), range(ast.source_id, 90, 101)],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: term_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        name_site: surface_site(name_id),
        name_range: name.range,
        proof_range: proof.range,
    })
}

fn exact_surface_node(
    ast: &SurfaceAst,
    kind: SurfaceNodeKind,
    start: usize,
    end: usize,
) -> Option<(mizar_syntax::SurfaceNodeId, &mizar_syntax::SurfaceNode)> {
    let matches = surface_nodes_with_kind(ast, kind)
        .into_iter()
        .filter(|(_, node)| node.range.start == start && node.range.end == end)
        .collect::<Vec<_>>();
    let [entry] = matches.as_slice() else {
        return None;
    };
    Some(*entry)
}

fn surface_is_descendant(
    ast: &SurfaceAst,
    ancestor: mizar_syntax::SurfaceNodeId,
    target: mizar_syntax::SurfaceNodeId,
) -> bool {
    ast.node(ancestor).is_some_and(|node| {
        node.children
            .iter()
            .any(|child| *child == target || surface_is_descendant(ast, *child, target))
    })
}

pub(in crate::runner) fn source_statement_output_with_source(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    if source_text == SOURCE_STATEMENT_B3N_TEXT {
        return source_statement_b3n_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3_TEXT {
        return source_statement_b3_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B2_TEXT {
        return source_statement_b2_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B1_TEXT {
        return source_statement_b1_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    source_statement_output_with_source_and_mutation_impl(ast, module, symbols, source_text, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3n_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3NRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3n_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3n_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_named_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3N_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3n_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_b3n_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3NRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_named_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3N_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3n_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_single_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_b3_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_single_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3_output(
        ast, module, &symbols, extracted, mutate,
    ))
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

#[cfg(test)]
pub(in crate::runner) fn source_statement_b2_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b2_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b2_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_single_assumption_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B2_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b2_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_b2_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_single_assumption_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B2_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b2_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b1_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB1RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b1_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

fn source_statement_b1_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB1RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_nested_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B1_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b1_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

fn enrich_source_statement_resolver_env(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementExtraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        &extracted.label_spelling,
        extracted.label_range,
    )
}

fn enrich_source_statement_resolver_env_for_owner(
    module: &ModuleId,
    symbols: &SymbolEnv,
    label_spelling: &str,
    label_range: SourceRange,
) -> Result<SymbolEnv, String> {
    if symbols.module_id() != module {
        return Err("Task258A label resolver module mismatch".to_owned());
    }
    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, label_spelling)
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
        label_spelling,
    ));
    let projection = LabelProjection::current_module(
        LabelProjectionData {
            origin_path: origin_path.clone(),
            module: module.clone(),
            namespace: namespace.clone(),
            primary_spelling: label_spelling.to_owned(),
            kind: LabelKind::Theorem,
            declaration_range: label_range,
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

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementB3Extraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3_LABEL,
        extracted.label_range,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3n_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementB3NExtraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3N_LABEL,
        extracted.label_range,
    )
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
        reference_use_ordinals: vec![
            extracted.payload.left_lookup_ordinal,
            extracted.payload.right_lookup_ordinal,
        ],
    })
}

fn build_source_statement_b3_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3Extraction,
    mutate: impl FnOnce(&mut SourceStatementB3RouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3n_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3NExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3NRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_witness_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementWitnessExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3RouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    if symbols.module_id() != &module {
        return Err(format!("{} symbol module mismatch", extracted.task));
    }
    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, extracted.label)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner_entry] = owners.as_slice() else {
        return Err(format!(
            "{} requires one exact resolver theorem owner",
            extracted.task
        ));
    };
    let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner_entry.symbol().clone(),
        ast.source_id,
        &module,
    )
    .map_err(|error| error.to_string())?;
    let labels = symbols
        .labels()
        .visible_candidates(&namespace, extracted.label);
    let [label] = labels.as_slice() else {
        return Err(format!(
            "{} resolver theorem label mismatch",
            extracted.task
        ));
    };
    let expected_origin_path = LabelOriginPath::new(format!(
        "{}::{}::theorem::{}",
        module.package().as_str(),
        module.path().as_str(),
        extracted.label,
    ));
    let contribution = symbols
        .contributions()
        .get(owner_entry.contribution())
        .ok_or_else(|| format!("{} resolver contribution is missing", extracted.task))?;
    if checked_owner.source_range() != extracted.theorem_range
        || checked_owner.origin().structural_path() != [2, 1]
        || owner_entry.contribution().index() != 0
        || symbols.labels().len() != 1
        || !symbols.imports().is_empty()
        || label.origin_path() != &expected_origin_path
        || label.kind() != LabelKind::Theorem
        || label.visibility() != Visibility::Public
        || label.export_status() != ExportStatus::Exported
        || label.namespace() != &namespace
        || label.primary_spelling() != extracted.label
        || extracted.label_range != range(ast.source_id, 27, 27 + extracted.label.len())
        || label.origin() != checked_owner.origin()
        || label.contribution() != owner_entry.contribution()
        || label.recovery() != RecoveryState::Normal
        || contribution.effects().labels() != [expected_origin_path]
    {
        return Err(format!(
            "{} resolver theorem provenance mismatch",
            extracted.task
        ));
    }

    let reserve =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .map_err(|()| format!("{} reserve extraction failed", extracted.task))?;
    let base_binding_env = reserve
        .bridge
        .prepare_binding_env(symbols)
        .map_err(|error| error.to_string())?;
    let mut contexts = base_binding_env.contexts().clone();
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: extracted.proof_range,
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    if proof != BindingContextId::new(1) {
        return Err(format!("{} binding context identity drift", extracted.task));
    }
    let binding_env = BindingEnv::try_new(BindingEnvParts {
        source_id: ast.source_id,
        module_id: module.clone(),
        contexts,
        bindings: base_binding_env.bindings().clone(),
        diagnostics: base_binding_env.diagnostics().clone(),
    })
    .map_err(|error| error.to_string())?;
    let mut owned_node_kinds = BTreeMap::new();
    for (site, kind) in extracted
        .statement_sites
        .iter()
        .zip(["source.statement.theorem", "source.statement.conclusion"])
    {
        owned_node_kinds.insert(site.node().index(), kind);
    }
    owned_node_kinds.insert(
        extracted.take_site.node().index(),
        "source.statement-witness.take",
    );
    owned_node_kinds.insert(
        extracted.witness_site.node().index(),
        "source.statement-witness.item",
    );
    if let Some((name_site, _)) = &extracted.name {
        owned_node_kinds.insert(name_site.node().index(), "source.statement-witness.name");
    }
    for site in &extracted.formula_sites {
        owned_node_kinds.insert(site.node().index(), "source.formula.atomic.equality");
    }
    let roots = extracted
        .term_sites
        .iter()
        .zip([0, 0, 1, 1, 1])
        .map(|(site, context)| (site.node().index(), BindingContextId::new(context)));
    let parts = source_term_parts_for_context_roots(
        ast,
        module.clone(),
        &binding_env,
        roots,
        &owned_node_kinds,
    )?;
    let primary = parts.handoff;
    let arena = parts.arena;
    if primary
        .terms()
        .iter()
        .zip(extracted.term_ranges)
        .any(|((_, term), expected)| term.source_range() != expected)
        || primary
            .references()
            .iter()
            .any(|(_, reference)| reference.use_ordinal() != 1)
    {
        return Err(format!("{} primary-term profile drift", extracted.task));
    }
    let atomic = SourceAtomicFormulaProducer::build(
        task258b3_atomic_input(ast, module.clone(), &extracted),
        &binding_env,
        symbols,
        &primary,
        None,
        None,
        None,
        &arena,
    )
    .map_err(|error| error.to_string())?;
    let statement = task258b3_statement_input(
        ast,
        module.clone(),
        owner_entry.symbol().clone(),
        owner_entry.contribution(),
        &extracted,
    );
    let name_id = extracted
        .name
        .as_ref()
        .map(|_| SourceStatementWitnessNameId::new(0));
    let witness = SourceStatementWitnessHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        witnesses: vec![SourceStatementWitnessInput {
            owner: SourceTheoremOwnerId::new(0),
            binding_context: BindingContextId::new(1),
            term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
            take_site: extracted.take_site.clone(),
            take_range: extracted.take_range,
            site: extracted.witness_site.clone(),
            source_range: extracted.witness_range,
            source_ordinal: 1,
            ordinal: 0,
            spelling: if name_id.is_some() { "y = x" } else { "x" }.to_owned(),
            kind: if name_id.is_some() {
                SourceStatementWitnessKind::Named
            } else {
                SourceStatementWitnessKind::Unnamed
            },
            recovery: SourceStatementRecovery::Normal,
            name: name_id,
        }],
        names: extracted
            .name
            .as_ref()
            .map(|(site, source_range)| SourceStatementWitnessNameInput {
                witness: mizar_checker::source_statement::SourceStatementWitnessId::new(0),
                site: site.clone(),
                source_range: *source_range,
                spelling: "y".to_owned(),
                recovery: SourceStatementRecovery::Normal,
            })
            .into_iter()
            .collect(),
    };
    let mut inputs = SourceStatementB3RouteInputs {
        binding_env,
        arena,
        primary,
        atomic,
        statement,
        witness,
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
    let witnesses = SourceStatementWitnessProducer::build(
        inputs.witness,
        &statement,
        &inputs.primary,
        &inputs.arena,
    )
    .map_err(|error| error.to_string())?;
    let reference_use_ordinals = inputs
        .primary
        .references()
        .iter()
        .map(|(_, reference)| reference.use_ordinal())
        .collect::<Vec<_>>();
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
    .with_source_statement_witnesses(statement, witnesses)
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
    if typed_ast.nodes().len() != extracted.node_count
        || typed_ast
            .nodes()
            .root()
            .is_none_or(|root| root.index() != extracted.root)
        || typed_ast.source_statement().is_none()
        || typed_ast.source_statement_witnesses().is_none()
        || typed_ast.source_statement_references().is_some()
        || typed_ast.source_statement() != resolved.source_statement()
        || typed_ast.source_statement_witnesses() != resolved.source_statement_witnesses()
        || resolved.source_statement_references().is_some()
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
        return Err(format!(
            "{} immutable final handoff mismatch",
            extracted.task
        ));
    }
    Ok(SourceStatementRouteOutput {
        typed_ast,
        resolved,
        left_lookup_ordinal: reference_use_ordinals[0],
        right_lookup_ordinal: reference_use_ordinals[1],
        reference_use_ordinals,
    })
}

fn build_source_statement_b2_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB2Extraction,
    mutate: impl FnOnce(&mut SourceStatementRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    if symbols.module_id() != &module {
        return Err("Task258B2 symbol module mismatch".to_owned());
    }
    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B2_LABEL)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner_entry] = owners.as_slice() else {
        return Err("Task258B2 requires one exact resolver theorem owner".to_owned());
    };
    let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner_entry.symbol().clone(),
        ast.source_id,
        &module,
    )
    .map_err(|error| error.to_string())?;
    let labels = symbols
        .labels()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B2_LABEL);
    let [label] = labels.as_slice() else {
        return Err("Task258B2 resolver theorem label mismatch".to_owned());
    };
    let expected_origin_path = LabelOriginPath::new(format!(
        "{}::{}::theorem::{}",
        module.package().as_str(),
        module.path().as_str(),
        SOURCE_STATEMENT_B2_LABEL,
    ));
    let contribution = symbols
        .contributions()
        .get(owner_entry.contribution())
        .ok_or_else(|| "Task258B2 resolver contribution is missing".to_owned())?;
    if checked_owner.source_range() != extracted.theorem_range
        || checked_owner.origin().structural_path() != [2, 1]
        || owner_entry.contribution().index() != 0
        || extracted.label_range != range(ast.source_id, 27, 64)
        || symbols.labels().len() != 1
        || !symbols.imports().is_empty()
        || label.origin_path() != &expected_origin_path
        || label.kind() != LabelKind::Theorem
        || label.visibility() != Visibility::Public
        || label.export_status() != ExportStatus::Exported
        || label.namespace() != &namespace
        || label.primary_spelling() != SOURCE_STATEMENT_B2_LABEL
        || label.origin() != checked_owner.origin()
        || label.contribution() != owner_entry.contribution()
        || label.recovery() != RecoveryState::Normal
        || contribution.effects().labels() != [expected_origin_path]
    {
        return Err("Task258B2 resolver theorem provenance mismatch".to_owned());
    }

    let reserve =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .map_err(|()| "Task258B2 reserve extraction failed".to_owned())?;
    let base_binding_env = reserve
        .bridge
        .prepare_binding_env(symbols)
        .map_err(|error| error.to_string())?;
    let mut contexts = base_binding_env.contexts().clone();
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: extracted.proof_range,
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    if proof != BindingContextId::new(1) {
        return Err("Task258B2 binding context identity drift".to_owned());
    }
    let binding_env = BindingEnv::try_new(BindingEnvParts {
        source_id: ast.source_id,
        module_id: module.clone(),
        contexts,
        bindings: base_binding_env.bindings().clone(),
        diagnostics: base_binding_env.diagnostics().clone(),
    })
    .map_err(|error| error.to_string())?;
    let mut owned_node_kinds = BTreeMap::new();
    for (site, kind) in extracted.statement_sites.iter().zip([
        "source.statement.theorem",
        "source.statement.assumption",
        "source.statement.conclusion",
    ]) {
        owned_node_kinds.insert(site.node().index(), kind);
    }
    for site in &extracted.formula_sites {
        owned_node_kinds.insert(site.node().index(), "source.formula.atomic.equality");
    }
    let term_contexts = [0, 0, 1, 1, 1, 1];
    let roots = extracted
        .term_sites
        .iter()
        .zip(term_contexts)
        .map(|(site, context)| (site.node().index(), BindingContextId::new(context)));
    let parts = source_term_parts_for_context_roots(
        ast,
        module.clone(),
        &binding_env,
        roots,
        &owned_node_kinds,
    )?;
    let primary = parts.handoff;
    let arena = parts.arena;
    if primary
        .terms()
        .iter()
        .zip(extracted.term_ranges)
        .any(|((_, term), expected)| term.source_range() != expected)
        || primary
            .references()
            .iter()
            .any(|(_, reference)| reference.use_ordinal() != 1)
    {
        return Err("Task258B2 primary-term profile drift".to_owned());
    }
    let atomic = SourceAtomicFormulaProducer::build(
        task258b2_atomic_input(ast, module.clone(), &extracted),
        &binding_env,
        symbols,
        &primary,
        None,
        None,
        None,
        &arena,
    )
    .map_err(|error| error.to_string())?;
    let statement = task258b2_statement_input(
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
    let reference_use_ordinals = inputs
        .primary
        .references()
        .iter()
        .map(|(_, reference)| reference.use_ordinal())
        .collect::<Vec<_>>();
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
    if typed_ast.nodes().len() != 55
        || typed_ast
            .nodes()
            .root()
            .is_none_or(|root| root.index() != 54)
        || typed_ast.source_statement().is_none()
        || typed_ast.source_statement_references().is_some()
        || typed_ast.source_statement() != resolved.source_statement()
        || resolved.source_statement_references().is_some()
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
        return Err("Task258B2 immutable final handoff mismatch".to_owned());
    }
    Ok(SourceStatementRouteOutput {
        typed_ast,
        resolved,
        left_lookup_ordinal: reference_use_ordinals[0],
        right_lookup_ordinal: reference_use_ordinals[1],
        reference_use_ordinals,
    })
}

fn build_source_statement_b1_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB1Extraction,
    mutate: impl FnOnce(&mut SourceStatementB1RouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    if symbols.module_id() != &module {
        return Err("Task258B1 symbol module mismatch".to_owned());
    }
    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B1_LABEL)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner_entry] = owners.as_slice() else {
        return Err("Task258B1 requires one exact resolver theorem owner".to_owned());
    };
    let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner_entry.symbol().clone(),
        ast.source_id,
        &module,
    )
    .map_err(|error| error.to_string())?;
    if checked_owner.source_range() != extracted.theorem_range
        || extracted.label_range != range(ast.source_id, 27, 61)
    {
        return Err("Task258B1 resolver theorem provenance mismatch".to_owned());
    }
    let reserve =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .map_err(|()| "Task258B1 reserve extraction failed".to_owned())?;
    let base_binding_env = reserve
        .bridge
        .prepare_binding_env(symbols)
        .map_err(|error| error.to_string())?;
    let mut contexts = base_binding_env.contexts().clone();
    let outer = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: extracted.proof_ranges[0],
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    let nested = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: extracted.proof_ranges[1],
        },
        parent: Some(outer),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    if outer != BindingContextId::new(1) || nested != BindingContextId::new(2) {
        return Err("Task258B1 binding context identity drift".to_owned());
    }
    let binding_env = BindingEnv::try_new(BindingEnvParts {
        source_id: ast.source_id,
        module_id: module.clone(),
        contexts,
        bindings: base_binding_env.bindings().clone(),
        diagnostics: base_binding_env.diagnostics().clone(),
    })
    .map_err(|error| error.to_string())?;
    let mut owned_node_kinds = BTreeMap::new();
    for (site, kind) in extracted.statement_sites.iter().zip([
        "source.statement.theorem",
        "source.statement.proof-step",
        "source.statement.conclusion",
        "source.statement.conclusion",
    ]) {
        owned_node_kinds.insert(site.node().index(), kind);
    }
    for site in &extracted.formula_sites {
        owned_node_kinds.insert(site.node().index(), "source.formula.atomic.equality");
    }
    let term_contexts = [0, 0, 1, 1, 2, 2, 1, 1];
    let roots = extracted
        .term_sites
        .iter()
        .zip(term_contexts)
        .map(|(site, context)| (site.node().index(), BindingContextId::new(context)));
    let parts = source_term_parts_for_context_roots(
        ast,
        module.clone(),
        &binding_env,
        roots,
        &owned_node_kinds,
    )?;
    let primary = parts.handoff;
    let arena = parts.arena;
    if primary
        .terms()
        .iter()
        .zip(extracted.term_ranges)
        .any(|((_, term), expected)| term.source_range() != expected)
    {
        return Err("Task258B1 primary-term range drift".to_owned());
    }
    let atomic = SourceAtomicFormulaProducer::build(
        task258b1_atomic_input(ast, module.clone(), &extracted),
        &binding_env,
        symbols,
        &primary,
        None,
        None,
        None,
        &arena,
    )
    .map_err(|error| error.to_string())?;
    let statement = task258b1_statement_input(
        ast,
        module.clone(),
        owner_entry.symbol().clone(),
        owner_entry.contribution(),
        &extracted,
    );
    let (resolver_ast, projection, reference, resolution) =
        task258b1_resolver_bundle(ast, &module, owner_entry.contribution())?;
    let reference_input = SourceStatementReferenceHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        labels: vec![SourceStatementLabelInput {
            statement: SourceStatementId::new(1),
            context: SourceStatementContextId::new(1),
            candidate: SourceStatementCandidateFactId::new(1),
            origin_path: projection.origin_path().clone(),
            proof_scope: LabelScopePath::new(vec![0]),
            source_range: range(ast.source_id, 77, 78),
            source_ordinal: 0,
            visible_after_ordinal: 1,
            spelling: "A".to_owned(),
            kind: SourceStatementLabelKind::ProofStep,
            recovery: SourceStatementRecovery::Normal,
        }],
        citations: vec![SourceStatementCitationInput {
            statement: SourceStatementId::new(3),
            context: SourceStatementContextId::new(3),
            label: SourceStatementLabelId::new(0),
            label_ref: resolution.ids()[0],
            proof_scope: LabelScopePath::new(vec![0]),
            source_range: range(ast.source_id, 131, 132),
            ordinal: 0,
            kind: SourceStatementCitationKind::SimpleLocal,
            recovery: SourceStatementRecovery::Normal,
        }],
    };
    let mut inputs = SourceStatementB1RouteInputs {
        binding_env,
        arena,
        primary,
        atomic,
        statement,
        resolver_ast,
        projection,
        reference,
        resolution,
        reference_input,
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
    let references = SourceStatementReferenceProducer::build(
        inputs.reference_input,
        &statement,
        &inputs.resolver_ast,
        &inputs.projection,
        &inputs.reference,
        &inputs.resolution,
        &inputs.arena,
    )
    .map_err(|error| error.to_string())?;
    let reference_use_ordinals = inputs
        .primary
        .references()
        .iter()
        .map(|(_, reference)| reference.use_ordinal())
        .collect::<Vec<_>>();
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
    .with_source_statement_references(statement, references)
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
        || typed_ast.source_statement_references().is_none()
        || typed_ast.source_statement() != resolved.source_statement()
        || typed_ast.source_statement_references() != resolved.source_statement_references()
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
        return Err("Task258B1 immutable final handoff mismatch".to_owned());
    }
    Ok(SourceStatementRouteOutput {
        typed_ast,
        resolved,
        left_lookup_ordinal: reference_use_ordinals[0],
        right_lookup_ordinal: reference_use_ordinals[1],
        reference_use_ordinals,
    })
}

fn task258b1_resolver_bundle(
    ast: &SurfaceAst,
    module: &ModuleId,
    contribution: mizar_resolve::env::SourceContributionId,
) -> Result<
    (
        ResolvedAst,
        LabelProjection,
        LabelReferenceCandidate,
        mizar_resolve::labels::LabelResolutionResult,
    ),
    String,
> {
    let preliminary = task258b1_resolved_arena(ast, module, None)?;
    let preliminary_keyed_nodes = preliminary
        .iter()
        .filter(|(_, node)| node.reference_key().is_some())
        .count();
    if preliminary.root().index() != 76
        || preliminary.len() != 77
        || preliminary_keyed_nodes != 0
        || preliminary.iter().any(|(id, node)| {
            node.kind() != &ast.nodes()[id.index()].kind
                || node
                    .children()
                    .iter()
                    .map(|child| child.index())
                    .ne(ast.nodes()[id.index()]
                        .children
                        .iter()
                        .map(|child| child.index()))
                || node.origin().source_id() != ast.source_id
                || node.origin().module_id() != module
                || node.origin().import_edge().is_some()
                || node.origin().structural_path() != [id.index() as u32]
                || node.origin().anchor() != &SourceAnchor::Range(ast.nodes()[id.index()].range)
                || node.origin().is_recovered()
                || node.recovery() != RecoveryState::Normal
                || node.resolution() != NodeResolutionState::NotApplicable
        })
    {
        return Err("Task258B1 preliminary resolver arena drift".to_owned());
    }
    let reference_node = preliminary
        .iter()
        .find_map(|(id, _)| (id.index() == 68).then_some(id))
        .ok_or_else(|| "Task258B1 preliminary reference node is missing".to_owned())?;
    let namespace = NamespacePath::new(module.path().as_str());
    let origin_path = LabelOriginPath::new(format!(
        "{}::{}::proof::A",
        module.package().as_str(),
        module.path().as_str()
    ));
    let projection = LabelProjection::proof_step(
        LabelProjectionData {
            origin_path,
            module: module.clone(),
            namespace: namespace.clone(),
            primary_spelling: "A".to_owned(),
            kind: LabelKind::ProofStep,
            declaration_range: range(ast.source_id, 77, 78),
            origin: SemanticOrigin::new(
                ast.source_id,
                module.clone(),
                SourceAnchor::Range(range(ast.source_id, 77, 78)),
                vec![12],
            ),
            contribution,
        },
        1,
        LabelScopePath::new(vec![0]),
    );
    let reference = LabelReferenceCandidate::unqualified_citation(
        ReferenceSite::new(reference_node, range(ast.source_id, 131, 132), "A"),
        SemanticOrigin::new(
            ast.source_id,
            module.clone(),
            SourceAnchor::Range(range(ast.source_id, 131, 132)),
            vec![68],
        ),
        3,
        Some(LabelScopePath::new(vec![0])),
    );
    let resolution = LabelResolver::new(std::slice::from_ref(&projection)).resolve(
        module,
        &namespace,
        std::slice::from_ref(&reference),
    );
    let [label_ref] = resolution.ids() else {
        return Err("Task258B1 resolver did not produce one reference id".to_owned());
    };
    let final_arena = task258b1_resolved_arena(ast, module, Some(*label_ref))?;
    let resolver_ast = ResolvedAst::try_new(
        ast.source_id,
        module.clone(),
        final_arena,
        NameRefTable::new(),
        resolution.table().clone(),
        ResolvedImports::new(),
    )
    .map_err(|error| error.to_string())?;
    if resolver_ast.nodes().root().index() != 76
        || resolver_ast.nodes().len() != 77
        || resolver_ast
            .nodes()
            .iter()
            .any(|(id, node)| node.kind() != &ast.nodes()[id.index()].kind)
    {
        return Err("Task258B1 resolver arena lost parser identity".to_owned());
    }
    Ok((resolver_ast, projection, reference, resolution))
}

fn task258b1_resolved_arena(
    ast: &SurfaceAst,
    module: &ModuleId,
    label_ref: Option<mizar_resolve::resolved_ast::LabelRefId>,
) -> Result<ResolvedArena, String> {
    let mut builder = ResolvedArenaBuilder::new();
    let mut ids = Vec::with_capacity(ast.nodes().len());
    for (index, node) in ast.nodes().iter().enumerate() {
        let children = node
            .children
            .iter()
            .map(|child| {
                ids.get(child.index())
                    .copied()
                    .ok_or_else(|| "Task258B1 resolver child order drift".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let origin = SemanticOrigin::new(
            ast.source_id,
            module.clone(),
            SourceAnchor::Range(node.range),
            vec![index as u32],
        );
        let mut resolved = ResolvedNode::new(node.kind.clone(), children, origin);
        if node.recovered {
            resolved = resolved.with_recovery(RecoveryState::Recovered);
        }
        if index == 68
            && let Some(label_ref) = label_ref
        {
            resolved = resolved
                .with_resolution(NodeResolutionState::Resolved)
                .with_reference_key(NodeReferenceKey::Label(label_ref));
        }
        let id = builder.push(resolved).map_err(|error| error.to_string())?;
        if id.index() != index {
            return Err("Task258B1 resolver node identity drift".to_owned());
        }
        ids.push(id);
    }
    let root = ast
        .root()
        .and_then(|root| ids.get(root.index()).copied())
        .ok_or_else(|| "Task258B1 resolver root is missing".to_owned())?;
    builder.finish(root).map_err(|error| error.to_string())
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

fn task258b3_atomic_input(
    ast: &SurfaceAst,
    module: ModuleId,
    extracted: &SourceStatementWitnessExtraction,
) -> SourceAtomicFormulaHandoffInput {
    let formulas = (0..2)
        .map(|index| SourceAtomicFormulaInput {
            site: extracted.formula_sites[index].clone(),
            source_range: extracted.formula_ranges[index],
            source_ordinal: index,
            context: BindingContextId::new(index),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = x".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        })
        .collect();
    let mut edges = Vec::new();
    let mut requests = Vec::new();
    for formula in 0..2 {
        let first_term = if formula == 0 { 0 } else { 3 };
        for ordinal in 0..2 {
            let edge = SourceAtomicEdgeId::new(formula * 2 + ordinal);
            edges.push(SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(formula),
                ordinal,
                role: if ordinal == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                },
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                    first_term + ordinal,
                )),
            });
            requests.push(SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(formula),
                ordinal,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(edge),
                candidate: None,
                type_site: None,
                attribute: None,
            });
        }
    }
    SourceAtomicFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        formulas,
        wrappers: Vec::new(),
        predicate_segments: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges,
        requests,
    }
}

fn task258b3_statement_input(
    ast: &SurfaceAst,
    module: ModuleId,
    symbol: mizar_resolve::resolved_ast::SymbolId,
    contribution: mizar_resolve::env::SourceContributionId,
    extracted: &SourceStatementWitnessExtraction,
) -> SourceStatementHandoffInput {
    let kinds = [
        SourceStatementKind::TheoremProposition,
        SourceStatementKind::Conclusion,
    ];
    let source_ordinals = [0, 2];
    SourceStatementHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        owners: vec![SourceTheoremOwnerInput {
            symbol,
            contribution,
            site: extracted.theorem_site.clone(),
            source_range: extracted.theorem_range,
            spelling: extracted.label.to_owned(),
            role: SourceTheoremRole::Theorem,
            status: SourceTheoremStatus::Unmodified,
            recovery: SourceStatementRecovery::Normal,
        }],
        statements: (0..2)
            .map(|index| SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(index),
                formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index)),
                site: extracted.statement_sites[index].clone(),
                source_range: extracted.statement_ranges[index],
                source_ordinal: source_ordinals[index],
                spelling: extracted.spellings[index].to_owned(),
                kind: kinds[index],
                recovery: SourceStatementRecovery::Normal,
            })
            .collect(),
        contexts: (0..2)
            .map(|index| SourceStatementContextInput {
                statement: SourceStatementId::new(index),
                binding_context: BindingContextId::new(index),
                source_range: extracted.statement_ranges[index],
                visible_bindings: vec![BindingId::new(0)],
            })
            .collect(),
        input_facts: (0..2)
            .map(|index| {
                let first_term = if index == 0 { 0 } else { 3 };
                SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first_term),
                        SourcePrimaryTermReferenceId::new(first_term + 1),
                    ],
                }
            })
            .collect(),
        candidate_facts: (0..2)
            .map(|index| SourceStatementCandidateFactInput {
                statement: SourceStatementId::new(index),
                context: SourceStatementContextId::new(index),
                ordinal: 0,
                kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index)),
            })
            .collect(),
    }
}

fn task258b2_atomic_input(
    ast: &SurfaceAst,
    module: ModuleId,
    extracted: &SourceStatementB2Extraction,
) -> SourceAtomicFormulaHandoffInput {
    let contexts = [0, 1, 1];
    let formulas = (0..3)
        .map(|index| SourceAtomicFormulaInput {
            site: extracted.formula_sites[index].clone(),
            source_range: extracted.formula_ranges[index],
            source_ordinal: index,
            context: BindingContextId::new(contexts[index]),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = x".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        })
        .collect();
    let mut edges = Vec::new();
    let mut requests = Vec::new();
    for formula in 0..3 {
        for ordinal in 0..2 {
            let edge = SourceAtomicEdgeId::new(formula * 2 + ordinal);
            edges.push(SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(formula),
                ordinal,
                role: if ordinal == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                },
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                    formula * 2 + ordinal,
                )),
            });
            requests.push(SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(formula),
                ordinal,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(edge),
                candidate: None,
                type_site: None,
                attribute: None,
            });
        }
    }
    SourceAtomicFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        formulas,
        wrappers: Vec::new(),
        predicate_segments: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges,
        requests,
    }
}

fn task258b2_statement_input(
    ast: &SurfaceAst,
    module: ModuleId,
    symbol: mizar_resolve::resolved_ast::SymbolId,
    contribution: mizar_resolve::env::SourceContributionId,
    extracted: &SourceStatementB2Extraction,
) -> SourceStatementHandoffInput {
    let kinds = [
        SourceStatementKind::TheoremProposition,
        SourceStatementKind::Assumption,
        SourceStatementKind::Conclusion,
    ];
    let binding_contexts = [0, 1, 1];
    SourceStatementHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        owners: vec![SourceTheoremOwnerInput {
            symbol,
            contribution,
            site: extracted.theorem_site.clone(),
            source_range: extracted.theorem_range,
            spelling: SOURCE_STATEMENT_B2_LABEL.to_owned(),
            role: SourceTheoremRole::Theorem,
            status: SourceTheoremStatus::Unmodified,
            recovery: SourceStatementRecovery::Normal,
        }],
        statements: (0..3)
            .map(|index| SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(index),
                formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index)),
                site: extracted.statement_sites[index].clone(),
                source_range: extracted.statement_ranges[index],
                source_ordinal: index,
                spelling: SOURCE_STATEMENT_B2_SPELLINGS[index].to_owned(),
                kind: kinds[index],
                recovery: SourceStatementRecovery::Normal,
            })
            .collect(),
        contexts: (0..3)
            .map(|index| SourceStatementContextInput {
                statement: SourceStatementId::new(index),
                binding_context: BindingContextId::new(binding_contexts[index]),
                source_range: extracted.statement_ranges[index],
                visible_bindings: vec![BindingId::new(0)],
            })
            .collect(),
        input_facts: (0..3)
            .map(|index| SourceStatementInputFactInput {
                statement: SourceStatementId::new(index),
                context: SourceStatementContextId::new(index),
                ordinal: 0,
                kind: SourceStatementInputFactKind::ReservedTypeGuard,
                binding: BindingId::new(0),
                uses: vec![
                    SourcePrimaryTermReferenceId::new(index * 2),
                    SourcePrimaryTermReferenceId::new(index * 2 + 1),
                ],
            })
            .collect(),
        candidate_facts: (0..3)
            .map(|index| SourceStatementCandidateFactInput {
                statement: SourceStatementId::new(index),
                context: SourceStatementContextId::new(index),
                ordinal: 0,
                kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index)),
            })
            .collect(),
    }
}

fn task258b1_atomic_input(
    ast: &SurfaceAst,
    module: ModuleId,
    extracted: &SourceStatementB1Extraction,
) -> SourceAtomicFormulaHandoffInput {
    let contexts = [0, 1, 2, 1];
    let formulas = (0..4)
        .map(|index| SourceAtomicFormulaInput {
            site: extracted.formula_sites[index].clone(),
            source_range: extracted.formula_ranges[index],
            source_ordinal: index,
            context: BindingContextId::new(contexts[index]),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = x".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        })
        .collect();
    let mut edges = Vec::new();
    let mut requests = Vec::new();
    for formula in 0..4 {
        for ordinal in 0..2 {
            let edge = SourceAtomicEdgeId::new(formula * 2 + ordinal);
            edges.push(SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(formula),
                ordinal,
                role: if ordinal == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                },
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                    formula * 2 + ordinal,
                )),
            });
            requests.push(SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(formula),
                ordinal,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(edge),
                candidate: None,
                type_site: None,
                attribute: None,
            });
        }
    }
    SourceAtomicFormulaHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        formulas,
        wrappers: Vec::new(),
        predicate_segments: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges,
        requests,
    }
}

fn task258b1_statement_input(
    ast: &SurfaceAst,
    module: ModuleId,
    symbol: mizar_resolve::resolved_ast::SymbolId,
    contribution: mizar_resolve::env::SourceContributionId,
    extracted: &SourceStatementB1Extraction,
) -> SourceStatementHandoffInput {
    let kinds = [
        SourceStatementKind::TheoremProposition,
        SourceStatementKind::ProofStepProposition,
        SourceStatementKind::Conclusion,
        SourceStatementKind::Conclusion,
    ];
    let binding_contexts = [0, 1, 2, 1];
    SourceStatementHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        owners: vec![SourceTheoremOwnerInput {
            symbol,
            contribution,
            site: extracted.theorem_site.clone(),
            source_range: extracted.theorem_range,
            spelling: SOURCE_STATEMENT_B1_LABEL.to_owned(),
            role: SourceTheoremRole::Theorem,
            status: SourceTheoremStatus::Unmodified,
            recovery: SourceStatementRecovery::Normal,
        }],
        statements: (0..4)
            .map(|index| SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(index),
                formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index)),
                site: extracted.statement_sites[index].clone(),
                source_range: extracted.statement_ranges[index],
                source_ordinal: index,
                spelling: SOURCE_STATEMENT_B1_SPELLINGS[index].to_owned(),
                kind: kinds[index],
                recovery: SourceStatementRecovery::Normal,
            })
            .collect(),
        contexts: (0..4)
            .map(|index| SourceStatementContextInput {
                statement: SourceStatementId::new(index),
                binding_context: BindingContextId::new(binding_contexts[index]),
                source_range: extracted.statement_ranges[index],
                visible_bindings: vec![BindingId::new(0)],
            })
            .collect(),
        input_facts: (0..4)
            .map(|index| SourceStatementInputFactInput {
                statement: SourceStatementId::new(index),
                context: SourceStatementContextId::new(index),
                ordinal: 0,
                kind: SourceStatementInputFactKind::ReservedTypeGuard,
                binding: BindingId::new(0),
                uses: vec![
                    SourcePrimaryTermReferenceId::new(index * 2),
                    SourcePrimaryTermReferenceId::new(index * 2 + 1),
                ],
            })
            .collect(),
        candidate_facts: (0..4)
            .map(|index| SourceStatementCandidateFactInput {
                statement: SourceStatementId::new(index),
                context: SourceStatementContextId::new(index),
                ordinal: 0,
                kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index)),
            })
            .collect(),
    }
}

const fn range(source_id: mizar_session::SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
