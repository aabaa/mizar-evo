use super::*;
use crate::{
    cluster_trace::ClusterFactTable,
    overload_resolution::{
        CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
        OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
        OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
        TemplateExpansionOutput,
    },
    resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstError, ResolvedTypedAstInputs},
    source_atomic_formula::{
        SourceAtomicEdgeInput, SourceAtomicFormulaHandoffInput, SourceAtomicFormulaInput,
        SourceAtomicFormulaProducer, SourceAtomicRequestInput,
    },
    source_context::{
        SourceBindingContextInput, SourceBindingContextOwner, SourceBindingContextProducer,
        SourceBindingSiteInput, SourceItemInput,
    },
    source_functor_definition::tests::actual_definition_family_typed_asts_for_task261,
    source_term::{
        SourcePrimaryTermHandoffInput, SourcePrimaryTermInput, SourcePrimaryTermProducer,
        SourcePrimaryTermReferenceInput,
    },
    source_type::{
        SourceTypeApplicationInput, SourceTypeExpressionInput, SourceTypeHandoffInput,
        SourceTypeProducer,
    },
    typed_ast::{
        CoercionTable, InitialObligationDraft, InitialObligationGoal, InitialObligationKind,
        InitialObligationProvenance, InitialObligationStatus, InitialObligationTable,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedAst, TypedAstError, TypedAstParts,
        TypedNode, TypedNodeLinks, TypingState,
    },
};
use mizar_resolve::{
    declarations::DeclarationShellCollector,
    env::{DefinitionShell, NamespacePath, SignatureShell, SymbolEntry, SymbolEnvIndexes},
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{FullyQualifiedName, LocalSymbolId},
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
};
use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind};

const TYPE_HEAD_X: TypedNodeId = TypedNodeId::new(24);
const TYPE_EXPRESSION_X: TypedNodeId = TypedNodeId::new(25);
const PARAMETER_X: TypedNodeId = TypedNodeId::new(27);
const TYPE_HEAD_Y: TypedNodeId = TypedNodeId::new(28);
const TYPE_EXPRESSION_Y: TypedNodeId = TypedNodeId::new(29);
const PARAMETER_Y: TypedNodeId = TypedNodeId::new(31);
const DEFINIENS_LEFT: TypedNodeId = TypedNodeId::new(33);
const DEFINIENS_RIGHT: TypedNodeId = TypedNodeId::new(35);
const DEFINIENS_FORMULA: TypedNodeId = TypedNodeId::new(37);
const DEFINIENS_OWNER: TypedNodeId = TypedNodeId::new(39);
const ATTRIBUTE_OWNER: TypedNodeId = TypedNodeId::new(40);
const DEFINITION_BLOCK: TypedNodeId = TypedNodeId::new(41);
const MODULE_ROOT: TypedNodeId = TypedNodeId::new(44);

#[derive(Clone)]
struct Fixture {
    source: SourceId,
    module: ModuleId,
    env: SymbolEnv,
    source_context: SourceBindingContextHandoff,
    source_type: SourceTypeApplicationHandoff,
    source_term: SourcePrimaryTermHandoff,
    source_atomic_formula: SourceAtomicFormulaHandoff,
    arena: TypedArena,
    input: SourceAttributeDefinitionHandoffInput,
}

impl Fixture {
    fn build(&self) -> Result<SourceAttributeDefinitionHandoff, SourceAttributeDefinitionError> {
        SourceAttributeDefinitionProducer::build(
            self.input.clone(),
            &self.env,
            &self.source_context,
            &self.source_type,
            &self.source_term,
            &self.source_atomic_formula,
            &self.arena,
        )
    }

    fn handoff(&self) -> SourceAttributeDefinitionHandoff {
        self.build().expect("exact Task 261 handoff")
    }

    fn typed_ast(&self, obligations: InitialObligationTable) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: Some(self.source_context.clone()),
            source_type: Some(self.source_type.clone()),
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: self.source_context.local_contexts().clone(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: obligations,
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task 261 typed baseline")
        .with_source_term(self.source_term.clone())
        .expect("Task 252 installation")
        .with_source_atomic_formula(self.source_atomic_formula.clone())
        .expect("Task 256 installation")
    }
}

pub(crate) fn actual_attribute_definition_typed_ast_for_task262() -> TypedAst {
    let fixture = fixture();
    fixture
        .typed_ast(InitialObligationTable::new())
        .with_source_attribute_definition(fixture.handoff())
        .expect("actual Task 261 attribute installation for Task 262 isolation")
}

