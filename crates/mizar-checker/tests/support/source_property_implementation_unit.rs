use super::*;
use crate::{
    binding_env::BindingRecoveryState,
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
        SourceAtomicFormulaProducer, SourceAtomicFormulaRecovery, SourceAtomicRequestInput,
    },
    source_context::{
        SourceBindingContextBuild, SourceBindingContextInput, SourceBindingContextOwner,
        SourceBindingContextProducer, SourceBindingSiteInput, SourceBindingSiteRole,
        SourceItemInput,
    },
    source_structure::{
        SourceStructureEdgeInput, SourceStructureHandoffInput, SourceStructureMemberInput,
        SourceStructureProducer, SourceStructureRecovery, SourceStructureRequestInput,
        SourceStructureTermInput,
    },
    source_term::{
        SourcePrimaryTermCorruptionForTest, SourcePrimaryTermHandoffInput, SourcePrimaryTermInput,
        SourcePrimaryTermProducer, SourcePrimaryTermReferenceInput,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationInput, SourceTypeExpressionId,
        SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeHead, SourceTypeProducer,
        SourceTypeStructureMemberHandoffInput, SourceTypeStructureMemberInput,
        SourceTypeStructureMemberProducer,
    },
    typed_ast::{
        CoercionTable, TypeDiagnosticTable, TypeFactTable, TypeRole, TypeTable, TypedAst,
        TypedAstError, TypedAstParts, TypedNode, TypedNodeLinks,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShellCollector, DeclarationShellId},
    env::{DefinitionShell, NamespacePath, SymbolEntry, SymbolEnvIndexes},
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{FullyQualifiedName, LocalSymbolId},
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
};
use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestProfile {
    Means,
    Equals,
}

type ResolverIdentity = (SymbolId, DefinitionId, SourceContributionId);

#[derive(Clone)]
struct Fixture {
    profile: TestProfile,
    source: SourceId,
    module: ModuleId,
    env: SymbolEnv,
    context: SourceBindingContextHandoff,
    source_type: SourceTypeApplicationHandoff,
    terms: SourcePrimaryTermHandoff,
    structure: Option<SourceStructureHandoff>,
    formula: Option<SourceAtomicFormulaHandoff>,
    arena: TypedArena,
    input: SourcePropertyImplementationHandoffInput,
}

impl Fixture {
    fn build(
        &self,
        base: &InitialObligationTable,
    ) -> Result<SourcePropertyImplementationProjection, SourcePropertyImplementationError> {
        SourcePropertyImplementationProducer::build(
            self.input.clone(),
            &self.env,
            &self.context,
            &self.source_type,
            &self.terms,
            None,
            self.structure.as_ref(),
            None,
            self.formula.as_ref(),
            base,
            &self.arena,
        )
    }

    fn projection(&self, base: &InitialObligationTable) -> SourcePropertyImplementationProjection {
        self.build(base).expect("exact Task 264 projection")
    }

    fn typed_parts(&self, obligations: InitialObligationTable) -> TypedAstParts {
        TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: Some(self.context.clone()),
            source_type: Some(self.source_type.clone()),
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: self.context.local_contexts().clone(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: obligations,
            diagnostics: TypeDiagnosticTable::new(),
        }
    }

    fn typed_base(&self, obligations: InitialObligationTable) -> TypedAst {
        let typed = TypedAst::try_new(self.typed_parts(obligations))
            .expect("Task 264 typed baseline")
            .with_source_term(self.terms.clone())
            .expect("Task 252 installation");
        match self.profile {
            TestProfile::Means => typed
                .with_source_atomic_formula(self.formula.clone().expect("means formula"))
                .expect("Task 256 installation"),
            TestProfile::Equals => typed
                .with_source_structure(self.structure.clone().expect("equals structure"))
                .expect("Task 254 installation"),
        }
    }

    fn validate_handoff(
        &self,
        handoff: &SourcePropertyImplementationHandoff,
        obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<(), SourcePropertyImplementationError> {
        handoff.validate_installation(
            self.source,
            &self.module,
            &self.context,
            &self.source_type,
            &self.terms,
            None,
            self.structure.as_ref(),
            None,
            self.formula.as_ref(),
            obligations,
            arena,
        )
    }
}

fn optional_fingerprint(value: Option<&str>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| format!("some({value:?})"))
}

fn expected_debug(fixture: &Fixture) -> String {
    let header = format!(
        concat!(
            "source-property-implementation-debug-v2\n",
            "module: {}\n",
            "source-context-fingerprint: {:?}\n",
            "source-type-fingerprint: {:?}\n",
            "source-term-fingerprint: {:?}\n",
            "source-functor-application-fingerprint: {}\n",
            "source-structure-fingerprint: {}\n",
            "source-set-term-fingerprint: {}\n",
            "source-atomic-formula-fingerprint: {}\n",
            "carrier-identity#0 role=structure symbol=\"pkg::task264::Task264Carrier\" definition=0 contribution=0 origin_range=13..101 origin_path=[4, 0, 11, 0]\n",
            "carrier-identity#1 role=field symbol=\"pkg::task264::carrier\" definition=1 contribution=0 origin_range=45..66 origin_path=[4, 0, 11, 0, 18, 0]\n",
            "carrier-identity#2 role=property symbol=\"pkg::task264::marker\" definition=2 contribution=0 origin_range=71..94 origin_path=[4, 0, 11, 0, 19, 1]\n"
        ),
        fixture.module.path().as_str(),
        fixture.context.debug_text(),
        fixture.source_type.debug_text(),
        fixture.terms.debug_text(),
        optional_fingerprint(None),
        optional_fingerprint(
            fixture
                .structure
                .as_ref()
                .map(SourceStructureHandoff::debug_text)
                .as_deref()
        ),
        optional_fingerprint(None),
        optional_fingerprint(
            fixture
                .formula
                .as_ref()
                .map(SourceAtomicFormulaHandoff::debug_text)
                .as_deref()
        ),
    );
    let rows = match fixture.profile {
        TestProfile::Means => concat!(
            "implementation#0 shell=4 range=108..262 site=node#81 ordinal=0 context=1 recovery=normal spelling=\"definition\\n  let M be Task264Carrier;\\n  property M.marker means it = it;\\n  existence by computation(steps: 1);\\n  uniqueness by computation(steps: 1);\\nend;\" style=means parameter=0 target=0 definiens=0\n",
            "parameter#0 owner=0 ordinal=0 binding=0 written_type=0 range=121..145 declaration_range=125..126 site=node#65 context=1 recovery=normal spelling=\"let M be Task264Carrier;\"\n",
            "target#0 owner=0 ordinal=0 subject=0 symbol=\"pkg::task264::marker\" definition=2 contribution=0 range=157..165 subject_range=157..158 name_range=159..165 site=role#81:source.property-implementation.target spelling=\"M.marker\" return_type=1 origin_range=71..94 origin_path=[4, 0, 11, 0, 19, 1]\n",
            "definiens#0 owner=0 ordinal=0 target=atomic-formula#0 range=172..179 site=node#72 context=1 recovery=normal spelling=\"it = it\"\n",
            "correctness#0 owner=0 ordinal=0 kind=existence range=183..218 site=node#76 justification=range:193..217 recovery=normal spelling=\"existence by computation(steps: 1);\" obligation=0\n",
            "correctness#1 owner=0 ordinal=1 kind=uniqueness range=221..257 site=node#80 justification=range:232..256 recovery=normal spelling=\"uniqueness by computation(steps: 1);\" obligation=1\n",
        ),
        TestProfile::Equals => concat!(
            "implementation#0 shell=4 range=108..188 site=node#52 ordinal=0 context=1 recovery=normal spelling=\"definition\\n  let M be Task264Carrier;\\n  property M.marker equals M.carrier;\\nend;\" style=equals parameter=0 target=0 definiens=0\n",
            "parameter#0 owner=0 ordinal=0 binding=0 written_type=0 range=121..145 declaration_range=125..126 site=node#47 context=1 recovery=normal spelling=\"let M be Task264Carrier;\"\n",
            "target#0 owner=0 ordinal=0 subject=0 symbol=\"pkg::task264::marker\" definition=2 contribution=0 range=157..165 subject_range=157..158 name_range=159..165 site=role#52:source.property-implementation.target spelling=\"M.marker\" return_type=1 origin_range=71..94 origin_path=[4, 0, 11, 0, 19, 1]\n",
            "definiens#0 owner=0 ordinal=0 target=structure#0 range=173..182 site=node#51 context=1 recovery=normal spelling=\"M.carrier\"\n",
        ),
    };
    header + rows
}

fn assert_carrier_identity_replay_rejected(
    fixture: &Fixture,
    projection: &SourcePropertyImplementationProjection,
    mutate: impl FnOnce(&mut SourcePropertyCarrierIdentity),
) {
    let mut handoff = projection.handoff().clone();
    mutate(&mut handoff.carrier_identity);
    assert_eq!(
        fixture.validate_handoff(&handoff, projection.initial_obligations(), &fixture.arena,),
        Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })
    );
}

