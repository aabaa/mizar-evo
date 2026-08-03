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
    source_attribute_definition::tests::actual_attribute_definition_typed_ast_for_task262,
    source_context::{
        SourceBindingContextInput, SourceBindingContextOwner, SourceBindingContextProducer,
        SourceBindingSiteInput, SourceItemInput,
    },
    source_functor_definition::tests::actual_definition_family_typed_asts_for_task261,
    source_type::{
        SourceTypeApplicationInput, SourceTypeExpressionInput, SourceTypeHandoffInput,
        SourceTypeModeRhsExtensionInput, SourceTypeModeRhsInput, SourceTypeModeRhsProducer,
        SourceTypeProducer,
    },
    typed_ast::{
        CoercionTable, InitialObligation, TypeDiagnosticTable, TypeFactId, TypeFactTable,
        TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode, TypedNodeLinks, TypingState,
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

const TYPE_HEAD_X: TypedNodeId = TypedNodeId::new(34);
const TYPE_EXPRESSION_X: TypedNodeId = TypedNodeId::new(35);
const PARAMETER_X: TypedNodeId = TypedNodeId::new(37);
const TYPE_HEAD_Y: TypedNodeId = TypedNodeId::new(38);
const TYPE_EXPRESSION_Y: TypedNodeId = TypedNodeId::new(39);
const PARAMETER_Y: TypedNodeId = TypedNodeId::new(41);
const APPLICATION: TypedNodeId = TypedNodeId::new(42);
const RHS_HEAD: TypedNodeId = TypedNodeId::new(43);
const RHS_EXPRESSION: TypedNodeId = TypedNodeId::new(44);
const PROPERTY: TypedNodeId = TypedNodeId::new(48);
const MODE_DEFINITION: TypedNodeId = TypedNodeId::new(49);
const DEFINITION_BLOCK: TypedNodeId = TypedNodeId::new(50);
const MODULE_ROOT: TypedNodeId = TypedNodeId::new(53);

#[derive(Clone)]
struct Fixture {
    source: SourceId,
    module: ModuleId,
    env: SymbolEnv,
    source_context: SourceBindingContextHandoff,
    source_type: SourceTypeApplicationHandoff,
    arena: TypedArena,
    input: SourceModeDefinitionHandoffInput,
}

impl Fixture {
    fn build(
        &self,
        baseline: &InitialObligationTable,
    ) -> Result<SourceModeDefinitionProjection, SourceModeDefinitionError> {
        SourceModeDefinitionProducer::build(
            self.input.clone(),
            &self.env,
            &self.source_context,
            &self.source_type,
            baseline,
            &self.arena,
        )
    }

    fn projection(&self, baseline: &InitialObligationTable) -> SourceModeDefinitionProjection {
        self.build(baseline).expect("exact Task 262 projection")
    }

    fn typed(&self, baseline: InitialObligationTable) -> TypedAst {
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
            initial_obligations: baseline,
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task 262 typed baseline")
    }
}

pub(crate) fn actual_mode_definition_for_task263() -> (TypedAst, SourceModeDefinitionProjection) {
    let fixture = fixture();
    let projection = fixture.projection(&InitialObligationTable::new());
    let typed = fixture
        .typed(InitialObligationTable::new())
        .with_source_mode_definition(projection.clone())
        .expect("actual Task 262 mode installation for Task 263 isolation");
    (typed, projection)
}

#[test]
fn task_262_mode_definition_exact_payload_and_obligations_are_deterministic() {
    let fixture = fixture();
    let baseline = model_domain_baseline(fixture.source);
    let projection = fixture.projection(&baseline);
    let handoff = projection.handoff();
    assert_eq!(projection.base_initial_obligations(), &baseline);
    assert_eq!(handoff.source_id(), fixture.source);
    assert_eq!(handoff.module_id(), &fixture.module);
    assert_eq!(
        handoff.source_context_fingerprint(),
        fixture.source_context.debug_text()
    );
    assert_eq!(
        handoff.source_type_fingerprint(),
        fixture.source_type.debug_text()
    );
    assert_eq!(handoff.base_initial_obligation_count(), 1);
    assert_eq!(handoff.definitions().len(), 1);
    assert_eq!(handoff.parameters().len(), 2);
    assert_eq!(handoff.applications().len(), 1);
    assert_eq!(handoff.expansions().len(), 1);
    assert_eq!(handoff.inhabitation_requests().len(), 1);
    assert_eq!(handoff.properties().len(), 1);
    assert!(!handoff.definitions().is_empty());
    assert!(!handoff.parameters().is_empty());
    assert!(!handoff.applications().is_empty());
    assert!(!handoff.expansions().is_empty());
    assert!(!handoff.inhabitation_requests().is_empty());
    assert!(!handoff.properties().is_empty());

    let definition = handoff
        .definitions()
        .get(SourceModeDefinitionId::new(0))
        .unwrap();
    assert_eq!(definition.id(), SourceModeDefinitionId::new(0));
    assert_eq!(definition.symbol(), &fixture.input.definitions[0].symbol);
    assert_eq!(definition.definition().index(), 0);
    assert_eq!(definition.contribution().index(), 0);
    assert_eq!(definition.site(), &TypedSiteRef::Node(MODE_DEFINITION));
    assert_eq!(definition.source_range(), range(fixture.source, 45, 135));
    assert_eq!(definition.source_ordinal(), 0);
    assert_eq!(definition.context(), BindingContextId::new(1));
    assert_eq!(definition.recovery(), SourceModeDefinitionRecovery::Normal);
    assert_eq!(definition.application(), SourceModeApplicationId::new(0));
    assert_eq!(definition.expansion(), SourceModeExpansionId::new(0));
    assert_eq!(
        definition.inhabitation_request(),
        SourceModeInhabitationRequestId::new(0)
    );
    assert_eq!(definition.property(), Some(SourceModePropertyId::new(0)));
    assert_eq!(definition.origin().structural_path(), &[4, 0, 10, 0]);
    assert!(definition.spelling().contains("Task262ModeDefinition"));

    for index in 0..2 {
        let row = handoff
            .parameters()
            .get(SourceModeParameterId::new(index))
            .unwrap();
        assert_eq!(row.id(), SourceModeParameterId::new(index));
        assert_eq!(row.owner(), SourceModeDefinitionId::new(0));
        assert_eq!(row.ordinal(), index);
        assert_eq!(row.binding(), BindingId::new(index));
        assert_eq!(row.written_type(), SourceTypeApplicationId::new(index));
        assert_eq!(row.context(), BindingContextId::new(1));
        assert_eq!(row.recovery(), SourceModeDefinitionRecovery::Normal);
    }
    let application = handoff
        .applications()
        .get(SourceModeApplicationId::new(0))
        .unwrap();
    assert_eq!(application.id(), SourceModeApplicationId::new(0));
    assert_eq!(application.owner(), SourceModeDefinitionId::new(0));
    assert_eq!(application.ordinal(), 0);
    assert_eq!(
        application.parameters(),
        &[SourceModeParameterId::new(0), SourceModeParameterId::new(1)]
    );
    assert_eq!(application.site(), &TypedSiteRef::Node(APPLICATION));
    assert_eq!(application.source_range(), range(fixture.source, 73, 91));
    assert_eq!(application.context(), BindingContextId::new(1));
    assert_eq!(application.recovery(), SourceModeDefinitionRecovery::Normal);
    assert_eq!(application.spelling(), "Task262Mode [ x , y ]");
    let expansion = handoff
        .expansions()
        .get(SourceModeExpansionId::new(0))
        .unwrap();
    assert_eq!(expansion.id(), SourceModeExpansionId::new(0));
    assert_eq!(expansion.owner(), SourceModeDefinitionId::new(0));
    assert_eq!(expansion.ordinal(), 0);
    assert_eq!(expansion.rhs(), SourceTypeModeRhsId::new(0));
    assert_eq!(expansion.site(), &TypedSiteRef::Node(RHS_EXPRESSION));
    assert_eq!(expansion.source_range(), range(fixture.source, 95, 98));
    assert_eq!(expansion.context(), BindingContextId::new(1));
    assert_eq!(expansion.recovery(), SourceModeDefinitionRecovery::Normal);
    assert_eq!(expansion.spelling(), "set");
    let request = handoff
        .inhabitation_requests()
        .get(SourceModeInhabitationRequestId::new(0))
        .unwrap();
    assert_eq!(request.id(), SourceModeInhabitationRequestId::new(0));
    assert_eq!(request.owner(), SourceModeDefinitionId::new(0));
    assert_eq!(request.ordinal(), 0);
    assert_eq!(request.expansion(), SourceModeExpansionId::new(0));
    assert_eq!(request.kind(), SourceModeInhabitationRequestKind::Rhs);
    assert_eq!(request.site(), expansion.site());
    assert_eq!(request.source_range(), expansion.source_range());
    assert_eq!(request.context(), expansion.context());
    assert_eq!(request.recovery(), SourceModeDefinitionRecovery::Normal);
    assert_eq!(request.spelling(), "set");
    let property = handoff
        .properties()
        .get(SourceModePropertyId::new(0))
        .unwrap();
    assert_eq!(property.id(), SourceModePropertyId::new(0));
    assert_eq!(property.owner(), SourceModeDefinitionId::new(0));
    assert_eq!(property.ordinal(), 0);
    assert_eq!(property.kind(), SourceModePropertyKind::Sethood);
    assert_eq!(property.site(), &TypedSiteRef::Node(PROPERTY));
    assert_eq!(property.source_range(), range(fixture.source, 102, 135));
    assert_eq!(
        property.justification(),
        &SourceAnchor::Range(range(fixture.source, 113, 134))
    );
    assert_eq!(property.recovery(), SourceModeDefinitionRecovery::Normal);
    assert_eq!(property.spelling(), "sethood by computation(steps: 1);");
    assert_eq!(property.obligation(), InitialObligationId::new(1));

    assert_eq!(projection.initial_obligations().len(), 2);
    assert_eq!(
        projection
            .initial_obligations()
            .get(InitialObligationId::new(0)),
        baseline.get(InitialObligationId::new(0))
    );
    assert_task262_obligation(
        projection
            .initial_obligations()
            .get(InitialObligationId::new(1))
            .unwrap(),
        fixture.source,
        1,
    );
    assert_eq!(handoff.debug_text(), handoff.clone().debug_text());
    assert!(
        handoff
            .debug_text()
            .starts_with("source-mode-definition-debug-v1\n")
    );
    assert!(handoff.debug_text().ends_with('\n'));
}

#[test]
fn task_262_mode_definition_row_field_corruption_fails_closed() {
    let fixture = fixture();
    let baseline = InitialObligationTable::new();
    let mutations: [fn(&mut SourceModeDefinitionHandoffInput); 11] = [
        |i| i.definitions[0].source_ordinal = 1,
        |i| i.definitions[0].property = None,
        |i| i.definitions[0].application = SourceModeApplicationId::new(1),
        |i| i.definitions[0].expansion = SourceModeExpansionId::new(1),
        |i| i.definitions[0].inhabitation_request = SourceModeInhabitationRequestId::new(1),
        |i| i.parameters[0].owner = SourceModeDefinitionId::new(1),
        |i| i.parameters[0].pattern_range.end -= 1,
        |i| i.applications[0].parameters.reverse(),
        |i| i.expansions[0].rhs = SourceTypeModeRhsId::new(1),
        |i| i.inhabitation_requests[0].expansion = SourceModeExpansionId::new(1),
        |i| {
            i.properties[0].justification = SourceAnchor::Range(SourceRange {
                source_id: i.source_id,
                start: 114,
                end: 134,
            })
        },
    ];
    for mutate in mutations {
        let mut input = fixture.input.clone();
        mutate(&mut input);
        assert!(
            SourceModeDefinitionProducer::build(
                input,
                &fixture.env,
                &fixture.source_context,
                &fixture.source_type,
                &baseline,
                &fixture.arena
            )
            .is_err()
        );
    }
    for remove in 0..6 {
        let mut input = fixture.input.clone();
        match remove {
            0 => input.definitions.clear(),
            1 => {
                input.parameters.pop();
            }
            2 => input.applications.clear(),
            3 => input.expansions.clear(),
            4 => input.inhabitation_requests.clear(),
            5 => input.properties.clear(),
            _ => unreachable!(),
        }
        assert_eq!(
            fixture_build(&fixture, input, &baseline),
            Err(SourceModeDefinitionError::UnsupportedTaskShape)
        );
    }
    for duplicate in 0..6 {
        let mut input = fixture.input.clone();
        match duplicate {
            0 => input.definitions.push(input.definitions[0].clone()),
            1 => input.parameters.push(input.parameters[0].clone()),
            2 => input.applications.push(input.applications[0].clone()),
            3 => input.expansions.push(input.expansions[0].clone()),
            4 => input
                .inhabitation_requests
                .push(input.inhabitation_requests[0].clone()),
            5 => input.properties.push(input.properties[0].clone()),
            _ => unreachable!(),
        }
        assert_eq!(
            fixture_build(&fixture, input, &baseline),
            Err(SourceModeDefinitionError::UnsupportedTaskShape)
        );
    }
    let mut reordered = fixture.input.clone();
    reordered.parameters.swap(0, 1);
    assert!(matches!(
        fixture_build(&fixture, reordered, &baseline),
        Err(SourceModeDefinitionError::InvalidParameter { .. })
    ));
    let degraded: [fn(&mut SourceModeDefinitionHandoffInput); 6] = [
        |i| i.definitions[0].recovery = SourceModeDefinitionRecovery::Degraded,
        |i| i.parameters[0].recovery = SourceModeDefinitionRecovery::Degraded,
        |i| i.applications[0].recovery = SourceModeDefinitionRecovery::Degraded,
        |i| i.expansions[0].recovery = SourceModeDefinitionRecovery::Degraded,
        |i| i.inhabitation_requests[0].recovery = SourceModeDefinitionRecovery::Degraded,
        |i| i.properties[0].recovery = SourceModeDefinitionRecovery::Degraded,
    ];
    for mutate in degraded {
        let mut input = fixture.input.clone();
        mutate(&mut input);
        assert!(fixture_build(&fixture, input, &baseline).is_err());
    }
    let recovered_arena = arena_with_recovery(&fixture.arena, PROPERTY);
    assert_eq!(
        SourceModeDefinitionProducer::build(
            fixture.input.clone(),
            &fixture.env,
            &fixture.source_context,
            &fixture.source_type,
            &baseline,
            &recovered_arena,
        ),
        Err(SourceModeDefinitionError::InvalidArenaOwnership)
    );

    let projection = fixture.projection(&baseline);
    let mut corrupt = projection.handoff().clone();
    corrupt.definitions.rows.clear();
    assert_eq!(
        corrupt.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &fixture.source_type,
            projection.initial_obligations(),
            &fixture.arena
        ),
        Err(SourceModeDefinitionError::UnsupportedTaskShape)
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.parameters.rows[0].id = SourceModeParameterId::new(1);
    assert_eq!(
        corrupt.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &fixture.source_type,
            projection.initial_obligations(),
            &fixture.arena
        ),
        Err(SourceModeDefinitionError::InvalidParameter { index: 0 })
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.properties.rows[0].obligation = InitialObligationId::new(1);
    assert_eq!(
        corrupt.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &fixture.source_type,
            projection.initial_obligations(),
            &fixture.arena
        ),
        Err(SourceModeDefinitionError::InvalidObligation)
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.resolver_identity.origin = SemanticOrigin::new(
        fixture.source,
        fixture.module.clone(),
        SourceAnchor::Range(range(fixture.source, 45, 135)),
        vec![4, 0, 10, 1],
    );
    assert_eq!(
        corrupt.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &fixture.source_type,
            projection.initial_obligations(),
            &fixture.arena
        ),
        Err(SourceModeDefinitionError::InvalidResolverDefinition { index: 0 })
    );
}

