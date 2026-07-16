use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use mizar_checker::binding_env::{
    BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
    BindingContextOwner, BindingContextRecovery, BindingContextTable, BindingDiagnosticTable,
    BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingLookupResult, BindingLookupSite,
    BindingStatus, BindingTable, BindingTypeSite,
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
    SourceNodeRole,
};
use mizar_checker::type_checker::{
    DeclarationCheckingOutput, DeclarationKind, DeclarationStatus, ExpectedTypeInput,
    FormulaDeferredReason, FormulaInput, FormulaKind, FormulaStatus, ModeExpansion,
    NormalizedTypeStatus, SourceReserveBindingInput, SourceReserveDeclarationBridge,
    TermFormulaChecker, TermFormulaInferenceOutput, TermInput, TermKind, TermReference, TermStatus,
    TypeExpressionInput, TypeHeadInput, TypeNormalizer,
};
use mizar_checker::typed_ast::{
    CoercionTable, InitialObligationTable, LocalTypeContextId, NodeRecoveryState, NormalizedTypeId,
    TypeEntryActual, TypeEntryId, TypeRole, TypeStatus, TypeTable, TypedArenaBuilder, TypedAst,
    TypedAstParts, TypedNode, TypedNodeId, TypedNodeLinks, TypedSiteRef, TypingState,
};
use mizar_core::{
    binder_normalization::{NormalizedVarClass, NormalizedVarSort},
    core_ir::{CoreSourceRef, CoreVarId, CoreVarRole},
    elaborator::{
        CheckerOwnedProvenance, CoreBinderSeed, CoreContextInput, CoreVariableSeed,
        ResolvedTypedAstSummary, prepare_core_context,
    },
};
use mizar_frontend::orchestration::{DiagnosticCode, FrontendDiagnostic};
use mizar_resolve::env::SymbolEnv;
use mizar_resolve::resolved_ast::{ModuleId as ResolverModuleId, SymbolId as ResolverSymbolId};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeKind};

use crate::diagnostic::{ValidationDiagnostic, ValidationSeverity};
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{DiscoveryConfig, HarnessError, TestCase, TestPlan, build_test_plan};
use crate::staged_model::Stage;

mod declaration_symbol;
mod import_fixtures;
mod parse_only;
mod shared;
mod type_elaboration;

use declaration_symbol::{declaration_symbol_failure_diagnostic, run_declaration_symbol_case};
use import_fixtures::{ParseOnlyImportProvider, augment_type_elaboration_import_summaries};
use parse_only::{parse_only_failure_diagnostic, run_parse_only_case};
use shared::{
    FrontendRun, normalized_tests_root, normalized_workspace_root, resolver_symbol_collection,
    run_frontend,
};
#[cfg(test)]
use type_elaboration::{
    SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS, resolve_visible_attribute,
    resolve_visible_type_head, source_mode_symbol_spelling,
};
use type_elaboration::{
    SourceImportedAttributeAssertionFormula, SourceParenthesizedOperandSide,
    SourceParenthesizedReservedVariableBinaryFormula, SourceReserveExtraction,
    SourceReservedVariableAssertedHeadRelation, SourceReservedVariableBinaryFormula,
    SourceReservedVariableBinaryFormulaConfig, SourceReservedVariableBuiltinType,
    SourceReservedVariableModeDefinition, SourceReservedVariableModeRadix,
    SourceReservedVariableTypeAssertionConfig, SourceTypeExpression, direct_token_texts,
    exact_identifier_term_operand, extract_builtin_source_reserve_declarations,
    extract_builtin_source_reserve_declarations_after_node_guard,
    extract_builtin_source_type_expression, extract_source_builtin_binary_term_formula,
    extract_source_builtin_type_assertion_formula, extract_source_contradiction_formula,
    extract_source_formula_connective_quantifier, extract_source_formula_statement,
    extract_source_imported_attribute_assertion_formula,
    extract_source_imported_non_empty_attribute_assertion_formula,
    extract_source_imported_predicate_functor_formula,
    extract_source_parenthesized_reserved_variable_binary_formula_with_config,
    extract_source_reserved_variable_binary_formula, extract_source_set_enumeration_formula,
    source_binding_matches_reserved_builtin_type, source_binding_use_ordinals,
    source_mode_terminal_builtin_input, source_reserved_variable_asserted_head_relation_is_exact,
    source_reserved_variable_mode_definition_is_exact,
    source_reserved_variable_mode_expansions_are_exact,
    source_type_expression_matches_reserved_builtin_type, structural_child_ids,
    subtree_has_recovery, surface_nodes_with_kind, surface_site,
};

const ACTIVE_PARSE_ONLY_TAG: &str = "active_parse_only";
const ACTIVE_DECLARATION_SYMBOL_TAG: &str = "active_declaration_symbol";
const ACTIVE_TYPE_ELABORATION_TAG: &str = "active_type_elaboration";
const ALLOW_FRONTEND_RECOVERY_DIAGNOSTICS_TAG: &str = "allow_frontend_recovery_diagnostics";
const TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY: &str =
    "type_elaboration.external_dependency.ast_payload_extraction";
const TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_object_variable_equality.invalid_payload";
const TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_object_variable_inequality.invalid_payload";
const TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.heterogeneous_reserve_membership.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.three_edge_local_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.four_edge_local_object_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.three_edge_local_object_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.two_edge_local_object_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.chained_local_object_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.chained_local_object_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.chained_local_object_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.three_edge_local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.local_object_mode_long_chain_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.local_object_mode_long_chain_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.local_object_mode_long_chain_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_mode_long_chain_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_mode_long_chain_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.local_mode_long_chain_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_two_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_three_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_four_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_five_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_six_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_six_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_five_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_four_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_long_chain_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_two_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_three_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_long_chain_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.local_object_mode_long_chain_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.four_edge_local_object_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.four_edge_local_object_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.three_edge_local_object_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.three_edge_local_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.three_edge_local_object_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.two_edge_local_object_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.two_edge_local_object_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_reserve_declaration_equality.invalid_payload";
const TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_object_reserve_declaration_equality.invalid_payload";
const TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_object_reserve_declaration_inequality.invalid_payload";
const TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_reserve_declaration_inequality.invalid_payload";
const TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_reserve_declaration_membership.invalid_payload";
const TYPE_ELABORATION_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_heterogeneous_reserve_membership.invalid_payload";
const TYPE_ELABORATION_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.right_parenthesized_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_two_edge_local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_object_variable_equality.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_object_variable_inequality.invalid_payload";
const TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_object_variable_equality.invalid_payload";
const TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_object_variable_inequality.invalid_payload";
const TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_object_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_mode_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_object_mode_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_mode_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_mode_two_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_object_mode_two_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.three_edge_local_mode_two_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.three_edge_local_object_mode_two_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_two_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_object_mode_two_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.three_edge_local_mode_three_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.three_edge_local_object_mode_three_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_three_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_four_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.four_edge_local_object_mode_four_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.four_edge_local_object_mode_three_hop_asserted_head.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_object_mode_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.three_edge_local_mode_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.three_edge_local_object_mode_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_object_mode_radix_asserted_head.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_object_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.three_edge_local_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_object_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.three_edge_local_object_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_object_mode_asserted_head.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.chained_local_mode_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.chained_local_object_mode_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.two_edge_local_mode_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.two_edge_local_object_mode_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.three_edge_local_mode_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.three_edge_local_object_mode_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.four_edge_local_mode_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.four_edge_local_object_mode_reserved_variable_type_assertion.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_reserved_variable_type_assertion.invalid_payload";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOnlyRunReport {
    pub results: Vec<ParseOnlyCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOnlyCaseResult {
    pub id: crate::expectation::TestCaseId,
    pub expectation_path: PathBuf,
    pub status: ParseOnlyCaseStatus,
    pub actual_diagnostic_codes: Vec<String>,
    pub snapshot_failure: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseOnlyCaseStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationSymbolRunReport {
    pub results: Vec<DeclarationSymbolCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationSymbolCaseResult {
    pub id: crate::expectation::TestCaseId,
    pub expectation_path: PathBuf,
    pub status: DeclarationSymbolCaseStatus,
    pub actual_detail_keys: Vec<String>,
    pub actual_payload_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeclarationSymbolCaseStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeElaborationRunReport {
    pub results: Vec<TypeElaborationCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeElaborationCaseResult {
    pub id: crate::expectation::TestCaseId,
    pub expectation_path: PathBuf,
    pub status: TypeElaborationCaseStatus,
    pub actual_detail_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypeElaborationCaseStatus {
    Passed,
    Failed,
}

impl ParseOnlyRunReport {
    pub fn passed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == ParseOnlyCaseStatus::Passed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == ParseOnlyCaseStatus::Failed)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Warning)
            .count()
    }
}

impl DeclarationSymbolRunReport {
    pub fn passed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == DeclarationSymbolCaseStatus::Passed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == DeclarationSymbolCaseStatus::Failed)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Warning)
            .count()
    }
}

impl TypeElaborationRunReport {
    pub fn passed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == TypeElaborationCaseStatus::Passed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == TypeElaborationCaseStatus::Failed)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Warning)
            .count()
    }
}

pub fn run_parse_only_corpus(config: &DiscoveryConfig) -> Result<ParseOnlyRunReport, HarnessError> {
    let workspace_root = normalized_workspace_root(config)?;
    let tests_root = normalized_tests_root(&workspace_root, config);
    let plan = build_test_plan(config)?;
    let mut diagnostics = plan.diagnostics.clone();
    if plan.error_count() > 0 {
        return Ok(ParseOnlyRunReport {
            results: Vec::new(),
            diagnostics,
        });
    }
    diagnostics.extend(validate_active_parse_only_tags(&plan));

    let mut results = Vec::new();
    for (ordinal, case) in active_parse_only_cases(&plan).enumerate() {
        let result = run_parse_only_case(&workspace_root, &tests_root, case, ordinal);
        if result.status == ParseOnlyCaseStatus::Failed {
            diagnostics.push(parse_only_failure_diagnostic(case, &result));
        }
        results.push(result);
    }
    diagnostics.sort();

    Ok(ParseOnlyRunReport {
        results,
        diagnostics,
    })
}

pub fn run_declaration_symbol_corpus(
    config: &DiscoveryConfig,
) -> Result<DeclarationSymbolRunReport, HarnessError> {
    let workspace_root = normalized_workspace_root(config)?;
    let plan = build_test_plan(config)?;
    let mut diagnostics = plan.diagnostics.clone();
    if plan.error_count() > 0 {
        return Ok(DeclarationSymbolRunReport {
            results: Vec::new(),
            diagnostics,
        });
    }
    diagnostics.extend(validate_active_declaration_symbol_tags(&plan));

    let mut results = Vec::new();
    for (ordinal, case) in active_declaration_symbol_cases(&plan).enumerate() {
        let result = run_declaration_symbol_case(&workspace_root, case, ordinal);
        if result.status == DeclarationSymbolCaseStatus::Failed {
            diagnostics.push(declaration_symbol_failure_diagnostic(case, &result));
        }
        results.push(result);
    }
    diagnostics.sort();

    Ok(DeclarationSymbolRunReport {
        results,
        diagnostics,
    })
}

pub fn run_type_elaboration_corpus(
    config: &DiscoveryConfig,
) -> Result<TypeElaborationRunReport, HarnessError> {
    let workspace_root = normalized_workspace_root(config)?;
    let plan = build_test_plan(config)?;
    let mut diagnostics = plan.diagnostics.clone();
    if plan.error_count() > 0 {
        return Ok(TypeElaborationRunReport {
            results: Vec::new(),
            diagnostics,
        });
    }
    diagnostics.extend(validate_active_type_elaboration_tags(&plan));

    let mut results = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let result = run_type_elaboration_case(&workspace_root, case, ordinal);
        if result.status == TypeElaborationCaseStatus::Failed {
            diagnostics.push(type_elaboration_failure_diagnostic(case, &result));
        }
        results.push(result);
    }
    diagnostics.sort();

    Ok(TypeElaborationRunReport {
        results,
        diagnostics,
    })
}

pub fn active_parse_only_cases(plan: &TestPlan) -> impl Iterator<Item = &TestCase> {
    plan.cases.iter().filter(|case| is_active_parse_only(case))
}

pub fn active_declaration_symbol_cases(plan: &TestPlan) -> impl Iterator<Item = &TestCase> {
    plan.cases
        .iter()
        .filter(|case| is_active_declaration_symbol(case))
}

pub fn active_type_elaboration_cases(plan: &TestPlan) -> impl Iterator<Item = &TestCase> {
    plan.cases
        .iter()
        .filter(|case| is_active_type_elaboration(case))
}

fn is_active_parse_only(case: &TestCase) -> bool {
    has_active_parse_only_tag(case)
        && case.expectation.stage == Stage::ParseOnly
        && case.expectation.expected_phase == Some(PipelinePhase::Parse)
        && matches!(
            case.expectation.expected_outcome,
            ExpectedOutcome::Pass | ExpectedOutcome::Fail
        )
        && case
            .source_path
            .extension()
            .is_some_and(|extension| extension == "miz")
}

fn is_active_declaration_symbol(case: &TestCase) -> bool {
    has_active_declaration_symbol_tag(case)
        && case.expectation.stage == Stage::DeclarationSymbol
        && case.expectation.expected_phase == Some(PipelinePhase::Resolve)
        && matches!(
            case.expectation.expected_outcome,
            ExpectedOutcome::Pass | ExpectedOutcome::Fail
        )
        && case
            .source_path
            .extension()
            .is_some_and(|extension| extension == "miz")
}

fn is_active_type_elaboration(case: &TestCase) -> bool {
    has_active_type_elaboration_tag(case)
        && case.expectation.stage == Stage::TypeElaboration
        && case.expectation.expected_phase == Some(PipelinePhase::TypeCheck)
        && matches!(
            case.expectation.expected_outcome,
            ExpectedOutcome::Pass | ExpectedOutcome::Fail
        )
        && case
            .source_path
            .extension()
            .is_some_and(|extension| extension == "miz")
}

fn has_active_parse_only_tag(case: &TestCase) -> bool {
    case.expectation
        .tags
        .iter()
        .any(|tag| tag == ACTIVE_PARSE_ONLY_TAG)
}

fn has_active_declaration_symbol_tag(case: &TestCase) -> bool {
    case.expectation
        .tags
        .iter()
        .any(|tag| tag == ACTIVE_DECLARATION_SYMBOL_TAG)
}

fn has_active_type_elaboration_tag(case: &TestCase) -> bool {
    case.expectation
        .tags
        .iter()
        .any(|tag| tag == ACTIVE_TYPE_ELABORATION_TAG)
}

fn validate_active_parse_only_tags(plan: &TestPlan) -> Vec<ValidationDiagnostic> {
    plan.cases
        .iter()
        .filter(|case| has_active_parse_only_tag(case) && !is_active_parse_only(case))
        .map(|case| {
            ValidationDiagnostic::error(
                &case.expectation_path,
                "parse_only",
                "E-PARSE-ONLY-ACTIVE-GATE",
                format!("parse_only.active_gate.{}", case.id.0),
                "active_parse_only cases must be .miz pass/fail expectations at stage parse_only and phase parse",
            )
        })
        .collect()
}

fn validate_active_declaration_symbol_tags(plan: &TestPlan) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    for case in plan
        .cases
        .iter()
        .filter(|case| has_active_declaration_symbol_tag(case))
    {
        if !is_active_declaration_symbol(case) {
            diagnostics.push(
                ValidationDiagnostic::error(
                    &case.expectation_path,
                    "declaration_symbol",
                    "E-DECLARATION-SYMBOL-ACTIVE-GATE",
                    format!("declaration_symbol.active_gate.{}", case.id.0),
                    "active_declaration_symbol cases must be .miz pass/fail expectations at stage declaration_symbol and phase resolve",
                ),
            );
        }
        if !case.expectation.diagnostic_codes.is_empty() {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "declaration_symbol",
                "E-DECLARATION-SYMBOL-PUBLIC-DIAGNOSTIC-CODES",
                format!("declaration_symbol.public_codes.{}", case.id.0),
                "active_declaration_symbol cases must keep diagnostic_codes empty until public resolver diagnostic codes are specified; use diagnostic_payloads or stable_detail_key for internal detail keys",
            ));
        }
    }
    diagnostics
}

fn validate_active_type_elaboration_tags(plan: &TestPlan) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    for case in plan
        .cases
        .iter()
        .filter(|case| has_active_type_elaboration_tag(case))
    {
        if !is_active_type_elaboration(case) {
            diagnostics.push(
                ValidationDiagnostic::error(
                    &case.expectation_path,
                    "type_elaboration",
                    "E-TYPE-ELABORATION-ACTIVE-GATE",
                    format!("type_elaboration.active_gate.{}", case.id.0),
                    "active_type_elaboration cases must be .miz pass/fail expectations at stage type_elaboration and phase type_check",
                ),
            );
        }
        if !case.expectation.diagnostic_codes.is_empty() {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "type_elaboration",
                "E-TYPE-ELABORATION-PUBLIC-DIAGNOSTIC-CODES",
                format!("type_elaboration.public_codes.{}", case.id.0),
                "active_type_elaboration cases must keep diagnostic_codes empty until public checker diagnostic codes are specified; use diagnostic_payloads or stable_detail_key for internal detail keys",
            ));
        }
    }
    diagnostics
}

fn run_type_elaboration_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> TypeElaborationCaseResult {
    let output = run_frontend(workspace_root, case, ordinal);
    let actual_detail_keys = match output {
        Ok(output) => type_elaboration_detail_keys(workspace_root, case, output),
        Err(error) => vec![format!("frontend_error:{error}")],
    };
    let expected_detail_keys = expected_type_elaboration_detail_keys(case);
    let status = match case.expectation.expected_outcome {
        ExpectedOutcome::Pass if actual_detail_keys.is_empty() => TypeElaborationCaseStatus::Passed,
        ExpectedOutcome::Fail if actual_detail_keys == expected_detail_keys => {
            TypeElaborationCaseStatus::Passed
        }
        _ => TypeElaborationCaseStatus::Failed,
    };

    TypeElaborationCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status,
        actual_detail_keys,
    }
}

