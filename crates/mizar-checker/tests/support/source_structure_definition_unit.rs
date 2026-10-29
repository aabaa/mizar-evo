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
    source_functor_definition::tests::{
        actual_definition_family_typed_asts_for_task261,
        actual_functor_definition_projection_for_task263,
    },
    source_mode_definition::tests::actual_mode_definition_for_task263,
    source_predicate_definition::tests::actual_predicate_definition_projection_for_task263,
    source_type::{
        SourceTypeExpressionInput, SourceTypeStructureMemberHandoffInput,
        SourceTypeStructureMemberInput, SourceTypeStructureMemberProducer,
    },
    typed_ast::{
        CoercionTable, InitialObligationDraft, InitialObligationGoal, InitialObligationProvenance,
        InitialObligationStatus, LocalTypeContextTable, TypeDiagnosticTable, TypeFactTable,
        TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode, TypingState,
    },
};
use mizar_resolve::{
    env::{DefinitionShell, NamespacePath, SignatureShell, SymbolEntry, SymbolEnvIndexes},
    resolved_ast::{FullyQualifiedName, LocalSymbolId},
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
};

#[derive(Clone)]
struct Fixture {
    source: SourceId,
    module: ModuleId,
    env: SymbolEnv,
    source_type: SourceTypeApplicationHandoff,
    arena: TypedArena,
    input: SourceStructureDefinitionHandoffInput,
}

impl Fixture {
    fn build(
        &self,
        baseline: &InitialObligationTable,
    ) -> Result<SourceStructureDefinitionProjection, SourceStructureDefinitionError> {
        SourceStructureDefinitionProducer::build(
            self.input.clone(),
            &self.env,
            &self.source_type,
            baseline,
            &self.arena,
        )
    }

    fn projection(&self, baseline: &InitialObligationTable) -> SourceStructureDefinitionProjection {
        self.build(baseline).expect("exact Task 263 projection")
    }

    fn typed(&self, baseline: InitialObligationTable) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: Some(self.source_type.clone()),
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: baseline,
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task 263 typed baseline")
    }
}

#[test]
fn task_263_structure_definition_exact_payload_and_debug_are_deterministic() {
    let fixture = fixture();
    let baseline = unrelated_baseline(fixture.source, "baseline-a");
    let projection = fixture.projection(&baseline);
    assert_eq!(projection.base_initial_obligations(), &baseline);
    assert_eq!(projection.initial_obligations(), &baseline);
    let handoff = projection.handoff();
    assert_eq!(handoff.source_id(), fixture.source);
    assert_eq!(handoff.module_id(), &fixture.module);
    assert_eq!(
        handoff.source_type_fingerprint(),
        fixture.source_type.debug_text()
    );
    assert_eq!(handoff.base_initial_obligation_count(), 1);
    assert_eq!(handoff.definitions().len(), 2);
    assert_eq!(handoff.members().len(), 4);
    assert_eq!(handoff.inheritances().len(), 1);
    assert_eq!(handoff.mappings().len(), 2);
    assert!(handoff.coherence_requests().is_empty());

    assert_exact_rows(&fixture, handoff);
    let debug = handoff.debug_text();
    assert_eq!(debug, expected_debug_text(&fixture));
    assert_eq!(debug, handoff.clone().debug_text());
}

fn assert_exact_rows(fixture: &Fixture, handoff: &SourceStructureDefinitionHandoff) {
    let expected_resolver_identities = fixture
        .input
        .definitions
        .iter()
        .map(|row| (row.symbol.clone(), row.definition, row.contribution))
        .chain(
            fixture
                .input
                .members
                .iter()
                .map(|row| (row.symbol.clone(), row.definition, row.contribution)),
        )
        .chain(
            fixture
                .input
                .mappings
                .iter()
                .map(|row| (row.symbol.clone(), row.definition, row.contribution)),
        )
        .collect::<Vec<_>>();
    assert_eq!(
        handoff.resolver_identity_snapshot,
        expected_resolver_identities
    );
    for (index, input) in fixture.input.definitions.iter().enumerate() {
        let row = handoff
            .definitions()
            .get(SourceStructureDefinitionId::new(index))
            .unwrap();
        assert_eq!(row.id(), SourceStructureDefinitionId::new(index));
        assert_eq!(row.symbol(), &input.symbol);
        assert_eq!(row.definition(), input.definition);
        assert_eq!(row.contribution(), input.contribution);
        assert_eq!(row.site(), &input.site);
        assert_eq!(row.source_range(), input.source_range);
        assert_eq!(row.source_ordinal(), input.source_ordinal);
        assert_eq!(row.recovery(), input.recovery);
        assert_eq!(row.spelling(), input.spelling);
        assert_eq!(row.members(), input.members);
        assert_eq!(row.constructor_fields(), input.constructor_fields);
        assert_eq!(
            row.origin(),
            fixture
                .env
                .definitions()
                .get(input.definition)
                .unwrap()
                .origin()
        );
    }
    for (index, input) in fixture.input.members.iter().enumerate() {
        let row = handoff
            .members()
            .get(SourceStructureMemberId::new(index))
            .unwrap();
        assert_eq!(row.id(), SourceStructureMemberId::new(index));
        assert_eq!(row.symbol(), &input.symbol);
        assert_eq!(row.definition(), input.definition);
        assert_eq!(row.contribution(), input.contribution);
        assert_eq!(row.owner(), input.owner);
        assert_eq!(row.ordinal(), input.ordinal);
        assert_eq!(row.kind(), input.kind);
        assert_eq!(row.site(), &input.site);
        assert_eq!(row.source_range(), input.source_range);
        assert_eq!(row.recovery(), input.recovery);
        assert_eq!(row.spelling(), input.spelling);
        assert_eq!(row.written_type(), input.written_type);
        assert_eq!(row.constructor_ordinal(), input.constructor_ordinal);
        assert_eq!(
            row.origin(),
            fixture
                .env
                .definitions()
                .get(input.definition)
                .unwrap()
                .origin()
        );
    }
    for (index, input) in fixture.input.inheritances.iter().enumerate() {
        let row = handoff
            .inheritances()
            .get(SourceStructureInheritanceId::new(index))
            .unwrap();
        assert_eq!(row.id(), SourceStructureInheritanceId::new(index));
        assert_eq!(row.child(), input.child);
        assert_eq!(row.parent(), input.parent);
        assert_eq!(row.site(), &input.site);
        assert_eq!(row.source_range(), input.source_range);
        assert_eq!(row.source_ordinal(), input.source_ordinal);
        assert_eq!(row.recovery(), input.recovery);
        assert_eq!(row.spelling(), input.spelling);
        assert_eq!(row.mappings(), input.mappings);
    }
    for (index, input) in fixture.input.mappings.iter().enumerate() {
        let row = handoff
            .mappings()
            .get(SourceStructureMappingId::new(index))
            .unwrap();
        assert_eq!(row.id(), SourceStructureMappingId::new(index));
        assert_eq!(row.symbol(), &input.symbol);
        assert_eq!(row.definition(), input.definition);
        assert_eq!(row.contribution(), input.contribution);
        assert_eq!(row.inheritance(), input.inheritance);
        assert_eq!(row.ordinal(), input.ordinal);
        assert_eq!(row.kind(), input.kind);
        assert_eq!(row.view_member(), input.view_member);
        assert_eq!(row.parent_member(), input.parent_member);
        assert_eq!(row.root_member(), input.root_member);
        assert_eq!(row.path(), input.path);
        assert_eq!(row.site(), &input.site);
        assert_eq!(row.source_range(), input.source_range);
        assert_eq!(row.recovery(), input.recovery);
        assert_eq!(row.spelling(), input.spelling);
        assert_eq!(
            row.origin(),
            fixture
                .env
                .definitions()
                .get(input.definition)
                .unwrap()
                .origin()
        );
    }
}

fn expected_debug_text(fixture: &Fixture) -> String {
    format!(
        r#"source-structure-definition-debug-v1
module: task263
source-type-fingerprint: {:?}
base-initial-obligation-count: 1
profile: definitions=2 members=4 inheritances=1 mappings=2 coherence_requests=0
definition#0 symbol="pkg::task263::0:Task263Base" definition=0 contribution=0 ordinal=0 range=13..98 site=node#57 recovery=normal origin_range=13..98 origin_path=[4, 0, 11, 0] spelling="struct Task263Base where\n    field carrier -> set;\n    property marker -> set;\n  end;" members=[0, 1] constructor_fields=[0]
definition#1 symbol="pkg::task263::3:Task263Derived" definition=3 contribution=0 ordinal=1 range=102..190 site=node#65 recovery=normal origin_range=102..190 origin_path=[4, 0, 11, 1] spelling="struct Task263Derived where\n    field carrier -> set;\n    property marker -> set;\n  end;" members=[2, 3] constructor_fields=[2]
member#0 symbol="pkg::task263::1:carrier" definition=1 contribution=0 owner=0 ordinal=0 kind=field range=42..63 site=node#53 recovery=normal origin_range=42..63 origin_path=[4, 0, 11, 0, 18, 0] spelling="field carrier -> set;" written_type=0 constructor_ordinal=0
member#1 symbol="pkg::task263::2:marker" definition=2 contribution=0 owner=0 ordinal=1 kind=property range=68..91 site=node#56 recovery=normal origin_range=68..91 origin_path=[4, 0, 11, 0, 19, 1] spelling="property marker -> set;" written_type=1 constructor_ordinal=none
member#2 symbol="pkg::task263::4:carrier" definition=4 contribution=0 owner=1 ordinal=0 kind=field range=134..155 site=node#61 recovery=normal origin_range=134..155 origin_path=[4, 0, 11, 1, 18, 0] spelling="field carrier -> set;" written_type=2 constructor_ordinal=0
member#3 symbol="pkg::task263::5:marker" definition=5 contribution=0 owner=1 ordinal=1 kind=property range=160..183 site=node#64 recovery=normal origin_range=160..183 origin_path=[4, 0, 11, 1, 19, 1] spelling="property marker -> set;" written_type=3 constructor_ordinal=none
inheritance#0 child=1 parent=0 ordinal=0 range=194..314 site=node#70 recovery=normal spelling="inherit Task263Derived extends Task263Base where\n    field carrier from carrier;\n    property marker from marker;\n  end;" mappings=[0, 1]
mapping#0 symbol="pkg::task263::6:carrier" definition=6 contribution=0 inheritance=0 ordinal=0 kind=field view_member=2 parent_member=0 root_member=0 path=[0] range=247..274 site=node#68 recovery=normal origin_range=247..274 origin_path=[4, 0, 20, 2, 21, 0] spelling="field carrier from carrier;"
mapping#1 symbol="pkg::task263::7:marker" definition=7 contribution=0 inheritance=0 ordinal=1 kind=property view_member=3 parent_member=1 root_member=1 path=[0] range=279..307 site=node#69 recovery=normal origin_range=279..307 origin_path=[4, 0, 20, 2, 22, 1] spelling="property marker from marker;"
"#,
        fixture.source_type.debug_text()
    )
}