#[test]
fn task_262_mode_definition_dependency_and_obligation_corruption_fails_closed() {
    let fixture = fixture();
    let baseline = InitialObligationTable::new();
    let projection = fixture.projection(&baseline);
    for family in 0..2 {
        let mut corrupt = projection.handoff().clone();
        if family == 0 {
            corrupt.source_context_fingerprint.push('!');
        } else {
            corrupt.source_type_fingerprint.push('!');
        }
        assert_eq!(
            corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.source_context,
                &fixture.source_type,
                projection.initial_obligations(),
                &fixture.arena
            ),
            Err(SourceModeDefinitionError::DependencyMismatch)
        );
    }
    let drafts = obligation_drafts(projection.initial_obligations());
    for mutate in [
        mutate_obligation_owner as fn(&mut InitialObligationDraft),
        mutate_obligation_range,
        mutate_obligation_assumption,
        mutate_obligation_goal,
        mutate_obligation_provenance,
        mutate_obligation_status,
    ] {
        let mut rows = drafts.clone();
        mutate(&mut rows[0]);
        assert_eq!(
            projection.handoff().validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.source_context,
                &fixture.source_type,
                &table_from_drafts(rows),
                &fixture.arena
            ),
            Err(SourceModeDefinitionError::InvalidObligation)
        );
    }
    let mut extra = drafts.clone();
    extra.push(unrelated_draft(
        fixture.source,
        "unrelated:extra",
        "unrelated:extra",
    ));
    assert_eq!(
        projection.handoff().validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &fixture.source_type,
            &table_from_drafts(extra),
            &fixture.arena
        ),
        Err(SourceModeDefinitionError::InvalidObligation)
    );

    for kind in [
        InitialObligationKind::PredicatePropertyCorrectness,
        InitialObligationKind::FunctorExistence,
        InitialObligationKind::FunctorUniqueness,
    ] {
        let mut forbidden = InitialObligationTable::new();
        let mut row = unrelated_draft(fixture.source, "unrelated", "unrelated");
        row.kind = kind;
        forbidden.insert(row);
        assert_eq!(
            fixture.build(&forbidden),
            Err(SourceModeDefinitionError::InvalidObligation)
        );
    }
    for (goal, provenance) in [
        ("source.definition.mode", "unrelated"),
        ("source.definition.mode.correctness:x", "unrelated"),
        ("unrelated", "source.definition.mode:orphan"),
    ] {
        let orphan = table_from_drafts(vec![unrelated_draft(fixture.source, goal, provenance)]);
        assert_eq!(
            validate_source_mode_definition_absence(&orphan),
            Err(SourceModeDefinitionError::InvalidObligation)
        );
    }
    assert!(
        validate_source_mode_definition_absence(&model_domain_baseline(fixture.source)).is_ok()
    );
}