fn type_elaboration_detail_keys(
    workspace_root: &Path,
    case: &TestCase,
    output: FrontendRun,
) -> Vec<String> {
    let frontend_diagnostic_keys = frontend_detail_keys(case, &output.diagnostics);
    if !frontend_diagnostic_keys.is_empty() {
        return frontend_diagnostic_keys
            .into_iter()
            .map(|key| format!("type_elaboration.lower_stage.{key}"))
            .collect();
    }

    let Some(ast) = output.ast else {
        return vec!["type_elaboration.lower_stage.declaration_symbol.no_ast".to_owned()];
    };
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    if !resolver.detail_keys.is_empty() {
        return resolver
            .detail_keys
            .into_iter()
            .map(|key| format!("type_elaboration.lower_stage.{key}"))
            .collect();
    }

    let symbols = augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    source_type_elaboration_detail_keys(&ast, resolver.module, &symbols)
}

fn frontend_detail_keys(case: &TestCase, diagnostics: &[FrontendDiagnostic]) -> Vec<String> {
    assertion_diagnostic_codes(case, diagnostics)
        .into_iter()
        .map(|code| format!("frontend:{code}"))
        .collect()
}

fn source_type_elaboration_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Vec<String> {
    if let Some(keys) = source_four_edge_local_mode_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_object_mode_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_mode_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_object_mode_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_mode_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_mode_long_chain_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_long_chain_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_long_chain_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_long_chain_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_mode_long_chain_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_mode_long_chain_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_object_mode_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_mode_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_object_mode_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_two_edge_local_object_mode_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_two_edge_local_mode_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_two_edge_local_object_mode_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_two_edge_local_mode_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_chained_local_object_mode_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_chained_local_object_mode_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_chained_local_mode_reserved_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_chained_local_mode_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_local_mode_reserved_variable_type_assertion_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_local_mode_asserted_head_detail_keys(ast, module.clone(), symbols) {
        return keys;
    }
    if let Some(keys) =
        source_local_object_mode_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_chained_local_mode_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_chained_local_mode_radix_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_chained_local_object_mode_radix_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_two_edge_local_mode_radix_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_two_edge_local_mode_two_hop_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_two_edge_local_object_mode_two_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_three_edge_local_mode_two_hop_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_object_mode_two_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_four_edge_local_mode_two_hop_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_object_mode_two_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_mode_three_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_object_mode_three_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_mode_three_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_object_mode_three_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_four_edge_local_mode_four_hop_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_object_mode_four_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_two_edge_local_object_mode_radix_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_three_edge_local_mode_radix_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_object_mode_radix_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_four_edge_local_mode_radix_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_object_mode_radix_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_chained_local_object_mode_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_two_edge_local_mode_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_three_edge_local_mode_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_four_edge_local_mode_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_four_edge_local_object_mode_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_three_edge_local_object_mode_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_two_edge_local_object_mode_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_chained_local_mode_reserved_variable_type_assertion_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_chained_local_object_mode_reserved_variable_type_assertion_detail_keys(
            ast,
            module.clone(),
            symbols,
        )
    {
        return keys;
    }
    if let Some(keys) = source_two_edge_local_mode_reserved_variable_type_assertion_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_mode_reserved_variable_type_assertion_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_three_edge_local_object_mode_reserved_variable_type_assertion_detail_keys(
            ast,
            module.clone(),
            symbols,
        )
    {
        return keys;
    }
    if let Some(keys) = source_local_mode_long_chain_reserved_variable_type_assertion_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_local_mode_long_chain_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_local_mode_long_chain_radix_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_local_mode_long_chain_two_hop_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_local_mode_long_chain_three_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_mode_long_chain_four_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_mode_long_chain_five_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_local_mode_long_chain_six_hop_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_long_chain_six_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_long_chain_five_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_long_chain_radix_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_long_chain_two_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_long_chain_three_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_long_chain_four_hop_asserted_head_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_local_object_mode_long_chain_asserted_head_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_local_object_mode_long_chain_reserved_variable_type_assertion_detail_keys(
            ast,
            module.clone(),
            symbols,
        )
    {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_mode_reserved_variable_type_assertion_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_four_edge_local_object_mode_reserved_variable_type_assertion_detail_keys(
            ast,
            module.clone(),
            symbols,
        )
    {
        return keys;
    }
    if let Some(keys) =
        source_two_edge_local_object_mode_reserved_variable_type_assertion_detail_keys(
            ast,
            module.clone(),
            symbols,
        )
    {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_reserved_variable_type_assertion_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_local_mode_reserved_variable_equality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_local_mode_reserved_variable_inequality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_reserved_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_chained_local_mode_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_mode_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_four_edge_local_object_mode_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_mode_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_three_edge_local_object_mode_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_two_edge_local_mode_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_two_edge_local_object_mode_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_chained_local_object_mode_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_local_mode_reserved_variable_membership_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_local_object_mode_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_heterogeneous_reserve_membership_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_distinct_reserved_variable_membership_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_distinct_reserved_variable_inequality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_multiple_object_reserve_declaration_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_multiple_object_reserve_declaration_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_multiple_reserve_declaration_equality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_multiple_reserve_declaration_inequality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_multiple_reserve_declaration_membership_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_distinct_reserved_object_variable_equality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_distinct_reserved_object_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_distinct_reserved_variable_equality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_reserved_object_variable_equality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_parenthesized_reserved_object_variable_equality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_parenthesized_reserved_object_variable_inequality_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_parenthesized_reserved_variable_equality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_parenthesized_reserved_variable_inequality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_parenthesized_reserved_variable_membership_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_parenthesized_heterogeneous_reserve_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_right_parenthesized_reserved_variable_membership_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) =
        source_parenthesized_two_edge_local_mode_reserved_variable_equality_detail_keys(
            ast,
            module.clone(),
            symbols,
        )
    {
        return keys;
    }
    if let Some(keys) = source_reserved_variable_equality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_reserved_object_variable_inequality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_reserved_variable_membership_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_reserved_variable_inequality_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_reserved_object_variable_type_assertion_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_reserved_variable_type_assertion_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_formula_statement_detail_keys(ast, module.clone(), symbols) {
        return keys;
    }
    if let Some(keys) = source_contradiction_formula_detail_keys(ast, module.clone(), symbols) {
        return keys;
    }
    if let Some(keys) = source_builtin_binary_term_formula_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_builtin_type_assertion_formula_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_imported_predicate_functor_formula_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) =
        source_imported_attribute_assertion_formula_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    if let Some(keys) = source_imported_non_empty_attribute_assertion_formula_detail_keys(
        ast,
        module.clone(),
        symbols,
    ) {
        return keys;
    }
    if let Some(keys) = source_set_enumeration_formula_detail_keys(ast, module.clone(), symbols) {
        return keys;
    }
    if let Some(keys) =
        source_formula_connective_quantifier_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    let Ok(source_reserve) =
        extract_builtin_source_reserve_declarations(ast, module.clone(), symbols)
    else {
        return vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()];
    };
    let handoff = match assemble_source_reserve_checker_handoff(
        symbols,
        &source_reserve.bridge,
        source_reserve.mode_expansions.clone(),
    ) {
        Ok(handoff) => handoff,
        Err(_) => return vec!["type_elaboration.checker.typed_ast_invalid".to_owned()],
    };
    if !handoff.declarations.diagnostics().is_empty() {
        let mut keys = handoff
            .declarations
            .diagnostics()
            .canonical_iter()
            .map(|(_, diagnostic)| format!("type_elaboration.checker.{}", diagnostic.message_key))
            .collect::<Vec<_>>();
        keys.sort();
        keys.dedup();
        return keys;
    }
    if let Err(error) = assert_source_reserve_handoff(&handoff, &source_reserve.bridge) {
        let detail_key = match error.as_str() {
            "resolved source reserve count mismatch" => {
                "type_elaboration.checker.resolved_typed_ast_count_mismatch"
            }
            "resolved typed AST produced diagnostics" => {
                "type_elaboration.checker.resolved_typed_ast_diagnostics"
            }
            _ => "type_elaboration.checker.resolved_typed_ast_invalid",
        };
        return vec![detail_key.to_owned()];
    }
    if assert_source_reserve_core_summary_readiness(&handoff).is_err() {
        return vec!["type_elaboration.core.resolved_typed_ast_summary_invalid".to_owned()];
    }
    if assert_source_reserve_core_context_readiness(&handoff, &source_reserve.bridge).is_err() {
        return vec!["type_elaboration.core.context_invalid".to_owned()];
    }
    Vec::new()
}

const SOURCE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG: SourceReservedVariableTypeAssertionConfig =
    SourceReservedVariableTypeAssertionConfig {
        label: "ReservedVariableTypeAssertionPayloadBoundary",
        invalid_payload_key: TYPE_ELABORATION_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        binding_spelling: "x",
        binding_type: SourceReservedVariableBuiltinType::Set,
        binding_source_mode_spelling: None,
        mode_definitions: &[],
        asserted_type: SourceReservedVariableBuiltinType::Set,
        asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
        subject_result_role: "reserved-variable-type-assertion-subject-result",
    };

const SOURCE_RESERVED_OBJECT_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ReservedObjectVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: None,
    mode_definitions: &[],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "reserved-object-variable-type-assertion-subject-result",
};

const SOURCE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LocalModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("LocalModeTypeAssertion"),
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalModeTypeAssertionDef",
        spelling: "LocalModeTypeAssertion",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
    }],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "local-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_LOCAL_MODE_ASSERTED_HEAD_CONFIG: SourceReservedVariableTypeAssertionConfig =
    SourceReservedVariableTypeAssertionConfig {
        label: "LocalModeAssertedHeadPayloadBoundary",
        invalid_payload_key: TYPE_ELABORATION_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        binding_spelling: "x",
        binding_type: SourceReservedVariableBuiltinType::Set,
        binding_source_mode_spelling: Some("LocalModeAssertedHead"),
        mode_definitions: &[SourceReservedVariableModeDefinition {
            label: "LocalModeAssertedHeadDef",
            spelling: "LocalModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        }],
        asserted_type: SourceReservedVariableBuiltinType::Set,
        asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
            "LocalModeAssertedHead",
        ),
        subject_result_role: "local-mode-asserted-head-subject-result",
    };

const SOURCE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG: SourceReservedVariableTypeAssertionConfig =
    SourceReservedVariableTypeAssertionConfig {
        label: "LocalObjectModeAssertedHeadPayloadBoundary",
        invalid_payload_key: TYPE_ELABORATION_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        binding_spelling: "x",
        binding_type: SourceReservedVariableBuiltinType::Object,
        binding_source_mode_spelling: Some("LocalObjectModeAssertedHead"),
        mode_definitions: &[SourceReservedVariableModeDefinition {
            label: "LocalObjectModeAssertedHeadDef",
            spelling: "LocalObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        }],
        asserted_type: SourceReservedVariableBuiltinType::Object,
        asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
            "LocalObjectModeAssertedHead",
        ),
        subject_result_role: "local-object-mode-asserted-head-subject-result",
    };

const SOURCE_CHAINED_LOCAL_MODE_ASSERTED_HEAD_CONFIG: SourceReservedVariableTypeAssertionConfig =
    SourceReservedVariableTypeAssertionConfig {
        label: "ChainedLocalModeAssertedHeadPayloadBoundary",
        invalid_payload_key: TYPE_ELABORATION_CHAINED_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        binding_spelling: "x",
        binding_type: SourceReservedVariableBuiltinType::Set,
        binding_source_mode_spelling: Some("ChainModeAssertedHead"),
        mode_definitions: &[
            SourceReservedVariableModeDefinition {
                label: "BaseModeAssertedHeadDef",
                spelling: "BaseModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Builtin(
                    SourceReservedVariableBuiltinType::Set,
                ),
            },
            SourceReservedVariableModeDefinition {
                label: "ChainModeAssertedHeadDef",
                spelling: "ChainModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("BaseModeAssertedHead"),
            },
        ],
        asserted_type: SourceReservedVariableBuiltinType::Set,
        asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
            "ChainModeAssertedHead",
        ),
        subject_result_role: "chained-local-mode-asserted-head-subject-result",
    };

const SOURCE_CHAINED_LOCAL_MODE_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ChainedLocalModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("OuterModeRadixAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseModeRadixAssertedHeadDef",
            spelling: "BaseModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterModeRadixAssertedHeadDef",
            spelling: "OuterModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseModeRadixAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "BaseModeRadixAssertedHead",
    ),
    subject_result_role: "chained-local-mode-radix-asserted-head-subject-result",
};

const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ChainedLocalObjectModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterObjectModeRadixAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeRadixAssertedHeadDef",
            spelling: "BaseObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterObjectModeRadixAssertedHeadDef",
            spelling: "OuterObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeRadixAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "BaseObjectModeRadixAssertedHead",
    ),
    subject_result_role: "chained-local-object-mode-radix-asserted-head-subject-result",
};

const SOURCE_TWO_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "TwoEdgeLocalModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("OuterTwoEdgeModeRadixAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeRadixAssertedHeadDef",
            spelling: "BaseTwoEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeModeRadixAssertedHeadDef",
            spelling: "MiddleTwoEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeRadixAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeModeRadixAssertedHeadDef",
            spelling: "OuterTwoEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeRadixAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "MiddleTwoEdgeModeRadixAssertedHead",
    ),
    subject_result_role: "two-edge-local-mode-radix-asserted-head-subject-result",
};

const SOURCE_TWO_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "TwoEdgeLocalModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("OuterTwoHopModeAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoHopModeAssertedHeadDef",
            spelling: "BaseTwoHopModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoHopModeAssertedHeadDef",
            spelling: "MiddleTwoHopModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoHopModeAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoHopModeAssertedHeadDef",
            spelling: "OuterTwoHopModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoHopModeAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "MiddleTwoHopModeAssertedHead",
        asserted_spelling: "BaseTwoHopModeAssertedHead",
    },
    subject_result_role: "two-edge-local-mode-two-hop-asserted-head-subject-result",
};

const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "TwoEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterTwoHopObjectModeAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoHopObjectModeAssertedHeadDef",
            spelling: "BaseTwoHopObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoHopObjectModeAssertedHeadDef",
            spelling: "MiddleTwoHopObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoHopObjectModeAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoHopObjectModeAssertedHeadDef",
            spelling: "OuterTwoHopObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoHopObjectModeAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "MiddleTwoHopObjectModeAssertedHead",
        asserted_spelling: "BaseTwoHopObjectModeAssertedHead",
    },
    subject_result_role: "two-edge-local-object-mode-two-hop-asserted-head-subject-result",
};

const SOURCE_THREE_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("OuterThreeEdgeModeTwoHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeTwoHopAssertedHeadDef",
            spelling: "BaseThreeEdgeModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeTwoHopAssertedHeadDef",
            spelling: "InnerThreeEdgeModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeTwoHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeTwoHopAssertedHeadDef",
            spelling: "MiddleThreeEdgeModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeTwoHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeTwoHopAssertedHeadDef",
            spelling: "OuterThreeEdgeModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeTwoHopAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "MiddleThreeEdgeModeTwoHopAssertedHead",
        asserted_spelling: "InnerThreeEdgeModeTwoHopAssertedHead",
    },
    subject_result_role: "three-edge-local-mode-two-hop-asserted-head-subject-result",
};

const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterThreeEdgeObjectModeTwoHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeTwoHopAssertedHeadDef",
            spelling: "BaseThreeEdgeObjectModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeTwoHopAssertedHeadDef",
            spelling: "InnerThreeEdgeObjectModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "BaseThreeEdgeObjectModeTwoHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeTwoHopAssertedHeadDef",
            spelling: "MiddleThreeEdgeObjectModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "InnerThreeEdgeObjectModeTwoHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeTwoHopAssertedHeadDef",
            spelling: "OuterThreeEdgeObjectModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleThreeEdgeObjectModeTwoHopAssertedHead",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "MiddleThreeEdgeObjectModeTwoHopAssertedHead",
        asserted_spelling: "InnerThreeEdgeObjectModeTwoHopAssertedHead",
    },
    subject_result_role: "three-edge-local-object-mode-two-hop-asserted-head-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("TooDeepFourEdgeModeTwoHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeTwoHopAssertedHeadDef",
            spelling: "BaseFourEdgeModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeTwoHopAssertedHeadDef",
            spelling: "InnerFourEdgeModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeTwoHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeTwoHopAssertedHeadDef",
            spelling: "MiddleFourEdgeModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeTwoHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeTwoHopAssertedHeadDef",
            spelling: "OuterFourEdgeModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeTwoHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeTwoHopAssertedHeadDef",
            spelling: "TooDeepFourEdgeModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeTwoHopAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "OuterFourEdgeModeTwoHopAssertedHead",
        asserted_spelling: "MiddleFourEdgeModeTwoHopAssertedHead",
    },
    subject_result_role: "four-edge-local-mode-two-hop-asserted-head-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("TooDeepFourEdgeObjectModeTwoHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeTwoHopAssertedHeadDef",
            spelling: "BaseFourEdgeObjectModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeTwoHopAssertedHeadDef",
            spelling: "InnerFourEdgeObjectModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "BaseFourEdgeObjectModeTwoHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeTwoHopAssertedHeadDef",
            spelling: "MiddleFourEdgeObjectModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "InnerFourEdgeObjectModeTwoHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeTwoHopAssertedHeadDef",
            spelling: "OuterFourEdgeObjectModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleFourEdgeObjectModeTwoHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeTwoHopAssertedHeadDef",
            spelling: "TooDeepFourEdgeObjectModeTwoHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "OuterFourEdgeObjectModeTwoHopAssertedHead",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "OuterFourEdgeObjectModeTwoHopAssertedHead",
        asserted_spelling: "MiddleFourEdgeObjectModeTwoHopAssertedHead",
    },
    subject_result_role: "four-edge-local-object-mode-two-hop-asserted-head-subject-result",
};

