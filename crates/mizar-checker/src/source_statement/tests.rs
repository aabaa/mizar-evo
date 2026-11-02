use super::*;
use crate::{
    binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextTable, BindingDiagnosticClass,
        BindingDiagnosticDraft, BindingDiagnosticId, BindingDiagnosticRecovery,
        BindingDiagnosticSeverity, BindingDiagnosticTable, BindingDraft, BindingEnvParts,
        BindingLookupResult, BindingLookupSite, BindingRecoveryState, BindingStatus, BindingTable,
        BindingTypeSite, CapturedFreeVariables,
    },
    cluster_trace::{
        ClusterAttributeFingerprint, ClusterFactDraft, ClusterFactFingerprint,
        ClusterFactProvenance, ClusterFactTable, ClusterTypeFingerprint,
    },
    overload_resolution::{
        CandidateDeclarationKind, CandidateOrigin, CandidateProvenance, CandidateProvenanceKey,
        CandidateScope, CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateId,
        OverloadCandidateInput, OverloadCollectionOutput, OverloadNameKey, OverloadSelectionOutput,
        OverloadSiteId, OverloadSiteInput, OverloadSiteKey, OverloadSiteKind, OverloadSiteRecovery,
        OverloadSiteResolutionInput, RefinementJoinPayload, RefinementJoinStatus, SourceQuaView,
        SpecificityComparisonInput, SpecificityGraphOutput, TemplateExpansionOutput,
    },
    resolved_typed_ast::{
        ExprId, ExpressionMetadataInput, ResolvedNodeKindHint, ResolvedNodeKindHintKind,
        ResolvedTypedAst, ResolvedTypedAstError, ResolvedTypedAstInputs, SourceNodeRole,
        StatementProofInputs, StatementSemanticInputs,
    },
    source_application::{
        SourceFunctorApplicationForm, SourceFunctorApplicationHandoff,
        SourceFunctorApplicationHandoffInput, SourceFunctorApplicationInput,
        SourceFunctorApplicationKind, SourceFunctorApplicationProducer,
        SourceFunctorApplicationRecovery, SourceFunctorArgumentInput, SourceFunctorArgumentTarget,
        SourceFunctorCandidateId, SourceFunctorCandidateInput, SourceFunctorHeadSite,
        SourceFunctorTypeRequestInput, SourceFunctorTypeRequestKind, SourceFunctorWrapperInput,
    },
    source_atomic_formula::{
        SourceAssertionAttributeInput, SourceAssertionAttributePolarityInput,
        SourceAssertionTypeHead, SourceAssertionTypeSiteInput, SourceAtomicEdgeId,
        SourceAtomicEdgeInput, SourceAtomicFormulaHandoffInput, SourceAtomicFormulaInput,
        SourceAtomicFormulaProducer, SourceAtomicRequestId, SourceAtomicRequestInput,
        SourceAtomicWrapperInput, SourcePredicateCandidateId, SourcePredicateCandidateInput,
        SourcePredicateHeadId, SourcePredicateHeadInput, SourcePredicateSegmentInput,
        SourcePredicateSegmentPolarityInput,
    },
    source_composite_formula::{
        SourceBinderTypeHead, SourceBinderTypeSiteId, SourceBinderTypeSiteInput,
        SourceCompositeFormulaHandoffInput, SourceCompositeFormulaInput,
        SourceCompositeFormulaProducer, SourceFormulaRequestInput, SourceFormulaRequestKind,
        SourceFormulaRootInput, SourceQuantifierBinderId, SourceQuantifierBinderInput,
    },
    source_formula_composition::{
        SourceFormulaAtomicEdgeId, SourceFormulaAtomicEdgeInput, SourceFormulaAtomicEdgeRole,
        SourceFormulaCompositionHandoffInput, SourceFormulaCompositionProducer,
        SourceQuantifierBoundUseInput,
    },
    source_proof_local_declaration::{
        SourceProofLocalBindingCorruptionForTest, SourceProofLocalDeclarationError,
        SourceProofLocalDeclarationHandoff, SourceProofLocalDeclarationHandoffInput,
        SourceProofLocalDeclarationId, SourceProofLocalDeclarationInput,
        SourceProofLocalDeclarationKind, SourceProofLocalDeclarationProducer,
        SourceProofLocalDeclarationRecovery,
    },
    source_set_term::{
        SourceSetConditionInput, SourceSetEdgeInput, SourceSetGeneratorInput,
        SourceSetRequestInput, SourceSetTermHandoffInput, SourceSetTermInput,
        SourceSetTermProducer, SourceSetTypeSiteInput, SourceSetWrapperInput,
    },
    source_structure::{
        SourceFieldUpdateInput, SourceStructureEdgeInput, SourceStructureHandoffInput,
        SourceStructureMemberInput, SourceStructureProducer, SourceStructureRequestInput,
        SourceStructureRootInput, SourceStructureTermInput,
    },
    source_term::{
        SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermInput,
        SourcePrimaryTermProducer, SourcePrimaryTermReferenceInput,
    },
    type_checker::TermFormulaChecker,
    typed_ast::{
        CoercionId, CoercionTable, FactProvenance, FactStatus, InitialObligationId,
        InitialObligationTable, LocalTypeContextId, LocalTypeContextTable, NodeRecoveryState,
        Polarity, StatementTransportTableForTest, TypeDiagnosticId, TypeDiagnosticTable,
        TypeEntryId, TypeFactDraft, TypeFactId, TypeFactTable, TypePredicateRef, TypeRole,
        TypeRuleId, TypeTable, TypedArenaBuilder, TypedAst, TypedAstError, TypedAstParts,
        TypedNode, TypedNodeId, TypedNodeLinks, TypingState,
    },
};
use mizar_resolve::{
    env::{
        DefinitionIndex, DefinitionKind, DefinitionShell, ImportIndexEntry, LabelEntry, LabelIndex,
        NamespacePath, SourceContributionIndex, SymbolEntry, SymbolEnvIndexes, SymbolIndex,
        SymbolKind,
    },
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{
        FullyQualifiedName, ImportResolution, LabelOriginPath, LocalSymbolId, ResolvedArenaBuilder,
        ResolvedImport, ResolvedImports, ResolvedNode, SemanticOrigin,
    },
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
};
use mizar_syntax as syntax;

const LABEL: &str = "FormulaStatementReservedVariableEqualitySmoke";
const STATEMENT: &str = "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;";
type StatementWitnessMutation = (&'static str, fn(&mut SourceStatementWitnessHandoff));
type StructureHandoffMutation = (&'static str, fn(&mut SourceStructureHandoffInput));

#[derive(Clone)]
struct Fixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl Fixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = symbol_env(source, &module);
        let bindings = binding_env(source, &module);
        let arena = typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![
                    SourcePrimaryTermInput {
                        site: node(0),
                        source_range: range(source, 74, 75),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
                    SourcePrimaryTermInput {
                        site: node(1),
                        source_range: range(source, 78, 79),
                        source_ordinal: 1,
                        context: BindingContextId::new(0),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
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
            },
            &bindings,
            &arena,
        )
        .expect("exact primary handoff");
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: vec![SourceAtomicFormulaInput {
                    site: node(3),
                    source_range: range(source, 74, 79),
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
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("exact atomic handoff");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            arena,
        }
    }

    fn input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(5),
                source_range: range(self.source, 19, 80),
                spelling: LABEL.to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: vec![SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(0),
                formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0)),
                site: node(5),
                source_range: range(self.source, 19, 80),
                source_ordinal: 0,
                spelling: STATEMENT.to_owned(),
                kind: SourceStatementKind::TheoremProposition,
                recovery: SourceStatementRecovery::Normal,
            }],
            contexts: vec![SourceStatementContextInput {
                statement: SourceStatementId::new(0),
                binding_context: BindingContextId::new(0),
                source_range: range(self.source, 19, 80),
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

    fn build(
        &self,
        input: SourceStatementHandoffInput,
    ) -> Result<SourceStatementHandoff, SourceStatementError> {
        SourceStatementProducer::build(
            input,
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
    }
}

#[derive(Clone)]
struct B3Fixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

fn b3_primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
    let term_sites = [26, 28, 32, 36, 38];
    let term_ranges = [(63, 64), (67, 68), (82, 83), (92, 93), (96, 97)];
    let term_contexts = [0, 0, 1, 1, 1];
    SourcePrimaryTermHandoffInput {
        source_id: source,
        module_id: module.clone(),
        terms: (0..5)
            .map(|index| SourcePrimaryTermInput {
                site: node(term_sites[index]),
                source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                source_ordinal: index,
                context: BindingContextId::new(term_contexts[index]),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: "x".to_owned(),
                kind: SourcePrimaryTermKind::VariableReference,
                role: SourcePrimaryTermRole::Value,
                parent: None,
            })
            .collect(),
        references: (0..5)
            .map(|index| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: BindingId::new(0),
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

impl B3Fixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b3_symbol_env(source, &module);
        let bindings = b3_binding_env(source, &module);
        let arena = b3_typed_arena(source);
        let primary =
            SourcePrimaryTermProducer::build(b3_primary_input(source, &module), &bindings, &arena)
                .expect("Task258B3 primary terms");
        let formula_sites = [30, 40];
        let formula_ranges = [(63, 68), (92, 97)];
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: (0..2)
                    .map(|index| SourceAtomicFormulaInput {
                        site: node(formula_sites[index]),
                        source_range: range(
                            source,
                            formula_ranges[index].0,
                            formula_ranges[index].1,
                        ),
                        source_ordinal: index,
                        context: BindingContextId::new(index),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    })
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B3 atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            arena,
        }
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        let statement_sites = [45, 43];
        let statement_ranges = [(19, 103), (87, 98)];
        let source_ordinals = [0, 2];
        let spellings = [
            "theorem FormulaStatementSingleWitnessSmoke : x = x proof take x ; thus x = x ; end ;",
            "thus x = x ;",
        ];
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(45),
                source_range: range(self.source, 19, 103),
                spelling: "FormulaStatementSingleWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: (0..2)
                .map(|index| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                        index,
                    )),
                    site: node(statement_sites[index]),
                    source_range: range(
                        self.source,
                        statement_ranges[index].0,
                        statement_ranges[index].1,
                    ),
                    source_ordinal: source_ordinals[index],
                    spelling: spellings[index].to_owned(),
                    kind: if index == 0 {
                        SourceStatementKind::TheoremProposition
                    } else {
                        SourceStatementKind::Conclusion
                    },
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: (0..2)
                .map(|index| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(
                        self.source,
                        statement_ranges[index].0,
                        statement_ranges[index].1,
                    ),
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
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                        index,
                    )),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                take_site: node(35),
                take_range: range(self.source, 77, 84),
                site: node(34),
                source_range: range(self.source, 82, 83),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "x".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3 base statement")
    }

    fn witnesses(
        &self,
        statements: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build(input, statements, &self.primary, &self.arena)
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3 empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3 Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B3 Task256")
    }
}

#[derive(Clone)]
struct B3NFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3NFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b3n_symbol_env(source, &module);
        let bindings = b3n_binding_env(source, &module);
        let arena = b3n_typed_arena(source);
        let term_sites = [28, 30, 34, 38, 40];
        let term_ranges = [(62, 63), (66, 67), (85, 86), (95, 96), (99, 100)];
        let term_contexts = [0, 0, 1, 1, 1];
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: (0..5)
                    .map(|index| SourcePrimaryTermInput {
                        site: node(term_sites[index]),
                        source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                        source_ordinal: index,
                        context: BindingContextId::new(term_contexts[index]),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    })
                    .collect(),
                references: (0..5)
                    .map(|index| SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(index),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    })
                    .collect(),
                numeric_type_requests: Vec::new(),
            },
            &bindings,
            &arena,
        )
        .expect("Task258B3N primary terms");
        let formula_sites = [32, 42];
        let formula_ranges = [(62, 67), (95, 100)];
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: (0..2)
                    .map(|index| SourceAtomicFormulaInput {
                        site: node(formula_sites[index]),
                        source_range: range(
                            source,
                            formula_ranges[index].0,
                            formula_ranges[index].1,
                        ),
                        source_ordinal: index,
                        context: BindingContextId::new(index),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    })
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B3N atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            arena,
        }
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        let statement_sites = [47, 45];
        let statement_ranges = [(19, 106), (90, 101)];
        let spellings = [
            "theorem FormulaStatementNamedWitnessSmoke : x = x proof take y = x ; thus x = x ; end ;",
            "thus x = x ;",
        ];
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(47),
                source_range: range(self.source, 19, 106),
                spelling: "FormulaStatementNamedWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: (0..2)
                .map(|index| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                        index,
                    )),
                    site: node(statement_sites[index]),
                    source_range: range(
                        self.source,
                        statement_ranges[index].0,
                        statement_ranges[index].1,
                    ),
                    source_ordinal: [0, 2][index],
                    spelling: spellings[index].to_owned(),
                    kind: if index == 0 {
                        SourceStatementKind::TheoremProposition
                    } else {
                        SourceStatementKind::Conclusion
                    },
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: (0..2)
                .map(|index| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(
                        self.source,
                        statement_ranges[index].0,
                        statement_ranges[index].1,
                    ),
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
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                        index,
                    )),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                take_site: node(37),
                take_range: range(self.source, 76, 87),
                site: node(36),
                source_range: range(self.source, 81, 86),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "y = x".to_owned(),
                kind: SourceStatementWitnessKind::Named,
                recovery: SourceStatementRecovery::Normal,
                name: Some(SourceStatementWitnessNameId::new(0)),
            }],
            names: vec![SourceStatementWitnessNameInput {
                witness: SourceStatementWitnessId::new(0),
                site: node(13),
                source_range: range(self.source, 81, 82),
                spelling: "y".to_owned(),
                recovery: SourceStatementRecovery::Normal,
            }],
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3N base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build(input, statement, &self.primary, &self.arena)
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3N empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3N Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B3N Task256")
    }

    fn proof_local_input(&self) -> SourceProofLocalDeclarationHandoffInput {
        SourceProofLocalDeclarationHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            declarations: vec![SourceProofLocalDeclarationInput {
                witness: SourceStatementWitnessId::new(0),
                name: SourceStatementWitnessNameId::new(0),
                rhs: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                binding_context: BindingContextId::new(1),
                source_ordinal: 1,
                local: LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    range(self.source, 81, 82),
                    1,
                ),
                kind: SourceProofLocalDeclarationKind::NamedWitness,
                recovery: SourceProofLocalDeclarationRecovery::Normal,
            }],
        }
    }

    fn proof_local(
        &self,
        statement: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        input: SourceProofLocalDeclarationHandoffInput,
    ) -> Result<SourceProofLocalDeclarationHandoff, SourceProofLocalDeclarationError> {
        SourceProofLocalDeclarationProducer::build(
            input,
            statement,
            witnesses,
            &self.primary,
            &self.arena,
        )
    }
}

#[derive(Clone)]
struct B3M1Fixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M1Fixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b3m1_symbol_env(source, &module);
        let bindings = b3m1_binding_env(source, &module);
        let arena = b3m1_typed_arena(source);
        let term_sites = [30, 32, 36, 39, 43, 45];
        let term_ranges = [
            (65, 66),
            (69, 70),
            (88, 89),
            (91, 92),
            (101, 102),
            (105, 106),
        ];
        let term_contexts = [0, 0, 1, 1, 1, 1];
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: (0..6)
                    .map(|index| SourcePrimaryTermInput {
                        site: node(term_sites[index]),
                        source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                        source_ordinal: index,
                        context: BindingContextId::new(term_contexts[index]),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    })
                    .collect(),
                references: (0..6)
                    .map(|index| SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(index),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    })
                    .collect(),
                numeric_type_requests: Vec::new(),
            },
            &bindings,
            &arena,
        )
        .expect("Task258B3M1 primary terms");
        let formula_sites = [34, 47];
        let formula_ranges = [(65, 70), (101, 106)];
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
            let first_term = if formula == 0 { 0 } else { 4 };
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: (0..2)
                    .map(|index| SourceAtomicFormulaInput {
                        site: node(formula_sites[index]),
                        source_range: range(
                            source,
                            formula_ranges[index].0,
                            formula_ranges[index].1,
                        ),
                        source_ordinal: index,
                        context: BindingContextId::new(index),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    })
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M1 atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            arena,
        }
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        let statement_sites = [52, 50];
        let statement_ranges = [(19, 112), (96, 107)];
        let spellings = [
            "theorem FormulaStatementMultipleWitnessSmoke : x = x proof take y = x , x ; thus x = x ; end ;",
            "thus x = x ;",
        ];
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(52),
                source_range: range(self.source, 19, 112),
                spelling: "FormulaStatementMultipleWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: (0..2)
                .map(|index| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                        index,
                    )),
                    site: node(statement_sites[index]),
                    source_range: range(
                        self.source,
                        statement_ranges[index].0,
                        statement_ranges[index].1,
                    ),
                    source_ordinal: [0, 2][index],
                    spelling: spellings[index].to_owned(),
                    kind: if index == 0 {
                        SourceStatementKind::TheoremProposition
                    } else {
                        SourceStatementKind::Conclusion
                    },
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: (0..2)
                .map(|index| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(
                        self.source,
                        statement_ranges[index].0,
                        statement_ranges[index].1,
                    ),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: (0..2)
                .map(|index| {
                    let first_term = if index == 0 { 0 } else { 4 };
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
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                        index,
                    )),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![
                SourceStatementWitnessInput {
                    owner: SourceTheoremOwnerId::new(0),
                    binding_context: BindingContextId::new(1),
                    term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                    take_site: node(42),
                    take_range: range(self.source, 79, 93),
                    site: node(38),
                    source_range: range(self.source, 84, 89),
                    source_ordinal: 1,
                    ordinal: 0,
                    spelling: "y = x".to_owned(),
                    kind: SourceStatementWitnessKind::Named,
                    recovery: SourceStatementRecovery::Normal,
                    name: Some(SourceStatementWitnessNameId::new(0)),
                },
                SourceStatementWitnessInput {
                    owner: SourceTheoremOwnerId::new(0),
                    binding_context: BindingContextId::new(1),
                    term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(3)),
                    take_site: node(42),
                    take_range: range(self.source, 79, 93),
                    site: node(41),
                    source_range: range(self.source, 91, 92),
                    source_ordinal: 1,
                    ordinal: 1,
                    spelling: "x".to_owned(),
                    kind: SourceStatementWitnessKind::Unnamed,
                    recovery: SourceStatementRecovery::Normal,
                    name: None,
                },
            ],
            names: vec![SourceStatementWitnessNameInput {
                witness: SourceStatementWitnessId::new(0),
                site: node(13),
                source_range: range(self.source, 84, 85),
                spelling: "y".to_owned(),
                recovery: SourceStatementRecovery::Normal,
            }],
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M1 base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build(input, statement, &self.primary, &self.arena)
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M1 empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3M1 Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B3M1 Task256")
    }

    fn proof_local_input(&self) -> SourceProofLocalDeclarationHandoffInput {
        SourceProofLocalDeclarationHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            declarations: vec![SourceProofLocalDeclarationInput {
                witness: SourceStatementWitnessId::new(0),
                name: SourceStatementWitnessNameId::new(0),
                rhs: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                binding_context: BindingContextId::new(1),
                source_ordinal: 1,
                local: LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    range(self.source, 84, 85),
                    1,
                ),
                kind: SourceProofLocalDeclarationKind::NamedWitness,
                recovery: SourceProofLocalDeclarationRecovery::Normal,
            }],
        }
    }

    fn proof_local(
        &self,
        statement: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        input: SourceProofLocalDeclarationHandoffInput,
    ) -> Result<SourceProofLocalDeclarationHandoff, SourceProofLocalDeclarationError> {
        SourceProofLocalDeclarationProducer::build(
            input,
            statement,
            witnesses,
            &self.primary,
            &self.arena,
        )
    }
}

#[derive(Clone)]
struct B3M2AFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2AFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b3m2a_symbol_env(source, &module);
        let bindings = b3m2a_binding_env(source, &module);
        let arena = b3m2a_typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2A primary terms");
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: [(30, 64, 69, 0), (40, 95, 100, 1)]
                    .into_iter()
                    .enumerate()
                    .map(
                        |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                            site: node(site),
                            source_range: range(source, start, end),
                            source_ordinal,
                            context: BindingContextId::new(context),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        },
                    )
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2A atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: [
                (26, 64, 65, 0),
                (28, 68, 69, 0),
                (32, 83, 86, 1),
                (36, 95, 96, 1),
                (38, 99, 100, 1),
            ]
            .into_iter()
            .enumerate()
            .map(|(source_ordinal, (site, start, end, context))| {
                let numeral = source_ordinal == 2;
                SourcePrimaryTermInput {
                    site: node(site),
                    source_range: range(source, start, end),
                    source_ordinal,
                    context: BindingContextId::new(context),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: if numeral { "101" } else { "x" }.to_owned(),
                    kind: if numeral {
                        SourcePrimaryTermKind::Numeral
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }
            })
            .collect(),
            references: [0, 1, 3, 4]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: vec![crate::source_term::SourceNumericTypeRequestInput {
                term: SourcePrimaryTermId::new(2),
                owner: node(32),
                source_range: range(source, 83, 86),
                spelling: "101".to_owned(),
                request_ordinal: 0,
            }],
        }
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(45),
                source_range: range(self.source, 19, 106),
                spelling: "FormulaStatementNumeralWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(45, 19, 106, 0), (43, 90, 101, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementNumeralWitnessSmoke : x = x proof take 101 ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(19, 106), (90, 101)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                take_site: node(35),
                take_range: range(self.source, 78, 87),
                site: node(34),
                source_range: range(self.source, 83, 86),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "101".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2A base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build(input, statement, &self.primary, &self.arena)
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2A empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3M2A Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B3M2A Task256")
    }
}

#[derive(Clone)]
struct B3M2B1Fixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B1Fixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b3m2b1_symbol_env(source, &module);
        let bindings = b3m2b1_binding_env(source, &module);
        let arena = b3m2b1_typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B1 primary terms");
        let atomic = Self::build_atomic(
            source,
            &module,
            &bindings,
            &symbols,
            &primary,
            &arena,
            [0, 4],
        )
        .expect("Task258B3M2B1 atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let sites = [28, 30, 36, 34, 40, 42];
        let ranges = [
            (70, 71),
            (74, 75),
            (89, 92),
            (90, 91),
            (101, 102),
            (105, 106),
        ];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..6)
                .map(|index| {
                    let parenthesized = index == 2;
                    SourcePrimaryTermInput {
                        site: node(sites[index]),
                        source_range: range(source, ranges[index].0, ranges[index].1),
                        source_ordinal: index,
                        context: BindingContextId::new(usize::from(index >= 2)),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: if parenthesized { "( x )" } else { "x" }.to_owned(),
                        kind: if parenthesized {
                            SourcePrimaryTermKind::Parenthesized
                        } else {
                            SourcePrimaryTermKind::VariableReference
                        },
                        role: SourcePrimaryTermRole::Value,
                        parent: (index == 3).then(|| SourcePrimaryTermId::new(2)),
                    }
                })
                .collect(),
            references: [0, 1, 3, 4, 5]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)] // Rationale: preserve the exact syntax-free dependency tuple in test construction.
    fn build_atomic(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
        first_terms: [usize; 2],
    ) -> Result<SourceAtomicFormulaHandoff, crate::source_atomic_formula::SourceAtomicFormulaError>
    {
        Self::build_atomic_targets(
            source,
            module,
            bindings,
            symbols,
            primary,
            arena,
            [
                first_terms[0],
                first_terms[0] + 1,
                first_terms[1],
                first_terms[1] + 1,
            ],
        )
    }

    #[allow(clippy::too_many_arguments)] // Rationale: vary only the four frozen atomic targets while retaining every dependency.
    fn build_atomic_targets(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
        targets: [usize; 4],
    ) -> Result<SourceAtomicFormulaHandoff, crate::source_atomic_formula::SourceAtomicFormulaError>
    {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
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
                        targets[formula * 2 + ordinal],
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
        SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: [(32, 70, 75, 0), (44, 101, 106, 1)]
                    .into_iter()
                    .enumerate()
                    .map(
                        |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                            site: node(site),
                            source_range: range(source, start, end),
                            source_ordinal,
                            context: BindingContextId::new(context),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        },
                    )
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            bindings,
            symbols,
            primary,
            None,
            None,
            None,
            arena,
        )
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(49),
                source_range: range(self.source, 19, 112),
                spelling: "FormulaStatementParenthesizedWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(49, 19, 112, 0), (47, 96, 107, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementParenthesizedWitnessSmoke : x = x proof take ( x ) ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(19, 112), (96, 107)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (3, 4)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                take_site: node(39),
                take_range: range(self.source, 84, 93),
                site: node(38),
                source_range: range(self.source, 89, 92),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "( x )".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B1 base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build(input, statement, &self.primary, &self.arena)
    }

    fn primary_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B1 primary-only typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3M2B1 Task252")
    }

    fn empty_typed(&self) -> TypedAst {
        self.primary_typed()
            .with_source_atomic_formula(self.atomic.clone())
            .expect("Task258B3M2B1 Task256")
    }
}

#[derive(Clone)]
struct B3M2B2AFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2AFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b3m2b2a_symbol_env(source, &module);
        let bindings = b3m2b2a_binding_env(source, &module);
        let arena = b3m2b2a_typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2A primary terms");
        let atomic = Self::build_atomic(
            source,
            &module,
            &bindings,
            &symbols,
            &primary,
            &arena,
            [0, 5],
        )
        .expect("Task258B3M2B2A atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let sites = [30, 32, 40, 38, 36, 44, 46];
        let ranges = [
            (76, 77),
            (80, 81),
            (95, 100),
            (96, 99),
            (97, 98),
            (109, 110),
            (113, 114),
        ];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..7)
                .map(|index| SourcePrimaryTermInput {
                    site: node(sites[index]),
                    source_range: range(source, ranges[index].0, ranges[index].1),
                    source_ordinal: index,
                    context: BindingContextId::new(usize::from(index >= 2)),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: match index {
                        2 => "( ( x ) )",
                        3 => "( x )",
                        _ => "x",
                    }
                    .to_owned(),
                    kind: if matches!(index, 2 | 3) {
                        SourcePrimaryTermKind::Parenthesized
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: match index {
                        3 => Some(SourcePrimaryTermId::new(2)),
                        4 => Some(SourcePrimaryTermId::new(3)),
                        _ => None,
                    },
                })
                .collect(),
            references: [0, 1, 4, 5, 6]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)] // Rationale: preserve the exact syntax-free dependency tuple in test construction.
    fn build_atomic(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
        first_terms: [usize; 2],
    ) -> Result<SourceAtomicFormulaHandoff, crate::source_atomic_formula::SourceAtomicFormulaError>
    {
        Self::build_atomic_targets(
            source,
            module,
            bindings,
            symbols,
            primary,
            arena,
            [
                first_terms[0],
                first_terms[0] + 1,
                first_terms[1],
                first_terms[1] + 1,
            ],
        )
    }

    #[allow(clippy::too_many_arguments)] // Rationale: vary only the four frozen atomic targets while retaining every dependency.
    fn build_atomic_targets(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
        targets: [usize; 4],
    ) -> Result<SourceAtomicFormulaHandoff, crate::source_atomic_formula::SourceAtomicFormulaError>
    {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
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
                        targets[formula * 2 + ordinal],
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
        SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: [(34, 76, 81, 0), (48, 109, 114, 1)]
                    .into_iter()
                    .enumerate()
                    .map(
                        |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                            site: node(site),
                            source_range: range(source, start, end),
                            source_ordinal,
                            context: BindingContextId::new(context),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        },
                    )
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            bindings,
            symbols,
            primary,
            None,
            None,
            None,
            arena,
        )
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(53),
                source_range: range(self.source, 19, 120),
                spelling: "FormulaStatementNestedParenthesizedWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(53, 19, 120, 0), (51, 104, 115, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementNestedParenthesizedWitnessSmoke : x = x proof take ( ( x ) ) ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(19, 120), (104, 115)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (3, 4)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                take_site: node(43),
                take_range: range(self.source, 90, 101),
                site: node(42),
                source_range: range(self.source, 95, 100),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "( ( x ) )".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2A base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build(input, statement, &self.primary, &self.arena)
    }

    fn primary_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2A primary-only typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3M2B2A Task252")
    }

    fn empty_typed(&self) -> TypedAst {
        self.primary_typed()
            .with_source_atomic_formula(self.atomic.clone())
            .expect("Task258B3M2B2A Task256")
    }
}