#[test]
fn source_attribute_definition_builds_exact_handoff_and_preserves_obligations() {
    let fixture = fixture();
    let first = fixture.handoff();
    let second = fixture.handoff();
    assert_eq!(first, second);
    assert_eq!(first.source_id(), fixture.source);
    assert_eq!(first.module_id(), &fixture.module);
    assert_eq!(
        first.source_context_fingerprint(),
        fixture.source_context.debug_text()
    );
    assert_eq!(
        first.source_type_fingerprint(),
        fixture.source_type.debug_text()
    );
    assert_eq!(
        first.source_term_fingerprint(),
        fixture.source_term.debug_text()
    );
    assert_eq!(
        first.source_atomic_formula_fingerprint(),
        fixture.source_atomic_formula.debug_text()
    );
    assert_eq!(first.definitions().len(), 1);
    assert_eq!(first.parameters().len(), 2);
    assert_eq!(first.subjects().len(), 1);
    assert_eq!(first.definientia().len(), 1);
    assert!(!first.definitions().is_empty());

    let definition = first
        .definitions()
        .get(SourceAttributeDefinitionId::new(0))
        .unwrap();
    assert_eq!(definition.id(), SourceAttributeDefinitionId::new(0));
    assert_eq!(definition.symbol(), &fixture.input.definitions[0].symbol);
    assert_eq!(definition.definition().index(), 0);
    assert_eq!(definition.contribution().index(), 0);
    assert_eq!(definition.site(), &TypedSiteRef::Node(ATTRIBUTE_OWNER));
    assert_eq!(definition.source_range(), range(fixture.source, 45, 110));
    assert_eq!(definition.source_ordinal(), 0);
    assert_eq!(definition.context(), BindingContextId::new(1));
    assert_eq!(
        definition.recovery(),
        SourceAttributeDefinitionRecovery::Normal
    );
    assert_eq!(
        definition.spelling(),
        "attr Task261AttributeDefinition: x is task261_marked means x = y;"
    );
    assert_eq!(definition.subject(), SourceAttributeSubjectId::new(0));
    assert_eq!(definition.definiens(), SourceAttributeDefiniensId::new(0));
    assert_eq!(definition.origin().structural_path(), &[4, 0, 7, 0]);

    for (index, row) in first.parameters().iter() {
        assert_eq!(row.id(), index);
        assert_eq!(row.owner(), SourceAttributeDefinitionId::new(0));
        assert_eq!(row.ordinal(), index.index());
        assert_eq!(row.binding(), BindingId::new(index.index()));
        assert_eq!(
            row.written_type(),
            SourceTypeApplicationId::new(index.index())
        );
        assert_eq!(row.context(), BindingContextId::new(1));
        assert_eq!(row.recovery(), SourceAttributeDefinitionRecovery::Normal);
    }
    let subject = first
        .subjects()
        .get(SourceAttributeSubjectId::new(0))
        .unwrap();
    assert_eq!(subject.id(), SourceAttributeSubjectId::new(0));
    assert_eq!(subject.owner(), SourceAttributeDefinitionId::new(0));
    assert_eq!(subject.binding(), BindingId::new(0));
    assert_eq!(subject.site(), &TypedSiteRef::Node(ATTRIBUTE_OWNER));
    assert_eq!(subject.source_range(), range(fixture.source, 78, 79));
    assert_eq!(subject.context(), BindingContextId::new(1));
    assert_eq!(
        subject.recovery(),
        SourceAttributeDefinitionRecovery::Normal
    );
    assert_eq!(subject.spelling(), "x");
    let definiens = first
        .definientia()
        .get(SourceAttributeDefiniensId::new(0))
        .unwrap();
    assert_eq!(definiens.id(), SourceAttributeDefiniensId::new(0));
    assert_eq!(definiens.owner(), SourceAttributeDefinitionId::new(0));
    assert_eq!(definiens.ordinal(), 0);
    assert_eq!(definiens.formula(), SourceAtomicFormulaId::new(0));
    assert_eq!(definiens.site(), &TypedSiteRef::Node(DEFINIENS_OWNER));
    assert_eq!(definiens.source_range(), range(fixture.source, 104, 109));
    assert_eq!(definiens.context(), BindingContextId::new(1));
    assert_eq!(
        definiens.recovery(),
        SourceAttributeDefinitionRecovery::Normal
    );
    assert_eq!(definiens.spelling(), "x = y");
    assert_eq!(first.debug_text(), expected_debug(&fixture));

    let baseline = non_empty_baseline(fixture.source);
    let untouched = baseline.clone();
    let typed = fixture
        .typed_ast(baseline)
        .with_source_attribute_definition(first)
        .expect("Task 261 installation");
    assert_eq!(typed.initial_obligations(), &untouched);
}

#[test]
fn source_attribute_definition_rejects_input_and_resolver_corruption() {
    let fixture = fixture();
    let mut input = fixture.input.clone();
    input.definitions.clear();
    assert_build_error(
        &fixture,
        input,
        SourceAttributeDefinitionError::UnsupportedTaskShape,
    );

    let mut input = fixture.input.clone();
    input.definitions[0].source_range = range(fixture.source, 45, 109);
    assert_build_error(
        &fixture,
        input,
        SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 },
    );

    let mut input = fixture.input.clone();
    input.parameters[1].binding = BindingId::new(0);
    assert_build_error(
        &fixture,
        input,
        SourceAttributeDefinitionError::InvalidParameter { index: 1 },
    );

    let mut input = fixture.input.clone();
    input.subjects[0].source_range = range(fixture.source, 77, 79);
    assert_build_error(
        &fixture,
        input,
        SourceAttributeDefinitionError::InvalidSubject { index: 0 },
    );

    let mut input = fixture.input.clone();
    input.definientia[0].formula = SourceAtomicFormulaId::new(1);
    assert_build_error(
        &fixture,
        input,
        SourceAttributeDefinitionError::InvalidDefiniens { index: 0 },
    );

    let (bad_env, symbol, definition, contribution) = resolver_env_with_path(
        fixture.source,
        fixture.module.clone(),
        vec![4, 0, 7, 1],
        Some(0),
    );
    let mut input = fixture.input.clone();
    input.definitions[0].symbol = symbol;
    input.definitions[0].definition = definition;
    input.definitions[0].contribution = contribution;
    assert_eq!(
        SourceAttributeDefinitionProducer::build(
            input,
            &bad_env,
            &fixture.source_context,
            &fixture.source_type,
            &fixture.source_term,
            &fixture.source_atomic_formula,
            &fixture.arena,
        ),
        Err(SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 })
    );

    let source = fixture.source;
    let module = &fixture.module;
    for arity in [None, Some(1), Some(2)] {
        let mut changed = fixture.clone();
        changed.env = resolver_env_with_path(source, module.clone(), vec![4, 0, 7, 0], arity).0;
        assert_eq!(
            changed.build(),
            Err(SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 })
        );
    }

    for error in [
        SourceAttributeDefinitionError::SourceIdentityMismatch,
        SourceAttributeDefinitionError::DependencyMismatch,
        SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 },
        SourceAttributeDefinitionError::InvalidDefinition { index: 0 },
        SourceAttributeDefinitionError::InvalidParameter { index: 0 },
        SourceAttributeDefinitionError::InvalidSubject { index: 0 },
        SourceAttributeDefinitionError::InvalidDefiniens { index: 0 },
        SourceAttributeDefinitionError::InvalidArenaOwnership,
        SourceAttributeDefinitionError::UnsupportedTaskShape,
    ] {
        assert!(!error.to_string().is_empty());
        let _: &dyn std::error::Error = &error;
    }
}