fn assert_exact_rows(
    fixture: &Fixture,
    handoff: &SourcePropertyImplementationHandoff,
    obligations: &InitialObligationTable,
) {
    let identity = handoff.carrier_identity();
    let definitions = (0..3)
        .map(|index| {
            fixture
                .env
                .definitions()
                .iter()
                .find(|row| row.id().index() == index)
                .expect("Task264 definition identity")
        })
        .collect::<Vec<_>>();
    assert_eq!(identity.structure_symbol(), definitions[0].symbol());
    assert_eq!(identity.structure_definition(), definitions[0].id());
    assert_eq!(
        identity.structure_contribution(),
        definitions[0].contribution()
    );
    assert_eq!(identity.structure_origin(), definitions[0].origin());
    assert_eq!(identity.field_symbol(), definitions[1].symbol());
    assert_eq!(identity.field_definition(), definitions[1].id());
    assert_eq!(identity.field_contribution(), definitions[1].contribution());
    assert_eq!(identity.field_origin(), definitions[1].origin());
    assert_eq!(identity.property_symbol(), definitions[2].symbol());
    assert_eq!(identity.property_definition(), definitions[2].id());
    assert_eq!(
        identity.property_contribution(),
        definitions[2].contribution()
    );
    assert_eq!(identity.property_origin(), definitions[2].origin());

    let implementation = handoff
        .implementations()
        .get(SourcePropertyImplementationId::new(0))
        .expect("implementation row 0");
    let expected = &fixture.input.implementations[0];
    assert_eq!(implementation.id(), SourcePropertyImplementationId::new(0));
    assert_eq!(implementation.shell(), expected.shell);
    assert_eq!(implementation.site(), &expected.site);
    assert_eq!(implementation.source_range(), expected.source_range);
    assert_eq!(implementation.source_ordinal(), expected.source_ordinal);
    assert_eq!(implementation.context(), expected.context);
    assert_eq!(implementation.recovery(), expected.recovery);
    assert_eq!(implementation.spelling(), expected.spelling);
    assert_eq!(implementation.style(), expected.style);
    assert_eq!(implementation.parameter(), expected.parameter);
    assert_eq!(implementation.target(), expected.target);
    assert_eq!(implementation.definiens(), expected.definiens);

    let parameter = handoff
        .parameters()
        .get(SourcePropertyParameterId::new(0))
        .expect("parameter row 0");
    let expected = &fixture.input.parameters[0];
    assert_eq!(parameter.id(), SourcePropertyParameterId::new(0));
    assert_eq!(parameter.owner(), expected.owner);
    assert_eq!(parameter.ordinal(), expected.ordinal);
    assert_eq!(parameter.binding(), expected.binding);
    assert_eq!(parameter.written_type(), expected.written_type);
    assert_eq!(parameter.site(), &expected.site);
    assert_eq!(parameter.source_range(), expected.source_range);
    assert_eq!(parameter.declaration_range(), expected.declaration_range);
    assert_eq!(parameter.context(), expected.context);
    assert_eq!(parameter.recovery(), expected.recovery);
    assert_eq!(parameter.spelling(), expected.spelling);

    let target = handoff
        .targets()
        .get(SourcePropertyTargetId::new(0))
        .expect("target row 0");
    let expected = &fixture.input.targets[0];
    assert_eq!(target.id(), SourcePropertyTargetId::new(0));
    assert_eq!(target.owner(), expected.owner);
    assert_eq!(target.ordinal(), expected.ordinal);
    assert_eq!(target.subject(), expected.subject);
    assert_eq!(target.symbol(), &expected.symbol);
    assert_eq!(target.definition(), expected.definition);
    assert_eq!(target.contribution(), expected.contribution);
    assert_eq!(target.site(), &expected.site);
    assert_eq!(target.source_range(), expected.source_range);
    assert_eq!(target.subject_range(), expected.subject_range);
    assert_eq!(target.name_range(), expected.name_range);
    assert_eq!(target.spelling(), expected.spelling);
    assert_eq!(target.return_type(), expected.return_type);
    assert!(normal_origin(
        target.origin(),
        fixture.source,
        &fixture.module,
        range(fixture.source, 71, 94),
        &[4, 0, 11, 0, 19, 1],
    ));

    let definiens = handoff
        .definientia()
        .get(SourcePropertyDefiniensId::new(0))
        .expect("definiens row 0");
    let expected = &fixture.input.definientia[0];
    assert_eq!(definiens.id(), SourcePropertyDefiniensId::new(0));
    assert_eq!(definiens.owner(), expected.owner);
    assert_eq!(definiens.ordinal(), expected.ordinal);
    assert_eq!(definiens.target(), expected.target);
    assert_eq!(definiens.site(), &expected.site);
    assert_eq!(definiens.source_range(), expected.source_range);
    assert_eq!(definiens.context(), expected.context);
    assert_eq!(definiens.recovery(), expected.recovery);
    assert_eq!(definiens.spelling(), expected.spelling);

    for (index, expected) in fixture.input.correctness.iter().enumerate() {
        let row = handoff
            .correctness()
            .get(SourcePropertyCorrectnessId::new(index))
            .expect("correctness row");
        assert_eq!(row.id(), SourcePropertyCorrectnessId::new(index));
        assert_eq!(row.owner(), expected.owner);
        assert_eq!(row.ordinal(), expected.ordinal);
        assert_eq!(row.kind(), expected.kind);
        assert_eq!(row.site(), &expected.site);
        assert_eq!(row.source_range(), expected.source_range);
        assert_eq!(row.justification(), &expected.justification);
        assert_eq!(row.recovery(), expected.recovery);
        assert_eq!(row.spelling(), expected.spelling);
        assert_eq!(row.obligation(), InitialObligationId::new(index));
        let obligation = obligations.get(row.obligation()).expect("obligation row");
        let (kind, goal, provenance) = if index == 0 {
            (
                InitialObligationKind::PropertyImplementationExistence,
                "source.definition.property-implementation.correctness:implementation=0:existence",
                "source.definition.property-implementation:implementation=0:correctness=0",
            )
        } else {
            (
                InitialObligationKind::PropertyImplementationUniqueness,
                "source.definition.property-implementation.correctness:implementation=0:uniqueness",
                "source.definition.property-implementation:implementation=0:correctness=1",
            )
        };
        assert_eq!(obligation.id, InitialObligationId::new(index));
        assert_eq!(obligation.kind, kind);
        assert_eq!(obligation.owner, expected.site);
        assert_eq!(obligation.source_range, expected.source_range);
        assert!(obligation.assumptions.is_empty());
        assert_eq!(obligation.goal.as_str(), goal);
        assert_eq!(obligation.provenance.as_str(), provenance);
        assert_eq!(obligation.status, InitialObligationStatus::Pending);
    }
}

#[test]
fn task_264_exact_means_equals_tables_fingerprints_debug_and_obligations() {
    for profile in [TestProfile::Means, TestProfile::Equals] {
        let fixture = fixture(profile);
        let projection = fixture.projection(&InitialObligationTable::new());
        let handoff = projection.handoff();
        let correctness = if profile == TestProfile::Means { 2 } else { 0 };
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(
            handoff.source_context_fingerprint(),
            fixture.context.debug_text()
        );
        assert_eq!(
            handoff.source_type_fingerprint(),
            fixture.source_type.debug_text()
        );
        assert_eq!(
            handoff.source_term_fingerprint(),
            fixture.terms.debug_text()
        );
        assert_eq!(handoff.source_functor_application_fingerprint(), None);
        assert_eq!(handoff.source_set_term_fingerprint(), None);
        assert_eq!(
            handoff.source_structure_fingerprint(),
            fixture
                .structure
                .as_ref()
                .map(SourceStructureHandoff::debug_text)
                .as_deref()
        );
        assert_eq!(
            handoff.source_atomic_formula_fingerprint(),
            fixture
                .formula
                .as_ref()
                .map(SourceAtomicFormulaHandoff::debug_text)
                .as_deref()
        );
        assert_eq!(
            (
                handoff.implementations().len(),
                handoff.parameters().len(),
                handoff.targets().len(),
                handoff.definientia().len(),
                handoff.correctness().len(),
            ),
            (1, 1, 1, 1, correctness)
        );
        assert_eq!(projection.initial_obligations().len(), correctness);
        assert_exact_rows(&fixture, handoff, projection.initial_obligations());
        assert_eq!(handoff.debug_text(), expected_debug(&fixture));
        assert_eq!(handoff.debug_text(), handoff.clone().debug_text());
        if profile == TestProfile::Means {
            assert_eq!(
                projection
                    .initial_obligations()
                    .iter()
                    .map(|(_, row)| row.kind)
                    .collect::<Vec<_>>(),
                vec![
                    InitialObligationKind::PropertyImplementationExistence,
                    InitialObligationKind::PropertyImplementationUniqueness,
                ]
            );
        }
    }
}

