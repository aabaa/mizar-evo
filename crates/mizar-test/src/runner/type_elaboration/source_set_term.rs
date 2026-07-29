use std::collections::{BTreeMap, BTreeSet};

use mizar_checker::{
    binding_env::{BindingContextId, BindingEnv},
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_application::{SourceFunctorApplicationHandoff, SourceFunctorApplicationId},
    source_context::{
        SourceBindingContextBuild, SourceBindingContextInput, SourceBindingContextOwner,
        SourceBindingContextProducer, SourceBindingSiteInput, SourceBindingSiteRole,
        SourceItemInput, SourceItemRecovery, SourceItemRole, SourceItemVisibility,
    },
    source_set_term::{
        SourceSetConditionInput, SourceSetEdgeInput, SourceSetEdgeRole, SourceSetGeneratorId,
        SourceSetGeneratorInput, SourceSetRequestInput, SourceSetRequestKind, SourceSetTarget,
        SourceSetTermHandoffInput, SourceSetTermId, SourceSetTermInput, SourceSetTermKind,
        SourceSetTermProducer, SourceSetTermRecovery, SourceSetTypeHead, SourceSetTypeOwner,
        SourceSetTypeRole, SourceSetTypeSiteId, SourceSetTypeSiteInput, SourceSetWrapperInput,
    },
    source_structure::{SourceStructureHandoff, SourceStructureTermId},
    source_term::SourcePrimaryTermHandoff,
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedAst, TypedAstParts,
        TypedNodeId, TypedSiteRef,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShell, DeclarationShellKind, DeclarationShellSet},
    env::SymbolEnv,
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::ModuleId,
};
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

