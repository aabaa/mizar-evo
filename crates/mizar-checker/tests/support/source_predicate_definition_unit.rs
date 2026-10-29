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
    source_term::{
        SourcePrimaryTermHandoffInput, SourcePrimaryTermInput, SourcePrimaryTermProducer,
        SourcePrimaryTermReferenceInput,
    },
    source_type::{
        SourceTypeApplicationInput, SourceTypeExpressionInput, SourceTypeHandoffInput,
        SourceTypeProducer,
    },
    typed_ast::{
        CoercionTable, InitialObligation, InitialObligationTable, LocalTypeContextId,
        TypeDiagnosticTable, TypeFactId, TypeFactTable, TypeTable, TypedArenaBuilder, TypedAst,
        TypedAstError, TypedAstParts, TypedNode, TypedNodeId, TypedNodeLinks, TypingState,
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

const TYPE_HEAD_X: TypedNodeId = TypedNodeId::new(0);
const TYPE_EXPRESSION_X: TypedNodeId = TypedNodeId::new(1);
const PARAMETER_X: TypedNodeId = TypedNodeId::new(2);
const TYPE_HEAD_Y: TypedNodeId = TypedNodeId::new(3);
const TYPE_EXPRESSION_Y: TypedNodeId = TypedNodeId::new(4);
const PARAMETER_Y: TypedNodeId = TypedNodeId::new(5);
const GUARD_LEFT: TypedNodeId = TypedNodeId::new(6);
const GUARD_RIGHT: TypedNodeId = TypedNodeId::new(7);
const GUARD_FORMULA: TypedNodeId = TypedNodeId::new(8);
const DEFINIENS_LEFT: TypedNodeId = TypedNodeId::new(9);
const DEFINIENS_RIGHT: TypedNodeId = TypedNodeId::new(10);
const DEFINIENS_FORMULA: TypedNodeId = TypedNodeId::new(11);
const GUARD_OWNER: TypedNodeId = TypedNodeId::new(12);
const PREDICATE_OWNER: TypedNodeId = TypedNodeId::new(13);
const PROPERTY_OWNER: TypedNodeId = TypedNodeId::new(14);
const DEFINITION_OWNER: TypedNodeId = TypedNodeId::new(15);
const MODULE_OWNER: TypedNodeId = TypedNodeId::new(16);

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
    input: SourcePredicateDefinitionHandoffInput,
}

impl Fixture {
    fn build(&self, base: &InitialObligationTable) -> SourcePredicateDefinitionProjection {
        SourcePredicateDefinitionProducer::build(
            self.input.clone(),
            &self.env,
            &self.source_context,
            &self.source_type,
            &self.source_term,
            &self.source_atomic_formula,
            base,
            &self.arena,
        )
        .expect("exact Task 259 projection")
    }

    fn typed_ast(&self, base: InitialObligationTable) -> TypedAst {
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
            initial_obligations: base,
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task 259 typed baseline")
        .with_source_term(self.source_term.clone())
        .expect("Task 252 installation")
        .with_source_atomic_formula(self.source_atomic_formula.clone())
        .expect("Task 256 installation")
    }
}

pub(crate) fn actual_predicate_definition_projection_for_task263()
-> SourcePredicateDefinitionProjection {
    fixture().build(&InitialObligationTable::new())
}

#[test]
fn task_259_exact_predicate_definition_payload_and_pending_obligation() {
    let fixture = fixture();
    let projection = fixture.build(&InitialObligationTable::new());
    let handoff = projection.handoff();
    assert_eq!(handoff.source_id(), fixture.source);
    assert_eq!(handoff.module_id(), &fixture.module);
    assert_eq!(handoff.definitions().len(), 1);
    assert!(!handoff.definitions().is_empty());
    assert_eq!(handoff.parameters().len(), 2);
    assert!(!handoff.parameters().is_empty());
    assert_eq!(handoff.guards().len(), 1);
    assert!(!handoff.guards().is_empty());
    assert_eq!(handoff.properties().len(), 1);
    assert!(!handoff.properties().is_empty());
    assert_eq!(handoff.correctness().len(), 1);
    assert!(!handoff.correctness().is_empty());
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

    let definition = handoff
        .definitions()
        .get(SourcePredicateDefinitionId::new(0))
        .unwrap();
    assert_eq!(definition.id(), SourcePredicateDefinitionId::new(0));
    assert_eq!(
        definition.symbol().fqn().as_str(),
        "pkg::task259::task259_rel"
    );
    assert_eq!(definition.definition().index(), 0);
    assert_eq!(definition.contribution().index(), 0);
    assert_eq!(definition.site(), &TypedSiteRef::Node(PREDICATE_OWNER));
    assert_eq!(definition.source_range(), range(fixture.source, 61, 122));
    assert_eq!(definition.source_ordinal(), 0);
    assert_eq!(definition.context(), BindingContextId::new(1));
    assert_eq!(
        definition.recovery(),
        SourcePredicateDefinitionRecovery::Normal
    );
    assert_eq!(
        definition.spelling(),
        "pred Task259PredicateDefinition: x task259_rel y means x = y;"
    );
    assert_eq!(definition.definiens(), SourceAtomicFormulaId::new(1));
    assert_eq!(definition.origin().source_id(), fixture.source);
    assert_eq!(definition.origin().module_id(), &fixture.module);
    assert_eq!(
        definition.origin().anchor(),
        &SourceAnchor::Range(range(fixture.source, 61, 122))
    );
    assert_eq!(definition.origin().structural_path(), [4, 0, 8, 0]);
    assert!(definition.origin().import_edge().is_none());
    assert!(!definition.origin().is_recovered());

    let parameters = handoff.parameters().iter().collect::<Vec<_>>();
    assert_eq!(parameters[0].0, SourcePredicateParameterId::new(0));
    assert_eq!(parameters[0].1.id(), SourcePredicateParameterId::new(0));
    assert_eq!(parameters[0].1.owner(), SourcePredicateDefinitionId::new(0));
    assert_eq!(parameters[0].1.ordinal(), 0);
    assert_eq!(parameters[0].1.binding(), BindingId::new(0));
    assert_eq!(
        parameters[0].1.written_type(),
        SourceTypeApplicationId::new(0)
    );
    assert_eq!(parameters[0].1.site(), &TypedSiteRef::Node(PARAMETER_X));
    assert_eq!(
        parameters[0].1.source_range(),
        range(fixture.source, 13, 26)
    );
    assert_eq!(
        parameters[0].1.declaration_range(),
        range(fixture.source, 17, 18)
    );
    assert_eq!(parameters[0].1.context(), BindingContextId::new(1));
    assert_eq!(
        parameters[0].1.recovery(),
        SourcePredicateDefinitionRecovery::Normal
    );
    assert_eq!(parameters[0].1.spelling(), "let x be set;");
    assert_eq!(parameters[1].0, SourcePredicateParameterId::new(1));
    assert_eq!(parameters[1].1.id(), SourcePredicateParameterId::new(1));
    assert_eq!(parameters[1].1.owner(), SourcePredicateDefinitionId::new(0));
    assert_eq!(parameters[1].1.ordinal(), 1);
    assert_eq!(parameters[1].1.binding(), BindingId::new(1));
    assert_eq!(
        parameters[1].1.written_type(),
        SourceTypeApplicationId::new(1)
    );
    assert_eq!(parameters[1].1.site(), &TypedSiteRef::Node(PARAMETER_Y));
    assert_eq!(
        parameters[1].1.source_range(),
        range(fixture.source, 29, 42)
    );
    assert_eq!(
        parameters[1].1.declaration_range(),
        range(fixture.source, 33, 34)
    );
    assert_eq!(parameters[1].1.context(), BindingContextId::new(1));
    assert_eq!(
        parameters[1].1.recovery(),
        SourcePredicateDefinitionRecovery::Normal
    );
    assert_eq!(parameters[1].1.spelling(), "let y be set;");

    let guard = handoff
        .guards()
        .get(SourcePredicateGuardId::new(0))
        .unwrap();
    assert_eq!(guard.id(), SourcePredicateGuardId::new(0));
    assert_eq!(guard.owner(), SourcePredicateDefinitionId::new(0));
    assert_eq!(guard.ordinal(), 0);
    assert_eq!(guard.formula(), SourceAtomicFormulaId::new(0));
    assert_eq!(guard.site(), &TypedSiteRef::Node(GUARD_OWNER));
    assert_eq!(guard.source_range(), range(fixture.source, 45, 58));
    assert_eq!(guard.context(), BindingContextId::new(1));
    assert_eq!(guard.recovery(), SourcePredicateDefinitionRecovery::Normal);
    assert_eq!(guard.spelling(), "assume x = x;");
    let property = handoff
        .properties()
        .get(SourcePredicatePropertyId::new(0))
        .unwrap();
    assert_eq!(property.id(), SourcePredicatePropertyId::new(0));
    assert_eq!(property.owner(), SourcePredicateDefinitionId::new(0));
    assert_eq!(property.ordinal(), 0);
    assert_eq!(property.kind(), SourcePredicatePropertyKind::Symmetry);
    assert_eq!(property.site(), &TypedSiteRef::Node(PROPERTY_OWNER));
    assert_eq!(property.source_range(), range(fixture.source, 125, 159));
    assert_eq!(
        property.justification(),
        &SourceAnchor::Range(range(fixture.source, 134, 158))
    );
    assert_eq!(
        property.recovery(),
        SourcePredicateDefinitionRecovery::Normal
    );
    assert_eq!(property.spelling(), "symmetry by computation(steps: 1);");
    let correctness = handoff
        .correctness()
        .get(SourcePredicateCorrectnessId::new(0))
        .unwrap();
    assert_eq!(correctness.id(), SourcePredicateCorrectnessId::new(0));
    assert_eq!(correctness.owner(), SourcePredicateDefinitionId::new(0));
    assert_eq!(correctness.property(), SourcePredicatePropertyId::new(0));
    assert_eq!(correctness.ordinal(), 0);
    assert_eq!(
        correctness.source_anchor(),
        &SourceAnchor::Range(range(fixture.source, 125, 159))
    );
    assert_eq!(correctness.obligation(), InitialObligationId::new(0));

    assert!(projection.base_initial_obligations().is_empty());
    let obligation = projection
        .initial_obligations()
        .get(InitialObligationId::new(0))
        .unwrap();
    assert_eq!(
        obligation.kind,
        InitialObligationKind::PredicatePropertyCorrectness
    );
    assert_eq!(obligation.owner, *property.site());
    assert_eq!(obligation.source_range, property.source_range());
    assert!(obligation.assumptions.is_empty());
    assert_eq!(
        obligation.goal.as_str(),
        "source.definition.predicate.correctness:property=0"
    );
    assert_eq!(
        obligation.provenance.as_str(),
        "source.definition.predicate:definition=0:property=0"
    );
    assert_eq!(obligation.status, InitialObligationStatus::Pending);

    let debug = handoff.debug_text();
    assert!(debug.starts_with(
        "source-predicate-definition-debug-v1\nmodule: task259\nsource-context-fingerprint: \""
    ));
    assert!(debug.contains(
        "definition#0 symbol=\"pkg::task259::task259_rel\" definition=0 contribution=0 ordinal=0 range=61..122 site=node#13 context=1 recovery=normal origin_range=61..122 origin_path=[4, 0, 8, 0] spelling=\"pred Task259PredicateDefinition: x task259_rel y means x = y;\" definiens=1\n"
    ));
    assert!(debug.contains(
        "parameter#0 owner=0 ordinal=0 binding=0 written_type=0 range=13..26 declaration_range=17..18 site=node#2 context=1 recovery=normal spelling=\"let x be set;\"\n"
    ));
    assert!(debug.contains(
        "parameter#1 owner=0 ordinal=1 binding=1 written_type=1 range=29..42 declaration_range=33..34 site=node#5 context=1 recovery=normal spelling=\"let y be set;\"\n"
    ));
    assert!(debug.contains(
        "guard#0 owner=0 ordinal=0 formula=0 range=45..58 site=node#12 context=1 recovery=normal spelling=\"assume x = x;\"\n"
    ));
    assert!(debug.contains(
        "property#0 owner=0 ordinal=0 kind=symmetry range=125..159 site=node#14 justification=range:134..158 recovery=normal spelling=\"symmetry by computation(steps: 1);\"\n"
    ));
    assert!(debug.ends_with(
        "correctness#0 owner=0 property=0 ordinal=0 anchor=range:125..159 obligation=0\n"
    ));
}