#[test]
fn task_263_structure_definition_resolver_and_row_corruption_fail_closed() {
    let fixture = fixture();
    let baseline = InitialObligationTable::new();
    let mut wrong_source = fixture.input.clone();
    wrong_source.source_id = other_source_id();
    assert_eq!(
        fixture_build(&fixture, wrong_source, &baseline),
        Err(SourceStructureDefinitionError::SourceIdentityMismatch)
    );

    let mut wrong_resolver = fixture.input.clone();
    wrong_resolver.members[3].definition = wrong_resolver.members[2].definition;
    assert_eq!(
        fixture_build(&fixture, wrong_resolver, &baseline),
        Err(SourceStructureDefinitionError::InvalidResolverDefinition { index: 5 })
    );

    let (stale_effect_env, _) =
        resolver_env_with_effects(fixture.source, fixture.module.clone(), true, None);
    assert_eq!(
        SourceStructureDefinitionProducer::build(
            fixture.input.clone(),
            &stale_effect_env,
            &fixture.source_type,
            &baseline,
            &fixture.arena,
        ),
        Err(SourceStructureDefinitionError::InvalidResolverDefinition { index: 0 })
    );

    let mut compound = fixture.input.clone();
    compound.definitions[1].source_ordinal = 9;
    compound.members[0].owner = SourceStructureDefinitionId::new(1);
    assert_eq!(
        fixture_build(&fixture, compound, &baseline),
        Err(SourceStructureDefinitionError::InvalidDefinition { index: 1 })
    );

    let definition_mutations: [fn(&mut SourceStructureDefinitionHandoffInput); 5] = [
        |input| input.definitions[0].site = node_site(65),
        |input| input.definitions[0].source_range.end -= 1,
        |input| input.definitions[0].source_ordinal = 1,
        |input| input.definitions[0].recovery = SourceStructureDefinitionRecovery::Degraded,
        |input| input.definitions[0].spelling.push(' '),
    ];
    for mutate in definition_mutations {
        let mut input = fixture.input.clone();
        mutate(&mut input);
        assert!(matches!(
            fixture_build(&fixture, input, &baseline),
            Err(SourceStructureDefinitionError::InvalidDefinition { .. })
        ));
    }
    let mut removed = fixture.input.clone();
    removed.mappings.pop();
    assert_eq!(
        fixture_build(&fixture, removed, &baseline),
        Err(SourceStructureDefinitionError::UnsupportedTaskShape)
    );

    let projection = fixture.projection(&baseline);
    let exact = projection.handoff();
    assert_resolver_handoff_corruptions(&fixture, exact, &baseline);

    let mut source_over_dependency = exact.clone();
    source_over_dependency.source_id = other_source_id();
    source_over_dependency.source_type_fingerprint = "stale lower".to_owned();
    assert_installation_error(
        &fixture,
        &source_over_dependency,
        &fixture.source_type,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::SourceIdentityMismatch,
    );

    let mut dependency_over_shape = exact.clone();
    dependency_over_shape.source_type_fingerprint = "stale lower".to_owned();
    dependency_over_shape.definitions.rows.pop();
    assert_installation_error(
        &fixture,
        &dependency_over_shape,
        &fixture.source_type,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::DependencyMismatch,
    );

    let mut shape_over_resolver = exact.clone();
    shape_over_resolver.definitions.rows.pop();
    shape_over_resolver.members.rows[3].origin = exact.members.rows[0].origin.clone();
    assert_installation_error(
        &fixture,
        &shape_over_resolver,
        &fixture.source_type,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::UnsupportedTaskShape,
    );

    let mut resolver_over_definition = exact.clone();
    resolver_over_definition.definitions.rows[1].origin = exact.definitions.rows[0].origin.clone();
    resolver_over_definition.definitions.rows[0]
        .spelling
        .push(' ');
    assert_installation_error(
        &fixture,
        &resolver_over_definition,
        &fixture.source_type,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::InvalidResolverDefinition { index: 1 },
    );

    let mut resolver_over_dense_id = exact.clone();
    resolver_over_dense_id.definitions.rows[0].id = SourceStructureDefinitionId::new(1);
    resolver_over_dense_id.definitions.rows[1].origin = exact.definitions.rows[0].origin.clone();
    assert_installation_error(
        &fixture,
        &resolver_over_dense_id,
        &fixture.source_type,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::InvalidResolverDefinition { index: 1 },
    );

    let mut definition_over_member = exact.clone();
    definition_over_member.definitions.rows[1].source_ordinal = 9;
    definition_over_member.members.rows[0].ordinal = 9;
    assert_installation_error(
        &fixture,
        &definition_over_member,
        &fixture.source_type,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::InvalidDefinition { index: 1 },
    );
}

#[test]
fn task_263_structure_definition_resolver_arity_is_authenticated_by_role() {
    let fixture = fixture();
    let baseline = InitialObligationTable::new();
    let (_, identities) = resolver_env(fixture.source, fixture.module.clone());

    for (identity_index, row_index) in [(0, 0), (3, 1)] {
        let definition_index = identities[identity_index].1.index();
        for arity in [None, Some(1), Some(2), Some(3)] {
            let (env, _) = resolver_env_with_effects(
                fixture.source,
                fixture.module.clone(),
                false,
                Some((definition_index, arity)),
            );
            assert_eq!(
                SourceStructureDefinitionProducer::build(
                    fixture.input.clone(),
                    &env,
                    &fixture.source_type,
                    &baseline,
                    &fixture.arena,
                ),
                Err(SourceStructureDefinitionError::InvalidResolverDefinition { index: row_index }),
                "structure definition row {row_index} with arity {arity:?}"
            );
        }
    }

    for (identity_index, row_index) in [(1, 2), (2, 3), (4, 4), (5, 5), (6, 6), (7, 7)] {
        let definition_index = identities[identity_index].1.index();
        for arity in [0, 1] {
            let (env, _) = resolver_env_with_effects(
                fixture.source,
                fixture.module.clone(),
                false,
                Some((definition_index, Some(arity))),
            );
            assert_eq!(
                SourceStructureDefinitionProducer::build(
                    fixture.input.clone(),
                    &env,
                    &fixture.source_type,
                    &baseline,
                    &fixture.arena,
                ),
                Err(SourceStructureDefinitionError::InvalidResolverDefinition { index: row_index }),
                "non-structure resolver row {row_index} with arity {arity}"
            );
        }
    }
}

#[test]
fn task_263_structure_definition_coverage_constructor_and_type_corruption_fail_closed() {
    let fixture = fixture();
    let baseline = InitialObligationTable::new();
    let mutations: [fn(&mut SourceStructureDefinitionHandoffInput); 10] = [
        |input| input.definitions[0].members.reverse(),
        |input| {
            input.definitions[0]
                .constructor_fields
                .push(SourceStructureMemberId::new(1))
        },
        |input| input.members[1].kind = SourceStructureMemberKind::Field,
        |input| input.members[0].constructor_ordinal = None,
        |input| input.inheritances[0].child = SourceStructureDefinitionId::new(0),
        |input| input.inheritances[0].mappings.reverse(),
        |input| input.mappings[0].parent_member = SourceStructureMemberId::new(1),
        |input| input.mappings[0].root_member = SourceStructureMemberId::new(1),
        |input| input.mappings[0].path.clear(),
        |input| input.mappings[0].view_member = SourceStructureMemberId::new(3),
    ];
    for mutate in mutations {
        let mut input = fixture.input.clone();
        mutate(&mut input);
        assert!(fixture_build(&fixture, input, &baseline).is_err());
    }

    let mut second_edge = fixture.input.clone();
    second_edge
        .inheritances
        .push(second_edge.inheritances[0].clone());
    assert_eq!(
        fixture_build(&fixture, second_edge, &baseline),
        Err(SourceStructureDefinitionError::UnsupportedTaskShape)
    );
    let mut renamed_mapping = fixture.input.clone();
    renamed_mapping.mappings[0].spelling = "field renamed from carrier;".to_owned();
    assert_eq!(
        fixture_build(&fixture, renamed_mapping, &baseline),
        Err(SourceStructureDefinitionError::InvalidMapping { index: 0 })
    );

    let mut lower_input = source_type_input(fixture.source, fixture.module.clone());
    lower_input.members[2].expression.head = SourceTypeHead::BuiltinObject;
    lower_input.members[2].expression.head_spelling = "object".to_owned();
    lower_input.members[2].expression.spelling = "object".to_owned();
    let lower = SourceTypeStructureMemberProducer::build(lower_input, &fixture.arena);
    assert!(
        lower.is_err(),
        "Task249S must reject narrowed/nonidentical lower input first"
    );

    let mut compound = fixture.input.clone();
    compound.members[3].ordinal = 9;
    compound.mappings[0].path.clear();
    assert_eq!(
        fixture_build(&fixture, compound, &baseline),
        Err(SourceStructureDefinitionError::InvalidMember { index: 3 })
    );

    let projection = fixture.projection(&baseline);
    let exact = projection.handoff();
    assert_structural_handoff_metadata_corruptions(&fixture, exact, &baseline);

    let mut second_edge = exact.clone();
    let mut duplicate_edge = second_edge.inheritances.rows[0].clone();
    duplicate_edge.id = SourceStructureInheritanceId::new(1);
    second_edge.inheritances.rows.push(duplicate_edge);
    assert_installation_error(
        &fixture,
        &second_edge,
        &fixture.source_type,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::UnsupportedTaskShape,
    );

    let mut member_over_inheritance = exact.clone();
    member_over_inheritance.members.rows[3].ordinal = 9;
    member_over_inheritance.inheritances.rows[0].child = SourceStructureDefinitionId::new(0);
    assert_installation_error(
        &fixture,
        &member_over_inheritance,
        &fixture.source_type,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::InvalidMember { index: 3 },
    );

    let mut inheritance_over_mapping = exact.clone();
    inheritance_over_mapping.inheritances.rows[0].child = SourceStructureDefinitionId::new(0);
    inheritance_over_mapping.mappings.rows[0].path.clear();
    assert_installation_error(
        &fixture,
        &inheritance_over_mapping,
        &fixture.source_type,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::InvalidInheritance { index: 0 },
    );

    let mut wrong_lower = fixture.source_type.clone();
    wrong_lower.set_structure_member_root_for_test(0, SourceTypeExpressionId::new(1));

    let mut mapping_over_lower = exact.clone();
    mapping_over_lower.source_type_fingerprint = wrong_lower.debug_text();
    mapping_over_lower.mappings.rows[1].path.clear();
    assert_installation_error(
        &fixture,
        &mapping_over_lower,
        &wrong_lower,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::InvalidMapping { index: 1 },
    );

    let coherence = SourceStructureCoherenceRequest {
        id: SourceStructureCoherenceRequestId::new(0),
        mapping: SourceStructureMappingId::new(0),
        kind: SourceStructureCoherenceRequestKind::MemberTypeInclusion,
        site: node_site(68),
        source_range: range(fixture.source, 247, 274),
    };
    let mut lower_over_coherence = exact.clone();
    lower_over_coherence.source_type_fingerprint = wrong_lower.debug_text();
    lower_over_coherence
        .coherence_requests
        .rows
        .push(coherence.clone());
    assert_installation_error(
        &fixture,
        &lower_over_coherence,
        &wrong_lower,
        &baseline,
        &fixture.arena,
        SourceStructureDefinitionError::InvalidMember { index: 0 },
    );

    let wrong_obligations = unrelated_baseline(fixture.source, "precedence");
    let mut coherence_over_obligation = exact.clone();
    coherence_over_obligation
        .coherence_requests
        .rows
        .push(coherence);
    assert_installation_error(
        &fixture,
        &coherence_over_obligation,
        &fixture.source_type,
        &wrong_obligations,
        &fixture.arena,
        SourceStructureDefinitionError::InvalidCoherenceRequest { index: 0 },
    );

    let wrong_arena = TypedArena::try_new(None, Vec::new()).unwrap();
    assert_installation_error(
        &fixture,
        exact,
        &fixture.source_type,
        &wrong_obligations,
        &wrong_arena,
        SourceStructureDefinitionError::InvalidObligation,
    );
}