#[test]
fn source_attribute_definition_rejects_lower_dependency_and_fingerprint_corruption() {
    let fixture = fixture();
    let exact = fixture.handoff();
    exact
        .validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &fixture.source_type,
            &fixture.source_term,
            &fixture.source_atomic_formula,
            &fixture.arena,
        )
        .unwrap();

    for field in 0..4 {
        let mut handoff = exact.clone();
        match field {
            0 => handoff.source_context_fingerprint.push_str("-stale"),
            1 => handoff.source_type_fingerprint.push_str("-stale"),
            2 => handoff.source_term_fingerprint.push_str("-stale"),
            3 => handoff.source_atomic_formula_fingerprint.push_str("-stale"),
            _ => unreachable!(),
        }
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.source_context,
                &fixture.source_type,
                &fixture.source_term,
                &fixture.source_atomic_formula,
                &fixture.arena,
            ),
            Err(SourceAttributeDefinitionError::DependencyMismatch)
        );
    }

    let mut handoff = exact.clone();
    handoff.subjects.rows[0].site = TypedSiteRef::Node(TypedNodeId::new(38));
    assert_eq!(
        validate_exact_handoff(&fixture, &handoff),
        Err(SourceAttributeDefinitionError::InvalidSubject { index: 0 })
    );
    let mut handoff = exact.clone();
    handoff.definientia.rows[0].id = SourceAttributeDefiniensId::new(1);
    assert_eq!(
        validate_exact_handoff(&fixture, &handoff),
        Err(SourceAttributeDefinitionError::InvalidDefiniens { index: 0 })
    );
    let mut handoff = exact;
    handoff.definitions.rows[0].origin = SemanticOrigin::new(
        fixture.source,
        fixture.module.clone(),
        SourceAnchor::Range(range(fixture.source, 45, 110)),
        vec![4, 0, 7, 1],
    );
    assert!(matches!(
        validate_exact_handoff(&fixture, &handoff),
        Err(SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 })
            | Err(SourceAttributeDefinitionError::InvalidDefinition { index: 0 })
    ));
}

#[test]
fn source_attribute_definition_installs_atomically_and_isolates_other_definition_families() {
    let fixture = fixture();
    let baseline = non_empty_baseline(fixture.source);
    let before = fixture.typed_ast(baseline.clone());
    let untouched = before.clone();
    let handoff = fixture.handoff();
    let typed = before
        .clone()
        .with_source_attribute_definition(handoff.clone())
        .expect("one-shot Task 261 installation");
    assert_eq!(before, untouched);
    assert!(before.source_attribute_definition().is_none());
    assert_eq!(typed.source_attribute_definition(), Some(&handoff));
    assert_eq!(typed.initial_obligations(), &baseline);
    assert!(typed.source_predicate_definition().is_none());
    assert!(typed.source_functor_definition().is_none());
    assert!(matches!(
        typed.clone().with_source_attribute_definition(handoff),
        Err(TypedAstError::InvalidSourceAttributeDefinition)
    ));

    let (predicate_typed, functor_typed) = actual_definition_family_typed_asts_for_task261();
    assert!(predicate_typed.source_predicate_definition().is_some());
    assert!(functor_typed.source_functor_definition().is_some());

    let mut predicate_preinstalled = before.clone();
    predicate_preinstalled.inject_source_predicate_definition_for_test(
        predicate_typed
            .source_predicate_definition()
            .expect("actual Task 259 handoff")
            .clone(),
    );
    assert!(matches!(
        predicate_preinstalled.with_source_attribute_definition(fixture.handoff()),
        Err(TypedAstError::InvalidSourceAttributeDefinition)
    ));

    let mut functor_preinstalled = before.clone();
    functor_preinstalled.inject_source_functor_definition_for_test(
        functor_typed
            .source_functor_definition()
            .expect("actual Task 260 handoff")
            .clone(),
    );
    assert!(matches!(
        functor_preinstalled.with_source_attribute_definition(fixture.handoff()),
        Err(TypedAstError::InvalidSourceAttributeDefinition)
    ));

    let mut predicate_mixed = typed.clone();
    predicate_mixed.inject_source_predicate_definition_for_test(
        predicate_typed
            .source_predicate_definition()
            .expect("actual Task 259 handoff")
            .clone(),
    );
    assert!(matches!(
        assemble_empty(&predicate_mixed),
        Err(ResolvedTypedAstError::InvalidSourceAttributeDefinition)
    ));

    let mut functor_mixed = typed.clone();
    functor_mixed.inject_source_functor_definition_for_test(
        functor_typed
            .source_functor_definition()
            .expect("actual Task 260 handoff")
            .clone(),
    );
    assert!(matches!(
        assemble_empty(&functor_mixed),
        Err(ResolvedTypedAstError::InvalidSourceAttributeDefinition)
    ));

    let mut injected = before.clone();
    injected.inject_source_attribute_definition_for_test(fixture.handoff());
    assert!(injected.source_attribute_definition().is_some());
    assert_eq!(injected.initial_obligations(), &baseline);

    let mut corrupt = fixture.handoff();
    corrupt.parameters.rows[0].ordinal = 1;
    let fresh = fixture.typed_ast(baseline.clone());
    let preserved = fresh.clone();
    assert!(matches!(
        fresh.with_source_attribute_definition(corrupt),
        Err(TypedAstError::InvalidSourceAttributeDefinition)
    ));
    assert!(preserved.source_attribute_definition().is_none());
    assert_eq!(preserved.initial_obligations(), &baseline);
}

#[test]
fn source_attribute_definition_finalizes_deterministically_without_semantic_publication() {
    let fixture = fixture();
    let baseline = non_empty_baseline(fixture.source);
    let handoff = fixture.handoff();
    let legacy = fixture.typed_ast(baseline.clone());
    assert!(
        !legacy
            .debug_text()
            .contains("source-attribute-definition-debug-v1")
    );
    let legacy_resolved = assemble_empty(&legacy).expect("legacy final assembly");
    assert!(legacy_resolved.source_attribute_definition().is_none());

    let typed = legacy
        .with_source_attribute_definition(handoff.clone())
        .expect("Task 261 installation");
    assert_eq!(
        typed
            .debug_text()
            .matches("source-attribute-definition-debug-v1")
            .count(),
        1
    );
    assert_eq!(typed.initial_obligations(), &baseline);
    assert!(typed.types().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.diagnostics().is_empty());

    let resolved = assemble_empty(&typed).expect("Task 261 final assembly");
    assert_eq!(resolved.source_attribute_definition(), Some(&handoff));
    assert_eq!(
        resolved
            .debug_text()
            .matches("source-attribute-definition-debug-v1")
            .count(),
        1
    );
    assert_eq!(resolved.clone(), resolved);
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
    assert!(resolved.diagnostics().is_empty());

    let equivalent = fixture
        .typed_ast(baseline)
        .with_source_attribute_definition(fixture.handoff())
        .unwrap();
    assert_eq!(resolved, assemble_empty(&equivalent).unwrap());
}