#[test]
fn task_259_independent_row_and_field_corruption_fails_closed() {
    let fixture = fixture();
    let base = InitialObligationTable::new();

    for arity in [None, Some(0), Some(1), Some(3)] {
        let (env, symbol, definition, contribution) =
            resolver_env(fixture.source, fixture.module.clone(), arity);
        assert_eq!(
            (&symbol, definition, contribution),
            (
                &fixture.input.definitions[0].symbol,
                fixture.input.definitions[0].definition,
                fixture.input.definitions[0].contribution,
            )
        );
        assert_eq!(
            env.symbols().get(&symbol),
            fixture.env.symbols().get(&symbol)
        );
        let actual_predicate = env.definitions().get(definition).unwrap();
        let original_predicate = fixture.env.definitions().get(definition).unwrap();
        assert_eq!(
            (
                actual_predicate.arity(),
                actual_predicate.symbol(),
                actual_predicate.notation_shape(),
                actual_predicate.origin(),
                actual_predicate.signature(),
            ),
            (
                arity,
                original_predicate.symbol(),
                original_predicate.notation_shape(),
                original_predicate.origin(),
                original_predicate.signature(),
            )
        );
        let property = fixture
            .env
            .definitions()
            .iter()
            .find(|definition| definition.kind() == DefinitionKind::Attribute)
            .unwrap();
        assert_eq!(env.definitions().get(property.id()), Some(property));
        assert_eq!(
            env.contributions().get(contribution),
            fixture.env.contributions().get(contribution)
        );
        let mut corrupted = fixture.clone();
        corrupted.env = env;
        assert_eq!(
            build_with_input(&corrupted, corrupted.input.clone(), &base).unwrap_err(),
            SourcePredicateDefinitionError::InvalidResolverDefinition,
            "{arity:?}"
        );
    }

    assert_input_rejected(&fixture, &base, |input| input.source_id = other_source_id());
    assert_input_rejected(&fixture, &base, |input| {
        input.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
    });

    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("stale"),
            FullyQualifiedName::new("pkg::task259::stale"),
        );
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].definition = alternate_definition_id(&fixture);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].contribution = alternate_contribution_id(&fixture);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].site = TypedSiteRef::Node(DEFINITION_OWNER);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].source_range = range(fixture.source, 60, 122);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].source_ordinal = 1;
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].context = BindingContextId::new(0);
    });
    for owner in [
        PREDICATE_OWNER,
        PARAMETER_X,
        PARAMETER_Y,
        GUARD_OWNER,
        PROPERTY_OWNER,
    ] {
        let module_context_arena =
            arena_with_node_context(&fixture.arena, owner, LocalTypeContextId::new(0));
        assert_eq!(
            SourcePredicateDefinitionProducer::build(
                fixture.input.clone(),
                &fixture.env,
                &fixture.source_context,
                &fixture.source_type,
                &fixture.source_term,
                &fixture.source_atomic_formula,
                &base,
                &module_context_arena,
            ),
            Err(SourcePredicateDefinitionError::InvalidArenaOwnership)
        );
    }
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].recovery = SourcePredicateDefinitionRecovery::Degraded;
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].spelling.push(' ');
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions[0].definiens = SourceAtomicFormulaId::new(0);
    });

    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[0].owner = SourcePredicateDefinitionId::new(1);
    });
    assert_input_rejected(&fixture, &base, |input| input.parameters[1].ordinal = 0);
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[0].binding = BindingId::new(1);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[0].written_type = SourceTypeApplicationId::new(1);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[0].site = TypedSiteRef::Node(PARAMETER_Y);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[0].source_range = range(fixture.source, 14, 26);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[0].declaration_range = range(fixture.source, 18, 19);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[0].context = BindingContextId::new(0);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[0].recovery = SourcePredicateDefinitionRecovery::Degraded;
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[0].spelling.push(' ')
    });

    assert_input_rejected(&fixture, &base, |input| {
        input.guards[0].owner = SourcePredicateDefinitionId::new(1);
    });
    assert_input_rejected(&fixture, &base, |input| input.guards[0].ordinal = 1);
    assert_input_rejected(&fixture, &base, |input| {
        input.guards[0].formula = SourceAtomicFormulaId::new(1);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.guards[0].site = TypedSiteRef::Node(DEFINIENS_FORMULA);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.guards[0].source_range = range(fixture.source, 46, 58);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.guards[0].context = BindingContextId::new(0);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.guards[0].recovery = SourcePredicateDefinitionRecovery::Degraded;
    });
    assert_input_rejected(&fixture, &base, |input| input.guards[0].spelling.push(' '));

    assert_input_rejected(&fixture, &base, |input| {
        input.properties[0].owner = SourcePredicateDefinitionId::new(1);
    });
    assert_input_rejected(&fixture, &base, |input| input.properties[0].ordinal = 1);
    assert_input_rejected(&fixture, &base, |input| {
        input.properties[0].site = TypedSiteRef::Node(PREDICATE_OWNER);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.properties[0].source_range = range(fixture.source, 124, 159);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.properties[0].justification = SourceAnchor::Range(range(fixture.source, 137, 158));
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.properties[0].recovery = SourcePredicateDefinitionRecovery::Degraded;
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.properties[0].spelling.push(' ')
    });

    assert_input_rejected(&fixture, &base, |input| {
        input.correctness[0].owner = SourcePredicateDefinitionId::new(1);
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.correctness[0].property = SourcePredicatePropertyId::new(1);
    });
    assert_input_rejected(&fixture, &base, |input| input.correctness[0].ordinal = 1);
    assert_input_rejected(&fixture, &base, |input| {
        input.correctness[0].source_anchor = SourceAnchor::Range(range(fixture.source, 126, 159));
    });

    assert_input_rejected(&fixture, &base, |input| input.definitions.clear());
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters.pop();
    });
    assert_input_rejected(&fixture, &base, |input| input.guards.clear());
    assert_input_rejected(&fixture, &base, |input| input.properties.clear());
    assert_input_rejected(&fixture, &base, |input| input.correctness.clear());
    assert_input_rejected(&fixture, &base, |input| {
        input.definitions.push(input.definitions[0].clone());
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters.push(input.parameters[1].clone());
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.guards.push(input.guards[0].clone());
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.properties.push(input.properties[0].clone());
    });
    assert_input_rejected(&fixture, &base, |input| {
        input.correctness.push(input.correctness[0].clone());
    });
    assert_input_rejected(&fixture, &base, |input| input.parameters.swap(0, 1));
    assert_input_rejected(&fixture, &base, |input| {
        input.parameters[1] = input.parameters[0].clone();
    });

    let projection = fixture.build(&base);
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.source_id = other_source_id();
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].id = SourcePredicateDefinitionId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].id = SourcePredicateParameterId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows[0].id = SourcePredicateGuardId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.properties.rows[0].id = SourcePredicatePropertyId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.correctness.rows[0].id = SourcePredicateCorrectnessId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows.swap(0, 1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[1] = handoff.parameters.rows[0].clone();
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows.clear();
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows.pop();
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows.clear();
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.properties.rows.clear();
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.correctness.rows.clear();
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        let mut row = handoff.definitions.rows[0].clone();
        row.id = SourcePredicateDefinitionId::new(1);
        handoff.definitions.rows.push(row);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        let mut row = handoff.parameters.rows[1].clone();
        row.id = SourcePredicateParameterId::new(2);
        handoff.parameters.rows.push(row);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        let mut row = handoff.guards.rows[0].clone();
        row.id = SourcePredicateGuardId::new(1);
        handoff.guards.rows.push(row);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        let mut row = handoff.properties.rows[0].clone();
        row.id = SourcePredicatePropertyId::new(1);
        handoff.properties.rows.push(row);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        let mut row = handoff.correctness.rows[0].clone();
        row.id = SourcePredicateCorrectnessId::new(1);
        handoff.correctness.rows.push(row);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].source_range.start += 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("coherent-stale"),
            FullyQualifiedName::new("pkg::task259::coherent-stale"),
        );
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].definition = alternate_definition_id(&fixture);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].contribution = alternate_contribution_id(&fixture);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].site = TypedSiteRef::Node(DEFINITION_OWNER);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].source_range.start -= 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].source_ordinal = 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].context = BindingContextId::new(0);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].recovery = SourcePredicateDefinitionRecovery::Degraded;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].spelling.push(' ');
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].definiens = SourceAtomicFormulaId::new(0);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.definitions.rows[0].origin = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 61, 122)),
            vec![4, 0, 8, 1],
        );
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].owner = SourcePredicateDefinitionId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].ordinal = 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].binding = BindingId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].written_type = SourceTypeApplicationId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].site = TypedSiteRef::Node(PARAMETER_Y);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].declaration_range.start += 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].context = BindingContextId::new(0);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].recovery = SourcePredicateDefinitionRecovery::Degraded;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.parameters.rows[0].spelling.push(' ');
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows[0].owner = SourcePredicateDefinitionId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows[0].ordinal = 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows[0].formula = SourceAtomicFormulaId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows[0].site = TypedSiteRef::Node(PREDICATE_OWNER);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows[0].source_range.start += 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows[0].context = BindingContextId::new(0);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows[0].recovery = SourcePredicateDefinitionRecovery::Degraded;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.guards.rows[0].spelling.push(' ');
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.properties.rows[0].owner = SourcePredicateDefinitionId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.properties.rows[0].ordinal = 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.properties.rows[0].site = TypedSiteRef::Node(PREDICATE_OWNER);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.properties.rows[0].source_range.start -= 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.properties.rows[0].justification =
            SourceAnchor::Range(range(fixture.source, 137, 158));
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.properties.rows[0].recovery = SourcePredicateDefinitionRecovery::Degraded;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.properties.rows[0].spelling.push(' ');
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.correctness.rows[0].owner = SourcePredicateDefinitionId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.correctness.rows[0].ordinal = 1;
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.resolver_identity.symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("coherent-stale"),
            FullyQualifiedName::new("pkg::task259::coherent-stale"),
        );
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.resolver_identity.definition = alternate_definition_id(&fixture);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.resolver_identity.contribution = alternate_contribution_id(&fixture);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.resolver_identity.origin = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 61, 122)),
            vec![4, 0, 8, 1],
        );
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.correctness.rows[0].property = SourcePredicatePropertyId::new(1);
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.correctness.rows[0].source_anchor =
            SourceAnchor::Range(range(fixture.source, 126, 159));
    });
    assert_handoff_rejected(&fixture, &projection, |handoff| {
        handoff.correctness.rows[0].obligation = InitialObligationId::new(1);
    });
}