#[test]
fn task_263_structure_definition_obligation_and_typed_installation_are_transactional() {
    let fixture = fixture();
    let baseline = unrelated_baseline(fixture.source, "baseline-a");
    let projection = fixture.projection(&baseline);
    let typed = fixture
        .typed(baseline.clone())
        .with_source_structure_definition(projection.clone())
        .expect("one-shot install");
    assert_eq!(typed.initial_obligations(), &baseline);
    assert_eq!(
        typed.source_structure_definition(),
        Some(projection.handoff())
    );
    assert_eq!(
        typed
            .clone()
            .with_source_structure_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceStructureDefinition)
    );

    let other = unrelated_baseline(fixture.source, "baseline-b");
    assert_eq!(
        fixture
            .typed(other)
            .with_source_structure_definition(projection.clone()),
        Err(TypedAstError::InvalidSourceStructureDefinition)
    );

    let mut corrupt = projection.clone();
    corrupt.handoff.base_initial_obligations_snapshot =
        unrelated_baseline(fixture.source, "baseline-b");
    assert_eq!(
        fixture
            .typed(baseline.clone())
            .with_source_structure_definition(corrupt),
        Err(TypedAstError::InvalidSourceStructureDefinition)
    );

    let mut changed_final = projection.clone();
    changed_final.initial_obligations = unrelated_baseline(fixture.source, "baseline-b");
    assert_eq!(
        fixture
            .typed(baseline.clone())
            .with_source_structure_definition(changed_final),
        Err(TypedAstError::InvalidSourceStructureDefinition)
    );

    let mut suffixed_final = projection;
    let mut suffix = baseline.clone();
    suffix.insert(obligation(fixture.source, "baseline-suffix"));
    suffixed_final.initial_obligations = suffix;
    assert_eq!(
        fixture
            .typed(baseline.clone())
            .with_source_structure_definition(suffixed_final),
        Err(TypedAstError::InvalidSourceStructureDefinition)
    );

    let mut orphan = InitialObligationTable::new();
    orphan.insert(obligation(
        fixture.source,
        "source.definition.functor.orphan",
    ));
    assert_eq!(
        fixture.build(&orphan),
        Err(SourceStructureDefinitionError::InvalidObligation)
    );
    let mut own_domain = InitialObligationTable::new();
    own_domain.insert(obligation(
        fixture.source,
        "source.definition.structure.orphan",
    ));
    assert_eq!(
        fixture.build(&own_domain),
        Err(SourceStructureDefinitionError::InvalidObligation)
    );
}

#[test]
fn task_263_structure_definition_final_clone_and_family_isolation_are_exact() {
    let fixture = fixture();
    let projection = fixture.projection(&InitialObligationTable::new());
    let typed = fixture
        .typed(InitialObligationTable::new())
        .with_source_structure_definition(projection.clone())
        .unwrap();
    let resolved = assemble_empty(&typed).expect("Task263 final assembly");
    assert_eq!(
        typed.source_structure_definition(),
        resolved.source_structure_definition()
    );
    assert_eq!(typed.debug_text(), typed.clone().debug_text());
    assert_eq!(
        resolved.debug_text(),
        assemble_empty(&typed).unwrap().debug_text()
    );

    let (predicate_typed, functor_typed) = actual_definition_family_typed_asts_for_task261();
    let attribute_typed = actual_attribute_definition_typed_ast_for_task262();
    let (mode_typed, mode_projection) = actual_mode_definition_for_task263();
    let predicate_projection = actual_predicate_definition_projection_for_task263();
    let functor_projection = actual_functor_definition_projection_for_task263();
    let predicate_handoff = predicate_typed
        .source_predicate_definition()
        .unwrap()
        .clone();
    let functor_handoff = functor_typed.source_functor_definition().unwrap().clone();
    let attribute_handoff = attribute_typed
        .source_attribute_definition()
        .unwrap()
        .clone();
    let mode_handoff = mode_typed.source_mode_definition().unwrap().clone();
    for sibling in [
        predicate_typed.clone(),
        functor_typed.clone(),
        attribute_typed.clone(),
        mode_typed.clone(),
    ] {
        assert_eq!(
            sibling.with_source_structure_definition(projection.clone()),
            Err(TypedAstError::InvalidSourceStructureDefinition)
        );
    }
    assert_eq!(
        typed
            .clone()
            .with_source_predicate_definition(predicate_projection),
        Err(TypedAstError::InvalidSourcePredicateDefinition)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_functor_definition(functor_projection),
        Err(TypedAstError::InvalidSourceFunctorDefinition)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_attribute_definition(attribute_handoff.clone()),
        Err(TypedAstError::InvalidSourceAttributeDefinition)
    );
    assert_eq!(
        typed.clone().with_source_mode_definition(mode_projection),
        Err(TypedAstError::InvalidSourceModeDefinition)
    );

    for mut sibling in [predicate_typed, functor_typed, attribute_typed, mode_typed] {
        sibling.inject_source_structure_definition_for_test(projection.handoff().clone());
        assert_eq!(
            assemble_empty(&sibling),
            Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
        );
    }
    let mut structure_then_predicate = typed.clone();
    structure_then_predicate.inject_source_predicate_definition_for_test(predicate_handoff);
    assert_eq!(
        assemble_empty(&structure_then_predicate),
        Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
    );
    let mut structure_then_functor = typed.clone();
    structure_then_functor.inject_source_functor_definition_for_test(functor_handoff);
    assert_eq!(
        assemble_empty(&structure_then_functor),
        Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
    );
    let mut structure_then_attribute = typed.clone();
    structure_then_attribute.inject_source_attribute_definition_for_test(attribute_handoff);
    assert_eq!(
        assemble_empty(&structure_then_attribute),
        Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
    );
    let mut structure_then_mode = typed.clone();
    structure_then_mode.inject_source_mode_definition_for_test(mode_handoff);
    assert_eq!(
        assemble_empty(&structure_then_mode),
        Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
    );

    let mut corrupted = typed.clone();
    let structure = corrupted.source_structure_definition().unwrap().clone();
    let mut changed = structure;
    changed.base_initial_obligations_snapshot = unrelated_baseline(fixture.source, "late-change");
    corrupted.inject_source_structure_definition_for_test(changed);
    assert_eq!(
        assemble_empty(&corrupted),
        Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
    );
}

fn assert_resolver_handoff_corruptions(
    fixture: &Fixture,
    exact: &SourceStructureDefinitionHandoff,
    baseline: &InitialObligationTable,
) {
    let symbols = exact
        .definitions
        .rows
        .iter()
        .map(|row| row.symbol.clone())
        .chain(exact.members.rows.iter().map(|row| row.symbol.clone()))
        .chain(exact.mappings.rows.iter().map(|row| row.symbol.clone()))
        .collect::<Vec<_>>();
    let origins = exact
        .definitions
        .rows
        .iter()
        .map(|row| row.origin.clone())
        .chain(exact.members.rows.iter().map(|row| row.origin.clone()))
        .chain(exact.mappings.rows.iter().map(|row| row.origin.clone()))
        .collect::<Vec<_>>();
    let stale_contribution = unrelated_contribution_id(fixture.source, fixture.module.clone());

    for index in 0..2 {
        let expected = SourceStructureDefinitionError::InvalidResolverDefinition { index };
        let mut wrong_symbol = exact.clone();
        wrong_symbol.definitions.rows[index].symbol = symbols[(index + 1) % symbols.len()].clone();
        assert_installation_error(
            fixture,
            &wrong_symbol,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            expected.clone(),
        );
        let mut wrong_contribution = exact.clone();
        wrong_contribution.definitions.rows[index].contribution = stale_contribution;
        assert_installation_error(
            fixture,
            &wrong_contribution,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            expected.clone(),
        );
        let mut wrong_origin = exact.clone();
        wrong_origin.definitions.rows[index].origin = origins[(index + 1) % origins.len()].clone();
        assert_installation_error(
            fixture,
            &wrong_origin,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            expected,
        );
    }
    for index in 0..4 {
        let resolver_index = index + 2;
        let expected = SourceStructureDefinitionError::InvalidResolverDefinition {
            index: resolver_index,
        };
        let mut wrong_symbol = exact.clone();
        wrong_symbol.members.rows[index].symbol =
            symbols[(resolver_index + 1) % symbols.len()].clone();
        assert_installation_error(
            fixture,
            &wrong_symbol,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            expected.clone(),
        );
        let mut wrong_contribution = exact.clone();
        wrong_contribution.members.rows[index].contribution = stale_contribution;
        assert_installation_error(
            fixture,
            &wrong_contribution,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            expected.clone(),
        );
        let mut wrong_origin = exact.clone();
        wrong_origin.members.rows[index].origin =
            origins[(resolver_index + 1) % origins.len()].clone();
        assert_installation_error(
            fixture,
            &wrong_origin,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            expected,
        );
    }
    for index in 0..2 {
        let resolver_index = index + 6;
        let expected = SourceStructureDefinitionError::InvalidResolverDefinition {
            index: resolver_index,
        };
        let mut wrong_symbol = exact.clone();
        wrong_symbol.mappings.rows[index].symbol =
            symbols[(resolver_index + 1) % symbols.len()].clone();
        assert_installation_error(
            fixture,
            &wrong_symbol,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            expected.clone(),
        );
        let mut wrong_contribution = exact.clone();
        wrong_contribution.mappings.rows[index].contribution = stale_contribution;
        assert_installation_error(
            fixture,
            &wrong_contribution,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            expected.clone(),
        );
        let mut wrong_origin = exact.clone();
        wrong_origin.mappings.rows[index].origin =
            origins[(resolver_index + 1) % origins.len()].clone();
        assert_installation_error(
            fixture,
            &wrong_origin,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            expected,
        );
    }
}

fn assert_structural_handoff_metadata_corruptions(
    fixture: &Fixture,
    exact: &SourceStructureDefinitionHandoff,
    baseline: &InitialObligationTable,
) {
    for index in 0..4 {
        for field in 0..4 {
            let mut changed = exact.clone();
            match field {
                0 => changed.members.rows[index].site = node_site(0),
                1 => changed.members.rows[index].source_range.end -= 1,
                2 => changed.members.rows[index].spelling.push(' '),
                3 => {
                    changed.members.rows[index].recovery =
                        SourceStructureDefinitionRecovery::Degraded
                }
                _ => unreachable!(),
            }
            assert_installation_error(
                fixture,
                &changed,
                &fixture.source_type,
                baseline,
                &fixture.arena,
                SourceStructureDefinitionError::InvalidMember { index },
            );
        }
    }
    for field in 0..4 {
        let mut changed = exact.clone();
        match field {
            0 => changed.inheritances.rows[0].site = node_site(0),
            1 => changed.inheritances.rows[0].source_range.end -= 1,
            2 => changed.inheritances.rows[0].spelling.push(' '),
            3 => {
                changed.inheritances.rows[0].recovery = SourceStructureDefinitionRecovery::Degraded
            }
            _ => unreachable!(),
        }
        assert_installation_error(
            fixture,
            &changed,
            &fixture.source_type,
            baseline,
            &fixture.arena,
            SourceStructureDefinitionError::InvalidInheritance { index: 0 },
        );
    }
    for index in 0..2 {
        for field in 0..4 {
            let mut changed = exact.clone();
            match field {
                0 => changed.mappings.rows[index].site = node_site(0),
                1 => changed.mappings.rows[index].source_range.end -= 1,
                2 => changed.mappings.rows[index].spelling.push(' '),
                3 => {
                    changed.mappings.rows[index].recovery =
                        SourceStructureDefinitionRecovery::Degraded
                }
                _ => unreachable!(),
            }
            assert_installation_error(
                fixture,
                &changed,
                &fixture.source_type,
                baseline,
                &fixture.arena,
                SourceStructureDefinitionError::InvalidMapping { index },
            );
        }
    }
}