#[cfg(not(test))]
use super::source_term::source_term_parts_for_roots;
#[cfg(test)]
use super::source_term::{
    source_term_parts_for_context_roots, synthetic_source_term_parts_for_roots,
};
use super::{
    checker_handoff::{assemble_empty_resolved_typed_ast, source_module_binding_env},
    source_application::{
        unwrapped_imported_source_application_handoff,
        unwrapped_imported_source_application_owned_node_kinds,
    },
    source_ast::{
        direct_token_texts, exact_compilation_item_list, is_exact_parser_type_fixtures_import,
        structural_child_ids, subtree_has_recovery, surface_site,
    },
    source_reserve::extract_builtin_source_reserve_declarations_after_node_guard,
    source_term::SourceTermParts,
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";
const PAYLOAD_EXTRACTION_GAP_KEY: &str =
    "type_elaboration.external_dependency.ast_payload_extraction";

#[derive(Debug)]
pub(in crate::runner) struct SourceSetTermRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
    #[cfg(test)]
    pub(in crate::runner) binding_env: BindingEnv,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SyntheticSourceSetTermDependencies {
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: SourcePrimaryTermHandoff,
    pub(in crate::runner) application: Option<SourceFunctorApplicationHandoff>,
    pub(in crate::runner) structure: Option<SourceStructureHandoff>,
}

#[derive(Debug, Clone)]
struct ExtractedSetTerms {
    context: BindingContextId,
    terms: Vec<ExtractedTerm>,
    wrappers: Vec<ExtractedWrapper>,
    generators: Vec<ExtractedGenerator>,
    type_sites: Vec<ExtractedTypeSite>,
    conditions: Vec<ExtractedCondition>,
    edges: Vec<ExtractedEdge>,
    primary_roots: Vec<usize>,
}

#[derive(Debug, Clone)]
struct ExtractedTerm {
    node: usize,
    kind: SourceSetTermKind,
    recovery: SourceSetTermRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedWrapper {
    term: SourceSetTermId,
    ordinal: usize,
    node: usize,
    recovery: SourceSetTermRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedGenerator {
    term: SourceSetTermId,
    ordinal: usize,
    node: usize,
    recovery: SourceSetTermRecovery,
    type_site: SourceSetTypeSiteId,
}

#[derive(Debug, Clone)]
struct ExtractedTypeSite {
    owner: SourceSetTypeOwner,
    node: usize,
    head_node: usize,
    head: SourceSetTypeHead,
    recovery: SourceSetTermRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedCondition {
    term: SourceSetTermId,
    ordinal: usize,
    colon_node: usize,
    condition_node: usize,
    recovery: SourceSetTermRecovery,
}

#[derive(Debug, Clone, Copy)]
enum ExtractedTarget {
    Primary(usize),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
    SetTerm(SourceSetTermId),
}

#[derive(Debug, Clone)]
struct ExtractedEdge {
    term: SourceSetTermId,
    ordinal: usize,
    role: SourceSetEdgeRole,
    target: ExtractedTarget,
}

const TASK255C1_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "definition\n",
    "  func Task255ConditionedComprehensionDef:\n",
    "    task255_conditioned_comprehension -> set\n",
    "    equals { 1 ++ 2 where candidate255c is set : 3 = 4 };\n",
    "end;\n",
);

#[cfg(test)]
const TASK258B3M2B2B3P_SOURCE: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementSetEnumerationWitnessSmoke: x = x proof\n",
    "  take {1, 2};\n",
    "  thus x = x;\n",
    "end;\n",
);

#[derive(Debug, Clone)]
struct ExactConditionedRoute {
    root: usize,
    application: usize,
    primary_roots: Vec<usize>,
}

/// Runs the bounded Task-255 source transport without replacing the existing
/// definition-extraction diagnostic owner.
pub(in crate::runner) fn source_set_term_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_set_term_output_with_optional_source(
        ast,
        module,
        shells,
        symbols,
        Some(source_text),
    ) {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_set_term().is_some()
                && output.typed_ast.source_set_term() == output.resolved.source_set_term()
                && output.typed_ast.source_term() == output.resolved.source_term() =>
        {
            Some(vec![PAYLOAD_EXTRACTION_GAP_KEY.to_owned()])
        }
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_set_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(ast, module, shells, symbols, None, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_set_term_output_with_source(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(
        ast,
        module,
        shells,
        symbols,
        Some(source_text),
        |_| {},
    )
}

fn source_set_term_output_with_optional_source(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: Option<&str>,
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(ast, module, shells, symbols, source_text, |_| {})
}

pub(super) fn conditioned_source_set_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    let route = exact_conditioned_route(ast, source_text)?;
    Some(build_conditioned_output(
        ast,
        module,
        symbols,
        route,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_set_term_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(ast, module, shells, symbols, None, mutate)
}

#[cfg(test)]
pub(in crate::runner) fn source_set_term_output_with_source_and_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(
        ast,
        module,
        shells,
        symbols,
        Some(source_text),
        mutate,
    )
}

fn source_set_term_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: Option<&str>,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    if let Some(source_text) = source_text
        && let Some(route) = exact_conditioned_route(ast, source_text)
    {
        return Some(build_conditioned_output(
            ast, module, symbols, route, mutate,
        ));
    }
    let roots = exact_real_roots(ast)?;
    let binding_env = match source_set_term_binding_env(ast, module.clone(), shells, symbols) {
        Ok(binding_env) => binding_env,
        Err(error) => return Some(Err(error)),
    };
    let extracted = match extract_set_terms(
        ast,
        &module,
        BindingContextId::new(1),
        &roots,
        None,
        None,
        &BTreeSet::new(),
        false,
    ) {
        Ok(extracted) => extracted,
        Err(error) => return Some(Err(error)),
    };
    Some(build_output(
        ast,
        module,
        binding_env,
        extracted,
        None,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3P_RESOLVER_FIELD_COUNT: usize = 63;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3P_BINDING_FIELD_COUNT: usize = 39;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SetEnumerationSurfaceMutation {
    None,
    NodeKind(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    RootIdentity,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SetEnumerationResolverMutation {
    None,
    Field(usize),
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SetEnumerationSelectionStage {
    Source,
    Surface,
    Resolver,
    Selected,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SetEnumerationBindingMutation {
    None,
    Field(usize),
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SetEnumerationPrimaryMutation {
    None,
    DuplicateRoot,
    MissingRoot,
    SourceId,
    ModuleId,
    TermSite(usize),
    TermRange(usize),
    TermOrdinal(usize),
    TermContext(usize),
    TermRecovery(usize),
    TermSpelling(usize),
    TermKind(usize),
    TermRole(usize),
    TermParent(usize),
    ReferenceTerm(usize),
    ReferenceBinding(usize),
    ReferenceRole(usize),
    ReferenceScopeModule,
    ReferenceScopeProof,
    ReferenceUseOrdinal,
    NumericTerm(usize),
    NumericOwner(usize),
    NumericRange(usize),
    NumericSpelling(usize),
    NumericOrdinal(usize),
    StaleFingerprintReplay,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SetEnumerationHandoffMutation {
    None,
    SourceId,
    ModuleId,
    TermSite,
    TermRange,
    TermOrdinal,
    TermContext,
    TermRecovery,
    TermSpelling,
    TermKind,
    ExtraWrapper,
    ExtraGenerator,
    ExtraTypeSite,
    ExtraCondition,
    EdgeTerm(usize),
    EdgeOrdinal(usize),
    EdgeRole(usize),
    EdgeTarget(usize),
    RequestTerm,
    RequestOrdinal,
    RequestKind,
    RequestGenerator,
    RequestTypeSite,
    CoherentApplicationFingerprint,
    CoherentStructureFingerprint,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SetEnumerationFinalMutation {
    None,
    TypedClone,
    ResolvedClone,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) struct SetEnumerationProofContextTestOptions {
    pub(in crate::runner) surface: SetEnumerationSurfaceMutation,
    pub(in crate::runner) resolver: SetEnumerationResolverMutation,
    pub(in crate::runner) binding: SetEnumerationBindingMutation,
    pub(in crate::runner) primary: SetEnumerationPrimaryMutation,
    pub(in crate::runner) handoff: SetEnumerationHandoffMutation,
    pub(in crate::runner) final_clone: SetEnumerationFinalMutation,
}

#[cfg(test)]
impl Default for SetEnumerationProofContextTestOptions {
    fn default() -> Self {
        Self {
            surface: SetEnumerationSurfaceMutation::None,
            resolver: SetEnumerationResolverMutation::None,
            binding: SetEnumerationBindingMutation::None,
            primary: SetEnumerationPrimaryMutation::None,
            handoff: SetEnumerationHandoffMutation::None,
            final_clone: SetEnumerationFinalMutation::None,
        }
    }
}

#[cfg(test)]
// Rationale: keep every ordered B3P selection-stage authority input explicit.
#[allow(clippy::too_many_arguments)]
pub(in crate::runner) fn set_enumeration_selection_stage_for_test(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    loaded_source: &str,
    surface: SetEnumerationSurfaceMutation,
    resolver: SetEnumerationResolverMutation,
) -> SetEnumerationSelectionStage {
    if loaded_source != TASK258B3M2B2B3P_SOURCE || loaded_source.len() != 117 {
        return SetEnumerationSelectionStage::Source;
    }
    if !task258b3m2b2b3p_surface_contract(ast, loaded_source, surface) {
        return SetEnumerationSelectionStage::Surface;
    }
    if !task258b3m2b2b3p_resolver_contract(ast, module, shells, symbols, resolver) {
        return SetEnumerationSelectionStage::Resolver;
    }
    SetEnumerationSelectionStage::Selected
}

#[cfg(test)]
fn task258b3m2b2b3p_surface_contract(
    ast: &SurfaceAst,
    loaded_source: &str,
    mutation: SetEnumerationSurfaceMutation,
) -> bool {
    const KINDS: [&str; 57] = [
        "Token(SurfaceToken { kind: ReservedWord, text: \"reserve\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"FormulaStatementSetEnumerationWitnessSmoke\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"proof\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"take\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"{\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"1\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \",\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"2\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"}\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"thus\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"end\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "TypeHead",
        "TypeExpression",
        "ReserveSegment",
        "ReserveItem",
        "TermReference",
        "TermExpression",
        "TermReference",
        "TermExpression",
        "BuiltinPredicateApplication",
        "FormulaExpression",
        "NumeralTerm",
        "TermExpression",
        "NumeralTerm",
        "TermExpression",
        "SetEnumeration",
        "TermExpression",
        "Witness",
        "TakeStatement",
        "TermReference",
        "TermExpression",
        "TermReference",
        "TermExpression",
        "BuiltinPredicateApplication",
        "FormulaExpression",
        "Proposition",
        "ConclusionStatement",
        "ProofBlock",
        "TheoremItem",
        "ItemList",
        "CompilationUnit",
        "Root",
    ];
    const RANGES: [(usize, usize); 57] = [
        (0, 7),
        (8, 9),
        (10, 13),
        (14, 17),
        (17, 18),
        (19, 26),
        (27, 69),
        (69, 70),
        (71, 72),
        (73, 74),
        (75, 76),
        (77, 82),
        (85, 89),
        (90, 91),
        (91, 92),
        (92, 93),
        (94, 95),
        (95, 96),
        (96, 97),
        (100, 104),
        (105, 106),
        (107, 108),
        (109, 110),
        (110, 111),
        (112, 115),
        (115, 116),
        (14, 17),
        (14, 17),
        (8, 17),
        (0, 18),
        (71, 72),
        (71, 72),
        (75, 76),
        (75, 76),
        (71, 76),
        (71, 76),
        (91, 92),
        (91, 92),
        (94, 95),
        (94, 95),
        (90, 96),
        (90, 96),
        (90, 96),
        (85, 97),
        (105, 106),
        (105, 106),
        (109, 110),
        (109, 110),
        (105, 110),
        (105, 110),
        (105, 110),
        (100, 111),
        (77, 115),
        (19, 116),
        (0, 116),
        (0, 116),
        (0, 116),
    ];
    const CHILDREN: [&[usize]; 57] = [
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[3],
        &[26],
        &[1, 2, 27],
        &[0, 28, 4],
        &[8],
        &[30],
        &[10],
        &[32],
        &[31, 9, 33],
        &[34],
        &[14],
        &[36],
        &[16],
        &[38],
        &[13, 37, 15, 39, 17],
        &[40],
        &[41],
        &[12, 42, 18],
        &[20],
        &[44],
        &[22],
        &[46],
        &[45, 21, 47],
        &[48],
        &[49],
        &[19, 50, 23],
        &[11, 43, 51, 24],
        &[5, 6, 7, 35, 52, 25],
        &[29, 53],
        &[54],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 55,
        ],
    ];
    if loaded_source != TASK258B3M2B2B3P_SOURCE || loaded_source.len() != 117 {
        return false;
    }
    let mut kinds = KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let mut ranges = RANGES.to_vec();
    let mut recoveries = [false; 57];
    let mut children = CHILDREN
        .iter()
        .map(|children| children.to_vec())
        .collect::<Vec<_>>();
    let mut root = Some(56);
    match mutation {
        SetEnumerationSurfaceMutation::None => {}
        SetEnumerationSurfaceMutation::NodeKind(index) => {
            if let Some(kind) = kinds.get_mut(index) {
                kind.push('!');
            }
        }
        SetEnumerationSurfaceMutation::NodeRange(index) => {
            if let Some(range) = ranges.get_mut(index) {
                range.1 = range.1.saturating_add(1);
            }
        }
        SetEnumerationSurfaceMutation::NodeRecovery(index) => {
            if let Some(recovered) = recoveries.get_mut(index) {
                *recovered = !*recovered;
            }
        }
        SetEnumerationSurfaceMutation::NodeChildren(index) => {
            if let Some(node_children) = children.get_mut(index) {
                if node_children.len() > 1 {
                    node_children.rotate_left(1);
                } else {
                    node_children.push(index);
                }
            }
        }
        SetEnumerationSurfaceMutation::RootIdentity => root = None,
    }
    ast.nodes().len() == 57
        && ast.root().map(|root| root.index()) == root
        && ast.nodes().iter().enumerate().all(|(index, node)| {
            format!("{:?}", node.kind) == kinds[index]
                && (node.range.start, node.range.end) == ranges[index]
                && node.range.source_id == ast.source_id
                && node.recovered == recoveries[index]
                && node
                    .children
                    .iter()
                    .map(|child| child.index())
                    .eq(children[index].iter().copied())
        })
}

#[cfg(test)]
fn task258b3m2b2b3p_resolver_contract(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutation: SetEnumerationResolverMutation,
) -> bool {
    use mizar_resolve::{
        declarations::{DeclarationShellKind, DeclarationShellVisibilityState},
        env::{ContributionKind, ExportStatus, SymbolKind, Visibility},
    };
    use mizar_session::SourceAnchor;

    let declarations = shells.declarations();
    let Some(symbol) = symbols.symbols().iter().next() else {
        return false;
    };
    let Some(contribution) = symbols.contributions().get(symbol.contribution()) else {
        return false;
    };
    let signature = symbol.signature();
    let mut fields = vec![
        format!("{:?}", symbols.module_id()),
        symbols.imports().iter().count().to_string(),
        shells.exports().len().to_string(),
        declarations.len().to_string(),
    ];
    for shell in declarations {
        fields.extend([
            shell.id().index().to_string(),
            shell.ordinal().to_string(),
            format!("{:?}", shell.kind()),
            format!("{:?}", shell.module()),
            shell.node_id().index().to_string(),
            format!("{:?}", shell.syntax_kind()),
            format!("{:?}", shell.range()),
            format!("{:?}", shell.parent()),
            format!("{:?}", shell.visibility().state()),
            format!("{:?}", shell.visibility().marker_range()),
            format!("{:?}", shell.visibility().spelling()),
            shell.recovered().to_string(),
        ]);
    }
    fields.extend([
        symbols.symbols().iter().count().to_string(),
        format!("{:?}", symbol.symbol().module()),
        symbol.symbol().local().as_str().to_owned(),
        symbol.symbol().fqn().as_str().to_owned(),
        format!("{:?}", symbol.kind()),
        format!("{:?}", symbol.visibility()),
        format!("{:?}", symbol.export_status()),
        symbol.namespace().as_str().to_owned(),
        symbol.primary_spelling().to_owned(),
        format!("{:?}", symbol.notation_spelling()),
        format!("{:?}", symbol.origin().source_id()),
        format!("{:?}", symbol.origin().module_id()),
        format!("{:?}", symbol.origin().anchor()),
        format!("{:?}", symbol.origin().structural_path()),
        format!("{:?}", symbol.origin().import_edge()),
        symbol.origin().is_recovered().to_string(),
        symbol.contribution().index().to_string(),
        format!("{signature:?}"),
        format!("{:?}", symbol.relations()),
        symbols.contributions().iter().count().to_string(),
        contribution.id().index().to_string(),
        format!("{:?}", contribution.module()),
        format!("{:?}", contribution.kind()),
        format!("{:?}", contribution.anchor()),
        format!("{:?}", contribution.effects().symbols()),
        format!("{:?}", contribution.effects().definitions()),
        contribution.effects().overload_groups().len().to_string(),
        contribution.effects().registrations().len().to_string(),
        contribution.effects().lexical_summaries().len().to_string(),
        contribution.effects().labels().len().to_string(),
        contribution.effects().namespace_edges().len().to_string(),
        contribution
            .effects()
            .declaration_dependencies()
            .len()
            .to_string(),
        contribution.effects().imports().len().to_string(),
        contribution.effects().exports().len().to_string(),
        contribution.effects().diagnostics().len().to_string(),
    ]);
    if fields.len() != TASK258B3M2B2B3P_RESOLVER_FIELD_COUNT {
        return false;
    }
    if let SetEnumerationResolverMutation::Field(index) = mutation
        && let Some(field) = fields.get_mut(index)
    {
        field.push('!');
    }

    let expected_local = format!(
        "contribution=0:namespace={}:owner=theorem#1:shell=theorem:kind=theorem:name=FormulaStatementSetEnumerationWitnessSmoke:notation=_:arity=_:definition=theorem:registration=_:policy=non-overloadable:slot=non-overloadable:_:theorem:_",
        module.path().as_str()
    );
    let expected_fqn = format!(
        "{}::{}::{expected_local}",
        module.package().as_str(),
        module.path().as_str()
    );
    let expected_signature = "Opaque { schema: \"parser-signature-v1\", payload: \"node=TheoremItem;symbol=theorem;definition=theorem;primary_tokens=theorem FormulaStatementSetEnumerationWitnessSmoke : x = x proof take { 1 , 2 } ; thus x = x ; end ;;notation=_;arity=_;roles=FormulaExpression,ProofBlock\" }";
    let module_debug = format!("{module:?}");
    let reserve_range = SourceRange {
        source_id: ast.source_id,
        start: 0,
        end: 18,
    };
    let theorem_range = SourceRange {
        source_id: ast.source_id,
        start: 19,
        end: 116,
    };
    let expected_symbol = mizar_resolve::resolved_ast::SymbolId::new(
        module.clone(),
        mizar_resolve::resolved_ast::LocalSymbolId::new(expected_local.clone()),
        mizar_resolve::resolved_ast::FullyQualifiedName::new(expected_fqn.clone()),
    );
    let expected_fields = vec![
        module_debug.clone(),
        "0".to_owned(),
        "0".to_owned(),
        "2".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "Reserve".to_owned(),
        module_debug.clone(),
        "29".to_owned(),
        "ReserveItem".to_owned(),
        format!("{reserve_range:?}"),
        "None".to_owned(),
        "Unspecified".to_owned(),
        "None".to_owned(),
        "None".to_owned(),
        "false".to_owned(),
        "1".to_owned(),
        "1".to_owned(),
        "Theorem".to_owned(),
        module_debug.clone(),
        "53".to_owned(),
        "TheoremItem".to_owned(),
        format!("{theorem_range:?}"),
        "None".to_owned(),
        "Unspecified".to_owned(),
        "None".to_owned(),
        "None".to_owned(),
        "false".to_owned(),
        "1".to_owned(),
        module_debug.clone(),
        expected_local.clone(),
        expected_fqn.clone(),
        "Theorem".to_owned(),
        "Public".to_owned(),
        "Exported".to_owned(),
        module.path().as_str().to_owned(),
        "FormulaStatementSetEnumerationWitnessSmoke".to_owned(),
        "None".to_owned(),
        format!("{:?}", ast.source_id),
        module_debug.clone(),
        format!("{:?}", SourceAnchor::Range(theorem_range)),
        "[2, 1]".to_owned(),
        "None".to_owned(),
        "false".to_owned(),
        "0".to_owned(),
        format!("Some({expected_signature})"),
        "[]".to_owned(),
        "1".to_owned(),
        "0".to_owned(),
        module_debug,
        format!(
            "{:?}",
            ContributionKind::LocalSource {
                source_id: ast.source_id
            }
        ),
        format!("{:?}", SourceAnchor::Range(reserve_range)),
        format!("{:?}", [expected_symbol]),
        "[DefinitionId(0)]".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
    ];
    fields[0] == format!("{module:?}")
        && fields[1] == "0"
        && fields[2] == "0"
        && fields[3] == "2"
        && declarations.first().is_some_and(|shell| {
            shell.id().index() == 0
                && shell.ordinal() == 0
                && shell.kind() == DeclarationShellKind::Reserve
                && shell.module() == module
                && shell.node_id().index() == 29
                && format!("{:?}", shell.syntax_kind()) == "ReserveItem"
                && shell.range()
                    == SourceRange {
                        source_id: ast.source_id,
                        start: 0,
                        end: 18,
                    }
                && shell.parent().is_none()
                && shell.visibility().state() == DeclarationShellVisibilityState::Unspecified
                && shell.visibility().marker_range().is_none()
                && shell.visibility().spelling().is_none()
                && !shell.recovered()
        })
        && declarations.get(1).is_some_and(|shell| {
            shell.id().index() == 1
                && shell.ordinal() == 1
                && shell.kind() == DeclarationShellKind::Theorem
                && shell.module() == module
                && shell.node_id().index() == 53
                && format!("{:?}", shell.syntax_kind()) == "TheoremItem"
                && shell.range()
                    == SourceRange {
                        source_id: ast.source_id,
                        start: 19,
                        end: 116,
                    }
                && shell.parent().is_none()
                && shell.visibility().state() == DeclarationShellVisibilityState::Unspecified
                && shell.visibility().marker_range().is_none()
                && shell.visibility().spelling().is_none()
                && !shell.recovered()
        })
        && fields[28] == "1"
        && symbol.symbol().module() == module
        && fields[30] == expected_local
        && fields[31] == expected_fqn
        && symbol.kind() == SymbolKind::Theorem
        && symbol.visibility() == Visibility::Public
        && symbol.export_status() == ExportStatus::Exported
        && symbol.namespace().as_str() == module.path().as_str()
        && symbol.primary_spelling() == "FormulaStatementSetEnumerationWitnessSmoke"
        && symbol.notation_spelling().is_none()
        && symbol.origin().source_id() == ast.source_id
        && symbol.origin().module_id() == module
        && matches!(
            symbol.origin().anchor(),
            SourceAnchor::Range(range)
                if *range == SourceRange {
                    source_id: ast.source_id,
                    start: 19,
                    end: 116,
                }
        )
        && symbol.origin().structural_path() == [2, 1]
        && symbol.origin().import_edge().is_none()
        && !symbol.origin().is_recovered()
        && symbol.contribution().index() == 0
        && fields[45] == format!("Some({expected_signature})")
        && symbol.relations().is_empty()
        && contribution.id().index() == 0
        && contribution.module() == module
        && matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == ast.source_id
        )
        && matches!(
            contribution.anchor(),
            SourceAnchor::Range(range)
                if *range == SourceRange {
                    source_id: ast.source_id,
                    start: 0,
                    end: 18,
                }
        )
        && contribution.effects().symbols() == [symbol.symbol().clone()]
        && contribution
            .effects()
            .definitions()
            .iter()
            .map(|definition| definition.index())
            .eq([0])
        && contribution.effects().overload_groups().is_empty()
        && contribution.effects().registrations().is_empty()
        && contribution.effects().lexical_summaries().is_empty()
        && contribution.effects().labels().is_empty()
        && contribution.effects().namespace_edges().is_empty()
        && contribution.effects().declaration_dependencies().is_empty()
        && contribution.effects().imports().is_empty()
        && contribution.effects().exports().is_empty()
        && contribution.effects().diagnostics().is_empty()
        && symbols.contributions().iter().count() == 1
        && expected_fields.len() == TASK258B3M2B2B3P_RESOLVER_FIELD_COUNT
        && fields == expected_fields
}

#[cfg(test)]
fn task258b3m2b2b3p_binding_contract(
    ast: &SurfaceAst,
    module: &ModuleId,
    binding_env: &BindingEnv,
    _mutation: SetEnumerationBindingMutation,
) -> bool {
    use mizar_checker::binding_env::{
        BinderIdentity, BindingContextLayer, BindingContextOwner, BindingContextRecovery,
        BindingKind, BindingRecoveryState, BindingStatus, BindingTypeSite,
    };
    let Some(module_context) = binding_env.contexts().get(BindingContextId::new(0)) else {
        return false;
    };
    let Some(proof_context) = binding_env.contexts().get(BindingContextId::new(1)) else {
        return false;
    };
    let Some((binding_id, binding)) = binding_env.bindings().iter().next() else {
        return false;
    };
    let fields = vec![
        format!("{:?}", binding_env.source_id()),
        format!("{:?}", binding_env.module_id()),
        binding_env.contexts().len().to_string(),
        binding_env.bindings().len().to_string(),
        binding_env.diagnostics().len().to_string(),
        format!("{:?}", module_context.owner),
        format!("{:?}", module_context.parent),
        format!("{:?}", module_context.layer),
        format!("{:?}", module_context.lexical_scope),
        format!("{:?}", module_context.bindings),
        format!("{:?}", module_context.visible_bindings),
        format!("{:?}", module_context.recovery),
        format!("{:?}", proof_context.owner),
        format!("{:?}", proof_context.parent),
        format!("{:?}", proof_context.layer),
        format!("{:?}", proof_context.lexical_scope),
        format!("{:?}", proof_context.bindings),
        format!("{:?}", proof_context.visible_bindings),
        format!("{:?}", proof_context.recovery),
        binding_id.index().to_string(),
        binding.id.index().to_string(),
        binding.spelling.clone(),
        format!("{:?}", binding.kind),
        format!("{:?}", binding.identity),
        format!("{:?}", binding.owner_context),
        format!("{:?}", binding.declaration_range),
        binding.visible_after_ordinal.to_string(),
        format!("{:?}", binding.type_site),
        format!("{:?}", binding.status),
        format!("{:?}", binding.captured),
        format!("{:?}", binding.diagnostics),
        format!("{:?}", binding.recovery),
        binding_env
            .contexts()
            .iter()
            .map(|(id, _)| id.index())
            .collect::<Vec<_>>()
            .len()
            .to_string(),
        binding_env
            .bindings()
            .iter()
            .map(|(id, _)| id.index())
            .collect::<Vec<_>>()
            .len()
            .to_string(),
        format!("{:?}", module_context.bindings.first()),
        format!("{:?}", module_context.visible_bindings.first()),
        format!("{:?}", proof_context.visible_bindings.first()),
        format!(
            "{:?}",
            proof_context
                .lexical_scope
                .as_ref()
                .map(|scope| scope.path().to_vec())
        ),
        format!("{:?}", binding.captured.identities()),
    ];
    if fields.len() != TASK258B3M2B2B3P_BINDING_FIELD_COUNT {
        return false;
    }
    let reserve_range = SourceRange {
        source_id: ast.source_id,
        start: 8,
        end: 9,
    };
    binding_env.source_id() == ast.source_id
        && binding_env.module_id() == module
        && binding_env.contexts().len() == 2
        && binding_env.bindings().len() == 1
        && binding_env.diagnostics().is_empty()
        && matches!(module_context.owner, BindingContextOwner::Module)
        && module_context.parent.is_none()
        && module_context.layer == BindingContextLayer::Module
        && module_context.lexical_scope.is_none()
        && module_context.bindings.iter().map(|id| id.index()).eq([0])
        && module_context
            .visible_bindings
            .iter()
            .map(|id| id.index())
            .eq([0])
        && module_context.recovery == BindingContextRecovery::Normal
        && matches!(
            proof_context.owner,
            BindingContextOwner::SourceStatement { source_range }
                if source_range == SourceRange {
                    source_id: ast.source_id,
                    start: 77,
                    end: 115,
                }
        )
        && proof_context.parent == Some(BindingContextId::new(0))
        && proof_context.layer == BindingContextLayer::Proof
        && proof_context
            .lexical_scope
            .as_ref()
            .is_some_and(|scope| scope.path() == [0])
        && proof_context.bindings.is_empty()
        && proof_context
            .visible_bindings
            .iter()
            .map(|id| id.index())
            .eq([0])
        && proof_context.recovery == BindingContextRecovery::Normal
        && binding_id.index() == 0
        && binding.id == binding_id
        && binding.spelling == "x"
        && binding.kind == BindingKind::ReservedVariable
        && matches!(
            &binding.identity,
            BinderIdentity::ReservedVariable {
                spelling,
                declaration_range,
            } if spelling == "x" && *declaration_range == reserve_range
        )
        && binding.owner_context == BindingContextId::new(0)
        && binding.declaration_range == reserve_range
        && binding.visible_after_ordinal == 0
        && matches!(
            binding.type_site,
            BindingTypeSite::Source(range)
                if range == SourceRange {
                    source_id: ast.source_id,
                    start: 14,
                    end: 17,
                }
        )
        && binding.status == BindingStatus::Reserved
        && binding.captured.identities().is_empty()
        && binding.diagnostics.is_empty()
        && binding.recovery == BindingRecoveryState::Normal
}

#[cfg(test)]
fn task258b3m2b2b3p_binding_env_with_field_mutation(
    binding_env: &BindingEnv,
    field: usize,
) -> Result<BindingEnv, String> {
    use mizar_checker::binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextLayer, BindingContextOwner,
        BindingContextRecovery, BindingContextTable, BindingDiagnosticClass,
        BindingDiagnosticDraft, BindingDiagnosticRecovery, BindingDiagnosticSeverity,
        BindingDiagnosticTable, BindingDraft, BindingEnvParts, BindingKind, BindingRecoveryState,
        BindingStatus, BindingTable, BindingTypeSite, CapturedFreeVariables,
    };
    let mut diagnostics = BindingDiagnosticTable::new();
    let diagnostic = diagnostics.insert(BindingDiagnosticDraft {
        source_range: None,
        class: BindingDiagnosticClass::UnsupportedSourceShape,
        severity: BindingDiagnosticSeverity::Note,
        message_key: "checker.binding.b3p.test-mutation".to_owned(),
        recovery: BindingDiagnosticRecovery::Degraded,
    });
    let mut binding_drafts = binding_env
        .bindings()
        .iter()
        .map(|(_, binding)| BindingDraft {
            spelling: binding.spelling.clone(),
            kind: binding.kind,
            identity: binding.identity.clone(),
            owner_context: binding.owner_context,
            declaration_range: binding.declaration_range,
            visible_after_ordinal: binding.visible_after_ordinal,
            type_site: binding.type_site.clone(),
            status: binding.status,
            captured: binding.captured.clone(),
            diagnostics: binding.diagnostics.clone(),
            recovery: binding.recovery,
        })
        .collect::<Vec<_>>();
    match field {
        3 | 19 | 20 | 33 => binding_drafts.push(BindingDraft {
            spelling: "b3p_extra".to_owned(),
            kind: BindingKind::Generated,
            identity: BinderIdentity::Generated {
                context: BindingContextId::new(0),
                counter: 1,
            },
            owner_context: BindingContextId::new(0),
            declaration_range: binding_drafts[0].declaration_range,
            visible_after_ordinal: 1,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }),
        21 => binding_drafts[0].spelling.push('!'),
        22 => binding_drafts[0].kind = BindingKind::LetBinding,
        23 => {
            binding_drafts[0].identity = BinderIdentity::Generated {
                context: BindingContextId::new(0),
                counter: 1,
            }
        }
        24 => binding_drafts[0].owner_context = BindingContextId::new(1),
        25 => binding_drafts[0].declaration_range.start += 1,
        26 => binding_drafts[0].visible_after_ordinal += 1,
        27 => binding_drafts[0].type_site = BindingTypeSite::Missing,
        28 => binding_drafts[0].status = BindingStatus::Active,
        29 | 38 => {
            binding_drafts[0].captured =
                CapturedFreeVariables::new(vec![binding_drafts[0].identity.clone()])
        }
        30 => binding_drafts[0].diagnostics = vec![diagnostic],
        31 => binding_drafts[0].recovery = BindingRecoveryState::Degraded,
        _ => {}
    }
    let mut bindings = BindingTable::new();
    for draft in binding_drafts {
        bindings.insert(draft);
    }
    let mut contexts = binding_env
        .contexts()
        .iter()
        .map(|(_, context)| BindingContextDraft {
            owner: context.owner.clone(),
            parent: context.parent,
            layer: context.layer,
            lexical_scope: context.lexical_scope.clone(),
            bindings: context.bindings.clone(),
            visible_bindings: context.visible_bindings.clone(),
            recovery: context.recovery,
        })
        .collect::<Vec<_>>();
    match field {
        5 => contexts[0].owner = BindingContextOwner::Generated("B3P".to_owned()),
        6 => contexts[0].parent = Some(BindingContextId::new(1)),
        7 => contexts[0].layer = BindingContextLayer::Block,
        8 => contexts[0].lexical_scope = contexts[1].lexical_scope.clone(),
        9 | 34 => contexts[0].bindings.clear(),
        10 | 35 => contexts[0].visible_bindings.clear(),
        11 => contexts[0].recovery = BindingContextRecovery::Degraded,
        12 => {
            let BindingContextOwner::SourceStatement { source_range } = contexts[1].owner else {
                return Err("B3P proof owner disappeared".to_owned());
            };
            contexts[1].owner = BindingContextOwner::SourceFormula { source_range };
        }
        13 => contexts[1].parent = None,
        14 => contexts[1].layer = BindingContextLayer::Block,
        15 | 37 => contexts[1].lexical_scope = None,
        16 => contexts[1]
            .bindings
            .push(mizar_checker::binding_env::BindingId::new(0)),
        17 | 36 => contexts[1].visible_bindings.clear(),
        18 => contexts[1].recovery = BindingContextRecovery::Degraded,
        _ => {}
    }
    if matches!(field, 2 | 32) {
        contexts.push(BindingContextDraft {
            owner: BindingContextOwner::Generated("B3P-extra".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
    }
    if matches!(field, 3 | 19 | 20 | 33) {
        contexts[0]
            .bindings
            .push(mizar_checker::binding_env::BindingId::new(1));
    }
    let mut context_table = BindingContextTable::new();
    for draft in contexts {
        context_table.insert(draft);
    }
    let source_id = if field == 0 {
        task258b3m2b2b3p_distinct_source_id()?
    } else {
        binding_env.source_id()
    };
    let module_id = if field == 1 {
        ModuleId::new(
            binding_env.module_id().package().clone(),
            mizar_session::ModulePath::new("tests.task258b3m2b2b3p.binding-substitute"),
        )
    } else {
        binding_env.module_id().clone()
    };
    if field != 4 && field != 30 {
        diagnostics = BindingDiagnosticTable::new();
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id,
        module_id,
        contexts: context_table,
        bindings,
        diagnostics,
    })
    .map_err(|error| error.to_string())
}

#[cfg(test)]
fn task258b3m2b2b3p_binding_env_with_prior_event(
    binding_env: &BindingEnv,
) -> Result<BindingEnv, String> {
    use mizar_checker::binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextTable, BindingDiagnosticTable,
        BindingDraft, BindingEnvParts, BindingKind, BindingRecoveryState, BindingStatus,
        BindingTable, BindingTypeSite, CapturedFreeVariables,
    };

    let Some((_, original)) = binding_env.bindings().iter().next() else {
        return Err("B3P use-ordinal profile lost BindingId(0)".to_owned());
    };
    let mut bindings = BindingTable::new();
    bindings.insert(BindingDraft {
        spelling: original.spelling.clone(),
        kind: original.kind,
        identity: original.identity.clone(),
        owner_context: original.owner_context,
        declaration_range: original.declaration_range,
        visible_after_ordinal: original.visible_after_ordinal,
        type_site: original.type_site.clone(),
        status: original.status,
        captured: original.captured.clone(),
        diagnostics: original.diagnostics.clone(),
        recovery: original.recovery,
    });
    let prior_range = SourceRange {
        source_id: binding_env.source_id(),
        start: 18,
        end: 19,
    };
    bindings.insert(BindingDraft {
        spelling: "b3p_prior_event".to_owned(),
        kind: BindingKind::Generated,
        identity: BinderIdentity::Generated {
            context: BindingContextId::new(0),
            counter: 1,
        },
        owner_context: BindingContextId::new(0),
        declaration_range: prior_range,
        visible_after_ordinal: 1,
        type_site: BindingTypeSite::Missing,
        status: BindingStatus::Active,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });

    let mut contexts = binding_env
        .contexts()
        .iter()
        .map(|(_, context)| BindingContextDraft {
            owner: context.owner.clone(),
            parent: context.parent,
            layer: context.layer,
            lexical_scope: context.lexical_scope.clone(),
            bindings: context.bindings.clone(),
            visible_bindings: context.visible_bindings.clone(),
            recovery: context.recovery,
        })
        .collect::<Vec<_>>();
    let prior = mizar_checker::binding_env::BindingId::new(1);
    contexts[0].bindings.push(prior);
    contexts[0].visible_bindings.push(prior);
    contexts[1].visible_bindings.push(prior);
    let mut context_table = BindingContextTable::new();
    for context in contexts {
        context_table.insert(context);
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: binding_env.source_id(),
        module_id: binding_env.module_id().clone(),
        contexts: context_table,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}

#[cfg(test)]
fn task258b3m2b2b3p_primary_roots() -> [(usize, BindingContextId); 6] {
    let module = BindingContextId::new(0);
    let proof = BindingContextId::new(1);
    [
        (30, module),
        (32, module),
        (36, proof),
        (38, proof),
        (44, proof),
        (46, proof),
    ]
}

#[cfg(test)]
fn task258b3m2b2b3p_primary_profile_is_exact(
    ast: &SurfaceAst,
    module: &ModuleId,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
) -> bool {
    use mizar_checker::{
        source_term::{
            SourcePrimaryTermKind, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceRole,
            SourcePrimaryTermRole,
        },
        typed_ast::{NodeRecoveryState, TypingState},
    };
    use mizar_session::SourceAnchor;

    let expected_terms = [
        (30, 71, 72, 0, "x", SourcePrimaryTermKind::VariableReference),
        (32, 75, 76, 0, "x", SourcePrimaryTermKind::VariableReference),
        (36, 91, 92, 1, "1", SourcePrimaryTermKind::Numeral),
        (38, 94, 95, 1, "2", SourcePrimaryTermKind::Numeral),
        (
            44,
            105,
            106,
            1,
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            46,
            109,
            110,
            1,
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
    ];
    if source_term.handoff.source_id() != ast.source_id
        || source_term.handoff.module_id() != module
        || source_term.handoff.terms().len() != 6
        || source_term.handoff.references().len() != 4
        || source_term.handoff.numeric_type_requests().len() != 2
        || source_term.handoff.terms().iter().zip(expected_terms).any(
            |((id, term), (site, start, end, context, spelling, kind))| {
                id.index() != term.source_ordinal()
                    || term.site().node().index() != site
                    || term.source_range()
                        != SourceRange {
                            source_id: ast.source_id,
                            start,
                            end,
                        }
                    || term.context() != BindingContextId::new(context)
                    || term.recovery() != SourcePrimaryTermRecovery::Normal
                    || term.spelling() != spelling
                    || term.kind() != kind
                    || term.role() != SourcePrimaryTermRole::Value
                    || term.parent().is_some()
            },
        )
    {
        return false;
    }
    let expected_references = [(0, false), (1, false), (4, true), (5, true)];
    if source_term
        .handoff
        .references()
        .iter()
        .zip(expected_references)
        .any(|((id, reference), (term, scoped))| {
            id.index() >= 4
                || reference.term().index() != term
                || reference.binding().index() != 0
                || reference.role() != SourcePrimaryTermReferenceRole::Variable
                || reference.use_ordinal() != 1
                || if scoped {
                    reference
                        .lexical_scope()
                        .is_none_or(|scope| scope.path() != [0])
                } else {
                    reference.lexical_scope().is_some()
                }
        })
    {
        return false;
    }
    let expected_requests = [(2, 36, 91, 92, "1"), (3, 38, 94, 95, "2")];
    if source_term
        .handoff
        .numeric_type_requests()
        .iter()
        .zip(expected_requests)
        .any(|((id, request), (term, owner, start, end, spelling))| {
            request.term().index() != term
                || request.owner().node().index() != owner
                || request.source_range()
                    != SourceRange {
                        source_id: ast.source_id,
                        start,
                        end,
                    }
                || request.spelling() != spelling
                || request.request_ordinal() != id.index()
        })
    {
        return false;
    }
    source_term.arena.len() == 57
        && source_term.arena.root().map(|root| root.index()) == Some(56)
        && source_term.arena.iter().all(|(id, node)| {
            let index = id.index();
            let expected_kind = match index {
                30 | 32 | 44 | 46 => "source.term.variable-reference",
                36 | 38 => "source.term.numeral",
                _ => "source.surface.unowned",
            };
            ast.nodes().get(index).is_some_and(|surface| {
                node.kind.as_str() == expected_kind
                    && node.resolved_node.is_none()
                    && node.anchor == SourceAnchor::Range(surface.range)
                    && node
                        .children
                        .iter()
                        .map(|child| child.index())
                        .eq(surface.children.iter().map(|child| child.index()))
                    && node.typing == TypingState::Unknown
                    && node.recovery == NodeRecoveryState::Normal
                    && node.links.context.is_none()
                    && node.links.type_entry.is_none()
                    && node.links.facts.is_empty()
                    && node.links.coercions.is_empty()
                    && node.links.initial_obligations.is_empty()
                    && node.links.diagnostics.is_empty()
            })
        })
        && binding_env.source_id() == ast.source_id
}

#[cfg(test)]
fn task258b3m2b2b3p_distinct_source_id() -> Result<mizar_session::SourceId, String> {
    let allocator = mizar_session::InMemorySessionIdAllocator::new();
    mizar_session::SessionIdAllocator::next_source_id(
        &allocator,
        super::super::shared::snapshot_id(258_330),
    )
    .map_err(|error| error.to_string())?;
    mizar_session::SessionIdAllocator::next_source_id(
        &allocator,
        super::super::shared::snapshot_id(258_330),
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
fn task258b3m2b2b3p_source_term_with_mutation(
    _ast: &SurfaceAst,
    module: &ModuleId,
    binding_env: &BindingEnv,
    source_term: SourceTermParts,
    mutation: SetEnumerationPrimaryMutation,
) -> Result<SourceTermParts, String> {
    use mizar_checker::source_term::{
        SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermId,
        SourcePrimaryTermInput, SourcePrimaryTermKind, SourcePrimaryTermProducer,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceInput, SourcePrimaryTermReferenceRole,
        SourcePrimaryTermRole,
    };
    if mutation == SetEnumerationPrimaryMutation::None
        || mutation == SetEnumerationPrimaryMutation::StaleFingerprintReplay
    {
        return Ok(source_term);
    }
    let mut input = SourcePrimaryTermHandoffInput {
        source_id: source_term.handoff.source_id(),
        module_id: source_term.handoff.module_id().clone(),
        terms: source_term
            .handoff
            .terms()
            .iter()
            .map(|(_, term)| SourcePrimaryTermInput {
                site: term.site().clone(),
                source_range: term.source_range(),
                source_ordinal: term.source_ordinal(),
                context: term.context(),
                recovery: term.recovery(),
                spelling: term.spelling().to_owned(),
                kind: term.kind(),
                role: term.role(),
                parent: term.parent(),
            })
            .collect(),
        references: source_term
            .handoff
            .references()
            .iter()
            .map(|(_, reference)| SourcePrimaryTermReferenceInput {
                term: reference.term(),
                binding: reference.binding(),
                role: reference.role(),
            })
            .collect(),
        numeric_type_requests: source_term
            .handoff
            .numeric_type_requests()
            .iter()
            .map(|(_, request)| SourceNumericTypeRequestInput {
                term: request.term(),
                owner: request.owner().clone(),
                source_range: request.source_range(),
                spelling: request.spelling().to_owned(),
                request_ordinal: request.request_ordinal(),
            })
            .collect(),
    };
    match mutation {
        SetEnumerationPrimaryMutation::None
        | SetEnumerationPrimaryMutation::DuplicateRoot
        | SetEnumerationPrimaryMutation::MissingRoot
        | SetEnumerationPrimaryMutation::StaleFingerprintReplay => {}
        SetEnumerationPrimaryMutation::SourceId => {
            input.source_id = task258b3m2b2b3p_distinct_source_id()?;
        }
        SetEnumerationPrimaryMutation::ModuleId => {
            input.module_id = ModuleId::new(
                module.package().clone(),
                mizar_session::ModulePath::new("tests.task258b3m2b2b3p.primary-substitute"),
            );
        }
        SetEnumerationPrimaryMutation::TermSite(index) => {
            input.terms[index].site = TypedSiteRef::Node(TypedNodeId::new(0));
        }
        SetEnumerationPrimaryMutation::TermRange(index) => {
            input.terms[index].source_range.start += 1;
        }
        SetEnumerationPrimaryMutation::TermOrdinal(index) => {
            input.terms[index].source_ordinal += 1;
        }
        SetEnumerationPrimaryMutation::TermContext(index) => {
            input.terms[index].context =
                BindingContextId::new(1 - input.terms[index].context.index());
        }
        SetEnumerationPrimaryMutation::TermRecovery(index) => {
            input.terms[index].recovery = SourcePrimaryTermRecovery::Degraded;
        }
        SetEnumerationPrimaryMutation::TermSpelling(index) => {
            input.terms[index].spelling.push('!');
        }
        SetEnumerationPrimaryMutation::TermKind(index) => {
            input.terms[index].kind = if input.terms[index].kind == SourcePrimaryTermKind::Numeral {
                SourcePrimaryTermKind::VariableReference
            } else {
                SourcePrimaryTermKind::Numeral
            };
        }
        SetEnumerationPrimaryMutation::TermRole(index) => {
            input.terms[index].role = SourcePrimaryTermRole::CurrentDefinitionResult;
        }
        SetEnumerationPrimaryMutation::TermParent(index) => {
            input.terms[index].parent = Some(SourcePrimaryTermId::new(index));
        }
        SetEnumerationPrimaryMutation::ReferenceTerm(index) => {
            input.references[index].term = SourcePrimaryTermId::new(2);
        }
        SetEnumerationPrimaryMutation::ReferenceBinding(index) => {
            input.references[index].binding = mizar_checker::binding_env::BindingId::new(1);
        }
        SetEnumerationPrimaryMutation::ReferenceRole(index) => {
            input.references[index].role = SourcePrimaryTermReferenceRole::LocalConstant;
        }
        SetEnumerationPrimaryMutation::ReferenceScopeModule
        | SetEnumerationPrimaryMutation::ReferenceScopeProof
        | SetEnumerationPrimaryMutation::ReferenceUseOrdinal => {}
        SetEnumerationPrimaryMutation::NumericTerm(index) => {
            input.numeric_type_requests[index].term = SourcePrimaryTermId::new(0);
        }
        SetEnumerationPrimaryMutation::NumericOwner(index) => {
            input.numeric_type_requests[index].owner = TypedSiteRef::Node(TypedNodeId::new(0));
        }
        SetEnumerationPrimaryMutation::NumericRange(index) => {
            input.numeric_type_requests[index].source_range.start += 1;
        }
        SetEnumerationPrimaryMutation::NumericSpelling(index) => {
            input.numeric_type_requests[index].spelling.push('!');
        }
        SetEnumerationPrimaryMutation::NumericOrdinal(index) => {
            input.numeric_type_requests[index].request_ordinal += 1;
        }
    }
    let handoff = SourcePrimaryTermProducer::build(input, binding_env, &source_term.arena)
        .map_err(|error| error.to_string())?;
    Ok(SourceTermParts {
        arena: source_term.arena,
        handoff,
    })
}

#[cfg(test)]
fn task258b3m2b2b3p_dependency_probe_arena(
    ast: &SurfaceAst,
    source: &TypedArena,
    owned: &[(usize, &'static str)],
    anchor_overrides: &[(usize, SourceRange)],
) -> Result<TypedArena, String> {
    use mizar_session::SourceAnchor;

    let kinds = owned.iter().copied().collect::<BTreeMap<_, _>>();
    let arena = arena_with_overrides(ast, source, &kinds, &BTreeMap::new())?;
    let mut nodes = arena
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for (site, range) in anchor_overrides {
        nodes
            .get_mut(*site)
            .ok_or_else(|| "Task255 dependency probe occurrence site disappeared".to_owned())?
            .anchor = SourceAnchor::Range(*range);
    }
    TypedArena::try_new(arena.root(), nodes).map_err(|error| error.to_string())
}

#[cfg(test)]
fn task258b3m2b2b3p_dependency_probe_set_input(
    ast: &SurfaceAst,
    module: &ModuleId,
    first: SourceSetTarget,
    first_spelling: &str,
) -> SourceSetTermHandoffInput {
    use mizar_checker::source_term::SourcePrimaryTermId;

    SourceSetTermHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms: vec![SourceSetTermInput {
            site: TypedSiteRef::Node(TypedNodeId::new(40)),
            source_range: ast.nodes()[40].range,
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceSetTermRecovery::Normal,
            spelling: format!("{{ {first_spelling} , 2 }}"),
            kind: SourceSetTermKind::Enumeration,
        }],
        wrappers: Vec::new(),
        generators: Vec::new(),
        type_sites: Vec::new(),
        conditions: Vec::new(),
        edges: vec![
            SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::EnumerationElement,
                target: first,
            },
            SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 1,
                role: SourceSetEdgeRole::EnumerationElement,
                target: SourceSetTarget::Primary(SourcePrimaryTermId::new(3)),
            },
        ],
        requests: vec![SourceSetRequestInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            kind: SourceSetRequestKind::ResultType,
            generator: None,
            type_site: None,
        }],
    }
}

#[cfg(test)]
fn task258b3m2b2b3p_fingerprint_profile_is_exact(
    primary_fingerprint: &str,
    handoff: &mizar_checker::source_set_term::SourceSetTermHandoff,
) -> bool {
    handoff.primary_term_fingerprint() == primary_fingerprint
        && handoff.application_fingerprint().is_none()
        && handoff.structure_fingerprint().is_none()
}

#[cfg(test)]
fn task258b3m2b2b3p_set_handoff_profile_is_exact(
    ast: &SurfaceAst,
    module: &ModuleId,
    primary_fingerprint: &str,
    handoff: &mizar_checker::source_set_term::SourceSetTermHandoff,
) -> bool {
    let Some(term) = handoff.terms().get(SourceSetTermId::new(0)) else {
        return false;
    };
    let edges = handoff
        .edges()
        .iter()
        .map(|(id, edge)| {
            (
                id.index(),
                edge.term().index(),
                edge.ordinal(),
                edge.role(),
                edge.target(),
            )
        })
        .collect::<Vec<_>>();
    let requests = handoff
        .requests()
        .iter()
        .map(|(id, request)| {
            (
                id.index(),
                request.term().index(),
                request.ordinal(),
                request.kind(),
                request.generator(),
                request.type_site(),
            )
        })
        .collect::<Vec<_>>();
    handoff.source_id() == ast.source_id
        && handoff.module_id() == module
        && handoff.terms().len() == 1
        && handoff.wrappers().is_empty()
        && handoff.generators().is_empty()
        && handoff.type_sites().is_empty()
        && handoff.conditions().is_empty()
        && term.site().node().index() == 40
        && term.source_range()
            == SourceRange {
                source_id: ast.source_id,
                start: 90,
                end: 96,
            }
        && term.source_ordinal() == 0
        && term.context() == BindingContextId::new(1)
        && term.recovery() == SourceSetTermRecovery::Normal
        && term.spelling() == "{ 1 , 2 }"
        && term.kind() == SourceSetTermKind::Enumeration
        && edges
            == [
                (
                    0,
                    0,
                    0,
                    SourceSetEdgeRole::EnumerationElement,
                    SourceSetTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(
                        2,
                    )),
                ),
                (
                    1,
                    0,
                    1,
                    SourceSetEdgeRole::EnumerationElement,
                    SourceSetTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(
                        3,
                    )),
                ),
            ]
        && requests == [(0, 0, 0, SourceSetRequestKind::ResultType, None, None)]
        && task258b3m2b2b3p_fingerprint_profile_is_exact(primary_fingerprint, handoff)
}

#[cfg(test)]
fn task258b3m2b2b3p_coherent_application_fingerprint_probe(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
) -> Result<mizar_checker::source_set_term::SourceSetTermHandoff, String> {
    use mizar_checker::source_application::{
        SourceFunctorApplicationForm, SourceFunctorApplicationHandoffInput,
        SourceFunctorApplicationInput, SourceFunctorApplicationKind,
        SourceFunctorApplicationProducer, SourceFunctorApplicationRecovery, SourceFunctorHeadSite,
    };

    let occurrence = SourceRange {
        source_id: ast.source_id,
        start: 91,
        end: 94,
    };
    let arena = task258b3m2b2b3p_dependency_probe_arena(
        ast,
        &source_term.arena,
        &[
            (37, "source.term.functor-head.single"),
            (41, "source.term.functor-application.inline"),
        ],
        &[(41, occurrence)],
    )?;
    let applications = SourceFunctorApplicationProducer::build(
        SourceFunctorApplicationHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            applications: vec![SourceFunctorApplicationInput {
                site: TypedSiteRef::Node(TypedNodeId::new(41)),
                source_range: occurrence,
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceFunctorApplicationRecovery::Normal,
                spelling: "b3p_inline ( )".to_owned(),
                kind: SourceFunctorApplicationKind::Inline,
                form: SourceFunctorApplicationForm::Functional,
                head_ordinal: 0,
                head: SourceFunctorHeadSite::Single {
                    site: TypedSiteRef::Node(TypedNodeId::new(37)),
                    source_range: ast.nodes()[37].range,
                    spelling: "b3p_inline".to_owned(),
                },
            }],
            wrappers: Vec::new(),
            candidates: Vec::new(),
            arguments: Vec::new(),
            type_requests: Vec::new(),
        },
        symbols,
        binding_env,
        &source_term.handoff,
        &arena,
    )
    .map_err(|error| error.to_string())?;
    let arena = arena_with_overrides(
        ast,
        &arena,
        &BTreeMap::from([(40, "source.term.set.enumeration")]),
        &BTreeMap::new(),
    )?;
    let handoff = SourceSetTermProducer::build(
        task258b3m2b2b3p_dependency_probe_set_input(
            ast,
            module,
            SourceSetTarget::Application(SourceFunctorApplicationId::new(0)),
            "b3p_inline ( )",
        ),
        binding_env,
        &source_term.handoff,
        Some(&applications),
        None,
        &arena,
    )
    .map_err(|error| error.to_string())?;
    if handoff.application_fingerprint() != Some(applications.debug_text().as_str())
        || handoff.structure_fingerprint().is_some()
    {
        return Err("coherent application dependency did not produce one fingerprint".to_owned());
    }
    Ok(handoff)
}

#[cfg(test)]
fn task258b3m2b2b3p_coherent_structure_fingerprint_probe(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
) -> Result<mizar_checker::source_set_term::SourceSetTermHandoff, String> {
    use mizar_checker::{
        source_structure::{
            SourceStructureEdgeInput, SourceStructureEdgeRole, SourceStructureHandoffInput,
            SourceStructureMemberId, SourceStructureMemberInput, SourceStructureMemberRole,
            SourceStructureProducer, SourceStructureRecovery, SourceStructureRequestInput,
            SourceStructureRequestKind, SourceStructureTarget, SourceStructureTermInput,
            SourceStructureTermKind,
        },
        source_term::SourcePrimaryTermId,
    };

    let occurrence = SourceRange {
        source_id: ast.source_id,
        start: 91,
        end: 94,
    };
    let member_range = SourceRange {
        source_id: ast.source_id,
        start: 93,
        end: 94,
    };
    let arena = task258b3m2b2b3p_dependency_probe_arena(
        ast,
        &source_term.arena,
        &[
            (39, "source.term.structure.member.selector"),
            (41, "source.term.structure.selector"),
        ],
        &[(39, member_range), (41, occurrence)],
    )?;
    let structures = SourceStructureProducer::build(
        SourceStructureHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            terms: vec![SourceStructureTermInput {
                site: TypedSiteRef::Node(TypedNodeId::new(41)),
                source_range: occurrence,
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceStructureRecovery::Normal,
                spelling: "1 . b3p_member".to_owned(),
                kind: SourceStructureTermKind::SelectorAccess,
            }],
            wrappers: Vec::new(),
            roots: Vec::new(),
            members: vec![SourceStructureMemberInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                site: TypedSiteRef::Node(TypedNodeId::new(39)),
                source_range: member_range,
                spelling: "b3p_member".to_owned(),
                role: SourceStructureMemberRole::Selector,
                parent: None,
            }],
            field_updates: Vec::new(),
            edges: vec![SourceStructureEdgeInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                role: SourceStructureEdgeRole::SelectorBase,
                member: None,
                target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(2)),
            }],
            requests: vec![
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(0)),
                    request_ordinal: 0,
                    kind: SourceStructureRequestKind::MemberIdentity,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(0)),
                    request_ordinal: 1,
                    kind: SourceStructureRequestKind::InheritancePath,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: None,
                    request_ordinal: 2,
                    kind: SourceStructureRequestKind::ResultType,
                },
            ],
        },
        symbols,
        binding_env,
        &source_term.handoff,
        None,
        &arena,
    )
    .map_err(|error| error.to_string())?;
    let arena = arena_with_overrides(
        ast,
        &arena,
        &BTreeMap::from([(40, "source.term.set.enumeration")]),
        &BTreeMap::new(),
    )?;
    let handoff = SourceSetTermProducer::build(
        task258b3m2b2b3p_dependency_probe_set_input(
            ast,
            module,
            SourceSetTarget::Structure(SourceStructureTermId::new(0)),
            "1 . b3p_member",
        ),
        binding_env,
        &source_term.handoff,
        None,
        Some(&structures),
        &arena,
    )
    .map_err(|error| error.to_string())?;
    if handoff.structure_fingerprint() != Some(structures.debug_text().as_str())
        || handoff.application_fingerprint().is_some()
    {
        return Err("coherent structure dependency did not produce one fingerprint".to_owned());
    }
    Ok(handoff)
}

#[cfg(test)]
fn task258b3m2b2b3p_mutate_handoff(
    input: &mut SourceSetTermHandoffInput,
    ast: &SurfaceAst,
    mutation: SetEnumerationHandoffMutation,
) {
    match mutation {
        SetEnumerationHandoffMutation::None => {}
        SetEnumerationHandoffMutation::SourceId => {
            input.source_id =
                task258b3m2b2b3p_distinct_source_id().expect("B3P distinct source identity");
        }
        SetEnumerationHandoffMutation::ModuleId => {
            input.module_id = ModuleId::new(
                input.module_id.package().clone(),
                mizar_session::ModulePath::new("tests.task258b3m2b2b3p.set-substitute"),
            );
        }
        SetEnumerationHandoffMutation::TermSite => {
            input.terms[0].site = TypedSiteRef::Node(TypedNodeId::new(41));
        }
        SetEnumerationHandoffMutation::TermRange => input.terms[0].source_range.start += 1,
        SetEnumerationHandoffMutation::TermOrdinal => input.terms[0].source_ordinal = 1,
        SetEnumerationHandoffMutation::TermContext => {
            input.terms[0].context = BindingContextId::new(0);
        }
        SetEnumerationHandoffMutation::TermRecovery => {
            input.terms[0].recovery = SourceSetTermRecovery::Degraded;
        }
        SetEnumerationHandoffMutation::TermSpelling => input.terms[0].spelling.push('!'),
        SetEnumerationHandoffMutation::TermKind => {
            input.terms[0].kind = SourceSetTermKind::Comprehension;
        }
        SetEnumerationHandoffMutation::ExtraWrapper => {
            input.wrappers.push(SourceSetWrapperInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: TypedSiteRef::Node(TypedNodeId::new(41)),
                source_range: ast.nodes()[41].range,
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "{ 1 , 2 }".to_owned(),
            });
        }
        SetEnumerationHandoffMutation::ExtraGenerator => {
            input.generators.push(SourceSetGeneratorInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: TypedSiteRef::Node(TypedNodeId::new(14)),
                source_range: ast.nodes()[14].range,
                spelling: "1".to_owned(),
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                type_site: SourceSetTypeSiteId::new(0),
            });
        }
        SetEnumerationHandoffMutation::ExtraTypeSite => {
            input.type_sites.push(SourceSetTypeSiteInput {
                owner: SourceSetTypeOwner::Term {
                    term: SourceSetTermId::new(0),
                    role: SourceSetTypeRole::ChoiceTarget,
                },
                site: TypedSiteRef::Node(TypedNodeId::new(27)),
                source_range: ast.nodes()[27].range,
                spelling: "set".to_owned(),
                head_site: TypedSiteRef::Node(TypedNodeId::new(26)),
                head_range: ast.nodes()[26].range,
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                head: SourceSetTypeHead::BuiltinSet,
            });
        }
        SetEnumerationHandoffMutation::ExtraCondition => {
            input.conditions.push(SourceSetConditionInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                colon_site: TypedSiteRef::Node(TypedNodeId::new(7)),
                colon_range: ast.nodes()[7].range,
                colon_spelling: ":".to_owned(),
                condition_site: TypedSiteRef::Node(TypedNodeId::new(35)),
                source_range: ast.nodes()[35].range,
                spelling: "x = x".to_owned(),
                recovery: SourceSetTermRecovery::Normal,
            });
        }
        SetEnumerationHandoffMutation::EdgeTerm(index) => {
            input.edges[index].term = SourceSetTermId::new(1);
        }
        SetEnumerationHandoffMutation::EdgeOrdinal(index) => {
            input.edges[index].ordinal = 1 - index;
        }
        SetEnumerationHandoffMutation::EdgeRole(index) => {
            input.edges[index].role = SourceSetEdgeRole::QuaBase;
        }
        SetEnumerationHandoffMutation::EdgeTarget(index) => {
            input.edges[index].target =
                SourceSetTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(0));
        }
        SetEnumerationHandoffMutation::RequestTerm => {
            input.requests[0].term = SourceSetTermId::new(1);
        }
        SetEnumerationHandoffMutation::RequestOrdinal => input.requests[0].ordinal = 1,
        SetEnumerationHandoffMutation::RequestKind => {
            input.requests[0].kind = SourceSetRequestKind::ChoiceNonempty;
        }
        SetEnumerationHandoffMutation::RequestGenerator => {
            input.requests[0].generator = Some(SourceSetGeneratorId::new(0));
        }
        SetEnumerationHandoffMutation::RequestTypeSite => {
            input.requests[0].type_site = Some(SourceSetTypeSiteId::new(0));
        }
        SetEnumerationHandoffMutation::CoherentApplicationFingerprint
        | SetEnumerationHandoffMutation::CoherentStructureFingerprint => {}
    }
}

