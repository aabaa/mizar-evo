use crate::{
    cluster_trace::{
        ClusterAttributeFingerprint, ClusterFactDraft, ClusterFactFingerprint, ClusterFactInput,
        ClusterFactProvenance, ClusterFactTable, ClusterRuleDraft, ClusterRuleFingerprint,
        ClusterRuleInput, ClusterRuleKind, ClusterStepId, ClusterTraceBuilder,
        ClusterTypeFingerprint, ReductionBinding, ReductionDraft, ReductionFingerprint,
        ReductionGuardEvidenceRef, ReductionGuardKind, ReductionGuardRequirement, ReductionInput,
        ReductionRedexPath, ReductionRuleFqn, ReductionRuleViewFingerprint, ReductionSelectionKey,
        ReductionStrategyAuditKey, ReductionTermFingerprint, ReductionTraceBuilder,
        ResolutionTrace, ResolutionTraceStep,
    },
    overload_resolution::{
        ArgumentViabilityEvidence, CandidateDeclarationKind, CandidateOrigin, CandidateProvenance,
        CandidateProvenanceKey, CandidateScope, CandidateViabilityInput, CandidateViabilityOutput,
        CoherenceStatus, ExposedResultPayload, ExposedResultSource, InsertedViewInput,
        InsertedViewKind, InsertedViewReasonKey, InsertedViewStatus, OverloadCandidateId,
        OverloadCandidateInput, OverloadCandidateTable, OverloadCollectionOutput, OverloadNameKey,
        OverloadSelectionOutput, OverloadSiteId, OverloadSiteInput, OverloadSiteKey,
        OverloadSiteKind, OverloadSiteRecovery, OverloadSiteResolutionInput, QuaPathKey,
        RefinementJoinPayload, RefinementJoinStatus, SourceQuaView, SpecificityComparisonInput,
        SpecificityComparisonStatus, SpecificityGraphOutput, SpecificityReasonKey,
        TemplateArgument, TemplateCandidatePayload, TemplateConstraintEvidence,
        TemplateConstraintEvidenceStatus, TemplateExpansionOutput, TemplateInstantiationKey,
        TemplateParameterKey,
    },
    registration_resolution::{
        ActivationInput, CheckerRegistrationId, RegistrationDatabase, RegistrationFingerprint,
        RegistrationTriggerKey,
    },
    resolved_typed_ast::{
        ExprId, ExpressionMetadataInput, ResolvedTypedAst, ResolvedTypedAstInputs,
    },
    type_checker::{
        TypeExpressionInput, TypeFactQuery, TypeFactQueryEngine, TypeFactQueryStatus,
        TypeHeadInput, TypeNormalizer,
    },
    typed_ast::{
        BuiltinRuleId, CoercionTable, ContextRecoveryState, FactProvenance, FactStatus,
        InitialObligationTable, LocalTypeContextDraft, LocalTypeContextId, LocalTypeContextTable,
        NormalizedTypeId, OpenCandidateSetId, Polarity, TypeContextLayer, TypeDiagnosticTable,
        TypeEntryActual, TypeEntryDraft, TypeFactDraft, TypeFactId, TypeFactTable,
        TypePredicateRef, TypeProvenance, TypeRole, TypeRuleId, TypeStatus, TypeTable,
        TypedArenaBuilder, TypedAst, TypedAstParts, TypedNode, TypedNodeId, TypedNodeLinks,
        TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    env::{
        ContributionKind, RegistrationId as ResolverRegistrationId, RegistrationIndex,
        RegistrationKind as ResolverRegistrationKind, SignatureShell, SourceContributionId,
        SourceContributionIndex, SymbolEnv, SymbolEnvIndexes,
    },
    resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SemanticOrigin, SymbolId},
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    SourceAnchor, SourceId, SourceRange,
};

#[test]
fn type_outputs_and_fact_queries_are_deterministic() {
    let first = type_snapshot(false);
    let second = type_snapshot(false);
    assert_eq!(first, second);

    let permuted = type_snapshot(true);
    assert_eq!(first.normalization, permuted.normalization);
    assert_eq!(first.fact_query_projection, permuted.fact_query_projection);
    assert_eq!(first.active_facts, [TypeFactId::new(0), TypeFactId::new(1)]);
    assert!(first.fact_query.contains("status=contradicted"));
    assert!(first.fact_query.contains("checker.fact.contradiction"));
}