#[test]
fn task_264d_equals_selector_identity_is_exact_and_deterministic() {
    let fixture = fixture(TestProfile::Equals);
    let projection = fixture.projection(&InitialObligationTable::new());
    let property = projection.handoff().clone();
    let structures = fixture.structure.clone().expect("Task264 equals structure");
    let first = SourcePropertyEqualsSelectorIdentityProducer::build(
        &fixture.env,
        property.clone(),
        fixture.terms.clone(),
        structures.clone(),
    )
    .expect("Task264D equals selector identity");
    let second = SourcePropertyEqualsSelectorIdentityProducer::build(
        &fixture.env,
        property.clone(),
        fixture.terms.clone(),
        structures.clone(),
    )
    .expect("Task264D deterministic replay");
    assert_eq!(first, second);
    assert_eq!(first.source_id(), fixture.source);
    assert_eq!(first.module_id(), &fixture.module);
    assert_eq!(first.property(), &property);
    assert_eq!(first.terms(), &fixture.terms);
    assert_eq!(first.structures(), &structures);

    let association = first.association();
    assert_eq!(association.implementation().index(), 0);
    assert_eq!(association.definiens().index(), 0);
    assert_eq!(association.structure_term().index(), 0);
    assert_eq!(association.member().index(), 0);
    assert_eq!(association.member_request().index(), 0);
    assert_eq!(association.base_edge().index(), 0);
    assert_eq!(association.base_term().index(), 0);
    assert_eq!(association.base_reference().index(), 0);
    assert_eq!(association.base_binding().index(), 0);
    assert_eq!(
        association.selector_symbol(),
        property.carrier_identity().field_symbol()
    );
    assert_eq!(
        first.debug_text(),
        format!(
            concat!(
                "source-property-equals-selector-identity-debug-v1\n",
                "module: {}\n",
                "property-fingerprint: {:?}\n",
                "primary-term-fingerprint: {:?}\n",
                "structure-fingerprint: {:?}\n",
                "association implementation=0 definiens=0 structure-term=0 member=0 ",
                "member-request=0 base-edge=0 base-term=0 base-reference=0 base-binding=0 selector={:?}\n",
            ),
            fixture.module.path().as_str(),
            property.debug_text(),
            fixture.terms.debug_text(),
            structures.debug_text(),
            property.carrier_identity().field_symbol().fqn().as_str(),
        )
    );
    assert_eq!(property.debug_text(), expected_debug(&fixture));
}

#[test]
fn task_264d_equals_selector_identity_fails_closed() {
    let equals = fixture(TestProfile::Equals);
    let projection = equals.projection(&InitialObligationTable::new());
    let property = projection.handoff().clone();
    let structures = equals.structure.clone().expect("Task264 equals structure");

    let mut wrong_profile = property.clone();
    wrong_profile.implementations.rows[0].style = SourcePropertyImplementationStyle::Means;
    assert_eq!(
        SourcePropertyEqualsSelectorIdentityProducer::build(
            &equals.env,
            wrong_profile,
            equals.terms.clone(),
            structures.clone(),
        ),
        Err(SourcePropertyEqualsSelectorIdentityError::UnsupportedProfile)
    );

    let mut wrong_fingerprint = property.clone();
    wrong_fingerprint.source_term_fingerprint.push_str("forged");
    assert_eq!(
        SourcePropertyEqualsSelectorIdentityProducer::build(
            &equals.env,
            wrong_fingerprint,
            equals.terms.clone(),
            structures.clone(),
        ),
        Err(SourcePropertyEqualsSelectorIdentityError::DependencyMismatch)
    );

    let mut wrong_terms = equals.terms.clone();
    wrong_terms.set_reference_use_ordinal_for_test(SourcePrimaryTermReferenceId::new(0), 0);
    assert_eq!(
        SourcePropertyEqualsSelectorIdentityProducer::build(
            &equals.env,
            property.clone(),
            wrong_terms,
            structures.clone(),
        ),
        Err(SourcePropertyEqualsSelectorIdentityError::DependencyMismatch)
    );
    let mut wrong_term_row = equals.terms.clone();
    wrong_term_row.corrupt_for_test(SourcePrimaryTermCorruptionForTest::Rewrite {
        term: SourcePrimaryTermId::new(0),
        site_and_range: None,
        spelling: None,
        kind_and_role: Some((
            SourcePrimaryTermKind::It,
            SourcePrimaryTermRole::CurrentDefinitionResult,
        )),
    });
    assert_eq!(
        SourcePropertyEqualsSelectorIdentityProducer::build(
            &equals.env,
            property.clone(),
            wrong_term_row,
            structures.clone(),
        ),
        Err(SourcePropertyEqualsSelectorIdentityError::DependencyMismatch)
    );

    let (wrong_env, _) = resolver_env_with_spelling_override(
        equals.source,
        equals.module.clone(),
        Some((1, "forged")),
    );
    assert_eq!(
        SourcePropertyEqualsSelectorIdentityProducer::build(
            &wrong_env,
            property.clone(),
            equals.terms.clone(),
            structures.clone(),
        ),
        Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
    );
    for mutation in [
        Task264ResolverEnvMutation::FieldPrivate,
        Task264ResolverEnvMutation::FieldLocalOnly,
        Task264ResolverEnvMutation::FieldWrongOrigin,
        Task264ResolverEnvMutation::MissingFieldEffects,
    ] {
        let (wrong_env, _) =
            resolver_env_with_mutation(equals.source, equals.module.clone(), mutation);
        assert_eq!(
            SourcePropertyEqualsSelectorIdentityProducer::build(
                &wrong_env,
                property.clone(),
                equals.terms.clone(),
                structures.clone(),
            ),
            Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
        );
    }
    for mutation in [
        Task264ResolverEnvMutation::ExtraContribution,
        Task264ResolverEnvMutation::ExtraSymbol,
        Task264ResolverEnvMutation::ExtraDefinition,
    ] {
        let (wrong_env, _) =
            resolver_env_with_mutation(equals.source, equals.module.clone(), mutation);
        assert_eq!(
            SourcePropertyEqualsSelectorIdentityProducer::build(
                &wrong_env,
                property.clone(),
                equals.terms.clone(),
                structures.clone(),
            ),
            Err(SourcePropertyEqualsSelectorIdentityError::EnvironmentMismatch)
        );
    }
    let (same_module_foreign_source, _) =
        resolver_env(source_id(TestProfile::Means), equals.module.clone());
    assert_eq!(
        SourcePropertyEqualsSelectorIdentityProducer::build(
            &same_module_foreign_source,
            property.clone(),
            equals.terms.clone(),
            structures.clone(),
        ),
        Err(SourcePropertyEqualsSelectorIdentityError::EnvironmentMismatch)
    );

    let foreign = fixture(TestProfile::Means);
    assert_eq!(
        SourcePropertyEqualsSelectorIdentityProducer::build(
            &foreign.env,
            property.clone(),
            equals.terms.clone(),
            structures.clone(),
        ),
        Err(SourcePropertyEqualsSelectorIdentityError::EnvironmentMismatch)
    );

    let mut handoff = SourcePropertyEqualsSelectorIdentityProducer::build(
        &equals.env,
        property.clone(),
        equals.terms.clone(),
        structures,
    )
    .expect("Task264D baseline");
    let exact = handoff.clone();
    handoff.association.structure_term = SourceStructureTermId::new(1);
    assert_eq!(
        handoff.validate(),
        Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
    );
    handoff = exact.clone();
    handoff.association.member = SourceStructureMemberId::new(1);
    assert_eq!(
        handoff.validate(),
        Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
    );
    handoff = exact.clone();
    handoff.association.base_edge = SourceStructureEdgeId::new(1);
    assert_eq!(
        handoff.validate(),
        Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
    );
    handoff = exact.clone();
    handoff.association.member_request = SourceStructureRequestId::new(1);
    assert_eq!(
        handoff.validate(),
        Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
    );
    handoff = exact.clone();
    handoff.association.base_term = SourcePrimaryTermId::new(1);
    assert_eq!(
        handoff.validate(),
        Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
    );
    handoff = exact.clone();
    handoff.association.base_reference = SourcePrimaryTermReferenceId::new(1);
    assert_eq!(
        handoff.validate(),
        Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
    );
    handoff = exact.clone();
    handoff.association.base_binding = BindingId::new(1);
    assert_eq!(
        handoff.validate(),
        Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
    );
    handoff = exact;
    handoff.association.selector_symbol = property.carrier_identity().property_symbol().clone();
    assert_eq!(
        handoff.validate(),
        Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)
    );
}