#[test]
fn task_259_dependency_and_obligation_corruption_fails_closed() {
    let fixture = fixture();
    let projection = fixture.build(&InitialObligationTable::new());
    for mutate in [
        0usize, // Task 248 fingerprint
        1,      // Task 249 fingerprint
        2,      // Task 252 fingerprint
        3,      // Task 256 fingerprint
    ] {
        let mut handoff = projection.handoff().clone();
        match mutate {
            0 => handoff.source_context_fingerprint.push_str("stale"),
            1 => handoff.source_type_fingerprint.push_str("stale"),
            2 => handoff.source_term_fingerprint.push_str("stale"),
            3 => handoff.source_atomic_formula_fingerprint.push_str("stale"),
            _ => unreachable!(),
        }
        assert_eq!(
            validate_handoff(&fixture, &handoff, projection.initial_obligations()),
            Err(SourcePredicateDefinitionError::DependencyMismatch)
        );
    }

    let alternate_context = task248_context(
        fixture.source,
        &fixture.module,
        TypedSiteRef::Node(PREDICATE_OWNER),
        1,
        1,
    );
    assert_eq!(
        projection.handoff().validate_installation(
            fixture.source,
            &fixture.module,
            &alternate_context,
            &fixture.source_type,
            &fixture.source_term,
            &fixture.source_atomic_formula,
            projection.initial_obligations(),
            &fixture.arena,
        ),
        Err(SourcePredicateDefinitionError::DependencyMismatch)
    );

    let mut alternate_type_input = task249_input(fixture.source, fixture.module.clone());
    alternate_type_input.expressions[0].spelling = " set".to_owned();
    let alternate_type = SourceTypeProducer::build(
        alternate_type_input,
        fixture.source_context.binding_env(),
        &fixture.env,
        &fixture.arena,
    )
    .expect("alternate Task 249 dependency");
    assert_eq!(
        projection.handoff().validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &alternate_type,
            &fixture.source_term,
            &fixture.source_atomic_formula,
            projection.initial_obligations(),
            &fixture.arena,
        ),
        Err(SourcePredicateDefinitionError::DependencyMismatch)
    );

    let mut swapped_type_input = task249_input(fixture.source, fixture.module.clone());
    let expression_site = swapped_type_input.expressions[0].site.clone();
    swapped_type_input.expressions[0].site = swapped_type_input.expressions[0].head_site.clone();
    swapped_type_input.expressions[0].head_site = expression_site;
    let swapped_type = SourceTypeProducer::build(
        swapped_type_input,
        fixture.source_context.binding_env(),
        &fixture.env,
        &fixture.arena,
    )
    .expect("same-range Task 249 site swap remains lower-stage coherent");
    assert_eq!(
        SourcePredicateDefinitionProducer::build(
            fixture.input.clone(),
            &fixture.env,
            &fixture.source_context,
            &swapped_type,
            &fixture.source_term,
            &fixture.source_atomic_formula,
            projection.base_initial_obligations(),
            &fixture.arena,
        ),
        Err(SourcePredicateDefinitionError::InvalidParameter { index: 0 })
    );

    let mut alternate_term = fixture.source_term.clone();
    alternate_term.set_reference_use_ordinal_for_test(
        crate::source_term::SourcePrimaryTermReferenceId::new(0),
        usize::MAX,
    );
    assert_eq!(
        projection.handoff().validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &fixture.source_type,
            &alternate_term,
            &fixture.source_atomic_formula,
            projection.initial_obligations(),
            &fixture.arena,
        ),
        Err(SourcePredicateDefinitionError::DependencyMismatch)
    );

    let mut alternate_atomic = fixture.source_atomic_formula.clone();
    alternate_atomic.set_primary_term_fingerprint_for_test("stale Task 252".to_owned());
    assert_eq!(
        projection.handoff().validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &fixture.source_type,
            &fixture.source_term,
            &alternate_atomic,
            projection.initial_obligations(),
            &fixture.arena,
        ),
        Err(SourcePredicateDefinitionError::DependencyMismatch)
    );

    assert_obligation_rejected(&fixture, &projection, |row| {
        row.owner = TypedSiteRef::Node(PREDICATE_OWNER);
    });
    assert_obligation_rejected(&fixture, &projection, |row| {
        row.source_range = range(fixture.source, 126, 159);
    });
    assert_obligation_rejected(&fixture, &projection, |row| {
        row.kind = InitialObligationKind::Sethood;
    });
    assert_obligation_rejected(&fixture, &projection, |row| {
        row.status = InitialObligationStatus::Blocked;
    });
    assert_obligation_rejected(&fixture, &projection, |row| {
        row.goal = InitialObligationGoal::new("stale.goal");
    });
    assert_obligation_rejected(&fixture, &projection, |row| {
        row.provenance = InitialObligationProvenance::new("stale.provenance");
    });
    assert_obligation_rejected(&fixture, &projection, |row| {
        row.assumptions = vec![TypeFactId::new(0)];
    });

    assert_eq!(
        validate_handoff(
            &fixture,
            projection.handoff(),
            &InitialObligationTable::new()
        ),
        Err(SourcePredicateDefinitionError::InvalidObligation)
    );
    let mut extra = projection.initial_obligations().clone();
    extra.insert(baseline_draft(&fixture, 99));
    assert_eq!(
        validate_handoff(&fixture, projection.handoff(), &extra),
        Err(SourcePredicateDefinitionError::InvalidObligation)
    );
}