#[test]
fn task_262_mode_definition_typed_installation_is_transactional() {
    let fixture = fixture();
    let baseline = model_domain_baseline(fixture.source);
    let projection = fixture.projection(&baseline);
    let untouched = fixture.typed(baseline.clone());
    let typed = untouched
        .clone()
        .with_source_mode_definition(projection.clone())
        .expect("Task 262 install");
    assert_eq!(typed.source_mode_definition(), Some(projection.handoff()));
    assert_eq!(
        typed.initial_obligations(),
        projection.initial_obligations()
    );
    assert!(untouched.source_mode_definition().is_none());
    assert_eq!(untouched.initial_obligations(), &baseline);
    assert_eq!(
        typed
            .clone()
            .with_source_mode_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceModeDefinition)
    );
    assert_eq!(
        fixture
            .typed(InitialObligationTable::new())
            .with_source_mode_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceModeDefinition)
    );

    let (predicate_typed, functor_typed) = actual_definition_family_typed_asts_for_task261();
    let attribute_typed = actual_attribute_definition_typed_ast_for_task262();
    let sibling_handoffs = [(
        0,
        predicate_typed
            .source_predicate_definition()
            .expect("actual Task 259 handoff")
            .clone(),
    )];
    for (_, predicate) in sibling_handoffs {
        let mut preinstalled = fixture.typed(baseline.clone());
        preinstalled.inject_source_predicate_definition_for_test(predicate);
        assert_eq!(
            preinstalled.with_source_mode_definition(projection.clone()),
            Err(TypedAstError::InvalidSourceModeDefinition)
        );
    }
    let mut preinstalled = fixture.typed(baseline.clone());
    preinstalled.inject_source_functor_definition_for_test(
        functor_typed
            .source_functor_definition()
            .expect("actual Task 260 handoff")
            .clone(),
    );
    assert_eq!(
        preinstalled.with_source_mode_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceModeDefinition)
    );
    let mut preinstalled = fixture.typed(baseline.clone());
    preinstalled.inject_source_attribute_definition_for_test(
        attribute_typed
            .source_attribute_definition()
            .expect("actual Task 261 handoff")
            .clone(),
    );
    assert_eq!(
        preinstalled.with_source_mode_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceModeDefinition)
    );

    let (retained, handoff, updated) = projection.clone().into_parts();
    assert_eq!(retained, baseline);
    assert_eq!(&handoff, projection.handoff());
    assert_eq!(&updated, projection.initial_obligations());
    let mut corrupt = projection;
    corrupt.initial_obligations = InitialObligationTable::new();
    assert_eq!(
        fixture.typed(baseline).with_source_mode_definition(corrupt),
        Err(TypedAstError::InvalidSourceModeDefinition)
    );
}

