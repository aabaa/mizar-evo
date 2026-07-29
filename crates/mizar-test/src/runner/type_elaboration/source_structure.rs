use std::collections::{BTreeMap, BTreeSet};

use mizar_checker::{
    binding_env::{
        BinderIdentity, BindingContextId, BindingContextLayer, BindingContextOwner,
        BindingContextRecovery, BindingEnv, BindingKind, BindingRecoveryState, BindingStatus,
        BindingTypeSite,
    },
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_application::{SourceFunctorApplicationHandoff, SourceFunctorApplicationId},
    source_context::{
        SourceBindingContextBuild, SourceBindingContextInput, SourceBindingContextOwner,
        SourceBindingContextProducer, SourceBindingSiteInput, SourceBindingSiteRole,
        SourceItemInput, SourceItemRecovery, SourceItemRole, SourceItemVisibility,
    },
    source_structure::{
        SourceFieldUpdateInput, SourceStructureEdgeInput, SourceStructureEdgeRole,
        SourceStructureHandoff, SourceStructureHandoffInput, SourceStructureMemberId,
        SourceStructureMemberInput, SourceStructureMemberRole, SourceStructureProducer,
        SourceStructureRecovery, SourceStructureRequestInput, SourceStructureRequestKind,
        SourceStructureRootInput, SourceStructureTarget, SourceStructureTermId,
        SourceStructureTermInput, SourceStructureTermKind, SourceStructureWrapperInput,
    },
    source_term::{
        SourcePrimaryTermHandoff, SourcePrimaryTermKind, SourcePrimaryTermRecovery,
        SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
    },
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedAst, TypedAstParts,
        TypedNodeId, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShell, DeclarationShellKind, DeclarationShellSet},
    env::{ContributionKind, ExportStatus, NamespacePath, SymbolEnv, SymbolKind, Visibility},
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{ModuleId, SymbolId},
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

#[cfg(test)]
use mizar_checker::{
    binding_env::{
        BindingContextDraft, BindingContextTable, BindingDiagnosticClass, BindingDiagnosticDraft,
        BindingDiagnosticRecovery, BindingDiagnosticSeverity, BindingDiagnosticTable, BindingDraft,
        BindingEnvParts, BindingTable, CapturedFreeVariables,
    },
    source_term::{
        SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermId,
        SourcePrimaryTermInput, SourcePrimaryTermProducer, SourcePrimaryTermReferenceInput,
    },
};

use super::source_term::SourceTermParts;
#[cfg(not(test))]
use super::source_term::source_term_parts_for_roots;
#[cfg(test)]
use super::source_term::{
    source_term_parts_for_context_roots, synthetic_source_term_parts_for_roots,
};
use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_ast::{
        direct_token_texts, exact_compilation_item_list, qualified_symbol_spelling,
        structural_child_ids, subtree_has_recovery, surface_site,
    },
    source_reserve::extract_builtin_source_reserve_declarations_after_node_guard,
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";
const PAYLOAD_EXTRACTION_GAP_KEY: &str =
    "type_elaboration.external_dependency.ast_payload_extraction";

#[derive(Debug)]
pub(in crate::runner) struct SourceStructureRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
    #[cfg(test)]
    pub(in crate::runner) binding_env: BindingEnv,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SyntheticSourceStructureDependencies {
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: SourcePrimaryTermHandoff,
    pub(in crate::runner) application: Option<SourceFunctorApplicationHandoff>,
}

#[derive(Debug, Clone)]
struct ExtractedStructure {
    context: BindingContextId,
    terms: Vec<ExtractedTerm>,
    wrappers: Vec<ExtractedWrapper>,
    roots: Vec<ExtractedRoot>,
    members: Vec<ExtractedMember>,
    field_updates: Vec<ExtractedFieldUpdate>,
    edges: Vec<ExtractedEdge>,
    primary_roots: Vec<usize>,
}

#[derive(Debug, Clone)]
struct ExtractedTerm {
    node: usize,
    kind: SourceStructureTermKind,
    recovery: SourceStructureRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedWrapper {
    term: SourceStructureTermId,
    ordinal: usize,
    node: usize,
    recovery: SourceStructureRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedRoot {
    term: SourceStructureTermId,
    symbol: SymbolId,
    contribution: mizar_resolve::env::SourceContributionId,
}

#[derive(Debug, Clone)]
struct ExtractedMember {
    term: SourceStructureTermId,
    ordinal: usize,
    node: usize,
    role: SourceStructureMemberRole,
    parent: Option<SourceStructureMemberId>,
}

#[derive(Debug, Clone)]
struct ExtractedFieldUpdate {
    term: SourceStructureTermId,
    ordinal: usize,
    node: usize,
    first_member: SourceStructureMemberId,
    final_member: SourceStructureMemberId,
}

#[derive(Debug, Clone, Copy)]
enum ExtractedTarget {
    Primary(usize),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
}

#[derive(Debug, Clone)]
struct ExtractedEdge {
    term: SourceStructureTermId,
    ordinal: usize,
    role: SourceStructureEdgeRole,
    member: Option<SourceStructureMemberId>,
    target: ExtractedTarget,
}

const TASK258B3M2B2B2P_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureConstructorWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2);\n",
    "  thus x = x;\n",
    "end;\n",
);

fn task258b3m2b2b2p_surface_contract(ast: &SurfaceAst, loaded_source: &str) -> bool {
    task258b3m2b2b2p_surface_contract_impl(ast, loaded_source, |_, _, _, _, _| {})
}

fn task258b3m2b2b2p_surface_contract_impl(
    ast: &SurfaceAst,
    loaded_source: &str,
    mutate: impl FnOnce(
        &mut Vec<String>,
        &mut Vec<(usize, usize)>,
        &mut Vec<bool>,
        &mut Vec<Vec<usize>>,
        &mut Option<usize>,
    ),
) -> bool {
    const KINDS: [&str; 76] = [
        "Token(SurfaceToken { kind: ReservedWord, text: \"import\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"parser\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \".\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"type_fixtures\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"reserve\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"FormulaStatementStructureConstructorWitnessSmoke\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"proof\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"take\" })",
        "Token(SurfaceToken { kind: UserSymbol, text: \"TypeCaseStruct\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"1\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \",\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"y\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"2\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"thus\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"end\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "PathSegment",
        "PathSegment",
        "ModulePath",
        "ImportAliasDecl",
        "ImportItem",
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
        "PathSegment",
        "QualifiedSymbol",
        "NumeralTerm",
        "TermExpression",
        "FieldArgument",
        "NumeralTerm",
        "TermExpression",
        "FieldArgument",
        "StructureConstructor",
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
    const RANGES: [(usize, usize); 76] = [
        (0, 6),
        (7, 13),
        (13, 14),
        (14, 27),
        (27, 28),
        (29, 36),
        (37, 38),
        (39, 42),
        (43, 46),
        (46, 47),
        (48, 55),
        (56, 104),
        (104, 105),
        (106, 107),
        (108, 109),
        (110, 111),
        (112, 117),
        (120, 124),
        (125, 139),
        (139, 140),
        (140, 141),
        (141, 142),
        (143, 144),
        (144, 145),
        (146, 147),
        (147, 148),
        (149, 150),
        (150, 151),
        (151, 152),
        (155, 159),
        (160, 161),
        (162, 163),
        (164, 165),
        (165, 166),
        (167, 170),
        (170, 171),
        (7, 13),
        (14, 27),
        (7, 27),
        (7, 27),
        (0, 28),
        (43, 46),
        (43, 46),
        (37, 46),
        (29, 47),
        (106, 107),
        (106, 107),
        (110, 111),
        (110, 111),
        (106, 111),
        (106, 111),
        (125, 139),
        (125, 139),
        (143, 144),
        (143, 144),
        (140, 144),
        (149, 150),
        (149, 150),
        (146, 150),
        (125, 151),
        (125, 151),
        (125, 151),
        (120, 152),
        (160, 161),
        (160, 161),
        (164, 165),
        (164, 165),
        (160, 165),
        (160, 165),
        (160, 165),
        (155, 166),
        (112, 170),
        (48, 171),
        (0, 171),
        (0, 171),
        (0, 171),
    ];
    const CHILDREN: [&[usize]; 76] = [
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
        &[1],
        &[3],
        &[36, 2, 37],
        &[38],
        &[0, 39, 4],
        &[8],
        &[41],
        &[6, 7, 42],
        &[5, 43, 9],
        &[13],
        &[45],
        &[15],
        &[47],
        &[46, 14, 48],
        &[49],
        &[18],
        &[51],
        &[22],
        &[53],
        &[20, 21, 54],
        &[26],
        &[56],
        &[24, 25, 57],
        &[52, 19, 55, 23, 58, 27],
        &[59],
        &[60],
        &[17, 61, 28],
        &[30],
        &[63],
        &[32],
        &[65],
        &[64, 31, 66],
        &[67],
        &[68],
        &[29, 69, 33],
        &[16, 62, 70, 34],
        &[10, 11, 12, 50, 71, 35],
        &[40, 44, 72],
        &[73],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 74,
        ],
    ];

    if loaded_source != TASK258B3M2B2B2P_SOURCE || ast.nodes().len() != 76 {
        return false;
    }
    let mut kinds = ast
        .nodes()
        .iter()
        .map(|node| format!("{:?}", node.kind))
        .collect::<Vec<_>>();
    let mut ranges = ast
        .nodes()
        .iter()
        .map(|node| (node.range.start, node.range.end))
        .collect::<Vec<_>>();
    let mut recoveries = ast
        .nodes()
        .iter()
        .map(|node| node.recovered)
        .collect::<Vec<_>>();
    let mut children = ast
        .nodes()
        .iter()
        .map(|node| {
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut root = ast.root().map(|root| root.index());
    mutate(
        &mut kinds,
        &mut ranges,
        &mut recoveries,
        &mut children,
        &mut root,
    );
    root == Some(75)
        && kinds.iter().map(String::as_str).eq(KINDS.iter().copied())
        && ranges == RANGES
        && recoveries.iter().all(|recovered| !recovered)
        && children
            .iter()
            .map(Vec::as_slice)
            .eq(CHILDREN.iter().copied())
}

const TASK258B3M2B2B2BP_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureSelectorWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2).x;\n",
    "  thus x = x;\n",
    "end;\n",
);

#[allow(dead_code)] // Rationale: B2CP freezes a production-private seam whose sole production consumer is the separately scoped B2C task.
const TASK258B3M2B2B2CP_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureUpdateWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2) with (x := 3);\n",
    "  thus x = x;\n",
    "end;\n",
);

#[allow(dead_code)] // Rationale: B2BP freezes a production-private seam whose sole production consumer is the separately scoped B2B task.
fn task258b3m2b2b2bp_surface_contract(ast: &SurfaceAst, loaded_source: &str) -> bool {
    task258b3m2b2b2bp_surface_contract_impl(ast, loaded_source, |_, _, _, _, _| {})
}

