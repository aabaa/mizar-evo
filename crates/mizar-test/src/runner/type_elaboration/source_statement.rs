use std::collections::BTreeMap;

#[cfg(test)]
use mizar_checker::{
    binding_env::{
        BinderIdentity, BindingContextTable, BindingDiagnosticClass, BindingDiagnosticDraft,
        BindingDiagnosticId, BindingDiagnosticRecovery, BindingDiagnosticSeverity,
        BindingDiagnosticTable, BindingDraft, BindingKind, BindingRecoveryState, BindingStatus,
        BindingTable, BindingTypeSite, CapturedFreeVariables,
    },
    source_set_term::{
        SourceSetConditionInput, SourceSetEdgeInput, SourceSetEdgeRole, SourceSetGeneratorId,
        SourceSetGeneratorInput, SourceSetRequestInput, SourceSetRequestKind, SourceSetTarget,
        SourceSetTermHandoffInput, SourceSetTermInput, SourceSetTermKind, SourceSetTermProducer,
        SourceSetTermRecovery, SourceSetTypeHead, SourceSetTypeOwner, SourceSetTypeRole,
        SourceSetTypeSiteId, SourceSetTypeSiteInput, SourceSetWrapperInput,
    },
    source_term::{
        SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermInput,
        SourcePrimaryTermKind, SourcePrimaryTermProducer, SourcePrimaryTermRecovery,
        SourcePrimaryTermReferenceInput, SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
    },
};
use mizar_checker::{
    binding_env::{
        BindingContextDraft, BindingContextId, BindingContextLayer, BindingContextOwner,
        BindingContextRecovery, BindingEnv, BindingEnvParts, BindingId,
    },
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_application::{SourceFunctorApplicationHandoff, SourceFunctorApplicationId},
    source_atomic_formula::{
        SourceAssertionAttributeInput, SourceAssertionAttributePolarityInput,
        SourceAssertionTypeHead, SourceAssertionTypeSiteInput, SourceAtomicEdgeId,
        SourceAtomicEdgeInput, SourceAtomicEdgeRole, SourceAtomicFormulaHandoff,
        SourceAtomicFormulaHandoffInput, SourceAtomicFormulaId, SourceAtomicFormulaInput,
        SourceAtomicFormulaKind, SourceAtomicFormulaProducer, SourceAtomicFormulaRecovery,
        SourceAtomicRequestInput, SourceAtomicRequestKind, SourceAtomicTermTarget,
        SourceAtomicWrapperInput, SourcePredicateCandidateInput, SourcePredicateHeadId,
        SourcePredicateHeadInput, SourcePredicateSegmentInput, SourcePredicateSegmentPolarityInput,
    },
    source_composite_formula::{SourceCompositeFormulaHandoff, SourceCompositeFormulaId},
    source_formula_composition::SourceFormulaCompositionHandoff,
    source_set_term::{SourceSetTermHandoff, SourceSetTermId},
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
    source_structure::{SourceStructureHandoff, SourceStructureTermId},
    source_term::{SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermReferenceId},
    type_checker::{CheckedStatementOwner, FormulaKind},
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
        TypeFactTable, TypeTable, TypedArena, TypedAst, TypedAstParts, TypedNodeId, TypedSiteRef,
    },
};
use mizar_resolve::{
    env::{
        ContributionKind, ExportStatus, NamespacePath, SourceContributionId, SymbolEnv,
        SymbolEnvIndexes, SymbolKind, Visibility,
    },
    labels::{
        LabelProjection, LabelProjectionData, LabelReferenceCandidate, LabelResolver,
        LabelScopePath,
    },
    names::LocalTermScope,
    resolved_ast::{
        LabelKind, LabelOriginPath, ModuleId, NameRefTable, NodeReferenceKey, NodeResolutionState,
        RecoveryState, ReferenceSite, ResolvedArena, ResolvedArenaBuilder, ResolvedAst,
        ResolvedImports, ResolvedNode, SemanticOrigin, SymbolId,
    },
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, SessionIdAllocator,
    SourceAnchor, SourceId, SourceRange,
};
use mizar_syntax::{SurfaceAst, SurfaceNodeKind, SurfaceTokenKind, SyntaxKind};

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_application::{
        WrappedImportedApplicationSite, unwrapped_imported_source_application_handoff_in_context,
        unwrapped_imported_source_application_owned_node_kinds,
        wrapped_imported_source_application_handoff_in_context,
        wrapped_imported_source_application_owned_node_kinds,
    },
    source_ast::{
        direct_token_texts, structural_child_ids, subtree_has_recovery, surface_nodes_with_kind,
        surface_site,
    },
    source_formula::{
        SourceReservedVariableBinaryFormula, SourceReservedVariableBinaryFormulaConfig,
        SourceReservedVariableBuiltinType, extract_source_reserved_variable_binary_formula,
    },
    source_formula_composition::source_formula_composition_output_with_source,
    source_reserve::extract_builtin_source_reserve_declarations_after_node_guard,
    source_set_term::source_set_term_output_with_source_term_in_context,
    source_structure::{
        ImportedStructureConstructorSite, ImportedStructureSelectorSite,
        ImportedStructureUpdateSite, imported_structure_constructor_handoff_in_context,
        imported_structure_constructor_owned_node_kinds,
        imported_structure_selector_handoff_in_context,
        imported_structure_selector_owned_node_kinds, imported_structure_update_handoff_in_context,
        imported_structure_update_owned_node_kinds,
    },
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

pub(in crate::runner) const SOURCE_STATEMENT_B3M1_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementMultipleWitnessSmoke: x = x proof\n",
    "  take y = x, x;\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2A_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementNumeralWitnessSmoke: x = x proof\n",
    "  take 101;\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B1_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementParenthesizedWitnessSmoke: x = x proof\n",
    "  take (x);\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2A_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementNestedParenthesizedWitnessSmoke: x = x proof\n",
    "  take ((x));\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B1A_TEXT: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementApplicationWitnessSmoke: x = x proof\n",
    "  take 1 ++ 2;\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B1B1_TEXT: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementParenthesizedApplicationWitnessSmoke: x = x proof\n",
    "  take (1 ++ 2);\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B2A_TEXT: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureConstructorWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2);\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B2B_TEXT: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureSelectorWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2).x;\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B2C_TEXT: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureUpdateWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2) with (x := 3);\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B3A_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementSetEnumerationWitnessSmoke: x = x proof\n",
    "  take {1, 2};\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B3B_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementEmptySetEnumerationWitnessSmoke: x = x proof\n",
    "  take {};\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B3C_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementChoiceWitnessSmoke: x = x proof\n",
    "  take the set;\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B3D_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementQuaWitnessSmoke: x = x proof\n",
    "  take 4 qua set;\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B3M2B2B3E_TEXT: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementComprehensionWitnessSmoke: x = x proof\n",
    "  take {3 where candidate255 is set};\n",
    "  thus x = x;\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_STATEMENT_B4A_TEXT: &str =
    "theorem FormulaQuantifierBoundUsePayloadBoundary: for x being set holds x = x;\n\n";

const SOURCE_STATEMENT_B4A_LABEL: &str = "FormulaQuantifierBoundUsePayloadBoundary";
const SOURCE_STATEMENT_B4A_SPELLING: &str =
    "theorem FormulaQuantifierBoundUsePayloadBoundary : for x being set holds x = x ;";
pub(in crate::runner) const SOURCE_STATEMENT_B4B_TEXT: &str = concat!(
    "theorem FormulaConnectiveGroupingPayloadBoundary: for x being set holds ",
    "((0 = 0 & ... & 0 = 3) or (0 = 0 or ... or 0 = 3)) iff ",
    "((0 = 0 & 0 = 0) or (0 = 0 or 0 = 0));\n\n",
);
const SOURCE_STATEMENT_B4B_LABEL: &str = "FormulaConnectiveGroupingPayloadBoundary";
const SOURCE_STATEMENT_B4B_SPELLING: &str = concat!(
    "theorem FormulaConnectiveGroupingPayloadBoundary : for x being set holds ",
    "( ( 0 = 0 & ... & 0 = 3 ) or ( 0 = 0 or ... or 0 = 3 ) ) iff ",
    "( ( 0 = 0 & 0 = 0 ) or ( 0 = 0 or 0 = 0 ) ) ;",
);
pub(in crate::runner) const SOURCE_STATEMENT_B4C_TEXT: &str = concat!(
    "reserve r for set; theorem FormulaNestedQuantifierPayloadBoundary: ",
    "for x being set st x = x ex y being set st for r st r = y holds x = r;\n\n",
);
const SOURCE_STATEMENT_B4C_LABEL: &str = "FormulaNestedQuantifierPayloadBoundary";
const SOURCE_STATEMENT_B4C_SPELLING: &str = concat!(
    "theorem FormulaNestedQuantifierPayloadBoundary : ",
    "for x being set st x = x ex y being set st for r st r = y holds x = r ;",
);
const TASK258B4A_SURFACE_KINDS: [&str; 26] = [
    "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"FormulaQuantifierBoundUsePayloadBoundary\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"being\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"holds\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
    "TypeHead",
    "TypeExpression",
    "QuantifierVariableSegment",
    "TermReference",
    "TermExpression",
    "TermReference",
    "TermExpression",
    "BuiltinPredicateApplication",
    "QuantifiedFormula(Universal)",
    "FormulaExpression",
    "TheoremItem",
    "ItemList",
    "CompilationUnit",
    "Root",
];
const TASK258B4A_SURFACE_RANGES: [(usize, usize); 26] = [
    (0, 7),
    (8, 48),
    (48, 49),
    (50, 53),
    (54, 55),
    (56, 61),
    (62, 65),
    (66, 71),
    (72, 73),
    (74, 75),
    (76, 77),
    (77, 78),
    (62, 65),
    (62, 65),
    (54, 65),
    (72, 73),
    (72, 73),
    (76, 77),
    (76, 77),
    (72, 77),
    (50, 77),
    (50, 77),
    (0, 78),
    (0, 78),
    (0, 78),
    (0, 78),
];
const TASK258B4A_SURFACE_CHILDREN: [&[usize]; 26] = [
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
    &[6],
    &[12],
    &[4, 5, 13],
    &[8],
    &[15],
    &[10],
    &[17],
    &[16, 9, 18],
    &[3, 14, 7, 19],
    &[20],
    &[0, 1, 2, 21, 11],
    &[22],
    &[23],
    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 24],
];

fn exact_task258b4a_surface_profile(ast: &SurfaceAst, source_text: &str) -> bool {
    source_text == SOURCE_STATEMENT_B4A_TEXT
        && source_text.len() == 80
        && source_text.ends_with("\n\n")
        && ast.nodes().len() == TASK258B4A_SURFACE_KINDS.len()
        && ast.root().map(|root| root.index()) == Some(25)
        && ast.nodes().iter().enumerate().all(|(index, node)| {
            format!("{:?}", node.kind) == TASK258B4A_SURFACE_KINDS[index]
                && node.range
                    == range(
                        ast.source_id,
                        TASK258B4A_SURFACE_RANGES[index].0,
                        TASK258B4A_SURFACE_RANGES[index].1,
                    )
                && !node.recovered
                && node
                    .children
                    .iter()
                    .map(|child| child.index())
                    .eq(TASK258B4A_SURFACE_CHILDREN[index].iter().copied())
        })
}

const TASK258B4B_SURFACE_KINDS: [&str; 124] = [
    "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"FormulaConnectiveGroupingPayloadBoundary\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"being\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"holds\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"&\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"...\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"&\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"3\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"or\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"or\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"...\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"or\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"3\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"iff\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"&\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"or\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"(\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"or\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Numeral, text: \"0\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \")\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
    "TypeHead",
    "TypeExpression",
    "QuantifierVariableSegment",
    "NumeralTerm",
    "TermExpression",
    "NumeralTerm",
    "TermExpression",
    "BuiltinPredicateApplication",
    "NumeralTerm",
    "TermExpression",
    "NumeralTerm",
    "TermExpression",
    "BuiltinPredicateApplication",
    "BinaryFormula(SurfaceFormulaBinaryOperator { connective: And, repeated: true })",
    "FormulaExpression",
    "ParenthesizedFormula",
    "NumeralTerm",
    "TermExpression",
    "NumeralTerm",
    "TermExpression",
    "BuiltinPredicateApplication",
    "NumeralTerm",
    "TermExpression",
    "NumeralTerm",
    "TermExpression",
    "BuiltinPredicateApplication",
    "BinaryFormula(SurfaceFormulaBinaryOperator { connective: Or, repeated: true })",
    "FormulaExpression",
    "ParenthesizedFormula",
    "BinaryFormula(SurfaceFormulaBinaryOperator { connective: Or, repeated: false })",
    "FormulaExpression",
    "ParenthesizedFormula",
    "NumeralTerm",
    "TermExpression",
    "NumeralTerm",
    "TermExpression",
    "BuiltinPredicateApplication",
    "NumeralTerm",
    "TermExpression",
    "NumeralTerm",
    "TermExpression",
    "BuiltinPredicateApplication",
    "BinaryFormula(SurfaceFormulaBinaryOperator { connective: And, repeated: false })",
    "FormulaExpression",
    "ParenthesizedFormula",
    "NumeralTerm",
    "TermExpression",
    "NumeralTerm",
    "TermExpression",
    "BuiltinPredicateApplication",
    "NumeralTerm",
    "TermExpression",
    "NumeralTerm",
    "TermExpression",
    "BuiltinPredicateApplication",
    "BinaryFormula(SurfaceFormulaBinaryOperator { connective: Or, repeated: false })",
    "FormulaExpression",
    "ParenthesizedFormula",
    "BinaryFormula(SurfaceFormulaBinaryOperator { connective: Or, repeated: false })",
    "FormulaExpression",
    "ParenthesizedFormula",
    "BinaryFormula(SurfaceFormulaBinaryOperator { connective: Iff, repeated: false })",
    "QuantifiedFormula(Universal)",
    "FormulaExpression",
    "TheoremItem",
    "ItemList",
    "CompilationUnit",
    "Root",
];

const TASK258B4B_SURFACE_RANGES: [(usize, usize); 124] = [
    (0, 7),
    (8, 48),
    (48, 49),
    (50, 53),
    (54, 55),
    (56, 61),
    (62, 65),
    (66, 71),
    (72, 73),
    (73, 74),
    (74, 75),
    (76, 77),
    (78, 79),
    (80, 81),
    (82, 85),
    (86, 87),
    (88, 89),
    (90, 91),
    (92, 93),
    (93, 94),
    (95, 97),
    (98, 99),
    (99, 100),
    (101, 102),
    (103, 104),
    (105, 107),
    (108, 111),
    (112, 114),
    (115, 116),
    (117, 118),
    (119, 120),
    (120, 121),
    (121, 122),
    (123, 126),
    (127, 128),
    (128, 129),
    (129, 130),
    (131, 132),
    (133, 134),
    (135, 136),
    (137, 138),
    (139, 140),
    (141, 142),
    (142, 143),
    (144, 146),
    (147, 148),
    (148, 149),
    (150, 151),
    (152, 153),
    (154, 156),
    (157, 158),
    (159, 160),
    (161, 162),
    (162, 163),
    (163, 164),
    (164, 165),
    (62, 65),
    (62, 65),
    (54, 65),
    (74, 75),
    (74, 75),
    (78, 79),
    (78, 79),
    (74, 79),
    (88, 89),
    (88, 89),
    (92, 93),
    (92, 93),
    (88, 93),
    (74, 93),
    (74, 93),
    (73, 94),
    (99, 100),
    (99, 100),
    (103, 104),
    (103, 104),
    (99, 104),
    (115, 116),
    (115, 116),
    (119, 120),
    (119, 120),
    (115, 120),
    (99, 120),
    (99, 120),
    (98, 121),
    (73, 121),
    (73, 121),
    (72, 122),
    (129, 130),
    (129, 130),
    (133, 134),
    (133, 134),
    (129, 134),
    (137, 138),
    (137, 138),
    (141, 142),
    (141, 142),
    (137, 142),
    (129, 142),
    (129, 142),
    (128, 143),
    (148, 149),
    (148, 149),
    (152, 153),
    (152, 153),
    (148, 153),
    (157, 158),
    (157, 158),
    (161, 162),
    (161, 162),
    (157, 162),
    (148, 162),
    (148, 162),
    (147, 163),
    (128, 163),
    (128, 163),
    (127, 164),
    (72, 164),
    (50, 164),
    (50, 164),
    (0, 165),
    (0, 165),
    (0, 165),
    (0, 165),
];

const TASK258B4B_SURFACE_CHILDREN: [&[usize]; 67] = [
    &[6],
    &[56],
    &[4, 5, 57],
    &[10],
    &[59],
    &[12],
    &[61],
    &[60, 11, 62],
    &[16],
    &[64],
    &[18],
    &[66],
    &[65, 17, 67],
    &[63, 13, 14, 15, 68],
    &[69],
    &[9, 70, 19],
    &[22],
    &[72],
    &[24],
    &[74],
    &[73, 23, 75],
    &[28],
    &[77],
    &[30],
    &[79],
    &[78, 29, 80],
    &[76, 25, 26, 27, 81],
    &[82],
    &[21, 83, 31],
    &[71, 20, 84],
    &[85],
    &[8, 86, 32],
    &[36],
    &[88],
    &[38],
    &[90],
    &[89, 37, 91],
    &[40],
    &[93],
    &[42],
    &[95],
    &[94, 41, 96],
    &[92, 39, 97],
    &[98],
    &[35, 99, 43],
    &[46],
    &[101],
    &[48],
    &[103],
    &[102, 47, 104],
    &[50],
    &[106],
    &[52],
    &[108],
    &[107, 51, 109],
    &[105, 49, 110],
    &[111],
    &[45, 112, 53],
    &[100, 44, 113],
    &[114],
    &[34, 115, 54],
    &[87, 33, 116],
    &[3, 58, 7, 117],
    &[118],
    &[0, 1, 2, 119, 55],
    &[120],
    &[121],
];

fn exact_task258b4b_surface_profile(ast: &SurfaceAst, source_text: &str) -> bool {
    source_text == SOURCE_STATEMENT_B4B_TEXT
        && source_text.len() == 167
        && source_text.ends_with("\n\n")
        && ast.nodes().len() == TASK258B4B_SURFACE_KINDS.len()
        && ast.root().map(|root| root.index()) == Some(123)
        && ast.nodes().iter().enumerate().all(|(index, node)| {
            let children_are_exact = if index < 56 {
                node.children.is_empty()
            } else if index < 123 {
                node.children
                    .iter()
                    .map(|child| child.index())
                    .eq(TASK258B4B_SURFACE_CHILDREN[index - 56].iter().copied())
            } else {
                node.children
                    .iter()
                    .map(|child| child.index())
                    .eq((0..=55).chain(std::iter::once(122)))
            };
            format!("{:?}", node.kind) == TASK258B4B_SURFACE_KINDS[index]
                && node.range
                    == range(
                        ast.source_id,
                        TASK258B4B_SURFACE_RANGES[index].0,
                        TASK258B4B_SURFACE_RANGES[index].1,
                    )
                && !node.recovered
                && children_are_exact
        })
}

const TASK258B4C_SURFACE_KINDS: [&str; 66] = [
    "Token(SurfaceToken { kind: ReservedWord, text: \"reserve\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"r\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"FormulaNestedQuantifierPayloadBoundary\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"being\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"st\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"ex\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"y\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"being\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"st\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"r\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"st\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"r\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"y\" })",
    "Token(SurfaceToken { kind: ReservedWord, text: \"holds\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
    "Token(SurfaceToken { kind: Identifier, text: \"r\" })",
    "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
    "TypeHead",
    "TypeExpression",
    "ReserveSegment",
    "ReserveItem",
    "TypeHead",
    "TypeExpression",
    "QuantifierVariableSegment",
    "TermReference",
    "TermExpression",
    "TermReference",
    "TermExpression",
    "BuiltinPredicateApplication",
    "TypeHead",
    "TypeExpression",
    "QuantifierVariableSegment",
    "QuantifierVariableSegment",
    "TermReference",
    "TermExpression",
    "TermReference",
    "TermExpression",
    "BuiltinPredicateApplication",
    "TermReference",
    "TermExpression",
    "TermReference",
    "TermExpression",
    "BuiltinPredicateApplication",
    "QuantifiedFormula(Universal)",
    "QuantifiedFormula(Existential)",
    "QuantifiedFormula(Universal)",
    "FormulaExpression",
    "TheoremItem",
    "ItemList",
    "CompilationUnit",
    "Root",
];
const TASK258B4C_SURFACE_RANGES: [(usize, usize); 66] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 65),
    (65, 66),
    (67, 70),
    (71, 72),
    (73, 78),
    (79, 82),
    (83, 85),
    (86, 87),
    (88, 89),
    (90, 91),
    (92, 94),
    (95, 96),
    (97, 102),
    (103, 106),
    (107, 109),
    (110, 113),
    (114, 115),
    (116, 118),
    (119, 120),
    (121, 122),
    (123, 124),
    (125, 130),
    (131, 132),
    (133, 134),
    (135, 136),
    (136, 137),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (79, 82),
    (79, 82),
    (71, 82),
    (86, 87),
    (86, 87),
    (90, 91),
    (90, 91),
    (86, 91),
    (103, 106),
    (103, 106),
    (95, 106),
    (114, 115),
    (119, 120),
    (119, 120),
    (123, 124),
    (123, 124),
    (119, 124),
    (131, 132),
    (131, 132),
    (135, 136),
    (135, 136),
    (131, 136),
    (110, 136),
    (92, 136),
    (67, 136),
    (67, 136),
    (19, 137),
    (0, 137),
    (0, 137),
    (0, 137),
];
const TASK258B4C_SURFACE_CHILDREN: [&[usize]; 66] = [
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
    &[3],
    &[32],
    &[1, 2, 33],
    &[0, 34, 4],
    &[11],
    &[36],
    &[9, 10, 37],
    &[13],
    &[39],
    &[15],
    &[41],
    &[40, 14, 42],
    &[19],
    &[44],
    &[17, 18, 45],
    &[22],
    &[24],
    &[48],
    &[26],
    &[50],
    &[49, 25, 51],
    &[28],
    &[53],
    &[30],
    &[55],
    &[54, 29, 56],
    &[21, 47, 23, 52, 27, 57],
    &[16, 46, 20, 58],
    &[8, 38, 12, 43, 59],
    &[60],
    &[5, 6, 7, 61, 31],
    &[35, 62],
    &[63],
    &[
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 64,
    ],
];

fn exact_task258b4c_surface_profile(ast: &SurfaceAst, source_text: &str) -> bool {
    source_text == SOURCE_STATEMENT_B4C_TEXT
        && source_text.len() == 139
        && source_text.ends_with("\n\n")
        && ast.nodes().len() == TASK258B4C_SURFACE_KINDS.len()
        && ast.root().map(|root| root.index()) == Some(65)
        && ast.nodes().iter().enumerate().all(|(index, node)| {
            format!("{:?}", node.kind) == TASK258B4C_SURFACE_KINDS[index]
                && node.range
                    == range(
                        ast.source_id,
                        TASK258B4C_SURFACE_RANGES[index].0,
                        TASK258B4C_SURFACE_RANGES[index].1,
                    )
                && !node.recovered
                && node
                    .children
                    .iter()
                    .map(|child| child.index())
                    .eq(TASK258B4C_SURFACE_CHILDREN[index].iter().copied())
        })
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceStatementB4ASurfaceMutation {
    None,
    NodeKind(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    RootIdentity,
}

#[cfg(test)]
pub(in crate::runner) fn task258b4a_surface_profile_with_mutation_for_test(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB4ASurfaceMutation,
) -> bool {
    let mut kinds = TASK258B4A_SURFACE_KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let mut ranges = TASK258B4A_SURFACE_RANGES.to_vec();
    let mut recoveries = [false; 26];
    let mut children = TASK258B4A_SURFACE_CHILDREN
        .iter()
        .map(|children| children.to_vec())
        .collect::<Vec<_>>();
    let mut root = Some(25);
    match mutation {
        SourceStatementB4ASurfaceMutation::None => {}
        SourceStatementB4ASurfaceMutation::NodeKind(index) => kinds[index].push('!'),
        SourceStatementB4ASurfaceMutation::NodeRange(index) => {
            ranges[index].1 = ranges[index].1.saturating_add(1);
        }
        SourceStatementB4ASurfaceMutation::NodeRecovery(index) => {
            recoveries[index] = !recoveries[index];
        }
        SourceStatementB4ASurfaceMutation::NodeChildren(index) => {
            if children[index].len() > 1 {
                children[index].rotate_left(1);
            } else {
                children[index].push(index);
            }
        }
        SourceStatementB4ASurfaceMutation::RootIdentity => root = None,
    }
    source_text == SOURCE_STATEMENT_B4A_TEXT
        && ast.nodes().len() == 26
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceStatementB4BSurfaceMutation {
    None,
    NodeKind(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    RootIdentity,
}

#[cfg(test)]
pub(in crate::runner) fn task258b4b_surface_profile_with_mutation_for_test(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB4BSurfaceMutation,
) -> bool {
    let mut kinds = TASK258B4B_SURFACE_KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let mut ranges = TASK258B4B_SURFACE_RANGES.to_vec();
    let mut recoveries = [false; 124];
    let mut children = (0..124)
        .map(|index| match index {
            0..=55 => Vec::new(),
            56..=122 => TASK258B4B_SURFACE_CHILDREN[index - 56].to_vec(),
            123 => (0..=55).chain(std::iter::once(122)).collect(),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    let mut root = Some(123);
    match mutation {
        SourceStatementB4BSurfaceMutation::None => {}
        SourceStatementB4BSurfaceMutation::NodeKind(index) => kinds[index].push('!'),
        SourceStatementB4BSurfaceMutation::NodeRange(index) => {
            ranges[index].1 = ranges[index].1.saturating_add(1);
        }
        SourceStatementB4BSurfaceMutation::NodeRecovery(index) => {
            recoveries[index] = !recoveries[index];
        }
        SourceStatementB4BSurfaceMutation::NodeChildren(index) => {
            if children[index].len() > 1 {
                children[index].rotate_left(1);
            } else {
                children[index].push(index);
            }
        }
        SourceStatementB4BSurfaceMutation::RootIdentity => root = None,
    }
    source_text == SOURCE_STATEMENT_B4B_TEXT
        && ast.nodes().len() == 124
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceStatementB4CSurfaceMutation {
    None,
    NodeKind(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    RootIdentity,
}

#[cfg(test)]
pub(in crate::runner) fn task258b4c_surface_profile_with_mutation_for_test(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB4CSurfaceMutation,
) -> bool {
    let mut kinds = TASK258B4C_SURFACE_KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let mut ranges = TASK258B4C_SURFACE_RANGES.to_vec();
    let mut recoveries = [false; 66];
    let mut children = TASK258B4C_SURFACE_CHILDREN
        .iter()
        .map(|children| children.to_vec())
        .collect::<Vec<_>>();
    let mut root = Some(65);
    match mutation {
        SourceStatementB4CSurfaceMutation::None => {}
        SourceStatementB4CSurfaceMutation::NodeKind(index) => kinds[index].push('!'),
        SourceStatementB4CSurfaceMutation::NodeRange(index) => {
            ranges[index].1 = ranges[index].1.saturating_add(1);
        }
        SourceStatementB4CSurfaceMutation::NodeRecovery(index) => {
            recoveries[index] = !recoveries[index];
        }
        SourceStatementB4CSurfaceMutation::NodeChildren(index) => {
            if children[index].len() > 1 {
                children[index].rotate_left(1);
            } else {
                children[index].push(index);
            }
        }
        SourceStatementB4CSurfaceMutation::RootIdentity => root = None,
    }
    source_text == SOURCE_STATEMENT_B4C_TEXT
        && ast.nodes().len() == 66
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
const SOURCE_STATEMENT_B3M1_LABEL: &str = "FormulaStatementMultipleWitnessSmoke";
const SOURCE_STATEMENT_B3M1_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementMultipleWitnessSmoke : x = x proof take y = x , x ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2A_LABEL: &str = "FormulaStatementNumeralWitnessSmoke";
const SOURCE_STATEMENT_B3M2A_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementNumeralWitnessSmoke : x = x proof take 101 ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B1_LABEL: &str = "FormulaStatementParenthesizedWitnessSmoke";
const SOURCE_STATEMENT_B3M2B1_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementParenthesizedWitnessSmoke : x = x proof take ( x ) ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2A_LABEL: &str = "FormulaStatementNestedParenthesizedWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2A_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementNestedParenthesizedWitnessSmoke : x = x proof take ( ( x ) ) ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B1A_LABEL: &str = "FormulaStatementApplicationWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B1A_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementApplicationWitnessSmoke : x = x proof take 1 ++ 2 ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B1B1_LABEL: &str =
    "FormulaStatementParenthesizedApplicationWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B1B1_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementParenthesizedApplicationWitnessSmoke : x = x proof take ( 1 ++ 2 ) ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B2A_LABEL: &str = "FormulaStatementStructureConstructorWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B2A_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementStructureConstructorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B2B_LABEL: &str = "FormulaStatementStructureSelectorWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B2B_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementStructureSelectorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) . x ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B2C_LABEL: &str = "FormulaStatementStructureUpdateWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B2C_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementStructureUpdateWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 ) ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B3A_LABEL: &str = "FormulaStatementSetEnumerationWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B3A_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementSetEnumerationWitnessSmoke : x = x proof take { 1 , 2 } ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B3B_LABEL: &str = "FormulaStatementEmptySetEnumerationWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B3B_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementEmptySetEnumerationWitnessSmoke : x = x proof take { } ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B3C_LABEL: &str = "FormulaStatementChoiceWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B3C_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementChoiceWitnessSmoke : x = x proof take the set ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B3D_LABEL: &str = "FormulaStatementQuaWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B3D_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementQuaWitnessSmoke : x = x proof take 4 qua set ; thus x = x ; end ;",
    "thus x = x ;",
];
const SOURCE_STATEMENT_B3M2B2B3E_LABEL: &str = "FormulaStatementComprehensionWitnessSmoke";
const SOURCE_STATEMENT_B3M2B2B3E_SPELLINGS: [&str; 2] = [
    "theorem FormulaStatementComprehensionWitnessSmoke : x = x proof take { 3 where candidate255 is set } ; thus x = x ; end ;",
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
pub(in crate::runner) struct SourceStatementB3M1Extraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 6],
    pub(in crate::runner) term_ranges: [SourceRange; 6],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_sites: [TypedSiteRef; 2],
    pub(in crate::runner) witness_ranges: [SourceRange; 2],
    pub(in crate::runner) name_site: TypedSiteRef,
    pub(in crate::runner) name_range: SourceRange,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2AExtraction {
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
pub(in crate::runner) struct SourceStatementB3M2B1Extraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 5],
    pub(in crate::runner) term_ranges: [SourceRange; 6],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2AExtraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 5],
    pub(in crate::runner) term_ranges: [SourceRange; 7],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B1AExtraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 6],
    pub(in crate::runner) term_ranges: [SourceRange; 6],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) application_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B1B1Extraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 6],
    pub(in crate::runner) term_ranges: [SourceRange; 6],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) application_node: usize,
    pub(in crate::runner) application_wrapper_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B2AExtraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 6],
    pub(in crate::runner) term_ranges: [SourceRange; 6],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) structure_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B2BExtraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 6],
    pub(in crate::runner) term_ranges: [SourceRange; 6],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) selector_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B2CExtraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 7],
    pub(in crate::runner) term_ranges: [SourceRange; 7],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) update_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B3AExtraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 6],
    pub(in crate::runner) term_ranges: [SourceRange; 6],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) set_term_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B3BExtraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 4],
    pub(in crate::runner) term_ranges: [SourceRange; 4],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) set_term_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B3CExtraction {
    pub(in crate::runner) theorem_site: TypedSiteRef,
    pub(in crate::runner) theorem_range: SourceRange,
    pub(in crate::runner) label_range: SourceRange,
    pub(in crate::runner) statement_sites: [TypedSiteRef; 2],
    pub(in crate::runner) statement_ranges: [SourceRange; 2],
    pub(in crate::runner) formula_sites: [TypedSiteRef; 2],
    pub(in crate::runner) formula_ranges: [SourceRange; 2],
    pub(in crate::runner) term_sites: [TypedSiteRef; 4],
    pub(in crate::runner) term_ranges: [SourceRange; 4],
    pub(in crate::runner) take_site: TypedSiteRef,
    pub(in crate::runner) take_range: SourceRange,
    pub(in crate::runner) witness_site: TypedSiteRef,
    pub(in crate::runner) witness_range: SourceRange,
    pub(in crate::runner) set_term_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B3DExtraction {
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
    pub(in crate::runner) set_term_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceStatementB3M2B2B3EExtraction {
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
    pub(in crate::runner) set_term_node: usize,
    pub(in crate::runner) proof_range: SourceRange,
}

#[derive(Debug, Clone)]
struct SourceStatementWitnessItemExtraction {
    site: TypedSiteRef,
    range: SourceRange,
    name: Option<(TypedSiteRef, SourceRange)>,
    spelling: &'static str,
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
    term_sites: Vec<TypedSiteRef>,
    term_ranges: Vec<SourceRange>,
    take_site: TypedSiteRef,
    take_range: SourceRange,
    witnesses: Vec<SourceStatementWitnessItemExtraction>,
    proof_range: SourceRange,
    label: &'static str,
    spellings: &'static [&'static str; 2],
    task: &'static str,
    node_count: usize,
    root: usize,
    atomic_term_starts: [usize; 2],
    input_fact_reference_starts: [usize; 2],
    application_node: Option<usize>,
    application_wrapper_node: Option<usize>,
    structure_node: Option<usize>,
    structure_selector_node: Option<usize>,
    structure_update_node: Option<usize>,
    set_term_node: Option<usize>,
    source_text: &'static str,
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
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "x",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3_LABEL,
            spellings: &SOURCE_STATEMENT_B3_SPELLINGS,
            task: "Task258B3",
            node_count: 49,
            root: 48,
            atomic_term_starts: [0, 3],
            input_fact_reference_starts: [0, 3],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3_TEXT,
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
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: Some((extracted.name_site, extracted.name_range)),
                spelling: "y = x",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3N_LABEL,
            spellings: &SOURCE_STATEMENT_B3N_SPELLINGS,
            task: "Task258B3N",
            node_count: 51,
            root: 50,
            atomic_term_starts: [0, 3],
            input_fact_reference_starts: [0, 3],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3N_TEXT,
        }
    }
}