#[derive(Clone)]
struct B3M2B2B1AFixture {
    source: SourceId,
    module: ModuleId,
    theorem_symbol: SymbolId,
    theorem_contribution: SourceContributionId,
    functor_symbol: SymbolId,
    functor_contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    application: SourceFunctorApplicationHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B1AFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (theorem_symbol, theorem_contribution, functor_symbol, functor_contribution, symbols) =
            b3m2b2b1a_symbol_env(source, &module);
        let bindings = b3m2b2b1a_binding_env(source, &module);
        let arena = b3m2b2b1a_typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B1A primary");
        let application = SourceFunctorApplicationProducer::build(
            Self::application_input(
                source,
                &module,
                functor_symbol.clone(),
                functor_contribution,
            ),
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("Task258B3M2B2B1A application");
        let atomic = Self::build_atomic(
            source,
            &module,
            &bindings,
            &symbols,
            &primary,
            &application,
            &arena,
        );
        Self {
            source,
            module,
            theorem_symbol,
            theorem_contribution,
            functor_symbol,
            functor_contribution,
            symbols,
            bindings,
            primary,
            application,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let sites = [38, 40, 44, 45, 50, 52];
        let ranges = [
            (97, 98),
            (101, 102),
            (116, 117),
            (121, 122),
            (131, 132),
            (135, 136),
        ];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..6)
                .map(|index| SourcePrimaryTermInput {
                    site: node(sites[index]),
                    source_range: range(source, ranges[index].0, ranges[index].1),
                    source_ordinal: index,
                    context: BindingContextId::new(usize::from(index >= 2)),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: match index {
                        2 => "1",
                        3 => "2",
                        _ => "x",
                    }
                    .to_owned(),
                    kind: if matches!(index, 2 | 3) {
                        SourcePrimaryTermKind::Numeral
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: [0, 1, 4, 5]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: [2, 3]
                .into_iter()
                .enumerate()
                .map(|(request_ordinal, term)| SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(term),
                    owner: node(sites[term]),
                    source_range: range(source, ranges[term].0, ranges[term].1),
                    spelling: if term == 2 { "1" } else { "2" }.to_owned(),
                    request_ordinal,
                })
                .collect(),
        }
    }

    fn application_input(
        source: SourceId,
        module: &ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
    ) -> SourceFunctorApplicationHandoffInput {
        SourceFunctorApplicationHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceFunctorApplicationInput {
                site: node(46),
                source_range: range(source, 116, 122),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceFunctorApplicationRecovery::Normal,
                spelling: "1 ++ 2".to_owned(),
                kind: SourceFunctorApplicationKind::Symbolic,
                form: SourceFunctorApplicationForm::Infix,
                head_ordinal: 1,
                head: SourceFunctorHeadSite::Single {
                    site: node(19),
                    source_range: range(source, 118, 120),
                    spelling: "++".to_owned(),
                },
            }],
            wrappers: Vec::new(),
            candidates: vec![SourceFunctorCandidateInput {
                application: SourceFunctorApplicationId::new(0),
                ordinal: 0,
                symbol,
                contribution,
            }],
            arguments: [2, 3]
                .into_iter()
                .enumerate()
                .map(|(ordinal, term)| SourceFunctorArgumentInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal,
                    target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(term)),
                })
                .collect(),
            type_requests: vec![
                SourceFunctorTypeRequestInput {
                    application: SourceFunctorApplicationId::new(0),
                    candidate: Some(SourceFunctorCandidateId::new(0)),
                    request_ordinal: 0,
                    kind: SourceFunctorTypeRequestKind::CandidateSignature,
                },
                SourceFunctorTypeRequestInput {
                    application: SourceFunctorApplicationId::new(0),
                    candidate: None,
                    request_ordinal: 1,
                    kind: SourceFunctorTypeRequestKind::ApplicationResultType,
                },
            ],
        }
    }

    #[allow(clippy::too_many_arguments)] // Rationale: preserve the exact B1A lower-family dependency tuple in test construction.
    fn build_atomic(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary: &SourcePrimaryTermHandoff,
        application: &SourceFunctorApplicationHandoff,
        arena: &TypedArena,
    ) -> SourceAtomicFormulaHandoff {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
            let first_term = if formula == 0 { 0 } else { 4 };
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
        SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: [(42, 97, 102, 0), (54, 131, 136, 1)]
                    .into_iter()
                    .enumerate()
                    .map(
                        |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                            site: node(site),
                            source_range: range(source, start, end),
                            source_ordinal,
                            context: BindingContextId::new(context),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        },
                    )
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            bindings,
            symbols,
            primary,
            Some(application),
            None,
            None,
            arena,
        )
        .expect("Task258B3M2B2B1A atomic")
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.theorem_symbol.clone(),
                contribution: self.theorem_contribution,
                site: node(59),
                source_range: range(self.source, 48, 142),
                spelling: "FormulaStatementApplicationWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(59, 48, 142, 0), (57, 126, 137, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementApplicationWitnessSmoke : x = x proof take 1 ++ 2 ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(48, 142), (126, 137)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Application(
                    SourceFunctorApplicationId::new(0),
                ),
                take_site: node(49),
                take_range: range(self.source, 111, 123),
                site: node(48),
                source_range: range(self.source, 116, 122),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "1 ++ 2".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B1A statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_application(
            input,
            statement,
            &self.primary,
            &self.application,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B1A typed")
        .with_source_term(self.primary.clone())
        .expect("Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task256")
    }
}

#[derive(Clone)]
struct B3M2B2B1B1Fixture {
    source: SourceId,
    module: ModuleId,
    theorem_symbol: SymbolId,
    theorem_contribution: SourceContributionId,
    functor_symbol: SymbolId,
    functor_contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    application: SourceFunctorApplicationHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B1B1Fixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (theorem_symbol, theorem_contribution, functor_symbol, functor_contribution, symbols) =
            b3m2b2b1b1_symbol_env(source, &module);
        let bindings = b3m2b2b1b1_binding_env(source, &module);
        let arena = b3m2b2b1b1_typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B1B1 primary");
        let application = SourceFunctorApplicationProducer::build(
            Self::application_input(
                source,
                &module,
                functor_symbol.clone(),
                functor_contribution,
            ),
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("Task258B3M2B2B1B1 application");
        let atomic = Self::build_atomic(
            source,
            &module,
            &bindings,
            &symbols,
            &primary,
            &application,
            &arena,
        );
        Self {
            source,
            module,
            theorem_symbol,
            theorem_contribution,
            functor_symbol,
            functor_contribution,
            symbols,
            bindings,
            primary,
            application,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let sites = [40, 42, 46, 47, 54, 56];
        let ranges = [
            (110, 111),
            (114, 115),
            (130, 131),
            (135, 136),
            (146, 147),
            (150, 151),
        ];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..6)
                .map(|index| SourcePrimaryTermInput {
                    site: node(sites[index]),
                    source_range: range(source, ranges[index].0, ranges[index].1),
                    source_ordinal: index,
                    context: BindingContextId::new(usize::from(index >= 2)),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: match index {
                        2 => "1",
                        3 => "2",
                        _ => "x",
                    }
                    .to_owned(),
                    kind: if matches!(index, 2 | 3) {
                        SourcePrimaryTermKind::Numeral
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: [0, 1, 4, 5]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: [2, 3]
                .into_iter()
                .enumerate()
                .map(|(request_ordinal, term)| SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(term),
                    owner: node(sites[term]),
                    source_range: range(source, ranges[term].0, ranges[term].1),
                    spelling: if term == 2 { "1" } else { "2" }.to_owned(),
                    request_ordinal,
                })
                .collect(),
        }
    }

    fn application_input(
        source: SourceId,
        module: &ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
    ) -> SourceFunctorApplicationHandoffInput {
        SourceFunctorApplicationHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceFunctorApplicationInput {
                site: node(48),
                source_range: range(source, 130, 136),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceFunctorApplicationRecovery::Normal,
                spelling: "1 ++ 2".to_owned(),
                kind: SourceFunctorApplicationKind::Symbolic,
                form: SourceFunctorApplicationForm::Infix,
                head_ordinal: 1,
                head: SourceFunctorHeadSite::Single {
                    site: node(20),
                    source_range: range(source, 132, 134),
                    spelling: "++".to_owned(),
                },
            }],
            wrappers: vec![SourceFunctorWrapperInput {
                application: SourceFunctorApplicationId::new(0),
                ordinal: 0,
                site: node(50),
                source_range: range(source, 129, 137),
                context: BindingContextId::new(1),
                spelling: "( 1 ++ 2 )".to_owned(),
                recovery: SourceFunctorApplicationRecovery::Normal,
            }],
            candidates: vec![SourceFunctorCandidateInput {
                application: SourceFunctorApplicationId::new(0),
                ordinal: 0,
                symbol,
                contribution,
            }],
            arguments: [2, 3]
                .into_iter()
                .enumerate()
                .map(|(ordinal, term)| SourceFunctorArgumentInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal,
                    target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(term)),
                })
                .collect(),
            type_requests: vec![
                SourceFunctorTypeRequestInput {
                    application: SourceFunctorApplicationId::new(0),
                    candidate: Some(SourceFunctorCandidateId::new(0)),
                    request_ordinal: 0,
                    kind: SourceFunctorTypeRequestKind::CandidateSignature,
                },
                SourceFunctorTypeRequestInput {
                    application: SourceFunctorApplicationId::new(0),
                    candidate: None,
                    request_ordinal: 1,
                    kind: SourceFunctorTypeRequestKind::ApplicationResultType,
                },
            ],
        }
    }

    #[allow(clippy::too_many_arguments)] // Rationale: preserve the exact wrapped lower-family dependency tuple in test construction.
    fn build_atomic(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary: &SourcePrimaryTermHandoff,
        application: &SourceFunctorApplicationHandoff,
        arena: &TypedArena,
    ) -> SourceAtomicFormulaHandoff {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
            let first_term = if formula == 0 { 0 } else { 4 };
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
        SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: [(44, 110, 115, 0), (58, 146, 151, 1)]
                    .into_iter()
                    .enumerate()
                    .map(
                        |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                            site: node(site),
                            source_range: range(source, start, end),
                            source_ordinal,
                            context: BindingContextId::new(context),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        },
                    )
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            bindings,
            symbols,
            primary,
            Some(application),
            None,
            None,
            arena,
        )
        .expect("Task258B3M2B2B1B1 atomic")
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.theorem_symbol.clone(),
                contribution: self.theorem_contribution,
                site: node(63),
                source_range: range(self.source, 48, 157),
                spelling: "FormulaStatementParenthesizedApplicationWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(63, 48, 157, 0), (61, 141, 152, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementParenthesizedApplicationWitnessSmoke : x = x proof take ( 1 ++ 2 ) ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(48, 157), (141, 152)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Application(
                    SourceFunctorApplicationId::new(0),
                ),
                take_site: node(53),
                take_range: range(self.source, 124, 138),
                site: node(52),
                source_range: range(self.source, 129, 137),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "( 1 ++ 2 )".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B1B1 statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_application(
            input,
            statement,
            &self.primary,
            &self.application,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B1B1 typed")
        .with_source_term(self.primary.clone())
        .expect("Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task256")
    }
}

const TASK258B3M2B2B2A_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureConstructorWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2);\n",
    "  thus x = x;\n",
    "end;\n",
);

#[derive(Clone)]
struct B3M2B2B2AFixture {
    source: SourceId,
    module: ModuleId,
    theorem_symbol: SymbolId,
    theorem_contribution: SourceContributionId,
    structure_symbol: SymbolId,
    structure_contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    structure: SourceStructureHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B2AFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (
            theorem_symbol,
            theorem_contribution,
            structure_symbol,
            structure_contribution,
            symbols,
        ) = b3m2b2b2a_symbol_env(source, &module);
        let bindings = b3m2b2b2a_binding_env(source, &module);
        let arena = b3m2b2b2a_typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B2A primary");
        let structure = SourceStructureProducer::build(
            Self::structure_input(
                source,
                &module,
                structure_symbol.clone(),
                structure_contribution,
            ),
            &symbols,
            &bindings,
            &primary,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B2A structure");
        let atomic = Self::build_atomic(source, &module, &bindings, &symbols, &primary, &arena);
        Self {
            source,
            module,
            theorem_symbol,
            theorem_contribution,
            structure_symbol,
            structure_contribution,
            symbols,
            bindings,
            primary,
            structure,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let sites = [45, 47, 53, 56, 63, 65];
        let ranges = [
            (106, 107),
            (110, 111),
            (143, 144),
            (149, 150),
            (160, 161),
            (164, 165),
        ];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..6)
                .map(|index| SourcePrimaryTermInput {
                    site: node(sites[index]),
                    source_range: range(source, ranges[index].0, ranges[index].1),
                    source_ordinal: index,
                    context: BindingContextId::new(usize::from(index >= 2)),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: match index {
                        2 => "1",
                        3 => "2",
                        _ => "x",
                    }
                    .to_owned(),
                    kind: if matches!(index, 2 | 3) {
                        SourcePrimaryTermKind::Numeral
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: [0, 1, 4, 5]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: [2, 3]
                .into_iter()
                .enumerate()
                .map(|(request_ordinal, term)| SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(term),
                    owner: node(sites[term]),
                    source_range: range(source, ranges[term].0, ranges[term].1),
                    spelling: if term == 2 { "1" } else { "2" }.to_owned(),
                    request_ordinal,
                })
                .collect(),
        }
    }

    fn structure_input(
        source: SourceId,
        module: &ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
    ) -> SourceStructureHandoffInput {
        SourceStructureHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceStructureTermInput {
                site: node(59),
                source_range: range(source, 125, 151),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceStructureRecovery::Normal,
                spelling: "TypeCaseStruct ( x : 1 , y : 2 )".to_owned(),
                kind: SourceStructureTermKind::Constructor,
            }],
            wrappers: Vec::new(),
            roots: vec![SourceStructureRootInput {
                term: SourceStructureTermId::new(0),
                symbol,
                contribution,
            }],
            members: [(20, 140, 141, "x"), (24, 146, 147, "y")]
                .into_iter()
                .enumerate()
                .map(
                    |(ordinal, (site, start, end, spelling))| SourceStructureMemberInput {
                        term: SourceStructureTermId::new(0),
                        ordinal,
                        site: node(site),
                        source_range: range(source, start, end),
                        spelling: spelling.to_owned(),
                        role: SourceStructureMemberRole::ConstructorAssignment,
                        parent: None,
                    },
                )
                .collect(),
            field_updates: Vec::new(),
            edges: (0..2)
                .map(|ordinal| SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal,
                    role: SourceStructureEdgeRole::ConstructorValue,
                    member: Some(SourceStructureMemberId::new(ordinal)),
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(ordinal + 2)),
                })
                .collect(),
            requests: [
                (None, SourceStructureRequestKind::ConstructorSignature),
                (
                    Some(SourceStructureMemberId::new(0)),
                    SourceStructureRequestKind::MemberIdentity,
                ),
                (
                    Some(SourceStructureMemberId::new(0)),
                    SourceStructureRequestKind::InheritancePath,
                ),
                (
                    Some(SourceStructureMemberId::new(1)),
                    SourceStructureRequestKind::MemberIdentity,
                ),
                (
                    Some(SourceStructureMemberId::new(1)),
                    SourceStructureRequestKind::InheritancePath,
                ),
                (None, SourceStructureRequestKind::ResultType),
            ]
            .into_iter()
            .enumerate()
            .map(
                |(request_ordinal, (member, kind))| SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member,
                    request_ordinal,
                    kind,
                },
            )
            .collect(),
        }
    }

    fn build_atomic(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> SourceAtomicFormulaHandoff {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
            let first_term = if formula == 0 { 0 } else { 4 };
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
        SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: [(49, 106, 111, 0), (67, 160, 165, 1)]
                    .into_iter()
                    .enumerate()
                    .map(
                        |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                            site: node(site),
                            source_range: range(source, start, end),
                            source_ordinal,
                            context: BindingContextId::new(context),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        },
                    )
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            bindings,
            symbols,
            primary,
            None,
            None,
            None,
            arena,
        )
        .expect("Task258B3M2B2B2A atomic")
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.theorem_symbol.clone(),
                contribution: self.theorem_contribution,
                site: node(72),
                source_range: range(self.source, 48, 171),
                spelling: "FormulaStatementStructureConstructorWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(72, 48, 171, 0), (70, 155, 166, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementStructureConstructorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(48, 171), (155, 166)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0)),
                take_site: node(62),
                take_range: range(self.source, 120, 152),
                site: node(61),
                source_range: range(self.source, 125, 151),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "TypeCaseStruct ( x : 1 , y : 2 )".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B2A statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_structure(
            input,
            statement,
            &self.primary,
            &self.structure,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B2A typed")
        .with_source_term(self.primary.clone())
        .expect("Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task256")
    }
}

const TASK258B3M2B2B2B_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureSelectorWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2).x;\n",
    "  thus x = x;\n",
    "end;\n",
);

#[derive(Clone)]
struct B3M2B2B2BFixture {
    source: SourceId,
    module: ModuleId,
    theorem_symbol: SymbolId,
    theorem_contribution: SourceContributionId,
    structure_symbol: SymbolId,
    structure_contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    structure: SourceStructureHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B2BFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (
            theorem_symbol,
            theorem_contribution,
            structure_symbol,
            structure_contribution,
            symbols,
        ) = b3m2b2b2b_symbol_env(source, &module);
        let bindings = b3m2b2b2b_binding_env(source, &module);
        let arena = b3m2b2b2b_typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B2B primary");
        let structure = SourceStructureProducer::build(
            Self::structure_input(
                source,
                &module,
                structure_symbol.clone(),
                structure_contribution,
            ),
            &symbols,
            &bindings,
            &primary,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B2B structure");
        let atomic = Self::build_atomic(source, &module, &bindings, &symbols, &primary, &arena);
        Self {
            source,
            module,
            theorem_symbol,
            theorem_contribution,
            structure_symbol,
            structure_contribution,
            symbols,
            bindings,
            primary,
            structure,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let sites = [47, 49, 55, 58, 66, 68];
        let ranges = [
            (103, 104),
            (107, 108),
            (140, 141),
            (146, 147),
            (159, 160),
            (163, 164),
        ];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..6)
                .map(|index| SourcePrimaryTermInput {
                    site: node(sites[index]),
                    source_range: range(source, ranges[index].0, ranges[index].1),
                    source_ordinal: index,
                    context: BindingContextId::new(usize::from(index >= 2)),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: match index {
                        2 => "1",
                        3 => "2",
                        _ => "x",
                    }
                    .to_owned(),
                    kind: if matches!(index, 2 | 3) {
                        SourcePrimaryTermKind::Numeral
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: [0, 1, 4, 5]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: [2, 3]
                .into_iter()
                .enumerate()
                .map(|(request_ordinal, term)| SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(term),
                    owner: node(sites[term]),
                    source_range: range(source, ranges[term].0, ranges[term].1),
                    spelling: if term == 2 { "1" } else { "2" }.to_owned(),
                    request_ordinal,
                })
                .collect(),
        }
    }

    fn structure_input(
        source: SourceId,
        module: &ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
    ) -> SourceStructureHandoffInput {
        SourceStructureHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![
                SourceStructureTermInput {
                    site: node(62),
                    source_range: range(source, 122, 150),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "TypeCaseStruct ( x : 1 , y : 2 ) . x".to_owned(),
                    kind: SourceStructureTermKind::SelectorAccess,
                },
                SourceStructureTermInput {
                    site: node(61),
                    source_range: range(source, 122, 148),
                    source_ordinal: 1,
                    context: BindingContextId::new(1),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "TypeCaseStruct ( x : 1 , y : 2 )".to_owned(),
                    kind: SourceStructureTermKind::Constructor,
                },
            ],
            wrappers: Vec::new(),
            roots: vec![SourceStructureRootInput {
                term: SourceStructureTermId::new(1),
                symbol,
                contribution,
            }],
            members: [
                (
                    SourceStructureTermId::new(0),
                    0,
                    29,
                    149,
                    150,
                    "x",
                    SourceStructureMemberRole::Selector,
                ),
                (
                    SourceStructureTermId::new(1),
                    0,
                    20,
                    137,
                    138,
                    "x",
                    SourceStructureMemberRole::ConstructorAssignment,
                ),
                (
                    SourceStructureTermId::new(1),
                    1,
                    24,
                    143,
                    144,
                    "y",
                    SourceStructureMemberRole::ConstructorAssignment,
                ),
            ]
            .into_iter()
            .map(
                |(term, ordinal, site, start, end, spelling, role)| SourceStructureMemberInput {
                    term,
                    ordinal,
                    site: node(site),
                    source_range: range(source, start, end),
                    spelling: spelling.to_owned(),
                    role,
                    parent: None,
                },
            )
            .collect(),
            field_updates: Vec::new(),
            edges: vec![
                SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::SelectorBase,
                    member: None,
                    target: SourceStructureTarget::Structure(SourceStructureTermId::new(1)),
                },
                SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(1),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::ConstructorValue,
                    member: Some(SourceStructureMemberId::new(1)),
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(2)),
                },
                SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(1),
                    ordinal: 1,
                    role: SourceStructureEdgeRole::ConstructorValue,
                    member: Some(SourceStructureMemberId::new(2)),
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(3)),
                },
            ],
            requests: [
                (
                    SourceStructureTermId::new(0),
                    Some(SourceStructureMemberId::new(0)),
                    SourceStructureRequestKind::MemberIdentity,
                ),
                (
                    SourceStructureTermId::new(0),
                    Some(SourceStructureMemberId::new(0)),
                    SourceStructureRequestKind::InheritancePath,
                ),
                (
                    SourceStructureTermId::new(0),
                    None,
                    SourceStructureRequestKind::ResultType,
                ),
                (
                    SourceStructureTermId::new(1),
                    None,
                    SourceStructureRequestKind::ConstructorSignature,
                ),
                (
                    SourceStructureTermId::new(1),
                    Some(SourceStructureMemberId::new(1)),
                    SourceStructureRequestKind::MemberIdentity,
                ),
                (
                    SourceStructureTermId::new(1),
                    Some(SourceStructureMemberId::new(1)),
                    SourceStructureRequestKind::InheritancePath,
                ),
                (
                    SourceStructureTermId::new(1),
                    Some(SourceStructureMemberId::new(2)),
                    SourceStructureRequestKind::MemberIdentity,
                ),
                (
                    SourceStructureTermId::new(1),
                    Some(SourceStructureMemberId::new(2)),
                    SourceStructureRequestKind::InheritancePath,
                ),
                (
                    SourceStructureTermId::new(1),
                    None,
                    SourceStructureRequestKind::ResultType,
                ),
            ]
            .into_iter()
            .enumerate()
            .map(
                |(request_ordinal, (term, member, kind))| SourceStructureRequestInput {
                    term,
                    member,
                    request_ordinal: if term == SourceStructureTermId::new(0) {
                        request_ordinal
                    } else {
                        request_ordinal - 3
                    },
                    kind,
                },
            )
            .collect(),
        }
    }

    fn build_atomic(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> SourceAtomicFormulaHandoff {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
            let first_term = if formula == 0 { 0 } else { 4 };
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
        SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: [(51, 103, 108, 0), (70, 159, 164, 1)]
                    .into_iter()
                    .enumerate()
                    .map(
                        |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                            site: node(site),
                            source_range: range(source, start, end),
                            source_ordinal,
                            context: BindingContextId::new(context),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        },
                    )
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            bindings,
            symbols,
            primary,
            None,
            None,
            None,
            arena,
        )
        .expect("Task258B3M2B2B2B atomic")
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.theorem_symbol.clone(),
                contribution: self.theorem_contribution,
                site: node(75),
                source_range: range(self.source, 48, 170),
                spelling: "FormulaStatementStructureSelectorWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(75, 48, 170, 0), (73, 154, 165, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementStructureSelectorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) . x ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(48, 170), (154, 165)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0)),
                take_site: node(65),
                take_range: range(self.source, 117, 151),
                site: node(64),
                source_range: range(self.source, 122, 150),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "TypeCaseStruct ( x : 1 , y : 2 ) . x".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B2B statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_structure(
            input,
            statement,
            &self.primary,
            &self.structure,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B2B typed")
        .with_source_term(self.primary.clone())
        .expect("Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task256")
    }
}

const TASK258B3M2B2B2C_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureUpdateWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2) with (x := 3);\n",
    "  thus x = x;\n",
    "end;\n",
);

#[derive(Clone)]
struct B3M2B2B2CFixture {
    source: SourceId,
    module: ModuleId,
    theorem_symbol: SymbolId,
    theorem_contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    structure: SourceStructureHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B2CFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (
            theorem_symbol,
            theorem_contribution,
            structure_symbol,
            structure_contribution,
            symbols,
        ) = structure_witness_symbol_env_with_structure_identity(
            source,
            &module,
            "FormulaStatementStructureUpdateWitnessSmoke",
            180,
            "statement.fixture.theorem.b3m2b2b2c",
            "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5",
            "parser.type_fixtures::TypeCaseStruct#5",
            vec![5],
            ExportStatus::Exported,
            false,
        );
        let bindings = structure_witness_binding_env(source, &module, 107, 179, "Task258B3M2B2B2C");
        let arena = b3m2b2b2c_typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B2C primary");
        let structure = SourceStructureProducer::build(
            Self::structure_input(source, &module, structure_symbol, structure_contribution),
            &symbols,
            &bindings,
            &primary,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B2C structure");
        let atomic = Self::build_atomic(source, &module, &bindings, &symbols, &primary, &arena);
        Self {
            source,
            module,
            theorem_symbol,
            theorem_contribution,
            symbols,
            bindings,
            primary,
            structure,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let sites = [51, 53, 59, 62, 66, 73, 75];
        let ranges = [
            (101, 102),
            (105, 106),
            (138, 139),
            (144, 145),
            (158, 159),
            (169, 170),
            (173, 174),
        ];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..7)
                .map(|index| SourcePrimaryTermInput {
                    site: node(sites[index]),
                    source_range: range(source, ranges[index].0, ranges[index].1),
                    source_ordinal: index,
                    context: BindingContextId::new(usize::from(index >= 2)),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: match index {
                        2 => "1",
                        3 => "2",
                        4 => "3",
                        _ => "x",
                    }
                    .to_owned(),
                    kind: if matches!(index, 2..=4) {
                        SourcePrimaryTermKind::Numeral
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: [0, 1, 5, 6]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: [2, 3, 4]
                .into_iter()
                .enumerate()
                .map(|(request_ordinal, term)| SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(term),
                    owner: node(sites[term]),
                    source_range: range(source, ranges[term].0, ranges[term].1),
                    spelling: (term - 1).to_string(),
                    request_ordinal,
                })
                .collect(),
        }
    }

    fn structure_input(
        source: SourceId,
        module: &ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
    ) -> SourceStructureHandoffInput {
        SourceStructureHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![
                SourceStructureTermInput {
                    site: node(69),
                    source_range: range(source, 120, 160),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 )".to_owned(),
                    kind: SourceStructureTermKind::FunctionalUpdate,
                },
                SourceStructureTermInput {
                    site: node(65),
                    source_range: range(source, 120, 146),
                    source_ordinal: 1,
                    context: BindingContextId::new(1),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "TypeCaseStruct ( x : 1 , y : 2 )".to_owned(),
                    kind: SourceStructureTermKind::Constructor,
                },
            ],
            wrappers: Vec::new(),
            roots: vec![SourceStructureRootInput {
                term: SourceStructureTermId::new(1),
                symbol,
                contribution,
            }],
            members: [
                (
                    SourceStructureTermId::new(0),
                    0,
                    30,
                    153,
                    154,
                    "x",
                    SourceStructureMemberRole::UpdatePathSegment,
                ),
                (
                    SourceStructureTermId::new(1),
                    0,
                    20,
                    135,
                    136,
                    "x",
                    SourceStructureMemberRole::ConstructorAssignment,
                ),
                (
                    SourceStructureTermId::new(1),
                    1,
                    24,
                    141,
                    142,
                    "y",
                    SourceStructureMemberRole::ConstructorAssignment,
                ),
            ]
            .into_iter()
            .map(
                |(term, ordinal, site, start, end, spelling, role)| SourceStructureMemberInput {
                    term,
                    ordinal,
                    site: node(site),
                    source_range: range(source, start, end),
                    spelling: spelling.to_owned(),
                    role,
                    parent: None,
                },
            )
            .collect(),
            field_updates: vec![SourceFieldUpdateInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                site: node(68),
                source_range: range(source, 153, 159),
                spelling: "x := 3".to_owned(),
                first_member: SourceStructureMemberId::new(0),
                final_member: SourceStructureMemberId::new(0),
            }],
            edges: vec![
                SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::UpdateBase,
                    member: None,
                    target: SourceStructureTarget::Structure(SourceStructureTermId::new(1)),
                },
                SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 1,
                    role: SourceStructureEdgeRole::UpdateValue,
                    member: Some(SourceStructureMemberId::new(0)),
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(4)),
                },
                SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(1),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::ConstructorValue,
                    member: Some(SourceStructureMemberId::new(1)),
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(2)),
                },
                SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(1),
                    ordinal: 1,
                    role: SourceStructureEdgeRole::ConstructorValue,
                    member: Some(SourceStructureMemberId::new(2)),
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(3)),
                },
            ],
            requests: [
                (
                    SourceStructureTermId::new(0),
                    Some(SourceStructureMemberId::new(0)),
                    SourceStructureRequestKind::MemberIdentity,
                ),
                (
                    SourceStructureTermId::new(0),
                    Some(SourceStructureMemberId::new(0)),
                    SourceStructureRequestKind::InheritancePath,
                ),
                (
                    SourceStructureTermId::new(0),
                    None,
                    SourceStructureRequestKind::ResultType,
                ),
                (
                    SourceStructureTermId::new(1),
                    None,
                    SourceStructureRequestKind::ConstructorSignature,
                ),
                (
                    SourceStructureTermId::new(1),
                    Some(SourceStructureMemberId::new(1)),
                    SourceStructureRequestKind::MemberIdentity,
                ),
                (
                    SourceStructureTermId::new(1),
                    Some(SourceStructureMemberId::new(1)),
                    SourceStructureRequestKind::InheritancePath,
                ),
                (
                    SourceStructureTermId::new(1),
                    Some(SourceStructureMemberId::new(2)),
                    SourceStructureRequestKind::MemberIdentity,
                ),
                (
                    SourceStructureTermId::new(1),
                    Some(SourceStructureMemberId::new(2)),
                    SourceStructureRequestKind::InheritancePath,
                ),
                (
                    SourceStructureTermId::new(1),
                    None,
                    SourceStructureRequestKind::ResultType,
                ),
            ]
            .into_iter()
            .enumerate()
            .map(
                |(request_ordinal, (term, member, kind))| SourceStructureRequestInput {
                    term,
                    member,
                    request_ordinal: if term == SourceStructureTermId::new(0) {
                        request_ordinal
                    } else {
                        request_ordinal - 3
                    },
                    kind,
                },
            )
            .collect(),
        }
    }

    fn build_atomic(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> SourceAtomicFormulaHandoff {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
            let first_term = if formula == 0 { 0 } else { 5 };
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
        SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: [(55, 101, 106, 0), (77, 169, 174, 1)]
                    .into_iter()
                    .enumerate()
                    .map(
                        |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                            site: node(site),
                            source_range: range(source, start, end),
                            source_ordinal,
                            context: BindingContextId::new(context),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        },
                    )
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            bindings,
            symbols,
            primary,
            None,
            None,
            None,
            arena,
        )
        .expect("Task258B3M2B2B2C atomic")
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.theorem_symbol.clone(),
                contribution: self.theorem_contribution,
                site: node(82),
                source_range: range(self.source, 48, 180),
                spelling: "FormulaStatementStructureUpdateWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(82, 48, 180, 0), (80, 164, 175, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementStructureUpdateWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 ) ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(48, 180), (164, 175)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0)),
                take_site: node(72),
                take_range: range(self.source, 115, 161),
                site: node(71),
                source_range: range(self.source, 120, 160),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 )".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B2C statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
        input: SourceStatementWitnessHandoffInput,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_structure(
            input,
            statement,
            &self.primary,
            &self.structure,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B2C typed")
        .with_source_term(self.primary.clone())
        .expect("Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task256")
    }
}