#[test]
fn task_262_mode_definition_final_clone_debug_determinism_and_family_isolation() {
    let fixture = fixture();
    let legacy = fixture.typed(InitialObligationTable::new());
    let legacy_debug = legacy.debug_text();
    assert!(legacy.source_mode_definition().is_none());
    assert!(!legacy_debug.contains("source-mode-definition-debug-v1"));
    let projection = fixture.projection(&InitialObligationTable::new());
    let mut injected = legacy.clone();
    injected.inject_source_mode_definition_for_test(projection.handoff().clone());
    assert_eq!(
        injected.source_mode_definition(),
        Some(projection.handoff())
    );
    let typed = legacy
        .with_source_mode_definition(projection.clone())
        .expect("Task 262 install");
    assert_eq!(typed.clone(), typed);
    assert_eq!(typed.debug_text(), typed.clone().debug_text());
    assert_eq!(
        typed
            .debug_text()
            .matches("source-mode-definition-debug-v1")
            .count(),
        1
    );
    assert!(typed.source_mode_definition().is_some());
    assert!(typed.source_predicate_definition().is_none());
    assert!(typed.source_functor_definition().is_none());
    assert!(typed.source_attribute_definition().is_none());
    assert!(typed.source_term().is_none());
    assert!(typed.source_application().is_none());
    assert!(typed.source_structure().is_none());
    assert!(typed.source_set_term().is_none());
    assert!(typed.source_atomic_formula().is_none());
    assert!(typed.source_composite_formula().is_none());
    assert!(typed.source_evidence().is_none());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.types().is_empty());
    assert_eq!(typed.initial_obligations().len(), 1);
    let debug = typed.debug_text().to_ascii_lowercase();
    for deferred in [
        "accepted",
        "activated",
        "verification-condition",
        "proof-discharge",
        "semantic-fact",
    ] {
        assert!(
            !debug.contains(deferred),
            "deferred semantic marker leaked: {deferred}"
        );
    }

    let resolved = assemble_empty(&typed).expect("valid Task 262 final assembly");
    assert_eq!(
        resolved.source_mode_definition(),
        typed.source_mode_definition()
    );
    assert_eq!(resolved.clone(), resolved);
    assert_eq!(resolved.debug_text(), resolved.clone().debug_text());
    assert_eq!(
        resolved
            .debug_text()
            .matches("source-mode-definition-debug-v1")
            .count(),
        1
    );

    let mut stale = typed.clone();
    let mut stale_handoff = projection.handoff().clone();
    stale_handoff
        .source_type_fingerprint
        .push_str(" stale-final");
    stale.inject_source_mode_definition_for_test(stale_handoff);
    assert_eq!(
        assemble_empty(&stale),
        Err(ResolvedTypedAstError::InvalidSourceModeDefinition)
    );

    let exact_obligations = obligation_drafts(projection.initial_obligations());
    let mut missing = typed.clone();
    missing.replace_initial_obligations_for_test(InitialObligationTable::new());
    assert_eq!(
        assemble_empty(&missing),
        Err(ResolvedTypedAstError::InvalidSourceModeDefinition)
    );
    let mut modified_rows = exact_obligations.clone();
    modified_rows[0].goal = InitialObligationGoal::new("forged final goal");
    let mut modified = typed.clone();
    modified.replace_initial_obligations_for_test(table_from_drafts(modified_rows));
    assert_eq!(
        assemble_empty(&modified),
        Err(ResolvedTypedAstError::InvalidSourceModeDefinition)
    );
    let mut extra_rows = exact_obligations.clone();
    extra_rows.push(exact_obligations[0].clone());
    let mut extra = typed.clone();
    extra.replace_initial_obligations_for_test(table_from_drafts(extra_rows));
    assert_eq!(
        assemble_empty(&extra),
        Err(ResolvedTypedAstError::InvalidSourceModeDefinition)
    );
    let mut orphan = fixture.typed(InitialObligationTable::new());
    orphan.replace_initial_obligations_for_test(table_from_drafts(vec![unrelated_draft(
        fixture.source,
        "source.definition.mode.correctness:orphan",
        "unrelated",
    )]));
    assert_eq!(
        assemble_empty(&orphan),
        Err(ResolvedTypedAstError::InvalidSourceModeDefinition)
    );

    let (predicate_typed, functor_typed) = actual_definition_family_typed_asts_for_task261();
    let attribute_typed = actual_attribute_definition_typed_ast_for_task262();
    let predicate_handoff = predicate_typed
        .source_predicate_definition()
        .expect("actual Task 259 handoff")
        .clone();
    let functor_handoff = functor_typed
        .source_functor_definition()
        .expect("actual Task 260 handoff")
        .clone();
    let attribute_handoff = attribute_typed
        .source_attribute_definition()
        .expect("actual Task 261 handoff")
        .clone();

    let mut mode_then_predicate = typed.clone();
    mode_then_predicate.inject_source_predicate_definition_for_test(predicate_handoff.clone());
    assert_eq!(
        assemble_empty(&mode_then_predicate),
        Err(ResolvedTypedAstError::InvalidSourcePredicateDefinition)
    );
    let mut predicate_then_mode = predicate_typed;
    predicate_then_mode.inject_source_mode_definition_for_test(projection.handoff().clone());
    assert_eq!(
        assemble_empty(&predicate_then_mode),
        Err(ResolvedTypedAstError::InvalidSourcePredicateDefinition)
    );

    let mut mode_then_functor = typed.clone();
    mode_then_functor.inject_source_functor_definition_for_test(functor_handoff.clone());
    assert_eq!(
        assemble_empty(&mode_then_functor),
        Err(ResolvedTypedAstError::InvalidSourceFunctorDefinition)
    );
    let mut functor_then_mode = functor_typed;
    functor_then_mode.inject_source_mode_definition_for_test(projection.handoff().clone());
    assert_eq!(
        assemble_empty(&functor_then_mode),
        Err(ResolvedTypedAstError::InvalidSourceFunctorDefinition)
    );

    let mut mode_then_attribute = typed.clone();
    mode_then_attribute.inject_source_attribute_definition_for_test(attribute_handoff.clone());
    assert_eq!(
        assemble_empty(&mode_then_attribute),
        Err(ResolvedTypedAstError::InvalidSourceAttributeDefinition)
    );
    let mut attribute_then_mode = attribute_typed;
    attribute_then_mode.inject_source_mode_definition_for_test(projection.handoff().clone());
    assert_eq!(
        assemble_empty(&attribute_then_mode),
        Err(ResolvedTypedAstError::InvalidSourceAttributeDefinition)
    );
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
    let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task262"));
    let arena = task262_arena(source);
    let source_context = task248_context(source, &module);
    let (env, symbol, definition, contribution) = resolver_env(source, module.clone());
    let type_base = SourceTypeProducer::build(
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
                bare_set_type(
                    source,
                    module.clone(),
                    TYPE_EXPRESSION_Y,
                    TYPE_HEAD_Y,
                    38,
                    41,
                ),
            ],
            arguments: Vec::new(),
        },
        source_context.binding_env(),
        &env,
        &arena,
    )
    .expect("Task 249 Profile B");
    let source_type = SourceTypeModeRhsProducer::extend(
        &type_base,
        SourceTypeModeRhsExtensionInput {
            source_id: source,
            module_id: module.clone(),
            rhs: vec![SourceTypeModeRhsInput {
                definition_site: TypedSiteRef::Node(MODE_DEFINITION),
                definition_range: range(source, 45, 135),
                source_ordinal: 0,
                expression: bare_set_type(source, module.clone(), RHS_EXPRESSION, RHS_HEAD, 95, 98),
            }],
        },
        &arena,
    )
    .expect("Task 249M mode RHS");
    let input = task262_input(source, module.clone(), symbol, definition, contribution);
    Fixture {
        source,
        module,
        env,
        source_context,
        source_type,
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
            source_range: range(source, 0, 140),
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
    .expect("complete Profile B")
    .into_handoff()
}