#[test]
fn cluster_trace_outputs_are_deterministic_across_equivalent_orders() {
    let first = cluster_snapshot(false);
    let second = cluster_snapshot(false);
    let permuted = cluster_snapshot(true);

    assert_eq!(first, second);
    assert_eq!(first, permuted);
    assert_eq!(first.generated_facts, ["fact:B", "fact:C"]);
    assert_eq!(first.closure_facts, ["fact:A", "fact:B", "fact:C"]);
    assert!(first.diagnostics.is_empty());
}

#[test]
fn reduction_trace_identity_tracks_discharged_side_condition_set() {
    let first = reduction_snapshot("evidence:such:P", false);
    let second = reduction_snapshot("evidence:such:P", false);
    let permuted = reduction_snapshot("evidence:such:P", true);
    let alternate_such = reduction_snapshot("evidence:such:alternate", false);

    assert_eq!(first, second);
    assert_eq!(first, permuted);
    assert_ne!(first.debug, alternate_such.debug);
    assert_ne!(first.discharged_guards, alternate_such.discharged_guards);
    assert_eq!(first.strategy_audit, alternate_such.strategy_audit);
    assert!(!first.strategy_audit.contains("such"));
    assert!(first.debug.contains("such:guard:such:P=evidence:such:P"));
    assert!(
        alternate_such
            .debug
            .contains("such:guard:such:P=evidence:such:alternate")
    );
}