#[derive(Clone)]
struct B3M2B2B3AFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    set_terms: SourceSetTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B3AFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b3m2b2b3a_symbol_env(source, &module);
        let bindings = b3m2b2b3a_binding_env(source, &module);
        let arena = b3m2b2b3a_typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B3A primary terms");
        let set_terms = SourceSetTermProducer::build(
            Self::set_input(source, &module),
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3A set terms");
        let atomic = SourceAtomicFormulaProducer::build(
            Self::atomic_input(source, &module),
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3A atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            set_terms,
            atomic,
            arena,
        }
    }

    fn atomic_input(source: SourceId, module: &ModuleId) -> SourceAtomicFormulaHandoffInput {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
            let first_term = if formula == 0 { 0 } else { 4 };
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
            source_id: source,
            module_id: module.clone(),
            formulas: [(34, 71, 76, 0), (48, 105, 110, 1)]
                .into_iter()
                .enumerate()
                .map(
                    |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                        site: node(site),
                        source_range: range(source, start, end),
                        source_ordinal,
                        context: BindingContextId::new(context),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    },
                )
                .collect(),
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

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: [
                (30, 71, 72, "x"),
                (32, 75, 76, "x"),
                (36, 91, 92, "1"),
                (38, 94, 95, "2"),
                (44, 105, 106, "x"),
                (46, 109, 110, "x"),
            ]
            .into_iter()
            .enumerate()
            .map(|(source_ordinal, (site, start, end, spelling))| {
                let numeral = matches!(source_ordinal, 2 | 3);
                SourcePrimaryTermInput {
                    site: node(site),
                    source_range: range(source, start, end),
                    source_ordinal,
                    context: BindingContextId::new(usize::from(source_ordinal >= 2)),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind: if numeral {
                        SourcePrimaryTermKind::Numeral
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }
            })
            .collect(),
            references: [0, 1, 4, 5]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: [(2, 36, 91, 92, "1"), (3, 38, 94, 95, "2")]
                .into_iter()
                .enumerate()
                .map(|(request_ordinal, (term, owner, start, end, spelling))| {
                    SourceNumericTypeRequestInput {
                        term: SourcePrimaryTermId::new(term),
                        owner: node(owner),
                        source_range: range(source, start, end),
                        spelling: spelling.to_owned(),
                        request_ordinal,
                    }
                })
                .collect(),
        }
    }

    fn set_input(source: SourceId, module: &ModuleId) -> SourceSetTermHandoffInput {
        SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceSetTermInput {
                site: node(40),
                source_range: range(source, 90, 96),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "{ 1 , 2 }".to_owned(),
                kind: SourceSetTermKind::Enumeration,
            }],
            wrappers: Vec::new(),
            generators: Vec::new(),
            type_sites: Vec::new(),
            conditions: Vec::new(),
            edges: (0..2)
                .map(|ordinal| SourceSetEdgeInput {
                    term: SourceSetTermId::new(0),
                    ordinal,
                    role: SourceSetEdgeRole::EnumerationElement,
                    target: SourceSetTarget::Primary(SourcePrimaryTermId::new(ordinal + 2)),
                })
                .collect(),
            requests: vec![SourceSetRequestInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                kind: SourceSetRequestKind::ResultType,
                generator: None,
                type_site: None,
            }],
        }
    }

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(53),
                source_range: range(self.source, 19, 116),
                spelling: "FormulaStatementSetEnumerationWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(53, 19, 116, 0), (51, 100, 111, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementSetEnumerationWitnessSmoke : x = x proof take { 1 , 2 } ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(19, 116), (100, 111)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0)),
                take_site: node(43),
                take_range: range(self.source, 85, 97),
                site: node(42),
                source_range: range(self.source, 90, 96),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "{ 1 , 2 }".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B3A base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_set_term(
            self.witness_input(),
            statement,
            &self.primary,
            &self.set_terms,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B3A empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3M2B2B3A Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B3M2B2B3A Task256")
    }
}

#[derive(Clone)]
struct B3M2B2B3BFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    set_terms: SourceSetTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B3BFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = Self::symbol_env(source, &module);
        let bindings = Self::binding_env(source, &module);
        let arena = Self::typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B3B primary terms");
        let set_terms = SourceSetTermProducer::build(
            Self::set_input(source, &module),
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3B set terms");
        let atomic = SourceAtomicFormulaProducer::build(
            Self::atomic_input(source, &module),
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3B atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            set_terms,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: [
                (27, 76, 77, 0),
                (29, 80, 81, 0),
                (37, 106, 107, 1),
                (39, 110, 111, 1),
            ]
            .into_iter()
            .enumerate()
            .map(
                |(source_ordinal, (site, start, end, context))| SourcePrimaryTermInput {
                    site: node(site),
                    source_range: range(source, start, end),
                    source_ordinal,
                    context: BindingContextId::new(context),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "x".to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect(),
            references: (0..4)
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: Vec::new(),
        }
    }

    fn set_input(source: SourceId, module: &ModuleId) -> SourceSetTermHandoffInput {
        SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceSetTermInput {
                site: node(33),
                source_range: range(source, 95, 97),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "{ }".to_owned(),
                kind: SourceSetTermKind::Enumeration,
            }],
            wrappers: Vec::new(),
            generators: Vec::new(),
            type_sites: Vec::new(),
            conditions: Vec::new(),
            edges: Vec::new(),
            requests: vec![SourceSetRequestInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                kind: SourceSetRequestKind::ResultType,
                generator: None,
                type_site: None,
            }],
        }
    }

    fn atomic_input(source: SourceId, module: &ModuleId) -> SourceAtomicFormulaHandoffInput {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
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
            source_id: source,
            module_id: module.clone(),
            formulas: [(31, 76, 81, 0), (41, 106, 111, 1)]
                .into_iter()
                .enumerate()
                .map(
                    |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                        site: node(site),
                        source_range: range(source, start, end),
                        source_ordinal,
                        context: BindingContextId::new(context),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    },
                )
                .collect(),
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

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(46),
                source_range: range(self.source, 19, 117),
                spelling: "FormulaStatementEmptySetEnumerationWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(46, 19, 117, 0), (44, 101, 112, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementEmptySetEnumerationWitnessSmoke : x = x proof take { } ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(19, 117), (101, 112)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0)),
                take_site: node(36),
                take_range: range(self.source, 90, 98),
                site: node(35),
                source_range: range(self.source, 95, 97),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "{ }".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B3B base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_set_term(
            self.witness_input(),
            statement,
            &self.primary,
            &self.set_terms,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B3B empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3M2B2B3B Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B3M2B2B3B Task256")
    }

    fn typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let mut ids = Vec::with_capacity(TASK258B3M2B2B3B_NODE_RANGES.len());
        for (index, (start, end)) in TASK258B3M2B2B3B_NODE_RANGES.iter().copied().enumerate() {
            let children = task258b3m2b2b3b_node_children(index)
                .iter()
                .map(|child| ids[*child])
                .collect();
            let id = builder
                .push(
                    TypedNode::new(
                        task258b3m2b2b3b_node_kind(index),
                        SourceAnchor::Range(range(source, start, end)),
                    )
                    .with_children(children),
                )
                .expect("Task258B3M2B2B3B typed node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder
            .finish(Some(ids[49]))
            .expect("Task258B3M2B2B3B arena")
    }

    fn binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        Self::binding_env_with_mutation(source, module, BindingContextRecovery::Normal, 116, 0)
    }

    fn binding_env_with_mutation(
        source: SourceId,
        module: &ModuleId,
        module_recovery: BindingContextRecovery,
        proof_end: usize,
        visible_after_ordinal: usize,
    ) -> BindingEnv {
        let binding = BindingId::new(0);
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: module_recovery,
        });
        let proof = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 82, proof_end),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(proof, BindingContextId::new(1));
        let mut bindings = BindingTable::new();
        bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(source, 8, 9),
            },
            owner_context: BindingContextId::new(0),
            declaration_range: range(source, 8, 9),
            visible_after_ordinal,
            type_site: BindingTypeSite::Source(range(source, 14, 17)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B3B binding env")
    }

    fn symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        const LABEL: &str = "FormulaStatementEmptySetEnumerationWitnessSmoke";
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 117)),
            vec![2, 1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                namespace.clone(),
                LABEL,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2b2b3b");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                origin_path.clone(),
                LabelKind::Theorem,
                namespace,
                LABEL,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        contributions.add_symbol(contribution, symbol.clone());
        contributions.add_definition(contribution, definition);
        contributions.add_label(contribution, origin_path);
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }
}

struct B3M2B2B3CFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    set_terms: SourceSetTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B3CFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = Self::symbol_env(source, &module);
        let bindings = Self::binding_env(source, &module);
        let arena = Self::typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B3C primary terms");
        let set_terms = SourceSetTermProducer::build(
            Self::set_input(source, &module),
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3C set terms");
        let atomic = SourceAtomicFormulaProducer::build(
            Self::atomic_input(source, &module),
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3C atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            set_terms,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: [
                (27, 63, 64, 0),
                (29, 67, 68, 0),
                (39, 98, 99, 1),
                (41, 102, 103, 1),
            ]
            .into_iter()
            .enumerate()
            .map(
                |(source_ordinal, (site, start, end, context))| SourcePrimaryTermInput {
                    site: node(site),
                    source_range: range(source, start, end),
                    source_ordinal,
                    context: BindingContextId::new(context),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "x".to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect(),
            references: (0..4)
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: Vec::new(),
        }
    }

    fn set_input(source: SourceId, module: &ModuleId) -> SourceSetTermHandoffInput {
        SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceSetTermInput {
                site: node(35),
                source_range: range(source, 82, 89),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "the set".to_owned(),
                kind: SourceSetTermKind::Choice,
            }],
            wrappers: Vec::new(),
            generators: Vec::new(),
            type_sites: vec![SourceSetTypeSiteInput {
                owner: SourceSetTypeOwner::Term {
                    term: SourceSetTermId::new(0),
                    role: SourceSetTypeRole::ChoiceTarget,
                },
                site: node(34),
                source_range: range(source, 86, 89),
                spelling: "set".to_owned(),
                head_site: node(33),
                head_range: range(source, 86, 89),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                head: SourceSetTypeHead::BuiltinSet,
            }],
            conditions: Vec::new(),
            edges: Vec::new(),
            requests: vec![
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ChoiceNonempty,
                    generator: None,
                    type_site: Some(SourceSetTypeSiteId::new(0)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 1,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
            ],
        }
    }

    fn atomic_input(source: SourceId, module: &ModuleId) -> SourceAtomicFormulaHandoffInput {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
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
            source_id: source,
            module_id: module.clone(),
            formulas: [(31, 63, 68, 0), (43, 98, 103, 1)]
                .into_iter()
                .enumerate()
                .map(
                    |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                        site: node(site),
                        source_range: range(source, start, end),
                        source_ordinal,
                        context: BindingContextId::new(context),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    },
                )
                .collect(),
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

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(48),
                source_range: range(self.source, 19, 109),
                spelling: "FormulaStatementChoiceWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(48, 19, 109, 0), (46, 93, 104, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementChoiceWitnessSmoke : x = x proof take the set ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(19, 109), (93, 104)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0)),
                take_site: node(38),
                take_range: range(self.source, 77, 90),
                site: node(37),
                source_range: range(self.source, 82, 89),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "the set".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B3C base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_set_term(
            self.witness_input(),
            statement,
            &self.primary,
            &self.set_terms,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B3C empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3M2B2B3C Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B3M2B2B3C Task256")
    }

    fn typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let mut ids = Vec::with_capacity(TASK258B3M2B2B3C_NODE_RANGES.len());
        for (index, (start, end)) in TASK258B3M2B2B3C_NODE_RANGES.iter().copied().enumerate() {
            let children = task258b3m2b2b3c_node_children(index)
                .iter()
                .map(|child| ids[*child])
                .collect();
            let id = builder
                .push(
                    TypedNode::new(
                        task258b3m2b2b3c_node_kind(index),
                        SourceAnchor::Range(range(source, start, end)),
                    )
                    .with_children(children),
                )
                .expect("Task258B3M2B2B3C typed node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder
            .finish(Some(ids[51]))
            .expect("Task258B3M2B2B3C arena")
    }

    fn binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        Self::binding_env_with_mutation(source, module, BindingContextRecovery::Normal, 108, 0)
    }

    fn binding_env_with_mutation(
        source: SourceId,
        module: &ModuleId,
        module_recovery: BindingContextRecovery,
        proof_end: usize,
        visible_after_ordinal: usize,
    ) -> BindingEnv {
        let binding = BindingId::new(0);
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: module_recovery,
        });
        let proof = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 69, proof_end),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(proof, BindingContextId::new(1));
        let mut bindings = BindingTable::new();
        bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(source, 8, 9),
            },
            owner_context: BindingContextId::new(0),
            declaration_range: range(source, 8, 9),
            visible_after_ordinal,
            type_site: BindingTypeSite::Source(range(source, 14, 17)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B3C binding env")
    }

    fn symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        const LABEL: &str = "FormulaStatementChoiceWitnessSmoke";
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 109)),
            vec![2, 1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                namespace.clone(),
                LABEL,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2b2b3c");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                origin_path.clone(),
                LabelKind::Theorem,
                namespace,
                LABEL,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        contributions.add_symbol(contribution, symbol.clone());
        contributions.add_definition(contribution, definition);
        contributions.add_label(contribution, origin_path);
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }
}

struct B3M2B2B3DFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    set_terms: SourceSetTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B3DFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = Self::symbol_env(source, &module);
        let bindings = Self::binding_env(source, &module);
        let arena = Self::typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B3D primary terms");
        let set_terms = SourceSetTermProducer::build(
            Self::set_input(source, &module),
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3D set terms");
        let atomic = SourceAtomicFormulaProducer::build(
            Self::atomic_input(source, &module),
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3D atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            set_terms,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let sites = [28, 30, 34, 41, 43];
        let ranges = [(60, 61), (64, 65), (79, 80), (97, 98), (101, 102)];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..5)
                .map(|index| SourcePrimaryTermInput {
                    site: node(sites[index]),
                    source_range: range(source, ranges[index].0, ranges[index].1),
                    source_ordinal: index,
                    context: BindingContextId::new(usize::from(index >= 2)),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: if index == 2 { "4" } else { "x" }.to_owned(),
                    kind: if index == 2 {
                        SourcePrimaryTermKind::Numeral
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: [0, 1, 3, 4]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: vec![SourceNumericTypeRequestInput {
                term: SourcePrimaryTermId::new(2),
                owner: node(34),
                source_range: range(source, 79, 80),
                spelling: "4".to_owned(),
                request_ordinal: 0,
            }],
        }
    }

    fn set_input(source: SourceId, module: &ModuleId) -> SourceSetTermHandoffInput {
        SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceSetTermInput {
                site: node(37),
                source_range: range(source, 79, 88),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "4 qua set".to_owned(),
                kind: SourceSetTermKind::Qua,
            }],
            wrappers: Vec::new(),
            generators: Vec::new(),
            type_sites: vec![SourceSetTypeSiteInput {
                owner: SourceSetTypeOwner::Term {
                    term: SourceSetTermId::new(0),
                    role: SourceSetTypeRole::QuaTarget,
                },
                site: node(36),
                source_range: range(source, 85, 88),
                spelling: "set".to_owned(),
                head_site: node(35),
                head_range: range(source, 85, 88),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                head: SourceSetTypeHead::BuiltinSet,
            }],
            conditions: Vec::new(),
            edges: vec![SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::QuaBase,
                target: SourceSetTarget::Primary(SourcePrimaryTermId::new(2)),
            }],
            requests: vec![
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::QuaWidening,
                    generator: None,
                    type_site: Some(SourceSetTypeSiteId::new(0)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 1,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
            ],
        }
    }

    fn atomic_input(source: SourceId, module: &ModuleId) -> SourceAtomicFormulaHandoffInput {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
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
                        if formula == 0 { ordinal } else { ordinal + 3 },
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
            source_id: source,
            module_id: module.clone(),
            formulas: [(32, 60, 65, 0), (45, 97, 102, 1)]
                .into_iter()
                .enumerate()
                .map(
                    |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                        site: node(site),
                        source_range: range(source, start, end),
                        source_ordinal,
                        context: BindingContextId::new(context),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    },
                )
                .collect(),
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

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(50),
                source_range: range(self.source, 19, 108),
                spelling: "FormulaStatementQuaWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(50, 19, 108, 0), (48, 92, 103, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementQuaWitnessSmoke : x = x proof take 4 qua set ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(19, 108), (92, 103)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0)),
                take_site: node(40),
                take_range: range(self.source, 74, 89),
                site: node(39),
                source_range: range(self.source, 79, 88),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "4 qua set".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B3D base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_set_term(
            self.witness_input(),
            statement,
            &self.primary,
            &self.set_terms,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B3D empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3M2B2B3D Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B3M2B2B3D Task256")
    }

    fn typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let mut ids = Vec::with_capacity(TASK258B3M2B2B3D_NODE_RANGES.len());
        for (index, (start, end)) in TASK258B3M2B2B3D_NODE_RANGES.iter().copied().enumerate() {
            let children = task258b3m2b2b3d_node_children(index)
                .iter()
                .map(|child| ids[*child])
                .collect();
            let id = builder
                .push(
                    TypedNode::new(
                        task258b3m2b2b3d_node_kind(index),
                        SourceAnchor::Range(range(source, start, end)),
                    )
                    .with_children(children),
                )
                .expect("Task258B3M2B2B3D typed node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder
            .finish(Some(ids[53]))
            .expect("Task258B3M2B2B3D arena")
    }

    fn binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        let binding = BindingId::new(0);
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        let proof = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 66, 107),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(proof, BindingContextId::new(1));
        let mut bindings = BindingTable::new();
        bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(source, 8, 9),
            },
            owner_context: BindingContextId::new(0),
            declaration_range: range(source, 8, 9),
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(range(source, 14, 17)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B3D binding env")
    }

    fn symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        const LABEL: &str = "FormulaStatementQuaWitnessSmoke";
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 108)),
            vec![2, 1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                namespace.clone(),
                LABEL,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2b2b3d");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                origin_path.clone(),
                LabelKind::Theorem,
                namespace,
                LABEL,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        contributions.add_symbol(contribution, symbol.clone());
        contributions.add_definition(contribution, definition);
        contributions.add_label(contribution, origin_path);
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }
}

struct B3M2B2B3EFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    set_terms: SourceSetTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B3M2B2B3EFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = Self::symbol_env(source, &module);
        let bindings = Self::binding_env(source, &module);
        let arena = Self::typed_arena(source);
        let primary = SourcePrimaryTermProducer::build(
            Self::primary_input(source, &module),
            &bindings,
            &arena,
        )
        .expect("Task258B3M2B2B3E primary terms");
        let set_terms = SourceSetTermProducer::build(
            Self::set_input(source, &module),
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3E set terms");
        let atomic = SourceAtomicFormulaProducer::build(
            Self::atomic_input(source, &module),
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B3M2B2B3E atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            set_terms,
            atomic,
            arena,
        }
    }

    fn primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let sites = [32, 34, 38, 47, 49];
        let ranges = [(70, 71), (74, 75), (90, 91), (127, 128), (131, 132)];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..5)
                .map(|index| SourcePrimaryTermInput {
                    site: node(sites[index]),
                    source_range: range(source, ranges[index].0, ranges[index].1),
                    source_ordinal: index,
                    context: BindingContextId::new(usize::from(index >= 2)),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: if index == 2 { "3" } else { "x" }.to_owned(),
                    kind: if index == 2 {
                        SourcePrimaryTermKind::Numeral
                    } else {
                        SourcePrimaryTermKind::VariableReference
                    },
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: [0, 1, 3, 4]
                .into_iter()
                .map(|term| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(term),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: vec![SourceNumericTypeRequestInput {
                term: SourcePrimaryTermId::new(2),
                owner: node(38),
                source_range: range(source, 90, 91),
                spelling: "3".to_owned(),
                request_ordinal: 0,
            }],
        }
    }

    fn set_input(source: SourceId, module: &ModuleId) -> SourceSetTermHandoffInput {
        SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceSetTermInput {
                site: node(43),
                source_range: range(source, 89, 118),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "{ 3 where candidate255 is set }".to_owned(),
                kind: SourceSetTermKind::Comprehension,
            }],
            wrappers: Vec::new(),
            generators: vec![SourceSetGeneratorInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(16),
                source_range: range(source, 98, 110),
                spelling: "candidate255".to_owned(),
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                type_site: SourceSetTypeSiteId::new(0),
            }],
            type_sites: vec![SourceSetTypeSiteInput {
                owner: SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0)),
                site: node(41),
                source_range: range(source, 114, 117),
                spelling: "set".to_owned(),
                head_site: node(40),
                head_range: range(source, 114, 117),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                head: SourceSetTypeHead::BuiltinSet,
            }],
            conditions: Vec::new(),
            edges: vec![SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::ComprehensionMapper,
                target: SourceSetTarget::Primary(SourcePrimaryTermId::new(2)),
            }],
            requests: vec![
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::GeneratorSethood,
                    generator: Some(SourceSetGeneratorId::new(0)),
                    type_site: Some(SourceSetTypeSiteId::new(0)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 1,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
            ],
        }
    }

    fn atomic_input(source: SourceId, module: &ModuleId) -> SourceAtomicFormulaHandoffInput {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
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
                        if formula == 0 { ordinal } else { ordinal + 3 },
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
            source_id: source,
            module_id: module.clone(),
            formulas: [(36, 70, 75, 0), (51, 127, 132, 1)]
                .into_iter()
                .enumerate()
                .map(
                    |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                        site: node(site),
                        source_range: range(source, start, end),
                        source_ordinal,
                        context: BindingContextId::new(context),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    },
                )
                .collect(),
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

    fn statement_input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(56),
                source_range: range(self.source, 19, 138),
                spelling: "FormulaStatementComprehensionWitnessSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: [(56, 19, 138, 0), (54, 122, 133, 2)]
                .into_iter()
                .enumerate()
                .map(|(index, (site, start, end, source_ordinal))| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: node(site),
                    source_range: range(self.source, start, end),
                    source_ordinal,
                    spelling: [
                        "theorem FormulaStatementComprehensionWitnessSmoke : x = x proof take { 3 where candidate255 is set } ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index]
                        .to_owned(),
                    kind: [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: [(19, 138), (122, 133)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(index),
                    source_range: range(self.source, start, end),
                    visible_bindings: vec![BindingId::new(0)],
                })
                .collect(),
            input_facts: [(0, 1), (2, 3)]
                .into_iter()
                .enumerate()
                .map(|(index, (first, second))| SourceStatementInputFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(first),
                        SourcePrimaryTermReferenceId::new(second),
                    ],
                })
                .collect(),
            candidate_facts: (0..2)
                .map(|index| SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(index),
                    context: SourceStatementContextId::new(index),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(
                        SourceAtomicFormulaId::new(index),
                    ),
                })
                .collect(),
        }
    }

    fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
        SourceStatementWitnessHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            witnesses: vec![SourceStatementWitnessInput {
                owner: SourceTheoremOwnerId::new(0),
                binding_context: BindingContextId::new(1),
                term: SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0)),
                take_site: node(46),
                take_range: range(self.source, 84, 119),
                site: node(45),
                source_range: range(self.source, 89, 118),
                source_ordinal: 1,
                ordinal: 0,
                spelling: "{ 3 where candidate255 is set }".to_owned(),
                kind: SourceStatementWitnessKind::Unnamed,
                recovery: SourceStatementRecovery::Normal,
                name: None,
            }],
            names: Vec::new(),
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        SourceStatementProducer::build(
            self.statement_input(),
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
        .expect("Task258B3M2B2B3E base statement")
    }

    fn witnesses(
        &self,
        statement: &SourceStatementHandoff,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        SourceStatementWitnessProducer::build_with_set_term(
            self.witness_input(),
            statement,
            &self.primary,
            &self.set_terms,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B3E empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B3M2B2B3E Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B3M2B2B3E Task256")
    }

    fn typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let mut ids = Vec::with_capacity(TASK258B3M2B2B3E_NODE_RANGES.len());
        for (index, (start, end)) in TASK258B3M2B2B3E_NODE_RANGES.iter().copied().enumerate() {
            let children = task258b3m2b2b3e_node_children(index)
                .iter()
                .map(|child| ids[*child])
                .collect();
            let id = builder
                .push(
                    TypedNode::new(
                        task258b3m2b2b3e_node_kind(index),
                        SourceAnchor::Range(range(source, start, end)),
                    )
                    .with_children(children),
                )
                .expect("Task258B3M2B2B3E typed node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder
            .finish(Some(ids[59]))
            .expect("Task258B3M2B2B3E arena")
    }

    fn binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        let binding = BindingId::new(0);
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        let proof = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 76, 137),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(proof, BindingContextId::new(1));
        let mut bindings = BindingTable::new();
        bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(source, 8, 9),
            },
            owner_context: BindingContextId::new(0),
            declaration_range: range(source, 8, 9),
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(range(source, 14, 17)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task258B3M2B2B3E binding env")
    }

    fn symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        const LABEL: &str = "FormulaStatementComprehensionWitnessSmoke";
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 138)),
            vec![2, 1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                namespace.clone(),
                LABEL,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2b2b3e");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                origin_path.clone(),
                LabelKind::Theorem,
                namespace,
                LABEL,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        contributions.add_symbol(contribution, symbol.clone());
        contributions.add_definition(contribution, definition);
        contributions.add_label(contribution, origin_path);
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }
}

fn task258b3m2b2b3d_mutated_binding_env(
    input: &BindingEnv,
    field: usize,
) -> Result<BindingEnv, crate::binding_env::BindingEnvError> {
    let mut mutated_source_id = input.source_id();
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
        0 => mutated_source_id = source_id(999_480),
        1 => module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign")),
        2 => contexts.clear(),
        3 => contexts.push(BindingContextDraft {
            owner: BindingContextOwner::Generated("Task258B3M2B2B3D extra".to_owned()),
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
        19 => bindings.push(bindings[0].clone()),
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
            message_key: "checker.binding.task258b3m2b2b3d.mutated".to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        }),
        _ => panic!("unknown Task48 field {field}"),
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
    let original_parts = BindingEnvParts {
        source_id: input.source_id(),
        module_id: input.module_id().clone(),
        contexts: input.contexts().clone(),
        bindings: input.bindings().clone(),
        diagnostics: input.diagnostics().clone(),
    };
    let mutated_parts = BindingEnvParts {
        source_id: mutated_source_id,
        module_id,
        contexts: context_table,
        bindings: binding_table,
        diagnostics: diagnostic_table,
    };
    assert_ne!(mutated_parts, original_parts, "Task48 field {field} delta");
    let before = mutated_parts.clone();
    let result = BindingEnv::try_new(mutated_parts.clone());
    assert_eq!(mutated_parts, before, "Task48 field {field} input purity");
    result
}

fn task258b3m2b2b3d_mutate_primary_input(input: &mut SourcePrimaryTermHandoffInput, field: usize) {
    match field {
        0 => input.source_id = source_id(999_252),
        1 => {
            input.module_id =
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
        }
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
        _ => panic!("unknown Task252 field {field}"),
    }
}

fn task258b3m2b2b3d_mutate_atomic_input(
    input: &mut SourceAtomicFormulaHandoffInput,
    symbol: &SymbolId,
    contribution: SourceContributionId,
    field: usize,
) {
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
                4 => input.requests[request].candidate = Some(SourcePredicateCandidateId::new(0)),
                5 => {
                    input.requests[request].type_site =
                        Some(crate::source_atomic_formula::SourceAssertionTypeSiteId::new(0))
                }
                6 => {
                    input.requests[request].attribute =
                        Some(crate::source_atomic_formula::SourceAssertionAttributeId::new(0))
                }
                _ => unreachable!(),
            }
        }
        60 => {
            input.module_id =
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
        }
        61 => input.source_id = source_id(999_256),
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
            symbol: symbol.clone(),
            contribution,
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
            symbol: symbol.clone(),
            contribution,
            polarity: SourceAssertionAttributePolarityInput::Positive,
        }),
        68 => input.edges.clear(),
        69 => input.edges.push(input.edges[0].clone()),
        70 => input.requests.clear(),
        71 => input.requests.push(input.requests[0].clone()),
        _ => panic!("unknown Task256 field {field}"),
    }
}

