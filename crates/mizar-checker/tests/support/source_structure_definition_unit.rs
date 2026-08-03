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
        resolver_env_with_effects(fixture.source, fixture.module.clone(), true);
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
    resolver_env_with_effects(source, module, false)
}

fn resolver_env_with_effects(
    source: SourceId,
    module: ModuleId,
    stale_effects: bool,
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
