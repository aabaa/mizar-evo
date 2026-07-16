    use super::{
        ParseOnlyImportProvider, SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS,
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        extract_source_builtin_binary_term_formula,
        active_type_elaboration_cases, assemble_source_checker_handoff,
        assert_source_reserved_variable_formula_output,
        assert_source_reserved_variable_type_assertion_output,
        augment_type_elaboration_import_summaries, build_source_reserved_variable_formula_output,
        build_source_reserved_variable_type_assertion_output,
        extract_builtin_source_reserve_declarations, extract_source_builtin_type_assertion_formula,
        extract_source_chained_local_mode_asserted_head,
        extract_source_chained_local_mode_radix_asserted_head,
        extract_source_chained_local_mode_reserved_variable_equality,
        extract_source_chained_local_mode_reserved_variable_inequality,
        extract_source_chained_local_mode_reserved_variable_membership,
        extract_source_chained_local_mode_reserved_variable_type_assertion,
        extract_source_chained_local_object_mode_asserted_head,
        extract_source_chained_local_object_mode_radix_asserted_head,
        extract_source_chained_local_object_mode_reserved_variable_inequality,
        extract_source_chained_local_object_mode_reserved_variable_membership,
        extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        extract_source_contradiction_formula, extract_source_distinct_reserved_variable_equality,
        extract_source_distinct_reserved_variable_inequality,
        extract_source_distinct_reserved_variable_membership,
        extract_source_formula_connective_quantifier, extract_source_formula_statement,
        extract_source_four_edge_local_mode_four_hop_asserted_head,
        extract_source_four_edge_local_mode_radix_asserted_head,
        extract_source_four_edge_local_mode_reserved_variable_equality,
        extract_source_four_edge_local_mode_reserved_variable_inequality,
        extract_source_four_edge_local_mode_reserved_variable_membership,
        extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        extract_source_four_edge_local_mode_three_hop_asserted_head,
        extract_source_four_edge_local_mode_two_hop_asserted_head,
        extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        extract_source_four_edge_local_object_mode_radix_asserted_head,
        extract_source_four_edge_local_object_mode_reserved_variable_equality,
        extract_source_four_edge_local_object_mode_reserved_variable_inequality,
        extract_source_four_edge_local_object_mode_reserved_variable_membership,
        extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        extract_source_heterogeneous_reserve_membership,
        extract_source_imported_attribute_assertion_formula,
        extract_source_imported_non_empty_attribute_assertion_formula,
        extract_source_imported_predicate_functor_formula, extract_source_local_mode_asserted_head,
        extract_source_local_mode_long_chain_radix_asserted_head,
        extract_source_local_mode_long_chain_reserved_variable_equality,
        extract_source_local_mode_long_chain_reserved_variable_inequality,
        extract_source_local_mode_long_chain_reserved_variable_membership,
        extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        extract_source_local_mode_reserved_variable_equality,
        extract_source_local_mode_reserved_variable_inequality,
        extract_source_local_mode_reserved_variable_membership,
        extract_source_local_mode_reserved_variable_type_assertion,
        extract_source_local_object_mode_asserted_head,
        extract_source_local_object_mode_long_chain_reserved_variable_equality,
        extract_source_local_object_mode_long_chain_reserved_variable_inequality,
        extract_source_local_object_mode_long_chain_reserved_variable_membership,
        extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        extract_source_local_object_mode_reserved_variable_equality,
        extract_source_local_object_mode_reserved_variable_inequality,
        extract_source_local_object_mode_reserved_variable_membership,
        extract_source_local_object_mode_reserved_variable_type_assertion,
        extract_source_multiple_reserve_declaration_equality,
        extract_source_multiple_reserve_declaration_inequality,
        extract_source_multiple_reserve_declaration_membership,
        extract_source_reserved_variable_equality, extract_source_reserved_variable_inequality,
        extract_source_reserved_variable_membership,
        extract_source_reserved_variable_type_assertion, extract_source_set_enumeration_formula,
        extract_source_three_edge_local_mode_asserted_head,
        extract_source_three_edge_local_mode_radix_asserted_head,
        extract_source_three_edge_local_mode_reserved_variable_equality,
        extract_source_three_edge_local_mode_reserved_variable_inequality,
        extract_source_three_edge_local_mode_reserved_variable_membership,
        extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        extract_source_three_edge_local_mode_three_hop_asserted_head,
        extract_source_three_edge_local_mode_two_hop_asserted_head,
        extract_source_three_edge_local_object_mode_asserted_head,
        extract_source_three_edge_local_object_mode_radix_asserted_head,
        extract_source_three_edge_local_object_mode_reserved_variable_equality,
        extract_source_three_edge_local_object_mode_reserved_variable_inequality,
        extract_source_three_edge_local_object_mode_reserved_variable_membership,
        extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        extract_source_two_edge_local_mode_asserted_head,
        extract_source_two_edge_local_mode_radix_asserted_head,
        extract_source_two_edge_local_mode_reserved_variable_equality,
        extract_source_two_edge_local_mode_reserved_variable_inequality,
        extract_source_two_edge_local_mode_reserved_variable_membership,
        extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        extract_source_two_edge_local_mode_two_hop_asserted_head,
        extract_source_two_edge_local_object_mode_asserted_head,
        extract_source_two_edge_local_object_mode_radix_asserted_head,
        extract_source_two_edge_local_object_mode_reserved_variable_equality,
        extract_source_two_edge_local_object_mode_reserved_variable_inequality,
        extract_source_two_edge_local_object_mode_reserved_variable_membership,
        extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        extract_source_two_edge_local_object_mode_two_hop_asserted_head, resolve_visible_attribute,
        resolve_visible_type_head, resolver_symbol_collection, run_frontend,
        source_builtin_type_assertion_formula_output,
        source_chained_local_mode_asserted_head_output,
        source_chained_local_mode_radix_asserted_head_output,
        source_chained_local_mode_reserved_variable_equality_output,
        source_chained_local_mode_reserved_variable_inequality_output,
        source_chained_local_mode_reserved_variable_membership_output,
        source_chained_local_mode_reserved_variable_type_assertion_output,
        source_chained_local_object_mode_asserted_head_output,
        source_chained_local_object_mode_radix_asserted_head_output,
        source_chained_local_object_mode_reserved_variable_inequality_output,
        source_chained_local_object_mode_reserved_variable_membership_output,
        source_chained_local_object_mode_reserved_variable_type_assertion_output,
        source_contradiction_formula_output, source_distinct_reserved_variable_equality_output,
        source_distinct_reserved_variable_inequality_output,
        source_distinct_reserved_variable_membership_output,
        source_formula_connective_quantifier_output, source_formula_statement_output,
        source_four_edge_local_mode_four_hop_asserted_head_output,
        source_four_edge_local_mode_radix_asserted_head_output,
        source_four_edge_local_mode_reserved_variable_equality_output,
        source_four_edge_local_mode_reserved_variable_inequality_output,
        source_four_edge_local_mode_reserved_variable_membership_output,
        source_four_edge_local_mode_reserved_variable_type_assertion_output,
        source_four_edge_local_mode_three_hop_asserted_head_output,
        source_four_edge_local_mode_two_hop_asserted_head_output,
        source_four_edge_local_object_mode_four_hop_asserted_head_output,
        source_four_edge_local_object_mode_radix_asserted_head_output,
        source_four_edge_local_object_mode_reserved_variable_equality_output,
        source_four_edge_local_object_mode_reserved_variable_inequality_output,
        source_four_edge_local_object_mode_reserved_variable_membership_output,
        source_four_edge_local_object_mode_reserved_variable_type_assertion_output,
        source_four_edge_local_object_mode_three_hop_asserted_head_output,
        source_four_edge_local_object_mode_two_hop_asserted_head_output,
        source_heterogeneous_reserve_membership_output,
        source_imported_attribute_assertion_formula_output,
        source_imported_non_empty_attribute_assertion_formula_output,
        source_imported_predicate_functor_formula_output, source_local_mode_asserted_head_output,
        source_local_mode_long_chain_reserved_variable_equality_output,
        source_local_mode_long_chain_reserved_variable_inequality_output,
        source_local_mode_long_chain_reserved_variable_membership_output,
        source_local_mode_long_chain_reserved_variable_type_assertion_output,
        source_local_mode_reserved_variable_equality_output,
        source_local_mode_reserved_variable_inequality_output,
        source_local_mode_reserved_variable_membership_output,
        source_local_mode_reserved_variable_type_assertion_output,
        source_local_object_mode_asserted_head_output,
        source_local_object_mode_long_chain_reserved_variable_equality_output,
        source_local_object_mode_long_chain_reserved_variable_inequality_output,
        source_local_object_mode_long_chain_reserved_variable_membership_output,
        source_local_object_mode_long_chain_reserved_variable_type_assertion_output,
        source_local_object_mode_reserved_variable_equality_output,
        source_local_object_mode_reserved_variable_inequality_output,
        source_local_object_mode_reserved_variable_membership_output,
        source_local_object_mode_reserved_variable_type_assertion_output,
        source_mode_symbol_spelling, source_multiple_reserve_declaration_equality_output,
        source_multiple_reserve_declaration_inequality_output,
        source_multiple_reserve_declaration_membership_output,
        source_reserved_variable_equality_output,
        source_reserved_variable_formula_output_detail_keys,
        source_reserved_variable_formula_result_detail_keys,
        source_reserved_variable_inequality_output, source_reserved_variable_membership_output,
        source_reserved_variable_type_assertion_output,
        source_reserved_variable_type_assertion_result_detail_keys,
        source_set_enumeration_formula_output,
        source_three_edge_local_mode_radix_asserted_head_output,
        source_three_edge_local_mode_reserved_variable_equality_output,
        source_three_edge_local_mode_reserved_variable_inequality_output,
        source_three_edge_local_mode_reserved_variable_membership_output,
        source_three_edge_local_mode_reserved_variable_type_assertion_output,
        source_three_edge_local_mode_three_hop_asserted_head_output,
        source_three_edge_local_mode_two_hop_asserted_head_output,
        source_three_edge_local_object_mode_radix_asserted_head_output,
        source_three_edge_local_object_mode_reserved_variable_equality_output,
        source_three_edge_local_object_mode_reserved_variable_inequality_output,
        source_three_edge_local_object_mode_reserved_variable_membership_output,
        source_three_edge_local_object_mode_reserved_variable_type_assertion_output,
        source_three_edge_local_object_mode_three_hop_asserted_head_output,
        source_three_edge_local_object_mode_two_hop_asserted_head_output,
        source_two_edge_local_mode_asserted_head_output,
        source_two_edge_local_mode_radix_asserted_head_output,
        source_two_edge_local_mode_reserved_variable_equality_output,
        source_two_edge_local_mode_reserved_variable_inequality_output,
        source_two_edge_local_mode_reserved_variable_membership_output,
        source_two_edge_local_mode_reserved_variable_type_assertion_output,
        source_two_edge_local_mode_two_hop_asserted_head_output,
        source_two_edge_local_object_mode_asserted_head_output,
        source_two_edge_local_object_mode_radix_asserted_head_output,
        source_two_edge_local_object_mode_reserved_variable_equality_output,
        source_two_edge_local_object_mode_reserved_variable_inequality_output,
        source_two_edge_local_object_mode_reserved_variable_membership_output,
        source_two_edge_local_object_mode_reserved_variable_type_assertion_output,
        source_two_edge_local_object_mode_two_hop_asserted_head_output,
        source_type_elaboration_detail_keys, surface_nodes_with_kind, surface_site,
    };
    use crate::harness::{DiscoveryConfig, TestProfile, ValidationMode, build_test_plan};
    use mizar_checker::binding_env::{BindingContextId, BindingId};
    use mizar_checker::resolved_typed_ast::{ResolvedTypedNodeId, ResolvedTypedNodeKind};
    use mizar_checker::type_checker::{
        AttributePolarity, FormulaDeferredReason, FormulaKind, FormulaStatus, TermKind,
        TermReference, TermStatus, TypeHeadInput, TypeHeadRef,
    };
    use mizar_checker::typed_ast::{LocalTypeContextId, TypeStatus, TypedSiteRef};
    use mizar_core::elaborator::ResolvedTypedAstSummary;
    use mizar_frontend::lexical_env::{
        ExportedOperatorAssociativity, ExportedOperatorFixity, ExportedOperatorMetadata,
        LexicalEnvironmentRequest, LexicalSummaryProvider, UserSymbolKind,
    };
    use mizar_frontend::preprocess::{ImportStub, ImportStubPath};
    use mizar_resolve::env::{
        ContributionKind, ExportStatus, NamespacePath, SymbolEntry, SymbolEnv, SymbolEnvIndexes,
        SymbolKind, Visibility,
    };
    use mizar_resolve::resolved_ast::{
        FullyQualifiedName, LocalSymbolId, ModuleId as ResolverModuleId, SemanticOrigin,
        SymbolId as ResolverSymbolId,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceAnchor, SourceId, SourceRange,
    };
    use mizar_syntax::recovery::SyntaxRecoveryKind;
    use mizar_syntax::{
        SurfaceAst, SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceFormulaBinaryOperator,
        SurfaceFormulaConnective, SurfaceFormulaConstant, SurfaceFormulaPrefixOperator,
        SurfaceNodeKind, SurfaceQuantifierKind, SurfaceTokenKind,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;
    use std::sync::Arc;

    fn source_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_local_symbol_env(module, "empty", SymbolKind::Attribute)
    }

    fn source_mode_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_local_symbol_env(module, "Mode", SymbolKind::Mode)
    }

    fn source_mode_chain_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_local_symbols_env(
            module,
            &[
                ("empty", SymbolKind::Attribute),
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
            ],
        )
    }

    fn source_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_local_symbol_env(module, "Struct", SymbolKind::Structure)
    }

    fn source_mode_attribute_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_symbol_pair_env(module, "Mode", SymbolKind::Mode)
    }

    fn source_structure_attribute_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_symbol_pair_env(module, "Struct", SymbolKind::Structure)
    }

    fn imported_attribute_mode_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_attribute_symbol_head_env(module, "Mode", SymbolKind::Mode)
    }

    fn imported_attribute_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_attribute_symbol_head_env(module, "Struct", SymbolKind::Structure)
    }

    fn imported_attribute_symbol_head_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(197);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        let local_symbol = ResolverSymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
            FullyQualifiedName::new(format!("{}::{spelling}/0", module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                local_symbol,
                kind,
                NamespacePath::new(module.path().as_str()),
                spelling,
                SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 0, 1)),
                    Vec::new(),
                ),
                local_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let attribute = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new("Attribute/empty/0"),
            FullyQualifiedName::new(format!("{}::empty/0", imported_module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                attribute,
                SymbolKind::Attribute,
                NamespacePath::new(module.path().as_str()),
                "empty",
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 1, 2)),
                    Vec::new(),
                ),
                imported_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_attribute_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        let source = source_id(198);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 3)),
        );
        for (ordinal, (spelling, kind)) in [
            ("Struct", SymbolKind::Structure),
            ("empty", SymbolKind::Attribute),
            ("empty", SymbolKind::Attribute),
        ]
        .into_iter()
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_attribute_mode_chain_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        let source = source_id(200);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 2)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 2, 3)),
        );
        for (ordinal, spelling) in ["A", "B"].into_iter().enumerate() {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("Mode/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    SymbolKind::Mode,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    local_contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        let attribute = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new("Attribute/empty/0"),
            FullyQualifiedName::new(format!("{}::empty/0", imported_module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                attribute,
                SymbolKind::Attribute,
                NamespacePath::new(module.path().as_str()),
                "empty",
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 2, 3)),
                    Vec::new(),
                ),
                imported_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_attribute_mode_chain_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        let source = source_id(201);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 4)),
        );
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
        ]
        .into_iter()
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_attributed_mode_chain_symbol_env(
        module: ResolverModuleId,
        ambiguous_mode: &'static str,
    ) -> SymbolEnv {
        let source = source_id(186);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 5)),
        );
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
        ]
        .into_iter()
        .chain(std::iter::once((ambiguous_mode, SymbolKind::Mode)))
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_attributed_structure_chain_symbol_env(
        module: ResolverModuleId,
        ambiguous_spelling: &'static str,
    ) -> SymbolEnv {
        let source = source_id(188);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 6)),
        );
        let ambiguous_kind = if ambiguous_spelling == "Struct" {
            SymbolKind::Structure
        } else {
            SymbolKind::Mode
        };
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
            ("Struct", SymbolKind::Structure),
        ]
        .into_iter()
        .chain(std::iter::once((ambiguous_spelling, ambiguous_kind)))
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn source_symbol_pair_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(193);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 10)),
        );
        for (ordinal, (spelling, kind)) in [("empty", SymbolKind::Attribute), (spelling, kind)]
            .into_iter()
            .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_mode_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        ambiguous_symbol_env(module, "Mode", SymbolKind::Mode)
    }

    fn ambiguous_mode_chain_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        let source = source_id(189);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 5)),
        );
        for (ordinal, spelling) in ["A", "A", "B"].into_iter().enumerate() {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("Mode/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    SymbolKind::Mode,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        ambiguous_symbol_env(module, "Struct", SymbolKind::Structure)
    }

    fn ambiguous_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(192);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 5)),
        );
        for ordinal in 0..2 {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn source_local_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        source_local_symbols_env(module, &[(spelling, kind)])
    }

    fn source_local_symbols_env(
        module: ResolverModuleId,
        symbols: &[(&'static str, SymbolKind)],
    ) -> SymbolEnv {
        let source = source_id(190);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        for (ordinal, (spelling, kind)) in symbols.iter().copied().enumerate() {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            )
        }
        SymbolEnv::new(module, indexes)
    }

    fn source_local_and_imported_symbols_env(
        module: ResolverModuleId,
        local_symbols: &[(&'static str, SymbolKind)],
        imported_symbols: &[(&'static str, SymbolKind)],
    ) -> SymbolEnv {
        let source = source_id(202);
        let imported_module =
            ResolverModuleId::new(module.package().clone(), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        for (ordinal, (spelling, kind)) in local_symbols.iter().copied().enumerate() {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/local/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/local/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, 0, 1)),
                        vec![ordinal as u32],
                    ),
                    local_contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        for (ordinal, (spelling, kind)) in imported_symbols.iter().copied().enumerate() {
            let symbol = ResolverSymbolId::new(
                imported_module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/imported/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/imported/{ordinal}",
                    imported_module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        imported_module.clone(),
                        SourceAnchor::Range(range(source, 1, 2)),
                        vec![ordinal as u32],
                    ),
                    imported_contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_attribute_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_symbol_env(module, "empty", SymbolKind::Attribute)
    }

    fn imported_empty_fixture_attribute_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_parser_fixture_symbol_env(module, "empty", SymbolKind::Attribute)
    }

    fn imported_fixture_attribute_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_parser_fixture_symbol_env(module, "TypeCaseAttr", SymbolKind::Attribute)
    }

    fn imported_empty_fixture_wrong_kind_env(module: ResolverModuleId) -> SymbolEnv {
        imported_parser_fixture_symbol_env(module, "empty", SymbolKind::Mode)
    }

    fn ambiguous_imported_attribute_assertion_env(module: ResolverModuleId) -> SymbolEnv {
        term_formula_symbol_env(
            module,
            &[],
            &[
                ("empty", SymbolKind::Attribute),
                ("empty", SymbolKind::Attribute),
            ],
        )
    }

    fn imported_mode_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_symbol_env(module, "Mode", SymbolKind::Mode)
    }

    fn local_and_imported_mode_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
    ) -> SymbolEnv {
        local_and_imported_parser_fixture_symbol_env(module, spelling, SymbolKind::Mode)
    }

    fn local_and_imported_structure_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
    ) -> SymbolEnv {
        local_and_imported_parser_fixture_symbol_env(module, spelling, SymbolKind::Structure)
    }

    fn local_and_imported_attribute_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
    ) -> SymbolEnv {
        local_and_imported_parser_fixture_symbol_env(module, spelling, SymbolKind::Attribute)
    }

    fn local_structure_and_imported_fixture_attribute_symbol_env(
        module: ResolverModuleId,
        structure_spelling: &'static str,
        attribute_spelling: &'static str,
    ) -> SymbolEnv {
        let source = source_id(199);
        let imported_module = ResolverModuleId::new(
            module.package().clone(),
            ModulePath::new("parser.type_fixtures"),
        );
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        let structure = ResolverSymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("Structure/{structure_spelling}/0")),
            FullyQualifiedName::new(format!(
                "{}::{structure_spelling}/0",
                module.path().as_str()
            )),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                structure,
                SymbolKind::Structure,
                NamespacePath::new(module.path().as_str()),
                structure_spelling,
                SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 0, 1)),
                    Vec::new(),
                ),
                local_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let attribute = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new(format!("Attribute/{attribute_spelling}/0")),
            FullyQualifiedName::new(format!(
                "{}::{attribute_spelling}/0",
                imported_module.path().as_str()
            )),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                attribute,
                SymbolKind::Attribute,
                NamespacePath::new(module.path().as_str()),
                attribute_spelling,
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 1, 2)),
                    Vec::new(),
                ),
                imported_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
    }

    fn local_and_imported_parser_fixture_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(198);
        let imported_module = ResolverModuleId::new(
            module.package().clone(),
            ModulePath::new("parser.type_fixtures"),
        );
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        for (ordinal, (symbol_module, contribution)) in [
            (module.clone(), local_contribution),
            (imported_module, imported_contribution),
        ]
        .into_iter()
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                symbol_module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    symbol_module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        symbol_module,
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_mode_chain_symbol_env(
        module: ResolverModuleId,
        imported_mode: &'static str,
    ) -> SymbolEnv {
        let source = source_id(185);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 2)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 2, 3)),
        );
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
        ]
        .into_iter()
        .enumerate()
        {
            let is_imported = kind == SymbolKind::Mode && spelling == imported_mode;
            let symbol_module = if is_imported {
                imported_module.clone()
            } else {
                module.clone()
            };
            let contribution = if is_imported {
                imported_contribution
            } else {
                local_contribution
            };
            let symbol = ResolverSymbolId::new(
                symbol_module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", symbol_module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        symbol_module,
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_structure_mode_chain_symbol_env(
        module: ResolverModuleId,
        imported_spelling: &'static str,
    ) -> SymbolEnv {
        let source = source_id(187);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 2)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 2, 3)),
        );
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
            ("Struct", SymbolKind::Structure),
        ]
        .into_iter()
        .enumerate()
        {
            let is_imported = spelling == imported_spelling;
            let symbol_module = if is_imported {
                imported_module.clone()
            } else {
                module.clone()
            };
            let contribution = if is_imported {
                imported_contribution
            } else {
                local_contribution
            };
            let symbol = ResolverSymbolId::new(
                symbol_module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", symbol_module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        symbol_module,
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_symbol_env(module, "Struct", SymbolKind::Structure)
    }

    fn imported_fixture_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_parser_fixture_symbol_env(module, "R", SymbolKind::Structure)
    }

    fn imported_parser_fixture_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(199);
        let imported_module = ResolverModuleId::new(
            module.package().clone(),
            ModulePath::new("parser.type_fixtures"),
        );
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let symbol = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
            FullyQualifiedName::new(format!("{}::{spelling}/0", imported_module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                symbol,
                kind,
                NamespacePath::new(module.path().as_str()),
                spelling,
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 0, 1)),
                    Vec::new(),
                ),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
    }

    fn imported_predicate_functor_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        term_formula_symbol_env(
            module,
            &[],
            &[
                ("divides", SymbolKind::Predicate),
                ("++", SymbolKind::Functor),
            ],
        )
    }

    fn imported_predicate_functor_local_contribution_env(module: ResolverModuleId) -> SymbolEnv {
        term_formula_symbol_env_with_imported_contribution(
            module,
            &[],
            &[
                ("divides", SymbolKind::Predicate),
                ("++", SymbolKind::Functor),
            ],
            false,
        )
    }

    fn source_local_predicate_and_imported_functor_env(module: ResolverModuleId) -> SymbolEnv {
        term_formula_symbol_env(
            module,
            &[("divides", SymbolKind::Predicate)],
            &[
                ("divides", SymbolKind::Predicate),
                ("++", SymbolKind::Functor),
            ],
        )
    }

    fn source_local_functor_and_imported_predicate_env(module: ResolverModuleId) -> SymbolEnv {
        term_formula_symbol_env(
            module,
            &[("++", SymbolKind::Functor)],
            &[
                ("divides", SymbolKind::Predicate),
                ("++", SymbolKind::Functor),
            ],
        )
    }

    fn imported_predicate_wrong_functor_kind_env(module: ResolverModuleId) -> SymbolEnv {
        term_formula_symbol_env(
            module,
            &[],
            &[
                ("divides", SymbolKind::Predicate),
                ("++", SymbolKind::Predicate),
            ],
        )
    }

    fn imported_functor_wrong_predicate_kind_env(module: ResolverModuleId) -> SymbolEnv {
        term_formula_symbol_env(
            module,
            &[],
            &[
                ("divides", SymbolKind::Functor),
                ("++", SymbolKind::Functor),
            ],
        )
    }

    fn ambiguous_imported_predicate_functor_env(
        module: ResolverModuleId,
        spelling: &'static str,
    ) -> SymbolEnv {
        match spelling {
            "divides" => term_formula_symbol_env(
                module,
                &[],
                &[
                    ("divides", SymbolKind::Predicate),
                    ("divides", SymbolKind::Predicate),
                    ("++", SymbolKind::Functor),
                ],
            ),
            "++" => term_formula_symbol_env(
                module,
                &[],
                &[
                    ("divides", SymbolKind::Predicate),
                    ("++", SymbolKind::Functor),
                    ("++", SymbolKind::Functor),
                ],
            ),
            _ => term_formula_symbol_env(module, &[], &[]),
        }
    }

    fn term_formula_symbol_env(
        module: ResolverModuleId,
        local_symbols: &[(&'static str, SymbolKind)],
        imported_symbols: &[(&'static str, SymbolKind)],
    ) -> SymbolEnv {
        term_formula_symbol_env_with_imported_contribution(
            module,
            local_symbols,
            imported_symbols,
            true,
        )
    }

    fn term_formula_symbol_env_with_imported_contribution(
        module: ResolverModuleId,
        local_symbols: &[(&'static str, SymbolKind)],
        imported_symbols: &[(&'static str, SymbolKind)],
        imported_source: bool,
    ) -> SymbolEnv {
        let source = source_id(201);
        let imported_module = ResolverModuleId::new(
            module.package().clone(),
            ModulePath::new("parser.type_fixtures"),
        );
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            if imported_source {
                ContributionKind::ImportedSource { source_id: source }
            } else {
                ContributionKind::LocalSource { source_id: source }
            },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        for (ordinal, (spelling, kind)) in local_symbols.iter().copied().enumerate() {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("local:{kind:?}:{spelling}:{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/local/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol.clone(),
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, 0, 1)),
                        vec![ordinal as u32],
                    ),
                    local_contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
            indexes.contributions.add_symbol(local_contribution, symbol);
        }
        for (ordinal, (spelling, kind)) in imported_symbols.iter().copied().enumerate() {
            let symbol = ResolverSymbolId::new(
                imported_module.clone(),
                LocalSymbolId::new(format!("imported:{kind:?}:{spelling}:{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/imported/{ordinal}",
                    imported_module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol.clone(),
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        imported_module.clone(),
                        SourceAnchor::Range(range(source, 1, 2)),
                        vec![ordinal as u32],
                    ),
                    imported_contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
            indexes
                .contributions
                .add_symbol(imported_contribution, symbol);
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(191);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let symbol = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
            FullyQualifiedName::new(format!("{}::{spelling}/0", imported_module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                symbol,
                kind,
                NamespacePath::new(module.path().as_str()),
                spelling,
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 0, 1)),
                    Vec::new(),
                ),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
    }

    fn import_stub(source_id: SourceId, spelling: &str, start: usize, end: usize) -> ImportStub {
        let span = range(source_id, start, end);
        ImportStub {
            path: ImportStubPath {
                spelling: Arc::from(spelling),
                relative: None,
                components: vec![Arc::from(spelling)],
                source_segments: vec![span],
                span,
            },
            alias: None,
            span,
        }
    }

    #[derive(Debug, Clone)]
    struct ReserveItemSpec {
        names: Vec<&'static str>,
        type_shape: ReserveTypeShape,
    }

    #[derive(Debug, Clone, Copy)]
    struct ModeDefinitionSpec {
        pattern: &'static str,
        label: Option<&'static str>,
        rhs_shape: ReserveTypeShape,
        local_context: bool,
        parameterized_pattern: bool,
        recovered: bool,
    }

    #[derive(Debug, Clone, Copy)]
    enum TypeAttributeShape {
        Positive,
        Negative,
        DuplicateNegative,
        MixedPolarity,
    }

    #[derive(Debug, Clone, Copy)]
    enum ReserveTypeShape {
        Builtin(&'static str),
        NonBuiltin(&'static str),
        QualifiedSymbol(&'static str),
        QualifiedSymbolWithArgs(&'static str),
        AttributedSetWithNamedAttribute(&'static str),
        AttributedSetWithNegativeNamedAttribute(&'static str),
        AttributedSetWithDuplicateNamedAttribute(&'static str),
        AttributedSetWithMixedPolarityNamedAttribute(&'static str),
        AttributedObjectWithNamedAttribute(&'static str),
        AttributedObjectWithDuplicateNamedAttribute(&'static str),
        AttributedQualifiedSymbolWithNamedAttribute(&'static str, &'static str),
        AttributedQualifiedSymbol(&'static str),
        AttributedQualifiedSymbolWithAttributeArgs(&'static str),
        QualifiedAttributeQualifiedSymbol(&'static str),
        AttributedSet,
        AttributedSetWithAttributeArgs,
        AttributedObject,
    }

    fn reserve_item(names: Vec<&'static str>, type_shape: ReserveTypeShape) -> ReserveItemSpec {
        ReserveItemSpec { names, type_shape }
    }

    const fn mode_definition(
        pattern: &'static str,
        rhs_shape: ReserveTypeShape,
    ) -> ModeDefinitionSpec {
        ModeDefinitionSpec {
            pattern,
            label: None,
            rhs_shape,
            local_context: false,
            parameterized_pattern: false,
            recovered: false,
        }
    }

    const fn contextual_mode_definition(
        pattern: &'static str,
        rhs_shape: ReserveTypeShape,
    ) -> ModeDefinitionSpec {
        ModeDefinitionSpec {
            pattern,
            label: None,
            rhs_shape,
            local_context: true,
            parameterized_pattern: false,
            recovered: false,
        }
    }

    const fn parameterized_mode_definition(
        pattern: &'static str,
        rhs_shape: ReserveTypeShape,
    ) -> ModeDefinitionSpec {
        ModeDefinitionSpec {
            pattern,
            label: None,
            rhs_shape,
            local_context: false,
            parameterized_pattern: true,
            recovered: false,
        }
    }

    const fn recovered_mode_definition(
        pattern: &'static str,
        rhs_shape: ReserveTypeShape,
    ) -> ModeDefinitionSpec {
        ModeDefinitionSpec {
            pattern,
            label: None,
            rhs_shape,
            local_context: false,
            parameterized_pattern: false,
            recovered: true,
        }
    }

    const fn mode_definition_with_label(
        pattern: &'static str,
        label: &'static str,
        rhs_shape: ReserveTypeShape,
    ) -> ModeDefinitionSpec {
        ModeDefinitionSpec {
            pattern,
            label: Some(label),
            rhs_shape,
            local_context: false,
            parameterized_pattern: false,
            recovered: false,
        }
    }

    fn mode_chain_reserve_ast(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        mode_chain_reserve_ast_with_order(source_id, [], modes, [], items)
    }

    fn mode_then_reserve_identifier_binary_theorem_ast(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
        theorem: IdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut root_children = Vec::new();
        let mut offset = 0;
        for mode in modes {
            root_children.push(add_mode_definition_item(
                &mut builder,
                source_id,
                &mut offset,
                mode,
            ));
        }
        root_children.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        root_children.push(add_identifier_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            theorem,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
        theorem: ParenthesizedIdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut root_children = Vec::new();
        let mut offset = 0;
        for mode in modes {
            root_children.push(add_mode_definition_item(
                &mut builder,
                source_id,
                &mut offset,
                mode,
            ));
        }
        root_children.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        root_children.push(add_parenthesized_identifier_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            theorem,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn modes_then_empty_definition_reserve_parenthesized_identifier_binary_theorem_ast(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
        theorem: ParenthesizedIdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = modes
            .into_iter()
            .map(|mode| add_mode_definition_item(&mut builder, source_id, &mut offset, mode))
            .collect::<Vec<_>>();
        root_children.push(add_empty_definition_item(
            &mut builder,
            source_id,
            &mut offset,
        ));
        root_children.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        root_children.push(add_parenthesized_identifier_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            theorem,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
        theorem: IdentifierTypeAssertionTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut root_children = Vec::new();
        let mut offset = 0;
        for mode in modes {
            root_children.push(add_mode_definition_item(
                &mut builder,
                source_id,
                &mut offset,
                mode,
            ));
        }
        root_children.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        root_children.push(add_type_assertion_theorem_item_with_status(
            &mut builder,
            source_id,
            &mut offset,
            theorem,
            true,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
        theorem: IdentifierTypeAssertionTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = modes
            .into_iter()
            .map(|mode| add_mode_definition_item(&mut builder, source_id, &mut offset, mode))
            .collect::<Vec<_>>();
        root_children.push(add_empty_definition_item(
            &mut builder,
            source_id,
            &mut offset,
        ));
        root_children.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        root_children.push(add_type_assertion_theorem_item_with_status(
            &mut builder,
            source_id,
            &mut offset,
            theorem,
            true,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_mode_identifier_type_assertion_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        mode: ModeDefinitionSpec,
        theorem: IdentifierTypeAssertionTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_mode_definition_item(
            &mut builder,
            source_id,
            &mut offset,
            mode,
        ));
        root_children.push(add_type_assertion_theorem_item_with_status(
            &mut builder,
            source_id,
            &mut offset,
            theorem,
            true,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_mode_identifier_binary_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        mode: ModeDefinitionSpec,
        theorem: IdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_mode_definition_item(
            &mut builder,
            source_id,
            &mut offset,
            mode,
        ));
        root_children.push(add_identifier_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            theorem,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
        theorem: IdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = modes
            .into_iter()
            .map(|mode| add_mode_definition_item(&mut builder, source_id, &mut offset, mode))
            .collect::<Vec<_>>();
        root_children.push(add_empty_definition_item(
            &mut builder,
            source_id,
            &mut offset,
        ));
        root_children.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        root_children.push(add_identifier_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            theorem,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn mode_chain_reserve_ast_with_structures(
        source_id: SourceId,
        structures: impl IntoIterator<Item = &'static str>,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        mode_chain_reserve_ast_with_order(source_id, structures, modes, [], items)
    }

    fn mode_chain_reserve_ast_with_trailing_structures(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        structures: impl IntoIterator<Item = &'static str>,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        mode_chain_reserve_ast_with_order(source_id, [], modes, structures, items)
    }

    fn mode_chain_reserve_ast_with_order(
        source_id: SourceId,
        leading_structures: impl IntoIterator<Item = &'static str>,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        trailing_structures: impl IntoIterator<Item = &'static str>,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut root_children = Vec::new();
        let mut offset = 0;
        for structure in leading_structures {
            root_children.push(add_structure_definition_item(
                &mut builder,
                source_id,
                &mut offset,
                structure,
            ));
        }
        for mode in modes {
            root_children.push(add_mode_definition_item(
                &mut builder,
                source_id,
                &mut offset,
                mode,
            ));
        }
        for structure in trailing_structures {
            root_children.push(add_structure_definition_item(
                &mut builder,
                source_id,
                &mut offset,
                structure,
            ));
        }
        root_children.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn reserve_ast(source_id: SourceId, items: Vec<ReserveItemSpec>) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn imported_reserve_ast(
        source_id: SourceId,
        imports: &[&str],
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut compilation_items = imports
            .iter()
            .map(|import| add_import_item(&mut builder, source_id, &mut offset, import))
            .collect::<Vec<_>>();
        compilation_items.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        finish_compilation_ast(builder, source_id, compilation_items)
    }

    fn imported_reserve_ast_with_extra_definition(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut compilation_items = vec![add_import_item(
            &mut builder,
            source_id,
            &mut offset,
            "parser.type_fixtures",
        )];
        compilation_items.push(add_mode_definition_item(
            &mut builder,
            source_id,
            &mut offset,
            mode_definition("UnrelatedMode", ReserveTypeShape::Builtin("set")),
        ));
        compilation_items.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        finish_compilation_ast(builder, source_id, compilation_items)
    }

    fn imported_reserve_ast_with_extra_recovery(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut compilation_items = vec![add_import_item(
            &mut builder,
            source_id,
            &mut offset,
            "parser.type_fixtures",
        )];
        let recovery_start = offset;
        offset += 1;
        compilation_items.push(builder.add_recovery(
            SyntaxRecoveryKind::MissingTerm,
            range(source_id, recovery_start, offset),
            Vec::new(),
        ));
        compilation_items.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        finish_compilation_ast(builder, source_id, compilation_items)
    }

    fn builtin_equality_theorem_ast(
        source_id: SourceId,
        label: &str,
        left: &str,
        right: &str,
    ) -> SurfaceAst {
        builtin_binary_theorem_ast(source_id, label, left, "=", right)
    }

    fn builtin_binary_theorem_ast(
        source_id: SourceId,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
    ) -> SurfaceAst {
        builtin_binary_theorem_ast_with_corruption(
            source_id,
            label,
            left,
            operator,
            right,
            BuiltinBinaryTheoremCorruption::default(),
        )
    }

    fn builtin_binary_theorem_ast_with_corruption(
        source_id: SourceId,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
        corruption: BuiltinBinaryTheoremCorruption,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem = add_builtin_binary_theorem_item_with_status(
            &mut builder,
            source_id,
            &mut offset,
            None,
            BuiltinBinaryTheoremShape {
                label,
                left,
                operator,
                right,
                corruption,
            },
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    fn double_builtin_binary_theorem_ast(
        source_id: SourceId,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorems = [
            add_builtin_binary_theorem_item(
                &mut builder,
                source_id,
                &mut offset,
                label,
                left,
                operator,
                right,
            ),
            add_builtin_binary_theorem_item(
                &mut builder,
                source_id,
                &mut offset,
                label,
                left,
                operator,
                right,
            ),
        ];
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            theorems.to_vec(),
        );
        builder.finish(Some(root), None)
    }

    fn builtin_binary_theorem_ast_with_status(
        source_id: SourceId,
        status: &str,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let shape = BuiltinBinaryTheoremShape {
            label,
            left,
            operator,
            right,
            corruption: BuiltinBinaryTheoremCorruption::default(),
        };
        let theorem = add_builtin_binary_theorem_item_with_status(
            &mut builder,
            source_id,
            &mut offset,
            Some(status),
            shape,
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    fn builtin_type_assertion_theorem_ast(
        source_id: SourceId,
        label: &str,
        subject: &str,
        type_shape: ReserveTypeShape,
    ) -> SurfaceAst {
        builtin_type_assertion_theorem_ast_with_corruption(
            source_id,
            label,
            subject,
            type_shape,
            BuiltinTypeAssertionTheoremCorruption::default(),
        )
    }

    fn builtin_type_assertion_theorem_ast_with_corruption(
        source_id: SourceId,
        label: &str,
        subject: &str,
        type_shape: ReserveTypeShape,
        corruption: BuiltinTypeAssertionTheoremCorruption,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem_count = if corruption.duplicate_theorem { 2 } else { 1 };
        let theorems = (0..theorem_count)
            .map(|_| {
                add_builtin_type_assertion_theorem_item_with_corruption(
                    &mut builder,
                    source_id,
                    &mut offset,
                    label,
                    subject,
                    type_shape,
                    corruption,
                )
            })
            .collect::<Vec<_>>();
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            theorems,
        );
        builder.finish(Some(root), None)
    }

    fn builtin_type_assertion_theorem_ast_with_status(
        source_id: SourceId,
        status: &str,
        label: &str,
        subject: &str,
        type_shape: ReserveTypeShape,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem = add_builtin_type_assertion_theorem_item_with_status(
            &mut builder,
            source_id,
            &mut offset,
            Some(status),
            label,
            subject,
            type_shape,
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_builtin_equality_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        label: &str,
        left: &str,
        right: &str,
    ) -> SurfaceAst {
        reserve_then_builtin_binary_theorem_ast(source_id, items, label, left, "=", right)
    }

    fn reserve_then_identifier_equality_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        label: &str,
        left: &str,
        right: &str,
    ) -> SurfaceAst {
        reserve_then_identifier_binary_theorem_ast(source_id, items, label, left, "=", right)
    }

    #[derive(Clone, Copy)]
    enum ParenthesizedIdentifierOperandShape<'a> {
        Direct(&'a str),
        Identifier {
            spelling: &'a str,
            depth: usize,
            recovered: bool,
            open: &'a str,
            close: &'a str,
        },
        Numeral(&'a str),
        Empty,
        DoubleIdentifier(&'a str),
    }

    #[derive(Clone, Copy)]
    struct ParenthesizedIdentifierBinaryTheoremSpec<'a> {
        status: Option<&'a str>,
        label: &'a str,
        left: ParenthesizedIdentifierOperandShape<'a>,
        operator: &'a str,
        right: ParenthesizedIdentifierOperandShape<'a>,
        recovered_label: bool,
    }

    fn reserve_then_parenthesized_identifier_binary_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        spec: ParenthesizedIdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_parenthesized_identifier_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            spec,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    #[derive(Clone, Copy)]
    enum ParenthesizedHeterogeneousTypeRangeCorruption {
        Collapsed,
        Reversed,
    }

    fn parenthesized_heterogeneous_reserve_membership_ast_with_corrupt_type_ranges(
        source_id: SourceId,
        corruption: ParenthesizedHeterogeneousTypeRangeCorruption,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = Vec::new();
        let mut first_type_range = None;
        for (index, (name, head)) in [("x", "object"), ("y", "set")].into_iter().enumerate() {
            let item_start = offset;
            let reserve = add_token(
                &mut builder,
                source_id,
                &mut offset,
                SurfaceTokenKind::ReservedWord,
                "reserve",
            );
            let segment_start = offset;
            let identifier = add_token(
                &mut builder,
                source_id,
                &mut offset,
                SurfaceTokenKind::Identifier,
                name,
            );
            let for_token = add_token(
                &mut builder,
                source_id,
                &mut offset,
                SurfaceTokenKind::ReservedWord,
                "for",
            );
            let type_start = offset;
            let type_token = add_token(
                &mut builder,
                source_id,
                &mut offset,
                SurfaceTokenKind::ReservedWord,
                head,
            );
            let actual_type_range = range(source_id, type_start, type_start + head.len());
            let type_head = builder.add_node(
                SurfaceNodeKind::TypeHead,
                actual_type_range,
                vec![type_token],
            );
            let type_range = if index == 0 {
                first_type_range = Some(actual_type_range);
                actual_type_range
            } else {
                let first = first_type_range.expect("the object type range should precede set");
                match corruption {
                    ParenthesizedHeterogeneousTypeRangeCorruption::Collapsed => first,
                    ParenthesizedHeterogeneousTypeRangeCorruption::Reversed => range(
                        source_id,
                        first.start.saturating_sub(1),
                        first.end.saturating_sub(1),
                    ),
                }
            };
            let type_expression =
                builder.add_node(SurfaceNodeKind::TypeExpression, type_range, vec![type_head]);
            let segment = builder.add_node(
                SurfaceNodeKind::ReserveSegment,
                range(source_id, segment_start, actual_type_range.end),
                vec![identifier, for_token, type_expression],
            );
            let semicolon = add_token(
                &mut builder,
                source_id,
                &mut offset,
                SurfaceTokenKind::ReservedSymbol,
                ";",
            );
            let item_end = builder
                .node_range(semicolon)
                .expect("just-created reserve semicolon should exist")
                .end;
            root_children.push(builder.add_node(
                SurfaceNodeKind::ReserveItem,
                range(source_id, item_start, item_end),
                vec![reserve, segment, semicolon],
            ));
        }
        root_children.push(add_parenthesized_identifier_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedHeterogeneousReserveMembershipPayloadBoundary",
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                operator: "in",
                right: ParenthesizedIdentifierOperandShape::Direct("y"),
                recovered_label: false,
            },
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_identifier_binary_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
    ) -> SurfaceAst {
        reserve_then_identifier_binary_theorem_ast_with_options(
            source_id,
            items,
            IdentifierBinaryTheoremSpec {
                status: None,
                label,
                left,
                operator,
                right,
                recovered_label: false,
            },
        )
    }

    fn reserve_then_identifier_binary_theorem_ast_with_options(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        spec: IdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_identifier_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            spec,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_two_identifier_equality_theorems_ast(source_id: SourceId) -> SurfaceAst {
        reserve_then_two_identifier_binary_theorems_ast(
            source_id,
            "ReservedVariableEqualityPayloadBoundary",
            "=",
        )
    }

    fn reserve_then_two_identifier_binary_theorems_ast(
        source_id: SourceId,
        label: &str,
        operator: &str,
    ) -> SurfaceAst {
        reserve_then_two_identifier_binary_theorems_with_options_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            IdentifierBinaryTheoremSpec {
                status: None,
                label,
                left: "x",
                operator,
                right: "x",
                recovered_label: false,
            },
        )
    }

    fn reserve_then_two_identifier_binary_theorems_with_options_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        spec: IdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        for _ in 0..2 {
            root_children.push(add_identifier_binary_theorem_item(
                &mut builder,
                source_id,
                &mut offset,
                spec,
            ));
        }
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn theorem_then_reserve_identifier_equality_ast(source_id: SourceId) -> SurfaceAst {
        theorem_then_reserve_identifier_binary_ast(
            source_id,
            "ReservedVariableEqualityPayloadBoundary",
            "=",
        )
    }

    fn theorem_then_reserve_identifier_binary_ast(
        source_id: SourceId,
        label: &str,
        operator: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem = add_identifier_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            IdentifierBinaryTheoremSpec {
                status: None,
                label,
                left: "x",
                operator,
                right: "x",
                recovered_label: false,
            },
        );
        let reserve = add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
        );
        let mut root_children = vec![theorem];
        root_children.extend(reserve);
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn reserved_variable_binary_gap_cases(
        source_id: SourceId,
        label: &'static str,
        operator: &'static str,
    ) -> Vec<SurfaceAst> {
        let reserve = || vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))];
        vec![
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                "OtherPayloadBoundary",
                "x",
                operator,
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                label,
                "y",
                operator,
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                label,
                "x",
                operator,
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                label,
                "x",
                if operator == "=" { "<>" } else { "=" },
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                label,
                "x",
                operator,
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                label,
                "x",
                operator,
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
                label,
                "x",
                operator,
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                label,
                "x",
                operator,
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label,
                    left: "x",
                    operator,
                    right: "x",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                reserve(),
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label,
                    left: "x",
                    operator,
                    right: "x",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(source_id, label, operator),
            theorem_then_reserve_identifier_binary_ast(source_id, label, operator),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                reserve(),
                label,
                "1",
                operator,
                "1",
            ),
        ]
    }

    fn reserve_then_builtin_binary_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_builtin_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            label,
            left,
            operator,
            right,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_builtin_type_assertion_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        label: &str,
        subject: &str,
        type_shape: ReserveTypeShape,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_builtin_type_assertion_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            label,
            subject,
            type_shape,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    #[derive(Clone, Copy)]
    struct IdentifierTypeAssertionTheoremSpec<'a> {
        status: Option<&'a str>,
        label: &'a str,
        subject: &'a str,
        asserted_type: ReserveTypeShape,
        recovered_label: bool,
        negated: bool,
    }

    fn exact_identifier_type_assertion_spec() -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("set"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_reserved_object_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ReservedObjectVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("object"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_local_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LocalModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("set"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_local_mode_asserted_head_spec() -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LocalModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("LocalModeAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_local_object_mode_asserted_head_spec() -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LocalObjectModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("LocalObjectModeAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_chained_local_mode_asserted_head_spec() -> IdentifierTypeAssertionTheoremSpec<'static>
    {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ChainedLocalModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("ChainModeAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_chained_local_mode_radix_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ChainedLocalModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("BaseModeRadixAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_chained_local_object_mode_radix_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ChainedLocalObjectModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("BaseObjectModeRadixAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_two_edge_local_mode_radix_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "TwoEdgeLocalModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeRadixAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_two_edge_local_mode_two_hop_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "TwoEdgeLocalModeTwoHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("BaseTwoHopModeAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_two_edge_local_object_mode_two_hop_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "TwoEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("BaseTwoHopObjectModeAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_three_edge_local_mode_two_hop_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalModeTwoHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(
                "InnerThreeEdgeModeTwoHopAssertedHead",
            ),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_three_edge_local_object_mode_two_hop_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(
                "InnerThreeEdgeObjectModeTwoHopAssertedHead",
            ),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_four_edge_local_mode_two_hop_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "FourEdgeLocalModeTwoHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(
                "MiddleFourEdgeModeTwoHopAssertedHead",
            ),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_two_edge_local_object_mode_radix_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "TwoEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(
                "MiddleTwoEdgeObjectModeRadixAssertedHead",
            ),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_three_edge_local_mode_radix_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(
                "MiddleThreeEdgeModeRadixAssertedHead",
            ),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_three_edge_local_object_mode_radix_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(
                "MiddleThreeEdgeObjectModeRadixAssertedHead",
            ),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_four_edge_local_mode_radix_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "FourEdgeLocalModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("OuterFourEdgeModeRadixAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_four_edge_local_object_mode_radix_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "FourEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(
                "OuterFourEdgeObjectModeRadixAssertedHead",
            ),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_chained_local_object_mode_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ChainedLocalObjectModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("ChainObjectModeAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_chained_local_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ChainedLocalModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("set"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_chained_local_object_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ChainedLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("object"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_two_edge_local_mode_asserted_head_spec() -> IdentifierTypeAssertionTheoremSpec<'static>
    {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "TwoEdgeLocalModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_two_edge_local_object_mode_asserted_head_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeAssertedHead"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_two_edge_local_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "TwoEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("set"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_two_edge_local_object_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "TwoEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("object"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_three_edge_local_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("set"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_three_edge_local_object_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("object"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_four_edge_local_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "FourEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("set"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_four_edge_local_object_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "FourEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("object"),
            recovered_label: false,
            negated: false,
        }
    }

    fn exact_local_object_mode_identifier_type_assertion_spec()
    -> IdentifierTypeAssertionTheoremSpec<'static> {
        IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("object"),
            recovered_label: false,
            negated: false,
        }
    }

    fn reserve_then_identifier_type_assertion_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        spec: IdentifierTypeAssertionTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_type_assertion_theorem_item_with_status(
            &mut builder,
            source_id,
            &mut offset,
            spec,
            true,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_two_identifier_type_assertion_theorems_ast(source_id: SourceId) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
        );
        let spec = exact_identifier_type_assertion_spec();
        for _ in 0..2 {
            root_children.push(add_type_assertion_theorem_item_with_status(
                &mut builder,
                source_id,
                &mut offset,
                spec,
                true,
            ));
        }
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn identifier_type_assertion_theorem_then_reserve_ast(source_id: SourceId) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let spec = exact_identifier_type_assertion_spec();
        let theorem = add_type_assertion_theorem_item_with_status(
            &mut builder,
            source_id,
            &mut offset,
            spec,
            true,
        );
        let mut root_children = vec![theorem];
        root_children.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    #[derive(Clone, Copy)]
    struct FormulaStatementTheoremSpec<'a> {
        status: Option<&'a str>,
        recovered_label: bool,
        label: &'a str,
        constant: SurfaceFormulaConstant,
    }

    fn exact_formula_statement_spec() -> FormulaStatementTheoremSpec<'static> {
        FormulaStatementTheoremSpec {
            status: None,
            recovered_label: false,
            label: "FormulaPayloadBoundary",
            constant: SurfaceFormulaConstant::Thesis,
        }
    }

    fn exact_contradiction_formula_spec() -> FormulaStatementTheoremSpec<'static> {
        FormulaStatementTheoremSpec {
            status: None,
            recovered_label: false,
            label: "SourceDerivedContradictionConstantBoundary",
            constant: SurfaceFormulaConstant::Contradiction,
        }
    }

    fn formula_statement_theorem_ast(
        source_id: SourceId,
        spec: FormulaStatementTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem =
            add_formula_statement_theorem_item(&mut builder, source_id, &mut offset, spec);
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_formula_statement_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        reserve_then_exact_formula_constant_theorem_ast(
            source_id,
            items,
            exact_formula_statement_spec(),
        )
    }

    fn reserve_then_exact_formula_constant_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        spec: FormulaStatementTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_formula_statement_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            spec,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn double_formula_statement_theorem_ast(source_id: SourceId) -> SurfaceAst {
        double_exact_formula_constant_theorem_ast(source_id, exact_formula_statement_spec())
    }

    fn double_exact_formula_constant_theorem_ast(
        source_id: SourceId,
        spec: FormulaStatementTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let items = vec![
            add_formula_statement_theorem_item(&mut builder, source_id, &mut offset, spec),
            add_formula_statement_theorem_item(&mut builder, source_id, &mut offset, spec),
        ];
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            items,
        );
        builder.finish(Some(root), None)
    }

    fn proof_block_formula_theorem_ast(source_id: SourceId, label: &str) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem =
            add_proof_block_formula_theorem_item(&mut builder, source_id, &mut offset, label);
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    #[derive(Clone, Copy)]
    struct FormulaConnectiveQuantifierTheoremSpec<'a> {
        status: Option<&'a str>,
        recovered_label: bool,
        label: &'a str,
        connective: SurfaceFormulaConnective,
        quantifier: SurfaceQuantifierKind,
        binder_type: ReserveTypeShape,
        negated: bool,
    }

    fn exact_formula_shell_spec() -> FormulaConnectiveQuantifierTheoremSpec<'static> {
        FormulaConnectiveQuantifierTheoremSpec {
            status: None,
            recovered_label: false,
            label: "FormulaConnectiveQuantifierPayloadBoundary",
            connective: SurfaceFormulaConnective::Implies,
            quantifier: SurfaceQuantifierKind::Universal,
            binder_type: ReserveTypeShape::Builtin("set"),
            negated: true,
        }
    }

    fn formula_connective_quantifier_theorem_ast(
        source_id: SourceId,
        spec: FormulaConnectiveQuantifierTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem = add_formula_connective_quantifier_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            spec,
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_formula_connective_quantifier_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_formula_connective_quantifier_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            exact_formula_shell_spec(),
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn double_formula_connective_quantifier_theorem_ast(source_id: SourceId) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let items = vec![
            add_formula_connective_quantifier_theorem_item(
                &mut builder,
                source_id,
                &mut offset,
                exact_formula_shell_spec(),
            ),
            add_formula_connective_quantifier_theorem_item(
                &mut builder,
                source_id,
                &mut offset,
                exact_formula_shell_spec(),
            ),
        ];
        finish_compilation_ast(builder, source_id, items)
    }

    fn attribute_assertion_theorem_ast(
        source_id: SourceId,
        label: &str,
        subject: &str,
        attribute: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem = add_attribute_assertion_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            label,
            subject,
            attribute,
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    fn imported_attribute_assertion_theorem_ast(
        source_id: SourceId,
        imports: &[&str],
        label: &str,
        subject: &str,
        attribute: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut items = imports
            .iter()
            .map(|import| add_import_item(&mut builder, source_id, &mut offset, import))
            .collect::<Vec<_>>();
        items.push(add_attribute_assertion_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            label,
            subject,
            attribute,
        ));
        finish_compilation_ast(builder, source_id, items)
    }

    fn imported_non_empty_attribute_assertion_theorem_ast(
        source_id: SourceId,
        imports: &[&str],
        label: &str,
        subject: &str,
        attribute: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut items = imports
            .iter()
            .map(|import| add_import_item(&mut builder, source_id, &mut offset, import))
            .collect::<Vec<_>>();
        items.push(add_attribute_assertion_theorem_item_with_polarity(
            &mut builder,
            source_id,
            &mut offset,
            label,
            subject,
            attribute,
            true,
        ));
        finish_compilation_ast(builder, source_id, items)
    }

    fn reserve_then_imported_attribute_assertion_theorem_ast(
        source_id: SourceId,
        reserve_items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut items = vec![add_import_item(
            &mut builder,
            source_id,
            &mut offset,
            "parser.type_fixtures",
        )];
        items.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            reserve_items,
        ));
        items.push(add_attribute_assertion_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            "ImportedAttributeAssertionPayloadBoundary",
            "1",
            "empty",
        ));
        finish_compilation_ast(builder, source_id, items)
    }

    fn reserve_then_imported_non_empty_attribute_assertion_theorem_ast(
        source_id: SourceId,
        reserve_items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut items = vec![add_import_item(
            &mut builder,
            source_id,
            &mut offset,
            "parser.type_fixtures",
        )];
        items.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            reserve_items,
        ));
        items.push(add_attribute_assertion_theorem_item_with_polarity(
            &mut builder,
            source_id,
            &mut offset,
            "ImportedNonEmptyAttributeAssertionPayloadBoundary",
            "1",
            "empty",
            true,
        ));
        finish_compilation_ast(builder, source_id, items)
    }

    fn proof_block_formula_shell_label_ast(source_id: SourceId) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem = add_proof_block_formula_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            "FormulaConnectiveQuantifierPayloadBoundary",
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    #[derive(Clone, Copy)]
    struct SetEnumerationTheoremSpec<'a> {
        status: Option<&'a str>,
        recovered_label: bool,
        label: &'a str,
        left: [&'a str; 2],
        operator: &'a str,
        right: [&'a str; 2],
    }

    fn exact_set_enumeration_theorem_spec() -> SetEnumerationTheoremSpec<'static> {
        SetEnumerationTheoremSpec {
            status: None,
            recovered_label: false,
            label: "SetEnumerationPayloadBoundary",
            left: ["1", "2"],
            operator: "=",
            right: ["1", "2"],
        }
    }

    fn set_enumeration_equality_theorem_ast(
        source_id: SourceId,
        label: &str,
        left: [&str; 2],
        operator: &str,
        right: [&str; 2],
    ) -> SurfaceAst {
        let spec = SetEnumerationTheoremSpec {
            status: None,
            recovered_label: false,
            label,
            left,
            operator,
            right,
        };
        set_enumeration_theorem_ast(source_id, spec)
    }

    fn set_enumeration_equality_theorem_ast_with_status(
        source_id: SourceId,
        status: &str,
        label: &str,
        left: [&str; 2],
        operator: &str,
        right: [&str; 2],
    ) -> SurfaceAst {
        let spec = SetEnumerationTheoremSpec {
            status: Some(status),
            recovered_label: false,
            label,
            left,
            operator,
            right,
        };
        set_enumeration_theorem_ast(source_id, spec)
    }

    fn recovered_set_enumeration_equality_theorem_ast(source_id: SourceId) -> SurfaceAst {
        set_enumeration_theorem_ast(
            source_id,
            SetEnumerationTheoremSpec {
                recovered_label: true,
                ..exact_set_enumeration_theorem_spec()
            },
        )
    }

    fn set_enumeration_theorem_ast(
        source_id: SourceId,
        spec: SetEnumerationTheoremSpec<'_>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem = add_set_enumeration_theorem_item(&mut builder, source_id, &mut offset, spec);
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_set_enumeration_equality_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_set_enumeration_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            exact_set_enumeration_theorem_spec(),
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn import_then_set_enumeration_equality_theorem_ast(
        source_id: SourceId,
        module_path: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let items = vec![
            add_import_item(&mut builder, source_id, &mut offset, module_path),
            add_set_enumeration_theorem_item(
                &mut builder,
                source_id,
                &mut offset,
                exact_set_enumeration_theorem_spec(),
            ),
        ];
        finish_compilation_ast(builder, source_id, items)
    }

    fn double_set_enumeration_equality_theorem_ast(source_id: SourceId) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let items = vec![
            add_set_enumeration_theorem_item(
                &mut builder,
                source_id,
                &mut offset,
                exact_set_enumeration_theorem_spec(),
            ),
            add_set_enumeration_theorem_item(
                &mut builder,
                source_id,
                &mut offset,
                exact_set_enumeration_theorem_spec(),
            ),
        ];
        finish_compilation_ast(builder, source_id, items)
    }

    #[derive(Clone, Copy)]
    struct ImportedPredicateFunctorTheoremSpec<'a> {
        status: Option<&'a str>,
        label: &'a str,
        predicate: &'a str,
        left: &'a str,
        functor: &'a str,
        functor_left: &'a str,
        functor_right: &'a str,
    }

    #[derive(Clone, Copy, Default)]
    struct ImportedPredicateFunctorTheoremCorruption {
        recovered_label: bool,
        recovered_functor: bool,
        duplicate_theorem: bool,
        duplicate_formula_expression: bool,
        extra_formula_child: bool,
        extra_predicate_segment: bool,
        extra_segment_child: bool,
        extra_predicate_head_child: bool,
        extra_parenthesized_child: bool,
        extra_inner_expression_child: bool,
        extra_infix_operand: bool,
    }

    fn exact_imported_predicate_functor_theorem_spec()
    -> ImportedPredicateFunctorTheoremSpec<'static> {
        ImportedPredicateFunctorTheoremSpec {
            status: None,
            label: "ImportedPredicateFunctorPayloadBoundary",
            predicate: "divides",
            left: "1",
            functor: "++",
            functor_left: "1",
            functor_right: "2",
        }
    }

    fn imported_predicate_functor_theorem_ast(
        source_id: SourceId,
        imports: &[&str],
        spec: ImportedPredicateFunctorTheoremSpec<'_>,
    ) -> SurfaceAst {
        imported_predicate_functor_theorem_ast_with_corruption(
            source_id,
            imports,
            spec,
            ImportedPredicateFunctorTheoremCorruption::default(),
        )
    }

    fn imported_predicate_functor_theorem_ast_with_corruption(
        source_id: SourceId,
        imports: &[&str],
        spec: ImportedPredicateFunctorTheoremSpec<'_>,
        corruption: ImportedPredicateFunctorTheoremCorruption,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut items = imports
            .iter()
            .map(|import| add_import_item(&mut builder, source_id, &mut offset, import))
            .collect::<Vec<_>>();
        let theorem_count = if corruption.duplicate_theorem { 2 } else { 1 };
        items.extend((0..theorem_count).map(|_| {
            add_imported_predicate_functor_theorem_item_with_corruption(
                &mut builder,
                source_id,
                &mut offset,
                spec,
                corruption,
            )
        }));
        finish_compilation_ast(builder, source_id, items)
    }

    fn reserve_then_imported_predicate_functor_theorem_ast(
        source_id: SourceId,
        reserve_items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut items = vec![add_import_item(
            &mut builder,
            source_id,
            &mut offset,
            "parser.type_fixtures",
        )];
        items.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            reserve_items,
        ));
        items.push(add_imported_predicate_functor_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            exact_imported_predicate_functor_theorem_spec(),
        ));
        finish_compilation_ast(builder, source_id, items)
    }

    fn finish_compilation_ast(
        mut builder: SurfaceAstBuilder,
        source_id: SourceId,
        items: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceAst {
        let end = items
            .last()
            .and_then(|item| builder.node_range(*item))
            .map_or(0, |range| range.end);
        let item_list =
            builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, end), items);
        let compilation_unit = builder.add_node(
            SurfaceNodeKind::CompilationUnit,
            range(source_id, 0, end),
            vec![item_list],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, end),
            vec![compilation_unit],
        );
        builder.finish(Some(root), None)
    }

    fn add_import_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        module_path: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let import = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "import",
        );
        let path = add_module_path(builder, source_id, offset, module_path);
        let decl_end = builder
            .node_range(path)
            .expect("just-created module path should exist")
            .end;
        let decl = builder.add_node(
            SurfaceNodeKind::ImportAliasDecl,
            range(
                source_id,
                builder
                    .node_range(path)
                    .expect("just-created module path should exist")
                    .start,
                decl_end,
            ),
            vec![path],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::ImportItem,
            range(source_id, start, end),
            vec![import, decl, semicolon],
        )
    }

    fn add_module_path(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        module_path: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let mut children = Vec::new();
        for (index, segment) in module_path.split('.').enumerate() {
            if index != 0 {
                children.push(add_token(
                    builder,
                    source_id,
                    offset,
                    SurfaceTokenKind::ReservedSymbol,
                    ".",
                ));
            }
            children.push(add_path_segment(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                segment,
            ));
        }
        let end = children
            .last()
            .and_then(|child| builder.node_range(*child))
            .map_or(start, |range| range.end);
        builder.add_node(
            SurfaceNodeKind::ModulePath,
            range(source_id, start, end),
            children,
        )
    }

    fn add_path_segment(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        token_kind: SurfaceTokenKind,
        spelling: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let token = add_token(builder, source_id, offset, token_kind, spelling);
        let end = builder
            .node_range(token)
            .expect("just-created path-segment token should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, start, end),
            vec![token],
        )
    }

    fn add_qualified_symbol(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spelling: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let segment = add_path_segment(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::UserSymbol,
            spelling,
        );
        let end = builder
            .node_range(segment)
            .expect("just-created qualified-symbol segment should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, start, end),
            vec![segment],
        )
    }

    fn add_imported_predicate_functor_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: ImportedPredicateFunctorTheoremSpec<'_>,
    ) -> SurfaceBuilderNodeId {
        add_imported_predicate_functor_theorem_item_with_corruption(
            builder,
            source_id,
            offset,
            spec,
            ImportedPredicateFunctorTheoremCorruption::default(),
        )
    }

    fn add_imported_predicate_functor_theorem_item_with_corruption(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: ImportedPredicateFunctorTheoremSpec<'_>,
        corruption: ImportedPredicateFunctorTheoremCorruption,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let status_token = spec.status.map(|status| {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                status,
            )
        });
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = if corruption.recovered_label {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        };
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let left_term = add_numeral_term_expression(builder, source_id, offset, spec.left);
        let predicate_head =
            add_predicate_head(builder, source_id, offset, spec.predicate, corruption);
        let right_term = add_parenthesized_infix_term_expression(
            builder,
            source_id,
            offset,
            spec.functor,
            spec.functor_left,
            spec.functor_right,
            corruption,
        );
        let formula_end = builder
            .node_range(right_term)
            .expect("just-created predicate right term should exist")
            .end;
        let mut segment_children = vec![left_term, predicate_head, right_term];
        if corruption.extra_segment_child {
            segment_children.push(builder.add_node(
                SurfaceNodeKind::TermExpression,
                range(source_id, formula_start, formula_end),
                Vec::new(),
            ));
        }
        let segment = builder.add_node(
            SurfaceNodeKind::PredicateSegment,
            range(source_id, formula_start, formula_end),
            segment_children,
        );
        let mut predicate_segments = vec![segment];
        if corruption.extra_predicate_segment {
            predicate_segments.push(builder.add_node(
                SurfaceNodeKind::PredicateSegment,
                range(source_id, formula_start, formula_end),
                Vec::new(),
            ));
        }
        let formula = builder.add_node(
            SurfaceNodeKind::PredicateApplication,
            range(source_id, formula_start, formula_end),
            predicate_segments,
        );
        let mut formula_children = vec![formula];
        if corruption.extra_formula_child {
            formula_children.push(builder.add_node(
                SurfaceNodeKind::PredicateApplication,
                range(source_id, formula_start, formula_end),
                Vec::new(),
            ));
        }
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            formula_children,
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut children = Vec::new();
        if let Some(status_token) = status_token {
            children.push(status_token);
        }
        children.extend([theorem, label_token, colon, formula_expression]);
        if corruption.duplicate_formula_expression {
            children.push(builder.add_node(
                SurfaceNodeKind::FormulaExpression,
                range(source_id, formula_start, formula_end),
                Vec::new(),
            ));
        }
        children.push(semicolon);
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            children,
        )
    }

    fn add_predicate_head(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        predicate: &str,
        corruption: ImportedPredicateFunctorTheoremCorruption,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let symbol = add_qualified_symbol(builder, source_id, offset, predicate);
        let end = builder
            .node_range(symbol)
            .expect("just-created predicate symbol should exist")
            .end;
        let mut children = vec![symbol];
        if corruption.extra_predicate_head_child {
            children.push(builder.add_node(
                SurfaceNodeKind::QualifiedSymbol,
                range(source_id, start, end),
                Vec::new(),
            ));
        }
        builder.add_node(
            SurfaceNodeKind::PredicateHead,
            range(source_id, start, end),
            children,
        )
    }

    fn add_parenthesized_infix_term_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        operator: &str,
        left: &str,
        right: &str,
        corruption: ImportedPredicateFunctorTheoremCorruption,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let open = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            "(",
        );
        let inner = add_infix_term_expression(
            builder,
            source_id,
            offset,
            operator,
            left,
            right,
            corruption,
        );
        let close = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ")",
        );
        let end = builder
            .node_range(close)
            .expect("just-created parenthesized close should exist")
            .end;
        let mut parenthesized_children = vec![open, inner];
        if corruption.extra_parenthesized_child {
            parenthesized_children.push(builder.add_node(
                SurfaceNodeKind::TermExpression,
                range(source_id, start, end),
                Vec::new(),
            ));
        }
        parenthesized_children.push(close);
        let parenthesized = builder.add_node(
            SurfaceNodeKind::ParenthesizedTerm,
            range(source_id, start, end),
            parenthesized_children,
        );
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, start, end),
            vec![parenthesized],
        )
    }

    fn add_infix_term_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        operator: &str,
        left: &str,
        right: &str,
        corruption: ImportedPredicateFunctorTheoremCorruption,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let left_term = add_numeral_term(builder, source_id, offset, left);
        let operator_token = if corruption.recovered_functor {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::UserSymbol,
                operator,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::UserSymbol,
                operator,
            )
        };
        let right_term = add_numeral_term(builder, source_id, offset, right);
        let end = builder
            .node_range(right_term)
            .expect("just-created infix right term should exist")
            .end;
        let infix_kind = mizar_syntax::SurfaceInfixOperator {
            spelling: operator.into(),
            precedence: 10,
            associativity: mizar_syntax::SurfaceOperatorAssociativity::Left,
        };
        let mut infix_children = vec![left_term, operator_token, right_term];
        if corruption.extra_infix_operand {
            infix_children.push(builder.add_node(
                SurfaceNodeKind::NumeralTerm,
                range(source_id, start, end),
                Vec::new(),
            ));
        }
        let infix = builder.add_node(
            SurfaceNodeKind::InfixExpression(infix_kind.clone()),
            range(source_id, start, end),
            infix_children,
        );
        let mut inner_children = vec![infix];
        if corruption.extra_inner_expression_child {
            inner_children.push(builder.add_node(
                SurfaceNodeKind::InfixExpression(infix_kind),
                range(source_id, start, end),
                Vec::new(),
            ));
        }
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, start, end),
            inner_children,
        )
    }

    fn add_builtin_binary_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
    ) -> SurfaceBuilderNodeId {
        let shape = BuiltinBinaryTheoremShape {
            label,
            left,
            operator,
            right,
            corruption: BuiltinBinaryTheoremCorruption::default(),
        };
        add_builtin_binary_theorem_item_with_status(builder, source_id, offset, None, shape)
    }

    #[derive(Clone, Copy)]
    struct IdentifierBinaryTheoremSpec<'a> {
        status: Option<&'a str>,
        label: &'a str,
        left: &'a str,
        operator: &'a str,
        right: &'a str,
        recovered_label: bool,
    }

    fn add_identifier_binary_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: IdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let status_token = spec.status.map(|status| {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                status,
            )
        });
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = if spec.recovered_label {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        };
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let left_term = add_identifier_term_expression(builder, source_id, offset, spec.left);
        let operator = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            spec.operator,
        );
        let right_term = add_identifier_term_expression(builder, source_id, offset, spec.right);
        let formula_end = builder
            .node_range(right_term)
            .expect("just-created right term should exist")
            .end;
        let formula = builder.add_node(
            SurfaceNodeKind::BuiltinPredicateApplication,
            range(source_id, formula_start, formula_end),
            vec![left_term, operator, right_term],
        );
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            vec![formula],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut children = Vec::new();
        if let Some(status_token) = status_token {
            children.push(status_token);
        }
        children.extend([theorem, label_token, colon, formula_expression, semicolon]);
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            children,
        )
    }

    fn add_parenthesized_identifier_binary_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: ParenthesizedIdentifierBinaryTheoremSpec<'_>,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let status_token = spec.status.map(|status| {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                status,
            )
        });
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = if spec.recovered_label {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        };
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let left_term =
            add_parenthesized_identifier_operand_expression(builder, source_id, offset, spec.left);
        let operator = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            spec.operator,
        );
        let right_term =
            add_parenthesized_identifier_operand_expression(builder, source_id, offset, spec.right);
        let formula_end = builder
            .node_range(right_term)
            .expect("just-created right term should exist")
            .end;
        let formula = builder.add_node(
            SurfaceNodeKind::BuiltinPredicateApplication,
            range(source_id, formula_start, formula_end),
            vec![left_term, operator, right_term],
        );
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            vec![formula],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut children = Vec::new();
        children.extend(status_token);
        children.extend([theorem, label_token, colon, formula_expression, semicolon]);
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            children,
        )
    }

    #[derive(Clone, Copy, Default)]
    struct BuiltinBinaryTheoremCorruption {
        recovered_label: bool,
        recovered_operator: bool,
        duplicate_formula_expression: bool,
        extra_term_expression: bool,
    }

    struct BuiltinBinaryTheoremShape<'a> {
        label: &'a str,
        left: &'a str,
        operator: &'a str,
        right: &'a str,
        corruption: BuiltinBinaryTheoremCorruption,
    }

    fn add_builtin_binary_theorem_item_with_status(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        status: Option<&str>,
        shape: BuiltinBinaryTheoremShape<'_>,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let status_token = status.map(|status| {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                status,
            )
        });
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = if shape.corruption.recovered_label {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                shape.label,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                shape.label,
            )
        };
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let left_term = add_numeral_term_expression(builder, source_id, offset, shape.left);
        let operator = if shape.corruption.recovered_operator {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                shape.operator,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                shape.operator,
            )
        };
        let right_term = add_numeral_term_expression(builder, source_id, offset, shape.right);
        let formula_end = builder
            .node_range(right_term)
            .expect("just-created right term should exist")
            .end;
        let mut formula_children = vec![left_term, operator, right_term];
        if shape.corruption.extra_term_expression {
            formula_children.push(builder.add_node(
                SurfaceNodeKind::TermExpression,
                range(source_id, formula_start, formula_end),
                Vec::new(),
            ));
        }
        let formula = builder.add_node(
            SurfaceNodeKind::BuiltinPredicateApplication,
            range(source_id, formula_start, formula_end),
            formula_children,
        );
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            vec![formula],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut formula_expressions = vec![formula_expression];
        if shape.corruption.duplicate_formula_expression {
            formula_expressions.push(builder.add_node(
                SurfaceNodeKind::FormulaExpression,
                range(source_id, formula_start, formula_end),
                Vec::new(),
            ));
        }
        let mut children = Vec::new();
        children.extend(status_token);
        children.extend([theorem, label_token, colon]);
        children.extend(formula_expressions);
        children.push(semicolon);
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            children,
        )
    }

    fn add_formula_statement_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: FormulaStatementTheoremSpec<'_>,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let status_token = spec.status.map(|status| {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                status,
            )
        });
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = if spec.recovered_label {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        };
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let formula = add_formula_constant(builder, source_id, offset, spec.constant);
        let formula_end = builder
            .node_range(formula)
            .expect("just-created formula constant should exist")
            .end;
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            vec![formula],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut children = Vec::new();
        if let Some(status_token) = status_token {
            children.push(status_token);
        }
        children.extend([theorem, label_token, colon, formula_expression, semicolon]);
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            children,
        )
    }

    fn add_formula_connective_quantifier_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: FormulaConnectiveQuantifierTheoremSpec<'_>,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let status_token = spec.status.map(|status| {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                status,
            )
        });
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = if spec.recovered_label {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        };
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let formula = add_formula_connective_quantifier_formula(builder, source_id, offset, spec);
        let formula_end = builder
            .node_range(formula)
            .expect("just-created connective/quantifier formula should exist")
            .end;
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            vec![formula],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut children = Vec::new();
        if let Some(status_token) = status_token {
            children.push(status_token);
        }
        children.extend([theorem, label_token, colon, formula_expression, semicolon]);
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            children,
        )
    }

    fn add_attribute_assertion_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        label: &str,
        subject: &str,
        attribute: &str,
    ) -> SurfaceBuilderNodeId {
        add_attribute_assertion_theorem_item_with_polarity(
            builder, source_id, offset, label, subject, attribute, false,
        )
    }

    fn add_attribute_assertion_theorem_item_with_polarity(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        label: &str,
        subject: &str,
        attribute: &str,
        negative_attribute: bool,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            label,
        );
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let subject_term = add_numeral_term_expression(builder, source_id, offset, subject);
        let is_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "is",
        );
        let attribute_start = *offset;
        let mut attribute_children = Vec::new();
        if negative_attribute {
            attribute_children.push(add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "non",
            ));
        }
        let attribute_symbol = add_attribute_symbol(builder, source_id, offset, attribute, false);
        attribute_children.push(attribute_symbol);
        let attribute_end = builder
            .node_range(attribute_symbol)
            .expect("just-created attribute symbol should exist")
            .end;
        let attribute_ref = builder.add_node(
            SurfaceNodeKind::AttributeRef,
            range(source_id, attribute_start, attribute_end),
            attribute_children,
        );
        let attribute_chain = builder.add_node(
            SurfaceNodeKind::AttributeTestChain,
            range(source_id, attribute_start, attribute_end),
            vec![attribute_ref],
        );
        let formula = builder.add_node(
            SurfaceNodeKind::IsAssertion,
            range(source_id, formula_start, attribute_end),
            vec![subject_term, is_token, attribute_chain],
        );
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, attribute_end),
            vec![formula],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            vec![theorem, label_token, colon, formula_expression, semicolon],
        )
    }

    fn add_proof_block_formula_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        label: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            label,
        );
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let thesis =
            add_formula_constant(builder, source_id, offset, SurfaceFormulaConstant::Thesis);
        let formula_end = builder
            .node_range(thesis)
            .expect("just-created thesis formula should exist")
            .end;
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            vec![thesis],
        );
        let proof_start = *offset;
        let proof = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "proof",
        );
        let end_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "end",
        );
        let proof_end = builder
            .node_range(end_token)
            .expect("just-created proof end should exist")
            .end;
        let proof_block = builder.add_node(
            SurfaceNodeKind::ProofBlock,
            range(source_id, proof_start, proof_end),
            vec![proof, end_token],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            vec![
                theorem,
                label_token,
                colon,
                formula_expression,
                proof_block,
                semicolon,
            ],
        )
    }

    fn add_formula_connective_quantifier_formula(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: FormulaConnectiveQuantifierTheoremSpec<'_>,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let left = add_formula_constant(
            builder,
            source_id,
            offset,
            SurfaceFormulaConstant::Contradiction,
        );
        let connective = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            formula_connective_text(spec.connective),
        );
        let right = add_quantified_formula_shell(builder, source_id, offset, spec);
        let end = builder
            .node_range(right)
            .expect("just-created quantified formula should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator {
                connective: spec.connective,
                repeated: false,
            }),
            range(source_id, start, end),
            vec![left, connective, right],
        )
    }

    fn add_quantified_formula_shell(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: FormulaConnectiveQuantifierTheoremSpec<'_>,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let quantifier = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            quantifier_text(spec.quantifier),
        );
        let segment_start = *offset;
        let variable = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            "x",
        );
        let being = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "being",
        );
        let type_expression =
            add_reserve_type_expression(builder, source_id, offset, spec.binder_type);
        let segment_end = builder
            .node_range(type_expression)
            .expect("just-created quantified binder type should exist")
            .end;
        let segment = builder.add_node(
            SurfaceNodeKind::QuantifierVariableSegment,
            range(source_id, segment_start, segment_end),
            vec![variable, being, type_expression],
        );
        let holds = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "holds",
        );
        let body = if spec.negated {
            add_negated_formula_constant(
                builder,
                source_id,
                offset,
                SurfaceFormulaConstant::Contradiction,
            )
        } else {
            add_formula_constant(
                builder,
                source_id,
                offset,
                SurfaceFormulaConstant::Contradiction,
            )
        };
        let end = builder
            .node_range(body)
            .expect("just-created quantified body should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::QuantifiedFormula(spec.quantifier),
            range(source_id, start, end),
            vec![quantifier, segment, holds, body],
        )
    }

    fn add_negated_formula_constant(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        constant: SurfaceFormulaConstant,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let not = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "not",
        );
        let formula = add_formula_constant(builder, source_id, offset, constant);
        let end = builder
            .node_range(formula)
            .expect("just-created negated formula should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::PrefixFormula(SurfaceFormulaPrefixOperator::Not),
            range(source_id, start, end),
            vec![not, formula],
        )
    }

    fn add_formula_constant(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        constant: SurfaceFormulaConstant,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            formula_constant_text(constant),
        );
        let end = builder
            .node_range(token)
            .expect("just-created formula constant token should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::FormulaConstant(constant),
            range(source_id, start, end),
            vec![token],
        )
    }

    const fn formula_connective_text(connective: SurfaceFormulaConnective) -> &'static str {
        match connective {
            SurfaceFormulaConnective::And => "and",
            SurfaceFormulaConnective::Or => "or",
            SurfaceFormulaConnective::Implies => "implies",
            SurfaceFormulaConnective::Iff => "iff",
        }
    }

    const fn quantifier_text(quantifier: SurfaceQuantifierKind) -> &'static str {
        match quantifier {
            SurfaceQuantifierKind::Universal => "for",
            SurfaceQuantifierKind::Existential => "ex",
        }
    }

    const fn formula_constant_text(constant: SurfaceFormulaConstant) -> &'static str {
        match constant {
            SurfaceFormulaConstant::Thesis => "thesis",
            SurfaceFormulaConstant::Contradiction => "contradiction",
        }
    }

    fn add_set_enumeration_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: SetEnumerationTheoremSpec<'_>,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let status_token = spec.status.map(|status| {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                status,
            )
        });
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = if spec.recovered_label {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        };
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let left_term = add_set_enumeration_term_expression(builder, source_id, offset, spec.left);
        let operator = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            spec.operator,
        );
        let right_term =
            add_set_enumeration_term_expression(builder, source_id, offset, spec.right);
        let formula_end = builder
            .node_range(right_term)
            .expect("just-created set-enumeration right term should exist")
            .end;
        let formula = builder.add_node(
            SurfaceNodeKind::BuiltinPredicateApplication,
            range(source_id, formula_start, formula_end),
            vec![left_term, operator, right_term],
        );
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            vec![formula],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut children = Vec::new();
        if let Some(status_token) = status_token {
            children.push(status_token);
        }
        children.extend([theorem, label_token, colon, formula_expression, semicolon]);
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            children,
        )
    }

    fn add_builtin_type_assertion_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        label: &str,
        subject: &str,
        type_shape: ReserveTypeShape,
    ) -> SurfaceBuilderNodeId {
        add_builtin_type_assertion_theorem_item_with_status(
            builder, source_id, offset, None, label, subject, type_shape,
        )
    }

    #[derive(Clone, Copy, Default)]
    struct BuiltinTypeAssertionTheoremCorruption {
        recovered_label: bool,
        recovered_is: bool,
        duplicate_theorem: bool,
        duplicate_formula_expression: bool,
        extra_formula_child: bool,
        negated: bool,
        extra_assertion_operand: bool,
    }

    fn add_builtin_type_assertion_theorem_item_with_corruption(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        label: &str,
        subject: &str,
        type_shape: ReserveTypeShape,
        corruption: BuiltinTypeAssertionTheoremCorruption,
    ) -> SurfaceBuilderNodeId {
        add_type_assertion_theorem_item_with_status_and_corruption(
            builder,
            source_id,
            offset,
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label,
                subject,
                asserted_type: type_shape,
                recovered_label: corruption.recovered_label,
                negated: corruption.negated,
            },
            false,
            corruption,
        )
    }

    fn add_builtin_type_assertion_theorem_item_with_status(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        status: Option<&str>,
        label: &str,
        subject: &str,
        type_shape: ReserveTypeShape,
    ) -> SurfaceBuilderNodeId {
        add_type_assertion_theorem_item_with_status(
            builder,
            source_id,
            offset,
            IdentifierTypeAssertionTheoremSpec {
                status,
                label,
                subject,
                asserted_type: type_shape,
                recovered_label: false,
                negated: false,
            },
            false,
        )
    }

    fn add_type_assertion_theorem_item_with_status(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: IdentifierTypeAssertionTheoremSpec<'_>,
        identifier_subject: bool,
    ) -> SurfaceBuilderNodeId {
        add_type_assertion_theorem_item_with_status_and_corruption(
            builder,
            source_id,
            offset,
            spec,
            identifier_subject,
            BuiltinTypeAssertionTheoremCorruption::default(),
        )
    }

    fn add_type_assertion_theorem_item_with_status_and_corruption(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spec: IdentifierTypeAssertionTheoremSpec<'_>,
        identifier_subject: bool,
        corruption: BuiltinTypeAssertionTheoremCorruption,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let status_token = spec.status.map(|status| {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                status,
            )
        });
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = if spec.recovered_label {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spec.label,
            )
        };
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let subject_term = if identifier_subject {
            add_identifier_term_expression(builder, source_id, offset, spec.subject)
        } else {
            add_numeral_term_expression(builder, source_id, offset, spec.subject)
        };
        let is_token = if corruption.recovered_is {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "is",
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "is",
            )
        };
        let not_token = spec.negated.then(|| {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "not",
            )
        });
        let asserted_type =
            add_reserve_type_expression(builder, source_id, offset, spec.asserted_type);
        let formula_end = builder
            .node_range(asserted_type)
            .expect("just-created asserted type should exist")
            .end;
        let mut formula_children = vec![subject_term, is_token];
        if let Some(not_token) = not_token {
            formula_children.push(not_token);
        }
        formula_children.push(asserted_type);
        if corruption.extra_assertion_operand {
            formula_children.push(builder.add_node(
                SurfaceNodeKind::TermExpression,
                range(source_id, formula_start, formula_end),
                Vec::new(),
            ));
        }
        let formula = builder.add_node(
            SurfaceNodeKind::IsAssertion,
            range(source_id, formula_start, formula_end),
            formula_children,
        );
        let mut formula_children = vec![formula];
        if corruption.extra_formula_child {
            formula_children.push(builder.add_node(
                SurfaceNodeKind::IsAssertion,
                range(source_id, formula_start, formula_end),
                Vec::new(),
            ));
        }
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            formula_children,
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut children = Vec::new();
        if let Some(status_token) = status_token {
            children.push(status_token);
        }
        children.extend([theorem, label_token, colon, formula_expression]);
        if corruption.duplicate_formula_expression {
            children.push(builder.add_node(
                SurfaceNodeKind::FormulaExpression,
                range(source_id, formula_start, formula_end),
                Vec::new(),
            ));
        }
        children.push(semicolon);
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            children,
        )
    }

    fn add_set_enumeration_term_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        items: [&str; 2],
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let set = add_set_enumeration_term(builder, source_id, offset, items);
        let end = builder
            .node_range(set)
            .expect("just-created set-enumeration term should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, start, end),
            vec![set],
        )
    }

    fn add_set_enumeration_term(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        items: [&str; 2],
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let open = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            "{",
        );
        let first = add_numeral_term_expression(builder, source_id, offset, items[0]);
        let comma = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ",",
        );
        let second = add_numeral_term_expression(builder, source_id, offset, items[1]);
        let close = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            "}",
        );
        let end = builder
            .node_range(close)
            .expect("just-created set-enumeration close should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::SetEnumeration,
            range(source_id, start, end),
            vec![open, first, comma, second, close],
        )
    }

    fn add_numeral_term_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spelling: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let numeral = add_numeral_term(builder, source_id, offset, spelling);
        let end = builder
            .node_range(numeral)
            .expect("just-created numeral term should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, start, end),
            vec![numeral],
        )
    }

    fn add_identifier_term_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spelling: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            spelling,
        );
        let end = builder
            .node_range(token)
            .expect("just-created identifier token should exist")
            .end;
        let reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, start, end),
            vec![token],
        );
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, start, end),
            vec![reference],
        )
    }

    fn add_parenthesized_identifier_operand_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        shape: ParenthesizedIdentifierOperandShape<'_>,
    ) -> SurfaceBuilderNodeId {
        match shape {
            ParenthesizedIdentifierOperandShape::Direct(spelling) => {
                add_identifier_term_expression(builder, source_id, offset, spelling)
            }
            ParenthesizedIdentifierOperandShape::Identifier {
                spelling,
                depth,
                recovered,
                open,
                close,
            } => {
                let start = *offset;
                let open_token = add_token(
                    builder,
                    source_id,
                    offset,
                    SurfaceTokenKind::ReservedSymbol,
                    open,
                );
                let inner = if depth <= 1 {
                    add_identifier_term_expression_with_recovery(
                        builder, source_id, offset, spelling, recovered,
                    )
                } else {
                    add_parenthesized_identifier_operand_expression(
                        builder,
                        source_id,
                        offset,
                        ParenthesizedIdentifierOperandShape::Identifier {
                            spelling,
                            depth: depth - 1,
                            recovered,
                            open,
                            close,
                        },
                    )
                };
                let close_token = add_token(
                    builder,
                    source_id,
                    offset,
                    SurfaceTokenKind::ReservedSymbol,
                    close,
                );
                let end = builder
                    .node_range(close_token)
                    .expect("just-created parenthesized close should exist")
                    .end;
                let parenthesized = builder.add_node(
                    SurfaceNodeKind::ParenthesizedTerm,
                    range(source_id, start, end),
                    vec![open_token, inner, close_token],
                );
                builder.add_node(
                    SurfaceNodeKind::TermExpression,
                    range(source_id, start, end),
                    vec![parenthesized],
                )
            }
            ParenthesizedIdentifierOperandShape::Numeral(spelling) => {
                add_parenthesized_non_identifier_operand_expression(
                    builder,
                    source_id,
                    offset,
                    Some(add_numeral_term_expression),
                    spelling,
                )
            }
            ParenthesizedIdentifierOperandShape::Empty => {
                add_parenthesized_non_identifier_operand_expression(
                    builder, source_id, offset, None, "",
                )
            }
            ParenthesizedIdentifierOperandShape::DoubleIdentifier(spelling) => {
                let start = *offset;
                let open = add_token(
                    builder,
                    source_id,
                    offset,
                    SurfaceTokenKind::ReservedSymbol,
                    "(",
                );
                let first = add_identifier_term_expression(builder, source_id, offset, spelling);
                let second = add_identifier_term_expression(builder, source_id, offset, spelling);
                let close = add_token(
                    builder,
                    source_id,
                    offset,
                    SurfaceTokenKind::ReservedSymbol,
                    ")",
                );
                let end = builder
                    .node_range(close)
                    .expect("just-created doubled parenthesized close should exist")
                    .end;
                let parenthesized = builder.add_node(
                    SurfaceNodeKind::ParenthesizedTerm,
                    range(source_id, start, end),
                    vec![open, first, second, close],
                );
                builder.add_node(
                    SurfaceNodeKind::TermExpression,
                    range(source_id, start, end),
                    vec![parenthesized],
                )
            }
        }
    }

    fn add_identifier_term_expression_with_recovery(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spelling: &str,
        recovered: bool,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let token = if recovered {
            add_recovered_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spelling,
            )
        } else {
            add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                spelling,
            )
        };
        let end = builder
            .node_range(token)
            .expect("just-created identifier token should exist")
            .end;
        let reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, start, end),
            vec![token],
        );
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, start, end),
            vec![reference],
        )
    }

    type ParenthesizedInnerBuilder =
        fn(&mut SurfaceAstBuilder, SourceId, &mut usize, &str) -> SurfaceBuilderNodeId;

    fn add_parenthesized_non_identifier_operand_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        inner_builder: Option<ParenthesizedInnerBuilder>,
        spelling: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let open = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            "(",
        );
        let inner =
            inner_builder.map(|inner_builder| inner_builder(builder, source_id, offset, spelling));
        let close = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ")",
        );
        let end = builder
            .node_range(close)
            .expect("just-created parenthesized close should exist")
            .end;
        let mut children = vec![open];
        children.extend(inner);
        children.push(close);
        let parenthesized = builder.add_node(
            SurfaceNodeKind::ParenthesizedTerm,
            range(source_id, start, end),
            children,
        );
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, start, end),
            vec![parenthesized],
        )
    }

    fn add_numeral_term(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spelling: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Numeral,
            spelling,
        );
        let end = builder
            .node_range(token)
            .expect("just-created numeral token should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::NumeralTerm,
            range(source_id, start, end),
            vec![token],
        )
    }

    fn add_reserve_items(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        items: Vec<ReserveItemSpec>,
    ) -> Vec<SurfaceBuilderNodeId> {
        let mut root_children = Vec::new();
        for item in items {
            let item_start = *offset;
            let reserve = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "reserve",
            );
            let segment_start = *offset;
            let mut segment_children = Vec::new();
            for (index, name) in item.names.iter().enumerate() {
                segment_children.push(add_token(
                    builder,
                    source_id,
                    offset,
                    SurfaceTokenKind::Identifier,
                    name,
                ));
                if index + 1 != item.names.len() {
                    segment_children.push(add_token(
                        builder,
                        source_id,
                        offset,
                        SurfaceTokenKind::ReservedSymbol,
                        ",",
                    ));
                }
            }
            segment_children.push(add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "for",
            ));
            let type_expression =
                add_reserve_type_expression(builder, source_id, offset, item.type_shape);
            segment_children.push(type_expression);
            let segment_end = builder
                .node_range(type_expression)
                .expect("just-created type expression should exist")
                .end;
            let segment = builder.add_node(
                SurfaceNodeKind::ReserveSegment,
                range(source_id, segment_start, segment_end),
                segment_children,
            );
            let semicolon = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                ";",
            );
            let item_end = builder
                .node_range(semicolon)
                .expect("just-created semicolon should exist")
                .end;
            let reserve_item = builder.add_node(
                SurfaceNodeKind::ReserveItem,
                range(source_id, item_start, item_end),
                vec![reserve, segment, semicolon],
            );
            root_children.push(reserve_item);
        }
        root_children
    }

    fn add_mode_definition_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        mode: ModeDefinitionSpec,
    ) -> SurfaceBuilderNodeId {
        let item_start = *offset;
        let definition = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "definition",
        );
        let mut block_children = vec![definition];
        if mode.local_context {
            block_children.push(add_definition_parameter(builder, source_id, offset));
        }
        let mode_start = *offset;
        let mode_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "mode",
        );
        let label = mode
            .label
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{}Def", mode.pattern));
        let label_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            &label,
        );
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let pattern_start = *offset;
        let pattern_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            mode.pattern,
        );
        let mut pattern_children = vec![pattern_token];
        let mut pattern_end = pattern_start + mode.pattern.len();
        if mode.parameterized_pattern {
            let of = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "of",
            );
            let arg = add_simple_type_expression(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "set",
            );
            pattern_end = builder
                .node_range(arg)
                .expect("just-created pattern argument should exist")
                .end;
            let pattern_args = builder.add_node(
                SurfaceNodeKind::TypeArguments,
                range(
                    source_id,
                    pattern_start + mode.pattern.len() + 1,
                    pattern_end,
                ),
                vec![of, arg],
            );
            pattern_children.push(pattern_args);
        }
        let pattern = builder.add_node(
            SurfaceNodeKind::ModePattern,
            range(source_id, pattern_start, pattern_end),
            pattern_children,
        );
        let is = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "is",
        );
        let rhs = add_reserve_type_expression(builder, source_id, offset, mode.rhs_shape);
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let mode_end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut mode_definition_children = vec![mode_token, label_token, colon, pattern, is, rhs];
        if mode.recovered {
            let recovery = builder.add_recovery(
                SyntaxRecoveryKind::MissingTerm,
                range(source_id, mode_end, mode_end),
                Vec::new(),
            );
            mode_definition_children.push(recovery);
        }
        mode_definition_children.push(semicolon);
        let mode_definition = builder.add_node(
            SurfaceNodeKind::ModeDefinition,
            range(source_id, mode_start, mode_end),
            mode_definition_children,
        );
        let end = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "end",
        );
        let block_semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let item_end = builder
            .node_range(block_semicolon)
            .expect("just-created block semicolon should exist")
            .end;
        block_children.extend([mode_definition, end, block_semicolon]);
        builder.add_node(
            SurfaceNodeKind::DefinitionBlockItem,
            range(source_id, item_start, item_end),
            block_children,
        )
    }

    fn add_empty_definition_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
    ) -> SurfaceBuilderNodeId {
        let item_start = *offset;
        let definition = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "definition",
        );
        let end = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "end",
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let item_end = builder
            .node_range(semicolon)
            .expect("just-created definition semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::DefinitionBlockItem,
            range(source_id, item_start, item_end),
            vec![definition, end, semicolon],
        )
    }

    fn add_definition_parameter(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
    ) -> SurfaceBuilderNodeId {
        let parameter_start = *offset;
        let let_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "let",
        );
        let name = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            "x",
        );
        let be = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "be",
        );
        let ty = add_simple_type_expression(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "set",
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let parameter_end = builder
            .node_range(semicolon)
            .expect("just-created definition parameter semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::DefinitionParameter,
            range(source_id, parameter_start, parameter_end),
            vec![let_token, name, be, ty, semicolon],
        )
    }

    fn add_structure_definition_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        structure: &'static str,
    ) -> SurfaceBuilderNodeId {
        let item_start = *offset;
        let definition = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "definition",
        );
        let structure_start = *offset;
        let struct_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "struct",
        );
        let pattern_start = *offset;
        let pattern_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            structure,
        );
        let pattern = builder.add_node(
            SurfaceNodeKind::StructurePattern,
            range(source_id, pattern_start, pattern_start + structure.len()),
            vec![pattern_token],
        );
        let where_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "where",
        );
        let field = add_structure_field(builder, source_id, offset);
        let end = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "end",
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let structure_end = builder
            .node_range(semicolon)
            .expect("just-created structure semicolon should exist")
            .end;
        let structure_definition = builder.add_node(
            SurfaceNodeKind::StructureDefinition,
            range(source_id, structure_start, structure_end),
            vec![struct_token, pattern, where_token, field, end, semicolon],
        );
        let block_end = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "end",
        );
        let block_semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let item_end = builder
            .node_range(block_semicolon)
            .expect("just-created block semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::DefinitionBlockItem,
            range(source_id, item_start, item_end),
            vec![definition, structure_definition, block_end, block_semicolon],
        )
    }

    fn add_structure_field(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
    ) -> SurfaceBuilderNodeId {
        let field_start = *offset;
        let field = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "field",
        );
        let name = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            "carrier",
        );
        let arrow = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            "->",
        );
        let field_type = add_simple_type_expression(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "set",
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let field_end = builder
            .node_range(semicolon)
            .expect("just-created field semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::StructureField,
            range(source_id, field_start, field_end),
            vec![field, name, arrow, field_type, semicolon],
        )
    }

    fn add_reserve_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        shape: ReserveTypeShape,
    ) -> SurfaceBuilderNodeId {
        match shape {
            ReserveTypeShape::Builtin(head) => add_simple_type_expression(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                head,
            ),
            ReserveTypeShape::NonBuiltin(head) => add_simple_type_expression(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::UserSymbol,
                head,
            ),
            ReserveTypeShape::QualifiedSymbol(head) => {
                add_qualified_type_expression(builder, source_id, offset, head, false)
            }
            ReserveTypeShape::QualifiedSymbolWithArgs(head) => {
                add_qualified_type_expression(builder, source_id, offset, head, true)
            }
            ReserveTypeShape::AttributedSetWithNamedAttribute(attribute) => {
                attributed_type_expression_with_attribute(
                    builder,
                    source_id,
                    offset,
                    attribute,
                    "set",
                    false,
                    TypeAttributeShape::Positive,
                )
            }
            ReserveTypeShape::AttributedSetWithNegativeNamedAttribute(attribute) => {
                attributed_type_expression_with_attribute(
                    builder,
                    source_id,
                    offset,
                    attribute,
                    "set",
                    false,
                    TypeAttributeShape::Negative,
                )
            }
            ReserveTypeShape::AttributedSetWithDuplicateNamedAttribute(attribute) => {
                attributed_type_expression_with_attribute(
                    builder,
                    source_id,
                    offset,
                    attribute,
                    "set",
                    false,
                    TypeAttributeShape::DuplicateNegative,
                )
            }
            ReserveTypeShape::AttributedSetWithMixedPolarityNamedAttribute(attribute) => {
                attributed_type_expression_with_attribute(
                    builder,
                    source_id,
                    offset,
                    attribute,
                    "set",
                    false,
                    TypeAttributeShape::MixedPolarity,
                )
            }
            ReserveTypeShape::AttributedObjectWithNamedAttribute(attribute) => {
                attributed_type_expression_with_attribute(
                    builder,
                    source_id,
                    offset,
                    attribute,
                    "object",
                    false,
                    TypeAttributeShape::Positive,
                )
            }
            ReserveTypeShape::AttributedObjectWithDuplicateNamedAttribute(attribute) => {
                attributed_type_expression_with_attribute(
                    builder,
                    source_id,
                    offset,
                    attribute,
                    "object",
                    false,
                    TypeAttributeShape::DuplicateNegative,
                )
            }
            ReserveTypeShape::AttributedQualifiedSymbolWithNamedAttribute(attribute, head) => {
                add_attributed_qualified_type_expression_with_attribute(
                    builder, source_id, offset, attribute, head, false, false,
                )
            }
            ReserveTypeShape::AttributedQualifiedSymbol(head) => {
                add_attributed_qualified_type_expression(
                    builder, source_id, offset, head, false, false,
                )
            }
            ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs(head) => {
                add_attributed_qualified_type_expression(
                    builder, source_id, offset, head, true, false,
                )
            }
            ReserveTypeShape::QualifiedAttributeQualifiedSymbol(head) => {
                add_attributed_qualified_type_expression(
                    builder, source_id, offset, head, false, true,
                )
            }
            ReserveTypeShape::AttributedSet => {
                attributed_type_expression(builder, source_id, offset, "set", false)
            }
            ReserveTypeShape::AttributedSetWithAttributeArgs => {
                attributed_type_expression(builder, source_id, offset, "set", true)
            }
            ReserveTypeShape::AttributedObject => {
                attributed_type_expression(builder, source_id, offset, "object", false)
            }
        }
    }

    fn add_simple_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        token_kind: SurfaceTokenKind,
        head: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let token = add_token(builder, source_id, offset, token_kind, head);
        let type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, start, start + head.len()),
            vec![token],
        );
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, start, start + head.len()),
            vec![type_head],
        )
    }

    fn add_qualified_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        head: &str,
        with_args: bool,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let (type_head, end) = add_qualified_type_head(builder, source_id, offset, head, with_args);
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, start, end),
            vec![type_head],
        )
    }

    fn add_qualified_type_head(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        head: &str,
        with_args: bool,
    ) -> (SurfaceBuilderNodeId, usize) {
        let start = *offset;
        let head_start = *offset;
        let token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::UserSymbol,
            head,
        );
        let segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, head_start, head_start + head.len()),
            vec![token],
        );
        let symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, head_start, head_start + head.len()),
            vec![segment],
        );
        let mut type_head_children = vec![symbol];
        let mut end = head_start + head.len();
        if with_args {
            let of = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "of",
            );
            let arg = add_simple_type_expression(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "set",
            );
            end = builder
                .node_range(arg)
                .expect("just-created type argument should exist")
                .end;
            let type_args = builder.add_node(
                SurfaceNodeKind::TypeArguments,
                range(source_id, head_start + head.len() + 1, end),
                vec![of, arg],
            );
            type_head_children.push(type_args);
        }
        let type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, start, end),
            type_head_children,
        );
        (type_head, end)
    }

    fn attributed_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        head: &str,
        with_attribute_args: bool,
    ) -> SurfaceBuilderNodeId {
        attributed_type_expression_with_attribute(
            builder,
            source_id,
            offset,
            "empty",
            head,
            with_attribute_args,
            TypeAttributeShape::Negative,
        )
    }

    fn attributed_type_expression_with_attribute(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        attribute: &str,
        head: &str,
        with_attribute_args: bool,
        attribute_shape: TypeAttributeShape,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let include_non = matches!(
            attribute_shape,
            TypeAttributeShape::Negative | TypeAttributeShape::DuplicateNegative
        );
        let (attribute_node, mut attribute_end) = add_type_attribute_ref(
            builder,
            source_id,
            offset,
            attribute,
            with_attribute_args,
            include_non,
        );
        let mut attribute_chain_children = vec![attribute_node];
        if matches!(attribute_shape, TypeAttributeShape::DuplicateNegative) {
            let (duplicate, duplicate_end) = add_type_attribute_ref(
                builder,
                source_id,
                offset,
                attribute,
                with_attribute_args,
                include_non,
            );
            attribute_chain_children.push(duplicate);
            attribute_end = duplicate_end;
        }
        if matches!(attribute_shape, TypeAttributeShape::MixedPolarity) {
            let (negative, negative_end) = add_type_attribute_ref(
                builder,
                source_id,
                offset,
                attribute,
                with_attribute_args,
                true,
            );
            attribute_chain_children.push(negative);
            attribute_end = negative_end;
        }
        let head_start = *offset;
        let head_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            head,
        );
        let attribute_chain = builder.add_node(
            SurfaceNodeKind::AttributeChain,
            range(source_id, start, attribute_end),
            attribute_chain_children,
        );
        let type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, head_start, head_start + head.len()),
            vec![head_token],
        );
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, start, head_start + head.len()),
            vec![attribute_chain, type_head],
        )
    }

    fn add_type_attribute_ref(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        attribute: &str,
        with_attribute_args: bool,
        include_non: bool,
    ) -> (SurfaceBuilderNodeId, usize) {
        let start = *offset;
        let mut attribute_children = Vec::new();
        if include_non {
            attribute_children.push(add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "non",
            ));
        }
        let attribute_start = *offset;
        let attribute_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::UserSymbol,
            attribute,
        );
        let attribute_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(
                source_id,
                attribute_start,
                attribute_start + attribute.len(),
            ),
            vec![attribute_token],
        );
        let attribute_symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(
                source_id,
                attribute_start,
                attribute_start + attribute.len(),
            ),
            vec![attribute_segment],
        );
        let mut attribute_end = attribute_start + attribute.len();
        attribute_children.push(attribute_symbol);
        if with_attribute_args {
            let open = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                "(",
            );
            let arg = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                "x",
            );
            let close = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                ")",
            );
            attribute_end = builder
                .node_range(close)
                .expect("just-created attribute argument close should exist")
                .end;
            attribute_children.extend([open, arg, close]);
        }
        let attribute = builder.add_node(
            SurfaceNodeKind::AttributeRef,
            range(source_id, start, attribute_end),
            attribute_children,
        );
        (attribute, attribute_end)
    }

    fn add_attributed_qualified_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        head: &str,
        with_attribute_args: bool,
        qualified_attribute: bool,
    ) -> SurfaceBuilderNodeId {
        add_attributed_qualified_type_expression_with_attribute(
            builder,
            source_id,
            offset,
            "empty",
            head,
            with_attribute_args,
            qualified_attribute,
        )
    }

    fn add_attributed_qualified_type_expression_with_attribute(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        attribute_name: &str,
        head: &str,
        with_attribute_args: bool,
        qualified_attribute: bool,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let mut attribute_children = Vec::new();
        let include_non = attribute_name == "empty";
        if include_non {
            attribute_children.push(add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "non",
            ));
        }
        let attribute_symbol = add_attribute_symbol(
            builder,
            source_id,
            offset,
            attribute_name,
            qualified_attribute,
        );
        let mut attribute_end = builder
            .node_range(attribute_symbol)
            .expect("just-created attribute symbol should exist")
            .end;
        attribute_children.push(attribute_symbol);
        if with_attribute_args {
            let open = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                "(",
            );
            let arg = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                "x",
            );
            let close = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                ")",
            );
            attribute_end = builder
                .node_range(close)
                .expect("just-created attribute argument close should exist")
                .end;
            attribute_children.extend([open, arg, close]);
        }
        let attribute = builder.add_node(
            SurfaceNodeKind::AttributeRef,
            range(source_id, start, attribute_end),
            attribute_children,
        );
        let attribute_chain = builder.add_node(
            SurfaceNodeKind::AttributeChain,
            range(source_id, start, attribute_end),
            vec![attribute],
        );
        let (head_node, end) = add_qualified_type_head(builder, source_id, offset, head, false);
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, start, end),
            vec![attribute_chain, head_node],
        )
    }

    fn add_attribute_symbol(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        attribute: &str,
        qualified: bool,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let mut children = Vec::new();
        if qualified {
            let qualifier = "Struct";
            let qualifier_start = *offset;
            let qualifier_token = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::UserSymbol,
                qualifier,
            );
            children.push(builder.add_node(
                SurfaceNodeKind::PathSegment,
                range(
                    source_id,
                    qualifier_start,
                    qualifier_start + qualifier.len(),
                ),
                vec![qualifier_token],
            ));
            children.push(add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                ".",
            ));
        }
        let attribute_start = *offset;
        let attribute_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::UserSymbol,
            attribute,
        );
        let attribute_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(
                source_id,
                attribute_start,
                attribute_start + attribute.len(),
            ),
            vec![attribute_token],
        );
        children.push(attribute_segment);
        builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, start, attribute_start + attribute.len()),
            children,
        )
    }

    fn add_token(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        kind: SurfaceTokenKind,
        text: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let end = start + text.len();
        let token = builder.add_token(kind, text, range(source_id, start, end));
        *offset = end + 1;
        token
    }

    fn add_recovered_token(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        kind: SurfaceTokenKind,
        text: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let end = start + text.len();
        let token = builder.add_recovered_token(kind, text, range(source_id, start, end));
        *offset = end + 1;
        token
    }

    fn surface_sites_for_kind_ranges(
        ast: &SurfaceAst,
        kind: SurfaceNodeKind,
        ranges: &[SourceRange],
    ) -> Vec<TypedSiteRef> {
        let sites = surface_nodes_with_kind(ast, kind)
            .into_iter()
            .filter(|(_, node)| ranges.contains(&node.range))
            .map(|(id, _)| surface_site(id))
            .collect::<Vec<_>>();
        assert_eq!(sites.len(), ranges.len());
        sites
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn source_id(byte: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(byte))
            .unwrap()
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