fn task258b3m2b2b3d_mutate_statement_input(input: &mut SourceStatementHandoffInput, field: usize) {
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
        56 => {
            input.module_id =
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
        }
        57 => input.source_id = source_id(999_258),
        58 => {
            input.owners[0].symbol = SymbolId::new(
                input.module_id.clone(),
                LocalSymbolId::new("Task258B3M2B2B3D/mutated-owner"),
                FullyQualifiedName::new(format!(
                    "{}::Task258B3M2B2B3D/mutated-owner",
                    input.module_id.path().as_str()
                )),
            )
        }
        59 => {
            let mut contributions = SourceContributionIndex::new();
            let kind = ContributionKind::LocalSource {
                source_id: input.source_id,
            };
            let anchor = SourceAnchor::Range(input.owners[0].source_range);
            contributions.insert(input.module_id.clone(), kind.clone(), anchor.clone());
            input.owners[0].contribution =
                contributions.insert(input.module_id.clone(), kind, anchor);
        }
        60 => input.statements[0].kind = SourceStatementKind::Conclusion,
        61 => input.statements[1].kind = SourceStatementKind::TheoremProposition,
        _ => panic!("unknown Task258 field {field}"),
    }
}

fn task258b3m2b2b3d_mutate_witness_input(
    input: &mut SourceStatementWitnessHandoffInput,
    field: usize,
) {
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
            witness: SourceStatementWitnessId::new(0),
            site: input.witnesses[0].site.clone(),
            source_range: input.witnesses[0].source_range,
            spelling: "y".to_owned(),
            recovery: SourceStatementRecovery::Normal,
        }),
        15 => {
            input.module_id =
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
        }
        16 => input.source_id = source_id(999_259),
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
        _ => panic!("unknown witness field {field}"),
    }
}

fn task258b3m2b2b3e_mutate_set_input(input: &mut SourceSetTermHandoffInput, field: usize) {
    match field {
        0 => input.source_id = source_id(999_255),
        1 => {
            input.module_id =
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
        }
        2 => input.terms.clear(),
        3 => input.terms.push(input.terms[0].clone()),
        4 => input.terms[0].site = node(42),
        5 => input.terms[0].source_range.end -= 1,
        6 => input.terms[0].source_ordinal = 1,
        7 => input.terms[0].context = BindingContextId::new(0),
        8 => input.terms[0].recovery = SourceSetTermRecovery::Degraded,
        9 => input.terms[0].spelling.push('!'),
        10 => input.terms[0].kind = SourceSetTermKind::Qua,
        11 => input.wrappers.push(SourceSetWrapperInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            site: node(44),
            source_range: input.terms[0].source_range,
            context: BindingContextId::new(1),
            recovery: SourceSetTermRecovery::Normal,
            spelling: input.terms[0].spelling.clone(),
        }),
        12 => input.generators.clear(),
        13 => input.generators.push(input.generators[0].clone()),
        14 => input.generators[0].term = SourceSetTermId::new(1),
        15 => input.generators[0].ordinal = 1,
        16 => input.generators[0].site = node(42),
        17 => input.generators[0].source_range.end -= 1,
        18 => input.generators[0].spelling.push('!'),
        19 => input.generators[0].context = BindingContextId::new(0),
        20 => input.generators[0].recovery = SourceSetTermRecovery::Degraded,
        21 => input.generators[0].type_site = SourceSetTypeSiteId::new(1),
        22 => input.type_sites.clear(),
        23 => input.type_sites.push(input.type_sites[0].clone()),
        24 => {
            input.type_sites[0].owner = SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0),
                role: SourceSetTypeRole::ChoiceTarget,
            }
        }
        25 => input.type_sites[0].site = node(40),
        26 => input.type_sites[0].source_range.end -= 1,
        27 => input.type_sites[0].spelling.push('!'),
        28 => input.type_sites[0].head_site = node(41),
        29 => input.type_sites[0].head_range.end -= 1,
        30 => input.type_sites[0].head_spelling.push('!'),
        31 => input.type_sites[0].context = BindingContextId::new(0),
        32 => input.type_sites[0].recovery = SourceSetTermRecovery::Degraded,
        33 => input.type_sites[0].head = SourceSetTypeHead::BuiltinObject,
        34 => input.conditions.push(SourceSetConditionInput {
            term: SourceSetTermId::new(0),
            ordinal: 0,
            colon_site: node(15),
            colon_range: range(input.source_id, 92, 97),
            colon_spelling: ":".to_owned(),
            condition_site: node(36),
            source_range: range(input.source_id, 70, 75),
            spelling: "x = x".to_owned(),
            recovery: SourceSetTermRecovery::Normal,
        }),
        35 => input.edges.clear(),
        36 => input.edges.push(input.edges[0].clone()),
        37 => input.edges[0].term = SourceSetTermId::new(1),
        38 => input.edges[0].ordinal = 1,
        39 => input.edges[0].role = SourceSetEdgeRole::QuaBase,
        40 => input.edges[0].target = SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
        41 => input.requests.clear(),
        42 => input.requests.push(input.requests[0].clone()),
        43..53 => {
            let offset = field - 43;
            let request = offset / 5;
            match offset % 5 {
                0 => input.requests[request].term = SourceSetTermId::new(1),
                1 => input.requests[request].ordinal += 2,
                2 => {
                    input.requests[request].kind = if request == 0 {
                        SourceSetRequestKind::ResultType
                    } else {
                        SourceSetRequestKind::GeneratorSethood
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
        _ => panic!("unknown Task255 field {field}"),
    }
}

#[derive(Clone)]
struct B2Fixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
}

impl B2Fixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b2_symbol_env(source, &module);
        let bindings = b2_binding_env(source, &module);
        let arena = b2_typed_arena(source);
        let term_sites = [28, 30, 34, 36, 42, 44];
        let term_ranges = [
            (66, 67),
            (70, 71),
            (87, 88),
            (91, 92),
            (101, 102),
            (105, 106),
        ];
        let term_contexts = [0, 0, 1, 1, 1, 1];
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: (0..6)
                    .map(|index| SourcePrimaryTermInput {
                        site: node(term_sites[index]),
                        source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                        source_ordinal: index,
                        context: BindingContextId::new(term_contexts[index]),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    })
                    .collect(),
                references: (0..6)
                    .map(|index| SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(index),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    })
                    .collect(),
                numeric_type_requests: Vec::new(),
            },
            &bindings,
            &arena,
        )
        .expect("Task258B2 primary terms");
        let formula_sites = [32, 38, 46];
        let formula_ranges = [(66, 71), (87, 92), (101, 106)];
        let formula_contexts = [0, 1, 1];
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: (0..3)
                    .map(|index| SourceAtomicFormulaInput {
                        site: node(formula_sites[index]),
                        source_range: range(
                            source,
                            formula_ranges[index].0,
                            formula_ranges[index].1,
                        ),
                        source_ordinal: index,
                        context: BindingContextId::new(formula_contexts[index]),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    })
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B2 atomic formulas");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            arena,
        }
    }

    fn input(&self) -> SourceStatementHandoffInput {
        let statement_sites = [51, 41, 49];
        let statement_ranges = [(19, 112), (80, 93), (96, 107)];
        let statement_kinds = [
            SourceStatementKind::TheoremProposition,
            SourceStatementKind::Assumption,
            SourceStatementKind::Conclusion,
        ];
        let spellings = [
            "theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ; thus x = x ; end ;",
            "assume x = x ;",
            "thus x = x ;",
        ];
        let binding_contexts = [0, 1, 1];
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(51),
                source_range: range(self.source, 19, 112),
                spelling: "FormulaStatementSingleAssumptionSmoke".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: (0..3)
                .map(|index| SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(index),
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                        index,
                    )),
                    site: node(statement_sites[index]),
                    source_range: range(
                        self.source,
                        statement_ranges[index].0,
                        statement_ranges[index].1,
                    ),
                    source_ordinal: index,
                    spelling: spellings[index].to_owned(),
                    kind: statement_kinds[index],
                    recovery: SourceStatementRecovery::Normal,
                })
                .collect(),
            contexts: (0..3)
                .map(|index| SourceStatementContextInput {
                    statement: SourceStatementId::new(index),
                    binding_context: BindingContextId::new(binding_contexts[index]),
                    source_range: range(
                        self.source,
                        statement_ranges[index].0,
                        statement_ranges[index].1,
                    ),
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
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                        index,
                    )),
                })
                .collect(),
        }
    }

    fn build(
        &self,
        input: SourceStatementHandoffInput,
    ) -> Result<SourceStatementHandoff, SourceStatementError> {
        SourceStatementProducer::build(
            input,
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B2 empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B2 Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B2 Task256")
    }
}

#[derive(Clone)]
struct B1Fixture {
    source: SourceId,
    module: ModuleId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
    statement: SourceStatementHandoff,
    resolver_ast: ResolvedAst,
    projection: LabelProjection,
    reference: LabelReferenceCandidate,
    resolution: LabelResolutionResult,
}

impl B1Fixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b1_symbol_env(source, &module);
        let bindings = b1_binding_env(source, &module);
        let arena = b1_typed_arena(source);
        let term_sites = [38, 40, 44, 46, 51, 53, 61, 63];
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
        let term_contexts = [0, 0, 1, 1, 2, 2, 1, 1];
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: (0..8)
                    .map(|index| SourcePrimaryTermInput {
                        site: node(term_sites[index]),
                        source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                        source_ordinal: index,
                        context: BindingContextId::new(term_contexts[index]),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    })
                    .collect(),
                references: (0..8)
                    .map(|index| SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(index),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    })
                    .collect(),
                numeric_type_requests: Vec::new(),
            },
            &bindings,
            &arena,
        )
        .expect("Task258B1 primary terms");
        let formula_sites = [42, 48, 55, 65];
        let formula_ranges = [(63, 68), (80, 85), (101, 106), (122, 127)];
        let formula_contexts = [0, 1, 2, 1];
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: (0..4)
                    .map(|index| SourceAtomicFormulaInput {
                        site: node(formula_sites[index]),
                        source_range: range(
                            source,
                            formula_ranges[index].0,
                            formula_ranges[index].1,
                        ),
                        source_ordinal: index,
                        context: BindingContextId::new(formula_contexts[index]),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    })
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B1 atomic formulas");
        let statement_sites = [73, 60, 58, 71];
        let statement_ranges = [(19, 138), (77, 114), (96, 107), (117, 133)];
        let statement_kinds = [
            SourceStatementKind::TheoremProposition,
            SourceStatementKind::ProofStepProposition,
            SourceStatementKind::Conclusion,
            SourceStatementKind::Conclusion,
        ];
        let binding_contexts = [0, 1, 2, 1];
        let statement = SourceStatementProducer::build(
            SourceStatementHandoffInput {
                source_id: source,
                module_id: module.clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol,
                    contribution,
                    site: node(73),
                    source_range: range(source, 19, 138),
                    spelling: "FormulaStatementNestedContextSmoke".to_owned(),
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: (0..4)
                    .map(|index| SourceStatementInput {
                        owner: SourceTheoremOwnerId::new(0),
                        context: SourceStatementContextId::new(index),
                        formula: SourceStatementFormulaTarget::Atomic(
                            SourceAtomicFormulaId::new(index),
                        ),
                        site: node(statement_sites[index]),
                        source_range: range(
                            source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        source_ordinal: index,
                        spelling: [
                            "theorem FormulaStatementNestedContextSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x by A ; end ;",
                            "A : x = x proof thus x = x ; end ;",
                            "thus x = x ;",
                            "thus x = x by A ;",
                        ][index]
                            .to_owned(),
                        kind: statement_kinds[index],
                        recovery: SourceStatementRecovery::Normal,
                    })
                    .collect(),
                contexts: (0..4)
                    .map(|index| SourceStatementContextInput {
                        statement: SourceStatementId::new(index),
                        binding_context: BindingContextId::new(binding_contexts[index]),
                        source_range: range(
                            source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
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
                        formula: SourceStatementFormulaTarget::Atomic(
                            SourceAtomicFormulaId::new(index),
                        ),
                    })
                    .collect(),
            },
            &symbols,
            &bindings,
            &primary,
            &atomic,
            &arena,
        )
        .expect("Task258B1 base statement");
        let preliminary = b1_resolved_arena(source, &module, None);
        let reference_node = preliminary
            .iter()
            .find_map(|(id, _)| (id.index() == 68).then_some(id))
            .expect("reference node");
        let namespace = NamespacePath::new(module.path().as_str());
        let projection = LabelProjection::proof_step(
            mizar_resolve::labels::LabelProjectionData {
                origin_path: LabelOriginPath::new("pkg::statement.fixture::proof::A"),
                module: module.clone(),
                namespace: namespace.clone(),
                primary_spelling: "A".to_owned(),
                kind: LabelKind::ProofStep,
                declaration_range: range(source, 77, 78),
                origin: SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 77, 78)),
                    vec![12],
                ),
                contribution,
            },
            1,
            LabelScopePath::new(vec![0]),
        );
        let reference = LabelReferenceCandidate::unqualified_citation(
            mizar_resolve::resolved_ast::ReferenceSite::new(
                reference_node,
                range(source, 131, 132),
                "A",
            ),
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 131, 132)),
                vec![68],
            ),
            3,
            Some(LabelScopePath::new(vec![0])),
        );
        let resolution = LabelResolver::new(std::slice::from_ref(&projection)).resolve(
            &module,
            &namespace,
            std::slice::from_ref(&reference),
        );
        let resolver_ast = ResolvedAst::try_new(
            source,
            module.clone(),
            b1_resolved_arena(source, &module, Some(resolution.ids()[0])),
            mizar_resolve::resolved_ast::NameRefTable::new(),
            resolution.table().clone(),
            mizar_resolve::resolved_ast::ResolvedImports::new(),
        )
        .expect("Task258B1 resolver AST");
        Self {
            source,
            module,
            symbols,
            bindings,
            primary,
            atomic,
            arena,
            statement,
            resolver_ast,
            projection,
            reference,
            resolution,
        }
    }

    fn reference_input(&self) -> SourceStatementReferenceHandoffInput {
        SourceStatementReferenceHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            labels: vec![SourceStatementLabelInput {
                statement: SourceStatementId::new(1),
                context: SourceStatementContextId::new(1),
                candidate: SourceStatementCandidateFactId::new(1),
                origin_path: self.projection.origin_path().clone(),
                proof_scope: LabelScopePath::new(vec![0]),
                source_range: range(self.source, 77, 78),
                source_ordinal: 0,
                visible_after_ordinal: 1,
                spelling: "A".to_owned(),
                kind: SourceStatementLabelKind::ProofStep,
                recovery: SourceStatementRecovery::Normal,
            }],
            citations: vec![SourceStatementCitationInput {
                statement: SourceStatementId::new(3),
                context: SourceStatementContextId::new(3),
                target: SourceStatementCitationTarget::Local(SourceStatementLabelId::new(0)),
                label_ref: self.resolution.ids()[0],
                proof_scope: LabelScopePath::new(vec![0]),
                source_range: range(self.source, 131, 132),
                ordinal: 0,
                kind: SourceStatementCitationKind::SimpleLocal,
                recovery: SourceStatementRecovery::Normal,
            }],
        }
    }

    fn references(
        &self,
        input: SourceStatementReferenceHandoffInput,
    ) -> Result<SourceStatementReferenceHandoff, SourceStatementReferenceError> {
        SourceStatementReferenceProducer::build(
            input,
            &self.statement,
            &self.resolver_ast,
            &self.projection,
            &self.reference,
            &self.resolution,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task256")
    }
}

#[derive(Clone)]
struct B5AFixture {
    source: SourceId,
    module: ModuleId,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
    statement: SourceStatementHandoff,
    resolver_ast: ResolvedAst,
    projection: LabelProjection,
    reference: LabelReferenceCandidate,
    resolution: LabelResolutionResult,
}

impl B5AFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, symbols) = b5a_symbol_env(source, &module);
        let bindings = b5a_binding_env(source, &module);
        let arena = b5a_typed_arena(source);
        let term_sites = [45, 47, 51, 53, 58, 60, 68, 70, 75, 77];
        let term_ranges = [
            (81, 82),
            (85, 86),
            (98, 99),
            (102, 103),
            (119, 120),
            (123, 124),
            (140, 141),
            (144, 145),
            (161, 162),
            (165, 166),
        ];
        let term_contexts = [0, 0, 1, 1, 2, 2, 1, 1, 3, 3];
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: (0..10)
                    .map(|index| SourcePrimaryTermInput {
                        site: node(term_sites[index]),
                        source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                        source_ordinal: index,
                        context: BindingContextId::new(term_contexts[index]),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    })
                    .collect(),
                references: (0..10)
                    .map(|index| SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(index),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    })
                    .collect(),
                numeric_type_requests: Vec::new(),
            },
            &bindings,
            &arena,
        )
        .expect("Task258B5A primary terms");
        let formula_sites = [49, 55, 62, 72, 79];
        let formula_ranges = [(81, 86), (98, 103), (119, 124), (140, 145), (161, 166)];
        let formula_contexts = [0, 1, 2, 1, 3];
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..5 {
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: (0..5)
                    .map(|index| SourceAtomicFormulaInput {
                        site: node(formula_sites[index]),
                        source_range: range(
                            source,
                            formula_ranges[index].0,
                            formula_ranges[index].1,
                        ),
                        source_ordinal: index,
                        context: BindingContextId::new(formula_contexts[index]),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    })
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B5A atomic formulas");
        let statement_sites = [89, 67, 65, 87, 85];
        let statement_ranges = [(19, 184), (95, 132), (114, 125), (135, 179), (156, 172)];
        let statement_kinds = [
            SourceStatementKind::TheoremProposition,
            SourceStatementKind::ProofStepProposition,
            SourceStatementKind::Conclusion,
            SourceStatementKind::Conclusion,
            SourceStatementKind::Conclusion,
        ];
        let binding_contexts = [0, 1, 2, 1, 3];
        let spellings = [
            "theorem FormulaStatementAncestorLabelDescendantCitationSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x proof thus x = x by A ; end ; end ;",
            "A : x = x proof thus x = x ; end ;",
            "thus x = x ;",
            "thus x = x proof thus x = x by A ; end ;",
            "thus x = x by A ;",
        ];
        let statement = SourceStatementProducer::build(
            SourceStatementHandoffInput {
                source_id: source,
                module_id: module.clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol,
                    contribution,
                    site: node(89),
                    source_range: range(source, 19, 184),
                    spelling: "FormulaStatementAncestorLabelDescendantCitationSmoke".to_owned(),
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: (0..5)
                    .map(|index| SourceStatementInput {
                        owner: SourceTheoremOwnerId::new(0),
                        context: SourceStatementContextId::new(index),
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                        site: node(statement_sites[index]),
                        source_range: range(
                            source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        source_ordinal: index,
                        spelling: spellings[index].to_owned(),
                        kind: statement_kinds[index],
                        recovery: SourceStatementRecovery::Normal,
                    })
                    .collect(),
                contexts: (0..5)
                    .map(|index| SourceStatementContextInput {
                        statement: SourceStatementId::new(index),
                        binding_context: BindingContextId::new(binding_contexts[index]),
                        source_range: range(
                            source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        visible_bindings: vec![BindingId::new(0)],
                    })
                    .collect(),
                input_facts: (0..5)
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
                candidate_facts: (0..5)
                    .map(|index| SourceStatementCandidateFactInput {
                        statement: SourceStatementId::new(index),
                        context: SourceStatementContextId::new(index),
                        ordinal: 0,
                        kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                    })
                    .collect(),
            },
            &symbols,
            &bindings,
            &primary,
            &atomic,
            &arena,
        )
        .expect("Task258B5A base statement");
        let preliminary = b5a_resolved_arena(source, &module, None);
        let reference_node = preliminary
            .iter()
            .find_map(|(id, _)| (id.index() == 82).then_some(id))
            .expect("Task258B5A reference node");
        let namespace = NamespacePath::new(module.path().as_str());
        let projection = LabelProjection::proof_step(
            mizar_resolve::labels::LabelProjectionData {
                origin_path: LabelOriginPath::new("pkg::statement.fixture::proof::A"),
                module: module.clone(),
                namespace: namespace.clone(),
                primary_spelling: "A".to_owned(),
                kind: LabelKind::ProofStep,
                declaration_range: range(source, 95, 96),
                origin: SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 95, 96)),
                    vec![12],
                ),
                contribution,
            },
            1,
            LabelScopePath::new(vec![0]),
        );
        let reference = LabelReferenceCandidate::unqualified_citation(
            mizar_resolve::resolved_ast::ReferenceSite::new(
                reference_node,
                range(source, 170, 171),
                "A",
            ),
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 170, 171)),
                vec![82],
            ),
            4,
            Some(LabelScopePath::new(vec![0, 1])),
        );
        let resolution = LabelResolver::new(std::slice::from_ref(&projection)).resolve(
            &module,
            &namespace,
            std::slice::from_ref(&reference),
        );
        let resolver_ast = ResolvedAst::try_new(
            source,
            module.clone(),
            b5a_resolved_arena(source, &module, Some(resolution.ids()[0])),
            mizar_resolve::resolved_ast::NameRefTable::new(),
            resolution.table().clone(),
            mizar_resolve::resolved_ast::ResolvedImports::new(),
        )
        .expect("Task258B5A resolver AST");
        Self {
            source,
            module,
            bindings,
            primary,
            atomic,
            arena,
            statement,
            resolver_ast,
            projection,
            reference,
            resolution,
        }
    }

    fn reference_input(&self) -> SourceStatementReferenceHandoffInput {
        SourceStatementReferenceHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            labels: vec![SourceStatementLabelInput {
                statement: SourceStatementId::new(1),
                context: SourceStatementContextId::new(1),
                candidate: SourceStatementCandidateFactId::new(1),
                origin_path: self.projection.origin_path().clone(),
                proof_scope: LabelScopePath::new(vec![0]),
                source_range: range(self.source, 95, 96),
                source_ordinal: 0,
                visible_after_ordinal: 1,
                spelling: "A".to_owned(),
                kind: SourceStatementLabelKind::ProofStep,
                recovery: SourceStatementRecovery::Normal,
            }],
            citations: vec![SourceStatementCitationInput {
                statement: SourceStatementId::new(4),
                context: SourceStatementContextId::new(4),
                target: SourceStatementCitationTarget::Local(SourceStatementLabelId::new(0)),
                label_ref: self.resolution.ids()[0],
                proof_scope: LabelScopePath::new(vec![0, 1]),
                source_range: range(self.source, 170, 171),
                ordinal: 0,
                kind: SourceStatementCitationKind::SimpleLocal,
                recovery: SourceStatementRecovery::Normal,
            }],
        }
    }

    fn references(
        &self,
        input: SourceStatementReferenceHandoffInput,
    ) -> Result<SourceStatementReferenceHandoff, SourceStatementReferenceError> {
        SourceStatementReferenceProducer::build(
            input,
            &self.statement,
            &self.resolver_ast,
            &self.projection,
            &self.reference,
            &self.resolution,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B5A empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B5A Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B5A Task256")
    }
}

#[derive(Clone)]
struct B5BFixture {
    source: SourceId,
    module: ModuleId,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    arena: TypedArena,
    statement: SourceStatementHandoff,
    resolver_ast: ResolvedAst,
    projection: LabelProjection,
    reference: LabelReferenceCandidate,
    resolution: LabelResolutionResult,
}

impl B5BFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
        let (symbol, contribution, imported_contribution, symbols) =
            b5b_symbol_env(source, &module);
        let bindings = b5b_binding_env(source, &module);
        let arena = b5b_typed_arena(source);
        let term_sites = [35, 37, 41, 43];
        let term_ranges = [(108, 109), (112, 113), (127, 128), (131, 132)];
        let term_contexts = [0, 0, 1, 1];
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: (0..4)
                    .map(|index| SourcePrimaryTermInput {
                        site: node(term_sites[index]),
                        source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                        source_ordinal: index,
                        context: BindingContextId::new(term_contexts[index]),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    })
                    .collect(),
                references: (0..4)
                    .map(|index| SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(index),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    })
                    .collect(),
                numeric_type_requests: Vec::new(),
            },
            &bindings,
            &arena,
        )
        .expect("Task258B5B primary terms");
        let formula_sites = [40, 46];
        let formula_ranges = [(108, 113), (127, 132)];
        let formula_contexts = [0, 1];
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: (0..2)
                    .map(|index| SourceAtomicFormulaInput {
                        site: node(formula_sites[index]),
                        source_range: range(
                            source,
                            formula_ranges[index].0,
                            formula_ranges[index].1,
                        ),
                        source_ordinal: index,
                        context: BindingContextId::new(formula_contexts[index]),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    })
                    .collect(),
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B5B atomic formulas");
        let statement_ranges = [(48, 145), (122, 140)];
        let statement = SourceStatementProducer::build(
            SourceStatementHandoffInput {
                source_id: source,
                module_id: module.clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol,
                    contribution,
                    site: node(53),
                    source_range: range(source, 48, 145),
                    spelling: "FormulaStatementImportedPublicTheoremCitationSmoke".to_owned(),
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: vec![
                    SourceStatementInput {
                        owner: SourceTheoremOwnerId::new(0),
                        context: SourceStatementContextId::new(0),
                        formula: SourceStatementFormulaTarget::Atomic(
                            SourceAtomicFormulaId::new(0),
                        ),
                        site: node(53),
                        source_range: range(source, 48, 145),
                        source_ordinal: 0,
                        spelling: "theorem FormulaStatementImportedPublicTheoremCitationSmoke : x = x proof thus x = x by Ref ; end ;".to_owned(),
                        kind: SourceStatementKind::TheoremProposition,
                        recovery: SourceStatementRecovery::Normal,
                    },
                    SourceStatementInput {
                        owner: SourceTheoremOwnerId::new(0),
                        context: SourceStatementContextId::new(1),
                        formula: SourceStatementFormulaTarget::Atomic(
                            SourceAtomicFormulaId::new(1),
                        ),
                        site: node(51),
                        source_range: range(source, 122, 140),
                        source_ordinal: 1,
                        spelling: "thus x = x by Ref ;".to_owned(),
                        kind: SourceStatementKind::Conclusion,
                        recovery: SourceStatementRecovery::Normal,
                    },
                ],
                contexts: (0..2)
                    .map(|index| SourceStatementContextInput {
                        statement: SourceStatementId::new(index),
                        binding_context: BindingContextId::new(index),
                        source_range: range(
                            source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        visible_bindings: vec![BindingId::new(0)],
                    })
                    .collect(),
                input_facts: (0..2)
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
                candidate_facts: (0..2)
                    .map(|index| SourceStatementCandidateFactInput {
                        statement: SourceStatementId::new(index),
                        context: SourceStatementContextId::new(index),
                        ordinal: 0,
                        kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                        formula: SourceStatementFormulaTarget::Atomic(
                            SourceAtomicFormulaId::new(index),
                        ),
                    })
                    .collect(),
            },
            &symbols,
            &bindings,
            &primary,
            &atomic,
            &arena,
        )
        .expect("Task258B5B base statement");
        let imported_module = ModuleId::new(
            module.package().clone(),
            ModulePath::new("parser.type_fixtures"),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let projection = LabelProjection::imported(mizar_resolve::labels::LabelProjectionData {
            origin_path: LabelOriginPath::new("summary:parser.type_fixtures::Ref:label:Ref"),
            module: imported_module,
            namespace: namespace.clone(),
            primary_spelling: "Ref".to_owned(),
            kind: LabelKind::Theorem,
            declaration_range: range(source, 7, 27),
            origin: SemanticOrigin::new(
                source,
                ModuleId::new(
                    module.package().clone(),
                    ModulePath::new("parser.type_fixtures"),
                ),
                SourceAnchor::Range(range(source, 7, 27)),
                vec![1, 0],
            ),
            contribution: imported_contribution,
        })
        .with_export_status(ExportStatus::Exported);
        let preliminary = b5b_resolved_arena(source, &module, None);
        let reference_node = preliminary
            .iter()
            .find_map(|(id, _)| (id.index() == 48).then_some(id))
            .expect("Task258B5B reference node");
        let reference = LabelReferenceCandidate::unqualified_citation(
            mizar_resolve::resolved_ast::ReferenceSite::new(
                reference_node,
                range(source, 136, 139),
                "Ref",
            ),
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 136, 139)),
                vec![48],
            ),
            1,
            Some(LabelScopePath::new(vec![0])),
        );
        let resolution = LabelResolver::new(std::slice::from_ref(&projection)).resolve(
            &module,
            &namespace,
            std::slice::from_ref(&reference),
        );
        let resolver_ast = b5b_resolved_ast(
            source,
            &module,
            Some(resolution.ids()[0]),
            resolution.table().clone(),
        );
        Self {
            source,
            module,
            bindings,
            primary,
            atomic,
            arena,
            statement,
            resolver_ast,
            projection,
            reference,
            resolution,
        }
    }

    fn reference_input(&self) -> SourceStatementReferenceHandoffInput {
        SourceStatementReferenceHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            labels: Vec::new(),
            citations: vec![SourceStatementCitationInput {
                statement: SourceStatementId::new(1),
                context: SourceStatementContextId::new(1),
                target: SourceStatementCitationTarget::Imported,
                label_ref: self.resolution.ids()[0],
                proof_scope: LabelScopePath::new(vec![0]),
                source_range: range(self.source, 136, 139),
                ordinal: 0,
                kind: SourceStatementCitationKind::SimpleImported,
                recovery: SourceStatementRecovery::Normal,
            }],
        }
    }

    fn references(
        &self,
        input: SourceStatementReferenceHandoffInput,
    ) -> Result<SourceStatementReferenceHandoff, SourceStatementReferenceError> {
        SourceStatementReferenceProducer::build(
            input,
            &self.statement,
            &self.resolver_ast,
            &self.projection,
            &self.reference,
            &self.resolution,
            &self.arena,
        )
    }

    fn empty_typed(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B5B empty typed AST")
        .with_source_term(self.primary.clone())
        .expect("Task258B5B Task252")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B5B Task256")
    }
}