#[test]
fn task_259_typed_installation_is_transactional() {
    let fixture = fixture();
    let baseline = baseline_table(&fixture, 3);
    let projection = fixture.build(&baseline);
    assert_eq!(projection.base_initial_obligations(), &baseline);
    assert_eq!(projection.handoff(), projection.clone().handoff());
    assert_eq!(projection.initial_obligations().len(), 4);
    assert_eq!(
        projection
            .handoff()
            .correctness()
            .get(SourcePredicateCorrectnessId::new(0))
            .unwrap()
            .obligation(),
        InitialObligationId::new(baseline.len())
    );

    let (parts_base, parts_handoff, parts_complete) = projection.clone().into_parts();
    assert_eq!(parts_base, baseline);
    assert_eq!(parts_handoff, *projection.handoff());
    assert_eq!(parts_complete, *projection.initial_obligations());

    let installed = fixture
        .typed_ast(baseline.clone())
        .with_source_predicate_definition(projection.clone())
        .expect("atomic Task 259 installation");
    assert_eq!(
        &installed.initial_obligations().iter().collect::<Vec<_>>()[..baseline.len()],
        &baseline.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        installed.source_predicate_definition(),
        Some(projection.handoff())
    );
    assert_eq!(
        installed.initial_obligations(),
        projection.initial_obligations()
    );
    assert_eq!(installed.initial_obligations().len(), 4);

    let mut stale = baseline.clone();
    stale.insert(baseline_draft(&fixture, 3));
    assert_install_rejected_without_partial_replay(&fixture, stale, &projection);

    let missing = baseline_table(&fixture, 2);
    assert_install_rejected_without_partial_replay(&fixture, missing, &projection);

    let extra = baseline_table(&fixture, 4);
    assert_install_rejected_without_partial_replay(&fixture, extra, &projection);

    let mut reordered_rows = baseline
        .iter()
        .map(|(_, row)| row.clone())
        .collect::<Vec<_>>();
    reordered_rows.reverse();
    let reordered = obligation_table_from_rows(reordered_rows);
    assert_install_rejected_without_partial_replay(&fixture, reordered, &projection);

    let stale = obligation_table_from_rows(
        baseline
            .iter()
            .map(|(_, row)| {
                let mut row = row.clone();
                if row.id == InitialObligationId::new(1) {
                    row.goal = InitialObligationGoal::new("stale.baseline");
                }
                row
            })
            .collect(),
    );
    assert_install_rejected_without_partial_replay(&fixture, stale, &projection);

    let colliding = obligation_table_from_rows(
        baseline
            .iter()
            .map(|(_, row)| row.clone())
            .enumerate()
            .map(|(index, mut row)| {
                if index + 1 == baseline.len() {
                    row.owner = TypedSiteRef::Node(PROPERTY_OWNER);
                    row.source_range = range(fixture.source, 125, 159);
                    row.goal = InitialObligationGoal::new(
                        "source.definition.predicate.correctness:property=0",
                    );
                    row.provenance = InitialObligationProvenance::new(
                        "source.definition.predicate:definition=0:property=0",
                    );
                }
                row
            })
            .collect(),
    );
    assert_install_rejected_without_partial_replay(&fixture, colliding, &projection);

    assert!(matches!(
        installed
            .clone()
            .with_source_predicate_definition(projection.clone()),
        Err(TypedAstError::InvalidSourcePredicateDefinition)
    ));
    assert_eq!(
        installed.source_predicate_definition(),
        Some(projection.handoff())
    );
    assert_eq!(
        installed.initial_obligations(),
        projection.initial_obligations()
    );
}

#[test]
fn task_259_final_clone_debug_determinism_and_family_isolation() {
    let fixture = fixture();
    let baseline = baseline_table(&fixture, 2);
    let first = fixture.build(&baseline);
    let second = fixture.build(&baseline);
    assert_eq!(first, second);
    assert_eq!(
        first.handoff().debug_text(),
        first.handoff().clone().debug_text()
    );
    assert_eq!(first.handoff().debug_text(), second.handoff().debug_text());
    let debug = first.handoff().debug_text();
    assert!(debug.starts_with("source-predicate-definition-debug-v1\n"));
    assert!(debug.ends_with('\n'));
    assert_eq!(debug.matches("definition#0 ").count(), 1);
    assert_eq!(debug.matches("parameter#").count(), 2);
    assert_eq!(debug.matches("guard#0 ").count(), 1);
    assert_eq!(debug.matches("property#0 ").count(), 1);
    assert_eq!(debug.matches("correctness#0 ").count(), 1);

    let typed = fixture
        .typed_ast(baseline.clone())
        .with_source_predicate_definition(first)
        .unwrap();
    let typed_clone = typed.clone();
    assert_eq!(typed.debug_text(), typed_clone.debug_text());
    assert_eq!(
        typed
            .debug_text()
            .matches("source-predicate-definition-debug-v1\n")
            .count(),
        1
    );
    assert!(typed.source_application().is_none());
    assert!(typed.source_structure().is_none());
    assert!(typed.source_set_term().is_none());
    assert!(typed.source_composite_formula().is_none());
    assert!(typed.source_formula_composition().is_none());
    assert!(typed.source_condition_formula_composition().is_none());
    assert!(typed.source_predicate_chain_composition().is_none());
    assert!(typed.source_statement().is_none());
    assert!(typed.source_statement_references().is_none());
    assert!(typed.source_statement_witnesses().is_none());
    assert_eq!(fixture.env.symbols().len(), 2);
    assert_eq!(fixture.env.definitions().len(), 2);
    assert_eq!(
        typed
            .source_predicate_definition()
            .unwrap()
            .definitions()
            .len(),
        1
    );
    assert!(typed.facts().is_empty());
    assert!(typed.types().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.diagnostics().is_empty());

    let resolved = assemble_empty(&typed).expect("final Task 259 assembly");
    let resolved_clone = resolved.clone();
    assert_eq!(resolved, resolved_clone);
    assert_eq!(resolved.debug_text(), resolved_clone.debug_text());
    assert_eq!(
        resolved.source_predicate_definition(),
        typed.source_predicate_definition()
    );
    assert_eq!(
        resolved
            .debug_text()
            .matches("source-predicate-definition-debug-v1\n")
            .count(),
        1
    );
    assert!(resolved.source_application().is_none());
    assert!(resolved.source_structure().is_none());
    assert!(resolved.source_set_term().is_none());
    assert!(resolved.source_composite_formula().is_none());
    assert!(resolved.source_formula_composition().is_none());
    assert!(resolved.source_condition_formula_composition().is_none());
    assert!(resolved.source_predicate_chain_composition().is_none());
    assert!(resolved.source_statement().is_none());
    assert!(resolved.source_statement_references().is_none());
    assert!(resolved.source_statement_witnesses().is_none());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
    assert!(resolved.diagnostics().is_empty());

    let equivalent_typed = fixture
        .typed_ast(baseline.clone())
        .with_source_predicate_definition(second)
        .unwrap();
    let equivalent_resolved = assemble_empty(&equivalent_typed).unwrap();
    assert_eq!(resolved, equivalent_resolved);

    let mut missing_lower = typed.clone();
    missing_lower.remove_source_atomic_formula_for_test();
    assert_eq!(
        assemble_empty(&missing_lower),
        Err(ResolvedTypedAstError::InvalidSourcePredicateDefinition)
    );
}

fn build_with_input(
    fixture: &Fixture,
    input: SourcePredicateDefinitionHandoffInput,
    base: &InitialObligationTable,
) -> Result<SourcePredicateDefinitionProjection, SourcePredicateDefinitionError> {
    SourcePredicateDefinitionProducer::build(
        input,
        &fixture.env,
        &fixture.source_context,
        &fixture.source_type,
        &fixture.source_term,
        &fixture.source_atomic_formula,
        base,
        &fixture.arena,
    )
}

#[track_caller]
fn assert_input_rejected(
    fixture: &Fixture,
    base: &InitialObligationTable,
    mutate: impl FnOnce(&mut SourcePredicateDefinitionHandoffInput),
) {
    let mut input = fixture.input.clone();
    mutate(&mut input);
    assert!(
        build_with_input(fixture, input, base).is_err(),
        "independent input corruption must fail closed"
    );
}

fn validate_handoff(
    fixture: &Fixture,
    handoff: &SourcePredicateDefinitionHandoff,
    obligations: &InitialObligationTable,
) -> Result<(), SourcePredicateDefinitionError> {
    handoff.validate_installation(
        fixture.source,
        &fixture.module,
        &fixture.source_context,
        &fixture.source_type,
        &fixture.source_term,
        &fixture.source_atomic_formula,
        obligations,
        &fixture.arena,
    )
}