fn unrelated_contribution_id(source: SourceId, module: ModuleId) -> SourceContributionId {
    let mut indexes = SymbolEnvIndexes::default();
    indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 1)),
    );
    indexes.contributions.insert(
        module,
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 1, 2)),
    )
}

fn fixture_build(
    fixture: &Fixture,
    input: SourceStructureDefinitionHandoffInput,
    baseline: &InitialObligationTable,
) -> Result<SourceStructureDefinitionProjection, SourceStructureDefinitionError> {
    SourceStructureDefinitionProducer::build(
        input,
        &fixture.env,
        &fixture.source_type,
        baseline,
        &fixture.arena,
    )
}

fn assert_installation_error(
    fixture: &Fixture,
    handoff: &SourceStructureDefinitionHandoff,
    source_type: &SourceTypeApplicationHandoff,
    obligations: &InitialObligationTable,
    arena: &TypedArena,
    expected: SourceStructureDefinitionError,
) {
    assert_eq!(
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            source_type,
            obligations,
            arena,
        ),
        Err(expected),
    );
}

fn fixture() -> Fixture {
    let source = source_id();
    let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task263"));
    let arena = task263_arena(source);
    let source_type =
        SourceTypeStructureMemberProducer::build(source_type_input(source, module.clone()), &arena)
            .expect("Task249S exact lower handoff");
    let (env, identities) = resolver_env(source, module.clone());
    let definitions = vec![
        SourceStructureDefinitionInput { symbol: identities[0].0.clone(), definition: identities[0].1, contribution: identities[0].2, site: node_site(57), source_range: range(source, 13, 98), source_ordinal: 0, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "struct Task263Base where\n    field carrier -> set;\n    property marker -> set;\n  end;".to_owned(), members: vec![SourceStructureMemberId::new(0), SourceStructureMemberId::new(1)], constructor_fields: vec![SourceStructureMemberId::new(0)] },
        SourceStructureDefinitionInput { symbol: identities[3].0.clone(), definition: identities[3].1, contribution: identities[3].2, site: node_site(65), source_range: range(source, 102, 190), source_ordinal: 1, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "struct Task263Derived where\n    field carrier -> set;\n    property marker -> set;\n  end;".to_owned(), members: vec![SourceStructureMemberId::new(2), SourceStructureMemberId::new(3)], constructor_fields: vec![SourceStructureMemberId::new(2)] },
    ];
    let member_specs = [
        (
            1,
            0,
            0,
            SourceStructureMemberKind::Field,
            53,
            42,
            63,
            "field carrier -> set;",
            Some(0),
        ),
        (
            2,
            0,
            1,
            SourceStructureMemberKind::Property,
            56,
            68,
            91,
            "property marker -> set;",
            None,
        ),
        (
            4,
            1,
            0,
            SourceStructureMemberKind::Field,
            61,
            134,
            155,
            "field carrier -> set;",
            Some(0),
        ),
        (
            5,
            1,
            1,
            SourceStructureMemberKind::Property,
            64,
            160,
            183,
            "property marker -> set;",
            None,
        ),
    ];
    let members = member_specs
        .into_iter()
        .enumerate()
        .map(
            |(
                index,
                (identity, owner, ordinal, kind, node, start, end, spelling, constructor_ordinal),
            )| SourceStructureMemberInput {
                symbol: identities[identity].0.clone(),
                definition: identities[identity].1,
                contribution: identities[identity].2,
                owner: SourceStructureDefinitionId::new(owner),
                ordinal,
                kind,
                site: node_site(node),
                source_range: range(source, start, end),
                recovery: SourceStructureDefinitionRecovery::Normal,
                spelling: spelling.to_owned(),
                written_type: SourceTypeStructureMemberId::new(index),
                constructor_ordinal,
            },
        )
        .collect();
    let inheritances = vec![SourceStructureInheritanceInput { child: SourceStructureDefinitionId::new(1), parent: SourceStructureDefinitionId::new(0), site: node_site(70), source_range: range(source, 194, 314), source_ordinal: 0, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "inherit Task263Derived extends Task263Base where\n    field carrier from carrier;\n    property marker from marker;\n  end;".to_owned(), mappings: vec![SourceStructureMappingId::new(0), SourceStructureMappingId::new(1)] }];
    let mapping_specs = [
        (
            6,
            SourceStructureMemberKind::Field,
            2,
            0,
            68,
            247,
            274,
            "field carrier from carrier;",
        ),
        (
            7,
            SourceStructureMemberKind::Property,
            3,
            1,
            69,
            279,
            307,
            "property marker from marker;",
        ),
    ];
    let mappings = mapping_specs
        .into_iter()
        .enumerate()
        .map(
            |(index, (identity, kind, view, parent, node, start, end, spelling))| {
                SourceStructureMappingInput {
                    symbol: identities[identity].0.clone(),
                    definition: identities[identity].1,
                    contribution: identities[identity].2,
                    inheritance: SourceStructureInheritanceId::new(0),
                    ordinal: index,
                    kind,
                    view_member: SourceStructureMemberId::new(view),
                    parent_member: SourceStructureMemberId::new(parent),
                    root_member: SourceStructureMemberId::new(parent),
                    path: vec![SourceStructureInheritanceId::new(0)],
                    site: node_site(node),
                    source_range: range(source, start, end),
                    recovery: SourceStructureDefinitionRecovery::Normal,
                    spelling: spelling.to_owned(),
                }
            },
        )
        .collect();
    Fixture {
        source,
        module: module.clone(),
        env,
        source_type,
        arena,
        input: SourceStructureDefinitionHandoffInput {
            source_id: source,
            module_id: module,
            definitions,
            members,
            inheritances,
            mappings,
        },
    }
}

type ResolverIdentity = (SymbolId, DefinitionId, SourceContributionId);

fn resolver_env(source: SourceId, module: ModuleId) -> (SymbolEnv, Vec<ResolverIdentity>) {
    resolver_env_with_effects(source, module, false, None)
}

fn resolver_env_with_effects(
    source: SourceId,
    module: ModuleId,
    stale_effects: bool,
    arity_override: Option<(usize, Option<u32>)>,
) -> (SymbolEnv, Vec<ResolverIdentity>) {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 319)),
    );
    let specs = [
        (
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task263Base",
            Some("Task263Base"),
            13,
            98,
            &[4, 0, 11, 0][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "carrier",
            None,
            42,
            63,
            &[4, 0, 11, 0, 18, 0][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "marker",
            None,
            68,
            91,
            &[4, 0, 11, 0, 19, 1][..],
        ),
        (
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task263Derived",
            Some("Task263Derived"),
            102,
            190,
            &[4, 0, 11, 1][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "carrier",
            None,
            134,
            155,
            &[4, 0, 11, 1, 18, 0][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "marker",
            None,
            160,
            183,
            &[4, 0, 11, 1, 19, 1][..],
        ),
        (
            SymbolKind::Redefinition,
            DefinitionKind::Redefinition,
            "carrier",
            Some("field carrier from carrier ;"),
            247,
            274,
            &[4, 0, 20, 2, 21, 0][..],
        ),
        (
            SymbolKind::Redefinition,
            DefinitionKind::Redefinition,
            "marker",
            Some("property marker from marker ;"),
            279,
            307,
            &[4, 0, 20, 2, 22, 1][..],
        ),
    ];
    let mut identities = Vec::new();
    for (index, (symbol_kind, definition_kind, spelling, notation, start, end, path)) in
        specs.into_iter().enumerate()
    {
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, start, end)),
            path.to_vec(),
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("task263-{index}")),
            FullyQualifiedName::new(format!("pkg::task263::{index}:{spelling}")),
        );
        let signature = SignatureShell::Opaque {
            schema: "parser-signature-v1".to_owned(),
            payload: format!("task263:{index}"),
        };
        let mut entry = SymbolEntry::new(
            symbol.clone(),
            symbol_kind,
            NamespacePath::new(module.path().as_str()),
            spelling,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
        .with_signature(signature.clone());
        if let Some(notation) = notation {
            entry = entry.with_notation_spelling(notation);
        }
        indexes.symbols.insert(entry);
        let mut shell = DefinitionShell::new(symbol.clone(), definition_kind, origin, contribution)
            .with_visibility(Visibility::Public)
            .with_signature(signature);
        let arity = match arity_override.filter(|(target, _)| *target == index) {
            Some((_, arity)) => arity,
            None if definition_kind == DefinitionKind::Structure => Some(0),
            None => None,
        };
        if let Some(arity) = arity {
            shell = shell.with_arity(arity);
        }
        if let Some(notation) = notation {
            shell = shell.with_notation_shape(notation);
        }
        let definition = indexes.definitions.insert(shell);
        if !stale_effects {
            indexes
                .contributions
                .add_symbol(contribution, symbol.clone());
        }
        indexes
            .contributions
            .add_definition(contribution, definition);
        identities.push((symbol, definition, contribution));
    }
    if stale_effects {
        for index in 0..8 {
            indexes.contributions.add_symbol(
                contribution,
                SymbolId::new(
                    module.clone(),
                    LocalSymbolId::new(format!("task263-stale-{index}")),
                    FullyQualifiedName::new(format!("pkg::task263::stale:{index}")),
                ),
            );
        }
    }
    (SymbolEnv::new(module, indexes), identities)
}

fn source_type_input(source: SourceId, module: ModuleId) -> SourceTypeStructureMemberHandoffInput {
    let specs = [
        (53, 42, 63, 52, 51, 59, 62),
        (56, 68, 91, 55, 54, 87, 90),
        (61, 134, 155, 60, 59, 151, 154),
        (64, 160, 183, 63, 62, 179, 182),
    ];
    SourceTypeStructureMemberHandoffInput {
        source_id: source,
        module_id: module.clone(),
        members: specs
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
            .collect(),
    }
}