#[cfg(test)]
// Rationale: keep every frozen B3P corruption boundary explicit in one test seam.
#[allow(clippy::too_many_arguments)]
pub(in crate::runner) fn set_enumeration_proof_context_handoff_for_test(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    loaded_source: &str,
    options: SetEnumerationProofContextTestOptions,
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    if set_enumeration_selection_stage_for_test(
        ast,
        module,
        shells,
        symbols,
        loaded_source,
        options.surface,
        options.resolver,
    ) != SetEnumerationSelectionStage::Selected
    {
        return None;
    }
    let mutated_binding_env;
    let binding_env = if let SetEnumerationBindingMutation::Field(field) = options.binding {
        mutated_binding_env =
            match task258b3m2b2b3p_binding_env_with_field_mutation(binding_env, field) {
                Ok(binding_env) => binding_env,
                Err(error) => return Some(Err(format!("Task48: {error}"))),
            };
        &mutated_binding_env
    } else {
        binding_env
    };
    if !task258b3m2b2b3p_binding_contract(
        ast,
        module,
        binding_env,
        SetEnumerationBindingMutation::None,
    ) {
        return Some(Err(
            "Task48: exact proof-context profile rejected".to_owned()
        ));
    }
    let mut roots = task258b3m2b2b3p_primary_roots().to_vec();
    match options.primary {
        SetEnumerationPrimaryMutation::DuplicateRoot => roots[3] = roots[2],
        SetEnumerationPrimaryMutation::MissingRoot => {
            roots.pop();
        }
        _ => {}
    }
    let primary_binding_env_storage;
    let primary_binding_env = match options.primary {
        SetEnumerationPrimaryMutation::ReferenceScopeModule => {
            primary_binding_env_storage =
                match task258b3m2b2b3p_binding_env_with_field_mutation(binding_env, 8) {
                    Ok(binding_env) => binding_env,
                    Err(error) => return Some(Err(format!("Task252: {error}"))),
                };
            &primary_binding_env_storage
        }
        SetEnumerationPrimaryMutation::ReferenceScopeProof => {
            primary_binding_env_storage =
                match task258b3m2b2b3p_binding_env_with_field_mutation(binding_env, 15) {
                    Ok(binding_env) => binding_env,
                    Err(error) => return Some(Err(format!("Task252: {error}"))),
                };
            &primary_binding_env_storage
        }
        SetEnumerationPrimaryMutation::ReferenceUseOrdinal => {
            primary_binding_env_storage =
                match task258b3m2b2b3p_binding_env_with_prior_event(binding_env) {
                    Ok(binding_env) => binding_env,
                    Err(error) => return Some(Err(format!("Task252: {error}"))),
                };
            &primary_binding_env_storage
        }
        _ => binding_env,
    };
    let source_term = match source_term_parts_for_context_roots(
        ast,
        module.clone(),
        primary_binding_env,
        roots,
        &BTreeMap::new(),
    ) {
        Ok(source_term) => source_term,
        Err(error) => return Some(Err(format!("Task252: {error}"))),
    };
    let source_term = match task258b3m2b2b3p_source_term_with_mutation(
        ast,
        module,
        primary_binding_env,
        source_term,
        options.primary,
    ) {
        Ok(source_term) => source_term,
        Err(error) => return Some(Err(format!("Task252: {error}"))),
    };
    if options.primary == SetEnumerationPrimaryMutation::ReferenceUseOrdinal {
        let changed = source_term
            .handoff
            .references()
            .iter()
            .map(|(_, reference)| (reference.binding().index(), reference.use_ordinal()))
            .collect::<Vec<_>>();
        if changed != [(0, 2), (0, 2), (0, 2), (0, 2)] {
            return Some(Err(format!(
                "Task252: use-ordinal substitution did not reach all four BindingId(0) rows: {changed:?}"
            )));
        }
    }
    if !task258b3m2b2b3p_primary_profile_is_exact(ast, module, binding_env, &source_term) {
        return Some(Err("Task252: exact lower profile mismatch".to_owned()));
    }
    let primary_fingerprint = source_term.handoff.debug_text();
    let dependency_probe = match options.handoff {
        SetEnumerationHandoffMutation::CoherentApplicationFingerprint => Some((
            "application",
            task258b3m2b2b3p_coherent_application_fingerprint_probe(
                ast,
                module,
                symbols,
                binding_env,
                &source_term,
            ),
        )),
        SetEnumerationHandoffMutation::CoherentStructureFingerprint => Some((
            "structure",
            task258b3m2b2b3p_coherent_structure_fingerprint_probe(
                ast,
                module,
                symbols,
                binding_env,
                &source_term,
            ),
        )),
        _ => None,
    };
    if let Some((kind, probe)) = dependency_probe {
        match probe {
            Ok(handoff)
                if !task258b3m2b2b3p_fingerprint_profile_is_exact(
                    &primary_fingerprint,
                    &handoff,
                ) =>
            {
                return Some(Err(format!(
                    "Task255: exact B3P fingerprint-only profile rejected coherent non-None {kind} dependency fingerprint"
                )));
            }
            Ok(_) => {
                return Some(Err(format!(
                    "Task255: BUG: exact B3P fingerprint-only profile accepted coherent non-None {kind} dependency fingerprint"
                )));
            }
            Err(error) => return Some(Err(format!("Task255: {error}"))),
        }
    }
    if options.primary == SetEnumerationPrimaryMutation::None
        && options.handoff == SetEnumerationHandoffMutation::None
        && options.final_clone == SetEnumerationFinalMutation::None
    {
        let result = source_set_term_output_with_source_term_in_context(
            ast,
            module.clone(),
            binding_env.clone(),
            &[40],
            source_term,
            BindingContextId::new(1),
        )
        .map_err(|error| format!("Task255: {error}"))
        .and_then(|output| {
            let handoff = output
                .typed_ast
                .source_set_term()
                .ok_or_else(|| "Task255: exact B3P handoff disappeared".to_owned())?;
            if task258b3m2b2b3p_set_handoff_profile_is_exact(
                ast,
                module,
                &primary_fingerprint,
                handoff,
            ) {
                Ok(output)
            } else {
                Err("Task255: exact B3P handoff profile mismatch".to_owned())
            }
        });
        return Some(result);
    }
    let extracted = match extract_set_terms(
        ast,
        module,
        BindingContextId::new(1),
        &[40],
        None,
        None,
        &BTreeSet::new(),
        false,
    ) {
        Ok(extracted) => extracted,
        Err(error) => return Some(Err(format!("Task255: {error}"))),
    };
    let output = match build_output(
        ast,
        module.clone(),
        binding_env.clone(),
        extracted,
        Some(SyntheticSourceSetTermDependencies {
            arena: source_term.arena,
            primary: source_term.handoff,
            application: None,
            structure: None,
        }),
        |input| task258b3m2b2b3p_mutate_handoff(input, ast, options.handoff),
    ) {
        Ok(output) => output,
        Err(error) => return Some(Err(format!("Task255: {error}"))),
    };
    let Some(set_handoff) = output.typed_ast.source_set_term() else {
        return Some(Err("Task255: exact B3P handoff disappeared".to_owned()));
    };
    if !task258b3m2b2b3p_set_handoff_profile_is_exact(
        ast,
        module,
        &primary_fingerprint,
        set_handoff,
    ) {
        return Some(Err("Task255: exact B3P handoff profile mismatch".to_owned()));
    }
    if options.primary == SetEnumerationPrimaryMutation::StaleFingerprintReplay {
        let stale = match source_term_parts_for_context_roots(
            ast,
            module.clone(),
            binding_env,
            task258b3m2b2b3p_primary_roots()
                .into_iter()
                .filter(|(root, _)| *root != 38),
            &BTreeMap::new(),
        ) {
            Ok(stale) => stale,
            Err(error) => return Some(Err(format!("Task252: {error}"))),
        };
        let typed = match TypedAst::try_new(TypedAstParts {
            source_id: ast.source_id,
            module_id: module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: output.typed_ast.nodes().clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        }) {
            Ok(typed) => match typed.with_source_term(stale.handoff) {
                Ok(typed) => typed,
                Err(error) => return Some(Err(format!("TypedAst: {error}"))),
            },
            Err(error) => return Some(Err(format!("TypedAst: {error}"))),
        };
        return Some(
            match typed.with_source_set_term(
                output
                    .typed_ast
                    .source_set_term()
                    .expect("B3P baseline set handoff")
                    .clone(),
            ) {
                Ok(_) => Err("BUG: TypedAst accepted stale Task252 fingerprint".to_owned()),
                Err(error) => Err(format!(
                    "TypedAst: rejected stale Task252 fingerprint: {error}"
                )),
            },
        );
    }
    if options.final_clone == SetEnumerationFinalMutation::TypedClone {
        let typed = TypedAst::try_new(TypedAstParts {
            source_id: ast.source_id,
            module_id: module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: output.typed_ast.nodes().clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .map_err(|error| error.to_string());
        return Some(
            match typed.and_then(|typed| {
                typed
                    .with_source_set_term(
                        output
                            .typed_ast
                            .source_set_term()
                            .expect("B3P set handoff")
                            .clone(),
                    )
                    .map_err(|error| error.to_string())
            }) {
                Ok(_) => Err("BUG: TypedAst accepted set handoff without Task252 clone".to_owned()),
                Err(error) => Err(format!("TypedAst: rejected clone corruption: {error}")),
            },
        );
    }
    if options.final_clone == SetEnumerationFinalMutation::ResolvedClone {
        let hints = vec![ResolvedNodeKindHint {
            typed_node: TypedNodeId::new(57),
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.term.surface"),
            },
        }];
        return Some(
            match assemble_empty_resolved_typed_ast(&output.typed_ast, hints) {
                Ok(_) => Err("BUG: ResolvedTypedAst accepted invalid clone hint".to_owned()),
                Err(error) => Err(format!(
                    "ResolvedTypedAst: rejected clone corruption: {error}"
                )),
            },
        );
    }
    Some(Ok(output))
}

fn exact_conditioned_route(ast: &SurfaceAst, source_text: &str) -> Option<ExactConditionedRoute> {
    if source_text != TASK255C1_SOURCE
        || source_text.len() != 191
        || ast.nodes().iter().any(|node| node.recovered)
    {
        return None;
    }
    let items = exact_compilation_item_list(ast)?;
    let item_ids = structural_child_ids(ast, items);
    let [import_id, definition_id] = item_ids.as_slice() else {
        return None;
    };
    let import = ast.node(*import_id)?;
    let definition = ast.node(*definition_id)?;
    if !is_exact_parser_type_fixtures_import(ast, import)
        || !matches!(definition.kind, SurfaceNodeKind::DefinitionBlockItem)
        || subtree_tokens(ast, definition)
            != [
                "definition",
                "func",
                "Task255ConditionedComprehensionDef",
                ":",
                "task255_conditioned_comprehension",
                "->",
                "set",
                "equals",
                "{",
                "1",
                "++",
                "2",
                "where",
                "candidate255c",
                "is",
                "set",
                ":",
                "3",
                "=",
                "4",
                "}",
                ";",
                "end",
                ";",
            ]
    {
        return None;
    }
    let unique = |kind: &SurfaceNodeKind, start: usize, end: usize| {
        let matches = ast
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                std::mem::discriminant(&node.kind) == std::mem::discriminant(kind)
                    && node.range.start == start
                    && node.range.end == end
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let [index] = matches.as_slice() else {
            return None;
        };
        Some(*index)
    };
    let root = unique(&SurfaceNodeKind::SetComprehension, 139, 184)?;
    let application = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            matches!(
                &node.kind,
                SurfaceNodeKind::InfixExpression(operator)
                    if operator.spelling.as_ref() == "++"
            ) && node.range.start == 141
                && node.range.end == 147
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [application] = application.as_slice() else {
        return None;
    };
    let generator = unique(&SurfaceNodeKind::ComprehensionVariableSegment, 154, 174)?;
    let mapper_expression = unique(&SurfaceNodeKind::TermExpression, 141, 147)?;
    let condition = unique(&SurfaceNodeKind::FormulaExpression, 177, 182)?;
    let inner = unique(&SurfaceNodeKind::BuiltinPredicateApplication, 177, 182)?;
    let root_node = &ast.nodes()[root];
    if direct_token_texts(ast, root_node).as_slice() != ["{", "where", ":", "}"]
        || structural_child_ids(ast, root_node)
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>()
            != [mapper_expression, generator, condition]
        || structural_child_ids(ast, &ast.nodes()[condition])
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>()
            != [inner]
        || subtree_tokens(ast, root_node)
            != [
                "{",
                "1",
                "++",
                "2",
                "where",
                "candidate255c",
                "is",
                "set",
                ":",
                "3",
                "=",
                "4",
                "}",
            ]
    {
        return None;
    }
    let primary_roots = [
        (141, 142, "1"),
        (146, 147, "2"),
        (177, 178, "3"),
        (181, 182, "4"),
    ]
    .into_iter()
    .map(|(start, end, spelling)| {
        let index = unique(&SurfaceNodeKind::NumeralTerm, start, end)?;
        (subtree_tokens(ast, &ast.nodes()[index]) == [spelling]).then_some(index)
    })
    .collect::<Option<Vec<_>>>()?;
    Some(ExactConditionedRoute {
        root,
        application: *application,
        primary_roots,
    })
}

fn build_conditioned_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    route: ExactConditionedRoute,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Result<SourceSetTermRouteOutput, String> {
    let binding_env =
        source_module_binding_env(ast, module.clone()).map_err(|error| error.to_string())?;
    let application_kinds = unwrapped_imported_source_application_owned_node_kinds(
        ast,
        &module,
        symbols,
        route.application,
    )
    .ok_or_else(|| "Task255C1 unwrapped Task253 selector rejected the exact mapper".to_owned())?;
    #[cfg(test)]
    let source_term = synthetic_source_term_parts_for_roots(
        ast,
        module.clone(),
        &binding_env,
        route.primary_roots.iter().copied(),
        BindingContextId::new(0),
        &application_kinds,
        &BTreeMap::new(),
    )?;
    #[cfg(not(test))]
    let source_term = source_term_parts_for_roots(
        ast,
        module.clone(),
        &binding_env,
        route.primary_roots.iter().copied(),
        BindingContextId::new(0),
        &application_kinds,
    )?;
    let application = unwrapped_imported_source_application_handoff(
        ast,
        &module,
        symbols,
        &binding_env,
        &source_term,
        route.application,
    )
    .ok_or_else(|| "Task255C1 reusable Task253 seam rejected the exact mapper".to_owned())??;
    let extracted = extract_set_terms(
        ast,
        &module,
        BindingContextId::new(0),
        &[route.root],
        Some(&application),
        None,
        &BTreeSet::new(),
        true,
    )?;
    build_output(
        ast,
        module,
        binding_env,
        extracted,
        Some(SyntheticSourceSetTermDependencies {
            arena: source_term.arena,
            primary: source_term.handoff,
            application: Some(application),
            structure: None,
        }),
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn synthetic_source_set_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceSetTermDependencies>,
    degraded_terms: &BTreeSet<usize>,
) -> Result<SourceSetTermRouteOutput, String> {
    synthetic_source_set_term_output_with_mutation(
        ast,
        module,
        binding_env,
        roots,
        dependencies,
        degraded_terms,
        |_| {},
    )
}

pub(super) fn source_set_term_output_with_source_term(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    roots: &[usize],
    source_term: SourceTermParts,
) -> Result<SourceSetTermRouteOutput, String> {
    source_set_term_output_with_source_term_in_context(
        ast,
        module,
        binding_env,
        roots,
        source_term,
        BindingContextId::new(0),
    )
}

pub(super) fn source_set_term_output_with_source_term_in_context(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    roots: &[usize],
    source_term: SourceTermParts,
    context: BindingContextId,
) -> Result<SourceSetTermRouteOutput, String> {
    let extracted = extract_set_terms(
        ast,
        &module,
        context,
        roots,
        None,
        None,
        &BTreeSet::new(),
        false,
    )?;
    build_output(
        ast,
        module,
        binding_env,
        extracted,
        Some(SyntheticSourceSetTermDependencies {
            arena: source_term.arena,
            primary: source_term.handoff,
            application: None,
            structure: None,
        }),
        |_| {},
    )
}

#[cfg(test)]
// Rationale: keep the synthetic mutation boundary's explicit authority inputs visible.
#[allow(clippy::too_many_arguments)]
pub(in crate::runner) fn synthetic_source_set_term_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceSetTermDependencies>,
    degraded_terms: &BTreeSet<usize>,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Result<SourceSetTermRouteOutput, String> {
    let applications = dependencies
        .as_ref()
        .and_then(|dependencies| dependencies.application.as_ref());
    let structures = dependencies
        .as_ref()
        .and_then(|dependencies| dependencies.structure.as_ref());
    let extracted = extract_set_terms(
        ast,
        &module,
        BindingContextId::new(0),
        roots,
        applications,
        structures,
        degraded_terms,
        false,
    )?;
    build_output(ast, module, binding_env, extracted, dependencies, mutate)
}

fn exact_real_roots(ast: &SurfaceAst) -> Option<Vec<usize>> {
    let items = exact_compilation_item_list(ast)?;
    let item_ids = structural_child_ids(ast, items);
    let [reserve_id, definition_id] = item_ids.as_slice() else {
        return None;
    };
    let reserve = ast.node(*reserve_id)?;
    let definition = ast.node(*definition_id)?;
    if !matches!(reserve.kind, SurfaceNodeKind::ReserveItem)
        || !matches!(definition.kind, SurfaceNodeKind::DefinitionBlockItem)
        || subtree_has_recovery(ast, reserve)
        || subtree_has_recovery(ast, definition)
        || subtree_tokens(ast, reserve) != ["reserve", "seed", "for", "set", ";"]
    {
        return None;
    }
    let expected_definition = [
        "definition",
        "let",
        "seed",
        "be",
        "set",
        ";",
        "func",
        "Task255EnumerationDef",
        ":",
        "task255_enumeration",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "{",
        "1",
        ",",
        "2",
        "}",
        ";",
        "func",
        "Task255ComprehensionDef",
        ":",
        "task255_comprehension",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "{",
        "3",
        "where",
        "candidate255",
        "is",
        "set",
        "}",
        ";",
        "func",
        "Task255ChoiceDef",
        ":",
        "task255_choice",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "the",
        "set",
        ";",
        "func",
        "Task255QuaDef",
        ":",
        "task255_qua",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "4",
        "qua",
        "set",
        ";",
        "end",
        ";",
    ];
    if subtree_tokens(ast, definition) != expected_definition {
        return None;
    }

    let parents = parent_indexes(ast);
    let roots = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| is_set_term_kind(&node.kind))
        .filter(|(index, _)| !has_set_term_ancestor(ast, &parents, *index))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let kinds = roots
        .iter()
        .filter_map(|root| ast.nodes().get(*root))
        .map(|node| &node.kind)
        .collect::<Vec<_>>();
    if roots.len() == 4
        && kinds
            .iter()
            .filter(|kind| matches!(kind, SurfaceNodeKind::SetEnumeration))
            .count()
            == 1
        && kinds
            .iter()
            .filter(|kind| matches!(kind, SurfaceNodeKind::SetComprehension))
            .count()
            == 1
        && kinds
            .iter()
            .filter(|kind| matches!(kind, SurfaceNodeKind::ChoiceTerm))
            .count()
            == 1
        && kinds
            .iter()
            .filter(|kind| matches!(kind, SurfaceNodeKind::QuaExpression))
            .count()
            == 1
    {
        Some(roots)
    } else {
        None
    }
}

// Rationale: keep all recursive extraction dependencies explicit at this private boundary.
#[allow(clippy::too_many_arguments)]
fn extract_set_terms(
    ast: &SurfaceAst,
    module: &ModuleId,
    context: BindingContextId,
    roots: &[usize],
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
) -> Result<ExtractedSetTerms, String> {
    let mut extracted = ExtractedSetTerms {
        context,
        terms: Vec::new(),
        wrappers: Vec::new(),
        generators: Vec::new(),
        type_sites: Vec::new(),
        conditions: Vec::new(),
        edges: Vec::new(),
        primary_roots: Vec::new(),
    };
    let mut ordered_roots = roots.to_vec();
    ordered_roots.sort_by_key(|root| {
        ast.nodes()
            .get(*root)
            .map_or((usize::MAX, usize::MAX), |node| {
                (node.range.start, usize::MAX - node.range.end)
            })
    });
    ordered_roots.dedup();
    for root in ordered_roots {
        if set_root_is_excluded(ast, root, applications, structures, admit_conditions) {
            continue;
        }
        collect_target(
            ast,
            module,
            root,
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            &mut extracted,
        )?
        .set_term()
        .ok_or_else(|| "synthetic source-set root is not a Task-255 term".to_owned())?;
    }
    normalize_extracted_tables(ast, &mut extracted);
    Ok(extracted)
}

fn normalize_extracted_tables(ast: &SurfaceAst, extracted: &mut ExtractedSetTerms) {
    extracted
        .wrappers
        .sort_by_key(|wrapper| (wrapper.term.index(), wrapper.ordinal));

    let mut generator_order = (0..extracted.generators.len()).collect::<Vec<_>>();
    generator_order.sort_by_key(|index| {
        let generator = &extracted.generators[*index];
        (generator.term.index(), generator.ordinal)
    });
    let generator_remap = generator_order
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, SourceSetGeneratorId::new(new)))
        .collect::<BTreeMap<_, _>>();
    let old_generators = std::mem::take(&mut extracted.generators);
    extracted.generators = generator_order
        .into_iter()
        .map(|old| old_generators[old].clone())
        .collect();
    for type_site in &mut extracted.type_sites {
        if let SourceSetTypeOwner::Generator(generator) = &mut type_site.owner {
            *generator = generator_remap[&generator.index()];
        }
    }

    let mut type_site_order = (0..extracted.type_sites.len()).collect::<Vec<_>>();
    type_site_order.sort_by_key(|index| {
        let node = &ast.nodes()[extracted.type_sites[*index].node];
        (node.range.start, node.range.end, *index)
    });
    let type_site_remap = type_site_order
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, SourceSetTypeSiteId::new(new)))
        .collect::<BTreeMap<_, _>>();
    let old_type_sites = std::mem::take(&mut extracted.type_sites);
    extracted.type_sites = type_site_order
        .into_iter()
        .map(|old| old_type_sites[old].clone())
        .collect();
    for generator in &mut extracted.generators {
        generator.type_site = type_site_remap[&generator.type_site.index()];
    }

    extracted
        .conditions
        .sort_by_key(|condition| (condition.term.index(), condition.ordinal));
    extracted
        .edges
        .sort_by_key(|edge| (edge.term.index(), edge.ordinal));
}