const SOURCE_THREE_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalModeThreeHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("OuterThreeEdgeModeThreeHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeThreeHopAssertedHeadDef",
            spelling: "BaseThreeEdgeModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeThreeHopAssertedHeadDef",
            spelling: "InnerThreeEdgeModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeThreeHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeThreeHopAssertedHeadDef",
            spelling: "MiddleThreeEdgeModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeThreeHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeThreeHopAssertedHeadDef",
            spelling: "OuterThreeEdgeModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeThreeHopAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingThreeHopRadix {
        first_intermediate_spelling: "MiddleThreeEdgeModeThreeHopAssertedHead",
        second_intermediate_spelling: "InnerThreeEdgeModeThreeHopAssertedHead",
        asserted_spelling: "BaseThreeEdgeModeThreeHopAssertedHead",
    },
    subject_result_role: "three-edge-local-mode-three-hop-asserted-head-subject-result",
};

const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterThreeEdgeObjectModeThreeHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeThreeHopAssertedHeadDef",
            spelling: "BaseThreeEdgeObjectModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeThreeHopAssertedHeadDef",
            spelling: "InnerThreeEdgeObjectModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "BaseThreeEdgeObjectModeThreeHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeThreeHopAssertedHeadDef",
            spelling: "MiddleThreeEdgeObjectModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "InnerThreeEdgeObjectModeThreeHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeThreeHopAssertedHeadDef",
            spelling: "OuterThreeEdgeObjectModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleThreeEdgeObjectModeThreeHopAssertedHead",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingThreeHopRadix {
        first_intermediate_spelling: "MiddleThreeEdgeObjectModeThreeHopAssertedHead",
        second_intermediate_spelling: "InnerThreeEdgeObjectModeThreeHopAssertedHead",
        asserted_spelling: "BaseThreeEdgeObjectModeThreeHopAssertedHead",
    },
    subject_result_role: "three-edge-local-object-mode-three-hop-asserted-head-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalModeThreeHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("TooDeepFourEdgeModeThreeHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeThreeHopAssertedHeadDef",
            spelling: "BaseFourEdgeModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeThreeHopAssertedHeadDef",
            spelling: "InnerFourEdgeModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeThreeHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeThreeHopAssertedHeadDef",
            spelling: "MiddleFourEdgeModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeThreeHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeThreeHopAssertedHeadDef",
            spelling: "OuterFourEdgeModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeThreeHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeThreeHopAssertedHeadDef",
            spelling: "TooDeepFourEdgeModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeThreeHopAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingThreeHopRadix {
        first_intermediate_spelling: "OuterFourEdgeModeThreeHopAssertedHead",
        second_intermediate_spelling: "MiddleFourEdgeModeThreeHopAssertedHead",
        asserted_spelling: "InnerFourEdgeModeThreeHopAssertedHead",
    },
    subject_result_role: "four-edge-local-mode-three-hop-asserted-head-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("TooDeepFourEdgeObjectModeThreeHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeThreeHopAssertedHeadDef",
            spelling: "BaseFourEdgeObjectModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeThreeHopAssertedHeadDef",
            spelling: "InnerFourEdgeObjectModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "BaseFourEdgeObjectModeThreeHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeThreeHopAssertedHeadDef",
            spelling: "MiddleFourEdgeObjectModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "InnerFourEdgeObjectModeThreeHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeThreeHopAssertedHeadDef",
            spelling: "OuterFourEdgeObjectModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleFourEdgeObjectModeThreeHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeThreeHopAssertedHeadDef",
            spelling: "TooDeepFourEdgeObjectModeThreeHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "OuterFourEdgeObjectModeThreeHopAssertedHead",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingThreeHopRadix {
        first_intermediate_spelling: "OuterFourEdgeObjectModeThreeHopAssertedHead",
        second_intermediate_spelling: "MiddleFourEdgeObjectModeThreeHopAssertedHead",
        asserted_spelling: "InnerFourEdgeObjectModeThreeHopAssertedHead",
    },
    subject_result_role: "four-edge-local-object-mode-three-hop-asserted-head-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_MODE_FOUR_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalModeFourHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("TooDeepFourEdgeModeFourHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeFourHopAssertedHeadDef",
            spelling: "BaseFourEdgeModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeFourHopAssertedHeadDef",
            spelling: "InnerFourEdgeModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeFourHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeFourHopAssertedHeadDef",
            spelling: "MiddleFourEdgeModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeFourHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeFourHopAssertedHeadDef",
            spelling: "OuterFourEdgeModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeFourHopAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeFourHopAssertedHeadDef",
            spelling: "TooDeepFourEdgeModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeFourHopAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFourHopRadix {
        first_intermediate_spelling: "OuterFourEdgeModeFourHopAssertedHead",
        second_intermediate_spelling: "MiddleFourEdgeModeFourHopAssertedHead",
        third_intermediate_spelling: "InnerFourEdgeModeFourHopAssertedHead",
        asserted_spelling: "BaseFourEdgeModeFourHopAssertedHead",
    },
    subject_result_role: "four-edge-local-mode-four-hop-asserted-head-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_FOUR_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalObjectModeFourHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("TooDeepFourEdgeObjectModeFourHopAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeFourHopAssertedHeadDef",
            spelling: "BaseFourEdgeObjectModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeFourHopAssertedHeadDef",
            spelling: "InnerFourEdgeObjectModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "BaseFourEdgeObjectModeFourHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeFourHopAssertedHeadDef",
            spelling: "MiddleFourEdgeObjectModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "InnerFourEdgeObjectModeFourHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeFourHopAssertedHeadDef",
            spelling: "OuterFourEdgeObjectModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleFourEdgeObjectModeFourHopAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeFourHopAssertedHeadDef",
            spelling: "TooDeepFourEdgeObjectModeFourHopAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "OuterFourEdgeObjectModeFourHopAssertedHead",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFourHopRadix {
        first_intermediate_spelling: "OuterFourEdgeObjectModeFourHopAssertedHead",
        second_intermediate_spelling: "MiddleFourEdgeObjectModeFourHopAssertedHead",
        third_intermediate_spelling: "InnerFourEdgeObjectModeFourHopAssertedHead",
        asserted_spelling: "BaseFourEdgeObjectModeFourHopAssertedHead",
    },
    subject_result_role: "four-edge-local-object-mode-four-hop-asserted-head-subject-result",
};

const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "TwoEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterTwoEdgeObjectModeRadixAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeRadixAssertedHeadDef",
            spelling: "BaseTwoEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeRadixAssertedHeadDef",
            spelling: "MiddleTwoEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeRadixAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeRadixAssertedHeadDef",
            spelling: "OuterTwoEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleTwoEdgeObjectModeRadixAssertedHead",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "MiddleTwoEdgeObjectModeRadixAssertedHead",
    ),
    subject_result_role: "two-edge-local-object-mode-radix-asserted-head-subject-result",
};

const SOURCE_THREE_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("OuterThreeEdgeModeRadixAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeRadixAssertedHeadDef",
            spelling: "BaseThreeEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeRadixAssertedHeadDef",
            spelling: "InnerThreeEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeRadixAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeRadixAssertedHeadDef",
            spelling: "MiddleThreeEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeRadixAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeRadixAssertedHeadDef",
            spelling: "OuterThreeEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeRadixAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "MiddleThreeEdgeModeRadixAssertedHead",
    ),
    subject_result_role: "three-edge-local-mode-radix-asserted-head-subject-result",
};

const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterThreeEdgeObjectModeRadixAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeRadixAssertedHeadDef",
            spelling: "BaseThreeEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeRadixAssertedHeadDef",
            spelling: "InnerThreeEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "BaseThreeEdgeObjectModeRadixAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeRadixAssertedHeadDef",
            spelling: "MiddleThreeEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "InnerThreeEdgeObjectModeRadixAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeRadixAssertedHeadDef",
            spelling: "OuterThreeEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleThreeEdgeObjectModeRadixAssertedHead",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "MiddleThreeEdgeObjectModeRadixAssertedHead",
    ),
    subject_result_role: "three-edge-local-object-mode-radix-asserted-head-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("TooDeepFourEdgeModeRadixAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeRadixAssertedHeadDef",
            spelling: "BaseFourEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeRadixAssertedHeadDef",
            spelling: "InnerFourEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeRadixAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeRadixAssertedHeadDef",
            spelling: "MiddleFourEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeRadixAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeRadixAssertedHeadDef",
            spelling: "OuterFourEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeRadixAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeRadixAssertedHeadDef",
            spelling: "TooDeepFourEdgeModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeRadixAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "OuterFourEdgeModeRadixAssertedHead",
    ),
    subject_result_role: "four-edge-local-mode-radix-asserted-head-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("TooDeepFourEdgeObjectModeRadixAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeRadixAssertedHeadDef",
            spelling: "BaseFourEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeRadixAssertedHeadDef",
            spelling: "InnerFourEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeRadixAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeRadixAssertedHeadDef",
            spelling: "MiddleFourEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "InnerFourEdgeObjectModeRadixAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeRadixAssertedHeadDef",
            spelling: "OuterFourEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleFourEdgeObjectModeRadixAssertedHead",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeRadixAssertedHeadDef",
            spelling: "TooDeepFourEdgeObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode(
                "OuterFourEdgeObjectModeRadixAssertedHead",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "OuterFourEdgeObjectModeRadixAssertedHead",
    ),
    subject_result_role: "four-edge-local-object-mode-radix-asserted-head-subject-result",
};

const SOURCE_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ChainedLocalObjectModeAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectModeAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeAssertedHeadDef",
            spelling: "BaseObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeAssertedHeadDef",
            spelling: "ChainObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
        "ChainObjectModeAssertedHead",
    ),
    subject_result_role: "chained-local-object-mode-asserted-head-subject-result",
};

const SOURCE_TWO_EDGE_LOCAL_MODE_ASSERTED_HEAD_CONFIG: SourceReservedVariableTypeAssertionConfig =
    SourceReservedVariableTypeAssertionConfig {
        label: "TwoEdgeLocalModeAssertedHeadPayloadBoundary",
        invalid_payload_key: TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        binding_spelling: "x",
        binding_type: SourceReservedVariableBuiltinType::Set,
        binding_source_mode_spelling: Some("OuterTwoEdgeModeAssertedHead"),
        mode_definitions: &[
            SourceReservedVariableModeDefinition {
                label: "BaseTwoEdgeModeAssertedHeadDef",
                spelling: "BaseTwoEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Builtin(
                    SourceReservedVariableBuiltinType::Set,
                ),
            },
            SourceReservedVariableModeDefinition {
                label: "MiddleTwoEdgeModeAssertedHeadDef",
                spelling: "MiddleTwoEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeAssertedHead"),
            },
            SourceReservedVariableModeDefinition {
                label: "OuterTwoEdgeModeAssertedHeadDef",
                spelling: "OuterTwoEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeAssertedHead"),
            },
        ],
        asserted_type: SourceReservedVariableBuiltinType::Set,
        asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
            "OuterTwoEdgeModeAssertedHead",
        ),
        subject_result_role: "two-edge-local-mode-asserted-head-subject-result",
    };

const SOURCE_THREE_EDGE_LOCAL_MODE_ASSERTED_HEAD_CONFIG: SourceReservedVariableTypeAssertionConfig =
    SourceReservedVariableTypeAssertionConfig {
        label: "ThreeEdgeLocalModeAssertedHeadPayloadBoundary",
        invalid_payload_key:
            TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        binding_spelling: "x",
        binding_type: SourceReservedVariableBuiltinType::Set,
        binding_source_mode_spelling: Some("OuterThreeEdgeModeAssertedHead"),
        mode_definitions: &[
            SourceReservedVariableModeDefinition {
                label: "BaseThreeEdgeModeAssertedHeadDef",
                spelling: "BaseThreeEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Builtin(
                    SourceReservedVariableBuiltinType::Set,
                ),
            },
            SourceReservedVariableModeDefinition {
                label: "InnerThreeEdgeModeAssertedHeadDef",
                spelling: "InnerThreeEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeAssertedHead"),
            },
            SourceReservedVariableModeDefinition {
                label: "MiddleThreeEdgeModeAssertedHeadDef",
                spelling: "MiddleThreeEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeAssertedHead"),
            },
            SourceReservedVariableModeDefinition {
                label: "OuterThreeEdgeModeAssertedHeadDef",
                spelling: "OuterThreeEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeAssertedHead"),
            },
        ],
        asserted_type: SourceReservedVariableBuiltinType::Set,
        asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
            "OuterThreeEdgeModeAssertedHead",
        ),
        subject_result_role: "three-edge-local-mode-asserted-head-subject-result",
    };

const SOURCE_FOUR_EDGE_LOCAL_MODE_ASSERTED_HEAD_CONFIG: SourceReservedVariableTypeAssertionConfig =
    SourceReservedVariableTypeAssertionConfig {
        label: "FourEdgeLocalModeAssertedHeadPayloadBoundary",
        invalid_payload_key:
            TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        binding_spelling: "x",
        binding_type: SourceReservedVariableBuiltinType::Set,
        binding_source_mode_spelling: Some("TooDeepFourEdgeModeAssertedHead"),
        mode_definitions: &[
            SourceReservedVariableModeDefinition {
                label: "BaseFourEdgeModeAssertedHeadDef",
                spelling: "BaseFourEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Builtin(
                    SourceReservedVariableBuiltinType::Set,
                ),
            },
            SourceReservedVariableModeDefinition {
                label: "InnerFourEdgeModeAssertedHeadDef",
                spelling: "InnerFourEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeAssertedHead"),
            },
            SourceReservedVariableModeDefinition {
                label: "MiddleFourEdgeModeAssertedHeadDef",
                spelling: "MiddleFourEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeAssertedHead"),
            },
            SourceReservedVariableModeDefinition {
                label: "OuterFourEdgeModeAssertedHeadDef",
                spelling: "OuterFourEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeAssertedHead"),
            },
            SourceReservedVariableModeDefinition {
                label: "TooDeepFourEdgeModeAssertedHeadDef",
                spelling: "TooDeepFourEdgeModeAssertedHead",
                radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeAssertedHead"),
            },
        ],
        asserted_type: SourceReservedVariableBuiltinType::Set,
        asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
            "TooDeepFourEdgeModeAssertedHead",
        ),
        subject_result_role: "four-edge-local-mode-asserted-head-subject-result",
    };

const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalObjectModeAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("TooDeepFourEdgeObjectModeAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeAssertedHeadDef",
            spelling: "BaseFourEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeAssertedHeadDef",
            spelling: "InnerFourEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeAssertedHeadDef",
            spelling: "MiddleFourEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeObjectModeAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeAssertedHeadDef",
            spelling: "OuterFourEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeObjectModeAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeAssertedHeadDef",
            spelling: "TooDeepFourEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeObjectModeAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
        "TooDeepFourEdgeObjectModeAssertedHead",
    ),
    subject_result_role: "four-edge-local-object-mode-asserted-head-subject-result",
};

const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterThreeEdgeObjectModeAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeAssertedHeadDef",
            spelling: "BaseThreeEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeAssertedHeadDef",
            spelling: "InnerThreeEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeObjectModeAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeAssertedHeadDef",
            spelling: "MiddleThreeEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeObjectModeAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeAssertedHeadDef",
            spelling: "OuterThreeEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeObjectModeAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
        "OuterThreeEdgeObjectModeAssertedHead",
    ),
    subject_result_role: "three-edge-local-object-mode-asserted-head-subject-result",
};

const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterTwoEdgeObjectModeAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeAssertedHeadDef",
            spelling: "BaseTwoEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeAssertedHeadDef",
            spelling: "MiddleTwoEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeAssertedHead"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeAssertedHeadDef",
            spelling: "OuterTwoEdgeObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeObjectModeAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
        "OuterTwoEdgeObjectModeAssertedHead",
    ),
    subject_result_role: "two-edge-local-object-mode-asserted-head-subject-result",
};

const SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ChainedLocalModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseModeTypeAssertionDef",
            spelling: "BaseModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainModeTypeAssertionDef",
            spelling: "ChainModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "chained-local-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ChainedLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeTypeAssertionDef",
            spelling: "BaseObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeTypeAssertionDef",
            spelling: "ChainObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "chained-local-object-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "TwoEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("OuterTwoEdgeModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeTypeAssertionDef",
            spelling: "BaseTwoEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeModeTypeAssertionDef",
            spelling: "MiddleTwoEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeModeTypeAssertionDef",
            spelling: "OuterTwoEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "two-edge-local-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "TwoEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterTwoEdgeObjectModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeTypeAssertionDef",
            spelling: "BaseTwoEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeTypeAssertionDef",
            spelling: "MiddleTwoEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeTypeAssertionDef",
            spelling: "OuterTwoEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeObjectModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "two-edge-local-object-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("OuterThreeEdgeModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeTypeAssertionDef",
            spelling: "BaseThreeEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeTypeAssertionDef",
            spelling: "InnerThreeEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeTypeAssertionDef",
            spelling: "MiddleThreeEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeTypeAssertionDef",
            spelling: "OuterThreeEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "three-edge-local-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterThreeEdgeObjectModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeTypeAssertionDef",
            spelling: "BaseThreeEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeTypeAssertionDef",
            spelling: "InnerThreeEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode(
                "BaseThreeEdgeObjectModeTypeAssertion",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeTypeAssertionDef",
            spelling: "MiddleThreeEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode(
                "InnerThreeEdgeObjectModeTypeAssertion",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeTypeAssertionDef",
            spelling: "OuterThreeEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleThreeEdgeObjectModeTypeAssertion",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "three-edge-local-object-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("TooDeepFourEdgeModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeTypeAssertionDef",
            spelling: "BaseFourEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeTypeAssertionDef",
            spelling: "InnerFourEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeTypeAssertionDef",
            spelling: "MiddleFourEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeTypeAssertionDef",
            spelling: "OuterFourEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeTypeAssertionDef",
            spelling: "TooDeepFourEdgeModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "four-edge-local-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("TooDeepFourEdgeObjectModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeTypeAssertionDef",
            spelling: "BaseFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeTypeAssertionDef",
            spelling: "InnerFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeTypeAssertionDef",
            spelling: "MiddleFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeObjectModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeTypeAssertionDef",
            spelling: "OuterFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeObjectModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeTypeAssertionDef",
            spelling: "TooDeepFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeObjectModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "four-edge-local-object-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("LocalObjectModeTypeAssertion"),
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalObjectModeTypeAssertionDef",
        spelling: "LocalObjectModeTypeAssertion",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    }],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "local-object-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_RESERVED_VARIABLE_EQUALITY_CONFIG: SourceReservedVariableBinaryFormulaConfig =
    SourceReservedVariableBinaryFormulaConfig {
        label: "ReservedVariableEqualityPayloadBoundary",
        operator: "=",
        formula_kind: FormulaKind::Equality,
        invalid_payload_key: TYPE_ELABORATION_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        reserve_item_count: 1,
        binding_spellings: &["x"],
        binding_types: &[SourceReservedVariableBuiltinType::Set],
        binding_source_mode_spellings: &[None],
        mode_definitions: &[],
        left_binding_index: 0,
        right_binding_index: 0,
        require_shared_type_range: false,
        require_distinct_type_ranges: false,
        left_result_role: "reserved-variable-left-result",
        right_result_role: "reserved-variable-right-result",
        left_expected_role: Some("reserved-variable-left-expected"),
        right_expected_role: Some("reserved-variable-right-expected"),
    };

static SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-variable-left-result",
    right_result_role: "parenthesized-reserved-variable-right-result",
    left_expected_role: Some("parenthesized-reserved-variable-left-expected"),
    right_expected_role: Some("parenthesized-reserved-variable-right-expected"),
};

static SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-variable-inequality-left-result",
    right_result_role: "parenthesized-reserved-variable-inequality-right-result",
    left_expected_role: Some("parenthesized-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("parenthesized-reserved-variable-inequality-right-expected"),
};

static SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-variable-membership-left-result",
    right_result_role: "parenthesized-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("parenthesized-reserved-variable-membership-right-expected"),
};

static SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedHeterogeneousReserveMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "parenthesized-heterogeneous-reserve-membership-left-result",
    right_result_role: "parenthesized-heterogeneous-reserve-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("parenthesized-heterogeneous-reserve-membership-right-expected"),
};

static SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "RightParenthesizedReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "right-parenthesized-reserved-variable-membership-left-result",
    right_result_role: "right-parenthesized-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("right-parenthesized-reserved-variable-membership-right-expected"),
};

static SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedTwoEdgeLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeEqualityDef",
            spelling: "BaseTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Set,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeModeEqualityDef",
            spelling: "MiddleTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeModeEqualityDef",
            spelling: "OuterTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-two-edge-local-mode-reserved-variable-equality-left-result",
    right_result_role: "parenthesized-two-edge-local-mode-reserved-variable-equality-right-result",
    left_expected_role: Some(
        "parenthesized-two-edge-local-mode-reserved-variable-equality-left-expected",
    ),
    right_expected_role: Some(
        "parenthesized-two-edge-local-mode-reserved-variable-equality-right-expected",
    ),
};

static SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedObjectVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-object-variable-left-result",
    right_result_role: "parenthesized-reserved-object-variable-right-result",
    left_expected_role: Some("parenthesized-reserved-object-variable-left-expected"),
    right_expected_role: Some("parenthesized-reserved-object-variable-right-expected"),
};

static SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedObjectVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-object-variable-inequality-left-result",
    right_result_role: "parenthesized-reserved-object-variable-inequality-right-result",
    left_expected_role: Some("parenthesized-reserved-object-variable-inequality-left-expected"),
    right_expected_role: Some("parenthesized-reserved-object-variable-inequality-right-expected"),
};

const SOURCE_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG: SourceReservedVariableBinaryFormulaConfig =
    SourceReservedVariableBinaryFormulaConfig {
        label: "ReservedObjectVariableEqualityPayloadBoundary",
        operator: "=",
        formula_kind: FormulaKind::Equality,
        invalid_payload_key: TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        reserve_item_count: 1,
        binding_spellings: &["x"],
        binding_types: &[SourceReservedVariableBuiltinType::Object],
        binding_source_mode_spellings: &[None],
        mode_definitions: &[],
        left_binding_index: 0,
        right_binding_index: 0,
        require_shared_type_range: false,
        require_distinct_type_ranges: false,
        left_result_role: "reserved-object-variable-left-result",
        right_result_role: "reserved-object-variable-right-result",
        left_expected_role: Some("reserved-object-variable-left-expected"),
        right_expected_role: Some("reserved-object-variable-right-expected"),
    };

const SOURCE_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "DistinctReservedObjectVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Object,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: true,
    require_distinct_type_ranges: false,
    left_result_role: "distinct-reserved-object-variable-left-result",
    right_result_role: "distinct-reserved-object-variable-right-result",
    left_expected_role: Some("distinct-reserved-object-variable-left-expected"),
    right_expected_role: Some("distinct-reserved-object-variable-right-expected"),
};

const SOURCE_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "DistinctReservedObjectVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Object,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: true,
    require_distinct_type_ranges: false,
    left_result_role: "distinct-reserved-object-variable-inequality-left-result",
    right_result_role: "distinct-reserved-object-variable-inequality-right-result",
    left_expected_role: Some("distinct-reserved-object-variable-inequality-left-expected"),
    right_expected_role: Some("distinct-reserved-object-variable-inequality-right-expected"),
};

const SOURCE_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG: SourceReservedVariableBinaryFormulaConfig =
    SourceReservedVariableBinaryFormulaConfig {
        label: "ReservedObjectVariableInequalityPayloadBoundary",
        operator: "<>",
        formula_kind: FormulaKind::Inequality,
        invalid_payload_key:
            TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        reserve_item_count: 1,
        binding_spellings: &["x"],
        binding_types: &[SourceReservedVariableBuiltinType::Object],
        binding_source_mode_spellings: &[None],
        mode_definitions: &[],
        left_binding_index: 0,
        right_binding_index: 0,
        require_shared_type_range: false,
        require_distinct_type_ranges: false,
        left_result_role: "reserved-object-variable-inequality-left-result",
        right_result_role: "reserved-object-variable-inequality-right-result",
        left_expected_role: Some("reserved-object-variable-inequality-left-expected"),
        right_expected_role: Some("reserved-object-variable-inequality-right-expected"),
    };

const SOURCE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG: SourceReservedVariableBinaryFormulaConfig =
    SourceReservedVariableBinaryFormulaConfig {
        label: "ReservedVariableMembershipPayloadBoundary",
        operator: "in",
        formula_kind: FormulaKind::Membership,
        invalid_payload_key: TYPE_ELABORATION_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        reserve_item_count: 1,
        binding_spellings: &["x"],
        binding_types: &[SourceReservedVariableBuiltinType::Set],
        binding_source_mode_spellings: &[None],
        mode_definitions: &[],
        left_binding_index: 0,
        right_binding_index: 0,
        require_shared_type_range: false,
        require_distinct_type_ranges: false,
        left_result_role: "reserved-variable-membership-left-result",
        right_result_role: "reserved-variable-membership-right-result",
        left_expected_role: None,
        right_expected_role: Some("reserved-variable-membership-right-expected"),
    };

const SOURCE_RESERVED_VARIABLE_INEQUALITY_CONFIG: SourceReservedVariableBinaryFormulaConfig =
    SourceReservedVariableBinaryFormulaConfig {
        label: "ReservedVariableInequalityPayloadBoundary",
        operator: "<>",
        formula_kind: FormulaKind::Inequality,
        invalid_payload_key: TYPE_ELABORATION_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        reserve_item_count: 1,
        binding_spellings: &["x"],
        binding_types: &[SourceReservedVariableBuiltinType::Set],
        binding_source_mode_spellings: &[None],
        mode_definitions: &[],
        left_binding_index: 0,
        right_binding_index: 0,
        require_shared_type_range: false,
        require_distinct_type_ranges: false,
        left_result_role: "reserved-variable-inequality-left-result",
        right_result_role: "reserved-variable-inequality-right-result",
        left_expected_role: Some("reserved-variable-inequality-left-expected"),
        right_expected_role: Some("reserved-variable-inequality-right-expected"),
    };

const SOURCE_DISTINCT_RESERVED_VARIABLE_EQUALITY_CONFIG: SourceReservedVariableBinaryFormulaConfig =
    SourceReservedVariableBinaryFormulaConfig {
        label: "DistinctReservedVariableEqualityPayloadBoundary",
        operator: "=",
        formula_kind: FormulaKind::Equality,
        invalid_payload_key:
            TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        reserve_item_count: 1,
        binding_spellings: &["x", "y"],
        binding_types: &[
            SourceReservedVariableBuiltinType::Set,
            SourceReservedVariableBuiltinType::Set,
        ],
        binding_source_mode_spellings: &[None, None],
        mode_definitions: &[],
        left_binding_index: 0,
        right_binding_index: 1,
        require_shared_type_range: true,
        require_distinct_type_ranges: false,
        left_result_role: "distinct-reserved-variable-left-result",
        right_result_role: "distinct-reserved-variable-right-result",
        left_expected_role: Some("distinct-reserved-variable-left-expected"),
        right_expected_role: Some("distinct-reserved-variable-right-expected"),
    };

const SOURCE_DISTINCT_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "DistinctReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key: TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: true,
    require_distinct_type_ranges: false,
    left_result_role: "distinct-reserved-variable-inequality-left-result",
    right_result_role: "distinct-reserved-variable-inequality-right-result",
    left_expected_role: Some("distinct-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("distinct-reserved-variable-inequality-right-expected"),
};

const SOURCE_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "DistinctReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key: TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: true,
    require_distinct_type_ranges: false,
    left_result_role: "distinct-reserved-variable-membership-left-result",
    right_result_role: "distinct-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("distinct-reserved-variable-membership-right-expected"),
};

const SOURCE_MULTIPLE_RESERVE_DECLARATION_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleReserveDeclarationEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key: TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-reserve-declaration-left-result",
    right_result_role: "multiple-reserve-declaration-right-result",
    left_expected_role: Some("multiple-reserve-declaration-left-expected"),
    right_expected_role: Some("multiple-reserve-declaration-right-expected"),
};

const SOURCE_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleObjectReserveDeclarationEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Object,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-object-reserve-declaration-left-result",
    right_result_role: "multiple-object-reserve-declaration-right-result",
    left_expected_role: Some("multiple-object-reserve-declaration-left-expected"),
    right_expected_role: Some("multiple-object-reserve-declaration-right-expected"),
};

const SOURCE_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleObjectReserveDeclarationInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Object,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-object-reserve-declaration-inequality-left-result",
    right_result_role: "multiple-object-reserve-declaration-inequality-right-result",
    left_expected_role: Some("multiple-object-reserve-declaration-inequality-left-expected"),
    right_expected_role: Some("multiple-object-reserve-declaration-inequality-right-expected"),
};

const SOURCE_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleReserveDeclarationInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-reserve-declaration-inequality-left-result",
    right_result_role: "multiple-reserve-declaration-inequality-right-result",
    left_expected_role: Some("multiple-reserve-declaration-inequality-left-expected"),
    right_expected_role: Some("multiple-reserve-declaration-inequality-right-expected"),
};

const SOURCE_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleReserveDeclarationMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-reserve-declaration-membership-left-result",
    right_result_role: "multiple-reserve-declaration-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("multiple-reserve-declaration-membership-right-expected"),
};

const SOURCE_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG: SourceReservedVariableBinaryFormulaConfig =
    SourceReservedVariableBinaryFormulaConfig {
        label: "HeterogeneousReserveMembershipPayloadBoundary",
        operator: "in",
        formula_kind: FormulaKind::Membership,
        invalid_payload_key: TYPE_ELABORATION_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        reserve_item_count: 2,
        binding_spellings: &["x", "y"],
        binding_types: &[
            SourceReservedVariableBuiltinType::Object,
            SourceReservedVariableBuiltinType::Set,
        ],
        binding_source_mode_spellings: &[None, None],
        mode_definitions: &[],
        left_binding_index: 0,
        right_binding_index: 1,
        require_shared_type_range: false,
        require_distinct_type_ranges: true,
        left_result_role: "heterogeneous-reserve-membership-left-result",
        right_result_role: "heterogeneous-reserve-membership-right-result",
        left_expected_role: None,
        right_expected_role: Some("heterogeneous-reserve-membership-right-expected"),
    };

const SOURCE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("LocalModeMembership"), None],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalModeMembershipDef",
        spelling: "LocalModeMembership",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
    }],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "local-mode-reserved-variable-membership-left-result",
    right_result_role: "local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("local-mode-reserved-variable-membership-right-expected"),
};

const SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("ChainModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseModeMembershipDef",
            spelling: "BaseModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainModeMembershipDef",
            spelling: "ChainModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "chained-local-mode-reserved-variable-membership-left-result",
    right_result_role: "chained-local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("chained-local-mode-reserved-variable-membership-right-expected"),
};

const SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeMembershipDef",
            spelling: "BaseTwoEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeModeMembershipDef",
            spelling: "MiddleTwoEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeModeMembershipDef",
            spelling: "OuterTwoEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "two-edge-local-mode-reserved-variable-membership-left-result",
    right_result_role: "two-edge-local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("two-edge-local-mode-reserved-variable-membership-right-expected"),
};

const SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeMembershipDef",
            spelling: "BaseThreeEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeMembershipDef",
            spelling: "InnerThreeEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeMembershipDef",
            spelling: "MiddleThreeEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeMembershipDef",
            spelling: "OuterThreeEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "three-edge-local-mode-reserved-variable-membership-left-result",
    right_result_role: "three-edge-local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("three-edge-local-mode-reserved-variable-membership-right-expected"),
};

const SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeMembershipDef",
            spelling: "BaseFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeMembershipDef",
            spelling: "InnerFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeMembershipDef",
            spelling: "MiddleFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeMembershipDef",
            spelling: "OuterFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeMembershipDef",
            spelling: "TooDeepFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "four-edge-local-mode-reserved-variable-membership-left-result",
    right_result_role: "four-edge-local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("four-edge-local-mode-reserved-variable-membership-right-expected"),
};

const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeObjectModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeMembershipDef",
            spelling: "BaseFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeMembershipDef",
            spelling: "InnerFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeMembershipDef",
            spelling: "MiddleFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeMembershipDef",
            spelling: "OuterFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeMembershipDef",
            spelling: "TooDeepFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeObjectModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "four-edge-local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "four-edge-local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some(
        "four-edge-local-object-mode-reserved-variable-membership-right-expected",
    ),
};

const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeObjectModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeMembershipDef",
            spelling: "BaseThreeEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeMembershipDef",
            spelling: "InnerThreeEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeMembershipDef",
            spelling: "MiddleThreeEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeMembershipDef",
            spelling: "OuterThreeEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeObjectModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "three-edge-local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "three-edge-local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some(
        "three-edge-local-object-mode-reserved-variable-membership-right-expected",
    ),
};

const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeObjectModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeMembershipDef",
            spelling: "BaseTwoEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeMembershipDef",
            spelling: "MiddleTwoEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeMembershipDef",
            spelling: "OuterTwoEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeObjectModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "two-edge-local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "two-edge-local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some(
        "two-edge-local-object-mode-reserved-variable-membership-right-expected",
    ),
};

const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("ChainObjectModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeMembershipDef",
            spelling: "BaseObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeMembershipDef",
            spelling: "ChainObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "chained-local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "chained-local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some(
        "chained-local-object-mode-reserved-variable-membership-right-expected",
    ),
};

const SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("LocalObjectModeMembership"), None],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalObjectModeMembershipDef",
        spelling: "LocalObjectModeMembership",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    }],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("local-object-mode-reserved-variable-membership-right-expected"),
};

const SOURCE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key: TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("LocalModeFormula")],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalModeFormulaDef",
        spelling: "LocalModeFormula",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
    }],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "local-mode-reserved-variable-left-result",
    right_result_role: "local-mode-reserved-variable-right-result",
    left_expected_role: Some("local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("local-mode-reserved-variable-right-expected"),
};

const SOURCE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("LocalModeInequality")],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalModeInequalityDef",
        spelling: "LocalModeInequality",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
    }],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "local-mode-reserved-variable-inequality-left-result",
    right_result_role: "local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("local-mode-reserved-variable-inequality-right-expected"),
};

const SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("LocalObjectModeInequality")],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalObjectModeInequalityDef",
        spelling: "LocalObjectModeInequality",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    }],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("local-object-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("local-object-mode-reserved-variable-inequality-right-expected"),
};

const SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("ChainModeFormula")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseModeFormulaDef",
            spelling: "BaseModeFormula",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainModeFormulaDef",
            spelling: "ChainModeFormula",
            radix: SourceReservedVariableModeRadix::Mode("BaseModeFormula"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "chained-local-mode-reserved-variable-left-result",
    right_result_role: "chained-local-mode-reserved-variable-right-result",
    left_expected_role: Some("chained-local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("chained-local-mode-reserved-variable-right-expected"),
};

const SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeEqualityDef",
            spelling: "BaseTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeModeEqualityDef",
            spelling: "MiddleTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeModeEqualityDef",
            spelling: "OuterTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "two-edge-local-mode-reserved-variable-left-result",
    right_result_role: "two-edge-local-mode-reserved-variable-right-result",
    left_expected_role: Some("two-edge-local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("two-edge-local-mode-reserved-variable-right-expected"),
};

const SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeEqualityDef",
            spelling: "BaseThreeEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeEqualityDef",
            spelling: "InnerThreeEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeEqualityDef",
            spelling: "MiddleThreeEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeEqualityDef",
            spelling: "OuterThreeEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "three-edge-local-mode-reserved-variable-left-result",
    right_result_role: "three-edge-local-mode-reserved-variable-right-result",
    left_expected_role: Some("three-edge-local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("three-edge-local-mode-reserved-variable-right-expected"),
};

const SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeEqualityDef",
            spelling: "BaseFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeEqualityDef",
            spelling: "InnerFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeEqualityDef",
            spelling: "MiddleFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeEqualityDef",
            spelling: "OuterFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeEqualityDef",
            spelling: "TooDeepFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "four-edge-local-mode-reserved-variable-left-result",
    right_result_role: "four-edge-local-mode-reserved-variable-right-result",
    left_expected_role: Some("four-edge-local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("four-edge-local-mode-reserved-variable-right-expected"),
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS: &[SourceReservedVariableModeDefinition] = &[
    SourceReservedVariableModeDefinition {
        label: "BaseModeDef",
        spelling: "BaseMode",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode1Def",
        spelling: "ChainMode1",
        radix: SourceReservedVariableModeRadix::Mode("BaseMode"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode2Def",
        spelling: "ChainMode2",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode1"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode3Def",
        spelling: "ChainMode3",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode2"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode4Def",
        spelling: "ChainMode4",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode3"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode5Def",
        spelling: "ChainMode5",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode4"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainMode6Def",
        spelling: "ChainMode6",
        radix: SourceReservedVariableModeRadix::Mode("ChainMode5"),
    },
];

const SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LongLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("ChainMode6")],
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "long-local-mode-reserved-variable-left-result",
    right_result_role: "long-local-mode-reserved-variable-right-result",
    left_expected_role: Some("long-local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("long-local-mode-reserved-variable-right-expected"),
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS: &[SourceReservedVariableModeDefinition] = &[
    SourceReservedVariableModeDefinition {
        label: "BaseObjectModeDef",
        spelling: "BaseObjectMode",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode1Def",
        spelling: "ChainObjectMode1",
        radix: SourceReservedVariableModeRadix::Mode("BaseObjectMode"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode2Def",
        spelling: "ChainObjectMode2",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode1"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode3Def",
        spelling: "ChainObjectMode3",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode2"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode4Def",
        spelling: "ChainObjectMode4",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode3"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode5Def",
        spelling: "ChainObjectMode5",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode4"),
    },
    SourceReservedVariableModeDefinition {
        label: "ChainObjectMode6Def",
        spelling: "ChainObjectMode6",
        radix: SourceReservedVariableModeRadix::Mode("ChainObjectMode5"),
    },
];

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LongLocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("ChainObjectMode6")],
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "long-local-object-mode-reserved-variable-left-result",
    right_result_role: "long-local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("long-local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("long-local-object-mode-reserved-variable-right-expected"),
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LongLocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("ChainObjectMode6")],
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "long-local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "long-local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some(
        "long-local-object-mode-reserved-variable-inequality-left-expected",
    ),
    right_expected_role: Some(
        "long-local-object-mode-reserved-variable-inequality-right-expected",
    ),
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LongLocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("ChainObjectMode6"), None],
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "long-local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "long-local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some(
        "long-local-object-mode-reserved-variable-membership-right-expected",
    ),
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LongLocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("ChainMode6")],
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "long-local-mode-reserved-variable-inequality-left-result",
    right_result_role: "long-local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("long-local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("long-local-mode-reserved-variable-inequality-right-expected"),
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LongLocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("ChainMode6"), None],
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "long-local-mode-reserved-variable-membership-left-result",
    right_result_role: "long-local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("long-local-mode-reserved-variable-membership-right-expected"),
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "long-local-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_CONFIG: SourceReservedVariableTypeAssertionConfig =
    SourceReservedVariableTypeAssertionConfig {
        label: "LongLocalModeAssertedHeadPayloadBoundary",
        invalid_payload_key:
            TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        binding_spelling: "x",
        binding_type: SourceReservedVariableBuiltinType::Set,
        binding_source_mode_spelling: Some("ChainMode6"),
        mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
        asserted_type: SourceReservedVariableBuiltinType::Set,
        asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode("ChainMode6"),
        subject_result_role: "long-local-mode-asserted-head-subject-result",
    };

const SOURCE_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "ChainMode5",
    ),
    subject_result_role: "long-local-mode-radix-asserted-head-subject-result",
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "ChainMode5",
        asserted_spelling: "ChainMode4",
    },
    subject_result_role: "long-local-mode-two-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeThreeHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingThreeHopRadix {
        first_intermediate_spelling: "ChainMode5",
        second_intermediate_spelling: "ChainMode4",
        asserted_spelling: "ChainMode3",
    },
    subject_result_role: "long-local-mode-three-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeFourHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFourHopRadix {
        first_intermediate_spelling: "ChainMode5",
        second_intermediate_spelling: "ChainMode4",
        third_intermediate_spelling: "ChainMode3",
        asserted_spelling: "ChainMode2",
    },
    subject_result_role: "long-local-mode-four-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeFiveHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFiveHopRadix {
        first_intermediate_spelling: "ChainMode5",
        second_intermediate_spelling: "ChainMode4",
        third_intermediate_spelling: "ChainMode3",
        fourth_intermediate_spelling: "ChainMode2",
        asserted_spelling: "ChainMode1",
    },
    subject_result_role: "long-local-mode-five-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeSixHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingSixHopRadix {
        first_intermediate_spelling: "ChainMode5",
        second_intermediate_spelling: "ChainMode4",
        third_intermediate_spelling: "ChainMode3",
        fourth_intermediate_spelling: "ChainMode2",
        fifth_intermediate_spelling: "ChainMode1",
        asserted_spelling: "BaseMode",
    },
    subject_result_role: "long-local-mode-six-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeSixHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingSixHopRadix {
        first_intermediate_spelling: "ChainObjectMode5",
        second_intermediate_spelling: "ChainObjectMode4",
        third_intermediate_spelling: "ChainObjectMode3",
        fourth_intermediate_spelling: "ChainObjectMode2",
        fifth_intermediate_spelling: "ChainObjectMode1",
        asserted_spelling: "BaseObjectMode",
    },
    subject_result_role: "long-local-object-mode-six-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeFiveHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFiveHopRadix {
        first_intermediate_spelling: "ChainObjectMode5",
        second_intermediate_spelling: "ChainObjectMode4",
        third_intermediate_spelling: "ChainObjectMode3",
        fourth_intermediate_spelling: "ChainObjectMode2",
        asserted_spelling: "ChainObjectMode1",
    },
    subject_result_role: "long-local-object-mode-five-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "ChainObjectMode5",
    ),
    subject_result_role: "long-local-object-mode-radix-asserted-head-subject-result",
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "ChainObjectMode5",
        asserted_spelling: "ChainObjectMode4",
    },
    subject_result_role: "long-local-object-mode-two-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingThreeHopRadix {
        first_intermediate_spelling: "ChainObjectMode5",
        second_intermediate_spelling: "ChainObjectMode4",
        asserted_spelling: "ChainObjectMode3",
    },
    subject_result_role: "long-local-object-mode-three-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeFourHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFourHopRadix {
        first_intermediate_spelling: "ChainObjectMode5",
        second_intermediate_spelling: "ChainObjectMode4",
        third_intermediate_spelling: "ChainObjectMode3",
        asserted_spelling: "ChainObjectMode2",
    },
    subject_result_role: "long-local-object-mode-four-hop-asserted-head-subject-result",
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
        "ChainObjectMode6",
    ),
    subject_result_role: "long-local-object-mode-asserted-head-subject-result",
};

const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "long-local-object-mode-reserved-variable-type-assertion-subject-result",
};

const SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeInequalityDef",
            spelling: "BaseFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeInequalityDef",
            spelling: "InnerFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeInequalityDef",
            spelling: "MiddleFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeInequalityDef",
            spelling: "OuterFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeInequalityDef",
            spelling: "TooDeepFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "four-edge-local-mode-reserved-variable-inequality-left-result",
    right_result_role: "four-edge-local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("four-edge-local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("four-edge-local-mode-reserved-variable-inequality-right-expected"),
};

const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeObjectModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeEqualityDef",
            spelling: "BaseFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeEqualityDef",
            spelling: "InnerFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeEqualityDef",
            spelling: "MiddleFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeEqualityDef",
            spelling: "OuterFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeEqualityDef",
            spelling: "TooDeepFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeObjectModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "four-edge-local-object-mode-reserved-variable-left-result",
    right_result_role: "four-edge-local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("four-edge-local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("four-edge-local-object-mode-reserved-variable-right-expected"),
};

const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeObjectModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeInequalityDef",
            spelling: "BaseFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeInequalityDef",
            spelling: "InnerFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeInequalityDef",
            spelling: "MiddleFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeInequalityDef",
            spelling: "OuterFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeInequalityDef",
            spelling: "TooDeepFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeObjectModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "four-edge-local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "four-edge-local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some(
        "four-edge-local-object-mode-reserved-variable-inequality-left-expected",
    ),
    right_expected_role: Some(
        "four-edge-local-object-mode-reserved-variable-inequality-right-expected",
    ),
};

const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeObjectModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeEqualityDef",
            spelling: "BaseThreeEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeEqualityDef",
            spelling: "InnerThreeEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeEqualityDef",
            spelling: "MiddleThreeEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeEqualityDef",
            spelling: "OuterThreeEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeObjectModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "three-edge-local-object-mode-reserved-variable-left-result",
    right_result_role: "three-edge-local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("three-edge-local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("three-edge-local-object-mode-reserved-variable-right-expected"),
};

const SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeInequalityDef",
            spelling: "BaseThreeEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeInequalityDef",
            spelling: "InnerThreeEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeInequalityDef",
            spelling: "MiddleThreeEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeInequalityDef",
            spelling: "OuterThreeEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "three-edge-local-mode-reserved-variable-inequality-left-result",
    right_result_role: "three-edge-local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("three-edge-local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("three-edge-local-mode-reserved-variable-inequality-right-expected"),
};

const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeObjectModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeInequalityDef",
            spelling: "BaseThreeEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeInequalityDef",
            spelling: "InnerThreeEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeInequalityDef",
            spelling: "MiddleThreeEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeInequalityDef",
            spelling: "OuterThreeEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeObjectModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "three-edge-local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "three-edge-local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some(
        "three-edge-local-object-mode-reserved-variable-inequality-left-expected",
    ),
    right_expected_role: Some(
        "three-edge-local-object-mode-reserved-variable-inequality-right-expected",
    ),
};

const SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeInequalityDef",
            spelling: "BaseTwoEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeModeInequalityDef",
            spelling: "MiddleTwoEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeModeInequalityDef",
            spelling: "OuterTwoEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "two-edge-local-mode-reserved-variable-inequality-left-result",
    right_result_role: "two-edge-local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("two-edge-local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("two-edge-local-mode-reserved-variable-inequality-right-expected"),
};

const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeObjectModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeInequalityDef",
            spelling: "BaseTwoEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeInequalityDef",
            spelling: "MiddleTwoEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeInequalityDef",
            spelling: "OuterTwoEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeObjectModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "two-edge-local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "two-edge-local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some(
        "two-edge-local-object-mode-reserved-variable-inequality-left-expected",
    ),
    right_expected_role: Some(
        "two-edge-local-object-mode-reserved-variable-inequality-right-expected",
    ),
};

const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeObjectModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeEqualityDef",
            spelling: "BaseTwoEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeEqualityDef",
            spelling: "MiddleTwoEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeEqualityDef",
            spelling: "OuterTwoEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeObjectModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "two-edge-local-object-mode-reserved-variable-left-result",
    right_result_role: "two-edge-local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("two-edge-local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("two-edge-local-object-mode-reserved-variable-right-expected"),
};

const SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("ChainModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseModeInequalityDef",
            spelling: "BaseModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainModeInequalityDef",
            spelling: "ChainModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "chained-local-mode-reserved-variable-inequality-left-result",
    right_result_role: "chained-local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("chained-local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("chained-local-mode-reserved-variable-inequality-right-expected"),
};

const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("ChainObjectMode")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeDef",
            spelling: "BaseObjectMode",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeDef",
            spelling: "ChainObjectMode",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectMode"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "chained-local-object-mode-reserved-variable-left-result",
    right_result_role: "chained-local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("chained-local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("chained-local-object-mode-reserved-variable-right-expected"),
};

const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("ChainObjectModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeInequalityDef",
            spelling: "BaseObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeInequalityDef",
            spelling: "ChainObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "chained-local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "chained-local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some(
        "chained-local-object-mode-reserved-variable-inequality-left-expected",
    ),
    right_expected_role: Some(
        "chained-local-object-mode-reserved-variable-inequality-right-expected",
    ),
};

const SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("LocalObjectMode")],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalObjectModeDef",
        spelling: "LocalObjectMode",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    }],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "local-object-mode-reserved-variable-left-result",
    right_result_role: "local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("local-object-mode-reserved-variable-right-expected"),
};

#[derive(Debug)]
struct SourceReservedVariableBinaryFormulaOutput {
    payload: SourceReservedVariableBinaryFormula,
    handoff: SourceReserveHandoff,
    left_binding: BindingId,
    right_binding: BindingId,
    left_result_input: TypeExpressionInput,
    right_result_input: TypeExpressionInput,
    left_expected_input: Option<TypeExpressionInput>,
    right_expected_input: Option<TypeExpressionInput>,
    term_formula: TermFormulaInferenceOutput,
}

#[derive(Debug)]
struct SourceParenthesizedReservedVariableBinaryFormulaOutput {
    source_wrapper_side: SourceParenthesizedOperandSide,
    source_wrapper_site: TypedSiteRef,
    source_wrapper_range: SourceRange,
    wrapper_side: SourceParenthesizedOperandSide,
    wrapper_site: TypedSiteRef,
    wrapper_range: SourceRange,
    formula: SourceReservedVariableBinaryFormulaOutput,
}

#[derive(Debug)]
struct SourceReservedVariableTypeAssertion {
    reserve: SourceReserveExtraction,
    config: &'static SourceReservedVariableTypeAssertionConfig,
    formula_site: TypedSiteRef,
    formula_range: SourceRange,
    subject_site: TypedSiteRef,
    subject_range: SourceRange,
    subject_spelling: String,
    subject_lookup_ordinal: usize,
    asserted_type_site: TypedSiteRef,
    asserted_type: SourceTypeExpression,
}

#[derive(Debug)]
struct SourceReservedVariableTypeAssertionOutput {
    payload: SourceReservedVariableTypeAssertion,
    handoff: SourceReserveHandoff,
    subject_binding: BindingId,
    subject_result_input: TypeExpressionInput,
    asserted_type_input: TypeExpressionInput,
    term_formula: TermFormulaInferenceOutput,
}

fn source_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_parenthesized_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_parenthesized_reserved_variable_equality(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

fn source_parenthesized_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_parenthesized_reserved_variable_inequality(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

fn source_parenthesized_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_parenthesized_reserved_variable_membership(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

fn source_parenthesized_heterogeneous_reserve_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_parenthesized_heterogeneous_reserve_membership(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

fn source_right_parenthesized_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_right_parenthesized_reserved_variable_membership(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
            SourceParenthesizedOperandSide::Right,
        ),
    )
}

fn source_parenthesized_two_edge_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

fn source_parenthesized_reserved_object_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_parenthesized_reserved_object_variable_equality(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

fn source_parenthesized_reserved_object_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_parenthesized_reserved_object_variable_inequality(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

fn source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
    payload: SourceParenthesizedReservedVariableBinaryFormula,
    symbols: &SymbolEnv,
    config: &'static SourceReservedVariableBinaryFormulaConfig,
    expected_side: SourceParenthesizedOperandSide,
) -> Vec<String> {
    match build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols) {
        Ok(output) => {
            source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
                &output,
                config,
                expected_side,
            )
        }
        Err(_) => vec![config.invalid_payload_key.to_owned()],
    }
}

fn source_reserved_object_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_object_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_distinct_reserved_object_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_distinct_reserved_object_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_distinct_reserved_object_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_distinct_reserved_object_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_reserved_object_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_object_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_distinct_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_distinct_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_distinct_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_distinct_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_distinct_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_distinct_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_heterogeneous_reserve_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_heterogeneous_reserve_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_chained_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_two_edge_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_three_edge_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_four_edge_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_four_edge_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_three_edge_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_two_edge_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_chained_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_chained_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_two_edge_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_three_edge_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_four_edge_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_mode_long_chain_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_mode_long_chain_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_object_mode_long_chain_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_equality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_object_mode_long_chain_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_object_mode_long_chain_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_membership(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_mode_long_chain_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_mode_long_chain_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_mode_long_chain_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_mode_long_chain_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_four_edge_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_four_edge_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_four_edge_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_three_edge_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_three_edge_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_three_edge_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_two_edge_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_two_edge_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_two_edge_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_chained_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_chained_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_chained_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_multiple_reserve_declaration_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_multiple_reserve_declaration_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_multiple_object_reserve_declaration_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_multiple_object_reserve_declaration_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_multiple_object_reserve_declaration_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_multiple_object_reserve_declaration_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_multiple_reserve_declaration_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_multiple_reserve_declaration_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_multiple_reserve_declaration_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_multiple_reserve_declaration_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

fn source_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

fn source_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_variable_type_assertion(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_reserved_object_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_object_variable_type_assertion(ast, module, symbols)?;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    ))
}

fn source_local_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_object_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_chained_local_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_chained_local_mode_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_mode_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_chained_local_object_mode_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_two_edge_local_mode_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_mode_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_two_edge_local_mode_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_mode_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_two_edge_local_object_mode_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_object_mode_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_mode_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_mode_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_object_mode_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_object_mode_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_mode_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_mode_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_object_mode_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_object_mode_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_mode_three_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_mode_three_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_object_mode_three_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_object_mode_three_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_mode_three_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_mode_three_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_object_mode_three_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_object_mode_three_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_mode_four_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_mode_four_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_object_mode_four_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_object_mode_four_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_two_edge_local_object_mode_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_mode_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_mode_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_object_mode_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_mode_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_mode_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_object_mode_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_chained_local_object_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_object_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_two_edge_local_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_object_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_object_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_object_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_object_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_two_edge_local_object_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_object_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_chained_local_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_chained_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_two_edge_local_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_two_edge_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_three_edge_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_mode_long_chain_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_mode_long_chain_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_mode_long_chain_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_mode_long_chain_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_mode_long_chain_three_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_mode_long_chain_three_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_mode_long_chain_four_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_mode_long_chain_four_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_mode_long_chain_five_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_mode_long_chain_five_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_mode_long_chain_six_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_six_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_long_chain_six_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_six_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_long_chain_five_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_five_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_long_chain_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_long_chain_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_long_chain_three_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_three_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_long_chain_four_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_four_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_long_chain_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_object_mode_long_chain_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_long_chain_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_four_edge_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