fn b5b_expected_reference_debug(references: &SourceStatementReferenceHandoff) -> String {
    format!(
        "source-statement-reference-debug-v1\n\
module: pkg::statement.fixture\n\
statement-fingerprint: {:?}\n\
resolver-ast root=56 nodes=57 name_refs=0 label_refs=1 imports=1 exports=0 label_node=absent reference_node=48 reference_state=resolved reference_key=label#0\n\
resolver-projection source=imported origin=summary:parser.type_fixtures::Ref:label:Ref module=parser.type_fixtures namespace=statement.fixture range=7..27 contribution=2 path=[1,0] kind=theorem visibility=public export=exported spelling=\"Ref\"\n\
resolver-reference node=48 range=136..139 source_ordinal=1 scope=[0] expectation=proof-or-theorem spelling=\"Ref\"\n\
resolver-result index=1 references=1 ids=[0] diagnostics=0\n\
citation#0 statement=1 context=1 target=imported label_ref=0 scope=[0] range=136..139 ordinal=0 kind=simple-imported recovery=normal\n",
        references.statement_fingerprint()
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum B5BResolverNodeMutation {
    MissingReferenceKey,
    RelocatedReferenceKey,
}

fn b5b_resolved_ast_with_node_mutation(
    fixture: &B5BFixture,
    mutation: B5BResolverNodeMutation,
) -> Result<ResolvedAst, mizar_resolve::resolved_ast::ResolvedAstError> {
    let mut builder = ResolvedArenaBuilder::new();
    let mut ids = Vec::with_capacity(57);
    for (id, node) in fixture.resolver_ast.nodes().iter() {
        let index = id.index();
        let children = node
            .children()
            .iter()
            .map(|child| ids[child.index()])
            .collect();
        let mut row = ResolvedNode::new(node.kind().clone(), children, node.origin().clone())
            .with_recovery(node.recovery())
            .with_resolution(node.resolution());
        if mutation == B5BResolverNodeMutation::RelocatedReferenceKey && index == 47 {
            row = row
                .with_resolution(NodeResolutionState::Resolved)
                .with_reference_key(NodeReferenceKey::Label(fixture.resolution.ids()[0]));
        } else if !(mutation == B5BResolverNodeMutation::MissingReferenceKey && index == 48)
            && let Some(key) = node.reference_key()
        {
            row = row.with_reference_key(key);
        }
        let inserted = builder.push(row).expect("Task258B5B mutated resolver row");
        assert_eq!(inserted.index(), index);
        ids.push(inserted);
    }
    ResolvedAst::try_new(
        fixture.source,
        fixture.module.clone(),
        builder
            .finish(ids[56])
            .expect("Task258B5B mutated resolver arena"),
        fixture.resolver_ast.name_refs().clone(),
        fixture.resolver_ast.label_refs().clone(),
        fixture.resolver_ast.imports().clone(),
    )
}

fn b5b_resolved_ast_with_relocated_import(fixture: &B5BFixture) -> ResolvedAst {
    let import_range = range(fixture.source, 6, 27);
    let import_owner = fixture
        .resolver_ast
        .nodes()
        .iter()
        .find_map(|(id, _)| (id.index() == 29).then_some(id))
        .expect("Task258B5B import owner");
    let imported_module = ModuleId::new(
        fixture.module.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let mut imports = ResolvedImports::new();
    imports.push_import(ResolvedImport::new(
        import_owner,
        import_range,
        "import parser.type_fixtures;",
        None,
        ImportResolution::Resolved(imported_module),
        SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(import_range),
            vec![0],
        ),
    ));
    ResolvedAst::try_new(
        fixture.source,
        fixture.module.clone(),
        fixture.resolver_ast.nodes().clone(),
        fixture.resolver_ast.name_refs().clone(),
        fixture.resolver_ast.label_refs().clone(),
        imports,
    )
    .expect("Task258B5B relocated import remains a valid resolver AST")
}

fn b5b_projection_with_relocated_origin(fixture: &B5BFixture) -> LabelProjection {
    let projection = &fixture.projection;
    LabelProjection::imported(mizar_resolve::labels::LabelProjectionData {
        origin_path: projection.origin_path().clone(),
        module: projection.module().clone(),
        namespace: projection.namespace().clone(),
        primary_spelling: projection.primary_spelling().to_owned(),
        kind: projection.kind(),
        declaration_range: projection.declaration_range(),
        origin: SemanticOrigin::new(
            projection.origin().source_id(),
            projection.origin().module_id().clone(),
            projection.origin().anchor().clone(),
            vec![1, 1],
        ),
        contribution: projection.contribution(),
    })
    .with_visibility(projection.visibility())
    .with_export_status(projection.export_status())
}

fn b5b_reference_with_relocated_origin(fixture: &B5BFixture) -> LabelReferenceCandidate {
    LabelReferenceCandidate::unqualified_citation(
        mizar_resolve::resolved_ast::ReferenceSite::new(
            fixture.reference.site().node(),
            fixture.reference.site().range(),
            fixture.reference.site().spelling(),
        ),
        SemanticOrigin::new(
            fixture.reference.origin().source_id(),
            fixture.reference.origin().module_id().clone(),
            fixture.reference.origin().anchor().clone(),
            vec![47],
        ),
        fixture.reference.ordinal(),
        Some(LabelScopePath::new(vec![0])),
    )
    .with_expectation(fixture.reference.expectation())
}

fn assert_b5b_final_assembly_rejects(
    fixture: &B5BFixture,
    references: SourceStatementReferenceHandoff,
) {
    let mut corrupted = fixture.empty_typed();
    corrupted.inject_source_statement_bundle_for_test(fixture.statement.clone(), references);
    assert_eq!(
        assemble_empty_resolved(&corrupted),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
}

fn b5b_node_range(index: usize) -> (usize, usize) {
    TASK258B5B_NODE_RANGES[index]
}

fn b5b_node_children(index: usize) -> &'static [usize] {
    TASK258B5B_NODE_CHILDREN[index]
}

fn b5b_typed_node_kind(index: usize) -> &'static str {
    task258b5b_typed_node_kind(index)
}

fn b5b_surface_kind(index: usize) -> syntax::SurfaceNodeKind {
    let token = match index {
        0 => Some((syntax::SurfaceTokenKind::ReservedWord, "import")),
        1 => Some((syntax::SurfaceTokenKind::Identifier, "parser")),
        2 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ".")),
        3 => Some((syntax::SurfaceTokenKind::Identifier, "type_fixtures")),
        4 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        5 => Some((syntax::SurfaceTokenKind::ReservedWord, "reserve")),
        6 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        7 => Some((syntax::SurfaceTokenKind::ReservedWord, "for")),
        8 => Some((syntax::SurfaceTokenKind::ReservedWord, "set")),
        9 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        10 => Some((syntax::SurfaceTokenKind::ReservedWord, "theorem")),
        11 => Some((
            syntax::SurfaceTokenKind::Identifier,
            "FormulaStatementImportedPublicTheoremCitationSmoke",
        )),
        12 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ":")),
        13 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        14 => Some((syntax::SurfaceTokenKind::ReservedSymbol, "=")),
        15 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        16 => Some((syntax::SurfaceTokenKind::ReservedWord, "proof")),
        17 => Some((syntax::SurfaceTokenKind::ReservedWord, "thus")),
        18 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        19 => Some((syntax::SurfaceTokenKind::ReservedSymbol, "=")),
        20 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        21 => Some((syntax::SurfaceTokenKind::ReservedWord, "by")),
        22 => Some((syntax::SurfaceTokenKind::Identifier, "Ref")),
        23 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        24 => Some((syntax::SurfaceTokenKind::ReservedWord, "end")),
        25 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        _ => None,
    };
    if let Some((kind, text)) = token {
        return syntax::SurfaceNodeKind::Token(syntax::SurfaceToken::new(kind, text));
    }
    match index {
        26 | 27 => syntax::SurfaceNodeKind::PathSegment,
        28 => syntax::SurfaceNodeKind::ModulePath,
        29 => syntax::SurfaceNodeKind::ImportAliasDecl,
        30 => syntax::SurfaceNodeKind::ImportItem,
        31 => syntax::SurfaceNodeKind::TypeHead,
        32 => syntax::SurfaceNodeKind::TypeExpression,
        33 => syntax::SurfaceNodeKind::ReserveSegment,
        34 => syntax::SurfaceNodeKind::ReserveItem,
        35 | 37 | 41 | 43 => syntax::SurfaceNodeKind::TermReference,
        36 | 38 | 42 | 44 => syntax::SurfaceNodeKind::TermExpression,
        39 | 45 => syntax::SurfaceNodeKind::BuiltinPredicateApplication,
        40 | 46 => syntax::SurfaceNodeKind::FormulaExpression,
        47 => syntax::SurfaceNodeKind::Proposition,
        48 => syntax::SurfaceNodeKind::Reference,
        49 => syntax::SurfaceNodeKind::ReferenceList,
        50 => syntax::SurfaceNodeKind::JustificationClause,
        51 => syntax::SurfaceNodeKind::ConclusionStatement,
        52 => syntax::SurfaceNodeKind::ProofBlock,
        53 => syntax::SurfaceNodeKind::TheoremItem,
        54 => syntax::SurfaceNodeKind::ItemList,
        55 => syntax::SurfaceNodeKind::CompilationUnit,
        56 => syntax::SurfaceNodeKind::Root,
        _ => unreachable!("Task258B5B has exactly 57 nodes"),
    }
}

fn b5b_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(57);
    for index in 0..57 {
        let (start, end) = b5b_node_range(index);
        let children = b5b_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    b5b_typed_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B5B typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[56])).expect("Task258B5B arena")
}

fn b5b_resolved_arena(
    source: SourceId,
    module: &ModuleId,
    label_ref: Option<LabelRefId>,
) -> ResolvedArena {
    let mut builder = ResolvedArenaBuilder::new();
    let mut ids = Vec::with_capacity(57);
    for index in 0..57 {
        let (start, end) = b5b_node_range(index);
        let children = b5b_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, start, end)),
            vec![index as u32],
        );
        let mut row = ResolvedNode::new(b5b_surface_kind(index), children, origin);
        if index == 48
            && let Some(label_ref) = label_ref
        {
            row = row
                .with_resolution(NodeResolutionState::Resolved)
                .with_reference_key(NodeReferenceKey::Label(label_ref));
        }
        let id = builder.push(row).expect("Task258B5B resolver node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(ids[56]).expect("Task258B5B resolver arena")
}

fn b5b_resolved_ast(
    source: SourceId,
    module: &ModuleId,
    label_ref: Option<LabelRefId>,
    labels: mizar_resolve::resolved_ast::LabelRefTable,
) -> ResolvedAst {
    let nodes = b5b_resolved_arena(source, module, label_ref);
    let import_range = range(source, 7, 27);
    let import_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(import_range),
        vec![0],
    );
    let imported_module = ModuleId::new(
        module.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let import_owner = nodes
        .iter()
        .find_map(|(id, _)| (id.index() == 29).then_some(id))
        .expect("Task258B5B import owner");
    let mut imports = ResolvedImports::new();
    imports.push_import(ResolvedImport::new(
        import_owner,
        import_range,
        "import parser.type_fixtures;",
        None,
        ImportResolution::Resolved(imported_module),
        import_origin,
    ));
    ResolvedAst::try_new(
        source,
        module.clone(),
        nodes,
        mizar_resolve::resolved_ast::NameRefTable::new(),
        labels,
        imports,
    )
    .expect("Task258B5B resolver AST")
}

fn b5b_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let binding = BindingId::new(0);
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: vec![binding],
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 114, 144),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    let mut rows = BindingTable::new();
    rows.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: range(source, 37, 38),
        },
        owner_context: BindingContextId::new(0),
        declaration_range: range(source, 37, 38),
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(range(source, 43, 46)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: rows,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .expect("Task258B5B binding env")
}

fn b5b_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (
    SymbolId,
    SourceContributionId,
    SourceContributionId,
    SymbolEnv,
) {
    const B5B_LABEL: &str = "FormulaStatementImportedPublicTheoremCitationSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(B5B_LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{B5B_LABEL}")),
    );
    let owner_range = range(source, 48, 145);
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(owner_range),
        vec![2, 1],
    );
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 29, 47)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    indexes.symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            B5B_LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let definition = indexes.definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    indexes
        .contributions
        .add_symbol(contribution, symbol.clone());
    indexes
        .contributions
        .add_definition(contribution, definition);
    let import_range = range(source, 7, 27);
    indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(import_range),
    );
    let imported_module = ModuleId::new(
        module.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let imported_contribution = indexes.contributions.insert(
        imported_module.clone(),
        ContributionKind::ImportedSource { source_id: source },
        SourceAnchor::Range(import_range),
    );
    let imported_origin_path = LabelOriginPath::new("summary:parser.type_fixtures::Ref:label:Ref");
    indexes.labels.insert(
        LabelEntry::new(
            imported_origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            "Ref",
            SemanticOrigin::new(
                source,
                imported_module,
                SourceAnchor::Range(import_range),
                vec![1, 0],
            ),
            imported_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    indexes
        .contributions
        .add_label(imported_contribution, imported_origin_path);
    (
        symbol,
        contribution,
        imported_contribution,
        SymbolEnv::new(module.clone(), indexes),
    )
}

fn assert_task269b_exact_mixed_witness_binding() {
    let fixture = B3M1Fixture::new(160);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task269B witnesses");
    let handoff = fixture
        .proof_local(&statement, &witnesses, fixture.proof_local_input())
        .expect("Task269B handoff");

    assert!(statement.is_task_258b3m1_profile());
    assert_eq!(
        (witnesses.witnesses().len(), witnesses.names().len()),
        (2, 1)
    );
    let named = witnesses
        .witnesses()
        .get(SourceStatementWitnessId::new(0))
        .expect("Task269B named witness");
    let unnamed = witnesses
        .witnesses()
        .get(SourceStatementWitnessId::new(1))
        .expect("Task269B unnamed witness");
    assert_eq!(
        (
            named.name(),
            named.term(),
            named.source_ordinal(),
            named.ordinal(),
            named.kind(),
        ),
        (
            Some(SourceStatementWitnessNameId::new(0)),
            SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
            1,
            0,
            SourceStatementWitnessKind::Named,
        )
    );
    assert_eq!(
        (
            unnamed.name(),
            unnamed.term(),
            unnamed.source_ordinal(),
            unnamed.ordinal(),
            unnamed.kind(),
        ),
        (
            None,
            SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(3)),
            1,
            1,
            SourceStatementWitnessKind::Unnamed,
        )
    );

    assert_eq!(
        handoff.base_binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
    assert_eq!(handoff.witness_fingerprint(), witnesses.debug_text());
    assert_eq!(
        handoff.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!(
        handoff.final_binding_fingerprint(),
        handoff.binding_env().debug_text()
    );
    assert_eq!(handoff.declarations().len(), 1);
    assert_eq!(
        (
            handoff.binding_env().contexts().len(),
            handoff.binding_env().bindings().len(),
            handoff.binding_env().diagnostics().len(),
        ),
        (2, 2, 0),
    );
    let proof_context = handoff
        .binding_env()
        .contexts()
        .get(BindingContextId::new(1))
        .expect("Task269B proof context");
    assert_eq!(
        proof_context
            .bindings
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(
        proof_context
            .visible_bindings
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(
        handoff
            .binding_env()
            .bindings()
            .get(BindingId::new(2))
            .is_none(),
        "Task269B unnamed witness must not allocate a checker binding"
    );
    let declaration = handoff
        .declarations()
        .get(SourceProofLocalDeclarationId::new(0))
        .expect("Task269B declaration");
    assert_eq!(
        (
            declaration.witness(),
            declaration.name(),
            declaration.rhs(),
            declaration.binding(),
            declaration.binding_context(),
            declaration.source_ordinal(),
            declaration.visible_after_ordinal(),
        ),
        (
            SourceStatementWitnessId::new(0),
            SourceStatementWitnessNameId::new(0),
            SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
            BindingId::new(1),
            BindingContextId::new(1),
            1,
            1,
        )
    );
    let binding = handoff
        .binding_env()
        .bindings()
        .get(BindingId::new(1))
        .expect("Task269B binding");
    assert_eq!(binding.spelling, "y");
    assert_eq!(binding.kind, BindingKind::LocalAbbreviation);
    assert_eq!(binding.owner_context, BindingContextId::new(1));
    assert_eq!(binding.declaration_range, range(fixture.source, 84, 85));
    assert_eq!(binding.visible_after_ordinal, 1);
    assert_eq!(binding.type_site, BindingTypeSite::Missing);
    assert_eq!(binding.status, BindingStatus::Active);
    assert!(binding.captured.identities().is_empty());
    assert!(binding.diagnostics.is_empty());
    assert_eq!(binding.recovery, BindingRecoveryState::Normal);
    assert_eq!(
        binding.identity,
        BinderIdentity::ResolverLocal {
            scope: LocalTermScope::new(vec![0]),
            ordinal: 1,
            declaration_range: range(fixture.source, 84, 85),
        }
    );
    assert!(matches!(
        handoff.binding_env().lookup(&BindingLookupSite::new(
            "y",
            BindingContextId::new(1),
            Some(LocalTermScope::new(vec![0])),
            1,
        )),
        Ok(BindingLookupResult::ForwardReference { candidates, .. })
            if candidates == [BindingId::new(1)]
    ));
    assert_eq!(
        handoff.binding_env().lookup(&BindingLookupSite::new(
            "y",
            BindingContextId::new(1),
            Some(LocalTermScope::new(vec![0])),
            2,
        )),
        Ok(BindingLookupResult::Local(BindingId::new(1)))
    );
    assert_eq!(
        handoff.debug_text(),
        format!(
            "source-proof-local-declaration-debug-v1\nmodule: pkg::statement.fixture\nbase-binding-fingerprint: {:?}\nstatement-fingerprint: {:?}\nwitness-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\ndeclaration#0 kind=named-witness witness=0 name=0 rhs=primary#2 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal\nfinal-binding-fingerprint: {:?}\n",
            statement.binding_env().debug_text(),
            statement.debug_text(),
            witnesses.debug_text(),
            fixture.primary.debug_text(),
            handoff.binding_env().debug_text(),
        )
    );
}

fn assert_task269b_mixed_witness_corruption_and_cross_profile_fail_closed(
    b3n_fixture: &B3NFixture,
    b3n_statement: &SourceStatementHandoff,
    b3n_witnesses: &SourceStatementWitnessHandoff,
) {
    let fixture = B3M1Fixture::new(161);
    let statement = fixture.statement();
    let pristine_statement = statement.clone();
    let pristine_symbols = fixture.symbols.clone();
    let pristine_bindings = fixture.bindings.clone();
    let pristine_primary = fixture.primary.clone();
    let pristine_atomic = fixture.atomic.clone();
    let pristine_arena = fixture.arena.clone();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task269B witnesses");
    let pristine_input = fixture.proof_local_input();
    let baseline = fixture
        .proof_local(&statement, &witnesses, pristine_input.clone())
        .expect("Task269B baseline");
    let pristine_base = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("Task269B lower pair");
    let pristine_typed = pristine_base
        .clone()
        .with_source_proof_local_declaration(baseline.clone())
        .expect("Task269B upper install");
    let pristine_final = assemble_empty_resolved(&pristine_typed).expect("Task269B final");
    let pristine_typed_debug = pristine_typed.debug_text();
    let pristine_final_debug = pristine_final.debug_text();
    let assert_replay = || {
        assert_eq!(fixture.symbols, pristine_symbols, "Task269B symbols");
        assert_eq!(fixture.bindings, pristine_bindings, "Task269B bindings");
        assert_eq!(fixture.primary, pristine_primary, "Task269B primary");
        assert_eq!(fixture.atomic, pristine_atomic, "Task269B atomic");
        assert_eq!(fixture.arena, pristine_arena, "Task269B arena");
        assert_eq!(statement, pristine_statement, "Task269B statement snapshot");
        assert_eq!(fixture.statement(), statement, "Task269B statement replay");
        assert_eq!(
            fixture
                .witnesses(&statement, fixture.witness_input())
                .expect("Task269B witness replay"),
            witnesses,
            "Task269B witness replay"
        );
        assert_eq!(
            fixture
                .proof_local(&statement, &witnesses, pristine_input.clone())
                .expect("Task269B upper replay"),
            baseline,
            "Task269B upper replay"
        );
        let replay_typed = fixture
            .empty_typed()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone())
            .expect("Task269B pair replay")
            .with_source_proof_local_declaration(baseline.clone())
            .expect("Task269B typed replay");
        assert_eq!(replay_typed.debug_text(), pristine_typed_debug);
        assert_eq!(
            assemble_empty_resolved(&replay_typed)
                .expect("Task269B final replay")
                .debug_text(),
            pristine_final_debug
        );
    };
    let assert_upper_refusal =
        |case: &str,
         input: SourceProofLocalDeclarationHandoffInput,
         attempted_witnesses: &SourceStatementWitnessHandoff| {
            let input_before = input.clone();
            let witnesses_before = attempted_witnesses.clone();
            let primary_before = fixture.primary.clone();
            let arena_before = fixture.arena.clone();
            assert_eq!(
                SourceProofLocalDeclarationProducer::build(
                    input.clone(),
                    &statement,
                    attempted_witnesses,
                    &fixture.primary,
                    &fixture.arena,
                ),
                Err(SourceProofLocalDeclarationError::InvalidDeclaration {
                    declaration: SourceProofLocalDeclarationId::new(0),
                }),
                "{case}"
            );
            assert_eq!(input, input_before, "{case} input purity");
            assert_eq!(
                attempted_witnesses, &witnesses_before,
                "{case} witness purity"
            );
            assert_eq!(fixture.primary, primary_before, "{case} primary purity");
            assert_eq!(fixture.arena, arena_before, "{case} arena purity");
            assert_eq!(statement, pristine_statement, "{case} statement purity");
            assert_replay();
        };

    let mut wrong_range = fixture.proof_local_input();
    wrong_range.declarations[0].local = LocalTermBinding::new(
        "y",
        LocalTermScope::new(vec![0]),
        range(fixture.source, 81, 82),
        1,
    );
    assert_eq!(
        fixture.proof_local(&statement, &witnesses, wrong_range),
        Err(SourceProofLocalDeclarationError::InvalidDeclaration {
            declaration: SourceProofLocalDeclarationId::new(0),
        })
    );
    assert_replay();

    for mutation in 0..9 {
        let mut input = pristine_input.clone();
        let row = &mut input.declarations[0];
        match mutation {
            0 => {
                row.local = LocalTermBinding::new(
                    "z",
                    LocalTermScope::new(vec![0]),
                    range(fixture.source, 84, 85),
                    1,
                )
            }
            1 => {
                row.local = LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![1]),
                    range(fixture.source, 84, 85),
                    1,
                )
            }
            2 => {
                row.local = LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    range(fixture.source, 80, 82),
                    1,
                )
            }
            3 => {
                row.local = LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    range(fixture.source, 84, 85),
                    2,
                )
            }
            4 => row.witness = SourceStatementWitnessId::new(1),
            5 => row.name = SourceStatementWitnessNameId::new(1),
            6 => row.rhs = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(3)),
            7 => row.binding_context = BindingContextId::new(0),
            8 => row.source_ordinal = 2,
            _ => unreachable!(),
        }
        assert_upper_refusal(
            &format!("Task269B declaration field {mutation}"),
            input,
            &witnesses,
        );
    }

    let mut invalid_unnamed = witnesses.clone();
    invalid_unnamed.witnesses.rows[1].source_ordinal = 2;
    assert_eq!(
        fixture.proof_local(&statement, &invalid_unnamed, fixture.proof_local_input(),),
        Err(SourceProofLocalDeclarationError::InvalidDeclaration {
            declaration: SourceProofLocalDeclarationId::new(0),
        })
    );
    assert_replay();
    let mut exact_lower_ordinal_three = witnesses.clone();
    exact_lower_ordinal_three.witnesses.rows[1].source_ordinal = 3;
    assert_eq!(
        SourceProofLocalDeclarationProducer::build(
            pristine_input.clone(),
            &statement,
            &exact_lower_ordinal_three,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidDeclaration {
            declaration: SourceProofLocalDeclarationId::new(0),
        }),
        "exact historical lower witness ordinal 3"
    );
    assert_eq!(
        exact_lower_ordinal_three.witnesses.rows[1].source_ordinal,
        3
    );
    assert_replay();

    for index in 0..TASK258B3M1_NODE_RANGES.len() {
        let input = fixture.proof_local_input();
        let input_before = input.clone();
        let witnesses_before = witnesses.clone();
        let primary_before = fixture.primary.clone();
        let arena = b3m1_typed_arena_with_corrupt_kind(fixture.source, index);
        let arena_before = arena.clone();
        assert_eq!(
            SourceProofLocalDeclarationProducer::build(
                input.clone(),
                &statement,
                &witnesses,
                &fixture.primary,
                &arena,
            ),
            Err(SourceProofLocalDeclarationError::InvalidArena),
            "Task269B node {index}"
        );
        assert_eq!(input, input_before, "Task269B node {index} input purity");
        assert_eq!(
            witnesses, witnesses_before,
            "Task269B node {index} witness purity"
        );
        assert_eq!(
            fixture.primary, primary_before,
            "Task269B node {index} primary purity"
        );
        assert_eq!(arena, arena_before, "Task269B node {index} arena purity");
        assert_replay();
    }

    for corruption in [
        Task269AArenaCorruption::Cardinality,
        Task269AArenaCorruption::Root,
        Task269AArenaCorruption::Anchor,
        Task269AArenaCorruption::Children,
        Task269AArenaCorruption::ResolvedNode,
        Task269AArenaCorruption::Recovery,
        Task269AArenaCorruption::Typing,
        Task269AArenaCorruption::Links,
    ] {
        let input = fixture.proof_local_input();
        let input_before = input.clone();
        let witnesses_before = witnesses.clone();
        let primary_before = fixture.primary.clone();
        let arena = b3m1_typed_arena_with_corruption(fixture.source, corruption);
        let arena_before = arena.clone();
        assert_eq!(
            SourceProofLocalDeclarationProducer::build(
                input.clone(),
                &statement,
                &witnesses,
                &fixture.primary,
                &arena,
            ),
            Err(SourceProofLocalDeclarationError::InvalidArena),
            "Task269B representative {corruption:?}"
        );
        assert_eq!(input, input_before, "Task269B {corruption:?} input purity");
        assert_eq!(
            witnesses, witnesses_before,
            "Task269B {corruption:?} witness purity"
        );
        assert_eq!(
            fixture.primary, primary_before,
            "Task269B {corruption:?} primary purity"
        );
        assert_eq!(arena, arena_before, "Task269B {corruption:?} arena purity");
        assert_replay();
    }

    let mut stale_final = baseline.clone();
    stale_final.set_final_binding_fingerprint_for_test("stale");
    assert_eq!(
        stale_final.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidBindingEnvironment)
    );

    let b3m1_same_identity = B3M1Fixture::new(61);
    let b3m1_statement = b3m1_same_identity.statement();
    let b3m1_witnesses = b3m1_same_identity
        .witnesses(&b3m1_statement, b3m1_same_identity.witness_input())
        .expect("Task269B same-identity witnesses");
    let b3m1_handoff = b3m1_same_identity
        .proof_local(
            &b3m1_statement,
            &b3m1_witnesses,
            b3m1_same_identity.proof_local_input(),
        )
        .expect("Task269B same-identity handoff");
    assert_eq!(b3n_fixture.source, b3m1_same_identity.source);
    assert_eq!(b3n_fixture.module, b3m1_same_identity.module);
    assert_eq!(
        b3m1_handoff.validate_installation(
            b3m1_same_identity.source,
            &b3m1_same_identity.module,
            &b3m1_statement,
            &b3m1_witnesses,
            &b3m1_same_identity.primary,
            &b3n_fixture.arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidArena),
        "Task269B isolated B3N arena mix"
    );
    assert_eq!(
        b3m1_handoff.validate_installation(
            b3m1_same_identity.source,
            &b3m1_same_identity.module,
            b3n_statement,
            &b3m1_witnesses,
            &b3m1_same_identity.primary,
            &b3m1_same_identity.arena,
        ),
        Err(SourceProofLocalDeclarationError::DependencyMismatch),
        "Task269B isolated B3N statement mix"
    );
    assert_eq!(
        b3m1_handoff.validate_installation(
            b3m1_same_identity.source,
            &b3m1_same_identity.module,
            &b3m1_statement,
            b3n_witnesses,
            &b3m1_same_identity.primary,
            &b3m1_same_identity.arena,
        ),
        Err(SourceProofLocalDeclarationError::DependencyMismatch),
        "Task269B isolated B3N witness mix"
    );
    assert_eq!(
        b3m1_handoff.validate_installation(
            b3m1_same_identity.source,
            &b3m1_same_identity.module,
            &b3m1_statement,
            &b3m1_witnesses,
            &b3n_fixture.primary,
            &b3m1_same_identity.arena,
        ),
        Err(SourceProofLocalDeclarationError::DependencyMismatch),
        "Task269B isolated B3N primary mix"
    );
    assert_eq!(
        b3m1_handoff.validate_installation(
            b3n_fixture.source,
            &b3n_fixture.module,
            b3n_statement,
            b3n_witnesses,
            &b3n_fixture.primary,
            &b3n_fixture.arena,
        ),
        Err(SourceProofLocalDeclarationError::DependencyMismatch)
    );
}

fn assert_task269b_mixed_witness_typed_ownership() {
    let fixture = B3M1Fixture::new(162);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task269B witnesses");
    let handoff = fixture
        .proof_local(&statement, &witnesses, fixture.proof_local_input())
        .expect("Task269B handoff");
    let base = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement, witnesses)
        .expect("Task269B lower install");
    let base_nodes = base.nodes().clone();
    let typed = base
        .clone()
        .with_source_proof_local_declaration(handoff.clone())
        .expect("Task269B typed install");
    assert_eq!(typed.source_proof_local_declaration(), Some(&handoff));
    assert_eq!(typed.nodes(), &base_nodes);
    assert_eq!(
        (typed.nodes().len(), typed.nodes().root()),
        (56, Some(TypedNodeId::new(55)))
    );
    assert_eq!(
        typed.with_source_proof_local_declaration(handoff.clone()),
        Err(TypedAstError::InvalidSourceProofLocalDeclaration)
    );
    assert_eq!(
        fixture
            .empty_typed()
            .with_source_proof_local_declaration(handoff),
        Err(TypedAstError::InvalidSourceProofLocalDeclaration)
    );
}