#[test]
fn task_264_row_style_it_structure_and_correctness_corruption_fail_closed() {
    let means = fixture(TestProfile::Means);
    for mutate in [
        |input: &mut SourcePropertyImplementationHandoffInput| {
            input.implementations[0].style = SourcePropertyImplementationStyle::Equals;
        },
        |input: &mut SourcePropertyImplementationHandoffInput| {
            input.parameters[0].ordinal = 1;
        },
        |input: &mut SourcePropertyImplementationHandoffInput| {
            input.targets[0].spelling = "M.carrier".to_owned();
        },
        |input: &mut SourcePropertyImplementationHandoffInput| {
            input.definientia[0].source_range.end -= 1;
        },
        |input: &mut SourcePropertyImplementationHandoffInput| {
            input.correctness.swap(0, 1);
        },
    ] {
        let mut corrupt = means.clone();
        mutate(&mut corrupt.input);
        assert!(corrupt.build(&InitialObligationTable::new()).is_err());
    }
    let mut missing_correctness = means.clone();
    missing_correctness.input.correctness.pop();
    assert!(
        missing_correctness
            .build(&InitialObligationTable::new())
            .is_err()
    );
    let mut extra_correctness = means.clone();
    extra_correctness
        .input
        .correctness
        .push(means.input.correctness[1].clone());
    assert!(
        extra_correctness
            .build(&InitialObligationTable::new())
            .is_err()
    );

    for count in [0, 1, 3] {
        let mut wrong_count = means.clone();
        if count < 2 {
            wrong_count
                .terms
                .corrupt_for_test(SourcePrimaryTermCorruptionForTest::Truncate(count));
        } else {
            wrong_count
                .terms
                .corrupt_for_test(SourcePrimaryTermCorruptionForTest::Duplicate(
                    SourcePrimaryTermId::new(1),
                ));
        }
        assert_eq!(
            wrong_count.build(&InitialObligationTable::new()),
            Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })
        );
    }

    let mut relocated_it = means.clone();
    relocated_it
        .terms
        .corrupt_for_test(SourcePrimaryTermCorruptionForTest::Rewrite {
            term: SourcePrimaryTermId::new(0),
            site_and_range: Some((node_site(66), range(relocated_it.source, 171, 174))),
            spelling: None,
            kind_and_role: None,
        });
    assert_eq!(
        relocated_it.build(&InitialObligationTable::new()),
        Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })
    );

    let mut misspelled_it = means.clone();
    misspelled_it
        .terms
        .corrupt_for_test(SourcePrimaryTermCorruptionForTest::Rewrite {
            term: SourcePrimaryTermId::new(0),
            site_and_range: None,
            spelling: Some("result".to_owned()),
            kind_and_role: None,
        });
    assert_eq!(
        misspelled_it.build(&InitialObligationTable::new()),
        Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })
    );

    let mut wrong_it = means.clone();
    wrong_it
        .terms
        .corrupt_for_test(SourcePrimaryTermCorruptionForTest::Rewrite {
            term: SourcePrimaryTermId::new(0),
            site_and_range: None,
            spelling: None,
            kind_and_role: Some((SourcePrimaryTermKind::It, SourcePrimaryTermRole::Value)),
        });
    wrong_it.formula = Some(atomic_formula(
        wrong_it.source,
        wrong_it.module.clone(),
        &wrong_it.env,
        &wrong_it.context,
        &wrong_it.terms,
        &wrong_it.arena,
    ));
    assert_eq!(
        wrong_it.build(&InitialObligationTable::new()),
        Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })
    );
    let equals = fixture(TestProfile::Equals);
    assert!(
        validate_source_type_profile(Profile::Equals, means.source, &means.source_type).is_err()
    );
    assert!(
        validate_source_type_profile(Profile::Means, equals.source, &equals.source_type).is_err()
    );

    let mut equals_with_it = equals.clone();
    equals_with_it
        .terms
        .corrupt_for_test(SourcePrimaryTermCorruptionForTest::Rewrite {
            term: SourcePrimaryTermId::new(0),
            site_and_range: None,
            spelling: None,
            kind_and_role: Some((
                SourcePrimaryTermKind::It,
                SourcePrimaryTermRole::CurrentDefinitionResult,
            )),
        });
    assert!(
        equals_with_it
            .build(&InitialObligationTable::new())
            .is_err()
    );

    let mut equals_with_formula = equals.clone();
    equals_with_formula.formula = means.formula.clone();
    assert!(
        equals_with_formula
            .build(&InitialObligationTable::new())
            .is_err()
    );

    let mut equals_with_correctness = equals.clone();
    equals_with_correctness.input.correctness = means.input.correctness.clone();
    assert!(
        equals_with_correctness
            .build(&InitialObligationTable::new())
            .is_err()
    );

    let mut wrong_structure = equals.clone();
    wrong_structure.structure = Some(structure(
        equals.source,
        equals.module.clone(),
        &equals.env,
        &equals.context,
        &equals.terms,
        &equals.arena,
        true,
    ));
    assert_eq!(
        wrong_structure.build(&InitialObligationTable::new()),
        Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })
    );
}

#[test]
fn task_264_resolver_return_fingerprint_arena_and_obligation_corruption_fail_closed() {
    let fixture = fixture(TestProfile::Means);
    let mut corrupt = fixture.clone();
    corrupt.input.targets[0].definition = fixture
        .env
        .definitions()
        .iter()
        .nth(1)
        .map(|row| row.id())
        .expect("carrier definition");
    assert_eq!(
        corrupt.build(&InitialObligationTable::new()),
        Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })
    );
    for role in 0..3 {
        let mut corrupt = fixture.clone();
        corrupt.env = resolver_env_with_spelling_override(
            fixture.source,
            fixture.module.clone(),
            Some((role, "forged")),
        )
        .0;
        assert_eq!(
            corrupt.build(&InitialObligationTable::new()),
            Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 }),
            "resolver role {role} must remain exact"
        );
    }
    let mut corrupt = fixture.clone();
    corrupt.input.targets[0].return_type = SourceTypeStructureMemberId::new(0);
    assert_eq!(
        corrupt.build(&InitialObligationTable::new()),
        Err(SourcePropertyImplementationError::InvalidTarget { index: 0 })
    );

    let projection = fixture.projection(&InitialObligationTable::new());
    let exact_identity = projection.handoff().carrier_identity.clone();
    let other_contribution = alternate_contribution(fixture.source, &fixture.module);
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.structure.symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("task264-forged-structure"),
            FullyQualifiedName::new("pkg::task264::forged-structure"),
        );
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.structure.definition = exact_identity.field.definition;
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.structure.contribution = other_contribution;
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.structure.origin = exact_identity.field.origin.clone();
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.field.symbol = exact_identity.property.symbol.clone();
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        let foreign_module = ModuleId::new(
            PackageId::new("foreign"),
            ModulePath::new("task264.foreign"),
        );
        identity.field.symbol = SymbolId::new(
            foreign_module,
            LocalSymbolId::new("task264-foreign-field"),
            FullyQualifiedName::new("foreign::task264::carrier"),
        );
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.field.definition = exact_identity.structure.definition;
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.field.contribution = other_contribution;
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.field.origin = exact_identity.property.origin.clone();
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.property.symbol = exact_identity.field.symbol.clone();
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.property.definition = exact_identity.field.definition;
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.property.contribution = other_contribution;
    });
    assert_carrier_identity_replay_rejected(&fixture, &projection, |identity| {
        identity.property.origin = exact_identity.field.origin.clone();
    });
    let mut corrupt_target_link = projection.handoff().clone();
    corrupt_target_link.targets.rows[0].symbol = exact_identity.field.symbol.clone();
    assert_eq!(
        fixture.validate_handoff(
            &corrupt_target_link,
            projection.initial_obligations(),
            &fixture.arena,
        ),
        Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })
    );
    let mut corrupt_handoff = projection.handoff().clone();
    corrupt_handoff.source_term_fingerprint.push_str("forged");
    assert_eq!(
        fixture.validate_handoff(
            &corrupt_handoff,
            projection.initial_obligations(),
            &fixture.arena,
        ),
        Err(SourcePropertyImplementationError::DependencyMismatch)
    );

    let corrupt_arena = arena(
        fixture.source,
        fixture.profile,
        Some((72, "source.surface.unowned")),
    );
    assert_eq!(
        fixture.validate_handoff(
            projection.handoff(),
            projection.initial_obligations(),
            &corrupt_arena,
        ),
        Err(SourcePropertyImplementationError::InvalidArenaOwnership)
    );
    let relocated_unowned =
        arena_with_range_override(fixture.source, fixture.profile, None, Some((0, 1, 10)));
    assert_eq!(
        fixture.validate_handoff(
            projection.handoff(),
            projection.initial_obligations(),
            &relocated_unowned,
        ),
        Err(SourcePropertyImplementationError::InvalidArenaOwnership)
    );

    for profile in [TestProfile::Means, TestProfile::Equals] {
        let profile_fixture = self::fixture(profile);
        let validator_profile = match profile {
            TestProfile::Means => Profile::Means,
            TestProfile::Equals => Profile::Equals,
        };
        for index in 0..=root_node(profile) {
            let [surface_start, surface_end] = surface_range(profile, index);
            let (start, end) = owned_node(profile, index)
                .map_or((surface_start, surface_end), |(start, end, _)| (start, end));
            let wrong_range = arena_with_overrides(
                profile_fixture.source,
                profile,
                None,
                Some((index, start, end + 1)),
                None,
                None,
            );
            assert_eq!(
                validate_arena(validator_profile, profile_fixture.source, &wrong_range),
                Err(SourcePropertyImplementationError::InvalidArenaOwnership),
                "profile {profile:?} node {index} range",
            );

            let wrong_kind = if owned_node(profile, index).is_some() {
                "source.surface.unowned"
            } else {
                "source.term.it"
            };
            let wrong_kind_arena = arena_with_overrides(
                profile_fixture.source,
                profile,
                Some((index, wrong_kind)),
                None,
                None,
                None,
            );
            assert_eq!(
                validate_arena(validator_profile, profile_fixture.source, &wrong_kind_arena,),
                Err(SourcePropertyImplementationError::InvalidArenaOwnership),
                "profile {profile:?} node {index} kind",
            );

            let wrong_context = if index == root_node(profile) {
                LocalTypeContextId::new(1)
            } else {
                LocalTypeContextId::new(0)
            };
            let wrong_context_arena = arena_with_overrides(
                profile_fixture.source,
                profile,
                None,
                None,
                Some((index, wrong_context)),
                None,
            );
            assert_eq!(
                validate_arena(
                    validator_profile,
                    profile_fixture.source,
                    &wrong_context_arena,
                ),
                Err(SourcePropertyImplementationError::InvalidArenaOwnership),
                "profile {profile:?} node {index} context",
            );
        }
        let rootless = arena_with_overrides(
            profile_fixture.source,
            profile,
            None,
            None,
            None,
            Some(None),
        );
        assert_eq!(
            validate_arena(validator_profile, profile_fixture.source, &rootless),
            Err(SourcePropertyImplementationError::InvalidArenaOwnership),
            "profile {profile:?} root",
        );
    }

    let mut drafts = obligation_drafts(projection.initial_obligations());
    drafts[0].goal = InitialObligationGoal::new("forged");
    assert_eq!(
        fixture.validate_handoff(
            projection.handoff(),
            &table_from_drafts(drafts),
            &fixture.arena,
        ),
        Err(SourcePropertyImplementationError::InvalidObligation)
    );

    for kind in [
        InitialObligationKind::PredicatePropertyCorrectness,
        InitialObligationKind::FunctorExistence,
        InitialObligationKind::FunctorUniqueness,
        InitialObligationKind::PropertyImplementationExistence,
        InitialObligationKind::PropertyImplementationUniqueness,
    ] {
        let mut draft = unrelated_draft(fixture.source);
        draft.kind = kind;
        let mut baseline = InitialObligationTable::new();
        baseline.insert(draft);
        assert_eq!(
            fixture.build(&baseline),
            Err(SourcePropertyImplementationError::InvalidObligation)
        );
    }
}