impl ExtractedTarget {
    const fn set_term(self) -> Option<SourceSetTermId> {
        match self {
            Self::SetTerm(term) => Some(term),
            Self::Primary(_) | Self::Application(_) | Self::Structure(_) => None,
        }
    }
}

// Rationale: keep the recursive target classifier's family dependencies explicit.
#[allow(clippy::too_many_arguments)]
fn collect_target(
    ast: &SurfaceAst,
    module: &ModuleId,
    node_index: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
    extracted: &mut ExtractedSetTerms,
) -> Result<ExtractedTarget, String> {
    let original = ast
        .nodes()
        .get(node_index)
        .ok_or_else(|| "source-set child site disappeared".to_owned())?;
    if original.range.source_id != ast.source_id {
        return Err("source-set child source identity mismatch".to_owned());
    }
    if let Some(application) = application_target(applications, original.range) {
        return Ok(ExtractedTarget::Application(application));
    }
    if let Some(structure) = structure_target(structures, original.range) {
        return Ok(ExtractedTarget::Structure(structure));
    }

    let (core, wrappers) = peel_set_shells(ast, node_index)?;
    let node = &ast.nodes()[core];
    if !is_set_term_kind(&node.kind) {
        extracted.primary_roots.push(node_index);
        return Ok(ExtractedTarget::Primary(node_index));
    }
    if set_term_shape_is_excluded(ast, core, admit_conditions) {
        return Err("source-set term has an excluded source shape".to_owned());
    }
    if subtree_has_recovery(ast, node) && !degraded_terms.contains(&core) {
        return Err("source-set normal term contains recovery".to_owned());
    }

    let term = SourceSetTermId::new(extracted.terms.len());
    let kind = match node.kind {
        SurfaceNodeKind::SetEnumeration => SourceSetTermKind::Enumeration,
        SurfaceNodeKind::SetComprehension => SourceSetTermKind::Comprehension,
        SurfaceNodeKind::ChoiceTerm => SourceSetTermKind::Choice,
        SurfaceNodeKind::QuaExpression => SourceSetTermKind::Qua,
        _ => unreachable!("guarded above"),
    };
    let recovery = if degraded_terms.contains(&core) {
        SourceSetTermRecovery::Degraded
    } else {
        SourceSetTermRecovery::Normal
    };
    extracted.terms.push(ExtractedTerm {
        node: core,
        kind,
        recovery,
    });
    for (ordinal, wrapper) in wrappers.into_iter().enumerate() {
        extracted.wrappers.push(ExtractedWrapper {
            term,
            ordinal,
            node: wrapper,
            recovery,
        });
    }

    match kind {
        SourceSetTermKind::Enumeration => collect_enumeration(
            ast,
            module,
            term,
            core,
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            extracted,
        )?,
        SourceSetTermKind::Comprehension => collect_comprehension(
            ast,
            module,
            term,
            core,
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            extracted,
        )?,
        SourceSetTermKind::Choice => collect_choice(ast, term, core, recovery, extracted)?,
        SourceSetTermKind::Qua => collect_qua(
            ast,
            module,
            term,
            core,
            recovery,
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            extracted,
        )?,
        _ => return Err("unsupported source-set term kind".to_owned()),
    }
    Ok(ExtractedTarget::SetTerm(term))
}