fn assert_task269b_mixed_witness_final_replay_and_deferrals() {
    let fixture = B3M1Fixture::new(163);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task269B witnesses");
    let handoff = fixture
        .proof_local(&statement, &witnesses, fixture.proof_local_input())
        .expect("Task269B handoff");
    let base = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement, witnesses)
        .expect("Task269B lower install");
    let typed = base
        .clone()
        .with_source_proof_local_declaration(handoff.clone())
        .expect("Task269B typed");
    let resolved = assemble_empty_resolved(&typed).expect("Task269B resolved");
    assert_eq!(resolved.source_proof_local_declaration(), Some(&handoff));
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("Task269B replay")
            .debug_text(),
        resolved.debug_text()
    );
    assert_eq!(
        typed
            .source_statement_witnesses()
            .expect("Task269B lower witnesses")
            .witnesses()
            .len(),
        2
    );
    assert_eq!(handoff.declarations().len(), 1);
    assert!(typed.contexts().is_empty());
    assert!(typed.types().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.initial_obligations().is_empty());
    assert!(typed.diagnostics().is_empty());
    assert!(resolved.expr_metadata().is_empty());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.diagnostics().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());

    let b3n = B3NFixture::new(163);
    let b3n_statement = b3n.statement();
    let b3n_witnesses = b3n
        .witnesses(&b3n_statement, b3n.witness_input())
        .expect("Task269A sibling witnesses");
    let b3n_handoff = b3n
        .proof_local(&b3n_statement, &b3n_witnesses, b3n.proof_local_input())
        .expect("Task269A sibling handoff");
    assert_eq!(
        base.with_source_proof_local_declaration(b3n_handoff),
        Err(TypedAstError::InvalidSourceProofLocalDeclaration)
    );
}

#[derive(Clone, Copy)]
enum ResolverMutation {
    ImportedLabelOrigin,
    MissingLabelEffect,
    RecoveredLabel,
    WrongContribution,
    DuplicateLabel,
    StaleLabel,
    WrongLabelKind,
    PrivateLabel,
    LocalOnlyLabel,
    WrongLabelSource,
    WrongLabelModule,
    WrongLabelNamespace,
    WrongResolverNamespace,
    WrongContributionAnchor,
}

fn source_id(source_ordinal: usize) -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "d9".repeat(32)
    ))
    .expect("snapshot");
    let allocator = InMemorySessionIdAllocator::new();
    (0..source_ordinal)
        .map(|_| allocator.next_source_id(snapshot).expect("source"))
        .last()
        .expect("positive source ordinal")
}

fn node(index: usize) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(index))
}

fn b3_node_range(index: usize) -> (usize, usize) {
    const RANGES: [(usize, usize); 49] = [
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
        (82, 83),
        (83, 84),
        (87, 91),
        (92, 93),
        (94, 95),
        (96, 97),
        (97, 98),
        (99, 102),
        (102, 103),
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
        (82, 83),
        (82, 83),
        (82, 83),
        (77, 84),
        (92, 93),
        (92, 93),
        (96, 97),
        (96, 97),
        (92, 97),
        (92, 97),
        (92, 97),
        (87, 98),
        (69, 102),
        (19, 103),
        (0, 103),
        (0, 103),
        (0, 103),
    ];
    RANGES[index]
}

fn b3_node_children(index: usize) -> Vec<usize> {
    match index {
        22 => vec![3],
        23 => vec![22],
        24 => vec![1, 2, 23],
        25 => vec![0, 24, 4],
        26 => vec![8],
        27 => vec![26],
        28 => vec![10],
        29 => vec![28],
        30 => vec![27, 9, 29],
        31 => vec![30],
        32 => vec![13],
        33 => vec![32],
        34 => vec![33],
        35 => vec![12, 34, 14],
        36 => vec![16],
        37 => vec![36],
        38 => vec![18],
        39 => vec![38],
        40 => vec![37, 17, 39],
        41 => vec![40],
        42 => vec![41],
        43 => vec![15, 42, 19],
        44 => vec![11, 35, 43, 20],
        45 => vec![5, 6, 7, 31, 44, 21],
        46 => vec![25, 45],
        47 => vec![46],
        48 => (0..22).chain(std::iter::once(47)).collect(),
        _ => Vec::new(),
    }
}

fn b3_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(49);
    for index in 0..49 {
        let (start, end) = b3_node_range(index);
        let kind = match index {
            26 | 28 | 32 | 36 | 38 => "source.term.variable-reference",
            30 | 40 => "source.formula.atomic.equality",
            34 => "source.statement-witness.item",
            35 => "source.statement-witness.take",
            43 => "source.statement.conclusion",
            45 => "source.statement.theorem",
            _ => "source.surface.unowned",
        };
        let children = b3_node_children(index)
            .into_iter()
            .map(|child| ids[child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                    .with_children(children),
            )
            .expect("Task258B3 typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[48])).expect("Task258B3 arena")
}

fn b3n_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3N_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3N_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3n_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3n_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3N typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[50])).expect("Task258B3N arena")
}

fn b3n_typed_arena_with_corrupt_kind(source: SourceId, corrupt_index: usize) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3N_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3N_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3n_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let kind = if index == corrupt_index {
            "source.task269a.corrupt"
        } else {
            task258b3n_node_kind(index)
        };
        let id = builder
            .push(
                TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                    .with_children(children),
            )
            .expect("Task269A corrupt typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder
        .finish(Some(ids[50]))
        .expect("Task269A corrupt arena")
}

#[derive(Debug, Clone, Copy)]
enum Task269AArenaCorruption {
    Cardinality,
    Root,
    Anchor,
    Children,
    ResolvedNode,
    Recovery,
    Typing,
    Links,
}

fn b3n_typed_arena_with_corruption(
    source: SourceId,
    corruption: Task269AArenaCorruption,
) -> TypedArena {
    let mut resolved_builder = ResolvedArenaBuilder::new();
    let foreign_resolved_node = resolved_builder
        .push(ResolvedNode::new(
            syntax::SurfaceNodeKind::Root,
            Vec::new(),
            SemanticOrigin::new(
                source,
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture")),
                SourceAnchor::Range(range(source, 0, 0)),
                vec![269],
            ),
        ))
        .expect("Task269A foreign resolved-node id");
    let node_count = if matches!(corruption, Task269AArenaCorruption::Cardinality) {
        50
    } else {
        TASK258B3N_NODE_RANGES.len()
    };
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(node_count);
    for (index, (start, end)) in TASK258B3N_NODE_RANGES
        .iter()
        .copied()
        .take(node_count)
        .enumerate()
    {
        let children = task258b3n_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let mut node = TypedNode::new(
            task258b3n_node_kind(index),
            SourceAnchor::Range(range(source, start, end)),
        )
        .with_children(children);
        match (corruption, index) {
            (Task269AArenaCorruption::Anchor, 13) => {
                node.anchor = SourceAnchor::Range(range(source, 80, 82));
            }
            (Task269AArenaCorruption::Children, 37) => node.children.clear(),
            (Task269AArenaCorruption::ResolvedNode, 13) => {
                node = node.with_resolved_node(foreign_resolved_node);
            }
            (Task269AArenaCorruption::Recovery, 13) => {
                node = node.with_recovery(NodeRecoveryState::Recovered);
            }
            (Task269AArenaCorruption::Typing, 13) => {
                node = node.with_typing(TypingState::Successful);
            }
            (Task269AArenaCorruption::Links, 13) => {
                node = node.with_links(TypedNodeLinks {
                    context: Some(LocalTypeContextId::new(0)),
                    ..TypedNodeLinks::default()
                });
            }
            _ => {}
        }
        let id = builder.push(node).expect("Task269A corrupt typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    let root = if matches!(
        corruption,
        Task269AArenaCorruption::Cardinality | Task269AArenaCorruption::Root
    ) {
        ids[49]
    } else {
        ids[50]
    };
    builder
        .finish(Some(root))
        .expect("Task269A representative corrupt arena")
}

fn b3m1_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M1_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M1_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m1_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m1_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M1 typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[55])).expect("Task258B3M1 arena")
}

fn b3m1_typed_arena_with_corrupt_kind(source: SourceId, corrupt_index: usize) -> TypedArena {
    let arena = b3m1_typed_arena(source);
    let mut nodes = arena
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    nodes[corrupt_index].kind = "source.task269b.corrupt".into();
    TypedArena::try_new(arena.root(), nodes)
        .expect("Task269B kind corruption remains structurally valid")
}

fn b3m1_typed_arena_with_corruption(
    source: SourceId,
    corruption: Task269AArenaCorruption,
) -> TypedArena {
    let mut resolved_builder = ResolvedArenaBuilder::new();
    let foreign_resolved_node = resolved_builder
        .push(ResolvedNode::new(
            syntax::SurfaceNodeKind::Root,
            Vec::new(),
            SemanticOrigin::new(
                source,
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture")),
                SourceAnchor::Range(range(source, 0, 0)),
                vec![269, 2],
            ),
        ))
        .expect("Task269B foreign resolved-node id");
    let node_count = if matches!(corruption, Task269AArenaCorruption::Cardinality) {
        55
    } else {
        TASK258B3M1_NODE_RANGES.len()
    };
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(node_count);
    for (index, (start, end)) in TASK258B3M1_NODE_RANGES
        .iter()
        .copied()
        .take(node_count)
        .enumerate()
    {
        let children = task258b3m1_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let mut node = TypedNode::new(
            task258b3m1_node_kind(index),
            SourceAnchor::Range(range(source, start, end)),
        )
        .with_children(children);
        match (corruption, index) {
            (Task269AArenaCorruption::Anchor, 13) => {
                node.anchor = SourceAnchor::Range(range(source, 83, 85));
            }
            (Task269AArenaCorruption::Children, 42) => node.children.clear(),
            (Task269AArenaCorruption::ResolvedNode, 13) => {
                node = node.with_resolved_node(foreign_resolved_node);
            }
            (Task269AArenaCorruption::Recovery, 13) => {
                node = node.with_recovery(NodeRecoveryState::Recovered);
            }
            (Task269AArenaCorruption::Typing, 13) => {
                node = node.with_typing(TypingState::Successful);
            }
            (Task269AArenaCorruption::Links, 13) => {
                node = node.with_links(TypedNodeLinks {
                    context: Some(LocalTypeContextId::new(0)),
                    ..TypedNodeLinks::default()
                });
            }
            _ => {}
        }
        let id = builder.push(node).expect("Task269B corrupt typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    let root = if matches!(
        corruption,
        Task269AArenaCorruption::Cardinality | Task269AArenaCorruption::Root
    ) {
        ids[54]
    } else {
        ids[55]
    };
    builder
        .finish(Some(root))
        .expect("Task269B representative corrupt arena")
}

fn b3m2a_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M2A_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M2A_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m2a_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m2a_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M2A typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[48])).expect("Task258B3M2A arena")
}

fn b3m2b1_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M2B1_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M2B1_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m2b1_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m2b1_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M2B1 typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[52])).expect("Task258B3M2B1 arena")
}

fn b3m2b2a_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M2B2A_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M2B2A_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m2b2a_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m2b2a_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M2B2A typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[56])).expect("Task258B3M2B2A arena")
}

fn b3m2b2b1a_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M2B2B1A_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M2B2B1A_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m2b2b1a_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m2b2b1a_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M2B2B1A typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder
        .finish(Some(ids[62]))
        .expect("Task258B3M2B2B1A arena")
}

fn b3m2b2b1b1_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M2B2B1B1_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M2B2B1B1_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m2b2b1b1_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m2b2b1b1_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M2B2B1B1 typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder
        .finish(Some(ids[66]))
        .expect("Task258B3M2B2B1B1 arena")
}

fn b3m2b2b2a_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M2B2B2A_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M2B2B2A_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m2b2b2a_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m2b2b2a_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M2B2B2A typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder
        .finish(Some(ids[75]))
        .expect("Task258B3M2B2B2A arena")
}

fn b3m2b2b2b_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M2B2B2B_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M2B2B2B_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m2b2b2b_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m2b2b2b_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M2B2B2B typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder
        .finish(Some(ids[78]))
        .expect("Task258B3M2B2B2B arena")
}

fn b3m2b2b2c_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M2B2B2C_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M2B2B2C_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m2b2b2c_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m2b2b2c_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M2B2B2C typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder
        .finish(Some(ids[85]))
        .expect("Task258B3M2B2B2C arena")
}

fn b3_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let base = binding_env(source, module);
    let mut contexts = base.contexts().clone();
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 69, 102),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    assert_eq!(proof, BindingContextId::new(1));
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: base.bindings().clone(),
        diagnostics: base.diagnostics().clone(),
    })
    .expect("Task258B3 binding env")
}

fn b3n_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let base = binding_env(source, module);
    let mut contexts = base.contexts().clone();
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 68, 105),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    assert_eq!(proof, BindingContextId::new(1));
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: base.bindings().clone(),
        diagnostics: base.diagnostics().clone(),
    })
    .expect("Task258B3N binding env")
}

fn b3m1_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let base = binding_env(source, module);
    let mut contexts = base.contexts().clone();
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 71, 111),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    assert_eq!(proof, BindingContextId::new(1));
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: base.bindings().clone(),
        diagnostics: base.diagnostics().clone(),
    })
    .expect("Task258B3M1 binding env")
}

fn b3m2a_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let base = binding_env(source, module);
    let mut contexts = base.contexts().clone();
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 70, 105),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    assert_eq!(proof, BindingContextId::new(1));
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: base.bindings().clone(),
        diagnostics: base.diagnostics().clone(),
    })
    .expect("Task258B3M2A binding env")
}

fn b3m2b1_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let base = binding_env(source, module);
    let mut contexts = base.contexts().clone();
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 76, 111),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    assert_eq!(proof, BindingContextId::new(1));
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: base.bindings().clone(),
        diagnostics: base.diagnostics().clone(),
    })
    .expect("Task258B3M2B1 binding env")
}

fn b3m2b2a_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let base = binding_env(source, module);
    let mut contexts = base.contexts().clone();
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 82, 119),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    assert_eq!(proof, BindingContextId::new(1));
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: base.bindings().clone(),
        diagnostics: base.diagnostics().clone(),
    })
    .expect("Task258B3M2B2A binding env")
}

fn b3m2b2b1a_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let binding = BindingId::new(0);
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: vec![binding],
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 103, 141),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    let mut bindings = BindingTable::new();
    bindings.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: range(source, 37, 38),
        },
        owner_context: BindingContextId::new(0),
        declaration_range: range(source, 37, 38),
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(range(source, 43, 46)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .expect("Task258B3M2B2B1A binding env")
}

fn b3m2b2b1b1_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let binding = BindingId::new(0);
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: vec![binding],
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 116, 156),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    let mut bindings = BindingTable::new();
    bindings.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: range(source, 37, 38),
        },
        owner_context: BindingContextId::new(0),
        declaration_range: range(source, 37, 38),
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(range(source, 43, 46)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .expect("Task258B3M2B2B1B1 binding env")
}

fn b3m2b2b2a_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    structure_witness_binding_env(source, module, 112, 170, "Task258B3M2B2B2A")
}

fn b3m2b2b2b_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    structure_witness_binding_env(source, module, 109, 169, "Task258B3M2B2B2B")
}

fn structure_witness_binding_env(
    source: SourceId,
    module: &ModuleId,
    proof_start: usize,
    proof_end: usize,
    task: &str,
) -> BindingEnv {
    let binding = BindingId::new(0);
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: vec![binding],
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, proof_start, proof_end),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    let mut bindings = BindingTable::new();
    bindings.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: range(source, 37, 38),
        },
        owner_context: BindingContextId::new(0),
        declaration_range: range(source, 37, 38),
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(range(source, 43, 46)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .unwrap_or_else(|error| panic!("{task} binding env: {error}"))
}

fn b3_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const B3_LABEL: &str = "FormulaStatementSingleWitnessSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(B3_LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{B3_LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 103)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            B3_LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            B3_LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn b3n_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaStatementNamedWitnessSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 106)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3n");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn b3m1_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaStatementMultipleWitnessSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 112)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m1");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn b3m2a_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaStatementNumeralWitnessSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 106)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2a");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn b3m2b1_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaStatementParenthesizedWitnessSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 112)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2b1");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn b3m2b2a_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaStatementNestedParenthesizedWitnessSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 120)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2b2a");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn b3m2b2b1a_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (
    SymbolId,
    SourceContributionId,
    SymbolId,
    SourceContributionId,
    SymbolEnv,
) {
    b3m2b2b1a_symbol_env_with_functor_identity(
        source,
        module,
        "summary:parser.type_fixtures#parse-only#++:12",
        "parser.type_fixtures::++#12",
        vec![12],
        ExportStatus::Exported,
        false,
    )
}

fn b3m2b2b1a_symbol_env_with_functor_identity(
    source: SourceId,
    module: &ModuleId,
    functor_local: &str,
    functor_fqn: &str,
    functor_path: Vec<u32>,
    functor_export: ExportStatus,
    pad_contribution: bool,
) -> (
    SymbolId,
    SourceContributionId,
    SymbolId,
    SourceContributionId,
    SymbolEnv,
) {
    const LABEL: &str = "FormulaStatementApplicationWitnessSmoke";
    let theorem = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let theorem_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 48, 142)),
        vec![2, 1],
    );
    let imported_module = ModuleId::new(
        module.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let functor = SymbolId::new(
        imported_module.clone(),
        LocalSymbolId::new(functor_local),
        FullyQualifiedName::new(functor_fqn),
    );
    let mut indexes = SymbolEnvIndexes::default();
    let theorem_contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 29, 47)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    indexes.symbols.insert(
        SymbolEntry::new(
            theorem.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            theorem_origin.clone(),
            theorem_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let definition = indexes.definitions.insert(
        DefinitionShell::new(
            theorem.clone(),
            DefinitionKind::Theorem,
            theorem_origin.clone(),
            theorem_contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2b2b1a");
    indexes.labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace.clone(),
            LABEL,
            theorem_origin,
            theorem_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    indexes
        .contributions
        .add_symbol(theorem_contribution, theorem.clone());
    indexes
        .contributions
        .add_definition(theorem_contribution, definition);
    indexes
        .contributions
        .add_label(theorem_contribution, origin_path);

    let import_range = range(source, 7, 27);
    let import_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(import_range),
        vec![0],
    );
    let mut import_nodes = ResolvedArenaBuilder::new();
    let import_owner = import_nodes
        .push(ResolvedNode::new(
            syntax::SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            import_origin.clone(),
        ))
        .expect("Task258B3M2B2B1A import owner");
    let mut imports = ResolvedImports::new();
    let import = imports.push_import(ResolvedImport::new(
        import_owner,
        import_range,
        "import parser.type_fixtures;",
        None,
        ImportResolution::Resolved(imported_module.clone()),
        import_origin,
    ));
    let import_contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(import_range),
    );
    indexes
        .contributions
        .add_import(import_contribution, import);
    indexes.imports.insert(ImportIndexEntry::new(
        import,
        Some(imported_module.clone()),
        None,
        import_contribution,
    ));

    let functor_contribution = indexes.contributions.insert(
        imported_module.clone(),
        ContributionKind::ImportedSource { source_id: source },
        SourceAnchor::Range(import_range),
    );
    let functor_contribution = if pad_contribution {
        indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(import_range),
        )
    } else {
        functor_contribution
    };
    indexes
        .contributions
        .add_import(functor_contribution, import);
    indexes.symbols.insert(
        SymbolEntry::new(
            functor.clone(),
            SymbolKind::Functor,
            namespace,
            "++",
            SemanticOrigin::new(
                source,
                imported_module,
                SourceAnchor::Range(import_range),
                functor_path,
            ),
            functor_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(functor_export),
    );
    indexes
        .contributions
        .add_symbol(functor_contribution, functor.clone());
    assert_eq!(theorem_contribution.index(), 0);
    assert_eq!(import_contribution.index(), 1);
    assert_eq!(
        functor_contribution.index(),
        if pad_contribution { 3 } else { 2 }
    );
    (
        theorem,
        theorem_contribution,
        functor,
        functor_contribution,
        SymbolEnv::new(module.clone(), indexes),
    )
}

fn b3m2b2b1b1_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (
    SymbolId,
    SourceContributionId,
    SymbolId,
    SourceContributionId,
    SymbolEnv,
) {
    const LABEL: &str = "FormulaStatementParenthesizedApplicationWitnessSmoke";
    let theorem = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let theorem_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 48, 157)),
        vec![2, 1],
    );
    let imported_module = ModuleId::new(
        module.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let functor = SymbolId::new(
        imported_module.clone(),
        LocalSymbolId::new("summary:parser.type_fixtures#parse-only#++:12"),
        FullyQualifiedName::new("parser.type_fixtures::++#12"),
    );
    let mut indexes = SymbolEnvIndexes::default();
    let theorem_contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 29, 47)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    indexes.symbols.insert(
        SymbolEntry::new(
            theorem.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            theorem_origin.clone(),
            theorem_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let definition = indexes.definitions.insert(
        DefinitionShell::new(
            theorem.clone(),
            DefinitionKind::Theorem,
            theorem_origin.clone(),
            theorem_contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2b2b1b1");
    indexes.labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace.clone(),
            LABEL,
            theorem_origin,
            theorem_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    indexes
        .contributions
        .add_symbol(theorem_contribution, theorem.clone());
    indexes
        .contributions
        .add_definition(theorem_contribution, definition);
    indexes
        .contributions
        .add_label(theorem_contribution, origin_path);

    let import_range = range(source, 7, 27);
    let import_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(import_range),
        vec![0],
    );
    let mut import_nodes = ResolvedArenaBuilder::new();
    let import_owner = import_nodes
        .push(ResolvedNode::new(
            syntax::SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            import_origin.clone(),
        ))
        .expect("Task258B3M2B2B1B1 import owner");
    let mut imports = ResolvedImports::new();
    let import = imports.push_import(ResolvedImport::new(
        import_owner,
        import_range,
        "import parser.type_fixtures;",
        None,
        ImportResolution::Resolved(imported_module.clone()),
        import_origin,
    ));
    let import_contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(import_range),
    );
    indexes
        .contributions
        .add_import(import_contribution, import);
    indexes.imports.insert(ImportIndexEntry::new(
        import,
        Some(imported_module.clone()),
        None,
        import_contribution,
    ));

    let functor_contribution = indexes.contributions.insert(
        imported_module.clone(),
        ContributionKind::ImportedSource { source_id: source },
        SourceAnchor::Range(import_range),
    );
    indexes
        .contributions
        .add_import(functor_contribution, import);
    indexes.symbols.insert(
        SymbolEntry::new(
            functor.clone(),
            SymbolKind::Functor,
            namespace,
            "++",
            SemanticOrigin::new(
                source,
                imported_module,
                SourceAnchor::Range(import_range),
                vec![12],
            ),
            functor_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    indexes
        .contributions
        .add_symbol(functor_contribution, functor.clone());
    assert_eq!(theorem_contribution.index(), 0);
    assert_eq!(import_contribution.index(), 1);
    assert_eq!(functor_contribution.index(), 2);
    (
        theorem,
        theorem_contribution,
        functor,
        functor_contribution,
        SymbolEnv::new(module.clone(), indexes),
    )
}

fn b3m2b2b2a_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (
    SymbolId,
    SourceContributionId,
    SymbolId,
    SourceContributionId,
    SymbolEnv,
) {
    b3m2b2b2a_symbol_env_with_structure_identity(
        source,
        module,
        "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5",
        "parser.type_fixtures::TypeCaseStruct#5",
        vec![5],
        ExportStatus::Exported,
        false,
    )
}

fn b3m2b2b2a_symbol_env_with_structure_identity(
    source: SourceId,
    module: &ModuleId,
    structure_local: &str,
    structure_fqn: &str,
    structure_path: Vec<u32>,
    structure_export: ExportStatus,
    pad_contribution: bool,
) -> (
    SymbolId,
    SourceContributionId,
    SymbolId,
    SourceContributionId,
    SymbolEnv,
) {
    structure_witness_symbol_env_with_structure_identity(
        source,
        module,
        "FormulaStatementStructureConstructorWitnessSmoke",
        171,
        "statement.fixture.theorem.b3m2b2b2a",
        structure_local,
        structure_fqn,
        structure_path,
        structure_export,
        pad_contribution,
    )
}

fn b3m2b2b2b_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (
    SymbolId,
    SourceContributionId,
    SymbolId,
    SourceContributionId,
    SymbolEnv,
) {
    structure_witness_symbol_env_with_structure_identity(
        source,
        module,
        "FormulaStatementStructureSelectorWitnessSmoke",
        170,
        "statement.fixture.theorem.b3m2b2b2b",
        "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5",
        "parser.type_fixtures::TypeCaseStruct#5",
        vec![5],
        ExportStatus::Exported,
        false,
    )
}

#[allow(clippy::too_many_arguments)] // Rationale: vary only frozen theorem/root provenance fields in fail-closed tests.
fn structure_witness_symbol_env_with_structure_identity(
    source: SourceId,
    module: &ModuleId,
    label: &str,
    theorem_end: usize,
    label_origin_path: &str,
    structure_local: &str,
    structure_fqn: &str,
    structure_path: Vec<u32>,
    structure_export: ExportStatus,
    pad_contribution: bool,
) -> (
    SymbolId,
    SourceContributionId,
    SymbolId,
    SourceContributionId,
    SymbolEnv,
) {
    let theorem = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(label),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{label}")),
    );
    let theorem_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 48, theorem_end)),
        vec![2, 1],
    );
    let imported_module = ModuleId::new(
        module.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let structure = SymbolId::new(
        imported_module.clone(),
        LocalSymbolId::new(structure_local),
        FullyQualifiedName::new(structure_fqn),
    );
    let mut indexes = SymbolEnvIndexes::default();
    let theorem_contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 29, 47)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    indexes.symbols.insert(
        SymbolEntry::new(
            theorem.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            label,
            theorem_origin.clone(),
            theorem_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let definition = indexes.definitions.insert(
        DefinitionShell::new(
            theorem.clone(),
            DefinitionKind::Theorem,
            theorem_origin.clone(),
            theorem_contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new(label_origin_path);
    indexes.labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace.clone(),
            label,
            theorem_origin,
            theorem_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    indexes
        .contributions
        .add_symbol(theorem_contribution, theorem.clone());
    indexes
        .contributions
        .add_definition(theorem_contribution, definition);
    indexes
        .contributions
        .add_label(theorem_contribution, origin_path);

    let import_range = range(source, 7, 27);
    let import_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(import_range),
        vec![0],
    );
    let mut import_nodes = ResolvedArenaBuilder::new();
    let import_owner = import_nodes
        .push(ResolvedNode::new(
            syntax::SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            import_origin.clone(),
        ))
        .expect("Task258B3M2B2B2A import owner");
    let mut imports = ResolvedImports::new();
    let import = imports.push_import(ResolvedImport::new(
        import_owner,
        import_range,
        "import parser.type_fixtures;",
        None,
        ImportResolution::Resolved(imported_module.clone()),
        import_origin,
    ));
    let import_contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(import_range),
    );
    indexes
        .contributions
        .add_import(import_contribution, import);
    indexes.imports.insert(ImportIndexEntry::new(
        import,
        Some(imported_module.clone()),
        None,
        import_contribution,
    ));

    let structure_contribution = indexes.contributions.insert(
        imported_module.clone(),
        ContributionKind::ImportedSource { source_id: source },
        SourceAnchor::Range(import_range),
    );
    let structure_contribution = if pad_contribution {
        indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(import_range),
        )
    } else {
        structure_contribution
    };
    indexes
        .contributions
        .add_import(structure_contribution, import);
    indexes.symbols.insert(
        SymbolEntry::new(
            structure.clone(),
            SymbolKind::Structure,
            namespace,
            "TypeCaseStruct",
            SemanticOrigin::new(
                source,
                imported_module,
                SourceAnchor::Range(import_range),
                structure_path,
            ),
            structure_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(structure_export),
    );
    indexes
        .contributions
        .add_symbol(structure_contribution, structure.clone());
    assert_eq!(theorem_contribution.index(), 0);
    assert_eq!(import_contribution.index(), 1);
    assert_eq!(
        structure_contribution.index(),
        if pad_contribution { 3 } else { 2 }
    );
    (
        theorem,
        theorem_contribution,
        structure,
        structure_contribution,
        SymbolEnv::new(module.clone(), indexes),
    )
}

fn b2_node_range(index: usize) -> (usize, usize) {
    const RANGES: [(usize, usize); 55] = [
        (0, 7),
        (8, 9),
        (10, 13),
        (14, 17),
        (17, 18),
        (19, 26),
        (27, 64),
        (64, 65),
        (66, 67),
        (68, 69),
        (70, 71),
        (72, 77),
        (80, 86),
        (87, 88),
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
        (66, 67),
        (66, 67),
        (70, 71),
        (70, 71),
        (66, 71),
        (66, 71),
        (87, 88),
        (87, 88),
        (91, 92),
        (91, 92),
        (87, 92),
        (87, 92),
        (87, 92),
        (80, 93),
        (101, 102),
        (101, 102),
        (105, 106),
        (105, 106),
        (101, 106),
        (101, 106),
        (101, 106),
        (96, 107),
        (72, 111),
        (19, 112),
        (0, 112),
        (0, 112),
        (0, 112),
    ];
    RANGES[index]
}

fn b2_node_children(index: usize) -> Vec<usize> {
    match index {
        24 => vec![3],
        25 => vec![24],
        26 => vec![1, 2, 25],
        27 => vec![0, 26, 4],
        28 => vec![8],
        29 => vec![28],
        30 => vec![10],
        31 => vec![30],
        32 => vec![29, 9, 31],
        33 => vec![32],
        34 => vec![13],
        35 => vec![34],
        36 => vec![15],
        37 => vec![36],
        38 => vec![35, 14, 37],
        39 => vec![38],
        40 => vec![39],
        41 => vec![12, 40, 16],
        42 => vec![18],
        43 => vec![42],
        44 => vec![20],
        45 => vec![44],
        46 => vec![43, 19, 45],
        47 => vec![46],
        48 => vec![47],
        49 => vec![17, 48, 21],
        50 => vec![11, 41, 49, 22],
        51 => vec![5, 6, 7, 33, 50, 23],
        52 => vec![27, 51],
        53 => vec![52],
        54 => (0..24).chain(std::iter::once(53)).collect(),
        _ => Vec::new(),
    }
}

fn b2_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(55);
    for index in 0..55 {
        let (start, end) = b2_node_range(index);
        let kind = match index {
            28 | 30 | 34 | 36 | 42 | 44 => "source.term.variable-reference",
            32 | 38 | 46 => "source.formula.atomic.equality",
            41 => "source.statement.assumption",
            49 => "source.statement.conclusion",
            51 => "source.statement.theorem",
            _ => "source.surface.unowned",
        };
        let children = b2_node_children(index)
            .into_iter()
            .map(|child| ids[child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                    .with_children(children),
            )
            .expect("Task258B2 typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[54])).expect("Task258B2 arena")
}

fn b2_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let base = binding_env(source, module);
    let mut contexts = base.contexts().clone();
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 72, 111),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    assert_eq!(proof, BindingContextId::new(1));
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: base.bindings().clone(),
        diagnostics: base.diagnostics().clone(),
    })
    .expect("Task258B2 binding env")
}