fn assert_build_error(
    fixture: &Fixture,
    input: SourceAttributeDefinitionHandoffInput,
    expected: SourceAttributeDefinitionError,
) {
    assert_eq!(
        SourceAttributeDefinitionProducer::build(
            input,
            &fixture.env,
            &fixture.source_context,
            &fixture.source_type,
            &fixture.source_term,
            &fixture.source_atomic_formula,
            &fixture.arena,
        ),
        Err(expected)
    );
}

fn validate_exact_handoff(
    fixture: &Fixture,
    handoff: &SourceAttributeDefinitionHandoff,
) -> Result<(), SourceAttributeDefinitionError> {
    handoff.validate_installation(
        fixture.source,
        &fixture.module,
        &fixture.source_context,
        &fixture.source_type,
        &fixture.source_term,
        &fixture.source_atomic_formula,
        &fixture.arena,
    )
}

fn assemble_empty(typed_ast: &TypedAst) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
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
        statement_semantics: None,
        statement_proofs: None,
    })
}

fn fixture() -> Fixture {
    let source = source_id();
    let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task261"));
    let arena = task261_arena(source);
    let source_context = task248_context(source, &module);
    let (env, symbol, definition, contribution) =
        resolver_env_with_path(source, module.clone(), vec![4, 0, 7, 0], Some(0));
    let source_type = SourceTypeProducer::build(
        task249_input(source, module.clone()),
        source_context.binding_env(),
        &env,
        &arena,
    )
    .expect("Task 249 profile");
    let source_term = SourcePrimaryTermProducer::build(
        task252_input(source, module.clone()),
        source_context.binding_env(),
        &arena,
    )
    .expect("Task 252 profile");
    let source_atomic_formula = SourceAtomicFormulaProducer::build(
        task256_input(source, module.clone()),
        source_context.binding_env(),
        &env,
        &source_term,
        None,
        None,
        None,
        &arena,
    )
    .expect("Task 256 profile");
    let input = task261_input(source, module.clone(), symbol, definition, contribution);
    Fixture {
        source,
        module,
        env,
        source_context,
        source_type,
        source_term,
        source_atomic_formula,
        arena,
        input,
    }
}

fn task248_context(source: SourceId, module: &ModuleId) -> SourceBindingContextHandoff {
    let shell = definition_block_shell(source, module);
    let local_scope = LocalTermScope::new(vec![0]);
    SourceBindingContextProducer::build(SourceBindingContextInput {
        source_id: source,
        module_id: module.clone(),
        module_site: TypedSiteRef::Node(MODULE_ROOT),
        items: vec![SourceItemInput {
            shell,
            shell_ordinal: 0,
            role: SourceItemRole::DefinitionBlock,
            module_id: module.clone(),
            source_range: range(source, 0, 115),
            parent: None,
            visibility: SourceItemVisibility::Unspecified,
            site: TypedSiteRef::Node(DEFINITION_BLOCK),
            local_scope: Some(local_scope.clone()),
            recovery: SourceItemRecovery::Normal,
        }],
        bindings: [
            (PARAMETER_X, "x", 17, 18, 22, 25),
            (PARAMETER_Y, "y", 33, 34, 38, 41),
        ]
        .into_iter()
        .enumerate()
        .map(
            |(ordinal, (site, spelling, start, end, type_start, type_end))| {
                let declaration_range = range(source, start, end);
                SourceBindingSiteInput {
                    shell,
                    context_owner: SourceBindingContextOwner::Shell(shell),
                    source_ordinal: ordinal,
                    spelling: spelling.to_owned(),
                    declaration_range,
                    written_type_range: range(source, type_start, type_end),
                    site: TypedSiteRef::Node(site),
                    role: SourceBindingSiteRole::DefinitionParameter {
                        local: LocalTermBinding::new(
                            spelling,
                            local_scope.clone(),
                            declaration_range,
                            ordinal,
                        ),
                    },
                    recovery: BindingRecoveryState::Normal,
                }
            },
        )
        .collect(),
    })
    .expect("Task 248 Profile B")
    .into_complete()
    .expect("complete Task 248 Profile B")
    .into_handoff()
}

fn definition_block_shell(
    source: SourceId,
    module: &ModuleId,
) -> mizar_resolve::declarations::DeclarationShellId {
    let mut builder = SurfaceAstBuilder::new(source);
    let block = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source, 0, 115),
        Vec::new(),
    );
    let items = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source, 0, 115),
        vec![block],
    );
    let unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source, 0, 115),
        vec![items],
    );
    let root = builder.add_node(SurfaceNodeKind::Root, range(source, 0, 115), vec![unit]);
    let ast = builder.finish(Some(root), None);
    DeclarationShellCollector::new(&ast, module)
        .collect()
        .declarations()[0]
        .id()
}

fn resolver_env_with_path(
    source: SourceId,
    module: ModuleId,
    path: Vec<u32>,
    arity: Option<u32>,
) -> (SymbolEnv, SymbolId, DefinitionId, SourceContributionId) {
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 45, 110)),
        path,
    );
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new("task261_marked"),
        FullyQualifiedName::new("pkg::task261::task261_marked"),
    );
    let signature = SignatureShell::Opaque {
        schema: "parser-signature-v1".to_owned(),
        payload: "attribute:task261_marked".to_owned(),
    };
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 115)),
    );
    indexes.symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Attribute,
            NamespacePath::new("main"),
            "task261_marked",
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
        .with_notation_spelling("task261_marked")
        .with_signature(signature.clone()),
    );
    let shell = DefinitionShell::new(
        symbol.clone(),
        DefinitionKind::Attribute,
        origin,
        contribution,
    )
    .with_visibility(Visibility::Public)
    .with_notation_shape("task261_marked")
    .with_signature(signature);
    let definition = indexes.definitions.insert(match arity {
        Some(arity) => shell.with_arity(arity),
        None => shell,
    });
    indexes
        .contributions
        .add_symbol(contribution, symbol.clone());
    indexes
        .contributions
        .add_definition(contribution, definition);
    (
        SymbolEnv::new(module, indexes),
        symbol,
        definition,
        contribution,
    )
}