fn definition_block_shell(
    source: SourceId,
    module: &ModuleId,
) -> mizar_resolve::declarations::DeclarationShellId {
    let mut builder = SurfaceAstBuilder::new(source);
    let block = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source, 0, 140),
        Vec::new(),
    );
    let items = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source, 0, 140),
        vec![block],
    );
    let unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source, 0, 140),
        vec![items],
    );
    let root = builder.add_node(SurfaceNodeKind::Root, range(source, 0, 140), vec![unit]);
    let ast = builder.finish(Some(root), None);
    DeclarationShellCollector::new(&ast, module)
        .collect()
        .declarations()[0]
        .id()
}

fn resolver_env(
    source: SourceId,
    module: ModuleId,
) -> (SymbolEnv, SymbolId, DefinitionId, SourceContributionId) {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 140)),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 45, 135)),
        vec![4, 0, 10, 0],
    );
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new("Task262Mode"),
        FullyQualifiedName::new("pkg::task262::Task262Mode"),
    );
    let notation = "Task262Mode [ x , y ]";
    let signature = SignatureShell::Opaque {
        schema: "parser-signature-v1".to_owned(),
        payload: "mode:Task262Mode [ x , y ]".to_owned(),
    };
    indexes.symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Mode,
            NamespacePath::new(module.path().as_str()),
            notation,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
        .with_notation_spelling(notation)
        .with_signature(signature.clone()),
    );
    let definition = indexes.definitions.insert(
        DefinitionShell::new(symbol.clone(), DefinitionKind::Mode, origin, contribution)
            .with_visibility(Visibility::Public)
            .with_notation_shape(notation)
            .with_signature(signature),
    );
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