impl From<SourceStatementB3M1Extraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M1Extraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site.clone(),
            take_range: extracted.take_range,
            witnesses: vec![
                SourceStatementWitnessItemExtraction {
                    site: extracted.witness_sites[0].clone(),
                    range: extracted.witness_ranges[0],
                    name: Some((extracted.name_site, extracted.name_range)),
                    spelling: "y = x",
                },
                SourceStatementWitnessItemExtraction {
                    site: extracted.witness_sites[1].clone(),
                    range: extracted.witness_ranges[1],
                    name: None,
                    spelling: "x",
                },
            ],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M1_LABEL,
            spellings: &SOURCE_STATEMENT_B3M1_SPELLINGS,
            task: "Task258B3M1",
            node_count: 56,
            root: 55,
            atomic_term_starts: [0, 4],
            input_fact_reference_starts: [0, 4],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3M1_TEXT,
        }
    }
}

impl From<SourceStatementB3M2AExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2AExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "101",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2A_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2A_SPELLINGS,
            task: "Task258B3M2A",
            node_count: 49,
            root: 48,
            atomic_term_starts: [0, 3],
            input_fact_reference_starts: [0, 2],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3M2A_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B1Extraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B1Extraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "( x )",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B1_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B1_SPELLINGS,
            task: "Task258B3M2B1",
            node_count: 53,
            root: 52,
            atomic_term_starts: [0, 4],
            input_fact_reference_starts: [0, 3],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3M2B1_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2AExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2AExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "( ( x ) )",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2A_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2A_SPELLINGS,
            task: "Task258B3M2B2A",
            node_count: 57,
            root: 56,
            atomic_term_starts: [0, 5],
            input_fact_reference_starts: [0, 3],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3M2B2A_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B1AExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B1AExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "1 ++ 2",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B1A_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B1A_SPELLINGS,
            task: "Task258B3M2B2B1A",
            node_count: 63,
            root: 62,
            atomic_term_starts: [0, 4],
            input_fact_reference_starts: [0, 2],
            application_node: Some(extracted.application_node),
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3M2B2B1A_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B1B1Extraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B1B1Extraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "( 1 ++ 2 )",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B1B1_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B1B1_SPELLINGS,
            task: "Task258B3M2B2B1B1",
            node_count: 67,
            root: 66,
            atomic_term_starts: [0, 4],
            input_fact_reference_starts: [0, 2],
            application_node: Some(extracted.application_node),
            application_wrapper_node: Some(extracted.application_wrapper_node),
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B2AExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B2AExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "TypeCaseStruct ( x : 1 , y : 2 )",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B2A_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B2A_SPELLINGS,
            task: "Task258B3M2B2B2A",
            node_count: 76,
            root: 75,
            atomic_term_starts: [0, 4],
            input_fact_reference_starts: [0, 2],
            application_node: None,
            application_wrapper_node: None,
            structure_node: Some(extracted.structure_node),
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B2BExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B2BExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "TypeCaseStruct ( x : 1 , y : 2 ) . x",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B2B_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B2B_SPELLINGS,
            task: "Task258B3M2B2B2B",
            node_count: 79,
            root: 78,
            atomic_term_starts: [0, 4],
            input_fact_reference_starts: [0, 2],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: Some(extracted.selector_node),
            structure_update_node: None,
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B2CExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B2CExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 )",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B2C_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B2C_SPELLINGS,
            task: "Task258B3M2B2B2C",
            node_count: 86,
            root: 85,
            atomic_term_starts: [0, 5],
            input_fact_reference_starts: [0, 2],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: Some(extracted.update_node),
            set_term_node: None,
            source_text: SOURCE_STATEMENT_B3M2B2B2C_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B3AExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B3AExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "{ 1 , 2 }",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B3A_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B3A_SPELLINGS,
            task: "Task258B3M2B2B3A",
            node_count: 57,
            root: 56,
            atomic_term_starts: [0, 4],
            input_fact_reference_starts: [0, 2],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: Some(extracted.set_term_node),
            source_text: SOURCE_STATEMENT_B3M2B2B3A_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B3BExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B3BExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "{ }",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B3B_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B3B_SPELLINGS,
            task: "Task258B3M2B2B3B",
            node_count: 50,
            root: 49,
            atomic_term_starts: [0, 2],
            input_fact_reference_starts: [0, 2],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: Some(extracted.set_term_node),
            source_text: SOURCE_STATEMENT_B3M2B2B3B_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B3CExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B3CExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "the set",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B3C_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B3C_SPELLINGS,
            task: "Task258B3M2B2B3C",
            node_count: 52,
            root: 51,
            atomic_term_starts: [0, 2],
            input_fact_reference_starts: [0, 2],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: Some(extracted.set_term_node),
            source_text: SOURCE_STATEMENT_B3M2B2B3C_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B3DExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B3DExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "4 qua set",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B3D_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B3D_SPELLINGS,
            task: "Task258B3M2B2B3D",
            node_count: 54,
            root: 53,
            atomic_term_starts: [0, 3],
            input_fact_reference_starts: [0, 2],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: Some(extracted.set_term_node),
            source_text: SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        }
    }
}

impl From<SourceStatementB3M2B2B3EExtraction> for SourceStatementWitnessExtraction {
    fn from(extracted: SourceStatementB3M2B2B3EExtraction) -> Self {
        Self {
            theorem_site: extracted.theorem_site,
            theorem_range: extracted.theorem_range,
            label_range: extracted.label_range,
            statement_sites: extracted.statement_sites,
            statement_ranges: extracted.statement_ranges,
            formula_sites: extracted.formula_sites,
            formula_ranges: extracted.formula_ranges,
            term_sites: extracted.term_sites.into(),
            term_ranges: extracted.term_ranges.into(),
            take_site: extracted.take_site,
            take_range: extracted.take_range,
            witnesses: vec![SourceStatementWitnessItemExtraction {
                site: extracted.witness_site,
                range: extracted.witness_range,
                name: None,
                spelling: "{ 3 where candidate255 is set }",
            }],
            proof_range: extracted.proof_range,
            label: SOURCE_STATEMENT_B3M2B2B3E_LABEL,
            spellings: &SOURCE_STATEMENT_B3M2B2B3E_SPELLINGS,
            task: "Task258B3M2B2B3E",
            node_count: 60,
            root: 59,
            atomic_term_starts: [0, 3],
            input_fact_reference_starts: [0, 2],
            application_node: None,
            application_wrapper_node: None,
            structure_node: None,
            structure_selector_node: None,
            structure_update_node: None,
            set_term_node: Some(extracted.set_term_node),
            source_text: SOURCE_STATEMENT_B3M2B2B3E_TEXT,
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
pub(in crate::runner) struct SourceStatementB4ARouteInputs {
    pub(in crate::runner) binding_env: BindingEnv,
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: SourcePrimaryTermHandoff,
    pub(in crate::runner) atomic: SourceAtomicFormulaHandoff,
    pub(in crate::runner) composite: SourceCompositeFormulaHandoff,
    pub(in crate::runner) composition: SourceFormulaCompositionHandoff,
    pub(in crate::runner) statement: SourceStatementHandoffInput,
}

pub(in crate::runner) type SourceStatementB4BRouteInputs = SourceStatementB4ARouteInputs;
pub(in crate::runner) type SourceStatementB4CRouteInputs = SourceStatementB4ARouteInputs;

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
    pub(in crate::runner) application: Option<SourceFunctorApplicationHandoff>,
    pub(in crate::runner) structure: Option<SourceStructureHandoff>,
    pub(in crate::runner) set_term: Option<SourceSetTermHandoff>,
}

pub(in crate::runner) type SourceStatementB3NRouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M1RouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2ARouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B1RouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2ARouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B1ARouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B1B1RouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B2ARouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B2BRouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B2CRouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B3ARouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B3BRouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B3CRouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B3DRouteInputs = SourceStatementB3RouteInputs;
pub(in crate::runner) type SourceStatementB3M2B2B3ERouteInputs = SourceStatementB3RouteInputs;

// Rationale: these variants type the exhaustive test-only lower-stage mutation seam and are intentionally dormant in non-test builds.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceStatementB3M2B2B3AStage {
    Task256,
    Task258,
    Witness,
}

#[cfg(test)]
pub(in crate::runner) type SourceStatementB3M2B2B3BStage = SourceStatementB3M2B2B3AStage;
#[cfg(test)]
pub(in crate::runner) type SourceStatementB3M2B2B3CStage = SourceStatementB3M2B2B3AStage;
#[cfg(test)]
pub(in crate::runner) type SourceStatementB3M2B2B3DStage = SourceStatementB3M2B2B3AStage;
#[cfg(test)]
pub(in crate::runner) type SourceStatementB3M2B2B3EStage = SourceStatementB3M2B2B3AStage;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceStatementB3M2B2B3BLowerStage {
    Task48,
    Task252,
    Task255,
}
#[cfg(test)]
pub(in crate::runner) type SourceStatementB3M2B2B3CLowerStage = SourceStatementB3M2B2B3BLowerStage;
#[cfg(test)]
pub(in crate::runner) type SourceStatementB3M2B2B3DLowerStage = SourceStatementB3M2B2B3BLowerStage;
#[cfg(test)]
pub(in crate::runner) type SourceStatementB3M2B2B3ELowerStage = SourceStatementB3M2B2B3BLowerStage;

#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3A_TASK256_FIELD_COUNT: usize = 72;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3A_TASK258_FIELD_COUNT: usize = 62;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3A_WITNESS_FIELD_COUNT: usize = 21;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3B_TASK256_FIELD_COUNT: usize =
    TASK258B3M2B2B3A_TASK256_FIELD_COUNT;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3B_TASK258_FIELD_COUNT: usize =
    TASK258B3M2B2B3A_TASK258_FIELD_COUNT;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3B_WITNESS_FIELD_COUNT: usize =
    TASK258B3M2B2B3A_WITNESS_FIELD_COUNT;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3B_TASK48_FIELD_COUNT: usize = 32;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3B_TASK252_FIELD_COUNT: usize = 55;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3B_TASK255_FIELD_COUNT: usize = 23;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3C_TASK48_FIELD_COUNT: usize =
    TASK258B3M2B2B3B_TASK48_FIELD_COUNT;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3C_TASK252_FIELD_COUNT: usize =
    TASK258B3M2B2B3B_TASK252_FIELD_COUNT;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3C_TASK255_FIELD_COUNT: usize = 39;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3C_TASK256_FIELD_COUNT: usize =
    TASK258B3M2B2B3A_TASK256_FIELD_COUNT;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3C_TASK258_FIELD_COUNT: usize =
    TASK258B3M2B2B3A_TASK258_FIELD_COUNT;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3C_WITNESS_FIELD_COUNT: usize =
    TASK258B3M2B2B3A_WITNESS_FIELD_COUNT;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3D_TASK48_FIELD_COUNT: usize = 32;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3D_TASK252_FIELD_COUNT: usize = 70;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3D_TASK255_FIELD_COUNT: usize = 44;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3D_TASK256_FIELD_COUNT: usize = 72;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3D_TASK258_FIELD_COUNT: usize = 62;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3D_WITNESS_FIELD_COUNT: usize = 21;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3E_TASK48_FIELD_COUNT: usize = 32;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3E_TASK252_FIELD_COUNT: usize = 70;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3E_TASK255_FIELD_COUNT: usize = 53;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3E_TASK256_FIELD_COUNT: usize = 72;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3E_TASK258_FIELD_COUNT: usize = 62;
#[cfg(test)]
pub(in crate::runner) const TASK258B3M2B2B3E_WITNESS_FIELD_COUNT: usize = 21;

#[derive(Debug)]
pub(in crate::runner) struct SourceStatementRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
    pub(in crate::runner) left_lookup_ordinal: usize,
    pub(in crate::runner) right_lookup_ordinal: usize,
    pub(in crate::runner) reference_use_ordinals: Vec<usize>,
}

fn source_statement_transport_output_is_valid(
    source_text: &str,
    output: &SourceStatementRouteOutput,
) -> bool {
    output.typed_ast.source_statement().is_some()
        && output.typed_ast.source_statement() == output.resolved.source_statement()
        && ((output.typed_ast.source_statement_references().is_none()
            && output.typed_ast.source_statement_witnesses().is_none()
            && output
                .typed_ast
                .source_statement()
                .is_some_and(|statement| {
                    statement.statements().len() == 1
                        && statement.composite_formula_fingerprint().is_some()
                        && statement.formula_composition_fingerprint().is_some()
                })
            && output.typed_ast.source_composite_formula().is_some()
            && output.typed_ast.source_formula_composition().is_some()
            && output.typed_ast.source_composite_formula()
                == output.resolved.source_composite_formula()
            && output.typed_ast.source_formula_composition()
                == output.resolved.source_formula_composition()
            && ((source_text == SOURCE_STATEMENT_B4A_TEXT
                && output.left_lookup_ordinal == 1
                && output.right_lookup_ordinal == 1
                && output.reference_use_ordinals == [1, 1])
                || (source_text == SOURCE_STATEMENT_B4B_TEXT
                    && output.left_lookup_ordinal == 0
                    && output.right_lookup_ordinal == 0
                    && output.reference_use_ordinals.is_empty())
                || (source_text == SOURCE_STATEMENT_B4C_TEXT
                    && output.left_lookup_ordinal == 2
                    && output.right_lookup_ordinal == 2
                    && output.reference_use_ordinals == [2, 2, 4, 4, 4, 4])))
            || (output.typed_ast.source_statement_references().is_none()
                && output.typed_ast.source_statement_witnesses().is_none()
                && output
                    .typed_ast
                    .source_statement()
                    .is_some_and(|statement| {
                        statement.statements().len() == 1
                            && statement.composite_formula_fingerprint().is_none()
                            && statement.formula_composition_fingerprint().is_none()
                    })
                && output.typed_ast.source_composite_formula().is_none()
                && output.typed_ast.source_formula_composition().is_none()
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
                && (output.reference_use_ordinals == [1; 4]
                    || output.reference_use_ordinals == [1; 5]
                    || output.reference_use_ordinals == [1; 6])))
}

fn source_statement_transport_detail_keys_for_output(
    source_text: &str,
    output: &SourceStatementRouteOutput,
) -> Vec<String> {
    if source_statement_transport_output_is_valid(source_text, output) {
        Vec::new()
    } else {
        vec!["type_elaboration.checker.typed_ast_invalid".to_owned()]
    }
}

pub(in crate::runner) fn source_statement_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_statement_output_with_source(ast, module, symbols, source_text) {
        None => None,
        Some(Ok(output)) => Some(source_statement_transport_detail_keys_for_output(
            source_text,
            &output,
        )),
        Some(Err(_)) => Some(vec![
            "type_elaboration.checker.typed_ast_invalid".to_owned(),
        ]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_transport_detail_keys_with_output_mutation_for_test(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementRouteOutput),
) -> Option<Vec<String>> {
    match source_statement_output_with_source(ast, module, symbols, source_text) {
        None => None,
        Some(Ok(mut output)) => {
            mutate(&mut output);
            Some(source_statement_transport_detail_keys_for_output(
                source_text,
                &output,
            ))
        }
        Some(Err(_)) => Some(vec![
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

pub(in crate::runner) fn extract_multiple_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M1Extraction> {
    if source_text != SOURCE_STATEMENT_B3M1_TEXT
        || source_text.len() != 113
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 56
        || ast.root()?.index() != 55
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
        || !exact_multiple_witness_surface_profile(ast)
    {
        return None;
    }
    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 112)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 71, 111)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 79, 93)?;
    let witness_nodes = surface_nodes_with_kind(ast, SurfaceNodeKind::Witness);
    let [
        (first_witness_id, first_witness),
        (second_witness_id, second_witness),
    ] = witness_nodes.as_slice()
    else {
        return None;
    };
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 96, 107)?;
    if item_children.len() != 2
        || item_children[0].index() != 29
        || item_children[1] != theorem_id
        || theorem_id.index() != 52
        || proof_id.index() != 51
        || take_id.index() != 42
        || first_witness_id.index() != 38
        || first_witness.range.start != 84
        || first_witness.range.end != 89
        || second_witness_id.index() != 41
        || second_witness.range.start != 91
        || second_witness.range.end != 92
        || conclusion_id.index() != 50
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3M1_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ",", ";"]
        || direct_token_texts(ast, first_witness).as_slice() != ["y", "="]
        || !direct_token_texts(ast, second_witness).is_empty()
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, *first_witness_id)
        || !surface_is_descendant(ast, take_id, *second_witness_id)
        || surface_is_descendant(ast, *first_witness_id, *second_witness_id)
        || surface_is_descendant(ast, *second_witness_id, *first_witness_id)
        || !surface_is_descendant(ast, proof_id, conclusion_id)
        || surface_is_descendant(ast, conclusion_id, take_id)
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::TakeStatement).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement).len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::CompactStatement).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::Reference).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::JustificationClause).is_empty()
    {
        return None;
    }
    let label = ast.nodes().get(6)?;
    let name_id = *first_witness.children.first()?;
    let name = ast.node(name_id)?;
    if label.range.start != 27
        || label.range.end != 63
        || label.token_text() != Some(SOURCE_STATEMENT_B3M1_LABEL)
        || name.range.start != 84
        || name.range.end != 85
        || name_id.index() != 13
        || name.token_text() != Some("y")
        || !surface_is_descendant(ast, *first_witness_id, name_id)
        || surface_is_descendant(ast, *second_witness_id, name_id)
    {
        return None;
    }
    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(65, 70), (101, 106)];
    let expected_formula_ids = [34, 47];
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
            || surface_is_descendant(ast, take_id, id)
        {
            return None;
        }
        formula_ids.push(id);
    }
    let term_ranges = [
        (65, 66),
        (69, 70),
        (88, 89),
        (91, 92),
        (101, 102),
        (105, 106),
    ];
    let expected_term_ids = [30, 32, 36, 39, 43, 45];
    let mut term_sites = Vec::new();
    for (index, (start, end)) in term_ranges.into_iter().enumerate() {
        let (id, term) = exact_surface_node(ast, SurfaceNodeKind::TermReference, start, end)?;
        if id.index() != expected_term_ids[index]
            || direct_token_texts(ast, term).as_slice() != ["x"]
            || (index == 2 && !surface_is_descendant(ast, *first_witness_id, id))
            || (index == 3 && !surface_is_descendant(ast, *second_witness_id, id))
            || (index < 2 && !surface_is_descendant(ast, theorem_id, id))
            || (index >= 4 && !surface_is_descendant(ast, conclusion_id, id))
        {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3M1Extraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [range(ast.source_id, 19, 112), range(ast.source_id, 96, 107)],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: term_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_sites: [
            surface_site(*first_witness_id),
            surface_site(*second_witness_id),
        ],
        witness_ranges: [first_witness.range, second_witness.range],
        name_site: surface_site(name_id),
        name_range: name.range,
        proof_range: proof.range,
    })
}

pub(in crate::runner) fn extract_numeral_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2AExtraction> {
    if source_text != SOURCE_STATEMENT_B3M2A_TEXT
        || source_text.len() != 107
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 49
        || ast.root()?.index() != 48
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
        || !exact_numeral_witness_surface_profile(ast)
    {
        return None;
    }
    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 106)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 70, 105)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 78, 87)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 83, 86)?;
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 90, 101)?;
    if item_children.len() != 2
        || item_children[0].index() != 25
        || item_children[1] != theorem_id
        || theorem_id.index() != 45
        || proof_id.index() != 44
        || take_id.index() != 35
        || witness_id.index() != 34
        || conclusion_id.index() != 43
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3M2A_LABEL, ":", ";"]
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
        || label.range.end != 62
        || label.token_text() != Some(SOURCE_STATEMENT_B3M2A_LABEL)
    {
        return None;
    }
    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(64, 69), (95, 100)];
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
            || surface_is_descendant(ast, take_id, id)
        {
            return None;
        }
        formula_ids.push(id);
    }
    let term_ranges = [(64, 65), (68, 69), (83, 86), (95, 96), (99, 100)];
    let expected_term_ids = [26, 28, 32, 36, 38];
    let mut term_sites = Vec::new();
    for (index, (start, end)) in term_ranges.into_iter().enumerate() {
        let kind = if index == 2 {
            SurfaceNodeKind::NumeralTerm
        } else {
            SurfaceNodeKind::TermReference
        };
        let (id, term) = exact_surface_node(ast, kind, start, end)?;
        if id.index() != expected_term_ids[index]
            || (index == 2 && direct_token_texts(ast, term).as_slice() != ["101"])
            || (index != 2 && direct_token_texts(ast, term).as_slice() != ["x"])
            || (index == 2 && !surface_is_descendant(ast, witness_id, id))
            || (index < 2 && !surface_is_descendant(ast, theorem_id, id))
            || (index >= 3 && !surface_is_descendant(ast, conclusion_id, id))
            || (index != 2 && surface_is_descendant(ast, witness_id, id))
        {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3M2AExtraction {
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
        proof_range: proof.range,
    })
}

const TASK258B3M2A_SURFACE_RANGES: [(usize, usize); 49] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 62),
    (62, 63),
    (64, 65),
    (66, 67),
    (68, 69),
    (70, 75),
    (78, 82),
    (83, 86),
    (86, 87),
    (90, 94),
    (95, 96),
    (97, 98),
    (99, 100),
    (100, 101),
    (102, 105),
    (105, 106),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (64, 65),
    (64, 65),
    (68, 69),
    (68, 69),
    (64, 69),
    (64, 69),
    (83, 86),
    (83, 86),
    (83, 86),
    (78, 87),
    (95, 96),
    (95, 96),
    (99, 100),
    (99, 100),
    (95, 100),
    (95, 100),
    (95, 100),
    (90, 101),
    (70, 105),
    (19, 106),
    (0, 106),
    (0, 106),
    (0, 106),
];

const TASK258B3M2A_TOKEN_TEXTS: [&str; 22] = [
    "reserve",
    "x",
    "for",
    "set",
    ";",
    "theorem",
    "FormulaStatementNumeralWitnessSmoke",
    ":",
    "x",
    "=",
    "x",
    "proof",
    "take",
    "101",
    ";",
    "thus",
    "x",
    "=",
    "x",
    ";",
    "end",
    ";",
];

fn task258b3m2a_token_kind(index: usize) -> SurfaceTokenKind {
    match index {
        0 | 2 | 3 | 5 | 11 | 12 | 15 | 20 => SurfaceTokenKind::ReservedWord,
        1 | 6 | 8 | 10 | 16 | 18 => SurfaceTokenKind::Identifier,
        13 => SurfaceTokenKind::Numeral,
        4 | 7 | 9 | 14 | 17 | 19 | 21 => SurfaceTokenKind::ReservedSymbol,
        _ => unreachable!("Task258B3M2A has exactly 22 tokens"),
    }
}

fn task258b3m2a_surface_kind(index: usize) -> SyntaxKind {
    match index {
        0..=21 => SyntaxKind::Token,
        22 => SyntaxKind::TypeHead,
        23 => SyntaxKind::TypeExpression,
        24 => SyntaxKind::ReserveSegment,
        25 => SyntaxKind::ReserveItem,
        26 | 28 | 36 | 38 => SyntaxKind::TermReference,
        27 | 29 | 33 | 37 | 39 => SyntaxKind::TermExpression,
        30 | 40 => SyntaxKind::BuiltinPredicateApplication,
        31 | 41 => SyntaxKind::FormulaExpression,
        32 => SyntaxKind::NumeralTerm,
        34 => SyntaxKind::Witness,
        35 => SyntaxKind::TakeStatement,
        42 => SyntaxKind::Proposition,
        43 => SyntaxKind::ConclusionStatement,
        44 => SyntaxKind::ProofBlock,
        45 => SyntaxKind::TheoremItem,
        46 => SyntaxKind::ItemList,
        47 => SyntaxKind::CompilationUnit,
        48 => SyntaxKind::Root,
        _ => unreachable!("Task258B3M2A has exactly 49 nodes"),
    }
}

fn task258b3m2a_surface_children(index: usize) -> &'static [usize] {
    match index {
        22 => &[3],
        23 => &[22],
        24 => &[1, 2, 23],
        25 => &[0, 24, 4],
        26 => &[8],
        27 => &[26],
        28 => &[10],
        29 => &[28],
        30 => &[27, 9, 29],
        31 => &[30],
        32 => &[13],
        33 => &[32],
        34 => &[33],
        35 => &[12, 34, 14],
        36 => &[16],
        37 => &[36],
        38 => &[18],
        39 => &[38],
        40 => &[37, 17, 39],
        41 => &[40],
        42 => &[41],
        43 => &[15, 42, 19],
        44 => &[11, 35, 43, 20],
        45 => &[5, 6, 7, 31, 44, 21],
        46 => &[25, 45],
        47 => &[46],
        48 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 47,
        ],
        _ => &[],
    }
}

fn exact_numeral_witness_surface_profile(ast: &SurfaceAst) -> bool {
    TASK258B3M2A_SURFACE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            ast.nodes().get(index).is_some_and(|node| {
                let exact_token = if index < TASK258B3M2A_TOKEN_TEXTS.len() {
                    matches!(
                        &node.kind,
                        SurfaceNodeKind::Token(token)
                            if token.kind == task258b3m2a_token_kind(index)
                                && token.text.as_ref() == TASK258B3M2A_TOKEN_TEXTS[index]
                    )
                } else {
                    true
                };
                exact_token
                    && node.range == range(ast.source_id, start, end)
                    && node.kind.syntax_kind() == task258b3m2a_surface_kind(index)
                    && node
                        .children
                        .iter()
                        .map(|child| child.index())
                        .eq(task258b3m2a_surface_children(index).iter().copied())
            })
        })
}

pub(in crate::runner) fn extract_parenthesized_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B1Extraction> {
    if source_text != SOURCE_STATEMENT_B3M2B1_TEXT
        || source_text.len() != 113
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 53
        || ast.root()?.index() != 52
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
        || !exact_parenthesized_witness_surface_profile(ast)
    {
        return None;
    }
    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 112)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 76, 111)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 84, 93)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 89, 92)?;
    let (parenthesized_id, parenthesized) =
        exact_surface_node(ast, SurfaceNodeKind::ParenthesizedTerm, 89, 92)?;
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 96, 107)?;
    if item_children.len() != 2
        || item_children[0].index() != 27
        || item_children[1] != theorem_id
        || theorem_id.index() != 49
        || proof_id.index() != 48
        || take_id.index() != 39
        || witness_id.index() != 38
        || parenthesized_id.index() != 36
        || conclusion_id.index() != 47
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3M2B1_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ";"]
        || !direct_token_texts(ast, witness).is_empty()
        || direct_token_texts(ast, parenthesized).as_slice() != ["(", ")"]
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, parenthesized_id)
        || !surface_is_descendant(ast, proof_id, conclusion_id)
        || surface_is_descendant(ast, conclusion_id, take_id)
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::TakeStatement).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::Witness).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ParenthesizedTerm).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement).len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::CompactStatement).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::Reference).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::JustificationClause).is_empty()
    {
        return None;
    }
    let label = ast.nodes().get(6)?;
    if label.range.start != 27
        || label.range.end != 68
        || label.token_text() != Some(SOURCE_STATEMENT_B3M2B1_LABEL)
    {
        return None;
    }
    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(70, 75), (101, 106)];
    let expected_formula_ids = [32, 44];
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
            || surface_is_descendant(ast, take_id, id)
        {
            return None;
        }
        formula_ids.push(id);
    }
    let root_ranges = [(70, 71), (74, 75), (89, 92), (101, 102), (105, 106)];
    let expected_root_ids = [28, 30, 36, 40, 42];
    let mut term_sites = Vec::new();
    for (index, (start, end)) in root_ranges.into_iter().enumerate() {
        let kind = if index == 2 {
            SurfaceNodeKind::ParenthesizedTerm
        } else {
            SurfaceNodeKind::TermReference
        };
        let (id, term) = exact_surface_node(ast, kind, start, end)?;
        if id.index() != expected_root_ids[index]
            || (index == 2 && direct_token_texts(ast, term).as_slice() != ["(", ")"])
            || (index != 2 && direct_token_texts(ast, term).as_slice() != ["x"])
            || (index == 2 && !surface_is_descendant(ast, witness_id, id))
            || (index < 2 && !surface_is_descendant(ast, theorem_id, id))
            || (index >= 3 && !surface_is_descendant(ast, conclusion_id, id))
            || (index != 2 && surface_is_descendant(ast, witness_id, id))
        {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let (child_id, child) = exact_surface_node(ast, SurfaceNodeKind::TermReference, 90, 91)?;
    if child_id.index() != 34
        || direct_token_texts(ast, child).as_slice() != ["x"]
        || !surface_is_descendant(ast, parenthesized_id, child_id)
    {
        return None;
    }
    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3M2B1Extraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [range(ast.source_id, 19, 112), range(ast.source_id, 96, 107)],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: [
            range(ast.source_id, 70, 71),
            range(ast.source_id, 74, 75),
            range(ast.source_id, 89, 92),
            range(ast.source_id, 90, 91),
            range(ast.source_id, 101, 102),
            range(ast.source_id, 105, 106),
        ],
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        proof_range: proof.range,
    })
}