#[test]
fn overload_pipeline_and_resolved_projection_are_deterministic() {
    let first = overload_resolved_snapshot(false);
    let second = overload_resolved_snapshot(false);
    let permuted = overload_resolved_snapshot(true);

    assert_eq!(first, second);
    assert_eq!(first, permuted);
    assert_eq!(first.inserted_view_count, 2);
    assert!(first.collection.contains("overload.site.duplicate_key"));
    assert!(first.collection.contains("overload.candidate.missing_site"));
    assert!(first.expansion.contains("origin=template_derived"));
    assert!(first.selection.contains("status=resolved(root=candidate#"));
    assert!(first.resolved.contains("inserted-coercions:"));
    assert!(
        first
            .resolved
            .contains("key=\"overload.site.duplicate_key\"")
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypeDeterminismSnapshot {
    normalization: String,
    fact_query: String,
    fact_query_projection: FactQueryProjection,
    active_facts: Vec<TypeFactId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FactQueryProjection {
    status: TypeFactQueryStatus,
    matched_count: usize,
    active_count: usize,
    diagnostic_keys: Vec<String>,
}

fn type_snapshot(reverse_types: bool) -> TypeDeterminismSnapshot {
    let source = source_id(10);
    let symbols = SymbolEnv::new(module(), SymbolEnvIndexes::default());
    let mut inputs = vec![
        type_input(
            source,
            20,
            TypedSiteRef::Node(TypedNodeId::new(2)),
            "object",
            TypeHeadInput::BuiltinObject,
        ),
        type_input(
            source,
            10,
            TypedSiteRef::Node(TypedNodeId::new(1)),
            "set",
            TypeHeadInput::BuiltinSet,
        ),
        type_input(
            source,
            30,
            TypedSiteRef::Node(TypedNodeId::new(3)),
            "MissingMode",
            TypeHeadInput::Unresolved("MissingMode".to_owned()),
        ),
    ];
    if reverse_types {
        inputs.reverse();
    }
    let normalization = TypeNormalizer::default()
        .normalize(&symbols, inputs)
        .debug_text();

    let subject = TypedSiteRef::Node(TypedNodeId::new(7));
    let predicate = TypePredicateRef::new("is_set_like");
    let mut facts = TypeFactTable::new();
    let mut fact_inputs = vec![
        TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Builtin(BuiltinRuleId::new("fixture-positive")),
            status: FactStatus::Known,
        },
        TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Negative,
            provenance: FactProvenance::Inferred(TypeRuleId::new("fixture-negative")),
            status: FactStatus::Known,
        },
    ];
    if reverse_types {
        fact_inputs.reverse();
    }
    for fact in fact_inputs {
        facts.insert(fact);
    }
    let engine = TypeFactQueryEngine::new(&facts);
    let query_output = engine.query(TypeFactQuery::new(
        subject,
        predicate,
        Polarity::Positive,
        range(source, 50, 51),
    ));
    let active_facts = engine.active_facts(None);
    let fact_query_projection = FactQueryProjection {
        status: query_output.status(),
        matched_count: query_output.matched_facts().len(),
        active_count: active_facts.len(),
        diagnostic_keys: query_output
            .diagnostics()
            .canonical_iter()
            .map(|(_, diagnostic)| diagnostic.message_key.clone())
            .collect(),
    };
    let fact_query = query_output.debug_text();

    TypeDeterminismSnapshot {
        normalization,
        fact_query,
        fact_query_projection,
        active_facts,
    }
}

fn type_input(
    source: SourceId,
    start: usize,
    site: TypedSiteRef,
    spelling: &str,
    head: TypeHeadInput,
) -> TypeExpressionInput {
    TypeExpressionInput::new(
        site,
        range(source, start, start + spelling.len()),
        spelling,
        head,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClusterDeterminismSnapshot {
    closure: String,
    generated_facts: Vec<String>,
    closure_facts: Vec<String>,
    diagnostics: Vec<String>,
}

fn cluster_snapshot(reverse: bool) -> ClusterDeterminismSnapshot {
    let fixture = cluster_env_fixture(reverse);
    let mut rules = vec![
        cluster_rule(
            fixture.cluster_b,
            "trigger:B",
            "fact:B",
            "fact:C",
            30,
            fixture.source,
        ),
        cluster_rule(
            fixture.cluster_a,
            "trigger:A",
            "fact:A",
            "fact:B",
            20,
            fixture.source,
        ),
    ];
    if reverse {
        rules.reverse();
    }
    let output = ClusterTraceBuilder::default().close(
        &fixture.database,
        fixture.source,
        fixture.module,
        [cluster_fact("fact:A", "attr:A", 10, fixture.source)],
        rules,
    );

    ClusterDeterminismSnapshot {
        closure: output.debug_text(),
        generated_facts: generated_cluster_facts(output.trace()),
        closure_facts: cluster_fact_fingerprints(output.closure_facts()),
        diagnostics: output
            .diagnostics()
            .canonical_iter()
            .map(|(_, diagnostic)| diagnostic.message_key().to_owned())
            .collect(),
    }
}

struct ClusterEnvFixture {
    database: RegistrationDatabase,
    source: SourceId,
    module: ModuleId,
    cluster_a: CheckerRegistrationId,
    cluster_b: CheckerRegistrationId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReductionTraceSnapshot {
    debug: String,
    discharged_guards: Vec<String>,
    strategy_audit: String,
}

fn reduction_snapshot(such_evidence: &str, reverse_guard_order: bool) -> ReductionTraceSnapshot {
    let fixture = reduction_env_fixture();
    let input = reduction_input(&fixture, such_evidence, reverse_guard_order);
    let output = ReductionTraceBuilder::new().record(
        &fixture.database,
        fixture.source,
        fixture.module,
        [input],
    );

    assert!(output.diagnostics().is_empty());
    let [ResolutionTraceStep::Reduction(step)] = output.trace().steps() else {
        panic!("expected one reduction step");
    };
    ReductionTraceSnapshot {
        debug: output.debug_text(),
        discharged_guards: step
            .discharged_guards()
            .iter()
            .map(|guard| {
                format!(
                    "{}:{}={}",
                    reduction_guard_kind_name(guard.kind()),
                    guard.guard().as_str(),
                    guard.evidence().as_str()
                )
            })
            .collect(),
        strategy_audit: step.strategy_audit_key().as_str().to_owned(),
    }
}

struct ReductionEnvFixture {
    database: RegistrationDatabase,
    source: SourceId,
    module: ModuleId,
    reduction: CheckerRegistrationId,
}

fn reduction_env_fixture() -> ReductionEnvFixture {
    let source = source_id(32);
    let module = module();
    let mut registrations = RegistrationIndex::new();
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source, 40);
    let reduction = insert_registration_with_kind(
        &mut registrations,
        module.clone(),
        source,
        contribution,
        ResolverRegistrationKind::Reduction,
        "ReduceR",
        40,
    );
    contributions.add_registration(contribution, reduction);

    let env = SymbolEnv::new(
        module.clone(),
        SymbolEnvIndexes {
            registrations,
            contributions,
            ..SymbolEnvIndexes::default()
        },
    );
    let database = RegistrationDatabase::from_symbol_env(
        &env,
        [activation_with_kind(
            reduction,
            ResolverRegistrationKind::Reduction,
            "trigger:R",
            "fingerprint:R",
        )],
    );

    ReductionEnvFixture {
        database,
        source,
        module,
        reduction: CheckerRegistrationId::new(reduction.index()),
    }
}

fn reduction_input(
    fixture: &ReductionEnvFixture,
    such_evidence: &str,
    reverse_guard_order: bool,
) -> ReductionInput {
    let enclosing = ReductionTermFingerprint::new("term:before:R");
    let redex_path = ReductionRedexPath::new("path:0.1");
    let rule_view = ReductionRuleViewFingerprint::new("fingerprint:R");
    let selection_key = ReductionSelectionKey::new("selection:R");
    let strategy_audit_key = ReductionStrategyAuditKey::new(format!(
        "enclosing={};redex_path={};rule_view={};selection={}",
        enclosing.as_str(),
        redex_path.as_str(),
        rule_view.as_str(),
        selection_key.as_str()
    ));
    let mut discharged_guards = vec![
        ReductionGuardEvidenceRef::new(ReductionGuardKind::Type, "guard:type:T", "evidence:type:T"),
        ReductionGuardEvidenceRef::new(
            ReductionGuardKind::Attribute,
            "guard:attr:A",
            "evidence:attr:A",
        ),
        ReductionGuardEvidenceRef::new(ReductionGuardKind::Such, "guard:such:P", such_evidence),
    ];
    if reverse_guard_order {
        discharged_guards.reverse();
    }

    ReductionInput::new(ReductionDraft {
        registration: fixture.reduction,
        trigger: RegistrationTriggerKey::new("trigger:R"),
        applied_reduction: ReductionFingerprint::new("reduction:R"),
        rule_fqn: ReductionRuleFqn::new("pkg::main::ReduceR"),
        enclosing_term_before: enclosing,
        redex_path,
        source_redex: ReductionTermFingerprint::new("term:redex:R"),
        target_term: ReductionTermFingerprint::new("term:target:R"),
        rule_view,
        selection_key,
        strategy_audit_key,
        source_range: range(fixture.source, 40, 41),
    })
    .with_substitution([ReductionBinding::new("var:x", "term:x")])
    .with_required_guards([
        ReductionGuardRequirement::new(ReductionGuardKind::Type, "guard:type:T"),
        ReductionGuardRequirement::new(ReductionGuardKind::Attribute, "guard:attr:A"),
        ReductionGuardRequirement::new(ReductionGuardKind::Such, "guard:such:P"),
    ])
    .with_discharged_guards(discharged_guards)
}

fn reduction_guard_kind_name(kind: ReductionGuardKind) -> &'static str {
    match kind {
        ReductionGuardKind::Type => "type",
        ReductionGuardKind::Attribute => "attribute",
        ReductionGuardKind::Such => "such",
    }
}

fn cluster_env_fixture(reverse_activations: bool) -> ClusterEnvFixture {
    let source = source_id(11);
    let module = module();
    let mut contributions = SourceContributionIndex::new();
    let contribution_a = contribution(&mut contributions, module.clone(), source, 0);
    let contribution_b = contribution(&mut contributions, module.clone(), source, 1);

    let mut registrations = RegistrationIndex::new();
    let cluster_a = insert_registration(
        &mut registrations,
        module.clone(),
        source,
        contribution_a,
        "ACluster",
        0,
    );
    let cluster_b = insert_registration(
        &mut registrations,
        module.clone(),
        source,
        contribution_b,
        "BCluster",
        1,
    );
    contributions.add_registration(contribution_a, cluster_a);
    contributions.add_registration(contribution_b, cluster_b);

    let env = SymbolEnv::new(
        module.clone(),
        SymbolEnvIndexes {
            registrations,
            contributions,
            ..SymbolEnvIndexes::default()
        },
    );
    let mut activations = vec![
        activation(cluster_a, "trigger:A", "fingerprint:A"),
        activation(cluster_b, "trigger:B", "fingerprint:B"),
    ];
    if reverse_activations {
        activations.reverse();
    }

    ClusterEnvFixture {
        database: RegistrationDatabase::from_symbol_env(&env, activations),
        source,
        module,
        cluster_a: CheckerRegistrationId::new(cluster_a.index()),
        cluster_b: CheckerRegistrationId::new(cluster_b.index()),
    }
}

fn activation(id: ResolverRegistrationId, trigger: &str, fingerprint: &str) -> ActivationInput {
    activation_with_kind(id, ResolverRegistrationKind::Cluster, trigger, fingerprint)
}

fn activation_with_kind(
    id: ResolverRegistrationId,
    kind: ResolverRegistrationKind,
    trigger: &str,
    fingerprint: &str,
) -> ActivationInput {
    ActivationInput::accepted(
        id,
        kind,
        trigger,
        format!("pattern:{trigger}"),
        format!("correctness:{trigger}"),
        format!("evidence:{trigger}"),
    )
    .with_fingerprint(RegistrationFingerprint::new(fingerprint))
}

fn contribution(
    contributions: &mut SourceContributionIndex,
    module: ModuleId,
    source: SourceId,
    offset: usize,
) -> SourceContributionId {
    contributions.insert(
        module,
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, offset, offset + 1)),
    )
}