#[test]
fn task_264_typed_installation_is_transactional_with_nonempty_baseline() {
    let fixture = fixture(TestProfile::Means);
    let base = unrelated_baseline(fixture.source);
    let projection = fixture.projection(&base);
    assert_eq!(projection.base_initial_obligations(), &base);
    assert_eq!(projection.initial_obligations().len(), 3);
    let baseline = fixture.typed_base(base.clone());
    let untouched = baseline.clone();
    let typed = baseline
        .clone()
        .with_source_property_implementation(projection.clone())
        .expect("Task 264 one-shot installation");
    assert_eq!(baseline, untouched);
    assert!(baseline.source_property_implementation().is_none());
    assert_eq!(
        typed.source_property_implementation(),
        Some(projection.handoff())
    );
    assert_eq!(
        typed.initial_obligations(),
        projection.initial_obligations()
    );
    assert!(matches!(
        typed
            .clone()
            .with_source_property_implementation(projection.clone()),
        Err(TypedAstError::InvalidSourcePropertyImplementation)
    ));
    let stale = fixture.typed_base(InitialObligationTable::new());
    assert!(matches!(
        stale.with_source_property_implementation(projection),
        Err(TypedAstError::InvalidSourcePropertyImplementation)
    ));
}

#[test]
fn task_264_final_clone_orphan_extra_task259_isolation_and_no_semantics() {
    let fixture = fixture(TestProfile::Means);
    let projection = fixture.projection(&InitialObligationTable::new());
    let typed = fixture
        .typed_base(InitialObligationTable::new())
        .with_source_property_implementation(projection.clone())
        .expect("Task 264 typed installation");
    let resolved = assemble_empty(&typed).expect("Task 264 final assembly");
    assert_eq!(
        resolved.source_property_implementation(),
        typed.source_property_implementation()
    );
    assert_eq!(
        resolved.debug_text(),
        assemble_empty(&typed)
            .expect("deterministic replay")
            .debug_text()
    );
    assert!(typed.types().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.diagnostics().is_empty());
    assert!(resolved.expr_metadata().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.diagnostics().is_empty());

    let mut missing = obligation_drafts(projection.initial_obligations());
    missing.pop();
    assert_eq!(
        fixture.validate_handoff(
            projection.handoff(),
            &table_from_drafts(missing),
            &fixture.arena,
        ),
        Err(SourcePropertyImplementationError::InvalidObligation)
    );
    let mut swapped = obligation_drafts(projection.initial_obligations());
    swapped.swap(0, 1);
    assert_eq!(
        fixture.validate_handoff(
            projection.handoff(),
            &table_from_drafts(swapped),
            &fixture.arena,
        ),
        Err(SourcePropertyImplementationError::InvalidObligation)
    );
    let equals = self::fixture(TestProfile::Equals);
    let equals_projection = equals.projection(&InitialObligationTable::new());
    let mut injected = InitialObligationTable::new();
    injected.insert(InitialObligationDraft {
        kind: InitialObligationKind::PropertyImplementationExistence,
        owner: TypedSiteRef::Node(TypedNodeId::new(47)),
        source_range: range(equals.source, 121, 145),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new("injected"),
        provenance: InitialObligationProvenance::new("injected"),
        status: InitialObligationStatus::Pending,
    });
    assert_eq!(
        equals.validate_handoff(equals_projection.handoff(), &injected, &equals.arena,),
        Err(SourcePropertyImplementationError::InvalidObligation)
    );

    for kind in [
        InitialObligationKind::PropertyImplementationExistence,
        InitialObligationKind::PropertyImplementationUniqueness,
    ] {
        let mut orphan = InitialObligationTable::new();
        orphan.insert(InitialObligationDraft {
            kind,
            owner: TypedSiteRef::Node(TypedNodeId::new(76)),
            source_range: range(fixture.source, 183, 218),
            assumptions: Vec::new(),
            goal: InitialObligationGoal::new("orphan"),
            provenance: InitialObligationProvenance::new("orphan"),
            status: InitialObligationStatus::Pending,
        });
        assert!(matches!(
            TypedAst::try_new(fixture.typed_parts(orphan)),
            Err(TypedAstError::InvalidSourcePropertyImplementation)
        ));
    }

    let mut extra = typed.clone();
    let mut extra_obligations = extra.initial_obligations().clone();
    extra_obligations.insert(unrelated_draft(fixture.source));
    extra.replace_initial_obligations_for_test(extra_obligations);
    assert!(matches!(
        assemble_empty(&extra),
        Err(ResolvedTypedAstError::InvalidSourcePropertyImplementation)
    ));

    let (task259, _) =
        crate::source_functor_definition::tests::actual_definition_family_typed_asts_for_task261();
    assert!(task259.source_predicate_definition().is_some());
    assert!(task259.source_property_implementation().is_none());
    assert!(typed.source_predicate_definition().is_none());
    let mut occupied = fixture.typed_base(InitialObligationTable::new());
    occupied.inject_source_predicate_definition_for_test(
        task259
            .source_predicate_definition()
            .expect("Task 259 handoff")
            .clone(),
    );
    let fresh = fixture.projection(&InitialObligationTable::new());
    assert!(matches!(
        occupied.with_source_property_implementation(fresh),
        Err(TypedAstError::InvalidSourcePropertyImplementation)
    ));
}

fn fixture(profile: TestProfile) -> Fixture {
    let source = source_id(profile);
    let module = ModuleId::new(
        PackageId::new("pkg"),
        ModulePath::new(match profile {
            TestProfile::Means => "task264.property_implementation.means",
            TestProfile::Equals => "task264.property_implementation.equals",
        }),
    );
    let arena = arena(source, profile, None);
    let shell = property_shell(source, &module, profile);
    let context = source_context(source, module.clone(), shell, profile);
    let (env, identities) = resolver_env(source, module.clone());
    let source_type = source_type(
        source,
        module.clone(),
        &identities,
        &context,
        &arena,
        profile,
        &env,
    );
    let terms = primary_terms(source, module.clone(), profile, &context, &arena);
    let structure = (profile == TestProfile::Equals).then(|| {
        structure(
            source,
            module.clone(),
            &env,
            &context,
            &terms,
            &arena,
            false,
        )
    });
    let formula = (profile == TestProfile::Means)
        .then(|| atomic_formula(source, module.clone(), &env, &context, &terms, &arena));
    let input = checker_input(source, module.clone(), profile, shell, &identities);
    Fixture {
        profile,
        source,
        module,
        env,
        context,
        source_type,
        terms,
        structure,
        formula,
        arena,
        input,
    }
}

fn source_id(profile: TestProfile) -> SourceId {
    let byte = if profile == TestProfile::Means {
        "64"
    } else {
        "65"
    };
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        byte.repeat(Hash::BYTE_LEN)
    ))
    .expect("Task 264 snapshot");
    let allocator = InMemorySessionIdAllocator::new();
    if profile == TestProfile::Equals {
        allocator
            .next_source_id(snapshot)
            .expect("Task 264 reserved source");
    }
    allocator.next_source_id(snapshot).expect("Task 264 source")
}