const TASK258B3M2B1_SURFACE_RANGES: [(usize, usize); 53] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 68),
    (68, 69),
    (70, 71),
    (72, 73),
    (74, 75),
    (76, 81),
    (84, 88),
    (89, 90),
    (90, 91),
    (91, 92),
    (92, 93),
    (96, 100),
    (101, 102),
    (103, 104),
    (105, 106),
    (106, 107),
    (108, 111),
    (111, 112),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (70, 71),
    (70, 71),
    (74, 75),
    (74, 75),
    (70, 75),
    (70, 75),
    (90, 91),
    (90, 91),
    (89, 92),
    (89, 92),
    (89, 92),
    (84, 93),
    (101, 102),
    (101, 102),
    (105, 106),
    (105, 106),
    (101, 106),
    (101, 106),
    (101, 106),
    (96, 107),
    (76, 111),
    (19, 112),
    (0, 112),
    (0, 112),
    (0, 112),
];

const TASK258B3M2B1_TOKEN_TEXTS: [&str; 24] = [
    "reserve",
    "x",
    "for",
    "set",
    ";",
    "theorem",
    "FormulaStatementParenthesizedWitnessSmoke",
    ":",
    "x",
    "=",
    "x",
    "proof",
    "take",
    "(",
    "x",
    ")",
    ";",
    "thus",
    "x",
    "=",
    "x",
    ";",
    "end",
    ";",
];

fn task258b3m2b1_token_kind(index: usize) -> SurfaceTokenKind {
    match index {
        0 | 2 | 3 | 5 | 11 | 12 | 17 | 22 => SurfaceTokenKind::ReservedWord,
        1 | 6 | 8 | 10 | 14 | 18 | 20 => SurfaceTokenKind::Identifier,
        4 | 7 | 9 | 13 | 15 | 16 | 19 | 21 | 23 => SurfaceTokenKind::ReservedSymbol,
        _ => unreachable!("Task258B3M2B1 has exactly 24 tokens"),
    }
}

fn task258b3m2b1_surface_kind(index: usize) -> SyntaxKind {
    match index {
        0..=23 => SyntaxKind::Token,
        24 => SyntaxKind::TypeHead,
        25 => SyntaxKind::TypeExpression,
        26 => SyntaxKind::ReserveSegment,
        27 => SyntaxKind::ReserveItem,
        28 | 30 | 34 | 40 | 42 => SyntaxKind::TermReference,
        29 | 31 | 35 | 37 | 41 | 43 => SyntaxKind::TermExpression,
        32 | 44 => SyntaxKind::BuiltinPredicateApplication,
        33 | 45 => SyntaxKind::FormulaExpression,
        36 => SyntaxKind::ParenthesizedTerm,
        38 => SyntaxKind::Witness,
        39 => SyntaxKind::TakeStatement,
        46 => SyntaxKind::Proposition,
        47 => SyntaxKind::ConclusionStatement,
        48 => SyntaxKind::ProofBlock,
        49 => SyntaxKind::TheoremItem,
        50 => SyntaxKind::ItemList,
        51 => SyntaxKind::CompilationUnit,
        52 => SyntaxKind::Root,
        _ => unreachable!("Task258B3M2B1 has exactly 53 nodes"),
    }
}

fn task258b3m2b1_surface_children(index: usize) -> &'static [usize] {
    match index {
        24 => &[3],
        25 => &[24],
        26 => &[1, 2, 25],
        27 => &[0, 26, 4],
        28 => &[8],
        29 => &[28],
        30 => &[10],
        31 => &[30],
        32 => &[29, 9, 31],
        33 => &[32],
        34 => &[14],
        35 => &[34],
        36 => &[13, 35, 15],
        37 => &[36],
        38 => &[37],
        39 => &[12, 38, 16],
        40 => &[18],
        41 => &[40],
        42 => &[20],
        43 => &[42],
        44 => &[41, 19, 43],
        45 => &[44],
        46 => &[45],
        47 => &[17, 46, 21],
        48 => &[11, 39, 47, 22],
        49 => &[5, 6, 7, 33, 48, 23],
        50 => &[27, 49],
        51 => &[50],
        52 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            51,
        ],
        _ => &[],
    }
}

fn exact_parenthesized_witness_surface_profile(ast: &SurfaceAst) -> bool {
    TASK258B3M2B1_SURFACE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            ast.nodes().get(index).is_some_and(|node| {
                let exact_token = if index < TASK258B3M2B1_TOKEN_TEXTS.len() {
                    matches!(
                        &node.kind,
                        SurfaceNodeKind::Token(token)
                            if token.kind == task258b3m2b1_token_kind(index)
                                && token.text.as_ref() == TASK258B3M2B1_TOKEN_TEXTS[index]
                    )
                } else {
                    true
                };
                exact_token
                    && node.range == range(ast.source_id, start, end)
                    && node.kind.syntax_kind() == task258b3m2b1_surface_kind(index)
                    && node
                        .children
                        .iter()
                        .map(|child| child.index())
                        .eq(task258b3m2b1_surface_children(index).iter().copied())
            })
        })
}

pub(in crate::runner) fn extract_nested_parenthesized_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2AExtraction> {
    if source_text != SOURCE_STATEMENT_B3M2B2A_TEXT
        || source_text.len() != 121
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 57
        || ast.root()?.index() != 56
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
        || !exact_nested_parenthesized_witness_surface_profile(ast)
    {
        return None;
    }
    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 120)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 82, 119)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 90, 101)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 95, 100)?;
    let parenthesized = surface_nodes_with_kind(ast, SurfaceNodeKind::ParenthesizedTerm);
    let [(inner_id, inner), (outer_id, outer)] = parenthesized.as_slice() else {
        return None;
    };
    let (inner_id, inner, outer_id, outer) = if inner.range.start == 96 {
        (*inner_id, *inner, *outer_id, *outer)
    } else {
        (*outer_id, *outer, *inner_id, *inner)
    };
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 104, 115)?;
    if item_children.len() != 2
        || item_children[0].index() != 29
        || item_children[1] != theorem_id
        || theorem_id.index() != 53
        || proof_id.index() != 52
        || take_id.index() != 43
        || witness_id.index() != 42
        || outer_id.index() != 40
        || inner_id.index() != 38
        || conclusion_id.index() != 51
        || outer.range != range(ast.source_id, 95, 100)
        || inner.range != range(ast.source_id, 96, 99)
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3M2B2A_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ";"]
        || !direct_token_texts(ast, witness).is_empty()
        || direct_token_texts(ast, outer).as_slice() != ["(", ")"]
        || direct_token_texts(ast, inner).as_slice() != ["(", ")"]
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, outer_id)
        || !surface_is_descendant(ast, outer_id, inner_id)
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
        || label.range.end != 74
        || label.token_text() != Some(SOURCE_STATEMENT_B3M2B2A_LABEL)
    {
        return None;
    }
    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(76, 81), (109, 114)];
    let expected_formula_ids = [34, 48];
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
            || surface_is_descendant(ast, take_id, id)
        {
            return None;
        }
        formula_ids.push(id);
    }
    let root_ranges = [(76, 77), (80, 81), (95, 100), (109, 110), (113, 114)];
    let expected_root_ids = [30, 32, 40, 44, 46];
    let mut term_sites = Vec::new();
    for (index, (start, end)) in root_ranges.into_iter().enumerate() {
        let kind = if index == 2 {
            SurfaceNodeKind::ParenthesizedTerm
        } else {
            SurfaceNodeKind::TermReference
        };
        let (id, term) = exact_surface_node(ast, kind, start, end)?;
        if id.index() != expected_root_ids[index]
            || (index == 2 && direct_token_texts(ast, term).as_slice() != ["(", ")"])
            || (index != 2 && direct_token_texts(ast, term).as_slice() != ["x"])
            || (index == 2 && !surface_is_descendant(ast, witness_id, id))
            || (index < 2 && !surface_is_descendant(ast, theorem_id, id))
            || (index >= 3 && !surface_is_descendant(ast, conclusion_id, id))
            || (index != 2 && surface_is_descendant(ast, witness_id, id))
        {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let (child_id, child) = exact_surface_node(ast, SurfaceNodeKind::TermReference, 97, 98)?;
    if child_id.index() != 36
        || direct_token_texts(ast, child).as_slice() != ["x"]
        || !surface_is_descendant(ast, inner_id, child_id)
    {
        return None;
    }
    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3M2B2AExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [
            range(ast.source_id, 19, 120),
            range(ast.source_id, 104, 115),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: [
            range(ast.source_id, 76, 77),
            range(ast.source_id, 80, 81),
            range(ast.source_id, 95, 100),
            range(ast.source_id, 96, 99),
            range(ast.source_id, 97, 98),
            range(ast.source_id, 109, 110),
            range(ast.source_id, 113, 114),
        ],
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        proof_range: proof.range,
    })
}

pub(in crate::runner) fn extract_application_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B1AExtraction> {
    if source_text != SOURCE_STATEMENT_B3M2B2B1A_TEXT
        || source_text.len() != 143
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 63
        || ast.root()?.index() != 62
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
    {
        return None;
    }
    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (import_id, import) = exact_surface_node(ast, SurfaceNodeKind::ImportItem, 0, 28)?;
    let (alias_id, alias) = exact_surface_node(ast, SurfaceNodeKind::ImportAliasDecl, 7, 27)?;
    let (module_path_id, module_path) =
        exact_surface_node(ast, SurfaceNodeKind::ModulePath, 7, 27)?;
    let (reserve_id, _) = exact_surface_node(ast, SurfaceNodeKind::ReserveItem, 29, 47)?;
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 48, 142)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 103, 141)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 111, 123)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 116, 122)?;
    let (transparent_id, transparent) =
        exact_surface_node(ast, SurfaceNodeKind::TermExpression, 116, 122)?;
    let transparent_children = structural_child_ids(ast, transparent);
    let [application_id] = transparent_children.as_slice() else {
        return None;
    };
    let application_id = *application_id;
    let application = ast.node(application_id)?;
    let (left_argument_id, left_argument) =
        exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 116, 117)?;
    let (right_argument_id, right_argument) =
        exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 121, 122)?;
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 126, 137)?;
    if item_children != [import_id, reserve_id, theorem_id]
        || theorem_id.index() != 59
        || proof_id.index() != 58
        || take_id.index() != 49
        || witness_id.index() != 48
        || transparent_id.index() != 47
        || application_id.index() != 46
        || application.range != range(ast.source_id, 116, 122)
        || !matches!(
            &application.kind,
            SurfaceNodeKind::InfixExpression(operator)
                if operator.spelling.as_ref() == "++"
        )
        || left_argument_id.index() != 44
        || right_argument_id.index() != 45
        || conclusion_id.index() != 57
        || direct_token_texts(ast, import).as_slice() != ["import", ";"]
        || !direct_token_texts(ast, alias).is_empty()
        || direct_token_texts(ast, module_path).as_slice() != ["."]
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3M2B2B1A_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ";"]
        || !direct_token_texts(ast, witness).is_empty()
        || !direct_token_texts(ast, transparent).is_empty()
        || direct_token_texts(ast, application).as_slice() != ["++"]
        || direct_token_texts(ast, left_argument).as_slice() != ["1"]
        || direct_token_texts(ast, right_argument).as_slice() != ["2"]
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || structural_child_ids(ast, import) != [alias_id]
        || structural_child_ids(ast, alias) != [module_path_id]
        || structural_child_ids(ast, witness) != [transparent_id]
        || transparent_children != [application_id]
        || !application.children.iter().map(|child| child.index()).eq([
            left_argument_id.index(),
            19,
            right_argument_id.index(),
        ])
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, transparent_id)
        || !surface_is_descendant(ast, transparent_id, application_id)
        || !surface_is_descendant(ast, application_id, left_argument_id)
        || !surface_is_descendant(ast, application_id, right_argument_id)
        || !surface_is_descendant(ast, proof_id, conclusion_id)
        || surface_is_descendant(ast, conclusion_id, take_id)
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ImportItem).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::TakeStatement).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::Witness).len() != 1
        || ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::InfixExpression(_)))
            .count()
            != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement).len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::CompactStatement).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::Reference).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::JustificationClause).is_empty()
    {
        return None;
    }
    let label = ast.nodes().get(11)?;
    if label.range != range(ast.source_id, 56, 95)
        || label.token_text() != Some(SOURCE_STATEMENT_B3M2B2B1A_LABEL)
    {
        return None;
    }
    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(97, 102), (131, 136)];
    let expected_formula_ids = [42, 54];
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
            || surface_is_descendant(ast, take_id, id)
        {
            return None;
        }
        formula_ids.push(id);
    }
    let root_ranges = [
        (97, 98),
        (101, 102),
        (116, 117),
        (121, 122),
        (131, 132),
        (135, 136),
    ];
    let expected_root_ids = [38, 40, 44, 45, 50, 52];
    let root_kinds = [
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::NumeralTerm,
        SurfaceNodeKind::NumeralTerm,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
    ];
    let expected_tokens = ["x", "x", "1", "2", "x", "x"];
    let mut term_sites = Vec::new();
    for index in 0..root_ranges.len() {
        let (start, end) = root_ranges[index];
        let (id, term) = exact_surface_node(ast, root_kinds[index].clone(), start, end)?;
        if id.index() != expected_root_ids[index]
            || direct_token_texts(ast, term).as_slice() != [expected_tokens[index]]
        {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3M2B2B1AExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [
            range(ast.source_id, 48, 142),
            range(ast.source_id, 126, 137),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: root_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        application_node: application_id.index(),
        proof_range: proof.range,
    })
}

pub(in crate::runner) fn extract_wrapped_application_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B1B1Extraction> {
    if source_text != SOURCE_STATEMENT_B3M2B2B1B1_TEXT
        || source_text.len() != 158
        || !source_text.ends_with('\n')
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
    {
        return None;
    }

    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 48, 157)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 116, 156)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 124, 138)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 129, 137)?;
    let (wrapper_id, wrapper) =
        exact_surface_node(ast, SurfaceNodeKind::ParenthesizedTerm, 129, 137)?;
    let wrapper_children = structural_child_ids(ast, wrapper);
    let [body_id] = wrapper_children.as_slice() else {
        return None;
    };
    let body = ast.node(*body_id)?;
    let body_children = structural_child_ids(ast, body);
    let [application_id] = body_children.as_slice() else {
        return None;
    };
    let application_id = *application_id;
    let application = ast.node(application_id)?;
    let (conclusion_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 141, 152)?;
    let label = ast.nodes().get(11)?;
    if theorem_id.index() != 63
        || proof_id.index() != 62
        || take_id.index() != 53
        || witness_id.index() != 52
        || wrapper_id.index() != 50
        || application_id.index() != 48
        || conclusion_id.index() != 61
        || label.range != range(ast.source_id, 56, 108)
        || label.token_text() != Some(SOURCE_STATEMENT_B3M2B2B1B1_LABEL)
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3M2B2B1B1_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ";"]
        || !direct_token_texts(ast, witness).is_empty()
        || direct_token_texts(ast, wrapper).as_slice() != ["(", ")"]
        || application.range != range(ast.source_id, 130, 136)
        || !matches!(
            &application.kind,
            SurfaceNodeKind::InfixExpression(operator)
                if operator.spelling.as_ref() == "++"
        )
        || direct_token_texts(ast, application).as_slice() != ["++"]
    {
        return None;
    }
    let statement_ids = [theorem_id, conclusion_id];
    let formula_ids = [
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 110, 115)?.0,
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 146, 151)?.0,
    ];
    if formula_ids.map(|id| id.index()) != [44, 58] {
        return None;
    }
    let root_ranges = [
        (110, 111),
        (114, 115),
        (130, 131),
        (135, 136),
        (146, 147),
        (150, 151),
    ];
    let expected_root_ids = [40, 42, 46, 47, 54, 56];
    let root_kinds = [
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::NumeralTerm,
        SurfaceNodeKind::NumeralTerm,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
    ];
    let expected_tokens = ["x", "x", "1", "2", "x", "x"];
    let mut term_sites = Vec::with_capacity(root_ranges.len());
    for index in 0..root_ranges.len() {
        let (start, end) = root_ranges[index];
        let (id, term) = exact_surface_node(ast, root_kinds[index].clone(), start, end)?;
        if id.index() != expected_root_ids[index]
            || direct_token_texts(ast, term).as_slice() != [expected_tokens[index]]
        {
            return None;
        }
        term_sites.push(surface_site(id));
    }

    Some(SourceStatementB3M2B2B1B1Extraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [
            range(ast.source_id, 48, 157),
            range(ast.source_id, 141, 152),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: [
            range(ast.source_id, 110, 115),
            range(ast.source_id, 146, 151),
        ],
        term_sites: term_sites.try_into().ok()?,
        term_ranges: root_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        application_node: application_id.index(),
        application_wrapper_node: wrapper_id.index(),
        proof_range: proof.range,
    })
}

pub(in crate::runner) fn extract_structure_constructor_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B2AExtraction> {
    if source_text != SOURCE_STATEMENT_B3M2B2B2A_TEXT
        || source_text.len() != 172
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 76
        || ast.root()?.index() != 75
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
    {
        return None;
    }

    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (import_id, import) = exact_surface_node(ast, SurfaceNodeKind::ImportItem, 0, 28)?;
    let (reserve_id, _) = exact_surface_node(ast, SurfaceNodeKind::ReserveItem, 29, 47)?;
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 48, 171)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 112, 170)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 120, 152)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 125, 151)?;
    let (transparent_id, transparent) =
        exact_surface_node(ast, SurfaceNodeKind::TermExpression, 125, 151)?;
    let (structure_id, structure) =
        exact_surface_node(ast, SurfaceNodeKind::StructureConstructor, 125, 151)?;
    let (left_member_id, left_member) =
        exact_surface_node(ast, SurfaceNodeKind::FieldArgument, 140, 144)?;
    let (right_member_id, right_member) =
        exact_surface_node(ast, SurfaceNodeKind::FieldArgument, 146, 150)?;
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 155, 166)?;
    let label = ast.nodes().get(11)?;
    if item_children != [import_id, reserve_id, theorem_id]
        || import_id.index() != 40
        || reserve_id.index() != 44
        || theorem_id.index() != 72
        || proof_id.index() != 71
        || take_id.index() != 62
        || witness_id.index() != 61
        || transparent_id.index() != 60
        || structure_id.index() != 59
        || left_member_id.index() != 55
        || right_member_id.index() != 58
        || conclusion_id.index() != 70
        || label.range != range(ast.source_id, 56, 104)
        || label.token_text() != Some(SOURCE_STATEMENT_B3M2B2B2A_LABEL)
        || direct_token_texts(ast, import).as_slice() != ["import", ";"]
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3M2B2B2A_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ";"]
        || !direct_token_texts(ast, witness).is_empty()
        || !direct_token_texts(ast, transparent).is_empty()
        || direct_token_texts(ast, structure).as_slice() != ["(", ",", ")"]
        || direct_token_texts(ast, left_member).as_slice() != ["x", ":"]
        || direct_token_texts(ast, right_member).as_slice() != ["y", ":"]
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || structural_child_ids(ast, witness) != [transparent_id]
        || structural_child_ids(ast, transparent) != [structure_id]
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, transparent_id)
        || !surface_is_descendant(ast, transparent_id, structure_id)
        || !surface_is_descendant(ast, structure_id, left_member_id)
        || !surface_is_descendant(ast, structure_id, right_member_id)
        || !surface_is_descendant(ast, proof_id, conclusion_id)
        || surface_is_descendant(ast, conclusion_id, take_id)
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ImportItem).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::TakeStatement).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::Witness).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::StructureConstructor).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::FieldArgument).len() != 2
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement).len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::CompactStatement).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::Reference).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::JustificationClause).is_empty()
    {
        return None;
    }

    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(106, 111), (160, 165)];
    let expected_formula_ids = [49, 67];
    let mut formula_ids = Vec::with_capacity(2);
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
            || surface_is_descendant(ast, take_id, id)
        {
            return None;
        }
        formula_ids.push(id);
    }

    let root_ranges = [
        (106, 107),
        (110, 111),
        (143, 144),
        (149, 150),
        (160, 161),
        (164, 165),
    ];
    let expected_root_ids = [45, 47, 54, 57, 63, 65];
    let root_kinds = [
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermExpression,
        SurfaceNodeKind::TermExpression,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
    ];
    let mut term_sites = Vec::with_capacity(root_ranges.len());
    for index in 0..root_ranges.len() {
        let (start, end) = root_ranges[index];
        let (id, term) = exact_surface_node(ast, root_kinds[index].clone(), start, end)?;
        let token_is_exact = if matches!(root_kinds[index], SurfaceNodeKind::TermReference) {
            direct_token_texts(ast, term).as_slice() == ["x"]
        } else {
            direct_token_texts(ast, term).is_empty()
        };
        if id.index() != expected_root_ids[index] || !token_is_exact {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let (left_value_id, left_value) =
        exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 143, 144)?;
    let (right_value_id, right_value) =
        exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 149, 150)?;
    if left_value_id.index() != 53
        || right_value_id.index() != 56
        || direct_token_texts(ast, left_value).as_slice() != ["1"]
        || direct_token_texts(ast, right_value).as_slice() != ["2"]
        || structural_child_ids(ast, ast.nodes().get(54)?) != [left_value_id]
        || structural_child_ids(ast, ast.nodes().get(57)?) != [right_value_id]
    {
        return None;
    }

    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3M2B2B2AExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [
            range(ast.source_id, 48, 171),
            range(ast.source_id, 155, 166),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: root_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        structure_node: structure_id.index(),
        proof_range: proof.range,
    })
}

pub(in crate::runner) fn extract_structure_selector_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B2BExtraction> {
    if source_text != SOURCE_STATEMENT_B3M2B2B2B_TEXT
        || source_text.len() != 171
        || !source_text.ends_with('\n')
    {
        return None;
    }

    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (import_id, import) = exact_surface_node(ast, SurfaceNodeKind::ImportItem, 0, 28)?;
    let (reserve_id, _) = exact_surface_node(ast, SurfaceNodeKind::ReserveItem, 29, 47)?;
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 48, 170)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 109, 169)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 117, 151)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 122, 150)?;
    let (transparent_id, transparent) =
        exact_surface_node(ast, SurfaceNodeKind::TermExpression, 122, 150)?;
    let (selector_id, selector) =
        exact_surface_node(ast, SurfaceNodeKind::SelectorAccess, 122, 150)?;
    let (structure_id, structure) =
        exact_surface_node(ast, SurfaceNodeKind::StructureConstructor, 122, 148)?;
    let (left_member_id, left_member) =
        exact_surface_node(ast, SurfaceNodeKind::FieldArgument, 137, 141)?;
    let (right_member_id, right_member) =
        exact_surface_node(ast, SurfaceNodeKind::FieldArgument, 143, 147)?;
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 154, 165)?;
    let label = ast.nodes().get(11)?;
    if item_children != [import_id, reserve_id, theorem_id]
        || import_id.index() != 42
        || reserve_id.index() != 46
        || theorem_id.index() != 75
        || proof_id.index() != 74
        || take_id.index() != 65
        || witness_id.index() != 64
        || transparent_id.index() != 63
        || selector_id.index() != 62
        || structure_id.index() != 61
        || left_member_id.index() != 57
        || right_member_id.index() != 60
        || conclusion_id.index() != 73
        || label.range != range(ast.source_id, 56, 101)
        || label.token_text() != Some(SOURCE_STATEMENT_B3M2B2B2B_LABEL)
        || direct_token_texts(ast, import).as_slice() != ["import", ";"]
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3M2B2B2B_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ";"]
        || !direct_token_texts(ast, witness).is_empty()
        || !direct_token_texts(ast, transparent).is_empty()
        || direct_token_texts(ast, selector).as_slice() != [".", "x"]
        || direct_token_texts(ast, structure).as_slice() != ["(", ",", ")"]
        || direct_token_texts(ast, left_member).as_slice() != ["x", ":"]
        || direct_token_texts(ast, right_member).as_slice() != ["y", ":"]
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || structural_child_ids(ast, witness) != [transparent_id]
        || structural_child_ids(ast, transparent) != [selector_id]
        || structural_child_ids(ast, selector) != [structure_id]
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, transparent_id)
        || !surface_is_descendant(ast, transparent_id, selector_id)
        || !surface_is_descendant(ast, selector_id, structure_id)
        || !surface_is_descendant(ast, structure_id, left_member_id)
        || !surface_is_descendant(ast, structure_id, right_member_id)
        || !surface_is_descendant(ast, proof_id, conclusion_id)
        || surface_is_descendant(ast, conclusion_id, take_id)
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ImportItem).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::TakeStatement).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::Witness).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::SelectorAccess).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::StructureConstructor).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::FieldArgument).len() != 2
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement).len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::CompactStatement).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::Reference).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::JustificationClause).is_empty()
    {
        return None;
    }

    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(103, 108), (159, 164)];
    let expected_formula_ids = [51, 70];
    let mut formula_ids = Vec::with_capacity(2);
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
            || surface_is_descendant(ast, take_id, id)
        {
            return None;
        }
        formula_ids.push(id);
    }

    let root_ranges = [
        (103, 104),
        (107, 108),
        (140, 141),
        (146, 147),
        (159, 160),
        (163, 164),
    ];
    let expected_root_ids = [47, 49, 56, 59, 66, 68];
    let root_kinds = [
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermExpression,
        SurfaceNodeKind::TermExpression,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
    ];
    let mut term_sites = Vec::with_capacity(root_ranges.len());
    for index in 0..root_ranges.len() {
        let (start, end) = root_ranges[index];
        let (id, term) = exact_surface_node(ast, root_kinds[index].clone(), start, end)?;
        let token_is_exact = if matches!(root_kinds[index], SurfaceNodeKind::TermReference) {
            direct_token_texts(ast, term).as_slice() == ["x"]
        } else {
            direct_token_texts(ast, term).is_empty()
        };
        if id.index() != expected_root_ids[index] || !token_is_exact {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    let (left_value_id, left_value) =
        exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 140, 141)?;
    let (right_value_id, right_value) =
        exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 146, 147)?;
    if left_value_id.index() != 55
        || right_value_id.index() != 58
        || direct_token_texts(ast, left_value).as_slice() != ["1"]
        || direct_token_texts(ast, right_value).as_slice() != ["2"]
        || structural_child_ids(ast, ast.nodes().get(56)?) != [left_value_id]
        || structural_child_ids(ast, ast.nodes().get(59)?) != [right_value_id]
    {
        return None;
    }

    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3M2B2B2BExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [
            range(ast.source_id, 48, 170),
            range(ast.source_id, 154, 165),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: root_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        selector_node: selector_id.index(),
        proof_range: proof.range,
    })
}

pub(in crate::runner) fn extract_structure_update_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B2CExtraction> {
    if source_text != SOURCE_STATEMENT_B3M2B2B2C_TEXT
        || source_text.len() != 181
        || !source_text.ends_with('\n')
        || ast.nodes().len() != 86
        || ast.root()?.index() != 85
        || ast
            .nodes()
            .iter()
            .any(|node| node.recovered || node.range.source_id != ast.source_id)
    {
        return None;
    }

    let item_list = super::source_ast::exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let (import_id, import) = exact_surface_node(ast, SurfaceNodeKind::ImportItem, 0, 28)?;
    let (reserve_id, _) = exact_surface_node(ast, SurfaceNodeKind::ReserveItem, 29, 47)?;
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 48, 180)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 107, 179)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 115, 161)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 120, 160)?;
    let (transparent_id, transparent) =
        exact_surface_node(ast, SurfaceNodeKind::TermExpression, 120, 160)?;
    let (update_id, update) = exact_surface_node(ast, SurfaceNodeKind::StructureUpdate, 120, 160)?;
    let (constructor_id, constructor) =
        exact_surface_node(ast, SurfaceNodeKind::StructureConstructor, 120, 146)?;
    let (field_update_id, field_update) =
        exact_surface_node(ast, SurfaceNodeKind::FieldUpdate, 153, 159)?;
    let (conclusion_id, conclusion) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 164, 175)?;
    let label = ast.nodes().get(11)?;
    if item_children != [import_id, reserve_id, theorem_id]
        || import_id.index() != 46
        || reserve_id.index() != 50
        || theorem_id.index() != 82
        || proof_id.index() != 81
        || take_id.index() != 72
        || witness_id.index() != 71
        || transparent_id.index() != 70
        || update_id.index() != 69
        || constructor_id.index() != 65
        || field_update_id.index() != 68
        || conclusion_id.index() != 80
        || label.range != range(ast.source_id, 56, 99)
        || label.token_text() != Some(SOURCE_STATEMENT_B3M2B2B2C_LABEL)
        || direct_token_texts(ast, import).as_slice() != ["import", ";"]
        || direct_token_texts(ast, theorem).as_slice()
            != ["theorem", SOURCE_STATEMENT_B3M2B2B2C_LABEL, ":", ";"]
        || direct_token_texts(ast, proof).as_slice() != ["proof", "end"]
        || direct_token_texts(ast, take).as_slice() != ["take", ";"]
        || !direct_token_texts(ast, witness).is_empty()
        || !direct_token_texts(ast, transparent).is_empty()
        || direct_token_texts(ast, constructor).as_slice() != ["(", ",", ")"]
        || direct_token_texts(ast, conclusion).as_slice() != ["thus", ";"]
        || structural_child_ids(ast, witness) != [transparent_id]
        || structural_child_ids(ast, transparent) != [update_id]
        || !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, transparent_id)
        || !surface_is_descendant(ast, transparent_id, update_id)
        || !surface_is_descendant(ast, update_id, constructor_id)
        || !surface_is_descendant(ast, update_id, field_update_id)
        || !surface_is_descendant(ast, proof_id, conclusion_id)
        || surface_is_descendant(ast, conclusion_id, take_id)
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ImportItem).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::TakeStatement).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::Witness).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::StructureUpdate).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::StructureConstructor).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::FieldUpdate).len() != 1
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement).len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::CompactStatement).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::Reference).is_empty()
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::JustificationClause).is_empty()
        || update.range != range(ast.source_id, 120, 160)
        || field_update.range != range(ast.source_id, 153, 159)
    {
        return None;
    }

    let statement_ids = [theorem_id, conclusion_id];
    let formula_ranges = [(101, 106), (169, 174)];
    let expected_formula_ids = [55, 77];
    let mut formula_ids = Vec::with_capacity(2);
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
            || surface_is_descendant(ast, take_id, id)
        {
            return None;
        }
        formula_ids.push(id);
    }

    let root_ranges = [
        (101, 102),
        (105, 106),
        (138, 139),
        (144, 145),
        (158, 159),
        (169, 170),
        (173, 174),
    ];
    let expected_root_ids = [51, 53, 60, 63, 67, 73, 75];
    let root_kinds = [
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermExpression,
        SurfaceNodeKind::TermExpression,
        SurfaceNodeKind::TermExpression,
        SurfaceNodeKind::TermReference,
        SurfaceNodeKind::TermReference,
    ];
    let mut term_sites = Vec::with_capacity(root_ranges.len());
    for index in 0..root_ranges.len() {
        let (start, end) = root_ranges[index];
        let (id, term) = exact_surface_node(ast, root_kinds[index].clone(), start, end)?;
        let token_is_exact = if matches!(root_kinds[index], SurfaceNodeKind::TermReference) {
            direct_token_texts(ast, term).as_slice() == ["x"]
        } else {
            direct_token_texts(ast, term).is_empty()
        };
        if id.index() != expected_root_ids[index] || !token_is_exact {
            return None;
        }
        term_sites.push(surface_site(id));
    }
    for (expression, numeral, spelling) in [(60, 59, "1"), (63, 62, "2"), (67, 66, "3")] {
        let numeral_node = ast.nodes().get(numeral)?;
        if !matches!(numeral_node.kind, SurfaceNodeKind::NumeralTerm)
            || direct_token_texts(ast, numeral_node).as_slice() != [spelling]
            || structural_child_ids(ast, ast.nodes().get(expression)?)
                .iter()
                .map(|child| child.index())
                .ne([numeral])
        {
            return None;
        }
    }

    let formula_ids: [mizar_syntax::SurfaceNodeId; 2] = formula_ids.try_into().ok()?;
    Some(SourceStatementB3M2B2B2CExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: label.range,
        statement_sites: statement_ids.map(surface_site),
        statement_ranges: [
            range(ast.source_id, 48, 180),
            range(ast.source_id, 164, 175),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: formula_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        term_sites: term_sites.try_into().ok()?,
        term_ranges: root_ranges.map(|(start, end)| range(ast.source_id, start, end)),
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        update_node: update_id.index(),
        proof_range: proof.range,
    })
}