fn insert_registration(
    registrations: &mut RegistrationIndex,
    module: ModuleId,
    source: SourceId,
    contribution: SourceContributionId,
    local: &str,
    offset: u32,
) -> ResolverRegistrationId {
    insert_registration_with_kind(
        registrations,
        module,
        source,
        contribution,
        ResolverRegistrationKind::Cluster,
        local,
        offset,
    )
}

fn insert_registration_with_kind(
    registrations: &mut RegistrationIndex,
    module: ModuleId,
    source: SourceId,
    contribution: SourceContributionId,
    kind: ResolverRegistrationKind,
    local: &str,
    offset: u32,
) -> ResolverRegistrationId {
    registrations.insert(
        Some(symbol(local)),
        kind,
        SignatureShell::Pending,
        SemanticOrigin::new(
            source,
            module,
            SourceAnchor::Range(range(source, offset as usize, offset as usize + 1)),
            vec![offset],
        ),
        contribution,
    )
}

fn cluster_fact(
    fingerprint: &str,
    attribute: &str,
    start: usize,
    source: SourceId,
) -> ClusterFactInput {
    ClusterFactInput::new(
        fingerprint,
        "type:T",
        attribute,
        "type:T",
        range(source, start, start + 1),
    )
}

fn cluster_rule(
    registration: CheckerRegistrationId,
    trigger: &str,
    antecedent: &str,
    generated: &str,
    start: usize,
    source: SourceId,
) -> ClusterRuleInput {
    ClusterRuleInput::new(ClusterRuleDraft {
        registration,
        kind: ClusterRuleKind::Conditional,
        trigger: RegistrationTriggerKey::new(trigger),
        source_type: ClusterTypeFingerprint::new("type:T"),
        generated_attribute: ClusterAttributeFingerprint::new(format!("attr:{generated}")),
        generated_type: ClusterTypeFingerprint::new("type:T"),
        generated_fact: ClusterFactFingerprint::new(generated),
        rule_fingerprint: ClusterRuleFingerprint::new(trigger.replacen(
            "trigger",
            "fingerprint",
            1,
        )),
        source_range: range(source, start, start + 1),
    })
    .with_antecedents([ClusterFactFingerprint::new(antecedent)])
}