fn task261_arena(source: SourceId) -> TypedArena {
    let mut nodes = Vec::with_capacity(45);
    for index in 0..45 {
        let (kind, source_range, context) = match index {
            24 => ("source.type.head", range(source, 22, 25), 1),
            25 => ("source.type.expression", range(source, 22, 25), 1),
            27 => (
                "source.definition.attribute.parameter",
                range(source, 17, 18),
                1,
            ),
            28 => ("source.type.head", range(source, 38, 41), 1),
            29 => ("source.type.expression", range(source, 38, 41), 1),
            31 => (
                "source.definition.attribute.parameter",
                range(source, 33, 34),
                1,
            ),
            33 => ("source.term.variable-reference", range(source, 104, 105), 1),
            35 => ("source.term.variable-reference", range(source, 108, 109), 1),
            37 => ("source.formula.atomic.equality", range(source, 104, 109), 1),
            39 => (
                "source.definition.attribute.definiens",
                range(source, 104, 109),
                1,
            ),
            40 => ("source.definition.attribute", range(source, 45, 110), 1),
            41 => ("source.definition", range(source, 0, 115), 1),
            44 => ("source.module", range(source, 0, 115), 0),
            _ => ("source.unowned", range(source, 0, 1), 0),
        };
        nodes.push(
            TypedNode::new(kind, SourceAnchor::Range(source_range))
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal)
                .with_links(TypedNodeLinks {
                    context: Some(LocalTypeContextId::new(context)),
                    ..TypedNodeLinks::default()
                }),
        );
    }
    TypedArena::try_new(Some(MODULE_ROOT), nodes).expect("Task 261 arena")
}

fn task249_input(source: SourceId, module: ModuleId) -> SourceTypeHandoffInput {
    SourceTypeHandoffInput {
        source_id: source,
        module_id: module.clone(),
        applications: vec![
            SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            },
            SourceTypeApplicationInput {
                binding: BindingId::new(1),
                source_ordinal: 1,
                root: SourceTypeExpressionId::new(1),
            },
        ],
        expressions: vec![
            bare_set_type(
                source,
                module.clone(),
                TYPE_EXPRESSION_X,
                TYPE_HEAD_X,
                22,
                25,
            ),
            bare_set_type(source, module, TYPE_EXPRESSION_Y, TYPE_HEAD_Y, 38, 41),
        ],
        arguments: Vec::new(),
    }
}

fn bare_set_type(
    source: SourceId,
    module: ModuleId,
    site: TypedNodeId,
    head_site: TypedNodeId,
    start: usize,
    end: usize,
) -> SourceTypeExpressionInput {
    SourceTypeExpressionInput {
        source_id: source,
        module_id: module,
        site: TypedSiteRef::Node(site),
        source_range: range(source, start, end),
        spelling: "set".to_owned(),
        head_site: TypedSiteRef::Node(head_site),
        head_range: range(source, start, end),
        head_spelling: "set".to_owned(),
        form: SourceTypeApplicationForm::Bare,
        head: SourceTypeHead::BuiltinSet,
        recovery: NodeRecoveryState::Normal,
    }
}