// Rationale: these variants type the exhaustive test-only surface-profile mutation seam and are intentionally dormant in non-test builds.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceStatementB3M2B2B3ASurfaceMutation {
    None,
    NodeKind(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    RootIdentity,
}

pub(in crate::runner) type SourceStatementB3M2B2B3BSurfaceMutation =
    SourceStatementB3M2B2B3ASurfaceMutation;
pub(in crate::runner) type SourceStatementB3M2B2B3CSurfaceMutation =
    SourceStatementB3M2B2B3ASurfaceMutation;
pub(in crate::runner) type SourceStatementB3M2B2B3DSurfaceMutation =
    SourceStatementB3M2B2B3ASurfaceMutation;
pub(in crate::runner) type SourceStatementB3M2B2B3ESurfaceMutation =
    SourceStatementB3M2B2B3ASurfaceMutation;

fn exact_set_enumeration_witness_surface_profile(ast: &SurfaceAst, source_text: &str) -> bool {
    exact_set_enumeration_witness_surface_profile_with_mutation(
        ast,
        source_text,
        SourceStatementB3M2B2B3ASurfaceMutation::None,
    )
}

fn exact_set_enumeration_witness_surface_profile_with_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3ASurfaceMutation,
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
        SourceStatementB3M2B2B3ASurfaceMutation::None => {}
        SourceStatementB3M2B2B3ASurfaceMutation::NodeKind(index) => {
            if let Some(kind) = kinds.get_mut(index) {
                kind.push('!');
            }
        }
        SourceStatementB3M2B2B3ASurfaceMutation::NodeRange(index) => {
            if let Some(range) = ranges.get_mut(index) {
                range.1 = range.1.saturating_add(1);
            }
        }
        SourceStatementB3M2B2B3ASurfaceMutation::NodeRecovery(index) => {
            if let Some(recovered) = recoveries.get_mut(index) {
                *recovered = !*recovered;
            }
        }
        SourceStatementB3M2B2B3ASurfaceMutation::NodeChildren(index) => {
            if let Some(node_children) = children.get_mut(index) {
                if node_children.len() > 1 {
                    node_children.rotate_left(1);
                } else {
                    node_children.push(index);
                }
            }
        }
        SourceStatementB3M2B2B3ASurfaceMutation::RootIdentity => root = None,
    }
    source_text == SOURCE_STATEMENT_B3M2B2B3A_TEXT
        && source_text.len() == 117
        && source_text.ends_with('\n')
        && ast.nodes().len() == 57
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
pub(in crate::runner) fn extract_set_enumeration_witness_source_statement_with_surface_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3ASurfaceMutation,
) -> Option<SourceStatementB3M2B2B3AExtraction> {
    if !exact_set_enumeration_witness_surface_profile_with_mutation(ast, source_text, mutation) {
        return None;
    }
    extract_set_enumeration_witness_source_statement(ast, source_text)
}

pub(in crate::runner) fn extract_set_enumeration_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B3AExtraction> {
    if !exact_set_enumeration_witness_surface_profile(ast, source_text) {
        return None;
    }
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 116)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 77, 115)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 85, 97)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 90, 96)?;
    let (set_term_id, _) = exact_surface_node(ast, SurfaceNodeKind::SetEnumeration, 90, 96)?;
    let (conclusion_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 100, 111)?;
    let formula_ids = [
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 71, 76)?.0,
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 105, 110)?.0,
    ];
    let term_ids = [
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 71, 72)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 75, 76)?.0,
        exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 91, 92)?.0,
        exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 94, 95)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 105, 106)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 109, 110)?.0,
    ];
    let label_id = ast
        .token_nodes()
        .iter()
        .copied()
        .find(|id| id.index() == 6)?;
    if !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, set_term_id)
        || !surface_is_descendant(ast, theorem_id, formula_ids[0])
        || !surface_is_descendant(ast, proof_id, formula_ids[1])
        || surface_is_descendant(ast, set_term_id, formula_ids[1])
    {
        return None;
    }
    Some(SourceStatementB3M2B2B3AExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: ast.node(label_id)?.range,
        statement_sites: [theorem_id, conclusion_id].map(surface_site),
        statement_ranges: [
            range(ast.source_id, 19, 116),
            range(ast.source_id, 100, 111),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: [range(ast.source_id, 71, 76), range(ast.source_id, 105, 110)],
        term_sites: term_ids.map(surface_site),
        term_ranges: [
            range(ast.source_id, 71, 72),
            range(ast.source_id, 75, 76),
            range(ast.source_id, 91, 92),
            range(ast.source_id, 94, 95),
            range(ast.source_id, 105, 106),
            range(ast.source_id, 109, 110),
        ],
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        set_term_node: set_term_id.index(),
        proof_range: proof.range,
    })
}

fn exact_empty_set_enumeration_witness_surface_profile(
    ast: &SurfaceAst,
    source_text: &str,
) -> bool {
    exact_empty_set_enumeration_witness_surface_profile_with_mutation(
        ast,
        source_text,
        SourceStatementB3M2B2B3BSurfaceMutation::None,
    )
}

fn exact_empty_set_enumeration_witness_surface_profile_with_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3BSurfaceMutation,
) -> bool {
    const KINDS: [&str; 50] = [
        "Token(SurfaceToken { kind: ReservedWord, text: \"reserve\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"FormulaStatementEmptySetEnumerationWitnessSmoke\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"proof\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"take\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"{\" })",
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
    const RANGES: [(usize, usize); 50] = [
        (0, 7),
        (8, 9),
        (10, 13),
        (14, 17),
        (17, 18),
        (19, 26),
        (27, 74),
        (74, 75),
        (76, 77),
        (78, 79),
        (80, 81),
        (82, 87),
        (90, 94),
        (95, 96),
        (96, 97),
        (97, 98),
        (101, 105),
        (106, 107),
        (108, 109),
        (110, 111),
        (111, 112),
        (113, 116),
        (116, 117),
        (14, 17),
        (14, 17),
        (8, 17),
        (0, 18),
        (76, 77),
        (76, 77),
        (80, 81),
        (80, 81),
        (76, 81),
        (76, 81),
        (95, 97),
        (95, 97),
        (95, 97),
        (90, 98),
        (106, 107),
        (106, 107),
        (110, 111),
        (110, 111),
        (106, 111),
        (106, 111),
        (106, 111),
        (101, 112),
        (82, 116),
        (19, 117),
        (0, 117),
        (0, 117),
        (0, 117),
    ];
    const CHILDREN: [&[usize]; 50] = [
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
        &[23],
        &[1, 2, 24],
        &[0, 25, 4],
        &[8],
        &[27],
        &[10],
        &[29],
        &[28, 9, 30],
        &[31],
        &[13, 14],
        &[33],
        &[34],
        &[12, 35, 15],
        &[17],
        &[37],
        &[19],
        &[39],
        &[38, 18, 40],
        &[41],
        &[42],
        &[16, 43, 20],
        &[11, 36, 44, 21],
        &[5, 6, 7, 32, 45, 22],
        &[26, 46],
        &[47],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 48,
        ],
    ];
    let mut kinds = KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let mut ranges = RANGES.to_vec();
    let mut recoveries = [false; 50];
    let mut children = CHILDREN
        .iter()
        .map(|children| children.to_vec())
        .collect::<Vec<_>>();
    let mut root = Some(49);
    match mutation {
        SourceStatementB3M2B2B3BSurfaceMutation::None => {}
        SourceStatementB3M2B2B3BSurfaceMutation::NodeKind(index) => {
            if let Some(kind) = kinds.get_mut(index) {
                kind.push('!');
            }
        }
        SourceStatementB3M2B2B3BSurfaceMutation::NodeRange(index) => {
            if let Some(range) = ranges.get_mut(index) {
                range.1 = range.1.saturating_add(1);
            }
        }
        SourceStatementB3M2B2B3BSurfaceMutation::NodeRecovery(index) => {
            if let Some(recovered) = recoveries.get_mut(index) {
                *recovered = !*recovered;
            }
        }
        SourceStatementB3M2B2B3BSurfaceMutation::NodeChildren(index) => {
            if let Some(node_children) = children.get_mut(index) {
                if node_children.len() > 1 {
                    node_children.rotate_left(1);
                } else {
                    node_children.push(index);
                }
            }
        }
        SourceStatementB3M2B2B3BSurfaceMutation::RootIdentity => root = None,
    }
    source_text == SOURCE_STATEMENT_B3M2B2B3B_TEXT
        && source_text.len() == 118
        && source_text.ends_with('\n')
        && ast.nodes().len() == 50
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
pub(in crate::runner) fn extract_empty_set_enumeration_witness_source_statement_with_surface_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3BSurfaceMutation,
) -> Option<SourceStatementB3M2B2B3BExtraction> {
    if !exact_empty_set_enumeration_witness_surface_profile_with_mutation(
        ast,
        source_text,
        mutation,
    ) {
        return None;
    }
    extract_empty_set_enumeration_witness_source_statement(ast, source_text)
}

pub(in crate::runner) fn extract_empty_set_enumeration_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B3BExtraction> {
    if !exact_empty_set_enumeration_witness_surface_profile(ast, source_text) {
        return None;
    }
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 117)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 82, 116)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 90, 98)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 95, 97)?;
    let (set_term_id, _) = exact_surface_node(ast, SurfaceNodeKind::SetEnumeration, 95, 97)?;
    let (conclusion_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 101, 112)?;
    let formula_ids = [
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 76, 81)?.0,
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 106, 111)?.0,
    ];
    let term_ids = [
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 76, 77)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 80, 81)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 106, 107)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 110, 111)?.0,
    ];
    let label_id = ast
        .token_nodes()
        .iter()
        .copied()
        .find(|id| id.index() == 6)?;
    if !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, set_term_id)
        || !surface_is_descendant(ast, theorem_id, formula_ids[0])
        || !surface_is_descendant(ast, proof_id, formula_ids[1])
        || surface_is_descendant(ast, set_term_id, formula_ids[1])
        || ast
            .node(set_term_id)
            .is_none_or(|node| node.children.iter().map(|id| id.index()).ne([13, 14]))
    {
        return None;
    }
    Some(SourceStatementB3M2B2B3BExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: ast.node(label_id)?.range,
        statement_sites: [theorem_id, conclusion_id].map(surface_site),
        statement_ranges: [
            range(ast.source_id, 19, 117),
            range(ast.source_id, 101, 112),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: [range(ast.source_id, 76, 81), range(ast.source_id, 106, 111)],
        term_sites: term_ids.map(surface_site),
        term_ranges: [
            range(ast.source_id, 76, 77),
            range(ast.source_id, 80, 81),
            range(ast.source_id, 106, 107),
            range(ast.source_id, 110, 111),
        ],
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        set_term_node: set_term_id.index(),
        proof_range: proof.range,
    })
}

fn exact_choice_witness_surface_profile(ast: &SurfaceAst, source_text: &str) -> bool {
    exact_choice_witness_surface_profile_with_mutation(
        ast,
        source_text,
        SourceStatementB3M2B2B3CSurfaceMutation::None,
    )
}

fn exact_choice_witness_surface_profile_with_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3CSurfaceMutation,
) -> bool {
    const KINDS: [&str; 52] = [
        "Token(SurfaceToken { kind: ReservedWord, text: \"reserve\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"FormulaStatementChoiceWitnessSmoke\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"proof\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"take\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"the\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
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
        "TypeHead",
        "TypeExpression",
        "ChoiceTerm",
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
    const RANGES: [(usize, usize); 52] = [
        (0, 7),
        (8, 9),
        (10, 13),
        (14, 17),
        (17, 18),
        (19, 26),
        (27, 61),
        (61, 62),
        (63, 64),
        (65, 66),
        (67, 68),
        (69, 74),
        (77, 81),
        (82, 85),
        (86, 89),
        (89, 90),
        (93, 97),
        (98, 99),
        (100, 101),
        (102, 103),
        (103, 104),
        (105, 108),
        (108, 109),
        (14, 17),
        (14, 17),
        (8, 17),
        (0, 18),
        (63, 64),
        (63, 64),
        (67, 68),
        (67, 68),
        (63, 68),
        (63, 68),
        (86, 89),
        (86, 89),
        (82, 89),
        (82, 89),
        (82, 89),
        (77, 90),
        (98, 99),
        (98, 99),
        (102, 103),
        (102, 103),
        (98, 103),
        (98, 103),
        (98, 103),
        (93, 104),
        (69, 108),
        (19, 109),
        (0, 109),
        (0, 109),
        (0, 109),
    ];
    const CHILDREN: [&[usize]; 52] = [
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
        &[23],
        &[1, 2, 24],
        &[0, 25, 4],
        &[8],
        &[27],
        &[10],
        &[29],
        &[28, 9, 30],
        &[31],
        &[14],
        &[33],
        &[13, 34],
        &[35],
        &[36],
        &[12, 37, 15],
        &[17],
        &[39],
        &[19],
        &[41],
        &[40, 18, 42],
        &[43],
        &[44],
        &[16, 45, 20],
        &[11, 38, 46, 21],
        &[5, 6, 7, 32, 47, 22],
        &[26, 48],
        &[49],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 50,
        ],
    ];
    let mut kinds = KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let mut ranges = RANGES.to_vec();
    let mut recoveries = [false; 52];
    let mut children = CHILDREN
        .iter()
        .map(|children| children.to_vec())
        .collect::<Vec<_>>();
    let mut root = Some(51);
    match mutation {
        SourceStatementB3M2B2B3CSurfaceMutation::None => {}
        SourceStatementB3M2B2B3CSurfaceMutation::NodeKind(index) => {
            if let Some(kind) = kinds.get_mut(index) {
                kind.push('!');
            }
        }
        SourceStatementB3M2B2B3CSurfaceMutation::NodeRange(index) => {
            if let Some(range) = ranges.get_mut(index) {
                range.1 = range.1.saturating_add(1);
            }
        }
        SourceStatementB3M2B2B3CSurfaceMutation::NodeRecovery(index) => {
            if let Some(recovered) = recoveries.get_mut(index) {
                *recovered = !*recovered;
            }
        }
        SourceStatementB3M2B2B3CSurfaceMutation::NodeChildren(index) => {
            if let Some(node_children) = children.get_mut(index) {
                if node_children.len() > 1 {
                    node_children.rotate_left(1);
                } else {
                    node_children.push(index);
                }
            }
        }
        SourceStatementB3M2B2B3CSurfaceMutation::RootIdentity => root = None,
    }
    source_text == SOURCE_STATEMENT_B3M2B2B3C_TEXT
        && source_text.len() == 110
        && source_text.ends_with('\n')
        && ast.nodes().len() == 52
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
pub(in crate::runner) fn extract_choice_witness_source_statement_with_surface_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3CSurfaceMutation,
) -> Option<SourceStatementB3M2B2B3CExtraction> {
    if !exact_choice_witness_surface_profile_with_mutation(ast, source_text, mutation) {
        return None;
    }
    extract_choice_witness_source_statement(ast, source_text)
}

pub(in crate::runner) fn extract_choice_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B3CExtraction> {
    if !exact_choice_witness_surface_profile(ast, source_text) {
        return None;
    }
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 109)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 69, 108)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 77, 90)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 82, 89)?;
    let (choice_id, choice) = exact_surface_node(ast, SurfaceNodeKind::ChoiceTerm, 82, 89)?;
    let (type_expression_id, _) = exact_surface_node(ast, SurfaceNodeKind::TypeExpression, 86, 89)?;
    let (type_head_id, _) = exact_surface_node(ast, SurfaceNodeKind::TypeHead, 86, 89)?;
    let (conclusion_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 93, 104)?;
    let formula_ids = [
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 63, 68)?.0,
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 98, 103)?.0,
    ];
    let term_ids = [
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 63, 64)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 67, 68)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 98, 99)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 102, 103)?.0,
    ];
    let label_id = ast
        .token_nodes()
        .iter()
        .copied()
        .find(|id| id.index() == 6)?;
    if !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, choice_id)
        || !surface_is_descendant(ast, choice_id, type_expression_id)
        || !surface_is_descendant(ast, type_expression_id, type_head_id)
        || !surface_is_descendant(ast, theorem_id, formula_ids[0])
        || !surface_is_descendant(ast, proof_id, formula_ids[1])
        || surface_is_descendant(ast, choice_id, formula_ids[1])
        || ast.node(choice_id).is_none_or(|node| {
            node.children
                .iter()
                .map(|id| id.index())
                .ne([13, type_expression_id.index()])
        })
        || direct_token_texts(ast, choice).as_slice() != ["the"]
    {
        return None;
    }
    Some(SourceStatementB3M2B2B3CExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: ast.node(label_id)?.range,
        statement_sites: [theorem_id, conclusion_id].map(surface_site),
        statement_ranges: [range(ast.source_id, 19, 109), range(ast.source_id, 93, 104)],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: [range(ast.source_id, 63, 68), range(ast.source_id, 98, 103)],
        term_sites: term_ids.map(surface_site),
        term_ranges: [
            range(ast.source_id, 63, 64),
            range(ast.source_id, 67, 68),
            range(ast.source_id, 98, 99),
            range(ast.source_id, 102, 103),
        ],
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        set_term_node: choice_id.index(),
        proof_range: proof.range,
    })
}

const TASK258B3M2B2A_SURFACE_RANGES: [(usize, usize); 57] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 74),
    (74, 75),
    (76, 77),
    (78, 79),
    (80, 81),
    (82, 87),
    (90, 94),
    (95, 96),
    (96, 97),
    (97, 98),
    (98, 99),
    (99, 100),
    (100, 101),
    (104, 108),
    (109, 110),
    (111, 112),
    (113, 114),
    (114, 115),
    (116, 119),
    (119, 120),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (76, 77),
    (76, 77),
    (80, 81),
    (80, 81),
    (76, 81),
    (76, 81),
    (97, 98),
    (97, 98),
    (96, 99),
    (96, 99),
    (95, 100),
    (95, 100),
    (95, 100),
    (90, 101),
    (109, 110),
    (109, 110),
    (113, 114),
    (113, 114),
    (109, 114),
    (109, 114),
    (109, 114),
    (104, 115),
    (82, 119),
    (19, 120),
    (0, 120),
    (0, 120),
    (0, 120),
];

const TASK258B3M2B2A_TOKEN_TEXTS: [&str; 26] = [
    "reserve",
    "x",
    "for",
    "set",
    ";",
    "theorem",
    "FormulaStatementNestedParenthesizedWitnessSmoke",
    ":",
    "x",
    "=",
    "x",
    "proof",
    "take",
    "(",
    "(",
    "x",
    ")",
    ")",
    ";",
    "thus",
    "x",
    "=",
    "x",
    ";",
    "end",
    ";",
];

fn task258b3m2b2a_token_kind(index: usize) -> SurfaceTokenKind {
    match index {
        0 | 2 | 3 | 5 | 11 | 12 | 19 | 24 => SurfaceTokenKind::ReservedWord,
        1 | 6 | 8 | 10 | 15 | 20 | 22 => SurfaceTokenKind::Identifier,
        4 | 7 | 9 | 13 | 14 | 16 | 17 | 18 | 21 | 23 | 25 => SurfaceTokenKind::ReservedSymbol,
        _ => unreachable!("Task258B3M2B2A has exactly 26 tokens"),
    }
}

fn task258b3m2b2a_surface_kind(index: usize) -> SyntaxKind {
    match index {
        0..=25 => SyntaxKind::Token,
        26 => SyntaxKind::TypeHead,
        27 => SyntaxKind::TypeExpression,
        28 => SyntaxKind::ReserveSegment,
        29 => SyntaxKind::ReserveItem,
        30 | 32 | 36 | 44 | 46 => SyntaxKind::TermReference,
        31 | 33 | 37 | 39 | 41 | 45 | 47 => SyntaxKind::TermExpression,
        34 | 48 => SyntaxKind::BuiltinPredicateApplication,
        35 | 49 => SyntaxKind::FormulaExpression,
        38 | 40 => SyntaxKind::ParenthesizedTerm,
        42 => SyntaxKind::Witness,
        43 => SyntaxKind::TakeStatement,
        50 => SyntaxKind::Proposition,
        51 => SyntaxKind::ConclusionStatement,
        52 => SyntaxKind::ProofBlock,
        53 => SyntaxKind::TheoremItem,
        54 => SyntaxKind::ItemList,
        55 => SyntaxKind::CompilationUnit,
        56 => SyntaxKind::Root,
        _ => unreachable!("Task258B3M2B2A has exactly 57 nodes"),
    }
}

fn task258b3m2b2a_surface_children(index: usize) -> &'static [usize] {
    match index {
        26 => &[3],
        27 => &[26],
        28 => &[1, 2, 27],
        29 => &[0, 28, 4],
        30 => &[8],
        31 => &[30],
        32 => &[10],
        33 => &[32],
        34 => &[31, 9, 33],
        35 => &[34],
        36 => &[15],
        37 => &[36],
        38 => &[14, 37, 16],
        39 => &[38],
        40 => &[13, 39, 17],
        41 => &[40],
        42 => &[41],
        43 => &[12, 42, 18],
        44 => &[20],
        45 => &[44],
        46 => &[22],
        47 => &[46],
        48 => &[45, 21, 47],
        49 => &[48],
        50 => &[49],
        51 => &[19, 50, 23],
        52 => &[11, 43, 51, 24],
        53 => &[5, 6, 7, 35, 52, 25],
        54 => &[29, 53],
        55 => &[54],
        56 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 55,
        ],
        _ => &[],
    }
}

fn exact_nested_parenthesized_witness_surface_profile(ast: &SurfaceAst) -> bool {
    TASK258B3M2B2A_SURFACE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            ast.nodes().get(index).is_some_and(|node| {
                let exact_token = if index < TASK258B3M2B2A_TOKEN_TEXTS.len() {
                    matches!(
                        &node.kind,
                        SurfaceNodeKind::Token(token)
                            if token.kind == task258b3m2b2a_token_kind(index)
                                && token.text.as_ref() == TASK258B3M2B2A_TOKEN_TEXTS[index]
                    )
                } else {
                    true
                };
                exact_token
                    && node.range == range(ast.source_id, start, end)
                    && node.kind.syntax_kind() == task258b3m2b2a_surface_kind(index)
                    && node
                        .children
                        .iter()
                        .map(|child| child.index())
                        .eq(task258b3m2b2a_surface_children(index).iter().copied())
            })
        })
}

const TASK258B3M1_SURFACE_RANGES: [(usize, usize); 56] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 63),
    (63, 64),
    (65, 66),
    (67, 68),
    (69, 70),
    (71, 76),
    (79, 83),
    (84, 85),
    (86, 87),
    (88, 89),
    (89, 90),
    (91, 92),
    (92, 93),
    (96, 100),
    (101, 102),
    (103, 104),
    (105, 106),
    (106, 107),
    (108, 111),
    (111, 112),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (65, 66),
    (65, 66),
    (69, 70),
    (69, 70),
    (65, 70),
    (65, 70),
    (88, 89),
    (88, 89),
    (84, 89),
    (91, 92),
    (91, 92),
    (91, 92),
    (79, 93),
    (101, 102),
    (101, 102),
    (105, 106),
    (105, 106),
    (101, 106),
    (101, 106),
    (101, 106),
    (96, 107),
    (71, 111),
    (19, 112),
    (0, 112),
    (0, 112),
    (0, 112),
];

const TASK258B3M1_TOKEN_TEXTS: [&str; 26] = [
    "reserve",
    "x",
    "for",
    "set",
    ";",
    "theorem",
    "FormulaStatementMultipleWitnessSmoke",
    ":",
    "x",
    "=",
    "x",
    "proof",
    "take",
    "y",
    "=",
    "x",
    ",",
    "x",
    ";",
    "thus",
    "x",
    "=",
    "x",
    ";",
    "end",
    ";",
];

fn task258b3m1_token_kind(index: usize) -> SurfaceTokenKind {
    match index {
        0 | 2 | 3 | 5 | 11 | 12 | 19 | 24 => SurfaceTokenKind::ReservedWord,
        1 | 6 | 8 | 10 | 13 | 15 | 17 | 20 | 22 => SurfaceTokenKind::Identifier,
        4 | 7 | 9 | 14 | 16 | 18 | 21 | 23 | 25 => SurfaceTokenKind::ReservedSymbol,
        _ => unreachable!("Task258B3M1 has exactly 26 tokens"),
    }
}

fn task258b3m1_surface_kind(index: usize) -> SyntaxKind {
    match index {
        0..=25 => SyntaxKind::Token,
        26 => SyntaxKind::TypeHead,
        27 => SyntaxKind::TypeExpression,
        28 => SyntaxKind::ReserveSegment,
        29 => SyntaxKind::ReserveItem,
        30 | 32 | 36 | 39 | 43 | 45 => SyntaxKind::TermReference,
        31 | 33 | 37 | 40 | 44 | 46 => SyntaxKind::TermExpression,
        34 | 47 => SyntaxKind::BuiltinPredicateApplication,
        35 | 48 => SyntaxKind::FormulaExpression,
        38 | 41 => SyntaxKind::Witness,
        42 => SyntaxKind::TakeStatement,
        49 => SyntaxKind::Proposition,
        50 => SyntaxKind::ConclusionStatement,
        51 => SyntaxKind::ProofBlock,
        52 => SyntaxKind::TheoremItem,
        53 => SyntaxKind::ItemList,
        54 => SyntaxKind::CompilationUnit,
        55 => SyntaxKind::Root,
        _ => unreachable!("Task258B3M1 has exactly 56 nodes"),
    }
}

fn task258b3m1_surface_children(index: usize) -> &'static [usize] {
    match index {
        26 => &[3],
        27 => &[26],
        28 => &[1, 2, 27],
        29 => &[0, 28, 4],
        30 => &[8],
        31 => &[30],
        32 => &[10],
        33 => &[32],
        34 => &[31, 9, 33],
        35 => &[34],
        36 => &[15],
        37 => &[36],
        38 => &[13, 14, 37],
        39 => &[17],
        40 => &[39],
        41 => &[40],
        42 => &[12, 38, 16, 41, 18],
        43 => &[20],
        44 => &[43],
        45 => &[22],
        46 => &[45],
        47 => &[44, 21, 46],
        48 => &[47],
        49 => &[48],
        50 => &[19, 49, 23],
        51 => &[11, 42, 50, 24],
        52 => &[5, 6, 7, 35, 51, 25],
        53 => &[29, 52],
        54 => &[53],
        55 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 54,
        ],
        _ => &[],
    }
}

