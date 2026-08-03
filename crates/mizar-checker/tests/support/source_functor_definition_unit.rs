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
    source_application::{
        SourceFunctorApplicationForm, SourceFunctorApplicationHandoffInput,
        SourceFunctorApplicationInput, SourceFunctorApplicationKind,
        SourceFunctorApplicationProducer, SourceFunctorApplicationRecovery, SourceFunctorHeadSite,
    },
    source_atomic_formula::{
        SourceAtomicEdgeInput, SourceAtomicFormulaHandoffInput, SourceAtomicFormulaInput,
        SourceAtomicFormulaProducer, SourceAtomicRequestInput,
    },
    source_context::{
        SourceBindingContextInput, SourceBindingContextOwner, SourceBindingContextProducer,
        SourceBindingSiteInput, SourceItemInput,
    },
    source_set_term::{
        SourceSetRequestInput, SourceSetRequestKind, SourceSetTermHandoffInput, SourceSetTermInput,
        SourceSetTermKind, SourceSetTermProducer, SourceSetTermRecovery,
    },
    source_structure::{
        SourceStructureHandoffInput, SourceStructureProducer, SourceStructureRecovery,
        SourceStructureRequestInput, SourceStructureRequestKind, SourceStructureRootInput,
        SourceStructureTermInput, SourceStructureTermKind,
    },
    source_term::{
        SourcePrimaryTermHandoffInput, SourcePrimaryTermInput, SourcePrimaryTermProducer,
        SourcePrimaryTermReferenceInput,
    },
    source_type::{
        SourceTypeApplicationInput, SourceTypeDefinitionReturnExtensionInput,
        SourceTypeDefinitionReturnInput, SourceTypeDefinitionReturnProducer,
        SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeProducer,
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

const TYPE_HEAD_X: TypedNodeId = TypedNodeId::new(62);
const TYPE_EXPRESSION_X: TypedNodeId = TypedNodeId::new(63);
const PARAMETER_X: TypedNodeId = TypedNodeId::new(65);
const TYPE_HEAD_Y: TypedNodeId = TypedNodeId::new(66);
const TYPE_EXPRESSION_Y: TypedNodeId = TypedNodeId::new(67);
const PARAMETER_Y: TypedNodeId = TypedNodeId::new(69);
const GUARD_LEFT: TypedNodeId = TypedNodeId::new(70);
const GUARD_RIGHT: TypedNodeId = TypedNodeId::new(72);
const GUARD_FORMULA: TypedNodeId = TypedNodeId::new(75);
const GUARD_OWNER: TypedNodeId = TypedNodeId::new(77);
const RETURN_HEAD_EQUALS: TypedNodeId = TypedNodeId::new(79);
const RETURN_EQUALS: TypedNodeId = TypedNodeId::new(80);
const EQUALS_TERM: TypedNodeId = TypedNodeId::new(81);
const EQUALS_DEFINIENS: TypedNodeId = TypedNodeId::new(83);
const EQUALS_DEFINITION: TypedNodeId = TypedNodeId::new(84);
const RETURN_HEAD_MEANS: TypedNodeId = TypedNodeId::new(86);
const RETURN_MEANS: TypedNodeId = TypedNodeId::new(87);
const MEANS_LEFT: TypedNodeId = TypedNodeId::new(88);
const MEANS_RIGHT: TypedNodeId = TypedNodeId::new(90);
const MEANS_FORMULA: TypedNodeId = TypedNodeId::new(93);
const MEANS_DEFINIENS: TypedNodeId = TypedNodeId::new(94);
const MEANS_DEFINITION: TypedNodeId = TypedNodeId::new(95);
const EXISTENCE: TypedNodeId = TypedNodeId::new(99);
const UNIQUENESS: TypedNodeId = TypedNodeId::new(103);
const DEFINITION_BLOCK: TypedNodeId = TypedNodeId::new(104);
const MODULE_ROOT: TypedNodeId = TypedNodeId::new(107);

#[derive(Clone)]
struct Fixture {
    source: SourceId,
    module: ModuleId,
    env: SymbolEnv,
    source_context: SourceBindingContextHandoff,
    source_type: SourceTypeApplicationHandoff,
    source_term: SourcePrimaryTermHandoff,
    atomic_formulas: SourceAtomicFormulaHandoff,
    arena: TypedArena,
    input: SourceFunctorDefinitionHandoffInput,
}

struct OptionalLowerFixture {
    arena: TypedArena,
    application: SourceFunctorApplicationHandoff,
    structure: SourceStructureHandoff,
    set_term: SourceSetTermHandoff,
}

pub(crate) fn actual_definition_family_typed_asts_for_task261() -> (TypedAst, TypedAst) {
    let predicate = task259_actual::typed();
    let fixture = fixture();
    let projection = fixture.exact_projection(&InitialObligationTable::new());
    let functor = fixture
        .typed_ast(InitialObligationTable::new())
        .with_source_functor_definition(projection)
        .expect("actual Task 260 functor installation for Task 261 isolation");
    (predicate, functor)
}

impl Fixture {
    fn build(
        &self,
        base: &InitialObligationTable,
    ) -> Result<SourceFunctorDefinitionProjection, SourceFunctorDefinitionError> {
        SourceFunctorDefinitionProducer::build(
            self.input.clone(),
            &self.env,
            &self.source_context,
            &self.source_type,
            &self.source_term,
            None,
            None,
            None,
            Some(&self.atomic_formulas),
            base,
            &self.arena,
        )
    }

    fn exact_projection(&self, base: &InitialObligationTable) -> SourceFunctorDefinitionProjection {
        self.build(base).expect("exact Task 260 projection")
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
        .expect("Task 260 typed baseline")
        .with_source_term(self.source_term.clone())
        .expect("Task 252 installation")
        .with_source_atomic_formula(self.atomic_formulas.clone())
        .expect("Task 256 installation")
    }
}

#[test]
fn task_260_exact_functor_definition_payload_and_pending_obligations() {
    let fixture = fixture();
    let projection = fixture.exact_projection(&InitialObligationTable::new());
    let handoff = projection.handoff();
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
    assert_eq!(
        handoff.source_term_fingerprint(),
        fixture.source_term.debug_text()
    );
    assert_eq!(handoff.application_fingerprint(), None);
    assert_eq!(handoff.structure_fingerprint(), None);
    assert_eq!(handoff.set_term_fingerprint(), None);
    assert_eq!(
        handoff.atomic_formula_fingerprint(),
        Some(fixture.atomic_formulas.debug_text().as_str())
    );
    assert_eq!(handoff.definitions().len(), 2);
    assert_eq!(handoff.parameters().len(), 2);
    assert_eq!(handoff.guards().len(), 1);
    assert_eq!(handoff.definientia().len(), 2);
    assert_eq!(handoff.correctness().len(), 2);
    assert!(!handoff.definitions().is_empty());
    assert!(!handoff.parameters().is_empty());
    assert!(!handoff.guards().is_empty());
    assert!(!handoff.definientia().is_empty());
    assert!(!handoff.correctness().is_empty());
    assert_eq!(
        handoff
            .definitions()
            .iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>(),
        vec![
            SourceFunctorDefinitionId::new(0),
            SourceFunctorDefinitionId::new(1),
        ]
    );
    assert_eq!(
        handoff
            .parameters()
            .iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>(),
        vec![
            SourceFunctorParameterId::new(0),
            SourceFunctorParameterId::new(1),
        ]
    );
    assert_eq!(
        handoff
            .guards()
            .iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>(),
        vec![SourceFunctorGuardId::new(0)]
    );
    assert_eq!(
        handoff
            .definientia()
            .iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>(),
        vec![
            SourceFunctorDefiniensId::new(0),
            SourceFunctorDefiniensId::new(1),
        ]
    );
    assert_eq!(
        handoff
            .correctness()
            .iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>(),
        vec![
            SourceFunctorCorrectnessId::new(0),
            SourceFunctorCorrectnessId::new(1),
        ]
    );
    assert!(
        handoff
            .definitions()
            .get(SourceFunctorDefinitionId::new(2))
            .is_none()
    );
    assert!(
        handoff
            .parameters()
            .get(SourceFunctorParameterId::new(0))
            .is_some()
    );
    assert!(
        handoff
            .parameters()
            .get(SourceFunctorParameterId::new(2))
            .is_none()
    );
    assert!(handoff.guards().get(SourceFunctorGuardId::new(1)).is_none());
    assert!(
        handoff
            .definientia()
            .get(SourceFunctorDefiniensId::new(0))
            .is_some()
    );
    assert!(
        handoff
            .definientia()
            .get(SourceFunctorDefiniensId::new(2))
            .is_none()
    );
    assert!(
        handoff
            .correctness()
            .get(SourceFunctorCorrectnessId::new(0))
            .is_some()
    );
    assert!(
        handoff
            .correctness()
            .get(SourceFunctorCorrectnessId::new(2))
            .is_none()
    );

    for index in 0..2 {
        let row = handoff
            .definitions()
            .get(SourceFunctorDefinitionId::new(index))
            .expect("definition row");
        assert_eq!(row.id(), SourceFunctorDefinitionId::new(index));
        assert_eq!(row.symbol(), &fixture.input.definitions[index].symbol);
        assert_eq!(row.definition().index(), index);
        assert_eq!(row.contribution().index(), 0);
        assert_eq!(row.source_ordinal(), index);
        assert_eq!(row.context(), BindingContextId::new(1));
        assert_eq!(row.recovery(), SourceFunctorDefinitionRecovery::Normal);
        assert_eq!(row.return_type(), SourceTypeDefinitionReturnId::new(index));
        assert_eq!(row.definiens(), SourceFunctorDefiniensId::new(index));
        assert_eq!(row.origin().source_id(), fixture.source);
        assert_eq!(row.origin().module_id(), &fixture.module);
        assert_eq!(
            row.origin().anchor(),
            &SourceAnchor::Range(row.source_range())
        );
        assert!(row.origin().import_edge().is_none());
        assert!(!row.origin().is_recovered());
    }
    let equals = handoff
        .definitions()
        .get(SourceFunctorDefinitionId::new(0))
        .unwrap();
    assert_eq!(equals.site(), &TypedSiteRef::Node(EQUALS_DEFINITION));
    assert_eq!(equals.source_range(), range(fixture.source, 61, 118));
    assert_eq!(equals.style(), SourceFunctorDefinitionStyle::Equals);
    assert_eq!(
        equals.spelling(),
        "func Task260EqualsDef: task260_equals(x) -> set equals x;"
    );
    assert_eq!(equals.origin().structural_path(), &[4, 0, 9, 0]);
    let means = handoff
        .definitions()
        .get(SourceFunctorDefinitionId::new(1))
        .unwrap();
    assert_eq!(means.site(), &TypedSiteRef::Node(MEANS_DEFINITION));
    assert_eq!(means.source_range(), range(fixture.source, 121, 179));
    assert_eq!(means.style(), SourceFunctorDefinitionStyle::Means);
    assert_eq!(
        means.spelling(),
        "func Task260MeansDef: task260_means(y) -> set means x = y;"
    );
    assert_eq!(means.origin().structural_path(), &[4, 0, 9, 1]);

    let parameters = handoff.parameters().iter().collect::<Vec<_>>();
    assert_eq!(parameters.len(), 2);
    assert_eq!(parameters[0].0, SourceFunctorParameterId::new(0));
    assert_eq!(parameters[0].1.id(), SourceFunctorParameterId::new(0));
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
        SourceFunctorDefinitionRecovery::Normal
    );
    assert_eq!(parameters[0].1.spelling(), "let x be set;");
    assert_eq!(parameters[1].0, SourceFunctorParameterId::new(1));
    assert_eq!(parameters[1].1.id(), SourceFunctorParameterId::new(1));
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
    assert_eq!(parameters[1].1.spelling(), "let y be set;");
    assert_eq!(parameters[1].1.context(), BindingContextId::new(1));
    assert_eq!(
        parameters[1].1.recovery(),
        SourceFunctorDefinitionRecovery::Normal
    );

    let guard = handoff.guards().get(SourceFunctorGuardId::new(0)).unwrap();
    assert_eq!(guard.id(), SourceFunctorGuardId::new(0));
    assert_eq!(guard.ordinal(), 0);
    assert_eq!(guard.formula(), SourceAtomicFormulaId::new(0));
    assert_eq!(guard.site(), &TypedSiteRef::Node(GUARD_OWNER));
    assert_eq!(guard.source_range(), range(fixture.source, 45, 58));
    assert_eq!(guard.context(), BindingContextId::new(1));
    assert_eq!(guard.recovery(), SourceFunctorDefinitionRecovery::Normal);
    assert_eq!(guard.spelling(), "assume x = x;");

    let definientia = handoff.definientia().iter().collect::<Vec<_>>();
    assert_eq!(definientia[0].0, SourceFunctorDefiniensId::new(0));
    assert_eq!(definientia[0].1.id(), SourceFunctorDefiniensId::new(0));
    assert_eq!(definientia[0].1.owner(), SourceFunctorDefinitionId::new(0));
    assert_eq!(definientia[0].1.ordinal(), 0);
    assert_eq!(
        definientia[0].1.target(),
        SourceFunctorDefiniensTarget::Primary(SourcePrimaryTermId::new(2))
    );
    assert_eq!(
        definientia[0].1.site(),
        &TypedSiteRef::Node(EQUALS_DEFINIENS)
    );
    assert_eq!(
        definientia[0].1.source_range(),
        range(fixture.source, 116, 117)
    );
    assert_eq!(definientia[0].1.context(), BindingContextId::new(1));
    assert_eq!(
        definientia[0].1.recovery(),
        SourceFunctorDefinitionRecovery::Normal
    );
    assert_eq!(definientia[0].1.spelling(), "x");
    assert_eq!(definientia[1].0, SourceFunctorDefiniensId::new(1));
    assert_eq!(definientia[1].1.id(), SourceFunctorDefiniensId::new(1));
    assert_eq!(definientia[1].1.owner(), SourceFunctorDefinitionId::new(1));
    assert_eq!(definientia[1].1.ordinal(), 1);
    assert_eq!(
        definientia[1].1.target(),
        SourceFunctorDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(1))
    );
    assert_eq!(
        definientia[1].1.site(),
        &TypedSiteRef::Node(MEANS_DEFINIENS)
    );
    assert_eq!(
        definientia[1].1.source_range(),
        range(fixture.source, 173, 178)
    );
    assert_eq!(definientia[1].1.spelling(), "x = y");
    assert_eq!(definientia[1].1.context(), BindingContextId::new(1));
    assert_eq!(
        definientia[1].1.recovery(),
        SourceFunctorDefinitionRecovery::Normal
    );

    let correctness = handoff.correctness().iter().collect::<Vec<_>>();
    for (index, (id, row)) in correctness.iter().enumerate() {
        assert_eq!(*id, SourceFunctorCorrectnessId::new(index));
        assert_eq!(row.id(), SourceFunctorCorrectnessId::new(index));
        assert_eq!(row.owner(), SourceFunctorDefinitionId::new(1));
        assert_eq!(row.ordinal(), index);
        assert_eq!(row.obligation(), InitialObligationId::new(index));
        assert_eq!(row.recovery(), SourceFunctorDefinitionRecovery::Normal);
    }
    assert_eq!(
        correctness[0].1.kind(),
        SourceFunctorCorrectnessKind::Existence
    );
    assert_eq!(correctness[0].1.site(), &TypedSiteRef::Node(EXISTENCE));
    assert_eq!(
        correctness[0].1.source_range(),
        range(fixture.source, 182, 217)
    );
    assert_eq!(
        correctness[0].1.justification(),
        &SourceAnchor::Range(range(fixture.source, 192, 216))
    );
    assert_eq!(
        correctness[0].1.spelling(),
        "existence by computation(steps: 1);"
    );
    assert_eq!(
        correctness[1].1.kind(),
        SourceFunctorCorrectnessKind::Uniqueness
    );
    assert_eq!(correctness[1].1.site(), &TypedSiteRef::Node(UNIQUENESS));
    assert_eq!(
        correctness[1].1.source_range(),
        range(fixture.source, 220, 256)
    );
    assert_eq!(
        correctness[1].1.justification(),
        &SourceAnchor::Range(range(fixture.source, 231, 255))
    );
    assert_eq!(
        correctness[1].1.spelling(),
        "uniqueness by computation(steps: 1);"
    );

    assert!(projection.base_initial_obligations().is_empty());
    let obligations = projection.initial_obligations();
    assert_eq!(obligations.len(), 2);
    assert_obligation(
        obligations.get(InitialObligationId::new(0)).unwrap(),
        InitialObligationId::new(0),
        InitialObligationKind::FunctorExistence,
        TypedSiteRef::Node(EXISTENCE),
        range(fixture.source, 182, 217),
        "source.definition.functor.correctness:definition=1:existence",
        "source.definition.functor:definition=1:correctness=0",
    );
    assert_obligation(
        obligations.get(InitialObligationId::new(1)).unwrap(),
        InitialObligationId::new(1),
        InitialObligationKind::FunctorUniqueness,
        TypedSiteRef::Node(UNIQUENESS),
        range(fixture.source, 220, 256),
        "source.definition.functor.correctness:definition=1:uniqueness",
        "source.definition.functor:definition=1:correctness=1",
    );
    assert_eq!(handoff.debug_text(), expected_debug(&fixture, 0));
    let typed = fixture
        .typed_ast(InitialObligationTable::new())
        .with_source_functor_definition(projection.clone())
        .expect("exact serializer installation");
    assert!(typed.debug_text().contains("kind=functor_existence"));
    assert!(typed.debug_text().contains("kind=functor_uniqueness"));
    assert_error_serializers();
    assert_eq!(
        target_text(SourceFunctorDefiniensTarget::Primary(
            SourcePrimaryTermId::new(2)
        )),
        "primary:2"
    );
    assert_eq!(
        target_text(SourceFunctorDefiniensTarget::Application(
            SourceFunctorApplicationId::new(3)
        )),
        "application:3"
    );
    assert_eq!(
        target_text(SourceFunctorDefiniensTarget::Structure(
            SourceStructureTermId::new(4)
        )),
        "structure:4"
    );
    assert_eq!(
        target_text(SourceFunctorDefiniensTarget::SetTerm(SourceSetTermId::new(
            5
        ))),
        "set-term:5"
    );
    assert_eq!(
        target_text(SourceFunctorDefiniensTarget::AtomicFormula(
            SourceAtomicFormulaId::new(1)
        )),
        "atomic-formula:1"
    );
}