fn source_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    ))
}

fn source_reserved_variable_type_assertion_result_detail_keys(
    output: Result<SourceReservedVariableTypeAssertionOutput, String>,
    invalid_payload_key: &str,
) -> Vec<String> {
    match output.and_then(|output| {
        assert_source_reserved_variable_type_assertion_output(&output)?;
        Ok(output)
    }) {
        Ok(output) => source_reserved_variable_type_assertion_output_detail_keys(&output),
        Err(_) => vec![invalid_payload_key.to_owned()],
    }
}

fn source_reserved_variable_type_assertion_output_detail_keys(
    output: &SourceReservedVariableTypeAssertionOutput,
) -> Vec<String> {
    let mut keys = output
        .handoff
        .binding_env
        .diagnostics()
        .canonical_iter()
        .map(|(_, diagnostic)| format!("type_elaboration.checker.{}", diagnostic.message_key))
        .chain(
            output
                .handoff
                .declarations
                .diagnostics()
                .canonical_iter()
                .map(|(_, diagnostic)| {
                    format!("type_elaboration.checker.{}", diagnostic.message_key)
                }),
        )
        .chain(
            output
                .term_formula
                .diagnostics()
                .canonical_iter()
                .map(|(_, diagnostic)| {
                    format!("type_elaboration.checker.{}", diagnostic.message_key)
                }),
        )
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys
}

fn source_reserved_variable_formula_result_detail_keys(
    output: Result<SourceReservedVariableBinaryFormulaOutput, String>,
    invalid_payload_key: &str,
) -> Vec<String> {
    match output {
        Ok(output) => source_reserved_variable_formula_output_detail_keys(&output),
        Err(_) => vec![invalid_payload_key.to_owned()],
    }
}

fn source_reserved_variable_formula_output_detail_keys(
    output: &SourceReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    if assert_source_reserved_variable_formula_output(output).is_err() {
        return vec![output.payload.config.invalid_payload_key.to_owned()];
    }
    let mut keys = output
        .handoff
        .declarations
        .diagnostics()
        .canonical_iter()
        .map(|(_, diagnostic)| format!("type_elaboration.checker.{}", diagnostic.message_key))
        .chain(
            output
                .term_formula
                .diagnostics()
                .canonical_iter()
                .map(|(_, diagnostic)| {
                    format!("type_elaboration.checker.{}", diagnostic.message_key)
                }),
        )
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys
}

#[cfg(test)]
fn source_parenthesized_reserved_variable_equality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn source_parenthesized_reserved_variable_inequality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn source_parenthesized_reserved_variable_membership_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn source_parenthesized_heterogeneous_reserve_membership_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn source_right_parenthesized_reserved_variable_membership_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Right,
    )
}

#[cfg(test)]
fn source_parenthesized_two_edge_local_mode_reserved_variable_equality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn source_parenthesized_reserved_object_variable_equality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn source_parenthesized_reserved_object_variable_inequality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

fn source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
    config: &'static SourceReservedVariableBinaryFormulaConfig,
    expected_side: SourceParenthesizedOperandSide,
) -> Vec<String> {
    if assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        config,
        expected_side,
    )
    .is_err()
    {
        return vec![config.invalid_payload_key.to_owned()];
    }
    source_reserved_variable_formula_output_detail_keys(&output.formula)
}