fn task252_input(source: SourceId, module: ModuleId) -> SourcePrimaryTermHandoffInput {
    let specs = [
        (DEFINIENS_LEFT, 104, 105, "x", BindingId::new(0)),
        (DEFINIENS_RIGHT, 108, 109, "y", BindingId::new(1)),
    ];
    SourcePrimaryTermHandoffInput {
        source_id: source,
        module_id: module,
        terms: specs
            .iter()
            .enumerate()
            .map(
                |(source_ordinal, (site, start, end, spelling, _))| SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(*site),
                    source_range: range(source, *start, *end),
                    source_ordinal,
                    context: BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: (*spelling).to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect(),
        references: specs
            .iter()
            .enumerate()
            .map(
                |(index, (_, _, _, _, binding))| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(index),
                    binding: *binding,
                    role: SourcePrimaryTermReferenceRole::Variable,
                },
            )
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

fn task256_input(source: SourceId, module: ModuleId) -> SourceAtomicFormulaHandoffInput {
    let edges = (0..2)
        .map(|index| SourceAtomicEdgeInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: index,
            role: if index == 0 {
                SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                SourceAtomicEdgeRole::BuiltinRightOperand
            },
            target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(index)),
        })
        .collect::<Vec<_>>();
    let requests = (0..2)
        .map(|index| SourceAtomicRequestInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: index,
            kind: SourceAtomicRequestKind::OperandExpectedType,
            edge: Some(SourceAtomicEdgeId::new(index)),
            candidate: None,
            type_site: None,
            attribute: None,
        })
        .collect();
    SourceAtomicFormulaHandoffInput {
        source_id: source,
        module_id: module,
        formulas: vec![SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(DEFINIENS_FORMULA),
            source_range: range(source, 104, 109),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = y".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        }],
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

fn task261_input(
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
) -> SourceAttributeDefinitionHandoffInput {
    SourceAttributeDefinitionHandoffInput {
        source_id: source,
        module_id: module,
        definitions: vec![SourceAttributeDefinitionInput {
            symbol,
            definition,
            contribution,
            site: TypedSiteRef::Node(ATTRIBUTE_OWNER),
            source_range: range(source, 45, 110),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAttributeDefinitionRecovery::Normal,
            spelling: "attr Task261AttributeDefinition: x is task261_marked means x = y;"
                .to_owned(),
            subject: SourceAttributeSubjectId::new(0),
            definiens: SourceAttributeDefiniensId::new(0),
        }],
        parameters: vec![
            SourceAttributeParameterInput {
                owner: SourceAttributeDefinitionId::new(0),
                ordinal: 0,
                binding: BindingId::new(0),
                written_type: SourceTypeApplicationId::new(0),
                site: TypedSiteRef::Node(PARAMETER_X),
                source_range: range(source, 13, 26),
                declaration_range: range(source, 17, 18),
                context: BindingContextId::new(1),
                recovery: SourceAttributeDefinitionRecovery::Normal,
                spelling: "let x be set;".to_owned(),
            },
            SourceAttributeParameterInput {
                owner: SourceAttributeDefinitionId::new(0),
                ordinal: 1,
                binding: BindingId::new(1),
                written_type: SourceTypeApplicationId::new(1),
                site: TypedSiteRef::Node(PARAMETER_Y),
                source_range: range(source, 29, 42),
                declaration_range: range(source, 33, 34),
                context: BindingContextId::new(1),
                recovery: SourceAttributeDefinitionRecovery::Normal,
                spelling: "let y be set;".to_owned(),
            },
        ],
        subjects: vec![SourceAttributeSubjectInput {
            owner: SourceAttributeDefinitionId::new(0),
            binding: BindingId::new(0),
            site: TypedSiteRef::Node(ATTRIBUTE_OWNER),
            source_range: range(source, 78, 79),
            context: BindingContextId::new(1),
            recovery: SourceAttributeDefinitionRecovery::Normal,
            spelling: "x".to_owned(),
        }],
        definientia: vec![SourceAttributeDefiniensInput {
            owner: SourceAttributeDefinitionId::new(0),
            ordinal: 0,
            formula: SourceAtomicFormulaId::new(0),
            site: TypedSiteRef::Node(DEFINIENS_OWNER),
            source_range: range(source, 104, 109),
            context: BindingContextId::new(1),
            recovery: SourceAttributeDefinitionRecovery::Normal,
            spelling: "x = y".to_owned(),
        }],
    }
}

fn expected_debug(fixture: &Fixture) -> String {
    format!(
        "source-attribute-definition-debug-v1\nmodule: task261\nsource-context-fingerprint: {:?}\nsource-type-fingerprint: {:?}\nsource-term-fingerprint: {:?}\nsource-atomic-formula-fingerprint: {:?}\ndefinition#0 symbol={:?} definition=0 contribution=0 ordinal=0 range=45..110 site=node#40 context=1 recovery=normal origin_range=45..110 origin_path=[4, 0, 7, 0] spelling={:?} subject=0 definiens=0\nparameter#0 owner=0 ordinal=0 binding=0 written_type=0 range=13..26 declaration_range=17..18 site=node#27 context=1 recovery=normal spelling={:?}\nparameter#1 owner=0 ordinal=1 binding=1 written_type=1 range=29..42 declaration_range=33..34 site=node#31 context=1 recovery=normal spelling={:?}\nsubject#0 owner=0 binding=0 range=78..79 site=node#40 context=1 recovery=normal spelling={:?}\ndefiniens#0 owner=0 ordinal=0 formula=0 range=104..109 site=node#39 context=1 recovery=normal spelling={:?}\n",
        fixture.source_context.debug_text(),
        fixture.source_type.debug_text(),
        fixture.source_term.debug_text(),
        fixture.source_atomic_formula.debug_text(),
        "pkg::task261::task261_marked",
        "attr Task261AttributeDefinition: x is task261_marked means x = y;",
        "let x be set;",
        "let y be set;",
        "x",
        "x = y",
    )
}

fn non_empty_baseline(source: SourceId) -> InitialObligationTable {
    let mut table = InitialObligationTable::new();
    table.insert(InitialObligationDraft {
        kind: InitialObligationKind::Sethood,
        owner: TypedSiteRef::Node(DEFINIENS_LEFT),
        source_range: range(source, 104, 105),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new("unrelated:sethood"),
        provenance: InitialObligationProvenance::new("task261:baseline"),
        status: InitialObligationStatus::Pending,
    });
    table
}

fn source_id() -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "63".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .unwrap()
}

#[test]
fn source_attribute_definition_accepts_coherent_nonuniform_layout_ranges() {
    use crate::binding_env::{BindingLookupResult, BindingLookupSite};
    use SourceAttributeDefinitionError::{
        DependencyMismatch, InvalidDefiniens, InvalidDefinition, InvalidParameter,
        InvalidResolverDefinition, InvalidSubject,
    };

    let foreign_snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "64".repeat(Hash::BYTE_LEN),
    ))
    .unwrap();
    let foreign_ids = InMemorySessionIdAllocator::new();
    foreign_ids.next_source_id(foreign_snapshot).unwrap();
    let foreign_source = foreign_ids.next_source_id(foreign_snapshot).unwrap();
    assert_ne!(foreign_source, source_id());
    for (case, expected_error) in [
        ("normal", None),
        ("scalar-slack", None),
        ("block-head-capacity", Some(InvalidParameter { index: 0 })),
        (
            "parameter-prefix-capacity",
            Some(InvalidParameter { index: 0 }),
        ),
        ("parameter-be-capacity", Some(InvalidParameter { index: 0 })),
        (
            "parameter-tail-capacity",
            Some(InvalidParameter { index: 0 }),
        ),
        (
            "definition-prefix-capacity",
            Some(InvalidSubject { index: 0 }),
        ),
        ("subject-body-capacity", Some(InvalidSubject { index: 0 })),
        (
            "definition-tail-capacity",
            Some(InvalidDefiniens { index: 0 }),
        ),
        ("block-tail-capacity", Some(InvalidDefinition { index: 0 })),
        ("equality-capacity", Some(DependencyMismatch)),
        ("declaration-width", Some(InvalidParameter { index: 0 })),
        ("declaration-y-width", Some(InvalidParameter { index: 1 })),
        ("type-width", Some(InvalidParameter { index: 0 })),
        ("left-width", Some(DependencyMismatch)),
        ("right-width", Some(DependencyMismatch)),
        ("formula-start-join", Some(DependencyMismatch)),
        ("formula-end-join", Some(DependencyMismatch)),
        ("parameter-overlap", Some(InvalidParameter { index: 1 })),
        (
            "definition-join",
            Some(InvalidResolverDefinition { index: 0 }),
        ),
        (
            "contribution-join",
            Some(InvalidResolverDefinition { index: 0 }),
        ),
        ("declaration-join", Some(InvalidParameter { index: 0 })),
        ("written-type-id", Some(InvalidParameter { index: 0 })),
        ("definiens-join", Some(DependencyMismatch)),
        ("subject-width", Some(InvalidSubject { index: 0 })),
        ("subject-binding", Some(InvalidSubject { index: 0 })),
        ("subject-context", Some(InvalidSubject { index: 0 })),
        ("subject-site", Some(InvalidSubject { index: 0 })),
        ("subject-foreign", Some(InvalidSubject { index: 0 })),
        ("subject-empty", Some(InvalidSubject { index: 0 })),
        ("subject-reversed", Some(InvalidSubject { index: 0 })),
        ("subject-overflow", Some(InvalidSubject { index: 0 })),
        ("subject-outside", Some(InvalidSubject { index: 0 })),
        ("parameter-context", Some(InvalidParameter { index: 0 })),
        ("parameter-foreign", Some(InvalidParameter { index: 0 })),
        ("parameter-empty", Some(InvalidParameter { index: 0 })),
        ("parameter-reversed", Some(InvalidParameter { index: 0 })),
        ("parameter-overflow", Some(InvalidParameter { index: 0 })),
    ] {
        let mut fixture = fixture();
        let source = fixture.source;
        let module = fixture.module.clone();
        let relocate = |old: SourceRange| {
            let (start, end) = match (old.start, old.end) {
                (0, 115) => (7, 192),
                (13, 26) => (24, 41),
                (17, 18) => (29, 30),
                (22, 25) => (37, 40),
                (29, 42) => (50, 69),
                (33, 34) => (55, 56),
                (38, 41) => (65, 68),
                (45, 110) => (86, 181),
                (78, 79) => (124, 125),
                (104, 105) => (166, 167),
                (108, 109) => (178, 179),
                (104, 109) => (166, 179),
                _ => return old,
            };
            let (start, end) = match (case, (old.start, old.end)) {
                ("scalar-slack", (13, 26)) => (23, 41),
                ("scalar-slack", (78, 79)) => (125, 126),
                ("block-head-capacity", (0, 115)) => (14, 192),
                ("parameter-prefix-capacity", (13, 26)) => (26, 41),
                ("parameter-be-capacity", (22, 25)) => (33, 36),
                ("parameter-tail-capacity", (13, 26)) => (24, 40),
                ("definition-prefix-capacity", (78, 79)) => (117, 118),
                ("subject-body-capacity", (78, 79)) => (141, 142),
                ("definition-tail-capacity", (45, 110)) => (86, 179),
                ("block-tail-capacity", (0, 115)) => (7, 184),
                ("equality-capacity", (108, 109)) => (167, 168),
                ("equality-capacity", (104, 109)) => (166, 168),
                ("declaration-width", (17, 18)) => (29, 31),
                ("declaration-y-width", (33, 34)) => (55, 57),
                ("type-width", (22, 25)) => (37, 39),
                ("left-width", (104, 105)) => (166, 168),
                ("right-width", (108, 109)) => (178, 180),
                ("right-width", (104, 109)) => (166, 180),
                ("formula-start-join", (104, 109)) => (165, 179),
                ("formula-end-join", (104, 109)) => (166, 180),
                ("parameter-overlap", (29, 42)) => (40, 69),
                _ => (start, end),
            };
            range(source, start, end)
        };
        let arena = TypedArena::try_new(
            fixture.arena.root(),
            fixture
                .arena
                .iter()
                .map(|(_, node)| {
                    let mut node = node.clone();
                    if let SourceAnchor::Range(old) = &node.anchor {
                        node.anchor = SourceAnchor::Range(relocate(*old));
                    }
                    node
                })
                .collect(),
        )
        .expect("relocated public role anchors");
        let old_definition = &fixture.input.definitions[0];
        let symbol = fixture.env.symbols().get(&old_definition.symbol).unwrap();
        let definition = fixture
            .env
            .definitions()
            .get(old_definition.definition)
            .unwrap();
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(relocate(old_definition.source_range)),
            symbol.origin().structural_path().to_vec(),
        );
        let item = fixture
            .source_context
            .items()
            .get(SourceItemId::new(0))
            .unwrap();
        let mut indexes = SymbolEnvIndexes::default();
        let mut contribution_range = relocate(item.source_range);
        if case == "contribution-join" {
            contribution_range.end += 1;
        }
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(contribution_range),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                symbol.symbol().clone(),
                symbol.kind(),
                symbol.namespace().clone(),
                symbol.primary_spelling(),
                origin.clone(),
                contribution,
            )
            .with_visibility(symbol.visibility())
            .with_export_status(symbol.export_status())
            .with_notation_spelling(symbol.notation_spelling().unwrap())
            .with_signature(symbol.signature().unwrap().clone()),
        );
        let definition_id = indexes.definitions.insert(
            DefinitionShell::new(
                symbol.symbol().clone(),
                definition.kind(),
                origin,
                contribution,
            )
            .with_visibility(definition.visibility())
            .with_arity(definition.arity().unwrap())
            .with_notation_shape(definition.notation_shape().unwrap())
            .with_signature(definition.signature().unwrap().clone()),
        );
        indexes
            .contributions
            .add_symbol(contribution, symbol.symbol().clone());
        indexes
            .contributions
            .add_definition(contribution, definition_id);
        let env = SymbolEnv::new(module.clone(), indexes);
        let source_context = SourceBindingContextProducer::build(SourceBindingContextInput {
            source_id: source,
            module_id: module.clone(),
            module_site: TypedSiteRef::Node(MODULE_ROOT),
            items: vec![SourceItemInput {
                shell: item.shell,
                shell_ordinal: item.shell_ordinal,
                role: item.role,
                module_id: module.clone(),
                source_range: relocate(item.source_range),
                parent: item.parent,
                visibility: item.visibility,
                site: item.site.clone(),
                local_scope: item.local_scope.clone(),
                recovery: item.recovery,
            }],
            bindings: fixture
                .source_context
                .declarations()
                .iter()
                .map(|(_, declaration)| {
                    let declaration_range = relocate(declaration.declaration_range);
                    SourceBindingSiteInput {
                        shell: item.shell,
                        context_owner: SourceBindingContextOwner::Shell(item.shell),
                        source_ordinal: declaration.source_ordinal,
                        spelling: declaration.spelling.clone(),
                        declaration_range,
                        written_type_range: relocate(declaration.written_type_range),
                        site: declaration.site.clone(),
                        role: SourceBindingSiteRole::DefinitionParameter {
                            local: LocalTermBinding::new(
                                declaration.spelling.clone(),
                                item.local_scope.clone().expect("definition local scope"),
                                declaration_range,
                                declaration.source_ordinal,
                            ),
                        },
                        recovery: BindingRecoveryState::Normal,
                    }
                })
                .collect(),
        })
        .unwrap_or_else(|error| panic!("{case}: SourceContext rejected {error:?}"))
        .into_complete()
        .expect("relocated complete SourceContext")
        .into_handoff();
        let mut type_input = task249_input(source, module.clone());
        for expression in &mut type_input.expressions {
            expression.source_range = relocate(expression.source_range);
            expression.head_range = relocate(expression.head_range);
        }
        let source_type =
            SourceTypeProducer::build(type_input, source_context.binding_env(), &env, &arena)
                .unwrap_or_else(|error| panic!("{case}: SourceType rejected {error:?}"));
        let mut term_input = task252_input(source, module.clone());
        for term in &mut term_input.terms {
            term.source_range = relocate(term.source_range);
        }
        let source_term =
            SourcePrimaryTermProducer::build(term_input, source_context.binding_env(), &arena)
                .unwrap_or_else(|error| panic!("{case}: Primary rejected {error:?}"));
        assert_eq!(
            (source_term.terms().len(), source_term.references().len()),
            (2, 2)
        );
        for (id, reference) in source_term.references().iter() {
            let term = source_term.terms().get(reference.term()).unwrap();
            let original = fixture.source_term.references().get(id).unwrap();
            assert_eq!(reference.use_ordinal(), original.use_ordinal());
            assert_eq!(reference.lexical_scope(), original.lexical_scope());
            assert_eq!(reference.binding(), BindingId::new(id.index()));
            assert_eq!(
                source_context.binding_env().lookup(&BindingLookupSite::new(
                    term.spelling(),
                    term.context(),
                    reference.lexical_scope().cloned(),
                    reference.use_ordinal(),
                )),
                Ok(BindingLookupResult::Local(BindingId::new(id.index()))),
            );
        }
        let mut atomic_input = task256_input(source, module.clone());
        for formula in &mut atomic_input.formulas {
            formula.source_range = relocate(formula.source_range);
        }
        let source_atomic_formula = SourceAtomicFormulaProducer::build(
            atomic_input,
            source_context.binding_env(),
            &env,
            &source_term,
            None,
            None,
            None,
            &arena,
        )
        .unwrap_or_else(|error| panic!("{case}: Atomic rejected {error:?}"));
        assert_eq!(
            (
                source_atomic_formula.formulas().len(),
                source_atomic_formula.edges().len(),
                source_atomic_formula.requests().len()
            ),
            (1, 2, 2)
        );
        let mut input = task261_input(
            source,
            module,
            symbol.symbol().clone(),
            definition_id,
            contribution,
        );
        for row in &mut input.definitions {
            row.source_range = relocate(row.source_range);
        }
        for row in &mut input.parameters {
            row.source_range = relocate(row.source_range);
            row.declaration_range = relocate(row.declaration_range);
        }
        input.subjects[0].source_range = relocate(input.subjects[0].source_range);
        input.definientia[0].source_range = relocate(input.definientia[0].source_range);
        match case {
            "definition-join" => input.definitions[0].source_range.end -= 1,
            "declaration-join" => {
                input.parameters[0].declaration_range.start += 1;
                input.parameters[0].declaration_range.end += 1;
            }
            "written-type-id" => input.parameters[0].written_type = SourceTypeApplicationId::new(1),
            "definiens-join" => input.definientia[0].source_range.end -= 1,
            "subject-width" => input.subjects[0].source_range.end += 1,
            "subject-binding" => input.subjects[0].binding = BindingId::new(1),
            "subject-context" => input.subjects[0].context = BindingContextId::new(0),
            "subject-site" => input.subjects[0].site = TypedSiteRef::Node(PARAMETER_X),
            "subject-foreign" => input.subjects[0].source_range.source_id = foreign_source,
            "subject-empty" => {
                input.subjects[0].source_range.end = input.subjects[0].source_range.start
            }
            "subject-reversed" => input.subjects[0].source_range = range(source, 125, 124),
            "subject-overflow" => input.subjects[0].source_range = range(source, usize::MAX, 0),
            "subject-outside" => input.subjects[0].source_range = range(source, 181, 182),
            "parameter-context" => input.parameters[0].context = BindingContextId::new(0),
            "parameter-foreign" => input.parameters[0].source_range.source_id = foreign_source,
            "parameter-empty" => input.parameters[0].source_range = range(source, 41, 41),
            "parameter-reversed" => input.parameters[0].source_range = range(source, 41, 24),
            "parameter-overflow" => input.parameters[0].source_range = range(source, usize::MAX, 0),
            _ => {}
        }
        fixture.env = env;
        fixture.source_context = source_context;
        fixture.source_type = source_type;
        fixture.source_term = source_term;
        fixture.source_atomic_formula = source_atomic_formula;
        fixture.arena = arena;
        fixture.input = input;

        if let Some(expected_error) = expected_error {
            assert_eq!(fixture.build(), Err(expected_error), "{case}");
            continue;
        }
        let handoff = fixture
            .build()
            .expect("coherent nonuniform public Attribute ranges");
        assert_eq!(
            (
                handoff.definitions().len(),
                handoff.parameters().len(),
                handoff.subjects().len(),
                handoff.definientia().len()
            ),
            (1, 2, 1, 1)
        );
        assert_eq!(
            handoff.source_context_fingerprint(),
            fixture.source_context.debug_text()
        );
        assert_eq!(
            handoff.source_type_fingerprint(),
            fixture.source_type.debug_text()
        );
        assert_eq!(
            handoff.source_term_fingerprint(),
            fixture.source_term.debug_text()
        );
        assert_eq!(
            handoff.source_atomic_formula_fingerprint(),
            fixture.source_atomic_formula.debug_text()
        );
        let baseline = non_empty_baseline(source);
        let typed = fixture
            .typed_ast(baseline.clone())
            .with_source_attribute_definition(handoff.clone())
            .expect("relocated Attribute installation");
        assert_eq!(typed.initial_obligations(), &baseline);
        let resolved = assemble_empty(&typed).expect("relocated Attribute final assembly");
        assert_eq!(resolved.source_attribute_definition(), Some(&handoff));
        assert_eq!(resolved.clone(), resolved);
        assert_eq!(assemble_empty(&typed.clone()).unwrap(), resolved);
    }
}