fn exact_multiple_witness_surface_profile(ast: &SurfaceAst) -> bool {
    TASK258B3M1_SURFACE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            ast.nodes().get(index).is_some_and(|node| {
                let exact_token = if index < TASK258B3M1_TOKEN_TEXTS.len() {
                    matches!(
                        &node.kind,
                        SurfaceNodeKind::Token(token)
                            if token.kind == task258b3m1_token_kind(index)
                                && token.text.as_ref() == TASK258B3M1_TOKEN_TEXTS[index]
                    )
                } else {
                    true
                };
                exact_token
                    && node.range == range(ast.source_id, start, end)
                    && node.kind.syntax_kind() == task258b3m1_surface_kind(index)
                    && node
                        .children
                        .iter()
                        .map(|child| child.index())
                        .eq(task258b3m1_surface_children(index).iter().copied())
            })
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
    if source_text == SOURCE_STATEMENT_B4C_TEXT {
        return source_statement_b4c_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B4B_TEXT {
        return source_statement_b4b_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B4A_TEXT {
        return source_statement_b4a_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B3E_TEXT {
        return source_statement_b3m2b2b3e_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B3D_TEXT {
        return source_statement_b3m2b2b3d_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B3C_TEXT {
        return source_statement_b3m2b2b3c_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B3B_TEXT {
        return source_statement_b3m2b2b3b_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B3A_TEXT {
        return source_statement_b3m2b2b3a_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B2C_TEXT {
        return source_statement_b3m2b2b2c_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B2B_TEXT {
        return source_statement_b3m2b2b2b_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B2A_TEXT {
        return source_statement_b3m2b2b2a_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B1B1_TEXT {
        return source_statement_b3m2b2b1b1_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2B1A_TEXT {
        return source_statement_b3m2b2b1a_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B2A_TEXT {
        return source_statement_b3m2b2a_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2B1_TEXT {
        return source_statement_b3m2b1_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M2A_TEXT {
        return source_statement_b3m2a_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
    if source_text == SOURCE_STATEMENT_B3M1_TEXT {
        return source_statement_b3m1_output_with_source_and_mutation_impl(
            ast,
            module,
            symbols,
            source_text,
            |_| {},
        );
    }
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
pub(in crate::runner) fn source_statement_b3m2b2b3e_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3ERouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b3e_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3e_output_with_stage_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    stage: SourceStatementB3M2B2B3EStage,
    field: usize,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_comprehension_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3E_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        Some((stage, field)),
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3e_output_with_lower_stage_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    stage: SourceStatementB3M2B2B3ELowerStage,
    field: usize,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let baseline = match source_statement_b3m2b2b3e_output_with_source_and_mutation_impl(
        ast,
        module.clone(),
        symbols,
        source_text,
        |_| {},
    )? {
        Ok(output) => output,
        Err(error) => return Some(Err(error)),
    };
    match stage {
        SourceStatementB3M2B2B3ELowerStage::Task48 => {
            let parts = match mutate_task258b3m2b2b3b_binding_env(
                baseline
                    .typed_ast
                    .source_statement()
                    .expect("Task258B3M2B2B3E statement")
                    .binding_env(),
                field,
            ) {
                Ok(parts) => parts,
                Err(error) => return Some(Err(error)),
            };
            let binding_env = match BindingEnv::try_new(parts) {
                Ok(binding_env) => binding_env,
                Err(error) => return Some(Err(format!("Task48: {error}"))),
            };
            source_statement_b3m2b2b3e_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.binding_env = binding_env,
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task48:") {
                        error
                    } else {
                        format!("Task48: {error}")
                    }
                })
            })
        }
        SourceStatementB3M2B2B3ELowerStage::Task252 => {
            let mut input = task258b3m2b2b3b_primary_input(
                baseline
                    .typed_ast
                    .source_term()
                    .expect("Task258B3M2B2B3E primary terms"),
            );
            if let Err(error) = mutate_task258b3m2b2b3d_primary_input(&mut input, field) {
                return Some(Err(error));
            }
            let primary = match SourcePrimaryTermProducer::build(
                input,
                baseline
                    .typed_ast
                    .source_statement()
                    .expect("Task258B3M2B2B3E statement")
                    .binding_env(),
                baseline.typed_ast.nodes(),
            ) {
                Ok(primary) => primary,
                Err(error) => return Some(Err(format!("Task252: {error}"))),
            };
            source_statement_b3m2b2b3e_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.primary = primary,
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task252:") {
                        error
                    } else {
                        format!("Task252: {error}")
                    }
                })
            })
        }
        SourceStatementB3M2B2B3ELowerStage::Task255 => {
            let mut input = task258b3m2b2b3e_set_input(
                baseline
                    .typed_ast
                    .source_set_term()
                    .expect("Task258B3M2B2B3E set term"),
            );
            if let Err(error) = mutate_task258b3m2b2b3e_set_input(&mut input, field) {
                return Some(Err(error));
            }
            let binding_env = baseline
                .typed_ast
                .source_statement()
                .expect("Task258B3M2B2B3E statement")
                .binding_env();
            let primary = baseline
                .typed_ast
                .source_term()
                .expect("Task258B3M2B2B3E primary terms");
            let set_term = match SourceSetTermProducer::build(
                input,
                binding_env,
                primary,
                None,
                None,
                baseline.typed_ast.nodes(),
            ) {
                Ok(set_term) => set_term,
                Err(error) => return Some(Err(format!("Task255: {error}"))),
            };
            source_statement_b3m2b2b3e_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.set_term = Some(set_term),
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task255:") {
                        error
                    } else {
                        format!("Task255: {error}")
                    }
                })
            })
        }
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3e_output_with_post_auth_family_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3ERouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_comprehension_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3E_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        None,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3e_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_comprehension_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3E_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3e_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3e_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    label_range: SourceRange,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3E_LABEL,
        label_range,
    )
}

fn source_statement_b3m2b2b3e_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3ERouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_comprehension_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3E_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3e_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3d_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3DRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b3d_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3d_output_with_stage_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    stage: SourceStatementB3M2B2B3DStage,
    field: usize,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_qua_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3D_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        Some((stage, field)),
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3d_output_with_lower_stage_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    stage: SourceStatementB3M2B2B3DLowerStage,
    field: usize,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let baseline = match source_statement_b3m2b2b3d_output_with_source_and_mutation_impl(
        ast,
        module.clone(),
        symbols,
        source_text,
        |_| {},
    )? {
        Ok(output) => output,
        Err(error) => return Some(Err(error)),
    };

    match stage {
        SourceStatementB3M2B2B3DLowerStage::Task48 => {
            let parts = match mutate_task258b3m2b2b3b_binding_env(
                baseline
                    .typed_ast
                    .source_statement()
                    .expect("Task258B3M2B2B3D statement")
                    .binding_env(),
                field,
            ) {
                Ok(parts) => parts,
                Err(error) => return Some(Err(error)),
            };
            let binding_env = match BindingEnv::try_new(parts) {
                Ok(binding_env) => binding_env,
                Err(error) => return Some(Err(format!("Task48: {error}"))),
            };
            source_statement_b3m2b2b3d_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.binding_env = binding_env,
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task48:") {
                        error
                    } else {
                        format!("Task48: {error}")
                    }
                })
            })
        }
        SourceStatementB3M2B2B3DLowerStage::Task252 => {
            let mut input = task258b3m2b2b3b_primary_input(
                baseline
                    .typed_ast
                    .source_term()
                    .expect("Task258B3M2B2B3D primary terms"),
            );
            if let Err(error) = mutate_task258b3m2b2b3d_primary_input(&mut input, field) {
                return Some(Err(error));
            }
            let primary = match SourcePrimaryTermProducer::build(
                input,
                baseline
                    .typed_ast
                    .source_statement()
                    .expect("Task258B3M2B2B3D statement")
                    .binding_env(),
                baseline.typed_ast.nodes(),
            ) {
                Ok(primary) => primary,
                Err(error) => return Some(Err(format!("Task252: {error}"))),
            };
            source_statement_b3m2b2b3d_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.primary = primary,
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task252:") {
                        error
                    } else {
                        format!("Task252: {error}")
                    }
                })
            })
        }
        SourceStatementB3M2B2B3DLowerStage::Task255 => {
            let mut input = task258b3m2b2b3d_set_input(
                baseline
                    .typed_ast
                    .source_set_term()
                    .expect("Task258B3M2B2B3D set term"),
            );
            if let Err(error) = mutate_task258b3m2b2b3d_set_input(&mut input, field) {
                return Some(Err(error));
            }
            let binding_env = baseline
                .typed_ast
                .source_statement()
                .expect("Task258B3M2B2B3D statement")
                .binding_env();
            let primary = baseline
                .typed_ast
                .source_term()
                .expect("Task258B3M2B2B3D primary terms");
            let set_term = match SourceSetTermProducer::build(
                input,
                binding_env,
                primary,
                None,
                None,
                baseline.typed_ast.nodes(),
            ) {
                Ok(set_term) => set_term,
                Err(error) => return Some(Err(format!("Task255: {error}"))),
            };
            source_statement_b3m2b2b3d_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.set_term = Some(set_term),
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task255:") {
                        error
                    } else {
                        format!("Task255: {error}")
                    }
                })
            })
        }
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3d_output_with_post_auth_family_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3DRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_qua_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3D_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        None,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3d_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_qua_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3D_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3d_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3d_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    label_range: SourceRange,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3D_LABEL,
        label_range,
    )
}

fn source_statement_b3m2b2b3d_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3DRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_qua_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3D_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3d_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3c_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3CRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b3c_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3c_output_with_stage_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    stage: SourceStatementB3M2B2B3CStage,
    field: usize,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_choice_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3C_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        Some((stage, field)),
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3c_output_with_lower_stage_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    stage: SourceStatementB3M2B2B3CLowerStage,
    field: usize,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let baseline = match source_statement_b3m2b2b3c_output_with_source_and_mutation_impl(
        ast,
        module.clone(),
        symbols,
        source_text,
        |_| {},
    )? {
        Ok(output) => output,
        Err(error) => return Some(Err(error)),
    };

    match stage {
        SourceStatementB3M2B2B3CLowerStage::Task48 => {
            let parts = match mutate_task258b3m2b2b3b_binding_env(
                baseline
                    .typed_ast
                    .source_statement()
                    .expect("Task258B3M2B2B3C statement")
                    .binding_env(),
                field,
            ) {
                Ok(parts) => parts,
                Err(error) => return Some(Err(error)),
            };
            let binding_env = match BindingEnv::try_new(parts) {
                Ok(binding_env) => binding_env,
                Err(error) => return Some(Err(format!("Task48: {error}"))),
            };
            source_statement_b3m2b2b3c_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.binding_env = binding_env,
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task48:") {
                        error
                    } else {
                        format!("Task48: {error}")
                    }
                })
            })
        }
        SourceStatementB3M2B2B3CLowerStage::Task252 => {
            let mut input = task258b3m2b2b3b_primary_input(
                baseline
                    .typed_ast
                    .source_term()
                    .expect("Task258B3M2B2B3C primary terms"),
            );
            if let Err(error) = mutate_task258b3m2b2b3b_primary_input(&mut input, field) {
                return Some(Err(error));
            }
            let primary = match SourcePrimaryTermProducer::build(
                input,
                baseline
                    .typed_ast
                    .source_statement()
                    .expect("Task258B3M2B2B3C statement")
                    .binding_env(),
                baseline.typed_ast.nodes(),
            ) {
                Ok(primary) => primary,
                Err(error) => return Some(Err(format!("Task252: {error}"))),
            };
            source_statement_b3m2b2b3c_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.primary = primary,
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task252:") {
                        error
                    } else {
                        format!("Task252: {error}")
                    }
                })
            })
        }
        SourceStatementB3M2B2B3CLowerStage::Task255 => {
            let mut input = task258b3m2b2b3c_set_input(
                baseline
                    .typed_ast
                    .source_set_term()
                    .expect("Task258B3M2B2B3C set term"),
            );
            if let Err(error) = mutate_task258b3m2b2b3c_set_input(&mut input, field) {
                return Some(Err(error));
            }
            let binding_env = baseline
                .typed_ast
                .source_statement()
                .expect("Task258B3M2B2B3C statement")
                .binding_env();
            let primary = baseline
                .typed_ast
                .source_term()
                .expect("Task258B3M2B2B3C primary terms");
            let set_term = match SourceSetTermProducer::build(
                input,
                binding_env,
                primary,
                None,
                None,
                baseline.typed_ast.nodes(),
            ) {
                Ok(set_term) => set_term,
                Err(error) => return Some(Err(format!("Task255: {error}"))),
            };
            source_statement_b3m2b2b3c_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.set_term = Some(set_term),
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task255:") {
                        error
                    } else {
                        format!("Task255: {error}")
                    }
                })
            })
        }
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3c_output_with_post_auth_family_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3CRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_choice_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3C_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        None,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3c_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_choice_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3C_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3c_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3c_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    label_range: SourceRange,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3C_LABEL,
        label_range,
    )
}

fn source_statement_b3m2b2b3c_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3CRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_choice_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3C_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3c_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3b_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3BRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b3b_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3b_output_with_stage_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    stage: SourceStatementB3M2B2B3BStage,
    field: usize,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_empty_set_enumeration_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3B_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        Some((stage, field)),
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3b_output_with_lower_stage_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    stage: SourceStatementB3M2B2B3BLowerStage,
    field: usize,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let baseline = match source_statement_b3m2b2b3b_output_with_source_and_mutation_impl(
        ast,
        module.clone(),
        symbols,
        source_text,
        |_| {},
    )? {
        Ok(output) => output,
        Err(error) => return Some(Err(error)),
    };

    match stage {
        SourceStatementB3M2B2B3BLowerStage::Task48 => {
            let parts = match mutate_task258b3m2b2b3b_binding_env(
                baseline
                    .typed_ast
                    .source_statement()
                    .expect("Task258B3M2B2B3B statement")
                    .binding_env(),
                field,
            ) {
                Ok(parts) => parts,
                Err(error) => return Some(Err(error)),
            };
            let binding_env = match BindingEnv::try_new(parts) {
                Ok(binding_env) => binding_env,
                Err(error) => return Some(Err(format!("Task48: {error}"))),
            };
            source_statement_b3m2b2b3b_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.binding_env = binding_env,
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task48:") {
                        error
                    } else {
                        format!("Task48: {error}")
                    }
                })
            })
        }
        SourceStatementB3M2B2B3BLowerStage::Task252 => {
            let mut input = task258b3m2b2b3b_primary_input(
                baseline
                    .typed_ast
                    .source_term()
                    .expect("Task258B3M2B2B3B primary terms"),
            );
            if let Err(error) = mutate_task258b3m2b2b3b_primary_input(&mut input, field) {
                return Some(Err(error));
            }
            let primary = match SourcePrimaryTermProducer::build(
                input,
                baseline
                    .typed_ast
                    .source_statement()
                    .expect("Task258B3M2B2B3B statement")
                    .binding_env(),
                baseline.typed_ast.nodes(),
            ) {
                Ok(primary) => primary,
                Err(error) => return Some(Err(format!("Task252: {error}"))),
            };
            source_statement_b3m2b2b3b_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.primary = primary,
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task252:") {
                        error
                    } else {
                        format!("Task252: {error}")
                    }
                })
            })
        }
        SourceStatementB3M2B2B3BLowerStage::Task255 => {
            let mut input = task258b3m2b2b3b_set_input(
                baseline
                    .typed_ast
                    .source_set_term()
                    .expect("Task258B3M2B2B3B set term"),
            );
            if let Err(error) = mutate_task258b3m2b2b3b_set_input(&mut input, field) {
                return Some(Err(error));
            }
            let binding_env = baseline
                .typed_ast
                .source_statement()
                .expect("Task258B3M2B2B3B statement")
                .binding_env();
            let primary = baseline
                .typed_ast
                .source_term()
                .expect("Task258B3M2B2B3B primary terms");
            let set_term = match SourceSetTermProducer::build(
                input,
                binding_env,
                primary,
                None,
                None,
                baseline.typed_ast.nodes(),
            ) {
                Ok(set_term) => set_term,
                Err(error) => return Some(Err(format!("Task255: {error}"))),
            };
            source_statement_b3m2b2b3b_output_with_post_auth_family_mutation(
                ast,
                module,
                symbols,
                source_text,
                move |input| input.set_term = Some(set_term),
            )
            .map(|result| {
                result.map_err(|error| {
                    if error.starts_with("Task255:") {
                        error
                    } else {
                        format!("Task255: {error}")
                    }
                })
            })
        }
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3b_output_with_post_auth_family_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3BRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_empty_set_enumeration_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3B_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        None,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3b_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_empty_set_enumeration_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3B_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3b_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3b_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    label_range: SourceRange,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3B_LABEL,
        label_range,
    )
}

fn source_statement_b3m2b2b3b_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3BRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_empty_set_enumeration_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3B_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3b_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3a_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b3a_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3a_output_with_stage_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    stage: SourceStatementB3M2B2B3AStage,
    field: usize,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_set_enumeration_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        Some((stage, field)),
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3a_output_with_post_auth_family_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_set_enumeration_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_witness_output_with_controls(
        ast,
        module,
        &symbols,
        extracted.into(),
        |_| {},
        None,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3a_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_set_enumeration_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3a_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b3a_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    label_range: SourceRange,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3A_LABEL,
        label_range,
    )
}

fn source_statement_b3m2b2b3a_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_set_enumeration_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B3A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b3a_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b2c_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B2CRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b2c_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b2c_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_structure_update_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B2C_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b2c_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b2c_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    label_range: SourceRange,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B2C_LABEL,
        label_range,
    )
}

fn source_statement_b3m2b2b2c_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B2CRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_structure_update_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B2C_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b2c_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b2b_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B2BRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b2b_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b2b_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_structure_selector_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B2B_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b2b_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b2b_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    label_range: SourceRange,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B2B_LABEL,
        label_range,
    )
}

fn source_statement_b3m2b2b2b_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B2BRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_structure_selector_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B2B_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b2b_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b2a_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B2ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b2a_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b2a_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_structure_constructor_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B2A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b2a_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b2a_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    label_range: SourceRange,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B2A_LABEL,
        label_range,
    )
}

fn source_statement_b3m2b2b2a_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B2ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_structure_constructor_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B2A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b2a_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b1b1_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B1B1RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b1b1_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b1b1_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_wrapped_application_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B1B1_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b1b1_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_b3m2b2b1b1_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B1B1RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_wrapped_application_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B1B1_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b1b1_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b1a_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B1ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2b1a_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b1a_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_application_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B1A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b1a_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_b3m2b2b1a_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B1ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_application_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B1A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2b1a_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2a_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b2a_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2a_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_nested_parenthesized_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2a_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_b3m2b2a_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_nested_parenthesized_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B2A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b2a_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b1_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B1RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2b1_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b1_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_parenthesized_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B1_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b1_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_b3m2b1_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2B1RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_parenthesized_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2B1_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2b1_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2a_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m2a_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2a_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_numeral_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2a_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_b3m2a_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M2ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_numeral_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M2A_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m2a_output(
        ast, module, &symbols, extracted, mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m1_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M1RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b3m1_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m1_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_multiple_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M1_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m1_output(
        ast,
        module,
        &symbols,
        extracted,
        |_| {},
    ))
}

fn source_statement_b3m1_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB3M1RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    let extracted = extract_multiple_witness_source_statement(ast, source_text)?;
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B3M1_LABEL,
        extracted.label_range,
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b3m1_output(
        ast, module, &symbols, extracted, mutate,
    ))
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

fn source_statement_b4c_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB4CRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    if source_text != SOURCE_STATEMENT_B4C_TEXT {
        return None;
    }
    if !exact_task258b4c_surface_profile(ast, source_text) {
        return Some(Err("Task258B4C exact surface identity mismatch".to_owned()));
    }
    if let Err(error) = validate_task258b4c_raw_resolver_env(ast, &module, symbols) {
        return Some(Err(error));
    }
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B4C_LABEL,
        range(ast.source_id, 27, 65),
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b4c_output(
        ast,
        module,
        &symbols,
        source_text,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4c_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB4CRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b4c_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4c_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    if source_text != SOURCE_STATEMENT_B4C_TEXT {
        return None;
    }
    if !exact_task258b4c_surface_profile(ast, source_text) {
        return Some(Err("Task258B4C exact surface identity mismatch".to_owned()));
    }
    if let Err(error) = validate_task258b4c_raw_resolver_env(ast, &module, symbols) {
        return Some(Err(error));
    }
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B4C_LABEL,
        range(ast.source_id, 27, 65),
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b4c_output(
        ast,
        module,
        &symbols,
        source_text,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4c_output_with_raw_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    if source_text != SOURCE_STATEMENT_B4C_TEXT {
        return None;
    }
    if !exact_task258b4c_surface_profile(ast, source_text) {
        return Some(Err("Task258B4C exact surface identity mismatch".to_owned()));
    }
    let symbols = mutate(symbols.clone());
    if let Err(error) = validate_task258b4c_raw_resolver_env(ast, &module, &symbols) {
        return Some(Err(error));
    }
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        &symbols,
        SOURCE_STATEMENT_B4C_LABEL,
        range(ast.source_id, 27, 65),
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b4c_output(
        ast,
        module,
        &symbols,
        source_text,
        |_| {},
    ))
}

fn source_statement_b4b_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB4BRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    if source_text != SOURCE_STATEMENT_B4B_TEXT {
        return None;
    }
    if !exact_task258b4b_surface_profile(ast, source_text) {
        return Some(Err("Task258B4B exact surface identity mismatch".to_owned()));
    }
    if let Err(error) = validate_task258b4b_raw_resolver_env(ast, &module, symbols) {
        return Some(Err(error));
    }
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B4B_LABEL,
        range(ast.source_id, 8, 48),
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b4b_output(
        ast,
        module,
        &symbols,
        source_text,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4b_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB4BRouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b4b_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4b_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    if source_text != SOURCE_STATEMENT_B4B_TEXT {
        return None;
    }
    if !exact_task258b4b_surface_profile(ast, source_text) {
        return Some(Err("Task258B4B exact surface identity mismatch".to_owned()));
    }
    if let Err(error) = validate_task258b4b_raw_resolver_env(ast, &module, symbols) {
        return Some(Err(error));
    }
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B4B_LABEL,
        range(ast.source_id, 8, 48),
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b4b_output(
        ast,
        module,
        &symbols,
        source_text,
        |_| {},
    ))
}

fn source_statement_b4a_output_with_source_and_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB4ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    if source_text != SOURCE_STATEMENT_B4A_TEXT {
        return None;
    }
    if !exact_task258b4a_surface_profile(ast, source_text) {
        return Some(Err("Task258B4A exact surface identity mismatch".to_owned()));
    }
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B4A_LABEL,
        range(ast.source_id, 8, 48),
    ) {
        Ok(symbols) => symbols,
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b4a_output(
        ast,
        module,
        &symbols,
        source_text,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4a_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB4ARouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    source_statement_b4a_output_with_source_and_mutation_impl(
        ast,
        module,
        symbols,
        source_text,
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4a_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    if source_text != SOURCE_STATEMENT_B4A_TEXT {
        return None;
    }
    if !exact_task258b4a_surface_profile(ast, source_text) {
        return Some(Err("Task258B4A exact surface identity mismatch".to_owned()));
    }
    let symbols = match enrich_source_statement_resolver_env_for_owner(
        &module,
        symbols,
        SOURCE_STATEMENT_B4A_LABEL,
        range(ast.source_id, 8, 48),
    ) {
        Ok(symbols) => mutate(symbols),
        Err(error) => return Some(Err(error)),
    };
    Some(build_source_statement_b4a_output(
        ast,
        module,
        &symbols,
        source_text,
        |_| {},
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

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m1_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementB3M1Extraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M1_LABEL,
        extracted.label_range,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2a_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementB3M2AExtraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2A_LABEL,
        extracted.label_range,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b1_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementB3M2B1Extraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B1_LABEL,
        extracted.label_range,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2a_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementB3M2B2AExtraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2A_LABEL,
        extracted.label_range,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b1a_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementB3M2B2B1AExtraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B1A_LABEL,
        extracted.label_range,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b3m2b2b1b1_resolver_env_for_test(
    module: &ModuleId,
    symbols: &SymbolEnv,
    extracted: &SourceStatementB3M2B2B1B1Extraction,
) -> Result<SymbolEnv, String> {
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B3M2B2B1B1_LABEL,
        extracted.label_range,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4a_resolver_env_for_test(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Result<SymbolEnv, String> {
    if source_text != SOURCE_STATEMENT_B4A_TEXT
        || !exact_task258b4a_surface_profile(ast, source_text)
    {
        return Err("Task258B4A exact surface identity mismatch".to_owned());
    }
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B4A_LABEL,
        range(ast.source_id, 8, 48),
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4c_resolver_env_for_test(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Result<SymbolEnv, String> {
    if source_text != SOURCE_STATEMENT_B4C_TEXT
        || !exact_task258b4c_surface_profile(ast, source_text)
    {
        return Err("Task258B4C exact surface identity mismatch".to_owned());
    }
    validate_task258b4c_raw_resolver_env(ast, module, symbols)?;
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B4C_LABEL,
        range(ast.source_id, 27, 65),
    )
}

fn validate_task258b4c_raw_resolver_env(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
) -> Result<(), String> {
    if symbols.module_id() != module
        || (
            symbols.symbols().len(),
            symbols.labels().len(),
            symbols.definitions().len(),
            symbols.contributions().len(),
            symbols.imports().len(),
        ) != (1, 0, 1, 1, 0)
    {
        return Err("Task258B4C raw resolver profile mismatch".to_owned());
    }
    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B4C_LABEL)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return Err("Task258B4C raw resolver theorem owner mismatch".to_owned());
    };
    let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner.symbol().clone(),
        ast.source_id,
        module,
    )
    .map_err(|error| error.to_string())?;
    let contribution = symbols
        .contributions()
        .get(owner.contribution())
        .ok_or_else(|| "Task258B4C raw resolver contribution is missing".to_owned())?;
    if owner.primary_spelling() != SOURCE_STATEMENT_B4C_LABEL
        || owner.visibility() != Visibility::Public
        || owner.export_status() != ExportStatus::Exported
        || owner.contribution().index() != 0
        || checked_owner.source_range() != range(ast.source_id, 19, 137)
        || checked_owner.origin().structural_path() != [2, 1]
        || checked_owner.origin().import_edge().is_some()
        || checked_owner.origin().is_recovered()
        || contribution.module() != module
        || !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == ast.source_id
        )
        || contribution.anchor() != &SourceAnchor::Range(range(ast.source_id, 0, 18))
        || contribution.effects().symbols() != [owner.symbol().clone()]
        || contribution.effects().definitions().len() != 1
        || !contribution.effects().labels().is_empty()
    {
        return Err("Task258B4C raw resolver provenance mismatch".to_owned());
    }
    Ok(())
}

#[cfg(test)]
pub(in crate::runner) fn source_statement_b4b_resolver_env_for_test(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Result<SymbolEnv, String> {
    if source_text != SOURCE_STATEMENT_B4B_TEXT
        || !exact_task258b4b_surface_profile(ast, source_text)
    {
        return Err("Task258B4B exact surface identity mismatch".to_owned());
    }
    validate_task258b4b_raw_resolver_env(ast, module, symbols)?;
    enrich_source_statement_resolver_env_for_owner(
        module,
        symbols,
        SOURCE_STATEMENT_B4B_LABEL,
        range(ast.source_id, 8, 48),
    )
}

fn validate_task258b4b_raw_resolver_env(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
) -> Result<(), String> {
    if symbols.module_id() != module
        || (
            symbols.symbols().len(),
            symbols.labels().len(),
            symbols.definitions().len(),
            symbols.contributions().len(),
            symbols.imports().len(),
        ) != (1, 0, 1, 1, 0)
    {
        return Err("Task258B4B raw resolver profile mismatch".to_owned());
    }
    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B4B_LABEL)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return Err("Task258B4B raw resolver theorem owner mismatch".to_owned());
    };
    let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner.symbol().clone(),
        ast.source_id,
        module,
    )
    .map_err(|error| error.to_string())?;
    let contribution = symbols
        .contributions()
        .get(owner.contribution())
        .ok_or_else(|| "Task258B4B raw resolver contribution is missing".to_owned())?;
    if owner.primary_spelling() != SOURCE_STATEMENT_B4B_LABEL
        || owner.visibility() != Visibility::Public
        || owner.export_status() != ExportStatus::Exported
        || owner.contribution().index() != 0
        || checked_owner.source_range() != range(ast.source_id, 0, 165)
        || checked_owner.origin().structural_path() != [2, 0]
        || checked_owner.origin().import_edge().is_some()
        || checked_owner.origin().is_recovered()
        || contribution.module() != module
        || !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == ast.source_id
        )
        || contribution.anchor() != &SourceAnchor::Range(range(ast.source_id, 0, 165))
        || !contribution.effects().labels().is_empty()
    {
        return Err("Task258B4B raw resolver provenance mismatch".to_owned());
    }
    Ok(())
}

fn build_source_statement_b4c_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB4CRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    if source_text != SOURCE_STATEMENT_B4C_TEXT || symbols.module_id() != &module {
        return Err("Task258B4C source or symbol module mismatch".to_owned());
    }
    let lower =
        source_formula_composition_output_with_source(ast, module.clone(), symbols, source_text)
            .ok_or_else(|| "Task258B4C lower Task257B3 route is missing".to_owned())??;
    let lower_typed = &lower.typed_ast;
    let primary = lower_typed
        .source_term()
        .ok_or_else(|| "Task258B4C primary handoff is missing".to_owned())?;
    let atomic = lower_typed
        .source_atomic_formula()
        .ok_or_else(|| "Task258B4C atomic handoff is missing".to_owned())?;
    let composite = lower_typed
        .source_composite_formula()
        .ok_or_else(|| "Task258B4C composite handoff is missing".to_owned())?;
    let composition = lower_typed
        .source_formula_composition()
        .ok_or_else(|| "Task258B4C composition handoff is missing".to_owned())?;
    if lower_typed.nodes().len() != 66
        || lower_typed.nodes().root().is_some()
        || lower_typed.source_statement().is_some()
        || (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ) != (6, 6, 0)
        || (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_segments().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ) != (3, 0, 0, 0, 0, 0, 0, 6, 6)
        || (
            composite.formulas().len(),
            composite.wrappers().len(),
            composite.roots().len(),
            composite.binders().len(),
            composite.type_sites().len(),
            composite.edges().len(),
            composite.requests().len(),
        ) != (3, 0, 1, 3, 3, 2, 6)
        || (
            composition.atomic_edges().len(),
            composition.bound_uses().len(),
        ) != (3, 6)
        || (
            composite.binding_env().contexts().len(),
            composite.binding_env().bindings().len(),
            composite.binding_env().diagnostics().len(),
        ) != (4, 4, 0)
    {
        return Err("Task258B4C lower Task257B3 profile mismatch".to_owned());
    }
    const LOWER_OWNED_NODES: [usize; 24] = [
        9, 17, 22, 32, 33, 36, 37, 38, 39, 41, 43, 44, 45, 46, 47, 48, 50, 52, 53, 55, 57, 58, 59,
        60,
    ];
    if lower_typed.nodes().iter().any(|(id, node)| {
        let lower_owned = LOWER_OWNED_NODES.contains(&id.index());
        lower_owned == (node.kind.as_str() == "source.formula.composition.unowned")
            || node.anchor != SourceAnchor::Range(ast.nodes()[id.index()].range)
            || node.recovery != mizar_checker::typed_ast::NodeRecoveryState::Normal
    }) {
        return Err("Task258B4C lower owned-site partition mismatch".to_owned());
    }
    let nodes = lower_typed
        .nodes()
        .iter()
        .map(|(id, node)| {
            let mut node = node.clone();
            if id == TypedNodeId::new(62) {
                if node.kind.as_str() != "source.formula.composition.unowned" {
                    return Err("Task258B4C theorem node is already lower-owned".to_owned());
                }
                node.kind = "source.statement.theorem".into();
            }
            Ok(node)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let arena = TypedArena::try_new(None, nodes).map_err(|error| error.to_string())?;

    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B4C_LABEL)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return Err("Task258B4C requires one exact resolver theorem owner".to_owned());
    };
    let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner.symbol().clone(),
        ast.source_id,
        &module,
    )
    .map_err(|error| error.to_string())?;
    let labels = symbols
        .labels()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B4C_LABEL);
    let [label] = labels.as_slice() else {
        return Err("Task258B4C resolver theorem label mismatch".to_owned());
    };
    let expected_origin_path = LabelOriginPath::new(format!(
        "{}::{}::theorem::{}",
        module.package().as_str(),
        module.path().as_str(),
        SOURCE_STATEMENT_B4C_LABEL,
    ));
    let contribution = symbols
        .contributions()
        .get(owner.contribution())
        .ok_or_else(|| "Task258B4C resolver contribution is missing".to_owned())?;
    if (
        symbols.symbols().len(),
        symbols.labels().len(),
        symbols.definitions().len(),
        symbols.contributions().len(),
        symbols.imports().len(),
    ) != (1, 1, 1, 1, 0)
        || checked_owner.source_range() != range(ast.source_id, 19, 137)
        || checked_owner.origin().structural_path() != [2, 1]
        || checked_owner.origin().import_edge().is_some()
        || owner.contribution().index() != 0
        || label.origin_path() != &expected_origin_path
        || label.kind() != LabelKind::Theorem
        || label.visibility() != Visibility::Public
        || label.export_status() != ExportStatus::Exported
        || label.namespace() != &namespace
        || label.primary_spelling() != SOURCE_STATEMENT_B4C_LABEL
        || label.origin() != checked_owner.origin()
        || label.contribution() != owner.contribution()
        || label.recovery() != RecoveryState::Normal
        || contribution.anchor() != &SourceAnchor::Range(range(ast.source_id, 0, 18))
        || contribution.effects().symbols() != [owner.symbol().clone()]
        || contribution.effects().definitions().len() != 1
        || contribution.effects().labels() != [expected_origin_path]
    {
        return Err("Task258B4C resolver theorem provenance mismatch".to_owned());
    }

    let mut inputs = SourceStatementB4CRouteInputs {
        binding_env: composite.binding_env().clone(),
        arena,
        primary: primary.clone(),
        atomic: atomic.clone(),
        composite: composite.clone(),
        composition: composition.clone(),
        statement: SourceStatementHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: owner.symbol().clone(),
                contribution: owner.contribution(),
                site: TypedSiteRef::Node(TypedNodeId::new(62)),
                source_range: range(ast.source_id, 19, 137),
                spelling: SOURCE_STATEMENT_B4C_LABEL.to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: vec![SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(0),
                formula: SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)),
                site: TypedSiteRef::Node(TypedNodeId::new(62)),
                source_range: range(ast.source_id, 19, 137),
                source_ordinal: 0,
                spelling: SOURCE_STATEMENT_B4C_SPELLING.to_owned(),
                kind: SourceStatementKind::TheoremProposition,
                recovery: SourceStatementRecovery::Normal,
            }],
            contexts: vec![SourceStatementContextInput {
                statement: SourceStatementId::new(0),
                binding_context: BindingContextId::new(0),
                source_range: range(ast.source_id, 19, 137),
                visible_bindings: vec![BindingId::new(0)],
            }],
            input_facts: Vec::new(),
            candidate_facts: vec![SourceStatementCandidateFactInput {
                statement: SourceStatementId::new(0),
                context: SourceStatementContextId::new(0),
                ordinal: 0,
                kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                formula: SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)),
            }],
        },
    };
    mutate(&mut inputs);
    let statement = SourceStatementProducer::build_with_formula_composition(
        inputs.statement,
        symbols,
        &inputs.binding_env,
        &inputs.primary,
        &inputs.atomic,
        &inputs.composite,
        &inputs.composition,
        &inputs.arena,
    )
    .map_err(|error| error.to_string())?;
    let reference_use_ordinals = inputs
        .primary
        .references()
        .iter()
        .map(|(_, row)| row.use_ordinal())
        .collect::<Vec<_>>();
    let [left_lookup_ordinal, right_lookup_ordinal, 4, 4, 4, 4] = reference_use_ordinals.as_slice()
    else {
        return Err("Task258B4C lower reference profile mismatch".to_owned());
    };
    if *left_lookup_ordinal != 2 || *right_lookup_ordinal != 2 {
        return Err("Task258B4C lower reference profile mismatch".to_owned());
    }
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
    .with_source_formula_composition_statement(inputs.composite, inputs.composition, statement)
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
    if typed_ast.source_statement() != resolved.source_statement()
        || typed_ast.source_composite_formula() != resolved.source_composite_formula()
        || typed_ast.source_formula_composition() != resolved.source_formula_composition()
        || typed_ast.source_statement_references().is_some()
        || typed_ast.source_statement_witnesses().is_some()
        || !typed_ast.contexts().is_empty()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
        || !resolved.checked_formulas().is_empty()
        || !resolved.statement_semantics().is_empty()
        || !resolved.checked_proofs().is_empty()
        || !resolved.checked_proof_nodes().is_empty()
        || !resolved.checked_terminal_goals().is_empty()
    {
        return Err("Task258B4C immutable final handoff mismatch".to_owned());
    }
    Ok(SourceStatementRouteOutput {
        typed_ast,
        resolved,
        left_lookup_ordinal: *left_lookup_ordinal,
        right_lookup_ordinal: *right_lookup_ordinal,
        reference_use_ordinals,
    })
}

