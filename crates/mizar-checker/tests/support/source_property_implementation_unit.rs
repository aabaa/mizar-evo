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
    fn attach_native_children(&mut self, source: &TypedAst) {
        self.arena = TypedArena::try_new(
            self.arena.root(),
            self.arena
                .iter()
                .map(|(id, node)| {
                    node.clone()
                        .with_children(source.nodes().node(id).unwrap().children.clone())
                })
                .collect(),
        )
        .unwrap();
    }

    fn build_native(
        &self,
        base: &InitialObligationTable,
        source: &TypedAst,
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
            source,
        )
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

#[test]
fn task_264d_equals_selector_identity_is_exact_and_deterministic() {
    let mut fixture = fixture(TestProfile::Equals);
    let source = native_property_source(&fixture, NativePropertySourceMutation::None);
    fixture.attach_native_children(&source);
    let projection = fixture
        .build_native(&InitialObligationTable::new(), &source)
        .unwrap();
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
    let mut equals = fixture(TestProfile::Equals);
    let source = native_property_source(&equals, NativePropertySourceMutation::None);
    equals.attach_native_children(&source);
    let projection = equals
        .build_native(&InitialObligationTable::new(), &source)
        .unwrap();
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
        let (wrong_env, _) = resolver_env_with_mutation(
            equals.source,
            equals.module.clone(),
            [
                range(equals.source, 13, 101),
                range(equals.source, 45, 66),
                range(equals.source, 71, 94),
            ],
            mutation,
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
    }
    for mutation in [
        Task264ResolverEnvMutation::ExtraContribution,
        Task264ResolverEnvMutation::ExtraSymbol,
        Task264ResolverEnvMutation::ExtraDefinition,
    ] {
        let (wrong_env, _) = resolver_env_with_mutation(
            equals.source,
            equals.module.clone(),
            [
                range(equals.source, 13, 101),
                range(equals.source, 45, 66),
                range(equals.source, 71, 94),
            ],
            mutation,
        );
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
    let context = source_context(source, module.clone(), shell, profile, &arena);
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
    let input = checker_input(source, module.clone(), profile, shell, &identities, &arena);
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
    resolver_env_with_mutation(
        source,
        module,
        [
            range(source, 13, 101),
            range(source, 45, 66),
            range(source, 71, 94),
        ],
        Task264ResolverEnvMutation::None,
    )
}

fn resolver_env_with_spelling_override(
    source: SourceId,
    module: ModuleId,
    spelling_override: Option<(usize, &str)>,
) -> (SymbolEnv, [ResolverIdentity; 3]) {
    resolver_env_with_mutation(
        source,
        module,
        [
            range(source, 13, 101),
            range(source, 45, 66),
            range(source, 71, 94),
        ],
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
    origins: [SourceRange; 3],
    mutation: Task264ResolverEnvMutation<'_>,
) -> (SymbolEnv, [ResolverIdentity; 3]) {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(origins[0]),
    );
    let specs = [
        (
            "Task264Carrier",
            SymbolKind::Structure,
            DefinitionKind::Structure,
            origins[0].start,
            origins[0].end,
            vec![4, 0, 11, 0],
        ),
        (
            "carrier",
            SymbolKind::Selector,
            DefinitionKind::Selector,
            origins[1].start,
            origins[1].end,
            vec![4, 0, 11, 0, 18, 0],
        ),
        (
            "marker",
            SymbolKind::Selector,
            DefinitionKind::Selector,
            origins[2].start,
            origins[2].end,
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

fn source_context(
    source: SourceId,
    module: ModuleId,
    shell: DeclarationShellId,
    profile: TestProfile,
    arena: &TypedArena,
) -> SourceBindingContextHandoff {
    let SourceAnchor::Range(owner_range) = arena
        .node(TypedNodeId::new(owner_node(profile)))
        .expect("owner node")
        .anchor
    else {
        panic!("owner range")
    };
    let SourceAnchor::Range(declaration_range) = arena
        .node(TypedNodeId::new(20))
        .expect("declaration leaf")
        .anchor
    else {
        panic!("declaration range")
    };
    let SourceAnchor::Range(written_type_range) = arena
        .node(TypedNodeId::new(22))
        .expect("written Type leaf")
        .anchor
    else {
        panic!("written Type range")
    };
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
            source_range: owner_range,
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
            declaration_range,
            written_type_range,
            site: node_site(parameter_node(profile)),
            role: SourceBindingSiteRole::DefinitionParameter {
                local: LocalTermBinding::new("M", scope, declaration_range, 0),
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
        TestProfile::Means => (63, 64, [(56, 55, 54, 7), (59, 58, 57, 12)]),
        TestProfile::Equals => (45, 46, [(38, 37, 36, 7), (41, 40, 39, 12)]),
    };
    let written_type_range = context
        .declarations()
        .iter()
        .next()
        .expect("parameter declaration")
        .1
        .written_type_range;
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
                source_range: written_type_range,
                spelling: "Task264Carrier".to_owned(),
                head_site: node_site(head_node),
                head_range: written_type_range,
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
            |(source_ordinal, (member_node, expression_node, head_node, set_leaf))| {
                let SourceAnchor::Range(member_range) = arena
                    .node(TypedNodeId::new(member_node))
                    .expect("member owner")
                    .anchor
                else {
                    panic!("member range")
                };
                let SourceAnchor::Range(written_set_range) = arena
                    .node(TypedNodeId::new(set_leaf))
                    .expect("Set leaf")
                    .anchor
                else {
                    panic!("Set range")
                };
                SourceTypeStructureMemberInput {
                    member_site: node_site(member_node),
                    member_range,
                    source_ordinal,
                    expression: SourceTypeExpressionInput {
                        source_id: source,
                        module_id: module.clone(),
                        site: node_site(expression_node),
                        source_range: written_set_range,
                        spelling: "set".to_owned(),
                        head_site: node_site(head_node),
                        head_range: written_set_range,
                        head_spelling: "set".to_owned(),
                        form: SourceTypeApplicationForm::Bare,
                        head: SourceTypeHead::BuiltinSet,
                        recovery: NodeRecoveryState::Normal,
                    },
                }
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
                    "it",
                    SourcePrimaryTermKind::It,
                    SourcePrimaryTermRole::CurrentDefinitionResult,
                ),
                (
                    68,
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
        .map(|(source_ordinal, (node, spelling, kind, role))| {
            let SourceAnchor::Range(source_range) = arena
                .node(TypedNodeId::new(node))
                .expect("Primary owner")
                .anchor
            else {
                panic!("Primary range")
            };
            SourcePrimaryTermInput {
                site: node_site(node),
                source_range,
                source_ordinal,
                context: BindingContextId::new(1),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: spelling.to_owned(),
                kind,
                role,
                parent: None,
            }
        })
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
    let SourceAnchor::Range(term_range) = arena
        .node(TypedNodeId::new(49))
        .expect("Structure owner")
        .anchor
    else {
        panic!("Structure range")
    };
    let SourceAnchor::Range(member_range) = arena
        .node(TypedNodeId::new(31))
        .expect("selector owner")
        .anchor
    else {
        panic!("selector range")
    };
    SourceStructureProducer::build_canonical(
        SourceStructureHandoffInput {
            source_id: source,
            module_id: module,
            terms: vec![SourceStructureTermInput {
                site: node_site(49),
                source_range: term_range,
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
                source_range: member_range,
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
    let SourceAnchor::Range(formula_range) = arena
        .node(TypedNodeId::new(70))
        .expect("Atomic owner")
        .anchor
    else {
        panic!("Atomic range")
    };
    SourceAtomicFormulaProducer::build(
        SourceAtomicFormulaHandoffInput {
            source_id: source,
            module_id: module,
            formulas: vec![SourceAtomicFormulaInput {
                site: node_site(70),
                source_range: formula_range,
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
    arena: &TypedArena,
) -> SourcePropertyImplementationHandoffInput {
    let (
        owner,
        parameter,
        style,
        spelling,
        definiens_node,
        definiens_spelling,
        target,
        correctness,
    ) = match profile {
        TestProfile::Means => (
            81,
            65,
            SourcePropertyImplementationStyle::Means,
            "definition\n  let M be Task264Carrier;\n  property M.marker means it = it;\n  existence by computation(steps: 1);\n  uniqueness by computation(steps: 1);\nend;",
            72,
            "it = it",
            SourcePropertyDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(0)),
            vec![
                (
                    SourcePropertyCorrectnessKind::Existence,
                    76,
                    "existence by computation(steps: 1);",
                ),
                (
                    SourcePropertyCorrectnessKind::Uniqueness,
                    80,
                    "uniqueness by computation(steps: 1);",
                ),
            ],
        ),
        TestProfile::Equals => (
            52,
            47,
            SourcePropertyImplementationStyle::Equals,
            "definition\n  let M be Task264Carrier;\n  property M.marker equals M.carrier;\nend;",
            51,
            "M.carrier",
            SourcePropertyDefiniensTarget::Structure(SourceStructureTermId::new(0)),
            Vec::new(),
        ),
    };
    let ranges = arena
        .iter()
        .map(|(_, node)| {
            let SourceAnchor::Range(span) = node.anchor else {
                panic!("Property fixture range")
            };
            span
        })
        .collect::<Vec<_>>();
    SourcePropertyImplementationHandoffInput {
        source_id: source,
        module_id: module,
        implementations: vec![SourcePropertyImplementationInput {
            shell,
            site: node_site(owner),
            source_range: ranges[owner],
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
            source_range: range(source, ranges[19].start, ranges[23].end),
            declaration_range: ranges[20],
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
            source_range: range(source, ranges[25].start, ranges[27].end),
            subject_range: ranges[25],
            name_range: ranges[27],
            spelling: "M.marker".to_owned(),
            return_type: SourceTypeStructureMemberId::new(1),
        }],
        definientia: vec![SourcePropertyDefiniensInput {
            owner: SourcePropertyImplementationId::new(0),
            ordinal: 0,
            target,
            site: node_site(definiens_node),
            source_range: ranges[definiens_node],
            context: BindingContextId::new(1),
            recovery: SourcePropertyImplementationRecovery::Normal,
            spelling: definiens_spelling.to_owned(),
        }],
        correctness: correctness
            .into_iter()
            .enumerate()
            .map(
                |(ordinal, (kind, node, spelling))| SourcePropertyCorrectnessInput {
                    owner: SourcePropertyImplementationId::new(0),
                    ordinal,
                    kind,
                    site: node_site(node),
                    source_range: ranges[node],
                    justification: SourceAnchor::Range(ranges[node - 1]),
                    recovery: SourcePropertyImplementationRecovery::Normal,
                    spelling: spelling.to_owned(),
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

pub(crate) fn assemble_empty(
    typed_ast: &TypedAst,
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
        statement_semantics: None,
        statement_proofs: None,
    })
}

enum NativePropertySourceMutation {
    None,
    PropertyType,
    HeadToken,
    MemberName,
    BodyContent,
}

fn native_property_source(f: &Fixture, mutation: NativePropertySourceMutation) -> TypedAst {
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    use mizar_syntax::SurfaceTokenKind;
    let (text, tokens, nodes) = match f.profile {
        TestProfile::Means => (
            include_str!(
                "../../../../tests/miz/pass/types/pass_type_elaboration_property_implementation_means_payload_001.miz"
            ),
            "definition struct Task264Carrier where field carrier -> set ; property marker -> set ; end ; end ; definition let M be Task264Carrier ; property M . marker means it = it ; existence by computation ( steps : 1 ) ; uniqueness by computation ( steps : 1 ) ; end ;",
            vec![
                (SurfaceNodeKind::StructurePattern, vec![2]),
                (SurfaceNodeKind::TypeHead, vec![7]),
                (SurfaceNodeKind::TypeExpression, vec![54]),
                (SurfaceNodeKind::StructureField, vec![4, 5, 6, 55, 8]),
                (SurfaceNodeKind::TypeHead, vec![12]),
                (SurfaceNodeKind::TypeExpression, vec![57]),
                (SurfaceNodeKind::StructureProperty, vec![9, 10, 11, 58, 13]),
                (
                    SurfaceNodeKind::StructureDefinition,
                    vec![1, 53, 3, 56, 59, 14, 15],
                ),
                (SurfaceNodeKind::DefinitionBlockItem, vec![0, 60, 16, 17]),
                (SurfaceNodeKind::PathSegment, vec![22]),
                (SurfaceNodeKind::QualifiedSymbol, vec![62]),
                (SurfaceNodeKind::TypeHead, vec![63]),
                (
                    SurfaceNodeKind::DefinitionParameter,
                    vec![19, 20, 21, 64, 23],
                ),
                (SurfaceNodeKind::ItTerm, vec![29]),
                (SurfaceNodeKind::TermExpression, vec![66]),
                (SurfaceNodeKind::ItTerm, vec![31]),
                (SurfaceNodeKind::TermExpression, vec![68]),
                (
                    SurfaceNodeKind::BuiltinPredicateApplication,
                    vec![67, 30, 69],
                ),
                (SurfaceNodeKind::FormulaExpression, vec![70]),
                (SurfaceNodeKind::FormulaDefiniens, vec![71]),
                (SurfaceNodeKind::ComputationOption, vec![37, 38, 39]),
                (
                    SurfaceNodeKind::ComputationJustification,
                    vec![35, 36, 73, 40],
                ),
                (SurfaceNodeKind::JustificationClause, vec![34, 74]),
                (SurfaceNodeKind::CorrectnessCondition, vec![33, 75, 41]),
                (SurfaceNodeKind::ComputationOption, vec![46, 47, 48]),
                (
                    SurfaceNodeKind::ComputationJustification,
                    vec![44, 45, 77, 49],
                ),
                (SurfaceNodeKind::JustificationClause, vec![43, 78]),
                (SurfaceNodeKind::CorrectnessCondition, vec![42, 79, 50]),
                (
                    SurfaceNodeKind::PropertyImplementation,
                    vec![18, 65, 24, 25, 26, 27, 28, 72, 32, 76, 80, 51, 52],
                ),
                (SurfaceNodeKind::ItemList, vec![61, 81]),
                (SurfaceNodeKind::CompilationUnit, vec![82]),
                (
                    SurfaceNodeKind::Root,
                    vec![
                        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                        21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
                        40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 83,
                    ],
                ),
            ],
        ),
        TestProfile::Equals => (
            include_str!(
                "../../../../tests/miz/pass/types/pass_type_elaboration_property_implementation_equals_payload_001.miz"
            ),
            "definition struct Task264Carrier where field carrier -> set ; property marker -> set ; end ; end ; definition let M be Task264Carrier ; property M . marker equals M . carrier ; end ;",
            vec![
                (SurfaceNodeKind::StructurePattern, vec![2]),
                (SurfaceNodeKind::TypeHead, vec![7]),
                (SurfaceNodeKind::TypeExpression, vec![36]),
                (SurfaceNodeKind::StructureField, vec![4, 5, 6, 37, 8]),
                (SurfaceNodeKind::TypeHead, vec![12]),
                (SurfaceNodeKind::TypeExpression, vec![39]),
                (SurfaceNodeKind::StructureProperty, vec![9, 10, 11, 40, 13]),
                (
                    SurfaceNodeKind::StructureDefinition,
                    vec![1, 35, 3, 38, 41, 14, 15],
                ),
                (SurfaceNodeKind::DefinitionBlockItem, vec![0, 42, 16, 17]),
                (SurfaceNodeKind::PathSegment, vec![22]),
                (SurfaceNodeKind::QualifiedSymbol, vec![44]),
                (SurfaceNodeKind::TypeHead, vec![45]),
                (
                    SurfaceNodeKind::DefinitionParameter,
                    vec![19, 20, 21, 46, 23],
                ),
                (SurfaceNodeKind::TermReference, vec![29]),
                (SurfaceNodeKind::SelectorAccess, vec![48, 30, 31]),
                (SurfaceNodeKind::TermExpression, vec![49]),
                (SurfaceNodeKind::TermDefiniens, vec![50]),
                (
                    SurfaceNodeKind::PropertyImplementation,
                    vec![18, 47, 24, 25, 26, 27, 28, 51, 32, 33, 34],
                ),
                (SurfaceNodeKind::ItemList, vec![43, 52]),
                (SurfaceNodeKind::CompilationUnit, vec![53]),
                (
                    SurfaceNodeKind::Root,
                    vec![
                        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                        21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 54,
                    ],
                ),
            ],
        ),
    };
    let text = if matches!(&mutation, NativePropertySourceMutation::MemberName) {
        text.replacen("property marker", "property wrongx", 1)
    } else if matches!(&mutation, NativePropertySourceMutation::BodyContent) {
        if f.profile == TestProfile::Means {
            text.replace("it = it", "it = ix")
        } else {
            text.replace("equals M.carrier", "equals M.wrongxx")
        }
    } else {
        text.to_owned()
    };
    let mut builder = SurfaceAstBuilder::new(f.source);
    let mut sites = Vec::new();
    let mut spans = Vec::new();
    let mut cursor = 0;
    for (index, token) in tokens.split_ascii_whitespace().enumerate() {
        let token = if index == 10 && matches!(&mutation, NativePropertySourceMutation::MemberName)
        {
            "wrongx"
        } else if index == 31 && matches!(&mutation, NativePropertySourceMutation::BodyContent) {
            if f.profile == TestProfile::Means {
                "ix"
            } else {
                "wrongxx"
            }
        } else {
            token
        };
        let start = cursor + text[cursor..].find(token).unwrap();
        cursor = start + token.len();
        let mut kind = match token {
            "Task264Carrier" if index == 22 => SurfaceTokenKind::UserSymbol,
            "Task264Carrier" | "carrier" | "marker" | "wrongx" | "wrongxx" | "M" | "steps"
            | "ix" => SurfaceTokenKind::Identifier,
            "1" => SurfaceTokenKind::Numeral,
            ";" | "->" | "." | "=" | "(" | ")" | ":" => SurfaceTokenKind::ReservedSymbol,
            _ => SurfaceTokenKind::ReservedWord,
        };
        if index == 12 && matches!(&mutation, NativePropertySourceMutation::HeadToken) {
            kind = SurfaceTokenKind::Identifier;
        }
        sites.push(builder.add_token(kind, token, range(f.source, start, cursor)));
        spans.push((start, cursor));
    }
    for (mut kind, children) in nodes {
        if kind == SurfaceNodeKind::TypeExpression
            && sites.len()
                == if f.profile == TestProfile::Means {
                    58
                } else {
                    40
                }
            && matches!(&mutation, NativePropertySourceMutation::PropertyType)
        {
            kind = SurfaceNodeKind::TermList;
        }
        if kind == SurfaceNodeKind::ItTerm
            && sites.len() == 68
            && matches!(&mutation, NativePropertySourceMutation::BodyContent)
        {
            kind = SurfaceNodeKind::TermReference;
        }
        let start = spans[children[0]].0;
        let end = spans[*children.last().unwrap()].1;
        let children = children.into_iter().map(|index| sites[index]).collect();
        sites.push(builder.add_node(kind, range(f.source, start, end), children));
        spans.push((start, end));
    }
    let ast = builder.finish(sites.last().copied(), None);
    let source = SurfaceResolvedArena::lower(&ast, &f.module).unwrap();
    let nodes = TypedArena::try_new(
        Some(TypedNodeId::new(source.arena().root().index())),
        source
            .arena()
            .iter()
            .map(|(id, node)| {
                TypedNode::new("syntax", node.origin().anchor().clone())
                    .with_resolved_node(id)
                    .with_children(
                        node.children()
                            .iter()
                            .map(|id| TypedNodeId::new(id.index()))
                            .collect(),
                    )
            })
            .collect(),
    )
    .unwrap();
    let mut parts = f.typed_parts(InitialObligationTable::new());
    parts.source_context = None;
    parts.source_type = None;
    parts.resolved_root = Some(source.arena().root());
    parts.nodes = nodes;
    let mut syntax = TypedAst::try_new(parts).unwrap();
    syntax.install_source_nodes(&source).unwrap();
    syntax
}

#[test]
fn structure_property_native_source_authority_and_content_are_required() {
    for profile in [TestProfile::Means, TestProfile::Equals] {
        let mut f = fixture(profile);
        let base = InitialObligationTable::new();
        let source = native_property_source(&f, NativePropertySourceMutation::None);
        f.attach_native_children(&source);
        let before = source.clone();
        let projection = f.build_native(&base, &source).unwrap();
        assert!(Arc::ptr_eq(
            source.source_nodes().unwrap(),
            projection.handoff.original_source_nodes.as_ref().unwrap()
        ));
        assert_eq!(source, before);
        assert_eq!(
            f.build_native(&base, &f.typed_base(base.clone())),
            Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })
        );
        for foreign_id in [false, true] {
            let mut foreign = f.clone();
            if foreign_id {
                foreign.source = source_id(if profile == TestProfile::Means {
                    TestProfile::Equals
                } else {
                    TestProfile::Means
                });
            } else {
                foreign.module =
                    ModuleId::new(PackageId::new("foreign"), ModulePath::new("property"));
            }
            let source = native_property_source(&foreign, NativePropertySourceMutation::None);
            assert_eq!(
                f.build_native(&base, &source),
                Err(SourcePropertyImplementationError::SourceIdentityMismatch)
            );
        }
        for mutation in [
            NativePropertySourceMutation::PropertyType,
            NativePropertySourceMutation::HeadToken,
            NativePropertySourceMutation::MemberName,
        ] {
            let bad = native_property_source(&f, mutation);
            assert_eq!(
                f.build_native(&base, &bad),
                Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })
            );
        }
        let bad_body = native_property_source(&f, NativePropertySourceMutation::BodyContent);
        assert_eq!(
            f.build_native(&base, &bad_body),
            Err(SourcePropertyImplementationError::InvalidImplementation { index: 0 })
        );
        let mut wrong_style = f.clone();
        wrong_style.input.implementations[0].style = if profile == TestProfile::Means {
            SourcePropertyImplementationStyle::Equals
        } else {
            SourcePropertyImplementationStyle::Means
        };
        assert_eq!(
            wrong_style.build_native(&base, &source),
            Err(SourcePropertyImplementationError::InvalidImplementation { index: 0 })
        );
    }
}

#[test]
fn structure_property_native_replay_keeps_late_identity_and_arena_priorities() {
    for profile in [TestProfile::Means, TestProfile::Equals] {
        let mut f = fixture(profile);
        let source = native_property_source(&f, NativePropertySourceMutation::None);
        f.attach_native_children(&source);
        let base = InitialObligationTable::new();
        let projection = f.build_native(&base, &source).unwrap();
        let typed = f
            .typed_base(base.clone())
            .with_source_property_implementation(projection.clone())
            .unwrap();
        assert!(typed.source_nodes().is_none());
        assert!(assemble_empty(&typed).is_ok());
        for fault in ["identity_path", "target_origin"] {
            let mut bad = projection.clone();
            if fault == "identity_path" {
                bad.handoff.carrier_identity.field.origin = SemanticOrigin::new(
                    f.source,
                    f.module.clone(),
                    bad.handoff.carrier_identity.field.origin.anchor().clone(),
                    vec![4, 0, 11, 0, 18, 9],
                );
            } else {
                bad.handoff.targets.rows[0].origin =
                    bad.handoff.carrier_identity.field.origin.clone();
            }
            assert!(Arc::ptr_eq(
                bad.handoff.original_source_nodes.as_ref().unwrap(),
                source.source_nodes().unwrap()
            ));
            assert_eq!(
                f.validate_handoff(&bad.handoff, &bad.initial_obligations, &f.arena),
                Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })
            );
            assert_eq!(
                f.typed_base(base.clone())
                    .with_source_property_implementation(bad.clone()),
                Err(TypedAstError::InvalidSourcePropertyImplementation)
            );
            let mut injected = typed.clone();
            injected.inject_source_property_implementation_for_test(bad.handoff.clone());
            assert_eq!(
                assemble_empty(&injected),
                Err(ResolvedTypedAstError::InvalidSourcePropertyImplementation)
            );
            bad.handoff.parameters.rows[0].ordinal = 1;
            assert_eq!(
                f.validate_handoff(&bad.handoff, &bad.initial_obligations, &f.arena),
                Err(SourcePropertyImplementationError::InvalidParameter { index: 0 })
            );
        }
        let mut malformed = projection.clone();
        let rows = Arc::make_mut(malformed.handoff.original_source_nodes.as_mut().unwrap());
        rows.iter_mut()
            .find(|row| row.kind == SurfaceNodeKind::StructureProperty)
            .unwrap()
            .kind = SurfaceNodeKind::TermList;
        assert_eq!(
            f.validate_handoff(&malformed.handoff, &malformed.initial_obligations, &f.arena),
            Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })
        );
        assert_eq!(
            f.typed_base(base.clone())
                .with_source_property_implementation(malformed.clone()),
            Err(TypedAstError::InvalidSourcePropertyImplementation)
        );
        let mut injected = typed.clone();
        injected.inject_source_property_implementation_for_test(malformed.handoff);
        assert_eq!(
            assemble_empty(&injected),
            Err(ResolvedTypedAstError::InvalidSourcePropertyImplementation)
        );
        assert!(assemble_empty(&typed).is_ok());
    }
}

#[test]
fn property_native_arena_original_rows_bound_forgery_and_replay() {
    for profile in [TestProfile::Means, TestProfile::Equals] {
        let mut f = fixture(profile);
        let source = native_property_source(&f, NativePropertySourceMutation::None);
        f.attach_native_children(&source);
        let base = InitialObligationTable::new();
        let projection = f.build_native(&base, &source).unwrap();
        let typed = f
            .typed_base(base.clone())
            .with_source_property_implementation(projection.clone())
            .unwrap();
        let original_parameter = source
            .nodes()
            .node(TypedNodeId::new(parameter_node(profile)))
            .unwrap();
        let identifier = source.nodes().node(TypedNodeId::new(20)).unwrap();
        assert_ne!(original_parameter.anchor, identifier.anchor);
        assert_eq!(
            f.arena
                .node(TypedNodeId::new(parameter_node(profile)))
                .unwrap()
                .anchor,
            identifier.anchor
        );
        for node_id in [
            root_node(profile),
            if profile == TestProfile::Means {
                67
            } else {
                50
            },
        ] {
            let mut counterfeit = f.clone();
            let nodes = f
                .arena
                .iter()
                .map(|(id, node)| {
                    let mut node = node.clone();
                    if id.index() == node_id {
                        node.children.clear();
                    }
                    node
                })
                .collect();
            counterfeit.arena = TypedArena::try_new(f.arena.root(), nodes).unwrap();
            assert_eq!(
                counterfeit.build_native(&base, &source),
                Err(SourcePropertyImplementationError::InvalidArenaOwnership)
            );
            assert_eq!(
                counterfeit.validate_handoff(
                    &projection.handoff,
                    &projection.initial_obligations,
                    &counterfeit.arena
                ),
                Err(SourcePropertyImplementationError::InvalidArenaOwnership)
            );
            assert_eq!(
                counterfeit
                    .typed_base(base.clone())
                    .with_source_property_implementation(projection.clone()),
                Err(TypedAstError::InvalidSourcePropertyImplementation)
            );
            let mut injected = counterfeit.typed_base(base.clone());
            injected.replace_initial_obligations_for_test(projection.initial_obligations.clone());
            injected.inject_source_property_implementation_for_test(projection.handoff.clone());
            assert_eq!(
                assemble_empty(&injected),
                Err(ResolvedTypedAstError::InvalidSourcePropertyImplementation)
            );
        }
        let mut forged = projection.clone();
        let rows = Arc::make_mut(forged.handoff.original_source_nodes.as_mut().unwrap());
        let wrapper = rows
            .iter_mut()
            .find(|row| {
                row.site.index()
                    == if profile == TestProfile::Means {
                        67
                    } else {
                        50
                    }
            })
            .unwrap();
        wrapper.range.start += 1;
        assert_eq!(
            f.validate_handoff(&forged.handoff, &forged.initial_obligations, &f.arena),
            Err(SourcePropertyImplementationError::InvalidArenaOwnership)
        );
        assert_eq!(
            f.typed_base(base.clone())
                .with_source_property_implementation(forged.clone()),
            Err(TypedAstError::InvalidSourcePropertyImplementation)
        );
        let mut injected = typed.clone();
        injected.inject_source_property_implementation_for_test(forged.handoff);
        assert_eq!(
            assemble_empty(&injected),
            Err(ResolvedTypedAstError::InvalidSourcePropertyImplementation)
        );
        assert!(Arc::ptr_eq(
            projection.handoff.original_source_nodes.as_ref().unwrap(),
            source.source_nodes().unwrap()
        ));
        assert!(assemble_empty(&typed).is_ok());
    }
}