// Rationale: keep enumeration child ownership inputs explicit during recursion.
#[allow(clippy::too_many_arguments)]
fn collect_enumeration(
    ast: &SurfaceAst,
    module: &ModuleId,
    term: SourceSetTermId,
    node_index: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
    extracted: &mut ExtractedSetTerms,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let children = structural_child_ids(ast, node);
    let punctuation = direct_token_texts(ast, node);
    let expected_punctuation = if children.is_empty() {
        vec!["{".to_owned(), "}".to_owned()]
    } else {
        let mut expected = vec!["{".to_owned()];
        expected.extend((1..children.len()).map(|_| ",".to_owned()));
        expected.push("}".to_owned());
        expected
    };
    if punctuation != expected_punctuation {
        return Err("source-set enumeration punctuation is not canonical".to_owned());
    }
    for child in children {
        let target = collect_target(
            ast,
            module,
            child.index(),
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            extracted,
        )?;
        push_edge(
            extracted,
            term,
            SourceSetEdgeRole::EnumerationElement,
            target,
        );
    }
    Ok(())
}

// Rationale: keep comprehension binding deferrals and child dependencies explicit.
#[allow(clippy::too_many_arguments)]
fn collect_comprehension(
    ast: &SurfaceAst,
    module: &ModuleId,
    term: SourceSetTermId,
    node_index: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
    extracted: &mut ExtractedSetTerms,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let children = structural_child_ids(ast, node);
    let Some((mapper, tail)) = children.split_first() else {
        return Err("source-set comprehension lost its mapper".to_owned());
    };
    let (generators, condition) = match tail.split_last() {
        Some((condition, generators))
            if ast
                .node(*condition)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FormulaExpression)) =>
        {
            if !admit_conditions {
                return Err("source-set conditioned comprehension is outside this route".to_owned());
            }
            (generators, Some(*condition))
        }
        _ => (tail, None),
    };
    if generators.is_empty()
        || generators.iter().any(|generator| {
            ast.node(*generator).is_none_or(|node| {
                !matches!(node.kind, SurfaceNodeKind::ComprehensionVariableSegment)
            })
        })
    {
        return Err("source-set comprehension generator shape drift".to_owned());
    }
    let punctuation = direct_token_texts(ast, node);
    let mut expected = vec!["{".to_owned(), "where".to_owned()];
    expected.extend((1..generators.len()).map(|_| ",".to_owned()));
    if condition.is_some() {
        expected.push(":".to_owned());
    }
    expected.push("}".to_owned());
    if punctuation != expected {
        return Err("source-set comprehension punctuation is not canonical".to_owned());
    }
    let target = collect_target(
        ast,
        module,
        mapper.index(),
        applications,
        structures,
        degraded_terms,
        admit_conditions,
        extracted,
    )?;
    push_edge(
        extracted,
        term,
        SourceSetEdgeRole::ComprehensionMapper,
        target,
    );

    for (ordinal, generator_id) in generators.iter().copied().enumerate() {
        let generator = ast
            .node(generator_id)
            .ok_or_else(|| "source-set generator disappeared".to_owned())?;
        let direct = generator
            .children
            .iter()
            .filter_map(|child| {
                ast.node(*child)
                    .and_then(|node| node.token_text().map(|text| (*child, text)))
            })
            .collect::<Vec<_>>();
        let [(identifier, spelling), (_, is_keyword)] = direct.as_slice() else {
            return Err("source-set generator requires one identifier and `is`".to_owned());
        };
        if spelling.is_empty() || *is_keyword != "is" {
            return Err("source-set generator spelling is not canonical".to_owned());
        }
        let type_children = structural_child_ids(ast, generator);
        let [type_expression] = type_children.as_slice() else {
            return Err("source-set generator requires one target type".to_owned());
        };
        let generator_id = SourceSetGeneratorId::new(extracted.generators.len());
        let type_site = SourceSetTypeSiteId::new(extracted.type_sites.len());
        let type_row = extract_bare_type_site(
            ast,
            type_expression.index(),
            SourceSetTypeOwner::Generator(generator_id),
            extracted.terms[term.index()].recovery,
        )?;
        extracted.type_sites.push(type_row);
        extracted.generators.push(ExtractedGenerator {
            term,
            ordinal,
            node: identifier.index(),
            recovery: extracted.terms[term.index()].recovery,
            type_site,
        });
    }
    if let Some(condition) = condition {
        let colon_nodes = node
            .children
            .iter()
            .copied()
            .filter(|child| ast.node(*child).and_then(SurfaceNode::token_text) == Some(":"))
            .collect::<Vec<_>>();
        let [colon] = colon_nodes.as_slice() else {
            return Err("source-set condition requires one direct colon".to_owned());
        };
        let condition_node = ast
            .node(condition)
            .ok_or_else(|| "source-set condition wrapper disappeared".to_owned())?;
        if condition_node.recovered || subtree_tokens(ast, condition_node).is_empty() {
            return Err("source-set condition wrapper is not a normal formula".to_owned());
        }
        extracted.conditions.push(ExtractedCondition {
            term,
            ordinal: 0,
            colon_node: colon.index(),
            condition_node: condition.index(),
            recovery: extracted.terms[term.index()].recovery,
        });
        collect_condition_primary_roots(ast, condition.index(), &mut extracted.primary_roots);
    }
    Ok(())
}