fn build_source_statement_b4b_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB4BRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    if source_text != SOURCE_STATEMENT_B4B_TEXT || symbols.module_id() != &module {
        return Err("Task258B4B source or symbol module mismatch".to_owned());
    }
    let lower =
        source_formula_composition_output_with_source(ast, module.clone(), symbols, source_text)
            .ok_or_else(|| "Task258B4B lower Task257B2 route is missing".to_owned())??;
    let lower_typed = &lower.typed_ast;
    let primary = lower_typed
        .source_term()
        .ok_or_else(|| "Task258B4B primary handoff is missing".to_owned())?;
    let atomic = lower_typed
        .source_atomic_formula()
        .ok_or_else(|| "Task258B4B atomic handoff is missing".to_owned())?;
    let composite = lower_typed
        .source_composite_formula()
        .ok_or_else(|| "Task258B4B composite handoff is missing".to_owned())?;
    let composition = lower_typed
        .source_formula_composition()
        .ok_or_else(|| "Task258B4B composition handoff is missing".to_owned())?;
    if lower_typed.nodes().len() != 124
        || lower_typed.nodes().root().is_some()
        || lower_typed.source_statement().is_some()
        || (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ) != (16, 0, 16)
        || (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_segments().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ) != (8, 0, 0, 0, 0, 0, 0, 16, 16)
        || (
            composite.formulas().len(),
            composite.wrappers().len(),
            composite.roots().len(),
            composite.binders().len(),
            composite.type_sites().len(),
            composite.edges().len(),
            composite.requests().len(),
        ) != (8, 6, 1, 1, 1, 7, 9)
        || (
            composition.atomic_edges().len(),
            composition.bound_uses().len(),
        ) != (8, 0)
        || (
            composite.binding_env().contexts().len(),
            composite.binding_env().bindings().len(),
            composite.binding_env().diagnostics().len(),
        ) != (2, 1, 4)
    {
        return Err("Task258B4B lower Task257B2 profile mismatch".to_owned());
    }
    const LOWER_OWNED_NODES: [usize; 42] = [
        4, 56, 57, 58, 59, 61, 63, 64, 66, 68, 69, 71, 72, 74, 76, 77, 79, 81, 82, 84, 85, 87, 88,
        90, 92, 93, 95, 97, 98, 100, 101, 103, 105, 106, 108, 110, 111, 113, 114, 116, 117, 118,
    ];
    if lower_typed.nodes().iter().any(|(id, node)| {
        let lower_owned = LOWER_OWNED_NODES.contains(&id.index());
        lower_owned == (node.kind.as_str() == "source.formula.composition.unowned")
            || node.anchor != SourceAnchor::Range(ast.nodes()[id.index()].range)
            || node.recovery != mizar_checker::typed_ast::NodeRecoveryState::Normal
    }) {
        return Err("Task258B4B lower owned-site partition mismatch".to_owned());
    }
    let nodes = lower_typed
        .nodes()
        .iter()
        .map(|(id, node)| {
            let mut node = node.clone();
            if id == TypedNodeId::new(120) {
                if node.kind.as_str() != "source.formula.composition.unowned" {
                    return Err("Task258B4B theorem node is already lower-owned".to_owned());
                }
                node.kind = "source.statement.theorem".into();
            }
            Ok(node)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let arena = TypedArena::try_new(None, nodes).map_err(|error| error.to_string())?;

    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B4B_LABEL)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return Err("Task258B4B requires one exact resolver theorem owner".to_owned());
    };
    let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner.symbol().clone(),
        ast.source_id,
        &module,
    )
    .map_err(|error| error.to_string())?;
    let labels = symbols
        .labels()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B4B_LABEL);
    let [label] = labels.as_slice() else {
        return Err("Task258B4B resolver theorem label mismatch".to_owned());
    };
    let expected_origin_path = LabelOriginPath::new(format!(
        "{}::{}::theorem::{}",
        module.package().as_str(),
        module.path().as_str(),
        SOURCE_STATEMENT_B4B_LABEL,
    ));
    let contribution = symbols
        .contributions()
        .get(owner.contribution())
        .ok_or_else(|| "Task258B4B resolver contribution is missing".to_owned())?;
    if (
        symbols.symbols().len(),
        symbols.labels().len(),
        symbols.definitions().len(),
        symbols.contributions().len(),
        symbols.imports().len(),
    ) != (1, 1, 1, 1, 0)
        || checked_owner.source_range() != range(ast.source_id, 0, 165)
        || checked_owner.origin().structural_path() != [2, 0]
        || checked_owner.origin().import_edge().is_some()
        || owner.contribution().index() != 0
        || label.origin_path() != &expected_origin_path
        || label.kind() != LabelKind::Theorem
        || label.visibility() != Visibility::Public
        || label.export_status() != ExportStatus::Exported
        || label.namespace() != &namespace
        || label.primary_spelling() != SOURCE_STATEMENT_B4B_LABEL
        || label.origin() != checked_owner.origin()
        || label.contribution() != owner.contribution()
        || label.recovery() != RecoveryState::Normal
        || contribution.effects().labels() != [expected_origin_path]
    {
        return Err("Task258B4B resolver theorem provenance mismatch".to_owned());
    }

    let mut inputs = SourceStatementB4BRouteInputs {
        binding_env: composite.binding_env().clone(),
        arena,
        primary: primary.clone(),
        atomic: atomic.clone(),
        composite: composite.clone(),
        composition: composition.clone(),
        statement: SourceStatementHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: owner.symbol().clone(),
                contribution: owner.contribution(),
                site: TypedSiteRef::Node(TypedNodeId::new(120)),
                source_range: range(ast.source_id, 0, 165),
                spelling: SOURCE_STATEMENT_B4B_LABEL.to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: vec![SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(0),
                formula: SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)),
                site: TypedSiteRef::Node(TypedNodeId::new(120)),
                source_range: range(ast.source_id, 0, 165),
                source_ordinal: 0,
                spelling: SOURCE_STATEMENT_B4B_SPELLING.to_owned(),
                kind: SourceStatementKind::TheoremProposition,
                recovery: SourceStatementRecovery::Normal,
            }],
            contexts: vec![SourceStatementContextInput {
                statement: SourceStatementId::new(0),
                binding_context: BindingContextId::new(0),
                source_range: range(ast.source_id, 0, 165),
                visible_bindings: Vec::new(),
            }],
            input_facts: Vec::new(),
            candidate_facts: vec![SourceStatementCandidateFactInput {
                statement: SourceStatementId::new(0),
                context: SourceStatementContextId::new(0),
                ordinal: 0,
                kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                formula: SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)),
            }],
        },
    };
    mutate(&mut inputs);
    let statement = SourceStatementProducer::build_with_formula_composition(
        inputs.statement,
        symbols,
        &inputs.binding_env,
        &inputs.primary,
        &inputs.atomic,
        &inputs.composite,
        &inputs.composition,
        &inputs.arena,
    )
    .map_err(|error| error.to_string())?;
    if !inputs.primary.references().is_empty() {
        return Err("Task258B4B lower reference profile mismatch".to_owned());
    }
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
    .with_source_formula_composition_statement(inputs.composite, inputs.composition, statement)
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
    if typed_ast.source_statement() != resolved.source_statement()
        || typed_ast.source_composite_formula() != resolved.source_composite_formula()
        || typed_ast.source_formula_composition() != resolved.source_formula_composition()
        || typed_ast.source_statement_references().is_some()
        || typed_ast.source_statement_witnesses().is_some()
        || !typed_ast.contexts().is_empty()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
        || !resolved.checked_formulas().is_empty()
        || !resolved.statement_semantics().is_empty()
        || !resolved.checked_proofs().is_empty()
        || !resolved.checked_proof_nodes().is_empty()
        || !resolved.checked_terminal_goals().is_empty()
    {
        return Err("Task258B4B immutable final handoff mismatch".to_owned());
    }
    Ok(SourceStatementRouteOutput {
        typed_ast,
        resolved,
        left_lookup_ordinal: 0,
        right_lookup_ordinal: 0,
        reference_use_ordinals: Vec::new(),
    })
}

fn build_source_statement_b4a_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceStatementB4ARouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    if source_text != SOURCE_STATEMENT_B4A_TEXT || symbols.module_id() != &module {
        return Err("Task258B4A source or symbol module mismatch".to_owned());
    }
    let lower =
        source_formula_composition_output_with_source(ast, module.clone(), symbols, source_text)
            .ok_or_else(|| "Task258B4A lower Task257B1 route is missing".to_owned())??;
    let lower_typed = &lower.typed_ast;
    if lower_typed.nodes().len() != 26
        || lower_typed.source_term().is_none()
        || lower_typed.source_atomic_formula().is_none()
        || lower_typed.source_composite_formula().is_none()
        || lower_typed.source_formula_composition().is_none()
        || lower_typed.source_statement().is_some()
    {
        return Err("Task258B4A lower Task257B1 profile mismatch".to_owned());
    }
    let nodes = lower_typed
        .nodes()
        .iter()
        .map(|(id, node)| {
            let mut node = node.clone();
            if id == TypedNodeId::new(22) {
                node.kind = "source.statement.theorem".into();
            }
            node
        })
        .collect::<Vec<_>>();
    let arena = TypedArena::try_new(lower_typed.nodes().root(), nodes)
        .map_err(|error| error.to_string())?;

    let namespace = NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, SOURCE_STATEMENT_B4A_LABEL)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return Err("Task258B4A requires one exact resolver theorem owner".to_owned());
    };
    let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner.symbol().clone(),
        ast.source_id,
        &module,
    )
    .map_err(|error| error.to_string())?;
    if checked_owner.source_range() != range(ast.source_id, 0, 78)
        || checked_owner.origin().structural_path() != [2, 0]
        || checked_owner.origin().import_edge().is_some()
        || owner.contribution().index() != 0
    {
        return Err("Task258B4A resolver theorem provenance mismatch".to_owned());
    }

    let composite = lower_typed
        .source_composite_formula()
        .cloned()
        .ok_or_else(|| "Task258B4A composite handoff is missing".to_owned())?;
    let mut inputs = SourceStatementB4ARouteInputs {
        binding_env: composite.binding_env().clone(),
        arena,
        primary: lower_typed
            .source_term()
            .cloned()
            .ok_or_else(|| "Task258B4A primary handoff is missing".to_owned())?,
        atomic: lower_typed
            .source_atomic_formula()
            .cloned()
            .ok_or_else(|| "Task258B4A atomic handoff is missing".to_owned())?,
        composite,
        composition: lower_typed
            .source_formula_composition()
            .cloned()
            .ok_or_else(|| "Task258B4A composition handoff is missing".to_owned())?,
        statement: SourceStatementHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: owner.symbol().clone(),
                contribution: owner.contribution(),
                site: TypedSiteRef::Node(TypedNodeId::new(22)),
                source_range: range(ast.source_id, 0, 78),
                spelling: SOURCE_STATEMENT_B4A_LABEL.to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: vec![SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(0),
                formula: SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)),
                site: TypedSiteRef::Node(TypedNodeId::new(22)),
                source_range: range(ast.source_id, 0, 78),
                source_ordinal: 0,
                spelling: SOURCE_STATEMENT_B4A_SPELLING.to_owned(),
                kind: SourceStatementKind::TheoremProposition,
                recovery: SourceStatementRecovery::Normal,
            }],
            contexts: vec![SourceStatementContextInput {
                statement: SourceStatementId::new(0),
                binding_context: BindingContextId::new(0),
                source_range: range(ast.source_id, 0, 78),
                visible_bindings: Vec::new(),
            }],
            input_facts: Vec::new(),
            candidate_facts: vec![SourceStatementCandidateFactInput {
                statement: SourceStatementId::new(0),
                context: SourceStatementContextId::new(0),
                ordinal: 0,
                kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                formula: SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)),
            }],
        },
    };
    mutate(&mut inputs);
    let statement = SourceStatementProducer::build_with_formula_composition(
        inputs.statement,
        symbols,
        &inputs.binding_env,
        &inputs.primary,
        &inputs.atomic,
        &inputs.composite,
        &inputs.composition,
        &inputs.arena,
    )
    .map_err(|error| error.to_string())?;
    let reference_use_ordinals = inputs
        .primary
        .references()
        .iter()
        .map(|(_, row)| row.use_ordinal())
        .collect::<Vec<_>>();
    let [left_lookup_ordinal, right_lookup_ordinal] = reference_use_ordinals.as_slice() else {
        return Err("Task258B4A lower reference profile mismatch".to_owned());
    };
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
    .with_source_formula_composition_statement(inputs.composite, inputs.composition, statement)
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
    if typed_ast.source_statement() != resolved.source_statement()
        || typed_ast.source_composite_formula() != resolved.source_composite_formula()
        || typed_ast.source_formula_composition() != resolved.source_formula_composition()
        || typed_ast.source_statement_references().is_some()
        || typed_ast.source_statement_witnesses().is_some()
        || !typed_ast.contexts().is_empty()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
        || !resolved.checked_formulas().is_empty()
        || !resolved.statement_semantics().is_empty()
        || !resolved.checked_proofs().is_empty()
        || !resolved.checked_proof_nodes().is_empty()
        || !resolved.checked_terminal_goals().is_empty()
    {
        return Err("Task258B4A immutable final handoff mismatch".to_owned());
    }
    Ok(SourceStatementRouteOutput {
        typed_ast,
        resolved,
        left_lookup_ordinal: *left_lookup_ordinal,
        right_lookup_ordinal: *right_lookup_ordinal,
        reference_use_ordinals,
    })
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

fn build_source_statement_b3m1_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M1Extraction,
    mutate: impl FnOnce(&mut SourceStatementB3M1RouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2a_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2AExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2ARouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b1_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B1Extraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B1RouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2a_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2AExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2ARouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b1a_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B1AExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B1ARouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b1b1_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B1B1Extraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B1B1RouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b2a_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B2AExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B2ARouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b2b_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B2BExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B2BRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b2c_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B2CExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B2CRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b3a_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B3AExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3ARouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b3b_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B3BExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3BRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b3c_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B3CExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3CRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b3d_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B3DExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3DRouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn build_source_statement_b3m2b2b3e_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementB3M2B2B3EExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3M2B2B3ERouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output(ast, module, symbols, extracted.into(), mutate)
}

fn task258b3m2b2b3a_mutated_module(module: &ModuleId) -> ModuleId {
    ModuleId::new(
        module.package().clone(),
        ModulePath::new(format!(
            "{}.task258b3m2b2b3a-mutated",
            module.path().as_str()
        )),
    )
}

fn task258b3m2b2b3a_mutated_source_id(source_id: SourceId) -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "a3".repeat(Hash::BYTE_LEN)
    ))
    .expect("Task258B3M2B2B3A test snapshot");
    let allocator = InMemorySessionIdAllocator::new();
    loop {
        let candidate = allocator
            .next_source_id(snapshot)
            .expect("Task258B3M2B2B3A test source id");
        if candidate != source_id {
            return candidate;
        }
    }
}

#[cfg(test)]
fn mutate_task258b3m2b2b3b_binding_env(
    input: &BindingEnv,
    field: usize,
) -> Result<BindingEnvParts, String> {
    let mut source_id = input.source_id();
    let mut module_id = input.module_id().clone();
    let mut contexts = input
        .contexts()
        .iter()
        .map(|(_, row)| BindingContextDraft {
            owner: row.owner.clone(),
            parent: row.parent,
            layer: row.layer,
            lexical_scope: row.lexical_scope.clone(),
            bindings: row.bindings.clone(),
            visible_bindings: row.visible_bindings.clone(),
            recovery: row.recovery,
        })
        .collect::<Vec<_>>();
    let mut bindings = input
        .bindings()
        .iter()
        .map(|(_, row)| BindingDraft {
            spelling: row.spelling.clone(),
            kind: row.kind,
            identity: row.identity.clone(),
            owner_context: row.owner_context,
            declaration_range: row.declaration_range,
            visible_after_ordinal: row.visible_after_ordinal,
            type_site: row.type_site.clone(),
            status: row.status,
            captured: row.captured.clone(),
            diagnostics: row.diagnostics.clone(),
            recovery: row.recovery,
        })
        .collect::<Vec<_>>();
    let mut diagnostics = input
        .diagnostics()
        .iter()
        .map(|(_, row)| BindingDiagnosticDraft {
            source_range: row.source_range,
            class: row.class,
            severity: row.severity,
            message_key: row.message_key.clone(),
            recovery: row.recovery,
        })
        .collect::<Vec<_>>();

    match field {
        0 => source_id = task258b3m2b2b3a_mutated_source_id(source_id),
        1 => module_id = task258b3m2b2b3a_mutated_module(&module_id),
        2 => contexts.clear(),
        3 => contexts.push(BindingContextDraft {
            owner: BindingContextOwner::Generated("Task258B3M2B2B3B extra".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![0, 1])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        }),
        4..18 => {
            let offset = field - 4;
            let context = offset / 7;
            match offset % 7 {
                0 => {
                    contexts[context].owner =
                        BindingContextOwner::Generated(format!("mutated-context-{context}"))
                }
                1 => {
                    contexts[context].parent = if context == 0 {
                        Some(BindingContextId::new(1))
                    } else {
                        None
                    }
                }
                2 => contexts[context].layer = BindingContextLayer::Expression,
                3 => {
                    contexts[context].lexical_scope = if context == 0 {
                        Some(LocalTermScope::new(vec![9]))
                    } else {
                        None
                    }
                }
                4 => {
                    if context == 0 {
                        contexts[context].bindings.clear();
                    } else {
                        contexts[context].bindings.push(BindingId::new(0));
                    }
                }
                5 => contexts[context].visible_bindings.clear(),
                6 => contexts[context].recovery = BindingContextRecovery::Degraded,
                _ => unreachable!(),
            }
        }
        18 => bindings.clear(),
        19 => {
            let mut extra = bindings[0].clone();
            extra.spelling.push('!');
            bindings.push(extra);
        }
        20 => bindings[0].spelling.push('!'),
        21 => bindings[0].kind = BindingKind::Generated,
        22 => {
            bindings[0].identity = BinderIdentity::Generated {
                context: BindingContextId::new(0),
                counter: 99,
            }
        }
        23 => bindings[0].owner_context = BindingContextId::new(1),
        24 => bindings[0].declaration_range.start += 1,
        25 => bindings[0].visible_after_ordinal += 1,
        26 => bindings[0].type_site = BindingTypeSite::Deferred("mutated".to_owned()),
        27 => bindings[0].status = BindingStatus::Degraded,
        28 => {
            bindings[0].captured = CapturedFreeVariables::new(vec![BinderIdentity::Generated {
                context: BindingContextId::new(0),
                counter: 99,
            }])
        }
        29 => bindings[0].diagnostics = vec![BindingDiagnosticId::new(0)],
        30 => bindings[0].recovery = BindingRecoveryState::Degraded,
        31 => diagnostics.push(BindingDiagnosticDraft {
            source_range: None,
            class: BindingDiagnosticClass::UnsupportedSourceShape,
            severity: BindingDiagnosticSeverity::Note,
            message_key: "checker.binding.task258b3m2b2b3b.mutated".to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        }),
        _ => return Err(format!("Task48: unknown mutation field {field}")),
    }

    let mut context_table = BindingContextTable::new();
    for context in contexts {
        context_table.insert(context);
    }
    let mut binding_table = BindingTable::new();
    for binding in bindings {
        binding_table.insert(binding);
    }
    let mut diagnostic_table = BindingDiagnosticTable::new();
    for diagnostic in diagnostics {
        diagnostic_table.insert(diagnostic);
    }
    Ok(BindingEnvParts {
        source_id,
        module_id,
        contexts: context_table,
        bindings: binding_table,
        diagnostics: diagnostic_table,
    })
}

#[cfg(test)]
fn task258b3m2b2b3b_primary_input(
    input: &SourcePrimaryTermHandoff,
) -> SourcePrimaryTermHandoffInput {
    SourcePrimaryTermHandoffInput {
        source_id: input.source_id(),
        module_id: input.module_id().clone(),
        terms: input
            .terms()
            .iter()
            .map(|(_, row)| SourcePrimaryTermInput {
                site: row.site().clone(),
                source_range: row.source_range(),
                source_ordinal: row.source_ordinal(),
                context: row.context(),
                recovery: row.recovery(),
                spelling: row.spelling().to_owned(),
                kind: row.kind(),
                role: row.role(),
                parent: row.parent(),
            })
            .collect(),
        references: input
            .references()
            .iter()
            .map(|(_, row)| SourcePrimaryTermReferenceInput {
                term: row.term(),
                binding: row.binding(),
                role: row.role(),
            })
            .collect(),
        numeric_type_requests: input
            .numeric_type_requests()
            .iter()
            .map(|(_, row)| SourceNumericTypeRequestInput {
                term: row.term(),
                owner: row.owner().clone(),
                source_range: row.source_range(),
                spelling: row.spelling().to_owned(),
                request_ordinal: row.request_ordinal(),
            })
            .collect(),
    }
}

#[cfg(test)]
fn mutate_task258b3m2b2b3b_primary_input(
    input: &mut SourcePrimaryTermHandoffInput,
    field: usize,
) -> Result<(), String> {
    match field {
        0 => input.source_id = task258b3m2b2b3a_mutated_source_id(input.source_id),
        1 => input.module_id = task258b3m2b2b3a_mutated_module(&input.module_id),
        2 => input.terms.clear(),
        3 => input.terms.push(input.terms[0].clone()),
        4..40 => {
            let offset = field - 4;
            let term = offset / 9;
            let other = (term + 1) % 4;
            match offset % 9 {
                0 => input.terms[term].site = input.terms[other].site.clone(),
                1 => input.terms[term].source_range.start += 1,
                2 => input.terms[term].source_ordinal += 5,
                3 => {
                    input.terms[term].context =
                        BindingContextId::new(1 - input.terms[term].context.index())
                }
                4 => input.terms[term].recovery = SourcePrimaryTermRecovery::Degraded,
                5 => input.terms[term].spelling.push('!'),
                6 => input.terms[term].kind = SourcePrimaryTermKind::Numeral,
                7 => input.terms[term].role = SourcePrimaryTermRole::CurrentDefinitionResult,
                8 => input.terms[term].parent = Some(SourcePrimaryTermId::new(other)),
                _ => unreachable!(),
            }
        }
        40 => input.references.clear(),
        41 => input.references.push(input.references[0].clone()),
        42..54 => {
            let offset = field - 42;
            let reference = offset / 3;
            match offset % 3 {
                0 => {
                    input.references[reference].term = SourcePrimaryTermId::new((reference + 1) % 4)
                }
                1 => input.references[reference].binding = BindingId::new(99),
                2 => {
                    input.references[reference].role = SourcePrimaryTermReferenceRole::LocalConstant
                }
                _ => unreachable!(),
            }
        }
        54 => input
            .numeric_type_requests
            .push(SourceNumericTypeRequestInput {
                term: SourcePrimaryTermId::new(0),
                owner: input.terms[0].site.clone(),
                source_range: input.terms[0].source_range,
                spelling: input.terms[0].spelling.clone(),
                request_ordinal: 0,
            }),
        _ => return Err(format!("Task252: unknown mutation field {field}")),
    }
    Ok(())
}

#[cfg(test)]
fn mutate_task258b3m2b2b3d_primary_input(
    input: &mut SourcePrimaryTermHandoffInput,
    field: usize,
) -> Result<(), String> {
    match field {
        0 => input.source_id = task258b3m2b2b3a_mutated_source_id(input.source_id),
        1 => input.module_id = task258b3m2b2b3a_mutated_module(&input.module_id),
        2 => input.terms.clear(),
        3 => input.terms.push(input.terms[0].clone()),
        4..49 => {
            let offset = field - 4;
            let term = offset / 9;
            let other = (term + 1) % 5;
            match offset % 9 {
                0 => input.terms[term].site = input.terms[other].site.clone(),
                1 => input.terms[term].source_range.start += 1,
                2 => input.terms[term].source_ordinal += 5,
                3 => {
                    input.terms[term].context =
                        BindingContextId::new(1 - input.terms[term].context.index())
                }
                4 => input.terms[term].recovery = SourcePrimaryTermRecovery::Degraded,
                5 => input.terms[term].spelling.push('!'),
                6 => {
                    input.terms[term].kind =
                        if input.terms[term].kind == SourcePrimaryTermKind::Numeral {
                            SourcePrimaryTermKind::VariableReference
                        } else {
                            SourcePrimaryTermKind::Numeral
                        }
                }
                7 => input.terms[term].role = SourcePrimaryTermRole::CurrentDefinitionResult,
                8 => input.terms[term].parent = Some(SourcePrimaryTermId::new(other)),
                _ => unreachable!(),
            }
        }
        49 => input.references.clear(),
        50 => input.references.push(input.references[0].clone()),
        51..63 => {
            let offset = field - 51;
            let reference = offset / 3;
            match offset % 3 {
                0 => {
                    input.references[reference].term =
                        SourcePrimaryTermId::new((input.references[reference].term.index() + 1) % 5)
                }
                1 => input.references[reference].binding = BindingId::new(99),
                2 => {
                    input.references[reference].role = SourcePrimaryTermReferenceRole::LocalConstant
                }
                _ => unreachable!(),
            }
        }
        63 => input.numeric_type_requests.clear(),
        64 => input
            .numeric_type_requests
            .push(input.numeric_type_requests[0].clone()),
        65 => input.numeric_type_requests[0].term = SourcePrimaryTermId::new(0),
        66 => input.numeric_type_requests[0].owner = input.terms[0].site.clone(),
        67 => input.numeric_type_requests[0].source_range.start += 1,
        68 => input.numeric_type_requests[0].spelling.push('!'),
        69 => input.numeric_type_requests[0].request_ordinal += 1,
        _ => return Err(format!("Task252: unknown mutation field {field}")),
    }
    Ok(())
}

#[cfg(test)]
fn task258b3m2b2b3b_set_input(input: &SourceSetTermHandoff) -> SourceSetTermHandoffInput {
    SourceSetTermHandoffInput {
        source_id: input.source_id(),
        module_id: input.module_id().clone(),
        terms: input
            .terms()
            .iter()
            .map(|(_, row)| SourceSetTermInput {
                site: row.site().clone(),
                source_range: row.source_range(),
                source_ordinal: row.source_ordinal(),
                context: row.context(),
                recovery: row.recovery(),
                spelling: row.spelling().to_owned(),
                kind: row.kind(),
            })
            .collect(),
        wrappers: Vec::new(),
        generators: Vec::new(),
        type_sites: Vec::new(),
        conditions: Vec::new(),
        edges: Vec::new(),
        requests: input
            .requests()
            .iter()
            .map(|(_, row)| SourceSetRequestInput {
                term: row.term(),
                ordinal: row.ordinal(),
                kind: row.kind(),
                generator: row.generator(),
                type_site: row.type_site(),
            })
            .collect(),
    }
}