fn task263_arena(source: SourceId) -> TypedArena {
    let nodes = (0..75)
        .map(|index| {
            let (kind, source_range) = match index {
                51 | 52 => (
                    if index == 51 {
                        "source.type.head"
                    } else {
                        "source.type.expression"
                    },
                    range(source, 59, 62),
                ),
                53 => ("source.definition.structure.member", range(source, 42, 63)),
                54 | 55 => (
                    if index == 54 {
                        "source.type.head"
                    } else {
                        "source.type.expression"
                    },
                    range(source, 87, 90),
                ),
                56 => ("source.definition.structure.member", range(source, 68, 91)),
                57 => ("source.definition.structure", range(source, 13, 98)),
                59 | 60 => (
                    if index == 59 {
                        "source.type.head"
                    } else {
                        "source.type.expression"
                    },
                    range(source, 151, 154),
                ),
                61 => (
                    "source.definition.structure.member",
                    range(source, 134, 155),
                ),
                62 | 63 => (
                    if index == 62 {
                        "source.type.head"
                    } else {
                        "source.type.expression"
                    },
                    range(source, 179, 182),
                ),
                64 => (
                    "source.definition.structure.member",
                    range(source, 160, 183),
                ),
                65 => ("source.definition.structure", range(source, 102, 190)),
                68 | 69 => (
                    "source.definition.structure.mapping",
                    if index == 68 {
                        range(source, 247, 274)
                    } else {
                        range(source, 279, 307)
                    },
                ),
                70 => (
                    "source.definition.structure.inheritance",
                    range(source, 194, 314),
                ),
                74 => ("source.module", range(source, 0, 319)),
                _ => ("source.unowned", range(source, 0, 1)),
            };
            TypedNode::new(kind, SourceAnchor::Range(source_range))
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal)
        })
        .collect();
    TypedArena::try_new(Some(TypedNodeId::new(74)), nodes).unwrap()
}

fn unrelated_baseline(source: SourceId, key: &str) -> InitialObligationTable {
    let mut table = InitialObligationTable::new();
    table.insert(obligation(source, key));
    table
}

fn obligation(source: SourceId, key: &str) -> InitialObligationDraft {
    let domain = if key.starts_with("source.") {
        key.to_owned()
    } else {
        format!("unrelated.{key}")
    };
    InitialObligationDraft {
        kind: InitialObligationKind::RegistrationCorrectness,
        owner: node_site(74),
        source_range: range(source, 0, 319),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new(domain.clone()),
        provenance: InitialObligationProvenance::new(domain),
        status: InitialObligationStatus::Pending,
    }
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

fn source_id() -> SourceId {
    source_id_for("e2")
}
fn other_source_id() -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "e2".repeat(32)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    allocator.next_source_id(snapshot).unwrap();
    allocator.next_source_id(snapshot).unwrap()
}
fn source_id_for(byte: &str) -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        byte.repeat(32)
    ))
    .unwrap();
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .unwrap()
}
const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
const fn node_site(index: usize) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(index))
}

#[test]
fn task_249s_public_nonuniform_member_ranges_install_and_replay() {
    use crate::source_type::{SourceTypeError, SourceTypeStructureMemberId};
    let source = source_id();
    let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task263"));
    let original_input = source_type_input(source, module.clone());
    let original_arena = task263_arena(source);
    let original =
        SourceTypeStructureMemberProducer::build(original_input.clone(), &original_arena)
            .expect("original public Task249S baseline");
    let mut input = original_input;
    let mut nodes = original_arena
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for (member, (start, end, type_start, type_end)) in input.members.iter_mut().zip([
        (43, 65, 61, 64),
        (70, 95, 91, 94),
        (137, 159, 155, 158),
        (164, 188, 184, 187),
    ]) {
        member.member_range = range(source, start, end);
        member.expression.source_range = range(source, type_start, type_end);
        member.expression.head_range = member.expression.source_range;
        for (site, expected) in [
            (&member.member_site, member.member_range),
            (&member.expression.site, member.expression.source_range),
            (&member.expression.head_site, member.expression.head_range),
        ] {
            nodes[site.node().index()].anchor = SourceAnchor::Range(expected);
        }
    }
    let arena =
        TypedArena::try_new(original_arena.root(), nodes).expect("coherent actual relocated arena");
    assert_eq!(input.members.len(), 4);
    for (index, member) in input.members.iter().enumerate() {
        assert_eq!(member.source_ordinal, index);
        assert_eq!(member.expression.source_id, source);
        assert_eq!(member.expression.module_id, module);
        assert_eq!(member.expression.recovery, NodeRecoveryState::Normal);
        assert_eq!(member.expression.form, SourceTypeApplicationForm::Bare);
        assert_eq!(member.expression.head, SourceTypeHead::BuiltinSet);
        assert_eq!(
            (
                member.expression.spelling.as_str(),
                member.expression.head_spelling.as_str()
            ),
            ("set", "set")
        );
        assert_eq!(
            member
                .expression
                .source_range
                .end
                .checked_sub(member.expression.source_range.start),
            Some(3)
        );
        assert_eq!(member.expression.head_range, member.expression.source_range);
        assert!(member.member_range.start <= member.expression.source_range.start);
        assert!(member.expression.source_range.end < member.member_range.end);
        for (site, expected) in [
            (&member.member_site, member.member_range),
            (&member.expression.site, member.expression.source_range),
            (&member.expression.head_site, member.expression.head_range),
        ] {
            let node = arena.node(site.node()).unwrap();
            assert_eq!(node.anchor, SourceAnchor::Range(expected));
            assert_eq!(node.recovery, NodeRecoveryState::Normal);
        }
    }
    assert!(
        input
            .members
            .windows(2)
            .all(|pair| pair[0].member_range.end <= pair[1].member_range.start)
    );
    let relocated_input = input;
    let relocated_arena = arena;
    let foreign = other_source_id();
    assert_ne!(foreign, source);
    let mut cases = vec![
        ("relocated", 0, None),
        ("wide_expression_head", 0, None),
        ("owner_prefix_slack", 0, None),
        ("member_adjacent", 0, None),
    ];
    for index in 0..4 {
        for label in [
            "normal_arena_owner",
            "input_owner",
            "tail_zero",
            "type_width",
            "head_root",
            "input_empty",
            "input_reversed",
            "input_foreign",
        ] {
            cases.push((
                label,
                index,
                Some(SourceTypeError::InvalidStructureMember {
                    structure_member: SourceTypeStructureMemberId::new(index),
                }),
            ));
        }
        for label in [
            "owner_empty",
            "owner_recovered",
            "owner_missing",
            "owner_foreign",
            "owner_zero",
        ] {
            cases.push((
                label,
                index,
                Some(SourceTypeError::InvalidStructureMemberSite {
                    structure_member: SourceTypeStructureMemberId::new(index),
                }),
            ));
        }
    }
    for label in [
        "later_normal_over_recovered",
        "later_normal_over_role",
        "later_normal_over_unsupported",
        "later_intrinsic_over_bad_slot",
        "member_overlap",
        "member_reverse",
        "overlap_over_recovered",
        "overlap_over_unsupported",
    ] {
        cases.push((
            label,
            1,
            Some(SourceTypeError::InvalidStructureMember {
                structure_member: SourceTypeStructureMemberId::new(1),
            }),
        ));
    }
    cases.push((
        "later_site_over_unsupported",
        1,
        Some(SourceTypeError::InvalidStructureMemberSite {
            structure_member: SourceTypeStructureMemberId::new(1),
        }),
    ));
    assert_eq!(cases.len(), 65);
    let mut order_failures = Vec::new();
    let mut relocated_handoff: Option<SourceTypeApplicationHandoff> = None;
    for (label, index, expected_error) in cases {
        let mut input = relocated_input.clone();
        let mut nodes = relocated_arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        let owner_slot = input.members[index].member_site.node().index();
        let mut root = relocated_arena.root();
        match label {
            "relocated" => {}
            "member_adjacent"
            | "member_overlap"
            | "overlap_over_recovered"
            | "overlap_over_unsupported" => {
                input.members[1].member_range.start =
                    input.members[0].member_range.end - usize::from(label != "member_adjacent");
                nodes[input.members[1].member_site.node().index()].anchor =
                    SourceAnchor::Range(input.members[1].member_range);
            }
            "member_reverse" => {
                let first = (
                    input.members[0].member_range,
                    input.members[0].expression.source_range,
                    input.members[0].expression.head_range,
                );
                let second = (
                    input.members[1].member_range,
                    input.members[1].expression.source_range,
                    input.members[1].expression.head_range,
                );
                for (member, (owner, expression, head)) in
                    input.members[..2].iter_mut().zip([second, first])
                {
                    member.member_range = owner;
                    member.expression.source_range = expression;
                    member.expression.head_range = head;
                    for (site, expected) in [
                        (&member.member_site, owner),
                        (&member.expression.site, expression),
                        (&member.expression.head_site, head),
                    ] {
                        nodes[site.node().index()].anchor = SourceAnchor::Range(expected);
                    }
                }
            }
            "wide_expression_head" => {
                for member in &input.members {
                    let widened = range(
                        source,
                        member.expression.source_range.start - 1,
                        member.expression.source_range.end + 1,
                    );
                    for site in [&member.expression.site, &member.expression.head_site] {
                        nodes[site.node().index()].anchor = SourceAnchor::Range(widened);
                    }
                }
            }
            "owner_prefix_slack" => {
                for member in &mut input.members {
                    member.member_range.start -= 1;
                    nodes[member.member_site.node().index()].anchor =
                        SourceAnchor::Range(member.member_range);
                }
            }
            "normal_arena_owner"
            | "later_normal_over_recovered"
            | "later_normal_over_role"
            | "later_normal_over_unsupported" => {
                let owner = input.members[index].member_range;
                nodes[owner_slot].anchor =
                    SourceAnchor::Range(range(source, owner.start - 1, owner.end));
            }
            "input_owner" => input.members[index].member_range.start -= 1,
            "owner_empty" => {
                let start = input.members[index].member_range.start;
                nodes[owner_slot].anchor = SourceAnchor::Range(range(source, start, start));
            }
            "owner_recovered" | "later_site_over_unsupported" => {
                nodes[owner_slot].recovery = NodeRecoveryState::Recovered
            }
            "owner_missing" => {
                nodes.truncate(owner_slot);
                root = None;
            }
            "owner_foreign" => {
                let mut owner = input.members[index].member_range;
                owner.source_id = foreign;
                nodes[owner_slot].anchor = SourceAnchor::Range(owner);
            }
            "owner_zero" => nodes[owner_slot].anchor = SourceAnchor::Range(range(source, 0, 1)),
            "tail_zero" => {
                input.members[index].member_range.end =
                    input.members[index].expression.source_range.end;
                nodes[owner_slot].anchor = SourceAnchor::Range(input.members[index].member_range);
            }
            "type_width" => {
                input.members[index].expression.source_range.start -= 1;
                input.members[index].expression.head_range =
                    input.members[index].expression.source_range;
            }
            "head_root" => input.members[index].expression.head_range.start -= 1,
            "input_empty" => {
                input.members[index].member_range.end = input.members[index].member_range.start
            }
            "input_reversed" => {
                input.members[index].member_range.end = input.members[index].member_range.start - 1
            }
            "input_foreign" => input.members[index].member_range.source_id = foreign,
            "later_intrinsic_over_bad_slot" => {
                input.members[index].source_ordinal += 1;
                nodes[input.members[0].member_site.node().index()].anchor =
                    SourceAnchor::Range(range(source, 0, 1));
            }
            _ => unreachable!("all explicit cases handled"),
        }
        match label {
            "later_normal_over_recovered" | "overlap_over_recovered" => {
                nodes[input.members[0].member_site.node().index()].recovery =
                    NodeRecoveryState::Recovered
            }
            "later_normal_over_role" => {
                input.members[0].member_site = TypedSiteRef::Role {
                    node: input.members[0].member_site.node(),
                    role: crate::typed_ast::TypeRole::new("member"),
                }
            }
            "later_normal_over_unsupported"
            | "later_site_over_unsupported"
            | "overlap_over_unsupported" => {
                input.members[0].expression.form = SourceTypeApplicationForm::Of
            }
            _ => {}
        }
        let arena = TypedArena::try_new(root, nodes).expect("actual public case arena");
        let order_probe = matches!(
            label,
            "member_adjacent"
                | "member_overlap"
                | "member_reverse"
                | "overlap_over_recovered"
                | "overlap_over_unsupported"
        );
        if order_probe {
            for (row_index, member) in input.members.iter().enumerate() {
                assert_eq!(member.source_ordinal, row_index);
                assert_eq!(
                    (
                        member.member_range.source_id,
                        member.expression.source_id,
                        member.expression.source_range.source_id
                    ),
                    (source, source, source)
                );
                assert_eq!(member.expression.module_id, module);
                assert_eq!(member.expression.head_range, member.expression.source_range);
                assert_eq!(
                    member.expression.source_range.end - member.expression.source_range.start,
                    3
                );
                assert!(member.member_range.start <= member.expression.source_range.start);
                assert!(member.expression.source_range.end < member.member_range.end);
                assert_eq!(
                    member.expression.form,
                    if label == "overlap_over_unsupported" && row_index == 0 {
                        SourceTypeApplicationForm::Of
                    } else {
                        SourceTypeApplicationForm::Bare
                    }
                );
                assert_eq!(member.expression.head, SourceTypeHead::BuiltinSet);
                assert_eq!(
                    (
                        member.expression.spelling.as_str(),
                        member.expression.head_spelling.as_str()
                    ),
                    ("set", "set")
                );
                for (site, expected) in [
                    (&member.member_site, member.member_range),
                    (&member.expression.site, member.expression.source_range),
                    (&member.expression.head_site, member.expression.head_range),
                ] {
                    let node = arena.node(site.node()).unwrap();
                    assert_eq!(node.anchor, SourceAnchor::Range(expected));
                    assert_eq!(
                        node.recovery,
                        if label == "overlap_over_recovered"
                            && row_index == 0
                            && site == &member.member_site
                        {
                            NodeRecoveryState::Recovered
                        } else {
                            NodeRecoveryState::Normal
                        }
                    );
                }
            }
            if label == "member_adjacent" {
                assert_eq!(
                    input.members[0].member_range.end,
                    input.members[1].member_range.start
                );
            } else {
                assert!(input.members[0].member_range.end > input.members[1].member_range.start);
            }
        }
        let input_before = input.clone();
        let arena_before = arena.clone();
        let result = SourceTypeStructureMemberProducer::build(input.clone(), &arena);
        assert_eq!(input, input_before, "{label} row{index}");
        assert_eq!(arena, arena_before, "{label} row{index}");
        if let Some(expected) = expected_error {
            if order_probe {
                if result != Err(expected) {
                    order_failures.push(format!("{label}: {result:?}"));
                }
            } else {
                assert_eq!(result, Err(expected), "{label} row{index}");
            }
            if arena != relocated_arena {
                assert!(
                    relocated_handoff
                        .as_ref()
                        .unwrap()
                        .validate_installation(source, &module, &arena)
                        .is_err(),
                    "installation {label} row{index}"
                );
            }
            continue;
        }
        let handoff = result.unwrap_or_else(|error| panic!("{label}: {error:?}"));
        if label == "relocated" {
            relocated_handoff = Some(handoff.clone());
        } else if label == "wide_expression_head" {
            assert_eq!(Some(&handoff), relocated_handoff.as_ref());
        } else {
            assert_ne!(Some(&handoff), relocated_handoff.as_ref());
        }
        assert_eq!(
            (
                handoff.applications().len(),
                handoff.expressions().len(),
                handoff.arguments().len(),
                handoff.definition_returns().len(),
                handoff.mode_rhs().len(),
                handoff.structure_members().len()
            ),
            (0, 4, 0, 0, 0, 4)
        );
        for (index, ((id, row), member)) in handoff
            .structure_members()
            .iter()
            .zip(&input.members)
            .enumerate()
        {
            assert_eq!(
                (
                    id.index(),
                    row.id().index(),
                    row.source_ordinal(),
                    row.root().index()
                ),
                (index, index, index, index)
            );
            assert_eq!(row.member_site(), &member.member_site);
            assert_eq!(row.member_range(), member.member_range);
            let expression = handoff.expressions().get(row.root()).unwrap();
            assert_eq!(expression.source_id(), source);
            assert_eq!(expression.module_id(), &module);
            assert_eq!(expression.site(), &member.expression.site);
            assert_eq!(expression.source_range(), member.expression.source_range);
            assert_eq!(expression.head_site(), &member.expression.head_site);
            assert_eq!(expression.head_range(), member.expression.head_range);
        }
        let fingerprint = handoff.debug_text();
        assert_ne!(fingerprint, original.debug_text());
        assert_eq!(
            SourceTypeStructureMemberProducer::build(input.clone(), &arena).unwrap(),
            handoff
        );
        let typed = TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: Some(handoff.clone()),
            source_attribute: None,
            nodes: arena,
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("actual relocated Type installation");
        assert_eq!(typed.source_type(), Some(&handoff));
        assert!(typed.source_structure_definition().is_none());
        assert_eq!(typed.clone().source_type(), Some(&handoff));
        assert!(
            typed.types().is_empty() && typed.facts().is_empty() && typed.coercions().is_empty()
        );
        assert!(typed.initial_obligations().is_empty() && typed.diagnostics().is_empty());
        let resolved = assemble_empty(&typed).expect("actual relocated final clone");
        assert_eq!(resolved.source_type(), Some(&handoff));
        assert!(resolved.source_structure_definition().is_none());
        assert!(
            resolved.expr_metadata().is_empty()
                && resolved.inserted_coercions().is_empty()
                && resolved.cluster_facts().is_empty()
                && resolved.diagnostics().is_empty()
        );
        assert_eq!(resolved.clone().source_type(), Some(&handoff));
        assert!(
            resolved.checked_formulas().is_empty()
                && resolved.statement_semantics().is_empty()
                && resolved.checked_proofs().is_empty()
        );
        assert_eq!(typed.debug_text().matches(fingerprint.as_str()).count(), 1);
        assert_eq!(
            resolved.debug_text().matches(fingerprint.as_str()).count(),
            1
        );
    }
    assert!(
        order_failures.is_empty(),
        "member source order failures: {order_failures:?}"
    );
}