fn task262_input(
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
) -> SourceModeDefinitionHandoffInput {
    SourceModeDefinitionHandoffInput {
        source_id: source,
        module_id: module,
        definitions: vec![SourceModeDefinitionInput {
            symbol, definition, contribution, site: TypedSiteRef::Node(MODE_DEFINITION),
            source_range: range(source, 45, 135), source_ordinal: 0, context: BindingContextId::new(1),
            recovery: SourceModeDefinitionRecovery::Normal,
            spelling: "mode Task262ModeDefinition: Task262Mode [x, y] is set;\n  sethood by computation(steps: 1);".to_owned(),
            application: SourceModeApplicationId::new(0), expansion: SourceModeExpansionId::new(0),
            inhabitation_request: SourceModeInhabitationRequestId::new(0), property: Some(SourceModePropertyId::new(0)),
        }],
        parameters: vec![
            SourceModeParameterInput { owner: SourceModeDefinitionId::new(0), ordinal: 0, binding: BindingId::new(0), written_type: SourceTypeApplicationId::new(0), site: TypedSiteRef::Node(PARAMETER_X), source_range: range(source, 13, 26), declaration_range: range(source, 17, 18), pattern_range: range(source, 86, 87), context: BindingContextId::new(1), recovery: SourceModeDefinitionRecovery::Normal, spelling: "let x be set;".to_owned() },
            SourceModeParameterInput { owner: SourceModeDefinitionId::new(0), ordinal: 1, binding: BindingId::new(1), written_type: SourceTypeApplicationId::new(1), site: TypedSiteRef::Node(PARAMETER_Y), source_range: range(source, 29, 42), declaration_range: range(source, 33, 34), pattern_range: range(source, 89, 90), context: BindingContextId::new(1), recovery: SourceModeDefinitionRecovery::Normal, spelling: "let y be set;".to_owned() },
        ],
        applications: vec![SourceModeApplicationInput { owner: SourceModeDefinitionId::new(0), ordinal: 0, parameters: vec![SourceModeParameterId::new(0), SourceModeParameterId::new(1)], site: TypedSiteRef::Node(APPLICATION), source_range: range(source, 73, 91), context: BindingContextId::new(1), recovery: SourceModeDefinitionRecovery::Normal, spelling: "Task262Mode [ x , y ]".to_owned() }],
        expansions: vec![SourceModeExpansionInput { owner: SourceModeDefinitionId::new(0), ordinal: 0, rhs: SourceTypeModeRhsId::new(0), site: TypedSiteRef::Node(RHS_EXPRESSION), source_range: range(source, 95, 98), context: BindingContextId::new(1), recovery: SourceModeDefinitionRecovery::Normal, spelling: "set".to_owned() }],
        inhabitation_requests: vec![SourceModeInhabitationRequestInput { owner: SourceModeDefinitionId::new(0), ordinal: 0, expansion: SourceModeExpansionId::new(0), kind: SourceModeInhabitationRequestKind::Rhs, site: TypedSiteRef::Node(RHS_EXPRESSION), source_range: range(source, 95, 98), context: BindingContextId::new(1), recovery: SourceModeDefinitionRecovery::Normal, spelling: "set".to_owned() }],
        properties: vec![SourceModePropertyInput { owner: SourceModeDefinitionId::new(0), ordinal: 0, kind: SourceModePropertyKind::Sethood, site: TypedSiteRef::Node(PROPERTY), source_range: range(source, 102, 135), justification: SourceAnchor::Range(range(source, 113, 134)), recovery: SourceModeDefinitionRecovery::Normal, spelling: "sethood by computation(steps: 1);".to_owned() }],
    }
}