fn property_shell(source: SourceId, module: &ModuleId, profile: TestProfile) -> DeclarationShellId {
    let end = profile_end(profile);
    let mut builder = SurfaceAstBuilder::new(source);
    let field = builder.add_node(
        SurfaceNodeKind::StructureField,
        range(source, 45, 66),
        Vec::new(),
    );
    let property = builder.add_node(
        SurfaceNodeKind::StructureProperty,
        range(source, 71, 94),
        Vec::new(),
    );
    let structure = builder.add_node(
        SurfaceNodeKind::StructureDefinition,
        range(source, 13, 101),
        vec![field, property],
    );
    let block = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source, 0, 106),
        vec![structure],
    );
    let implementation = builder.add_node(
        SurfaceNodeKind::PropertyImplementation,
        range(source, 108, end),
        Vec::new(),
    );
    let items = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source, 0, end),
        vec![block, implementation],
    );
    let unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source, 0, end),
        vec![items],
    );
    let root = builder.add_node(SurfaceNodeKind::Root, range(source, 0, end), vec![unit]);
    let shells =
        DeclarationShellCollector::new(&builder.finish(Some(root), None), module).collect();
    assert_eq!(shells.declarations().len(), 5);
    let shell = shells.declarations()[4].id();
    assert_eq!(shell.index(), 4);
    shell
}

fn resolver_env(source: SourceId, module: ModuleId) -> (SymbolEnv, [ResolverIdentity; 3]) {
    resolver_env_with_mutation(source, module, Task264ResolverEnvMutation::None)
}

fn resolver_env_with_spelling_override(
    source: SourceId,
    module: ModuleId,
    spelling_override: Option<(usize, &str)>,
) -> (SymbolEnv, [ResolverIdentity; 3]) {
    resolver_env_with_mutation(
        source,
        module,
        spelling_override.map_or(Task264ResolverEnvMutation::None, |(role, spelling)| {
            Task264ResolverEnvMutation::Spelling(role, spelling)
        }),
    )
}

#[derive(Clone, Copy)]
enum Task264ResolverEnvMutation<'a> {
    None,
    Spelling(usize, &'a str),
    FieldPrivate,
    FieldLocalOnly,
    FieldWrongOrigin,
    MissingFieldEffects,
    ExtraContribution,
    ExtraSymbol,
    ExtraDefinition,
}

fn resolver_env_with_mutation(
    source: SourceId,
    module: ModuleId,
    mutation: Task264ResolverEnvMutation<'_>,
) -> (SymbolEnv, [ResolverIdentity; 3]) {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 13, 101)),
    );
    let specs = [
        (
            "Task264Carrier",
            SymbolKind::Structure,
            DefinitionKind::Structure,
            13,
            101,
            vec![4, 0, 11, 0],
        ),
        (
            "carrier",
            SymbolKind::Selector,
            DefinitionKind::Selector,
            45,
            66,
            vec![4, 0, 11, 0, 18, 0],
        ),
        (
            "marker",
            SymbolKind::Selector,
            DefinitionKind::Selector,
            71,
            94,
            vec![4, 0, 11, 0, 19, 1],
        ),
    ];
    let mut identities = Vec::new();
    for (index, (spelling, symbol_kind, definition_kind, start, end, path)) in
        specs.into_iter().enumerate()
    {
        let spelling = match mutation {
            Task264ResolverEnvMutation::Spelling(role, value) if role == index => value,
            _ => spelling,
        };
        let origin_start =
            if index == 1 && matches!(mutation, Task264ResolverEnvMutation::FieldWrongOrigin) {
                start + 1
            } else {
                start
            };
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, origin_start, end)),
            path,
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("task264-{index}-{spelling}")),
            FullyQualifiedName::new(format!("pkg::task264::{spelling}")),
        );
        let mut symbol_entry = SymbolEntry::new(
            symbol.clone(),
            symbol_kind,
            NamespacePath::new(module.path().as_str()),
            spelling,
            origin.clone(),
            contribution,
        );
        symbol_entry = symbol_entry.with_visibility(
            if index == 1 && matches!(mutation, Task264ResolverEnvMutation::FieldPrivate) {
                Visibility::Private
            } else {
                Visibility::Public
            },
        );
        symbol_entry = symbol_entry.with_export_status(
            if index == 1 && matches!(mutation, Task264ResolverEnvMutation::FieldLocalOnly) {
                ExportStatus::LocalOnly
            } else {
                ExportStatus::Exported
            },
        );
        indexes.symbols.insert(symbol_entry);
        let definition = indexes.definitions.insert(
            DefinitionShell::new(symbol.clone(), definition_kind, origin, contribution)
                .with_visibility(
                    if index == 1 && matches!(mutation, Task264ResolverEnvMutation::FieldPrivate) {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    },
                ),
        );
        assert_eq!(definition.index(), index);
        if index != 1 || !matches!(mutation, Task264ResolverEnvMutation::MissingFieldEffects) {
            indexes
                .contributions
                .add_symbol(contribution, symbol.clone());
            indexes
                .contributions
                .add_definition(contribution, definition);
        }
        identities.push((symbol, definition, contribution));
    }
    if matches!(mutation, Task264ResolverEnvMutation::ExtraContribution) {
        indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
    }
    if matches!(mutation, Task264ResolverEnvMutation::ExtraSymbol) {
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 0, 1)),
            vec![9],
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new("task264-extra"),
            FullyQualifiedName::new("pkg::task264::extra"),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Selector,
                NamespacePath::new(module.path().as_str()),
                "extra",
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        indexes.contributions.add_symbol(contribution, symbol);
    }
    if matches!(mutation, Task264ResolverEnvMutation::ExtraDefinition) {
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 0, 1)),
            vec![10],
        );
        let definition = indexes.definitions.insert(
            DefinitionShell::new(
                identities[1].0.clone(),
                DefinitionKind::Selector,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        indexes
            .contributions
            .add_definition(contribution, definition);
    }
    (
        SymbolEnv::new(module, indexes),
        identities.try_into().expect("three Task 264 identities"),
    )
}

fn alternate_contribution(source: SourceId, module: &ModuleId) -> SourceContributionId {
    let mut indexes = SymbolEnvIndexes::default();
    indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 1)),
    );
    indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 1, 2)),
    )
}

fn source_context(
    source: SourceId,
    module: ModuleId,
    shell: DeclarationShellId,
    profile: TestProfile,
) -> SourceBindingContextHandoff {
    let scope = LocalTermScope::new(vec![4]);
    let input = SourceBindingContextInput {
        source_id: source,
        module_id: module.clone(),
        module_site: node_site(root_node(profile)),
        items: vec![SourceItemInput {
            shell,
            shell_ordinal: 4,
            role: SourceItemRole::PropertyImplementation,
            module_id: module,
            source_range: range(source, 108, profile_end(profile)),
            parent: None,
            visibility: SourceItemVisibility::Unspecified,
            site: node_site(owner_node(profile)),
            local_scope: Some(scope.clone()),
            recovery: SourceItemRecovery::Normal,
        }],
        bindings: vec![SourceBindingSiteInput {
            shell,
            context_owner: SourceBindingContextOwner::Shell(shell),
            source_ordinal: 0,
            spelling: "M".to_owned(),
            declaration_range: range(source, 125, 126),
            written_type_range: range(source, 130, 144),
            site: node_site(parameter_node(profile)),
            role: SourceBindingSiteRole::DefinitionParameter {
                local: LocalTermBinding::new("M", scope, range(source, 125, 126), 0),
            },
            recovery: BindingRecoveryState::Normal,
        }],
    };
    match SourceBindingContextProducer::build(input).expect("Task 248P context") {
        SourceBindingContextBuild::Complete(row) => row.into_handoff(),
        _ => panic!("Task 248P context must be complete"),
    }
}

fn source_type(
    source: SourceId,
    module: ModuleId,
    identities: &[ResolverIdentity; 3],
    context: &SourceBindingContextHandoff,
    arena: &TypedArena,
    profile: TestProfile,
    env: &SymbolEnv,
) -> SourceTypeApplicationHandoff {
    let (expression_node, head_node, member_specs) = match profile {
        TestProfile::Means => (
            63,
            64,
            [(56, 45, 66, 55, 54, 62, 65), (59, 71, 94, 58, 57, 90, 93)],
        ),
        TestProfile::Equals => (
            45,
            46,
            [(38, 45, 66, 37, 36, 62, 65), (41, 71, 94, 40, 39, 90, 93)],
        ),
    };
    let base = SourceTypeProducer::build(
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            }],
            expressions: vec![SourceTypeExpressionInput {
                source_id: source,
                module_id: module.clone(),
                site: node_site(expression_node),
                source_range: range(source, 130, 144),
                spelling: "Task264Carrier".to_owned(),
                head_site: node_site(head_node),
                head_range: range(source, 130, 144),
                head_spelling: "Task264Carrier".to_owned(),
                form: SourceTypeApplicationForm::Bare,
                head: SourceTypeHead::Symbol {
                    symbol: identities[0].0.clone(),
                    contribution: identities[0].2,
                },
                recovery: NodeRecoveryState::Normal,
            }],
            arguments: Vec::new(),
        },
        context.binding_env(),
        env,
        arena,
    )
    .expect("Task 249 source type");
    let members = member_specs
        .into_iter()
        .enumerate()
        .map(
            |(
                source_ordinal,
                (member_node, member_start, member_end, expression_node, head_node, start, end),
            )| SourceTypeStructureMemberInput {
                member_site: node_site(member_node),
                member_range: range(source, member_start, member_end),
                source_ordinal,
                expression: SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: node_site(expression_node),
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: node_site(head_node),
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                },
            },
        )
        .collect();
    SourceTypeStructureMemberProducer::extend_property_implementation(
        &base,
        SourceTypeStructureMemberHandoffInput {
            source_id: source,
            module_id: module,
            members,
        },
        arena,
    )
    .expect("Task 249PI source type")
}