fn task258b3m2b2b2bp_surface_contract_impl(
    ast: &SurfaceAst,
    loaded_source: &str,
    mutate: impl FnOnce(
        &mut Vec<String>,
        &mut Vec<(usize, usize)>,
        &mut Vec<bool>,
        &mut Vec<Vec<usize>>,
        &mut Option<usize>,
    ),
) -> bool {
    const KINDS: [&str; 79] = [
        "Token(SurfaceToken { kind: ReservedWord, text: \"import\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"parser\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \".\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"type_fixtures\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"reserve\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"FormulaStatementStructureSelectorWitnessSmoke\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"proof\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"take\" })",
        "Token(SurfaceToken { kind: UserSymbol, text: \"TypeCaseStruct\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"1\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \",\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"y\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"2\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \".\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"thus\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"end\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "PathSegment",
        "PathSegment",
        "ModulePath",
        "ImportAliasDecl",
        "ImportItem",
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
        "PathSegment",
        "QualifiedSymbol",
        "NumeralTerm",
        "TermExpression",
        "FieldArgument",
        "NumeralTerm",
        "TermExpression",
        "FieldArgument",
        "StructureConstructor",
        "SelectorAccess",
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
    const RANGES: [(usize, usize); 79] = [
        (0, 6),
        (7, 13),
        (13, 14),
        (14, 27),
        (27, 28),
        (29, 36),
        (37, 38),
        (39, 42),
        (43, 46),
        (46, 47),
        (48, 55),
        (56, 101),
        (101, 102),
        (103, 104),
        (105, 106),
        (107, 108),
        (109, 114),
        (117, 121),
        (122, 136),
        (136, 137),
        (137, 138),
        (138, 139),
        (140, 141),
        (141, 142),
        (143, 144),
        (144, 145),
        (146, 147),
        (147, 148),
        (148, 149),
        (149, 150),
        (150, 151),
        (154, 158),
        (159, 160),
        (161, 162),
        (163, 164),
        (164, 165),
        (166, 169),
        (169, 170),
        (7, 13),
        (14, 27),
        (7, 27),
        (7, 27),
        (0, 28),
        (43, 46),
        (43, 46),
        (37, 46),
        (29, 47),
        (103, 104),
        (103, 104),
        (107, 108),
        (107, 108),
        (103, 108),
        (103, 108),
        (122, 136),
        (122, 136),
        (140, 141),
        (140, 141),
        (137, 141),
        (146, 147),
        (146, 147),
        (143, 147),
        (122, 148),
        (122, 150),
        (122, 150),
        (122, 150),
        (117, 151),
        (159, 160),
        (159, 160),
        (163, 164),
        (163, 164),
        (159, 164),
        (159, 164),
        (159, 164),
        (154, 165),
        (109, 169),
        (48, 170),
        (0, 170),
        (0, 170),
        (0, 170),
    ];
    const CHILDREN: [&[usize]; 79] = [
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
        &[1],
        &[3],
        &[38, 2, 39],
        &[40],
        &[0, 41, 4],
        &[8],
        &[43],
        &[6, 7, 44],
        &[5, 45, 9],
        &[13],
        &[47],
        &[15],
        &[49],
        &[48, 14, 50],
        &[51],
        &[18],
        &[53],
        &[22],
        &[55],
        &[20, 21, 56],
        &[26],
        &[58],
        &[24, 25, 59],
        &[54, 19, 57, 23, 60, 27],
        &[61, 28, 29],
        &[62],
        &[63],
        &[17, 64, 30],
        &[32],
        &[66],
        &[34],
        &[68],
        &[67, 33, 69],
        &[70],
        &[71],
        &[31, 72, 35],
        &[16, 65, 73, 36],
        &[10, 11, 12, 52, 74, 37],
        &[42, 46, 75],
        &[76],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 77,
        ],
    ];

    if loaded_source != TASK258B3M2B2B2BP_SOURCE || ast.nodes().len() != 79 {
        return false;
    }
    let mut kinds = ast
        .nodes()
        .iter()
        .map(|node| format!("{:?}", node.kind))
        .collect::<Vec<_>>();
    let mut ranges = ast
        .nodes()
        .iter()
        .map(|node| (node.range.start, node.range.end))
        .collect::<Vec<_>>();
    let mut recoveries = ast
        .nodes()
        .iter()
        .map(|node| node.recovered)
        .collect::<Vec<_>>();
    let mut children = ast
        .nodes()
        .iter()
        .map(|node| {
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut root = ast.root().map(|root| root.index());
    mutate(
        &mut kinds,
        &mut ranges,
        &mut recoveries,
        &mut children,
        &mut root,
    );
    root == Some(78)
        && kinds.iter().map(String::as_str).eq(KINDS.iter().copied())
        && ranges == RANGES
        && recoveries.iter().all(|recovered| !recovered)
        && children
            .iter()
            .map(Vec::as_slice)
            .eq(CHILDREN.iter().copied())
}

#[allow(dead_code)] // Rationale: B2CP freezes a production-private seam whose sole production consumer is the separately scoped B2C task.
fn task258b3m2b2b2cp_surface_contract(ast: &SurfaceAst, loaded_source: &str) -> bool {
    task258b3m2b2b2cp_surface_contract_impl(ast, loaded_source, |_, _, _, _, _| {})
}

#[allow(dead_code)] // Rationale: exercised by B2CP tests before the separately scoped B2C production consumer exists.
fn task258b3m2b2b2cp_surface_contract_impl(
    ast: &SurfaceAst,
    loaded_source: &str,
    mutate: impl FnOnce(
        &mut Vec<String>,
        &mut Vec<(usize, usize)>,
        &mut Vec<bool>,
        &mut Vec<Vec<usize>>,
        &mut Option<usize>,
    ),
) -> bool {
    const KINDS: [&str; 86] = [
        "Token(SurfaceToken { kind: ReservedWord, text: \"import\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"parser\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \".\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"type_fixtures\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"reserve\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"FormulaStatementStructureUpdateWitnessSmoke\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"proof\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"take\" })",
        "Token(SurfaceToken { kind: UserSymbol, text: \"TypeCaseStruct\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"1\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \",\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"y\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"2\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"with\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":=\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"3\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"thus\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"end\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "PathSegment",
        "PathSegment",
        "ModulePath",
        "ImportAliasDecl",
        "ImportItem",
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
        "PathSegment",
        "QualifiedSymbol",
        "NumeralTerm",
        "TermExpression",
        "FieldArgument",
        "NumeralTerm",
        "TermExpression",
        "FieldArgument",
        "StructureConstructor",
        "NumeralTerm",
        "TermExpression",
        "FieldUpdate",
        "StructureUpdate",
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
    const RANGES: [(usize, usize); 86] = [
        (0, 6),
        (7, 13),
        (13, 14),
        (14, 27),
        (27, 28),
        (29, 36),
        (37, 38),
        (39, 42),
        (43, 46),
        (46, 47),
        (48, 55),
        (56, 99),
        (99, 100),
        (101, 102),
        (103, 104),
        (105, 106),
        (107, 112),
        (115, 119),
        (120, 134),
        (134, 135),
        (135, 136),
        (136, 137),
        (138, 139),
        (139, 140),
        (141, 142),
        (142, 143),
        (144, 145),
        (145, 146),
        (147, 151),
        (152, 153),
        (153, 154),
        (155, 157),
        (158, 159),
        (159, 160),
        (160, 161),
        (164, 168),
        (169, 170),
        (171, 172),
        (173, 174),
        (174, 175),
        (176, 179),
        (179, 180),
        (7, 13),
        (14, 27),
        (7, 27),
        (7, 27),
        (0, 28),
        (43, 46),
        (43, 46),
        (37, 46),
        (29, 47),
        (101, 102),
        (101, 102),
        (105, 106),
        (105, 106),
        (101, 106),
        (101, 106),
        (120, 134),
        (120, 134),
        (138, 139),
        (138, 139),
        (135, 139),
        (144, 145),
        (144, 145),
        (141, 145),
        (120, 146),
        (158, 159),
        (158, 159),
        (153, 159),
        (120, 160),
        (120, 160),
        (120, 160),
        (115, 161),
        (169, 170),
        (169, 170),
        (173, 174),
        (173, 174),
        (169, 174),
        (169, 174),
        (169, 174),
        (164, 175),
        (107, 179),
        (48, 180),
        (0, 180),
        (0, 180),
        (0, 180),
    ];
    const CHILDREN: [&[usize]; 86] = [
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
        &[1],
        &[3],
        &[42, 2, 43],
        &[44],
        &[0, 45, 4],
        &[8],
        &[47],
        &[6, 7, 48],
        &[5, 49, 9],
        &[13],
        &[51],
        &[15],
        &[53],
        &[52, 14, 54],
        &[55],
        &[18],
        &[57],
        &[22],
        &[59],
        &[20, 21, 60],
        &[26],
        &[62],
        &[24, 25, 63],
        &[58, 19, 61, 23, 64, 27],
        &[32],
        &[66],
        &[30, 31, 67],
        &[65, 28, 29, 68, 33],
        &[69],
        &[70],
        &[17, 71, 34],
        &[36],
        &[73],
        &[38],
        &[75],
        &[74, 37, 76],
        &[77],
        &[78],
        &[35, 79, 39],
        &[16, 72, 80, 40],
        &[10, 11, 12, 56, 81, 41],
        &[46, 50, 82],
        &[83],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 84,
        ],
    ];

    if loaded_source != TASK258B3M2B2B2CP_SOURCE || ast.nodes().len() != 86 {
        return false;
    }
    let mut kinds = ast
        .nodes()
        .iter()
        .map(|node| format!("{:?}", node.kind))
        .collect::<Vec<_>>();
    let mut ranges = ast
        .nodes()
        .iter()
        .map(|node| (node.range.start, node.range.end))
        .collect::<Vec<_>>();
    let mut recoveries = ast
        .nodes()
        .iter()
        .map(|node| node.recovered)
        .collect::<Vec<_>>();
    let mut children = ast
        .nodes()
        .iter()
        .map(|node| {
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut root = ast.root().map(|root| root.index());
    mutate(
        &mut kinds,
        &mut ranges,
        &mut recoveries,
        &mut children,
        &mut root,
    );
    root == Some(85)
        && kinds.iter().map(String::as_str).eq(KINDS.iter().copied())
        && ranges == RANGES
        && recoveries == [false; 86]
        && children
            .iter()
            .map(Vec::as_slice)
            .eq(CHILDREN.iter().copied())
}

/// Runs the bounded Task-254 transport only for its frozen real consumer.
pub(in crate::runner) fn source_structure_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    match source_structure_output(ast, module, shells, symbols) {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_structure().is_some()
                && output.typed_ast.source_structure() == output.resolved.source_structure()
                && output.typed_ast.source_term() == output.resolved.source_term() =>
        {
            Some(vec![PAYLOAD_EXTRACTION_GAP_KEY.to_owned()])
        }
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_structure_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<SourceStructureRouteOutput, String>> {
    source_structure_output_with_mutation(ast, module, shells, symbols, |_| {})
}

#[cfg(not(test))]
fn source_structure_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<SourceStructureRouteOutput, String>> {
    source_structure_output_with_mutation(ast, module, shells, symbols, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_structure_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Option<Result<SourceStructureRouteOutput, String>> {
    source_structure_output_with_mutation_impl(ast, module, shells, symbols, mutate)
}

#[cfg(not(test))]
fn source_structure_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Option<Result<SourceStructureRouteOutput, String>> {
    source_structure_output_with_mutation_impl(ast, module, shells, symbols, mutate)
}

fn source_structure_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Option<Result<SourceStructureRouteOutput, String>> {
    let roots = exact_real_roots(ast)?;
    let binding_env = match source_structure_binding_env(ast, module.clone(), shells, symbols) {
        Ok(binding_env) => binding_env,
        Err(error) => return Some(Err(error)),
    };
    let extracted = match extract_structure(
        ast,
        &module,
        symbols,
        BindingContextId::new(1),
        &roots,
        None,
        &BTreeSet::new(),
    ) {
        Ok(extracted) => extracted,
        Err(error) => return Some(Err(error)),
    };
    Some(build_output(
        ast,
        module,
        symbols,
        binding_env,
        extracted,
        None,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn synthetic_source_structure_output(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    symbols: &SymbolEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceStructureDependencies>,
    degraded_terms: &BTreeSet<usize>,
) -> Result<SourceStructureRouteOutput, String> {
    synthetic_source_structure_output_with_mutation(
        ast,
        module,
        binding_env,
        symbols,
        roots,
        dependencies,
        degraded_terms,
        |_| {},
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)] // Rationale: keep every synthetic dependency and corruption seam explicit.
pub(in crate::runner) fn synthetic_source_structure_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    symbols: &SymbolEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceStructureDependencies>,
    degraded_terms: &BTreeSet<usize>,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Result<SourceStructureRouteOutput, String> {
    let application = dependencies
        .as_ref()
        .and_then(|dependencies| dependencies.application.as_ref());
    let extracted = extract_structure(
        ast,
        &module,
        symbols,
        BindingContextId::new(0),
        roots,
        application,
        degraded_terms,
    )?;
    build_output(
        ast,
        module,
        symbols,
        binding_env,
        extracted,
        dependencies,
        mutate,
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)] // Rationale: keep extraction and producer-validation provenance authorities explicit.
pub(in crate::runner) fn synthetic_source_structure_output_with_validation_symbols(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    extraction_symbols: &SymbolEnv,
    validation_symbols: &SymbolEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceStructureDependencies>,
    degraded_terms: &BTreeSet<usize>,
) -> Result<SourceStructureRouteOutput, String> {
    let application = dependencies
        .as_ref()
        .and_then(|dependencies| dependencies.application.as_ref());
    let extracted = extract_structure(
        ast,
        &module,
        extraction_symbols,
        BindingContextId::new(0),
        roots,
        application,
        degraded_terms,
    )?;
    build_output(
        ast,
        module,
        validation_symbols,
        binding_env,
        extracted,
        dependencies,
        |_| {},
    )
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
        "struct",
        "Task254Pair",
        "where",
        "field",
        "carrier",
        "->",
        "set",
        ";",
        "field",
        "marker",
        "->",
        "set",
        ";",
        "end",
        ";",
        "func",
        "Task254ConstructorDef",
        ":",
        "task254_constructor",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "Task254Pair",
        "(",
        "carrier",
        ":",
        "1",
        ",",
        "marker",
        ":",
        "2",
        ")",
        ";",
        "func",
        "Task254SelectorDef",
        ":",
        "task254_selector",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "Task254Pair",
        "(",
        "carrier",
        ":",
        "3",
        ",",
        "marker",
        ":",
        "4",
        ")",
        ".",
        "carrier",
        ";",
        "func",
        "Task254UpdateDef",
        ":",
        "task254_update",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "Task254Pair",
        "(",
        "carrier",
        ":",
        "5",
        ",",
        "marker",
        ":",
        "6",
        ")",
        "with",
        "(",
        "carrier",
        ":=",
        "7",
        ",",
        "marker",
        ":=",
        "8",
        ")",
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
        .filter(|(_, node)| is_structure_kind(&node.kind))
        .filter(|(index, _)| !has_structure_ancestor(ast, &parents, *index))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if roots.len() == 3
        && ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::StructureConstructor))
            .count()
            == 3
        && ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::SelectorAccess))
            .count()
            == 1
        && ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::StructureUpdate))
            .count()
            == 1
    {
        Some(roots)
    } else {
        None
    }
}

fn extract_structure(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    context: BindingContextId,
    roots: &[usize],
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
) -> Result<ExtractedStructure, String> {
    if symbols.module_id() != module {
        return Err("source-structure resolver module identity mismatch".to_owned());
    }
    let mut extracted = ExtractedStructure {
        context,
        terms: Vec::new(),
        wrappers: Vec::new(),
        roots: Vec::new(),
        members: Vec::new(),
        field_updates: Vec::new(),
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
        if structure_root_is_excluded(ast, root, application) {
            continue;
        }
        collect_target(
            ast,
            module,
            symbols,
            root,
            application,
            degraded_terms,
            &mut extracted,
        )?
        .structure()
        .ok_or_else(|| "synthetic source-structure root is not a structure term".to_owned())?;
    }
    normalize_extracted_tables(&mut extracted);
    Ok(extracted)
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ImportedStructureConstructorSite {
    pub(super) constructor: usize,
}

pub(super) fn imported_structure_constructor_owned_node_kinds(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    loaded_source: &str,
    site: ImportedStructureConstructorSite,
) -> Option<BTreeMap<usize, &'static str>> {
    let extracted =
        extract_task258b3m2b2b2p_constructor(ast, module, symbols, loaded_source, site)?;
    Some(BTreeMap::from([
        (extracted.terms[0].node, "source.term.structure.constructor"),
        (
            extracted.members[0].node,
            "source.term.structure.member.constructor-assignment",
        ),
        (
            extracted.members[1].node,
            "source.term.structure.member.constructor-assignment",
        ),
    ]))
}

#[allow(clippy::too_many_arguments)] // Rationale: keep every authenticated lower-stage authority explicit at the private reuse seam.
pub(super) fn imported_structure_constructor_handoff_in_context(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
    loaded_source: &str,
    site: ImportedStructureConstructorSite,
    context: BindingContextId,
) -> Option<Result<SourceStructureHandoff, String>> {
    let mut extracted =
        extract_task258b3m2b2b2p_constructor(ast, module, symbols, loaded_source, site)?;
    if !task258b3m2b2b2p_lower_profile_is_exact(ast, module, binding_env, source_term, context) {
        return Some(Err(
            "exact Task-48/252 proof-context profile mismatch".to_owned()
        ));
    }
    extracted.context = context;
    Some(build_handoff_with_source_term(
        ast,
        module,
        symbols,
        binding_env,
        &extracted,
        source_term,
        None,
        |_| {},
    ))
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ImportedStructureSelectorSite {
    pub(super) selector: usize,
}

pub(super) fn imported_structure_selector_owned_node_kinds(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    loaded_source: &str,
    site: ImportedStructureSelectorSite,
) -> Option<BTreeMap<usize, &'static str>> {
    let extracted = extract_task258b3m2b2b2bp_selector(ast, module, symbols, loaded_source, site)?;
    Some(BTreeMap::from([
        (extracted.terms[0].node, "source.term.structure.selector"),
        (extracted.terms[1].node, "source.term.structure.constructor"),
        (
            extracted.members[0].node,
            "source.term.structure.member.selector",
        ),
        (
            extracted.members[1].node,
            "source.term.structure.member.constructor-assignment",
        ),
        (
            extracted.members[2].node,
            "source.term.structure.member.constructor-assignment",
        ),
    ]))
}

#[allow(clippy::too_many_arguments)] // Rationale: keep every authenticated lower-stage authority explicit at the private reuse seam.
pub(super) fn imported_structure_selector_handoff_in_context(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
    loaded_source: &str,
    site: ImportedStructureSelectorSite,
    context: BindingContextId,
) -> Option<Result<SourceStructureHandoff, String>> {
    let mut extracted =
        extract_task258b3m2b2b2bp_selector(ast, module, symbols, loaded_source, site)?;
    if !task258b3m2b2b2bp_lower_profile_is_exact(ast, module, binding_env, source_term, context) {
        return Some(Err(
            "exact Task-48/252 selector proof-context profile mismatch".to_owned(),
        ));
    }
    extracted.context = context;
    Some(build_handoff_with_source_term(
        ast,
        module,
        symbols,
        binding_env,
        &extracted,
        source_term,
        None,
        |_| {},
    ))
}

#[allow(dead_code)]
// Rationale: B2CP freezes a production-private seam whose sole production consumer is the separately scoped B2C task.
#[derive(Debug, Clone, Copy)]
pub(super) struct ImportedStructureUpdateSite {
    pub(super) update: usize,
}

#[allow(dead_code)] // Rationale: B2CP freezes a production-private seam whose sole production consumer is the separately scoped B2C task.
pub(super) fn imported_structure_update_owned_node_kinds(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    loaded_source: &str,
    site: ImportedStructureUpdateSite,
) -> Option<BTreeMap<usize, &'static str>> {
    let extracted = extract_task258b3m2b2b2cp_update(ast, module, symbols, loaded_source, site)?;
    Some(BTreeMap::from([
        (extracted.terms[0].node, "source.term.structure.update"),
        (extracted.terms[1].node, "source.term.structure.constructor"),
        (
            extracted.members[0].node,
            "source.term.structure.member.update-path-segment",
        ),
        (
            extracted.members[1].node,
            "source.term.structure.member.constructor-assignment",
        ),
        (
            extracted.members[2].node,
            "source.term.structure.member.constructor-assignment",
        ),
        (
            extracted.field_updates[0].node,
            "source.term.structure.field-update",
        ),
    ]))
}

#[allow(clippy::too_many_arguments)] // Rationale: keep every authenticated lower-stage authority explicit at the private reuse seam.
#[allow(dead_code)] // Rationale: B2CP freezes a production-private seam whose sole production consumer is the separately scoped B2C task.
pub(super) fn imported_structure_update_handoff_in_context(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
    loaded_source: &str,
    site: ImportedStructureUpdateSite,
    context: BindingContextId,
) -> Option<Result<SourceStructureHandoff, String>> {
    let mut extracted =
        extract_task258b3m2b2b2cp_update(ast, module, symbols, loaded_source, site)?;
    if !task258b3m2b2b2cp_lower_profile_is_exact(ast, module, binding_env, source_term, context) {
        return Some(Err(
            "exact Task-48/252 update proof-context profile mismatch".to_owned(),
        ));
    }
    extracted.context = context;
    Some(build_handoff_with_source_term(
        ast,
        module,
        symbols,
        binding_env,
        &extracted,
        source_term,
        None,
        |_| {},
    ))
}

fn task258b3m2b2b2p_lower_profile_is_exact(
    ast: &SurfaceAst,
    module: &ModuleId,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
    context: BindingContextId,
) -> bool {
    if context != BindingContextId::new(1)
        || binding_env.source_id() != ast.source_id
        || binding_env.module_id() != module
        || source_term.handoff.source_id() != ast.source_id
        || source_term.handoff.module_id() != module
        || binding_env.contexts().len() != 2
        || binding_env.bindings().len() != 1
        || !binding_env.diagnostics().is_empty()
    {
        return false;
    }

    let Some(module_context) = binding_env.contexts().get(BindingContextId::new(0)) else {
        return false;
    };
    let Some(proof_context) = binding_env.contexts().get(BindingContextId::new(1)) else {
        return false;
    };
    if !matches!(module_context.owner, BindingContextOwner::Module)
        || module_context.parent.is_some()
        || module_context.layer != BindingContextLayer::Module
        || module_context.lexical_scope.is_some()
        || module_context
            .bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>()
            != [0]
        || module_context
            .visible_bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>()
            != [0]
        || module_context.recovery != BindingContextRecovery::Normal
        || !matches!(
            proof_context.owner,
            BindingContextOwner::SourceStatement { source_range }
                if source_range == task258b3m2b2b2p_range(ast, 112, 170)
        )
        || proof_context.parent != Some(BindingContextId::new(0))
        || proof_context.layer != BindingContextLayer::Proof
        || proof_context
            .lexical_scope
            .as_ref()
            .is_none_or(|scope| scope.path() != [0])
        || !proof_context.bindings.is_empty()
        || proof_context
            .visible_bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>()
            != [0]
        || proof_context.recovery != BindingContextRecovery::Normal
    {
        return false;
    }

    let Some((binding_id, binding)) = binding_env.bindings().iter().next() else {
        return false;
    };
    if binding_id.index() != 0
        || binding.id != binding_id
        || binding.spelling != "x"
        || binding.kind != BindingKind::ReservedVariable
        || !matches!(
            &binding.identity,
            BinderIdentity::ReservedVariable {
                spelling,
                declaration_range,
            } if spelling == "x"
                && *declaration_range == task258b3m2b2b2p_range(ast, 37, 38)
        )
        || binding.owner_context != BindingContextId::new(0)
        || binding.declaration_range != task258b3m2b2b2p_range(ast, 37, 38)
        || binding.visible_after_ordinal != 0
        || !matches!(
            binding.type_site,
            BindingTypeSite::Source(source_range)
                if source_range == task258b3m2b2b2p_range(ast, 43, 46)
        )
        || binding.status != BindingStatus::Reserved
        || !binding.captured.identities().is_empty()
        || !binding.diagnostics.is_empty()
        || binding.recovery != BindingRecoveryState::Normal
    {
        return false;
    }

    let expected_terms = [
        (
            45,
            106,
            107,
            BindingContextId::new(0),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            47,
            110,
            111,
            BindingContextId::new(0),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            53,
            143,
            144,
            BindingContextId::new(1),
            "1",
            SourcePrimaryTermKind::Numeral,
        ),
        (
            56,
            149,
            150,
            BindingContextId::new(1),
            "2",
            SourcePrimaryTermKind::Numeral,
        ),
        (
            63,
            160,
            161,
            BindingContextId::new(1),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            65,
            164,
            165,
            BindingContextId::new(1),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
    ];
    if source_term.handoff.terms().len() != expected_terms.len()
        || source_term.handoff.terms().iter().zip(expected_terms).any(
            |((id, term), (site, start, end, expected_context, spelling, kind))| {
                id.index() != term.source_ordinal()
                    || term.site().node().index() != site
                    || term.source_range() != task258b3m2b2b2p_range(ast, start, end)
                    || term.context() != expected_context
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
    if source_term.handoff.references().len() != expected_references.len()
        || source_term
            .handoff
            .references()
            .iter()
            .zip(expected_references)
            .any(|((id, reference), (term, scoped))| {
                id.index() >= expected_references.len()
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

    let expected_requests = [(2, 53, 143, 144, "1"), (3, 56, 149, 150, "2")];
    if source_term.handoff.numeric_type_requests().len() != expected_requests.len()
        || source_term
            .handoff
            .numeric_type_requests()
            .iter()
            .zip(expected_requests)
            .any(|((id, request), (term, owner, start, end, spelling))| {
                request.term().index() != term
                    || request.owner().node().index() != owner
                    || request.source_range() != task258b3m2b2b2p_range(ast, start, end)
                    || request.spelling() != spelling
                    || request.request_ordinal() != id.index()
            })
    {
        return false;
    }

    if source_term.arena.len() != ast.nodes().len()
        || source_term.arena.root().map(|root| root.index()) != ast.root().map(|root| root.index())
    {
        return false;
    }
    source_term.arena.iter().all(|(id, node)| {
        let index = id.index();
        let Some(surface) = ast.nodes().get(index) else {
            return false;
        };
        let expected_kind = match index {
            20 | 24 => "source.term.structure.member.constructor-assignment",
            45 | 47 | 63 | 65 => "source.term.variable-reference",
            53 | 56 => "source.term.numeral",
            59 => "source.term.structure.constructor",
            _ => "source.surface.unowned",
        };
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
}

fn task258b3m2b2b2p_range(ast: &SurfaceAst, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id: ast.source_id,
        start,
        end,
    }
}

fn extract_task258b3m2b2b2p_constructor(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    loaded_source: &str,
    site: ImportedStructureConstructorSite,
) -> Option<ExtractedStructure> {
    if site.constructor != 59
        || !task258b3m2b2b2p_surface_contract(ast, loaded_source)
        || symbols.module_id() != module
    {
        return None;
    }
    let extracted = extract_structure(
        ast,
        module,
        symbols,
        BindingContextId::new(0),
        &[site.constructor],
        None,
        &BTreeSet::new(),
    )
    .ok()?;
    let [term] = extracted.terms.as_slice() else {
        return None;
    };
    let [root] = extracted.roots.as_slice() else {
        return None;
    };
    let [left, right] = extracted.members.as_slice() else {
        return None;
    };
    let [left_edge, right_edge] = extracted.edges.as_slice() else {
        return None;
    };
    if term.node != 59
        || term.kind != SourceStructureTermKind::Constructor
        || term.recovery != SourceStructureRecovery::Normal
        || !extracted.wrappers.is_empty()
        || root.term != SourceStructureTermId::new(0)
        || left.term != SourceStructureTermId::new(0)
        || left.ordinal != 0
        || left.node != 20
        || left.role != SourceStructureMemberRole::ConstructorAssignment
        || left.parent.is_some()
        || right.term != SourceStructureTermId::new(0)
        || right.ordinal != 1
        || right.node != 24
        || right.role != SourceStructureMemberRole::ConstructorAssignment
        || right.parent.is_some()
        || !extracted.field_updates.is_empty()
        || left_edge.term != SourceStructureTermId::new(0)
        || left_edge.ordinal != 0
        || left_edge.role != SourceStructureEdgeRole::ConstructorValue
        || left_edge.member != Some(SourceStructureMemberId::new(0))
        || !matches!(left_edge.target, ExtractedTarget::Primary(54))
        || right_edge.term != SourceStructureTermId::new(0)
        || right_edge.ordinal != 1
        || right_edge.role != SourceStructureEdgeRole::ConstructorValue
        || right_edge.member != Some(SourceStructureMemberId::new(1))
        || !matches!(right_edge.target, ExtractedTarget::Primary(57))
        || extracted.primary_roots != [54, 57]
        || !task258b3m2b2b2p_root_is_exact(ast, module, symbols, &root.symbol)
    {
        return None;
    }
    Some(extracted)
}

fn task258b3m2b2b2p_root_is_exact(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    symbol: &SymbolId,
) -> bool {
    let Some(entry) = symbols.symbols().get(symbol) else {
        return false;
    };
    let Some(contribution) = symbols.contributions().get(entry.contribution()) else {
        return false;
    };
    let import_range = SourceRange {
        source_id: ast.source_id,
        start: 7,
        end: 27,
    };
    symbol.module().package().as_str() == "mizar-test-task253-corruption"
        && symbol.module().package() == module.package()
        && symbol.module().path().as_str() == "parser.type_fixtures"
        && symbol.local().as_str() == "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5"
        && symbol.fqn().as_str() == "parser.type_fixtures::TypeCaseStruct#5"
        && entry.kind() == SymbolKind::Structure
        && entry.namespace() == &NamespacePath::new(module.path().as_str())
        && entry.primary_spelling() == "TypeCaseStruct"
        && entry.visibility() == Visibility::Public
        && entry.export_status() == ExportStatus::Exported
        && entry.signature().is_none()
        && entry.contribution().index() == 2
        && entry.origin().source_id() == ast.source_id
        && entry.origin().module_id() == symbol.module()
        && entry.origin().anchor() == &SourceAnchor::Range(import_range)
        && entry.origin().structural_path() == [5]
        && entry.origin().import_edge().is_none()
        && !entry.origin().is_recovered()
        && contribution.id() == entry.contribution()
        && contribution.module() == symbol.module()
        && matches!(
            contribution.kind(),
            ContributionKind::ImportedSource { source_id } if *source_id == ast.source_id
        )
        && contribution.anchor() == &SourceAnchor::Range(import_range)
}

fn task258b3m2b2b2bp_lower_profile_is_exact(
    ast: &SurfaceAst,
    module: &ModuleId,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
    context: BindingContextId,
) -> bool {
    if context != BindingContextId::new(1)
        || binding_env.source_id() != ast.source_id
        || binding_env.module_id() != module
        || source_term.handoff.source_id() != ast.source_id
        || source_term.handoff.module_id() != module
        || binding_env.contexts().len() != 2
        || binding_env.bindings().len() != 1
        || !binding_env.diagnostics().is_empty()
    {
        return false;
    }

    let Some(module_context) = binding_env.contexts().get(BindingContextId::new(0)) else {
        return false;
    };
    let Some(proof_context) = binding_env.contexts().get(BindingContextId::new(1)) else {
        return false;
    };
    if !matches!(module_context.owner, BindingContextOwner::Module)
        || module_context.parent.is_some()
        || module_context.layer != BindingContextLayer::Module
        || module_context.lexical_scope.is_some()
        || module_context
            .bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>()
            != [0]
        || module_context
            .visible_bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>()
            != [0]
        || module_context.recovery != BindingContextRecovery::Normal
        || !matches!(
            proof_context.owner,
            BindingContextOwner::SourceStatement { source_range }
                if source_range == task258b3m2b2b2bp_range(ast, 109, 169)
        )
        || proof_context.parent != Some(BindingContextId::new(0))
        || proof_context.layer != BindingContextLayer::Proof
        || proof_context
            .lexical_scope
            .as_ref()
            .is_none_or(|scope| scope.path() != [0])
        || !proof_context.bindings.is_empty()
        || proof_context
            .visible_bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>()
            != [0]
        || proof_context.recovery != BindingContextRecovery::Normal
    {
        return false;
    }

    let Some((binding_id, binding)) = binding_env.bindings().iter().next() else {
        return false;
    };
    if binding_id.index() != 0
        || binding.id != binding_id
        || binding.spelling != "x"
        || binding.kind != BindingKind::ReservedVariable
        || !matches!(
            &binding.identity,
            BinderIdentity::ReservedVariable {
                spelling,
                declaration_range,
            } if spelling == "x"
                && *declaration_range == task258b3m2b2b2bp_range(ast, 37, 38)
        )
        || binding.owner_context != BindingContextId::new(0)
        || binding.declaration_range != task258b3m2b2b2bp_range(ast, 37, 38)
        || binding.visible_after_ordinal != 0
        || !matches!(
            binding.type_site,
            BindingTypeSite::Source(source_range)
                if source_range == task258b3m2b2b2bp_range(ast, 43, 46)
        )
        || binding.status != BindingStatus::Reserved
        || !binding.captured.identities().is_empty()
        || !binding.diagnostics.is_empty()
        || binding.recovery != BindingRecoveryState::Normal
    {
        return false;
    }

    let expected_terms = [
        (
            47,
            103,
            104,
            BindingContextId::new(0),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            49,
            107,
            108,
            BindingContextId::new(0),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            55,
            140,
            141,
            BindingContextId::new(1),
            "1",
            SourcePrimaryTermKind::Numeral,
        ),
        (
            58,
            146,
            147,
            BindingContextId::new(1),
            "2",
            SourcePrimaryTermKind::Numeral,
        ),
        (
            66,
            159,
            160,
            BindingContextId::new(1),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            68,
            163,
            164,
            BindingContextId::new(1),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
    ];
    if source_term.handoff.terms().len() != expected_terms.len()
        || source_term.handoff.terms().iter().zip(expected_terms).any(
            |((id, term), (site, start, end, expected_context, spelling, kind))| {
                id.index() != term.source_ordinal()
                    || term.site().node().index() != site
                    || term.source_range() != task258b3m2b2b2bp_range(ast, start, end)
                    || term.context() != expected_context
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
    if source_term.handoff.references().len() != expected_references.len()
        || source_term
            .handoff
            .references()
            .iter()
            .zip(expected_references)
            .any(|((id, reference), (term, scoped))| {
                id.index() >= expected_references.len()
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

    let expected_requests = [(2, 55, 140, 141, "1"), (3, 58, 146, 147, "2")];
    if source_term.handoff.numeric_type_requests().len() != expected_requests.len()
        || source_term
            .handoff
            .numeric_type_requests()
            .iter()
            .zip(expected_requests)
            .any(|((id, request), (term, owner, start, end, spelling))| {
                request.term().index() != term
                    || request.owner().node().index() != owner
                    || request.source_range() != task258b3m2b2b2bp_range(ast, start, end)
                    || request.spelling() != spelling
                    || request.request_ordinal() != id.index()
            })
    {
        return false;
    }

    if source_term.arena.len() != ast.nodes().len()
        || source_term.arena.root().map(|root| root.index()) != ast.root().map(|root| root.index())
    {
        return false;
    }
    source_term.arena.iter().all(|(id, node)| {
        let index = id.index();
        let Some(surface) = ast.nodes().get(index) else {
            return false;
        };
        let expected_kind = match index {
            20 | 24 => "source.term.structure.member.constructor-assignment",
            29 => "source.term.structure.member.selector",
            47 | 49 | 66 | 68 => "source.term.variable-reference",
            55 | 58 => "source.term.numeral",
            61 => "source.term.structure.constructor",
            62 => "source.term.structure.selector",
            _ => "source.surface.unowned",
        };
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
}

#[allow(dead_code)] // Rationale: B2CP freezes a production-private seam whose sole production consumer is the separately scoped B2C task.
fn task258b3m2b2b2cp_lower_profile_is_exact(
    ast: &SurfaceAst,
    module: &ModuleId,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
    context: BindingContextId,
) -> bool {
    if context != BindingContextId::new(1)
        || binding_env.source_id() != ast.source_id
        || binding_env.module_id() != module
        || source_term.handoff.source_id() != ast.source_id
        || source_term.handoff.module_id() != module
        || binding_env.contexts().len() != 2
        || binding_env.bindings().len() != 1
        || !binding_env.diagnostics().is_empty()
    {
        return false;
    }

    let Some(module_context) = binding_env.contexts().get(BindingContextId::new(0)) else {
        return false;
    };
    let Some(proof_context) = binding_env.contexts().get(BindingContextId::new(1)) else {
        return false;
    };
    if !matches!(module_context.owner, BindingContextOwner::Module)
        || module_context.parent.is_some()
        || module_context.layer != BindingContextLayer::Module
        || module_context.lexical_scope.is_some()
        || module_context
            .bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>()
            != [0]
        || module_context
            .visible_bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>()
            != [0]
        || module_context.recovery != BindingContextRecovery::Normal
        || !matches!(
            proof_context.owner,
            BindingContextOwner::SourceStatement { source_range }
                if source_range == task258b3m2b2b2cp_range(ast, 107, 179)
        )
        || proof_context.parent != Some(BindingContextId::new(0))
        || proof_context.layer != BindingContextLayer::Proof
        || proof_context
            .lexical_scope
            .as_ref()
            .is_none_or(|scope| scope.path() != [0])
        || !proof_context.bindings.is_empty()
        || proof_context
            .visible_bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>()
            != [0]
        || proof_context.recovery != BindingContextRecovery::Normal
    {
        return false;
    }

    let Some((binding_id, binding)) = binding_env.bindings().iter().next() else {
        return false;
    };
    if binding_id.index() != 0
        || binding.id != binding_id
        || binding.spelling != "x"
        || binding.kind != BindingKind::ReservedVariable
        || !matches!(
            &binding.identity,
            BinderIdentity::ReservedVariable {
                spelling,
                declaration_range,
            } if spelling == "x"
                && *declaration_range == task258b3m2b2b2cp_range(ast, 37, 38)
        )
        || binding.owner_context != BindingContextId::new(0)
        || binding.declaration_range != task258b3m2b2b2cp_range(ast, 37, 38)
        || binding.visible_after_ordinal != 0
        || !matches!(
            binding.type_site,
            BindingTypeSite::Source(source_range)
                if source_range == task258b3m2b2b2cp_range(ast, 43, 46)
        )
        || binding.status != BindingStatus::Reserved
        || !binding.captured.identities().is_empty()
        || !binding.diagnostics.is_empty()
        || binding.recovery != BindingRecoveryState::Normal
    {
        return false;
    }

    let expected_terms = [
        (
            51,
            101,
            102,
            BindingContextId::new(0),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            53,
            105,
            106,
            BindingContextId::new(0),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            59,
            138,
            139,
            BindingContextId::new(1),
            "1",
            SourcePrimaryTermKind::Numeral,
        ),
        (
            62,
            144,
            145,
            BindingContextId::new(1),
            "2",
            SourcePrimaryTermKind::Numeral,
        ),
        (
            66,
            158,
            159,
            BindingContextId::new(1),
            "3",
            SourcePrimaryTermKind::Numeral,
        ),
        (
            73,
            169,
            170,
            BindingContextId::new(1),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
        (
            75,
            173,
            174,
            BindingContextId::new(1),
            "x",
            SourcePrimaryTermKind::VariableReference,
        ),
    ];
    if source_term.handoff.terms().len() != expected_terms.len()
        || source_term.handoff.terms().iter().zip(expected_terms).any(
            |((id, term), (site, start, end, expected_context, spelling, kind))| {
                id.index() != term.source_ordinal()
                    || term.site().node().index() != site
                    || term.source_range() != task258b3m2b2b2cp_range(ast, start, end)
                    || term.context() != expected_context
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

    let expected_references = [(0, false), (1, false), (5, true), (6, true)];
    if source_term.handoff.references().len() != expected_references.len()
        || source_term
            .handoff
            .references()
            .iter()
            .zip(expected_references)
            .any(|((id, reference), (term, scoped))| {
                id.index() >= expected_references.len()
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

    let expected_requests = [
        (2, 59, 138, 139, "1"),
        (3, 62, 144, 145, "2"),
        (4, 66, 158, 159, "3"),
    ];
    if source_term.handoff.numeric_type_requests().len() != expected_requests.len()
        || source_term
            .handoff
            .numeric_type_requests()
            .iter()
            .zip(expected_requests)
            .any(|((id, request), (term, owner, start, end, spelling))| {
                request.term().index() != term
                    || request.owner().node().index() != owner
                    || request.source_range() != task258b3m2b2b2cp_range(ast, start, end)
                    || request.spelling() != spelling
                    || request.request_ordinal() != id.index()
            })
    {
        return false;
    }

    if source_term.arena.len() != ast.nodes().len()
        || source_term.arena.root().map(|root| root.index()) != ast.root().map(|root| root.index())
    {
        return false;
    }
    source_term.arena.iter().all(|(id, node)| {
        let index = id.index();
        let Some(surface) = ast.nodes().get(index) else {
            return false;
        };
        let expected_kind = match index {
            20 | 24 => "source.term.structure.member.constructor-assignment",
            30 => "source.term.structure.member.update-path-segment",
            51 | 53 | 73 | 75 => "source.term.variable-reference",
            59 | 62 | 66 => "source.term.numeral",
            65 => "source.term.structure.constructor",
            68 => "source.term.structure.field-update",
            69 => "source.term.structure.update",
            _ => "source.surface.unowned",
        };
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
}

#[allow(dead_code)] // Rationale: B2CP freezes a production-private seam whose sole production consumer is the separately scoped B2C task.
fn task258b3m2b2b2cp_range(ast: &SurfaceAst, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id: ast.source_id,
        start,
        end,
    }
}

fn task258b3m2b2b2bp_range(ast: &SurfaceAst, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id: ast.source_id,
        start,
        end,
    }
}

fn extract_task258b3m2b2b2bp_selector(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    loaded_source: &str,
    site: ImportedStructureSelectorSite,
) -> Option<ExtractedStructure> {
    if site.selector != 62
        || !task258b3m2b2b2bp_surface_contract(ast, loaded_source)
        || symbols.module_id() != module
    {
        return None;
    }
    let extracted = extract_structure(
        ast,
        module,
        symbols,
        BindingContextId::new(0),
        &[site.selector],
        None,
        &BTreeSet::new(),
    )
    .ok()?;
    let [selector, constructor] = extracted.terms.as_slice() else {
        return None;
    };
    let [root] = extracted.roots.as_slice() else {
        return None;
    };
    let [selector_member, left, right] = extracted.members.as_slice() else {
        return None;
    };
    let [base_edge, left_edge, right_edge] = extracted.edges.as_slice() else {
        return None;
    };
    if selector.node != 62
        || selector.kind != SourceStructureTermKind::SelectorAccess
        || selector.recovery != SourceStructureRecovery::Normal
        || constructor.node != 61
        || constructor.kind != SourceStructureTermKind::Constructor
        || constructor.recovery != SourceStructureRecovery::Normal
        || !extracted.wrappers.is_empty()
        || root.term != SourceStructureTermId::new(1)
        || selector_member.term != SourceStructureTermId::new(0)
        || selector_member.ordinal != 0
        || selector_member.node != 29
        || selector_member.role != SourceStructureMemberRole::Selector
        || selector_member.parent.is_some()
        || left.term != SourceStructureTermId::new(1)
        || left.ordinal != 0
        || left.node != 20
        || left.role != SourceStructureMemberRole::ConstructorAssignment
        || left.parent.is_some()
        || right.term != SourceStructureTermId::new(1)
        || right.ordinal != 1
        || right.node != 24
        || right.role != SourceStructureMemberRole::ConstructorAssignment
        || right.parent.is_some()
        || !extracted.field_updates.is_empty()
        || base_edge.term != SourceStructureTermId::new(0)
        || base_edge.ordinal != 0
        || base_edge.role != SourceStructureEdgeRole::SelectorBase
        || base_edge.member.is_some()
        || !matches!(base_edge.target, ExtractedTarget::Structure(term) if term == SourceStructureTermId::new(1))
        || left_edge.term != SourceStructureTermId::new(1)
        || left_edge.ordinal != 0
        || left_edge.role != SourceStructureEdgeRole::ConstructorValue
        || left_edge.member != Some(SourceStructureMemberId::new(1))
        || !matches!(left_edge.target, ExtractedTarget::Primary(56))
        || right_edge.term != SourceStructureTermId::new(1)
        || right_edge.ordinal != 1
        || right_edge.role != SourceStructureEdgeRole::ConstructorValue
        || right_edge.member != Some(SourceStructureMemberId::new(2))
        || !matches!(right_edge.target, ExtractedTarget::Primary(59))
        || extracted.primary_roots != [56, 59]
        || !task258b3m2b2b2p_root_is_exact(ast, module, symbols, &root.symbol)
    {
        return None;
    }
    Some(extracted)
}

#[allow(dead_code)] // Rationale: B2CP freezes a production-private seam whose sole production consumer is the separately scoped B2C task.
fn extract_task258b3m2b2b2cp_update(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    loaded_source: &str,
    site: ImportedStructureUpdateSite,
) -> Option<ExtractedStructure> {
    if site.update != 69
        || !task258b3m2b2b2cp_surface_contract(ast, loaded_source)
        || symbols.module_id() != module
    {
        return None;
    }
    let extracted = extract_structure(
        ast,
        module,
        symbols,
        BindingContextId::new(0),
        &[site.update],
        None,
        &BTreeSet::new(),
    )
    .ok()?;
    let [update, constructor] = extracted.terms.as_slice() else {
        return None;
    };
    let [root] = extracted.roots.as_slice() else {
        return None;
    };
    let [update_member, left, right] = extracted.members.as_slice() else {
        return None;
    };
    let [field_update] = extracted.field_updates.as_slice() else {
        return None;
    };
    let [base_edge, value_edge, left_edge, right_edge] = extracted.edges.as_slice() else {
        return None;
    };
    if update.node != 69
        || update.kind != SourceStructureTermKind::FunctionalUpdate
        || update.recovery != SourceStructureRecovery::Normal
        || constructor.node != 65
        || constructor.kind != SourceStructureTermKind::Constructor
        || constructor.recovery != SourceStructureRecovery::Normal
        || !extracted.wrappers.is_empty()
        || root.term != SourceStructureTermId::new(1)
        || update_member.term != SourceStructureTermId::new(0)
        || update_member.ordinal != 0
        || update_member.node != 30
        || update_member.role != SourceStructureMemberRole::UpdatePathSegment
        || update_member.parent.is_some()
        || left.term != SourceStructureTermId::new(1)
        || left.ordinal != 0
        || left.node != 20
        || left.role != SourceStructureMemberRole::ConstructorAssignment
        || left.parent.is_some()
        || right.term != SourceStructureTermId::new(1)
        || right.ordinal != 1
        || right.node != 24
        || right.role != SourceStructureMemberRole::ConstructorAssignment
        || right.parent.is_some()
        || field_update.term != SourceStructureTermId::new(0)
        || field_update.ordinal != 0
        || field_update.node != 68
        || field_update.first_member != SourceStructureMemberId::new(0)
        || field_update.final_member != SourceStructureMemberId::new(0)
        || base_edge.term != SourceStructureTermId::new(0)
        || base_edge.ordinal != 0
        || base_edge.role != SourceStructureEdgeRole::UpdateBase
        || base_edge.member.is_some()
        || !matches!(base_edge.target, ExtractedTarget::Structure(term) if term == SourceStructureTermId::new(1))
        || value_edge.term != SourceStructureTermId::new(0)
        || value_edge.ordinal != 1
        || value_edge.role != SourceStructureEdgeRole::UpdateValue
        || value_edge.member != Some(SourceStructureMemberId::new(0))
        || !matches!(value_edge.target, ExtractedTarget::Primary(67))
        || left_edge.term != SourceStructureTermId::new(1)
        || left_edge.ordinal != 0
        || left_edge.role != SourceStructureEdgeRole::ConstructorValue
        || left_edge.member != Some(SourceStructureMemberId::new(1))
        || !matches!(left_edge.target, ExtractedTarget::Primary(60))
        || right_edge.term != SourceStructureTermId::new(1)
        || right_edge.ordinal != 1
        || right_edge.role != SourceStructureEdgeRole::ConstructorValue
        || right_edge.member != Some(SourceStructureMemberId::new(2))
        || !matches!(right_edge.target, ExtractedTarget::Primary(63))
        || extracted.primary_roots != [60, 63, 67]
        || !task258b3m2b2b2p_root_is_exact(ast, module, symbols, &root.symbol)
    {
        return None;
    }
    Some(extracted)
}

fn structure_root_is_excluded(
    ast: &SurfaceAst,
    root: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
) -> bool {
    fn excluded_descendant_kind(kind: &SurfaceNodeKind) -> bool {
        matches!(
            kind,
            SurfaceNodeKind::TypeArguments
                | SurfaceNodeKind::SetEnumeration
                | SurfaceNodeKind::SetComprehension
                | SurfaceNodeKind::ChoiceTerm
                | SurfaceNodeKind::QuaExpression
                | SurfaceNodeKind::TemplateParameter
                | SurfaceNodeKind::TemplateLoci
                | SurfaceNodeKind::TemplateLocus
                | SurfaceNodeKind::TemplateArguments
                | SurfaceNodeKind::TemplateArgument
        )
    }
    fn subtree_is_excluded(ast: &SurfaceAst, node: usize) -> bool {
        ast.nodes()[node].children.iter().any(|child| {
            excluded_descendant_kind(&ast.nodes()[child.index()].kind)
                || subtree_is_excluded(ast, child.index())
        })
    }

    if subtree_is_excluded(ast, root) {
        return true;
    }
    let parents = parent_indexes(ast);
    let mut cursor = parents[root];
    while let Some(parent) = cursor {
        let node = &ast.nodes()[parent];
        if matches!(
            node.kind,
            SurfaceNodeKind::TemplateParameter
                | SurfaceNodeKind::TemplateLoci
                | SurfaceNodeKind::TemplateLocus
                | SurfaceNodeKind::TemplateArguments
                | SurfaceNodeKind::TemplateArgument
        ) {
            return true;
        }
        if matches!(
            node.kind,
            SurfaceNodeKind::ApplicationTerm
                | SurfaceNodeKind::PrefixExpression(_)
                | SurfaceNodeKind::InfixExpression(_)
                | SurfaceNodeKind::PostfixExpression(_)
        ) && applications.is_some_and(|applications| {
            applications
                .applications()
                .iter()
                .any(|(_, application)| application.source_range() == node.range)
        }) {
            return true;
        }
        cursor = parents[parent];
    }
    false
}

fn normalize_extracted_tables(extracted: &mut ExtractedStructure) {
    extracted
        .wrappers
        .sort_by_key(|wrapper| (wrapper.term.index(), wrapper.ordinal));
    extracted.roots.sort_by_key(|root| root.term.index());

    let mut member_order = (0..extracted.members.len()).collect::<Vec<_>>();
    member_order.sort_by_key(|index| {
        let member = &extracted.members[*index];
        (member.term.index(), member.ordinal)
    });
    let member_remap = member_order
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, SourceStructureMemberId::new(new)))
        .collect::<BTreeMap<_, _>>();
    let old_members = std::mem::take(&mut extracted.members);
    extracted.members = member_order
        .into_iter()
        .map(|old| {
            let mut member = old_members[old].clone();
            member.parent = member.parent.map(|parent| member_remap[&parent.index()]);
            member
        })
        .collect();
    for update in &mut extracted.field_updates {
        update.first_member = member_remap[&update.first_member.index()];
        update.final_member = member_remap[&update.final_member.index()];
    }
    for edge in &mut extracted.edges {
        edge.member = edge.member.map(|member| member_remap[&member.index()]);
    }
    extracted
        .field_updates
        .sort_by_key(|update| (update.term.index(), update.ordinal));
    extracted
        .edges
        .sort_by_key(|edge| (edge.term.index(), edge.ordinal));
}

impl ExtractedTarget {
    const fn structure(self) -> Option<SourceStructureTermId> {
        match self {
            Self::Structure(term) => Some(term),
            Self::Primary(_) | Self::Application(_) => None,
        }
    }
}

#[allow(clippy::too_many_arguments)] // Rationale: preserve the recursive extractor's explicit authority inputs.
fn collect_target(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    node_index: usize,
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
    extracted: &mut ExtractedStructure,
) -> Result<ExtractedTarget, String> {
    let original = ast
        .nodes()
        .get(node_index)
        .ok_or_else(|| "source-structure child site disappeared".to_owned())?;
    if let Some(application_id) = application_target(application, original.range) {
        return Ok(ExtractedTarget::Application(application_id));
    }

    let (core, wrappers) = peel_structure_shells(ast, node_index)?;
    let node = &ast.nodes()[core];
    if !is_structure_kind(&node.kind) {
        extracted.primary_roots.push(node_index);
        return Ok(ExtractedTarget::Primary(node_index));
    }
    if subtree_has_recovery(ast, node) && !degraded_terms.contains(&core) {
        return Err("source-structure normal term contains recovery".to_owned());
    }

    let term = SourceStructureTermId::new(extracted.terms.len());
    let kind = match node.kind {
        SurfaceNodeKind::StructureConstructor => SourceStructureTermKind::Constructor,
        SurfaceNodeKind::SelectorAccess => SourceStructureTermKind::SelectorAccess,
        SurfaceNodeKind::StructureUpdate => SourceStructureTermKind::FunctionalUpdate,
        _ => unreachable!("guarded above"),
    };
    let recovery = if degraded_terms.contains(&core) {
        SourceStructureRecovery::Degraded
    } else {
        SourceStructureRecovery::Normal
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
        SourceStructureTermKind::Constructor => collect_constructor(
            ast,
            module,
            symbols,
            term,
            core,
            application,
            degraded_terms,
            extracted,
        )?,
        SourceStructureTermKind::SelectorAccess => collect_selector(
            ast,
            module,
            symbols,
            term,
            core,
            application,
            degraded_terms,
            extracted,
        )?,
        SourceStructureTermKind::FunctionalUpdate => collect_update(
            ast,
            module,
            symbols,
            term,
            core,
            application,
            degraded_terms,
            extracted,
        )?,
        _ => return Err("unsupported source-structure term kind".to_owned()),
    }
    Ok(ExtractedTarget::Structure(term))
}

#[allow(clippy::too_many_arguments)] // Rationale: preserve constructor extraction and dependency ownership inputs.
fn collect_constructor(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    term: SourceStructureTermId,
    node_index: usize,
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
    extracted: &mut ExtractedStructure,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let children = structural_child_ids(ast, node);
    let Some(root_id) = children.first().copied() else {
        return Err("source-structure constructor lost its root".to_owned());
    };
    let root_node = ast
        .node(root_id)
        .ok_or_else(|| "source-structure constructor root disappeared".to_owned())?;
    if !matches!(root_node.kind, SurfaceNodeKind::QualifiedSymbol)
        || children.iter().any(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::TypeArguments))
        })
    {
        return Err("source-structure constructor root/type-argument shape drift".to_owned());
    }
    let spelling = qualified_symbol_spelling(ast, root_node)
        .map_err(|()| "source-structure constructor root spelling drift".to_owned())?;
    let namespace = NamespacePath::new(module.path().as_str());
    let candidates = symbols
        .symbols()
        .visible_candidates(&namespace, &spelling)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Structure)
        .collect::<Vec<_>>();
    let [root] = candidates.as_slice() else {
        return Err("source-structure constructor needs one structure root".to_owned());
    };
    extracted.roots.push(ExtractedRoot {
        term,
        symbol: root.symbol().clone(),
        contribution: root.contribution(),
    });

    let fields = children
        .into_iter()
        .filter(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FieldArgument))
        })
        .collect::<Vec<_>>();
    for (ordinal, field_id) in fields.into_iter().enumerate() {
        let field = ast.node(field_id).expect("field id came from the AST");
        let label = one_identifier_token(ast, field, &[":"])?;
        let member = SourceStructureMemberId::new(extracted.members.len());
        extracted.members.push(ExtractedMember {
            term,
            ordinal,
            node: label.index(),
            role: SourceStructureMemberRole::ConstructorAssignment,
            parent: None,
        });
        let value = one_structural_child(ast, field)?;
        let target = collect_target(
            ast,
            module,
            symbols,
            value.index(),
            application,
            degraded_terms,
            extracted,
        )?;
        push_edge(
            extracted,
            term,
            SourceStructureEdgeRole::ConstructorValue,
            Some(member),
            target,
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Rationale: preserve selector extraction and dependency ownership inputs.
fn collect_selector(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    term: SourceStructureTermId,
    node_index: usize,
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
    extracted: &mut ExtractedStructure,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let label = one_identifier_token(ast, node, &[".", "(", ",", ")"])?;
    extracted.members.push(ExtractedMember {
        term,
        ordinal: 0,
        node: label.index(),
        role: SourceStructureMemberRole::Selector,
        parent: None,
    });
    let children = structural_child_ids(ast, node);
    let Some((base, arguments)) = children.split_first() else {
        return Err("source-structure selector lost its base".to_owned());
    };
    let base = collect_target(
        ast,
        module,
        symbols,
        base.index(),
        application,
        degraded_terms,
        extracted,
    )?;
    push_edge(
        extracted,
        term,
        SourceStructureEdgeRole::SelectorBase,
        None,
        base,
    );
    for argument in arguments {
        let target = collect_target(
            ast,
            module,
            symbols,
            argument.index(),
            application,
            degraded_terms,
            extracted,
        )?;
        push_edge(
            extracted,
            term,
            SourceStructureEdgeRole::SelectorArgument,
            None,
            target,
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Rationale: preserve update extraction and dependency ownership inputs.
fn collect_update(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    term: SourceStructureTermId,
    node_index: usize,
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
    extracted: &mut ExtractedStructure,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let children = structural_child_ids(ast, node);
    let Some((base, updates)) = children.split_first() else {
        return Err("source-structure update lost its base".to_owned());
    };
    let base = collect_target(
        ast,
        module,
        symbols,
        base.index(),
        application,
        degraded_terms,
        extracted,
    )?;
    push_edge(
        extracted,
        term,
        SourceStructureEdgeRole::UpdateBase,
        None,
        base,
    );
    for (ordinal, update_id) in updates.iter().copied().enumerate() {
        let update = ast
            .node(update_id)
            .ok_or_else(|| "source-structure field update disappeared".to_owned())?;
        if !matches!(update.kind, SurfaceNodeKind::FieldUpdate) {
            return Err("source-structure update has a non-FieldUpdate child".to_owned());
        }
        let path = identifier_tokens(ast, update, &[".", ":="])?;
        if path.is_empty() {
            return Err("source-structure field update has an empty path".to_owned());
        }
        let mut parent = None;
        let first = SourceStructureMemberId::new(extracted.members.len());
        for (member_ordinal, segment) in path.into_iter().enumerate() {
            let member = SourceStructureMemberId::new(extracted.members.len());
            extracted.members.push(ExtractedMember {
                term,
                ordinal: extracted
                    .members
                    .iter()
                    .filter(|member| member.term == term)
                    .count(),
                node: segment.index(),
                role: SourceStructureMemberRole::UpdatePathSegment,
                parent,
            });
            parent = Some(member);
            debug_assert_eq!(member.index(), first.index() + member_ordinal);
        }
        let final_member = parent.expect("nonempty path");
        extracted.field_updates.push(ExtractedFieldUpdate {
            term,
            ordinal,
            node: update_id.index(),
            first_member: first,
            final_member,
        });
        let value = one_structural_child(ast, update)?;
        let target = collect_target(
            ast,
            module,
            symbols,
            value.index(),
            application,
            degraded_terms,
            extracted,
        )?;
        push_edge(
            extracted,
            term,
            SourceStructureEdgeRole::UpdateValue,
            Some(final_member),
            target,
        );
    }
    Ok(())
}

fn push_edge(
    extracted: &mut ExtractedStructure,
    term: SourceStructureTermId,
    role: SourceStructureEdgeRole,
    member: Option<SourceStructureMemberId>,
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
        member,
        target,
    });
}

fn peel_structure_shells(ast: &SurfaceAst, start: usize) -> Result<(usize, Vec<usize>), String> {
    let mut current = start;
    let mut wrappers = Vec::new();
    loop {
        let node = ast
            .nodes()
            .get(current)
            .ok_or_else(|| "source-structure shell disappeared".to_owned())?;
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
                    && contains_structure(ast, node) =>
            {
                wrappers.push(current);
                let children = structural_child_ids(ast, node);
                let [child] = children.as_slice() else {
                    return Err("source-structure wrapper lost its child".to_owned());
                };
                current = child.index();
            }
            _ => break,
        }
    }
    Ok((current, wrappers))
}

fn contains_structure(ast: &SurfaceAst, node: &SurfaceNode) -> bool {
    node.children.iter().any(|child| {
        ast.node(*child)
            .is_some_and(|child| is_structure_kind(&child.kind) || contains_structure(ast, child))
    })
}

fn application_target(
    application: Option<&SourceFunctorApplicationHandoff>,
    range: SourceRange,
) -> Option<SourceFunctorApplicationId> {
    let application = application?;
    application.applications().iter().find_map(|(id, row)| {
        let outer = application
            .wrappers()
            .iter()
            .filter(|(_, wrapper)| wrapper.application() == id)
            .min_by_key(|(_, wrapper)| wrapper.ordinal())
            .map_or(row.source_range(), |(_, wrapper)| wrapper.source_range());
        (outer == range || row.source_range() == range).then_some(id)
    })
}

fn one_structural_child(ast: &SurfaceAst, node: &SurfaceNode) -> Result<SurfaceNodeId, String> {
    let children = structural_child_ids(ast, node);
    let [child] = children.as_slice() else {
        return Err("source-structure association requires one value child".to_owned());
    };
    Ok(*child)
}

fn one_identifier_token(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    punctuation: &[&str],
) -> Result<SurfaceNodeId, String> {
    let identifiers = identifier_tokens(ast, node, punctuation)?;
    let [identifier] = identifiers.as_slice() else {
        return Err("source-structure member requires one identifier".to_owned());
    };
    Ok(*identifier)
}

fn identifier_tokens(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    punctuation: &[&str],
) -> Result<Vec<SurfaceNodeId>, String> {
    let mut identifiers = Vec::new();
    for child in &node.children {
        let Some(child_node) = ast.node(*child) else {
            return Err("source-structure token site disappeared".to_owned());
        };
        let Some(text) = child_node.token_text() else {
            continue;
        };
        if punctuation.contains(&text) {
            continue;
        }
        if text.is_empty() {
            return Err("source-structure member spelling is empty".to_owned());
        }
        identifiers.push(*child);
    }
    Ok(identifiers)
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    binding_env: BindingEnv,
    extracted: ExtractedStructure,
    dependencies: Option<SyntheticSourceStructureDependencies>,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Result<SourceStructureRouteOutput, String> {
    if binding_env.source_id() != ast.source_id || binding_env.module_id() != &module {
        return Err("source-structure binding environment identity mismatch".to_owned());
    }
    let mut kinds = BTreeMap::new();
    let mut recoveries = BTreeMap::new();
    for term in &extracted.terms {
        insert_kind(&mut kinds, term.node, term_kind_key(term.kind))?;
        if term.recovery == SourceStructureRecovery::Degraded {
            recoveries.insert(term.node, NodeRecoveryState::Degraded);
        }
    }
    for wrapper in &extracted.wrappers {
        insert_kind(
            &mut kinds,
            wrapper.node,
            "source.term.structure.parenthesized",
        )?;
        if wrapper.recovery == SourceStructureRecovery::Degraded {
            recoveries.insert(wrapper.node, NodeRecoveryState::Degraded);
        }
    }
    for member in &extracted.members {
        insert_kind(&mut kinds, member.node, member_kind_key(member.role))?;
        if extracted.terms[member.term.index()].recovery == SourceStructureRecovery::Degraded {
            recoveries.insert(member.node, NodeRecoveryState::Degraded);
        }
    }
    for update in &extracted.field_updates {
        insert_kind(
            &mut kinds,
            update.node,
            "source.term.structure.field-update",
        )?;
        if extracted.terms[update.term.index()].recovery == SourceStructureRecovery::Degraded {
            recoveries.insert(update.node, NodeRecoveryState::Degraded);
        }
    }

    let (source_term, application) = if let Some(dependencies) = dependencies {
        let arena = arena_with_overrides(ast, &dependencies.arena, &kinds, &recoveries)?;
        (
            SourceTermParts {
                arena,
                handoff: dependencies.primary,
            },
            dependencies.application,
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
        (parts, None)
    };
    let handoff = build_handoff_with_source_term(
        ast,
        &module,
        symbols,
        &binding_env,
        &extracted,
        &source_term,
        application.as_ref(),
        mutate,
    )?;
    let SourceTermParts {
        arena,
        handoff: primary,
    } = source_term;
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
    let typed_ast = typed_ast
        .with_source_structure(handoff)
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
    if typed_ast.source_structure().is_none()
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
        return Err("source-structure immutable final handoff mismatch".to_owned());
    }
    Ok(SourceStructureRouteOutput {
        typed_ast,
        resolved,
        #[cfg(test)]
        binding_env,
    })
}

#[allow(clippy::too_many_arguments)] // Rationale: keep Task-48/252/253 validation authorities explicit at the shared Task-254 producer seam.
fn build_handoff_with_source_term(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    extracted: &ExtractedStructure,
    source_term: &SourceTermParts,
    application: Option<&SourceFunctorApplicationHandoff>,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Result<SourceStructureHandoff, String> {
    if binding_env.source_id() != ast.source_id
        || binding_env.module_id() != module
        || symbols.module_id() != module
    {
        return Err("source-structure shared dependency identity mismatch".to_owned());
    }
    let primary_id = |root: usize| {
        let range = ast.nodes().get(root)?.range;
        source_term
            .handoff
            .terms()
            .iter()
            .find(|(_, term)| term.parent().is_none() && term.source_range() == range)
            .map(|(id, _)| id)
    };
    let mut requests = Vec::new();
    for (term_ordinal, _) in extracted.terms.iter().enumerate() {
        let term = SourceStructureTermId::new(term_ordinal);
        let mut ordinal = 0;
        if extracted
            .terms
            .get(term_ordinal)
            .is_some_and(|row| row.kind == SourceStructureTermKind::Constructor)
        {
            requests.push(SourceStructureRequestInput {
                term,
                member: None,
                request_ordinal: ordinal,
                kind: SourceStructureRequestKind::ConstructorSignature,
            });
            ordinal += 1;
        }
        for (member_id, _) in extracted
            .members
            .iter()
            .enumerate()
            .filter(|(_, member)| member.term == term)
        {
            let member = SourceStructureMemberId::new(member_id);
            requests.push(SourceStructureRequestInput {
                term,
                member: Some(member),
                request_ordinal: ordinal,
                kind: SourceStructureRequestKind::MemberIdentity,
            });
            ordinal += 1;
            requests.push(SourceStructureRequestInput {
                term,
                member: Some(member),
                request_ordinal: ordinal,
                kind: SourceStructureRequestKind::InheritancePath,
            });
            ordinal += 1;
        }
        requests.push(SourceStructureRequestInput {
            term,
            member: None,
            request_ordinal: ordinal,
            kind: SourceStructureRequestKind::ResultType,
        });
    }

    let mut input = SourceStructureHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms: extracted
            .terms
            .iter()
            .enumerate()
            .map(|(source_ordinal, term)| {
                let node = &ast.nodes()[term.node];
                SourceStructureTermInput {
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
                SourceStructureWrapperInput {
                    term: wrapper.term,
                    ordinal: wrapper.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(wrapper.node)),
                    source_range: node.range,
                    context: extracted.context,
                    spelling: subtree_tokens(ast, node).join(" "),
                    recovery: wrapper.recovery,
                }
            })
            .collect(),
        roots: extracted
            .roots
            .iter()
            .map(|root| SourceStructureRootInput {
                term: root.term,
                symbol: root.symbol.clone(),
                contribution: root.contribution,
            })
            .collect(),
        members: extracted
            .members
            .iter()
            .map(|member| {
                let node = &ast.nodes()[member.node];
                SourceStructureMemberInput {
                    term: member.term,
                    ordinal: member.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(member.node)),
                    source_range: node.range,
                    spelling: node.token_text().unwrap_or_default().to_owned(),
                    role: member.role,
                    parent: member.parent,
                }
            })
            .collect(),
        field_updates: extracted
            .field_updates
            .iter()
            .map(|update| {
                let node = &ast.nodes()[update.node];
                SourceFieldUpdateInput {
                    term: update.term,
                    ordinal: update.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(update.node)),
                    source_range: node.range,
                    spelling: subtree_tokens(ast, node).join(" "),
                    first_member: update.first_member,
                    final_member: update.final_member,
                }
            })
            .collect(),
        edges: extracted
            .edges
            .iter()
            .map(|edge| {
                let target = match edge.target {
                    ExtractedTarget::Primary(root) => {
                        SourceStructureTarget::Primary(primary_id(root).ok_or_else(|| {
                            "source-structure primary child is not a Task-252 root".to_owned()
                        })?)
                    }
                    ExtractedTarget::Application(application) => {
                        SourceStructureTarget::Application(application)
                    }
                    ExtractedTarget::Structure(structure) => {
                        SourceStructureTarget::Structure(structure)
                    }
                };
                Ok(SourceStructureEdgeInput {
                    term: edge.term,
                    ordinal: edge.ordinal,
                    role: edge.role,
                    member: edge.member,
                    target,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        requests,
    };
    mutate(&mut input);
    SourceStructureProducer::build(
        input,
        symbols,
        binding_env,
        &source_term.handoff,
        application,
        &source_term.arena,
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum ImportedStructureConstructorSurfaceMutation {
    None,
    NodeKind(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    RootIdentity,
    DirectProductionSeam,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum ImportedStructureConstructorTestMutation {
    None,
    TermSite,
    TermRange,
    TermOrdinal,
    TermContext,
    TermRecovery,
    TermSpelling,
    TermKind,
    RootTerm,
    RootSymbol,
    RootContribution,
    MemberTerm,
    MemberOrdinal,
    MemberSite,
    MemberRange,
    MemberSpelling,
    MemberRole,
    MemberParent,
    FieldUpdateExtra,
    EdgeTerm,
    EdgeOrdinal,
    EdgeRole,
    EdgeMember,
    EdgeTarget,
    RequestTerm,
    RequestOrdinal,
    RequestMember,
    RequestKind,
    StalePrimaryReplay,
    TermRangeAndStalePrimaryReplay,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(in crate::runner) struct ImportedStructureConstructorTestOptions {
    pub(in crate::runner) constructor: usize,
    pub(in crate::runner) context: BindingContextId,
    pub(in crate::runner) surface_mutation: ImportedStructureConstructorSurfaceMutation,
    pub(in crate::runner) handoff_mutation: ImportedStructureConstructorTestMutation,
}

#[cfg(test)]
#[derive(Debug)]
pub(in crate::runner) struct ImportedStructureConstructorTestOutput {
    pub(in crate::runner) handoff: SourceStructureHandoff,
    pub(in crate::runner) primary_counts: (usize, usize, usize),
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum ImportedStructureSelectorSurfaceMutation {
    None,
    NodeKind(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    RootIdentity,
    DirectProductionSeam,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum ImportedStructureSelectorTestMutation {
    None,
    BindingSourceId,
    BindingModuleId,
    BindingContextCount,
    BindingCount,
    BindingDiagnosticCount,
    ModuleContextOwner,
    ModuleContextParent,
    ModuleContextLayer,
    ModuleContextScope,
    ModuleContextBindings,
    ModuleContextVisibleBindings,
    ModuleContextRecovery,
    ProofContextOwner,
    ProofContextParent,
    ProofContextLayer,
    ProofContextScope,
    ProofContextBindings,
    ProofContextVisibleBindings,
    ProofContextRecovery,
    BindingSpelling,
    BindingKind,
    BindingIdentityKind,
    BindingIdentitySpelling,
    BindingIdentityRange,
    BindingOwner,
    BindingDeclarationRange,
    BindingVisibleOrdinal,
    BindingTypeSite,
    BindingStatus,
    BindingCaptured,
    BindingDiagnostics,
    BindingRecovery,
    PrimarySourceId,
    PrimaryModuleId,
    PrimaryTermSite(usize),
    PrimaryTermRange(usize),
    PrimaryTermOrdinal(usize),
    PrimaryTermContext(usize),
    PrimaryTermRecovery(usize),
    PrimaryTermSpelling(usize),
    PrimaryTermKind(usize),
    PrimaryTermRole(usize),
    PrimaryTermParent(usize),
    PrimaryReferenceTerm(usize),
    PrimaryReferenceBinding(usize),
    PrimaryReferenceRole(usize),
    PrimaryReferenceUseOrdinal(usize),
    PrimaryReferenceScope(usize),
    NumericRequestTerm(usize),
    NumericRequestOwner(usize),
    NumericRequestRange(usize),
    NumericRequestSpelling(usize),
    NumericRequestOrdinal(usize),
    TermSite(usize),
    TermRange(usize),
    TermOrdinal(usize),
    TermContext(usize),
    TermRecovery(usize),
    TermSpelling(usize),
    TermKind(usize),
    RootTerm,
    RootSymbol,
    RootContribution,
    MemberTerm(usize),
    MemberOrdinal(usize),
    MemberSite(usize),
    MemberRange(usize),
    MemberSpelling(usize),
    MemberRole(usize),
    MemberParent(usize),
    FieldUpdateExtra,
    EdgeTerm(usize),
    EdgeOrdinal(usize),
    EdgeRole(usize),
    EdgeMember(usize),
    EdgeTarget(usize),
    RequestTerm(usize),
    RequestOrdinal(usize),
    RequestMember(usize),
    RequestKind(usize),
    StalePrimaryReplay,
    TermRangeAndStalePrimaryReplay,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(in crate::runner) struct ImportedStructureSelectorTestOptions {
    pub(in crate::runner) selector: usize,
    pub(in crate::runner) context: BindingContextId,
    pub(in crate::runner) surface_mutation: ImportedStructureSelectorSurfaceMutation,
    pub(in crate::runner) handoff_mutation: ImportedStructureSelectorTestMutation,
}

#[cfg(test)]
#[derive(Debug)]
pub(in crate::runner) struct ImportedStructureSelectorTestOutput {
    pub(in crate::runner) handoff: SourceStructureHandoff,
    pub(in crate::runner) primary_counts: (usize, usize, usize),
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum ImportedStructureUpdateSurfaceMutation {
    None,
    NodeKind(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    RootIdentity,
    DirectProductionSeam,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum ImportedStructureUpdateTestMutation {
    None,
    BindingSourceId,
    BindingModuleId,
    BindingContextCount,
    BindingCount,
    BindingDiagnosticCount,
    ModuleContextOwner,
    ModuleContextParent,
    ModuleContextLayer,
    ModuleContextScope,
    ModuleContextBindings,
    ModuleContextVisibleBindings,
    ModuleContextRecovery,
    ProofContextOwner,
    ProofContextParent,
    ProofContextLayer,
    ProofContextScope,
    ProofContextBindings,
    ProofContextVisibleBindings,
    ProofContextRecovery,
    BindingSpelling,
    BindingKind,
    BindingIdentityKind,
    BindingIdentitySpelling,
    BindingIdentityRange,
    BindingOwner,
    BindingDeclarationRange,
    BindingVisibleOrdinal,
    BindingTypeSite,
    BindingStatus,
    BindingCaptured,
    BindingDiagnostics,
    BindingRecovery,
    PrimarySourceId,
    PrimaryModuleId,
    PrimaryTermSite(usize),
    PrimaryTermRange(usize),
    PrimaryTermOrdinal(usize),
    PrimaryTermContext(usize),
    PrimaryTermRecovery(usize),
    PrimaryTermSpelling(usize),
    PrimaryTermKind(usize),
    PrimaryTermRole(usize),
    PrimaryTermParent(usize),
    PrimaryReferenceTerm(usize),
    PrimaryReferenceBinding(usize),
    PrimaryReferenceRole(usize),
    PrimaryReferenceUseOrdinal(usize),
    PrimaryReferenceScope(usize),
    NumericRequestTerm(usize),
    NumericRequestOwner(usize),
    NumericRequestRange(usize),
    NumericRequestSpelling(usize),
    NumericRequestOrdinal(usize),
    TermSite(usize),
    TermRange(usize),
    TermOrdinal(usize),
    TermContext(usize),
    TermRecovery(usize),
    TermSpelling(usize),
    TermKind(usize),
    RootTerm,
    RootSymbol,
    RootContribution,
    MemberTerm(usize),
    MemberOrdinal(usize),
    MemberSite(usize),
    MemberRange(usize),
    MemberSpelling(usize),
    MemberRole(usize),
    MemberParent(usize),
    FieldUpdateMissing,
    FieldUpdateExtra,
    FieldUpdateTerm(usize),
    FieldUpdateOrdinal(usize),
    FieldUpdateSite(usize),
    FieldUpdateRange(usize),
    FieldUpdateSpelling(usize),
    FieldUpdateFirstMember(usize),
    FieldUpdateFinalMember(usize),
    EdgeTerm(usize),
    EdgeOrdinal(usize),
    EdgeRole(usize),
    EdgeMember(usize),
    EdgeTarget(usize),
    RequestTerm(usize),
    RequestOrdinal(usize),
    RequestMember(usize),
    RequestKind(usize),
    StalePrimaryReplay,
    TermRangeAndStalePrimaryReplay,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(in crate::runner) struct ImportedStructureUpdateTestOptions {
    pub(in crate::runner) update: usize,
    pub(in crate::runner) context: BindingContextId,
    pub(in crate::runner) surface_mutation: ImportedStructureUpdateSurfaceMutation,
    pub(in crate::runner) handoff_mutation: ImportedStructureUpdateTestMutation,
}

#[cfg(test)]
#[derive(Debug)]
pub(in crate::runner) struct ImportedStructureUpdateTestOutput {
    pub(in crate::runner) handoff: SourceStructureHandoff,
    pub(in crate::runner) primary_counts: (usize, usize, usize),
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[cfg(test)]
fn task258b3m2b2b2p_surface_contract_with_mutation(
    ast: &SurfaceAst,
    loaded_source: &str,
    mutation: ImportedStructureConstructorSurfaceMutation,
) -> bool {
    task258b3m2b2b2p_surface_contract_impl(
        ast,
        loaded_source,
        |kinds, ranges, recoveries, children, root| match mutation {
            ImportedStructureConstructorSurfaceMutation::None
            | ImportedStructureConstructorSurfaceMutation::DirectProductionSeam => {}
            ImportedStructureConstructorSurfaceMutation::NodeKind(index) => {
                if let Some(kind) = kinds.get_mut(index) {
                    kind.push('!');
                }
            }
            ImportedStructureConstructorSurfaceMutation::NodeRange(index) => {
                if let Some(range) = ranges.get_mut(index) {
                    range.1 = range.1.saturating_add(1);
                }
            }
            ImportedStructureConstructorSurfaceMutation::NodeRecovery(index) => {
                if let Some(recovered) = recoveries.get_mut(index) {
                    *recovered = !*recovered;
                }
            }
            ImportedStructureConstructorSurfaceMutation::NodeChildren(index) => {
                if let Some(node_children) = children.get_mut(index) {
                    if node_children.len() > 1 {
                        node_children.rotate_left(1);
                    } else {
                        node_children.push(index);
                    }
                }
            }
            ImportedStructureConstructorSurfaceMutation::RootIdentity => *root = None,
        },
    )
}

#[cfg(test)]
fn task258b3m2b2b2bp_surface_contract_with_mutation(
    ast: &SurfaceAst,
    loaded_source: &str,
    mutation: ImportedStructureSelectorSurfaceMutation,
) -> bool {
    task258b3m2b2b2bp_surface_contract_impl(
        ast,
        loaded_source,
        |kinds, ranges, recoveries, children, root| match mutation {
            ImportedStructureSelectorSurfaceMutation::None
            | ImportedStructureSelectorSurfaceMutation::DirectProductionSeam => {}
            ImportedStructureSelectorSurfaceMutation::NodeKind(index) => {
                if let Some(kind) = kinds.get_mut(index) {
                    kind.push('!');
                }
            }
            ImportedStructureSelectorSurfaceMutation::NodeRange(index) => {
                if let Some(range) = ranges.get_mut(index) {
                    range.1 = range.1.saturating_add(1);
                }
            }
            ImportedStructureSelectorSurfaceMutation::NodeRecovery(index) => {
                if let Some(recovered) = recoveries.get_mut(index) {
                    *recovered = !*recovered;
                }
            }
            ImportedStructureSelectorSurfaceMutation::NodeChildren(index) => {
                if let Some(node_children) = children.get_mut(index) {
                    if node_children.len() > 1 {
                        node_children.rotate_left(1);
                    } else {
                        node_children.push(index);
                    }
                }
            }
            ImportedStructureSelectorSurfaceMutation::RootIdentity => *root = None,
        },
    )
}

#[cfg(test)]
fn task258b3m2b2b2cp_surface_contract_with_mutation(
    ast: &SurfaceAst,
    loaded_source: &str,
    mutation: ImportedStructureUpdateSurfaceMutation,
) -> bool {
    task258b3m2b2b2cp_surface_contract_impl(
        ast,
        loaded_source,
        |kinds, ranges, recoveries, children, root| match mutation {
            ImportedStructureUpdateSurfaceMutation::None
            | ImportedStructureUpdateSurfaceMutation::DirectProductionSeam => {}
            ImportedStructureUpdateSurfaceMutation::NodeKind(index) => {
                if let Some(kind) = kinds.get_mut(index) {
                    kind.push('!');
                }
            }
            ImportedStructureUpdateSurfaceMutation::NodeRange(index) => {
                if let Some(range) = ranges.get_mut(index) {
                    range.1 = range.1.saturating_add(1);
                }
            }
            ImportedStructureUpdateSurfaceMutation::NodeRecovery(index) => {
                if let Some(recovered) = recoveries.get_mut(index) {
                    *recovered = !*recovered;
                }
            }
            ImportedStructureUpdateSurfaceMutation::NodeChildren(index) => {
                if let Some(node_children) = children.get_mut(index) {
                    if node_children.len() > 1 {
                        node_children.rotate_left(1);
                    } else {
                        node_children.push(index);
                    }
                }
            }
            ImportedStructureUpdateSurfaceMutation::RootIdentity => *root = None,
        },
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)] // Rationale: keep exact source, lower rows, context, and both corruption classes explicit.
pub(in crate::runner) fn imported_structure_constructor_handoff_for_test(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    loaded_source: &str,
    roots: &[(usize, BindingContextId)],
    options: ImportedStructureConstructorTestOptions,
) -> Option<Result<ImportedStructureConstructorTestOutput, String>> {
    let ImportedStructureConstructorTestOptions {
        constructor,
        context,
        surface_mutation,
        handoff_mutation,
    } = options;
    let site = ImportedStructureConstructorSite { constructor };
    if surface_mutation != ImportedStructureConstructorSurfaceMutation::DirectProductionSeam
        && !task258b3m2b2b2p_surface_contract_with_mutation(ast, loaded_source, surface_mutation)
    {
        return None;
    }
    let owned_node_kinds =
        imported_structure_constructor_owned_node_kinds(ast, module, symbols, loaded_source, site)?;
    let source_term = match source_term_parts_for_context_roots(
        ast,
        module.clone(),
        binding_env,
        roots.iter().copied(),
        &owned_node_kinds,
    ) {
        Ok(source_term) => source_term,
        Err(error) => return Some(Err(format!("Task252: {error}"))),
    };
    let mut extracted =
        extract_task258b3m2b2b2p_constructor(ast, module, symbols, loaded_source, site)?;
    extracted.context = context;
    let selected_contribution = extracted.roots[0].contribution;
    let substitute_contribution = symbols
        .contributions()
        .iter()
        .map(|entry| entry.id())
        .find(|candidate| *candidate != selected_contribution)
        .unwrap_or(selected_contribution);
    let stale_replay = matches!(
        handoff_mutation,
        ImportedStructureConstructorTestMutation::StalePrimaryReplay
            | ImportedStructureConstructorTestMutation::TermRangeAndStalePrimaryReplay
    );
    let handoff = if matches!(
        handoff_mutation,
        ImportedStructureConstructorTestMutation::None
            | ImportedStructureConstructorTestMutation::StalePrimaryReplay
    ) {
        match imported_structure_constructor_handoff_in_context(
            ast,
            module,
            symbols,
            binding_env,
            &source_term,
            loaded_source,
            site,
            context,
        )? {
            Ok(handoff) => handoff,
            Err(error) => return Some(Err(format!("Task254: {error}"))),
        }
    } else {
        match build_handoff_with_source_term(
            ast,
            module,
            symbols,
            binding_env,
            &extracted,
            &source_term,
            None,
            |input| match handoff_mutation {
                ImportedStructureConstructorTestMutation::None
                | ImportedStructureConstructorTestMutation::StalePrimaryReplay => {}
                ImportedStructureConstructorTestMutation::TermSite => {
                    input.terms[0].site = TypedSiteRef::Node(TypedNodeId::new(60));
                }
                ImportedStructureConstructorTestMutation::TermRange
                | ImportedStructureConstructorTestMutation::TermRangeAndStalePrimaryReplay => {
                    input.terms[0].source_range.start += 1;
                }
                ImportedStructureConstructorTestMutation::TermOrdinal => {
                    input.terms[0].source_ordinal = 1;
                }
                ImportedStructureConstructorTestMutation::TermContext => {
                    input.terms[0].context = BindingContextId::new(0);
                }
                ImportedStructureConstructorTestMutation::TermRecovery => {
                    input.terms[0].recovery = SourceStructureRecovery::Degraded;
                }
                ImportedStructureConstructorTestMutation::TermSpelling => {
                    input.terms[0].spelling.push(' ');
                }
                ImportedStructureConstructorTestMutation::TermKind => {
                    input.terms[0].kind = SourceStructureTermKind::SelectorAccess;
                }
                ImportedStructureConstructorTestMutation::RootTerm => {
                    input.roots[0].term = SourceStructureTermId::new(1);
                }
                ImportedStructureConstructorTestMutation::RootSymbol => {
                    input.roots[0].symbol = SymbolId::new(
                        module.clone(),
                        mizar_resolve::resolved_ast::LocalSymbolId::new("B2P/substitute"),
                        mizar_resolve::resolved_ast::FullyQualifiedName::new(
                            "b2p::substitute::TypeCaseStruct",
                        ),
                    );
                }
                ImportedStructureConstructorTestMutation::RootContribution => {
                    input.roots[0].contribution = substitute_contribution;
                }
                ImportedStructureConstructorTestMutation::MemberTerm => {
                    input.members[0].term = SourceStructureTermId::new(1);
                }
                ImportedStructureConstructorTestMutation::MemberOrdinal => {
                    input.members[1].ordinal = 0;
                }
                ImportedStructureConstructorTestMutation::MemberSite => {
                    input.members[0].site = TypedSiteRef::Node(TypedNodeId::new(24));
                }
                ImportedStructureConstructorTestMutation::MemberRange => {
                    input.members[0].source_range.start += 1;
                }
                ImportedStructureConstructorTestMutation::MemberSpelling => {
                    input.members[0].spelling.push('!');
                }
                ImportedStructureConstructorTestMutation::MemberRole => {
                    input.members[0].role = SourceStructureMemberRole::Selector;
                }
                ImportedStructureConstructorTestMutation::MemberParent => {
                    input.members[0].parent = Some(SourceStructureMemberId::new(1));
                }
                ImportedStructureConstructorTestMutation::FieldUpdateExtra => {
                    input.field_updates.push(SourceFieldUpdateInput {
                        term: SourceStructureTermId::new(0),
                        ordinal: 0,
                        site: TypedSiteRef::Node(TypedNodeId::new(55)),
                        source_range: ast.nodes()[55].range,
                        spelling: "x : 1".to_owned(),
                        first_member: SourceStructureMemberId::new(0),
                        final_member: SourceStructureMemberId::new(0),
                    });
                }
                ImportedStructureConstructorTestMutation::EdgeTerm => {
                    input.edges[0].term = SourceStructureTermId::new(1);
                }
                ImportedStructureConstructorTestMutation::EdgeOrdinal => {
                    input.edges[1].ordinal = 0;
                }
                ImportedStructureConstructorTestMutation::EdgeRole => {
                    input.edges[0].role = SourceStructureEdgeRole::SelectorBase;
                }
                ImportedStructureConstructorTestMutation::EdgeMember => {
                    input.edges[0].member = None;
                }
                ImportedStructureConstructorTestMutation::EdgeTarget => {
                    input.edges[0].target = SourceStructureTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(4),
                    );
                }
                ImportedStructureConstructorTestMutation::RequestTerm => {
                    input.requests[0].term = SourceStructureTermId::new(1);
                }
                ImportedStructureConstructorTestMutation::RequestOrdinal => {
                    input.requests[1].request_ordinal = 0;
                }
                ImportedStructureConstructorTestMutation::RequestMember => {
                    input.requests[0].member = Some(SourceStructureMemberId::new(0));
                }
                ImportedStructureConstructorTestMutation::RequestKind => {
                    input.requests[0].kind = SourceStructureRequestKind::ResultType;
                }
            },
        ) {
            Ok(handoff) => handoff,
            Err(error) => return Some(Err(format!("Task254: {error}"))),
        }
    };

    if stale_replay {
        let stale_source_term = match source_term_parts_for_context_roots(
            ast,
            module.clone(),
            binding_env,
            roots.iter().copied().filter(|(root, _)| *root != 57),
            &owned_node_kinds,
        ) {
            Ok(source_term) => source_term,
            Err(error) => return Some(Err(format!("Task252: {error}"))),
        };
        let stale_typed = match task258b3m2b2b2p_typed_ast(ast, module, stale_source_term) {
            Ok(typed) => typed,
            Err(error) => return Some(Err(format!("TypedAst: {error}"))),
        };
        return Some(match stale_typed.with_source_structure(handoff) {
            Ok(_) => Err("BUG: TypedAst accepted stale Task252 fingerprint".to_owned()),
            Err(error) => Err(format!(
                "TypedAst: rejected stale Task252 fingerprint: {error}"
            )),
        });
    }

    let primary_counts = (
        source_term.handoff.terms().len(),
        source_term.handoff.references().len(),
        source_term.handoff.numeric_type_requests().len(),
    );
    let typed_ast = match task258b3m2b2b2p_typed_ast(ast, module, source_term) {
        Ok(typed) => match typed.with_source_structure(handoff.clone()) {
            Ok(typed) => typed,
            Err(error) => return Some(Err(format!("TypedAst: {error}"))),
        },
        Err(error) => return Some(Err(format!("TypedAst: {error}"))),
    };
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
    let resolved = match assemble_empty_resolved_typed_ast(&typed_ast, node_hints) {
        Ok(resolved) => resolved,
        Err(error) => return Some(Err(format!("ResolvedTypedAst: {error}"))),
    };
    Some(Ok(ImportedStructureConstructorTestOutput {
        handoff,
        primary_counts,
        typed_ast,
        resolved,
    }))
}

#[cfg(test)]
fn task258b3m2b2b2bp_is_binding_mutation(mutation: ImportedStructureSelectorTestMutation) -> bool {
    matches!(
        mutation,
        ImportedStructureSelectorTestMutation::BindingSourceId
            | ImportedStructureSelectorTestMutation::BindingModuleId
            | ImportedStructureSelectorTestMutation::BindingContextCount
            | ImportedStructureSelectorTestMutation::BindingCount
            | ImportedStructureSelectorTestMutation::BindingDiagnosticCount
            | ImportedStructureSelectorTestMutation::ModuleContextOwner
            | ImportedStructureSelectorTestMutation::ModuleContextParent
            | ImportedStructureSelectorTestMutation::ModuleContextLayer
            | ImportedStructureSelectorTestMutation::ModuleContextScope
            | ImportedStructureSelectorTestMutation::ModuleContextBindings
            | ImportedStructureSelectorTestMutation::ModuleContextVisibleBindings
            | ImportedStructureSelectorTestMutation::ModuleContextRecovery
            | ImportedStructureSelectorTestMutation::ProofContextOwner
            | ImportedStructureSelectorTestMutation::ProofContextParent
            | ImportedStructureSelectorTestMutation::ProofContextLayer
            | ImportedStructureSelectorTestMutation::ProofContextScope
            | ImportedStructureSelectorTestMutation::ProofContextBindings
            | ImportedStructureSelectorTestMutation::ProofContextVisibleBindings
            | ImportedStructureSelectorTestMutation::ProofContextRecovery
            | ImportedStructureSelectorTestMutation::BindingSpelling
            | ImportedStructureSelectorTestMutation::BindingKind
            | ImportedStructureSelectorTestMutation::BindingIdentityKind
            | ImportedStructureSelectorTestMutation::BindingIdentitySpelling
            | ImportedStructureSelectorTestMutation::BindingIdentityRange
            | ImportedStructureSelectorTestMutation::BindingOwner
            | ImportedStructureSelectorTestMutation::BindingDeclarationRange
            | ImportedStructureSelectorTestMutation::BindingVisibleOrdinal
            | ImportedStructureSelectorTestMutation::BindingTypeSite
            | ImportedStructureSelectorTestMutation::BindingStatus
            | ImportedStructureSelectorTestMutation::BindingCaptured
            | ImportedStructureSelectorTestMutation::BindingDiagnostics
            | ImportedStructureSelectorTestMutation::BindingRecovery
    )
}

#[cfg(test)]
fn task258b3m2b2b2bp_is_primary_mutation(mutation: ImportedStructureSelectorTestMutation) -> bool {
    matches!(
        mutation,
        ImportedStructureSelectorTestMutation::PrimarySourceId
            | ImportedStructureSelectorTestMutation::PrimaryModuleId
            | ImportedStructureSelectorTestMutation::PrimaryTermSite(_)
            | ImportedStructureSelectorTestMutation::PrimaryTermRange(_)
            | ImportedStructureSelectorTestMutation::PrimaryTermOrdinal(_)
            | ImportedStructureSelectorTestMutation::PrimaryTermContext(_)
            | ImportedStructureSelectorTestMutation::PrimaryTermRecovery(_)
            | ImportedStructureSelectorTestMutation::PrimaryTermSpelling(_)
            | ImportedStructureSelectorTestMutation::PrimaryTermKind(_)
            | ImportedStructureSelectorTestMutation::PrimaryTermRole(_)
            | ImportedStructureSelectorTestMutation::PrimaryTermParent(_)
            | ImportedStructureSelectorTestMutation::PrimaryReferenceTerm(_)
            | ImportedStructureSelectorTestMutation::PrimaryReferenceBinding(_)
            | ImportedStructureSelectorTestMutation::PrimaryReferenceRole(_)
            | ImportedStructureSelectorTestMutation::PrimaryReferenceUseOrdinal(_)
            | ImportedStructureSelectorTestMutation::PrimaryReferenceScope(_)
            | ImportedStructureSelectorTestMutation::NumericRequestTerm(_)
            | ImportedStructureSelectorTestMutation::NumericRequestOwner(_)
            | ImportedStructureSelectorTestMutation::NumericRequestRange(_)
            | ImportedStructureSelectorTestMutation::NumericRequestSpelling(_)
            | ImportedStructureSelectorTestMutation::NumericRequestOrdinal(_)
    )
}

#[cfg(test)]
fn task258b3m2b2b2bp_binding_env_with_mutation(
    binding_env: &BindingEnv,
    mutation: ImportedStructureSelectorTestMutation,
) -> Result<BindingEnv, String> {
    let mut diagnostics = BindingDiagnosticTable::new();
    for (_, diagnostic) in binding_env.diagnostics().iter() {
        diagnostics.insert(BindingDiagnosticDraft {
            source_range: diagnostic.source_range,
            class: diagnostic.class,
            severity: diagnostic.severity,
            message_key: diagnostic.message_key.clone(),
            recovery: diagnostic.recovery,
        });
    }
    let added_diagnostic = matches!(
        mutation,
        ImportedStructureSelectorTestMutation::BindingDiagnosticCount
            | ImportedStructureSelectorTestMutation::BindingDiagnostics
    )
    .then(|| {
        diagnostics.insert(BindingDiagnosticDraft {
            source_range: None,
            class: BindingDiagnosticClass::UnsupportedSourceShape,
            severity: BindingDiagnosticSeverity::Note,
            message_key: "checker.binding.b2bp.test-mutation".to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        })
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
    let binding = &mut binding_drafts[0];
    match mutation {
        ImportedStructureSelectorTestMutation::BindingSpelling => {
            binding.spelling = "z".to_owned();
        }
        ImportedStructureSelectorTestMutation::BindingKind => {
            binding.kind = BindingKind::LetBinding;
        }
        ImportedStructureSelectorTestMutation::BindingIdentityKind => {
            binding.identity = BinderIdentity::Generated {
                context: BindingContextId::new(0),
                counter: 1,
            };
        }
        ImportedStructureSelectorTestMutation::BindingIdentitySpelling => {
            let BinderIdentity::ReservedVariable { spelling, .. } = &mut binding.identity else {
                return Err("Task48 baseline binding identity is not reserved".to_owned());
            };
            *spelling = "z".to_owned();
        }
        ImportedStructureSelectorTestMutation::BindingIdentityRange => {
            let BinderIdentity::ReservedVariable {
                declaration_range, ..
            } = &mut binding.identity
            else {
                return Err("Task48 baseline binding identity is not reserved".to_owned());
            };
            declaration_range.start += 1;
        }
        ImportedStructureSelectorTestMutation::BindingOwner => {
            binding.owner_context = BindingContextId::new(1);
        }
        ImportedStructureSelectorTestMutation::BindingDeclarationRange => {
            binding.declaration_range.start += 1;
        }
        ImportedStructureSelectorTestMutation::BindingVisibleOrdinal => {
            binding.visible_after_ordinal += 1;
        }
        ImportedStructureSelectorTestMutation::BindingTypeSite => {
            binding.type_site = BindingTypeSite::Missing;
        }
        ImportedStructureSelectorTestMutation::BindingStatus => {
            binding.status = BindingStatus::Active;
        }
        ImportedStructureSelectorTestMutation::BindingCaptured => {
            binding.captured = CapturedFreeVariables::new(vec![binding.identity.clone()]);
        }
        ImportedStructureSelectorTestMutation::BindingDiagnostics => {
            binding.diagnostics = vec![added_diagnostic.expect("diagnostic was inserted")];
        }
        ImportedStructureSelectorTestMutation::BindingRecovery => {
            binding.recovery = BindingRecoveryState::Degraded;
        }
        _ => {}
    }
    if mutation == ImportedStructureSelectorTestMutation::BindingCount {
        binding_drafts.push(BindingDraft {
            spelling: "b2bp_extra".to_owned(),
            kind: BindingKind::Generated,
            identity: BinderIdentity::Generated {
                context: BindingContextId::new(0),
                counter: 1,
            },
            owner_context: BindingContextId::new(0),
            declaration_range: SourceRange {
                source_id: binding_env.source_id(),
                start: 39,
                end: 40,
            },
            visible_after_ordinal: 1,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
    }
    let mut bindings = BindingTable::new();
    for draft in binding_drafts {
        bindings.insert(draft);
    }

    let baseline_proof_scope = binding_env
        .contexts()
        .get(BindingContextId::new(1))
        .and_then(|context| context.lexical_scope.clone());
    let mut context_drafts = binding_env
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
    match mutation {
        ImportedStructureSelectorTestMutation::ModuleContextOwner => {
            context_drafts[0].owner = BindingContextOwner::Generated("B2BP".to_owned());
        }
        ImportedStructureSelectorTestMutation::ModuleContextParent => {
            context_drafts[0].parent = Some(BindingContextId::new(1));
        }
        ImportedStructureSelectorTestMutation::ModuleContextLayer => {
            context_drafts[0].layer = BindingContextLayer::Block;
        }
        ImportedStructureSelectorTestMutation::ModuleContextScope => {
            context_drafts[0].lexical_scope = baseline_proof_scope.clone();
        }
        ImportedStructureSelectorTestMutation::ModuleContextBindings => {
            context_drafts[0].bindings.clear();
        }
        ImportedStructureSelectorTestMutation::ModuleContextVisibleBindings => {
            context_drafts[0].visible_bindings.clear();
        }
        ImportedStructureSelectorTestMutation::ModuleContextRecovery => {
            context_drafts[0].recovery = BindingContextRecovery::Degraded;
        }
        ImportedStructureSelectorTestMutation::ProofContextOwner => {
            let BindingContextOwner::SourceStatement { source_range } = context_drafts[1].owner
            else {
                return Err("Task48 proof owner is not source-statement".to_owned());
            };
            context_drafts[1].owner = BindingContextOwner::SourceFormula { source_range };
        }
        ImportedStructureSelectorTestMutation::ProofContextParent => {
            context_drafts[1].parent = None;
        }
        ImportedStructureSelectorTestMutation::ProofContextLayer => {
            context_drafts[1].layer = BindingContextLayer::Block;
        }
        ImportedStructureSelectorTestMutation::ProofContextScope => {
            context_drafts[1].lexical_scope = None;
        }
        ImportedStructureSelectorTestMutation::ProofContextBindings => {
            context_drafts[1]
                .bindings
                .push(mizar_checker::binding_env::BindingId::new(0));
        }
        ImportedStructureSelectorTestMutation::ProofContextVisibleBindings => {
            context_drafts[1].visible_bindings.clear();
        }
        ImportedStructureSelectorTestMutation::ProofContextRecovery => {
            context_drafts[1].recovery = BindingContextRecovery::Degraded;
        }
        _ => {}
    }
    if mutation == ImportedStructureSelectorTestMutation::BindingCount {
        context_drafts[0]
            .bindings
            .push(mizar_checker::binding_env::BindingId::new(1));
    }
    if mutation == ImportedStructureSelectorTestMutation::BindingContextCount {
        context_drafts.push(BindingContextDraft {
            owner: BindingContextOwner::Generated("B2BP-extra".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: baseline_proof_scope,
            bindings: Vec::new(),
            visible_bindings: vec![mizar_checker::binding_env::BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
    }
    let mut contexts = BindingContextTable::new();
    for draft in context_drafts {
        contexts.insert(draft);
    }

    let module_id = if mutation == ImportedStructureSelectorTestMutation::BindingModuleId {
        ModuleId::new(
            binding_env.module_id().package().clone(),
            mizar_session::ModulePath::new("tests.b2bp.binding-module-substitute"),
        )
    } else {
        binding_env.module_id().clone()
    };
    let source_id = if mutation == ImportedStructureSelectorTestMutation::BindingSourceId {
        let allocator = mizar_session::InMemorySessionIdAllocator::new();
        mizar_session::SessionIdAllocator::next_source_id(
            &allocator,
            super::super::shared::snapshot_id(25_801),
        )
        .map_err(|error| error.to_string())?;
        mizar_session::SessionIdAllocator::next_source_id(
            &allocator,
            super::super::shared::snapshot_id(25_801),
        )
        .map_err(|error| error.to_string())?
    } else {
        binding_env.source_id()
    };
    BindingEnv::try_new(BindingEnvParts {
        source_id,
        module_id,
        contexts,
        bindings,
        diagnostics,
    })
    .map_err(|error| error.to_string())
}

#[cfg(test)]
fn task258b3m2b2b2bp_source_term_with_mutation(
    source_term: &SourceTermParts,
    binding_env: &BindingEnv,
    mutation: ImportedStructureSelectorTestMutation,
) -> Result<SourceTermParts, String> {
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
    let mut producer_binding_env = None;
    match mutation {
        ImportedStructureSelectorTestMutation::PrimarySourceId => {
            let allocator = mizar_session::InMemorySessionIdAllocator::new();
            mizar_session::SessionIdAllocator::next_source_id(
                &allocator,
                super::super::shared::snapshot_id(25_802),
            )
            .map_err(|error| error.to_string())?;
            input.source_id = mizar_session::SessionIdAllocator::next_source_id(
                &allocator,
                super::super::shared::snapshot_id(25_802),
            )
            .map_err(|error| error.to_string())?;
        }
        ImportedStructureSelectorTestMutation::PrimaryModuleId => {
            input.module_id = ModuleId::new(
                input.module_id.package().clone(),
                mizar_session::ModulePath::new("tests.b2bp.primary-module-substitute"),
            );
        }
        ImportedStructureSelectorTestMutation::PrimaryTermSite(index) => {
            input.terms[index].site =
                TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 49 } else { 47 }));
        }
        ImportedStructureSelectorTestMutation::PrimaryTermRange(index) => {
            input.terms[index].source_range.start += 1;
        }
        ImportedStructureSelectorTestMutation::PrimaryTermOrdinal(index) => {
            input.terms[index].source_ordinal += 1;
        }
        ImportedStructureSelectorTestMutation::PrimaryTermContext(index) => {
            input.terms[index].context =
                BindingContextId::new(1 - input.terms[index].context.index());
        }
        ImportedStructureSelectorTestMutation::PrimaryTermRecovery(index) => {
            input.terms[index].recovery = SourcePrimaryTermRecovery::Degraded;
        }
        ImportedStructureSelectorTestMutation::PrimaryTermSpelling(index) => {
            input.terms[index].spelling.push('0');
        }
        ImportedStructureSelectorTestMutation::PrimaryTermKind(index) => {
            input.terms[index].kind = if input.terms[index].kind == SourcePrimaryTermKind::Numeral {
                SourcePrimaryTermKind::VariableReference
            } else {
                SourcePrimaryTermKind::Numeral
            };
        }
        ImportedStructureSelectorTestMutation::PrimaryTermRole(index) => {
            input.terms[index].role = SourcePrimaryTermRole::CurrentDefinitionResult;
        }
        ImportedStructureSelectorTestMutation::PrimaryTermParent(index) => {
            input.terms[index].parent = Some(SourcePrimaryTermId::new(0));
        }
        ImportedStructureSelectorTestMutation::PrimaryReferenceTerm(index) => {
            input.references[index].term =
                SourcePrimaryTermId::new((input.references[index].term.index() + 1) % 6);
        }
        ImportedStructureSelectorTestMutation::PrimaryReferenceBinding(index) => {
            input.references[index].binding = mizar_checker::binding_env::BindingId::new(1);
        }
        ImportedStructureSelectorTestMutation::PrimaryReferenceRole(index) => {
            input.references[index].role = SourcePrimaryTermReferenceRole::LocalConstant;
        }
        ImportedStructureSelectorTestMutation::PrimaryReferenceUseOrdinal(_) => {
            producer_binding_env = Some(task258b3m2b2b2bp_binding_env_with_mutation(
                binding_env,
                ImportedStructureSelectorTestMutation::BindingCount,
            )?);
        }
        ImportedStructureSelectorTestMutation::PrimaryReferenceScope(index) => {
            producer_binding_env = Some(task258b3m2b2b2bp_binding_env_with_mutation(
                binding_env,
                if index < 2 {
                    ImportedStructureSelectorTestMutation::ModuleContextScope
                } else {
                    ImportedStructureSelectorTestMutation::ProofContextScope
                },
            )?);
        }
        ImportedStructureSelectorTestMutation::NumericRequestTerm(index) => {
            input.numeric_type_requests[index].term =
                SourcePrimaryTermId::new(3 - input.numeric_type_requests[index].term.index());
        }
        ImportedStructureSelectorTestMutation::NumericRequestOwner(index) => {
            input.numeric_type_requests[index].owner =
                TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 58 } else { 55 }));
        }
        ImportedStructureSelectorTestMutation::NumericRequestRange(index) => {
            input.numeric_type_requests[index].source_range.start += 1;
        }
        ImportedStructureSelectorTestMutation::NumericRequestSpelling(index) => {
            input.numeric_type_requests[index].spelling.push('0');
        }
        ImportedStructureSelectorTestMutation::NumericRequestOrdinal(index) => {
            input.numeric_type_requests[index].request_ordinal += 1;
        }
        _ => {}
    }
    let producer_binding_env = producer_binding_env.as_ref().unwrap_or(binding_env);
    let handoff = SourcePrimaryTermProducer::build(input, producer_binding_env, &source_term.arena)
        .map_err(|error| error.to_string())?;
    Ok(SourceTermParts {
        arena: source_term.arena.clone(),
        handoff,
    })
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)] // Rationale: keep exact source, lower rows, context, and both corruption classes explicit.
pub(in crate::runner) fn imported_structure_selector_handoff_for_test(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    loaded_source: &str,
    roots: &[(usize, BindingContextId)],
    options: ImportedStructureSelectorTestOptions,
) -> Option<Result<ImportedStructureSelectorTestOutput, String>> {
    let ImportedStructureSelectorTestOptions {
        selector,
        context,
        surface_mutation,
        handoff_mutation,
    } = options;
    let site = ImportedStructureSelectorSite { selector };
    if surface_mutation != ImportedStructureSelectorSurfaceMutation::DirectProductionSeam
        && !task258b3m2b2b2bp_surface_contract_with_mutation(ast, loaded_source, surface_mutation)
    {
        return None;
    }
    let owned_node_kinds =
        imported_structure_selector_owned_node_kinds(ast, module, symbols, loaded_source, site)?;
    let source_term = match source_term_parts_for_context_roots(
        ast,
        module.clone(),
        binding_env,
        roots.iter().copied(),
        &owned_node_kinds,
    ) {
        Ok(source_term) => source_term,
        Err(error) => return Some(Err(format!("Task252: {error}"))),
    };
    let mutated_binding_env = if task258b3m2b2b2bp_is_binding_mutation(handoff_mutation) {
        match task258b3m2b2b2bp_binding_env_with_mutation(binding_env, handoff_mutation) {
            Ok(binding_env) => Some(binding_env),
            Err(error) => return Some(Err(format!("Task48: {error}"))),
        }
    } else {
        None
    };
    let effective_binding_env = mutated_binding_env.as_ref().unwrap_or(binding_env);
    let mutated_source_term = if task258b3m2b2b2bp_is_primary_mutation(handoff_mutation) {
        match task258b3m2b2b2bp_source_term_with_mutation(
            &source_term,
            binding_env,
            handoff_mutation,
        ) {
            Ok(source_term) => Some(source_term),
            Err(error) => return Some(Err(format!("Task252: {error}"))),
        }
    } else {
        None
    };
    let effective_source_term = mutated_source_term.as_ref().unwrap_or(&source_term);
    let mut extracted =
        extract_task258b3m2b2b2bp_selector(ast, module, symbols, loaded_source, site)?;
    extracted.context = context;
    let selected_contribution = extracted.roots[0].contribution;
    let substitute_contribution = symbols
        .contributions()
        .iter()
        .map(|entry| entry.id())
        .find(|candidate| *candidate != selected_contribution)
        .unwrap_or(selected_contribution);
    let stale_replay = matches!(
        handoff_mutation,
        ImportedStructureSelectorTestMutation::StalePrimaryReplay
            | ImportedStructureSelectorTestMutation::TermRangeAndStalePrimaryReplay
    );
    let lower_dependency_mutation = task258b3m2b2b2bp_is_binding_mutation(handoff_mutation)
        || task258b3m2b2b2bp_is_primary_mutation(handoff_mutation);
    let handoff = if matches!(
        handoff_mutation,
        ImportedStructureSelectorTestMutation::None
            | ImportedStructureSelectorTestMutation::StalePrimaryReplay
    ) || lower_dependency_mutation
    {
        match imported_structure_selector_handoff_in_context(
            ast,
            module,
            symbols,
            effective_binding_env,
            effective_source_term,
            loaded_source,
            site,
            context,
        )? {
            Ok(handoff) => handoff,
            Err(error) => return Some(Err(format!("Task254: {error}"))),
        }
    } else {
        match build_handoff_with_source_term(
            ast,
            module,
            symbols,
            effective_binding_env,
            &extracted,
            effective_source_term,
            None,
            |input| match handoff_mutation {
                ImportedStructureSelectorTestMutation::None
                | ImportedStructureSelectorTestMutation::StalePrimaryReplay => {}
                ImportedStructureSelectorTestMutation::TermSite(index) => {
                    input.terms[index].site =
                        TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 61 } else { 62 }));
                }
                ImportedStructureSelectorTestMutation::TermRange(index) => {
                    input.terms[index].source_range.start += 1;
                }
                ImportedStructureSelectorTestMutation::TermRangeAndStalePrimaryReplay => {
                    input.terms[0].source_range.start += 1;
                }
                ImportedStructureSelectorTestMutation::TermOrdinal(index) => {
                    input.terms[index].source_ordinal += 1;
                }
                ImportedStructureSelectorTestMutation::TermContext(index) => {
                    input.terms[index].context = BindingContextId::new(0);
                }
                ImportedStructureSelectorTestMutation::TermRecovery(index) => {
                    input.terms[index].recovery = SourceStructureRecovery::Degraded;
                }
                ImportedStructureSelectorTestMutation::TermSpelling(index) => {
                    input.terms[index].spelling.push(' ');
                }
                ImportedStructureSelectorTestMutation::TermKind(index) => {
                    input.terms[index].kind = if index == 0 {
                        SourceStructureTermKind::Constructor
                    } else {
                        SourceStructureTermKind::SelectorAccess
                    };
                }
                ImportedStructureSelectorTestMutation::RootTerm => {
                    input.roots[0].term = SourceStructureTermId::new(0);
                }
                ImportedStructureSelectorTestMutation::RootSymbol => {
                    input.roots[0].symbol = SymbolId::new(
                        module.clone(),
                        mizar_resolve::resolved_ast::LocalSymbolId::new("B2BP/substitute"),
                        mizar_resolve::resolved_ast::FullyQualifiedName::new(
                            "b2bp::substitute::TypeCaseStruct",
                        ),
                    );
                }
                ImportedStructureSelectorTestMutation::RootContribution => {
                    input.roots[0].contribution = substitute_contribution;
                }
                ImportedStructureSelectorTestMutation::MemberTerm(index) => {
                    input.members[index].term =
                        SourceStructureTermId::new(1 - input.members[index].term.index());
                }
                ImportedStructureSelectorTestMutation::MemberOrdinal(index) => {
                    input.members[index].ordinal += 1;
                }
                ImportedStructureSelectorTestMutation::MemberSite(index) => {
                    input.members[index].site =
                        TypedSiteRef::Node(TypedNodeId::new([20, 24, 29][index]));
                }
                ImportedStructureSelectorTestMutation::MemberRange(index) => {
                    input.members[index].source_range.start += 1;
                }
                ImportedStructureSelectorTestMutation::MemberSpelling(index) => {
                    input.members[index].spelling.push('!');
                }
                ImportedStructureSelectorTestMutation::MemberRole(index) => {
                    input.members[index].role = if index == 0 {
                        SourceStructureMemberRole::ConstructorAssignment
                    } else {
                        SourceStructureMemberRole::Selector
                    };
                }
                ImportedStructureSelectorTestMutation::MemberParent(index) => {
                    input.members[index].parent =
                        Some(SourceStructureMemberId::new((index + 1) % 3));
                }
                ImportedStructureSelectorTestMutation::FieldUpdateExtra => {
                    input.field_updates.push(SourceFieldUpdateInput {
                        term: SourceStructureTermId::new(0),
                        ordinal: 0,
                        site: TypedSiteRef::Node(TypedNodeId::new(57)),
                        source_range: ast.nodes()[57].range,
                        spelling: "x : 1".to_owned(),
                        first_member: SourceStructureMemberId::new(1),
                        final_member: SourceStructureMemberId::new(1),
                    });
                }
                ImportedStructureSelectorTestMutation::EdgeTerm(index) => {
                    input.edges[index].term =
                        SourceStructureTermId::new(1 - input.edges[index].term.index());
                }
                ImportedStructureSelectorTestMutation::EdgeOrdinal(index) => {
                    input.edges[index].ordinal += 1;
                }
                ImportedStructureSelectorTestMutation::EdgeRole(index) => {
                    input.edges[index].role = SourceStructureEdgeRole::UpdateBase;
                }
                ImportedStructureSelectorTestMutation::EdgeMember(index) => {
                    input.edges[index].member = if input.edges[index].member.is_some() {
                        None
                    } else {
                        Some(SourceStructureMemberId::new(0))
                    };
                }
                ImportedStructureSelectorTestMutation::EdgeTarget(index) => {
                    input.edges[index].target = if index == 0 {
                        SourceStructureTarget::Primary(
                            mizar_checker::source_term::SourcePrimaryTermId::new(4),
                        )
                    } else {
                        SourceStructureTarget::Structure(SourceStructureTermId::new(0))
                    };
                }
                ImportedStructureSelectorTestMutation::RequestTerm(index) => {
                    input.requests[index].term =
                        SourceStructureTermId::new(1 - input.requests[index].term.index());
                }
                ImportedStructureSelectorTestMutation::RequestOrdinal(index) => {
                    input.requests[index].request_ordinal += 1;
                }
                ImportedStructureSelectorTestMutation::RequestMember(index) => {
                    input.requests[index].member = if input.requests[index].member.is_some() {
                        None
                    } else {
                        Some(SourceStructureMemberId::new(0))
                    };
                }
                ImportedStructureSelectorTestMutation::RequestKind(index) => {
                    input.requests[index].kind =
                        if input.requests[index].kind == SourceStructureRequestKind::ResultType {
                            SourceStructureRequestKind::MemberIdentity
                        } else {
                            SourceStructureRequestKind::ResultType
                        };
                }
                _ => unreachable!("lower dependency mutation uses the production seam"),
            },
        ) {
            Ok(handoff) => handoff,
            Err(error) => return Some(Err(format!("Task254: {error}"))),
        }
    };

    if stale_replay {
        let stale_source_term = match source_term_parts_for_context_roots(
            ast,
            module.clone(),
            binding_env,
            roots.iter().copied().filter(|(root, _)| *root != 59),
            &owned_node_kinds,
        ) {
            Ok(source_term) => source_term,
            Err(error) => return Some(Err(format!("Task252: {error}"))),
        };
        let stale_typed = match task258b3m2b2b2p_typed_ast(ast, module, stale_source_term) {
            Ok(typed) => typed,
            Err(error) => return Some(Err(format!("TypedAst: {error}"))),
        };
        return Some(match stale_typed.with_source_structure(handoff) {
            Ok(_) => Err("BUG: TypedAst accepted stale Task252 fingerprint".to_owned()),
            Err(error) => Err(format!(
                "TypedAst: rejected stale Task252 fingerprint: {error}"
            )),
        });
    }

    let primary_counts = (
        effective_source_term.handoff.terms().len(),
        effective_source_term.handoff.references().len(),
        effective_source_term.handoff.numeric_type_requests().len(),
    );
    let typed_ast =
        match task258b3m2b2b2p_typed_ast(ast, module, mutated_source_term.unwrap_or(source_term)) {
            Ok(typed) => match typed.with_source_structure(handoff.clone()) {
                Ok(typed) => typed,
                Err(error) => return Some(Err(format!("TypedAst: {error}"))),
            },
            Err(error) => return Some(Err(format!("TypedAst: {error}"))),
        };
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
    let resolved = match assemble_empty_resolved_typed_ast(&typed_ast, node_hints) {
        Ok(resolved) => resolved,
        Err(error) => return Some(Err(format!("ResolvedTypedAst: {error}"))),
    };
    Some(Ok(ImportedStructureSelectorTestOutput {
        handoff,
        primary_counts,
        typed_ast,
        resolved,
    }))
}

#[cfg(test)]
fn task258b3m2b2b2cp_binding_mutation(
    mutation: ImportedStructureUpdateTestMutation,
) -> Option<ImportedStructureSelectorTestMutation> {
    Some(match mutation {
        ImportedStructureUpdateTestMutation::BindingSourceId => {
            ImportedStructureSelectorTestMutation::BindingSourceId
        }
        ImportedStructureUpdateTestMutation::BindingModuleId => {
            ImportedStructureSelectorTestMutation::BindingModuleId
        }
        ImportedStructureUpdateTestMutation::BindingContextCount => {
            ImportedStructureSelectorTestMutation::BindingContextCount
        }
        ImportedStructureUpdateTestMutation::BindingCount => {
            ImportedStructureSelectorTestMutation::BindingCount
        }
        ImportedStructureUpdateTestMutation::BindingDiagnosticCount => {
            ImportedStructureSelectorTestMutation::BindingDiagnosticCount
        }
        ImportedStructureUpdateTestMutation::ModuleContextOwner => {
            ImportedStructureSelectorTestMutation::ModuleContextOwner
        }
        ImportedStructureUpdateTestMutation::ModuleContextParent => {
            ImportedStructureSelectorTestMutation::ModuleContextParent
        }
        ImportedStructureUpdateTestMutation::ModuleContextLayer => {
            ImportedStructureSelectorTestMutation::ModuleContextLayer
        }
        ImportedStructureUpdateTestMutation::ModuleContextScope => {
            ImportedStructureSelectorTestMutation::ModuleContextScope
        }
        ImportedStructureUpdateTestMutation::ModuleContextBindings => {
            ImportedStructureSelectorTestMutation::ModuleContextBindings
        }
        ImportedStructureUpdateTestMutation::ModuleContextVisibleBindings => {
            ImportedStructureSelectorTestMutation::ModuleContextVisibleBindings
        }
        ImportedStructureUpdateTestMutation::ModuleContextRecovery => {
            ImportedStructureSelectorTestMutation::ModuleContextRecovery
        }
        ImportedStructureUpdateTestMutation::ProofContextOwner => {
            ImportedStructureSelectorTestMutation::ProofContextOwner
        }
        ImportedStructureUpdateTestMutation::ProofContextParent => {
            ImportedStructureSelectorTestMutation::ProofContextParent
        }
        ImportedStructureUpdateTestMutation::ProofContextLayer => {
            ImportedStructureSelectorTestMutation::ProofContextLayer
        }
        ImportedStructureUpdateTestMutation::ProofContextScope => {
            ImportedStructureSelectorTestMutation::ProofContextScope
        }
        ImportedStructureUpdateTestMutation::ProofContextBindings => {
            ImportedStructureSelectorTestMutation::ProofContextBindings
        }
        ImportedStructureUpdateTestMutation::ProofContextVisibleBindings => {
            ImportedStructureSelectorTestMutation::ProofContextVisibleBindings
        }
        ImportedStructureUpdateTestMutation::ProofContextRecovery => {
            ImportedStructureSelectorTestMutation::ProofContextRecovery
        }
        ImportedStructureUpdateTestMutation::BindingSpelling => {
            ImportedStructureSelectorTestMutation::BindingSpelling
        }
        ImportedStructureUpdateTestMutation::BindingKind => {
            ImportedStructureSelectorTestMutation::BindingKind
        }
        ImportedStructureUpdateTestMutation::BindingIdentityKind => {
            ImportedStructureSelectorTestMutation::BindingIdentityKind
        }
        ImportedStructureUpdateTestMutation::BindingIdentitySpelling => {
            ImportedStructureSelectorTestMutation::BindingIdentitySpelling
        }
        ImportedStructureUpdateTestMutation::BindingIdentityRange => {
            ImportedStructureSelectorTestMutation::BindingIdentityRange
        }
        ImportedStructureUpdateTestMutation::BindingOwner => {
            ImportedStructureSelectorTestMutation::BindingOwner
        }
        ImportedStructureUpdateTestMutation::BindingDeclarationRange => {
            ImportedStructureSelectorTestMutation::BindingDeclarationRange
        }
        ImportedStructureUpdateTestMutation::BindingVisibleOrdinal => {
            ImportedStructureSelectorTestMutation::BindingVisibleOrdinal
        }
        ImportedStructureUpdateTestMutation::BindingTypeSite => {
            ImportedStructureSelectorTestMutation::BindingTypeSite
        }
        ImportedStructureUpdateTestMutation::BindingStatus => {
            ImportedStructureSelectorTestMutation::BindingStatus
        }
        ImportedStructureUpdateTestMutation::BindingCaptured => {
            ImportedStructureSelectorTestMutation::BindingCaptured
        }
        ImportedStructureUpdateTestMutation::BindingDiagnostics => {
            ImportedStructureSelectorTestMutation::BindingDiagnostics
        }
        ImportedStructureUpdateTestMutation::BindingRecovery => {
            ImportedStructureSelectorTestMutation::BindingRecovery
        }
        _ => return None,
    })
}

#[cfg(test)]
fn task258b3m2b2b2cp_is_primary_mutation(mutation: ImportedStructureUpdateTestMutation) -> bool {
    matches!(
        mutation,
        ImportedStructureUpdateTestMutation::PrimarySourceId
            | ImportedStructureUpdateTestMutation::PrimaryModuleId
            | ImportedStructureUpdateTestMutation::PrimaryTermSite(_)
            | ImportedStructureUpdateTestMutation::PrimaryTermRange(_)
            | ImportedStructureUpdateTestMutation::PrimaryTermOrdinal(_)
            | ImportedStructureUpdateTestMutation::PrimaryTermContext(_)
            | ImportedStructureUpdateTestMutation::PrimaryTermRecovery(_)
            | ImportedStructureUpdateTestMutation::PrimaryTermSpelling(_)
            | ImportedStructureUpdateTestMutation::PrimaryTermKind(_)
            | ImportedStructureUpdateTestMutation::PrimaryTermRole(_)
            | ImportedStructureUpdateTestMutation::PrimaryTermParent(_)
            | ImportedStructureUpdateTestMutation::PrimaryReferenceTerm(_)
            | ImportedStructureUpdateTestMutation::PrimaryReferenceBinding(_)
            | ImportedStructureUpdateTestMutation::PrimaryReferenceRole(_)
            | ImportedStructureUpdateTestMutation::PrimaryReferenceUseOrdinal(_)
            | ImportedStructureUpdateTestMutation::PrimaryReferenceScope(_)
            | ImportedStructureUpdateTestMutation::NumericRequestTerm(_)
            | ImportedStructureUpdateTestMutation::NumericRequestOwner(_)
            | ImportedStructureUpdateTestMutation::NumericRequestRange(_)
            | ImportedStructureUpdateTestMutation::NumericRequestSpelling(_)
            | ImportedStructureUpdateTestMutation::NumericRequestOrdinal(_)
    )
}

#[cfg(test)]
fn task258b3m2b2b2cp_source_term_with_mutation(
    source_term: &SourceTermParts,
    binding_env: &BindingEnv,
    mutation: ImportedStructureUpdateTestMutation,
) -> Result<SourceTermParts, String> {
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
    let mut producer_binding_env = None;
    match mutation {
        ImportedStructureUpdateTestMutation::PrimarySourceId => {
            let allocator = mizar_session::InMemorySessionIdAllocator::new();
            mizar_session::SessionIdAllocator::next_source_id(
                &allocator,
                super::super::shared::snapshot_id(25_803),
            )
            .map_err(|error| error.to_string())?;
            input.source_id = mizar_session::SessionIdAllocator::next_source_id(
                &allocator,
                super::super::shared::snapshot_id(25_803),
            )
            .map_err(|error| error.to_string())?;
        }
        ImportedStructureUpdateTestMutation::PrimaryModuleId => {
            input.module_id = ModuleId::new(
                input.module_id.package().clone(),
                mizar_session::ModulePath::new("tests.b2cp.primary-module-substitute"),
            );
        }
        ImportedStructureUpdateTestMutation::PrimaryTermSite(index) => {
            let replacement = input.terms[(index + 1) % input.terms.len()].site.clone();
            input.terms[index].site = replacement;
        }
        ImportedStructureUpdateTestMutation::PrimaryTermRange(index) => {
            input.terms[index].source_range.start += 1;
        }
        ImportedStructureUpdateTestMutation::PrimaryTermOrdinal(index) => {
            input.terms[index].source_ordinal += 1;
        }
        ImportedStructureUpdateTestMutation::PrimaryTermContext(index) => {
            input.terms[index].context =
                BindingContextId::new(1 - input.terms[index].context.index());
        }
        ImportedStructureUpdateTestMutation::PrimaryTermRecovery(index) => {
            input.terms[index].recovery = SourcePrimaryTermRecovery::Degraded;
        }
        ImportedStructureUpdateTestMutation::PrimaryTermSpelling(index) => {
            input.terms[index].spelling.push('0');
        }
        ImportedStructureUpdateTestMutation::PrimaryTermKind(index) => {
            input.terms[index].kind = if input.terms[index].kind == SourcePrimaryTermKind::Numeral {
                SourcePrimaryTermKind::VariableReference
            } else {
                SourcePrimaryTermKind::Numeral
            };
        }
        ImportedStructureUpdateTestMutation::PrimaryTermRole(index) => {
            input.terms[index].role = SourcePrimaryTermRole::CurrentDefinitionResult;
        }
        ImportedStructureUpdateTestMutation::PrimaryTermParent(index) => {
            input.terms[index].parent = Some(SourcePrimaryTermId::new(0));
        }
        ImportedStructureUpdateTestMutation::PrimaryReferenceTerm(index) => {
            input.references[index].term = SourcePrimaryTermId::new(
                (input.references[index].term.index() + 1) % input.terms.len(),
            );
        }
        ImportedStructureUpdateTestMutation::PrimaryReferenceBinding(index) => {
            input.references[index].binding = mizar_checker::binding_env::BindingId::new(1);
        }
        ImportedStructureUpdateTestMutation::PrimaryReferenceRole(index) => {
            input.references[index].role = SourcePrimaryTermReferenceRole::LocalConstant;
        }
        ImportedStructureUpdateTestMutation::PrimaryReferenceUseOrdinal(_) => {
            producer_binding_env = Some(task258b3m2b2b2bp_binding_env_with_mutation(
                binding_env,
                ImportedStructureSelectorTestMutation::BindingCount,
            )?);
        }
        ImportedStructureUpdateTestMutation::PrimaryReferenceScope(index) => {
            producer_binding_env = Some(task258b3m2b2b2bp_binding_env_with_mutation(
                binding_env,
                if index < 2 {
                    ImportedStructureSelectorTestMutation::ModuleContextScope
                } else {
                    ImportedStructureSelectorTestMutation::ProofContextScope
                },
            )?);
        }
        ImportedStructureUpdateTestMutation::NumericRequestTerm(index) => {
            input.numeric_type_requests[index].term = SourcePrimaryTermId::new(
                (input.numeric_type_requests[index].term.index() + 1) % input.terms.len(),
            );
        }
        ImportedStructureUpdateTestMutation::NumericRequestOwner(index) => {
            let replacement = input.numeric_type_requests
                [(index + 1) % input.numeric_type_requests.len()]
            .owner
            .clone();
            input.numeric_type_requests[index].owner = replacement;
        }
        ImportedStructureUpdateTestMutation::NumericRequestRange(index) => {
            input.numeric_type_requests[index].source_range.start += 1;
        }
        ImportedStructureUpdateTestMutation::NumericRequestSpelling(index) => {
            input.numeric_type_requests[index].spelling.push('0');
        }
        ImportedStructureUpdateTestMutation::NumericRequestOrdinal(index) => {
            input.numeric_type_requests[index].request_ordinal += 1;
        }
        _ => {}
    }
    let producer_binding_env = producer_binding_env.as_ref().unwrap_or(binding_env);
    let handoff = SourcePrimaryTermProducer::build(input, producer_binding_env, &source_term.arena)
        .map_err(|error| error.to_string())?;
    Ok(SourceTermParts {
        arena: source_term.arena.clone(),
        handoff,
    })
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)] // Rationale: keep exact source, lower rows, context, and both corruption classes explicit.
pub(in crate::runner) fn imported_structure_update_handoff_for_test(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    loaded_source: &str,
    roots: &[(usize, BindingContextId)],
    options: ImportedStructureUpdateTestOptions,
) -> Option<Result<ImportedStructureUpdateTestOutput, String>> {
    let ImportedStructureUpdateTestOptions {
        update,
        context,
        surface_mutation,
        handoff_mutation,
    } = options;
    let site = ImportedStructureUpdateSite { update };
    if surface_mutation != ImportedStructureUpdateSurfaceMutation::DirectProductionSeam
        && !task258b3m2b2b2cp_surface_contract_with_mutation(ast, loaded_source, surface_mutation)
    {
        return None;
    }
    let owned_node_kinds =
        imported_structure_update_owned_node_kinds(ast, module, symbols, loaded_source, site)?;
    let source_term = match source_term_parts_for_context_roots(
        ast,
        module.clone(),
        binding_env,
        roots.iter().copied(),
        &owned_node_kinds,
    ) {
        Ok(source_term) => source_term,
        Err(error) => return Some(Err(format!("Task252: {error}"))),
    };
    let binding_mutation = task258b3m2b2b2cp_binding_mutation(handoff_mutation);
    let mutated_binding_env = if let Some(binding_mutation) = binding_mutation {
        match task258b3m2b2b2bp_binding_env_with_mutation(binding_env, binding_mutation) {
            Ok(binding_env) => Some(binding_env),
            Err(error) => return Some(Err(format!("Task48: {error}"))),
        }
    } else {
        None
    };
    let effective_binding_env = mutated_binding_env.as_ref().unwrap_or(binding_env);
    let mutated_source_term = if task258b3m2b2b2cp_is_primary_mutation(handoff_mutation) {
        match task258b3m2b2b2cp_source_term_with_mutation(
            &source_term,
            binding_env,
            handoff_mutation,
        ) {
            Ok(source_term) => Some(source_term),
            Err(error) => return Some(Err(format!("Task252: {error}"))),
        }
    } else {
        None
    };
    let effective_source_term = mutated_source_term.as_ref().unwrap_or(&source_term);
    let mut extracted =
        extract_task258b3m2b2b2cp_update(ast, module, symbols, loaded_source, site)?;
    extracted.context = context;
    let selected_contribution = extracted.roots[0].contribution;
    let substitute_contribution = symbols
        .contributions()
        .iter()
        .map(|entry| entry.id())
        .find(|candidate| *candidate != selected_contribution)
        .unwrap_or(selected_contribution);
    let stale_replay = matches!(
        handoff_mutation,
        ImportedStructureUpdateTestMutation::StalePrimaryReplay
            | ImportedStructureUpdateTestMutation::TermRangeAndStalePrimaryReplay
    );
    let lower_dependency_mutation =
        binding_mutation.is_some() || task258b3m2b2b2cp_is_primary_mutation(handoff_mutation);
    let handoff = if matches!(
        handoff_mutation,
        ImportedStructureUpdateTestMutation::None
            | ImportedStructureUpdateTestMutation::StalePrimaryReplay
    ) || lower_dependency_mutation
    {
        match imported_structure_update_handoff_in_context(
            ast,
            module,
            symbols,
            effective_binding_env,
            effective_source_term,
            loaded_source,
            site,
            context,
        )? {
            Ok(handoff) => handoff,
            Err(error) => return Some(Err(format!("Task254: {error}"))),
        }
    } else {
        match build_handoff_with_source_term(
            ast,
            module,
            symbols,
            effective_binding_env,
            &extracted,
            effective_source_term,
            None,
            |input| match handoff_mutation {
                ImportedStructureUpdateTestMutation::None
                | ImportedStructureUpdateTestMutation::StalePrimaryReplay => {}
                ImportedStructureUpdateTestMutation::TermSite(index) => {
                    input.terms[index].site =
                        TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 65 } else { 69 }));
                }
                ImportedStructureUpdateTestMutation::TermRange(index) => {
                    input.terms[index].source_range.start += 1;
                }
                ImportedStructureUpdateTestMutation::TermRangeAndStalePrimaryReplay => {
                    input.terms[0].source_range.start += 1;
                }
                ImportedStructureUpdateTestMutation::TermOrdinal(index) => {
                    input.terms[index].source_ordinal += 1;
                }
                ImportedStructureUpdateTestMutation::TermContext(index) => {
                    input.terms[index].context = BindingContextId::new(0);
                }
                ImportedStructureUpdateTestMutation::TermRecovery(index) => {
                    input.terms[index].recovery = SourceStructureRecovery::Degraded;
                }
                ImportedStructureUpdateTestMutation::TermSpelling(index) => {
                    input.terms[index].spelling.push(' ');
                }
                ImportedStructureUpdateTestMutation::TermKind(index) => {
                    input.terms[index].kind = if index == 0 {
                        SourceStructureTermKind::Constructor
                    } else {
                        SourceStructureTermKind::FunctionalUpdate
                    };
                }
                ImportedStructureUpdateTestMutation::RootTerm => {
                    input.roots[0].term = SourceStructureTermId::new(0);
                }
                ImportedStructureUpdateTestMutation::RootSymbol => {
                    input.roots[0].symbol = SymbolId::new(
                        module.clone(),
                        mizar_resolve::resolved_ast::LocalSymbolId::new("B2CP/substitute"),
                        mizar_resolve::resolved_ast::FullyQualifiedName::new(
                            "b2cp::substitute::TypeCaseStruct",
                        ),
                    );
                }
                ImportedStructureUpdateTestMutation::RootContribution => {
                    input.roots[0].contribution = substitute_contribution;
                }
                ImportedStructureUpdateTestMutation::MemberTerm(index) => {
                    input.members[index].term =
                        SourceStructureTermId::new(1 - input.members[index].term.index());
                }
                ImportedStructureUpdateTestMutation::MemberOrdinal(index) => {
                    input.members[index].ordinal += 1;
                }
                ImportedStructureUpdateTestMutation::MemberSite(index) => {
                    input.members[index].site =
                        TypedSiteRef::Node(TypedNodeId::new([20, 24, 30][index]));
                }
                ImportedStructureUpdateTestMutation::MemberRange(index) => {
                    input.members[index].source_range.start += 1;
                }
                ImportedStructureUpdateTestMutation::MemberSpelling(index) => {
                    input.members[index].spelling.push('!');
                }
                ImportedStructureUpdateTestMutation::MemberRole(index) => {
                    input.members[index].role = if index == 0 {
                        SourceStructureMemberRole::ConstructorAssignment
                    } else {
                        SourceStructureMemberRole::UpdatePathSegment
                    };
                }
                ImportedStructureUpdateTestMutation::MemberParent(index) => {
                    input.members[index].parent =
                        Some(SourceStructureMemberId::new((index + 1) % 3));
                }
                ImportedStructureUpdateTestMutation::FieldUpdateMissing => {
                    input.field_updates.clear();
                }
                ImportedStructureUpdateTestMutation::FieldUpdateExtra => {
                    let mut extra = input.field_updates[0].clone();
                    extra.ordinal += 1;
                    input.field_updates.push(extra);
                }
                ImportedStructureUpdateTestMutation::FieldUpdateTerm(index) => {
                    input.field_updates[index].term = SourceStructureTermId::new(1);
                }
                ImportedStructureUpdateTestMutation::FieldUpdateOrdinal(index) => {
                    input.field_updates[index].ordinal += 1;
                }
                ImportedStructureUpdateTestMutation::FieldUpdateSite(index) => {
                    input.field_updates[index].site = TypedSiteRef::Node(TypedNodeId::new(65));
                }
                ImportedStructureUpdateTestMutation::FieldUpdateRange(index) => {
                    input.field_updates[index].source_range.start += 1;
                }
                ImportedStructureUpdateTestMutation::FieldUpdateSpelling(index) => {
                    input.field_updates[index].spelling.push(' ');
                }
                ImportedStructureUpdateTestMutation::FieldUpdateFirstMember(index) => {
                    input.field_updates[index].first_member = SourceStructureMemberId::new(1);
                }
                ImportedStructureUpdateTestMutation::FieldUpdateFinalMember(index) => {
                    input.field_updates[index].final_member = SourceStructureMemberId::new(1);
                }
                ImportedStructureUpdateTestMutation::EdgeTerm(index) => {
                    input.edges[index].term =
                        SourceStructureTermId::new(1 - input.edges[index].term.index());
                }
                ImportedStructureUpdateTestMutation::EdgeOrdinal(index) => {
                    input.edges[index].ordinal += 1;
                }
                ImportedStructureUpdateTestMutation::EdgeRole(index) => {
                    input.edges[index].role =
                        if input.edges[index].role == SourceStructureEdgeRole::UpdateBase {
                            SourceStructureEdgeRole::ConstructorValue
                        } else {
                            SourceStructureEdgeRole::UpdateBase
                        };
                }
                ImportedStructureUpdateTestMutation::EdgeMember(index) => {
                    input.edges[index].member = if input.edges[index].member.is_some() {
                        None
                    } else {
                        Some(SourceStructureMemberId::new(0))
                    };
                }
                ImportedStructureUpdateTestMutation::EdgeTarget(index) => {
                    input.edges[index].target = if index == 0 {
                        SourceStructureTarget::Primary(SourcePrimaryTermId::new(4))
                    } else {
                        SourceStructureTarget::Structure(SourceStructureTermId::new(0))
                    };
                }
                ImportedStructureUpdateTestMutation::RequestTerm(index) => {
                    input.requests[index].term =
                        SourceStructureTermId::new(1 - input.requests[index].term.index());
                }
                ImportedStructureUpdateTestMutation::RequestOrdinal(index) => {
                    input.requests[index].request_ordinal += 1;
                }
                ImportedStructureUpdateTestMutation::RequestMember(index) => {
                    input.requests[index].member = if input.requests[index].member.is_some() {
                        None
                    } else {
                        Some(SourceStructureMemberId::new(0))
                    };
                }
                ImportedStructureUpdateTestMutation::RequestKind(index) => {
                    input.requests[index].kind =
                        if input.requests[index].kind == SourceStructureRequestKind::ResultType {
                            SourceStructureRequestKind::MemberIdentity
                        } else {
                            SourceStructureRequestKind::ResultType
                        };
                }
                _ => unreachable!("lower dependency mutation uses the production seam"),
            },
        ) {
            Ok(handoff) => handoff,
            Err(error) => return Some(Err(format!("Task254: {error}"))),
        }
    };

    if stale_replay {
        let stale_source_term = match source_term_parts_for_context_roots(
            ast,
            module.clone(),
            binding_env,
            roots.iter().copied().filter(|(root, _)| *root != 67),
            &owned_node_kinds,
        ) {
            Ok(source_term) => source_term,
            Err(error) => return Some(Err(format!("Task252: {error}"))),
        };
        let stale_typed = match task258b3m2b2b2p_typed_ast(ast, module, stale_source_term) {
            Ok(typed) => typed,
            Err(error) => return Some(Err(format!("TypedAst: {error}"))),
        };
        return Some(match stale_typed.with_source_structure(handoff) {
            Ok(_) => Err("BUG: TypedAst accepted stale Task252 fingerprint".to_owned()),
            Err(error) => Err(format!(
                "TypedAst: rejected stale Task252 fingerprint: {error}"
            )),
        });
    }

    let primary_counts = (
        effective_source_term.handoff.terms().len(),
        effective_source_term.handoff.references().len(),
        effective_source_term.handoff.numeric_type_requests().len(),
    );
    let typed_ast =
        match task258b3m2b2b2p_typed_ast(ast, module, mutated_source_term.unwrap_or(source_term)) {
            Ok(typed) => match typed.with_source_structure(handoff.clone()) {
                Ok(typed) => typed,
                Err(error) => return Some(Err(format!("TypedAst: {error}"))),
            },
            Err(error) => return Some(Err(format!("TypedAst: {error}"))),
        };
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
    let resolved = match assemble_empty_resolved_typed_ast(&typed_ast, node_hints) {
        Ok(resolved) => resolved,
        Err(error) => return Some(Err(format!("ResolvedTypedAst: {error}"))),
    };
    Some(Ok(ImportedStructureUpdateTestOutput {
        handoff,
        primary_counts,
        typed_ast,
        resolved,
    }))
}

#[cfg(test)]
fn task258b3m2b2b2p_typed_ast(
    ast: &SurfaceAst,
    module: &ModuleId,
    source_term: SourceTermParts,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module.clone(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: source_term.arena,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())?
    .with_source_term(source_term.handoff)
    .map_err(|error| error.to_string())
}

fn arena_with_overrides(
    ast: &SurfaceAst,
    source: &TypedArena,
    kinds: &BTreeMap<usize, &'static str>,
    recoveries: &BTreeMap<usize, NodeRecoveryState>,
) -> Result<TypedArena, String> {
    if source.len() != ast.nodes().len() {
        return Err("source-structure dependency arena is not surface-indexed".to_owned());
    }
    let mut nodes = source
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for (index, kind) in kinds {
        let node = nodes
            .get_mut(*index)
            .ok_or_else(|| "source-structure arena override site disappeared".to_owned())?;
        if node.kind.as_str() != "source.surface.unowned" {
            return Err("source-structure arena override site is already owned".to_owned());
        }
        node.kind = (*kind).into();
    }
    for (index, recovery) in recoveries {
        nodes
            .get_mut(*index)
            .ok_or_else(|| "source-structure recovery override site disappeared".to_owned())?
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
        return Err("source-structure arena site is multiply owned".to_owned());
    }
    Ok(())
}

fn term_kind_key(kind: SourceStructureTermKind) -> &'static str {
    match kind {
        SourceStructureTermKind::Constructor => "source.term.structure.constructor",
        SourceStructureTermKind::SelectorAccess => "source.term.structure.selector",
        SourceStructureTermKind::FunctionalUpdate => "source.term.structure.update",
        _ => "source.term.structure.unsupported",
    }
}

fn member_kind_key(role: SourceStructureMemberRole) -> &'static str {
    match role {
        SourceStructureMemberRole::ConstructorAssignment => {
            "source.term.structure.member.constructor-assignment"
        }
        SourceStructureMemberRole::Selector => "source.term.structure.member.selector",
        SourceStructureMemberRole::UpdatePathSegment => {
            "source.term.structure.member.update-path-segment"
        }
        _ => "source.term.structure.member.unsupported",
    }
}

fn source_structure_binding_env(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Result<BindingEnv, String> {
    let items = exact_compilation_item_list(ast)
        .ok_or_else(|| "Task254 compilation item list disappeared".to_owned())?;
    let item_ids = structural_child_ids(ast, items);
    let [reserve_id, definition_id] = item_ids.as_slice() else {
        return Err("Task254 requires one reserve and one definition block".to_owned());
    };
    let reserve = ast
        .node(*reserve_id)
        .ok_or_else(|| "Task254 reserve disappeared".to_owned())?;
    let definition = ast
        .node(*definition_id)
        .ok_or_else(|| "Task254 definition block disappeared".to_owned())?;
    let reserve_shell = unique_shell(shells, *reserve_id, DeclarationShellKind::Reserve)?;
    let definition_shell = unique_shell(
        shells,
        *definition_id,
        DeclarationShellKind::DefinitionBlock,
    )?;
    let reserve_payload =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .map_err(|()| "Task254 reserve extraction failed".to_owned())?;
    let [reserve_binding] = reserve_payload.bridge.bindings() else {
        return Err("Task254 requires one reserve binding".to_owned());
    };
    let reserve_children = structural_child_ids(ast, reserve);
    let [reserve_segment_id] = reserve_children.as_slice() else {
        return Err("Task254 reserve requires one segment".to_owned());
    };
    let parameter_id = structural_child_ids(ast, definition)
        .into_iter()
        .find(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::DefinitionParameter))
        })
        .ok_or_else(|| "Task254 definition parameter disappeared".to_owned())?;
    let parameter = ast
        .node(parameter_id)
        .ok_or_else(|| "Task254 definition parameter disappeared".to_owned())?;
    let parameter_children = structural_child_ids(ast, parameter);
    let [segment_id] = parameter_children.as_slice() else {
        return Err("Task254 definition parameter requires one segment".to_owned());
    };
    let segment = ast
        .node(*segment_id)
        .ok_or_else(|| "Task254 definition parameter segment disappeared".to_owned())?;
    let segment_children = structural_child_ids(ast, segment);
    let [type_id] = segment_children.as_slice() else {
        return Err("Task254 definition parameter requires one written type".to_owned());
    };
    let written_type = ast
        .node(*type_id)
        .ok_or_else(|| "Task254 definition parameter type disappeared".to_owned())?;
    if !matches!(segment.kind, SurfaceNodeKind::QualifiedVariableSegment)
        || direct_token_texts(ast, segment).as_slice() != ["seed", "be"]
        || !matches!(written_type.kind, SurfaceNodeKind::TypeExpression)
        || subtree_tokens(ast, written_type) != ["set"]
    {
        return Err("Task254 definition parameter shape drift".to_owned());
    }
    let declaration_range = unique_direct_token_range(ast, segment, "seed")?;
    let root_id = ast
        .root()
        .ok_or_else(|| "Task254 root disappeared".to_owned())?;
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
                return Err("Task254 binding projection cardinality mismatch".to_owned());
            }
            Ok(handoff.binding_env().clone())
        }
        SourceBindingContextBuild::Incomplete(_) => {
            Err("Task254 binding projection remained incomplete".to_owned())
        }
        _ => Err("Task254 binding projection returned an unsupported state".to_owned()),
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
        _ => Err(format!("Task254 requires one normal {kind:?} shell")),
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
        return Err(format!("Task254 requires one direct `{spelling}` token"));
    };
    Ok(*range)
}

fn is_structure_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::StructureConstructor
            | SurfaceNodeKind::SelectorAccess
            | SurfaceNodeKind::StructureUpdate
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

fn has_structure_ancestor(ast: &SurfaceAst, parents: &[Option<usize>], node: usize) -> bool {
    let mut cursor = parents[node];
    while let Some(parent) = cursor {
        if is_structure_kind(&ast.nodes()[parent].kind) {
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