#[test]
fn task_263_public_nonuniform_upper_ranges_preserve_real_lower_and_baseline() {
    let original = fixture();
    let baseline = unrelated_baseline(original.source, "layout-baseline");
    assert!(!baseline.is_empty());
    original
        .build(&baseline)
        .expect("original public upper Structure baseline");
    let mut relocated = original.clone();
    let source = relocated.source;
    let block = range(source, 4, 520);
    let definitions = [range(source, 17, 140), range(source, 160, 290)];
    let members = [
        range(source, 50, 72),
        range(source, 90, 118),
        range(source, 190, 216),
        range(source, 230, 260),
    ];
    let types = [
        range(source, 68, 71),
        range(source, 114, 117),
        range(source, 212, 215),
        range(source, 256, 259),
    ];
    let inheritance = range(source, 310, 510);
    let mappings = [range(source, 365, 397), range(source, 405, 440)];
    for (row, expected) in relocated.input.definitions.iter_mut().zip(definitions) {
        row.source_range = expected;
    }
    for (row, expected) in relocated.input.members.iter_mut().zip(members) {
        row.source_range = expected;
    }
    relocated.input.inheritances[0].source_range = inheritance;
    for (row, expected) in relocated.input.mappings.iter_mut().zip(mappings) {
        row.source_range = expected;
    }
    let roles = [
        (51, types[0]),
        (52, types[0]),
        (53, members[0]),
        (54, types[1]),
        (55, types[1]),
        (56, members[1]),
        (57, definitions[0]),
        (59, types[2]),
        (60, types[2]),
        (61, members[2]),
        (62, types[3]),
        (63, types[3]),
        (64, members[3]),
        (65, definitions[1]),
        (68, mappings[0]),
        (69, mappings[1]),
        (70, inheritance),
        (74, block),
    ];
    let mut nodes = relocated
        .arena
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for (node, expected) in roles {
        nodes[node].anchor = SourceAnchor::Range(expected);
    }
    relocated.arena =
        TypedArena::try_new(relocated.arena.root(), nodes).expect("coherent normal public arena");
    for (node, expected) in roles {
        let actual = relocated.arena.node(TypedNodeId::new(node)).unwrap();
        assert_eq!(actual.anchor, SourceAnchor::Range(expected));
        assert_eq!(actual.recovery, NodeRecoveryState::Normal);
    }
    let mut lower_input = source_type_input(source, relocated.module.clone());
    for (index, row) in lower_input.members.iter_mut().enumerate() {
        row.member_range = members[index];
        row.expression.source_range = types[index];
        row.expression.head_range = types[index];
        assert_eq!(row.source_ordinal, index);
        assert_eq!(types[index].end.checked_sub(types[index].start), Some(3));
        assert!(
            members[index].start <= types[index].start && types[index].end < members[index].end
        );
        for (site, expected) in [
            (&row.member_site, members[index]),
            (&row.expression.site, types[index]),
            (&row.expression.head_site, types[index]),
        ] {
            assert_eq!(
                relocated.arena.node(site.node()).unwrap().anchor,
                SourceAnchor::Range(expected)
            );
        }
    }
    assert!(members.windows(2).all(|pair| pair[0].end <= pair[1].start));
    relocated.source_type = SourceTypeStructureMemberProducer::build(lower_input, &relocated.arena)
        .expect("genuine relocated four-member Type prerequisite");
    assert_eq!(
        (
            relocated.source_type.applications().len(),
            relocated.source_type.expressions().len(),
            relocated.source_type.arguments().len(),
            relocated.source_type.definition_returns().len(),
            relocated.source_type.mode_rhs().len(),
            relocated.source_type.structure_members().len()
        ),
        (0, 4, 0, 0, 0, 4)
    );
    for (index, (_, member)) in relocated.source_type.structure_members().iter().enumerate() {
        assert_eq!(
            (
                member.id().index(),
                member.source_ordinal(),
                member.root().index()
            ),
            (index, index, index)
        );
        assert_eq!(member.member_range(), members[index]);
        let expression = relocated
            .source_type
            .expressions()
            .get(member.root())
            .unwrap();
        assert_eq!(
            (expression.source_range(), expression.head_range()),
            (types[index], types[index])
        );
    }
    let lower_typed = relocated.typed(baseline.clone());
    assert_eq!(lower_typed.source_type(), Some(&relocated.source_type));
    assert!(lower_typed.source_structure_definition().is_none());
    assert_eq!(lower_typed.initial_obligations(), &baseline);

    let mut indexes = SymbolEnvIndexes {
        imports: original.env.imports().clone(),
        exports: original.env.exports().clone(),
        symbols: Default::default(),
        labels: original.env.labels().clone(),
        definitions: Default::default(),
        overloads: original.env.overloads().clone(),
        registrations: original.env.registrations().clone(),
        lexical_summaries: original.env.lexical_summaries().clone(),
        namespace_graph: original.env.namespace_graph().clone(),
        declaration_dependencies: original.env.declaration_dependencies().clone(),
        contributions: Default::default(),
        module_summaries: original.env.module_summaries().clone(),
    };
    for old in original.env.contributions().iter() {
        let id = indexes.contributions.insert(
            old.module().clone(),
            old.kind().clone(),
            SourceAnchor::Range(block),
        );
        assert_eq!(id, old.id());
    }
    let origin_ranges = [
        definitions[0],
        members[0],
        members[1],
        definitions[1],
        members[2],
        members[3],
        mappings[0],
        mappings[1],
    ];
    let mut old_definitions = original.env.definitions().iter().collect::<Vec<_>>();
    old_definitions.sort_by_key(|row| row.id().index());
    assert_eq!(old_definitions.len(), 8);
    for old in old_definitions {
        assert!(!old.origin().is_recovered() && old.origin().import_edge().is_none());
        let origin = SemanticOrigin::new(
            old.origin().source_id(),
            old.origin().module_id().clone(),
            SourceAnchor::Range(origin_ranges[old.id().index()]),
            old.origin().structural_path().to_vec(),
        );
        let mut shell =
            DefinitionShell::new(old.symbol().clone(), old.kind(), origin, old.contribution())
                .with_visibility(old.visibility())
                .with_parameters(old.parameters().to_vec())
                .with_binders(old.binders().to_vec())
                .with_dependencies(old.dependencies().to_vec());
        if let Some(arity) = old.arity() {
            shell = shell.with_arity(arity);
        }
        if let Some(notation) = old.notation_shape() {
            shell = shell.with_notation_shape(notation);
        }
        if let Some(doc) = old.doc_attachment() {
            shell = shell.with_doc_attachment(doc.clone());
        }
        if let Some(conflict) = old.conflict() {
            shell = shell.with_conflict(conflict.clone());
        }
        if let Some(signature) = old.signature() {
            shell = shell.with_signature(signature.clone());
        }
        let id = indexes.definitions.insert(shell);
        assert_eq!(id, old.id(), "registered definition IDs stay stable");
        indexes.contributions.add_definition(old.contribution(), id);
    }
    for old in original.env.symbols().iter() {
        let definition = original
            .env
            .definitions()
            .iter()
            .find(|row| row.symbol() == old.symbol())
            .unwrap();
        assert!(!old.origin().is_recovered() && old.origin().import_edge().is_none());
        let origin = SemanticOrigin::new(
            old.origin().source_id(),
            old.origin().module_id().clone(),
            SourceAnchor::Range(origin_ranges[definition.id().index()]),
            old.origin().structural_path().to_vec(),
        );
        let mut symbol = SymbolEntry::new(
            old.symbol().clone(),
            old.kind(),
            old.namespace().clone(),
            old.primary_spelling(),
            origin,
            old.contribution(),
        )
        .with_visibility(old.visibility())
        .with_export_status(old.export_status())
        .with_relations(old.relations().to_vec());
        if let Some(notation) = old.notation_spelling() {
            symbol = symbol.with_notation_spelling(notation);
        }
        if let Some(signature) = old.signature() {
            symbol = symbol.with_signature(signature.clone());
        }
        indexes.symbols.insert(symbol);
        indexes
            .contributions
            .add_symbol(old.contribution(), old.symbol().clone());
    }
    let registered_indexes = indexes.clone();
    relocated.env = SymbolEnv::new(relocated.module.clone(), indexes);
    assert_eq!(
        (
            relocated.env.symbols().len(),
            relocated.env.definitions().len(),
            relocated.env.contributions().len()
        ),
        (8, 8, 1)
    );
    for old in original.env.definitions().iter() {
        let actual = relocated.env.definitions().get(old.id()).unwrap();
        let symbol = relocated.env.symbols().get(actual.symbol()).unwrap();
        assert_eq!(actual.symbol(), old.symbol());
        assert_eq!(actual.kind(), old.kind());
        assert_eq!(actual.arity(), old.arity());
        assert_eq!(actual.signature(), old.signature());
        assert_eq!(actual.origin().source_id(), source);
        assert_eq!(actual.origin().module_id(), &relocated.module);
        assert_eq!(
            actual.origin().structural_path(),
            old.origin().structural_path()
        );
        assert_eq!(
            actual.origin().anchor(),
            &SourceAnchor::Range(origin_ranges[old.id().index()])
        );
        assert_eq!(symbol.origin(), actual.origin());
        assert_eq!(symbol.signature(), actual.signature());
    }
    let contribution = relocated.env.contributions().iter().next().unwrap();
    let old_contribution = original.env.contributions().get(contribution.id()).unwrap();
    assert_eq!(contribution.effects(), old_contribution.effects());
    assert_eq!(
        (
            contribution.effects().symbols().len(),
            contribution.effects().definitions().len()
        ),
        (8, 8)
    );
    assert_eq!(contribution.anchor(), &SourceAnchor::Range(block));
    assert_eq!(
        contribution.kind(),
        &ContributionKind::LocalSource { source_id: source }
    );
    assert_eq!(contribution.module(), &relocated.module);
    for owned in origin_ranges.into_iter().chain([inheritance]) {
        assert!(block.start <= owned.start && owned.end <= block.end);
    }
    for index in 0..2 {
        assert!(
            definitions[index].start <= members[2 * index].start
                && members[2 * index + 1].end < definitions[index].end
        );
    }
    assert!(definitions[0].end <= definitions[1].start && definitions[1].end <= inheritance.start);
    assert!(
        inheritance.start <= mappings[0].start
            && mappings[0].end <= mappings[1].start
            && mappings[1].end < inheritance.end
    );
    let projection = relocated
        .build(&baseline)
        .expect("coherent nonuniform public upper Structure layout");
    assert_eq!(projection.base_initial_obligations(), &baseline);
    assert_eq!(projection.initial_obligations(), &baseline);
    let handoff = projection.handoff();
    assert_eq!(
        (
            handoff.definitions().len(),
            handoff.members().len(),
            handoff.inheritances().len(),
            handoff.mappings().len(),
            handoff.coherence_requests().len()
        ),
        (2, 4, 1, 2, 0)
    );
    assert_eq!(
        handoff.source_type_fingerprint(),
        relocated.source_type.debug_text()
    );
    assert_eq!(handoff.base_initial_obligation_count(), baseline.len());
    for ((_, row), input) in handoff
        .definitions()
        .iter()
        .zip(&relocated.input.definitions)
    {
        assert_eq!(row.source_range(), input.source_range);
        assert_eq!(
            row.origin(),
            relocated
                .env
                .definitions()
                .get(input.definition)
                .unwrap()
                .origin()
        );
    }
    for ((_, row), input) in handoff.members().iter().zip(&relocated.input.members) {
        assert_eq!(row.source_range(), input.source_range);
        assert_eq!(
            row.origin(),
            relocated
                .env
                .definitions()
                .get(input.definition)
                .unwrap()
                .origin()
        );
    }
    for ((_, row), input) in handoff.mappings().iter().zip(&relocated.input.mappings) {
        assert_eq!(row.source_range(), input.source_range);
        assert_eq!(
            row.origin(),
            relocated
                .env
                .definitions()
                .get(input.definition)
                .unwrap()
                .origin()
        );
    }
    assert_eq!(
        handoff
            .inheritances()
            .iter()
            .next()
            .unwrap()
            .1
            .source_range(),
        inheritance
    );
    let typed = lower_typed
        .with_source_structure_definition(projection.clone())
        .expect("actual upper install with unchanged baseline");
    assert_eq!(typed.initial_obligations(), &baseline);
    assert_eq!(typed.source_structure_definition(), Some(handoff));
    let resolved = assemble_empty(&typed).expect("actual final clone");
    assert_eq!(resolved.source_structure_definition(), Some(handoff));
    assert_eq!(
        resolved.clone().source_structure_definition(),
        Some(handoff)
    );
    assert_eq!(relocated.build(&baseline).unwrap(), projection);
    assert!(typed.types().is_empty() && typed.facts().is_empty() && typed.coercions().is_empty());
    assert!(
        resolved.checked_formulas().is_empty()
            && resolved.statement_semantics().is_empty()
            && resolved.checked_proofs().is_empty()
            && resolved.cluster_facts().is_empty()
    );
    let mut cases = Vec::new();
    for index in 0..2 {
        for label in ["d_header", "d_tail", "h_width", "d_origin", "h_origin"] {
            cases.push((label, index));
        }
    }
    for index in 0..4 {
        for label in ["m_origin", "m_prefix"] {
            cases.push((label, index));
        }
    }
    for label in [
        "i_header",
        "i_tail",
        "i_input",
        "i_arena",
        "i_recovered",
        "i_zero",
        "i_missing",
        "d_adjacent",
        "wide_rhs",
        "no_root",
        "mapping_before_prefix",
        "b_prefix",
        "b_tail",
        "d_overlap",
        "h_overlap",
        "m_overlap",
        "member_escape",
    ] {
        cases.push((label, 0));
    }
    assert_eq!(cases.len(), 35);
    for (label, mut index) in cases {
        let mut changed = handoff.clone();
        let mut nodes = relocated
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        let mut root = relocated.arena.root();
        let mut expected = None;
        let linked = match label {
            "d_header" | "d_tail" | "d_origin" | "d_adjacent" | "d_overlap" => {
                if label == "d_overlap" {
                    index = 1;
                }
                let row = &mut changed.definitions.rows[index];
                let mut new_range = row.source_range;
                match label {
                    "d_header" => new_range.start = members[2 * index].start - [25, 28][index] + 1,
                    "d_tail" => new_range.end = members[2 * index + 1].end + 3,
                    "d_adjacent" => new_range.end = definitions[1].start,
                    "d_overlap" => new_range.start = definitions[0].end - 1,
                    _ => new_range.start += 1,
                }
                if label != "d_adjacent" {
                    expected = Some(if label == "d_overlap" {
                        SourceStructureDefinitionError::InvalidResolverDefinition { index: 0 }
                    } else {
                        SourceStructureDefinitionError::InvalidDefinition { index }
                    });
                }
                Some((
                    &mut row.source_range,
                    &mut row.origin,
                    row.site.clone(),
                    new_range,
                    label != "d_origin",
                ))
            }
            "h_width" | "h_origin" | "h_overlap" => {
                if label == "h_overlap" {
                    index = 1;
                }
                let row = &mut changed.mappings.rows[index];
                let mut new_range = row.source_range;
                match label {
                    "h_width" => new_range.end = new_range.start + [27, 28][index] - 1,
                    "h_overlap" => new_range.start = mappings[0].end - 1,
                    _ => new_range.start += 1,
                }
                expected = Some(if label == "h_overlap" {
                    SourceStructureDefinitionError::InvalidResolverDefinition { index: 0 }
                } else {
                    SourceStructureDefinitionError::InvalidMapping { index }
                });
                Some((
                    &mut row.source_range,
                    &mut row.origin,
                    row.site.clone(),
                    new_range,
                    label != "h_origin",
                ))
            }
            "m_origin" | "m_overlap" | "member_escape" => {
                if label != "m_origin" {
                    index = 1;
                }
                let row = &mut changed.members.rows[index];
                let mut new_range = row.source_range;
                match label {
                    "m_overlap" => new_range.start = members[0].end - 1,
                    "member_escape" => new_range.end = definitions[0].end + 1,
                    _ => new_range.start += 1,
                }
                expected = Some(match label {
                    "m_overlap" => {
                        SourceStructureDefinitionError::InvalidResolverDefinition { index: 0 }
                    }
                    "member_escape" => {
                        SourceStructureDefinitionError::InvalidDefinition { index: 0 }
                    }
                    _ => SourceStructureDefinitionError::InvalidMember { index },
                });
                Some((
                    &mut row.source_range,
                    &mut row.origin,
                    row.site.clone(),
                    new_range,
                    label != "m_origin",
                ))
            }
            _ => None,
        };
        if let Some((row_range, origin, site, new_range, joined)) = linked {
            assert!(new_range.start < new_range.end);
            *origin = SemanticOrigin::new(
                origin.source_id(),
                origin.module_id().clone(),
                SourceAnchor::Range(new_range),
                origin.structural_path().to_vec(),
            );
            assert_eq!(origin.source_id(), source);
            assert_eq!(origin.module_id(), &relocated.module);
            assert!(!origin.is_recovered() && origin.import_edge().is_none());
            assert_eq!(origin.anchor(), &SourceAnchor::Range(new_range));
            if joined {
                *row_range = new_range;
                nodes[site.node().index()].anchor = SourceAnchor::Range(new_range);
                assert_eq!(*row_range, new_range);
                assert_eq!(nodes[site.node().index()].anchor, *origin.anchor());
            } else {
                assert_ne!(*row_range, new_range);
                match label {
                    "d_origin" => {
                        assert!(
                            new_range.start <= members[2 * index].start
                                && members[2 * index + 1].end < new_range.end
                        );
                    }
                    "m_origin" => {
                        assert!(
                            new_range.start <= types[index].start
                                && types[index].end < new_range.end
                        );
                    }
                    "h_origin" => {
                        assert!(
                            inheritance.start < new_range.start && new_range.end < inheritance.end
                        );
                    }
                    _ => unreachable!(),
                }
            }
        }
        match label {
            "i_header" | "i_tail" | "i_input" | "i_arena" => {
                let mut new_range = inheritance;
                match label {
                    "i_header" => new_range.start = mappings[0].start - 48,
                    "i_tail" => new_range.end = mappings[1].end + 3,
                    _ => new_range.start += 1,
                }
                if label != "i_arena" {
                    changed.inheritances.rows[0].source_range = new_range;
                }
                if label != "i_input" {
                    nodes[70].anchor = SourceAnchor::Range(new_range);
                }
                expected = Some(SourceStructureDefinitionError::InvalidInheritance { index: 0 });
            }
            "i_recovered" => {
                nodes[70].recovery = NodeRecoveryState::Recovered;
                nodes[70].anchor =
                    SourceAnchor::Range(range(source, inheritance.start + 1, inheritance.end));
                assert_eq!(nodes[70].recovery, NodeRecoveryState::Recovered);
            }
            "i_zero" => nodes[70].anchor = SourceAnchor::Range(range(source, 0, 1)),
            "i_missing" => {
                nodes.truncate(70);
                root = None;
            }
            "wide_rhs" => {
                for row in &relocated.input.members {
                    let expression = relocated
                        .source_type
                        .expressions()
                        .get(SourceTypeExpressionId::new(row.written_type.index()))
                        .unwrap();
                    let actual = expression.source_range();
                    for site in [expression.site(), expression.head_site()] {
                        nodes[site.node().index()].anchor =
                            SourceAnchor::Range(range(source, actual.start - 1, actual.end + 1));
                    }
                }
            }
            "no_root" => {
                root = None;
                nodes[74] =
                    TypedNode::new("source.unowned", SourceAnchor::Range(range(source, 0, 1)))
                        .with_recovery(NodeRecoveryState::Normal);
            }
            _ => {}
        }
        match label {
            "d_header" => assert_eq!(
                members[2 * index].start - changed.definitions.rows[index].source_range.start,
                [25, 28][index] - 1
            ),
            "d_tail" => assert_eq!(
                changed.definitions.rows[index].source_range.end - members[2 * index + 1].end,
                3
            ),
            "h_width" => assert_eq!(
                changed.mappings.rows[index].source_range.end
                    - changed.mappings.rows[index].source_range.start,
                [27, 28][index] - 1
            ),
            "i_header" => assert_eq!(
                mappings[0].start - changed.inheritances.rows[0].source_range.start,
                48
            ),
            "i_tail" => assert_eq!(
                changed.inheritances.rows[0].source_range.end - mappings[1].end,
                3
            ),
            "i_input" | "i_arena" | "i_recovered" => {
                let SourceAnchor::Range(reference) = &nodes[70].anchor else {
                    unreachable!()
                };
                let reference = *reference;
                assert_ne!(reference, changed.inheritances.rows[0].source_range);
                for value in [reference, changed.inheritances.rows[0].source_range] {
                    assert_eq!(value.source_id, source);
                    assert!(
                        mappings[0].start - value.start >= 49 && value.end - mappings[1].end >= 4
                    );
                }
            }
            "d_adjacent" => assert_eq!(
                changed.definitions.rows[0].source_range.end,
                changed.definitions.rows[1].source_range.start
            ),
            _ => {}
        }
        let mut lower = relocated.source_type.clone();
        if matches!(
            label,
            "m_prefix" | "mapping_before_prefix" | "member_escape" | "m_overlap"
        ) {
            let mut lower_input = source_type_input(source, relocated.module.clone());
            for (member_index, row) in lower_input.members.iter_mut().enumerate() {
                row.member_range = changed.members.rows[member_index].source_range;
                let mut actual = types[member_index];
                if matches!(label, "m_prefix" | "mapping_before_prefix") && member_index == index {
                    actual.start = members[index].start + if index % 2 == 0 { 14 } else { 16 };
                    actual.end = actual.start + 3;
                }
                row.expression.source_range = actual;
                row.expression.head_range = actual;
                for site in [&row.expression.site, &row.expression.head_site] {
                    nodes[site.node().index()].anchor = SourceAnchor::Range(actual);
                }
                assert!(
                    row.member_range.start <= actual.start && actual.end < row.member_range.end
                );
                assert_eq!(actual.end - actual.start, 3);
            }
            let arena = TypedArena::try_new(root, nodes.clone()).unwrap();
            let result = SourceTypeStructureMemberProducer::build(lower_input, &arena);
            if label == "m_overlap" {
                assert!(
                    matches!(result, Err(crate::source_type::SourceTypeError::InvalidStructureMember { structure_member }) if structure_member.index() == 1),
                    "lower correctly refuses the deliberate overlap; upper Resolver remains earlier"
                );
            } else {
                lower = result.expect("real lower succeeds before testing the upper-only geometry");
                assert_eq!(
                    (
                        lower.applications().len(),
                        lower.expressions().len(),
                        lower.structure_members().len()
                    ),
                    (0, 4, 4)
                );
                changed.source_type_fingerprint = lower.debug_text();
            }
            if matches!(label, "m_prefix" | "mapping_before_prefix") {
                let mut fixture = relocated.clone();
                fixture.source_type = lower;
                fixture.arena = arena;
                if label == "mapping_before_prefix" {
                    fixture.input.mappings[1].path.clear();
                }
                let error = if label == "mapping_before_prefix" {
                    SourceStructureDefinitionError::InvalidMapping { index: 1 }
                } else {
                    SourceStructureDefinitionError::InvalidMember { index }
                };
                let expression = fixture
                    .source_type
                    .expressions()
                    .get(SourceTypeExpressionId::new(index))
                    .unwrap();
                assert_eq!(
                    expression.source_range().start
                        - fixture.input.members[index].source_range.start,
                    if index % 2 == 0 { 14 } else { 16 }
                );
                let before_input = fixture.input.clone();
                let before_lower = fixture.source_type.clone();
                let before_arena = fixture.arena.clone();
                assert_eq!(fixture.build(&baseline), Err(error));
                assert_eq!(fixture.input, before_input);
                assert_eq!(fixture.source_type, before_lower);
                assert_eq!(fixture.arena, before_arena);
                continue;
            }
        }
        if matches!(label, "b_prefix" | "b_tail") {
            let mut indexes = registered_indexes.clone();
            indexes.contributions = Default::default();
            let actual_block = if label == "b_prefix" {
                range(source, definitions[0].start - 10, block.end)
            } else {
                range(source, block.start, inheritance.end + 3)
            };
            for old in relocated.env.contributions().iter() {
                let id = indexes.contributions.insert(
                    old.module().clone(),
                    old.kind().clone(),
                    SourceAnchor::Range(actual_block),
                );
                assert_eq!(id, old.id());
                for symbol in old.effects().symbols() {
                    indexes.contributions.add_symbol(id, symbol.clone());
                }
                for definition in old.effects().definitions() {
                    indexes.contributions.add_definition(id, *definition);
                }
                assert_eq!(
                    indexes.contributions.get(id).unwrap().effects(),
                    old.effects()
                );
            }
            let mut fixture = relocated.clone();
            fixture.env = SymbolEnv::new(relocated.module.clone(), indexes);
            assert_eq!(fixture.env.symbols(), relocated.env.symbols());
            assert_eq!(fixture.env.definitions(), relocated.env.definitions());
            assert_eq!(
                fixture.env.contributions().iter().next().unwrap().anchor(),
                &SourceAnchor::Range(actual_block)
            );
            let error = if label == "b_prefix" {
                SourceStructureDefinitionError::InvalidResolverDefinition { index: 0 }
            } else {
                SourceStructureDefinitionError::InvalidInheritance { index: 0 }
            };
            let before_env = fixture.env.clone();
            let before_input = fixture.input.clone();
            assert_eq!(fixture.build(&baseline), Err(error));
            assert_eq!(fixture.env, before_env);
            assert_eq!(fixture.input, before_input);
            continue;
        }
        let arena = TypedArena::try_new(root, nodes).unwrap();
        let before = changed.clone();
        let lower_before = lower.clone();
        let arena_before = arena.clone();
        if matches!(label, "i_recovered" | "i_zero" | "i_missing") {
            let different_baseline = unrelated_baseline(source, "different-baseline");
            assert_eq!(different_baseline.len(), baseline.len());
            assert_ne!(different_baseline, baseline);
            assert_installation_error(
                &relocated,
                &changed,
                &lower,
                &baseline,
                &arena,
                SourceStructureDefinitionError::InvalidArenaOwnership,
            );
            assert_installation_error(
                &relocated,
                &changed,
                &lower,
                &different_baseline,
                &arena,
                SourceStructureDefinitionError::InvalidObligation,
            );
            assert_eq!(changed, before, "{label}");
            assert_eq!(lower, lower_before, "{label}");
            assert_eq!(arena, arena_before, "{label}");
            continue;
        }
        if let Some(error) = expected {
            assert_installation_error(&relocated, &changed, &lower, &baseline, &arena, error);
        } else {
            changed
                .validate_installation(source, &relocated.module, &lower, &baseline, &arena)
                .unwrap_or_else(|error| panic!("{label}: {error:?}"));
            let mut fixture = relocated.clone();
            fixture.arena = arena.clone();
            fixture.source_type = lower.clone();
            let mut actual_projection = projection.clone();
            actual_projection.handoff = changed.clone();
            let typed = fixture
                .typed(baseline.clone())
                .with_source_structure_definition(actual_projection)
                .expect("valid geometry/reference slack installs");
            assert_eq!(typed.initial_obligations(), &baseline);
            assert_eq!(typed.source_structure_definition(), Some(&changed));
            let resolved = assemble_empty(&typed).unwrap();
            assert_eq!(resolved.source_structure_definition(), Some(&changed));
            assert_eq!(
                resolved.clone().source_structure_definition(),
                Some(&changed)
            );
            if label != "d_adjacent" {
                assert_eq!(&changed, handoff);
            }
        }
        assert_eq!(changed, before, "{label}");
        assert_eq!(lower, lower_before, "{label}");
        assert_eq!(arena, arena_before, "{label}");
    }
}