fn assert_handoff_rejected(
    fixture: &Fixture,
    projection: &SourcePredicateDefinitionProjection,
    mutate: impl FnOnce(&mut SourcePredicateDefinitionHandoff),
) {
    let mut handoff = projection.handoff().clone();
    mutate(&mut handoff);
    assert!(
        validate_handoff(fixture, &handoff, projection.initial_obligations()).is_err(),
        "independent immutable-row corruption must fail closed"
    );
}

fn assert_obligation_rejected(
    fixture: &Fixture,
    projection: &SourcePredicateDefinitionProjection,
    mutate: impl FnOnce(&mut InitialObligation),
) {
    let mut rows = projection
        .initial_obligations()
        .iter()
        .map(|(_, row)| row.clone())
        .collect::<Vec<_>>();
    mutate(rows.last_mut().expect("Task 259 obligation row"));
    let obligations = obligation_table_from_rows(rows);
    assert_eq!(
        validate_handoff(fixture, projection.handoff(), &obligations),
        Err(SourcePredicateDefinitionError::InvalidObligation)
    );
}

fn baseline_draft(fixture: &Fixture, index: usize) -> InitialObligationDraft {
    let owners = [PREDICATE_OWNER, GUARD_OWNER, DEFINITION_OWNER, MODULE_OWNER];
    let kinds = [
        InitialObligationKind::Sethood,
        InitialObligationKind::NonEmptiness,
        InitialObligationKind::Narrowing,
        InitialObligationKind::RegistrationCorrectness,
    ];
    InitialObligationDraft {
        kind: kinds[index % kinds.len()],
        owner: TypedSiteRef::Node(owners[index % owners.len()]),
        source_range: range(fixture.source, index, index + 1),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new(format!("baseline.goal.{index}")),
        provenance: InitialObligationProvenance::new(format!("baseline.provenance.{index}")),
        status: InitialObligationStatus::Pending,
    }
}

fn alternate_definition_id(fixture: &Fixture) -> DefinitionId {
    fixture
        .env
        .definitions()
        .iter()
        .find(|entry| entry.id().index() == 1)
        .expect("fixture property definition")
        .id()
}

fn alternate_contribution_id(fixture: &Fixture) -> SourceContributionId {
    let mut indexes = SymbolEnvIndexes::default();
    indexes.contributions.insert(
        fixture.module.clone(),
        ContributionKind::LocalSource {
            source_id: fixture.source,
        },
        SourceAnchor::Range(range(fixture.source, 0, 164)),
    );
    indexes.contributions.insert(
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("other")),
        ContributionKind::LocalSource {
            source_id: fixture.source,
        },
        SourceAnchor::Range(range(fixture.source, 0, 164)),
    )
}

fn baseline_table(fixture: &Fixture, count: usize) -> InitialObligationTable {
    let mut table = InitialObligationTable::new();
    for index in 0..count {
        let id = table.insert(baseline_draft(fixture, index));
        assert_eq!(id, InitialObligationId::new(index));
    }
    table
}

fn obligation_table_from_rows(rows: Vec<InitialObligation>) -> InitialObligationTable {
    let mut table = InitialObligationTable::new();
    for row in rows {
        table.insert(InitialObligationDraft {
            kind: row.kind,
            owner: row.owner,
            source_range: row.source_range,
            assumptions: row.assumptions,
            goal: row.goal,
            provenance: row.provenance,
            status: row.status,
        });
    }
    table
}

fn assert_install_rejected_without_partial_replay(
    fixture: &Fixture,
    current: InitialObligationTable,
    projection: &SourcePredicateDefinitionProjection,
) {
    let typed = fixture.typed_ast(current.clone());
    let untouched = typed.clone();
    assert!(matches!(
        typed
            .clone()
            .with_source_predicate_definition(projection.clone()),
        Err(TypedAstError::InvalidSourcePredicateDefinition)
    ));
    assert_eq!(typed, untouched);
    assert!(typed.source_predicate_definition().is_none());
    assert_eq!(typed.initial_obligations(), &current);
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
    let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task259"));
    let arena = task259_arena(source);
    let source_context = task248_context(source, &module, TypedSiteRef::Node(MODULE_OWNER), 0, 0);
    let (env, symbol, definition, contribution) = resolver_env(source, module.clone(), Some(2));
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
    let input = task259_input(source, module.clone(), symbol, definition, contribution);
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

fn task248_context(
    source: SourceId,
    module: &ModuleId,
    module_site: TypedSiteRef,
    local_scope_segment: u32,
    first_declaration_shift: usize,
) -> SourceBindingContextHandoff {
    let shell = definition_block_shell(source, module);
    let local_scope = LocalTermScope::new(vec![local_scope_segment]);
    let context_projection = SourceBindingContextProducer::build(SourceBindingContextInput {
        source_id: source,
        module_id: module.clone(),
        module_site,
        items: vec![SourceItemInput {
            shell,
            shell_ordinal: 0,
            role: SourceItemRole::DefinitionBlock,
            module_id: module.clone(),
            source_range: range(source, 0, 164),
            parent: None,
            visibility: SourceItemVisibility::Unspecified,
            site: TypedSiteRef::Node(DEFINITION_OWNER),
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
                let adjusted_start = if ordinal == 0 {
                    start - first_declaration_shift
                } else {
                    start
                };
                let declaration_range = range(source, adjusted_start, end);
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
    .expect("complete Task 248 Profile B");
    context_projection.into_handoff()
}

fn definition_block_shell(
    source: SourceId,
    module: &ModuleId,
) -> mizar_resolve::declarations::DeclarationShellId {
    let mut builder = SurfaceAstBuilder::new(source);
    let block = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source, 0, 164),
        Vec::new(),
    );
    let items = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source, 0, 164),
        vec![block],
    );
    let unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source, 0, 164),
        vec![items],
    );
    let root = builder.add_node(SurfaceNodeKind::Root, range(source, 0, 164), vec![unit]);
    let ast = builder.finish(Some(root), None);
    let shells = DeclarationShellCollector::new(&ast, module).collect();
    assert_eq!(shells.declarations().len(), 1);
    shells.declarations()[0].id()
}