#[test]
fn task_260_independent_row_and_field_corruption_fails_closed() {
    let fixture = fixture();
    let baseline = InitialObligationTable::new();

    assert_input_error(
        &fixture,
        &baseline,
        |input| input.source_id = other_source_id(),
        SourceFunctorDefinitionError::SourceIdentityMismatch,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task260.other"))
        },
        SourceFunctorDefinitionError::SourceIdentityMismatch,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].symbol = input.definitions[1].symbol.clone(),
        SourceFunctorDefinitionError::InvalidResolverDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].symbol = input.definitions[0].symbol.clone(),
        SourceFunctorDefinitionError::InvalidResolverDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].definition = input.definitions[1].definition,
        SourceFunctorDefinitionError::InvalidResolverDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].definition = input.definitions[0].definition,
        SourceFunctorDefinitionError::InvalidResolverDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].contribution = other_contribution_id(input.source_id),
        SourceFunctorDefinitionError::InvalidResolverDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].contribution = other_contribution_id(input.source_id),
        SourceFunctorDefinitionError::InvalidResolverDefinition { index: 1 },
    );

    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.definitions.pop();
        },
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.definitions.push(input.definitions[1].clone());
        },
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.parameters.pop();
        },
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters.push(input.parameters[1].clone()),
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.guards.clear(),
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.guards.push(input.guards[0].clone()),
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia.clear(),
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia.push(input.definientia[1].clone()),
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia.swap(0, 1),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.correctness.pop();
        },
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness.push(input.correctness[1].clone()),
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].source_ordinal = 1,
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].source_range.end -= 1,
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].site = TypedSiteRef::Node(MEANS_DEFINITION),
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].context = BindingContextId::new(0),
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].recovery = SourceFunctorDefinitionRecovery::Degraded,
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].spelling.push('!'),
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].style = SourceFunctorDefinitionStyle::Means,
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].return_type = SourceTypeDefinitionReturnId::new(1),
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[0].definiens = SourceFunctorDefiniensId::new(1),
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].source_ordinal = 0,
        SourceFunctorDefinitionError::InvalidDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].source_range.start += 1,
        SourceFunctorDefinitionError::InvalidDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].site = TypedSiteRef::Node(EQUALS_DEFINITION),
        SourceFunctorDefinitionError::InvalidDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].context = BindingContextId::new(0),
        SourceFunctorDefinitionError::InvalidDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].recovery = SourceFunctorDefinitionRecovery::Degraded,
        SourceFunctorDefinitionError::InvalidDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].spelling.push('!'),
        SourceFunctorDefinitionError::InvalidDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].style = SourceFunctorDefinitionStyle::Equals,
        SourceFunctorDefinitionError::InvalidDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].return_type = SourceTypeDefinitionReturnId::new(0),
        SourceFunctorDefinitionError::InvalidDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definitions[1].definiens = SourceFunctorDefiniensId::new(0),
        SourceFunctorDefinitionError::InvalidDefinition { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters.swap(0, 1),
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[0].ordinal = 1,
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[0].binding = BindingId::new(1),
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[0].written_type = SourceTypeApplicationId::new(1),
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[0].site = TypedSiteRef::Node(PARAMETER_Y),
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[0].source_range.end -= 1,
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[0].declaration_range.end += 1,
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[0].context = BindingContextId::new(0),
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[0].recovery = SourceFunctorDefinitionRecovery::Degraded,
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[0].spelling.clear(),
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[1].ordinal = 0,
        SourceFunctorDefinitionError::InvalidParameter { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[1].binding = BindingId::new(0),
        SourceFunctorDefinitionError::InvalidParameter { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[1].written_type = SourceTypeApplicationId::new(0),
        SourceFunctorDefinitionError::InvalidParameter { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[1].site = TypedSiteRef::Node(PARAMETER_X),
        SourceFunctorDefinitionError::InvalidParameter { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[1].source_range.start += 1,
        SourceFunctorDefinitionError::InvalidParameter { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[1].declaration_range.start += 1,
        SourceFunctorDefinitionError::InvalidParameter { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[1].context = BindingContextId::new(0),
        SourceFunctorDefinitionError::InvalidParameter { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[1].recovery = SourceFunctorDefinitionRecovery::Degraded,
        SourceFunctorDefinitionError::InvalidParameter { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.parameters[1].spelling.clear(),
        SourceFunctorDefinitionError::InvalidParameter { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.guards[0].ordinal = 1,
        SourceFunctorDefinitionError::InvalidGuard { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.guards[0].formula = SourceAtomicFormulaId::new(1),
        SourceFunctorDefinitionError::InvalidGuard { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.guards[0].site = TypedSiteRef::Node(EQUALS_DEFINITION),
        SourceFunctorDefinitionError::InvalidGuard { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.guards[0].source_range.end -= 1,
        SourceFunctorDefinitionError::InvalidGuard { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.guards[0].context = BindingContextId::new(0),
        SourceFunctorDefinitionError::InvalidGuard { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.guards[0].recovery = SourceFunctorDefinitionRecovery::Degraded,
        SourceFunctorDefinitionError::InvalidGuard { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.guards[0].spelling.clear(),
        SourceFunctorDefinitionError::InvalidGuard { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[0].owner = SourceFunctorDefinitionId::new(1),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[0].ordinal = 1,
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.definientia[0].target =
                SourceFunctorDefiniensTarget::Primary(SourcePrimaryTermId::new(1))
        },
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.definientia[0].target =
                SourceFunctorDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(0))
        },
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[0].site = TypedSiteRef::Node(MEANS_DEFINIENS),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[0].source_range.end += 1,
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[0].context = BindingContextId::new(0),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[0].recovery = SourceFunctorDefinitionRecovery::Degraded,
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[0].spelling.clear(),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    for target in [
        SourceFunctorDefiniensTarget::Application(SourceFunctorApplicationId::new(0)),
        SourceFunctorDefiniensTarget::Structure(SourceStructureTermId::new(0)),
        SourceFunctorDefiniensTarget::SetTerm(SourceSetTermId::new(0)),
    ] {
        assert_input_error(
            &fixture,
            &baseline,
            |input| input.definientia[0].target = target,
            SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
        );
    }
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[1].owner = SourceFunctorDefinitionId::new(0),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[1].ordinal = 0,
        SourceFunctorDefinitionError::InvalidDefiniens { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.definientia[1].target =
                SourceFunctorDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(0))
        },
        SourceFunctorDefinitionError::InvalidDefiniens { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[1].site = TypedSiteRef::Node(EQUALS_DEFINIENS),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[1].source_range.start += 1,
        SourceFunctorDefinitionError::InvalidDefiniens { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[1].context = BindingContextId::new(0),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[1].recovery = SourceFunctorDefinitionRecovery::Degraded,
        SourceFunctorDefinitionError::InvalidDefiniens { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.definientia[1].spelling.clear(),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness.swap(0, 1),
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[0].owner = SourceFunctorDefinitionId::new(0),
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[0].ordinal = 1,
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[0].kind = SourceFunctorCorrectnessKind::Uniqueness,
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[0].site = TypedSiteRef::Node(UNIQUENESS),
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[0].source_range.end -= 1,
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.correctness[0].justification =
                SourceAnchor::Range(range(input.source_id, 193, 216))
        },
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[0].recovery = SourceFunctorDefinitionRecovery::Degraded,
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[0].spelling.clear(),
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[1].owner = SourceFunctorDefinitionId::new(0),
        SourceFunctorDefinitionError::InvalidCorrectness { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[1].ordinal = 0,
        SourceFunctorDefinitionError::InvalidCorrectness { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[1].kind = SourceFunctorCorrectnessKind::Existence,
        SourceFunctorDefinitionError::InvalidCorrectness { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[1].site = TypedSiteRef::Node(EXISTENCE),
        SourceFunctorDefinitionError::InvalidCorrectness { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[1].source_range.start += 1,
        SourceFunctorDefinitionError::InvalidCorrectness { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| {
            input.correctness[1].justification =
                SourceAnchor::Range(range(input.source_id, 232, 255))
        },
        SourceFunctorDefinitionError::InvalidCorrectness { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[1].recovery = SourceFunctorDefinitionRecovery::Degraded,
        SourceFunctorDefinitionError::InvalidCorrectness { index: 1 },
    );
    assert_input_error(
        &fixture,
        &baseline,
        |input| input.correctness[1].spelling.clear(),
        SourceFunctorDefinitionError::InvalidCorrectness { index: 1 },
    );

    let projection = fixture.exact_projection(&baseline);
    let mut corrupt = projection.handoff().clone();
    corrupt.definitions.rows[0].id = SourceFunctorDefinitionId::new(1);
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidDefinition { index: 0 },
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.parameters.rows[0].id = SourceFunctorParameterId::new(1);
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidParameter { index: 0 },
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.guards.rows[0].id = SourceFunctorGuardId::new(1);
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidGuard { index: 0 },
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.definientia.rows[0].id = SourceFunctorDefiniensId::new(1);
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidDefiniens { index: 0 },
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.correctness.rows[0].id = SourceFunctorCorrectnessId::new(1);
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidCorrectness { index: 0 },
    );
}

#[test]
fn task_260_dependency_and_obligation_corruption_fails_closed() {
    let fixture = fixture();
    let baseline = InitialObligationTable::new();
    assert!(matches!(
        SourceFunctorDefinitionProducer::build(
            fixture.input.clone(),
            &fixture.env,
            &fixture.source_context,
            &fixture.source_type,
            &fixture.source_term,
            None,
            None,
            None,
            None,
            &baseline,
            &fixture.arena,
        ),
        Err(SourceFunctorDefinitionError::DependencyMismatch)
    ));

    let optional = optional_lower_fixture(&fixture);
    for (
        target,
        applications,
        structures,
        set_terms,
        owner,
        owner_kind,
        owner_range,
        arena_error,
    ) in [
        (
            SourceFunctorDefiniensTarget::Application(SourceFunctorApplicationId::new(0)),
            Some(&optional.application),
            None,
            None,
            TypedNodeId::new(40),
            "source.term.functor-application.inline",
            range(fixture.source, 100, 104),
            SourceFunctorDefinitionError::InvalidArenaOwnership,
        ),
        (
            SourceFunctorDefiniensTarget::Structure(SourceStructureTermId::new(0)),
            None,
            Some(&optional.structure),
            None,
            TypedNodeId::new(42),
            "source.term.structure.constructor",
            range(fixture.source, 110, 114),
            SourceFunctorDefinitionError::DependencyMismatch,
        ),
        (
            SourceFunctorDefiniensTarget::SetTerm(SourceSetTermId::new(0)),
            None,
            None,
            Some(&optional.set_term),
            TypedNodeId::new(43),
            "source.term.set.enumeration",
            range(fixture.source, 130, 134),
            SourceFunctorDefinitionError::DependencyMismatch,
        ),
    ] {
        let mut deferred_input = fixture.input.clone();
        deferred_input.definientia[0].target = target;
        assert_eq!(
            SourceFunctorDefinitionProducer::build(
                deferred_input.clone(),
                &fixture.env,
                &fixture.source_context,
                &fixture.source_type,
                &fixture.source_term,
                applications,
                structures,
                set_terms,
                Some(&fixture.atomic_formulas),
                &baseline,
                &optional.arena,
            ),
            Err(SourceFunctorDefinitionError::DependencyMismatch)
        );
        let invalid_id = match target {
            SourceFunctorDefiniensTarget::Application(_) => {
                SourceFunctorDefiniensTarget::Application(SourceFunctorApplicationId::new(1))
            }
            SourceFunctorDefiniensTarget::Structure(_) => {
                SourceFunctorDefiniensTarget::Structure(SourceStructureTermId::new(1))
            }
            SourceFunctorDefiniensTarget::SetTerm(_) => {
                SourceFunctorDefiniensTarget::SetTerm(SourceSetTermId::new(1))
            }
            SourceFunctorDefiniensTarget::Primary(_)
            | SourceFunctorDefiniensTarget::AtomicFormula(_) => unreachable!(),
        };
        let mut invalid_id_input = deferred_input.clone();
        invalid_id_input.definientia[0].target = invalid_id;
        assert_eq!(
            SourceFunctorDefinitionProducer::build(
                invalid_id_input,
                &fixture.env,
                &fixture.source_context,
                &fixture.source_type,
                &fixture.source_term,
                applications,
                structures,
                set_terms,
                Some(&fixture.atomic_formulas),
                &baseline,
                &optional.arena,
            ),
            Err(SourceFunctorDefinitionError::InvalidDefiniens { index: 0 })
        );
        let corrupt_arena = arena_with_source_node(
            &optional.arena,
            owner,
            "source.unowned",
            owner_range,
            LocalTypeContextId::new(1),
        );
        assert_eq!(
            SourceFunctorDefinitionProducer::build(
                deferred_input,
                &fixture.env,
                &fixture.source_context,
                &fixture.source_type,
                &fixture.source_term,
                applications,
                structures,
                set_terms,
                Some(&fixture.atomic_formulas),
                &baseline,
                &corrupt_arena,
            ),
            Err(arena_error),
            "optional owner {owner_kind} must be revalidated"
        );
    }

    let projection = fixture.exact_projection(&baseline);
    for mutate in [
        |handoff: &mut SourceFunctorDefinitionHandoff| handoff.source_context_fingerprint.push('!'),
        |handoff: &mut SourceFunctorDefinitionHandoff| handoff.source_type_fingerprint.push('!'),
        |handoff: &mut SourceFunctorDefinitionHandoff| handoff.source_term_fingerprint.push('!'),
        |handoff: &mut SourceFunctorDefinitionHandoff| {
            handoff
                .atomic_formula_fingerprint
                .as_mut()
                .unwrap()
                .push('!')
        },
    ] {
        let mut corrupt = projection.handoff().clone();
        mutate(&mut corrupt);
        assert_handoff_error(
            &fixture,
            &corrupt,
            projection.initial_obligations(),
            SourceFunctorDefinitionError::DependencyMismatch,
        );
    }
    for family in 0..3 {
        let mut corrupt = projection.handoff().clone();
        match family {
            0 => corrupt.application_fingerprint = Some("forged application".to_owned()),
            1 => corrupt.structure_fingerprint = Some("forged structure".to_owned()),
            2 => corrupt.set_term_fingerprint = Some("forged set term".to_owned()),
            _ => unreachable!(),
        }
        assert_handoff_error(
            &fixture,
            &corrupt,
            projection.initial_obligations(),
            SourceFunctorDefinitionError::DependencyMismatch,
        );
    }
    let mut corrupt = projection.handoff().clone();
    corrupt.resolver_identities[0].definition = fixture.input.definitions[1].definition;
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidResolverDefinition { index: 0 },
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.definitions.rows[0].origin = SemanticOrigin::new(
        fixture.source,
        fixture.module.clone(),
        SourceAnchor::Range(range(fixture.source, 61, 118)),
        vec![4, 0, 9, 1],
    );
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidResolverDefinition { index: 0 },
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.definitions.rows[1].origin = SemanticOrigin::new(
        fixture.source,
        fixture.module.clone(),
        SourceAnchor::Range(range(fixture.source, 121, 179)),
        vec![4, 0, 9, 0],
    );
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidResolverDefinition { index: 1 },
    );

    let relocated_term_arena = arena_with_source_node(
        &fixture.arena,
        TypedNodeId::new(71),
        "source.term.variable-reference",
        range(fixture.source, 56, 57),
        LocalTypeContextId::new(1),
    );
    let mut relocated_term_input = task252_input(fixture.source, fixture.module.clone());
    relocated_term_input.terms[1].site = TypedSiteRef::Node(TypedNodeId::new(71));
    let relocated_term = SourcePrimaryTermProducer::build(
        relocated_term_input,
        fixture.source_context.binding_env(),
        &relocated_term_arena,
    )
    .expect("coherent relocated Task 252 site");
    let relocated_term_atomic = SourceAtomicFormulaProducer::build(
        task256_input(fixture.source, fixture.module.clone()),
        fixture.source_context.binding_env(),
        &fixture.env,
        &relocated_term,
        None,
        None,
        None,
        &relocated_term_arena,
    )
    .expect("coherent Task 256 over relocated Task 252 site");
    assert_eq!(
        SourceFunctorDefinitionProducer::build(
            fixture.input.clone(),
            &fixture.env,
            &fixture.source_context,
            &fixture.source_type,
            &relocated_term,
            None,
            None,
            None,
            Some(&relocated_term_atomic),
            &baseline,
            &relocated_term_arena,
        ),
        Err(SourceFunctorDefinitionError::DependencyMismatch)
    );

    let relocated_formula_arena = arena_with_source_node(
        &fixture.arena,
        TypedNodeId::new(74),
        "source.formula.atomic.equality",
        range(fixture.source, 52, 57),
        LocalTypeContextId::new(1),
    );
    let mut relocated_formula_input = task256_input(fixture.source, fixture.module.clone());
    relocated_formula_input.formulas[0].site = TypedSiteRef::Node(TypedNodeId::new(74));
    let relocated_formula = SourceAtomicFormulaProducer::build(
        relocated_formula_input,
        fixture.source_context.binding_env(),
        &fixture.env,
        &fixture.source_term,
        None,
        None,
        None,
        &relocated_formula_arena,
    )
    .expect("coherent relocated Task 256 site");
    assert_eq!(
        SourceFunctorDefinitionProducer::build(
            fixture.input.clone(),
            &fixture.env,
            &fixture.source_context,
            &fixture.source_type,
            &fixture.source_term,
            None,
            None,
            None,
            Some(&relocated_formula),
            &baseline,
            &relocated_formula_arena,
        ),
        Err(SourceFunctorDefinitionError::DependencyMismatch)
    );

    let drafts = obligation_drafts(projection.initial_obligations());
    for row_index in 0..2 {
        for mutate in [
            mutate_obligation_kind as fn(&mut InitialObligationDraft),
            mutate_obligation_owner,
            mutate_obligation_range,
            mutate_obligation_assumption,
            mutate_obligation_goal,
            mutate_obligation_provenance,
            mutate_obligation_status,
        ] {
            let mut rows = drafts.clone();
            mutate(&mut rows[row_index]);
            let table = table_from_drafts(rows);
            assert_handoff_error(
                &fixture,
                projection.handoff(),
                &table,
                SourceFunctorDefinitionError::InvalidObligation,
            );
        }
    }
    let mut rows = drafts.clone();
    rows.swap(0, 1);
    assert_handoff_error(
        &fixture,
        projection.handoff(),
        &table_from_drafts(rows),
        SourceFunctorDefinitionError::InvalidObligation,
    );
    let mut rows = drafts.clone();
    rows.pop();
    assert_handoff_error(
        &fixture,
        projection.handoff(),
        &table_from_drafts(rows),
        SourceFunctorDefinitionError::InvalidObligation,
    );
    let mut rows = drafts.clone();
    rows.push(unrelated_draft(fixture.source));
    assert_handoff_error(
        &fixture,
        projection.handoff(),
        &table_from_drafts(rows),
        SourceFunctorDefinitionError::InvalidObligation,
    );
    for kind in [
        InitialObligationKind::FunctorExistence,
        InitialObligationKind::FunctorUniqueness,
    ] {
        let mut rows = drafts.clone();
        let mut extra = unrelated_draft(fixture.source);
        extra.kind = kind;
        rows.push(extra);
        assert_handoff_error(
            &fixture,
            projection.handoff(),
            &table_from_drafts(rows),
            SourceFunctorDefinitionError::InvalidObligation,
        );
        assert_orphan_obligation_rejected(&fixture, kind);
    }
    let mut corrupt = projection.handoff().clone();
    corrupt.correctness.rows[0].obligation = InitialObligationId::new(1);
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidObligation,
    );
    let mut corrupt = projection.handoff().clone();
    corrupt.correctness.rows[1].obligation = InitialObligationId::new(0);
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::InvalidObligation,
    );

    for owner in [
        PARAMETER_X,
        PARAMETER_Y,
        GUARD_OWNER,
        EQUALS_DEFINIENS,
        MEANS_DEFINIENS,
        EQUALS_DEFINITION,
        MEANS_DEFINITION,
        EXISTENCE,
        UNIQUENESS,
    ] {
        let corrupt_arena =
            arena_with_node_context(&fixture.arena, owner, LocalTypeContextId::new(0));
        assert_eq!(
            SourceFunctorDefinitionProducer::build(
                fixture.input.clone(),
                &fixture.env,
                &fixture.source_context,
                &fixture.source_type,
                &fixture.source_term,
                None,
                None,
                None,
                Some(&fixture.atomic_formulas),
                &baseline,
                &corrupt_arena,
            ),
            Err(SourceFunctorDefinitionError::InvalidArenaOwnership)
        );
    }

    for kind in [
        InitialObligationKind::FunctorExistence,
        InitialObligationKind::FunctorUniqueness,
        InitialObligationKind::PredicatePropertyCorrectness,
    ] {
        let mut forbidden = InitialObligationTable::new();
        let mut draft = unrelated_draft(fixture.source);
        draft.kind = kind;
        forbidden.insert(draft);
        assert!(matches!(
            fixture.build(&forbidden),
            Err(SourceFunctorDefinitionError::InvalidObligation)
        ));
    }
}

#[test]
fn task_260_typed_installation_is_transactional() {
    let fixture = fixture();
    let base = unrelated_baseline(fixture.source);
    let projection = fixture.exact_projection(&base);
    assert_eq!(projection.base_initial_obligations(), &base);
    assert_eq!(projection.initial_obligations().len(), base.len() + 2);
    assert_eq!(
        projection
            .initial_obligations()
            .get(InitialObligationId::new(0)),
        base.get(InitialObligationId::new(0))
    );
    assert_eq!(
        projection.correctness_ids(),
        [InitialObligationId::new(1), InitialObligationId::new(2)]
    );
    let (retained, handoff, final_table) = projection.clone().into_parts();
    assert_eq!(retained, base);
    assert_eq!(&handoff, projection.handoff());
    assert_eq!(&final_table, projection.initial_obligations());

    let baseline_typed = fixture.typed_ast(base.clone());
    let untouched = baseline_typed.clone();
    let typed = baseline_typed
        .clone()
        .with_source_functor_definition(projection.clone())
        .expect("one-shot Task 260 installation");
    assert_eq!(
        typed.source_functor_definition(),
        Some(projection.handoff())
    );
    assert_eq!(
        typed.initial_obligations(),
        projection.initial_obligations()
    );
    assert_eq!(baseline_typed, untouched);
    assert!(baseline_typed.source_functor_definition().is_none());
    assert!(matches!(
        typed
            .clone()
            .with_source_functor_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    ));

    let stale = fixture.typed_ast(InitialObligationTable::new());
    let stale_copy = stale.clone();
    assert!(matches!(
        stale.with_source_functor_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    ));
    assert!(stale_copy.source_functor_definition().is_none());

    let same_length_stale = table_from_drafts(vec![InitialObligationDraft {
        kind: InitialObligationKind::Sethood,
        owner: TypedSiteRef::Node(GUARD_LEFT),
        source_range: range(fixture.source, 52, 53),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new("same-length stale goal"),
        provenance: InitialObligationProvenance::new("same-length:stale"),
        status: InitialObligationStatus::Pending,
    }]);
    let same_length_typed = fixture.typed_ast(same_length_stale);
    let same_length_copy = same_length_typed.clone();
    assert!(matches!(
        same_length_typed.with_source_functor_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    ));
    assert!(same_length_copy.source_functor_definition().is_none());

    let ordered = ordered_baseline(fixture.source);
    let ordered_projection = fixture.exact_projection(&ordered);
    let reordered = table_from_drafts(obligation_drafts(&ordered).into_iter().rev().collect());
    let reordered_typed = fixture.typed_ast(reordered);
    let reordered_copy = reordered_typed.clone();
    assert!(matches!(
        reordered_typed.with_source_functor_definition(ordered_projection),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    ));
    assert!(reordered_copy.source_functor_definition().is_none());

    assert!(matches!(
        TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: Some(fixture.source_context.clone()),
            source_type: Some(fixture.source_type.clone()),
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: fixture.source_context.local_contexts().clone(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: projection.initial_obligations().clone(),
            diagnostics: TypeDiagnosticTable::new(),
        }),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    ));
}

#[test]
fn task_260_final_clone_debug_determinism_and_predicate_isolation() {
    let fixture = fixture();
    let legacy = fixture.typed_ast(InitialObligationTable::new());
    assert!(legacy.source_functor_definition().is_none());
    assert!(
        !legacy
            .debug_text()
            .contains("source-functor-definition-debug-v1")
    );
    let legacy_final = assemble_empty(&legacy).expect("legacy final assembly");
    assert!(legacy_final.source_functor_definition().is_none());
    assert!(
        !legacy_final
            .debug_text()
            .contains("source-functor-definition-debug-v1")
    );

    let projection = fixture.exact_projection(&InitialObligationTable::new());
    let typed = legacy
        .with_source_functor_definition(projection.clone())
        .expect("Task 260 installation");
    assert_eq!(
        typed
            .debug_text()
            .matches("source-functor-definition-debug-v1")
            .count(),
        1
    );
    assert_eq!(
        typed.source_functor_definition().unwrap().debug_text(),
        expected_debug(&fixture, 0)
    );
    let resolved = assemble_empty(&typed).expect("Task 260 final assembly");
    assert_eq!(
        resolved.source_functor_definition(),
        typed.source_functor_definition()
    );
    assert_eq!(
        resolved
            .debug_text()
            .matches("source-functor-definition-debug-v1")
            .count(),
        1
    );
    assert_eq!(resolved.clone(), resolved);
    assert_eq!(typed.source_id(), fixture.source);
    assert_eq!(typed.module_id(), &fixture.module);
    assert!(typed.source_context().is_some());
    assert!(typed.source_type().is_some());
    assert!(typed.source_term().is_some());
    assert!(typed.source_atomic_formula().is_some());
    assert!(typed.source_functor_definition().is_some());
    assert!(typed.source_attribute().is_none());
    assert!(typed.source_evidence().is_none());
    assert!(typed.source_application().is_none());
    assert!(typed.source_structure().is_none());
    assert!(typed.source_set_term().is_none());
    assert!(typed.source_predicate_definition().is_none());
    assert!(typed.source_composite_formula().is_none());
    assert!(typed.source_formula_composition().is_none());
    assert!(typed.source_condition_formula_composition().is_none());
    assert!(typed.source_predicate_chain_composition().is_none());
    assert!(typed.source_statement().is_none());
    assert!(typed.source_statement_references().is_none());
    assert!(typed.source_statement_witnesses().is_none());
    assert!(typed.types().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.diagnostics().is_empty());
    assert_eq!(typed.initial_obligations().len(), 2);
    assert!(typed.initial_obligations().iter().all(|(_, row)| {
        row.status == InitialObligationStatus::Pending
            && row.assumptions.is_empty()
            && matches!(
                row.kind,
                InitialObligationKind::FunctorExistence | InitialObligationKind::FunctorUniqueness
            )
    }));

    assert_eq!(resolved.source_id(), fixture.source);
    assert_eq!(resolved.module_id(), &fixture.module);
    assert!(resolved.source_context().is_some());
    assert!(resolved.source_type().is_some());
    assert!(resolved.source_term().is_some());
    assert!(resolved.source_atomic_formula().is_some());
    assert!(resolved.source_functor_definition().is_some());
    assert!(resolved.source_attribute().is_none());
    assert!(resolved.source_evidence().is_none());
    assert!(resolved.source_application().is_none());
    assert!(resolved.source_structure().is_none());
    assert!(resolved.source_set_term().is_none());
    assert!(resolved.source_predicate_definition().is_none());
    assert!(resolved.source_composite_formula().is_none());
    assert!(resolved.source_formula_composition().is_none());
    assert!(resolved.source_condition_formula_composition().is_none());
    assert!(resolved.source_predicate_chain_composition().is_none());
    assert!(resolved.source_statement().is_none());
    assert!(resolved.source_statement_references().is_none());
    assert!(resolved.source_statement_witnesses().is_none());
    assert!(resolved.expr_metadata().is_empty());
    assert!(resolved.collection_candidates().is_empty());
    assert!(resolved.expanded_candidates().is_empty());
    assert!(resolved.template_expansions().is_empty());
    assert!(resolved.viable_candidates().is_empty());
    assert!(resolved.viability_decisions().is_empty());
    assert!(resolved.specificity_graphs().is_empty());
    assert!(resolved.resolved_overloads().is_empty());
    assert!(resolved.inserted_coercions().is_empty());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.diagnostics().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());

    let combined_debug = format!("{}{}", typed.debug_text(), resolved.debug_text());
    for deferred in [
        "accepted",
        "activated",
        "core-ir",
        "control-flow",
        "verification-condition",
    ] {
        assert!(
            !combined_debug.to_ascii_lowercase().contains(deferred),
            "Task 260 leaked deferred semantic marker {deferred}"
        );
    }

    let mut missing_projection = projection.clone();
    let mut missing_rows = obligation_drafts(missing_projection.initial_obligations());
    missing_rows.pop();
    missing_projection.initial_obligations = table_from_drafts(missing_rows);
    assert!(matches!(
        fixture
            .typed_ast(InitialObligationTable::new())
            .with_source_functor_definition(missing_projection),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    ));
    for kind in [
        InitialObligationKind::FunctorExistence,
        InitialObligationKind::FunctorUniqueness,
    ] {
        assert_orphan_obligation_rejected(&fixture, kind);
        let mut extra_projection = projection.clone();
        let mut extra = unrelated_draft(fixture.source);
        extra.kind = kind;
        extra_projection.initial_obligations.insert(extra);
        assert!(matches!(
            fixture
                .typed_ast(InitialObligationTable::new())
                .with_source_functor_definition(extra_projection),
            Err(TypedAstError::InvalidSourceFunctorDefinition)
        ));
    }

    let mut corrupt_dependency_final = typed.clone();
    let mut corrupt_dependency_handoff = projection.handoff().clone();
    corrupt_dependency_handoff
        .source_type_fingerprint
        .push_str(" stale-final");
    corrupt_dependency_final.inject_source_functor_definition_for_test(corrupt_dependency_handoff);
    assert!(matches!(
        assemble_empty(&corrupt_dependency_final),
        Err(ResolvedTypedAstError::InvalidSourceFunctorDefinition)
    ));

    let mut corrupt_obligation_rows = obligation_drafts(projection.initial_obligations());
    corrupt_obligation_rows[0].goal = InitialObligationGoal::new("forged final goal");
    let mut corrupt_obligation_final = typed.clone();
    corrupt_obligation_final
        .replace_initial_obligations_for_test(table_from_drafts(corrupt_obligation_rows));
    assert!(matches!(
        assemble_empty(&corrupt_obligation_final),
        Err(ResolvedTypedAstError::InvalidSourceFunctorDefinition)
    ));

    let mut missing_obligation_final = typed.clone();
    let mut final_rows = obligation_drafts(projection.initial_obligations());
    final_rows.pop();
    missing_obligation_final.replace_initial_obligations_for_test(table_from_drafts(final_rows));
    assert!(matches!(
        assemble_empty(&missing_obligation_final),
        Err(ResolvedTypedAstError::InvalidSourceFunctorDefinition)
    ));

    let mut extra_obligation_final = typed.clone();
    let mut final_rows = obligation_drafts(projection.initial_obligations());
    let mut extra = unrelated_draft(fixture.source);
    extra.kind = InitialObligationKind::FunctorExistence;
    final_rows.push(extra);
    extra_obligation_final.replace_initial_obligations_for_test(table_from_drafts(final_rows));
    assert!(matches!(
        assemble_empty(&extra_obligation_final),
        Err(ResolvedTypedAstError::InvalidSourceFunctorDefinition)
    ));

    for kind in [
        InitialObligationKind::FunctorExistence,
        InitialObligationKind::FunctorUniqueness,
    ] {
        let mut orphan_final = fixture.typed_ast(InitialObligationTable::new());
        let mut orphan = unrelated_draft(fixture.source);
        orphan.kind = kind;
        orphan_final.replace_initial_obligations_for_test(table_from_drafts(vec![orphan]));
        assert!(matches!(
            assemble_empty(&orphan_final),
            Err(ResolvedTypedAstError::InvalidSourceFunctorDefinition)
        ));
    }

    let mut predicate_baseline = InitialObligationTable::new();
    let mut predicate = unrelated_draft(fixture.source);
    predicate.kind = InitialObligationKind::PredicatePropertyCorrectness;
    predicate_baseline.insert(predicate);
    assert!(matches!(
        fixture.build(&predicate_baseline),
        Err(SourceFunctorDefinitionError::InvalidObligation)
    ));

    let predicate_typed = task259_actual::typed();
    assert!(predicate_typed.source_predicate_definition().is_some());
    assert!(predicate_typed.source_functor_definition().is_none());
    let predicate_final = assemble_empty(&predicate_typed).expect("actual Task 259 final assembly");
    assert!(predicate_final.source_predicate_definition().is_some());
    assert!(predicate_final.source_functor_definition().is_none());
    let predicate_copy = predicate_typed.clone();
    assert!(matches!(
        predicate_typed.with_source_functor_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    ));
    assert!(predicate_copy.source_predicate_definition().is_some());
    assert!(predicate_copy.source_functor_definition().is_none());

    let mut dual_final = typed.clone();
    dual_final.inject_source_predicate_definition_for_test(
        predicate_copy
            .source_predicate_definition()
            .expect("actual Task 259 handoff")
            .clone(),
    );
    assert!(matches!(
        assemble_empty(&dual_final),
        Err(ResolvedTypedAstError::InvalidSourceFunctorDefinition)
    ));

    let mut corrupt = projection.handoff().clone();
    corrupt.correctness.rows.pop();
    assert_handoff_error(
        &fixture,
        &corrupt,
        projection.initial_obligations(),
        SourceFunctorDefinitionError::UnsupportedTaskShape,
    );

    let mut corrupt_projection = projection.clone();
    corrupt_projection.handoff.definitions.rows[1].origin = SemanticOrigin::new(
        fixture.source,
        fixture.module.clone(),
        SourceAnchor::Range(range(fixture.source, 121, 179)),
        vec![4, 0, 9, 0],
    );
    let baseline = fixture.typed_ast(InitialObligationTable::new());
    let baseline_copy = baseline.clone();
    assert!(matches!(
        baseline.with_source_functor_definition(corrupt_projection),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    ));
    assert_eq!(baseline_copy.source_functor_definition(), None);
}

trait ProjectionTestExt {
    fn correctness_ids(&self) -> [InitialObligationId; 2];
}

impl ProjectionTestExt for SourceFunctorDefinitionProjection {
    fn correctness_ids(&self) -> [InitialObligationId; 2] {
        [
            self.handoff.correctness.rows[0].obligation,
            self.handoff.correctness.rows[1].obligation,
        ]
    }
}

fn assert_input_error(
    fixture: &Fixture,
    baseline: &InitialObligationTable,
    mutate: impl FnOnce(&mut SourceFunctorDefinitionHandoffInput),
    expected: SourceFunctorDefinitionError,
) {
    let mut input = fixture.input.clone();
    mutate(&mut input);
    let baseline_copy = baseline.clone();
    assert_eq!(
        SourceFunctorDefinitionProducer::build(
            input,
            &fixture.env,
            &fixture.source_context,
            &fixture.source_type,
            &fixture.source_term,
            None,
            None,
            None,
            Some(&fixture.atomic_formulas),
            baseline,
            &fixture.arena,
        ),
        Err(expected)
    );
    assert_eq!(baseline, &baseline_copy);
}

fn assert_handoff_error(
    fixture: &Fixture,
    handoff: &SourceFunctorDefinitionHandoff,
    obligations: &InitialObligationTable,
    expected: SourceFunctorDefinitionError,
) {
    assert_eq!(
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.source_context,
            &fixture.source_type,
            &fixture.source_term,
            None,
            None,
            None,
            Some(&fixture.atomic_formulas),
            obligations,
            &fixture.arena,
        ),
        Err(expected)
    );
}

fn assert_orphan_obligation_rejected(fixture: &Fixture, kind: InitialObligationKind) {
    let mut initial_obligations = InitialObligationTable::new();
    let mut orphan = unrelated_draft(fixture.source);
    orphan.kind = kind;
    initial_obligations.insert(orphan);
    assert!(matches!(
        TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: Some(fixture.source_context.clone()),
            source_type: Some(fixture.source_type.clone()),
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: fixture.source_context.local_contexts().clone(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations,
            diagnostics: TypeDiagnosticTable::new(),
        }),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    ));
}

fn assert_obligation(
    row: &InitialObligation,
    id: InitialObligationId,
    kind: InitialObligationKind,
    owner: TypedSiteRef,
    source_range: SourceRange,
    goal: &str,
    provenance: &str,
) {
    assert_eq!(row.id, id);
    assert_eq!(row.kind, kind);
    assert_eq!(row.owner, owner);
    assert_eq!(row.source_range, source_range);
    assert!(row.assumptions.is_empty());
    assert_eq!(row.goal.as_str(), goal);
    assert_eq!(row.provenance.as_str(), provenance);
    assert_eq!(row.status, InitialObligationStatus::Pending);
}

fn assert_error_serializers() {
    let cases = [
        (
            SourceFunctorDefinitionError::SourceIdentityMismatch,
            "functor-definition source identity mismatch",
        ),
        (
            SourceFunctorDefinitionError::DependencyMismatch,
            "functor-definition dependency mismatch",
        ),
        (
            SourceFunctorDefinitionError::InvalidResolverDefinition { index: 7 },
            "invalid functor resolver definition 7",
        ),
        (
            SourceFunctorDefinitionError::InvalidDefinition { index: 7 },
            "invalid source functor definition 7",
        ),
        (
            SourceFunctorDefinitionError::InvalidParameter { index: 7 },
            "invalid source functor parameter 7",
        ),
        (
            SourceFunctorDefinitionError::InvalidGuard { index: 7 },
            "invalid source functor guard 7",
        ),
        (
            SourceFunctorDefinitionError::InvalidDefiniens { index: 7 },
            "invalid source functor definiens 7",
        ),
        (
            SourceFunctorDefinitionError::InvalidCorrectness { index: 7 },
            "invalid source functor correctness 7",
        ),
        (
            SourceFunctorDefinitionError::InvalidObligation,
            "invalid functor correctness obligation",
        ),
        (
            SourceFunctorDefinitionError::InvalidArenaOwnership,
            "invalid functor-definition typed-arena ownership",
        ),
        (
            SourceFunctorDefinitionError::UnsupportedTaskShape,
            "unsupported functor-definition task shape",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}

fn target_text(target: SourceFunctorDefiniensTarget) -> String {
    let mut output = String::new();
    write_target(&mut output, target);
    output
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

fn mutate_obligation_kind(row: &mut InitialObligationDraft) {
    row.kind = InitialObligationKind::Sethood;
}
fn mutate_obligation_owner(row: &mut InitialObligationDraft) {
    row.owner = TypedSiteRef::Node(MEANS_DEFINITION);
}
fn mutate_obligation_range(row: &mut InitialObligationDraft) {
    row.source_range.end -= 1;
}
fn mutate_obligation_assumption(row: &mut InitialObligationDraft) {
    row.assumptions.push(TypeFactId::new(0));
}
fn mutate_obligation_goal(row: &mut InitialObligationDraft) {
    row.goal = InitialObligationGoal::new("forged goal");
}
fn mutate_obligation_provenance(row: &mut InitialObligationDraft) {
    row.provenance = InitialObligationProvenance::new("forged provenance");
}
fn mutate_obligation_status(row: &mut InitialObligationDraft) {
    row.status = InitialObligationStatus::Blocked;
}

fn unrelated_draft(source: SourceId) -> InitialObligationDraft {
    InitialObligationDraft {
        kind: InitialObligationKind::Sethood,
        owner: TypedSiteRef::Node(GUARD_LEFT),
        source_range: range(source, 52, 53),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new("unrelated:sethood"),
        provenance: InitialObligationProvenance::new("unrelated:baseline"),
        status: InitialObligationStatus::Pending,
    }
}

fn unrelated_baseline(source: SourceId) -> InitialObligationTable {
    let mut table = InitialObligationTable::new();
    table.insert(unrelated_draft(source));
    table
}

fn ordered_baseline(source: SourceId) -> InitialObligationTable {
    let mut table = unrelated_baseline(source);
    table.insert(InitialObligationDraft {
        kind: InitialObligationKind::NonEmptiness,
        owner: TypedSiteRef::Node(GUARD_RIGHT),
        source_range: range(source, 56, 57),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new("unrelated:nonemptiness"),
        provenance: InitialObligationProvenance::new("unrelated:ordered-baseline"),
        status: InitialObligationStatus::Pending,
    });
    table
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
    let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task260"));
    let arena = task260_arena(source);
    let source_context = task248_context(source, &module);
    let (env, symbols, definitions, contribution) = resolver_env(source, module.clone());
    let source_type_base = SourceTypeProducer::build(
        task249_input(source, module.clone()),
        source_context.binding_env(),
        &env,
        &arena,
    )
    .expect("Task 249 Profile B");
    let source_type = SourceTypeDefinitionReturnProducer::extend(
        &source_type_base,
        SourceTypeDefinitionReturnExtensionInput {
            source_id: source,
            module_id: module.clone(),
            returns: vec![
                SourceTypeDefinitionReturnInput {
                    definition_site: TypedSiteRef::Node(EQUALS_DEFINITION),
                    definition_range: range(source, 61, 118),
                    source_ordinal: 0,
                    expression: bare_set_type(
                        source,
                        module.clone(),
                        RETURN_EQUALS,
                        RETURN_HEAD_EQUALS,
                        105,
                        108,
                    ),
                },
                SourceTypeDefinitionReturnInput {
                    definition_site: TypedSiteRef::Node(MEANS_DEFINITION),
                    definition_range: range(source, 121, 179),
                    source_ordinal: 1,
                    expression: bare_set_type(
                        source,
                        module.clone(),
                        RETURN_MEANS,
                        RETURN_HEAD_MEANS,
                        163,
                        166,
                    ),
                },
            ],
        },
        &arena,
    )
    .expect("Task 249R definition returns");
    let source_term = SourcePrimaryTermProducer::build(
        task252_input(source, module.clone()),
        source_context.binding_env(),
        &arena,
    )
    .expect("Task 252 profile");
    let atomic_formulas = SourceAtomicFormulaProducer::build(
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
    let input = task260_input(source, module.clone(), symbols, definitions, contribution);
    Fixture {
        source,
        module,
        env,
        source_context,
        source_type,
        source_term,
        atomic_formulas,
        arena,
        input,
    }
}

fn optional_lower_fixture(fixture: &Fixture) -> OptionalLowerFixture {
    let arena = arena_with_source_node(
        &arena_with_source_node(
            &arena_with_source_node(
                &arena_with_source_node(
                    &fixture.arena,
                    TypedNodeId::new(40),
                    "source.term.functor-application.inline",
                    range(fixture.source, 100, 104),
                    LocalTypeContextId::new(1),
                ),
                TypedNodeId::new(41),
                "source.term.functor-head.single",
                range(fixture.source, 100, 101),
                LocalTypeContextId::new(1),
            ),
            TypedNodeId::new(42),
            "source.term.structure.constructor",
            range(fixture.source, 110, 114),
            LocalTypeContextId::new(1),
        ),
        TypedNodeId::new(43),
        "source.term.set.enumeration",
        range(fixture.source, 130, 134),
        LocalTypeContextId::new(1),
    );
    let application = SourceFunctorApplicationProducer::build(
        SourceFunctorApplicationHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            applications: vec![SourceFunctorApplicationInput {
                site: TypedSiteRef::Node(TypedNodeId::new(40)),
                source_range: range(fixture.source, 100, 104),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceFunctorApplicationRecovery::Normal,
                spelling: "inline ( )".to_owned(),
                kind: SourceFunctorApplicationKind::Inline,
                form: SourceFunctorApplicationForm::Functional,
                head_ordinal: 0,
                head: SourceFunctorHeadSite::Single {
                    site: TypedSiteRef::Node(TypedNodeId::new(41)),
                    source_range: range(fixture.source, 100, 101),
                    spelling: "inline".to_owned(),
                },
            }],
            wrappers: Vec::new(),
            candidates: Vec::new(),
            arguments: Vec::new(),
            type_requests: Vec::new(),
        },
        &fixture.env,
        fixture.source_context.binding_env(),
        &fixture.source_term,
        &arena,
    )
    .expect("Task 253 optional lower family");
    let (structure_env, structure_symbol, structure_contribution) =
        optional_structure_env(fixture.source, fixture.module.clone());
    let structure = SourceStructureProducer::build(
        SourceStructureHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            terms: vec![SourceStructureTermInput {
                site: TypedSiteRef::Node(TypedNodeId::new(42)),
                source_range: range(fixture.source, 110, 114),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceStructureRecovery::Normal,
                spelling: "Task260Structure".to_owned(),
                kind: SourceStructureTermKind::Constructor,
            }],
            wrappers: Vec::new(),
            roots: vec![SourceStructureRootInput {
                term: SourceStructureTermId::new(0),
                symbol: structure_symbol,
                contribution: structure_contribution,
            }],
            members: Vec::new(),
            field_updates: Vec::new(),
            edges: Vec::new(),
            requests: vec![
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: None,
                    request_ordinal: 0,
                    kind: SourceStructureRequestKind::ConstructorSignature,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: None,
                    request_ordinal: 1,
                    kind: SourceStructureRequestKind::ResultType,
                },
            ],
        },
        &structure_env,
        fixture.source_context.binding_env(),
        &fixture.source_term,
        None,
        &arena,
    )
    .expect("Task 254 optional lower family");
    let set_term = SourceSetTermProducer::build(
        SourceSetTermHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            terms: vec![SourceSetTermInput {
                site: TypedSiteRef::Node(TypedNodeId::new(43)),
                source_range: range(fixture.source, 130, 134),
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
        },
        fixture.source_context.binding_env(),
        &fixture.source_term,
        None,
        None,
        &arena,
    )
    .expect("Task 255 optional lower family");
    OptionalLowerFixture {
        arena,
        application,
        structure,
        set_term,
    }
}

fn optional_structure_env(
    source: SourceId,
    module: ModuleId,
) -> (SymbolEnv, SymbolId, SourceContributionId) {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 10)),
    );
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 1, 5)),
        vec![4, 0, 4, 0],
    );
    let symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new("task260-structure"),
        FullyQualifiedName::new("pkg::task260::task260-structure"),
    );
    let signature = SignatureShell::Opaque {
        schema: "parser-signature-v1".to_owned(),
        payload: "structure:Task260Structure".to_owned(),
    };
    indexes.symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Structure,
            NamespacePath::new(module.path().as_str()),
            "Task260Structure",
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
        .with_signature(signature.clone()),
    );
    let definition = indexes.definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Structure,
            origin,
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_signature(signature),
    );
    indexes
        .contributions
        .add_symbol(contribution, symbol.clone());
    indexes
        .contributions
        .add_definition(contribution, definition);
    (SymbolEnv::new(module, indexes), symbol, contribution)
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
            source_range: range(source, 0, 261),
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
        range(source, 0, 261),
        Vec::new(),
    );
    let items = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source, 0, 261),
        vec![block],
    );
    let unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source, 0, 261),
        vec![items],
    );
    let root = builder.add_node(SurfaceNodeKind::Root, range(source, 0, 261), vec![unit]);
    let ast = builder.finish(Some(root), None);
    DeclarationShellCollector::new(&ast, module)
        .collect()
        .declarations()[0]
        .id()
}