#[cfg(test)]
fn source_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_parenthesized_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_parenthesized_reserved_variable_equality(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_parenthesized_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_parenthesized_reserved_variable_inequality(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_parenthesized_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_parenthesized_reserved_variable_membership(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_parenthesized_heterogeneous_reserve_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_parenthesized_heterogeneous_reserve_membership(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_right_parenthesized_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_right_parenthesized_reserved_variable_membership(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_parenthesized_two_edge_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_parenthesized_reserved_object_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_parenthesized_reserved_object_variable_equality(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_parenthesized_reserved_object_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_parenthesized_reserved_object_variable_inequality(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_reserved_object_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_object_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_distinct_reserved_object_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_distinct_reserved_object_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_distinct_reserved_object_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_distinct_reserved_object_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_reserved_object_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_object_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_distinct_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_distinct_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_distinct_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_distinct_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_distinct_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_distinct_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_heterogeneous_reserve_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_heterogeneous_reserve_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_local_object_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_local_object_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_local_mode_long_chain_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_equality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_membership(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_local_mode_long_chain_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_local_mode_long_chain_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_two_edge_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_chained_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_multiple_reserve_declaration_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_multiple_reserve_declaration_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_multiple_object_reserve_declaration_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_multiple_object_reserve_declaration_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_multiple_object_reserve_declaration_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_multiple_object_reserve_declaration_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_multiple_reserve_declaration_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_multiple_reserve_declaration_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_multiple_reserve_declaration_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_multiple_reserve_declaration_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_reserved_variable_type_assertion(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_reserved_object_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_reserved_object_variable_type_assertion(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_object_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_chained_local_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_mode_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_chained_local_mode_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_object_mode_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_chained_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_mode_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_two_edge_local_mode_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_mode_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_two_edge_local_mode_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_object_mode_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_two_edge_local_object_mode_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_mode_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_three_edge_local_mode_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_object_mode_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_three_edge_local_object_mode_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_mode_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_four_edge_local_mode_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_object_mode_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_four_edge_local_object_mode_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_mode_three_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_three_edge_local_mode_three_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_object_mode_three_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_three_edge_local_object_mode_three_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_mode_three_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_four_edge_local_mode_three_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_object_mode_three_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_four_edge_local_object_mode_three_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_mode_four_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_four_edge_local_mode_four_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_object_mode_four_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_four_edge_local_object_mode_four_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_object_mode_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_two_edge_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_mode_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_three_edge_local_mode_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_object_mode_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_three_edge_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_mode_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_four_edge_local_mode_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_object_mode_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_four_edge_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_object_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_chained_local_object_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_two_edge_local_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_three_edge_local_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_four_edge_local_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_object_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_four_edge_local_object_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_object_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_three_edge_local_object_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_object_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_two_edge_local_object_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_chained_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_two_edge_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_three_edge_local_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_three_edge_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_three_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_mode_long_chain_three_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_four_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_mode_long_chain_four_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_five_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_mode_long_chain_five_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_mode_long_chain_six_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_six_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_six_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_six_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_five_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_five_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_three_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_three_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_four_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_four_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_object_mode_long_chain_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_four_edge_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

#[cfg(test)]
fn source_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

fn build_source_reserved_variable_type_assertion_output(
    payload: SourceReservedVariableTypeAssertion,
    symbols: &SymbolEnv,
) -> Result<SourceReservedVariableTypeAssertionOutput, String> {
    let handoff = assemble_source_reserve_checker_handoff(
        symbols,
        &payload.reserve.bridge,
        payload.reserve.mode_expansions.clone(),
    )?;
    let context = payload.reserve.bridge.module_context();
    let subject_binding = match handoff
        .binding_env
        .lookup(&BindingLookupSite::new(
            payload.subject_spelling.clone(),
            context,
            None,
            payload.subject_lookup_ordinal,
        ))
        .map_err(|error| error.to_string())?
    {
        BindingLookupResult::Local(binding) => binding,
        _ => {
            return Err(
                "reserved-variable type assertion lookup did not resolve locally".to_owned(),
            );
        }
    };
    if subject_binding != BindingId::new(0) {
        return Err("reserved-variable type assertion binding identity mismatch".to_owned());
    }
    let source_binding = payload
        .reserve
        .bridge
        .bindings()
        .get(subject_binding.index())
        .ok_or_else(|| "reserved-variable type assertion source binding missing".to_owned())?;
    if source_binding.spelling != payload.config.binding_spelling
        || !source_binding_matches_reserved_builtin_type(
            source_binding,
            payload.config.binding_type,
            payload.config.binding_source_mode_spelling,
            &payload.reserve.mode_expansions,
        )
    {
        return Err("reserved-variable type assertion source binding mismatch".to_owned());
    }

    let subject_result_input = source_reserved_type_projection(
        source_binding,
        payload.subject_site.node(),
        payload.config.subject_result_role,
    );
    let asserted_type_input = TypeExpressionInput::new(
        payload.asserted_type_site.clone(),
        payload.asserted_type.range,
        payload.asserted_type.spelling.clone(),
        payload.asserted_type.head.clone(),
    )
    .with_attributes(payload.asserted_type.attributes.clone());
    let term_formula =
        TermFormulaChecker::new(TypeNormalizer::new(payload.reserve.mode_expansions.clone()))
            .infer(
                symbols,
                &handoff.binding_env,
                [TermInput::new(
                    payload.subject_site.clone(),
                    context,
                    payload.subject_range,
                    TermKind::Variable,
                )
                .with_reference(TermReference::Binding(subject_binding))
                .with_result_type(subject_result_input.clone())],
                [FormulaInput::new(
                    payload.formula_site.clone(),
                    context,
                    payload.formula_range,
                    FormulaKind::TypeAssertion,
                )
                .with_terms(vec![payload.subject_site.clone()])
                .with_asserted_type(asserted_type_input.clone())],
            );

    Ok(SourceReservedVariableTypeAssertionOutput {
        payload,
        handoff,
        subject_binding,
        subject_result_input,
        asserted_type_input,
        term_formula,
    })
}

fn build_source_reserved_variable_formula_output(
    payload: SourceReservedVariableBinaryFormula,
    symbols: &SymbolEnv,
) -> Result<SourceReservedVariableBinaryFormulaOutput, String> {
    let handoff = assemble_source_reserve_checker_handoff(
        symbols,
        &payload.reserve.bridge,
        payload.reserve.mode_expansions.clone(),
    )?;

    let context = payload.reserve.bridge.module_context();
    let left_binding = match handoff
        .binding_env
        .lookup(&BindingLookupSite::new(
            payload.left_spelling.clone(),
            context,
            None,
            payload.left_lookup_ordinal,
        ))
        .map_err(|error| error.to_string())?
    {
        BindingLookupResult::Local(binding) => binding,
        _ => {
            return Err("left reserved-variable formula lookup did not resolve locally".to_owned());
        }
    };
    let right_binding = match handoff
        .binding_env
        .lookup(&BindingLookupSite::new(
            payload.right_spelling.clone(),
            context,
            None,
            payload.right_lookup_ordinal,
        ))
        .map_err(|error| error.to_string())?
    {
        BindingLookupResult::Local(binding) => binding,
        _ => {
            return Err(
                "right reserved-variable formula lookup did not resolve locally".to_owned(),
            );
        }
    };
    let expected_left_binding = BindingId::new(payload.config.left_binding_index);
    let expected_right_binding = BindingId::new(payload.config.right_binding_index);
    if left_binding != expected_left_binding || right_binding != expected_right_binding {
        return Err("reserved-variable formula binding identity mismatch".to_owned());
    }
    let left_source_binding = payload
        .reserve
        .bridge
        .bindings()
        .get(left_binding.index())
        .ok_or_else(|| "left reserved-variable formula source binding missing".to_owned())?;
    let right_source_binding = payload
        .reserve
        .bridge
        .bindings()
        .get(right_binding.index())
        .ok_or_else(|| "right reserved-variable formula source binding missing".to_owned())?;
    if left_source_binding.spelling != payload.left_spelling
        || right_source_binding.spelling != payload.right_spelling
    {
        return Err("reserved-variable formula source binding shape mismatch".to_owned());
    }

    let left_result_type = source_reserved_type_projection(
        left_source_binding,
        payload.left_site.node(),
        payload.config.left_result_role,
    );
    let right_result_type = source_reserved_type_projection(
        right_source_binding,
        payload.right_site.node(),
        payload.config.right_result_role,
    );
    let left_result_input = left_result_type.clone();
    let right_result_input = right_result_type.clone();
    let mut expected_types = Vec::new();
    let left_expected_input = payload.config.left_expected_role.map(|role| {
        source_reserved_type_projection(left_source_binding, payload.left_site.node(), role)
    });
    let right_expected_input = payload.config.right_expected_role.map(|role| {
        source_reserved_type_projection(right_source_binding, payload.right_site.node(), role)
    });
    if let Some(expected) = &left_expected_input {
        expected_types.push(ExpectedTypeInput::new(
            payload.left_site.clone(),
            expected.clone(),
            payload.left_range,
        ));
    }
    if let Some(expected) = &right_expected_input {
        expected_types.push(ExpectedTypeInput::new(
            payload.right_site.clone(),
            expected.clone(),
            payload.right_range,
        ));
    }
    let term_formula =
        TermFormulaChecker::new(TypeNormalizer::new(payload.reserve.mode_expansions.clone()))
            .infer(
                symbols,
                &handoff.binding_env,
                [
                    TermInput::new(
                        payload.left_site.clone(),
                        context,
                        payload.left_range,
                        TermKind::Variable,
                    )
                    .with_reference(TermReference::Binding(left_binding))
                    .with_result_type(left_result_type),
                    TermInput::new(
                        payload.right_site.clone(),
                        context,
                        payload.right_range,
                        TermKind::Variable,
                    )
                    .with_reference(TermReference::Binding(right_binding))
                    .with_result_type(right_result_type),
                ],
                [FormulaInput::new(
                    payload.formula_site.clone(),
                    context,
                    payload.formula_range,
                    payload.config.formula_kind,
                )
                .with_terms(vec![payload.left_site.clone(), payload.right_site.clone()])
                .with_expected_types(expected_types)],
            );

    Ok(SourceReservedVariableBinaryFormulaOutput {
        payload,
        handoff,
        left_binding,
        right_binding,
        left_result_input,
        right_result_input,
        left_expected_input,
        right_expected_input,
        term_formula,
    })
}

fn build_source_parenthesized_reserved_variable_binary_formula_output(
    payload: SourceParenthesizedReservedVariableBinaryFormula,
    symbols: &SymbolEnv,
) -> Result<SourceParenthesizedReservedVariableBinaryFormulaOutput, String> {
    let SourceParenthesizedReservedVariableBinaryFormula {
        wrapper_side,
        wrapper_site,
        wrapper_range,
        formula,
    } = payload;
    let source_wrapper_side = wrapper_side;
    let source_wrapper_site = wrapper_site.clone();
    let source_wrapper_range = wrapper_range;
    let formula = build_source_reserved_variable_formula_output(formula, symbols)?;
    Ok(SourceParenthesizedReservedVariableBinaryFormulaOutput {
        source_wrapper_side,
        source_wrapper_site,
        source_wrapper_range,
        wrapper_side,
        wrapper_site,
        wrapper_range,
        formula,
    })
}

fn assert_source_reserved_variable_type_assertion_output(
    output: &SourceReservedVariableTypeAssertionOutput,
) -> Result<(), String> {
    let payload = &output.payload;
    let [source_binding] = payload.reserve.bridge.bindings() else {
        return Err("reserved-variable type assertion binding count mismatch".to_owned());
    };
    assert_source_reserve_handoff(&output.handoff, &payload.reserve.bridge)?;
    if source_binding.spelling != payload.config.binding_spelling
        || !source_binding_matches_reserved_builtin_type(
            source_binding,
            payload.config.binding_type,
            payload.config.binding_source_mode_spelling,
            &payload.reserve.mode_expansions,
        )
        || !source_reserved_variable_mode_expansions_are_exact(
            &payload.reserve,
            payload.config.mode_definitions,
        )
        || output.subject_binding != BindingId::new(0)
        || output.handoff.binding_env.bindings().len() != 1
        || !output.handoff.binding_env.diagnostics().is_empty()
        || output.handoff.declarations.declarations().len() != 1
        || !output.handoff.declarations.facts().is_empty()
        || !output.handoff.declarations.diagnostics().is_empty()
    {
        return Err("reserved-variable type assertion handoff mismatch".to_owned());
    }
    let [expected_ordinal] =
        source_binding_use_ordinals(payload.reserve.bridge.bindings(), [payload.subject_range])?;
    if payload.subject_lookup_ordinal != expected_ordinal {
        return Err("reserved-variable type assertion lookup ordinal mismatch".to_owned());
    }
    match output
        .handoff
        .binding_env
        .lookup(&BindingLookupSite::new(
            payload.subject_spelling.clone(),
            payload.reserve.bridge.module_context(),
            None,
            payload.subject_lookup_ordinal,
        ))
        .map_err(|error| error.to_string())?
    {
        BindingLookupResult::Local(binding) if binding == output.subject_binding => {}
        _ => return Err("reserved-variable type assertion lookup result mismatch".to_owned()),
    }

    if output.subject_result_input.site
        != (TypedSiteRef::Role {
            node: payload.subject_site.node(),
            role: TypeRole::new(payload.config.subject_result_role),
        })
        || output.subject_result_input.source_range != source_binding.type_range
        || output.subject_result_input.spelling != source_binding.type_spelling
        || output.subject_result_input.head != source_binding.type_head
        || !output.subject_result_input.args.is_empty()
        || !output.subject_result_input.attributes.is_empty()
        || output.asserted_type_input.site != payload.asserted_type_site
        || output.asserted_type_input.source_range != payload.asserted_type.range
        || output.asserted_type_input.spelling != payload.asserted_type.spelling
        || output.asserted_type_input.head != payload.asserted_type.head
        || !output.asserted_type_input.args.is_empty()
        || !output.asserted_type_input.attributes.is_empty()
        || !source_type_expression_matches_reserved_builtin_type(
            &payload.asserted_type,
            payload.config.asserted_type,
            payload.config.asserted_head_relation.source_mode_spelling(),
            &payload.reserve.mode_expansions,
        )
        || !source_reserved_variable_asserted_head_relation_is_exact(
            source_binding,
            &output.asserted_type_input.spelling,
            &output.asserted_type_input.head,
            payload.config,
            &payload.reserve.mode_expansions,
        )
        || output.asserted_type_input.site == output.subject_result_input.site
        || output.asserted_type_input.source_range == output.subject_result_input.source_range
    {
        return Err("reserved-variable type assertion input provenance mismatch".to_owned());
    }

    let term_formula = &output.term_formula;
    if term_formula.terms().len() != 1
        || term_formula.formulas().len() != 1
        || term_formula.type_entries().len() != 3
        || term_formula.normalized_types().len() != 1
        || !term_formula.candidate_sets().is_empty()
        || !term_formula.facts().is_empty()
        || !term_formula.diagnostics().is_empty()
    {
        return Err("reserved-variable type assertion checker count mismatch".to_owned());
    }
    let term = term_formula
        .terms()
        .iter()
        .map(|(_, term)| term)
        .next()
        .ok_or_else(|| "reserved-variable type assertion subject missing".to_owned())?;
    if term.site != payload.subject_site
        || term.context != payload.reserve.bridge.module_context()
        || term.kind != TermKind::Variable
        || term.reference != Some(TermReference::Binding(output.subject_binding))
        || term.expected_type.is_some()
        || term.candidate_set.is_some()
        || term.status != TermStatus::Inferred
        || !term.deferred.is_empty()
    {
        return Err("reserved-variable type assertion subject mismatch".to_owned());
    }
    let subject_entry = term_formula
        .type_entries()
        .get(term.type_entry)
        .ok_or_else(|| "reserved-variable type assertion term type entry missing".to_owned())?;
    if subject_entry.owner != payload.subject_site
        || subject_entry.expected.is_some()
        || subject_entry.status != TypeStatus::Known
    {
        return Err("reserved-variable type assertion subject type entry mismatch".to_owned());
    }
    let TypeEntryActual::Known(subject_actual) = subject_entry.actual else {
        return Err("reserved-variable type assertion subject type unknown".to_owned());
    };
    let result_role_actual = type_entry_known_actual_for_owner(
        term_formula,
        &output.subject_result_input.site,
        "reserved-variable type assertion result role",
    )?;
    let asserted_role_actual = type_entry_known_actual_for_owner(
        term_formula,
        &output.asserted_type_input.site,
        "reserved-variable type assertion asserted role",
    )?;

    let formula = term_formula
        .formulas()
        .iter()
        .map(|(_, formula)| formula)
        .next()
        .ok_or_else(|| "reserved-variable type assertion formula missing".to_owned())?;
    if formula.site != payload.formula_site
        || formula.context != payload.reserve.bridge.module_context()
        || formula.kind != FormulaKind::TypeAssertion
        || formula.terms != [payload.subject_site.clone()]
        || formula.asserted_type != Some(asserted_role_actual)
        || !formula.expected_types.is_empty()
        || formula.candidate_set.is_some()
        || !formula.facts.is_empty()
        || formula.status != FormulaStatus::Checked
        || !formula.deferred.is_empty()
        || subject_actual != result_role_actual
        || subject_actual != asserted_role_actual
    {
        return Err("reserved-variable type assertion formula mismatch".to_owned());
    }
    let normalized = term_formula
        .normalized_types()
        .get(subject_actual)
        .ok_or_else(|| "reserved-variable type assertion normalized type missing".to_owned())?;
    if !normalized_type_is_reserved_builtin_type(
        term_formula,
        subject_actual,
        payload.config.binding_type,
    ) || !normalized.args.is_empty()
        || !normalized.attributes.positive().is_empty()
        || !normalized.attributes.negative().is_empty()
        || normalized.status != NormalizedTypeStatus::Known
    {
        return Err("reserved-variable type assertion normalized type mismatch".to_owned());
    }
    let canonical_source = if payload.config.binding_source_mode_spelling.is_some() {
        let TypeHeadInput::Symbol(symbol) = &source_binding.type_head else {
            return Err("reserved-variable type assertion mode head missing".to_owned());
        };
        let terminal = source_mode_terminal_builtin_input(
            symbol,
            payload.config.binding_type,
            &payload.reserve.mode_expansions,
        )
        .ok_or_else(|| "reserved-variable type assertion terminal source missing".to_owned())?;
        (terminal.source_range, terminal.spelling.as_str())
    } else {
        (
            source_binding.type_range,
            source_binding.type_spelling.as_str(),
        )
    };
    if normalized.source.range != canonical_source.0
        || normalized.source.spelling != canonical_source.1
    {
        return Err("reserved-variable type assertion canonical source mismatch".to_owned());
    }
    Ok(())
}

fn type_entry_known_actual_for_owner(
    output: &TermFormulaInferenceOutput,
    owner: &TypedSiteRef,
    description: &str,
) -> Result<NormalizedTypeId, String> {
    let (_, entry) = output
        .type_entries()
        .iter()
        .find(|(_, entry)| &entry.owner == owner)
        .ok_or_else(|| format!("{description} type entry missing"))?;
    if entry.expected.is_some() || entry.status != TypeStatus::Known {
        return Err(format!("{description} type entry mismatch"));
    }
    match entry.actual {
        TypeEntryActual::Known(actual) => Ok(actual),
        _ => Err(format!("{description} type entry unknown")),
    }
}

fn assert_source_reserved_variable_formula_output(
    output: &SourceReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    let payload = &output.payload;
    let source_bindings = payload.reserve.bridge.bindings();
    if source_bindings.len() != payload.config.binding_spellings.len()
        || source_bindings.len() != payload.config.binding_types.len()
        || source_bindings.len() != payload.config.binding_source_mode_spellings.len()
    {
        return Err("reserved-variable formula binding count mismatch".to_owned());
    }
    assert_source_reserve_handoff(&output.handoff, &payload.reserve.bridge)?;
    if source_bindings.iter().enumerate().any(|(index, binding)| {
        let spelling = payload.config.binding_spellings[index];
        binding.spelling != spelling
            || !source_binding_matches_reserved_builtin_type(
                binding,
                payload.config.binding_types[index],
                payload.config.binding_source_mode_spellings[index],
                &payload.reserve.mode_expansions,
            )
    }) || !source_reserved_variable_mode_expansions_are_exact(
        &payload.reserve,
        payload.config.mode_definitions,
    ) || (payload.config.require_shared_type_range
        && source_bindings
            .windows(2)
            .any(|pair| pair[0].type_range != pair[1].type_range))
        || (payload.config.require_distinct_type_ranges
            && source_bindings.windows(2).any(|pair| {
                pair[0].type_range == pair[1].type_range
                    || (pair[0].type_range.start, pair[0].type_range.end)
                        >= (pair[1].type_range.start, pair[1].type_range.end)
            }))
        || output.handoff.binding_env.bindings().len() != source_bindings.len()
        || !output.handoff.binding_env.diagnostics().is_empty()
        || output
            .handoff
            .binding_env
            .bindings()
            .iter()
            .any(|(_, binding)| !binding.diagnostics.is_empty())
        || output.handoff.declarations.declarations().len() != source_bindings.len()
        || !output.handoff.declarations.facts().is_empty()
        || !output.handoff.declarations.diagnostics().is_empty()
    {
        return Err("reserved-variable formula declaration payload mismatch".to_owned());
    }

    let expected_ordinals = source_binding_use_ordinals(
        payload.reserve.bridge.bindings(),
        [payload.left_range, payload.right_range],
    )?;
    let expected_left_binding = BindingId::new(payload.config.left_binding_index);
    let expected_right_binding = BindingId::new(payload.config.right_binding_index);
    if [payload.left_lookup_ordinal, payload.right_lookup_ordinal] != expected_ordinals
        || output.left_binding != expected_left_binding
        || output.right_binding != expected_right_binding
    {
        return Err("reserved-variable formula lookup metadata mismatch".to_owned());
    }
    for (spelling, ordinal, expected_binding) in [
        (
            payload.left_spelling.as_str(),
            payload.left_lookup_ordinal,
            output.left_binding,
        ),
        (
            payload.right_spelling.as_str(),
            payload.right_lookup_ordinal,
            output.right_binding,
        ),
    ] {
        match output
            .handoff
            .binding_env
            .lookup(&BindingLookupSite::new(
                spelling,
                payload.reserve.bridge.module_context(),
                None,
                ordinal,
            ))
            .map_err(|error| error.to_string())?
        {
            BindingLookupResult::Local(binding) if binding == expected_binding => {}
            _ => return Err("reserved-variable formula lookup result mismatch".to_owned()),
        }
    }

    for (input, source_binding, node, role) in [
        (
            &output.left_result_input,
            &source_bindings[payload.config.left_binding_index],
            payload.left_site.node(),
            payload.config.left_result_role,
        ),
        (
            &output.right_result_input,
            &source_bindings[payload.config.right_binding_index],
            payload.right_site.node(),
            payload.config.right_result_role,
        ),
    ] {
        if !source_type_projection_matches(input, source_binding, node, role) {
            return Err("reserved-variable formula result input provenance mismatch".to_owned());
        }
    }
    for (input, source_binding, node, role) in [
        (
            output.left_expected_input.as_ref(),
            &source_bindings[payload.config.left_binding_index],
            payload.left_site.node(),
            payload.config.left_expected_role,
        ),
        (
            output.right_expected_input.as_ref(),
            &source_bindings[payload.config.right_binding_index],
            payload.right_site.node(),
            payload.config.right_expected_role,
        ),
    ] {
        match (input, role) {
            (Some(input), Some(role))
                if source_type_projection_matches(input, source_binding, node, role) => {}
            (None, None) => {}
            _ => {
                return Err(
                    "reserved-variable formula expected input provenance mismatch".to_owned(),
                );
            }
        }
    }

    let term_formula = &output.term_formula;
    let expected_type_count = usize::from(payload.config.left_expected_role.is_some())
        + usize::from(payload.config.right_expected_role.is_some());
    if term_formula.terms().len() != 2
        || term_formula.formulas().len() != 1
        || !term_formula.candidate_sets().is_empty()
        || !term_formula.facts().is_empty()
        || !term_formula.diagnostics().is_empty()
        || term_formula.type_entries().len() != 4 + expected_type_count
    {
        return Err(format!(
            "reserved-variable formula checker count mismatch: terms={} formulas={} candidates={} facts={} diagnostics={} type_entries={} expected_type_entries={}",
            term_formula.terms().len(),
            term_formula.formulas().len(),
            term_formula.candidate_sets().len(),
            term_formula.facts().len(),
            term_formula.diagnostics().len(),
            term_formula.type_entries().len(),
            4 + expected_type_count,
        ));
    }
    let mut term_actuals = BTreeMap::new();
    let mut semantic_ids_by_type = BTreeMap::new();
    for (site, binding, binding_index) in [
        (
            &payload.left_site,
            output.left_binding,
            payload.config.left_binding_index,
        ),
        (
            &payload.right_site,
            output.right_binding,
            payload.config.right_binding_index,
        ),
    ] {
        let term = term_formula
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| &term.site == site)
            .ok_or_else(|| "reserved-variable formula term missing".to_owned())?;
        if term.context != payload.reserve.bridge.module_context()
            || term.kind != TermKind::Variable
            || term.reference != Some(TermReference::Binding(binding))
            || term.expected_type.is_some()
            || term.candidate_set.is_some()
            || term.status != TermStatus::Inferred
            || !term.deferred.is_empty()
        {
            return Err("reserved-variable formula term payload mismatch".to_owned());
        }
        let expected_type = payload.config.binding_types[binding_index];
        let actual = assert_reserved_variable_builtin_type_entry(
            term_formula,
            &term.site,
            Some(term.type_entry),
            expected_type,
        )?;
        if semantic_ids_by_type
            .insert(expected_type, actual)
            .is_some_and(|existing| existing != actual)
        {
            return Err("reserved-variable formula semantic type identity mismatch".to_owned());
        }
        term_actuals.insert(term.site.clone(), actual);
    }

    let formula = term_formula
        .formulas()
        .iter()
        .map(|(_, formula)| formula)
        .next()
        .ok_or_else(|| "reserved-variable formula missing".to_owned())?;
    if formula.site != payload.formula_site
        || formula.context != payload.reserve.bridge.module_context()
        || formula.kind != payload.config.formula_kind
        || formula.terms != [payload.left_site.clone(), payload.right_site.clone()]
        || formula.asserted_type.is_some()
        || formula.candidate_set.is_some()
        || formula.status != FormulaStatus::Checked
        || !formula.facts.is_empty()
        || !formula.deferred.is_empty()
        || formula.expected_types.len() != expected_type_count
    {
        return Err("reserved-variable formula payload mismatch".to_owned());
    }
    let expected_constraints = [
        payload.config.left_expected_role.map(|role| {
            (
                &payload.left_site,
                payload.left_range,
                role,
                payload.config.left_binding_index,
            )
        }),
        payload.config.right_expected_role.map(|role| {
            (
                &payload.right_site,
                payload.right_range,
                role,
                payload.config.right_binding_index,
            )
        }),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    for (constraint, (site, range, role, binding_index)) in formula
        .expected_types
        .iter()
        .zip(expected_constraints.iter().copied())
    {
        let expected_type = payload.config.binding_types[binding_index];
        if &constraint.term != site
            || constraint.source_range != range
            || constraint.status != TypeStatus::Known
            || !normalized_type_is_reserved_builtin_type(
                term_formula,
                constraint.expected,
                expected_type,
            )
        {
            return Err("reserved-variable formula expected type mismatch".to_owned());
        }
        let owner = TypedSiteRef::Role {
            node: site.node(),
            role: TypeRole::new(role),
        };
        let role_actual =
            assert_reserved_variable_builtin_type_entry(term_formula, &owner, None, expected_type)?;
        if role_actual != constraint.expected || term_actuals.get(site) != Some(&role_actual) {
            return Err("reserved-variable expected role is not referenced".to_owned());
        }
    }
    for (site, role, binding_index) in [
        (
            &payload.left_site,
            payload.config.left_result_role,
            payload.config.left_binding_index,
        ),
        (
            &payload.right_site,
            payload.config.right_result_role,
            payload.config.right_binding_index,
        ),
    ] {
        let owner = TypedSiteRef::Role {
            node: site.node(),
            role: TypeRole::new(role),
        };
        let role_actual = assert_reserved_variable_builtin_type_entry(
            term_formula,
            &owner,
            None,
            payload.config.binding_types[binding_index],
        )?;
        if term_actuals.get(site) != Some(&role_actual) {
            return Err("reserved-variable result role is not referenced".to_owned());
        }
    }
    let expected_semantic_type_count = payload
        .config
        .binding_types
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .len();
    if semantic_ids_by_type.len() != expected_semantic_type_count
        || term_formula.normalized_types().len() != expected_semantic_type_count
    {
        return Err("reserved-variable formula semantic type identity mismatch".to_owned());
    }
    for (expected_type, semantic_id) in semantic_ids_by_type {
        let canonical_source = source_bindings
            .iter()
            .enumerate()
            .filter(|(index, _)| payload.config.binding_types[*index] == expected_type)
            .filter_map(|(index, binding)| {
                let Some(_) = payload.config.binding_source_mode_spellings[index] else {
                    return Some((binding.type_range, binding.type_spelling.as_str()));
                };
                let TypeHeadInput::Symbol(symbol) = &binding.type_head else {
                    return None;
                };
                source_mode_terminal_builtin_input(
                    symbol,
                    expected_type,
                    &payload.reserve.mode_expansions,
                )
                .map(|terminal| (terminal.source_range, terminal.spelling.as_str()))
            })
            .min_by_key(|(range, _)| (range.start, range.end))
            .ok_or_else(|| "reserved-variable formula canonical source missing".to_owned())?;
        let normalized = term_formula
            .normalized_types()
            .get(semantic_id)
            .ok_or_else(|| "reserved-variable formula normalized type missing".to_owned())?;
        if normalized.source.range != canonical_source.0
            || normalized.source.spelling != canonical_source.1
        {
            return Err("reserved-variable formula canonical source mismatch".to_owned());
        }
    }
    Ok(())
}

#[cfg(test)]
fn assert_source_parenthesized_reserved_variable_equality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn assert_source_parenthesized_reserved_variable_inequality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn assert_source_parenthesized_reserved_variable_membership_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn assert_source_parenthesized_heterogeneous_reserve_membership_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn assert_source_right_parenthesized_reserved_variable_membership_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Right,
    )
}

#[cfg(test)]
fn assert_source_parenthesized_two_edge_local_mode_reserved_variable_equality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn assert_source_parenthesized_reserved_object_variable_equality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
fn assert_source_parenthesized_reserved_object_variable_inequality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

fn assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
    config: &'static SourceReservedVariableBinaryFormulaConfig,
    expected_side: SourceParenthesizedOperandSide,
) -> Result<(), String> {
    assert_source_reserved_variable_formula_output(&output.formula)?;
    let payload = &output.formula.payload;
    let wrapper_range_is_valid = match expected_side {
        SourceParenthesizedOperandSide::Left => {
            output.wrapper_range.start < payload.left_range.start
                && output.wrapper_range.end > payload.left_range.end
                && output.wrapper_range.end <= payload.right_range.start
                && payload.formula_range.start <= output.wrapper_range.start
                && payload.formula_range.end >= payload.right_range.end
        }
        SourceParenthesizedOperandSide::Right => {
            payload.left_range.end <= output.wrapper_range.start
                && output.wrapper_range.start < payload.right_range.start
                && output.wrapper_range.end > payload.right_range.end
                && payload.formula_range.start <= payload.left_range.start
                && payload.formula_range.end >= output.wrapper_range.end
        }
    };
    if !std::ptr::eq(payload.config, config)
        || output.source_wrapper_side != expected_side
        || output.wrapper_side != output.source_wrapper_side
        || output.wrapper_site != output.source_wrapper_site
        || output.wrapper_range != output.source_wrapper_range
        || output.wrapper_site == payload.formula_site
        || output.wrapper_site == payload.left_site
        || output.wrapper_site == payload.right_site
        || payload.formula_site == payload.left_site
        || payload.formula_site == payload.right_site
        || payload.left_site == payload.right_site
        || output.wrapper_range.source_id != payload.left_range.source_id
        || output.wrapper_range.source_id != payload.right_range.source_id
        || output.wrapper_range.source_id != payload.formula_range.source_id
        || !wrapper_range_is_valid
        || output
            .formula
            .term_formula
            .terms()
            .iter()
            .any(|(_, term)| term.site.node() == output.wrapper_site.node())
        || output
            .formula
            .term_formula
            .type_entries()
            .iter()
            .any(|(_, entry)| entry.owner.node() == output.wrapper_site.node())
        || output
            .formula
            .term_formula
            .formulas()
            .iter()
            .any(|(_, formula)| {
                formula.site.node() == output.wrapper_site.node()
                    || formula
                        .terms
                        .iter()
                        .any(|term| term.node() == output.wrapper_site.node())
            })
    {
        return Err("parenthesized reserved-variable binary formula wrapper mismatch".to_owned());
    }
    Ok(())
}

fn source_type_projection_matches(
    input: &TypeExpressionInput,
    source_binding: &SourceReserveBindingInput,
    node: TypedNodeId,
    role: &str,
) -> bool {
    input.site
        == (TypedSiteRef::Role {
            node,
            role: TypeRole::new(role),
        })
        && input.source_range == source_binding.type_range
        && input.spelling == source_binding.type_spelling
        && input.head == source_binding.type_head
        && input.args.is_empty()
        && input.attributes == source_binding.type_attributes
}

fn assert_reserved_variable_builtin_type_entry(
    output: &TermFormulaInferenceOutput,
    owner: &TypedSiteRef,
    expected_id: Option<TypeEntryId>,
    expected_type: SourceReservedVariableBuiltinType,
) -> Result<NormalizedTypeId, String> {
    let (id, entry) = output
        .type_entries()
        .iter()
        .find(|(_, entry)| &entry.owner == owner)
        .ok_or_else(|| "reserved-variable equality type entry missing".to_owned())?;
    if expected_id.is_some_and(|expected| expected != id)
        || entry.expected.is_some()
        || entry.status != TypeStatus::Known
    {
        return Err("reserved-variable equality type entry mismatch".to_owned());
    }
    let TypeEntryActual::Known(actual) = entry.actual else {
        return Err("reserved-variable equality type entry is not known".to_owned());
    };
    if !normalized_type_is_reserved_builtin_type(output, actual, expected_type) {
        return Err("reserved-variable equality normalized type mismatch".to_owned());
    }
    Ok(actual)
}

fn normalized_type_is_reserved_builtin_type(
    output: &TermFormulaInferenceOutput,
    id: NormalizedTypeId,
    expected_type: SourceReservedVariableBuiltinType,
) -> bool {
    matches!(
        output.normalized_types().get(id),
        Some(normalized)
            if normalized.head == expected_type.normalized_head()
                && normalized.args.is_empty()
                && normalized.attributes.positive().is_empty()
                && normalized.attributes.negative().is_empty()
                && normalized.status == NormalizedTypeStatus::Known
    )
}

fn source_reserved_type_projection(
    binding: &SourceReserveBindingInput,
    node: TypedNodeId,
    role: &str,
) -> TypeExpressionInput {
    TypeExpressionInput::new(
        TypedSiteRef::Role {
            node,
            role: TypeRole::new(role),
        },
        binding.type_range,
        binding.type_spelling.clone(),
        binding.type_head.clone(),
    )
    .with_attributes(binding.type_attributes.clone())
}

fn source_formula_statement_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_formula_statement_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

fn source_contradiction_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_contradiction_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

fn source_builtin_binary_term_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_builtin_binary_term_formula(ast)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [
            TermInput::new(
                payload.left_site.clone(),
                context,
                payload.left_range,
                TermKind::Numeral,
            ),
            TermInput::new(
                payload.right_site.clone(),
                context,
                payload.right_range,
                TermKind::Numeral,
            ),
        ],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            payload.formula_kind,
        )
        .with_terms(vec![payload.left_site, payload.right_site])],
    );
    Some(term_formula_output_detail_keys(&output))
}

fn source_builtin_type_assertion_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_builtin_type_assertion_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

fn source_imported_predicate_functor_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_imported_predicate_functor_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

fn source_imported_attribute_assertion_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_imported_attribute_assertion_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

fn source_imported_non_empty_attribute_assertion_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output =
        source_imported_non_empty_attribute_assertion_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

fn source_set_enumeration_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_set_enumeration_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

fn source_formula_connective_quantifier_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_formula_connective_quantifier_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

fn term_formula_output_detail_keys(output: &TermFormulaInferenceOutput) -> Vec<String> {
    let mut keys = output
        .diagnostics()
        .canonical_iter()
        .map(|(_, diagnostic)| format!("type_elaboration.checker.{}", diagnostic.message_key))
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys
}

fn source_formula_statement_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_formula_statement(ast)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::Thesis,
        )
        .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload])],
    );
    Some(output)
}