fn primary_terms(
    source: SourceId,
    module: ModuleId,
    profile: TestProfile,
    context: &SourceBindingContextHandoff,
    arena: &TypedArena,
) -> SourcePrimaryTermHandoff {
    let (rows, references) = match profile {
        TestProfile::Means => (
            vec![
                (
                    66,
                    172,
                    174,
                    "it",
                    SourcePrimaryTermKind::It,
                    SourcePrimaryTermRole::CurrentDefinitionResult,
                ),
                (
                    68,
                    177,
                    179,
                    "it",
                    SourcePrimaryTermKind::It,
                    SourcePrimaryTermRole::CurrentDefinitionResult,
                ),
            ],
            Vec::new(),
        ),
        TestProfile::Equals => (
            vec![(
                48,
                173,
                174,
                "M",
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermRole::Value,
            )],
            vec![SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(0),
                binding: BindingId::new(0),
                role: SourcePrimaryTermReferenceRole::Variable,
            }],
        ),
    };
    let terms = rows
        .into_iter()
        .enumerate()
        .map(
            |(source_ordinal, (node, start, end, spelling, kind, role))| SourcePrimaryTermInput {
                site: node_site(node),
                source_range: range(source, start, end),
                source_ordinal,
                context: BindingContextId::new(1),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: spelling.to_owned(),
                kind,
                role,
                parent: None,
            },
        )
        .collect::<Vec<_>>();
    SourcePrimaryTermProducer::build(
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module,
            terms,
            references,
            numeric_type_requests: Vec::new(),
        },
        context.binding_env(),
        arena,
    )
    .expect("Task 252 terms")
}