fn collect_condition_primary_roots(ast: &SurfaceAst, node: usize, roots: &mut Vec<usize>) {
    let Some(node_row) = ast.nodes().get(node) else {
        return;
    };
    if matches!(
        node_row.kind,
        SurfaceNodeKind::TermReference
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::ItTerm
            | SurfaceNodeKind::ParenthesizedTerm
    ) {
        roots.push(node);
        return;
    }
    for child in &node_row.children {
        collect_condition_primary_roots(ast, child.index(), roots);
    }
}

fn collect_choice(
    ast: &SurfaceAst,
    term: SourceSetTermId,
    node_index: usize,
    recovery: SourceSetTermRecovery,
    extracted: &mut ExtractedSetTerms,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    if direct_token_texts(ast, node).as_slice() != ["the"] {
        return Err("source-set choice punctuation is not canonical".to_owned());
    }
    let children = structural_child_ids(ast, node);
    let [target_type] = children.as_slice() else {
        return Err("source-set choice requires one target type".to_owned());
    };
    let type_site = extract_bare_type_site(
        ast,
        target_type.index(),
        SourceSetTypeOwner::Term {
            term,
            role: SourceSetTypeRole::ChoiceTarget,
        },
        recovery,
    )?;
    extracted.type_sites.push(type_site);
    Ok(())
}