#[cfg(test)]
fn mutate_task258b3m2b2b3b_set_input(
    input: &mut SourceSetTermHandoffInput,
    field: usize,
) -> Result<(), String> {
    match field {
        0 => input.source_id = task258b3m2b2b3a_mutated_source_id(input.source_id),
        1 => input.module_id = task258b3m2b2b3a_mutated_module(&input.module_id),
        2 => input.terms.clear(),
        3 => input.terms.push(input.terms[0].clone()),
        4 => {
            input.terms[0].site = TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(27))
        }
        5 => input.terms[0].source_range.start += 1,
        6 => input.terms[0].source_ordinal += 1,
        7 => input.terms[0].context = BindingContextId::new(0),
        8 => input.terms[0].recovery = SourceSetTermRecovery::Degraded,
        9 => input.terms[0].spelling.push('!'),
        10 => input.terms[0].kind = SourceSetTermKind::Choice,
        11 => input.wrappers.push(SourceSetWrapperInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            context: input.terms[0].context,
            recovery: SourceSetTermRecovery::Normal,
            spelling: "( { } )".to_owned(),
        }),
        12 => input.generators.push(SourceSetGeneratorInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            spelling: "x being set".to_owned(),
            context: input.terms[0].context,
            recovery: SourceSetTermRecovery::Normal,
            type_site: SourceSetTypeSiteId::new(0),
        }),
        13 => input.type_sites.push(SourceSetTypeSiteInput {
            owner: SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0),
                role: SourceSetTypeRole::ChoiceTarget,
            },
            site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            spelling: "set".to_owned(),
            head_site: input.terms[0].site.clone(),
            head_range: input.terms[0].source_range,
            head_spelling: "set".to_owned(),
            context: input.terms[0].context,
            recovery: SourceSetTermRecovery::Normal,
            head: SourceSetTypeHead::BuiltinSet,
        }),
        14 => input.conditions.push(SourceSetConditionInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            colon_site: input.terms[0].site.clone(),
            colon_range: input.terms[0].source_range,
            colon_spelling: ":".to_owned(),
            condition_site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            spelling: "x = x".to_owned(),
            recovery: SourceSetTermRecovery::Normal,
        }),
        15 => input.edges.push(SourceSetEdgeInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            role: SourceSetEdgeRole::EnumerationElement,
            target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
        }),
        16 => input.requests.clear(),
        17 => input.requests.push(input.requests[0].clone()),
        18 => input.requests[0].term = SourceSetTermId::new(1),
        19 => input.requests[0].ordinal += 1,
        20 => input.requests[0].kind = SourceSetRequestKind::ChoiceNonempty,
        21 => input.requests[0].generator = Some(SourceSetGeneratorId::new(0)),
        22 => input.requests[0].type_site = Some(SourceSetTypeSiteId::new(0)),
        _ => return Err(format!("Task255: unknown mutation field {field}")),
    }
    Ok(())
}

#[cfg(test)]
fn task258b3m2b2b3c_set_input(input: &SourceSetTermHandoff) -> SourceSetTermHandoffInput {
    SourceSetTermHandoffInput {
        source_id: input.source_id(),
        module_id: input.module_id().clone(),
        terms: input
            .terms()
            .iter()
            .map(|(_, row)| SourceSetTermInput {
                site: row.site().clone(),
                source_range: row.source_range(),
                source_ordinal: row.source_ordinal(),
                context: row.context(),
                recovery: row.recovery(),
                spelling: row.spelling().to_owned(),
                kind: row.kind(),
            })
            .collect(),
        wrappers: Vec::new(),
        generators: Vec::new(),
        type_sites: input
            .type_sites()
            .iter()
            .map(|(_, row)| SourceSetTypeSiteInput {
                owner: row.owner(),
                site: row.site().clone(),
                source_range: row.source_range(),
                spelling: row.spelling().to_owned(),
                head_site: row.head_site().clone(),
                head_range: row.head_range(),
                head_spelling: row.head_spelling().to_owned(),
                context: row.context(),
                recovery: row.recovery(),
                head: row.head(),
            })
            .collect(),
        conditions: Vec::new(),
        edges: Vec::new(),
        requests: input
            .requests()
            .iter()
            .map(|(_, row)| SourceSetRequestInput {
                term: row.term(),
                ordinal: row.ordinal(),
                kind: row.kind(),
                generator: row.generator(),
                type_site: row.type_site(),
            })
            .collect(),
    }
}

#[cfg(test)]
fn mutate_task258b3m2b2b3c_set_input(
    input: &mut SourceSetTermHandoffInput,
    field: usize,
) -> Result<(), String> {
    match field {
        0 => input.source_id = task258b3m2b2b3a_mutated_source_id(input.source_id),
        1 => input.module_id = task258b3m2b2b3a_mutated_module(&input.module_id),
        2 => input.terms.clear(),
        3 => input.terms.push(input.terms[0].clone()),
        4 => {
            input.terms[0].site = TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(27))
        }
        5 => input.terms[0].source_range.start += 1,
        6 => input.terms[0].source_ordinal += 1,
        7 => input.terms[0].context = BindingContextId::new(0),
        8 => input.terms[0].recovery = SourceSetTermRecovery::Degraded,
        9 => input.terms[0].spelling.push('!'),
        10 => input.terms[0].kind = SourceSetTermKind::Enumeration,
        11 => input.wrappers.push(SourceSetWrapperInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            context: input.terms[0].context,
            recovery: SourceSetTermRecovery::Normal,
            spelling: "( the set )".to_owned(),
        }),
        12 => input.generators.push(SourceSetGeneratorInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            spelling: "x being set".to_owned(),
            context: input.terms[0].context,
            recovery: SourceSetTermRecovery::Normal,
            type_site: SourceSetTypeSiteId::new(0),
        }),
        13 => input.type_sites.clear(),
        14 => input.type_sites.push(input.type_sites[0].clone()),
        15 => {
            input.type_sites[0].owner = SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0),
                role: SourceSetTypeRole::QuaTarget,
            }
        }
        16 => {
            input.type_sites[0].site =
                TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(33))
        }
        17 => input.type_sites[0].source_range.start += 1,
        18 => input.type_sites[0].spelling.push('!'),
        19 => {
            input.type_sites[0].head_site =
                TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(34))
        }
        20 => input.type_sites[0].head_range.start += 1,
        21 => input.type_sites[0].head_spelling.push('!'),
        22 => input.type_sites[0].context = BindingContextId::new(0),
        23 => input.type_sites[0].recovery = SourceSetTermRecovery::Degraded,
        24 => input.type_sites[0].head = SourceSetTypeHead::BuiltinObject,
        25 => input.conditions.push(SourceSetConditionInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            colon_site: input.terms[0].site.clone(),
            colon_range: input.terms[0].source_range,
            colon_spelling: ":".to_owned(),
            condition_site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            spelling: "x = x".to_owned(),
            recovery: SourceSetTermRecovery::Normal,
        }),
        26 => input.edges.push(SourceSetEdgeInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            role: SourceSetEdgeRole::EnumerationElement,
            target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
        }),
        27 => input.requests.clear(),
        28 => input.requests.push(input.requests[0].clone()),
        29..39 => {
            let offset = field - 29;
            let request = offset / 5;
            match offset % 5 {
                0 => input.requests[request].term = SourceSetTermId::new(1),
                1 => input.requests[request].ordinal = 1 - input.requests[request].ordinal,
                2 => {
                    input.requests[request].kind = if request == 0 {
                        SourceSetRequestKind::ResultType
                    } else {
                        SourceSetRequestKind::ChoiceNonempty
                    }
                }
                3 => input.requests[request].generator = Some(SourceSetGeneratorId::new(0)),
                4 => {
                    input.requests[request].type_site = if request == 0 {
                        None
                    } else {
                        Some(SourceSetTypeSiteId::new(0))
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => return Err(format!("Task255: unknown mutation field {field}")),
    }
    Ok(())
}

#[cfg(test)]
fn task258b3m2b2b3d_set_input(input: &SourceSetTermHandoff) -> SourceSetTermHandoffInput {
    SourceSetTermHandoffInput {
        source_id: input.source_id(),
        module_id: input.module_id().clone(),
        terms: input
            .terms()
            .iter()
            .map(|(_, row)| SourceSetTermInput {
                site: row.site().clone(),
                source_range: row.source_range(),
                source_ordinal: row.source_ordinal(),
                context: row.context(),
                recovery: row.recovery(),
                spelling: row.spelling().to_owned(),
                kind: row.kind(),
            })
            .collect(),
        wrappers: Vec::new(),
        generators: Vec::new(),
        type_sites: input
            .type_sites()
            .iter()
            .map(|(_, row)| SourceSetTypeSiteInput {
                owner: row.owner(),
                site: row.site().clone(),
                source_range: row.source_range(),
                spelling: row.spelling().to_owned(),
                head_site: row.head_site().clone(),
                head_range: row.head_range(),
                head_spelling: row.head_spelling().to_owned(),
                context: row.context(),
                recovery: row.recovery(),
                head: row.head(),
            })
            .collect(),
        conditions: Vec::new(),
        edges: input
            .edges()
            .iter()
            .map(|(_, row)| SourceSetEdgeInput {
                term: row.term(),
                ordinal: row.ordinal(),
                role: row.role(),
                target: row.target(),
            })
            .collect(),
        requests: input
            .requests()
            .iter()
            .map(|(_, row)| SourceSetRequestInput {
                term: row.term(),
                ordinal: row.ordinal(),
                kind: row.kind(),
                generator: row.generator(),
                type_site: row.type_site(),
            })
            .collect(),
    }
}

#[cfg(test)]
fn mutate_task258b3m2b2b3d_set_input(
    input: &mut SourceSetTermHandoffInput,
    field: usize,
) -> Result<(), String> {
    match field {
        0 => input.source_id = task258b3m2b2b3a_mutated_source_id(input.source_id),
        1 => input.module_id = task258b3m2b2b3a_mutated_module(&input.module_id),
        2 => input.terms.clear(),
        3 => input.terms.push(input.terms[0].clone()),
        4 => {
            input.terms[0].site = TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(36))
        }
        5 => input.terms[0].source_range.start += 1,
        6 => input.terms[0].source_ordinal += 1,
        7 => input.terms[0].context = BindingContextId::new(0),
        8 => input.terms[0].recovery = SourceSetTermRecovery::Degraded,
        9 => input.terms[0].spelling.push('!'),
        10 => input.terms[0].kind = SourceSetTermKind::Choice,
        11 => input.wrappers.push(SourceSetWrapperInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            context: input.terms[0].context,
            recovery: SourceSetTermRecovery::Normal,
            spelling: "( 4 qua set )".to_owned(),
        }),
        12 => input.generators.push(SourceSetGeneratorInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            spelling: "x being set".to_owned(),
            context: input.terms[0].context,
            recovery: SourceSetTermRecovery::Normal,
            type_site: SourceSetTypeSiteId::new(0),
        }),
        13 => input.type_sites.clear(),
        14 => input.type_sites.push(input.type_sites[0].clone()),
        15 => {
            input.type_sites[0].owner = SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0),
                role: SourceSetTypeRole::ChoiceTarget,
            }
        }
        16 => {
            input.type_sites[0].site =
                TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(35))
        }
        17 => input.type_sites[0].source_range.start += 1,
        18 => input.type_sites[0].spelling.push('!'),
        19 => {
            input.type_sites[0].head_site =
                TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(36))
        }
        20 => input.type_sites[0].head_range.start += 1,
        21 => input.type_sites[0].head_spelling.push('!'),
        22 => input.type_sites[0].context = BindingContextId::new(0),
        23 => input.type_sites[0].recovery = SourceSetTermRecovery::Degraded,
        24 => input.type_sites[0].head = SourceSetTypeHead::BuiltinObject,
        25 => input.conditions.push(SourceSetConditionInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            colon_site: input.terms[0].site.clone(),
            colon_range: input.terms[0].source_range,
            colon_spelling: ":".to_owned(),
            condition_site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            spelling: "x = x".to_owned(),
            recovery: SourceSetTermRecovery::Normal,
        }),
        26 => input.edges.clear(),
        27 => input.edges.push(input.edges[0].clone()),
        28 => input.edges[0].term = SourceSetTermId::new(1),
        29 => input.edges[0].ordinal += 1,
        30 => input.edges[0].role = SourceSetEdgeRole::EnumerationElement,
        31 => input.edges[0].target = SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
        32 => input.requests.clear(),
        33 => input.requests.push(input.requests[0].clone()),
        34..44 => {
            let offset = field - 34;
            let request = offset / 5;
            match offset % 5 {
                0 => input.requests[request].term = SourceSetTermId::new(1),
                1 => input.requests[request].ordinal = 1 - input.requests[request].ordinal,
                2 => {
                    input.requests[request].kind = if request == 0 {
                        SourceSetRequestKind::ResultType
                    } else {
                        SourceSetRequestKind::ChoiceNonempty
                    }
                }
                3 => input.requests[request].generator = Some(SourceSetGeneratorId::new(0)),
                4 => {
                    input.requests[request].type_site = if request == 0 {
                        None
                    } else {
                        Some(SourceSetTypeSiteId::new(0))
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => return Err(format!("Task255: unknown mutation field {field}")),
    }
    Ok(())
}

#[cfg(test)]
fn task258b3m2b2b3e_set_input(input: &SourceSetTermHandoff) -> SourceSetTermHandoffInput {
    SourceSetTermHandoffInput {
        source_id: input.source_id(),
        module_id: input.module_id().clone(),
        terms: input
            .terms()
            .iter()
            .map(|(_, row)| SourceSetTermInput {
                site: row.site().clone(),
                source_range: row.source_range(),
                source_ordinal: row.source_ordinal(),
                context: row.context(),
                recovery: row.recovery(),
                spelling: row.spelling().to_owned(),
                kind: row.kind(),
            })
            .collect(),
        wrappers: Vec::new(),
        generators: input
            .generators()
            .iter()
            .map(|(_, row)| SourceSetGeneratorInput {
                term: row.term(),
                ordinal: row.ordinal(),
                site: row.site().clone(),
                source_range: row.source_range(),
                spelling: row.spelling().to_owned(),
                context: row.context(),
                recovery: row.recovery(),
                type_site: row.type_site(),
            })
            .collect(),
        type_sites: input
            .type_sites()
            .iter()
            .map(|(_, row)| SourceSetTypeSiteInput {
                owner: row.owner(),
                site: row.site().clone(),
                source_range: row.source_range(),
                spelling: row.spelling().to_owned(),
                head_site: row.head_site().clone(),
                head_range: row.head_range(),
                head_spelling: row.head_spelling().to_owned(),
                context: row.context(),
                recovery: row.recovery(),
                head: row.head(),
            })
            .collect(),
        conditions: Vec::new(),
        edges: input
            .edges()
            .iter()
            .map(|(_, row)| SourceSetEdgeInput {
                term: row.term(),
                ordinal: row.ordinal(),
                role: row.role(),
                target: row.target(),
            })
            .collect(),
        requests: input
            .requests()
            .iter()
            .map(|(_, row)| SourceSetRequestInput {
                term: row.term(),
                ordinal: row.ordinal(),
                kind: row.kind(),
                generator: row.generator(),
                type_site: row.type_site(),
            })
            .collect(),
    }
}

#[cfg(test)]
fn mutate_task258b3m2b2b3e_set_input(
    input: &mut SourceSetTermHandoffInput,
    field: usize,
) -> Result<(), String> {
    match field {
        0 => input.source_id = task258b3m2b2b3a_mutated_source_id(input.source_id),
        1 => input.module_id = task258b3m2b2b3a_mutated_module(&input.module_id),
        2 => input.terms.clear(),
        3 => input.terms.push(input.terms[0].clone()),
        4 => {
            input.terms[0].site = TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(42))
        }
        5 => input.terms[0].source_range.start += 1,
        6 => input.terms[0].source_ordinal += 1,
        7 => input.terms[0].context = BindingContextId::new(0),
        8 => input.terms[0].recovery = SourceSetTermRecovery::Degraded,
        9 => input.terms[0].spelling.push('!'),
        10 => input.terms[0].kind = SourceSetTermKind::Qua,
        11 => input.wrappers.push(SourceSetWrapperInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            context: input.terms[0].context,
            recovery: SourceSetTermRecovery::Normal,
            spelling: "( { 3 where candidate255 is set } )".to_owned(),
        }),
        12 => input.generators.clear(),
        13 => input.generators.push(input.generators[0].clone()),
        14 => input.generators[0].term = SourceSetTermId::new(1),
        15 => input.generators[0].ordinal += 1,
        16 => {
            input.generators[0].site =
                TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(42))
        }
        17 => input.generators[0].source_range.start += 1,
        18 => input.generators[0].spelling.push('!'),
        19 => input.generators[0].context = BindingContextId::new(0),
        20 => input.generators[0].recovery = SourceSetTermRecovery::Degraded,
        21 => input.generators[0].type_site = SourceSetTypeSiteId::new(1),
        22 => input.type_sites.clear(),
        23 => input.type_sites.push(input.type_sites[0].clone()),
        24 => {
            input.type_sites[0].owner = SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0),
                role: SourceSetTypeRole::QuaTarget,
            }
        }
        25 => {
            input.type_sites[0].site =
                TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(40))
        }
        26 => input.type_sites[0].source_range.start += 1,
        27 => input.type_sites[0].spelling.push('!'),
        28 => {
            input.type_sites[0].head_site =
                TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(41))
        }
        29 => input.type_sites[0].head_range.start += 1,
        30 => input.type_sites[0].head_spelling.push('!'),
        31 => input.type_sites[0].context = BindingContextId::new(0),
        32 => input.type_sites[0].recovery = SourceSetTermRecovery::Degraded,
        33 => input.type_sites[0].head = SourceSetTypeHead::BuiltinObject,
        34 => input.conditions.push(SourceSetConditionInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            colon_site: input.terms[0].site.clone(),
            colon_range: input.terms[0].source_range,
            colon_spelling: ":".to_owned(),
            condition_site: input.terms[0].site.clone(),
            source_range: input.terms[0].source_range,
            spelling: "x = x".to_owned(),
            recovery: SourceSetTermRecovery::Normal,
        }),
        35 => input.edges.clear(),
        36 => input.edges.push(input.edges[0].clone()),
        37 => input.edges[0].term = SourceSetTermId::new(1),
        38 => input.edges[0].ordinal += 1,
        39 => input.edges[0].role = SourceSetEdgeRole::QuaBase,
        40 => input.edges[0].target = SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
        41 => input.requests.clear(),
        42 => input.requests.push(input.requests[0].clone()),
        43..53 => {
            let offset = field - 43;
            let request = offset / 5;
            match offset % 5 {
                0 => input.requests[request].term = SourceSetTermId::new(1),
                1 => input.requests[request].ordinal = 1 - input.requests[request].ordinal,
                2 => {
                    input.requests[request].kind = if request == 0 {
                        SourceSetRequestKind::ResultType
                    } else {
                        SourceSetRequestKind::ChoiceNonempty
                    }
                }
                3 => {
                    input.requests[request].generator = if request == 0 {
                        None
                    } else {
                        Some(SourceSetGeneratorId::new(0))
                    }
                }
                4 => {
                    input.requests[request].type_site = if request == 0 {
                        None
                    } else {
                        Some(SourceSetTypeSiteId::new(0))
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => return Err(format!("Task255: unknown mutation field {field}")),
    }
    Ok(())
}

fn mutate_task258b3m2b2b3a_atomic_input(
    input: &mut SourceAtomicFormulaHandoffInput,
    owner_symbol: &SymbolId,
    owner_contribution: SourceContributionId,
    field: usize,
) -> Result<(), String> {
    match field {
        0 => input.formulas.clear(),
        1 => input.formulas.push(input.formulas[0].clone()),
        2..16 => {
            let offset = field - 2;
            let formula = offset / 7;
            let member = offset % 7;
            let other = 1 - formula;
            match member {
                0 => input.formulas[formula].site = input.formulas[other].site.clone(),
                1 => input.formulas[formula].source_range.start += 1,
                2 => input.formulas[formula].source_ordinal += 2,
                3 => {
                    input.formulas[formula].context =
                        BindingContextId::new(1 - input.formulas[formula].context.index())
                }
                4 => input.formulas[formula].recovery = SourceAtomicFormulaRecovery::Degraded,
                5 => input.formulas[formula].spelling.push('!'),
                6 => input.formulas[formula].kind = SourceAtomicFormulaKind::Inequality,
                _ => unreachable!(),
            }
        }
        16..32 => {
            let offset = field - 16;
            let edge = offset / 4;
            match offset % 4 {
                0 => {
                    input.edges[edge].formula =
                        SourceAtomicFormulaId::new(1 - input.edges[edge].formula.index())
                }
                1 => input.edges[edge].ordinal += 2,
                2 => input.edges[edge].role = SourceAtomicEdgeRole::PredicateLeftArgument,
                3 => {
                    input.edges[edge].target =
                        SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(99))
                }
                _ => unreachable!(),
            }
        }
        32..60 => {
            let offset = field - 32;
            let request = offset / 7;
            match offset % 7 {
                0 => {
                    input.requests[request].formula =
                        SourceAtomicFormulaId::new(1 - input.requests[request].formula.index())
                }
                1 => input.requests[request].ordinal += 2,
                2 => {
                    input.requests[request].kind =
                        SourceAtomicRequestKind::PredicateCandidateSignature
                }
                3 => input.requests[request].edge = None,
                4 => {
                    input.requests[request].candidate = Some(
                        mizar_checker::source_atomic_formula::SourcePredicateCandidateId::new(0),
                    )
                }
                5 => {
                    input.requests[request].type_site = Some(
                        mizar_checker::source_atomic_formula::SourceAssertionTypeSiteId::new(0),
                    )
                }
                6 => {
                    input.requests[request].attribute = Some(
                        mizar_checker::source_atomic_formula::SourceAssertionAttributeId::new(0),
                    )
                }
                _ => unreachable!(),
            }
        }
        60 => input.module_id = task258b3m2b2b3a_mutated_module(&input.module_id),
        61 => input.source_id = task258b3m2b2b3a_mutated_source_id(input.source_id),
        62 => input.wrappers.push(SourceAtomicWrapperInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            site: input.formulas[0].site.clone(),
            source_range: input.formulas[0].source_range,
            context: input.formulas[0].context,
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "( x = x )".to_owned(),
        }),
        63 => input.predicate_segments.push(SourcePredicateSegmentInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            site: input.formulas[0].site.clone(),
            source_range: input.formulas[0].source_range,
            context: input.formulas[0].context,
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = x".to_owned(),
            head: SourcePredicateHeadId::new(0),
            polarity: SourcePredicateSegmentPolarityInput::Positive,
            left_edge: SourceAtomicEdgeId::new(0),
            right_edge: SourceAtomicEdgeId::new(1),
        }),
        64 => input.predicate_heads.push(SourcePredicateHeadInput {
            formula: SourceAtomicFormulaId::new(0),
            site: input.formulas[0].site.clone(),
            source_range: input.formulas[0].source_range,
            context: input.formulas[0].context,
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "=".to_owned(),
            left_arity: 1,
            right_arity: 1,
        }),
        65 => input.candidates.push(SourcePredicateCandidateInput {
            head: SourcePredicateHeadId::new(0),
            ordinal: 0,
            symbol: owner_symbol.clone(),
            contribution: owner_contribution,
        }),
        66 => input.type_sites.push(SourceAssertionTypeSiteInput {
            formula: SourceAtomicFormulaId::new(0),
            site: input.formulas[0].site.clone(),
            source_range: input.formulas[0].source_range,
            spelling: "set".to_owned(),
            head_site: input.formulas[0].site.clone(),
            head_range: input.formulas[0].source_range,
            head_spelling: "set".to_owned(),
            context: input.formulas[0].context,
            recovery: SourceAtomicFormulaRecovery::Normal,
            head: SourceAssertionTypeHead::BuiltinSet,
        }),
        67 => input.attributes.push(SourceAssertionAttributeInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            site: input.formulas[0].site.clone(),
            source_range: input.formulas[0].source_range,
            spelling: "attr".to_owned(),
            target_site: input.formulas[0].site.clone(),
            target_range: input.formulas[0].source_range,
            target_spelling: "attr".to_owned(),
            context: input.formulas[0].context,
            recovery: SourceAtomicFormulaRecovery::Normal,
            symbol: owner_symbol.clone(),
            contribution: owner_contribution,
            polarity: SourceAssertionAttributePolarityInput::Positive,
        }),
        68 => input.edges.clear(),
        69 => {
            let extra = input.edges[0].clone();
            input.edges.push(extra);
        }
        70 => input.requests.clear(),
        71 => {
            let extra = input.requests[0].clone();
            input.requests.push(extra);
        }
        _ => return Err(format!("Task256: unknown mutation field {field}")),
    }
    Ok(())
}

fn mutate_task258b3m2b2b3a_statement_input(
    input: &mut SourceStatementHandoffInput,
    field: usize,
) -> Result<(), String> {
    match field {
        0 => input.owners.clear(),
        1 => input.owners.push(input.owners[0].clone()),
        2 => input.owners[0].site = input.statements[1].site.clone(),
        3 => input.owners[0].source_range.start += 1,
        4 => input.owners[0].spelling.push('!'),
        5 => input.owners[0].recovery = SourceStatementRecovery::Degraded,
        6 => input.statements.clear(),
        7 => input.statements.push(input.statements[0].clone()),
        8..24 => {
            let offset = field - 8;
            let statement = offset / 8;
            let other = 1 - statement;
            match offset % 8 {
                0 => input.statements[statement].owner = SourceTheoremOwnerId::new(1),
                1 => input.statements[statement].context = SourceStatementContextId::new(other),
                2 => {
                    input.statements[statement].formula =
                        SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
                }
                3 => input.statements[statement].site = input.statements[other].site.clone(),
                4 => input.statements[statement].source_range.start += 1,
                5 => input.statements[statement].source_ordinal += 3,
                6 => input.statements[statement].spelling.push('!'),
                7 => input.statements[statement].recovery = SourceStatementRecovery::Degraded,
                _ => unreachable!(),
            }
        }
        24 => input.contexts.clear(),
        25 => input.contexts.push(input.contexts[0].clone()),
        26..34 => {
            let offset = field - 26;
            let context = offset / 4;
            let other = 1 - context;
            match offset % 4 {
                0 => input.contexts[context].statement = SourceStatementId::new(other),
                1 => input.contexts[context].binding_context = BindingContextId::new(other),
                2 => input.contexts[context].source_range.start += 1,
                3 => input.contexts[context].visible_bindings.clear(),
                _ => unreachable!(),
            }
        }
        34 => input.input_facts.clear(),
        35 => input.input_facts.push(input.input_facts[0].clone()),
        36..46 => {
            let offset = field - 36;
            let fact = offset / 5;
            let other = 1 - fact;
            match offset % 5 {
                0 => input.input_facts[fact].statement = SourceStatementId::new(other),
                1 => input.input_facts[fact].context = SourceStatementContextId::new(other),
                2 => input.input_facts[fact].ordinal += 1,
                3 => input.input_facts[fact].binding = BindingId::new(99),
                4 => input.input_facts[fact].uses.swap(0, 1),
                _ => unreachable!(),
            }
        }
        46 => input.candidate_facts.clear(),
        47 => input.candidate_facts.push(input.candidate_facts[0].clone()),
        48..56 => {
            let offset = field - 48;
            let candidate = offset / 4;
            let other = 1 - candidate;
            match offset % 4 {
                0 => input.candidate_facts[candidate].statement = SourceStatementId::new(other),
                1 => {
                    input.candidate_facts[candidate].context = SourceStatementContextId::new(other)
                }
                2 => input.candidate_facts[candidate].ordinal += 1,
                3 => {
                    input.candidate_facts[candidate].formula =
                        SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
                }
                _ => unreachable!(),
            }
        }
        56 => input.module_id = task258b3m2b2b3a_mutated_module(&input.module_id),
        57 => input.source_id = task258b3m2b2b3a_mutated_source_id(input.source_id),
        58 => {
            input.owners[0].symbol = SymbolId::new(
                input.module_id.clone(),
                mizar_resolve::resolved_ast::LocalSymbolId::new("Task258B3M2B2B3A/mutated-owner"),
                mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                    "{}::Task258B3M2B2B3A/mutated-owner",
                    input.module_id.path().as_str()
                )),
            )
        }
        59 => {
            let mut contributions = mizar_resolve::env::SourceContributionIndex::new();
            let kind = mizar_resolve::env::ContributionKind::LocalSource {
                source_id: input.source_id,
            };
            let anchor = SourceAnchor::Range(input.owners[0].source_range);
            contributions.insert(input.module_id.clone(), kind.clone(), anchor.clone());
            input.owners[0].contribution =
                contributions.insert(input.module_id.clone(), kind, anchor);
        }
        60 => input.statements[0].kind = SourceStatementKind::Conclusion,
        61 => input.statements[1].kind = SourceStatementKind::TheoremProposition,
        _ => return Err(format!("Task258: unknown mutation field {field}")),
    }
    Ok(())
}

fn mutate_task258b3m2b2b3a_witness_input(
    input: &mut SourceStatementWitnessHandoffInput,
    field: usize,
) -> Result<(), String> {
    match field {
        0 => input.witnesses.clear(),
        1 => input.witnesses.push(input.witnesses[0].clone()),
        2 => input.witnesses[0].owner = SourceTheoremOwnerId::new(1),
        3 => input.witnesses[0].binding_context = BindingContextId::new(0),
        4 => {
            input.witnesses[0].term =
                SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
        }
        5 => input.witnesses[0].take_site = input.witnesses[0].site.clone(),
        6 => input.witnesses[0].take_range.start += 1,
        7 => input.witnesses[0].site = input.witnesses[0].take_site.clone(),
        8 => input.witnesses[0].source_range.end -= 1,
        9 => input.witnesses[0].source_ordinal = 0,
        10 => input.witnesses[0].ordinal = 1,
        11 => input.witnesses[0].spelling.push('!'),
        12 => input.witnesses[0].recovery = SourceStatementRecovery::Degraded,
        13 => input.witnesses[0].name = Some(SourceStatementWitnessNameId::new(0)),
        14 => input.names.push(SourceStatementWitnessNameInput {
            witness: mizar_checker::source_statement::SourceStatementWitnessId::new(0),
            site: input.witnesses[0].site.clone(),
            source_range: input.witnesses[0].source_range,
            spelling: "y".to_owned(),
            recovery: SourceStatementRecovery::Normal,
        }),
        15 => input.module_id = task258b3m2b2b3a_mutated_module(&input.module_id),
        16 => input.source_id = task258b3m2b2b3a_mutated_source_id(input.source_id),
        17 => input.witnesses[0].kind = SourceStatementWitnessKind::Named,
        18 => {
            input.witnesses[0].term =
                SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(1))
        }
        19 => {
            input.witnesses[0].term =
                SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(0))
        }
        20 => {
            input.witnesses[0].term =
                SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0))
        }
        _ => return Err(format!("B3A: unknown mutation field {field}")),
    }
    Ok(())
}

fn build_source_statement_witness_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementWitnessExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3RouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    build_source_statement_witness_output_with_controls(
        ast,
        module,
        symbols,
        extracted,
        mutate,
        None,
        |_| {},
    )
}