fn structure(
    source: SourceId,
    module: ModuleId,
    env: &SymbolEnv,
    context: &SourceBindingContextHandoff,
    terms: &SourcePrimaryTermHandoff,
    arena: &TypedArena,
    wrong: bool,
) -> SourceStructureHandoff {
    SourceStructureProducer::build(
        SourceStructureHandoffInput {
            source_id: source,
            module_id: module,
            terms: vec![SourceStructureTermInput {
                site: node_site(49),
                source_range: range(source, 173, 182),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceStructureRecovery::Normal,
                spelling: "M.carrier".to_owned(),
                kind: SourceStructureTermKind::SelectorAccess,
            }],
            wrappers: Vec::new(),
            roots: Vec::new(),
            members: vec![SourceStructureMemberInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                site: node_site(31),
                source_range: range(source, 175, 182),
                spelling: if wrong { "marker" } else { "carrier" }.to_owned(),
                role: SourceStructureMemberRole::Selector,
                parent: None,
            }],
            field_updates: Vec::new(),
            edges: vec![SourceStructureEdgeInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                role: SourceStructureEdgeRole::SelectorBase,
                member: None,
                target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
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
        env,
        context.binding_env(),
        terms,
        None,
        arena,
    )
    .expect("Task 254 structure")
}

fn atomic_formula(
    source: SourceId,
    module: ModuleId,
    env: &SymbolEnv,
    context: &SourceBindingContextHandoff,
    terms: &SourcePrimaryTermHandoff,
    arena: &TypedArena,
) -> SourceAtomicFormulaHandoff {
    SourceAtomicFormulaProducer::build(
        SourceAtomicFormulaHandoffInput {
            source_id: source,
            module_id: module,
            formulas: vec![SourceAtomicFormulaInput {
                site: node_site(70),
                source_range: range(source, 172, 179),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "it = it".to_owned(),
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
        context.binding_env(),
        env,
        terms,
        None,
        None,
        None,
        arena,
    )
    .expect("Task 256 formula")
}

fn checker_input(
    source: SourceId,
    module: ModuleId,
    profile: TestProfile,
    shell: DeclarationShellId,
    identities: &[ResolverIdentity; 3],
) -> SourcePropertyImplementationHandoffInput {
    let (
        owner,
        end,
        parameter,
        style,
        spelling,
        definiens_node,
        definiens_range,
        definiens_spelling,
        target,
        correctness,
    ) = match profile {
        TestProfile::Means => (
            81,
            262,
            65,
            SourcePropertyImplementationStyle::Means,
            "definition\n  let M be Task264Carrier;\n  property M.marker means it = it;\n  existence by computation(steps: 1);\n  uniqueness by computation(steps: 1);\nend;",
            72,
            (172, 179),
            "it = it",
            SourcePropertyDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(0)),
            vec![
                (
                    SourcePropertyCorrectnessKind::Existence,
                    76,
                    183,
                    218,
                    193,
                    217,
                    "existence by computation(steps: 1);",
                ),
                (
                    SourcePropertyCorrectnessKind::Uniqueness,
                    80,
                    221,
                    257,
                    232,
                    256,
                    "uniqueness by computation(steps: 1);",
                ),
            ],
        ),
        TestProfile::Equals => (
            52,
            188,
            47,
            SourcePropertyImplementationStyle::Equals,
            "definition\n  let M be Task264Carrier;\n  property M.marker equals M.carrier;\nend;",
            51,
            (173, 182),
            "M.carrier",
            SourcePropertyDefiniensTarget::Structure(SourceStructureTermId::new(0)),
            Vec::new(),
        ),
    };
    SourcePropertyImplementationHandoffInput {
        source_id: source,
        module_id: module,
        implementations: vec![SourcePropertyImplementationInput {
            shell,
            site: node_site(owner),
            source_range: range(source, 108, end),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourcePropertyImplementationRecovery::Normal,
            spelling: spelling.to_owned(),
            style,
            parameter: SourcePropertyParameterId::new(0),
            target: SourcePropertyTargetId::new(0),
            definiens: SourcePropertyDefiniensId::new(0),
        }],
        parameters: vec![SourcePropertyParameterInput {
            owner: SourcePropertyImplementationId::new(0),
            ordinal: 0,
            binding: BindingId::new(0),
            written_type: SourceTypeApplicationId::new(0),
            site: node_site(parameter),
            source_range: range(source, 121, 145),
            declaration_range: range(source, 125, 126),
            context: BindingContextId::new(1),
            recovery: SourcePropertyImplementationRecovery::Normal,
            spelling: "let M be Task264Carrier;".to_owned(),
        }],
        targets: vec![SourcePropertyTargetInput {
            owner: SourcePropertyImplementationId::new(0),
            ordinal: 0,
            subject: BindingId::new(0),
            symbol: identities[2].0.clone(),
            definition: identities[2].1,
            contribution: identities[2].2,
            site: TypedSiteRef::Role {
                node: TypedNodeId::new(owner),
                role: TypeRole::new("source.property-implementation.target"),
            },
            source_range: range(source, 157, 165),
            subject_range: range(source, 157, 158),
            name_range: range(source, 159, 165),
            spelling: "M.marker".to_owned(),
            return_type: SourceTypeStructureMemberId::new(1),
        }],
        definientia: vec![SourcePropertyDefiniensInput {
            owner: SourcePropertyImplementationId::new(0),
            ordinal: 0,
            target,
            site: node_site(definiens_node),
            source_range: range(source, definiens_range.0, definiens_range.1),
            context: BindingContextId::new(1),
            recovery: SourcePropertyImplementationRecovery::Normal,
            spelling: definiens_spelling.to_owned(),
        }],
        correctness: correctness
            .into_iter()
            .enumerate()
            .map(
                |(ordinal, (kind, node, start, end, proof_start, proof_end, spelling))| {
                    SourcePropertyCorrectnessInput {
                        owner: SourcePropertyImplementationId::new(0),
                        ordinal,
                        kind,
                        site: node_site(node),
                        source_range: range(source, start, end),
                        justification: SourceAnchor::Range(range(source, proof_start, proof_end)),
                        recovery: SourcePropertyImplementationRecovery::Normal,
                        spelling: spelling.to_owned(),
                    }
                },
            )
            .collect(),
    }
}

fn arena(
    source: SourceId,
    profile: TestProfile,
    kind_override: Option<(usize, &'static str)>,
) -> TypedArena {
    arena_with_overrides(source, profile, kind_override, None, None, None)
}

fn arena_with_range_override(
    source: SourceId,
    profile: TestProfile,
    kind_override: Option<(usize, &'static str)>,
    range_override: Option<(usize, usize, usize)>,
) -> TypedArena {
    arena_with_overrides(source, profile, kind_override, range_override, None, None)
}

fn arena_with_overrides(
    source: SourceId,
    profile: TestProfile,
    kind_override: Option<(usize, &'static str)>,
    range_override: Option<(usize, usize, usize)>,
    context_override: Option<(usize, LocalTypeContextId)>,
    root_override: Option<Option<usize>>,
) -> TypedArena {
    let max = root_node(profile);
    let mut nodes = Vec::with_capacity(max + 1);
    for index in 0..=max {
        let (start, end, kind) = owned_node(profile, index).unwrap_or_else(|| {
            let [start, end] = surface_range(profile, index);
            (start, end, "source.surface.unowned")
        });
        let kind = kind_override
            .filter(|(node, _)| *node == index)
            .map_or(kind, |(_, kind)| kind);
        let (start, end) = range_override
            .filter(|(node, _, _)| *node == index)
            .map_or((start, end), |(_, start, end)| (start, end));
        let context = if index == max {
            LocalTypeContextId::new(0)
        } else {
            LocalTypeContextId::new(1)
        };
        let context = context_override
            .filter(|(node, _)| *node == index)
            .map_or(context, |(_, context)| context);
        nodes.push(
            TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal)
                .with_links(TypedNodeLinks {
                    context: Some(context),
                    ..TypedNodeLinks::default()
                }),
        );
    }
    let root = root_override.unwrap_or(Some(max)).map(TypedNodeId::new);
    TypedArena::try_new(root, nodes).expect("Task 264 arena")
}

fn surface_range(profile: TestProfile, index: usize) -> [usize; 2] {
    const MEANS: &[[usize; 2]] = &[
        [0, 10],
        [13, 19],
        [20, 34],
        [35, 40],
        [45, 50],
        [51, 58],
        [59, 61],
        [62, 65],
        [65, 66],
        [71, 79],
        [80, 86],
        [87, 89],
        [90, 93],
        [93, 94],
        [97, 100],
        [100, 101],
        [102, 105],
        [105, 106],
        [108, 118],
        [121, 124],
        [125, 126],
        [127, 129],
        [130, 144],
        [144, 145],
        [148, 156],
        [157, 158],
        [158, 159],
        [159, 165],
        [166, 171],
        [172, 174],
        [175, 176],
        [177, 179],
        [179, 180],
        [183, 192],
        [193, 195],
        [196, 207],
        [207, 208],
        [208, 213],
        [213, 214],
        [215, 216],
        [216, 217],
        [217, 218],
        [221, 231],
        [232, 234],
        [235, 246],
        [246, 247],
        [247, 252],
        [252, 253],
        [254, 255],
        [255, 256],
        [256, 257],
        [258, 261],
        [261, 262],
        [20, 34],
        [62, 65],
        [62, 65],
        [45, 66],
        [90, 93],
        [90, 93],
        [71, 94],
        [13, 101],
        [0, 106],
        [130, 144],
        [130, 144],
        [130, 144],
        [121, 145],
        [172, 174],
        [172, 174],
        [177, 179],
        [177, 179],
        [172, 179],
        [172, 179],
        [172, 179],
        [208, 216],
        [196, 217],
        [193, 217],
        [183, 218],
        [247, 255],
        [235, 256],
        [232, 256],
        [221, 257],
        [108, 262],
        [0, 262],
        [0, 262],
        [0, 262],
    ];
    const EQUALS: &[[usize; 2]] = &[
        [0, 10],
        [13, 19],
        [20, 34],
        [35, 40],
        [45, 50],
        [51, 58],
        [59, 61],
        [62, 65],
        [65, 66],
        [71, 79],
        [80, 86],
        [87, 89],
        [90, 93],
        [93, 94],
        [97, 100],
        [100, 101],
        [102, 105],
        [105, 106],
        [108, 118],
        [121, 124],
        [125, 126],
        [127, 129],
        [130, 144],
        [144, 145],
        [148, 156],
        [157, 158],
        [158, 159],
        [159, 165],
        [166, 172],
        [173, 174],
        [174, 175],
        [175, 182],
        [182, 183],
        [184, 187],
        [187, 188],
        [20, 34],
        [62, 65],
        [62, 65],
        [45, 66],
        [90, 93],
        [90, 93],
        [71, 94],
        [13, 101],
        [0, 106],
        [130, 144],
        [130, 144],
        [130, 144],
        [121, 145],
        [173, 174],
        [173, 182],
        [173, 182],
        [173, 182],
        [108, 188],
        [0, 188],
        [0, 188],
        [0, 188],
    ];
    match profile {
        TestProfile::Means => MEANS[index],
        TestProfile::Equals => EQUALS[index],
    }
}

fn owned_node(profile: TestProfile, index: usize) -> Option<(usize, usize, &'static str)> {
    let row = match (profile, index) {
        (TestProfile::Means, 54 | 57 | 64) => (
            if index == 54 {
                62
            } else if index == 57 {
                90
            } else {
                130
            },
            if index == 54 {
                65
            } else if index == 57 {
                93
            } else {
                144
            },
            "source.type.head",
        ),
        (TestProfile::Means, 55 | 58 | 63) => (
            if index == 55 {
                62
            } else if index == 58 {
                90
            } else {
                130
            },
            if index == 55 {
                65
            } else if index == 58 {
                93
            } else {
                144
            },
            "source.type.expression",
        ),
        (TestProfile::Means, 56) => (45, 66, "source.definition.structure.member"),
        (TestProfile::Means, 59) => (71, 94, "source.definition.structure.member"),
        (TestProfile::Means, 60) => (13, 101, "source.definition.structure"),
        (TestProfile::Means, 65) => (
            125,
            126,
            "source.definition.property-implementation.parameter",
        ),
        (TestProfile::Means, 66) => (172, 174, "source.term.it"),
        (TestProfile::Means, 68) => (177, 179, "source.term.it"),
        (TestProfile::Means, 70) => (172, 179, "source.formula.atomic.equality"),
        (TestProfile::Means, 72) => (
            172,
            179,
            "source.definition.property-implementation.definiens",
        ),
        (TestProfile::Means, 76) => (
            183,
            218,
            "source.definition.property-implementation.correctness",
        ),
        (TestProfile::Means, 80) => (
            221,
            257,
            "source.definition.property-implementation.correctness",
        ),
        (TestProfile::Means, 81) => (108, 262, "source.definition.property-implementation"),
        (TestProfile::Means, 84) => (0, 262, "source.module"),
        (TestProfile::Equals, 36 | 39 | 46) => (
            if index == 36 {
                62
            } else if index == 39 {
                90
            } else {
                130
            },
            if index == 36 {
                65
            } else if index == 39 {
                93
            } else {
                144
            },
            "source.type.head",
        ),
        (TestProfile::Equals, 37 | 40 | 45) => (
            if index == 37 {
                62
            } else if index == 40 {
                90
            } else {
                130
            },
            if index == 37 {
                65
            } else if index == 40 {
                93
            } else {
                144
            },
            "source.type.expression",
        ),
        (TestProfile::Equals, 38) => (45, 66, "source.definition.structure.member"),
        (TestProfile::Equals, 41) => (71, 94, "source.definition.structure.member"),
        (TestProfile::Equals, 42) => (13, 101, "source.definition.structure"),
        (TestProfile::Equals, 47) => (
            125,
            126,
            "source.definition.property-implementation.parameter",
        ),
        (TestProfile::Equals, 31) => (175, 182, "source.term.structure.member.selector"),
        (TestProfile::Equals, 48) => (173, 174, "source.term.variable-reference"),
        (TestProfile::Equals, 49) => (173, 182, "source.term.structure.selector"),
        (TestProfile::Equals, 51) => (
            173,
            182,
            "source.definition.property-implementation.definiens",
        ),
        (TestProfile::Equals, 52) => (108, 188, "source.definition.property-implementation"),
        (TestProfile::Equals, 55) => (0, 188, "source.module"),
        _ => return None,
    };
    Some(row)
}

const fn owner_node(profile: TestProfile) -> usize {
    match profile {
        TestProfile::Means => 81,
        TestProfile::Equals => 52,
    }
}

const fn parameter_node(profile: TestProfile) -> usize {
    match profile {
        TestProfile::Means => 65,
        TestProfile::Equals => 47,
    }
}

const fn root_node(profile: TestProfile) -> usize {
    match profile {
        TestProfile::Means => 84,
        TestProfile::Equals => 55,
    }
}

const fn profile_end(profile: TestProfile) -> usize {
    match profile {
        TestProfile::Means => 262,
        TestProfile::Equals => 188,
    }
}

fn node_site(index: usize) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(index))
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

fn unrelated_draft(source: SourceId) -> InitialObligationDraft {
    InitialObligationDraft {
        kind: InitialObligationKind::Sethood,
        owner: TypedSiteRef::Node(TypedNodeId::new(54)),
        source_range: range(source, 62, 65),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new("unrelated:sethood"),
        provenance: InitialObligationProvenance::new("unrelated:baseline"),
        status: InitialObligationStatus::Pending,
    }
}

fn unrelated_baseline(source: SourceId) -> InitialObligationTable {
    table_from_drafts(vec![unrelated_draft(source)])
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