// Rationale: keep qua base ownership and optional family dependencies explicit.
#[allow(clippy::too_many_arguments)]
fn collect_qua(
    ast: &SurfaceAst,
    module: &ModuleId,
    term: SourceSetTermId,
    node_index: usize,
    recovery: SourceSetTermRecovery,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
    extracted: &mut ExtractedSetTerms,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    if direct_token_texts(ast, node).as_slice() != ["qua"] {
        return Err("source-set qua punctuation is not canonical".to_owned());
    }
    let children = structural_child_ids(ast, node);
    let [base, target_type] = children.as_slice() else {
        return Err("source-set qua requires one base and one target type".to_owned());
    };
    let target = collect_target(
        ast,
        module,
        base.index(),
        applications,
        structures,
        degraded_terms,
        admit_conditions,
        extracted,
    )?;
    push_edge(extracted, term, SourceSetEdgeRole::QuaBase, target);
    let type_site = extract_bare_type_site(
        ast,
        target_type.index(),
        SourceSetTypeOwner::Term {
            term,
            role: SourceSetTypeRole::QuaTarget,
        },
        recovery,
    )?;
    extracted.type_sites.push(type_site);
    Ok(())
}

fn extract_bare_type_site(
    ast: &SurfaceAst,
    node_index: usize,
    owner: SourceSetTypeOwner,
    recovery: SourceSetTermRecovery,
) -> Result<ExtractedTypeSite, String> {
    let node = ast
        .nodes()
        .get(node_index)
        .ok_or_else(|| "source-set target type disappeared".to_owned())?;
    if !matches!(node.kind, SurfaceNodeKind::TypeExpression)
        || (recovery == SourceSetTermRecovery::Normal && subtree_has_recovery(ast, node))
    {
        return Err("source-set target type is not one normal type expression".to_owned());
    }
    let children = structural_child_ids(ast, node);
    let [head_id] = children.as_slice() else {
        return Err("source-set target type is not bare".to_owned());
    };
    let head_node = ast
        .node(*head_id)
        .ok_or_else(|| "source-set target type head disappeared".to_owned())?;
    if !matches!(head_node.kind, SurfaceNodeKind::TypeHead)
        || !structural_child_ids(ast, head_node).is_empty()
    {
        return Err("source-set target type head is not bare".to_owned());
    }
    let head = match direct_token_texts(ast, head_node).as_slice() {
        [spelling] if spelling == "set" => SourceSetTypeHead::BuiltinSet,
        [spelling] if spelling == "object" => SourceSetTypeHead::BuiltinObject,
        _ => return Err("source-set target type head is unsupported".to_owned()),
    };
    Ok(ExtractedTypeSite {
        owner,
        node: node_index,
        head_node: head_id.index(),
        head,
        recovery,
    })
}

fn push_edge(
    extracted: &mut ExtractedSetTerms,
    term: SourceSetTermId,
    role: SourceSetEdgeRole,
    target: ExtractedTarget,
) {
    let ordinal = extracted
        .edges
        .iter()
        .filter(|edge| edge.term == term)
        .count();
    extracted.edges.push(ExtractedEdge {
        term,
        ordinal,
        role,
        target,
    });
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    extracted: ExtractedSetTerms,
    dependencies: Option<SyntheticSourceSetTermDependencies>,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Result<SourceSetTermRouteOutput, String> {
    if binding_env.source_id() != ast.source_id || binding_env.module_id() != &module {
        return Err("source-set binding environment identity mismatch".to_owned());
    }
    let mut kinds = BTreeMap::new();
    let mut recoveries = BTreeMap::new();
    for term in &extracted.terms {
        insert_kind(&mut kinds, term.node, term_kind_key(term.kind))?;
        if term.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(term.node, NodeRecoveryState::Degraded);
        }
    }
    for wrapper in &extracted.wrappers {
        insert_kind(&mut kinds, wrapper.node, "source.term.set.parenthesized")?;
        if wrapper.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(wrapper.node, NodeRecoveryState::Degraded);
        }
    }
    for generator in &extracted.generators {
        insert_kind(
            &mut kinds,
            generator.node,
            "source.term.set.comprehension-generator",
        )?;
        if generator.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(generator.node, NodeRecoveryState::Degraded);
        }
    }
    for type_site in &extracted.type_sites {
        insert_kind(&mut kinds, type_site.node, "source.term.set.target-type")?;
        insert_kind(
            &mut kinds,
            type_site.head_node,
            "source.term.set.target-type-head",
        )?;
        if type_site.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(type_site.node, NodeRecoveryState::Degraded);
            recoveries.insert(type_site.head_node, NodeRecoveryState::Degraded);
        }
    }
    for condition in &extracted.conditions {
        insert_kind(
            &mut kinds,
            condition.colon_node,
            "source.term.set.comprehension-condition-colon",
        )?;
        insert_kind(
            &mut kinds,
            condition.condition_node,
            "source.term.set.comprehension-condition",
        )?;
        if condition.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(condition.colon_node, NodeRecoveryState::Degraded);
            recoveries.insert(condition.condition_node, NodeRecoveryState::Degraded);
        }
    }

    let (arena, primary, application, structure) = if let Some(dependencies) = dependencies {
        let arena = arena_with_overrides(ast, &dependencies.arena, &kinds, &recoveries)?;
        (
            arena,
            dependencies.primary,
            dependencies.application,
            dependencies.structure,
        )
    } else {
        #[cfg(test)]
        let parts = synthetic_source_term_parts_for_roots(
            ast,
            module.clone(),
            &binding_env,
            extracted.primary_roots.iter().copied(),
            extracted.context,
            &kinds,
            &recoveries,
        )?;
        #[cfg(not(test))]
        let parts = source_term_parts_for_roots(
            ast,
            module.clone(),
            &binding_env,
            extracted.primary_roots.iter().copied(),
            extracted.context,
            &kinds,
        )?;
        (parts.arena, parts.handoff, None, None)
    };
    let primary_id = |root: usize| {
        let range = ast.nodes().get(root)?.range;
        primary
            .terms()
            .iter()
            .find(|(_, term)| term.parent().is_none() && term.source_range() == range)
            .map(|(id, _)| id)
    };
    let mut requests = Vec::new();
    for (term_index, term_row) in extracted.terms.iter().enumerate() {
        let term = SourceSetTermId::new(term_index);
        let mut ordinal = 0;
        if term_row.kind == SourceSetTermKind::Comprehension {
            for (generator_index, generator) in extracted
                .generators
                .iter()
                .enumerate()
                .filter(|(_, generator)| generator.term == term)
            {
                requests.push(SourceSetRequestInput {
                    term,
                    ordinal,
                    kind: SourceSetRequestKind::GeneratorSethood,
                    generator: Some(SourceSetGeneratorId::new(generator_index)),
                    type_site: Some(generator.type_site),
                });
                ordinal += 1;
            }
        } else if term_row.kind == SourceSetTermKind::Choice {
            let type_site = type_site_for_term(&extracted, term)?;
            requests.push(SourceSetRequestInput {
                term,
                ordinal,
                kind: SourceSetRequestKind::ChoiceNonempty,
                generator: None,
                type_site: Some(type_site),
            });
            ordinal += 1;
        } else if term_row.kind == SourceSetTermKind::Qua {
            let type_site = type_site_for_term(&extracted, term)?;
            requests.push(SourceSetRequestInput {
                term,
                ordinal,
                kind: SourceSetRequestKind::QuaWidening,
                generator: None,
                type_site: Some(type_site),
            });
            ordinal += 1;
        }
        requests.push(SourceSetRequestInput {
            term,
            ordinal,
            kind: SourceSetRequestKind::ResultType,
            generator: None,
            type_site: None,
        });
    }

    let mut input = SourceSetTermHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms: extracted
            .terms
            .iter()
            .enumerate()
            .map(|(source_ordinal, term)| {
                let node = &ast.nodes()[term.node];
                SourceSetTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(term.node)),
                    source_range: node.range,
                    source_ordinal,
                    context: extracted.context,
                    recovery: term.recovery,
                    spelling: subtree_tokens(ast, node).join(" "),
                    kind: term.kind,
                }
            })
            .collect(),
        wrappers: extracted
            .wrappers
            .iter()
            .map(|wrapper| {
                let node = &ast.nodes()[wrapper.node];
                SourceSetWrapperInput {
                    term: wrapper.term,
                    ordinal: wrapper.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(wrapper.node)),
                    source_range: node.range,
                    context: extracted.context,
                    recovery: wrapper.recovery,
                    spelling: subtree_tokens(ast, node).join(" "),
                }
            })
            .collect(),
        generators: extracted
            .generators
            .iter()
            .map(|generator| {
                let node = &ast.nodes()[generator.node];
                SourceSetGeneratorInput {
                    term: generator.term,
                    ordinal: generator.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(generator.node)),
                    source_range: node.range,
                    spelling: node.token_text().unwrap_or_default().to_owned(),
                    context: extracted.context,
                    recovery: generator.recovery,
                    type_site: generator.type_site,
                }
            })
            .collect(),
        type_sites: extracted
            .type_sites
            .iter()
            .map(|type_site| {
                let node = &ast.nodes()[type_site.node];
                let head = &ast.nodes()[type_site.head_node];
                SourceSetTypeSiteInput {
                    owner: type_site.owner,
                    site: TypedSiteRef::Node(TypedNodeId::new(type_site.node)),
                    source_range: node.range,
                    spelling: subtree_tokens(ast, node).join(" "),
                    head_site: TypedSiteRef::Node(TypedNodeId::new(type_site.head_node)),
                    head_range: head.range,
                    head_spelling: subtree_tokens(ast, head).join(" "),
                    context: extracted.context,
                    recovery: type_site.recovery,
                    head: type_site.head,
                }
            })
            .collect(),
        conditions: extracted
            .conditions
            .iter()
            .map(|condition| {
                let colon = &ast.nodes()[condition.colon_node];
                let formula = &ast.nodes()[condition.condition_node];
                SourceSetConditionInput {
                    term: condition.term,
                    ordinal: condition.ordinal,
                    colon_site: TypedSiteRef::Node(TypedNodeId::new(condition.colon_node)),
                    colon_range: colon.range,
                    colon_spelling: colon.token_text().unwrap_or_default().to_owned(),
                    condition_site: TypedSiteRef::Node(TypedNodeId::new(condition.condition_node)),
                    source_range: formula.range,
                    spelling: subtree_tokens(ast, formula).join(" "),
                    recovery: condition.recovery,
                }
            })
            .collect(),
        edges: extracted
            .edges
            .iter()
            .map(|edge| {
                let target = match edge.target {
                    ExtractedTarget::Primary(root) => {
                        SourceSetTarget::Primary(primary_id(root).ok_or_else(|| {
                            "source-set primary child is not a Task-252 root".to_owned()
                        })?)
                    }
                    ExtractedTarget::Application(application) => {
                        SourceSetTarget::Application(application)
                    }
                    ExtractedTarget::Structure(structure) => SourceSetTarget::Structure(structure),
                    ExtractedTarget::SetTerm(term) => SourceSetTarget::SetTerm(term),
                };
                Ok(SourceSetEdgeInput {
                    term: edge.term,
                    ordinal: edge.ordinal,
                    role: edge.role,
                    target,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        requests,
    };
    mutate(&mut input);
    let handoff = SourceSetTermProducer::build(
        input,
        &binding_env,
        &primary,
        application.as_ref(),
        structure.as_ref(),
        &arena,
    )
    .map_err(|error| error.to_string())?;

    let mut typed_ast = TypedAst::try_new(TypedAstParts {
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
    .map_err(|error| error.to_string())?;
    if let Some(application) = application {
        typed_ast = typed_ast
            .with_source_application(application)
            .map_err(|error| error.to_string())?;
    }
    if let Some(structure) = structure {
        typed_ast = typed_ast
            .with_source_structure(structure)
            .map_err(|error| error.to_string())?;
    }
    let typed_ast = typed_ast
        .with_source_set_term(handoff)
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
    if typed_ast.source_set_term().is_none()
        || resolved.source_set_term() != typed_ast.source_set_term()
        || resolved.source_structure() != typed_ast.source_structure()
        || resolved.source_application() != typed_ast.source_application()
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
        return Err("source-set immutable final handoff mismatch".to_owned());
    }
    Ok(SourceSetTermRouteOutput {
        typed_ast,
        resolved,
        #[cfg(test)]
        binding_env,
    })
}

fn type_site_for_term(
    extracted: &ExtractedSetTerms,
    term: SourceSetTermId,
) -> Result<SourceSetTypeSiteId, String> {
    let matches = extracted
        .type_sites
        .iter()
        .enumerate()
        .filter(|(_, site)| matches!(site.owner, SourceSetTypeOwner::Term { term: owner, .. } if owner == term))
        .map(|(index, _)| SourceSetTypeSiteId::new(index))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [site] => Ok(*site),
        _ => Err("source-set term requires exactly one target type site".to_owned()),
    }
}

fn arena_with_overrides(
    ast: &SurfaceAst,
    source: &TypedArena,
    kinds: &BTreeMap<usize, &'static str>,
    recoveries: &BTreeMap<usize, NodeRecoveryState>,
) -> Result<TypedArena, String> {
    if source.len() != ast.nodes().len() {
        return Err("source-set dependency arena is not surface-indexed".to_owned());
    }
    let mut nodes = source
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for (index, kind) in kinds {
        let node = nodes
            .get_mut(*index)
            .ok_or_else(|| "source-set arena override site disappeared".to_owned())?;
        if node.kind.as_str() != "source.surface.unowned" {
            return Err("source-set arena override site is already owned".to_owned());
        }
        node.kind = (*kind).into();
    }
    for (index, recovery) in recoveries {
        nodes
            .get_mut(*index)
            .ok_or_else(|| "source-set recovery override site disappeared".to_owned())?
            .recovery = *recovery;
    }
    TypedArena::try_new(source.root(), nodes).map_err(|error| error.to_string())
}

fn insert_kind(
    kinds: &mut BTreeMap<usize, &'static str>,
    node: usize,
    kind: &'static str,
) -> Result<(), String> {
    if kinds.insert(node, kind).is_some() {
        return Err("source-set arena site is multiply owned".to_owned());
    }
    Ok(())
}

fn term_kind_key(kind: SourceSetTermKind) -> &'static str {
    match kind {
        SourceSetTermKind::Enumeration => "source.term.set.enumeration",
        SourceSetTermKind::Comprehension => "source.term.set.comprehension",
        SourceSetTermKind::Choice => "source.term.set.choice",
        SourceSetTermKind::Qua => "source.term.set.qua",
        _ => "source.term.set.unsupported",
    }
}

fn set_root_is_excluded(
    ast: &SurfaceAst,
    root: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    admit_conditions: bool,
) -> bool {
    let Some(root_node) = ast.nodes().get(root) else {
        return true;
    };
    if set_term_shape_is_excluded(ast, root, admit_conditions)
        || reverse_family_contains_set(ast, root, false)
        || contains_template_boundary(ast, root)
    {
        return true;
    }
    let parents = parent_indexes(ast);
    let mut cursor = parents[root];
    while let Some(parent) = cursor {
        let node = &ast.nodes()[parent];
        if is_template_kind(&node.kind)
            || is_reverse_application_kind(&node.kind)
            || is_reverse_structure_kind(&node.kind)
            || application_range_owned(applications, node.range)
            || structure_range_owned(structures, node.range)
        {
            return true;
        }
        cursor = parents[parent];
    }
    root_node.range.source_id != ast.source_id
}

fn set_term_shape_is_excluded(ast: &SurfaceAst, root: usize, admit_conditions: bool) -> bool {
    let Some(node) = ast.nodes().get(root) else {
        return true;
    };
    match node.kind {
        SurfaceNodeKind::SetEnumeration => structural_child_ids(ast, node)
            .into_iter()
            .any(|child| nested_set_shape_is_excluded(ast, child.index(), admit_conditions)),
        SurfaceNodeKind::SetComprehension => {
            let children = structural_child_ids(ast, node);
            let Some((mapper, tail)) = children.split_first() else {
                return true;
            };
            let has_colon = direct_token_texts(ast, node)
                .iter()
                .any(|token| token == ":");
            if has_colon && !admit_conditions {
                return true;
            }
            let generators = if has_colon {
                let Some((condition, generators)) = tail.split_last() else {
                    return true;
                };
                if ast
                    .node(*condition)
                    .is_none_or(|node| !matches!(node.kind, SurfaceNodeKind::FormulaExpression))
                {
                    return true;
                }
                generators
            } else {
                tail
            };
            if generators.is_empty() {
                return true;
            }
            let mut names = BTreeSet::new();
            for generator in generators {
                let Some(generator_node) = ast.node(*generator) else {
                    return true;
                };
                if !matches!(
                    generator_node.kind,
                    SurfaceNodeKind::ComprehensionVariableSegment
                ) {
                    return true;
                }
                let identifiers = generator_node
                    .children
                    .iter()
                    .filter_map(|child| ast.node(*child))
                    .filter_map(SurfaceNode::token_text)
                    .filter(|token| *token != "is")
                    .collect::<Vec<_>>();
                let [identifier] = identifiers.as_slice() else {
                    return true;
                };
                names.insert((*identifier).to_owned());
                let type_children = structural_child_ids(ast, generator_node);
                let [target_type] = type_children.as_slice() else {
                    return true;
                };
                if extract_bare_type_site(
                    ast,
                    target_type.index(),
                    SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0)),
                    SourceSetTermRecovery::Degraded,
                )
                .is_err()
                {
                    return true;
                }
            }
            let mapper_node = &ast.nodes()[mapper.index()];
            let mapper_tokens = subtree_tokens(ast, mapper_node);
            mapper_tokens.iter().any(|token| names.contains(*token))
                || nested_set_shape_is_excluded(ast, mapper.index(), admit_conditions)
        }
        SurfaceNodeKind::ChoiceTerm => structural_child_ids(ast, node)
            .as_slice()
            .first()
            .is_none_or(|target_type| {
                extract_bare_type_site(
                    ast,
                    target_type.index(),
                    SourceSetTypeOwner::Term {
                        term: SourceSetTermId::new(0),
                        role: SourceSetTypeRole::ChoiceTarget,
                    },
                    SourceSetTermRecovery::Degraded,
                )
                .is_err()
            }),
        SurfaceNodeKind::QuaExpression => {
            let children = structural_child_ids(ast, node);
            let [base, target_type] = children.as_slice() else {
                return true;
            };
            extract_bare_type_site(
                ast,
                target_type.index(),
                SourceSetTypeOwner::Term {
                    term: SourceSetTermId::new(0),
                    role: SourceSetTypeRole::QuaTarget,
                },
                SourceSetTermRecovery::Degraded,
            )
            .is_err()
                || nested_set_shape_is_excluded(ast, base.index(), admit_conditions)
        }
        _ => true,
    }
}