fn resolver_env(
    source: SourceId,
    module: ModuleId,
    arity: Option<u32>,
) -> (SymbolEnv, SymbolId, DefinitionId, SourceContributionId) {
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 61, 122)),
        vec![4, 0, 8, 0],
    );
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new("task259_rel"),
        FullyQualifiedName::new("pkg::task259::task259_rel"),
    );
    let signature = SignatureShell::Opaque {
        schema: "parser-signature-v1".to_owned(),
        payload: "predicate:x task259_rel y".to_owned(),
    };
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 164)),
    );
    indexes.symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Predicate,
            NamespacePath::new("main"),
            "x task259_rel y",
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
        .with_notation_spelling("x task259_rel y")
        .with_signature(signature.clone()),
    );
    let definition = DefinitionShell::new(
        symbol.clone(),
        DefinitionKind::Predicate,
        origin,
        contribution,
    )
    .with_visibility(Visibility::Public)
    .with_notation_shape("x task259_rel y")
    .with_signature(signature);
    let definition = indexes.definitions.insert(match arity {
        Some(arity) => definition.with_arity(arity),
        None => definition,
    });
    let property_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 125, 159)),
        vec![4, 0, 17, 1],
    );
    let property_symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new("symmetry-property"),
        FullyQualifiedName::new("pkg::task259::symmetry-property"),
    );
    indexes.symbols.insert(
        SymbolEntry::new(
            property_symbol.clone(),
            SymbolKind::Attribute,
            NamespacePath::new("main"),
            "symmetry",
            property_origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    let property_definition = indexes.definitions.insert(
        DefinitionShell::new(
            property_symbol.clone(),
            DefinitionKind::Attribute,
            property_origin,
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
    indexes
        .contributions
        .add_symbol(contribution, property_symbol);
    indexes
        .contributions
        .add_definition(contribution, property_definition);
    (
        SymbolEnv::new(module, indexes),
        symbol,
        definition,
        contribution,
    )
}

fn task259_arena(source: SourceId) -> TypedArena {
    let mut builder = TypedArenaBuilder::new();
    push_node(
        &mut builder,
        "source.type.head",
        range(source, 22, 25),
        LocalTypeContextId::new(1),
        Vec::new(),
    );
    push_node(
        &mut builder,
        "source.type.expression",
        range(source, 22, 25),
        LocalTypeContextId::new(1),
        vec![TYPE_HEAD_X],
    );
    push_node(
        &mut builder,
        "source.definition.predicate.parameter",
        range(source, 17, 18),
        LocalTypeContextId::new(1),
        vec![TYPE_EXPRESSION_X],
    );
    push_node(
        &mut builder,
        "source.type.head",
        range(source, 38, 41),
        LocalTypeContextId::new(1),
        Vec::new(),
    );
    push_node(
        &mut builder,
        "source.type.expression",
        range(source, 38, 41),
        LocalTypeContextId::new(1),
        vec![TYPE_HEAD_Y],
    );
    push_node(
        &mut builder,
        "source.definition.predicate.parameter",
        range(source, 33, 34),
        LocalTypeContextId::new(1),
        vec![TYPE_EXPRESSION_Y],
    );
    for (id, start, end) in [
        (GUARD_LEFT, 52, 53),
        (GUARD_RIGHT, 56, 57),
        (DEFINIENS_LEFT, 116, 117),
        (DEFINIENS_RIGHT, 120, 121),
    ] {
        assert_eq!(
            push_node(
                &mut builder,
                "source.term.variable-reference",
                range(source, start, end),
                LocalTypeContextId::new(1),
                Vec::new(),
            ),
            id
        );
        if id == GUARD_RIGHT {
            assert_eq!(
                push_node(
                    &mut builder,
                    "source.formula.atomic.equality",
                    range(source, 52, 57),
                    LocalTypeContextId::new(1),
                    vec![GUARD_LEFT, GUARD_RIGHT],
                ),
                GUARD_FORMULA
            );
        }
        if id == DEFINIENS_RIGHT {
            assert_eq!(
                push_node(
                    &mut builder,
                    "source.formula.atomic.equality",
                    range(source, 116, 121),
                    LocalTypeContextId::new(1),
                    vec![DEFINIENS_LEFT, DEFINIENS_RIGHT],
                ),
                DEFINIENS_FORMULA
            );
        }
    }
    assert_eq!(
        push_node(
            &mut builder,
            "source.definition.predicate.guard",
            range(source, 45, 58),
            LocalTypeContextId::new(1),
            vec![GUARD_FORMULA],
        ),
        GUARD_OWNER
    );
    assert_eq!(
        push_node(
            &mut builder,
            "source.definition.predicate",
            range(source, 61, 122),
            LocalTypeContextId::new(1),
            vec![DEFINIENS_FORMULA],
        ),
        PREDICATE_OWNER
    );
    assert_eq!(
        push_node(
            &mut builder,
            "source.definition.predicate.property",
            range(source, 125, 159),
            LocalTypeContextId::new(1),
            Vec::new(),
        ),
        PROPERTY_OWNER
    );
    assert_eq!(
        push_node(
            &mut builder,
            "source.definition",
            range(source, 0, 164),
            LocalTypeContextId::new(1),
            vec![
                PARAMETER_X,
                PARAMETER_Y,
                GUARD_OWNER,
                PREDICATE_OWNER,
                PROPERTY_OWNER,
            ],
        ),
        DEFINITION_OWNER
    );
    assert_eq!(
        push_node(
            &mut builder,
            "source.module",
            range(source, 0, 164),
            LocalTypeContextId::new(0),
            vec![DEFINITION_OWNER],
        ),
        MODULE_OWNER
    );
    builder.finish(Some(MODULE_OWNER)).unwrap()
}

fn push_node(
    builder: &mut TypedArenaBuilder,
    kind: &str,
    source_range: SourceRange,
    context: LocalTypeContextId,
    children: Vec<TypedNodeId>,
) -> TypedNodeId {
    builder
        .push(
            TypedNode::new(kind, SourceAnchor::Range(source_range))
                .with_children(children)
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal)
                .with_links(TypedNodeLinks {
                    context: Some(context),
                    ..TypedNodeLinks::default()
                }),
        )
        .unwrap()
}

fn arena_with_node_context(
    arena: &TypedArena,
    node: TypedNodeId,
    context: LocalTypeContextId,
) -> TypedArena {
    let mut nodes = arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
    nodes[node.index()].links.context = Some(context);
    TypedArena::try_new(arena.root(), nodes).expect("coherent alternate owner context")
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
        (GUARD_LEFT, 52, 53, "x", BindingId::new(0)),
        (GUARD_RIGHT, 56, 57, "x", BindingId::new(0)),
        (DEFINIENS_LEFT, 116, 117, "x", BindingId::new(0)),
        (DEFINIENS_RIGHT, 120, 121, "y", BindingId::new(1)),
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
    let formulas = vec![
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(GUARD_FORMULA),
            source_range: range(source, 52, 57),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = x".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        },
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(DEFINIENS_FORMULA),
            source_range: range(source, 116, 121),
            source_ordinal: 1,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = y".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        },
    ];
    let edges = [
        (0, 0, SourceAtomicEdgeRole::BuiltinLeftOperand, 0),
        (0, 1, SourceAtomicEdgeRole::BuiltinRightOperand, 1),
        (1, 0, SourceAtomicEdgeRole::BuiltinLeftOperand, 2),
        (1, 1, SourceAtomicEdgeRole::BuiltinRightOperand, 3),
    ]
    .into_iter()
    .map(|(formula, ordinal, role, target)| SourceAtomicEdgeInput {
        formula: SourceAtomicFormulaId::new(formula),
        ordinal,
        role,
        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(target)),
    })
    .collect::<Vec<_>>();
    let requests = (0..4)
        .map(|edge| SourceAtomicRequestInput {
            formula: SourceAtomicFormulaId::new(edge / 2),
            ordinal: edge % 2,
            kind: SourceAtomicRequestKind::OperandExpectedType,
            edge: Some(SourceAtomicEdgeId::new(edge)),
            candidate: None,
            type_site: None,
            attribute: None,
        })
        .collect();
    SourceAtomicFormulaHandoffInput {
        source_id: source,
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

fn task259_input(
    source: SourceId,
    module: ModuleId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
) -> SourcePredicateDefinitionHandoffInput {
    SourcePredicateDefinitionHandoffInput {
        source_id: source,
        module_id: module,
        definitions: vec![SourcePredicateDefinitionInput {
            symbol,
            definition,
            contribution,
            site: TypedSiteRef::Node(PREDICATE_OWNER),
            source_range: range(source, 61, 122),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourcePredicateDefinitionRecovery::Normal,
            spelling: "pred Task259PredicateDefinition: x task259_rel y means x = y;".to_owned(),
            definiens: SourceAtomicFormulaId::new(1),
        }],
        parameters: vec![
            SourcePredicateParameterInput {
                owner: SourcePredicateDefinitionId::new(0),
                ordinal: 0,
                binding: BindingId::new(0),
                written_type: SourceTypeApplicationId::new(0),
                site: TypedSiteRef::Node(PARAMETER_X),
                source_range: range(source, 13, 26),
                declaration_range: range(source, 17, 18),
                context: BindingContextId::new(1),
                recovery: SourcePredicateDefinitionRecovery::Normal,
                spelling: "let x be set;".to_owned(),
            },
            SourcePredicateParameterInput {
                owner: SourcePredicateDefinitionId::new(0),
                ordinal: 1,
                binding: BindingId::new(1),
                written_type: SourceTypeApplicationId::new(1),
                site: TypedSiteRef::Node(PARAMETER_Y),
                source_range: range(source, 29, 42),
                declaration_range: range(source, 33, 34),
                context: BindingContextId::new(1),
                recovery: SourcePredicateDefinitionRecovery::Normal,
                spelling: "let y be set;".to_owned(),
            },
        ],
        guards: vec![SourcePredicateGuardInput {
            owner: SourcePredicateDefinitionId::new(0),
            ordinal: 0,
            formula: SourceAtomicFormulaId::new(0),
            site: TypedSiteRef::Node(GUARD_OWNER),
            source_range: range(source, 45, 58),
            context: BindingContextId::new(1),
            recovery: SourcePredicateDefinitionRecovery::Normal,
            spelling: "assume x = x;".to_owned(),
        }],
        properties: vec![SourcePredicatePropertyInput {
            owner: SourcePredicateDefinitionId::new(0),
            ordinal: 0,
            kind: SourcePredicatePropertyKind::Symmetry,
            site: TypedSiteRef::Node(PROPERTY_OWNER),
            source_range: range(source, 125, 159),
            justification: SourceAnchor::Range(range(source, 134, 158)),
            recovery: SourcePredicateDefinitionRecovery::Normal,
            spelling: "symmetry by computation(steps: 1);".to_owned(),
        }],
        correctness: vec![SourcePredicateCorrectnessInput {
            owner: SourcePredicateDefinitionId::new(0),
            property: SourcePredicatePropertyId::new(0),
            ordinal: 0,
            source_anchor: SourceAnchor::Range(range(source, 125, 159)),
        }],
    }
}

fn source_id() -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "59".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .unwrap()
}

fn other_source_id() -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "60".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    allocator.next_source_id(snapshot).unwrap();
    allocator.next_source_id(snapshot).unwrap()
}

#[test]
fn source_predicate_definition_accepts_coherent_nonuniform_layout_ranges() {
    use crate::binding_env::{BindingLookupResult, BindingLookupSite};

    let foreign_source = other_source_id();
    assert_ne!(foreign_source, source_id());
    for (case, accepted) in [
        ("normal", true),
        ("compact", true),
        ("parameter-slack", true),
        ("justification-slack", true),
        ("block-head-capacity", false),
        ("let-prefix-capacity", false),
        ("be-capacity", false),
        ("parameter-tail-capacity", false),
        ("guard-prefix-capacity", false),
        ("guard-tail-capacity", false),
        ("definition-prefix-capacity", false),
        ("definition-tail-capacity", false),
        ("property-prefix-capacity", false),
        ("justification-capacity", false),
        ("property-tail-capacity", false),
        ("block-tail-capacity", false),
        ("guard-equality-capacity", false),
        ("body-equality-capacity", false),
        ("declaration-x-width", false),
        ("declaration-y-width", false),
        ("set-x-width", false),
        ("set-y-width", false),
        ("guard-left-width", false),
        ("guard-right-width", false),
        ("body-left-width", false),
        ("body-right-width", false),
        ("guard-formula-start", false),
        ("guard-formula-end", false),
        ("body-formula-start", false),
        ("body-formula-end", false),
        ("parameter-overlap", false),
        ("guard-overlap", false),
        ("definition-overlap", false),
        ("property-overlap", false),
        ("definition-join", false),
        ("resolver-origin", false),
        ("contribution-join", false),
        ("declaration-join", false),
        ("written-type-join", false),
        ("type-site-join", false),
        ("binding-join", false),
        ("body-formula-link", false),
        ("guard-formula-link", false),
        ("correctness-join", false),
        ("definition-arena", false),
        ("property-arena", false),
        ("context-arena", false),
        ("parameter-arena", false),
        ("input-source", false),
        ("parameter-source", false),
        ("declaration-source", false),
        ("guard-source", false),
        ("definition-source", false),
        ("property-source", false),
        ("justification-source", false),
        ("correctness-source", false),
        ("parameter-empty", false),
        ("parameter-reversed", false),
        ("parameter-overflow", false),
        ("justification-empty", false),
        ("justification-reversed", false),
        ("justification-overflow", false),
        ("guard-empty", false),
        ("definition-reversed", false),
        ("property-overflow", false),
    ] {
        let mut fixture = fixture();
        let source = fixture.source;
        let module = fixture.module.clone();
        let relocate = |old: SourceRange| {
            let (start, end) = match (old.start, old.end) {
                (0, 164) => (7, 282),
                (13, 26) => (24, 41),
                (17, 18) => (29, 30),
                (22, 25) => (37, 40),
                (29, 42) => (50, 69),
                (33, 34) => (55, 56),
                (38, 41) => (65, 68),
                (45, 58) => (82, 103),
                (52, 53) => (91, 92),
                (56, 57) => (100, 101),
                (52, 57) => (91, 101),
                (61, 122) => (121, 204),
                (116, 117) => (185, 186),
                (120, 121) => (201, 202),
                (116, 121) => (185, 202),
                (125, 159) => (217, 270),
                (134, 158) => (230, 268),
                _ => return old,
            };
            let (start, end) = match (case, (old.start, old.end)) {
                ("compact", (0, 164)) => (7, 150),
                ("compact", (13, 26)) => (18, 31),
                ("compact", (17, 18)) => (22, 23),
                ("compact", (22, 25)) => (27, 30),
                ("compact", (29, 42)) => (31, 44),
                ("compact", (33, 34)) => (35, 36),
                ("compact", (38, 41)) => (40, 43),
                ("compact", (45, 58)) => (44, 55),
                ("compact", (52, 53)) => (51, 52),
                ("compact", (56, 57)) => (53, 54),
                ("compact", (52, 57)) => (51, 54),
                ("compact", (61, 122)) => (55, 113),
                ("compact", (116, 117)) => (109, 110),
                ("compact", (120, 121)) => (111, 112),
                ("compact", (116, 121)) => (109, 112),
                ("compact", (125, 159)) => (113, 146),
                ("compact", (134, 158)) => (122, 145),
                ("parameter-slack", (13, 26)) => (23, 42),
                ("justification-slack", (134, 158)) => (231, 269),
                ("block-head-capacity", (0, 164)) => (14, 282),
                ("let-prefix-capacity", (13, 26)) => (26, 41),
                ("be-capacity", (22, 25)) => (33, 36),
                ("parameter-tail-capacity", (13, 26)) => (24, 40),
                ("guard-prefix-capacity", (52, 53)) => (88, 89),
                ("guard-prefix-capacity", (52, 57)) => (88, 101),
                ("guard-tail-capacity", (45, 58)) => (82, 101),
                ("definition-prefix-capacity", (116, 117)) => (174, 175),
                ("definition-prefix-capacity", (116, 121)) => (174, 202),
                ("definition-tail-capacity", (61, 122)) => (121, 202),
                ("property-prefix-capacity", (134, 158)) => (225, 268),
                ("justification-capacity", (134, 158)) => (230, 252),
                ("property-tail-capacity", (125, 159)) => (217, 268),
                ("block-tail-capacity", (0, 164)) => (7, 273),
                ("guard-equality-capacity", (56, 57)) => (92, 93),
                ("guard-equality-capacity", (52, 57)) => (91, 93),
                ("body-equality-capacity", (120, 121)) => (186, 187),
                ("body-equality-capacity", (116, 121)) => (185, 187),
                ("declaration-x-width", (17, 18)) => (29, 31),
                ("declaration-y-width", (33, 34)) => (55, 57),
                ("set-x-width", (22, 25)) => (37, 39),
                ("set-y-width", (38, 41)) => (65, 67),
                ("guard-left-width", (52, 53)) => (91, 93),
                ("guard-right-width", (56, 57)) => (100, 102),
                ("guard-right-width", (52, 57)) => (91, 102),
                ("body-left-width", (116, 117)) => (185, 187),
                ("body-right-width", (120, 121)) => (201, 203),
                ("body-right-width", (116, 121)) => (185, 203),
                ("guard-formula-start", (52, 57)) => (90, 101),
                ("guard-formula-end", (52, 57)) => (91, 102),
                ("body-formula-start", (116, 121)) => (184, 202),
                ("body-formula-end", (116, 121)) => (185, 203),
                ("parameter-overlap", (29, 42)) => (40, 69),
                ("guard-overlap", (45, 58)) => (68, 103),
                ("definition-overlap", (61, 122)) => (102, 204),
                ("property-overlap", (125, 159)) => (203, 270),
                _ => (start, end),
            };
            range(source, start, end)
        };
        let arena = TypedArena::try_new(
            fixture.arena.root(),
            fixture
                .arena
                .iter()
                .map(|(id, node)| {
                    let mut node = node.clone();
                    if let SourceAnchor::Range(old) = &node.anchor {
                        node.anchor = SourceAnchor::Range(relocate(*old));
                    }
                    let mismatched_role = match case {
                        "definition-arena" => Some(PREDICATE_OWNER),
                        "property-arena" => Some(PROPERTY_OWNER),
                        "context-arena" => Some(DEFINITION_OWNER),
                        "parameter-arena" => Some(PARAMETER_X),
                        _ => None,
                    };
                    if mismatched_role == Some(id) {
                        let SourceAnchor::Range(range) = &mut node.anchor else {
                            unreachable!()
                        };
                        range.start += 1;
                    }
                    node
                })
                .collect(),
        )
        .expect("nonuniform public role anchors");
        let item = fixture
            .source_context
            .items()
            .get(crate::source_context::SourceItemId::new(0))
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
        for old in fixture.env.symbols().iter() {
            let SourceAnchor::Range(old_range) = old.origin().anchor() else {
                panic!("fixture origin must be a source range")
            };
            let mut origin_range = relocate(*old_range);
            if case == "resolver-origin" && old.kind() == SymbolKind::Predicate {
                origin_range.start += 1;
            }
            let origin = SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(origin_range),
                old.origin().structural_path().to_vec(),
            );
            let mut symbol = SymbolEntry::new(
                old.symbol().clone(),
                old.kind(),
                old.namespace().clone(),
                old.primary_spelling(),
                origin,
                contribution,
            )
            .with_visibility(old.visibility())
            .with_export_status(old.export_status());
            if let Some(notation) = old.notation_spelling() {
                symbol = symbol.with_notation_spelling(notation);
            }
            if let Some(signature) = old.signature() {
                symbol = symbol.with_signature(signature.clone());
            }
            indexes.symbols.insert(symbol);
            indexes
                .contributions
                .add_symbol(contribution, old.symbol().clone());
        }
        let mut definitions = fixture.env.definitions().iter().collect::<Vec<_>>();
        definitions.sort_by_key(|definition| definition.id());
        for old in definitions {
            let SourceAnchor::Range(old_range) = old.origin().anchor() else {
                panic!("fixture definition origin must be a range")
            };
            let mut origin_range = relocate(*old_range);
            if case == "resolver-origin" && old.kind() == DefinitionKind::Predicate {
                origin_range.start += 1;
            }
            let origin = SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(origin_range),
                old.origin().structural_path().to_vec(),
            );
            let mut definition =
                DefinitionShell::new(old.symbol().clone(), old.kind(), origin, contribution)
                    .with_visibility(old.visibility());
            if let Some(arity) = old.arity() {
                definition = definition.with_arity(arity);
            }
            if let Some(notation) = old.notation_shape() {
                definition = definition.with_notation_shape(notation);
            }
            if let Some(signature) = old.signature() {
                definition = definition.with_signature(signature.clone());
            }
            let id = indexes.definitions.insert(definition);
            assert_eq!(id, old.id());
            indexes.contributions.add_definition(contribution, id);
        }
        let env = SymbolEnv::new(module.clone(), indexes);
        let source_context = SourceBindingContextProducer::build(SourceBindingContextInput {
            source_id: source,
            module_id: module.clone(),
            module_site: TypedSiteRef::Node(MODULE_OWNER),
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
                .map(|(_, old)| {
                    let declaration_range = relocate(old.declaration_range);
                    SourceBindingSiteInput {
                        shell: item.shell,
                        context_owner: SourceBindingContextOwner::Shell(item.shell),
                        source_ordinal: old.source_ordinal,
                        spelling: old.spelling.clone(),
                        declaration_range,
                        written_type_range: relocate(old.written_type_range),
                        site: old.site.clone(),
                        role: SourceBindingSiteRole::DefinitionParameter {
                            local: LocalTermBinding::new(
                                old.spelling.clone(),
                                item.local_scope.clone().unwrap(),
                                declaration_range,
                                old.source_ordinal,
                            ),
                        },
                        recovery: BindingRecoveryState::Normal,
                    }
                })
                .collect(),
        })
        .unwrap_or_else(|error| panic!("{case}: SourceContext rejected {error:?}"))
        .into_complete()
        .expect("complete Context")
        .into_handoff();
        let mut type_input = task249_input(source, module.clone());
        for row in &mut type_input.expressions {
            row.source_range = relocate(row.source_range);
            row.head_range = relocate(row.head_range);
        }
        if case == "type-site-join" {
            let row = &mut type_input.expressions[0];
            std::mem::swap(&mut row.site, &mut row.head_site);
        }
        let source_type =
            SourceTypeProducer::build(type_input, source_context.binding_env(), &env, &arena)
                .unwrap_or_else(|error| panic!("{case}: Type rejected {error:?}"));
        let mut term_input = task252_input(source, module.clone());
        for row in &mut term_input.terms {
            row.source_range = relocate(row.source_range);
        }
        let source_term =
            SourcePrimaryTermProducer::build(term_input, source_context.binding_env(), &arena)
                .unwrap_or_else(|error| panic!("{case}: Primary rejected {error:?}"));
        assert_eq!(
            (source_term.terms().len(), source_term.references().len()),
            (4, 4)
        );
        for (id, reference) in source_term.references().iter() {
            let term = source_term.terms().get(reference.term()).unwrap();
            let old = fixture.source_term.references().get(id).unwrap();
            let binding = BindingId::new([0, 0, 0, 1][id.index()]);
            assert_eq!(reference.binding(), binding);
            assert_eq!(reference.use_ordinal(), old.use_ordinal());
            assert_eq!(reference.lexical_scope(), old.lexical_scope());
            assert_eq!(
                source_context.binding_env().lookup(&BindingLookupSite::new(
                    term.spelling(),
                    term.context(),
                    reference.lexical_scope().cloned(),
                    reference.use_ordinal()
                )),
                Ok(BindingLookupResult::Local(binding))
            );
        }
        let mut atomic_input = task256_input(source, module.clone());
        for row in &mut atomic_input.formulas {
            row.source_range = relocate(row.source_range);
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
            (2, 4, 4)
        );
        let old_definition = &fixture.input.definitions[0];
        let mut input = task259_input(
            source,
            module,
            old_definition.symbol.clone(),
            old_definition.definition,
            contribution,
        );
        input.definitions[0].source_range = relocate(input.definitions[0].source_range);
        for row in &mut input.parameters {
            row.source_range = relocate(row.source_range);
            row.declaration_range = relocate(row.declaration_range);
        }
        input.guards[0].source_range = relocate(input.guards[0].source_range);
        input.properties[0].source_range = relocate(input.properties[0].source_range);
        let SourceAnchor::Range(old_justification) = &input.properties[0].justification else {
            panic!("fixture justification must be a range")
        };
        input.properties[0].justification = SourceAnchor::Range(relocate(*old_justification));
        let SourceAnchor::Range(old_correctness) = &input.correctness[0].source_anchor else {
            panic!("fixture correctness must be a range")
        };
        input.correctness[0].source_anchor = SourceAnchor::Range(relocate(*old_correctness));
        match case {
            "definition-join" => input.definitions[0].source_range.start += 1,
            "declaration-join" => {
                input.parameters[0].declaration_range.start += 1;
                input.parameters[0].declaration_range.end += 1;
            }
            "written-type-join" => {
                input.parameters[0].written_type = SourceTypeApplicationId::new(1)
            }
            "binding-join" => input.parameters[1].binding = BindingId::new(0),
            "body-formula-link" => input.definitions[0].definiens = SourceAtomicFormulaId::new(0),
            "guard-formula-link" => input.guards[0].formula = SourceAtomicFormulaId::new(1),
            "correctness-join" => {
                input.correctness[0].source_anchor = SourceAnchor::Range(range(source, 218, 270))
            }
            "input-source" => input.source_id = foreign_source,
            "parameter-source" => input.parameters[0].source_range.source_id = foreign_source,
            "declaration-source" => {
                input.parameters[0].declaration_range.source_id = foreign_source
            }
            "guard-source" => input.guards[0].source_range.source_id = foreign_source,
            "definition-source" => input.definitions[0].source_range.source_id = foreign_source,
            "property-source" => input.properties[0].source_range.source_id = foreign_source,
            "justification-source" => {
                input.properties[0].justification =
                    SourceAnchor::Range(range(foreign_source, 230, 268))
            }
            "correctness-source" => {
                input.correctness[0].source_anchor =
                    SourceAnchor::Range(range(foreign_source, 217, 270))
            }
            "parameter-empty" => input.parameters[0].source_range = range(source, 24, 24),
            "parameter-reversed" => input.parameters[0].source_range = range(source, 41, 24),
            "parameter-overflow" => {
                input.parameters[0].source_range = range(source, 24, usize::MAX)
            }
            "justification-empty" => {
                input.properties[0].justification = SourceAnchor::Range(range(source, 230, 230))
            }
            "justification-reversed" => {
                input.properties[0].justification = SourceAnchor::Range(range(source, 268, 230))
            }
            "justification-overflow" => {
                input.properties[0].justification =
                    SourceAnchor::Range(range(source, 230, usize::MAX))
            }
            "guard-empty" => input.guards[0].source_range = range(source, 82, 82),
            "definition-reversed" => input.definitions[0].source_range = range(source, 204, 121),
            "property-overflow" => input.properties[0].source_range = range(source, usize::MAX, 0),
            _ => {}
        }
        fixture.env = env;
        fixture.source_context = source_context;
        fixture.source_type = source_type;
        fixture.source_term = source_term;
        fixture.source_atomic_formula = source_atomic_formula;
        fixture.arena = arena;
        fixture.input = input;
        let baseline = baseline_table(&fixture, 2);
        let untouched = baseline.clone();
        let result = build_with_input(&fixture, fixture.input.clone(), &baseline);
        if !accepted {
            assert!(
                result.is_err(),
                "{case}: invalid upper Predicate associations accepted"
            );
            assert_eq!(baseline, untouched, "{case}: baseline changed on error");
            continue;
        }
        let projection =
            result.unwrap_or_else(|error| panic!("{case}: valid Predicate rejected {error:?}"));
        assert_eq!(baseline, untouched);
        assert_eq!(projection.base_initial_obligations(), &baseline);
        assert_eq!(projection.initial_obligations().len(), baseline.len() + 1);
        for (id, row) in baseline.iter() {
            assert_eq!(projection.initial_obligations().get(id), Some(row));
        }
        let suffix_id = InitialObligationId::new(baseline.len());
        let suffix = projection.initial_obligations().get(suffix_id).unwrap();
        let property = projection
            .handoff()
            .properties()
            .get(SourcePredicatePropertyId::new(0))
            .unwrap();
        assert_eq!(suffix.owner, *property.site());
        assert_eq!(suffix.source_range, property.source_range());
        assert_eq!(
            suffix.kind,
            InitialObligationKind::PredicatePropertyCorrectness
        );
        assert_eq!(suffix.status, InitialObligationStatus::Pending);
        assert!(suffix.assumptions.is_empty());
        assert_eq!(
            suffix.goal.as_str(),
            "source.definition.predicate.correctness:property=0"
        );
        assert_eq!(
            suffix.provenance.as_str(),
            "source.definition.predicate:definition=0:property=0"
        );
        assert_eq!(
            projection
                .handoff()
                .correctness()
                .get(SourcePredicateCorrectnessId::new(0))
                .unwrap()
                .obligation(),
            suffix_id
        );
        assert_eq!(
            projection.handoff().source_context_fingerprint(),
            fixture.source_context.debug_text()
        );
        assert_eq!(
            projection.handoff().source_type_fingerprint(),
            fixture.source_type.debug_text()
        );
        assert_eq!(
            projection.handoff().source_term_fingerprint(),
            fixture.source_term.debug_text()
        );
        assert_eq!(
            projection.handoff().source_atomic_formula_fingerprint(),
            fixture.source_atomic_formula.debug_text()
        );
        let typed = fixture
            .typed_ast(baseline)
            .with_source_predicate_definition(projection.clone())
            .expect("nonuniform Predicate installation");
        assert_eq!(
            typed.initial_obligations(),
            projection.initial_obligations()
        );
        let resolved = assemble_empty(&typed).expect("nonuniform Predicate final assembly");
        assert_eq!(
            resolved.source_predicate_definition(),
            Some(projection.handoff())
        );
        assert_eq!(resolved.clone(), resolved);
        assert_eq!(assemble_empty(&typed.clone()).unwrap(), resolved);
    }
}