fn build_source_statement_witness_output_with_controls(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    extracted: SourceStatementWitnessExtraction,
    mutate: impl FnOnce(&mut SourceStatementB3RouteInputs),
    stage_mutation: Option<(SourceStatementB3M2B2B3AStage, usize)>,
    post_auth_mutate: impl FnOnce(&mut SourceStatementB3RouteInputs),
) -> Result<SourceStatementRouteOutput, String> {
    let structure_family_count = [
        extracted.structure_node,
        extracted.structure_selector_node,
        extracted.structure_update_node,
    ]
    .into_iter()
    .flatten()
    .count();
    let has_structure = structure_family_count == 1;
    let lower_family_count = usize::from(extracted.application_node.is_some())
        + usize::from(has_structure)
        + usize::from(extracted.set_term_node.is_some());
    if lower_family_count > 1 || structure_family_count > 1 {
        return Err(format!(
            "{} lower-family dependencies must be mutually exclusive",
            extracted.task
        ));
    }
    let has_imported_lower = extracted.application_node.is_some() || has_structure;
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
        || (!has_imported_lower && !symbols.imports().is_empty())
        || (has_imported_lower && symbols.imports().len() != 1)
        || !symbols
            .symbols()
            .visible_candidates(&namespace, "y")
            .is_empty()
        || label.origin_path() != &expected_origin_path
        || label.kind() != LabelKind::Theorem
        || label.visibility() != Visibility::Public
        || label.export_status() != ExportStatus::Exported
        || label.namespace() != &namespace
        || label.primary_spelling() != extracted.label
        || extracted.label_range
            != range(
                ast.source_id,
                if has_imported_lower { 56 } else { 27 },
                if has_imported_lower { 56 } else { 27 } + extracted.label.len(),
            )
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
    for witness in &extracted.witnesses {
        owned_node_kinds.insert(witness.site.node().index(), "source.statement-witness.item");
        if let Some((name_site, _)) = &witness.name {
            owned_node_kinds.insert(name_site.node().index(), "source.statement-witness.name");
        }
    }
    for site in &extracted.formula_sites {
        owned_node_kinds.insert(site.node().index(), "source.formula.atomic.equality");
    }
    if let Some(application_node) = extracted.application_node {
        let application_owned = if let Some(wrapper) = extracted.application_wrapper_node {
            wrapped_imported_source_application_owned_node_kinds(
                ast,
                &module,
                symbols,
                WrappedImportedApplicationSite {
                    application: application_node,
                    wrapper,
                },
            )
        } else {
            unwrapped_imported_source_application_owned_node_kinds(
                ast,
                &module,
                symbols,
                application_node,
            )
        }
        .ok_or_else(|| format!("{} application ownership selector mismatch", extracted.task))?;
        owned_node_kinds.extend(application_owned);
    }
    let mut structure_owned_node_kinds = None;
    if let Some(structure_node) = extracted.structure_node {
        let structure_owned = imported_structure_constructor_owned_node_kinds(
            ast,
            &module,
            symbols,
            extracted.source_text,
            ImportedStructureConstructorSite {
                constructor: structure_node,
            },
        )
        .ok_or_else(|| format!("{} structure ownership selector mismatch", extracted.task))?;
        owned_node_kinds.extend(structure_owned.clone());
        structure_owned_node_kinds = Some(structure_owned);
    } else if let Some(selector_node) = extracted.structure_selector_node {
        let structure_owned = imported_structure_selector_owned_node_kinds(
            ast,
            &module,
            symbols,
            extracted.source_text,
            ImportedStructureSelectorSite {
                selector: selector_node,
            },
        )
        .ok_or_else(|| format!("{} structure ownership selector mismatch", extracted.task))?;
        owned_node_kinds.extend(structure_owned.clone());
        structure_owned_node_kinds = Some(structure_owned);
    } else if let Some(update_node) = extracted.structure_update_node {
        let structure_owned = imported_structure_update_owned_node_kinds(
            ast,
            &module,
            symbols,
            extracted.source_text,
            ImportedStructureUpdateSite {
                update: update_node,
            },
        )
        .ok_or_else(|| format!("{} structure ownership selector mismatch", extracted.task))?;
        owned_node_kinds.extend(structure_owned.clone());
        structure_owned_node_kinds = Some(structure_owned);
    }
    let roots = extracted
        .term_sites
        .iter()
        .enumerate()
        .map(|(index, site)| {
            (
                site.node().index(),
                BindingContextId::new(usize::from(index >= 2)),
            )
        })
        .collect::<Vec<_>>();
    let parts = source_term_parts_for_context_roots(
        ast,
        module.clone(),
        &binding_env,
        roots.iter().copied(),
        &owned_node_kinds,
    )?;
    let application = if let Some(application_node) = extracted.application_node {
        let application = if let Some(wrapper) = extracted.application_wrapper_node {
            wrapped_imported_source_application_handoff_in_context(
                ast,
                &module,
                symbols,
                &binding_env,
                &parts,
                WrappedImportedApplicationSite {
                    application: application_node,
                    wrapper,
                },
                BindingContextId::new(1),
            )
        } else {
            unwrapped_imported_source_application_handoff_in_context(
                ast,
                &module,
                symbols,
                &binding_env,
                &parts,
                application_node,
                BindingContextId::new(1),
            )
        };
        match application {
            Some(Ok(application)) => Some(application),
            Some(Err(error)) => return Err(error),
            None => {
                return Err(format!(
                    "{} application handoff selector mismatch",
                    extracted.task
                ));
            }
        }
    } else {
        None
    };
    let structure = if let Some(structure_node) = extracted.structure_node {
        let structure_parts = source_term_parts_for_context_roots(
            ast,
            module.clone(),
            &binding_env,
            roots.iter().copied(),
            structure_owned_node_kinds
                .as_ref()
                .ok_or_else(|| format!("{} structure ownership is missing", extracted.task))?,
        )?;
        match imported_structure_constructor_handoff_in_context(
            ast,
            &module,
            symbols,
            &binding_env,
            &structure_parts,
            extracted.source_text,
            ImportedStructureConstructorSite {
                constructor: structure_node,
            },
            BindingContextId::new(1),
        ) {
            Some(Ok(structure)) if structure_parts.handoff == parts.handoff => Some(structure),
            Some(Ok(_)) => {
                return Err(format!(
                    "{} structure primary-term dependency mismatch",
                    extracted.task
                ));
            }
            Some(Err(error)) => return Err(error),
            None => {
                return Err(format!(
                    "{} structure handoff selector mismatch",
                    extracted.task
                ));
            }
        }
    } else if let Some(selector_node) = extracted.structure_selector_node {
        let structure_parts = source_term_parts_for_context_roots(
            ast,
            module.clone(),
            &binding_env,
            roots.iter().copied(),
            structure_owned_node_kinds
                .as_ref()
                .ok_or_else(|| format!("{} structure ownership is missing", extracted.task))?,
        )?;
        match imported_structure_selector_handoff_in_context(
            ast,
            &module,
            symbols,
            &binding_env,
            &structure_parts,
            extracted.source_text,
            ImportedStructureSelectorSite {
                selector: selector_node,
            },
            BindingContextId::new(1),
        ) {
            Some(Ok(structure)) if structure_parts.handoff == parts.handoff => Some(structure),
            Some(Ok(_)) => {
                return Err(format!(
                    "{} structure primary-term dependency mismatch",
                    extracted.task
                ));
            }
            Some(Err(error)) => return Err(error),
            None => {
                return Err(format!(
                    "{} structure handoff selector mismatch",
                    extracted.task
                ));
            }
        }
    } else if let Some(update_node) = extracted.structure_update_node {
        let structure_parts = source_term_parts_for_context_roots(
            ast,
            module.clone(),
            &binding_env,
            roots.iter().copied(),
            structure_owned_node_kinds
                .as_ref()
                .ok_or_else(|| format!("{} structure ownership is missing", extracted.task))?,
        )?;
        match imported_structure_update_handoff_in_context(
            ast,
            &module,
            symbols,
            &binding_env,
            &structure_parts,
            extracted.source_text,
            ImportedStructureUpdateSite {
                update: update_node,
            },
            BindingContextId::new(1),
        ) {
            Some(Ok(structure)) if structure_parts.handoff == parts.handoff => Some(structure),
            Some(Ok(_)) => {
                return Err(format!(
                    "{} structure primary-term dependency mismatch",
                    extracted.task
                ));
            }
            Some(Err(error)) => return Err(error),
            None => {
                return Err(format!(
                    "{} structure handoff selector mismatch",
                    extracted.task
                ));
            }
        }
    } else {
        None
    };
    let (primary, arena, set_term) = if let Some(set_term_node) = extracted.set_term_node {
        let output = source_set_term_output_with_source_term_in_context(
            ast,
            module.clone(),
            binding_env.clone(),
            &[set_term_node],
            parts,
            BindingContextId::new(1),
        )?;
        let primary = output
            .typed_ast
            .source_term()
            .cloned()
            .ok_or_else(|| format!("{} set-term primary handoff is missing", extracted.task))?;
        let set_term = output
            .typed_ast
            .source_set_term()
            .cloned()
            .ok_or_else(|| format!("{} set-term handoff is missing", extracted.task))?;
        if output.resolved.source_term() != Some(&primary)
            || output.resolved.source_set_term() != Some(&set_term)
        {
            return Err(format!("{} set-term lower replay mismatch", extracted.task));
        }
        (primary, output.typed_ast.nodes().clone(), Some(set_term))
    } else {
        (parts.handoff, parts.arena, None)
    };
    if primary.terms().len() != extracted.term_ranges.len()
        || primary
            .terms()
            .iter()
            .zip(&extracted.term_ranges)
            .any(|((_, term), expected)| term.source_range() != *expected)
        || primary
            .references()
            .iter()
            .any(|(_, reference)| reference.use_ordinal() != 1)
    {
        return Err(format!("{} primary-term profile drift", extracted.task));
    }
    let mut atomic_input = task258b3_atomic_input(ast, module.clone(), &extracted);
    if let Some((SourceStatementB3M2B2B3AStage::Task256, field)) = stage_mutation {
        mutate_task258b3m2b2b3a_atomic_input(
            &mut atomic_input,
            owner_entry.symbol(),
            owner_entry.contribution(),
            field,
        )?;
    }
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
    .map_err(|error| {
        if matches!(
            stage_mutation,
            Some((SourceStatementB3M2B2B3AStage::Task256, _))
        ) {
            format!("Task256: {error}")
        } else {
            error.to_string()
        }
    })?;
    let mut statement = task258b3_statement_input(
        ast,
        module.clone(),
        owner_entry.symbol().clone(),
        owner_entry.contribution(),
        &extracted,
    );
    if let Some((SourceStatementB3M2B2B3AStage::Task258, field)) = stage_mutation {
        mutate_task258b3m2b2b3a_statement_input(&mut statement, field)?;
    }
    let mut names = Vec::new();
    let mut witnesses = Vec::new();
    for (index, witness) in extracted.witnesses.iter().enumerate() {
        let name = witness.name.as_ref().map(|(site, source_range)| {
            let name = SourceStatementWitnessNameId::new(names.len());
            names.push(SourceStatementWitnessNameInput {
                witness: mizar_checker::source_statement::SourceStatementWitnessId::new(index),
                site: site.clone(),
                source_range: *source_range,
                spelling: "y".to_owned(),
                recovery: SourceStatementRecovery::Normal,
            });
            name
        });
        witnesses.push(SourceStatementWitnessInput {
            owner: SourceTheoremOwnerId::new(0),
            binding_context: BindingContextId::new(1),
            term: if extracted.application_node.is_some() {
                SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(
                    index,
                ))
            } else if has_structure {
                SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(index))
            } else if set_term.is_some() {
                SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(index))
            } else {
                SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2 + index))
            },
            take_site: extracted.take_site.clone(),
            take_range: extracted.take_range,
            site: witness.site.clone(),
            source_range: witness.range,
            source_ordinal: 1,
            ordinal: index,
            spelling: witness.spelling.to_owned(),
            kind: if name.is_some() {
                SourceStatementWitnessKind::Named
            } else {
                SourceStatementWitnessKind::Unnamed
            },
            recovery: SourceStatementRecovery::Normal,
            name,
        });
    }
    let mut witness = SourceStatementWitnessHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        witnesses,
        names,
    };
    if let Some((SourceStatementB3M2B2B3AStage::Witness, field)) = stage_mutation {
        mutate_task258b3m2b2b3a_witness_input(&mut witness, field)?;
    }
    let expected_binding_env = binding_env.clone();
    let expected_arena = arena.clone();
    let expected_primary = primary.clone();
    let expected_atomic = atomic.clone();
    let expected_application = application.clone();
    let expected_structure = structure.clone();
    let expected_set_term = set_term.clone();
    let mut inputs = SourceStatementB3RouteInputs {
        binding_env,
        arena,
        primary,
        atomic,
        statement,
        witness,
        application,
        structure,
        set_term,
    };
    mutate(&mut inputs);
    if inputs.binding_env != expected_binding_env
        || inputs.arena != expected_arena
        || inputs.primary != expected_primary
        || inputs.atomic != expected_atomic
        || inputs.application != expected_application
        || inputs.structure != expected_structure
        || inputs.set_term != expected_set_term
    {
        return Err(format!("{} lower dependency mismatch", extracted.task));
    }
    post_auth_mutate(&mut inputs);
    let statement = SourceStatementProducer::build(
        inputs.statement,
        symbols,
        &inputs.binding_env,
        &inputs.primary,
        &inputs.atomic,
        &inputs.arena,
    )
    .map_err(|error| match stage_mutation {
        Some((SourceStatementB3M2B2B3AStage::Task256, _)) => {
            format!("Task256: {error}")
        }
        Some((SourceStatementB3M2B2B3AStage::Task258, _)) => {
            format!("Task258: {error}")
        }
        _ => error.to_string(),
    })?;
    let witnesses = match (&inputs.application, &inputs.structure, &inputs.set_term) {
        (Some(application), None, None) => SourceStatementWitnessProducer::build_with_application(
            inputs.witness,
            &statement,
            &inputs.primary,
            application,
            &inputs.arena,
        ),
        (None, Some(structure), None) => SourceStatementWitnessProducer::build_with_structure(
            inputs.witness,
            &statement,
            &inputs.primary,
            structure,
            &inputs.arena,
        ),
        (None, None, Some(set_term)) => SourceStatementWitnessProducer::build_with_set_term(
            inputs.witness,
            &statement,
            &inputs.primary,
            set_term,
            &inputs.arena,
        ),
        (None, None, None) => SourceStatementWitnessProducer::build(
            inputs.witness,
            &statement,
            &inputs.primary,
            &inputs.arena,
        ),
        _ => {
            return Err(format!(
                "{} lower-family dependencies must be mutually exclusive",
                extracted.task
            ));
        }
    }
    .map_err(|error| {
        if matches!(
            stage_mutation,
            Some((SourceStatementB3M2B2B3AStage::Witness, _))
        ) {
            let stage = match extracted.task {
                "Task258B3M2B2B3C" => "B3C",
                "Task258B3M2B2B3D" => "B3D",
                "Task258B3M2B2B3E" => "B3E",
                _ => "B3A",
            };
            format!("{stage}: {error}")
        } else {
            error.to_string()
        }
    })?;
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
    .map_err(|error| error.to_string())?;
    let typed_ast = match (inputs.application, inputs.structure, inputs.set_term) {
        (Some(application), None, None) => {
            typed_ast.with_source_application_statement_witnesses(application, statement, witnesses)
        }
        (None, Some(structure), None) => {
            typed_ast.with_source_structure_statement_witnesses(structure, statement, witnesses)
        }
        (None, None, Some(set_term)) => {
            typed_ast.with_source_set_term_statement_witnesses(set_term, statement, witnesses)
        }
        (None, None, None) => typed_ast.with_source_statement_witnesses(statement, witnesses),
        _ => {
            return Err(format!(
                "{} lower-family dependencies must be mutually exclusive",
                extracted.task
            ));
        }
    }
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
        || typed_ast.source_application().is_some() != extracted.application_node.is_some()
        || typed_ast.source_application() != resolved.source_application()
        || typed_ast.source_structure().is_some() != has_structure
        || typed_ast.source_structure() != resolved.source_structure()
        || typed_ast.source_set_term().is_some() != extracted.set_term_node.is_some()
        || typed_ast.source_set_term() != resolved.source_set_term()
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
        let first_term = extracted.atomic_term_starts[formula];
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
                let first_reference = extracted.input_fact_reference_starts[index];
                SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first_reference),
                        SourcePrimaryTermReferenceId::new(first_reference + 1),
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

fn exact_qua_witness_surface_profile(ast: &SurfaceAst, source_text: &str) -> bool {
    exact_qua_witness_surface_profile_with_mutation(
        ast,
        source_text,
        SourceStatementB3M2B2B3DSurfaceMutation::None,
    )
}

fn exact_qua_witness_surface_profile_with_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3DSurfaceMutation,
) -> bool {
    const KINDS: [&str; 54] = [
        "Token(SurfaceToken { kind: ReservedWord, text: \"reserve\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"FormulaStatementQuaWitnessSmoke\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"proof\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"take\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"4\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"qua\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
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
        "TypeHead",
        "TypeExpression",
        "QuaExpression",
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
    const RANGES: [(usize, usize); 54] = [
        (0, 7),
        (8, 9),
        (10, 13),
        (14, 17),
        (17, 18),
        (19, 26),
        (27, 58),
        (58, 59),
        (60, 61),
        (62, 63),
        (64, 65),
        (66, 71),
        (74, 78),
        (79, 80),
        (81, 84),
        (85, 88),
        (88, 89),
        (92, 96),
        (97, 98),
        (99, 100),
        (101, 102),
        (102, 103),
        (104, 107),
        (107, 108),
        (14, 17),
        (14, 17),
        (8, 17),
        (0, 18),
        (60, 61),
        (60, 61),
        (64, 65),
        (64, 65),
        (60, 65),
        (60, 65),
        (79, 80),
        (85, 88),
        (85, 88),
        (79, 88),
        (79, 88),
        (79, 88),
        (74, 89),
        (97, 98),
        (97, 98),
        (101, 102),
        (101, 102),
        (97, 102),
        (97, 102),
        (97, 102),
        (92, 103),
        (66, 107),
        (19, 108),
        (0, 108),
        (0, 108),
        (0, 108),
    ];
    const CHILDREN: [&[usize]; 54] = [
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
        &[24],
        &[1, 2, 25],
        &[0, 26, 4],
        &[8],
        &[28],
        &[10],
        &[30],
        &[29, 9, 31],
        &[32],
        &[13],
        &[15],
        &[35],
        &[34, 14, 36],
        &[37],
        &[38],
        &[12, 39, 16],
        &[18],
        &[41],
        &[20],
        &[43],
        &[42, 19, 44],
        &[45],
        &[46],
        &[17, 47, 21],
        &[11, 40, 48, 22],
        &[5, 6, 7, 33, 49, 23],
        &[27, 50],
        &[51],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            52,
        ],
    ];
    let mut kinds = KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let mut ranges = RANGES.to_vec();
    let mut recoveries = [false; 54];
    let mut children = CHILDREN
        .iter()
        .map(|children| children.to_vec())
        .collect::<Vec<_>>();
    let mut root = Some(53);
    match mutation {
        SourceStatementB3M2B2B3DSurfaceMutation::None => {}
        SourceStatementB3M2B2B3DSurfaceMutation::NodeKind(index) => {
            if let Some(kind) = kinds.get_mut(index) {
                kind.push('!');
            }
        }
        SourceStatementB3M2B2B3DSurfaceMutation::NodeRange(index) => {
            if let Some(range) = ranges.get_mut(index) {
                range.1 = range.1.saturating_add(1);
            }
        }
        SourceStatementB3M2B2B3DSurfaceMutation::NodeRecovery(index) => {
            if let Some(recovered) = recoveries.get_mut(index) {
                *recovered = !*recovered;
            }
        }
        SourceStatementB3M2B2B3DSurfaceMutation::NodeChildren(index) => {
            if let Some(node_children) = children.get_mut(index) {
                if node_children.len() > 1 {
                    node_children.rotate_left(1);
                } else {
                    node_children.push(index);
                }
            }
        }
        SourceStatementB3M2B2B3DSurfaceMutation::RootIdentity => root = None,
    }
    source_text == SOURCE_STATEMENT_B3M2B2B3D_TEXT
        && source_text.len() == 109
        && source_text.ends_with('\n')
        && ast.nodes().len() == 54
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
pub(in crate::runner) fn extract_qua_witness_source_statement_with_surface_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3DSurfaceMutation,
) -> Option<SourceStatementB3M2B2B3DExtraction> {
    if !exact_qua_witness_surface_profile_with_mutation(ast, source_text, mutation) {
        return None;
    }
    extract_qua_witness_source_statement(ast, source_text)
}

pub(in crate::runner) fn extract_qua_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B3DExtraction> {
    if !exact_qua_witness_surface_profile(ast, source_text) {
        return None;
    }
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 108)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 66, 107)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 74, 89)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 79, 88)?;
    let (qua_id, qua) = exact_surface_node(ast, SurfaceNodeKind::QuaExpression, 79, 88)?;
    let (numeral_id, _) = exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 79, 80)?;
    let (type_expression_id, _) = exact_surface_node(ast, SurfaceNodeKind::TypeExpression, 85, 88)?;
    let (type_head_id, _) = exact_surface_node(ast, SurfaceNodeKind::TypeHead, 85, 88)?;
    let (conclusion_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 92, 103)?;
    let formula_ids = [
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 60, 65)?.0,
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 97, 102)?.0,
    ];
    let term_ids = [
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 60, 61)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 64, 65)?.0,
        numeral_id,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 97, 98)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 101, 102)?.0,
    ];
    let label_id = ast
        .token_nodes()
        .iter()
        .copied()
        .find(|id| id.index() == 6)?;
    if !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, qua_id)
        || !surface_is_descendant(ast, qua_id, numeral_id)
        || !surface_is_descendant(ast, qua_id, type_expression_id)
        || !surface_is_descendant(ast, type_expression_id, type_head_id)
        || !surface_is_descendant(ast, theorem_id, formula_ids[0])
        || !surface_is_descendant(ast, proof_id, formula_ids[1])
        || surface_is_descendant(ast, qua_id, formula_ids[1])
        || ast.node(qua_id).is_none_or(|node| {
            node.children.iter().map(|id| id.index()).ne([
                numeral_id.index(),
                14,
                type_expression_id.index(),
            ])
        })
        || direct_token_texts(ast, qua).as_slice() != ["qua"]
    {
        return None;
    }
    Some(SourceStatementB3M2B2B3DExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: ast.node(label_id)?.range,
        statement_sites: [theorem_id, conclusion_id].map(surface_site),
        statement_ranges: [range(ast.source_id, 19, 108), range(ast.source_id, 92, 103)],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: [range(ast.source_id, 60, 65), range(ast.source_id, 97, 102)],
        term_sites: term_ids.map(surface_site),
        term_ranges: [
            range(ast.source_id, 60, 61),
            range(ast.source_id, 64, 65),
            range(ast.source_id, 79, 80),
            range(ast.source_id, 97, 98),
            range(ast.source_id, 101, 102),
        ],
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        set_term_node: qua_id.index(),
        proof_range: proof.range,
    })
}

fn exact_comprehension_witness_surface_profile(ast: &SurfaceAst, source_text: &str) -> bool {
    exact_comprehension_witness_surface_profile_with_mutation(
        ast,
        source_text,
        SourceStatementB3M2B2B3ESurfaceMutation::None,
    )
}

fn exact_comprehension_witness_surface_profile_with_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3ESurfaceMutation,
) -> bool {
    const KINDS: [&str; 60] = [
        "Token(SurfaceToken { kind: ReservedWord, text: \"reserve\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"for\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \";\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"theorem\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"FormulaStatementComprehensionWitnessSmoke\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \":\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"=\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"x\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"proof\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"take\" })",
        "Token(SurfaceToken { kind: ReservedSymbol, text: \"{\" })",
        "Token(SurfaceToken { kind: Numeral, text: \"3\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"where\" })",
        "Token(SurfaceToken { kind: Identifier, text: \"candidate255\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"is\" })",
        "Token(SurfaceToken { kind: ReservedWord, text: \"set\" })",
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
        "TypeHead",
        "TypeExpression",
        "ComprehensionVariableSegment",
        "SetComprehension",
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
    const RANGES: [(usize, usize); 60] = [
        (0, 7),
        (8, 9),
        (10, 13),
        (14, 17),
        (17, 18),
        (19, 26),
        (27, 68),
        (68, 69),
        (70, 71),
        (72, 73),
        (74, 75),
        (76, 81),
        (84, 88),
        (89, 90),
        (90, 91),
        (92, 97),
        (98, 110),
        (111, 113),
        (114, 117),
        (117, 118),
        (118, 119),
        (122, 126),
        (127, 128),
        (129, 130),
        (131, 132),
        (132, 133),
        (134, 137),
        (137, 138),
        (14, 17),
        (14, 17),
        (8, 17),
        (0, 18),
        (70, 71),
        (70, 71),
        (74, 75),
        (74, 75),
        (70, 75),
        (70, 75),
        (90, 91),
        (90, 91),
        (114, 117),
        (114, 117),
        (98, 117),
        (89, 118),
        (89, 118),
        (89, 118),
        (84, 119),
        (127, 128),
        (127, 128),
        (131, 132),
        (131, 132),
        (127, 132),
        (127, 132),
        (127, 132),
        (122, 133),
        (76, 137),
        (19, 138),
        (0, 138),
        (0, 138),
        (0, 138),
    ];
    const CHILDREN: [&[usize]; 60] = [
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
        &[3],
        &[28],
        &[1, 2, 29],
        &[0, 30, 4],
        &[8],
        &[32],
        &[10],
        &[34],
        &[33, 9, 35],
        &[36],
        &[14],
        &[38],
        &[18],
        &[40],
        &[16, 17, 41],
        &[13, 39, 15, 42, 19],
        &[43],
        &[44],
        &[12, 45, 20],
        &[22],
        &[47],
        &[24],
        &[49],
        &[48, 23, 50],
        &[51],
        &[52],
        &[21, 53, 25],
        &[11, 46, 54, 26],
        &[5, 6, 7, 37, 55, 27],
        &[31, 56],
        &[57],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 58,
        ],
    ];
    let mut kinds = KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let mut ranges = RANGES.to_vec();
    let mut recoveries = [false; 60];
    let mut children = CHILDREN
        .iter()
        .map(|children| children.to_vec())
        .collect::<Vec<_>>();
    let mut root = Some(59);
    match mutation {
        SourceStatementB3M2B2B3ESurfaceMutation::None => {}
        SourceStatementB3M2B2B3ESurfaceMutation::NodeKind(index) => {
            if let Some(kind) = kinds.get_mut(index) {
                kind.push('!');
            }
        }
        SourceStatementB3M2B2B3ESurfaceMutation::NodeRange(index) => {
            if let Some(range) = ranges.get_mut(index) {
                range.1 = range.1.saturating_add(1);
            }
        }
        SourceStatementB3M2B2B3ESurfaceMutation::NodeRecovery(index) => {
            if let Some(recovered) = recoveries.get_mut(index) {
                *recovered = !*recovered;
            }
        }
        SourceStatementB3M2B2B3ESurfaceMutation::NodeChildren(index) => {
            if let Some(node_children) = children.get_mut(index) {
                if node_children.len() > 1 {
                    node_children.rotate_left(1);
                } else {
                    node_children.push(index);
                }
            }
        }
        SourceStatementB3M2B2B3ESurfaceMutation::RootIdentity => root = None,
    }
    source_text == SOURCE_STATEMENT_B3M2B2B3E_TEXT
        && source_text.len() == 139
        && source_text.ends_with('\n')
        && ast.nodes().len() == 60
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
pub(in crate::runner) fn extract_comprehension_witness_source_statement_with_surface_mutation(
    ast: &SurfaceAst,
    source_text: &str,
    mutation: SourceStatementB3M2B2B3ESurfaceMutation,
) -> Option<SourceStatementB3M2B2B3EExtraction> {
    if !exact_comprehension_witness_surface_profile_with_mutation(ast, source_text, mutation) {
        return None;
    }
    extract_comprehension_witness_source_statement(ast, source_text)
}

pub(in crate::runner) fn extract_comprehension_witness_source_statement(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<SourceStatementB3M2B2B3EExtraction> {
    if !exact_comprehension_witness_surface_profile(ast, source_text) {
        return None;
    }
    let (theorem_id, theorem) = exact_surface_node(ast, SurfaceNodeKind::TheoremItem, 19, 138)?;
    let (proof_id, proof) = exact_surface_node(ast, SurfaceNodeKind::ProofBlock, 76, 137)?;
    let (take_id, take) = exact_surface_node(ast, SurfaceNodeKind::TakeStatement, 84, 119)?;
    let (witness_id, witness) = exact_surface_node(ast, SurfaceNodeKind::Witness, 89, 118)?;
    let (set_id, set) = exact_surface_node(ast, SurfaceNodeKind::SetComprehension, 89, 118)?;
    let (mapper_id, _) = exact_surface_node(ast, SurfaceNodeKind::NumeralTerm, 90, 91)?;
    let (generator_id, generator) =
        exact_surface_node(ast, SurfaceNodeKind::ComprehensionVariableSegment, 98, 117)?;
    let (type_expression_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::TypeExpression, 114, 117)?;
    let (type_head_id, _) = exact_surface_node(ast, SurfaceNodeKind::TypeHead, 114, 117)?;
    let (conclusion_id, _) =
        exact_surface_node(ast, SurfaceNodeKind::ConclusionStatement, 122, 133)?;
    let formula_ids = [
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 70, 75)?.0,
        exact_surface_node(ast, SurfaceNodeKind::BuiltinPredicateApplication, 127, 132)?.0,
    ];
    let term_ids = [
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 70, 71)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 74, 75)?.0,
        mapper_id,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 127, 128)?.0,
        exact_surface_node(ast, SurfaceNodeKind::TermReference, 131, 132)?.0,
    ];
    let label_id = ast
        .token_nodes()
        .iter()
        .copied()
        .find(|id| id.index() == 6)?;
    if !surface_is_descendant(ast, theorem_id, proof_id)
        || !surface_is_descendant(ast, proof_id, take_id)
        || !surface_is_descendant(ast, take_id, witness_id)
        || !surface_is_descendant(ast, witness_id, set_id)
        || !surface_is_descendant(ast, set_id, mapper_id)
        || !surface_is_descendant(ast, set_id, generator_id)
        || !surface_is_descendant(ast, generator_id, type_expression_id)
        || !surface_is_descendant(ast, type_expression_id, type_head_id)
        || !surface_is_descendant(ast, theorem_id, formula_ids[0])
        || !surface_is_descendant(ast, proof_id, formula_ids[1])
        || surface_is_descendant(ast, set_id, formula_ids[1])
        || ast.node(set_id).is_none_or(|node| {
            node.children
                .iter()
                .map(|id| id.index())
                .ne([13, 39, 15, generator_id.index(), 19])
        })
        || direct_token_texts(ast, set).as_slice() != ["{", "where", "}"]
        || direct_token_texts(ast, generator).as_slice() != ["candidate255", "is"]
    {
        return None;
    }
    Some(SourceStatementB3M2B2B3EExtraction {
        theorem_site: surface_site(theorem_id),
        theorem_range: theorem.range,
        label_range: ast.node(label_id)?.range,
        statement_sites: [theorem_id, conclusion_id].map(surface_site),
        statement_ranges: [
            range(ast.source_id, 19, 138),
            range(ast.source_id, 122, 133),
        ],
        formula_sites: formula_ids.map(surface_site),
        formula_ranges: [range(ast.source_id, 70, 75), range(ast.source_id, 127, 132)],
        term_sites: term_ids.map(surface_site),
        term_ranges: [
            range(ast.source_id, 70, 71),
            range(ast.source_id, 74, 75),
            range(ast.source_id, 90, 91),
            range(ast.source_id, 127, 128),
            range(ast.source_id, 131, 132),
        ],
        take_site: surface_site(take_id),
        take_range: take.range,
        witness_site: surface_site(witness_id),
        witness_range: witness.range,
        set_term_node: set_id.index(),
        proof_range: proof.range,
    })
}