fn nested_set_shape_is_excluded(ast: &SurfaceAst, root: usize, admit_conditions: bool) -> bool {
    let Some(node) = ast.nodes().get(root) else {
        return true;
    };
    if is_set_term_kind(&node.kind) {
        return set_term_shape_is_excluded(ast, root, admit_conditions);
    }
    node.children
        .iter()
        .any(|child| nested_set_shape_is_excluded(ast, child.index(), admit_conditions))
}

fn reverse_family_contains_set(ast: &SurfaceAst, root: usize, inside_reverse: bool) -> bool {
    let Some(node) = ast.nodes().get(root) else {
        return true;
    };
    if inside_reverse && is_set_term_kind(&node.kind) {
        return true;
    }
    let next_reverse = inside_reverse
        || is_reverse_application_kind(&node.kind)
        || is_reverse_structure_kind(&node.kind);
    node.children
        .iter()
        .any(|child| reverse_family_contains_set(ast, child.index(), next_reverse))
}

fn contains_template_boundary(ast: &SurfaceAst, root: usize) -> bool {
    let Some(node) = ast.nodes().get(root) else {
        return true;
    };
    is_template_kind(&node.kind)
        || node
            .children
            .iter()
            .any(|child| contains_template_boundary(ast, child.index()))
}

fn peel_set_shells(ast: &SurfaceAst, start: usize) -> Result<(usize, Vec<usize>), String> {
    let mut current = start;
    let mut wrappers = Vec::new();
    loop {
        let node = ast
            .nodes()
            .get(current)
            .ok_or_else(|| "source-set shell disappeared".to_owned())?;
        match node.kind {
            SurfaceNodeKind::TermExpression => {
                if !direct_token_texts(ast, node).is_empty() {
                    break;
                }
                let children = structural_child_ids(ast, node);
                let [child] = children.as_slice() else {
                    break;
                };
                current = child.index();
            }
            SurfaceNodeKind::ParenthesizedTerm
                if direct_token_texts(ast, node).as_slice() == ["(", ")"]
                    && contains_set_term(ast, node) =>
            {
                wrappers.push(current);
                let children = structural_child_ids(ast, node);
                let [child] = children.as_slice() else {
                    return Err("source-set wrapper lost its child".to_owned());
                };
                current = child.index();
            }
            _ => break,
        }
    }
    Ok((current, wrappers))
}

fn contains_set_term(ast: &SurfaceAst, node: &SurfaceNode) -> bool {
    node.children.iter().any(|child| {
        ast.node(*child)
            .is_some_and(|child| is_set_term_kind(&child.kind) || contains_set_term(ast, child))
    })
}

fn application_target(
    applications: Option<&SourceFunctorApplicationHandoff>,
    range: SourceRange,
) -> Option<SourceFunctorApplicationId> {
    let applications = applications?;
    applications.applications().iter().find_map(|(id, row)| {
        let outer = applications
            .wrappers()
            .iter()
            .filter(|(_, wrapper)| wrapper.application() == id)
            .min_by_key(|(_, wrapper)| wrapper.ordinal())
            .map_or(row.source_range(), |(_, wrapper)| wrapper.source_range());
        (outer == range || row.source_range() == range).then_some(id)
    })
}

fn structure_target(
    structures: Option<&SourceStructureHandoff>,
    range: SourceRange,
) -> Option<SourceStructureTermId> {
    let structures = structures?;
    structures.terms().iter().find_map(|(id, row)| {
        let outer = structures
            .wrappers()
            .iter()
            .filter(|(_, wrapper)| wrapper.term() == id)
            .min_by_key(|(_, wrapper)| wrapper.ordinal())
            .map_or(row.source_range(), |(_, wrapper)| wrapper.source_range());
        (outer == range || row.source_range() == range).then_some(id)
    })
}

fn application_range_owned(
    applications: Option<&SourceFunctorApplicationHandoff>,
    range: SourceRange,
) -> bool {
    applications.is_some_and(|applications| {
        applications
            .applications()
            .iter()
            .any(|(_, row)| row.source_range() == range)
            || applications
                .wrappers()
                .iter()
                .any(|(_, row)| row.source_range() == range)
    })
}

fn structure_range_owned(structures: Option<&SourceStructureHandoff>, range: SourceRange) -> bool {
    structures.is_some_and(|structures| {
        structures
            .terms()
            .iter()
            .any(|(_, row)| row.source_range() == range)
            || structures
                .wrappers()
                .iter()
                .any(|(_, row)| row.source_range() == range)
    })
}

fn is_reverse_application_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::ApplicationTerm
            | SurfaceNodeKind::PrefixExpression(_)
            | SurfaceNodeKind::InfixExpression(_)
            | SurfaceNodeKind::PostfixExpression(_)
    )
}

fn is_reverse_structure_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::StructureConstructor
            | SurfaceNodeKind::SelectorAccess
            | SurfaceNodeKind::StructureUpdate
    )
}

fn is_template_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::TemplateParameter
            | SurfaceNodeKind::TemplateLoci
            | SurfaceNodeKind::TemplateLocus
            | SurfaceNodeKind::TemplateArguments
            | SurfaceNodeKind::TemplateArgument
    )
}

fn source_set_term_binding_env(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Result<BindingEnv, String> {
    let items = exact_compilation_item_list(ast)
        .ok_or_else(|| "Task255 compilation item list disappeared".to_owned())?;
    let item_ids = structural_child_ids(ast, items);
    let [reserve_id, definition_id] = item_ids.as_slice() else {
        return Err("Task255 requires one reserve and one definition block".to_owned());
    };
    let reserve = ast
        .node(*reserve_id)
        .ok_or_else(|| "Task255 reserve disappeared".to_owned())?;
    let definition = ast
        .node(*definition_id)
        .ok_or_else(|| "Task255 definition block disappeared".to_owned())?;
    let reserve_shell = unique_shell(shells, *reserve_id, DeclarationShellKind::Reserve)?;
    let definition_shell = unique_shell(
        shells,
        *definition_id,
        DeclarationShellKind::DefinitionBlock,
    )?;
    let reserve_payload =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .map_err(|()| "Task255 reserve extraction failed".to_owned())?;
    let [reserve_binding] = reserve_payload.bridge.bindings() else {
        return Err("Task255 requires one reserve binding".to_owned());
    };
    let reserve_children = structural_child_ids(ast, reserve);
    let [reserve_segment_id] = reserve_children.as_slice() else {
        return Err("Task255 reserve requires one segment".to_owned());
    };
    let parameter_id = structural_child_ids(ast, definition)
        .into_iter()
        .find(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::DefinitionParameter))
        })
        .ok_or_else(|| "Task255 definition parameter disappeared".to_owned())?;
    let parameter = ast
        .node(parameter_id)
        .ok_or_else(|| "Task255 definition parameter disappeared".to_owned())?;
    let parameter_children = structural_child_ids(ast, parameter);
    let [segment_id] = parameter_children.as_slice() else {
        return Err("Task255 definition parameter requires one segment".to_owned());
    };
    let segment = ast
        .node(*segment_id)
        .ok_or_else(|| "Task255 definition parameter segment disappeared".to_owned())?;
    let segment_children = structural_child_ids(ast, segment);
    let [type_id] = segment_children.as_slice() else {
        return Err("Task255 definition parameter requires one written type".to_owned());
    };
    let written_type = ast
        .node(*type_id)
        .ok_or_else(|| "Task255 definition parameter type disappeared".to_owned())?;
    if !matches!(segment.kind, SurfaceNodeKind::QualifiedVariableSegment)
        || direct_token_texts(ast, segment).as_slice() != ["seed", "be"]
        || !matches!(written_type.kind, SurfaceNodeKind::TypeExpression)
        || subtree_tokens(ast, written_type) != ["set"]
    {
        return Err("Task255 definition parameter shape drift".to_owned());
    }
    let declaration_range = unique_direct_token_range(ast, segment, "seed")?;
    let root_id = ast
        .root()
        .ok_or_else(|| "Task255 root disappeared".to_owned())?;
    let local_scope = LocalTermScope::new(vec![definition_shell.id().index() as u32]);
    let build = SourceBindingContextProducer::build(SourceBindingContextInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        module_site: surface_site(root_id),
        items: vec![
            SourceItemInput {
                shell: reserve_shell.id(),
                shell_ordinal: reserve_shell.ordinal(),
                role: SourceItemRole::Reserve,
                module_id: module.clone(),
                source_range: reserve.range,
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: surface_site(*reserve_id),
                local_scope: None,
                recovery: SourceItemRecovery::Normal,
            },
            SourceItemInput {
                shell: definition_shell.id(),
                shell_ordinal: definition_shell.ordinal(),
                role: SourceItemRole::DefinitionBlock,
                module_id: module.clone(),
                source_range: definition.range,
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: surface_site(*definition_id),
                local_scope: Some(local_scope.clone()),
                recovery: SourceItemRecovery::Normal,
            },
        ],
        bindings: vec![
            SourceBindingSiteInput {
                shell: reserve_shell.id(),
                context_owner: SourceBindingContextOwner::Module,
                source_ordinal: 0,
                spelling: "seed".to_owned(),
                declaration_range: reserve_binding.binding_range,
                written_type_range: reserve_binding.type_range,
                site: surface_site(*reserve_segment_id),
                role: SourceBindingSiteRole::ReserveDefault,
                recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
            },
            SourceBindingSiteInput {
                shell: definition_shell.id(),
                context_owner: SourceBindingContextOwner::Shell(definition_shell.id()),
                source_ordinal: 1,
                spelling: "seed".to_owned(),
                declaration_range,
                written_type_range: written_type.range,
                site: surface_site(parameter_id),
                role: SourceBindingSiteRole::DefinitionParameter {
                    local: LocalTermBinding::new("seed", local_scope, declaration_range, 1),
                },
                recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
            },
        ],
    })
    .map_err(|error| error.to_string())?;
    match build {
        SourceBindingContextBuild::Complete(projection) => {
            let handoff = projection.into_handoff();
            if handoff.binding_env().contexts().len() != 2
                || handoff.binding_env().bindings().len() != 2
            {
                return Err("Task255 binding projection cardinality mismatch".to_owned());
            }
            Ok(handoff.binding_env().clone())
        }
        SourceBindingContextBuild::Incomplete(_) => {
            Err("Task255 binding projection remained incomplete".to_owned())
        }
        _ => Err("Task255 binding projection returned an unsupported state".to_owned()),
    }
}

fn unique_shell(
    shells: &DeclarationShellSet,
    node: SurfaceNodeId,
    kind: DeclarationShellKind,
) -> Result<&DeclarationShell, String> {
    let matches = shells
        .declarations()
        .iter()
        .filter(|shell| shell.node_id() == node && shell.kind() == kind && !shell.recovered())
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [shell] => Ok(*shell),
        _ => Err(format!("Task255 requires one normal {kind:?} shell")),
    }
}

fn unique_direct_token_range(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    spelling: &str,
) -> Result<SourceRange, String> {
    let ranges = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter(|child| child.token_text() == Some(spelling))
        .map(|child| child.range)
        .collect::<Vec<_>>();
    let [range] = ranges.as_slice() else {
        return Err(format!("Task255 requires one direct `{spelling}` token"));
    };
    Ok(*range)
}

fn is_set_term_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::SetEnumeration
            | SurfaceNodeKind::SetComprehension
            | SurfaceNodeKind::ChoiceTerm
            | SurfaceNodeKind::QuaExpression
    )
}

fn parent_indexes(ast: &SurfaceAst) -> Vec<Option<usize>> {
    let mut parents = vec![None; ast.nodes().len()];
    for (parent, node) in ast.nodes().iter().enumerate() {
        for child in &node.children {
            parents[child.index()] = Some(parent);
        }
    }
    parents
}

fn has_set_term_ancestor(ast: &SurfaceAst, parents: &[Option<usize>], node: usize) -> bool {
    let mut cursor = parents[node];
    while let Some(parent) = cursor {
        if is_set_term_kind(&ast.nodes()[parent].kind) {
            return true;
        }
        cursor = parents[parent];
    }
    false
}

fn subtree_tokens<'a>(ast: &'a SurfaceAst, node: &'a SurfaceNode) -> Vec<&'a str> {
    let mut tokens = Vec::new();
    collect_subtree_tokens(ast, node, &mut tokens);
    tokens
}

fn collect_subtree_tokens<'a>(
    ast: &'a SurfaceAst,
    node: &'a SurfaceNode,
    tokens: &mut Vec<&'a str>,
) {
    if let Some(token) = node.token_text() {
        tokens.push(token);
        return;
    }
    for child in &node.children {
        if let Some(child) = ast.node(*child) {
            collect_subtree_tokens(ast, child, tokens);
        }
    }
}