fn generated_cluster_facts(trace: &ResolutionTrace) -> Vec<String> {
    trace
        .steps()
        .iter()
        .filter_map(|step| match step {
            ResolutionTraceStep::Cluster(step) => Some(step.generated_fact().as_str().to_owned()),
            ResolutionTraceStep::Reduction(_) => None,
        })
        .collect()
}

fn cluster_fact_fingerprints(facts: &ClusterFactTable) -> Vec<String> {
    facts
        .canonical_iter()
        .map(|(_, fact)| fact.fingerprint().as_str().to_owned())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OverloadResolvedSnapshot {
    collection: String,
    expansion: String,
    viability: String,
    specificity: String,
    selection: String,
    resolved: String,
    collection_candidates: Vec<String>,
    expanded_candidates: Vec<String>,
    inserted_view_count: usize,
}

fn overload_resolved_snapshot(reverse: bool) -> OverloadResolvedSnapshot {
    let source = source_id(12);
    let expr_site = TypedSiteRef::Node(TypedNodeId::new(0));
    let role_site = TypedSiteRef::Role {
        node: TypedNodeId::new(0),
        role: TypeRole::new("arg0"),
    };
    let typed_ast = typed_ast_fixture(source);
    let mut cluster_facts = ClusterFactTable::new();
    let input_fact = cluster_facts.insert(cluster_fact_draft("fact:A", "attr:A", source, 70));
    let trace_fact = cluster_facts.insert(ClusterFactDraft {
        provenance: ClusterFactProvenance::TraceStep(ClusterStepId::new(0)),
        ..cluster_fact_draft("fact:B", "attr:B", source, 71)
    });

    let mut sites = vec![
        overload_site("call", expr_site.clone(), source, 10),
        overload_site("call", expr_site.clone(), source, 40),
    ];
    let mut candidates = vec![
        overload_candidate(
            "call",
            "root",
            "ordinary",
            0,
            Some(NormalizedTypeId::new(10)),
        ),
        redefinition_candidate("call", "root", "ordinary", "refinement", 1),
        template_candidate("call", "other-root", "templated-other", 2),
        overload_candidate(
            "missing-site",
            "missing-root",
            "missing",
            3,
            Some(NormalizedTypeId::new(30)),
        ),
    ];
    if reverse {
        sites.reverse();
        candidates.reverse();
    }

    let collection = OverloadCollectionOutput::collect(sites, candidates);
    let expansion = TemplateExpansionOutput::expand(&collection);
    let ordinary = candidate_id_by_symbol(expansion.candidates(), "ordinary");
    let refinement = candidate_id_by_symbol(expansion.candidates(), "refinement");
    let templated_other = candidate_id_by_symbol(expansion.candidates(), "templated-other");
    let mut viability_inputs = vec![
        viable_input(ordinary),
        viable_input(refinement),
        viable_input(templated_other),
    ];
    if reverse {
        viability_inputs.reverse();
    }
    let viability = CandidateViabilityOutput::filter(&expansion, viability_inputs);
    let ordinary = candidate_id_by_symbol(viability.candidates(), "ordinary");
    let refinement = candidate_id_by_symbol(viability.candidates(), "refinement");
    let templated_other = candidate_id_by_symbol(viability.candidates(), "templated-other");
    let mut comparisons = vec![
        SpecificityComparisonInput {
            left: refinement,
            right: ordinary,
            status: SpecificityComparisonStatus::Equivalent,
            reasons: vec![SpecificityReasonKey::new("same-root-refinement")],
        },
        SpecificityComparisonInput {
            left: refinement,
            right: templated_other,
            status: SpecificityComparisonStatus::LeftAtLeastRight,
            reasons: vec![SpecificityReasonKey::new("refinement-beats-template")],
        },
        SpecificityComparisonInput {
            left: ordinary,
            right: templated_other,
            status: SpecificityComparisonStatus::LeftAtLeastRight,
            reasons: vec![SpecificityReasonKey::new("ordinary-beats-template")],
        },
    ];
    if reverse {
        comparisons.reverse();
    }
    let specificity = SpecificityGraphOutput::build(&viability, comparisons);
    let ordinary = candidate_id_by_symbol(specificity.candidates(), "ordinary");
    let refinement = candidate_id_by_symbol(specificity.candidates(), "refinement");
    let site = site_id_by_candidate_symbol(specificity.candidates(), "ordinary");
    let mut inserted_views = vec![
        inserted_view(
            role_site.clone(),
            NormalizedTypeId::new(40),
            ordinary,
            InsertedViewKind::Widening,
            "argument-widening",
        ),
        inserted_view(
            expr_site.clone(),
            NormalizedTypeId::new(41),
            refinement,
            InsertedViewKind::SourceQua,
            "source-qua-view",
        ),
    ];
    if reverse {
        inserted_views.reverse();
    }
    let selection = OverloadSelectionOutput::resolve(
        &specificity,
        [OverloadSiteResolutionInput {
            site,
            refinements: vec![refinement],
            refinement_join: RefinementJoinPayload {
                status: RefinementJoinStatus::Compatible,
                exposed_result: Some(ExposedResultPayload {
                    result: Some(NormalizedTypeId::new(50)),
                    source: ExposedResultSource::StrongestRefinement,
                    evidence: vec![TypeFactId::new(0)],
                }),
            },
            inserted_views,
        }],
    );

    let mut expressions = vec![
        ExpressionMetadataInput {
            expr: ExprId::new("expr.call"),
            typed_site: expr_site,
            local_context: None,
            cluster_facts: if reverse {
                vec![trace_fact, input_fact, input_fact]
            } else {
                vec![input_fact, trace_fact, input_fact]
            },
        },
        ExpressionMetadataInput {
            expr: ExprId::new("expr.arg0"),
            typed_site: role_site,
            local_context: None,
            cluster_facts: Vec::new(),
        },
    ];
    if reverse {
        expressions.reverse();
    }
    let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
        typed_ast: &typed_ast,
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
    .expect("deterministic resolved typed AST assembly succeeds");

    OverloadResolvedSnapshot {
        collection: collection.debug_text(),
        expansion: expansion.debug_text(),
        viability: viability.debug_text(),
        specificity: specificity.debug_text(),
        selection: selection.debug_text(),
        resolved: resolved.debug_text(),
        collection_candidates: candidate_symbols(collection.candidates()),
        expanded_candidates: candidate_symbols(expansion.candidates()),
        inserted_view_count: selection.inserted_views().len(),
    }
}

fn typed_ast_fixture(source: SourceId) -> TypedAst {
    let expr_node = TypedNodeId::new(0);
    let root_node = TypedNodeId::new(1);
    let context = LocalTypeContextId::new(0);
    let type_entry = crate::typed_ast::TypeEntryId::new(0);
    let fact = TypeFactId::new(0);
    let mut builder = TypedArenaBuilder::new();
    builder
        .push(
            TypedNode::new(
                "FunctorApplication",
                SourceAnchor::Range(range(source, 10, 20)),
            )
            .with_typing(TypingState::Successful)
            .with_links(TypedNodeLinks {
                context: Some(context),
                type_entry: Some(type_entry),
                facts: vec![fact],
                ..TypedNodeLinks::default()
            }),
        )
        .expect("expression typed node is valid");
    builder
        .push(
            TypedNode::new("CompilationUnit", SourceAnchor::Range(range(source, 0, 30)))
                .with_children(vec![expr_node])
                .with_typing(TypingState::Successful),
        )
        .expect("root typed node is valid");
    let nodes = builder
        .finish(Some(root_node))
        .expect("typed arena is valid");

    let mut contexts = LocalTypeContextTable::new();
    contexts.insert(LocalTypeContextDraft {
        owner: TypedSiteRef::Node(expr_node),
        parent: None,
        layer: TypeContextLayer::Expression,
        bindings: Vec::new(),
        introduced_assumptions: Vec::new(),
        visible_facts: vec![fact],
        recovery: ContextRecoveryState::Normal,
    });
    let mut types = TypeTable::new();
    types.insert(TypeEntryDraft {
        owner: TypedSiteRef::Node(expr_node),
        expected: None,
        actual: TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
        status: TypeStatus::Known,
        provenance: TypeProvenance::Builtin(BuiltinRuleId::new("fixture-type")),
    });
    let mut facts = TypeFactTable::new();
    facts.insert(TypeFactDraft {
        subject: TypedSiteRef::Node(expr_node),
        predicate: TypePredicateRef::new("registered"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Builtin(BuiltinRuleId::new("fixture-fact")),
        status: FactStatus::Known,
    });

    TypedAst::try_new(TypedAstParts {
        source_id: source,
        module_id: module(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        nodes,
        contexts,
        types,
        facts,
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .expect("typed AST fixture is valid")
}

fn overload_site(
    key: &str,
    owner: TypedSiteRef,
    source: SourceId,
    start: usize,
) -> OverloadSiteInput {
    OverloadSiteInput {
        key: OverloadSiteKey::new(key),
        owner,
        source_range: range(source, start, start + 5),
        kind: OverloadSiteKind::FunctorApplication,
        name: OverloadNameKey::new(key),
        arguments: vec![TypedSiteRef::Role {
            node: TypedNodeId::new(0),
            role: TypeRole::new("arg0"),
        }],
        expected: Some(NormalizedTypeId::new(50)),
        source_qua: Vec::<SourceQuaView>::new(),
        recovery: OverloadSiteRecovery::Normal,
    }
}

fn overload_candidate(
    site: &str,
    root: &str,
    name: &str,
    declaration_order: usize,
    result: Option<NormalizedTypeId>,
) -> OverloadCandidateInput {
    OverloadCandidateInput {
        site: OverloadSiteKey::new(site),
        symbol: symbol(name),
        ordinary_root: symbol(root),
        declaration_kind: CandidateDeclarationKind::Functor,
        parameters: vec![NormalizedTypeId::new(1)],
        result,
        origin: CandidateOrigin::Ordinary,
        template: None,
        coherence: None,
        provenance: CandidateProvenance {
            stable_key: CandidateProvenanceKey::new(format!("{name}-provenance")),
            source_range: Some(range(
                source_id(13),
                declaration_order,
                declaration_order + 1,
            )),
            scope: CandidateScope::Local,
            declaration_order,
        },
    }
}

fn redefinition_candidate(
    site: &str,
    root: &str,
    refined: &str,
    name: &str,
    declaration_order: usize,
) -> OverloadCandidateInput {
    let mut candidate = overload_candidate(
        site,
        root,
        name,
        declaration_order,
        Some(NormalizedTypeId::new(20)),
    );
    candidate.declaration_kind = CandidateDeclarationKind::Redefinition;
    candidate.origin = CandidateOrigin::Redefinition {
        refined: symbol(refined),
    };
    candidate.coherence = Some(CoherenceStatus::Accepted);
    candidate
}

fn template_candidate(
    site: &str,
    root: &str,
    name: &str,
    declaration_order: usize,
) -> OverloadCandidateInput {
    let mut candidate = overload_candidate(
        site,
        root,
        name,
        declaration_order,
        Some(NormalizedTypeId::new(30)),
    );
    candidate.template = Some(TemplateCandidatePayload {
        template: symbol("template-source"),
        instantiation_key: TemplateInstantiationKey::new("T=object"),
        parameters: vec![TemplateParameterKey::new("T")],
        arguments: vec![TemplateArgument::Explicit(NormalizedTypeId::new(1))],
        inferred_arguments: Vec::new(),
        constraints: vec![TemplateConstraintEvidence {
            parameter: TemplateParameterKey::new("T"),
            evidence_key: CandidateProvenanceKey::new("template-constraint"),
            facts: vec![TypeFactId::new(0)],
            status: TemplateConstraintEvidenceStatus::Accepted,
        }],
    });
    candidate
}

fn viable_input(candidate: OverloadCandidateId) -> CandidateViabilityInput {
    CandidateViabilityInput {
        candidate,
        arguments: vec![ArgumentViabilityEvidence::Exact {
            actual: NormalizedTypeId::new(1),
        }],
    }
}

fn inserted_view(
    argument: TypedSiteRef,
    target: NormalizedTypeId,
    selected_candidate: OverloadCandidateId,
    kind: InsertedViewKind,
    reason: &str,
) -> InsertedViewInput {
    InsertedViewInput {
        argument,
        target,
        selected_candidate,
        kind,
        status: InsertedViewStatus::Accepted,
        reason: InsertedViewReasonKey::new(reason),
        evidence_facts: vec![TypeFactId::new(0)],
        path: Some(QuaPathKey::new(format!("{reason}-path"))),
    }
}

fn candidate_id_by_symbol(candidates: &OverloadCandidateTable, name: &str) -> OverloadCandidateId {
    candidates
        .iter()
        .find_map(|(id, candidate)| (candidate.symbol.local().as_str() == name).then_some(id))
        .expect("candidate symbol exists")
}

fn site_id_by_candidate_symbol(candidates: &OverloadCandidateTable, name: &str) -> OverloadSiteId {
    candidates
        .iter()
        .find_map(|(_, candidate)| {
            (candidate.symbol.local().as_str() == name).then_some(candidate.site)
        })
        .expect("candidate site exists")
}

fn candidate_symbols(candidates: &OverloadCandidateTable) -> Vec<String> {
    candidates
        .canonical_iter()
        .map(|(_, candidate)| candidate.symbol.local().as_str().to_owned())
        .collect()
}

fn cluster_fact_draft(
    fingerprint: &str,
    attribute: &str,
    source: SourceId,
    start: usize,
) -> ClusterFactDraft {
    ClusterFactDraft {
        fingerprint: ClusterFactFingerprint::new(fingerprint),
        source_type: ClusterTypeFingerprint::new("type:T"),
        attribute: ClusterAttributeFingerprint::new(attribute),
        generated_type: ClusterTypeFingerprint::new("type:T"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(source, start, start + 1),
    }
}

fn symbol(name: &str) -> SymbolId {
    SymbolId::new(
        module(),
        LocalSymbolId::new(name),
        FullyQualifiedName::new(format!("pkg::main::{name}")),
    )
}

fn module() -> ModuleId {
    ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
}

fn range(source: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id: source,
        start,
        end,
    }
}

fn source_id(seed: u8) -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{seed:064x}"
    ))
    .expect("valid build snapshot id");
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id allocation succeeds")
}