fn source_contradiction_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_contradiction_formula(ast)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    Some(TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::Contradiction,
        )],
    ))
}

fn source_builtin_type_assertion_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_builtin_type_assertion_formula(ast, &module, symbols)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let asserted_type = TypeExpressionInput::new(
        payload.asserted_type_site,
        payload.asserted_type.range,
        payload.asserted_type.spelling,
        payload.asserted_type.head,
    )
    .with_attributes(payload.asserted_type.attributes);
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [TermInput::new(
            payload.subject_site.clone(),
            context,
            payload.subject_range,
            TermKind::Numeral,
        )],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::TypeAssertion,
        )
        .with_terms(vec![payload.subject_site])
        .with_asserted_type(asserted_type)],
    );
    Some(output)
}

fn source_imported_predicate_functor_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_imported_predicate_functor_formula(ast, &module, symbols)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let _predicate_symbol = payload.predicate_symbol.clone();
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [
            TermInput::new(
                payload.left_site.clone(),
                context,
                payload.left_range,
                TermKind::Numeral,
            ),
            TermInput::new(
                payload.functor_left_site.clone(),
                context,
                payload.functor_left_range,
                TermKind::Numeral,
            ),
            TermInput::new(
                payload.functor_right_site.clone(),
                context,
                payload.functor_right_range,
                TermKind::Numeral,
            ),
            TermInput::new(
                payload.functor_site.clone(),
                context,
                payload.functor_range,
                TermKind::FunctorApplication,
            )
            .with_reference(TermReference::Symbol(payload.functor_symbol)),
        ],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::PredicateApplication,
        )
        .with_terms(vec![payload.left_site, payload.functor_site])],
    );
    Some(output)
}

fn source_imported_attribute_assertion_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_imported_attribute_assertion_formula(ast, &module, symbols)?;
    source_imported_attribute_assertion_formula_output_from_payload(ast, module, symbols, payload)
}

fn source_imported_non_empty_attribute_assertion_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload =
        extract_source_imported_non_empty_attribute_assertion_formula(ast, &module, symbols)?;
    source_imported_attribute_assertion_formula_output_from_payload(ast, module, symbols, payload)
}

fn source_imported_attribute_assertion_formula_output_from_payload(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
    payload: SourceImportedAttributeAssertionFormula,
) -> Option<TermFormulaInferenceOutput> {
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let _attribute_symbol = payload.attribute_symbol.clone();
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [TermInput::new(
            payload.subject_site.clone(),
            context,
            payload.subject_range,
            TermKind::Numeral,
        )],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::AttributeAssertion,
        )
        .with_terms(vec![payload.subject_site])
        .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload])],
    );
    Some(output)
}

fn source_set_enumeration_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_set_enumeration_formula(ast)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let mut term_inputs = Vec::new();
    for (site, range) in payload.left_items.iter().chain(payload.right_items.iter()) {
        term_inputs.push(TermInput::new(
            site.clone(),
            context,
            *range,
            TermKind::Numeral,
        ));
    }
    term_inputs.push(TermInput::new(
        payload.left_site.clone(),
        context,
        payload.left_range,
        TermKind::SetEnumeration,
    ));
    term_inputs.push(TermInput::new(
        payload.right_site.clone(),
        context,
        payload.right_range,
        TermKind::SetEnumeration,
    ));
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        term_inputs,
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::Equality,
        )
        .with_terms(vec![payload.left_site, payload.right_site])],
    );
    Some(output)
}

fn source_formula_connective_quantifier_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_formula_connective_quantifier(ast, &module, symbols)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [],
        [
            FormulaInput::new(
                payload.premise_constant_site,
                context,
                payload.premise_constant_range,
                FormulaKind::Contradiction,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
            FormulaInput::new(
                payload.implication_site,
                context,
                payload.implication_range,
                FormulaKind::Implication,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
            FormulaInput::new(
                payload.quantified_site,
                context,
                payload.quantified_range,
                FormulaKind::Quantified,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingQuantifierPayload]),
            FormulaInput::new(
                payload.negation_site,
                context,
                payload.negation_range,
                FormulaKind::Negation,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
            FormulaInput::new(
                payload.body_constant_site,
                context,
                payload.body_constant_range,
                FormulaKind::Contradiction,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
        ],
    );
    Some(output)
}

fn extract_source_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_parenthesized_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

fn extract_source_parenthesized_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

fn extract_source_parenthesized_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

fn extract_source_parenthesized_heterogeneous_reserve_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

fn extract_source_right_parenthesized_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Right,
    )
}

fn extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

fn extract_source_parenthesized_reserved_object_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

fn extract_source_parenthesized_reserved_object_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

fn extract_source_reserved_object_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_distinct_reserved_object_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_distinct_reserved_object_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_reserved_object_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_distinct_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_distinct_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_distinct_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_heterogeneous_reserve_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_chained_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_two_edge_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_three_edge_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_four_edge_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_four_edge_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_three_edge_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_two_edge_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_chained_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_chained_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_two_edge_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_three_edge_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_four_edge_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_four_edge_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_four_edge_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_four_edge_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_three_edge_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_three_edge_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_three_edge_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_two_edge_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_two_edge_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_two_edge_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_chained_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_chained_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_chained_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

fn extract_source_multiple_reserve_declaration_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_RESERVE_DECLARATION_EQUALITY_CONFIG,
    )
}

fn extract_source_multiple_object_reserve_declaration_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_CONFIG,
    )
}

fn extract_source_multiple_object_reserve_declaration_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_CONFIG,
    )
}

fn extract_source_multiple_reserve_declaration_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_CONFIG,
    )
}

fn extract_source_multiple_reserve_declaration_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

fn extract_source_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

fn extract_source_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_reserved_object_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_OBJECT_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_local_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_local_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_object_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_chained_local_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_chained_local_mode_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_MODE_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_chained_local_object_mode_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_two_edge_local_mode_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_two_edge_local_mode_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_two_edge_local_object_mode_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_three_edge_local_mode_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_three_edge_local_object_mode_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_mode_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_object_mode_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_three_edge_local_mode_three_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_three_edge_local_object_mode_three_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_mode_three_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_object_mode_three_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_mode_four_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_FOUR_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_object_mode_four_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_FOUR_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_two_edge_local_object_mode_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_three_edge_local_mode_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_three_edge_local_object_mode_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_mode_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_object_mode_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_chained_local_object_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_two_edge_local_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_three_edge_local_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_four_edge_local_object_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_three_edge_local_object_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_two_edge_local_object_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_chained_local_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_chained_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_two_edge_local_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_three_edge_local_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_three_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_four_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_five_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_mode_long_chain_six_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_six_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_five_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_three_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_four_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_CONFIG,
    )
}

fn extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_four_edge_local_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

fn extract_source_reserved_variable_type_assertion_with_config(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
    config: &'static SourceReservedVariableTypeAssertionConfig,
) -> Option<SourceReservedVariableTypeAssertion> {
    if ast.nodes().iter().any(|node| {
        !(is_supported_reserved_variable_type_assertion_bridge_node(node)
            || !config.mode_definitions.is_empty()
                && matches!(
                    node.kind,
                    SurfaceNodeKind::DefinitionBlockItem
                        | SurfaceNodeKind::ModeDefinition
                        | SurfaceNodeKind::ModePattern
                        | SurfaceNodeKind::QualifiedSymbol
                        | SurfaceNodeKind::PathSegment
                ))
    }) {
        return None;
    }
    let reserve_items = surface_nodes_with_kind(ast, SurfaceNodeKind::ReserveItem);
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let mode_definitions = surface_nodes_with_kind(ast, SurfaceNodeKind::ModeDefinition);
    let ([(_, reserve_item)], [(_, theorem)]) =
        (reserve_items.as_slice(), theorem_items.as_slice())
    else {
        return None;
    };
    if mode_definitions.len() != config.mode_definitions.len()
        || !source_reserved_variable_mode_definition_is_exact(ast, config.mode_definitions)
        || reserve_item.range.end > theorem.range.start
        || subtree_has_recovery(ast, reserve_item)
        || subtree_has_recovery(ast, theorem)
        || direct_token_texts(ast, theorem).as_slice() != ["theorem", config.label, ":", ";"]
    {
        return None;
    }

    let reserve =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .ok()?;
    let [source_binding] = reserve.bridge.bindings() else {
        return None;
    };
    if source_binding.spelling != config.binding_spelling
        || !source_binding_matches_reserved_builtin_type(
            source_binding,
            config.binding_type,
            config.binding_source_mode_spelling,
            &reserve.mode_expansions,
        )
        || !source_reserved_variable_mode_expansions_are_exact(&reserve, config.mode_definitions)
    {
        return None;
    }

    let theorem_children = structural_child_ids(ast, theorem);
    let [formula_expression_id] = theorem_children.as_slice() else {
        return None;
    };
    let formula_expression = ast.node(*formula_expression_id)?;
    if !matches!(formula_expression.kind, SurfaceNodeKind::FormulaExpression) {
        return None;
    }
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    if !matches!(formula.kind, SurfaceNodeKind::IsAssertion)
        || subtree_has_recovery(ast, formula)
        || direct_token_texts(ast, formula).as_slice() != ["is"]
    {
        return None;
    }
    let assertion_children = structural_child_ids(ast, formula);
    let [subject_expression_id, asserted_type_id] = assertion_children.as_slice() else {
        return None;
    };
    let (subject_id, subject_range, subject_spelling) =
        exact_identifier_term_operand(ast, *subject_expression_id)?;
    if subject_spelling != source_binding.spelling {
        return None;
    }
    let asserted_type_node = ast.node(*asserted_type_id)?;
    if !matches!(asserted_type_node.kind, SurfaceNodeKind::TypeExpression) {
        return None;
    }
    let asserted_type =
        extract_builtin_source_type_expression(ast, asserted_type_node, &module, symbols).ok()?;
    if !source_type_expression_matches_reserved_builtin_type(
        &asserted_type,
        config.asserted_type,
        config.asserted_head_relation.source_mode_spelling(),
        &reserve.mode_expansions,
    ) || !source_reserved_variable_asserted_head_relation_is_exact(
        source_binding,
        &asserted_type.spelling,
        &asserted_type.head,
        config,
        &reserve.mode_expansions,
    ) || asserted_type.range == source_binding.type_range
    {
        return None;
    }
    let [subject_lookup_ordinal] =
        source_binding_use_ordinals(reserve.bridge.bindings(), [subject_range]).ok()?;

    Some(SourceReservedVariableTypeAssertion {
        reserve,
        config,
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
        subject_site: surface_site(subject_id),
        subject_range,
        subject_spelling,
        subject_lookup_ordinal,
        asserted_type_site: surface_site(*asserted_type_id),
        asserted_type,
    })
}

fn is_supported_reserved_variable_type_assertion_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::ReserveItem
            | SurfaceNodeKind::ReserveSegment
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::TypeHead
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::IsAssertion
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::TermReference
            | SurfaceNodeKind::Token(_)
    )
}

fn source_module_binding_env(
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

#[derive(Debug)]
struct SourceReserveHandoff {
    binding_env: BindingEnv,
    declarations: DeclarationCheckingOutput,
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

fn assemble_source_reserve_checker_handoff(
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
    })
    .map_err(|error| error.to_string())
}

fn assert_source_reserve_handoff(
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

fn assert_source_reserve_core_summary_readiness(
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

fn assert_source_reserve_core_context_readiness(
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
fn assemble_source_checker_handoff(
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

fn expected_type_elaboration_detail_keys(case: &TestCase) -> Vec<String> {
    if !case.expectation.diagnostic_payloads.is_empty() {
        return case.expectation.diagnostic_payloads.clone();
    }
    case.expectation.stable_detail_key.iter().cloned().collect()
}

fn frontend_diagnostic_code(diagnostic: &FrontendDiagnostic) -> String {
    match &diagnostic.code {
        DiagnosticCode::SourceLoad => "source_load".to_owned(),
        DiagnosticCode::Preprocess(kind) => format!("preprocess:{kind:?}"),
        DiagnosticCode::LexicalEnvironment(code) => {
            format!("lexical_environment:{code:?}")
        }
        DiagnosticCode::Lexing(kind) => format!("lexing:{kind:?}"),
        DiagnosticCode::Syntax(code) => code.to_string(),
        _ => "frontend_diagnostic".to_owned(),
    }
}

fn assertion_diagnostic_codes(case: &TestCase, diagnostics: &[FrontendDiagnostic]) -> Vec<String> {
    let syntax_codes = diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.code {
            DiagnosticCode::Syntax(code) => Some(code.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let has_non_syntax = diagnostics
        .iter()
        .any(|diagnostic| !matches!(diagnostic.code, DiagnosticCode::Syntax(_)));
    if !syntax_codes.is_empty()
        && (!has_non_syntax
            || case
                .expectation
                .tags
                .iter()
                .any(|tag| tag == ALLOW_FRONTEND_RECOVERY_DIAGNOSTICS_TAG))
    {
        syntax_codes
    } else {
        diagnostics.iter().map(frontend_diagnostic_code).collect()
    }
}

fn frontend_error_code(error: &str) -> String {
    format!("frontend_error:{error}")
}

fn type_elaboration_failure_diagnostic(
    case: &TestCase,
    result: &TypeElaborationCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "type_elaboration",
        "E-TYPE-ELABORATION-ASSERT",
        format!("type_elaboration.{}", case.id.0),
        format!(
            "type-elaboration case `{}` expected detail keys {:?} but got {:?}",
            case.id.0,
            expected_type_elaboration_detail_keys(case),
            result.actual_detail_keys
        ),
    )
}

impl fmt::Display for ParseOnlyCaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        })
    }
}

impl fmt::Display for DeclarationSymbolCaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        })
    }
}

impl fmt::Display for TypeElaborationCaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        })
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests;