fn resolver_env(
    source: SourceId,
    module: ModuleId,
) -> (
    SymbolEnv,
    [SymbolId; 2],
    [DefinitionId; 2],
    SourceContributionId,
) {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 261)),
    );
    let specs = [
        (
            "task260_equals",
            "task260_equals ( x )",
            61,
            118,
            vec![4, 0, 9, 0],
        ),
        (
            "task260_means",
            "task260_means ( y )",
            121,
            179,
            vec![4, 0, 9, 1],
        ),
    ];
    let mut symbols = Vec::new();
    let mut definitions = Vec::new();
    for (index, (local, notation, start, end, path)) in specs.into_iter().enumerate() {
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, start, end)),
            path,
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task260::{local}")),
        );
        let signature = SignatureShell::Opaque {
            schema: "parser-signature-v1".to_owned(),
            payload: format!("functor:{notation}"),
        };
        indexes.symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Functor,
                NamespacePath::new("main"),
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
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Functor,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_notation_shape(notation)
            .with_signature(signature),
        );
        assert_eq!(definition.index(), index);
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        indexes
            .contributions
            .add_definition(contribution, definition);
        symbols.push(symbol);
        definitions.push(definition);
    }
    (
        SymbolEnv::new(module, indexes),
        [symbols.remove(0), symbols.remove(0)],
        [definitions[0], definitions[1]],
        contribution,
    )
}