fn b2_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const B2_LABEL: &str = "FormulaStatementSingleAssumptionSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(B2_LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{B2_LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 112)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            B2_LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b2");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            B2_LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn b5a_surface_kind(index: usize) -> syntax::SurfaceNodeKind {
    let token = match index {
        0 => Some((syntax::SurfaceTokenKind::ReservedWord, "reserve")),
        1 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        2 => Some((syntax::SurfaceTokenKind::ReservedWord, "for")),
        3 => Some((syntax::SurfaceTokenKind::ReservedWord, "set")),
        4 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        5 => Some((syntax::SurfaceTokenKind::ReservedWord, "theorem")),
        6 => Some((
            syntax::SurfaceTokenKind::Identifier,
            "FormulaStatementAncestorLabelDescendantCitationSmoke",
        )),
        7 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ":")),
        8 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        9 => Some((syntax::SurfaceTokenKind::ReservedSymbol, "=")),
        10 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        11 => Some((syntax::SurfaceTokenKind::ReservedWord, "proof")),
        12 => Some((syntax::SurfaceTokenKind::Identifier, "A")),
        13 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ":")),
        14 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        15 => Some((syntax::SurfaceTokenKind::ReservedSymbol, "=")),
        16 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        17 => Some((syntax::SurfaceTokenKind::ReservedWord, "proof")),
        18 => Some((syntax::SurfaceTokenKind::ReservedWord, "thus")),
        19 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        20 => Some((syntax::SurfaceTokenKind::ReservedSymbol, "=")),
        21 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        22 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        23 => Some((syntax::SurfaceTokenKind::ReservedWord, "end")),
        24 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        25 => Some((syntax::SurfaceTokenKind::ReservedWord, "thus")),
        26 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        27 => Some((syntax::SurfaceTokenKind::ReservedSymbol, "=")),
        28 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        29 => Some((syntax::SurfaceTokenKind::ReservedWord, "proof")),
        30 => Some((syntax::SurfaceTokenKind::ReservedWord, "thus")),
        31 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        32 => Some((syntax::SurfaceTokenKind::ReservedSymbol, "=")),
        33 => Some((syntax::SurfaceTokenKind::Identifier, "x")),
        34 => Some((syntax::SurfaceTokenKind::ReservedWord, "by")),
        35 => Some((syntax::SurfaceTokenKind::Identifier, "A")),
        36 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        37 => Some((syntax::SurfaceTokenKind::ReservedWord, "end")),
        38 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        39 => Some((syntax::SurfaceTokenKind::ReservedWord, "end")),
        40 => Some((syntax::SurfaceTokenKind::ReservedSymbol, ";")),
        _ => None,
    };
    if let Some((kind, text)) = token {
        return syntax::SurfaceNodeKind::Token(syntax::SurfaceToken::new(kind, text));
    }
    match index {
        41 => syntax::SurfaceNodeKind::TypeHead,
        42 => syntax::SurfaceNodeKind::TypeExpression,
        43 => syntax::SurfaceNodeKind::ReserveSegment,
        44 => syntax::SurfaceNodeKind::ReserveItem,
        45 | 47 | 51 | 53 | 58 | 60 | 68 | 70 | 75 | 77 => syntax::SurfaceNodeKind::TermReference,
        46 | 48 | 52 | 54 | 59 | 61 | 69 | 71 | 76 | 78 => syntax::SurfaceNodeKind::TermExpression,
        49 | 55 | 62 | 72 | 79 => syntax::SurfaceNodeKind::BuiltinPredicateApplication,
        50 | 56 | 63 | 73 | 80 => syntax::SurfaceNodeKind::FormulaExpression,
        57 | 64 | 74 | 81 => syntax::SurfaceNodeKind::Proposition,
        65 | 85 | 87 => syntax::SurfaceNodeKind::ConclusionStatement,
        66 | 86 | 88 => syntax::SurfaceNodeKind::ProofBlock,
        67 => syntax::SurfaceNodeKind::CompactStatement,
        82 => syntax::SurfaceNodeKind::Reference,
        83 => syntax::SurfaceNodeKind::ReferenceList,
        84 => syntax::SurfaceNodeKind::JustificationClause,
        89 => syntax::SurfaceNodeKind::TheoremItem,
        90 => syntax::SurfaceNodeKind::ItemList,
        91 => syntax::SurfaceNodeKind::CompilationUnit,
        92 => syntax::SurfaceNodeKind::Root,
        _ => unreachable!("Task258B5A has exactly 93 nodes"),
    }
}

fn b5a_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B5A_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B5A_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b5a_node_children(index)
            .iter()
            .copied()
            .map(|child| ids[child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b5a_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B5A typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[92])).expect("Task258B5A arena")
}

fn b5a_resolved_arena(
    source: SourceId,
    module: &ModuleId,
    label_ref: Option<LabelRefId>,
) -> ResolvedArena {
    b5a_resolved_arena_with_kind_mutation(source, module, label_ref, None)
}

fn b5a_resolved_arena_with_kind_mutation(
    source: SourceId,
    module: &ModuleId,
    label_ref: Option<LabelRefId>,
    kind_mutation: Option<usize>,
) -> ResolvedArena {
    let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B5A_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B5A_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b5a_node_children(index)
            .iter()
            .copied()
            .map(|child| ids[child])
            .collect();
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, start, end)),
            vec![index as u32],
        );
        let kind = if kind_mutation == Some(index) {
            syntax::SurfaceNodeKind::Root
        } else {
            b5a_surface_kind(index)
        };
        let mut row = ResolvedNode::new(kind, children, origin);
        if index == 82
            && let Some(label_ref) = label_ref
        {
            row = row
                .with_resolution(NodeResolutionState::Resolved)
                .with_reference_key(NodeReferenceKey::Label(label_ref));
        }
        let id = builder.push(row).expect("Task258B5A resolver node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(ids[92]).expect("Task258B5A resolver arena")
}

fn b5a_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    let binding = BindingId::new(0);
    let mut contexts = BindingContextTable::new();
    for draft in [
        BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        },
        BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 87, 183),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        },
        BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 104, 131),
            },
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: Vec::new(),
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        },
        BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 146, 178),
            },
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0, 1])),
            bindings: Vec::new(),
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        },
    ] {
        contexts.insert(draft);
    }
    let mut rows = BindingTable::new();
    rows.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: range(source, 8, 9),
        },
        owner_context: BindingContextId::new(0),
        declaration_range: range(source, 8, 9),
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(range(source, 14, 17)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: rows,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .expect("Task258B5A binding env")
}

fn b5a_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaStatementAncestorLabelDescendantCitationSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 184)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b5a");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn b1_node_range(index: usize) -> (usize, usize) {
    const RANGES: [(usize, usize); 77] = [
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
        (77, 78),
        (78, 79),
        (80, 81),
        (82, 83),
        (84, 85),
        (86, 91),
        (96, 100),
        (101, 102),
        (103, 104),
        (105, 106),
        (106, 107),
        (110, 113),
        (113, 114),
        (117, 121),
        (122, 123),
        (124, 125),
        (126, 127),
        (128, 130),
        (131, 132),
        (132, 133),
        (134, 137),
        (137, 138),
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
        (80, 81),
        (80, 81),
        (84, 85),
        (84, 85),
        (80, 85),
        (80, 85),
        (77, 85),
        (101, 102),
        (101, 102),
        (105, 106),
        (105, 106),
        (101, 106),
        (101, 106),
        (101, 106),
        (96, 107),
        (86, 113),
        (77, 114),
        (122, 123),
        (122, 123),
        (126, 127),
        (126, 127),
        (122, 127),
        (122, 127),
        (122, 127),
        (131, 132),
        (131, 132),
        (128, 132),
        (117, 133),
        (69, 137),
        (19, 138),
        (0, 138),
        (0, 138),
        (0, 138),
    ];
    RANGES[index]
}

fn b1_node_children(index: usize) -> Vec<usize> {
    match index {
        34 => vec![3],
        35 => vec![34],
        36 => vec![1, 2, 35],
        37 => vec![0, 36, 4],
        38 => vec![8],
        39 => vec![38],
        40 => vec![10],
        41 => vec![40],
        42 => vec![39, 9, 41],
        43 => vec![42],
        44 => vec![14],
        45 => vec![44],
        46 => vec![16],
        47 => vec![46],
        48 => vec![45, 15, 47],
        49 => vec![48],
        50 => vec![12, 13, 49],
        51 => vec![19],
        52 => vec![51],
        53 => vec![21],
        54 => vec![53],
        55 => vec![52, 20, 54],
        56 => vec![55],
        57 => vec![56],
        58 => vec![18, 57, 22],
        59 => vec![17, 58, 23],
        60 => vec![50, 59, 24],
        61 => vec![26],
        62 => vec![61],
        63 => vec![28],
        64 => vec![63],
        65 => vec![62, 27, 64],
        66 => vec![65],
        67 => vec![66],
        68 => vec![30],
        69 => vec![68],
        70 => vec![29, 69],
        71 => vec![25, 67, 70, 31],
        72 => vec![11, 60, 71, 32],
        73 => vec![5, 6, 7, 43, 72, 33],
        74 => vec![37, 73],
        75 => vec![74],
        76 => (0..34).chain(std::iter::once(75)).collect(),
        _ => Vec::new(),
    }
}

fn b1_surface_kind(index: usize) -> syntax::SurfaceNodeKind {
    match index {
        34 => syntax::SurfaceNodeKind::TypeHead,
        35 => syntax::SurfaceNodeKind::TypeExpression,
        36 => syntax::SurfaceNodeKind::ReserveSegment,
        37 => syntax::SurfaceNodeKind::ReserveItem,
        38 | 40 | 44 | 46 | 51 | 53 | 61 | 63 => syntax::SurfaceNodeKind::TermReference,
        39 | 41 | 45 | 47 | 52 | 54 | 62 | 64 => syntax::SurfaceNodeKind::TermExpression,
        42 | 48 | 55 | 65 => syntax::SurfaceNodeKind::BuiltinPredicateApplication,
        43 | 49 | 56 | 66 => syntax::SurfaceNodeKind::FormulaExpression,
        50 | 57 | 67 => syntax::SurfaceNodeKind::Proposition,
        58 | 71 => syntax::SurfaceNodeKind::ConclusionStatement,
        59 | 72 => syntax::SurfaceNodeKind::ProofBlock,
        60 => syntax::SurfaceNodeKind::CompactStatement,
        68 => syntax::SurfaceNodeKind::Reference,
        69 => syntax::SurfaceNodeKind::ReferenceList,
        70 => syntax::SurfaceNodeKind::JustificationClause,
        73 => syntax::SurfaceNodeKind::TheoremItem,
        74 => syntax::SurfaceNodeKind::ItemList,
        75 => syntax::SurfaceNodeKind::CompilationUnit,
        _ => syntax::SurfaceNodeKind::Root,
    }
}

fn b1_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(77);
    for index in 0..77 {
        let (start, end) = b1_node_range(index);
        let kind = match index {
            38 | 40 | 44 | 46 | 51 | 53 | 61 | 63 => "source.term.variable-reference",
            42 | 48 | 55 | 65 => "source.formula.atomic.equality",
            58 | 71 => "source.statement.conclusion",
            60 => "source.statement.proof-step",
            73 => "source.statement.theorem",
            _ => "source.surface.unowned",
        };
        let children = b1_node_children(index)
            .into_iter()
            .map(|child| ids[child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                    .with_children(children),
            )
            .expect("Task258B1 typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(Some(ids[76])).expect("Task258B1 arena")
}

fn b1_resolved_arena(
    source: SourceId,
    module: &ModuleId,
    label_ref: Option<LabelRefId>,
) -> ResolvedArena {
    let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let mut ids = Vec::with_capacity(77);
    for index in 0..77 {
        let (start, end) = b1_node_range(index);
        let children = b1_node_children(index)
            .into_iter()
            .map(|child| ids[child])
            .collect();
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, start, end)),
            vec![index as u32],
        );
        let mut row = ResolvedNode::new(b1_surface_kind(index), children, origin);
        if index == 68
            && let Some(label_ref) = label_ref
        {
            row = row
                .with_resolution(NodeResolutionState::Resolved)
                .with_reference_key(NodeReferenceKey::Label(label_ref));
        }
        let id = builder.push(row).expect("Task258B1 resolver node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(ids[76]).expect("Task258B1 resolver arena")
}

fn b1_resolved_arena_with_import_edge(
    fixture: &B1Fixture,
    import: mizar_resolve::resolved_ast::ResolvedImportId,
) -> ResolvedArena {
    let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let mut ids = Vec::with_capacity(77);
    for index in 0..77 {
        let (start, end) = b1_node_range(index);
        let children = b1_node_children(index)
            .into_iter()
            .map(|child| ids[child])
            .collect();
        let mut origin = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, start, end)),
            vec![index as u32],
        );
        if index == 50 {
            origin = origin.with_import_edge(import);
        }
        let mut row = ResolvedNode::new(b1_surface_kind(index), children, origin);
        if index == 68 {
            row = row
                .with_resolution(NodeResolutionState::Resolved)
                .with_reference_key(NodeReferenceKey::Label(fixture.resolution.ids()[0]));
        }
        let id = builder.push(row).expect("import-edge resolver row");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder.finish(ids[76]).expect("import-edge resolver arena")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum B1ResolverMutation {
    Root,
    Count,
    OriginSource,
    OriginModule,
    StructuralPath,
    Anchor,
    Children,
    Recovered,
    ReferenceWithoutKey,
    ExtraResolution,
}

fn b1_mutated_resolver_ast(fixture: &B1Fixture, mutation: B1ResolverMutation) -> ResolvedAst {
    b1_mutated_resolver_ast_result(fixture, mutation).expect("valid mutated resolver AST")
}

fn b1_mutated_resolver_ast_result(
    fixture: &B1Fixture,
    mutation: B1ResolverMutation,
) -> Result<ResolvedAst, mizar_resolve::resolved_ast::ResolvedAstError> {
    let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let mut ids = Vec::with_capacity(78);
    for (id, node) in fixture.resolver_ast.nodes().iter() {
        let index = id.index();
        let mut structural_path = node.origin().structural_path().to_vec();
        let mut anchor = node.origin().anchor().clone();
        let mut children = node
            .children()
            .iter()
            .map(|child| ids[child.index()])
            .collect::<Vec<_>>();
        if mutation == B1ResolverMutation::StructuralPath && index == 50 {
            structural_path = vec![999];
        }
        if mutation == B1ResolverMutation::Anchor && index == 50 {
            anchor = SourceAnchor::Range(range(fixture.source, 0, 0));
        }
        if mutation == B1ResolverMutation::Children && index == 71 {
            children.push(ids[58]);
        }
        let origin_source = if mutation == B1ResolverMutation::OriginSource && index == 50 {
            source_id(999)
        } else {
            fixture.source
        };
        let origin_module = if mutation == B1ResolverMutation::OriginModule && index == 50 {
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.other"))
        } else {
            fixture.module.clone()
        };
        let mut row = ResolvedNode::new(
            node.kind().clone(),
            children,
            SemanticOrigin::new(origin_source, origin_module, anchor, structural_path),
        );
        if mutation == B1ResolverMutation::Recovered && index == 50 {
            row = row.with_recovery(RecoveryState::Recovered);
        } else {
            row = row.with_recovery(node.recovery());
        }
        let resolution = if mutation == B1ResolverMutation::ExtraResolution && index == 67 {
            NodeResolutionState::Resolved
        } else {
            node.resolution()
        };
        row = row.with_resolution(resolution);
        if !(mutation == B1ResolverMutation::ReferenceWithoutKey && index == 68)
            && let Some(key) = node.reference_key()
        {
            row = row.with_reference_key(key);
        }
        let inserted = builder.push(row).expect("mutated resolver row");
        assert_eq!(inserted.index(), index);
        ids.push(inserted);
    }
    if mutation == B1ResolverMutation::Count {
        let extra = builder
            .push(ResolvedNode::new(
                syntax::SurfaceNodeKind::Root,
                Vec::new(),
                SemanticOrigin::new(
                    fixture.source,
                    fixture.module.clone(),
                    SourceAnchor::Range(range(fixture.source, 0, 0)),
                    vec![77],
                ),
            ))
            .expect("extra resolver row");
        assert_eq!(extra.index(), 77);
        ids.push(extra);
    }
    let root = if mutation == B1ResolverMutation::Root {
        ids[75]
    } else {
        ids[76]
    };
    ResolvedAst::try_new(
        fixture.source,
        fixture.module.clone(),
        builder.finish(root).expect("mutated resolver arena"),
        mizar_resolve::resolved_ast::NameRefTable::new(),
        fixture.resolution.table().clone(),
        mizar_resolve::resolved_ast::ResolvedImports::new(),
    )
}

fn b1_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    b1_binding_env_with_mutation(source, module, |_| {}).expect("Task258B1 binding env")
}

fn b1_binding_env_with_mutation(
    source: SourceId,
    module: &ModuleId,
    mutate: impl FnOnce(&mut [BindingContextDraft]),
) -> Result<BindingEnv, crate::binding_env::BindingEnvError> {
    let binding = BindingId::new(0);
    let mut drafts = [
        BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        },
        BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 69, 137),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        },
        BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 86, 113),
            },
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: Vec::new(),
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        },
    ];
    mutate(&mut drafts);
    let mut contexts = BindingContextTable::new();
    for draft in drafts {
        contexts.insert(draft);
    }
    let mut rows = BindingTable::new();
    rows.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: range(source, 8, 9),
        },
        owner_context: BindingContextId::new(0),
        declaration_range: range(source, 8, 9),
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(range(source, 14, 17)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: rows,
        diagnostics: BindingDiagnosticTable::new(),
    })
}

fn b1_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const B1_LABEL: &str = "FormulaStatementNestedContextSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(B1_LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{B1_LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 138)),
        vec![1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            B1_LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b1");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            B1_LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let left = builder
        .push(TypedNode::new(
            "source.term.variable-reference",
            SourceAnchor::Range(range(source, 74, 75)),
        ))
        .expect("left");
    let right = builder
        .push(TypedNode::new(
            "source.term.variable-reference",
            SourceAnchor::Range(range(source, 78, 79)),
        ))
        .expect("right");
    let equality_token = builder
        .push(TypedNode::new(
            "source.surface.unowned",
            SourceAnchor::Range(range(source, 76, 77)),
        ))
        .expect("equality token");
    let formula = builder
        .push(
            TypedNode::new(
                "source.formula.atomic.equality",
                SourceAnchor::Range(range(source, 74, 79)),
            )
            .with_children(vec![left, equality_token, right]),
        )
        .expect("formula");
    let wrapper = builder
        .push(
            TypedNode::new(
                "source.surface.unowned",
                SourceAnchor::Range(range(source, 74, 79)),
            )
            .with_children(vec![formula]),
        )
        .expect("formula-expression wrapper");
    let theorem = builder
        .push(
            TypedNode::new(
                "source.statement.theorem",
                SourceAnchor::Range(range(source, 19, 80)),
            )
            .with_children(vec![wrapper]),
        )
        .expect("theorem");
    let root = builder
        .push(
            TypedNode::new("source.module", SourceAnchor::Range(range(source, 0, 80)))
                .with_children(vec![theorem]),
        )
        .expect("root");
    builder.finish(Some(root)).expect("arena")
}

fn mutate_arena(
    arena: &TypedArena,
    mut mutate: impl FnMut(TypedNodeId, &mut TypedNode),
) -> TypedArena {
    TypedArena::try_new(
        arena.root(),
        arena
            .iter()
            .map(|(id, row)| {
                let mut row = row.clone();
                mutate(id, &mut row);
                row
            })
            .collect(),
    )
    .expect("mutated arena remains structurally valid")
}

fn binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    binding_env_with_visible_after(source, module, 0)
}

fn binding_env_with_visible_after(
    source: SourceId,
    module: &ModuleId,
    visible_after_ordinal: usize,
) -> BindingEnv {
    let binding = BindingId::new(0);
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: vec![binding],
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    let mut rows = BindingTable::new();
    rows.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: range(source, 8, 9),
        },
        owner_context: BindingContextId::new(0),
        declaration_range: range(source, 8, 9),
        visible_after_ordinal,
        type_site: BindingTypeSite::Source(range(source, 14, 17)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: rows,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .expect("binding env")
}

fn symbol_env(source: SourceId, module: &ModuleId) -> (SymbolId, SourceContributionId, SymbolEnv) {
    symbol_env_with_mutation(source, module, None)
}

fn symbol_env_with_mutation(
    source: SourceId,
    module: &ModuleId,
    mutation: impl Into<Option<ResolverMutation>>,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    let mutation = mutation.into();
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 80)),
        vec![1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution_anchor = if matches!(mutation, Some(ResolverMutation::WrongContributionAnchor))
    {
        SourceAnchor::Range(range(source, 19, 80))
    } else {
        SourceAnchor::Range(range(source, 0, 18))
    };
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        contribution_anchor,
    );
    let row_contribution = if matches!(mutation, Some(ResolverMutation::WrongContribution)) {
        contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        )
    } else {
        contribution
    };
    let mut symbols = SymbolIndex::new();
    let symbol_namespace = if matches!(mutation, Some(ResolverMutation::WrongResolverNamespace)) {
        NamespacePath::new("statement.wrong")
    } else {
        NamespacePath::new(module.path().as_str())
    };
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            symbol_namespace.clone(),
            LABEL,
            origin.clone(),
            row_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            row_contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let label_path = LabelOriginPath::new("statement.fixture.theorem.0");
    let label_origin = match mutation {
        Some(ResolverMutation::ImportedLabelOrigin) => {
            let mut nodes = ResolvedArenaBuilder::new();
            let owner = nodes
                .push(ResolvedNode::new(
                    syntax::SurfaceNodeKind::ImportAliasDecl,
                    Vec::new(),
                    origin.clone(),
                ))
                .expect("historical imported-label owner");
            let mut imports = ResolvedImports::new();
            let import = imports.push_import(ResolvedImport::new(
                owner,
                range(source, 0, 1),
                "import statement.fixture;",
                None,
                ImportResolution::Resolved(module.clone()),
                origin.clone(),
            ));
            origin.clone().with_import_edge(import)
        }
        Some(ResolverMutation::RecoveredLabel) => origin.clone().recovered(),
        Some(ResolverMutation::StaleLabel) => SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 20, 80)),
            vec![1],
        ),
        Some(ResolverMutation::WrongLabelSource) => SemanticOrigin::new(
            source_id(97),
            module.clone(),
            SourceAnchor::Range(range(source_id(97), 19, 80)),
            vec![1],
        ),
        Some(ResolverMutation::WrongLabelModule) => SemanticOrigin::new(
            source,
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.wrong")),
            SourceAnchor::Range(range(source, 19, 80)),
            vec![1],
        ),
        _ => origin.clone(),
    };
    let label_kind = if matches!(mutation, Some(ResolverMutation::WrongLabelKind)) {
        LabelKind::ProofStep
    } else {
        LabelKind::Theorem
    };
    let label_visibility = if matches!(mutation, Some(ResolverMutation::PrivateLabel)) {
        Visibility::Private
    } else {
        Visibility::Public
    };
    let label_export = if matches!(mutation, Some(ResolverMutation::LocalOnlyLabel)) {
        ExportStatus::LocalOnly
    } else {
        ExportStatus::Exported
    };
    let label_namespace = if matches!(mutation, Some(ResolverMutation::WrongLabelNamespace)) {
        NamespacePath::new("statement.wrong")
    } else {
        symbol_namespace
    };
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            label_path.clone(),
            label_kind,
            label_namespace,
            LABEL,
            label_origin,
            row_contribution,
        )
        .with_visibility(label_visibility)
        .with_export_status(label_export),
    );
    let duplicate_label_path = LabelOriginPath::new("statement.fixture.theorem.1");
    if matches!(mutation, Some(ResolverMutation::DuplicateLabel)) {
        labels.insert(
            LabelEntry::new(
                duplicate_label_path.clone(),
                LabelKind::Theorem,
                NamespacePath::new(module.path().as_str()),
                LABEL,
                origin.clone(),
                row_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
    }
    contributions.add_symbol(row_contribution, symbol.clone());
    contributions.add_definition(row_contribution, definition);
    if !matches!(mutation, Some(ResolverMutation::MissingLabelEffect)) {
        contributions.add_label(row_contribution, label_path);
        if matches!(mutation, Some(ResolverMutation::DuplicateLabel)) {
            contributions.add_label(row_contribution, duplicate_label_path);
        }
    }
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

// Keep each macro-expanded fixture matrix below libtest's default thread stack.
#[inline(never)]
fn run_in_isolated_test_stack_frame(operation: impl FnOnce()) {
    operation();
}

fn b3m2b2b3a_typed_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    let mut ids = Vec::with_capacity(TASK258B3M2B2B3A_NODE_RANGES.len());
    for (index, (start, end)) in TASK258B3M2B2B3A_NODE_RANGES.iter().copied().enumerate() {
        let children = task258b3m2b2b3a_node_children(index)
            .iter()
            .map(|child| ids[*child])
            .collect();
        let id = builder
            .push(
                TypedNode::new(
                    task258b3m2b2b3a_node_kind(index),
                    SourceAnchor::Range(range(source, start, end)),
                )
                .with_children(children),
            )
            .expect("Task258B3M2B2B3A typed node");
        assert_eq!(id.index(), index);
        ids.push(id);
    }
    builder
        .finish(Some(ids[56]))
        .expect("Task258B3M2B2B3A arena")
}

fn b3m2b2b3a_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
    b3m2b2b3a_binding_env_with_mutation(source, module, BindingContextRecovery::Normal, 115, 0)
}

fn b3m2b2b3a_binding_env_with_mutation(
    source: SourceId,
    module: &ModuleId,
    module_recovery: BindingContextRecovery,
    proof_end: usize,
    visible_after_ordinal: usize,
) -> BindingEnv {
    let binding = BindingId::new(0);
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: vec![binding],
        visible_bindings: vec![binding],
        recovery: module_recovery,
    });
    let proof = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(source, 77, proof_end),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    assert_eq!(proof, BindingContextId::new(1));
    let mut bindings = BindingTable::new();
    bindings.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: range(source, 8, 9),
        },
        owner_context: BindingContextId::new(0),
        declaration_range: range(source, 8, 9),
        visible_after_ordinal,
        type_site: BindingTypeSite::Source(range(source, 14, 17)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .expect("Task258B3M2B2B3A binding env")
}

fn b3m2b2b3a_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaStatementSetEnumerationWitnessSmoke";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 116)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m2b2b3a");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

#[derive(Clone)]
struct Task258B4AFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    composite: SourceCompositeFormulaHandoff,
    composition: SourceFormulaCompositionHandoff,
    arena: TypedArena,
}

impl Task258B4AFixture {
    fn new(source_ordinal: usize) -> Self {
        let source = source_id(source_ordinal);
        let module = ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("statement.task258b4a.fixture"),
        );
        let arena = task258b4a_arena(source);
        let (symbol, contribution, symbols) = task258b4a_symbol_env(source, &module);
        let base = task258b4a_binding_base(source, &module);
        let composite_input = task258b4a_composite_input(source, &module);
        let bindings =
            SourceCompositeFormulaProducer::extend_bindings(&composite_input, &base, &arena)
                .expect("Task258B4A bindings");
        let composite = SourceCompositeFormulaProducer::build(composite_input, &bindings, &arena)
            .expect("Task258B4A composite");
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![
                    SourcePrimaryTermInput {
                        site: node(15),
                        source_range: range(source, 72, 73),
                        source_ordinal: 0,
                        context: BindingContextId::new(1),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
                    SourcePrimaryTermInput {
                        site: node(17),
                        source_range: range(source, 76, 77),
                        source_ordinal: 1,
                        context: BindingContextId::new(1),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
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
            },
            &bindings,
            &arena,
        )
        .expect("Task258B4A primary");
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: vec![SourceAtomicFormulaInput {
                    site: node(19),
                    source_range: range(source, 72, 77),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
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
            },
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B4A atomic");
        let composition = SourceFormulaCompositionProducer::build(
            SourceFormulaCompositionHandoffInput {
                source_id: source,
                module_id: module.clone(),
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
            },
            &primary,
            &atomic,
            &composite,
            &arena,
        )
        .expect("Task258B4A composition");
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            composite,
            composition,
            arena,
        }
    }