fn task262_arena(source: SourceId) -> TypedArena {
    let mut nodes = Vec::with_capacity(54);
    for index in 0..54 {
        let (kind, source_range, context) = match index {
            34 => ("source.type.head", range(source, 22, 25), 1),
            35 => ("source.type.expression", range(source, 22, 25), 1),
            37 => ("source.definition.mode.parameter", range(source, 17, 18), 1),
            38 => ("source.type.head", range(source, 38, 41), 1),
            39 => ("source.type.expression", range(source, 38, 41), 1),
            41 => ("source.definition.mode.parameter", range(source, 33, 34), 1),
            42 => (
                "source.definition.mode.application",
                range(source, 73, 91),
                1,
            ),
            43 => ("source.type.head", range(source, 95, 98), 1),
            44 => ("source.type.expression", range(source, 95, 98), 1),
            46 => (
                "source.definition.mode.property.justification",
                range(source, 113, 134),
                1,
            ),
            48 => (
                "source.definition.mode.property",
                range(source, 102, 135),
                1,
            ),
            49 => ("source.definition.mode", range(source, 45, 135), 1),
            50 => ("source.definition", range(source, 0, 140), 1),
            53 => ("source.module", range(source, 0, 140), 0),
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
    TypedArena::try_new(Some(MODULE_ROOT), nodes).expect("Task 262 arena")
}

fn arena_with_recovery(arena: &TypedArena, node: TypedNodeId) -> TypedArena {
    let mut nodes = arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
    nodes[node.index()].recovery = NodeRecoveryState::Recovered;
    TypedArena::try_new(arena.root(), nodes).expect("coherent recovered Task 262 arena")
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

fn fixture_build(
    fixture: &Fixture,
    input: SourceModeDefinitionHandoffInput,
    baseline: &InitialObligationTable,
) -> Result<SourceModeDefinitionProjection, SourceModeDefinitionError> {
    SourceModeDefinitionProducer::build(
        input,
        &fixture.env,
        &fixture.source_context,
        &fixture.source_type,
        baseline,
        &fixture.arena,
    )
}

fn obligation_drafts(table: &InitialObligationTable) -> Vec<InitialObligationDraft> {
    table
        .iter()
        .map(|(_, row)| InitialObligationDraft {
            kind: row.kind,
            owner: row.owner.clone(),
            source_range: row.source_range,
            assumptions: row.assumptions.clone(),
            goal: row.goal.clone(),
            provenance: row.provenance.clone(),
            status: row.status,
        })
        .collect()
}

fn table_from_drafts(rows: Vec<InitialObligationDraft>) -> InitialObligationTable {
    let mut table = InitialObligationTable::new();
    for row in rows {
        table.insert(row);
    }
    table
}

fn unrelated_draft(source: SourceId, goal: &str, provenance: &str) -> InitialObligationDraft {
    InitialObligationDraft {
        kind: InitialObligationKind::Sethood,
        owner: TypedSiteRef::Node(TYPE_HEAD_X),
        source_range: range(source, 22, 25),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new(goal),
        provenance: InitialObligationProvenance::new(provenance),
        status: InitialObligationStatus::Pending,
    }
}

fn model_domain_baseline(source: SourceId) -> InitialObligationTable {
    table_from_drafts(vec![unrelated_draft(
        source,
        "source.definition.model:allowed",
        "source.definition.model.allowed",
    )])
}

fn mutate_obligation_owner(row: &mut InitialObligationDraft) {
    row.owner = TypedSiteRef::Node(MODE_DEFINITION);
}
fn mutate_obligation_range(row: &mut InitialObligationDraft) {
    row.source_range.end -= 1;
}
fn mutate_obligation_assumption(row: &mut InitialObligationDraft) {
    row.assumptions.push(TypeFactId::new(0));
}
fn mutate_obligation_goal(row: &mut InitialObligationDraft) {
    row.goal = InitialObligationGoal::new("forged");
}
fn mutate_obligation_provenance(row: &mut InitialObligationDraft) {
    row.provenance = InitialObligationProvenance::new("forged");
}
fn mutate_obligation_status(row: &mut InitialObligationDraft) {
    row.status = InitialObligationStatus::Blocked;
}

fn assert_task262_obligation(row: &InitialObligation, source: SourceId, index: usize) {
    assert_eq!(row.id, InitialObligationId::new(index));
    assert_eq!(row.kind, InitialObligationKind::Sethood);
    assert_eq!(row.owner, TypedSiteRef::Node(PROPERTY));
    assert_eq!(row.source_range, range(source, 102, 135));
    assert!(row.assumptions.is_empty());
    assert_eq!(
        row.goal.as_str(),
        "source.definition.mode.correctness:definition=0:sethood"
    );
    assert_eq!(
        row.provenance.as_str(),
        "source.definition.mode:definition=0:property=0"
    );
    assert_eq!(row.status, InitialObligationStatus::Pending);
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
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