fn task260_arena(source: SourceId) -> TypedArena {
    let mut nodes = Vec::with_capacity(108);
    for index in 0..108 {
        let (kind, source_range, context) = match index {
            62 => ("source.type.head", range(source, 22, 25), 1),
            63 => ("source.type.expression", range(source, 22, 25), 1),
            65 => (
                "source.definition.functor.parameter",
                range(source, 17, 18),
                1,
            ),
            66 => ("source.type.head", range(source, 38, 41), 1),
            67 => ("source.type.expression", range(source, 38, 41), 1),
            69 => (
                "source.definition.functor.parameter",
                range(source, 33, 34),
                1,
            ),
            70 => ("source.term.variable-reference", range(source, 52, 53), 1),
            72 => ("source.term.variable-reference", range(source, 56, 57), 1),
            75 => ("source.formula.atomic.equality", range(source, 52, 57), 1),
            77 => ("source.definition.functor.guard", range(source, 45, 58), 1),
            79 => ("source.type.head", range(source, 105, 108), 1),
            80 => ("source.type.expression", range(source, 105, 108), 1),
            81 => ("source.term.variable-reference", range(source, 116, 117), 1),
            83 => (
                "source.definition.functor.definiens",
                range(source, 116, 117),
                1,
            ),
            84 => ("source.definition.functor", range(source, 61, 118), 1),
            86 => ("source.type.head", range(source, 163, 166), 1),
            87 => ("source.type.expression", range(source, 163, 166), 1),
            88 => ("source.term.variable-reference", range(source, 173, 174), 1),
            90 => ("source.term.variable-reference", range(source, 177, 178), 1),
            93 => ("source.formula.atomic.equality", range(source, 173, 178), 1),
            94 => (
                "source.definition.functor.definiens",
                range(source, 173, 178),
                1,
            ),
            95 => ("source.definition.functor", range(source, 121, 179), 1),
            99 => (
                "source.definition.functor.correctness",
                range(source, 182, 217),
                1,
            ),
            103 => (
                "source.definition.functor.correctness",
                range(source, 220, 256),
                1,
            ),
            104 => ("source.definition", range(source, 0, 261), 1),
            107 => ("source.module", range(source, 0, 261), 0),
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
    TypedArena::try_new(Some(MODULE_ROOT), nodes).expect("Task 260 arena")
}

fn arena_with_node_context(
    arena: &TypedArena,
    node: TypedNodeId,
    context: LocalTypeContextId,
) -> TypedArena {
    let mut nodes = arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
    nodes[node.index()].links.context = Some(context);
    TypedArena::try_new(arena.root(), nodes).expect("coherent alternate Task 260 owner context")
}

fn arena_with_source_node(
    arena: &TypedArena,
    node: TypedNodeId,
    kind: &str,
    source_range: SourceRange,
    context: LocalTypeContextId,
) -> TypedArena {
    let mut nodes = arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
    nodes[node.index()] = TypedNode::new(kind, SourceAnchor::Range(source_range))
        .with_typing(TypingState::Unknown)
        .with_recovery(NodeRecoveryState::Normal)
        .with_links(TypedNodeLinks {
            context: Some(context),
            ..TypedNodeLinks::default()
        });
    TypedArena::try_new(arena.root(), nodes).expect("coherent relocated lower-stage site")
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
        (EQUALS_TERM, 116, 117, "x", BindingId::new(0)),
        (MEANS_LEFT, 173, 174, "x", BindingId::new(0)),
        (MEANS_RIGHT, 177, 178, "y", BindingId::new(1)),
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
            site: TypedSiteRef::Node(MEANS_FORMULA),
            source_range: range(source, 173, 178),
            source_ordinal: 1,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = y".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        },
    ];
    let targets = [0, 1, 3, 4];
    let edges = (0..4)
        .map(|edge| SourceAtomicEdgeInput {
            formula: SourceAtomicFormulaId::new(edge / 2),
            ordinal: edge % 2,
            role: if edge % 2 == 0 {
                SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                SourceAtomicEdgeRole::BuiltinRightOperand
            },
            target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(targets[edge])),
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

fn task260_input(
    source: SourceId,
    module: ModuleId,
    symbols: [SymbolId; 2],
    definitions: [DefinitionId; 2],
    contribution: SourceContributionId,
) -> SourceFunctorDefinitionHandoffInput {
    SourceFunctorDefinitionHandoffInput {
        source_id: source,
        module_id: module,
        definitions: vec![
            SourceFunctorDefinitionInput {
                symbol: symbols[0].clone(),
                definition: definitions[0],
                contribution,
                site: TypedSiteRef::Node(EQUALS_DEFINITION),
                source_range: range(source, 61, 118),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "func Task260EqualsDef: task260_equals(x) -> set equals x;".to_owned(),
                style: SourceFunctorDefinitionStyle::Equals,
                return_type: SourceTypeDefinitionReturnId::new(0),
                definiens: SourceFunctorDefiniensId::new(0),
            },
            SourceFunctorDefinitionInput {
                symbol: symbols[1].clone(),
                definition: definitions[1],
                contribution,
                site: TypedSiteRef::Node(MEANS_DEFINITION),
                source_range: range(source, 121, 179),
                source_ordinal: 1,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "func Task260MeansDef: task260_means(y) -> set means x = y;".to_owned(),
                style: SourceFunctorDefinitionStyle::Means,
                return_type: SourceTypeDefinitionReturnId::new(1),
                definiens: SourceFunctorDefiniensId::new(1),
            },
        ],
        parameters: vec![
            SourceFunctorParameterInput {
                ordinal: 0,
                binding: BindingId::new(0),
                written_type: SourceTypeApplicationId::new(0),
                site: TypedSiteRef::Node(PARAMETER_X),
                source_range: range(source, 13, 26),
                declaration_range: range(source, 17, 18),
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "let x be set;".to_owned(),
            },
            SourceFunctorParameterInput {
                ordinal: 1,
                binding: BindingId::new(1),
                written_type: SourceTypeApplicationId::new(1),
                site: TypedSiteRef::Node(PARAMETER_Y),
                source_range: range(source, 29, 42),
                declaration_range: range(source, 33, 34),
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "let y be set;".to_owned(),
            },
        ],
        guards: vec![SourceFunctorGuardInput {
            ordinal: 0,
            formula: SourceAtomicFormulaId::new(0),
            site: TypedSiteRef::Node(GUARD_OWNER),
            source_range: range(source, 45, 58),
            context: BindingContextId::new(1),
            recovery: SourceFunctorDefinitionRecovery::Normal,
            spelling: "assume x = x;".to_owned(),
        }],
        definientia: vec![
            SourceFunctorDefiniensInput {
                owner: SourceFunctorDefinitionId::new(0),
                ordinal: 0,
                target: SourceFunctorDefiniensTarget::Primary(SourcePrimaryTermId::new(2)),
                site: TypedSiteRef::Node(EQUALS_DEFINIENS),
                source_range: range(source, 116, 117),
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "x".to_owned(),
            },
            SourceFunctorDefiniensInput {
                owner: SourceFunctorDefinitionId::new(1),
                ordinal: 1,
                target: SourceFunctorDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(1)),
                site: TypedSiteRef::Node(MEANS_DEFINIENS),
                source_range: range(source, 173, 178),
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "x = y".to_owned(),
            },
        ],
        correctness: vec![
            SourceFunctorCorrectnessInput {
                owner: SourceFunctorDefinitionId::new(1),
                ordinal: 0,
                kind: SourceFunctorCorrectnessKind::Existence,
                site: TypedSiteRef::Node(EXISTENCE),
                source_range: range(source, 182, 217),
                justification: SourceAnchor::Range(range(source, 192, 216)),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "existence by computation(steps: 1);".to_owned(),
            },
            SourceFunctorCorrectnessInput {
                owner: SourceFunctorDefinitionId::new(1),
                ordinal: 1,
                kind: SourceFunctorCorrectnessKind::Uniqueness,
                site: TypedSiteRef::Node(UNIQUENESS),
                source_range: range(source, 220, 256),
                justification: SourceAnchor::Range(range(source, 231, 255)),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "uniqueness by computation(steps: 1);".to_owned(),
            },
        ],
    }
}

fn expected_debug(fixture: &Fixture, base: usize) -> String {
    let definitions = &fixture.input.definitions;
    format!(
        concat!(
            "source-functor-definition-debug-v1\n",
            "module: task260\n",
            "source-context-fingerprint: {:?}\n",
            "source-type-fingerprint: {:?}\n",
            "source-term-fingerprint: {:?}\n",
            "application-fingerprint: none\n",
            "structure-fingerprint: none\n",
            "set-term-fingerprint: none\n",
            "atomic-formula-fingerprint: {:?}\n",
            "definition#0 symbol={:?} definition=0 contribution=0 ordinal=0 range=61..118 site=node#84 context=1 recovery=normal origin_range=61..118 origin_path=[4, 0, 9, 0] spelling={:?} style=equals return_type=0 definiens=0\n",
            "definition#1 symbol={:?} definition=1 contribution=0 ordinal=1 range=121..179 site=node#95 context=1 recovery=normal origin_range=121..179 origin_path=[4, 0, 9, 1] spelling={:?} style=means return_type=1 definiens=1\n",
            "parameter#0 ordinal=0 binding=0 written_type=0 range=13..26 declaration_range=17..18 site=node#65 context=1 recovery=normal spelling=\"let x be set;\"\n",
            "parameter#1 ordinal=1 binding=1 written_type=1 range=29..42 declaration_range=33..34 site=node#69 context=1 recovery=normal spelling=\"let y be set;\"\n",
            "guard#0 ordinal=0 formula=0 range=45..58 site=node#77 context=1 recovery=normal spelling=\"assume x = x;\"\n",
            "definiens#0 owner=0 ordinal=0 target=primary:2 range=116..117 site=node#83 context=1 recovery=normal spelling=\"x\"\n",
            "definiens#1 owner=1 ordinal=1 target=atomic-formula:1 range=173..178 site=node#94 context=1 recovery=normal spelling=\"x = y\"\n",
            "correctness#0 owner=1 ordinal=0 kind=existence range=182..217 site=node#99 justification=range:192..216 recovery=normal spelling=\"existence by computation(steps: 1);\" obligation={}\n",
            "correctness#1 owner=1 ordinal=1 kind=uniqueness range=220..256 site=node#103 justification=range:231..255 recovery=normal spelling=\"uniqueness by computation(steps: 1);\" obligation={}\n"
        ),
        fixture.source_context.debug_text(),
        fixture.source_type.debug_text(),
        fixture.source_term.debug_text(),
        fixture.atomic_formulas.debug_text(),
        definitions[0].symbol.fqn().as_str(),
        definitions[0].spelling,
        definitions[1].symbol.fqn().as_str(),
        definitions[1].spelling,
        base,
        base + 1,
    )
}

mod task259_actual {
    use super::*;
    use crate::{
        source_predicate_definition::{
            SourcePredicateCorrectnessInput, SourcePredicateDefinitionHandoffInput,
            SourcePredicateDefinitionId, SourcePredicateDefinitionInput,
            SourcePredicateDefinitionProducer, SourcePredicateDefinitionRecovery,
            SourcePredicateGuardInput, SourcePredicateParameterInput, SourcePredicatePropertyId,
            SourcePredicatePropertyInput, SourcePredicatePropertyKind,
        },
        typed_ast::TypedArenaBuilder,
    };

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

    pub(super) fn typed() -> TypedAst {
        let source = source_id();
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task259"));
        let arena = arena(source);
        let source_context = context(source, &module);
        let (env, symbol, definition, contribution) = resolver_env(source, module.clone());
        let source_type = SourceTypeProducer::build(
            type_input(source, module.clone()),
            source_context.binding_env(),
            &env,
            &arena,
        )
        .expect("actual Task 259 type handoff");
        let source_term = SourcePrimaryTermProducer::build(
            term_input(source, module.clone()),
            source_context.binding_env(),
            &arena,
        )
        .expect("actual Task 259 term handoff");
        let source_atomic_formula = SourceAtomicFormulaProducer::build(
            atomic_input(source, module.clone()),
            source_context.binding_env(),
            &env,
            &source_term,
            None,
            None,
            None,
            &arena,
        )
        .expect("actual Task 259 atomic handoff");
        let projection = SourcePredicateDefinitionProducer::build(
            predicate_input(source, module.clone(), symbol, definition, contribution),
            &env,
            &source_context,
            &source_type,
            &source_term,
            &source_atomic_formula,
            &InitialObligationTable::new(),
            &arena,
        )
        .expect("actual Task 259 predicate-definition projection");
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
            source_context: Some(source_context.clone()),
            source_type: Some(source_type),
            source_attribute: None,
            nodes: arena,
            contexts: source_context.local_contexts().clone(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("actual Task 259 typed baseline")
        .with_source_term(source_term)
        .expect("actual Task 259 term installation")
        .with_source_atomic_formula(source_atomic_formula)
        .expect("actual Task 259 atomic installation")
        .with_source_predicate_definition(projection)
        .expect("actual Task 259 predicate installation")
    }

    fn context(source: SourceId, module: &ModuleId) -> SourceBindingContextHandoff {
        let shell = definition_shell(source, module);
        let scope = LocalTermScope::new(vec![0]);
        SourceBindingContextProducer::build(SourceBindingContextInput {
            source_id: source,
            module_id: module.clone(),
            module_site: TypedSiteRef::Node(MODULE_OWNER),
            items: vec![SourceItemInput {
                shell,
                shell_ordinal: 0,
                role: SourceItemRole::DefinitionBlock,
                module_id: module.clone(),
                source_range: range(source, 0, 164),
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: TypedSiteRef::Node(DEFINITION_OWNER),
                local_scope: Some(scope.clone()),
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
                                scope.clone(),
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
        .expect("actual Task 259 context")
        .into_complete()
        .expect("complete actual Task 259 context")
        .into_handoff()
    }

    fn definition_shell(
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
            SourceAnchor::Range(range(source, 0, 164)),
        );
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
        let definition = indexes.definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Predicate,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_notation_shape("x task259_rel y")
            .with_signature(signature),
        );
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

    fn arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        for (kind, start, end, context, children) in [
            ("source.type.head", 22, 25, 1, vec![]),
            ("source.type.expression", 22, 25, 1, vec![TYPE_HEAD_X]),
            (
                "source.definition.predicate.parameter",
                17,
                18,
                1,
                vec![TYPE_EXPRESSION_X],
            ),
            ("source.type.head", 38, 41, 1, vec![]),
            ("source.type.expression", 38, 41, 1, vec![TYPE_HEAD_Y]),
            (
                "source.definition.predicate.parameter",
                33,
                34,
                1,
                vec![TYPE_EXPRESSION_Y],
            ),
            ("source.term.variable-reference", 52, 53, 1, vec![]),
            ("source.term.variable-reference", 56, 57, 1, vec![]),
            (
                "source.formula.atomic.equality",
                52,
                57,
                1,
                vec![GUARD_LEFT, GUARD_RIGHT],
            ),
            ("source.term.variable-reference", 116, 117, 1, vec![]),
            ("source.term.variable-reference", 120, 121, 1, vec![]),
            (
                "source.formula.atomic.equality",
                116,
                121,
                1,
                vec![DEFINIENS_LEFT, DEFINIENS_RIGHT],
            ),
            (
                "source.definition.predicate.guard",
                45,
                58,
                1,
                vec![GUARD_FORMULA],
            ),
            (
                "source.definition.predicate",
                61,
                122,
                1,
                vec![DEFINIENS_FORMULA],
            ),
            ("source.definition.predicate.property", 125, 159, 1, vec![]),
            (
                "source.definition",
                0,
                164,
                1,
                vec![
                    PARAMETER_X,
                    PARAMETER_Y,
                    GUARD_OWNER,
                    PREDICATE_OWNER,
                    PROPERTY_OWNER,
                ],
            ),
            ("source.module", 0, 164, 0, vec![DEFINITION_OWNER]),
        ] {
            builder
                .push(
                    TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                        .with_children(children)
                        .with_typing(TypingState::Unknown)
                        .with_recovery(NodeRecoveryState::Normal)
                        .with_links(TypedNodeLinks {
                            context: Some(LocalTypeContextId::new(context)),
                            ..TypedNodeLinks::default()
                        }),
                )
                .unwrap();
        }
        builder.finish(Some(MODULE_OWNER)).unwrap()
    }

    fn type_input(source: SourceId, module: ModuleId) -> SourceTypeHandoffInput {
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
                bare_type(
                    source,
                    module.clone(),
                    TYPE_EXPRESSION_X,
                    TYPE_HEAD_X,
                    22,
                    25,
                ),
                bare_type(source, module, TYPE_EXPRESSION_Y, TYPE_HEAD_Y, 38, 41),
            ],
            arguments: Vec::new(),
        }
    }

    fn bare_type(
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

    fn term_input(source: SourceId, module: ModuleId) -> SourcePrimaryTermHandoffInput {
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
                    |(ordinal, (site, start, end, spelling, _))| SourcePrimaryTermInput {
                        site: TypedSiteRef::Node(*site),
                        source_range: range(source, *start, *end),
                        source_ordinal: ordinal,
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

    fn atomic_input(source: SourceId, module: ModuleId) -> SourceAtomicFormulaHandoffInput {
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
        .collect();
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

    fn predicate_input(
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
                spelling: "pred Task259PredicateDefinition: x task259_rel y means x = y;"
                    .to_owned(),
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
}

fn source_id() -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "61".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .unwrap()
}

fn other_source_id() -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "62".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    allocator.next_source_id(snapshot).unwrap();
    allocator.next_source_id(snapshot).unwrap()
}

fn other_contribution_id(source: SourceId) -> SourceContributionId {
    let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task260"));
    let mut indexes = SymbolEnvIndexes::default();
    indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 261)),
    );
    indexes.contributions.insert(
        module,
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 261)),
    )
}