    fn input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(22),
                source_range: range(self.source, 0, 78),
                spelling: "FormulaQuantifierBoundUsePayloadBoundary".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: vec![SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(0),
                formula: SourceStatementFormulaTarget::Composite(
                    SourceCompositeFormulaId::new(0),
                ),
                site: node(22),
                source_range: range(self.source, 0, 78),
                source_ordinal: 0,
                spelling: "theorem FormulaQuantifierBoundUsePayloadBoundary : for x being set holds x = x ;".to_owned(),
                kind: SourceStatementKind::TheoremProposition,
                recovery: SourceStatementRecovery::Normal,
            }],
            contexts: vec![SourceStatementContextInput {
                statement: SourceStatementId::new(0),
                binding_context: BindingContextId::new(0),
                source_range: range(self.source, 0, 78),
                visible_bindings: Vec::new(),
            }],
            input_facts: Vec::new(),
            candidate_facts: vec![SourceStatementCandidateFactInput {
                statement: SourceStatementId::new(0),
                context: SourceStatementContextId::new(0),
                ordinal: 0,
                kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                formula: SourceStatementFormulaTarget::Composite(
                    SourceCompositeFormulaId::new(0),
                ),
            }],
        }
    }

    fn statement(&self) -> SourceStatementHandoff {
        self.build_statement(self.input())
            .expect("Task258B4A statement")
    }

    fn build_statement(
        &self,
        input: SourceStatementHandoffInput,
    ) -> Result<SourceStatementHandoff, SourceStatementError> {
        SourceStatementProducer::build_with_formula_composition(
            input,
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.composite,
            &self.composition,
            &self.arena,
        )
    }

    fn coherent_relocated_primary_lower(
        &self,
    ) -> (
        TypedArena,
        SourcePrimaryTermHandoff,
        SourceAtomicFormulaHandoff,
        SourceFormulaCompositionHandoff,
    ) {
        let mut nodes = self
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[18].kind = "source.term.variable-reference".into();
        nodes[18].anchor = SourceAnchor::Range(range(self.source, 72, 73));
        let arena = TypedArena::try_new(None, nodes).expect("coherent near-miss arena");
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                terms: vec![
                    SourcePrimaryTermInput {
                        site: node(18),
                        source_range: range(self.source, 72, 73),
                        source_ordinal: 0,
                        context: BindingContextId::new(1),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
                    SourcePrimaryTermInput {
                        site: node(17),
                        source_range: range(self.source, 76, 77),
                        source_ordinal: 1,
                        context: BindingContextId::new(1),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
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
            },
            &self.bindings,
            &arena,
        )
        .expect("coherent near-miss primary");
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                formulas: vec![SourceAtomicFormulaInput {
                    site: node(19),
                    source_range: range(self.source, 72, 77),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
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
            },
            &self.bindings,
            &self.symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("coherent near-miss atomic");
        self.composite
            .validate_installation(self.source, &self.module, &arena)
            .expect("coherent near-miss composite");
        let composition = SourceFormulaCompositionProducer::build(
            SourceFormulaCompositionHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
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
            },
            &primary,
            &atomic,
            &self.composite,
            &arena,
        )
        .expect("coherent near-miss composition");
        (arena, primary, atomic, composition)
    }

    fn typed_base(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B4A typed base")
        .with_source_term(self.primary.clone())
        .expect("Task258B4A primary install")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B4A atomic install")
    }

    fn typed(&self) -> TypedAst {
        self.typed_base()
            .with_source_formula_composition_statement(
                self.composite.clone(),
                self.composition.clone(),
                self.statement(),
            )
            .expect("Task258B4A paired install")
    }
}

#[derive(Clone)]
struct Task258B4BFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    composite: SourceCompositeFormulaHandoff,
    composition: SourceFormulaCompositionHandoff,
    arena: TypedArena,
}

impl Task258B4BFixture {
    fn new() -> Self {
        let lower = crate::source_composite_formula::tests::task_257b2_composite_fixture();
        let source = lower.source;
        let module = lower.module.clone();
        let arena = task258b4b_arena(source);
        let mut composite_input = lower.input.clone();
        for (row, site) in composite_input
            .formulas
            .iter_mut()
            .zip([118, 117, 85, 69, 82, 114, 98, 111])
        {
            row.site = node(site);
        }
        for (row, site) in composite_input
            .wrappers
            .iter_mut()
            .zip([87, 71, 84, 116, 100, 113])
        {
            row.site = node(site);
        }
        composite_input.binders[0].segment_site = node(58);
        composite_input.binders[0].identifier_site = node(4);
        composite_input.type_sites[0].site = node(57);
        composite_input.type_sites[0].head_site = node(56);
        let bindings =
            SourceCompositeFormulaProducer::extend_bindings(&composite_input, &lower.base, &arena)
                .expect("Task258B4B bindings");
        let composite = SourceCompositeFormulaProducer::build(composite_input, &bindings, &arena)
            .expect("Task258B4B composite");

        let numeral_ranges = [
            (74, 75),
            (78, 79),
            (88, 89),
            (92, 93),
            (99, 100),
            (103, 104),
            (115, 116),
            (119, 120),
            (129, 130),
            (133, 134),
            (137, 138),
            (141, 142),
            (148, 149),
            (152, 153),
            (157, 158),
            (161, 162),
        ];
        let numeral_sites = [
            59, 61, 64, 66, 72, 74, 77, 79, 88, 90, 93, 95, 101, 103, 106, 108,
        ];
        let numeral_spellings = [
            "0", "0", "0", "3", "0", "0", "0", "3", "0", "0", "0", "0", "0", "0", "0", "0",
        ];
        let terms = numeral_ranges
            .into_iter()
            .zip(numeral_sites)
            .zip(numeral_spellings)
            .enumerate()
            .map(
                |(source_ordinal, (((start, end), site), spelling))| SourcePrimaryTermInput {
                    site: node(site),
                    source_range: range(source, start, end),
                    source_ordinal,
                    context: BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind: SourcePrimaryTermKind::Numeral,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect::<Vec<_>>();
        let numeric_type_requests = terms
            .iter()
            .enumerate()
            .map(|(request_ordinal, term)| SourceNumericTypeRequestInput {
                term: SourcePrimaryTermId::new(request_ordinal),
                owner: term.site.clone(),
                source_range: term.source_range,
                spelling: term.spelling.clone(),
                request_ordinal,
            })
            .collect();
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms,
                references: Vec::new(),
                numeric_type_requests,
            },
            &bindings,
            &arena,
        )
        .expect("Task258B4B primary");

        let equality_ranges = [
            (74, 79),
            (88, 93),
            (99, 104),
            (115, 120),
            (129, 134),
            (137, 142),
            (148, 153),
            (157, 162),
        ];
        let equality_sites = [63, 68, 76, 81, 92, 97, 105, 110];
        let equality_spellings = [
            "0 = 0", "0 = 3", "0 = 0", "0 = 3", "0 = 0", "0 = 0", "0 = 0", "0 = 0",
        ];
        let formulas = equality_ranges
            .into_iter()
            .zip(equality_sites)
            .zip(equality_spellings)
            .enumerate()
            .map(
                |(source_ordinal, (((start, end), site), spelling))| SourceAtomicFormulaInput {
                    site: node(site),
                    source_range: range(source, start, end),
                    source_ordinal,
                    context: BindingContextId::new(1),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind: SourceAtomicFormulaKind::Equality,
                },
            )
            .collect::<Vec<_>>();
        let edges = (0..8)
            .flat_map(|formula| {
                [
                    SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal: 0,
                        role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            formula * 2,
                        )),
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
        let requests = edges
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas,
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B4B atomic");
        let composition = SourceFormulaCompositionProducer::build(
            SourceFormulaCompositionHandoffInput {
                source_id: source,
                module_id: module.clone(),
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
            },
            &primary,
            &atomic,
            &composite,
            &arena,
        )
        .expect("Task258B4B composition");
        let (symbol, contribution, symbols) = task258b4b_symbol_env(source, &module);
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            composite,
            composition,
            arena,
        }
    }

    fn input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(120),
                source_range: range(self.source, 0, 165),
                spelling: "FormulaConnectiveGroupingPayloadBoundary".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: vec![SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(0),
                formula: SourceStatementFormulaTarget::Composite(
                    SourceCompositeFormulaId::new(0),
                ),
                site: node(120),
                source_range: range(self.source, 0, 165),
                source_ordinal: 0,
                spelling: "theorem FormulaConnectiveGroupingPayloadBoundary : for x being set holds ( ( 0 = 0 & ... & 0 = 3 ) or ( 0 = 0 or ... or 0 = 3 ) ) iff ( ( 0 = 0 & 0 = 0 ) or ( 0 = 0 or 0 = 0 ) ) ;".to_owned(),
                kind: SourceStatementKind::TheoremProposition,
                recovery: SourceStatementRecovery::Normal,
            }],
            contexts: vec![SourceStatementContextInput {
                statement: SourceStatementId::new(0),
                binding_context: BindingContextId::new(0),
                source_range: range(self.source, 0, 165),
                visible_bindings: Vec::new(),
            }],
            input_facts: Vec::new(),
            candidate_facts: vec![SourceStatementCandidateFactInput {
                statement: SourceStatementId::new(0),
                context: SourceStatementContextId::new(0),
                ordinal: 0,
                kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                formula: SourceStatementFormulaTarget::Composite(
                    SourceCompositeFormulaId::new(0),
                ),
            }],
        }
    }

    fn build_statement(
        &self,
        input: SourceStatementHandoffInput,
    ) -> Result<SourceStatementHandoff, SourceStatementError> {
        SourceStatementProducer::build_with_formula_composition(
            input,
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.composite,
            &self.composition,
            &self.arena,
        )
    }

    fn statement(&self) -> SourceStatementHandoff {
        self.build_statement(self.input())
            .expect("Task258B4B statement")
    }

    fn typed_base(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B4B typed base")
        .with_source_term(self.primary.clone())
        .expect("Task258B4B primary install")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B4B atomic install")
    }

    fn typed(&self) -> TypedAst {
        self.typed_base()
            .with_source_formula_composition_statement(
                self.composite.clone(),
                self.composition.clone(),
                self.statement(),
            )
            .expect("Task258B4B paired install")
    }
}

#[derive(Clone)]
struct Task258B4CFixture {
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    symbols: SymbolEnv,
    bindings: BindingEnv,
    primary: SourcePrimaryTermHandoff,
    atomic: SourceAtomicFormulaHandoff,
    composite: SourceCompositeFormulaHandoff,
    composition: SourceFormulaCompositionHandoff,
    arena: TypedArena,
}

impl Task258B4CFixture {
    fn new() -> Self {
        let lower = crate::source_composite_formula::tests::task_257b3_composite_fixture();
        let source = lower.source;
        let module = lower.module.clone();
        let arena = task258b4c_arena(source);
        let mut composite_input = lower.input;
        for (row, site) in composite_input.formulas.iter_mut().zip([60, 59, 58]) {
            row.site = node(site);
        }
        for (row, (segment, identifier)) in
            composite_input
                .binders
                .iter_mut()
                .zip([(38, 9), (46, 17), (47, 22)])
        {
            row.segment_site = node(segment);
            row.identifier_site = node(identifier);
        }
        for (row, (site, head)) in
            composite_input
                .type_sites
                .iter_mut()
                .zip([(37, 36), (45, 44), (33, 32)])
        {
            row.site = node(site);
            row.head_site = node(head);
        }
        let bindings =
            SourceCompositeFormulaProducer::extend_bindings(&composite_input, &lower.base, &arena)
                .expect("Task258B4C bindings");
        let composite = SourceCompositeFormulaProducer::build(composite_input, &bindings, &arena)
            .expect("Task258B4C composite");

        let term_ranges = [
            (86, 87),
            (90, 91),
            (119, 120),
            (123, 124),
            (131, 132),
            (135, 136),
        ];
        let term_sites = [39, 41, 48, 50, 53, 55];
        let term_spellings = ["x", "x", "r", "y", "x", "r"];
        let term_contexts = [1, 1, 3, 3, 3, 3];
        let term_bindings = [1, 1, 3, 2, 1, 3];
        let terms = term_ranges
            .into_iter()
            .zip(term_sites)
            .zip(term_spellings)
            .zip(term_contexts)
            .enumerate()
            .map(
                |(source_ordinal, ((((start, end), site), spelling), context))| {
                    SourcePrimaryTermInput {
                        site: node(site),
                        source_range: range(source, start, end),
                        source_ordinal,
                        context: BindingContextId::new(context),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: spelling.to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    }
                },
            )
            .collect();
        let references = term_bindings
            .into_iter()
            .enumerate()
            .map(|(index, binding)| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: BindingId::new(binding),
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect();
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms,
                references,
                numeric_type_requests: Vec::new(),
            },
            &bindings,
            &arena,
        )
        .expect("Task258B4C primary");

        let equality_ranges = [(86, 91), (119, 124), (131, 136)];
        let equality_sites = [43, 52, 57];
        let equality_spellings = ["x = x", "r = y", "x = r"];
        let equality_contexts = [1, 3, 3];
        let formulas = equality_ranges
            .into_iter()
            .zip(equality_sites)
            .zip(equality_spellings)
            .zip(equality_contexts)
            .enumerate()
            .map(
                |(source_ordinal, ((((start, end), site), spelling), context))| {
                    SourceAtomicFormulaInput {
                        site: node(site),
                        source_range: range(source, start, end),
                        source_ordinal,
                        context: BindingContextId::new(context),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: spelling.to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    }
                },
            )
            .collect::<Vec<_>>();
        let edges = (0..3)
            .flat_map(|formula| {
                [
                    SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal: 0,
                        role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            formula * 2,
                        )),
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
        let requests = edges
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
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas,
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges,
                requests,
            },
            &bindings,
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task258B4C atomic");
        let composition = SourceFormulaCompositionProducer::build(
            SourceFormulaCompositionHandoffInput {
                source_id: source,
                module_id: module.clone(),
                atomic_edges: [
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
                .collect(),
                bound_uses: [0, 0, 2, 1, 0, 2]
                    .into_iter()
                    .zip([0, 1, 0, 0, 2, 1])
                    .zip([0, 0, 1, 1, 2, 2])
                    .enumerate()
                    .map(
                        |(index, ((binder, ordinal), body_edge))| SourceQuantifierBoundUseInput {
                            binder: SourceQuantifierBinderId::new(binder),
                            ordinal,
                            body_edge: SourceFormulaAtomicEdgeId::new(body_edge),
                            term: SourcePrimaryTermId::new(index),
                            reference: SourcePrimaryTermReferenceId::new(index),
                        },
                    )
                    .collect(),
            },
            &primary,
            &atomic,
            &composite,
            &arena,
        )
        .expect("Task258B4C composition");
        let (symbol, contribution, symbols) = task258b4c_symbol_env(source, &module);
        Self {
            source,
            module,
            symbol,
            contribution,
            symbols,
            bindings,
            primary,
            atomic,
            composite,
            composition,
            arena,
        }
    }

    fn input(&self) -> SourceStatementHandoffInput {
        SourceStatementHandoffInput {
            source_id: self.source,
            module_id: self.module.clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: self.symbol.clone(),
                contribution: self.contribution,
                site: node(62),
                source_range: range(self.source, 19, 137),
                spelling: "FormulaNestedQuantifierPayloadBoundary".to_owned(),
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: vec![SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context: SourceStatementContextId::new(0),
                formula: SourceStatementFormulaTarget::Composite(
                    SourceCompositeFormulaId::new(0),
                ),
                site: node(62),
                source_range: range(self.source, 19, 137),
                source_ordinal: 0,
                spelling: "theorem FormulaNestedQuantifierPayloadBoundary : for x being set st x = x ex y being set st for r st r = y holds x = r ;".to_owned(),
                kind: SourceStatementKind::TheoremProposition,
                recovery: SourceStatementRecovery::Normal,
            }],
            contexts: vec![SourceStatementContextInput {
                statement: SourceStatementId::new(0),
                binding_context: BindingContextId::new(0),
                source_range: range(self.source, 19, 137),
                visible_bindings: vec![BindingId::new(0)],
            }],
            input_facts: Vec::new(),
            candidate_facts: vec![SourceStatementCandidateFactInput {
                statement: SourceStatementId::new(0),
                context: SourceStatementContextId::new(0),
                ordinal: 0,
                kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                formula: SourceStatementFormulaTarget::Composite(
                    SourceCompositeFormulaId::new(0),
                ),
            }],
        }
    }

    fn build_statement(
        &self,
        input: SourceStatementHandoffInput,
    ) -> Result<SourceStatementHandoff, SourceStatementError> {
        SourceStatementProducer::build_with_formula_composition(
            input,
            &self.symbols,
            &self.bindings,
            &self.primary,
            &self.atomic,
            &self.composite,
            &self.composition,
            &self.arena,
        )
    }

    fn statement(&self) -> SourceStatementHandoff {
        self.build_statement(self.input())
            .expect("Task258B4C statement")
    }

    fn typed_base(&self) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B4C typed base")
        .with_source_term(self.primary.clone())
        .expect("Task258B4C primary install")
        .with_source_atomic_formula(self.atomic.clone())
        .expect("Task258B4C atomic install")
    }

    fn typed(&self) -> TypedAst {
        self.typed_base()
            .with_source_formula_composition_statement(
                self.composite.clone(),
                self.composition.clone(),
                self.statement(),
            )
            .expect("Task258B4C paired install")
    }
}

fn task258b4c_arena(source: SourceId) -> TypedArena {
    const RANGES: [(usize, usize); 66] = [
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
    let mut nodes = RANGES
        .into_iter()
        .map(|(start, end)| {
            TypedNode::new(
                "source.formula.composition.unowned",
                SourceAnchor::Range(range(source, start, end)),
            )
        })
        .collect::<Vec<_>>();
    let owned = [
        (9, (71, 72), "source.formula.quantifier-binder"),
        (17, (95, 96), "source.formula.quantifier-binder"),
        (22, (114, 115), "source.formula.quantifier-binder"),
        (32, (14, 17), "source.formula.binder-type-head"),
        (33, (14, 17), "source.formula.binder-type"),
        (36, (79, 82), "source.formula.binder-type-head"),
        (37, (79, 82), "source.formula.binder-type"),
        (38, (71, 82), "source.formula.quantifier-binder"),
        (39, (86, 87), "source.term.variable-reference"),
        (41, (90, 91), "source.term.variable-reference"),
        (43, (86, 91), "source.formula.atomic.equality"),
        (44, (103, 106), "source.formula.binder-type-head"),
        (45, (103, 106), "source.formula.binder-type"),
        (46, (95, 106), "source.formula.quantifier-binder"),
        (47, (114, 115), "source.formula.quantifier-binder"),
        (48, (119, 120), "source.term.variable-reference"),
        (50, (123, 124), "source.term.variable-reference"),
        (52, (119, 124), "source.formula.atomic.equality"),
        (53, (131, 132), "source.term.variable-reference"),
        (55, (135, 136), "source.term.variable-reference"),
        (57, (131, 136), "source.formula.atomic.equality"),
        (58, (110, 136), "source.formula.composite.universal"),
        (59, (92, 136), "source.formula.composite.existential"),
        (60, (67, 136), "source.formula.composite.universal"),
        (62, (19, 137), "source.statement.theorem"),
    ];
    for (site, (start, end), kind) in owned {
        nodes[site] = TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)));
    }
    TypedArena::try_new(None, nodes).expect("Task258B4C arena")
}

fn task258b4c_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaNestedQuantifierPayloadBoundary";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.task258b4c.fixture::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 19, 137)),
        vec![2, 1],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 18)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.task258b4c.fixture");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn task258b4b_arena(source: SourceId) -> TypedArena {
    let mut nodes = (0..124)
        .map(|_| {
            TypedNode::new(
                "source.formula.composition.unowned",
                SourceAnchor::Range(range(source, 0, 165)),
            )
        })
        .collect::<Vec<_>>();
    let owned = [
        (4, (54, 55), "source.formula.quantifier-binder"),
        (56, (62, 65), "source.formula.binder-type-head"),
        (57, (62, 65), "source.formula.binder-type"),
        (58, (54, 65), "source.formula.quantifier-binder"),
        (59, (74, 75), "source.term.numeral"),
        (61, (78, 79), "source.term.numeral"),
        (63, (74, 79), "source.formula.atomic.equality"),
        (64, (88, 89), "source.term.numeral"),
        (66, (92, 93), "source.term.numeral"),
        (68, (88, 93), "source.formula.atomic.equality"),
        (
            69,
            (74, 93),
            "source.formula.composite.repeated-conjunction",
        ),
        (71, (73, 94), "source.formula.parenthesized"),
        (72, (99, 100), "source.term.numeral"),
        (74, (103, 104), "source.term.numeral"),
        (76, (99, 104), "source.formula.atomic.equality"),
        (77, (115, 116), "source.term.numeral"),
        (79, (119, 120), "source.term.numeral"),
        (81, (115, 120), "source.formula.atomic.equality"),
        (
            82,
            (99, 120),
            "source.formula.composite.repeated-disjunction",
        ),
        (84, (98, 121), "source.formula.parenthesized"),
        (85, (73, 121), "source.formula.composite.disjunction"),
        (87, (72, 122), "source.formula.parenthesized"),
        (88, (129, 130), "source.term.numeral"),
        (90, (133, 134), "source.term.numeral"),
        (92, (129, 134), "source.formula.atomic.equality"),
        (93, (137, 138), "source.term.numeral"),
        (95, (141, 142), "source.term.numeral"),
        (97, (137, 142), "source.formula.atomic.equality"),
        (98, (129, 142), "source.formula.composite.conjunction"),
        (100, (128, 143), "source.formula.parenthesized"),
        (101, (148, 149), "source.term.numeral"),
        (103, (152, 153), "source.term.numeral"),
        (105, (148, 153), "source.formula.atomic.equality"),
        (106, (157, 158), "source.term.numeral"),
        (108, (161, 162), "source.term.numeral"),
        (110, (157, 162), "source.formula.atomic.equality"),
        (111, (148, 162), "source.formula.composite.disjunction"),
        (113, (147, 163), "source.formula.parenthesized"),
        (114, (128, 163), "source.formula.composite.disjunction"),
        (116, (127, 164), "source.formula.parenthesized"),
        (117, (72, 164), "source.formula.composite.biconditional"),
        (118, (50, 164), "source.formula.composite.universal"),
        (120, (0, 165), "source.statement.theorem"),
    ];
    for (site, (start, end), kind) in owned {
        nodes[site] = TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)));
    }
    TypedArena::try_new(None, nodes).expect("Task258B4B arena")
}

fn task258b4b_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaConnectiveGroupingPayloadBoundary";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.task258b4b.fixture::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 0, 165)),
        vec![2, 0],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 165)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.task258b4b.fixture.theorem.0");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn task258b4a_arena(source: SourceId) -> TypedArena {
    let anchors = [
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
    let kinds = [
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.quantifier-binder",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.binder-type-head",
        "source.formula.binder-type",
        "source.formula.quantifier-binder",
        "source.term.variable-reference",
        "source.formula.composition.unowned",
        "source.term.variable-reference",
        "source.formula.composition.unowned",
        "source.formula.atomic.equality",
        "source.formula.composite.universal",
        "source.formula.composition.unowned",
        "source.statement.theorem",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
        "source.formula.composition.unowned",
    ];
    TypedArena::try_new(
        None,
        kinds
            .into_iter()
            .zip(anchors)
            .map(|(kind, (start, end))| {
                TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
            })
            .collect(),
    )
    .expect("Task258B4A arena")
}

fn task258b4a_binding_base(source: SourceId, module: &ModuleId) -> BindingEnv {
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: Vec::new(),
        visible_bindings: Vec::new(),
        recovery: BindingContextRecovery::Normal,
    });
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
        source_id: source,
        module_id: module.clone(),
        contexts,
        bindings: BindingTable::new(),
        diagnostics,
    })
    .expect("Task258B4A binding base")
}

fn task258b4a_composite_input(
    source: SourceId,
    module: &ModuleId,
) -> SourceCompositeFormulaHandoffInput {
    SourceCompositeFormulaHandoffInput {
        source_id: source,
        module_id: module.clone(),
        formulas: vec![SourceCompositeFormulaInput {
            site: node(20),
            source_range: range(source, 50, 77),
            source_ordinal: 0,
            context: BindingContextId::new(0),
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
            segment_site: node(14),
            segment_range: range(source, 54, 65),
            segment_spelling: "x being".to_owned(),
            identifier_site: node(4),
            identifier_range: range(source, 54, 55),
            identifier_spelling: "x".to_owned(),
            local: LocalTermBinding::new(
                "x",
                LocalTermScope::new(vec![0]),
                range(source, 54, 55),
                0,
            ),
            binding: BindingId::new(0),
            body_context: BindingContextId::new(1),
            type_site: SourceBinderTypeSiteId::new(0),
            recovery: SourceCompositeFormulaRecovery::Normal,
        }],
        type_sites: vec![SourceBinderTypeSiteInput {
            binder: SourceQuantifierBinderId::new(0),
            site: node(13),
            source_range: range(source, 62, 65),
            spelling: "set".to_owned(),
            head_site: node(12),
            head_range: range(source, 62, 65),
            head_spelling: "set".to_owned(),
            context: BindingContextId::new(0),
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
    }
}

fn task258b4a_symbol_env(
    source: SourceId,
    module: &ModuleId,
) -> (SymbolId, SourceContributionId, SymbolEnv) {
    const LABEL: &str = "FormulaQuantifierBoundUsePayloadBoundary";
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new(LABEL),
        FullyQualifiedName::new(format!("pkg::statement.task258b4a.fixture::{LABEL}")),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 0, 78)),
        vec![2, 0],
    );
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 78)),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            namespace.clone(),
            LABEL,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Theorem,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public),
    );
    let origin_path = LabelOriginPath::new("statement.task258b4a.fixture.theorem.0");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            origin_path.clone(),
            LabelKind::Theorem,
            namespace,
            LABEL,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    contributions.add_symbol(contribution, symbol.clone());
    contributions.add_definition(contribution, definition);
    contributions.add_label(contribution, origin_path);
    (
        symbol,
        contribution,
        SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                symbols,
                labels,
                definitions,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        ),
    )
}

fn assemble_empty_resolved(
    typed_ast: &TypedAst,
) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
    assemble_resolved(typed_ast, &ClusterFactTable::new(), Vec::new(), None)
}

fn assemble_resolved(
    typed_ast: &TypedAst,
    cluster_facts: &ClusterFactTable,
    node_hints: Vec<ResolvedNodeKindHint>,
    statement_proofs: Option<StatementProofInputs<'_>>,
) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
    let collection = OverloadCollectionOutput::collect(
        Vec::<OverloadSiteInput>::new(),
        Vec::<OverloadCandidateInput>::new(),
    );
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability =
        CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
    let specificity =
        SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
    let selection =
        OverloadSelectionOutput::resolve(&specificity, Vec::<OverloadSiteResolutionInput>::new());
    ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
        typed_ast,
        cluster_facts,
        overload_collection: &collection,
        template_expansion: &expansion,
        viability: &viability,
        specificity: &specificity,
        overload_selection: &selection,
        expressions: Vec::new(),
        node_hints,
        statement_semantics: None,
        statement_proofs,
    })
}

fn assemble_resolved_with_expressions(
    typed_ast: &TypedAst,
    expressions: Vec<ExpressionMetadataInput>,
) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
    let cluster_facts = ClusterFactTable::new();
    let collection = OverloadCollectionOutput::collect(
        Vec::<OverloadSiteInput>::new(),
        Vec::<OverloadCandidateInput>::new(),
    );
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability =
        CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
    let specificity =
        SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
    let selection =
        OverloadSelectionOutput::resolve(&specificity, Vec::<OverloadSiteResolutionInput>::new());
    ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
        typed_ast,
        cluster_facts: &cluster_facts,
        overload_collection: &collection,
        template_expansion: &expansion,
        viability: &viability,
        specificity: &specificity,
        overload_selection: &selection,
        expressions,
        node_hints: Vec::new(),
        statement_semantics: None,
        statement_proofs: None,
    })
}

fn assemble_resolved_with_statement_semantics(
    typed_ast: &TypedAst,
    owner: &CheckedStatementOwner,
    binding_env: &BindingEnv,
    term_formula: &crate::type_checker::TermFormulaInferenceOutput,
) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
    let cluster_facts = ClusterFactTable::new();
    let collection = OverloadCollectionOutput::collect(
        Vec::<OverloadSiteInput>::new(),
        Vec::<OverloadCandidateInput>::new(),
    );
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability =
        CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
    let specificity =
        SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
    let selection =
        OverloadSelectionOutput::resolve(&specificity, Vec::<OverloadSiteResolutionInput>::new());
    ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
        typed_ast,
        cluster_facts: &cluster_facts,
        overload_collection: &collection,
        template_expansion: &expansion,
        viability: &viability,
        specificity: &specificity,
        overload_selection: &selection,
        expressions: Vec::new(),
        node_hints: Vec::new(),
        statement_semantics: Some(StatementSemanticInputs {
            owner,
            binding_env,
            term_formula,
            rows: Vec::new(),
        }),
        statement_proofs: Some(StatementProofInputs {
            owner,
            rows: Vec::new(),
        }),
    })
}

mod corruption;
mod lower;
mod misc;
mod task;
mod task258b1;
mod task258b2;
mod task258b3;
mod task258b3m1;
mod task258b3m2a;
mod task258b3m2b1;
mod task258b3m2b2a;
mod task258b3m2b2b1a;
mod task258b3m2b2b1b1;
mod task258b3m2b2b2a;
mod task258b3m2b2b2b;
mod task258b3m2b2b2c;
mod task258b3m2b2b3a;
mod task258b3m2b2b3b;
mod task258b3m2b2b3c;
mod task258b3m2b2b3d;
mod task258b3m2b2b3e;
mod task258b3n;
mod task269a;
