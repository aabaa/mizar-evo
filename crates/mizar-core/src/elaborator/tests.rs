use super::*;
use crate::core_ir::{
    CoreAlgorithmStmtTable, CoreAlgorithmTable, CoreDefinitionTable, CoreIr, CoreIrParts,
    CoreProofNodeTable, CoreProofTable,
};
use mizar_checker::{
    binding_env::{
        BindingContextDraft, BindingContextTable, BindingDiagnosticClass, BindingDiagnosticDraft,
        BindingDiagnosticRecovery, BindingDiagnosticSeverity, BindingDiagnosticTable, BindingDraft,
        BindingEnvParts, BindingTable, BindingTypeSite, CapturedFreeVariables,
    },
    typed_ast::TypeRole,
};
use mizar_resolve::names::LocalTermScope;
use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
};

fn source_id_for(hex_pair: &str) -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        hex_pair.repeat(32)
    ))
    .expect("valid snapshot id");
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id")
}

fn source_id() -> SourceId {
    source_id_for("08")
}

fn alternate_source_id() -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "09".repeat(32)
    ))
    .expect("valid alternate snapshot id");
    let allocator = InMemorySessionIdAllocator::new();
    let _discarded = allocator
        .next_source_id(snapshot)
        .expect("first alternate source id");
    allocator
        .next_source_id(snapshot)
        .expect("second alternate source id")
}

fn range(start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id: source_id(),
        start,
        end,
    }
}

fn direct(start: usize, end: usize) -> CoreSourceRef {
    CoreSourceRef::direct(range(start, end))
}

fn module_id() -> ModuleId {
    ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
}

fn external_module_id() -> ModuleId {
    ModuleId::new(PackageId::new("pkg"), ModulePath::new("dep"))
}

fn symbol(name: &str) -> SymbolId {
    SymbolId::new(
        module_id(),
        LocalSymbolId::new(name),
        FullyQualifiedName::new(format!("pkg::main::{name}")),
    )
}

fn external_symbol(name: &str) -> SymbolId {
    SymbolId::new(
        external_module_id(),
        LocalSymbolId::new(name),
        FullyQualifiedName::new(format!("pkg::dep::{name}")),
    )
}

fn provenance(key: &str) -> CheckerOwnedProvenance {
    CheckerOwnedProvenance::checker(key)
}

fn summary() -> ResolvedTypedAstSummary {
    ResolvedTypedAstSummary::new(source_id(), module_id())
}

fn source_binding_env_with_one(
    source_id: SourceId,
    module_id: ModuleId,
    kind: BindingKind,
    identity: BinderIdentity,
    status: BindingStatus,
) -> BindingEnv {
    let declaration_range = SourceRange {
        source_id,
        start: 10,
        end: 11,
    };
    let mut bindings = BindingTable::new();
    let binding = bindings.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind,
        identity,
        owner_context: BindingContextId::new(0),
        declaration_range,
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(SourceRange {
            source_id,
            start: 12,
            end: 15,
        }),
        status,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
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
    BindingEnv::try_new(BindingEnvParts {
        source_id,
        module_id,
        contexts,
        bindings,
        diagnostics: mizar_checker::binding_env::BindingDiagnosticTable::new(),
    })
    .expect("valid test binding environment")
}

fn source_binding_reserved_draft() -> BindingDraft {
    let declaration_range = range(10, 11);
    BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range,
        },
        owner_context: BindingContextId::new(0),
        declaration_range,
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(range(12, 15)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    }
}

fn source_binding_second_reserved_draft() -> BindingDraft {
    let declaration_range = range(20, 21);
    BindingDraft {
        spelling: "y".to_owned(),
        kind: BindingKind::ReservedVariable,
        identity: BinderIdentity::ReservedVariable {
            spelling: "y".to_owned(),
            declaration_range,
        },
        owner_context: BindingContextId::new(0),
        declaration_range,
        visible_after_ordinal: 1,
        type_site: BindingTypeSite::Source(range(22, 25)),
        status: BindingStatus::Reserved,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    }
}

fn source_binding_env_with_options(
    module_recovery: BindingContextRecovery,
    first: BindingDraft,
    second: BindingDraft,
    diagnostics: BindingDiagnosticTable,
) -> BindingEnv {
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: vec![BindingId::new(0), BindingId::new(1)],
        visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
        recovery: module_recovery,
    });
    let mut bindings = BindingTable::new();
    bindings.insert(first);
    bindings.insert(second);
    BindingEnv::try_new(BindingEnvParts {
        source_id: source_id(),
        module_id: module_id(),
        contexts,
        bindings,
        diagnostics,
    })
    .expect("valid test binding environment")
}

fn source_binding_env() -> BindingEnv {
    source_binding_env_with_options(
        BindingContextRecovery::Normal,
        source_binding_reserved_draft(),
        source_binding_second_reserved_draft(),
        BindingDiagnosticTable::new(),
    )
}

fn empty_source_binding_env() -> BindingEnv {
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
    BindingEnv::try_new(BindingEnvParts {
        source_id: source_id(),
        module_id: module_id(),
        contexts,
        bindings: BindingTable::new(),
        diagnostics: BindingDiagnosticTable::new(),
    })
    .expect("valid empty binding environment")
}

fn input_with_items(item_seeds: Vec<CoreItemSeed>) -> CoreContextInput {
    let mut input = CoreContextInput::new(summary());
    input.item_seeds = item_seeds;
    input
}

fn item_seed(name: &str, start: usize) -> CoreItemSeed {
    CoreItemSeed::new(
        symbol(name),
        CoreItemKind::Theorem,
        "public",
        direct(start, start + 3),
        provenance(format!("checker:item:{name}").as_str()),
    )
    .with_definition_boundary(DefinitionBoundaryKind::Theorem)
}

fn exact_task180_draft() -> ExactTask180Draft {
    let owner_range = range(10, 90);
    ExactTask180Draft {
        source_id: source_id(),
        module_id: module_id(),
        owner: symbol("SourceDerivedContradictionConstantBoundary"),
        owner_range,
        owner_origin: SemanticOrigin::new(
            source_id(),
            module_id(),
            SourceAnchor::Range(owner_range),
            Vec::new(),
        ),
        owner_node: TypedNodeId::new(1),
        formula_range: range(50, 63),
        formula_site_node: TypedNodeId::new(42),
        formula_node: TypedNodeId::new(0),
    }
}

#[test]
fn exact_task267_encoders_are_byte_canonical_and_keep_node_roles_distinct() {
    let mut draft = exact_task180_draft();
    draft.owner = SymbolId::new(
        module_id(),
        LocalSymbolId::new("unicode"),
        FullyQualifiedName::new("pkg::main::証明"),
    );
    assert_eq!(
        exact_task267_resolver_key(&draft.owner, &draft.owner_origin),
        "task267/v1;owner-fqn=17:pkg::main::証明;origin-path=0:"
    );

    draft.owner_origin = SemanticOrigin::new(
        source_id(),
        module_id(),
        SourceAnchor::Range(draft.owner_range),
        vec![0, 12],
    );
    assert_eq!(
        exact_task267_resolver_key(&draft.owner, &draft.owner_origin),
        "task267/v1;owner-fqn=17:pkg::main::証明;origin-path=2:0,12"
    );

    draft.owner_node = TypedNodeId::new(12);
    draft.formula_site_node = TypedNodeId::new(34);
    draft.formula_node = TypedNodeId::new(5);
    assert_eq!(
        exact_task267_statement_key(&draft),
        "task267/v1;statement=0;owner-node=12;formula=0;formula-site-node=34;formula-node=5"
    );
    assert_eq!(
        exact_task267_terminal_key(&draft),
        "task267/v1;proof-node=0;terminal-goal=0;formula=0;formula-site-node=34;formula-node=5"
    );
    assert_eq!(
        exact_task180_formula_site_node(&TypedSiteRef::Node(TypedNodeId::new(34)))
            .expect("node sites are exact inputs"),
        TypedNodeId::new(34)
    );
    assert!(matches!(
        exact_task180_formula_site_node(&TypedSiteRef::Role {
            node: TypedNodeId::new(34),
            role: TypeRole::new("result"),
        }),
        Err(ExactTask180LoweringError::InvalidCheckerBundle { .. })
    ));
}

#[test]
fn exact_task180_draft_lowers_atomically_and_deterministically() {
    let draft = exact_task180_draft();
    let first = lower_exact_task180_draft(draft.clone(), ExactTask180FailureInjection::None)
        .expect("exact draft lowers");
    let second = lower_exact_task180_draft(draft, ExactTask180FailureInjection::None)
        .expect("equivalent exact draft lowers");

    assert_eq!(first, second);
    assert_eq!(first.debug_text(), second.debug_text());
    assert!(!first.has_error_nodes());
    assert_eq!(
        first
            .proofs()
            .get(CoreProofId::new(0))
            .expect("proof zero")
            .status,
        CoreProofStatus::PendingAutomaticProof
    );
}

#[test]
fn exact_task180_failure_injections_return_only_typed_errors() {
    let draft = exact_task180_draft();
    assert!(matches!(
        lower_exact_task180_draft(draft.clone(), ExactTask180FailureInjection::Preflight),
        Err(ExactTask180LoweringError::InvalidCheckerBundle { .. })
    ));
    assert!(matches!(
        lower_exact_task180_draft(draft.clone(), ExactTask180FailureInjection::GenericLowering),
        Err(ExactTask180LoweringError::GenericLowering { .. })
    ));
    assert!(matches!(
        lower_exact_task180_draft(
            draft.clone(),
            ExactTask180FailureInjection::ProvenanceEnrichment
        ),
        Err(ExactTask180LoweringError::ProvenanceEnrichment { .. })
    ));
    assert!(matches!(
        lower_exact_task180_draft(draft, ExactTask180FailureInjection::InvalidProjection),
        Err(ExactTask180LoweringError::InvalidProjection { .. })
    ));
}

fn exact_task180_core_parts(core: &CoreIr) -> CoreIrParts {
    CoreIrParts {
        source_id: core.source_id(),
        module_id: core.module_id().clone(),
        items: core.items().clone(),
        terms: core.terms().clone(),
        formulas: core.formulas().clone(),
        definitions: core.definitions().clone(),
        proofs: core.proofs().clone(),
        proof_nodes: core.proof_nodes().clone(),
        algorithms: core.algorithms().clone(),
        algorithm_statements: core.algorithm_statements().clone(),
        generated: core.generated().clone(),
        obligation_seeds: core.obligation_seeds().clone(),
        source_map: core.source_map().clone(),
        diagnostics: core.diagnostics().clone(),
    }
}

type ExactTask180ProjectionMutation = fn(&mut CoreIrParts);

fn assert_exact_task180_projection_rejects(
    case: &str,
    core: &CoreIr,
    draft: &ExactTask180Draft,
    mutate: ExactTask180ProjectionMutation,
) {
    let mut parts = exact_task180_core_parts(core);
    mutate(&mut parts);
    let mutated = CoreIr::try_new(parts)
        .unwrap_or_else(|error| panic!("{case}: near-miss must remain generic CoreIr: {error}"));
    let resolver_key = exact_task267_resolver_key(&draft.owner, &draft.owner_origin);
    let statement_key = exact_task267_statement_key(draft);
    let proof_key = "task267/v1;proof=0;statement=0;policy=unmodified;justification=omitted;status=pending-automatic-proof";
    let terminal_key = exact_task267_terminal_key(draft);
    assert!(
        matches!(
            validate_exact_task180_projection(
                &mutated,
                draft,
                ExactTask267Keys {
                    resolver: &resolver_key,
                    statement: &statement_key,
                    proof: proof_key,
                    terminal: &terminal_key,
                    skeleton: "task267/v1;local-path=7:proof/0",
                },
            ),
            Err(ExactTask180LoweringError::InvalidProjection { .. })
        ),
        "{case}: exact projection validation must fail closed",
    );
}

#[test]
fn exact_task180_postvalidator_rejects_structurally_valid_near_misses() {
    let draft = exact_task180_draft();
    let core = lower_exact_task180_draft(draft.clone(), ExactTask180FailureInjection::None)
        .expect("exact draft lowers");
    let cases: &[(&str, ExactTask180ProjectionMutation)] = &[
        ("item visibility", |parts| {
            parts
                .items
                .get_mut(CoreItemId::new(0))
                .expect("item zero")
                .visibility = CoreVisibility::new("private");
        }),
        ("formula kind", |parts| {
            parts
                .formulas
                .get_mut(CoreFormulaId::new(0))
                .expect("formula zero")
                .kind = CoreFormulaKind::True;
        }),
        ("formula source", |parts| {
            let source = CoreSourceRef::direct(SourceRange {
                source_id: parts.source_id,
                start: 51,
                end: 62,
            });
            parts
                .formulas
                .get_mut(CoreFormulaId::new(0))
                .expect("formula zero")
                .source = source.clone();
            parts
                .source_map
                .formula_sources
                .insert(CoreFormulaId::new(0), source);
        }),
        ("proof source", |parts| {
            parts
                .proofs
                .get_mut(CoreProofId::new(0))
                .expect("proof zero")
                .source = CoreSourceRef::direct(SourceRange {
                source_id: parts.source_id,
                start: 11,
                end: 89,
            });
        }),
        ("proof status", |parts| {
            parts
                .proofs
                .get_mut(CoreProofId::new(0))
                .expect("proof zero")
                .status = CoreProofStatus::Open;
        }),
        ("proof-node kind", |parts| {
            parts
                .proof_nodes
                .get_mut(CoreProofNodeId::new(0))
                .expect("proof node zero")
                .kind = CoreProofNodeKind::Sequence {
                children: Vec::new(),
            };
        }),
        ("terminal citations", |parts| {
            let CoreProofNodeKind::TerminalGoal { citations, .. } = &mut parts
                .proof_nodes
                .get_mut(CoreProofNodeId::new(0))
                .expect("proof node zero")
                .kind
            else {
                panic!("exact proof node must be terminal");
            };
            citations.push(CoreCitation::Symbol(external_symbol("ExternalCitation")));
        }),
        ("terminal source", |parts| {
            let source = CoreSourceRef::direct(SourceRange {
                source_id: parts.source_id,
                start: 52,
                end: 61,
            });
            parts
                .proof_nodes
                .get_mut(CoreProofNodeId::new(0))
                .expect("proof node zero")
                .source = source.clone();
            parts
                .source_map
                .proof_sources
                .insert(CoreProofNodeId::new(0), source);
        }),
        ("obligation status", |parts| {
            parts
                .obligation_seeds
                .get_mut(ObligationSeedId::new(0))
                .expect("obligation zero")
                .status = ObligationSeedStatus::Deferred;
        }),
        ("obligation path", |parts| {
            parts
                .obligation_seeds
                .get_mut(ObligationSeedId::new(0))
                .expect("obligation zero")
                .local_path = LocalProofOrProgramPath::new("proof/1");
        }),
        ("obligation provenance", |parts| {
            parts
                .obligation_seeds
                .get_mut(ObligationSeedId::new(0))
                .expect("obligation zero")
                .provenance = vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "task267/v1;unexpected-obligation-provenance",
            )];
        }),
        ("obligation core refs", |parts| {
            parts
                .obligation_seeds
                .get_mut(ObligationSeedId::new(0))
                .expect("obligation zero")
                .core_refs
                .retain(|core_ref| !matches!(core_ref, CoreNodeRef::Formula(_)));
        }),
        ("obligation source-map ownership", |parts| {
            let source = CoreSourceRef::direct(SourceRange {
                source_id: parts.source_id,
                start: 53,
                end: 60,
            });
            parts
                .obligation_seeds
                .get_mut(ObligationSeedId::new(0))
                .expect("obligation zero")
                .source = source.clone();
            parts
                .source_map
                .obligation_sources
                .insert(ObligationSeedId::new(0), source);
        }),
        ("item source-map ownership", |parts| {
            let source = parts
                .formulas
                .get(CoreFormulaId::new(0))
                .expect("formula zero")
                .source
                .clone();
            parts
                .items
                .get_mut(CoreItemId::new(0))
                .expect("item zero")
                .source = source.clone();
            parts
                .source_map
                .item_sources
                .insert(CoreItemId::new(0), source);
        }),
    ];

    for (case, mutate) in cases {
        assert_exact_task180_projection_rejects(case, &core, &draft, *mutate);
    }
}

fn algorithm_item_seed(name: &str, start: usize) -> CoreItemSeed {
    CoreItemSeed::new(
        symbol(name),
        CoreItemKind::Algorithm,
        "public",
        direct(start, start + 3),
        provenance(format!("checker:item:{name}").as_str()),
    )
    .with_definition_boundary(DefinitionBoundaryKind::Algorithm)
}

fn context_with_algorithm_var_sorts(
    vars: Vec<(CoreVarId, NormalizedVarSort)>,
) -> (CoreContext, CoreItemId) {
    let mut input = input_with_items(vec![algorithm_item_seed("Owner", 0)]);
    input.variable_seeds = vars
        .iter()
        .map(|(var, sort)| {
            CoreVariableSeed::new(
                *var,
                NormalizedVarClass::Free,
                "term-binder",
                *sort,
                provenance(format!("checker:algorithm-var:{}", var.index()).as_str()),
            )
        })
        .collect();
    input.binder_seeds = vars
        .iter()
        .map(|(var, _)| {
            CoreBinderSeed::new(
                *var,
                direct(var.index() + 1, var.index() + 2),
                provenance(format!("checker:algorithm-binder:{}", var.index()).as_str()),
            )
        })
        .collect();
    let context = prepare_core_context(input).expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("Owner"))
        .expect("owner id");
    (context, owner)
}

fn context_with_var(var: CoreVarId) -> (CoreContext, CoreItemId) {
    context_with_var_sort(var, NormalizedVarSort::Term)
}

fn context_with_var_sort(var: CoreVarId, sort: NormalizedVarSort) -> (CoreContext, CoreItemId) {
    let mut input = input_with_items(vec![item_seed("Owner", 0)]);
    input.variable_seeds = vec![CoreVariableSeed::new(
        var,
        NormalizedVarClass::Free,
        "term-binder",
        sort,
        provenance("checker:var"),
    )];
    input.binder_seeds = vec![CoreBinderSeed::new(
        var,
        direct(1, 2),
        provenance("checker:binder"),
    )];
    let context = prepare_core_context(input).expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("Owner"))
        .expect("owner id");
    (context, owner)
}

fn context_with_var_sorts(vars: Vec<(CoreVarId, NormalizedVarSort)>) -> (CoreContext, CoreItemId) {
    let mut input = input_with_items(vec![item_seed("Owner", 0)]);
    input.variable_seeds = vars
        .iter()
        .map(|(var, sort)| {
            CoreVariableSeed::new(
                *var,
                NormalizedVarClass::Free,
                "term-binder",
                *sort,
                provenance(format!("checker:var:{}", var.index()).as_str()),
            )
        })
        .collect();
    input.binder_seeds = vars
        .iter()
        .map(|(var, _)| {
            CoreBinderSeed::new(
                *var,
                direct(var.index() + 1, var.index() + 2),
                provenance(format!("checker:binder:{}", var.index()).as_str()),
            )
        })
        .collect();
    let context = prepare_core_context(input).expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("Owner"))
        .expect("owner id");
    (context, owner)
}

fn context_with_existing_choice_origin(
    var: CoreVarId,
    key: GeneratedOriginKey,
) -> (CoreContext, CoreItemId, GeneratedOriginId) {
    let mut input = input_with_items(vec![item_seed("Owner", 0)]);
    input.variable_seeds = vec![CoreVariableSeed::new(
        var,
        NormalizedVarClass::Free,
        "term-binder",
        NormalizedVarSort::Term,
        provenance("checker:choice-var"),
    )];
    input.binder_seeds = vec![CoreBinderSeed::new(
        var,
        direct(1, 2),
        provenance("checker:choice-binder"),
    )];
    input.generated_origin_seeds = vec![
        GeneratedOriginSeed::new(
            symbol("Owner"),
            GeneratedOriginKind::StableChoice,
            key.clone(),
            direct(90, 91),
            provenance("checker:existing-choice"),
        )
        .with_functor(symbol("choice_existing"))
        .with_params(vec![var])
        .with_evidence(vec![CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "checker:existing-choice:evidence",
        )]),
    ];
    let context = prepare_core_context(input).expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("Owner"))
        .expect("owner id");
    let origin = context
        .generated_origins()
        .get_by_key(owner, GeneratedOriginKind::StableChoice, &key)
        .expect("existing generated origin");
    (context, owner, origin)
}

fn type_fact(
    subject: CoreVarId,
    predicate: &str,
    start: usize,
    polarity: Polarity,
) -> TypePredicateSeed {
    TypePredicateSeed::positive(
        subject,
        CoreTypePredicate::new(predicate),
        direct(start, start + 1),
        provenance(format!("checker:fact:{predicate}").as_str()),
    )
    .with_polarity(polarity)
}

fn active_obligation(
    subject: CoreVarId,
    predicate: &str,
    start: usize,
) -> CarriedInitialObligationSeed {
    CarriedInitialObligationSeed::active(
        InitialObligationId::new(start),
        InitialObligationKind::Narrowing,
        ObligationFormulaSeed::positive(subject, predicate, direct(start, start + 1)),
        format!("type-obligation/{start}"),
        format!("pkg::main::Owner.type-obligation.{start}"),
        direct(start, start + 1),
        provenance(format!("checker:obligation:{start}").as_str()),
    )
}

fn term_seed(kind: CoreTermSeedKind, start: usize) -> CoreTermSeed {
    CoreTermSeed::new(
        kind,
        direct(start, start + 1),
        provenance(format!("checker:term:{start}").as_str()),
    )
}

fn reduct_view(path: &str, functors: &[&str]) -> ReductViewSeed {
    ReductViewSeed {
        path: QuaPathKey::new(path),
        functors: functors.iter().copied().map(symbol).collect(),
    }
}

fn template_type_parameter(
    parameter: &str,
    witness: CoreVarId,
    predicate: &str,
    start: usize,
) -> TemplateTypeParameterInhabitationSeed {
    TemplateTypeParameterInhabitationSeed {
        parameter: TemplateParameterKey::new(parameter),
        witness,
        witness_role: CoreVarRole::new("template-type-witness"),
        witness_source_name: Some(format!("{parameter}$inhabitant")),
        predicate: CoreTypePredicate::new(predicate),
        source: direct(start, start + 1),
        provenance: provenance(format!("checker:template-param:{parameter}").as_str()),
    }
}

fn template_type_actual_gate(
    instantiation: &str,
    parameter: &str,
    status: ExistentialGateStatus,
    start: usize,
) -> TemplateTypeActualGateSeed {
    TemplateTypeActualGateSeed {
        instantiation: TemplateInstantiationKey::new(instantiation),
        parameter: TemplateParameterKey::new(parameter),
        actual_type: NormalizedTypeId::new(start),
        gate: Some(ExistentialGateId::new(start)),
        status,
        registration: None,
        base_evidence_kind: None,
        base_evidence_coverage: None,
        facts: Vec::new(),
        diagnostics: Vec::new(),
        source: direct(start, start + 1),
        provenance: provenance(
            format!("checker:template-gate:{instantiation}:{parameter}").as_str(),
        ),
    }
}

fn template_sethood_seed(
    parameter: &str,
    evidence_key: &str,
    source_kind: TemplateTypeParameterSethoodSource,
    status: TemplateTypeParameterSethoodStatus,
    start: usize,
) -> TemplateTypeParameterSethoodSeed {
    TemplateTypeParameterSethoodSeed {
        parameter: TemplateParameterKey::new(parameter),
        evidence_key: TemplateSethoodEvidenceKey::new(evidence_key),
        normalized_type: NormalizedTypeId::new(start),
        source_kind,
        status,
        facts: Vec::new(),
        diagnostics: Vec::new(),
        source: direct(start, start + 1),
        provenance: provenance(format!("checker:template-sethood:{parameter}").as_str()),
    }
}

fn template_fraenkel_sethood(
    parameter: &str,
    evidence_key: &str,
    normalized_type: usize,
    start: usize,
) -> TemplateFraenkelSethoodEvidenceSeed {
    TemplateFraenkelSethoodEvidenceSeed {
        parameter: TemplateParameterKey::new(parameter),
        evidence_key: TemplateSethoodEvidenceKey::new(evidence_key),
        normalized_type: NormalizedTypeId::new(normalized_type),
        source: direct(start, start + 1),
        provenance: provenance(format!("checker:fraenkel-template-sethood:{start}").as_str()),
    }
}

fn template_scheme_actual(
    instantiation: &str,
    parameter: &str,
    parameter_kind: TemplateSchemeParameterKind,
    actual_kind: TemplateSchemeActualKind,
    status: TemplateSchemeActualStatus,
    arity: usize,
    start: usize,
) -> TemplateSchemeActualSeed {
    TemplateSchemeActualSeed {
        instantiation: TemplateInstantiationKey::new(instantiation),
        parameter: TemplateParameterKey::new(parameter),
        parameter_kind,
        actual_kind,
        status,
        expected_arity: arity,
        actual_arity: arity,
        domain_evidence: Vec::new(),
        codomain_evidence: None,
        guard_obligation: None,
        substitution: None,
        checker_diagnostics: Vec::new(),
        source: direct(start, start + 1),
        provenance: provenance(
            format!("checker:scheme-actual:{instantiation}:{parameter}").as_str(),
        ),
    }
}

fn widening_evidence(from: usize, to: usize, facts: &[usize]) -> TemplateWideningEvidenceSeed {
    TemplateWideningEvidenceSeed {
        from_type: NormalizedTypeId::new(from),
        to_type: NormalizedTypeId::new(to),
        status: TemplateWideningEvidenceStatus::Accepted,
        facts: facts.iter().copied().map(TypeFactId::new).collect(),
    }
}

fn skipped_guard_obligation(start: usize) -> CarriedInitialObligationSeed {
    CarriedInitialObligationSeed {
        checker_obligation: Some(InitialObligationId::new(start)),
        checker_kind: InitialObligationKind::Narrowing,
        status: ObligationSeedStatus::Skipped,
        goal: None,
        context: Vec::new(),
        local_path: format!("scheme-actual-guard/{start}").into(),
        semantic_origin: format!("pkg::main::Owner.scheme-actual-guard.{start}").into(),
        source: direct(start, start + 1),
        provenance: provenance(format!("checker:scheme-actual:guard:{start}").as_str()),
    }
}

fn substitution_composition(
    enclosing_parameter: &str,
    start: usize,
) -> TemplateSubstitutionCompositionSeed {
    TemplateSubstitutionCompositionSeed {
        enclosing_parameter: TemplateParameterKey::new(enclosing_parameter),
        source: direct(start, start + 1),
        provenance: provenance(
            format!("checker:scheme-actual:substitution:{enclosing_parameter}:{start}").as_str(),
        ),
    }
}

fn source_qua_explanation(
    path: &str,
    functors: &[&str],
    target_type: usize,
    start: usize,
) -> ViewExplanationSeed {
    ViewExplanationSeed {
        kind: ViewExplanationKind::SourceQua,
        inserted_view: None,
        target_type: Some(NormalizedTypeId::new(target_type)),
        reduct: Some(reduct_view(path, functors)),
        evidence_facts: Vec::new(),
        source: direct(start, start + 1),
        provenance: provenance(format!("checker:reduct-view:{path}").as_str()),
    }
}

fn formula_seed(kind: CoreFormulaSeedKind, start: usize) -> CoreFormulaSeed {
    CoreFormulaSeed::new(
        kind,
        direct(start, start + 1),
        provenance(format!("checker:formula:{start}").as_str()),
    )
}

fn failed_site(message: &str, start: usize) -> FailedSemanticSiteSeed {
    FailedSemanticSiteSeed::error(
        message,
        direct(start, start + 1),
        provenance(format!("checker:failed:{message}").as_str()),
    )
}

fn expected_checker_source(start: usize, end: usize, key: &str) -> CoreSourceRef {
    source_with_provenance(direct(start, end), &provenance(key))
}

fn assert_step2_delta_valid(context: &CoreContext, output: &TypeAndFactLoweringOutput) {
    let mut source_map = output.source_map.clone();
    source_map.item_sources = context.source_map().item_sources.clone();
    let parts = CoreIrParts {
        source_id: context.source_id(),
        module_id: context.module_id().clone(),
        items: context.item_registry().items().clone(),
        terms: output.terms.clone(),
        formulas: output.formulas.clone(),
        definitions: CoreDefinitionTable::new(),
        proofs: CoreProofTable::new(),
        proof_nodes: CoreProofNodeTable::new(),
        algorithms: CoreAlgorithmTable::new(),
        algorithm_statements: CoreAlgorithmStmtTable::new(),
        generated: GeneratedOriginTable::new(),
        obligation_seeds: output.obligation_seeds.clone(),
        source_map,
        diagnostics: output.diagnostics.clone(),
    };
    CoreIr::try_new(parts).expect("step 2 delta validates when merged with context items");
}

fn assert_step3_delta_valid(context: &CoreContext, output: &TermAndFormulaLoweringOutput) {
    let parts = CoreIrParts {
        source_id: context.source_id(),
        module_id: context.module_id().clone(),
        items: context.item_registry().items().clone(),
        terms: output.terms.clone(),
        formulas: output.formulas.clone(),
        definitions: CoreDefinitionTable::new(),
        proofs: CoreProofTable::new(),
        proof_nodes: CoreProofNodeTable::new(),
        algorithms: CoreAlgorithmTable::new(),
        algorithm_statements: CoreAlgorithmStmtTable::new(),
        generated: output.generated.clone(),
        obligation_seeds: output.obligation_seeds.clone(),
        source_map: output.source_map.clone(),
        diagnostics: output.diagnostics.clone(),
    };
    CoreIr::try_new(parts).expect("step 3 delta validates when merged with context items");
}

fn assert_step4_delta_valid(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    output: &DefinitionLoweringOutput,
) {
    let parts = CoreIrParts {
        source_id: context.source_id(),
        module_id: context.module_id().clone(),
        items: context.item_registry().items().clone(),
        terms: term_formula.terms.clone(),
        formulas: term_formula.formulas.clone(),
        definitions: output.definitions.clone(),
        proofs: CoreProofTable::new(),
        proof_nodes: CoreProofNodeTable::new(),
        algorithms: CoreAlgorithmTable::new(),
        algorithm_statements: CoreAlgorithmStmtTable::new(),
        generated: term_formula.generated.clone(),
        obligation_seeds: output.obligation_seeds.clone(),
        source_map: output.source_map.clone(),
        diagnostics: output.diagnostics.clone(),
    };
    CoreIr::try_new(parts).expect("step 4 delta validates when merged with prior lowering");
}

fn assert_step5_delta_valid(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    definitions: &DefinitionLoweringOutput,
    output: &ProofLoweringOutput,
) {
    let parts = CoreIrParts {
        source_id: context.source_id(),
        module_id: context.module_id().clone(),
        items: context.item_registry().items().clone(),
        terms: term_formula.terms.clone(),
        formulas: term_formula.formulas.clone(),
        definitions: definitions.definitions.clone(),
        proofs: output.proofs.clone(),
        proof_nodes: output.proof_nodes.clone(),
        algorithms: CoreAlgorithmTable::new(),
        algorithm_statements: CoreAlgorithmStmtTable::new(),
        generated: term_formula.generated.clone(),
        obligation_seeds: output.obligation_seeds.clone(),
        source_map: output.source_map.clone(),
        diagnostics: output.diagnostics.clone(),
    };
    CoreIr::try_new(parts).expect("step 5 delta validates when merged with prior lowering");
}

fn assert_step6_delta_valid(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    definitions: &DefinitionLoweringOutput,
    proofs: &ProofLoweringOutput,
    output: &AlgorithmLoweringOutput,
) {
    let parts = CoreIrParts {
        source_id: context.source_id(),
        module_id: context.module_id().clone(),
        items: context.item_registry().items().clone(),
        terms: term_formula.terms.clone(),
        formulas: term_formula.formulas.clone(),
        definitions: definitions.definitions.clone(),
        proofs: proofs.proofs.clone(),
        proof_nodes: proofs.proof_nodes.clone(),
        algorithms: output.algorithms.clone(),
        algorithm_statements: output.algorithm_statements.clone(),
        generated: term_formula.generated.clone(),
        obligation_seeds: proofs.obligation_seeds.clone(),
        source_map: output.source_map.clone(),
        diagnostics: output.diagnostics.clone(),
    };
    CoreIr::try_new(parts).expect("step 6 delta validates when merged with prior lowering");
}

fn test_binder(var: CoreVarId, ty_guard: Option<CoreFormulaId>, start: usize) -> CoreBinder {
    CoreBinder {
        var,
        role: CoreVarRole::new("term-binder"),
        ty_guard,
        source_name: Some(format!("v{}", var.index())),
        source: direct(start, start + 1),
    }
}

fn lower_test_terms_and_formulas(
    context: &CoreContext,
    owner: CoreItemId,
    terms: Vec<CoreTermSeed>,
    formulas: Vec<CoreFormulaSeed>,
) -> TermAndFormulaLoweringOutput {
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = terms;
    input.formulas = formulas;
    lower_term_and_formula_inputs(context, input).expect("term/formula lowering")
}

fn empty_definition_output(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
) -> DefinitionLoweringOutput {
    lower_definition_inputs(context, term_formula, DefinitionLoweringInput::new())
        .expect("empty definition lowering")
}

fn empty_proof_output(
    context: &CoreContext,
    term_formula: &TermAndFormulaLoweringOutput,
    definitions: &DefinitionLoweringOutput,
) -> ProofLoweringOutput {
    lower_proof_inputs(
        context,
        term_formula,
        definitions,
        ProofLoweringInput::new(),
    )
    .expect("empty proof lowering")
}

fn definition_seed(
    owner: CoreItemId,
    symbol: SymbolId,
    body: DefinitionBodySeed,
    start: usize,
) -> DefinitionSeed {
    DefinitionSeed {
        owner,
        symbol,
        params: Vec::new(),
        body,
        expansion: ExpansionPolicy::Opaque,
        correctness: Vec::new(),
        generated_dependencies: Vec::new(),
        source: direct(start, start + 1),
        provenance: provenance(format!("checker:definition:{start}").as_str()),
    }
}

fn algorithm_seed(
    owner: CoreItemId,
    symbol: SymbolId,
    payload: AlgorithmPayloadSeed,
    start: usize,
) -> AlgorithmSeed {
    AlgorithmSeed {
        owner,
        symbol,
        params: Vec::new(),
        result: None,
        contracts: CoreContractSet::default(),
        payload,
        ghost_effects: Vec::new(),
        source: direct(start, start + 1),
        provenance: provenance(format!("checker:algorithm:{start}").as_str()),
    }
}

fn proof_seed(
    owner: CoreItemId,
    symbol: SymbolId,
    proposition: CoreFormulaId,
    status: CoreProofStatus,
    skeleton: ProofSkeletonSeed,
    start: usize,
) -> ProofSeed {
    ProofSeed {
        owner,
        symbol,
        proposition,
        status,
        skeleton,
        source: direct(start, start + 1),
        provenance: provenance(format!("checker:proof:{start}").as_str()),
    }
}

fn malformed_proof(message: &str, start: usize) -> MalformedProofSkeletonSeed {
    MalformedProofSkeletonSeed::error(
        message,
        direct(start, start + 1),
        provenance(format!("checker:proof:malformed:{message}").as_str()),
    )
}

fn proof_step_node(start: usize) -> ProofNodeSeed {
    ProofNodeSeed::Step {
        label: None,
        formula: ProofFormulaRef::Thesis,
        justification: ProofJustificationSeed::new(
            Vec::new(),
            direct(start, start + 1),
            provenance(format!("checker:proof:step:{start}:justification").as_str()),
        ),
        source: direct(start + 1, start + 2),
        provenance: provenance(format!("checker:proof:step:{start}").as_str()),
    }
}

fn proof_step_skeleton(start: usize) -> ProofSkeletonSeed {
    ProofSkeletonSeed::Node(proof_step_node(start))
}

fn assert_type_predicate(
    output: &TypeAndFactLoweringOutput,
    formula: CoreFormulaId,
    expected_var: CoreVarId,
    expected_predicate: &str,
) {
    let CoreFormulaKind::TypePred { subject, ty } =
        &output.formulas.get(formula).expect("type predicate").kind
    else {
        panic!("expected TypePred");
    };
    assert_eq!(ty.as_str(), expected_predicate);
    assert!(matches!(
        output.terms.get(*subject).expect("subject term").kind,
        CoreTermKind::Var(var) if var == expected_var
    ));
}

#[test]
fn declared_binder_type_lowers_to_guard_and_assumption() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.declared_binders = vec![
        DeclaredBinderTypeSeed::new(
            var,
            "term-binder",
            "Nat",
            direct(2, 5),
            provenance("checker:declared-type"),
        )
        .with_source_name("x"),
    ];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    let guard = output.binder_guards[0].binder.ty_guard.expect("guard");

    assert_eq!(output.assumptions, vec![guard]);
    assert_eq!(output.binder_guards[0].assumption, guard);
    assert_eq!(
        output.binder_guards[0].binder.source_name.as_deref(),
        Some("x")
    );
    assert_type_predicate(&output, guard, var, "Nat");
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn formula_assertion_lowers_to_type_predicate_formula() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.formula_assertions = vec![type_fact(var, "set", 6, Polarity::Positive)];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    let assertion = output.assertions[0];

    assert_type_predicate(&output, assertion, var, "set");
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn attribute_chains_lower_polarity_and_deterministic_conjunction_order() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.attribute_chains = vec![AttributeChainSeed {
        facts: vec![
            type_fact(var, "Z", 10, Polarity::Positive),
            type_fact(var, "A", 8, Polarity::Negative),
        ],
        source: direct(8, 12),
        provenance: provenance("checker:attribute-chain"),
    }];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    let conjunction = output.attribute_formulas[0];
    let CoreFormulaKind::And(children) = &output.formulas.get(conjunction).expect("and").kind
    else {
        panic!("expected conjunction");
    };
    assert_eq!(children.len(), 2);
    let CoreFormulaKind::Not(negative_atom) =
        output.formulas.get(children[0]).expect("negative").kind
    else {
        panic!("expected negative attribute");
    };
    assert_type_predicate(&output, negative_atom, var, "A");
    assert_type_predicate(&output, children[1], var, "Z");
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn mode_expansion_uses_checker_normalized_type_id() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.mode_expansions = vec![ModeExpansionSeed {
        subject: var,
        normalized_type: NormalizedTypeId::new(42),
        predicate: CoreTypePredicate::new("mode:Element"),
        source: direct(12, 15),
        provenance: provenance("checker:mode-expansion"),
    }];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    let lowered = &output.mode_expansions[0];

    assert_eq!(lowered.normalized_type, NormalizedTypeId::new(42));
    assert_type_predicate(&output, lowered.formula, var, "mode:Element");
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn cluster_facts_lower_without_registration_activation() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.cluster_facts = vec![ClusterFactSeed {
        cluster_fact: ClusterFactId::new(3),
        fact: type_fact(var, "cluster:inhabited", 16, Polarity::Positive)
            .with_checker_fact(TypeFactId::new(5)),
    }];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");

    assert_eq!(output.cluster_facts.len(), 1);
    assert_eq!(output.cluster_facts[0].cluster_fact, ClusterFactId::new(3));
    assert_type_predicate(
        &output,
        output.cluster_facts[0].formula,
        var,
        "cluster:inhabited",
    );
    assert!(output.obligation_seeds.is_empty());
    assert!(output.diagnostics.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn qua_and_inserted_views_record_provenance_without_cast_or_proof_steps() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.view_explanations = vec![
        ViewExplanationSeed {
            kind: ViewExplanationKind::SourceQua,
            inserted_view: None,
            target_type: Some(NormalizedTypeId::new(1)),
            reduct: None,
            evidence_facts: vec![TypeFactId::new(2), TypeFactId::new(1), TypeFactId::new(1)],
            source: direct(20, 23),
            provenance: provenance("checker:view:source-qua"),
        },
        ViewExplanationSeed {
            kind: ViewExplanationKind::InsertedView,
            inserted_view: Some(CoercionInsertionId::new(0)),
            target_type: Some(NormalizedTypeId::new(2)),
            reduct: None,
            evidence_facts: vec![TypeFactId::new(4)],
            source: direct(24, 25),
            provenance: provenance("checker:view:inserted"),
        },
    ];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");

    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert!(output.obligation_seeds.is_empty());
    assert_eq!(
        output.view_explanations[0].kind,
        ViewExplanationKind::SourceQua
    );
    assert_eq!(
        output.view_explanations[0].target_type,
        Some(NormalizedTypeId::new(1))
    );
    assert_eq!(output.view_explanations[0].source, direct(20, 23));
    assert_eq!(
        output.view_explanations[0].provenance,
        vec![CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "checker:view:source-qua"
        )]
    );
    assert_eq!(
        output.view_explanations[0].evidence_facts,
        vec![TypeFactId::new(1), TypeFactId::new(2)]
    );
    assert_eq!(
        output.view_explanations[1].kind,
        ViewExplanationKind::InsertedView
    );
    assert_eq!(
        output.view_explanations[1].inserted_view,
        Some(CoercionInsertionId::new(0))
    );
    assert_eq!(
        output.view_explanations[1].target_type,
        Some(NormalizedTypeId::new(2))
    );
    assert_eq!(
        output.view_explanations[1].evidence_facts,
        vec![TypeFactId::new(4)]
    );
    assert_eq!(output.view_explanations[1].source, direct(24, 25));
    assert_eq!(
        output.view_explanations[1].provenance,
        vec![CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "checker:view:inserted"
        )]
    );
}

#[test]
fn type_fact_lowering_preserves_valid_reduct_view_metadata_without_terms() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.view_explanations = vec![ViewExplanationSeed {
        kind: ViewExplanationKind::SourceQua,
        inserted_view: None,
        target_type: Some(NormalizedTypeId::new(3)),
        reduct: Some(reduct_view("Ring>AddGroup>Magma", &["z_step", "a_step"])),
        evidence_facts: vec![TypeFactId::new(8), TypeFactId::new(8), TypeFactId::new(7)],
        source: direct(26, 27),
        provenance: provenance("checker:view:reduct-metadata"),
    }];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");

    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert_eq!(output.view_explanations.len(), 1);
    let reduct = output.view_explanations[0]
        .reduct
        .as_ref()
        .expect("reduct metadata");
    assert_eq!(reduct.path.as_str(), "Ring>AddGroup>Magma");
    assert_eq!(reduct.functors, vec![symbol("z_step"), symbol("a_step")]);
    assert_eq!(
        output.view_explanations[0].evidence_facts,
        vec![TypeFactId::new(7), TypeFactId::new(8)]
    );
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn reconsidering_carries_checker_obligation_seed() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let obligation = active_obligation(var, "narrowed:Nat", 30);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.reconsiderings = vec![
        ReconsideringSeed::new(
            var,
            "term-binder",
            direct(28, 31),
            provenance("checker:reconsider"),
        )
        .with_predicate("narrowed:Nat")
        .with_obligation(obligation),
    ];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    let reconsidered = &output.reconsidered_binders[0];
    let obligation = reconsidered.obligation.expect("obligation");
    let seed = output
        .obligation_seeds
        .get(obligation)
        .expect("obligation seed");

    assert_eq!(reconsidered.binder.var, var);
    assert_eq!(reconsidered.binder.role, CoreVarRole::new("term-binder"));
    assert_eq!(reconsidered.binder.source, direct(28, 31));
    assert!(reconsidered.binder.ty_guard.is_some());
    assert_type_predicate(
        &output,
        reconsidered.binder.ty_guard.expect("guard"),
        var,
        "narrowed:Nat",
    );
    assert_eq!(seed.status, ObligationSeedStatus::Active);
    assert_eq!(seed.kind, ObligationSeedKind::CheckerInitial);
    assert!(seed.goal.is_some());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn standalone_carried_obligations_populate_output_vector() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.carried_obligations = vec![active_obligation(var, "standalone:goal", 34)];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    let obligation = output.carried_obligations[0];
    let seed = output
        .obligation_seeds
        .get(obligation)
        .expect("obligation seed");

    assert_eq!(output.carried_obligations, vec![obligation]);
    assert_eq!(seed.status, ObligationSeedStatus::Active);
    assert!(seed.goal.is_some());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn missing_evidence_emits_diagnostic_and_deferred_seed_without_proving() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let deferred = CarriedInitialObligationSeed {
        checker_obligation: Some(InitialObligationId::new(99)),
        checker_kind: InitialObligationKind::Sethood,
        status: ObligationSeedStatus::Deferred,
        goal: None,
        context: vec![ObligationFormulaSeed::positive(var, "set", direct(35, 36))],
        local_path: LocalProofOrProgramPath::new("type/missing/sethood"),
        semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Owner.missing-sethood"),
        source: direct(35, 36),
        provenance: provenance("checker:missing:sethood"),
    };
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.missing_evidence = vec![MissingEvidenceSeed {
        kind: MissingEvidenceKind::Sethood,
        diagnostic: Some(TypeDiagnosticId::new(7)),
        deferred_obligation: Some(deferred),
        source: direct(35, 36),
        provenance: provenance("checker:missing-evidence"),
    }];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    let missing = &output.missing_evidence[0];
    let diagnostic = output
        .diagnostics
        .get(missing.diagnostic)
        .expect("diagnostic");
    let obligation = missing.obligation.expect("deferred seed");
    let seed = output
        .obligation_seeds
        .get(obligation)
        .expect("obligation seed");

    assert_eq!(missing.checker_diagnostic, Some(TypeDiagnosticId::new(7)));
    assert!(!missing.provenance.is_empty());
    assert_eq!(diagnostic.message_key.as_str(), "missing-sethood-evidence");
    assert_eq!(seed.status, ObligationSeedStatus::Deferred);
    assert_eq!(seed.kind, ObligationSeedKind::GeneratedSethood);
    assert!(seed.goal.is_none());
    assert_eq!(seed.diagnostics, vec![missing.diagnostic]);
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn missing_evidence_matrix_preserves_each_required_category() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.missing_evidence = vec![
        MissingEvidenceSeed {
            kind: MissingEvidenceKind::NonEmptiness,
            diagnostic: Some(TypeDiagnosticId::new(11)),
            deferred_obligation: None,
            source: direct(40, 41),
            provenance: provenance("checker:missing:non-empty"),
        },
        MissingEvidenceSeed {
            kind: MissingEvidenceKind::Coercion,
            diagnostic: Some(TypeDiagnosticId::new(12)),
            deferred_obligation: None,
            source: direct(42, 43),
            provenance: provenance("checker:missing:coercion"),
        },
        MissingEvidenceSeed {
            kind: MissingEvidenceKind::Cluster,
            diagnostic: Some(TypeDiagnosticId::new(13)),
            deferred_obligation: None,
            source: direct(44, 45),
            provenance: provenance("checker:missing:cluster"),
        },
    ];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    let messages = output
        .missing_evidence
        .iter()
        .map(|missing| {
            output
                .diagnostics
                .get(missing.diagnostic)
                .expect("diagnostic")
                .message_key
                .as_str()
                .to_owned()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "missing-non-emptiness-evidence",
            "missing-coercion-evidence",
            "missing-cluster-evidence"
        ]
    );
    assert_eq!(
        output
            .missing_evidence
            .iter()
            .map(|missing| missing.checker_diagnostic)
            .collect::<Vec<_>>(),
        vec![
            Some(TypeDiagnosticId::new(11)),
            Some(TypeDiagnosticId::new(12)),
            Some(TypeDiagnosticId::new(13))
        ]
    );
    assert!(output.obligation_seeds.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn template_type_parameter_inhabitation_emits_schema_exists_assumption() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let witness = CoreVarId::new(42);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_parameters = vec![template_type_parameter("T", witness, "is_T", 46)];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    assert_eq!(output.template_type_parameter_inhabitations.len(), 1);
    assert_eq!(output.assumptions.len(), 1);
    let lowered = &output.template_type_parameter_inhabitations[0];
    assert_eq!(lowered.parameter, TemplateParameterKey::new("T"));
    assert_eq!(lowered.assumption, output.assumptions[0]);
    assert_eq!(lowered.witness.var, witness);
    assert_eq!(
        lowered.witness.role,
        CoreVarRole::new("template-type-witness")
    );
    assert_eq!(lowered.witness.source_name.as_deref(), Some("T$inhabitant"));

    let CoreFormulaKind::Exists { binders, body } = &output
        .formulas
        .get(lowered.assumption)
        .expect("exists formula")
        .kind
    else {
        panic!("schema parameter assumption must be an existential formula");
    };
    assert_eq!(binders, &vec![lowered.witness.clone()]);
    assert_eq!(*body, lowered.witness_fact);
    let CoreFormulaKind::TypePred { subject, ty } = &output
        .formulas
        .get(lowered.witness_fact)
        .expect("witness fact")
        .kind
    else {
        panic!("schema witness body must be a type predicate");
    };
    assert_eq!(*subject, lowered.witness_term);
    assert_eq!(ty, &CoreTypePredicate::new("is_T"));
    assert!(matches!(
        output.terms.get(lowered.witness_term).expect("witness term").kind,
        CoreTermKind::Var(actual) if actual == witness
    ));
    assert!(output.template_type_actual_gates.is_empty());
    assert!(output.diagnostics.is_empty());
    assert!(output.obligation_seeds.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn template_type_actual_gates_preserve_accepted_evidence_without_axioms() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let mut registration_gate = template_type_actual_gate(
        "Inhab[prime Nat]",
        "T",
        ExistentialGateStatus::Satisfied,
        47,
    );
    registration_gate.registration = Some(CheckerRegistrationId::new(5));
    let mut base_gate =
        template_type_actual_gate("Inhab[set]", "U", ExistentialGateStatus::Satisfied, 48);
    base_gate.base_evidence_kind = Some(ExistentialGateBaseEvidenceKind::BuiltinSet);
    base_gate.base_evidence_coverage = Some(ExistentialGateBaseEvidenceCoverage::Builtin);
    let mut guard_gate =
        template_type_actual_gate("Inhab[Struct]", "V", ExistentialGateStatus::Satisfied, 49);
    guard_gate.facts = vec![TypeFactId::new(9), TypeFactId::new(3), TypeFactId::new(3)];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_actual_gates = vec![registration_gate, base_gate, guard_gate];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    assert_eq!(output.template_type_actual_gates.len(), 3);
    assert!(output.assumptions.is_empty());
    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert!(output.diagnostics.is_empty());
    assert!(output.obligation_seeds.is_empty());

    assert_eq!(
        output.template_type_actual_gates[0].registration,
        Some(CheckerRegistrationId::new(5))
    );
    assert_eq!(
        output.template_type_actual_gates[1].base_evidence_kind,
        Some(ExistentialGateBaseEvidenceKind::BuiltinSet)
    );
    assert_eq!(
        output.template_type_actual_gates[1].base_evidence_coverage,
        Some(ExistentialGateBaseEvidenceCoverage::Builtin)
    );
    assert_eq!(
        output.template_type_actual_gates[2].facts,
        vec![TypeFactId::new(3), TypeFactId::new(9)]
    );
    assert!(
        output
            .template_type_actual_gates
            .iter()
            .all(|gate| gate.status == ExistentialGateStatus::Satisfied
                && gate.diagnostic.is_none()
                && !gate.provenance.is_empty())
    );
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn missing_template_type_actual_gate_rejects_without_existential_axiom() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let mut missing = template_type_actual_gate(
        "Inhab[hollow set]",
        "T",
        ExistentialGateStatus::MissingExistential,
        50,
    );
    missing.diagnostics = vec![RegistrationDiagnosticId::new(12)];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_actual_gates = vec![missing];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    assert_eq!(output.template_type_actual_gates.len(), 1);
    let gate = &output.template_type_actual_gates[0];
    let diagnostic = gate.diagnostic.expect("core diagnostic");
    assert_eq!(
        gate.checker_diagnostics,
        vec![RegistrationDiagnosticId::new(12)]
    );
    assert_eq!(
        output
            .diagnostics
            .get(diagnostic)
            .expect("diagnostic")
            .message_key
            .as_str(),
        "missing-template-type-actual-inhabitation"
    );
    assert!(output.assumptions.is_empty());
    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert!(output.obligation_seeds.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn unsatisfied_template_type_actual_gate_status_matrix_is_diagnostic_only() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let cases = [
        (
            ExistentialGateStatus::MissingExistential,
            "missing-template-type-actual-inhabitation",
        ),
        (
            ExistentialGateStatus::BlockedGuard,
            "blocked-template-type-actual-inhabitation-guard",
        ),
        (
            ExistentialGateStatus::InvalidCandidate,
            "invalid-template-type-actual-inhabitation-candidate",
        ),
        (
            ExistentialGateStatus::DegradedRecovery,
            "degraded-template-type-actual-inhabitation",
        ),
    ];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_actual_gates = cases
        .iter()
        .enumerate()
        .map(|(offset, (status, _))| {
            let mut seed = template_type_actual_gate(
                format!("Inhab[case{offset}]").as_str(),
                format!("T{offset}").as_str(),
                *status,
                56 + offset,
            );
            seed.diagnostics = vec![RegistrationDiagnosticId::new(100 + offset)];
            seed
        })
        .collect();

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    assert_eq!(output.template_type_actual_gates.len(), cases.len());
    for (gate, (status, message)) in output.template_type_actual_gates.iter().zip(cases) {
        assert_eq!(gate.status, status);
        assert!(gate.registration.is_none());
        assert!(gate.base_evidence_kind.is_none());
        assert!(gate.base_evidence_coverage.is_none());
        assert!(gate.facts.is_empty());
        let diagnostic = gate.diagnostic.expect("diagnostic-only rejected gate");
        assert_eq!(
            output
                .diagnostics
                .get(diagnostic)
                .expect("diagnostic")
                .message_key
                .as_str(),
            message
        );
    }
    assert!(output.assumptions.is_empty());
    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert!(output.obligation_seeds.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn template_type_actual_gate_payloads_fail_closed() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let no_evidence = template_type_actual_gate(
        "Inhab[NoEvidence]",
        "T",
        ExistentialGateStatus::Satisfied,
        51,
    );
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_actual_gates = vec![no_evidence];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::SatisfiedTemplateTypeActualWithoutEvidence { .. })
    ));

    let mut partial_base =
        template_type_actual_gate("Inhab[partial]", "T", ExistentialGateStatus::Satisfied, 52);
    partial_base.base_evidence_kind = Some(ExistentialGateBaseEvidenceKind::BuiltinObject);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_actual_gates = vec![partial_base];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::PartialTemplateTypeActualBaseEvidence { .. })
    ));

    let mut unsatisfied_with_registration = template_type_actual_gate(
        "Inhab[hollow set]",
        "T",
        ExistentialGateStatus::MissingExistential,
        53,
    );
    unsatisfied_with_registration.registration = Some(CheckerRegistrationId::new(9));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_actual_gates = vec![unsatisfied_with_registration];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(
            TypeAndFactLoweringError::UnsatisfiedTemplateTypeActualCarriesEvidence {
                status: ExistentialGateStatus::MissingExistential,
                ..
            }
        )
    ));

    let mut unsatisfied_with_base = template_type_actual_gate(
        "Inhab[blocked-base]",
        "T",
        ExistentialGateStatus::BlockedGuard,
        60,
    );
    unsatisfied_with_base.base_evidence_kind = Some(ExistentialGateBaseEvidenceKind::BuiltinObject);
    unsatisfied_with_base.base_evidence_coverage =
        Some(ExistentialGateBaseEvidenceCoverage::Builtin);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_actual_gates = vec![unsatisfied_with_base];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(
            TypeAndFactLoweringError::UnsatisfiedTemplateTypeActualCarriesEvidence {
                status: ExistentialGateStatus::BlockedGuard,
                ..
            }
        )
    ));

    let mut unsatisfied_with_facts = template_type_actual_gate(
        "Inhab[invalid-fact]",
        "T",
        ExistentialGateStatus::InvalidCandidate,
        61,
    );
    unsatisfied_with_facts.facts = vec![TypeFactId::new(10)];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_actual_gates = vec![unsatisfied_with_facts];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(
            TypeAndFactLoweringError::UnsatisfiedTemplateTypeActualCarriesEvidence {
                status: ExistentialGateStatus::InvalidCandidate,
                ..
            }
        )
    ));

    let mut first =
        template_type_actual_gate("Inhab[set]", "T", ExistentialGateStatus::Satisfied, 62);
    first.registration = Some(CheckerRegistrationId::new(1));
    let mut second =
        template_type_actual_gate("Inhab[set]", "T", ExistentialGateStatus::Satisfied, 63);
    second.registration = Some(CheckerRegistrationId::new(2));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_actual_gates = vec![first, second];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::DuplicateTemplateTypeActualGate { .. })
    ));
}

#[test]
fn template_type_parameter_sethood_records_bound_constraint_and_bare_missing() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let mut bound = template_sethood_seed(
        "T",
        "T:bound:Nat",
        TemplateTypeParameterSethoodSource::BoundInherited,
        TemplateTypeParameterSethoodStatus::Accepted,
        64,
    );
    bound.facts = vec![TypeFactId::new(9), TypeFactId::new(8), TypeFactId::new(9)];
    bound.diagnostics = vec![TypeDiagnosticId::new(4), TypeDiagnosticId::new(4)];
    let mut constraint = template_sethood_seed(
        "U",
        "U:constraint:sethood",
        TemplateTypeParameterSethoodSource::ConstraintSupplied,
        TemplateTypeParameterSethoodStatus::Accepted,
        65,
    );
    constraint.facts = vec![TypeFactId::new(10)];
    let bare = template_sethood_seed(
        "Bare",
        "Bare:bare",
        TemplateTypeParameterSethoodSource::BareParameter,
        TemplateTypeParameterSethoodStatus::Missing,
        66,
    );

    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_parameter_sethoods = vec![bound, constraint, bare];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");

    assert_eq!(output.template_type_parameter_sethoods.len(), 3);
    let bound = &output.template_type_parameter_sethoods[0];
    assert_eq!(bound.parameter.as_str(), "T");
    assert_eq!(bound.evidence_key.as_str(), "T:bound:Nat");
    assert_eq!(
        bound.source_kind,
        TemplateTypeParameterSethoodSource::BoundInherited
    );
    assert_eq!(bound.status, TemplateTypeParameterSethoodStatus::Accepted);
    assert_eq!(bound.facts, vec![TypeFactId::new(8), TypeFactId::new(9)]);
    assert_eq!(bound.checker_diagnostics, vec![TypeDiagnosticId::new(4)]);
    assert!(bound.diagnostic.is_none());

    let constraint = &output.template_type_parameter_sethoods[1];
    assert_eq!(constraint.parameter.as_str(), "U");
    assert_eq!(constraint.evidence_key.as_str(), "U:constraint:sethood");
    assert_eq!(
        constraint.source_kind,
        TemplateTypeParameterSethoodSource::ConstraintSupplied
    );
    assert_eq!(
        constraint.status,
        TemplateTypeParameterSethoodStatus::Accepted
    );
    assert_eq!(constraint.facts, vec![TypeFactId::new(10)]);
    assert!(constraint.diagnostic.is_none());

    let bare = &output.template_type_parameter_sethoods[2];
    assert_eq!(bare.parameter.as_str(), "Bare");
    assert_eq!(
        bare.source_kind,
        TemplateTypeParameterSethoodSource::BareParameter
    );
    assert_eq!(bare.status, TemplateTypeParameterSethoodStatus::Missing);
    assert!(bare.facts.is_empty());
    let diagnostic = output
        .diagnostics
        .get(bare.diagnostic.expect("bare diagnostic"))
        .expect("diagnostic");
    assert_eq!(
        diagnostic.message_key.as_str(),
        "missing-template-type-parameter-sethood"
    );
    assert!(output.assumptions.is_empty());
    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert!(output.obligation_seeds.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn template_type_parameter_sethood_payloads_fail_closed() {
    let (context, owner) = context_with_var(CoreVarId::new(0));

    let mut accepted_bare = template_sethood_seed(
        "Bare",
        "Bare:accepted",
        TemplateTypeParameterSethoodSource::BareParameter,
        TemplateTypeParameterSethoodStatus::Accepted,
        67,
    );
    accepted_bare.facts = vec![TypeFactId::new(11)];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_parameter_sethoods = vec![accepted_bare];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::BareTemplateTypeParameterSethoodAccepted { .. })
    ));

    let no_facts = template_sethood_seed(
        "T",
        "T:no-facts",
        TemplateTypeParameterSethoodSource::BoundInherited,
        TemplateTypeParameterSethoodStatus::Accepted,
        68,
    );
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_parameter_sethoods = vec![no_facts];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::AcceptedTemplateTypeParameterSethoodWithoutEvidence { .. })
    ));

    let mut missing_with_facts = template_sethood_seed(
        "Bare",
        "Bare:missing-with-facts",
        TemplateTypeParameterSethoodSource::BareParameter,
        TemplateTypeParameterSethoodStatus::Missing,
        69,
    );
    missing_with_facts.facts = vec![TypeFactId::new(12)];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_parameter_sethoods = vec![missing_with_facts];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(
            TypeAndFactLoweringError::MissingTemplateTypeParameterSethoodCarriesEvidence {
                status: TemplateTypeParameterSethoodStatus::Missing,
                ..
            }
        )
    ));

    let missing_bound = template_sethood_seed(
        "T",
        "T:missing-bound",
        TemplateTypeParameterSethoodSource::BoundInherited,
        TemplateTypeParameterSethoodStatus::Missing,
        70,
    );
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_parameter_sethoods = vec![missing_bound];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(
            TypeAndFactLoweringError::MissingTemplateTypeParameterSethoodWrongSource {
                source_kind: TemplateTypeParameterSethoodSource::BoundInherited,
                ..
            }
        )
    ));

    let mut degraded_with_facts = template_sethood_seed(
        "T",
        "T:degraded",
        TemplateTypeParameterSethoodSource::ConstraintSupplied,
        TemplateTypeParameterSethoodStatus::DegradedRecovery,
        71,
    );
    degraded_with_facts.facts = vec![TypeFactId::new(13)];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_parameter_sethoods = vec![degraded_with_facts];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(
            TypeAndFactLoweringError::MissingTemplateTypeParameterSethoodCarriesEvidence {
                status: TemplateTypeParameterSethoodStatus::DegradedRecovery,
                ..
            }
        )
    ));

    let mut first = template_sethood_seed(
        "T",
        "T:dup",
        TemplateTypeParameterSethoodSource::BoundInherited,
        TemplateTypeParameterSethoodStatus::Accepted,
        72,
    );
    first.facts = vec![TypeFactId::new(14)];
    let mut second = first.clone();
    second.source = direct(73, 74);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_parameter_sethoods = vec![first, second];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::DuplicateTemplateTypeParameterSethood { .. })
    ));

    let mut accepted = template_sethood_seed(
        "T",
        "T:conflict",
        TemplateTypeParameterSethoodSource::BoundInherited,
        TemplateTypeParameterSethoodStatus::Accepted,
        74,
    );
    accepted.facts = vec![TypeFactId::new(15)];
    let conflicting_missing = template_sethood_seed(
        "T",
        "T:conflict",
        TemplateTypeParameterSethoodSource::BareParameter,
        TemplateTypeParameterSethoodStatus::Missing,
        75,
    );
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_type_parameter_sethoods = vec![accepted, conflicting_missing];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::DuplicateTemplateTypeParameterSethood { .. })
    ));
}

#[test]
fn template_scheme_deffunc_actual_preserves_signature_evidence_and_guard_seed() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let mut actual = template_scheme_actual(
        "Iter[double]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Deffunc,
        TemplateSchemeActualStatus::Accepted,
        1,
        70,
    );
    actual.domain_evidence = vec![widening_evidence(10, 11, &[9, 3, 3])];
    actual.codomain_evidence = Some(widening_evidence(12, 13, &[7]));
    actual.guard_obligation = Some(skipped_guard_obligation(71));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![actual];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    assert_eq!(output.template_scheme_actuals.len(), 1);
    let lowered = &output.template_scheme_actuals[0];
    assert_eq!(lowered.actual_kind, TemplateSchemeActualKind::Deffunc);
    assert_eq!(lowered.status, TemplateSchemeActualStatus::Accepted);
    assert_eq!(
        lowered.domain_evidence,
        vec![TemplateDirectionalWideningEvidence {
            from_type: NormalizedTypeId::new(10),
            to_type: NormalizedTypeId::new(11),
            status: TemplateWideningEvidenceStatus::Accepted,
            facts: vec![TypeFactId::new(3), TypeFactId::new(9)],
        }]
    );
    assert_eq!(
        lowered.codomain_evidence,
        Some(TemplateDirectionalWideningEvidence {
            from_type: NormalizedTypeId::new(12),
            to_type: NormalizedTypeId::new(13),
            status: TemplateWideningEvidenceStatus::Accepted,
            facts: vec![TypeFactId::new(7)],
        })
    );

    let guard = lowered.guard_obligation.expect("skipped guard seed");
    let guard_seed = output
        .obligation_seeds
        .get(guard)
        .expect("guard obligation");
    assert_eq!(guard_seed.kind, ObligationSeedKind::CheckerInitial);
    assert_eq!(guard_seed.status, ObligationSeedStatus::Skipped);
    assert!(guard_seed.goal.is_none());
    assert!(output.assumptions.is_empty());
    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert!(output.diagnostics.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn rejected_template_scheme_actuals_are_diagnostic_only() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let result_mismatch = template_scheme_actual(
        "Iter[shrink]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Deffunc,
        TemplateSchemeActualStatus::SignatureMismatch,
        1,
        72,
    );
    let defpred_mismatch = template_scheme_actual(
        "Induction[narrow]",
        "P",
        TemplateSchemeParameterKind::Predicate,
        TemplateSchemeActualKind::Defpred,
        TemplateSchemeActualStatus::SignatureMismatch,
        1,
        73,
    );
    let mut partial_algorithm = template_scheme_actual(
        "Sigma[partial]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::PartialAlgorithm,
        TemplateSchemeActualStatus::PartialAlgorithm,
        1,
        74,
    );
    partial_algorithm.checker_diagnostics = vec![TypeDiagnosticId::new(8)];
    let void_algorithm = template_scheme_actual(
        "Sigma[void]",
        "F3",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::VoidAlgorithm,
        TemplateSchemeActualStatus::VoidAlgorithm,
        1,
        75,
    );
    let unsupported_actual = template_scheme_actual(
        "Scheme[template-predicate]",
        "P2",
        TemplateSchemeParameterKind::Predicate,
        TemplateSchemeActualKind::Unsupported,
        TemplateSchemeActualStatus::Unsupported,
        1,
        76,
    );
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![
        result_mismatch,
        defpred_mismatch,
        partial_algorithm,
        void_algorithm,
        unsupported_actual,
    ];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    assert_eq!(output.template_scheme_actuals.len(), 5);
    let messages: Vec<_> = output
        .template_scheme_actuals
        .iter()
        .map(|actual| {
            let diagnostic = actual.diagnostic.expect("diagnostic-only rejection");
            output
                .diagnostics
                .get(diagnostic)
                .expect("diagnostic")
                .message_key
                .as_str()
                .to_owned()
        })
        .collect();
    assert_eq!(
        messages,
        vec![
            "template-scheme-actual-signature-mismatch",
            "template-scheme-actual-signature-mismatch",
            "partial-algorithm-template-functor-actual",
            "void-algorithm-template-functor-actual",
            "unsupported-template-scheme-actual",
        ]
    );
    assert_eq!(
        output.template_scheme_actuals[2].checker_diagnostics,
        vec![TypeDiagnosticId::new(8)]
    );
    assert!(
        output
            .template_scheme_actuals
            .iter()
            .all(|actual| actual.domain_evidence.is_empty()
                && actual.codomain_evidence.is_none()
                && actual.guard_obligation.is_none()
                && actual.substitution.is_none())
    );
    assert!(output.assumptions.is_empty());
    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert!(output.obligation_seeds.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn accepted_plain_template_scheme_type_actual_preserves_type_row_without_callable_evidence() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let type_actual = template_scheme_actual(
        "Choice[Nat]",
        "T",
        TemplateSchemeParameterKind::Type,
        TemplateSchemeActualKind::TypeExpression,
        TemplateSchemeActualStatus::Accepted,
        0,
        74,
    );

    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![type_actual];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    assert_eq!(output.template_scheme_actuals.len(), 1);
    let lowered = &output.template_scheme_actuals[0];
    assert_eq!(lowered.parameter_kind, TemplateSchemeParameterKind::Type);
    assert_eq!(
        lowered.actual_kind,
        TemplateSchemeActualKind::TypeExpression
    );
    assert_eq!(lowered.status, TemplateSchemeActualStatus::Accepted);
    assert!(lowered.domain_evidence.is_empty());
    assert!(lowered.codomain_evidence.is_none());
    assert!(lowered.guard_obligation.is_none());
    assert!(lowered.substitution.is_none());
    assert!(output.assumptions.is_empty());
    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert!(output.obligation_seeds.is_empty());
    assert!(output.diagnostics.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn accepted_defpred_template_functor_and_promoted_algorithm_actuals() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let mut pred = template_scheme_actual(
        "Induction[IsOdd]",
        "P",
        TemplateSchemeParameterKind::Predicate,
        TemplateSchemeActualKind::Defpred,
        TemplateSchemeActualStatus::Accepted,
        1,
        75,
    );
    pred.domain_evidence = vec![widening_evidence(20, 21, &[11])];

    let mut template_functor = template_scheme_actual(
        "Sigma[Square]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::TemplateFunctor,
        TemplateSchemeActualStatus::Accepted,
        1,
        76,
    );
    template_functor.domain_evidence = vec![widening_evidence(22, 23, &[12])];
    template_functor.codomain_evidence = Some(widening_evidence(24, 25, &[13]));
    template_functor.guard_obligation = Some(skipped_guard_obligation(77));

    let mut promoted_algorithm = template_scheme_actual(
        "Sigma[factorial]",
        "F2",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::PromotedTerminatingAlgorithm,
        TemplateSchemeActualStatus::Accepted,
        1,
        78,
    );
    promoted_algorithm.domain_evidence = vec![widening_evidence(26, 27, &[14])];
    promoted_algorithm.codomain_evidence = Some(widening_evidence(28, 29, &[15]));
    promoted_algorithm.guard_obligation = Some(skipped_guard_obligation(79));

    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![pred, template_functor, promoted_algorithm];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    assert_eq!(output.template_scheme_actuals.len(), 3);
    assert_eq!(
        output.template_scheme_actuals[0].actual_kind,
        TemplateSchemeActualKind::Defpred
    );
    assert!(output.template_scheme_actuals[0].guard_obligation.is_none());
    assert!(
        output.template_scheme_actuals[0]
            .codomain_evidence
            .is_none()
    );
    assert_eq!(
        output.template_scheme_actuals[1].actual_kind,
        TemplateSchemeActualKind::TemplateFunctor
    );
    assert_eq!(
        output.template_scheme_actuals[2].actual_kind,
        TemplateSchemeActualKind::PromotedTerminatingAlgorithm
    );
    assert_eq!(output.obligation_seeds.iter().count(), 2);
    assert!(
        output
            .obligation_seeds
            .iter()
            .all(|(_, seed)| seed.status == ObligationSeedStatus::Skipped
                && seed.kind == ObligationSeedKind::CheckerInitial)
    );
    assert!(output.diagnostics.is_empty());
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn enclosing_scheme_parameters_preserve_substitution_metadata_without_symbols() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let mut type_actual = template_scheme_actual(
        "Inner[T]",
        "T",
        TemplateSchemeParameterKind::Type,
        TemplateSchemeActualKind::EnclosingTypeParameter,
        TemplateSchemeActualStatus::Accepted,
        0,
        80,
    );
    type_actual.substitution = Some(substitution_composition("OuterT", 81));

    let mut pred_actual = template_scheme_actual(
        "Inner[P]",
        "P",
        TemplateSchemeParameterKind::Predicate,
        TemplateSchemeActualKind::EnclosingPredicateParameter,
        TemplateSchemeActualStatus::Accepted,
        1,
        82,
    );
    pred_actual.domain_evidence = vec![widening_evidence(30, 31, &[16])];
    pred_actual.substitution = Some(substitution_composition("OuterP", 83));

    let mut functor_actual = template_scheme_actual(
        "Inner[F]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::EnclosingFunctorParameter,
        TemplateSchemeActualStatus::Accepted,
        1,
        84,
    );
    functor_actual.domain_evidence = vec![widening_evidence(32, 33, &[17])];
    functor_actual.codomain_evidence = Some(widening_evidence(34, 35, &[18]));
    functor_actual.guard_obligation = Some(skipped_guard_obligation(85));
    functor_actual.substitution = Some(substitution_composition("OuterF", 86));

    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![type_actual, pred_actual, functor_actual];

    let output = lower_type_and_fact_inputs(&context, input).expect("lowering");
    assert_eq!(output.template_scheme_actuals.len(), 3);
    assert_eq!(
        output
            .template_scheme_actuals
            .iter()
            .map(|actual| actual
                .substitution
                .as_ref()
                .expect("substitution")
                .enclosing_parameter
                .as_str()
                .to_owned())
            .collect::<Vec<_>>(),
        vec!["OuterT", "OuterP", "OuterF"]
    );
    assert!(output.assumptions.is_empty());
    assert!(output.terms.is_empty());
    assert!(output.formulas.is_empty());
    assert!(output.diagnostics.is_empty());
    assert_eq!(output.obligation_seeds.iter().count(), 1);
    assert_step2_delta_valid(&context, &output);
}

#[test]
fn template_scheme_actual_payloads_fail_closed() {
    let (context, owner) = context_with_var(CoreVarId::new(0));

    let mut missing_guard = template_scheme_actual(
        "Sigma[no-guard]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Deffunc,
        TemplateSchemeActualStatus::Accepted,
        1,
        87,
    );
    missing_guard.domain_evidence = vec![widening_evidence(40, 41, &[19])];
    missing_guard.codomain_evidence = Some(widening_evidence(42, 43, &[20]));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![missing_guard];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeFunctorMissingGuardSeed { .. })
    ));

    let mut active_guard = template_scheme_actual(
        "Sigma[active-guard]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Deffunc,
        TemplateSchemeActualStatus::Accepted,
        1,
        88,
    );
    active_guard.domain_evidence = vec![widening_evidence(44, 45, &[21])];
    active_guard.codomain_evidence = Some(widening_evidence(46, 47, &[22]));
    active_guard.guard_obligation = Some(active_obligation(CoreVarId::new(0), "guard", 89));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![active_guard];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(
            TypeAndFactLoweringError::TemplateSchemeFunctorInvalidGuardSeedStatus {
                status: ObligationSeedStatus::Active,
                ..
            }
        )
    ));

    for (kind, start) in [
        (InitialObligationKind::Sethood, 112),
        (InitialObligationKind::NonEmptiness, 114),
    ] {
        let mut generated_kind_guard = template_scheme_actual(
            "Sigma[generated-guard]",
            "F",
            TemplateSchemeParameterKind::Functor,
            TemplateSchemeActualKind::Deffunc,
            TemplateSchemeActualStatus::Accepted,
            1,
            start,
        );
        generated_kind_guard.domain_evidence = vec![widening_evidence(80, 81, &[37])];
        generated_kind_guard.codomain_evidence = Some(widening_evidence(82, 83, &[38]));
        let mut guard = skipped_guard_obligation(start + 1);
        guard.checker_kind = kind;
        generated_kind_guard.guard_obligation = Some(guard);
        let mut input = TypeAndFactLoweringInput::new(owner);
        input.template_scheme_actuals = vec![generated_kind_guard];
        assert!(matches!(
            lower_type_and_fact_inputs(&context, input),
            Err(
                TypeAndFactLoweringError::TemplateSchemeFunctorInvalidGuardSeedKind {
                    kind: rejected_kind,
                    ..
                }
            ) if rejected_kind == kind
        ));
    }

    let mut predicate_with_codomain = template_scheme_actual(
        "Induction[codomain]",
        "P",
        TemplateSchemeParameterKind::Predicate,
        TemplateSchemeActualKind::Defpred,
        TemplateSchemeActualStatus::Accepted,
        1,
        90,
    );
    predicate_with_codomain.domain_evidence = vec![widening_evidence(48, 49, &[23])];
    predicate_with_codomain.codomain_evidence = Some(widening_evidence(50, 51, &[24]));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![predicate_with_codomain];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeActualInvalidCodomainEvidence { .. })
    ));

    let mut rejected_with_evidence = template_scheme_actual(
        "Sigma[bad]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::PartialAlgorithm,
        TemplateSchemeActualStatus::PartialAlgorithm,
        1,
        91,
    );
    rejected_with_evidence.domain_evidence = vec![widening_evidence(52, 53, &[25])];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![rejected_with_evidence];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(
            TypeAndFactLoweringError::RejectedTemplateSchemeActualCarriesEvidence {
                status: TemplateSchemeActualStatus::PartialAlgorithm,
                ..
            }
        )
    ));

    let mut type_with_callable_evidence = template_scheme_actual(
        "Inner[type-evidence]",
        "T",
        TemplateSchemeParameterKind::Type,
        TemplateSchemeActualKind::TypeExpression,
        TemplateSchemeActualStatus::Accepted,
        0,
        92,
    );
    type_with_callable_evidence.domain_evidence = vec![widening_evidence(54, 55, &[26])];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![type_with_callable_evidence];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeTypeActualCarriesCallableEvidence { .. })
    ));

    let missing_predicate_domain = template_scheme_actual(
        "Induction[missing-domain]",
        "P",
        TemplateSchemeParameterKind::Predicate,
        TemplateSchemeActualKind::Defpred,
        TemplateSchemeActualStatus::Accepted,
        1,
        93,
    );
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![missing_predicate_domain];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::AcceptedTemplateSchemeActualMissingEvidence { .. })
    ));

    let mut missing_functor_domain = template_scheme_actual(
        "Sigma[missing-domain]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Deffunc,
        TemplateSchemeActualStatus::Accepted,
        1,
        94,
    );
    missing_functor_domain.codomain_evidence = Some(widening_evidence(64, 65, &[31]));
    missing_functor_domain.guard_obligation = Some(skipped_guard_obligation(95));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![missing_functor_domain];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::AcceptedTemplateSchemeActualMissingEvidence { .. })
    ));

    let mut missing_functor_codomain = template_scheme_actual(
        "Sigma[missing-codomain]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Deffunc,
        TemplateSchemeActualStatus::Accepted,
        1,
        96,
    );
    missing_functor_codomain.domain_evidence = vec![widening_evidence(66, 67, &[32])];
    missing_functor_codomain.guard_obligation = Some(skipped_guard_obligation(97));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![missing_functor_codomain];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::AcceptedTemplateSchemeActualMissingEvidence { .. })
    ));

    let mut partial_domain = template_scheme_actual(
        "Induction[partial-domain]",
        "P",
        TemplateSchemeParameterKind::Predicate,
        TemplateSchemeActualKind::Defpred,
        TemplateSchemeActualStatus::Accepted,
        1,
        98,
    );
    partial_domain.domain_evidence = vec![TemplateWideningEvidenceSeed {
        from_type: NormalizedTypeId::new(68),
        to_type: NormalizedTypeId::new(69),
        status: TemplateWideningEvidenceStatus::DeferredExternalDependency,
        facts: Vec::new(),
    }];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![partial_domain];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeActualPartialDomainEvidence { .. })
    ));

    let mut partial_codomain = template_scheme_actual(
        "Sigma[partial-codomain]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Deffunc,
        TemplateSchemeActualStatus::Accepted,
        1,
        99,
    );
    partial_codomain.domain_evidence = vec![widening_evidence(70, 71, &[33])];
    partial_codomain.codomain_evidence = Some(TemplateWideningEvidenceSeed {
        from_type: NormalizedTypeId::new(72),
        to_type: NormalizedTypeId::new(73),
        status: TemplateWideningEvidenceStatus::Missing,
        facts: Vec::new(),
    });
    partial_codomain.guard_obligation = Some(skipped_guard_obligation(100));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![partial_codomain];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeActualInvalidCodomainEvidence { .. })
    ));

    let missing_type_substitution = template_scheme_actual(
        "Inner[missing-type-subst]",
        "T",
        TemplateSchemeParameterKind::Type,
        TemplateSchemeActualKind::EnclosingTypeParameter,
        TemplateSchemeActualStatus::Accepted,
        0,
        101,
    );
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![missing_type_substitution];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeActualMissingSubstitutionEvidence { .. })
    ));

    let mut missing_predicate_substitution = template_scheme_actual(
        "Inner[missing-predicate-subst]",
        "P",
        TemplateSchemeParameterKind::Predicate,
        TemplateSchemeActualKind::EnclosingPredicateParameter,
        TemplateSchemeActualStatus::Accepted,
        1,
        102,
    );
    missing_predicate_substitution.domain_evidence = vec![widening_evidence(74, 75, &[34])];
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![missing_predicate_substitution];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeActualMissingSubstitutionEvidence { .. })
    ));

    let mut missing_functor_substitution = template_scheme_actual(
        "Inner[missing-functor-subst]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::EnclosingFunctorParameter,
        TemplateSchemeActualStatus::Accepted,
        1,
        103,
    );
    missing_functor_substitution.domain_evidence = vec![widening_evidence(76, 77, &[35])];
    missing_functor_substitution.codomain_evidence = Some(widening_evidence(78, 79, &[36]));
    missing_functor_substitution.guard_obligation = Some(skipped_guard_obligation(104));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![missing_functor_substitution];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeActualMissingSubstitutionEvidence { .. })
    ));

    let role_mismatch = template_scheme_actual(
        "Sigma[predicate-as-functor]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Defpred,
        TemplateSchemeActualStatus::RoleMismatch,
        1,
        105,
    );
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![role_mismatch];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeActualKindMismatch { .. })
    ));

    let mut arity_mismatch = template_scheme_actual(
        "Induction[arity]",
        "P",
        TemplateSchemeParameterKind::Predicate,
        TemplateSchemeActualKind::Defpred,
        TemplateSchemeActualStatus::ArityMismatch,
        2,
        106,
    );
    arity_mismatch.actual_arity = 1;
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![arity_mismatch];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeActualArityMismatch { .. })
    ));

    let mut unsupported_accepted = template_scheme_actual(
        "Sigma[unsupported]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Unsupported,
        TemplateSchemeActualStatus::Accepted,
        1,
        107,
    );
    unsupported_accepted.domain_evidence = vec![widening_evidence(56, 57, &[27])];
    unsupported_accepted.codomain_evidence = Some(widening_evidence(58, 59, &[28]));
    unsupported_accepted.guard_obligation = Some(skipped_guard_obligation(108));
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![unsupported_accepted];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::TemplateSchemeActualKindMismatch { .. })
    ));

    let mut first = template_scheme_actual(
        "Sigma[dup]",
        "F",
        TemplateSchemeParameterKind::Functor,
        TemplateSchemeActualKind::Deffunc,
        TemplateSchemeActualStatus::Accepted,
        1,
        109,
    );
    first.domain_evidence = vec![widening_evidence(60, 61, &[29])];
    first.codomain_evidence = Some(widening_evidence(62, 63, &[30]));
    first.guard_obligation = Some(skipped_guard_obligation(110));
    let mut second = first.clone();
    second.source = direct(111, 112);
    let mut input = TypeAndFactLoweringInput::new(owner);
    input.template_scheme_actuals = vec![first, second];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, input),
        Err(TypeAndFactLoweringError::DuplicateTemplateSchemeActual { .. })
    ));
}

#[test]
fn type_fact_subject_must_be_declared_term_variable() {
    let var = CoreVarId::new(0);
    let (formula_context, owner) = context_with_var_sort(var, NormalizedVarSort::Formula);
    let mut formula_input = TypeAndFactLoweringInput::new(owner);
    formula_input.formula_assertions = vec![type_fact(var, "set", 50, Polarity::Positive)];

    assert!(matches!(
        lower_type_and_fact_inputs(&formula_context, formula_input),
        Err(TypeAndFactLoweringError::NonTermSubject { var: actual, sort: NormalizedVarSort::Formula })
            if actual == var
    ));

    let (context, owner) = context_with_var(CoreVarId::new(1));
    let mut undeclared_input = TypeAndFactLoweringInput::new(owner);
    undeclared_input.formula_assertions =
        vec![type_fact(CoreVarId::new(99), "set", 51, Polarity::Positive)];

    assert!(matches!(
        lower_type_and_fact_inputs(&context, undeclared_input),
        Err(TypeAndFactLoweringError::UndeclaredSubject { var }) if var == CoreVarId::new(99)
    ));
}

#[test]
fn reconsidering_and_cluster_facts_enforce_checker_boundaries() {
    let var = CoreVarId::new(0);
    let (context, owner) = context_with_var(var);

    let mut reconsidering_input = TypeAndFactLoweringInput::new(owner);
    reconsidering_input.reconsiderings = vec![ReconsideringSeed::new(
        CoreVarId::new(77),
        "term-binder",
        direct(52, 53),
        provenance("checker:bad-reconsider"),
    )];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, reconsidering_input),
        Err(TypeAndFactLoweringError::UndeclaredSubject { var }) if var == CoreVarId::new(77)
    ));

    let mut cluster_input = TypeAndFactLoweringInput::new(owner);
    cluster_input.cluster_facts = vec![ClusterFactSeed {
        cluster_fact: ClusterFactId::new(9),
        fact: type_fact(var, "cluster:accepted-only", 54, Polarity::Positive),
    }];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, cluster_input),
        Err(TypeAndFactLoweringError::ClusterFactMissingCheckerFact { cluster_fact })
            if cluster_fact == ClusterFactId::new(9)
    ));
}

#[test]
fn numeral_seed_spelling_is_preserved_and_invalid_digits_rejected() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    for digits in [
        "0",
        "0008",
        "18446744073709551616000000000",
        "",
        "-1",
        "+1",
        "a",
        "１",
    ] {
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input
            .terms
            .push(term_seed(CoreTermSeedKind::Numeral(digits.into()), 60));
        let output = lower_term_and_formula_inputs(&context, input);
        if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
            let output = output.unwrap();
            assert_eq!(
                output.terms.get(CoreTermId::new(0)).unwrap().kind,
                CoreTermKind::Numeral(digits.into())
            );
        } else {
            assert!(matches!(
                output,
                Err(TermAndFormulaLoweringError::InvalidNumeral)
            ));
        }
    }
}

#[test]
fn term_lowering_covers_core_term_shapes() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 60),
        term_seed(CoreTermSeedKind::Const(symbol("Const")), 61),
        term_seed(
            CoreTermSeedKind::Apply {
                functor: symbol("Functor"),
                args: vec![CoreTermSeedId::new(0), CoreTermSeedId::new(1)],
            },
            62,
        ),
        term_seed(
            CoreTermSeedKind::Select {
                selector: symbol("selector"),
                base: CoreTermSeedId::new(2),
            },
            63,
        ),
        term_seed(
            CoreTermSeedKind::Tuple(vec![CoreTermSeedId::new(0), CoreTermSeedId::new(3)]),
            64,
        ),
        term_seed(
            CoreTermSeedKind::SetEnum(vec![CoreTermSeedId::new(1), CoreTermSeedId::new(4)]),
            65,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];

    assert!(matches!(
        output.terms.get(term_id(0)).expect("var").kind,
        CoreTermKind::Var(var) if var == x
    ));
    assert!(matches!(
        &output.terms.get(term_id(1)).expect("const").kind,
        CoreTermKind::Const(symbol_id) if symbol_id == &symbol("Const")
    ));
    assert!(matches!(
        &output.terms.get(term_id(2)).expect("apply").kind,
        CoreTermKind::Apply { functor, args }
            if functor == &symbol("Functor") && args == &vec![term_id(0), term_id(1)]
    ));
    assert!(matches!(
        &output.terms.get(term_id(3)).expect("select").kind,
        CoreTermKind::Select { selector, base }
            if selector == &symbol("selector") && *base == term_id(2)
    ));
    assert!(matches!(
        &output.terms.get(term_id(4)).expect("tuple").kind,
        CoreTermKind::Tuple(args) if args == &vec![term_id(0), term_id(3)]
    ));
    assert!(matches!(
        &output.terms.get(term_id(5)).expect("set enum").kind,
        CoreTermKind::SetEnum(args) if args == &vec![term_id(1), term_id(4)]
    ));
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn membership_lowering_preserves_remapped_operands_and_rejects_missing_refs() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(
            CoreTermSeedKind::Apply {
                functor: symbol("F"),
                args: vec![CoreTermSeedId::new(1)],
            },
            70,
        ),
        term_seed(CoreTermSeedKind::Var(CoreVarId::new(0)), 71),
    ];
    input.formulas = vec![formula_seed(
        CoreFormulaSeedKind::Membership {
            element: CoreTermSeedId::new(1),
            set: CoreTermSeedId::new(0),
        },
        72,
    )];
    let output = lower_term_and_formula_inputs(&context, input.clone()).unwrap();
    assert_eq!(output.term_map[&CoreTermSeedId::new(0)], CoreTermId::new(1));
    let formula = output.formula_map[&CoreFormulaSeedId::new(0)];
    assert_eq!(
        output.formulas.get(formula).unwrap().kind,
        CoreFormulaKind::Membership {
            element: CoreTermId::new(0),
            set: CoreTermId::new(1)
        }
    );
    let mut refs = DefinitionBodyRefs::default();
    collect_reachable_formula_refs(&output, formula, &mut refs);
    assert_eq!(refs.terms, [CoreTermId::new(0), CoreTermId::new(1)].into());
    assert_step3_delta_valid(&context, &output);
    assert_eq!(
        output.source_map.formula_sources[&formula],
        output.formulas.get(formula).unwrap().source
    );
    for missing_element in [false, true] {
        let mut bad = input.clone();
        bad.formulas[0].kind = CoreFormulaSeedKind::Membership {
            element: CoreTermSeedId::new(if missing_element { 99 } else { 1 }),
            set: CoreTermSeedId::new(if missing_element { 0 } else { 99 }),
        };
        assert!(lower_term_and_formula_inputs(&context, bad).is_err());
    }
    input.owner = CoreItemId::new(99);
    assert!(matches!(
        lower_term_and_formula_inputs(&context, input),
        Err(TermAndFormulaLoweringError::MissingOwnerItem { .. })
    ));
}

#[test]
fn formula_lowering_covers_constants_atoms_connectives_and_type_predicates() {
    let x = CoreVarId::new(0);
    let y = CoreVarId::new(1);
    let (context, owner) = context_with_var_sorts(vec![
        (x, NormalizedVarSort::Term),
        (y, NormalizedVarSort::Term),
    ]);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 70),
        term_seed(CoreTermSeedKind::Var(y), 71),
    ];
    input.formulas = vec![
        formula_seed(CoreFormulaSeedKind::True, 72),
        formula_seed(CoreFormulaSeedKind::False, 73),
        formula_seed(
            CoreFormulaSeedKind::Atom {
                predicate: symbol("Predicate"),
                args: vec![CoreTermSeedId::new(0), CoreTermSeedId::new(1)],
            },
            74,
        ),
        formula_seed(
            CoreFormulaSeedKind::Equals {
                left: CoreTermSeedId::new(0),
                right: CoreTermSeedId::new(1),
            },
            75,
        ),
        formula_seed(
            CoreFormulaSeedKind::TypePred {
                subject: CoreTermSeedId::new(0),
                ty: CoreTypePredicate::new("set"),
            },
            76,
        ),
        formula_seed(CoreFormulaSeedKind::Not(CoreFormulaSeedId::new(1)), 77),
        formula_seed(
            CoreFormulaSeedKind::And(vec![
                CoreFormulaSeedId::new(2),
                CoreFormulaSeedId::new(3),
                CoreFormulaSeedId::new(4),
            ]),
            78,
        ),
        formula_seed(
            CoreFormulaSeedKind::Or(vec![CoreFormulaSeedId::new(0), CoreFormulaSeedId::new(1)]),
            79,
        ),
        formula_seed(
            CoreFormulaSeedKind::Implies {
                premise: CoreFormulaSeedId::new(2),
                conclusion: CoreFormulaSeedId::new(3),
            },
            80,
        ),
        formula_seed(
            CoreFormulaSeedKind::Iff {
                left: CoreFormulaSeedId::new(8),
                right: CoreFormulaSeedId::new(7),
            },
            81,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];
    let formula_id = |seed| output.formula_map[&CoreFormulaSeedId::new(seed)];

    assert!(matches!(
        output.formulas.get(formula_id(0)).expect("true").kind,
        CoreFormulaKind::True
    ));
    assert!(matches!(
        output.formulas.get(formula_id(1)).expect("false").kind,
        CoreFormulaKind::False
    ));
    assert!(matches!(
        &output.formulas.get(formula_id(2)).expect("atom").kind,
        CoreFormulaKind::Atom { predicate, args }
            if predicate == &symbol("Predicate") && args == &vec![term_id(0), term_id(1)]
    ));
    assert!(matches!(
        output.formulas.get(formula_id(3)).expect("equals").kind,
        CoreFormulaKind::Equals { left, right } if left == term_id(0) && right == term_id(1)
    ));
    assert!(matches!(
        &output.formulas.get(formula_id(4)).expect("type pred").kind,
        CoreFormulaKind::TypePred { subject, ty } if *subject == term_id(0) && ty.as_str() == "set"
    ));
    assert!(matches!(
        output.formulas.get(formula_id(5)).expect("not").kind,
        CoreFormulaKind::Not(inner) if inner == formula_id(1)
    ));
    assert!(matches!(
        &output.formulas.get(formula_id(6)).expect("and").kind,
        CoreFormulaKind::And(children)
            if children == &vec![formula_id(2), formula_id(3), formula_id(4)]
    ));
    assert!(matches!(
        &output.formulas.get(formula_id(7)).expect("or").kind,
        CoreFormulaKind::Or(children) if children == &vec![formula_id(0), formula_id(1)]
    ));
    assert!(matches!(
        output.formulas.get(formula_id(8)).expect("implies").kind,
        CoreFormulaKind::Implies { premise, conclusion }
            if premise == formula_id(2) && conclusion == formula_id(3)
    ));
    assert!(matches!(
        output.formulas.get(formula_id(9)).expect("iff").kind,
        CoreFormulaKind::Iff { left, right } if left == formula_id(8) && right == formula_id(7)
    ));
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn quantifier_guards_allow_self_and_prior_binders_but_reject_later_binders() {
    let x = CoreVarId::new(0);
    let y = CoreVarId::new(1);
    let (context, owner) = context_with_var_sorts(vec![
        (x, NormalizedVarSort::Term),
        (y, NormalizedVarSort::Term),
    ]);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.formulas = vec![
        formula_seed(CoreFormulaSeedKind::True, 82),
        formula_seed(
            CoreFormulaSeedKind::Forall {
                binders: vec![
                    QuantifierBinderSeed::new(
                        x,
                        "term-binder",
                        direct(83, 84),
                        provenance("checker:forall:x"),
                    )
                    .with_guard(CoreFormulaSeedId::new(0), vec![x])
                    .with_source_name("x"),
                    QuantifierBinderSeed::new(
                        y,
                        "term-binder",
                        direct(85, 86),
                        provenance("checker:forall:y"),
                    )
                    .with_guard(CoreFormulaSeedId::new(0), vec![x, y])
                    .with_source_name("y"),
                ],
                body: CoreFormulaSeedId::new(0),
            },
            87,
        ),
        formula_seed(
            CoreFormulaSeedKind::Exists {
                binders: vec![
                    QuantifierBinderSeed::new(
                        x,
                        "term-binder",
                        direct(87, 88),
                        provenance("checker:exists:x"),
                    )
                    .with_source_name("x"),
                ],
                body: CoreFormulaSeedId::new(0),
            },
            88,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let forall = output.formula_map[&CoreFormulaSeedId::new(1)];
    let CoreFormulaKind::Forall { binders, body } =
        &output.formulas.get(forall).expect("forall").kind
    else {
        panic!("expected forall");
    };
    assert_eq!(binders.len(), 2);
    assert_eq!(binders[0].source_name.as_deref(), Some("x"));
    assert_eq!(binders[1].source_name.as_deref(), Some("y"));
    assert!(binders.iter().all(|binder| binder.ty_guard.is_some()));
    assert_eq!(*body, output.formula_map[&CoreFormulaSeedId::new(0)]);
    let exists = output.formula_map[&CoreFormulaSeedId::new(2)];
    let CoreFormulaKind::Exists {
        binders: exists_binders,
        body: exists_body,
    } = &output.formulas.get(exists).expect("exists").kind
    else {
        panic!("expected exists");
    };
    assert_eq!(exists_binders.len(), 1);
    assert_eq!(exists_binders[0].var, x);
    assert_eq!(*exists_body, output.formula_map[&CoreFormulaSeedId::new(0)]);
    assert_step3_delta_valid(&context, &output);

    let mut bad_input = TermAndFormulaLoweringInput::new(owner);
    bad_input.terms = vec![term_seed(CoreTermSeedKind::Var(y), 88)];
    bad_input.formulas = vec![
        formula_seed(
            CoreFormulaSeedKind::TypePred {
                subject: CoreTermSeedId::new(0),
                ty: CoreTypePredicate::new("Nat"),
            },
            88,
        ),
        formula_seed(
            CoreFormulaSeedKind::Forall {
                binders: vec![
                    QuantifierBinderSeed::new(
                        x,
                        "term-binder",
                        direct(89, 90),
                        provenance("checker:forall:bad-x"),
                    )
                    .with_guard(CoreFormulaSeedId::new(0), Vec::new()),
                    QuantifierBinderSeed::new(
                        y,
                        "term-binder",
                        direct(91, 92),
                        provenance("checker:forall:bad-y"),
                    ),
                ],
                body: CoreFormulaSeedId::new(0),
            },
            93,
        ),
    ];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, bad_input),
        Err(TermAndFormulaLoweringError::FutureBinderInGuard { binder, later })
            if binder == x && later == y
    ));
}

#[test]
fn qua_terms_reuse_underlying_term_and_record_view_explanation() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 94),
        term_seed(
            CoreTermSeedKind::Qua {
                base: CoreTermSeedId::new(0),
                explanation: ViewExplanationSeed {
                    kind: ViewExplanationKind::SourceQua,
                    inserted_view: None,
                    target_type: Some(NormalizedTypeId::new(12)),
                    reduct: None,
                    evidence_facts: vec![
                        TypeFactId::new(3),
                        TypeFactId::new(2),
                        TypeFactId::new(2),
                    ],
                    source: direct(95, 96),
                    provenance: provenance("checker:term-qua"),
                },
            },
            95,
        ),
        term_seed(
            CoreTermSeedKind::Qua {
                base: CoreTermSeedId::new(0),
                explanation: ViewExplanationSeed {
                    kind: ViewExplanationKind::InsertedView,
                    inserted_view: Some(CoercionInsertionId::new(7)),
                    target_type: Some(NormalizedTypeId::new(13)),
                    reduct: None,
                    evidence_facts: vec![TypeFactId::new(5)],
                    source: direct(96, 97),
                    provenance: provenance("checker:term-inserted-view"),
                },
            },
            96,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");

    assert_eq!(output.terms.len(), 1);
    assert_eq!(
        output.term_map[&CoreTermSeedId::new(1)],
        output.term_map[&CoreTermSeedId::new(0)]
    );
    assert_eq!(
        output.term_map[&CoreTermSeedId::new(2)],
        output.term_map[&CoreTermSeedId::new(0)]
    );
    assert_eq!(output.view_explanations.len(), 2);
    assert_eq!(
        output.view_explanations[0].evidence_facts,
        vec![TypeFactId::new(2), TypeFactId::new(3)]
    );
    assert_eq!(
        output.view_explanations[1].kind,
        ViewExplanationKind::InsertedView
    );
    assert_eq!(
        output.view_explanations[1].inserted_view,
        Some(CoercionInsertionId::new(7))
    );
    assert!(output.view_explanations[0].reduct.is_none());
    assert!(output.view_explanations[1].reduct.is_none());
    assert_eq!(
        output.view_explanations[1].evidence_facts,
        vec![TypeFactId::new(5)]
    );
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn reduct_qua_lowers_renamed_views_to_distinct_terms() {
    let r = CoreVarId::new(0);
    let (context, owner) = context_with_var(r);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(r), 94),
        term_seed(
            CoreTermSeedKind::Qua {
                base: CoreTermSeedId::new(0),
                explanation: source_qua_explanation("Ring>AddMagma", &["view_add_magma"], 20, 95),
            },
            95,
        ),
        term_seed(
            CoreTermSeedKind::Qua {
                base: CoreTermSeedId::new(0),
                explanation: source_qua_explanation("Ring>MulMagma", &["view_mul_magma"], 21, 96),
            },
            96,
        ),
        term_seed(
            CoreTermSeedKind::Select {
                selector: symbol("binop"),
                base: CoreTermSeedId::new(1),
            },
            97,
        ),
        term_seed(
            CoreTermSeedKind::Select {
                selector: symbol("binop"),
                base: CoreTermSeedId::new(2),
            },
            98,
        ),
    ];
    input.formulas = vec![
        formula_seed(
            CoreFormulaSeedKind::Atom {
                predicate: symbol("is_commutative"),
                args: vec![CoreTermSeedId::new(1)],
            },
            99,
        ),
        formula_seed(
            CoreFormulaSeedKind::TypePred {
                subject: CoreTermSeedId::new(2),
                ty: CoreTypePredicate::new("commutative_Magma"),
            },
            100,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];
    let formula_id = |seed| output.formula_map[&CoreFormulaSeedId::new(seed)];
    let base = term_id(0);
    let add_view = term_id(1);
    let mul_view = term_id(2);

    assert_ne!(add_view, base);
    assert_ne!(mul_view, base);
    assert_ne!(add_view, mul_view);
    assert!(matches!(
        &output.terms.get(add_view).expect("add view").kind,
        CoreTermKind::Apply { functor, args }
            if functor == &symbol("view_add_magma") && args == &vec![base]
    ));
    assert!(matches!(
        &output.terms.get(mul_view).expect("mul view").kind,
        CoreTermKind::Apply { functor, args }
            if functor == &symbol("view_mul_magma") && args == &vec![base]
    ));
    assert!(matches!(
        &output.terms.get(term_id(3)).expect("add binop").kind,
        CoreTermKind::Select { selector, base }
            if selector == &symbol("binop") && *base == add_view
    ));
    assert!(matches!(
        &output.terms.get(term_id(4)).expect("mul binop").kind,
        CoreTermKind::Select { selector, base }
            if selector == &symbol("binop") && *base == mul_view
    ));
    assert!(matches!(
        &output.formulas.get(formula_id(0)).expect("attribute").kind,
        CoreFormulaKind::Atom { predicate, args }
            if predicate == &symbol("is_commutative") && args == &vec![add_view]
    ));
    assert!(matches!(
        &output.formulas.get(formula_id(1)).expect("type pred").kind,
        CoreFormulaKind::TypePred { subject, ty }
            if *subject == mul_view && ty == &CoreTypePredicate::new("commutative_Magma")
    ));
    assert!(
        output
            .formulas
            .iter()
            .all(|(_, formula)| !matches!(formula.kind, CoreFormulaKind::Equals { .. }))
    );
    assert_eq!(
        output.view_explanations[0]
            .reduct
            .as_ref()
            .expect("add reduct")
            .path
            .as_str(),
        "Ring>AddMagma"
    );
    assert_eq!(
        output.view_explanations[0]
            .reduct
            .as_ref()
            .expect("add reduct")
            .functors,
        vec![symbol("view_add_magma")]
    );
    assert_eq!(
        output.view_explanations[1]
            .reduct
            .as_ref()
            .expect("mul reduct")
            .path
            .as_str(),
        "Ring>MulMagma"
    );
    assert_eq!(
        output.view_explanations[1]
            .reduct
            .as_ref()
            .expect("mul reduct")
            .functors,
        vec![symbol("view_mul_magma")]
    );
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn composed_reduct_view_lowers_nested_and_template_bounds_use_final_view() {
    let r = CoreVarId::new(0);
    let (context, owner) = context_with_var(r);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(r), 101),
        term_seed(
            CoreTermSeedKind::Qua {
                base: CoreTermSeedId::new(0),
                explanation: source_qua_explanation(
                    "Ring>AddGroup>Magma",
                    &["z_view_add_group", "a_view_group_magma"],
                    22,
                    102,
                ),
            },
            102,
        ),
        term_seed(
            CoreTermSeedKind::Select {
                selector: symbol("binop"),
                base: CoreTermSeedId::new(1),
            },
            103,
        ),
    ];
    input.formulas = vec![formula_seed(
        CoreFormulaSeedKind::TypePred {
            subject: CoreTermSeedId::new(1),
            ty: CoreTypePredicate::new("template_bound_commutative"),
        },
        104,
    )];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];
    let formula_id = |seed| output.formula_map[&CoreFormulaSeedId::new(seed)];
    let base = term_id(0);
    let final_view = term_id(1);
    let CoreTermKind::Apply {
        functor: final_functor,
        args: final_args,
    } = &output.terms.get(final_view).expect("final view").kind
    else {
        panic!("expected final view apply");
    };
    assert_eq!(final_functor, &symbol("a_view_group_magma"));
    let [intermediate] = final_args.as_slice() else {
        panic!("expected unary final view");
    };
    assert!(matches!(
        &output.terms.get(*intermediate).expect("intermediate view").kind,
        CoreTermKind::Apply { functor, args }
            if functor == &symbol("z_view_add_group") && args == &vec![base]
    ));
    assert!(matches!(
        &output.terms.get(term_id(2)).expect("selected field").kind,
        CoreTermKind::Select { selector, base }
            if selector == &symbol("binop") && *base == final_view
    ));
    assert!(matches!(
        &output.formulas.get(formula_id(0)).expect("template bound").kind,
        CoreFormulaKind::TypePred { subject, ty }
            if *subject == final_view
                && ty == &CoreTypePredicate::new("template_bound_commutative")
    ));
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn exact_instance_extensionality_guard_is_preserved_on_reduct_term() {
    let r = CoreVarId::new(0);
    let (context, owner) = context_with_var(r);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(r), 105),
        term_seed(
            CoreTermSeedKind::Qua {
                base: CoreTermSeedId::new(0),
                explanation: source_qua_explanation("Ring>AddMagma", &["view_add_magma"], 23, 106),
            },
            106,
        ),
    ];
    input.formulas = vec![
        formula_seed(
            CoreFormulaSeedKind::TypePred {
                subject: CoreTermSeedId::new(1),
                ty: CoreTypePredicate::new("exact_Magma"),
            },
            107,
        ),
        formula_seed(
            CoreFormulaSeedKind::Atom {
                predicate: symbol("magma_field_extensionality"),
                args: vec![CoreTermSeedId::new(1)],
            },
            108,
        ),
        formula_seed(
            CoreFormulaSeedKind::Implies {
                premise: CoreFormulaSeedId::new(0),
                conclusion: CoreFormulaSeedId::new(1),
            },
            109,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let term_id = |seed| output.term_map[&CoreTermSeedId::new(seed)];
    let formula_id = |seed| output.formula_map[&CoreFormulaSeedId::new(seed)];
    let view = term_id(1);

    assert_eq!(output.formulas.len(), 3);
    assert!(matches!(
        &output.formulas.get(formula_id(0)).expect("exact guard").kind,
        CoreFormulaKind::TypePred { subject, ty }
            if *subject == view && ty == &CoreTypePredicate::new("exact_Magma")
    ));
    assert!(matches!(
        &output.formulas.get(formula_id(1)).expect("extensionality atom").kind,
        CoreFormulaKind::Atom { predicate, args }
            if predicate == &symbol("magma_field_extensionality") && args == &vec![view]
    ));
    assert!(matches!(
        &output.formulas.get(formula_id(2)).expect("guarded formula").kind,
        CoreFormulaKind::Implies { premise, conclusion }
            if *premise == formula_id(0) && *conclusion == formula_id(1)
    ));
    assert!(output.formulas.iter().all(|(_, formula)| {
        !matches!(
            &formula.kind,
            CoreFormulaKind::TypePred { ty, .. } if ty == &CoreTypePredicate::new("is_Magma")
        )
    }));
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn reduct_view_payload_requires_explicit_functors() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let empty_reduct = reduct_view("Ring>Magma", &[]);
    let mut type_input = TypeAndFactLoweringInput::new(owner);
    type_input.view_explanations = vec![ViewExplanationSeed {
        kind: ViewExplanationKind::SourceQua,
        inserted_view: None,
        target_type: Some(NormalizedTypeId::new(24)),
        reduct: Some(empty_reduct.clone()),
        evidence_facts: Vec::new(),
        source: direct(110, 111),
        provenance: provenance("checker:empty-reduct-step2"),
    }];
    assert!(matches!(
        lower_type_and_fact_inputs(&context, type_input),
        Err(TypeAndFactLoweringError::EmptyReductViewPayload { path })
            if path.as_str() == "Ring>Magma"
    ));

    let mut term_input = TermAndFormulaLoweringInput::new(owner);
    term_input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 112),
        term_seed(
            CoreTermSeedKind::Qua {
                base: CoreTermSeedId::new(0),
                explanation: ViewExplanationSeed {
                    kind: ViewExplanationKind::SourceQua,
                    inserted_view: None,
                    target_type: Some(NormalizedTypeId::new(25)),
                    reduct: Some(empty_reduct),
                    evidence_facts: Vec::new(),
                    source: direct(113, 114),
                    provenance: provenance("checker:empty-reduct-step3"),
                },
            },
            113,
        ),
    ];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, term_input),
        Err(TermAndFormulaLoweringError::EmptyReductViewPayload { path })
            if path.as_str() == "Ring>Magma"
    ));
}

#[test]
fn stable_choice_reuses_existing_and_delta_generated_origin_keys() {
    let x = CoreVarId::new(0);
    let existing_key = GeneratedOriginKey::new("choice:existing");
    let new_key = GeneratedOriginKey::new("choice:new");
    let (context, owner, existing_origin) =
        context_with_existing_choice_origin(x, existing_key.clone());
    let existing_generated_len = context.generated_origins().table().len();
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 96),
        term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("choice_existing"),
                origin_functor: symbol("choice_existing"),
                key: existing_key.clone(),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:choice:use-existing-1",
                )],
            },
            97,
        ),
        term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("choice_existing"),
                origin_functor: symbol("choice_existing"),
                key: existing_key,
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:choice:use-existing-2",
                )],
            },
            98,
        ),
        term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("choice_new"),
                origin_functor: symbol("choice_new"),
                key: new_key.clone(),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:choice:new-1",
                )],
            },
            99,
        ),
        term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("choice_new"),
                origin_functor: symbol("choice_new"),
                key: new_key.clone(),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:choice:new-2",
                )],
            },
            100,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");

    assert_eq!(output.generated.len(), existing_generated_len + 1);
    assert_eq!(output.generated_delta.len(), 1);
    assert_eq!(output.new_generated_origins.len(), 1);
    let (_, delta_origin) = output
        .generated_delta
        .iter()
        .next()
        .expect("single generated delta");
    assert_eq!(delta_origin.kind, GeneratedOriginKind::StableChoice);
    assert_eq!(delta_origin.key, new_key);
    assert_eq!(delta_origin.params, vec![x]);
    assert_eq!(output.generated_origin_refs.len(), 4);
    assert_eq!(output.generated_origin_refs[0].origin, existing_origin);
    assert!(output.generated_origin_refs[0].reused_existing);
    assert_eq!(
        output.generated_origin_refs[0].reuse_source,
        GeneratedOriginReuseSource::ExistingRegistry
    );
    assert_eq!(output.generated_origin_refs[1].origin, existing_origin);
    assert!(output.generated_origin_refs[1].reused_existing);
    assert_eq!(
        output.generated_origin_refs[1].reuse_source,
        GeneratedOriginReuseSource::ExistingRegistry
    );
    let new_origin = output.generated_origin_refs[2].origin;
    assert_ne!(new_origin, existing_origin);
    assert!(!output.generated_origin_refs[2].reused_existing);
    assert_eq!(
        output.generated_origin_refs[2].reuse_source,
        GeneratedOriginReuseSource::NewDelta
    );
    assert_eq!(output.new_generated_origins, vec![new_origin]);
    assert_eq!(output.generated_origin_refs[3].origin, new_origin);
    assert!(output.generated_origin_refs[3].reused_existing);
    assert_eq!(
        output.generated_origin_refs[3].reuse_source,
        GeneratedOriginReuseSource::CurrentDelta
    );
    for (seed, expected_functor, expected_key) in [
        (1, symbol("choice_existing"), "choice:existing"),
        (2, symbol("choice_existing"), "choice:existing"),
        (3, symbol("choice_new"), "choice:new"),
        (4, symbol("choice_new"), "choice:new"),
    ] {
        let CoreTermKind::Apply { functor, args } = &output
            .terms
            .get(output.term_map[&CoreTermSeedId::new(seed)])
            .expect("choice term")
            .kind
        else {
            panic!("expected stable choice apply");
        };
        assert_eq!(functor, &expected_functor);
        assert_eq!(args, &vec![output.term_map[&CoreTermSeedId::new(0)]]);
        let use_record = &output.generated_origin_refs[seed - 1];
        assert_eq!(use_record.term, output.term_map[&CoreTermSeedId::new(seed)]);
        assert_eq!(use_record.key.as_str(), expected_key);
        assert_eq!(use_record.functor, expected_functor);
        assert_eq!(
            use_record.args,
            vec![output.term_map[&CoreTermSeedId::new(0)]]
        );
    }
    assert_step3_delta_valid(&context, &output);

    let mut mismatch_input = TermAndFormulaLoweringInput::new(owner);
    mismatch_input.terms = vec![term_seed(
        CoreTermSeedKind::StableChoice {
            functor: symbol("choice_existing"),
            origin_functor: symbol("choice_existing"),
            key: GeneratedOriginKey::new("choice:existing"),
            params: Vec::new(),
            args: Vec::new(),
            evidence: vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "checker:choice:param-mismatch",
            )],
        },
        100,
    )];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, mismatch_input),
        Err(TermAndFormulaLoweringError::GeneratedOriginParameterMismatch { origin, key })
            if origin == existing_origin && key.as_str() == "choice:existing"
    ));

    let mut functor_mismatch = TermAndFormulaLoweringInput::new(owner);
    functor_mismatch.terms = vec![term_seed(
        CoreTermSeedKind::StableChoice {
            functor: symbol("ordinary_functor"),
            origin_functor: symbol("choice_existing"),
            key: GeneratedOriginKey::new("choice:existing"),
            params: vec![x],
            args: Vec::new(),
            evidence: vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "checker:choice:functor-mismatch",
            )],
        },
        100,
    )];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, functor_mismatch),
        Err(TermAndFormulaLoweringError::GeneratedFunctorMismatch { key, .. })
            if key.as_str() == "choice:existing"
    ));

    let mut registry_functor_mismatch = TermAndFormulaLoweringInput::new(owner);
    registry_functor_mismatch.terms = vec![term_seed(
        CoreTermSeedKind::StableChoice {
            functor: symbol("ordinary_functor"),
            origin_functor: symbol("ordinary_functor"),
            key: GeneratedOriginKey::new("choice:existing"),
            params: vec![x],
            args: Vec::new(),
            evidence: vec![CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "checker:choice:registry-functor-mismatch",
            )],
        },
        101,
    )];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, registry_functor_mismatch),
        Err(TermAndFormulaLoweringError::GeneratedFunctorMismatch { key, expected, actual })
            if key.as_str() == "choice:existing"
                && expected.as_ref() == &symbol("choice_existing")
                && actual.as_ref() == &symbol("ordinary_functor")
    ));
}

#[test]
fn fraenkel_lowering_preserves_evidence_and_membership_obligation() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 101),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("fraenkel_set"),
                origin_functor: symbol("fraenkel_set"),
                key: GeneratedOriginKey::new("fraenkel:mapper:predicate"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:fraenkel:sethood",
                )],
                template_type_parameter_sethood: None,
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::New(
                    CoreObligationSeed::active(
                        ObligationSeedKind::FraenkelMembershipAxiom,
                        CoreFormulaSeedId::new(0),
                        "fraenkel/membership",
                        "pkg::main::Owner.fraenkel.membership",
                        direct(103, 104),
                        provenance("checker:fraenkel:obligation"),
                    ),
                )),
                missing_sethood_obligation: None,
            },
            102,
        ),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("fraenkel_set"),
                origin_functor: symbol("fraenkel_set"),
                key: GeneratedOriginKey::new("fraenkel:mapper:predicate"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:fraenkel:sethood:reuse",
                )],
                template_type_parameter_sethood: None,
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::AlreadyCarried(
                    AlreadyCarriedFraenkelMembershipSeed {
                        source: direct(104, 105),
                        provenance: provenance("checker:fraenkel:already-carried"),
                    },
                )),
                missing_sethood_obligation: None,
            },
            104,
        ),
    ];
    input.formulas = vec![formula_seed(CoreFormulaSeedKind::True, 103)];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let fraenkel_term = output.term_map[&CoreTermSeedId::new(1)];
    let generated_ref = &output.generated_origin_refs[0];
    let generated = output
        .generated
        .get(generated_ref.origin)
        .expect("fraenkel origin");
    let reused_generated_ref = &output.generated_origin_refs[1];
    let obligation = output
        .obligation_seeds
        .get(output.generated_obligations[0].obligation)
        .expect("fraenkel obligation");

    assert!(matches!(
        output.terms.get(fraenkel_term).expect("fraenkel").kind,
        CoreTermKind::Apply { .. }
    ));
    assert_eq!(output.generated_delta.len(), 1);
    assert_eq!(output.generated_origin_refs.len(), 2);
    assert_eq!(
        generated_ref.reuse_source,
        GeneratedOriginReuseSource::NewDelta
    );
    assert_eq!(
        reused_generated_ref.reuse_source,
        GeneratedOriginReuseSource::CurrentDelta
    );
    assert_eq!(reused_generated_ref.origin, generated_ref.origin);
    assert_eq!(
        reused_generated_ref.term,
        output.term_map[&CoreTermSeedId::new(2)]
    );
    assert_eq!(reused_generated_ref.functor, symbol("fraenkel_set"));
    assert_eq!(
        reused_generated_ref.args,
        vec![output.term_map[&CoreTermSeedId::new(0)]]
    );
    assert_eq!(generated.kind, GeneratedOriginKind::FraenkelComprehension);
    assert!(generated.evidence.iter().any(|entry| {
        entry.phase == CoreProvenancePhase::Checker
            && entry.key.as_str() == "checker:fraenkel:sethood"
    }));
    assert_eq!(
        output.generated_obligations[0].kind,
        ObligationSeedKind::FraenkelMembershipAxiom
    );
    assert_eq!(obligation.status, ObligationSeedStatus::Active);
    assert!(obligation.goal.is_some());
    assert_eq!(output.already_carried_generated_obligations.len(), 1);
    assert_eq!(
        output.already_carried_generated_obligations[0].origin,
        generated_ref.origin
    );
    assert_step3_delta_valid(&context, &output);

    let mut bad_membership = TermAndFormulaLoweringInput::new(owner);
    bad_membership.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 107),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("fraenkel_set"),
                origin_functor: symbol("fraenkel_set"),
                key: GeneratedOriginKey::new("fraenkel:bad-membership"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:fraenkel:bad:sethood",
                )],
                template_type_parameter_sethood: None,
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::New(
                    CoreObligationSeed::active(
                        ObligationSeedKind::GeneratedSethood,
                        CoreFormulaSeedId::new(0),
                        "fraenkel/bad-membership",
                        "pkg::main::Owner.fraenkel.bad-membership",
                        direct(108, 109),
                        provenance("checker:fraenkel:bad-membership"),
                    ),
                )),
                missing_sethood_obligation: None,
            },
            108,
        ),
    ];
    bad_membership.formulas = vec![formula_seed(CoreFormulaSeedKind::True, 109)];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, bad_membership),
        Err(TermAndFormulaLoweringError::InvalidFraenkelMembershipObligation {
            kind,
            status: ObligationSeedStatus::Active,
        }) if kind == ObligationSeedKind::GeneratedSethood
    ));

    let mut deferred_membership = TermAndFormulaLoweringInput::new(owner);
    deferred_membership.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 110),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("fraenkel_set"),
                origin_functor: symbol("fraenkel_set"),
                key: GeneratedOriginKey::new("fraenkel:deferred-membership"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:fraenkel:deferred:sethood",
                )],
                template_type_parameter_sethood: None,
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::New(
                    CoreObligationSeed::deferred(
                        ObligationSeedKind::FraenkelMembershipAxiom,
                        "fraenkel/deferred-membership",
                        "pkg::main::Owner.fraenkel.deferred-membership",
                        direct(111, 112),
                        provenance("checker:fraenkel:deferred-membership"),
                    ),
                )),
                missing_sethood_obligation: None,
            },
            111,
        ),
    ];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, deferred_membership),
        Err(TermAndFormulaLoweringError::InvalidFraenkelMembershipObligation {
            kind,
            status: ObligationSeedStatus::Deferred,
        }) if kind == ObligationSeedKind::FraenkelMembershipAxiom
    ));
}

#[test]
fn ordinary_fraenkel_sethood_evidence_does_not_need_template_record() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let degraded = template_sethood_seed(
        "T",
        "T:degraded-unused",
        TemplateTypeParameterSethoodSource::BoundInherited,
        TemplateTypeParameterSethoodStatus::DegradedRecovery,
        199,
    );
    let mut type_input = TypeAndFactLoweringInput::new(owner);
    type_input.template_type_parameter_sethoods = vec![degraded];
    let type_output = lower_type_and_fact_inputs(&context, type_input).expect("type lowering");

    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.template_type_parameter_sethoods = type_output.template_type_parameter_sethoods.clone();
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 200),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("ordinary_fraenkel"),
                origin_functor: symbol("ordinary_fraenkel"),
                key: GeneratedOriginKey::new("fraenkel:ordinary"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:fraenkel:ordinary-sethood",
                )],
                template_type_parameter_sethood: None,
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::AlreadyCarried(
                    AlreadyCarriedFraenkelMembershipSeed {
                        source: direct(201, 202),
                        provenance: provenance("checker:fraenkel:ordinary-carried"),
                    },
                )),
                missing_sethood_obligation: None,
            },
            201,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");

    assert_eq!(output.generated_origin_refs.len(), 1);
    assert!(matches!(
        output
            .terms
            .get(output.term_map[&CoreTermSeedId::new(1)])
            .expect("fraenkel")
            .kind,
        CoreTermKind::Apply { .. }
    ));
    let generated = output
        .generated
        .get(output.generated_origin_refs[0].origin)
        .expect("ordinary fraenkel origin");
    assert!(generated.evidence.iter().any(|entry| {
        entry.phase == CoreProvenancePhase::Checker
            && entry.key.as_str() == "checker:fraenkel:ordinary-sethood"
    }));
    assert!(!generated.evidence.iter().any(|entry| {
        entry.phase == CoreProvenancePhase::Checker
            && entry.key.as_str() == "checker:template-sethood:T"
    }));
    assert!(output.diagnostics.is_empty());
    assert!(output.generated_obligations.is_empty());
    assert!(output.obligation_seeds.is_empty());
    assert_step2_delta_valid(&context, &type_output);
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn fraenkel_template_type_parameter_sethood_records_gate_generated_origins() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let mut bound = template_sethood_seed(
        "T",
        "T:bound:Nat",
        TemplateTypeParameterSethoodSource::BoundInherited,
        TemplateTypeParameterSethoodStatus::Accepted,
        210,
    );
    bound.facts = vec![TypeFactId::new(20)];
    let mut constraint = template_sethood_seed(
        "U",
        "U:constraint:sethood",
        TemplateTypeParameterSethoodSource::ConstraintSupplied,
        TemplateTypeParameterSethoodStatus::Accepted,
        211,
    );
    constraint.facts = vec![TypeFactId::new(21)];
    let mut type_input = TypeAndFactLoweringInput::new(owner);
    type_input.template_type_parameter_sethoods = vec![bound, constraint];
    let type_output = lower_type_and_fact_inputs(&context, type_input).expect("type lowering");

    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.template_type_parameter_sethoods = type_output.template_type_parameter_sethoods.clone();
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 212),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("template_fraenkel_T"),
                origin_functor: symbol("template_fraenkel_T"),
                key: GeneratedOriginKey::new("fraenkel:template:T"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: Vec::new(),
                template_type_parameter_sethood: Some(template_fraenkel_sethood(
                    "T",
                    "T:bound:Nat",
                    210,
                    213,
                )),
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::New(
                    CoreObligationSeed::active(
                        ObligationSeedKind::FraenkelMembershipAxiom,
                        CoreFormulaSeedId::new(0),
                        "fraenkel/template/T-membership",
                        "pkg::main::Owner.fraenkel.template.T-membership",
                        direct(214, 215),
                        provenance("checker:fraenkel:template:T-membership"),
                    ),
                )),
                missing_sethood_obligation: None,
            },
            213,
        ),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("template_fraenkel_U"),
                origin_functor: symbol("template_fraenkel_U"),
                key: GeneratedOriginKey::new("fraenkel:template:U"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: Vec::new(),
                template_type_parameter_sethood: Some(template_fraenkel_sethood(
                    "U",
                    "U:constraint:sethood",
                    211,
                    216,
                )),
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::AlreadyCarried(
                    AlreadyCarriedFraenkelMembershipSeed {
                        source: direct(217, 218),
                        provenance: provenance("checker:fraenkel:template:U-carried"),
                    },
                )),
                missing_sethood_obligation: None,
            },
            216,
        ),
    ];
    input.formulas = vec![formula_seed(CoreFormulaSeedKind::True, 215)];

    let output = lower_term_and_formula_inputs(&context, input).expect("term lowering");

    assert!(output.diagnostics.is_empty());
    assert_eq!(output.generated_origin_refs.len(), 2);
    assert_eq!(output.generated_obligations.len(), 1);
    assert_eq!(output.already_carried_generated_obligations.len(), 1);
    for (index, expected_template_key, expected_fraenkel_key) in [
        (
            0,
            "checker:template-sethood:T",
            "checker:fraenkel-template-sethood:213",
        ),
        (
            1,
            "checker:template-sethood:U",
            "checker:fraenkel-template-sethood:216",
        ),
    ] {
        let generated = output
            .generated
            .get(output.generated_origin_refs[index].origin)
            .expect("template fraenkel origin");
        assert_eq!(generated.kind, GeneratedOriginKind::FraenkelComprehension);
        assert!(generated.evidence.iter().any(|entry| {
            entry.phase == CoreProvenancePhase::Checker
                && entry.key.as_str() == expected_template_key
        }));
        assert!(generated.evidence.iter().any(|entry| {
            entry.phase == CoreProvenancePhase::Checker
                && entry.key.as_str() == expected_fraenkel_key
        }));
    }
    assert_step2_delta_valid(&context, &type_output);
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn fraenkel_bare_template_type_parameter_remains_missing_sethood_error() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let bare = template_sethood_seed(
        "Bare",
        "Bare:bare",
        TemplateTypeParameterSethoodSource::BareParameter,
        TemplateTypeParameterSethoodStatus::Missing,
        220,
    );
    let mut type_input = TypeAndFactLoweringInput::new(owner);
    type_input.template_type_parameter_sethoods = vec![bare];
    let type_output = lower_type_and_fact_inputs(&context, type_input).expect("type lowering");

    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.template_type_parameter_sethoods = type_output.template_type_parameter_sethoods.clone();
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 221),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("bare_template_fraenkel"),
                origin_functor: symbol("bare_template_fraenkel"),
                key: GeneratedOriginKey::new("fraenkel:template:bare"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:fraenkel:bare-template-raw",
                )],
                template_type_parameter_sethood: Some(template_fraenkel_sethood(
                    "Bare",
                    "Bare:bare",
                    220,
                    222,
                )),
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::AlreadyCarried(
                    AlreadyCarriedFraenkelMembershipSeed {
                        source: direct(223, 224),
                        provenance: provenance("checker:fraenkel:bare-carried"),
                    },
                )),
                missing_sethood_obligation: Some(Box::new(CoreObligationSeed::deferred(
                    ObligationSeedKind::GeneratedSethood,
                    "fraenkel/template/bare-missing-sethood",
                    "pkg::main::Owner.fraenkel.template.bare-missing-sethood",
                    direct(224, 225),
                    provenance("checker:fraenkel:bare-missing-sethood"),
                ))),
            },
            222,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("term lowering");
    let CoreTermKind::Error(diagnostic_id) = output
        .terms
        .get(output.term_map[&CoreTermSeedId::new(1)])
        .expect("bare template fraenkel")
        .kind
    else {
        panic!("expected error term");
    };
    let diagnostic = output.diagnostics.get(diagnostic_id).expect("diagnostic");
    let obligation = output
        .obligation_seeds
        .get(output.generated_obligations[0].obligation)
        .expect("generated sethood obligation");

    assert_eq!(
        diagnostic.message_key.as_str(),
        "missing-fraenkel-sethood-evidence"
    );
    assert!(output.generated_origin_refs.is_empty());
    assert_eq!(obligation.kind, ObligationSeedKind::GeneratedSethood);
    assert_eq!(obligation.status, ObligationSeedStatus::Deferred);
    assert_step2_delta_valid(&context, &type_output);
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn fraenkel_template_sethood_cross_references_fail_closed() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let mut seed = template_sethood_seed(
        "T",
        "T:bound:Nat",
        TemplateTypeParameterSethoodSource::BoundInherited,
        TemplateTypeParameterSethoodStatus::Accepted,
        230,
    );
    seed.facts = vec![TypeFactId::new(30)];
    let mut type_input = TypeAndFactLoweringInput::new(owner);
    type_input.template_type_parameter_sethoods = vec![seed];
    let type_output = lower_type_and_fact_inputs(&context, type_input).expect("type lowering");
    let record = type_output.template_type_parameter_sethoods[0].clone();

    let mut duplicate = TermAndFormulaLoweringInput::new(owner);
    duplicate.template_type_parameter_sethoods = vec![record.clone(), record.clone()];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, duplicate),
        Err(
            TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                reason: TemplateSethoodRecordErrorKind::Duplicate,
                ..
            }
        )
    ));

    let mut missing_record = TermAndFormulaLoweringInput::new(owner);
    missing_record.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 231),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("missing_template_sethood"),
                origin_functor: symbol("missing_template_sethood"),
                key: GeneratedOriginKey::new("fraenkel:template:missing-record"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: Vec::new(),
                template_type_parameter_sethood: Some(template_fraenkel_sethood(
                    "T",
                    "T:bound:Nat",
                    230,
                    232,
                )),
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::AlreadyCarried(
                    AlreadyCarriedFraenkelMembershipSeed {
                        source: direct(233, 234),
                        provenance: provenance("checker:fraenkel:missing-record-carried"),
                    },
                )),
                missing_sethood_obligation: None,
            },
            232,
        ),
    ];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, missing_record),
        Err(
            TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                reason: TemplateSethoodRecordErrorKind::Missing,
                ..
            }
        )
    ));

    let mut normalized_mismatch = TermAndFormulaLoweringInput::new(owner);
    normalized_mismatch.template_type_parameter_sethoods = vec![record.clone()];
    normalized_mismatch.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 234),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("mismatched_template_sethood"),
                origin_functor: symbol("mismatched_template_sethood"),
                key: GeneratedOriginKey::new("fraenkel:template:normalized-mismatch"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: Vec::new(),
                template_type_parameter_sethood: Some(template_fraenkel_sethood(
                    "T",
                    "T:bound:Nat",
                    231,
                    235,
                )),
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::AlreadyCarried(
                    AlreadyCarriedFraenkelMembershipSeed {
                        source: direct(236, 237),
                        provenance: provenance("checker:fraenkel:normalized-mismatch-carried"),
                    },
                )),
                missing_sethood_obligation: None,
            },
            235,
        ),
    ];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, normalized_mismatch),
        Err(
            TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                reason: TemplateSethoodRecordErrorKind::NormalizedTypeMismatch,
                ..
            }
        )
    ));

    let referenced_malformed_input = |record: TemplateTypeParameterSethood, start: usize| {
        let mut input = TermAndFormulaLoweringInput::new(owner);
        input.template_type_parameter_sethoods = vec![record];
        input.terms = vec![
            term_seed(CoreTermSeedKind::Var(x), start),
            term_seed(
                CoreTermSeedKind::Fraenkel {
                    functor: symbol("malformed_template_sethood"),
                    origin_functor: symbol("malformed_template_sethood"),
                    key: GeneratedOriginKey::new(format!("fraenkel:template:malformed:{start}")),
                    params: vec![x],
                    args: vec![CoreTermSeedId::new(0)],
                    sethood_evidence: Vec::new(),
                    template_type_parameter_sethood: Some(template_fraenkel_sethood(
                        "T",
                        "T:bound:Nat",
                        230,
                        start + 1,
                    )),
                    membership_obligation: Box::new(
                        FraenkelMembershipObligationSeed::AlreadyCarried(
                            AlreadyCarriedFraenkelMembershipSeed {
                                source: direct(start + 2, start + 3),
                                provenance: provenance(
                                    format!("checker:fraenkel:malformed-carried:{start}").as_str(),
                                ),
                            },
                        ),
                    ),
                    missing_sethood_obligation: None,
                },
                start + 1,
            ),
        ];
        input
    };

    let mut degraded = record.clone();
    degraded.status = TemplateTypeParameterSethoodStatus::DegradedRecovery;
    degraded.facts.clear();
    let wrong_status = referenced_malformed_input(degraded, 238);
    assert!(matches!(
        lower_term_and_formula_inputs(&context, wrong_status),
        Err(
            TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                reason: TemplateSethoodRecordErrorKind::NotAccepted,
                ..
            }
        )
    ));

    let mut accepted_bare = record.clone();
    accepted_bare.source_kind = TemplateTypeParameterSethoodSource::BareParameter;
    let wrong_source = referenced_malformed_input(accepted_bare, 242);
    assert!(matches!(
        lower_term_and_formula_inputs(&context, wrong_source),
        Err(
            TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                reason: TemplateSethoodRecordErrorKind::WrongSource,
                ..
            }
        )
    ));

    let mut missing_evidence = record.clone();
    missing_evidence.facts.clear();
    let missing_evidence_input = referenced_malformed_input(missing_evidence, 246);
    assert!(matches!(
        lower_term_and_formula_inputs(&context, missing_evidence_input),
        Err(
            TermAndFormulaLoweringError::InvalidTemplateFraenkelSethoodEvidence {
                reason: TemplateSethoodRecordErrorKind::MissingEvidence,
                ..
            }
        )
    ));

    let mut generated_provenance = record.clone();
    generated_provenance.provenance = vec![CoreProvenance::new(
        CoreProvenancePhase::Generated,
        "generated:template-sethood",
    )];
    let generated_provenance_input = referenced_malformed_input(generated_provenance, 250);
    assert!(matches!(
        lower_term_and_formula_inputs(&context, generated_provenance_input),
        Err(TermAndFormulaLoweringError::InvalidSeedProvenance(_))
    ));

    let mut bad_evidence_seed = template_fraenkel_sethood("T", "T:bound:Nat", 230, 254);
    bad_evidence_seed.provenance = CheckerOwnedProvenance {
        entries: vec![CoreProvenance::new(
            CoreProvenancePhase::Generated,
            "generated:fraenkel-template-sethood",
        )],
    };
    let mut generated_evidence_seed_input = TermAndFormulaLoweringInput::new(owner);
    generated_evidence_seed_input.template_type_parameter_sethoods = vec![record.clone()];
    generated_evidence_seed_input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 253),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("bad_template_evidence_seed"),
                origin_functor: symbol("bad_template_evidence_seed"),
                key: GeneratedOriginKey::new("fraenkel:template:bad-evidence-seed"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: Vec::new(),
                template_type_parameter_sethood: Some(bad_evidence_seed),
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::AlreadyCarried(
                    AlreadyCarriedFraenkelMembershipSeed {
                        source: direct(255, 256),
                        provenance: provenance("checker:fraenkel:bad-evidence-seed-carried"),
                    },
                )),
                missing_sethood_obligation: None,
            },
            254,
        ),
    ];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, generated_evidence_seed_input),
        Err(TermAndFormulaLoweringError::InvalidSeedProvenance(_))
    ));
}

#[test]
fn fraenkel_missing_sethood_remains_error_and_deferred_seed() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 104),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("fraenkel_missing"),
                origin_functor: symbol("fraenkel_missing"),
                key: GeneratedOriginKey::new("fraenkel:missing"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: Vec::new(),
                template_type_parameter_sethood: None,
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::AlreadyCarried(
                    AlreadyCarriedFraenkelMembershipSeed {
                        source: direct(105, 106),
                        provenance: provenance("checker:fraenkel:already-carried"),
                    },
                )),
                missing_sethood_obligation: Some(Box::new(CoreObligationSeed::deferred(
                    ObligationSeedKind::GeneratedSethood,
                    "fraenkel/missing-sethood",
                    "pkg::main::Owner.fraenkel.missing-sethood",
                    direct(106, 107),
                    provenance("checker:fraenkel:missing-sethood"),
                ))),
            },
            105,
        ),
    ];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let fraenkel_term = output.term_map[&CoreTermSeedId::new(1)];
    let CoreTermKind::Error(diagnostic_id) =
        output.terms.get(fraenkel_term).expect("error term").kind
    else {
        panic!("expected error term");
    };
    let diagnostic = output.diagnostics.get(diagnostic_id).expect("diagnostic");
    let obligation = output
        .obligation_seeds
        .get(output.generated_obligations[0].obligation)
        .expect("deferred obligation");

    assert_eq!(
        diagnostic.message_key.as_str(),
        "missing-fraenkel-sethood-evidence"
    );
    assert!(output.generated_origin_refs.is_empty());
    assert_eq!(obligation.status, ObligationSeedStatus::Deferred);
    assert_eq!(obligation.kind, ObligationSeedKind::GeneratedSethood);
    assert_step3_delta_valid(&context, &output);

    let mut bad_missing = TermAndFormulaLoweringInput::new(owner);
    bad_missing.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 109),
        term_seed(
            CoreTermSeedKind::Fraenkel {
                functor: symbol("fraenkel_missing"),
                origin_functor: symbol("fraenkel_missing"),
                key: GeneratedOriginKey::new("fraenkel:bad-missing"),
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                sethood_evidence: Vec::new(),
                template_type_parameter_sethood: None,
                membership_obligation: Box::new(FraenkelMembershipObligationSeed::AlreadyCarried(
                    AlreadyCarriedFraenkelMembershipSeed {
                        source: direct(109, 110),
                        provenance: provenance("checker:fraenkel:already-carried:bad"),
                    },
                )),
                missing_sethood_obligation: Some(Box::new(CoreObligationSeed::deferred(
                    ObligationSeedKind::DefinitionCorrectness,
                    "fraenkel/bad-missing-sethood",
                    "pkg::main::Owner.fraenkel.bad-missing-sethood",
                    direct(110, 111),
                    provenance("checker:fraenkel:bad-missing-sethood"),
                ))),
            },
            110,
        ),
    ];
    assert!(matches!(
        lower_term_and_formula_inputs(&context, bad_missing),
        Err(TermAndFormulaLoweringError::InvalidFraenkelMissingSethoodObligation { kind })
            if kind == ObligationSeedKind::DefinitionCorrectness
    ));
}

#[test]
fn failed_semantic_sites_lower_to_error_nodes_and_diagnostics() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![term_seed(
        CoreTermSeedKind::Error(failed_site("failed-term-overload", 108)),
        108,
    )];
    input.formulas = vec![formula_seed(
        CoreFormulaSeedKind::Error(failed_site("unsupported-formula", 109)),
        109,
    )];
    input.failed_sites = vec![failed_site("standalone-failed-site", 110)];

    let output = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let term = output.term_map[&CoreTermSeedId::new(0)];
    let formula = output.formula_map[&CoreFormulaSeedId::new(0)];

    assert!(matches!(
        output.terms.get(term).expect("term").kind,
        CoreTermKind::Error(_)
    ));
    assert!(matches!(
        output.formulas.get(formula).expect("formula").kind,
        CoreFormulaKind::Error(_)
    ));
    assert_eq!(output.failed_sites.len(), 1);
    assert_eq!(output.diagnostics.len(), 3);
    assert_step3_delta_valid(&context, &output);
}

#[test]
fn definition_lowering_records_boundary_policy_params_and_correctness() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        vec![term_seed(CoreTermSeedKind::Var(x), 112)],
        vec![formula_seed(CoreFormulaSeedKind::True, 113)],
    );
    let term = term_formula.term_map[&CoreTermSeedId::new(0)];
    let goal = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let mut seed = definition_seed(owner, symbol("Owner"), DefinitionBodySeed::Term(term), 114);
    seed.params = vec![test_binder(x, Some(goal), 115)];
    seed.correctness = vec![DefinitionCorrectnessSeed::New(Box::new(
        DefinitionObligationSeed::active(
            goal,
            "definition/owner/coherence",
            "pkg::main::Owner.definition.coherence",
            direct(116, 117),
            provenance("checker:definition:coherence"),
        ),
    ))];

    let output = lower_definition_inputs(
        &context,
        &term_formula,
        DefinitionLoweringInput {
            definitions: vec![seed],
        },
    )
    .expect("definition lowering");
    let definition_id = output.definition_map[&owner];
    let definition = output
        .definitions
        .get(definition_id)
        .expect("definition row");
    let obligation_id = definition.correctness[0];
    let obligation = output
        .obligation_seeds
        .get(obligation_id)
        .expect("correctness obligation");

    assert_eq!(definition.owner.anchor_item(), owner);
    assert_eq!(definition.owner.item(), Some(owner));
    assert_eq!(definition.owner.property_symbol(), None);
    assert!(matches!(definition.body, DefinitionBody::Term(actual) if actual == term));
    assert_eq!(definition.expansion, ExpansionPolicy::Opaque);
    assert_eq!(definition.params[0].ty_guard, Some(goal));
    assert_eq!(output.correctness_obligations.len(), 1);
    assert!(output.correctness_obligations[0].is_new);
    assert_eq!(obligation.kind, ObligationSeedKind::DefinitionCorrectness);
    assert_eq!(obligation.status, ObligationSeedStatus::Active);
    assert!(obligation.core_refs.contains(&CoreNodeRef::Item(owner)));
    assert!(
        obligation
            .core_refs
            .contains(&CoreNodeRef::Definition(definition_id))
    );
    assert!(obligation.core_refs.contains(&CoreNodeRef::Term(term)));
    assert!(obligation.core_refs.contains(&CoreNodeRef::Formula(goal)));
    assert_step4_delta_valid(&context, &term_formula, &output);
}

#[test]
fn definition_correctness_handles_deferred_and_existing_seeds_with_backrefs() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let mut term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        vec![term_seed(CoreTermSeedKind::Var(x), 118)],
        vec![formula_seed(CoreFormulaSeedKind::True, 119)],
    );
    let term = term_formula.term_map[&CoreTermSeedId::new(0)];
    let goal = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let existing_source = direct(120, 121);
    let existing = term_formula.obligation_seeds.insert(ObligationSeed {
        owner,
        kind: ObligationSeedKind::DefinitionCorrectness,
        goal: Some(goal),
        context: Vec::new(),
        local_path: LocalProofOrProgramPath::new("definition/existing"),
        label: None,
        semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Owner.definition.existing"),
        provenance: vec![CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "checker:definition:existing",
        )],
        source: existing_source.clone(),
        core_refs: vec![CoreNodeRef::Item(owner), CoreNodeRef::Formula(goal)],
        status: ObligationSeedStatus::Active,
        diagnostics: Vec::new(),
    });
    term_formula
        .source_map
        .obligation_sources
        .insert(existing, existing_source);

    let mut seed = definition_seed(owner, symbol("Owner"), DefinitionBodySeed::Term(term), 121);
    seed.correctness = vec![
        DefinitionCorrectnessSeed::Existing(existing),
        DefinitionCorrectnessSeed::New(Box::new(DefinitionObligationSeed::deferred(
            "definition/deferred",
            "pkg::main::Owner.definition.deferred",
            direct(122, 123),
            provenance("checker:definition:deferred"),
        ))),
    ];

    let output = lower_definition_inputs(
        &context,
        &term_formula,
        DefinitionLoweringInput {
            definitions: vec![seed],
        },
    )
    .expect("definition lowering");
    let definition = output.definition_map[&owner];
    let row = output.definitions.get(definition).expect("definition");
    let existing_seed = output
        .obligation_seeds
        .get(existing)
        .expect("existing obligation");
    let deferred = row.correctness[1];
    let deferred_seed = output
        .obligation_seeds
        .get(deferred)
        .expect("deferred obligation");

    assert_eq!(row.correctness[0], existing);
    assert_eq!(output.correctness_obligations.len(), 2);
    assert!(!output.correctness_obligations[0].is_new);
    assert!(output.correctness_obligations[1].is_new);
    assert!(
        existing_seed
            .core_refs
            .contains(&CoreNodeRef::Definition(definition))
    );
    assert!(existing_seed.core_refs.contains(&CoreNodeRef::Term(term)));
    assert_eq!(deferred_seed.status, ObligationSeedStatus::Deferred);
    assert!(deferred_seed.goal.is_none());
    assert!(
        deferred_seed
            .core_refs
            .contains(&CoreNodeRef::Definition(definition))
    );
    assert!(deferred_seed.core_refs.contains(&CoreNodeRef::Term(term)));
    assert_step4_delta_valid(&context, &term_formula, &output);
}

#[test]
fn definition_correctness_existing_seed_must_match_definition_owner() {
    let context = prepare_core_context(input_with_items(vec![
        item_seed("Owner", 118),
        item_seed("OtherOwner", 119),
    ]))
    .expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("Owner"))
        .expect("owner");
    let other_owner = context
        .item_registry()
        .id_for_symbol(&symbol("OtherOwner"))
        .expect("other owner");
    let mut term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 120)],
    );
    let goal = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let source = direct(121, 122);
    let existing = term_formula.obligation_seeds.insert(ObligationSeed {
        owner: other_owner,
        kind: ObligationSeedKind::DefinitionCorrectness,
        goal: Some(goal),
        context: Vec::new(),
        local_path: LocalProofOrProgramPath::new("definition/other-owner"),
        label: None,
        semantic_origin: NormalizedSemanticOrigin::new(
            "pkg::main::OtherOwner.definition.correctness",
        ),
        provenance: vec![CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "checker:definition:other-owner",
        )],
        source: source.clone(),
        core_refs: vec![CoreNodeRef::Item(other_owner), CoreNodeRef::Formula(goal)],
        status: ObligationSeedStatus::Active,
        diagnostics: Vec::new(),
    });
    term_formula
        .source_map
        .obligation_sources
        .insert(existing, source);
    let mut seed = definition_seed(
        owner,
        symbol("Owner"),
        DefinitionBodySeed::Formula(goal),
        122,
    );
    seed.correctness = vec![DefinitionCorrectnessSeed::Existing(existing)];

    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![seed],
            },
        ),
        Err(DefinitionLoweringError::ExistingCorrectnessOwnerMismatch {
            obligation,
            expected,
            actual,
        }) if obligation == existing && expected == owner && actual == other_owner
    ));
}

#[test]
fn definition_lowering_preserves_all_expansion_policies() {
    let mut input = input_with_items(vec![
        CoreItemSeed::new(
            symbol("OpaqueDef"),
            CoreItemKind::Functor,
            "public",
            direct(120, 121),
            provenance("checker:item:opaque-def"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
        CoreItemSeed::new(
            symbol("TransparentDef"),
            CoreItemKind::Predicate,
            "public",
            direct(121, 122),
            provenance("checker:item:transparent-def"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
        CoreItemSeed::new(
            symbol("ReducibleDef"),
            CoreItemKind::Reduction,
            "public",
            direct(122, 123),
            provenance("checker:item:reducible-def"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::Reduction),
        CoreItemSeed::new(
            symbol("ComputableDef"),
            CoreItemKind::Functor,
            "public",
            direct(123, 124),
            provenance("checker:item:computable-def"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
    ]);
    input.variable_seeds = Vec::new();
    let context = prepare_core_context(input).expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("OpaqueDef"))
        .expect("owner");
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        vec![term_seed(CoreTermSeedKind::Const(symbol("Const")), 125)],
        vec![formula_seed(CoreFormulaSeedKind::True, 126)],
    );
    let term = term_formula.term_map[&CoreTermSeedId::new(0)];
    let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let mut definitions = Vec::new();
    for (name, body, expansion, start) in [
        (
            "OpaqueDef",
            DefinitionBodySeed::Term(term),
            ExpansionPolicy::Opaque,
            127,
        ),
        (
            "TransparentDef",
            DefinitionBodySeed::Formula(formula),
            ExpansionPolicy::Transparent,
            128,
        ),
        (
            "ReducibleDef",
            DefinitionBodySeed::Term(term),
            ExpansionPolicy::Reducible {
                registration: symbol("ReduceRegistration"),
            },
            129,
        ),
        (
            "ComputableDef",
            DefinitionBodySeed::Term(term),
            ExpansionPolicy::Computable {
                algorithm: symbol("RuntimeAlgorithm"),
            },
            130,
        ),
    ] {
        let item = context
            .item_registry()
            .id_for_symbol(&symbol(name))
            .expect("definition item");
        let mut seed = definition_seed(item, symbol(name), body, start);
        seed.expansion = expansion;
        definitions.push(seed);
    }

    let output = lower_definition_inputs(
        &context,
        &term_formula,
        DefinitionLoweringInput { definitions },
    )
    .expect("definition lowering");
    let policies = [
        "OpaqueDef",
        "TransparentDef",
        "ReducibleDef",
        "ComputableDef",
    ]
    .iter()
    .map(|name| {
        let item = context
            .item_registry()
            .id_for_symbol(&symbol(name))
            .expect("definition item");
        output
            .definitions
            .get(output.definition_map[&item])
            .expect("definition")
            .expansion
            .clone()
    })
    .collect::<Vec<_>>();

    assert!(matches!(policies[0], ExpansionPolicy::Opaque));
    assert!(matches!(policies[1], ExpansionPolicy::Transparent));
    assert!(matches!(policies[2], ExpansionPolicy::Reducible { .. }));
    assert!(matches!(policies[3], ExpansionPolicy::Computable { .. }));
    assert_step4_delta_valid(&context, &term_formula, &output);
}

#[test]
fn definition_lowering_rejects_invalid_definition_boundaries() {
    let context = prepare_core_context(input_with_items(vec![
        item_seed("HasBoundary", 131),
        CoreItemSeed::new(
            symbol("NoBoundary"),
            CoreItemKind::Functor,
            "public",
            direct(132, 133),
            provenance("checker:item:no-boundary"),
        ),
        CoreItemSeed::new(
            symbol("AlgorithmBoundary"),
            CoreItemKind::Algorithm,
            "public",
            direct(133, 134),
            provenance("checker:item:algorithm-boundary"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::Algorithm),
    ]))
    .expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("HasBoundary"))
        .expect("owner");
    let no_boundary = context
        .item_registry()
        .id_for_symbol(&symbol("NoBoundary"))
        .expect("no boundary");
    let algorithm = context
        .item_registry()
        .id_for_symbol(&symbol("AlgorithmBoundary"))
        .expect("algorithm boundary");
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 134)],
    );
    let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];

    let duplicate = definition_seed(
        owner,
        symbol("HasBoundary"),
        DefinitionBodySeed::Formula(formula),
        135,
    );
    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![duplicate.clone(), duplicate],
            },
        ),
        Err(DefinitionLoweringError::DuplicateDefinitionOwner { owner: actual })
            if actual == owner
    ));

    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![definition_seed(
                    owner,
                    symbol("WrongSymbol"),
                    DefinitionBodySeed::Formula(formula),
                    136,
                )],
            },
        ),
        Err(DefinitionLoweringError::DefinitionSymbolMismatch { owner: actual, .. })
            if actual == owner
    ));

    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![definition_seed(
                    no_boundary,
                    symbol("NoBoundary"),
                    DefinitionBodySeed::Formula(formula),
                    137,
                )],
            },
        ),
        Err(DefinitionLoweringError::MissingDefinitionBoundary { owner: actual })
            if actual == no_boundary
    ));

    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![definition_seed(
                    algorithm,
                    symbol("AlgorithmBoundary"),
                    DefinitionBodySeed::Formula(formula),
                    138,
                )],
            },
        ),
        Err(DefinitionLoweringError::AlgorithmBoundaryRequiresDeferredBody { owner: actual })
            if actual == algorithm
    ));
}

#[test]
fn guarded_definition_otherwise_records_checker_owned_exclusions() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        vec![term_seed(CoreTermSeedKind::Var(x), 132)],
        vec![
            formula_seed(CoreFormulaSeedKind::True, 133),
            formula_seed(CoreFormulaSeedKind::False, 134),
            formula_seed(CoreFormulaSeedKind::True, 135),
        ],
    );
    let term = term_formula.term_map[&CoreTermSeedId::new(0)];
    let guard_a = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let guard_b = term_formula.formula_map[&CoreFormulaSeedId::new(1)];
    let otherwise = term_formula.formula_map[&CoreFormulaSeedId::new(2)];
    let guarded = DefinitionBodySeed::Guarded(vec![
        GuardedDefinitionBranchSeed {
            guard: DefinitionGuardSeed::Explicit(guard_a),
            body: DefinitionBranchBody::Term(term),
        },
        GuardedDefinitionBranchSeed {
            guard: DefinitionGuardSeed::Explicit(guard_b),
            body: DefinitionBranchBody::Formula(guard_b),
        },
        GuardedDefinitionBranchSeed {
            guard: DefinitionGuardSeed::Otherwise {
                guard: otherwise,
                excludes: vec![guard_a, guard_b],
                provenance: provenance("checker:otherwise"),
            },
            body: DefinitionBranchBody::Term(term),
        },
    ]);

    let output = lower_definition_inputs(
        &context,
        &term_formula,
        DefinitionLoweringInput {
            definitions: vec![definition_seed(owner, symbol("Owner"), guarded, 136)],
        },
    )
    .expect("definition lowering");
    let definition = output
        .definitions
        .get(output.definition_map[&owner])
        .expect("definition");

    assert!(matches!(
        &definition.body,
        DefinitionBody::Guarded(branches)
            if branches.len() == 3 && branches[2].guard == otherwise
    ));
    assert_eq!(output.otherwise_guards.len(), 1);
    assert_eq!(output.otherwise_guards[0].excludes, vec![guard_a, guard_b]);
    assert_step4_delta_valid(&context, &term_formula, &output);

    let bad_guarded = DefinitionBodySeed::Guarded(vec![
        GuardedDefinitionBranchSeed {
            guard: DefinitionGuardSeed::Explicit(guard_a),
            body: DefinitionBranchBody::Term(term),
        },
        GuardedDefinitionBranchSeed {
            guard: DefinitionGuardSeed::Otherwise {
                guard: otherwise,
                excludes: vec![guard_b],
                provenance: provenance("checker:otherwise:bad"),
            },
            body: DefinitionBranchBody::Term(term),
        },
    ]);
    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![definition_seed(owner, symbol("Owner"), bad_guarded, 137)],
            },
        ),
        Err(DefinitionLoweringError::OtherwiseExcludesMismatch { branch: 1 })
    ));
}

#[test]
fn definition_generated_dependencies_are_reachable_through_formula_bodies() {
    let x = CoreVarId::new(0);
    let key = GeneratedOriginKey::new("choice:exported");
    let (context, owner, existing_origin) = context_with_existing_choice_origin(x, key.clone());
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 138),
        term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("choice_existing"),
                origin_functor: symbol("choice_existing"),
                key,
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:exported-choice",
                )],
            },
            139,
        ),
    ];
    input.formulas = vec![
        formula_seed(
            CoreFormulaSeedKind::Atom {
                predicate: symbol("HasChoice"),
                args: vec![CoreTermSeedId::new(1)],
            },
            140,
        ),
        formula_seed(CoreFormulaSeedKind::True, 141),
    ];
    let term_formula = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let body = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let unreachable_body = term_formula.formula_map[&CoreFormulaSeedId::new(1)];
    let choice_term = term_formula.term_map[&CoreTermSeedId::new(1)];
    let mut seed = definition_seed(
        owner,
        symbol("Owner"),
        DefinitionBodySeed::Formula(body),
        142,
    );
    seed.generated_dependencies = vec![existing_origin];

    let output = lower_definition_inputs(
        &context,
        &term_formula,
        DefinitionLoweringInput {
            definitions: vec![seed],
        },
    )
    .expect("definition lowering");

    assert_eq!(output.generated_dependencies.len(), 1);
    assert_eq!(output.generated_dependencies[0].origin, existing_origin);
    assert!(term_formula.generated_delta.is_empty());
    assert!(term_formula.generated_origin_refs[0].reused_existing);
    assert_eq!(
        output.generated_dependencies[0].use_terms,
        vec![choice_term]
    );
    assert_step4_delta_valid(&context, &term_formula, &output);

    let missing_dependency = definition_seed(
        owner,
        symbol("Owner"),
        DefinitionBodySeed::Formula(body),
        143,
    );
    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![missing_dependency],
            },
        ),
        Err(DefinitionLoweringError::MissingGeneratedDependency { origin })
            if origin == existing_origin
    ));

    let mut spurious_dependency = definition_seed(
        owner,
        symbol("Owner"),
        DefinitionBodySeed::Formula(unreachable_body),
        144,
    );
    spurious_dependency.generated_dependencies = vec![existing_origin];
    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![spurious_dependency],
            },
        ),
        Err(DefinitionLoweringError::SpuriousGeneratedDependency { origin })
            if origin == existing_origin
    ));
}

#[test]
fn algorithm_and_unavailable_definition_bodies_remain_deferred_or_error() {
    let context = prepare_core_context(input_with_items(vec![
        CoreItemSeed::new(
            symbol("AlgorithmDef"),
            CoreItemKind::Algorithm,
            "public",
            direct(144, 145),
            provenance("checker:item:algorithm-def"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::Algorithm),
        CoreItemSeed::new(
            symbol("UnavailableDef"),
            CoreItemKind::Functor,
            "public",
            direct(146, 147),
            provenance("checker:item:unavailable-def"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
    ]))
    .expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("AlgorithmDef"))
        .expect("algorithm owner");
    let other_owner = context
        .item_registry()
        .id_for_symbol(&symbol("UnavailableDef"))
        .expect("unavailable owner");
    let term_formula = lower_test_terms_and_formulas(&context, owner, Vec::new(), Vec::new());
    let definitions = vec![
        definition_seed(
            owner,
            symbol("AlgorithmDef"),
            DefinitionBodySeed::AlgorithmDeferred(failed_site("algorithm-body-deferred", 148)),
            148,
        ),
        definition_seed(
            other_owner,
            symbol("UnavailableDef"),
            DefinitionBodySeed::Unavailable(failed_site("definition-prerequisite-error", 149)),
            149,
        ),
    ];

    let output = lower_definition_inputs(
        &context,
        &term_formula,
        DefinitionLoweringInput { definitions },
    )
    .expect("definition lowering");

    assert_eq!(
        output
            .item_status_updates
            .iter()
            .map(|update| (update.item, update.status))
            .collect::<Vec<_>>(),
        vec![
            (owner, CoreItemStatus::Skipped),
            (other_owner, CoreItemStatus::Error)
        ]
    );
    for item in [owner, other_owner] {
        let definition = output
            .definitions
            .get(output.definition_map[&item])
            .expect("definition");
        assert!(matches!(definition.body, DefinitionBody::Unavailable(_)));
    }
    assert_eq!(output.diagnostics.len(), 2);
    assert_step4_delta_valid(&context, &term_formula, &output);
}

#[test]
fn definition_lowering_rejects_nonterm_params_and_wrong_obligations() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var_sort(x, NormalizedVarSort::Formula);
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 150)],
    );
    let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let mut bad_param = definition_seed(
        owner,
        symbol("Owner"),
        DefinitionBodySeed::Formula(formula),
        151,
    );
    bad_param.params = vec![test_binder(x, None, 152)];

    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![bad_param],
            },
        ),
        Err(DefinitionLoweringError::NonTermDefinitionParam { var, sort })
            if var == x && sort == NormalizedVarSort::Formula
    ));

    let (context, owner) = context_with_var(CoreVarId::new(1));
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 153)],
    );
    let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let mut wrong_obligation = definition_seed(
        owner,
        symbol("Owner"),
        DefinitionBodySeed::Formula(formula),
        154,
    );
    wrong_obligation.correctness = vec![DefinitionCorrectnessSeed::New(Box::new(
        DefinitionObligationSeed {
            kind: ObligationSeedKind::TheoremProof,
            status: ObligationSeedStatus::Active,
            goal: Some(formula),
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("definition/wrong-kind"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new(
                "pkg::main::Owner.definition.wrong-kind",
            ),
            source: direct(155, 156),
            provenance: provenance("checker:definition:wrong-kind"),
        },
    ))];

    assert!(matches!(
        lower_definition_inputs(
            &context,
            &term_formula,
            DefinitionLoweringInput {
                definitions: vec![wrong_obligation],
            },
        ),
        Err(DefinitionLoweringError::InvalidCorrectnessObligation { kind, status })
            if kind == ObligationSeedKind::TheoremProof
                && status == ObligationSeedStatus::Active
    ));
}

#[test]
fn proof_lowering_replaces_thesis_and_emits_terminal_obligation_backrefs() {
    let x = CoreVarId::new(0);
    let (context, owner) = context_with_var(x);
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 160)],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let assumption_label = CoreLabelRef::new("A1");
    let terminal_citations = vec![CoreCitation::Label(assumption_label.clone())];
    let terminal = ProofNodeSeed::TerminalGoal(
        ProofTerminalGoalSeed::active(
            ProofFormulaRef::Thesis,
            "proof/terminal",
            "pkg::main::Owner.proof.terminal",
            direct(164, 165),
            provenance("checker:proof:terminal"),
        )
        .with_context(vec![proposition])
        .with_citations(terminal_citations.clone()),
    );
    let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::IntroduceBinder {
        binder: test_binder(x, Some(proposition), 161),
        child: Box::new(ProofNodeSeed::Assume {
            label: Some(assumption_label.clone()),
            formula: ProofFormulaRef::Thesis,
            child: Box::new(terminal),
            source: direct(162, 163),
            provenance: provenance("checker:proof:assume"),
        }),
        source: direct(161, 162),
        provenance: provenance("checker:proof:introduce"),
    });

    let output = lower_proof_inputs(
        &context,
        &term_formula,
        &definitions,
        ProofLoweringInput {
            proofs: vec![proof_seed(
                owner,
                symbol("Owner"),
                proposition,
                CoreProofStatus::Open,
                skeleton,
                165,
            )],
        },
    )
    .expect("proof lowering");
    let proof_id = output.proof_map[&owner];
    let proof = output.proofs.get(proof_id).expect("proof");
    let CoreProofNodeKind::IntroduceBinder {
        binder,
        child: assume,
    } = &output.proof_nodes.get(proof.root).expect("root").kind
    else {
        panic!("expected introduced binder root");
    };
    let CoreProofNodeKind::Assume {
        label,
        formula,
        child: terminal,
    } = &output.proof_nodes.get(*assume).expect("assume").kind
    else {
        panic!("expected assumption node");
    };
    let CoreProofNodeKind::TerminalGoal {
        obligation: obligation_id,
        citations,
    } = &output.proof_nodes.get(*terminal).expect("terminal").kind
    else {
        panic!("expected terminal goal");
    };
    let obligation = output
        .obligation_seeds
        .get(*obligation_id)
        .expect("terminal obligation");

    assert_eq!(proof.status, CoreProofStatus::Open);
    assert_eq!(proof.proposition, proposition);
    assert_eq!(binder.var, x);
    assert_eq!(binder.ty_guard, Some(proposition));
    assert_eq!(label.as_ref(), Some(&assumption_label));
    assert_eq!(*formula, proposition);
    assert_eq!(obligation.kind, ObligationSeedKind::TheoremProof);
    assert_eq!(obligation.status, ObligationSeedStatus::Active);
    assert_eq!(obligation.goal, Some(proposition));
    assert_eq!(obligation.context, vec![proposition]);
    assert!(obligation.core_refs.contains(&CoreNodeRef::Item(owner)));
    assert!(obligation.core_refs.contains(&CoreNodeRef::Proof(proof_id)));
    assert!(
        obligation
            .core_refs
            .contains(&CoreNodeRef::ProofNode(*terminal))
    );
    assert!(
        obligation
            .core_refs
            .contains(&CoreNodeRef::Formula(proposition))
    );
    assert_eq!(output.terminal_obligations.len(), 1);
    assert_eq!(output.terminal_citations.len(), 1);
    assert_eq!(output.terminal_citations[0].proof, proof_id);
    assert_eq!(output.terminal_citations[0].node, *terminal);
    assert_eq!(output.terminal_citations[0].obligation, *obligation_id);
    assert_eq!(citations, &terminal_citations);
    assert_eq!(output.terminal_citations[0].citations, terminal_citations);
    assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
}

#[test]
fn proof_lowering_tracks_current_goal_sequence_labels_and_active_formulas() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![
            formula_seed(CoreFormulaSeedKind::True, 166),
            formula_seed(CoreFormulaSeedKind::False, 167),
            formula_seed(CoreFormulaSeedKind::True, 168),
        ],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let current_goal = term_formula.formula_map[&CoreFormulaSeedId::new(1)];
    let assumption_formula = term_formula.formula_map[&CoreFormulaSeedId::new(2)];
    let step_label = CoreLabelRef::new("SEQ1");
    let terminal_citations = vec![CoreCitation::Label(step_label.clone())];
    let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::Sequence {
        children: vec![
            ProofNodeSeed::Step {
                label: Some(step_label.clone()),
                formula: ProofFormulaRef::Formula(assumption_formula),
                justification: ProofJustificationSeed::new(
                    Vec::new(),
                    direct(169, 170),
                    provenance("checker:proof:sequence-step:justification"),
                ),
                source: direct(170, 171),
                provenance: provenance("checker:proof:sequence-step"),
            },
            ProofNodeSeed::CurrentGoal {
                thesis: ProofFormulaRef::Formula(current_goal),
                child: Box::new(ProofNodeSeed::Assume {
                    label: None,
                    formula: ProofFormulaRef::Formula(assumption_formula),
                    child: Box::new(ProofNodeSeed::TerminalGoal(
                        ProofTerminalGoalSeed::active(
                            ProofFormulaRef::Thesis,
                            "proof/current-goal/terminal",
                            "pkg::main::Owner.proof.current-goal.terminal",
                            direct(171, 172),
                            provenance("checker:proof:current-goal-terminal"),
                        )
                        .with_citations(terminal_citations.clone()),
                    )),
                    source: direct(172, 173),
                    provenance: provenance("checker:proof:current-goal-assume"),
                }),
                source: direct(173, 174),
                provenance: provenance("checker:proof:current-goal"),
            },
        ],
        source: direct(174, 175),
        provenance: provenance("checker:proof:sequence"),
    });

    let output = lower_proof_inputs(
        &context,
        &term_formula,
        &definitions,
        ProofLoweringInput {
            proofs: vec![proof_seed(
                owner,
                symbol("Owner"),
                proposition,
                CoreProofStatus::Conditional,
                skeleton,
                175,
            )],
        },
    )
    .expect("proof lowering");
    let proof_id = output.proof_map[&owner];
    let proof = output.proofs.get(proof_id).expect("proof");
    let CoreProofNodeKind::Sequence { children } =
        &output.proof_nodes.get(proof.root).expect("sequence").kind
    else {
        panic!("expected sequence root");
    };
    let CoreProofNodeKind::CurrentGoal {
        thesis,
        child: assume,
    } = &output
        .proof_nodes
        .get(children[1])
        .expect("current goal")
        .kind
    else {
        panic!("expected current-goal child");
    };
    let CoreProofNodeKind::Assume {
        child: terminal, ..
    } = output.proof_nodes.get(*assume).expect("assume").kind
    else {
        panic!("expected assumption child");
    };
    let CoreProofNodeKind::TerminalGoal {
        obligation,
        citations,
    } = &output.proof_nodes.get(terminal).expect("terminal").kind
    else {
        panic!("expected terminal goal");
    };
    let obligation = output
        .obligation_seeds
        .get(*obligation)
        .expect("terminal obligation");

    assert_eq!(*thesis, current_goal);
    assert_eq!(obligation.goal, Some(current_goal));
    assert_eq!(obligation.context, vec![assumption_formula]);
    assert!(
        obligation
            .core_refs
            .contains(&CoreNodeRef::Formula(current_goal))
    );
    assert!(
        obligation
            .core_refs
            .contains(&CoreNodeRef::Formula(assumption_formula))
    );
    assert_eq!(citations, &terminal_citations);
    assert_eq!(output.terminal_citations[0].citations, terminal_citations);
    assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
}

#[test]
fn proof_lowering_preserves_branches_steps_citations_and_generated_refs() {
    let x = CoreVarId::new(0);
    let key = GeneratedOriginKey::new("choice:proof");
    let (context, owner, existing_origin) = context_with_existing_choice_origin(x, key.clone());
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 166),
        term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("choice_existing"),
                origin_functor: symbol("choice_existing"),
                key,
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:proof:choice",
                )],
            },
            167,
        ),
    ];
    input.formulas = vec![formula_seed(
        CoreFormulaSeedKind::Atom {
            predicate: symbol("ProofPredicate"),
            args: vec![CoreTermSeedId::new(1)],
        },
        168,
    )];
    let term_formula = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let label = CoreLabelRef::new("A1");
    let step = ProofNodeSeed::Assume {
        label: Some(label.clone()),
        formula: ProofFormulaRef::Thesis,
        child: Box::new(ProofNodeSeed::Step {
            label: Some(CoreLabelRef::new("S1")),
            formula: ProofFormulaRef::Thesis,
            justification: ProofJustificationSeed::new(
                vec![
                    CoreCitation::Label(label.clone()),
                    CoreCitation::Generated(existing_origin),
                    CoreCitation::Symbol(symbol("Owner")),
                ],
                direct(169, 170),
                provenance("checker:proof:justification"),
            ),
            source: direct(170, 171),
            provenance: provenance("checker:proof:step"),
        }),
        source: direct(171, 172),
        provenance: provenance("checker:proof:assume:branch"),
    };
    let terminal = ProofNodeSeed::TerminalGoal(
        ProofTerminalGoalSeed::active(
            ProofFormulaRef::Thesis,
            "proof/branch/open",
            "pkg::main::Owner.proof.branch.open",
            direct(172, 173),
            provenance("checker:proof:branch-terminal"),
        )
        .with_citations(vec![
            CoreCitation::Generated(existing_origin),
            CoreCitation::Symbol(symbol("Owner")),
        ]),
    );
    let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::Branch {
        kind: ProofBranchKind::Cases,
        children: vec![step, terminal],
        source: direct(173, 174),
        provenance: provenance("checker:proof:cases"),
    });

    let output = lower_proof_inputs(
        &context,
        &term_formula,
        &definitions,
        ProofLoweringInput {
            proofs: vec![proof_seed(
                owner,
                symbol("Owner"),
                proposition,
                CoreProofStatus::Conditional,
                skeleton,
                174,
            )],
        },
    )
    .expect("proof lowering");
    let proof = output.proofs.get(output.proof_map[&owner]).expect("proof");
    let CoreProofNodeKind::Branch { kind, children } =
        &output.proof_nodes.get(proof.root).expect("branch").kind
    else {
        panic!("expected branch root");
    };
    let CoreProofNodeKind::Assume { child: step_id, .. } = output
        .proof_nodes
        .get(children[0])
        .expect("branch child")
        .kind
    else {
        panic!("expected assumption child");
    };
    let CoreProofNodeKind::Step { justification, .. } =
        &output.proof_nodes.get(step_id).expect("step").kind
    else {
        panic!("expected step");
    };
    let terminal_record = &output.terminal_obligations[0];
    let obligation = output
        .obligation_seeds
        .get(terminal_record.obligation)
        .expect("terminal obligation");
    let terminal_citation_record = &output.terminal_citations[0];

    assert_eq!(proof.status, CoreProofStatus::Conditional);
    assert_eq!(kind, &ProofBranchKind::Cases);
    assert_eq!(children.len(), 2);
    assert_eq!(
        justification.citations,
        vec![
            CoreCitation::Label(label),
            CoreCitation::Generated(existing_origin),
            CoreCitation::Symbol(symbol("Owner"))
        ]
    );
    assert!(term_formula.generated_delta.is_empty());
    assert!(term_formula.generated_origin_refs[0].reused_existing);
    assert!(
        obligation
            .core_refs
            .contains(&CoreNodeRef::Generated(existing_origin))
    );
    assert!(obligation.core_refs.contains(&CoreNodeRef::Item(owner)));
    assert_eq!(terminal_citation_record.proof, output.proof_map[&owner]);
    assert_eq!(terminal_citation_record.node, terminal_record.node);
    assert_eq!(
        terminal_citation_record.obligation,
        terminal_record.obligation
    );
    assert_eq!(
        terminal_citation_record.citations,
        vec![
            CoreCitation::Generated(existing_origin),
            CoreCitation::Symbol(symbol("Owner"))
        ]
    );
    assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
}

#[test]
fn proof_lowering_lowers_lemma_stable_choice_terminal_citations() {
    let x = CoreVarId::new(0);
    let key = GeneratedOriginKey::new("choice:lemma-proof");
    let mut context_input = input_with_items(vec![
        CoreItemSeed::new(
            symbol("LemmaOwner"),
            CoreItemKind::Lemma,
            "public",
            direct(175, 176),
            provenance("checker:item:lemma-owner"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::Lemma),
    ]);
    context_input.variable_seeds = vec![CoreVariableSeed::new(
        x,
        NormalizedVarClass::Free,
        "term-binder",
        NormalizedVarSort::Term,
        provenance("checker:lemma-choice-var"),
    )];
    context_input.binder_seeds = vec![CoreBinderSeed::new(
        x,
        direct(176, 177),
        provenance("checker:lemma-choice-binder"),
    )];
    context_input.generated_origin_seeds = vec![
        GeneratedOriginSeed::new(
            symbol("LemmaOwner"),
            GeneratedOriginKind::StableChoice,
            key.clone(),
            direct(177, 178),
            provenance("checker:lemma-existing-choice"),
        )
        .with_functor(symbol("lemma_choice"))
        .with_params(vec![x])
        .with_evidence(vec![CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "checker:lemma-existing-choice:evidence",
        )]),
    ];
    let context = prepare_core_context(context_input).expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("LemmaOwner"))
        .expect("lemma owner");
    let existing_origin = context
        .generated_origins()
        .get_by_key(owner, GeneratedOriginKind::StableChoice, &key)
        .expect("existing generated origin");
    let mut input = TermAndFormulaLoweringInput::new(owner);
    input.terms = vec![
        term_seed(CoreTermSeedKind::Var(x), 178),
        term_seed(
            CoreTermSeedKind::StableChoice {
                functor: symbol("lemma_choice"),
                origin_functor: symbol("lemma_choice"),
                key,
                params: vec![x],
                args: vec![CoreTermSeedId::new(0)],
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    "checker:lemma-proof:choice",
                )],
            },
            179,
        ),
    ];
    input.formulas = vec![formula_seed(
        CoreFormulaSeedKind::Atom {
            predicate: symbol("LemmaProofPredicate"),
            args: vec![CoreTermSeedId::new(1)],
        },
        180,
    )];
    let term_formula = lower_term_and_formula_inputs(&context, input).expect("lowering");
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::TerminalGoal(
        ProofTerminalGoalSeed::active(
            ProofFormulaRef::Thesis,
            "proof/lemma/stable-choice",
            "pkg::main::LemmaOwner.proof.stable-choice",
            direct(181, 182),
            provenance("checker:proof:lemma-terminal"),
        )
        .with_citations(vec![CoreCitation::Generated(existing_origin)]),
    ));

    let output = lower_proof_inputs(
        &context,
        &term_formula,
        &definitions,
        ProofLoweringInput {
            proofs: vec![proof_seed(
                owner,
                symbol("LemmaOwner"),
                proposition,
                CoreProofStatus::Conditional,
                skeleton,
                182,
            )],
        },
    )
    .expect("lemma proof lowering");
    let terminal_record = &output.terminal_obligations[0];
    let obligation = output
        .obligation_seeds
        .get(terminal_record.obligation)
        .expect("terminal obligation");

    assert!(term_formula.generated_delta.is_empty());
    assert!(term_formula.generated_origin_refs[0].reused_existing);
    assert!(
        obligation
            .core_refs
            .contains(&CoreNodeRef::Generated(existing_origin))
    );
    assert_eq!(output.terminal_citations.len(), 1);
    assert_eq!(
        output.terminal_citations[0].citations,
        vec![CoreCitation::Generated(existing_origin)]
    );
    assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
}

#[test]
fn proof_lowering_reports_malformed_and_status_boundaries() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 175)],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let output = lower_proof_inputs(
        &context,
        &term_formula,
        &definitions,
        ProofLoweringInput {
            proofs: vec![proof_seed(
                owner,
                symbol("Owner"),
                proposition,
                CoreProofStatus::Error,
                ProofSkeletonSeed::Missing(malformed_proof("missing-proof-skeleton", 176)),
                177,
            )],
        },
    )
    .expect("proof lowering");
    let proof = output.proofs.get(output.proof_map[&owner]).expect("proof");
    let CoreProofNodeKind::Error(diagnostic_id) =
        output.proof_nodes.get(proof.root).expect("error root").kind
    else {
        panic!("expected error root");
    };
    let diagnostic = output.diagnostics.get(diagnostic_id).expect("diagnostic");

    assert_eq!(proof.status, CoreProofStatus::Error);
    assert_eq!(
        diagnostic.class,
        CoreDiagnosticClass::MalformedProofSkeleton
    );
    assert!(output.terminal_obligations.is_empty());
    assert_step5_delta_valid(&context, &term_formula, &definitions, &output);

    let explicit_error = lower_proof_inputs(
        &context,
        &term_formula,
        &definitions,
        ProofLoweringInput {
            proofs: vec![proof_seed(
                owner,
                symbol("Owner"),
                proposition,
                CoreProofStatus::Error,
                ProofSkeletonSeed::Node(ProofNodeSeed::Error(malformed_proof(
                    "explicit-error-proof",
                    178,
                ))),
                179,
            )],
        },
    )
    .expect("explicit error proof lowering");
    let proof = explicit_error
        .proofs
        .get(explicit_error.proof_map[&owner])
        .expect("proof");
    let CoreProofNodeKind::Error(diagnostic_id) = explicit_error
        .proof_nodes
        .get(proof.root)
        .expect("error")
        .kind
    else {
        panic!("expected explicit error root");
    };
    let diagnostic = explicit_error
        .diagnostics
        .get(diagnostic_id)
        .expect("diagnostic");

    assert_eq!(
        diagnostic.class,
        CoreDiagnosticClass::MalformedProofSkeleton
    );
    assert!(explicit_error.terminal_obligations.is_empty());
    assert_step5_delta_valid(&context, &term_formula, &definitions, &explicit_error);

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    ProofSkeletonSeed::Missing(malformed_proof("missing-open-proof", 180)),
                    181,
                )],
            },
        ),
        Err(ProofLoweringError::MalformedSkeletonRequiresErrorStatus {
            status: CoreProofStatus::Open
        })
    ));

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    ProofSkeletonSeed::Node(ProofNodeSeed::Error(malformed_proof(
                        "open-error-root",
                        182,
                    ))),
                    183,
                )],
            },
        ),
        Err(ProofLoweringError::MalformedSkeletonRequiresErrorStatus {
            status: CoreProofStatus::Open
        })
    ));

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Error,
                    ProofSkeletonSeed::Node(ProofNodeSeed::Step {
                        label: None,
                        formula: ProofFormulaRef::Thesis,
                        justification: ProofJustificationSeed::new(
                            Vec::new(),
                            direct(184, 185),
                            provenance("checker:proof:error-status-justification"),
                        ),
                        source: direct(185, 186),
                        provenance: provenance("checker:proof:error-status-step"),
                    }),
                    186,
                )],
            },
        ),
        Err(ProofLoweringError::ErrorStatusRequiresMalformedSkeleton)
    ));
}

#[test]
fn proof_lowering_validates_introduced_binders() {
    let x = CoreVarId::new(0);
    let y = CoreVarId::new(1);
    let (context, owner) = context_with_var(x);
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 183)],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let undeclared = ProofSkeletonSeed::Node(ProofNodeSeed::IntroduceBinder {
        binder: test_binder(y, None, 184),
        child: Box::new(proof_step_node(185)),
        source: direct(186, 187),
        provenance: provenance("checker:proof:undeclared-binder"),
    });

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    undeclared,
                    187,
                )],
            },
        ),
        Err(ProofLoweringError::UndeclaredIntroducedBinder { var }) if var == y
    ));

    let (context, owner) = context_with_var_sort(x, NormalizedVarSort::Formula);
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 188)],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let nonterm = ProofSkeletonSeed::Node(ProofNodeSeed::IntroduceBinder {
        binder: test_binder(x, None, 189),
        child: Box::new(proof_step_node(190)),
        source: direct(191, 192),
        provenance: provenance("checker:proof:nonterm-binder"),
    });

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    nonterm,
                    192,
                )],
            },
        ),
        Err(ProofLoweringError::NonTermIntroducedBinder { var, sort })
            if var == x && sort == NormalizedVarSort::Formula
    ));
}

#[test]
fn proof_lowering_accepts_assumed_steps_and_now_suppose_branches() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 193)],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let assumed = lower_proof_inputs(
        &context,
        &term_formula,
        &definitions,
        ProofLoweringInput {
            proofs: vec![proof_seed(
                owner,
                symbol("Owner"),
                proposition,
                CoreProofStatus::Assumed,
                proof_step_skeleton(194),
                195,
            )],
        },
    )
    .expect("assumed proof without terminal goals is accepted");
    let proof = assumed
        .proofs
        .get(assumed.proof_map[&owner])
        .expect("assumed proof");

    assert_eq!(proof.status, CoreProofStatus::Assumed);
    assert!(assumed.terminal_obligations.is_empty());
    assert!(assumed.terminal_citations.is_empty());
    assert_step5_delta_valid(&context, &term_formula, &definitions, &assumed);

    for (kind, start) in [(ProofBranchKind::Now, 196), (ProofBranchKind::Suppose, 200)] {
        let skeleton = ProofSkeletonSeed::Node(ProofNodeSeed::Branch {
            kind: kind.clone(),
            children: vec![proof_step_node(start)],
            source: direct(start + 2, start + 3),
            provenance: provenance(format!("checker:proof:branch:{start}").as_str()),
        });
        let output = lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    skeleton,
                    start + 3,
                )],
            },
        )
        .expect("branch proof lowering");
        let proof = output.proofs.get(output.proof_map[&owner]).expect("proof");
        let CoreProofNodeKind::Branch {
            kind: actual,
            children,
        } = &output.proof_nodes.get(proof.root).expect("branch").kind
        else {
            panic!("expected branch root");
        };

        assert_eq!(actual, &kind);
        assert_eq!(children.len(), 1);
        assert_step5_delta_valid(&context, &term_formula, &definitions, &output);
    }
}

#[test]
fn proof_lowering_validates_labels_citations_and_owners() {
    let (context, owner) = context_with_var(CoreVarId::new(0));
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 183)],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let duplicate = ProofSkeletonSeed::Node(ProofNodeSeed::Branch {
        kind: ProofBranchKind::Now,
        children: vec![
            ProofNodeSeed::Step {
                label: Some(CoreLabelRef::new("DUP")),
                formula: ProofFormulaRef::Thesis,
                justification: ProofJustificationSeed::new(
                    Vec::new(),
                    direct(184, 185),
                    provenance("checker:proof:dup:left:justification"),
                ),
                source: direct(185, 186),
                provenance: provenance("checker:proof:dup:left"),
            },
            ProofNodeSeed::Step {
                label: Some(CoreLabelRef::new("DUP")),
                formula: ProofFormulaRef::Thesis,
                justification: ProofJustificationSeed::new(
                    Vec::new(),
                    direct(186, 187),
                    provenance("checker:proof:dup:right:justification"),
                ),
                source: direct(187, 188),
                provenance: provenance("checker:proof:dup:right"),
            },
        ],
        source: direct(188, 189),
        provenance: provenance("checker:proof:dup:branch"),
    });
    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    duplicate,
                    189,
                )],
            },
        ),
        Err(ProofLoweringError::DuplicateProofLabel { label })
            if label.as_str() == "DUP"
    ));

    let sibling_label = CoreLabelRef::new("SIB");
    let sibling_citation = ProofSkeletonSeed::Node(ProofNodeSeed::Branch {
        kind: ProofBranchKind::Cases,
        children: vec![
            ProofNodeSeed::Step {
                label: Some(sibling_label.clone()),
                formula: ProofFormulaRef::Thesis,
                justification: ProofJustificationSeed::new(
                    Vec::new(),
                    direct(190, 191),
                    provenance("checker:proof:sibling:left:justification"),
                ),
                source: direct(191, 192),
                provenance: provenance("checker:proof:sibling:left"),
            },
            ProofNodeSeed::Step {
                label: None,
                formula: ProofFormulaRef::Thesis,
                justification: ProofJustificationSeed::new(
                    vec![CoreCitation::Label(sibling_label.clone())],
                    direct(192, 193),
                    provenance("checker:proof:sibling:right:justification"),
                ),
                source: direct(193, 194),
                provenance: provenance("checker:proof:sibling:right"),
            },
        ],
        source: direct(194, 195),
        provenance: provenance("checker:proof:sibling:branch"),
    });
    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    sibling_citation,
                    195,
                )],
            },
        ),
        Err(ProofLoweringError::UnknownProofLabel { label })
            if label == sibling_label
    ));

    let bad_symbol = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
        label: None,
        formula: ProofFormulaRef::Thesis,
        justification: ProofJustificationSeed::new(
            vec![CoreCitation::Symbol(symbol("MissingTheorem"))],
            direct(196, 197),
            provenance("checker:proof:bad-symbol:justification"),
        ),
        source: direct(197, 198),
        provenance: provenance("checker:proof:bad-symbol"),
    });
    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    bad_symbol,
                    198,
                )],
            },
        ),
        Err(ProofLoweringError::InvalidSymbolCitation { symbol: cited })
            if cited.as_ref() == &symbol("MissingTheorem")
    ));

    let empty_label = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
        label: Some(CoreLabelRef::new("")),
        formula: ProofFormulaRef::Thesis,
        justification: ProofJustificationSeed::new(
            Vec::new(),
            direct(199, 200),
            provenance("checker:proof:empty-label:justification"),
        ),
        source: direct(200, 201),
        provenance: provenance("checker:proof:empty-label"),
    });
    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    empty_label,
                    201,
                )],
            },
        ),
        Err(ProofLoweringError::InvalidProofLabel { label })
            if label.as_str().is_empty()
    ));

    let forward_label = CoreLabelRef::new("FWD");
    let forward_citation = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
        label: Some(forward_label.clone()),
        formula: ProofFormulaRef::Thesis,
        justification: ProofJustificationSeed::new(
            vec![CoreCitation::Label(forward_label.clone())],
            direct(202, 203),
            provenance("checker:proof:forward-label:justification"),
        ),
        source: direct(203, 204),
        provenance: provenance("checker:proof:forward-label"),
    });
    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    forward_citation,
                    204,
                )],
            },
        ),
        Err(ProofLoweringError::UnknownProofLabel { label })
            if label == forward_label
    ));

    let missing_generated = GeneratedOriginId::new(99);
    let bad_generated = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
        label: None,
        formula: ProofFormulaRef::Thesis,
        justification: ProofJustificationSeed::new(
            vec![CoreCitation::Generated(missing_generated)],
            direct(205, 206),
            provenance("checker:proof:bad-generated:justification"),
        ),
        source: direct(206, 207),
        provenance: provenance("checker:proof:bad-generated"),
    });
    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    bad_generated,
                    207,
                )],
            },
        ),
        Err(ProofLoweringError::MissingGeneratedCitation { origin })
            if origin == missing_generated
    ));
}

#[test]
fn proof_lowering_rejects_wrong_owner_kind_and_assumed_terminal_goals() {
    let mut context_input = input_with_items(vec![
        item_seed("Owner", 199),
        CoreItemSeed::new(
            symbol("FunctorOwner"),
            CoreItemKind::Functor,
            "public",
            direct(200, 201),
            provenance("checker:item:functor-owner"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
    ]);
    context_input.variable_seeds = Vec::new();
    context_input.dependency_summaries = vec![
        CoreDependencySummary::new(
            external_symbol("ExternalTheorem"),
            CoreItemKind::Theorem,
            "public",
            provenance("checker:dependency:external-theorem"),
        ),
        CoreDependencySummary::new(
            external_symbol("ExternalFunctor"),
            CoreItemKind::Functor,
            "public",
            provenance("checker:dependency:external-functor"),
        ),
    ];
    let context = prepare_core_context(context_input).expect("context");
    let owner = context
        .item_registry()
        .id_for_symbol(&symbol("Owner"))
        .expect("owner");
    let functor_owner = context
        .item_registry()
        .id_for_symbol(&symbol("FunctorOwner"))
        .expect("functor owner");
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        Vec::new(),
        vec![formula_seed(CoreFormulaSeedKind::True, 201)],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proposition = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let terminal =
        ProofSkeletonSeed::Node(ProofNodeSeed::TerminalGoal(ProofTerminalGoalSeed::active(
            ProofFormulaRef::Thesis,
            "proof/assumed-terminal",
            "pkg::main::Owner.proof.assumed-terminal",
            direct(202, 203),
            provenance("checker:proof:assumed-terminal"),
        )));

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    functor_owner,
                    symbol("FunctorOwner"),
                    proposition,
                    CoreProofStatus::Open,
                    ProofSkeletonSeed::Node(ProofNodeSeed::Step {
                        label: None,
                        formula: ProofFormulaRef::Thesis,
                        justification: ProofJustificationSeed::new(
                            Vec::new(),
                            direct(203, 204),
                            provenance("checker:proof:functor:justification"),
                        ),
                        source: direct(204, 205),
                        provenance: provenance("checker:proof:functor"),
                    }),
                    205,
                )],
            },
        ),
        Err(ProofLoweringError::UnsupportedProofItemKind { owner: actual, .. })
            if actual == functor_owner
    ));

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    CoreItemId::new(99),
                    symbol("MissingOwner"),
                    proposition,
                    CoreProofStatus::Open,
                    proof_step_skeleton(208),
                    209,
                )],
            },
        ),
        Err(ProofLoweringError::MissingOwnerItem { owner })
            if owner == CoreItemId::new(99)
    ));

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("WrongOwnerSymbol"),
                    proposition,
                    CoreProofStatus::Open,
                    proof_step_skeleton(210),
                    211,
                )],
            },
        ),
        Err(ProofLoweringError::ProofSymbolMismatch { owner: actual, .. })
            if actual == owner
    ));

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![
                    proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        proof_step_skeleton(212),
                        213,
                    ),
                    proof_seed(
                        owner,
                        symbol("Owner"),
                        proposition,
                        CoreProofStatus::Open,
                        proof_step_skeleton(214),
                        215,
                    ),
                ],
            },
        ),
        Err(ProofLoweringError::DuplicateProofOwner { owner: actual })
            if actual == owner
    ));

    let external_theorem = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
        label: None,
        formula: ProofFormulaRef::Thesis,
        justification: ProofJustificationSeed::new(
            vec![CoreCitation::Symbol(external_symbol("ExternalTheorem"))],
            direct(216, 217),
            provenance("checker:proof:external-theorem:justification"),
        ),
        source: direct(217, 218),
        provenance: provenance("checker:proof:external-theorem"),
    });
    lower_proof_inputs(
        &context,
        &term_formula,
        &definitions,
        ProofLoweringInput {
            proofs: vec![proof_seed(
                owner,
                symbol("Owner"),
                proposition,
                CoreProofStatus::Assumed,
                external_theorem,
                218,
            )],
        },
    )
    .expect("external theorem citations from dependency summaries are accepted");

    let external_terminal = ProofSkeletonSeed::Node(ProofNodeSeed::TerminalGoal(
        ProofTerminalGoalSeed::active(
            ProofFormulaRef::Thesis,
            "proof/external-terminal",
            "pkg::main::Owner.proof.external-terminal",
            direct(219, 220),
            provenance("checker:proof:external-terminal"),
        )
        .with_citations(vec![CoreCitation::Symbol(external_symbol(
            "ExternalTheorem",
        ))]),
    ));
    let output = lower_proof_inputs(
        &context,
        &term_formula,
        &definitions,
        ProofLoweringInput {
            proofs: vec![proof_seed(
                owner,
                symbol("Owner"),
                proposition,
                CoreProofStatus::Conditional,
                external_terminal,
                220,
            )],
        },
    )
    .expect("external theorem terminal citations are accepted");
    let terminal_record = &output.terminal_obligations[0];
    let obligation = output
        .obligation_seeds
        .get(terminal_record.obligation)
        .expect("terminal obligation");
    let terminal_node = output
        .proof_nodes
        .get(terminal_record.node)
        .expect("terminal node");
    let external_citation = CoreCitation::Symbol(external_symbol("ExternalTheorem"));
    let CoreProofNodeKind::TerminalGoal { citations, .. } = &terminal_node.kind else {
        panic!("expected terminal goal");
    };

    assert_eq!(citations, &vec![external_citation.clone()]);
    assert_eq!(
        output.terminal_citations[0].citations,
        vec![external_citation]
    );
    assert_eq!(
        obligation
            .core_refs
            .iter()
            .filter(|reference| matches!(reference, CoreNodeRef::Item(_)))
            .count(),
        1
    );
    assert!(obligation.core_refs.contains(&CoreNodeRef::Item(owner)));

    let local_functor_citation = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
        label: None,
        formula: ProofFormulaRef::Thesis,
        justification: ProofJustificationSeed::new(
            vec![CoreCitation::Symbol(symbol("FunctorOwner"))],
            direct(221, 222),
            provenance("checker:proof:local-functor-citation:justification"),
        ),
        source: direct(222, 223),
        provenance: provenance("checker:proof:local-functor-citation"),
    });
    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    local_functor_citation,
                    223,
                )],
            },
        ),
        Err(ProofLoweringError::InvalidSymbolCitation { symbol: cited })
            if cited.as_ref() == &symbol("FunctorOwner")
    ));

    let external_functor_citation = ProofSkeletonSeed::Node(ProofNodeSeed::Step {
        label: None,
        formula: ProofFormulaRef::Thesis,
        justification: ProofJustificationSeed::new(
            vec![CoreCitation::Symbol(external_symbol("ExternalFunctor"))],
            direct(224, 225),
            provenance("checker:proof:external-functor-citation:justification"),
        ),
        source: direct(225, 226),
        provenance: provenance("checker:proof:external-functor-citation"),
    });
    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Open,
                    external_functor_citation,
                    226,
                )],
            },
        ),
        Err(ProofLoweringError::InvalidSymbolCitation { symbol: cited })
            if cited.as_ref() == &external_symbol("ExternalFunctor")
    ));

    assert!(matches!(
        lower_proof_inputs(
            &context,
            &term_formula,
            &definitions,
            ProofLoweringInput {
                proofs: vec![proof_seed(
                    owner,
                    symbol("Owner"),
                    proposition,
                    CoreProofStatus::Assumed,
                    terminal,
                    206,
                )],
            },
        ),
        Err(ProofLoweringError::AssumedProofCannotHaveTerminalGoals { owner: actual })
            if actual == owner
    ));
}

#[test]
fn algorithm_lowering_preserves_shells_contracts_pick_and_nested_order() {
    let x = CoreVarId::new(0);
    let y = CoreVarId::new(1);
    let z = CoreVarId::new(2);
    let result = CoreVarId::new(3);
    let ghost_pick = CoreVarId::new(4);
    let (context, owner) = context_with_algorithm_var_sorts(vec![
        (x, NormalizedVarSort::Term),
        (y, NormalizedVarSort::Term),
        (z, NormalizedVarSort::Term),
        (result, NormalizedVarSort::Term),
        (ghost_pick, NormalizedVarSort::Term),
    ]);
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        vec![
            term_seed(CoreTermSeedKind::Var(x), 230),
            term_seed(CoreTermSeedKind::Var(y), 231),
        ],
        vec![
            formula_seed(CoreFormulaSeedKind::True, 232),
            formula_seed(CoreFormulaSeedKind::False, 233),
            formula_seed(
                CoreFormulaSeedKind::TypePred {
                    subject: CoreTermSeedId::new(0),
                    ty: CoreTypePredicate::new("set"),
                },
                234,
            ),
        ],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proofs = empty_proof_output(&context, &term_formula, &definitions);
    let term_x = term_formula.term_map[&CoreTermSeedId::new(0)];
    let term_y = term_formula.term_map[&CoreTermSeedId::new(1)];
    let requires = term_formula.formula_map[&CoreFormulaSeedId::new(0)];
    let ensures = term_formula.formula_map[&CoreFormulaSeedId::new(1)];
    let invariant = term_formula.formula_map[&CoreFormulaSeedId::new(2)];
    let param = test_binder(x, Some(requires), 235);
    let result_binder = test_binder(result, Some(ensures), 236);
    let runtime_pick = test_binder(z, Some(invariant), 237);
    let ghost_pick_binder = test_binder(ghost_pick, Some(requires), 238);
    let mut seed = algorithm_seed(
        owner,
        symbol("Owner"),
        AlgorithmPayloadSeed::Statements(vec![
            AlgorithmStmtSeed::Let {
                binder: test_binder(y, Some(invariant), 239),
                value: Some(term_x),
                ghost: false,
                source: direct(240, 241),
                provenance: provenance("checker:algorithm:let"),
            },
            AlgorithmStmtSeed::Pick {
                binder: runtime_pick.clone(),
                witness_ty: Some(invariant),
                ghost: false,
                source: direct(241, 242),
                provenance: provenance("checker:algorithm:pick:runtime"),
            },
            AlgorithmStmtSeed::Pick {
                binder: ghost_pick_binder.clone(),
                witness_ty: Some(requires),
                ghost: true,
                source: direct(242, 243),
                provenance: provenance("checker:algorithm:pick:ghost"),
            },
            AlgorithmStmtSeed::If {
                condition: requires,
                then_body: vec![AlgorithmStmtSeed::Assert {
                    formula: ensures,
                    source: direct(243, 244),
                    provenance: provenance("checker:algorithm:if:assert"),
                }],
                else_body: vec![AlgorithmStmtSeed::Break {
                    source: direct(244, 245),
                    provenance: provenance("checker:algorithm:if:break"),
                }],
                source: direct(245, 246),
                provenance: provenance("checker:algorithm:if"),
            },
            AlgorithmStmtSeed::While {
                condition: ensures,
                invariants: vec![invariant],
                decreasing: vec![term_x],
                body: vec![AlgorithmStmtSeed::Continue {
                    source: direct(246, 247),
                    provenance: provenance("checker:algorithm:while:continue"),
                }],
                source: direct(247, 248),
                provenance: provenance("checker:algorithm:while"),
            },
            AlgorithmStmtSeed::Match {
                scrutinee: term_x,
                arms: vec![AlgorithmMatchArmSeed {
                    pattern: CoreProvenanceKey::new("case:some"),
                    body: vec![AlgorithmStmtSeed::Return {
                        value: Some(term_y),
                        source: direct(248, 249),
                        provenance: provenance("checker:algorithm:match:return"),
                    }],
                    provenance: provenance("checker:algorithm:match:arm"),
                }],
                source: direct(249, 250),
                provenance: provenance("checker:algorithm:match"),
            },
            AlgorithmStmtSeed::Assign {
                target: CorePlace::new("result"),
                value: term_y,
                source: direct(250, 251),
                provenance: provenance("checker:algorithm:assign"),
            },
            AlgorithmStmtSeed::Return {
                value: None,
                source: direct(251, 252),
                provenance: provenance("checker:algorithm:return"),
            },
        ]),
        252,
    );
    seed.params = vec![param.clone()];
    seed.result = Some(result_binder.clone());
    seed.contracts = CoreContractSet {
        requires: vec![requires],
        ensures: vec![ensures],
        invariants: vec![invariant],
        assertions: vec![ensures],
        decreasing: vec![term_x],
    };
    seed.ghost_effects = vec![
        GhostEffectKey::new("runtime-state"),
        GhostEffectKey::new("ghost-proof"),
    ];

    let output = lower_algorithm_inputs(
        &context,
        &term_formula,
        &proofs,
        AlgorithmLoweringInput {
            algorithms: vec![seed],
        },
    )
    .expect("algorithm lowering");
    let algorithm_id = output.algorithm_map[&owner];
    let algorithm = output.algorithms.get(algorithm_id).expect("algorithm");

    assert_eq!(algorithm_id, CoreAlgorithmId::new(0));
    assert_eq!(algorithm.item, owner);
    assert_eq!(algorithm.symbol, symbol("Owner"));
    assert_eq!(
        algorithm.source,
        expected_checker_source(252, 253, "checker:algorithm:252")
    );
    assert_eq!(algorithm.params, vec![param]);
    assert_eq!(algorithm.result.as_ref(), Some(&result_binder));
    assert_eq!(algorithm.contracts.requires, vec![requires]);
    assert_eq!(algorithm.contracts.ensures, vec![ensures]);
    assert_eq!(algorithm.contracts.invariants, vec![invariant]);
    assert_eq!(algorithm.contracts.assertions, vec![ensures]);
    assert_eq!(algorithm.contracts.decreasing, vec![term_x]);
    assert_eq!(
        algorithm.ghost_effects,
        vec![
            GhostEffectKey::new("runtime-state"),
            GhostEffectKey::new("ghost-proof")
        ]
    );
    assert_eq!(algorithm.statements.len(), 8);
    assert!(algorithm.diagnostics.is_empty());
    assert!(term_formula.generated_delta.is_empty());
    assert_eq!(
        output.source_map.algorithm_sources.len(),
        output.algorithm_statements.len()
    );
    for (_, statement) in output.algorithm_statements.iter() {
        assert_eq!(statement.owner, algorithm_id);
    }

    let let_statement = output
        .algorithm_statements
        .get(algorithm.statements[0])
        .expect("let statement");
    let expected_let_source = expected_checker_source(240, 241, "checker:algorithm:let");
    assert_eq!(let_statement.source, expected_let_source);
    assert_eq!(
        output.source_map.algorithm_sources[&algorithm.statements[0]],
        expected_let_source
    );
    let CoreAlgorithmStmtKind::Let {
        value,
        ghost,
        binder,
    } = &let_statement.kind
    else {
        panic!("expected let statement");
    };
    assert_eq!(binder.var, y);
    assert_eq!(*value, Some(term_x));
    assert!(!ghost);

    let pick_statement = output
        .algorithm_statements
        .get(algorithm.statements[1])
        .expect("runtime pick statement");
    assert_eq!(
        pick_statement.source,
        expected_checker_source(241, 242, "checker:algorithm:pick:runtime")
    );
    let CoreAlgorithmStmtKind::Pick {
        binder,
        witness_ty,
        ghost,
    } = &pick_statement.kind
    else {
        panic!("expected pick statement");
    };
    assert_eq!(binder, &runtime_pick);
    assert_eq!(*witness_ty, Some(invariant));
    assert!(!ghost);

    let ghost_pick_statement = output
        .algorithm_statements
        .get(algorithm.statements[2])
        .expect("ghost pick statement");
    let CoreAlgorithmStmtKind::Pick { binder, ghost, .. } = &ghost_pick_statement.kind else {
        panic!("expected ghost pick statement");
    };
    assert_eq!(binder, &ghost_pick_binder);
    assert!(ghost);

    let if_statement = output
        .algorithm_statements
        .get(algorithm.statements[3])
        .expect("if statement");
    assert_eq!(
        if_statement.source,
        expected_checker_source(245, 246, "checker:algorithm:if")
    );
    let CoreAlgorithmStmtKind::If {
        condition,
        then_body,
        else_body,
    } = &if_statement.kind
    else {
        panic!("expected if statement");
    };
    assert_eq!(*condition, requires);
    assert_eq!(then_body.len(), 1);
    assert_eq!(else_body.len(), 1);
    assert!(matches!(
        output
            .algorithm_statements
            .get(then_body[0])
            .expect("then assertion")
            .kind,
        CoreAlgorithmStmtKind::Assert { formula } if formula == ensures
    ));
    assert!(matches!(
        output
            .algorithm_statements
            .get(else_body[0])
            .expect("else break")
            .kind,
        CoreAlgorithmStmtKind::Break
    ));

    let while_statement = output
        .algorithm_statements
        .get(algorithm.statements[4])
        .expect("while statement");
    assert_eq!(
        while_statement.source,
        expected_checker_source(247, 248, "checker:algorithm:while")
    );
    let CoreAlgorithmStmtKind::While {
        condition,
        invariants,
        decreasing,
        body,
    } = &while_statement.kind
    else {
        panic!("expected while statement");
    };
    assert_eq!(*condition, ensures);
    assert_eq!(invariants, &vec![invariant]);
    assert_eq!(decreasing, &vec![term_x]);
    assert_eq!(body.len(), 1);
    assert!(matches!(
        output
            .algorithm_statements
            .get(body[0])
            .expect("while continue")
            .kind,
        CoreAlgorithmStmtKind::Continue
    ));

    let match_statement = output
        .algorithm_statements
        .get(algorithm.statements[5])
        .expect("match statement");
    let CoreAlgorithmStmtKind::Match { scrutinee, arms } = &match_statement.kind else {
        panic!("expected match statement");
    };
    assert_eq!(*scrutinee, term_x);
    assert_eq!(arms.len(), 1);
    assert_eq!(arms[0].pattern, CoreProvenanceKey::new("case:some"));
    assert_eq!(arms[0].body.len(), 1);
    assert!(matches!(
        output
            .algorithm_statements
            .get(arms[0].body[0])
            .expect("match return")
            .kind,
        CoreAlgorithmStmtKind::Return(Some(term)) if term == term_y
    ));

    assert!(matches!(
        &output
            .algorithm_statements
            .get(algorithm.statements[6])
            .expect("assignment")
            .kind,
        CoreAlgorithmStmtKind::Assign { target, value }
            if target == &CorePlace::new("result") && *value == term_y
    ));
    assert!(matches!(
        output
            .algorithm_statements
            .get(algorithm.statements[7])
            .expect("return")
            .kind,
        CoreAlgorithmStmtKind::Return(None)
    ));
    assert_step6_delta_valid(&context, &term_formula, &definitions, &proofs, &output);
}

#[test]
fn algorithm_lowering_missing_payload_records_error_statement_and_diagnostic() {
    let (context, owner) = context_with_algorithm_var_sorts(Vec::new());
    let term_formula = lower_test_terms_and_formulas(&context, owner, Vec::new(), Vec::new());
    let definitions = empty_definition_output(&context, &term_formula);
    let proofs = empty_proof_output(&context, &term_formula, &definitions);

    let output = lower_algorithm_inputs(
        &context,
        &term_formula,
        &proofs,
        AlgorithmLoweringInput {
            algorithms: vec![algorithm_seed(
                owner,
                symbol("Owner"),
                AlgorithmPayloadSeed::Missing(failed_site("algorithm-payload-missing", 260)),
                261,
            )],
        },
    )
    .expect("algorithm lowering");
    let algorithm_id = output.algorithm_map[&owner];
    let algorithm = output.algorithms.get(algorithm_id).expect("algorithm");
    let statement_id = algorithm.statements[0];
    let statement = output
        .algorithm_statements
        .get(statement_id)
        .expect("error statement");
    let expected_source =
        expected_checker_source(260, 261, "checker:failed:algorithm-payload-missing");
    assert_eq!(statement.source, expected_source);
    assert_eq!(
        output.source_map.algorithm_sources[&statement_id],
        expected_source
    );
    let CoreAlgorithmStmtKind::Error(diagnostic_id) = &statement.kind else {
        panic!("expected error statement");
    };
    let diagnostic_id = *diagnostic_id;
    let diagnostic = output
        .diagnostics
        .get(diagnostic_id)
        .expect("algorithm diagnostic");

    assert_eq!(algorithm.statements, vec![statement_id]);
    assert_eq!(algorithm.diagnostics, vec![diagnostic_id]);
    assert_eq!(statement.diagnostics, vec![diagnostic_id]);
    assert_eq!(diagnostic.class, CoreDiagnosticClass::AlgorithmShell);
    assert_eq!(diagnostic.severity, CoreDiagnosticSeverity::Error);
    assert_eq!(diagnostic.recovery, CoreDiagnosticRecovery::Fatal);
    assert_eq!(diagnostic.message_key.as_str(), "algorithm-payload-missing");
    assert_eq!(diagnostic.primary_source, expected_source);
    assert_eq!(
        diagnostic.owner,
        Some(CoreNodeRef::AlgorithmStmt(statement_id))
    );
    assert_step6_delta_valid(&context, &term_formula, &definitions, &proofs, &output);
}

#[test]
fn algorithm_lowering_malformed_statement_records_parent_diagnostic() {
    let (context, owner) = context_with_algorithm_var_sorts(Vec::new());
    let term_formula = lower_test_terms_and_formulas(&context, owner, Vec::new(), Vec::new());
    let definitions = empty_definition_output(&context, &term_formula);
    let proofs = empty_proof_output(&context, &term_formula, &definitions);

    let output = lower_algorithm_inputs(
        &context,
        &term_formula,
        &proofs,
        AlgorithmLoweringInput {
            algorithms: vec![algorithm_seed(
                owner,
                symbol("Owner"),
                AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Error(failed_site(
                    "algorithm-statement-malformed",
                    262,
                ))]),
                263,
            )],
        },
    )
    .expect("algorithm lowering");
    let algorithm_id = output.algorithm_map[&owner];
    let algorithm = output.algorithms.get(algorithm_id).expect("algorithm");
    let statement_id = algorithm.statements[0];
    let statement = output
        .algorithm_statements
        .get(statement_id)
        .expect("error statement");
    let expected_source =
        expected_checker_source(262, 263, "checker:failed:algorithm-statement-malformed");
    assert_eq!(statement.source, expected_source);
    assert_eq!(
        output.source_map.algorithm_sources[&statement_id],
        expected_source
    );
    let CoreAlgorithmStmtKind::Error(diagnostic_id) = statement.kind else {
        panic!("expected error statement");
    };
    let diagnostic = output
        .diagnostics
        .get(diagnostic_id)
        .expect("algorithm diagnostic");

    assert_eq!(algorithm.statements, vec![statement_id]);
    assert_eq!(algorithm.diagnostics, vec![diagnostic_id]);
    assert_eq!(statement.diagnostics, vec![diagnostic_id]);
    assert_eq!(diagnostic.class, CoreDiagnosticClass::AlgorithmShell);
    assert_eq!(diagnostic.severity, CoreDiagnosticSeverity::Error);
    assert_eq!(diagnostic.recovery, CoreDiagnosticRecovery::Fatal);
    assert_eq!(
        diagnostic.message_key.as_str(),
        "algorithm-statement-malformed"
    );
    assert_eq!(diagnostic.primary_source, expected_source);
    assert_eq!(
        diagnostic.owner,
        Some(CoreNodeRef::AlgorithmStmt(statement_id))
    );
    assert_step6_delta_valid(&context, &term_formula, &definitions, &proofs, &output);
}

#[test]
fn algorithm_lowering_rejects_owner_symbol_and_boundary_mismatches() {
    let context = prepare_core_context(input_with_items(vec![
        item_seed("TheoremOwner", 270),
        algorithm_item_seed("AlgorithmOwner", 271),
        CoreItemSeed::new(
            symbol("NoBoundary"),
            CoreItemKind::Algorithm,
            "public",
            direct(272, 273),
            provenance("checker:item:no-boundary"),
        ),
        CoreItemSeed::new(
            symbol("WrongBoundary"),
            CoreItemKind::Algorithm,
            "public",
            direct(273, 274),
            provenance("checker:item:wrong-boundary"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::DefinitionalItem),
    ]))
    .expect("context");
    let theorem_owner = context
        .item_registry()
        .id_for_symbol(&symbol("TheoremOwner"))
        .expect("theorem owner");
    let algorithm_owner = context
        .item_registry()
        .id_for_symbol(&symbol("AlgorithmOwner"))
        .expect("algorithm owner");
    let no_boundary = context
        .item_registry()
        .id_for_symbol(&symbol("NoBoundary"))
        .expect("no boundary");
    let wrong_boundary = context
        .item_registry()
        .id_for_symbol(&symbol("WrongBoundary"))
        .expect("wrong boundary");
    let term_formula =
        lower_test_terms_and_formulas(&context, algorithm_owner, Vec::new(), Vec::new());
    let definitions = empty_definition_output(&context, &term_formula);
    let proofs = empty_proof_output(&context, &term_formula, &definitions);
    let valid_seed = algorithm_seed(
        algorithm_owner,
        symbol("AlgorithmOwner"),
        AlgorithmPayloadSeed::Statements(Vec::new()),
        274,
    );

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![valid_seed.clone(), valid_seed],
            },
        ),
        Err(AlgorithmLoweringError::DuplicateAlgorithmOwner { owner })
            if owner == algorithm_owner
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    CoreItemId::new(99),
                    symbol("MissingOwner"),
                    AlgorithmPayloadSeed::Statements(Vec::new()),
                    275,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingOwnerItem { owner })
            if owner == CoreItemId::new(99)
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    theorem_owner,
                    symbol("TheoremOwner"),
                    AlgorithmPayloadSeed::Statements(Vec::new()),
                    276,
                )],
            },
        ),
        Err(AlgorithmLoweringError::UnsupportedAlgorithmItemKind { owner, kind })
            if owner == theorem_owner && kind == CoreItemKind::Theorem
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    algorithm_owner,
                    symbol("WrongOwnerSymbol"),
                    AlgorithmPayloadSeed::Statements(Vec::new()),
                    277,
                )],
            },
        ),
        Err(AlgorithmLoweringError::AlgorithmSymbolMismatch { owner, .. })
            if owner == algorithm_owner
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    no_boundary,
                    symbol("NoBoundary"),
                    AlgorithmPayloadSeed::Statements(Vec::new()),
                    278,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmBoundary { owner })
            if owner == no_boundary
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    wrong_boundary,
                    symbol("WrongBoundary"),
                    AlgorithmPayloadSeed::Statements(Vec::new()),
                    279,
                )],
            },
        ),
        Err(AlgorithmLoweringError::AlgorithmBoundaryMismatch { owner, kind })
            if owner == wrong_boundary && kind == DefinitionBoundaryKind::DefinitionalItem
    ));

    let mut skipped_context = context.clone();
    skipped_context
        .definition_boundaries
        .by_item
        .get_mut(&algorithm_owner)
        .expect("algorithm boundary")
        .status = DefinitionBoundaryStatus::Skipped;
    assert!(matches!(
        lower_algorithm_inputs(
            &skipped_context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    algorithm_owner,
                    symbol("AlgorithmOwner"),
                    AlgorithmPayloadSeed::Statements(Vec::new()),
                    280,
                )],
            },
        ),
        Err(AlgorithmLoweringError::AlgorithmBoundaryNotPending { owner, status })
            if owner == algorithm_owner && status == DefinitionBoundaryStatus::Skipped
    ));
}

#[test]
fn algorithm_lowering_rejects_invalid_binders_terms_formulas_and_targets() {
    let x = CoreVarId::new(0);
    let formula_var = CoreVarId::new(1);
    let (context, owner) = context_with_algorithm_var_sorts(vec![
        (x, NormalizedVarSort::Term),
        (formula_var, NormalizedVarSort::Formula),
    ]);
    let term_formula = lower_test_terms_and_formulas(
        &context,
        owner,
        vec![term_seed(CoreTermSeedKind::Var(x), 280)],
        vec![formula_seed(CoreFormulaSeedKind::True, 281)],
    );
    let definitions = empty_definition_output(&context, &term_formula);
    let proofs = empty_proof_output(&context, &term_formula, &definitions);
    let term = term_formula.term_map[&CoreTermSeedId::new(0)];
    let formula = term_formula.formula_map[&CoreFormulaSeedId::new(0)];

    let mut non_term_binder = algorithm_seed(
        owner,
        symbol("Owner"),
        AlgorithmPayloadSeed::Statements(Vec::new()),
        282,
    );
    non_term_binder.params = vec![test_binder(formula_var, None, 283)];
    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![non_term_binder],
            },
        ),
        Err(AlgorithmLoweringError::NonTermAlgorithmBinder { var, sort })
            if var == formula_var && sort == NormalizedVarSort::Formula
    ));

    let undeclared = CoreVarId::new(99);
    let mut undeclared_binder = algorithm_seed(
        owner,
        symbol("Owner"),
        AlgorithmPayloadSeed::Statements(Vec::new()),
        284,
    );
    undeclared_binder.result = Some(test_binder(undeclared, None, 285));
    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![undeclared_binder],
            },
        ),
        Err(AlgorithmLoweringError::UndeclaredAlgorithmBinder { var })
            if var == undeclared
    ));

    let missing_formula = CoreFormulaId::new(99);
    let mut bad_contract_formula = algorithm_seed(
        owner,
        symbol("Owner"),
        AlgorithmPayloadSeed::Statements(Vec::new()),
        286,
    );
    bad_contract_formula.contracts.requires = vec![missing_formula];
    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![bad_contract_formula],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
            if formula == missing_formula
    ));

    let missing_term = CoreTermId::new(99);
    let mut bad_contract_term = algorithm_seed(
        owner,
        symbol("Owner"),
        AlgorithmPayloadSeed::Statements(Vec::new()),
        287,
    );
    bad_contract_term.contracts.decreasing = vec![missing_term];
    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![bad_contract_term],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
            if term == missing_term
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::If {
                        condition: missing_formula,
                        then_body: Vec::new(),
                        else_body: Vec::new(),
                        source: direct(288, 289),
                        provenance: provenance("checker:algorithm:bad-if"),
                    }]),
                    289,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
            if formula == missing_formula
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::While {
                        condition: formula,
                        invariants: vec![missing_formula],
                        decreasing: Vec::new(),
                        body: Vec::new(),
                        source: direct(290, 291),
                        provenance: provenance("checker:algorithm:bad-while-invariant"),
                    }]),
                    291,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
            if formula == missing_formula
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::While {
                        condition: formula,
                        invariants: Vec::new(),
                        decreasing: vec![missing_term],
                        body: Vec::new(),
                        source: direct(292, 293),
                        provenance: provenance("checker:algorithm:bad-while-decreasing"),
                    }]),
                    293,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
            if term == missing_term
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Match {
                        scrutinee: missing_term,
                        arms: Vec::new(),
                        source: direct(294, 295),
                        provenance: provenance("checker:algorithm:bad-match"),
                    }]),
                    295,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
            if term == missing_term
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Return {
                        value: Some(missing_term),
                        source: direct(296, 297),
                        provenance: provenance("checker:algorithm:bad-return"),
                    }]),
                    297,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
            if term == missing_term
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Assign {
                        target: CorePlace::new("result"),
                        value: missing_term,
                        source: direct(298, 299),
                        provenance: provenance("checker:algorithm:missing-term"),
                    }]),
                    299,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmTerm { term })
            if term == missing_term
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Assert {
                        formula: missing_formula,
                        source: direct(300, 301),
                        provenance: provenance("checker:algorithm:missing-formula"),
                    }]),
                    301,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
            if formula == missing_formula
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Assign {
                        target: CorePlace::new(""),
                        value: term,
                        source: direct(302, 303),
                        provenance: provenance("checker:algorithm:bad-target"),
                    }]),
                    303,
                )],
            },
        ),
        Err(AlgorithmLoweringError::InvalidAlgorithmTarget { target })
            if target.as_str().is_empty()
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Pick {
                        binder: test_binder(x, None, 304),
                        witness_ty: Some(missing_formula),
                        ghost: false,
                        source: direct(304, 305),
                        provenance: provenance("checker:algorithm:bad-pick-witness"),
                    }]),
                    305,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
            if formula == missing_formula
    ));

    assert!(matches!(
        lower_algorithm_inputs(
            &context,
            &term_formula,
            &proofs,
            AlgorithmLoweringInput {
                algorithms: vec![algorithm_seed(
                    owner,
                    symbol("Owner"),
                    AlgorithmPayloadSeed::Statements(vec![AlgorithmStmtSeed::Pick {
                        binder: test_binder(x, Some(missing_formula), 306),
                        witness_ty: Some(formula),
                        ghost: false,
                        source: direct(306, 307),
                        provenance: provenance("checker:algorithm:bad-pick-binder"),
                    }]),
                    307,
                )],
            },
        ),
        Err(AlgorithmLoweringError::MissingAlgorithmFormula { formula })
            if formula == missing_formula
    ));
}

#[test]
fn context_assigns_item_ids_in_deterministic_source_order() {
    let input = input_with_items(vec![item_seed("Later", 20), item_seed("Earlier", 0)]);

    let context = prepare_core_context(input).expect("context");
    let earlier = context
        .item_registry()
        .id_for_symbol(&symbol("Earlier"))
        .expect("earlier id");
    let later = context
        .item_registry()
        .id_for_symbol(&symbol("Later"))
        .expect("later id");

    assert_eq!(earlier.index(), 0);
    assert_eq!(later.index(), 1);
    assert!(
        context
            .definition_boundaries()
            .get_by_item(earlier)
            .is_some()
    );
    assert_eq!(
        context
            .worklist()
            .entries()
            .iter()
            .map(|entry| &entry.kind)
            .collect::<Vec<_>>(),
        vec![
            &ElaborationWorkItemKind::Item(earlier),
            &ElaborationWorkItemKind::Item(later)
        ]
    );
}

#[test]
fn missing_dependency_summary_is_diagnostic_not_source_inspection() {
    let missing = external_symbol("UnavailableDependency");
    let input = input_with_items(vec![
        item_seed("UsesMissing", 0).with_dependencies(vec![missing]),
    ]);

    let context = prepare_core_context(input).expect("context with diagnostic");
    let item = context
        .item_registry()
        .id_for_symbol(&symbol("UsesMissing"))
        .expect("item id");
    let item_row = context.item_registry().items().get(item).expect("item row");
    let resolution = context
        .item_registry()
        .dependencies(item)
        .expect("dependency resolution");
    let diagnostic = context
        .diagnostics()
        .get(item_row.diagnostics[0])
        .expect("diagnostic");

    assert_eq!(item_row.status, CoreItemStatus::Partial);
    assert_eq!(resolution.missing.len(), 1);
    assert_eq!(
        diagnostic.class,
        CoreDiagnosticClass::UnresolvedSemanticInput
    );
    assert_eq!(
        diagnostic.message_key.as_str(),
        "missing-dependency-summary"
    );
    assert!(matches!(
        context.worklist().entries()[0].status,
        ElaborationWorkStatus::Skipped
    ));
}

#[test]
fn item_registry_uses_canonical_symbol_ids_without_raw_spelling_identity() {
    let canonical = SymbolId::new(
        module_id(),
        LocalSymbolId::new("CanonicalLocal"),
        FullyQualifiedName::new("pkg::main::CanonicalFqn"),
    );
    let raw_spelling = "source wrote a different spelling";
    let input = input_with_items(vec![CoreItemSeed::new(
        canonical.clone(),
        CoreItemKind::Predicate,
        "public",
        direct(0, raw_spelling.len()),
        provenance("checker:item:canonical"),
    )]);

    let context = prepare_core_context(input).expect("context");
    let id = context
        .item_registry()
        .id_for_symbol(&canonical)
        .expect("canonical lookup");
    let item = context.item_registry().items().get(id).expect("item");

    assert_eq!(item.symbol, canonical);
    assert!(!format!("{:?}", context.item_registry()).contains(raw_spelling));
}

#[test]
fn dependency_resolution_uses_exact_canonical_symbol_identity() {
    let local_shared = symbol("Shared");
    let external_shared = external_symbol("Shared");
    let mut input = input_with_items(vec![
        CoreItemSeed::new(
            local_shared.clone(),
            CoreItemKind::Predicate,
            "public",
            direct(0, 3),
            provenance("checker:item:local-shared"),
        ),
        item_seed("UsesBoth", 10)
            .with_dependencies(vec![external_shared.clone(), local_shared.clone()]),
    ]);
    input.dependency_summaries = vec![CoreDependencySummary::new(
        external_shared.clone(),
        CoreItemKind::Predicate,
        "public",
        provenance("checker:dependency:external-shared"),
    )];

    let context = prepare_core_context(input).expect("context");
    let local_id = context
        .item_registry()
        .id_for_symbol(&local_shared)
        .expect("local shared id");
    let uses_id = context
        .item_registry()
        .id_for_symbol(&symbol("UsesBoth"))
        .expect("uses id");
    let resolution = context
        .item_registry()
        .dependencies(uses_id)
        .expect("dependency resolution");

    assert_eq!(resolution.local, vec![local_id]);
    assert_eq!(resolution.external, vec![external_shared]);
    assert!(resolution.missing.is_empty());
}

#[test]
fn definition_boundaries_are_initialized_before_body_lowering() {
    let input = input_with_items(vec![
        item_seed("RecursiveA", 0).with_dependencies(vec![symbol("RecursiveB")]),
        item_seed("RecursiveB", 10).with_dependencies(vec![symbol("RecursiveA")]),
    ]);

    let context = prepare_core_context(input).expect("context");
    let a = context
        .definition_boundaries()
        .get_by_symbol(&symbol("RecursiveA"))
        .expect("boundary a");
    let b = context
        .definition_boundaries()
        .get_by_symbol(&symbol("RecursiveB"))
        .expect("boundary b");

    assert_eq!(a.status, DefinitionBoundaryStatus::PendingBody);
    assert_eq!(b.status, DefinitionBoundaryStatus::PendingBody);
    assert_eq!(a.kind, DefinitionBoundaryKind::Theorem);
    assert_eq!(b.kind, DefinitionBoundaryKind::Theorem);
}

#[test]
fn failed_checker_sites_are_preserved_as_error_work_items() {
    let failed_site =
        CheckerSiteSummary::failed_overload(OverloadResolutionId::new(7), direct(5, 9));
    let resolved = ResolvedTypedAstSummary::new(source_id(), module_id())
        .with_checker_sites(vec![failed_site.clone()]);
    let input = CoreContextInput::new(resolved);

    let context = prepare_core_context(input).expect("context");
    let entry = &context.worklist().entries()[0];
    let diagnostic = context
        .diagnostics()
        .get(entry.diagnostics[0])
        .expect("diagnostic");

    assert_eq!(
        entry.kind,
        ElaborationWorkItemKind::CheckerSite(failed_site.kind)
    );
    assert_eq!(entry.status, ElaborationWorkStatus::Error);
    assert_eq!(diagnostic.class, CoreDiagnosticClass::UnsupportedLowering);
    assert_eq!(
        diagnostic.message_key.as_str(),
        "checker-error-site-preserved"
    );
}

#[test]
fn recovered_checker_sites_are_preserved_as_skipped_work_items() {
    let recovered_diagnostic = ResolvedTypedDiagnosticId::new(3);
    let note_diagnostic = ResolvedTypedDiagnosticId::new(4);
    let recovered_site = CheckerSiteSummary {
        kind: CheckerSiteKind::RecoveredNode {
            node: ResolvedTypedNodeId::new(2),
            recovery: ResolvedNodeRecovery::Recovered,
        },
        source: direct(1, 2),
        diagnostics: vec![recovered_diagnostic],
        severity: CheckerSiteSeverity::Warning,
    };
    let note_site = CheckerSiteSummary {
        kind: CheckerSiteKind::CheckerDiagnostic {
            diagnostic: note_diagnostic,
        },
        source: direct(3, 4),
        diagnostics: vec![note_diagnostic],
        severity: CheckerSiteSeverity::Note,
    };
    let resolved = ResolvedTypedAstSummary::new(source_id(), module_id())
        .with_checker_sites(vec![note_site, recovered_site]);
    let input = CoreContextInput::new(resolved);

    let context = prepare_core_context(input).expect("context");
    let entries = context.worklist().entries();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].status, ElaborationWorkStatus::Skipped);
    assert_eq!(entries[0].checker_diagnostics, vec![recovered_diagnostic]);
    assert_eq!(entries[1].status, ElaborationWorkStatus::Skipped);
    assert_eq!(entries[1].checker_diagnostics, vec![note_diagnostic]);
    assert_eq!(
        context
            .diagnostics()
            .get(entries[0].diagnostics[0])
            .expect("warning diagnostic")
            .message_key
            .as_str(),
        "checker-recovered-site-preserved"
    );
    assert_eq!(
        context
            .diagnostics()
            .get(entries[1].diagnostics[0])
            .expect("note diagnostic")
            .message_key
            .as_str(),
        "checker-note-site-preserved"
    );
}

#[test]
fn source_map_and_generated_origin_registry_are_initialized() {
    let owner = symbol("Owner");
    let origin_key = GeneratedOriginKey::new("choice:Owner:0");
    let mut input = input_with_items(vec![
        CoreItemSeed::new(
            owner.clone(),
            CoreItemKind::GeneratedDefinition,
            "private",
            direct(0, 4),
            provenance("checker:item:owner"),
        )
        .with_definition_boundary(DefinitionBoundaryKind::GeneratedDefinition),
    ]);
    input.generated_origin_seeds = vec![
        GeneratedOriginSeed::new(
            owner.clone(),
            GeneratedOriginKind::StableChoice,
            origin_key.clone(),
            direct(10, 12),
            provenance("checker:generated:choice"),
        )
        .with_params(vec![CoreVarId::new(0)])
        .with_evidence(vec![CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "choice-evidence",
        )]),
    ];

    let context = prepare_core_context(input).expect("context");
    let owner_id = context
        .item_registry()
        .id_for_symbol(&owner)
        .expect("owner id");
    let generated_id = context
        .generated_origins()
        .get_by_key(owner_id, GeneratedOriginKind::StableChoice, &origin_key)
        .expect("generated id");

    assert!(context.source_map().item_sources.contains_key(&owner_id));
    assert!(
        context
            .source_map()
            .generated_sources
            .contains_key(&generated_id)
    );
    assert!(context.source_map().term_sources.is_empty());
    assert_eq!(context.generated_origins().table().len(), 1);
}

#[test]
fn binder_and_variable_seeds_prepare_binder_metadata_context() {
    let var = CoreVarId::new(3);
    let mut input = CoreContextInput::new(summary());
    input.variable_seeds = vec![
        CoreVariableSeed::new(
            var,
            NormalizedVarClass::Free,
            "term-binder",
            NormalizedVarSort::Term,
            provenance("checker:var:3"),
        )
        .with_type_facts(vec![
            TypeFactId::new(2),
            TypeFactId::new(1),
            TypeFactId::new(1),
        ]),
    ];
    input.binder_seeds = vec![CoreBinderSeed::new(
        var,
        direct(15, 16),
        provenance("checker:binder:3"),
    )];

    let context = prepare_core_context(input).expect("context");

    assert!(context.binder_context().free_variables.contains(&var));
    assert_eq!(
        context.binder_context().variable_classes.get(&var),
        Some(&NormalizedVarClass::Free)
    );
    assert_eq!(
        context.binder_context().variable_roles.get(&var),
        Some(&CoreVarRole::new("term-binder"))
    );
    assert_eq!(
        context.binder_context().variable_sorts.get(&var),
        Some(&NormalizedVarSort::Term)
    );
    assert_eq!(
        context.binder_type_facts().get(&var),
        Some(&vec![TypeFactId::new(1), TypeFactId::new(2)])
    );
    assert!(context.binder_sources().get(var).is_some());
}

#[test]
fn binder_source_requires_declared_variable_metadata() {
    let var = CoreVarId::new(9);
    let mut input = CoreContextInput::new(summary());
    input.binder_seeds = vec![CoreBinderSeed::new(
        var,
        direct(20, 21),
        provenance("checker:binder:undeclared"),
    )];

    assert!(matches!(
        prepare_core_context(input),
        Err(CoreContextError::UndeclaredBinderVariable { var: actual }) if actual == var
    ));
}

#[test]
fn current_and_external_module_inputs_are_kept_distinct() {
    let external = external_symbol("ForeignItem");
    let foreign_item_input = input_with_items(vec![CoreItemSeed::new(
        external.clone(),
        CoreItemKind::Theorem,
        "public",
        direct(0, 3),
        provenance("checker:item:foreign"),
    )]);

    assert!(matches!(
        prepare_core_context(foreign_item_input),
        Err(CoreContextError::ForeignItemSeed { symbol, .. }) if symbol.as_ref() == &external
    ));

    let current = symbol("CurrentSummary");
    let mut current_summary_input = CoreContextInput::new(summary());
    current_summary_input.dependency_summaries = vec![CoreDependencySummary::new(
        current.clone(),
        CoreItemKind::Predicate,
        "public",
        provenance("checker:dependency:current"),
    )];

    assert!(matches!(
        prepare_core_context(current_summary_input),
        Err(CoreContextError::CurrentModuleDependencySummary { symbol }) if symbol.as_ref() == &current
    ));
}

#[test]
fn unprovenanced_checker_seed_is_rejected() {
    let fabricated = CheckerOwnedProvenance {
        entries: Vec::new(),
    };
    let input = input_with_items(vec![CoreItemSeed::new(
        symbol("Fabricated"),
        CoreItemKind::Theorem,
        "public",
        direct(0, 3),
        fabricated,
    )]);

    assert!(matches!(
        prepare_core_context(input),
        Err(CoreContextError::MissingProvenance { input: "item seed" })
    ));
}

#[test]
fn non_checker_owned_seed_phase_is_rejected() {
    let generated_phase = CheckerOwnedProvenance {
        entries: vec![CoreProvenance::new(
            CoreProvenancePhase::Generated,
            "generated-only",
        )],
    };
    let input = input_with_items(vec![CoreItemSeed::new(
        symbol("GeneratedOnly"),
        CoreItemKind::Theorem,
        "public",
        direct(0, 3),
        generated_phase,
    )]);

    assert!(matches!(
        prepare_core_context(input),
        Err(CoreContextError::UnsupportedProvenancePhase {
            input: "item seed",
            phase: CoreProvenancePhase::Generated
        })
    ));
}

#[test]
fn generated_origin_evidence_must_be_checker_owned() {
    let owner = symbol("Owner");
    let mut input = input_with_items(vec![CoreItemSeed::new(
        owner.clone(),
        CoreItemKind::GeneratedDefinition,
        "private",
        direct(0, 4),
        provenance("checker:item:owner"),
    )]);
    input.generated_origin_seeds = vec![
        GeneratedOriginSeed::new(
            owner,
            GeneratedOriginKind::StableChoice,
            "choice:bad-evidence",
            direct(10, 12),
            provenance("checker:generated:choice"),
        )
        .with_evidence(vec![CoreProvenance::new(
            CoreProvenancePhase::Generated,
            "generated-only",
        )]),
    ];

    assert!(matches!(
        prepare_core_context(input),
        Err(CoreContextError::UnsupportedProvenancePhase {
            input: "generated origin evidence",
            phase: CoreProvenancePhase::Generated
        })
    ));
}

fn task33c4c8_variable_seed(var: CoreVarId, role: &str) -> CoreVariableSeed {
    CoreVariableSeed::new(
        var,
        NormalizedVarClass::Free,
        role,
        NormalizedVarSort::Term,
        provenance(format!("checker:task33c4c8:var:{}", var.index()).as_str()),
    )
}

fn task33c4c8_context_with_complete_used_inventory() -> CoreContext {
    let owner = symbol("CaptureOwner");
    let mut input = input_with_items(vec![CoreItemSeed::new(
        owner.clone(),
        CoreItemKind::Functor,
        "public",
        direct(20, 30),
        provenance("checker:task33c4c8:owner"),
    )]);
    for index in [2, 9] {
        let var = CoreVarId::new(index);
        input
            .variable_seeds
            .push(task33c4c8_variable_seed(var, "existing-term"));
        input.binder_seeds.push(CoreBinderSeed::new(
            var,
            direct(30 + index, 31 + index),
            provenance(format!("checker:task33c4c8:binder:{index}").as_str()),
        ));
    }
    input.generated_origin_seeds.push(
        GeneratedOriginSeed::new(
            owner,
            GeneratedOriginKind::StableChoice,
            "task33c4c8-existing-origin",
            direct(50, 55),
            provenance("checker:task33c4c8:origin"),
        )
        .with_params(vec![CoreVarId::new(9)]),
    );
    let mut context = prepare_core_context(input).expect("Task33C4C8 unit context");
    context
        .binder_context
        .frames
        .push(crate::binder_normalization::BinderFrame::new(
            0,
            CoreVarId::new(2),
            "existing-frame",
            direct(60, 61),
        ));
    context
}

#[test]
fn task33c4c8_core_context_shape_is_complete_and_fail_closed() {
    let context = task33c4c8_context_with_complete_used_inventory();
    let used = validate_core_context_shape(&context, &BTreeSet::new())
        .expect("complete used-variable inventory");
    assert_eq!(used, BTreeSet::from([CoreVarId::new(2), CoreVarId::new(9)]));
    assert_eq!(
        allocate_capture_core_vars(&used, 2).expect("checked max-plus-one allocation"),
        [CoreVarId::new(10), CoreVarId::new(11)]
    );

    let mut missing_sort = context.clone();
    missing_sort
        .binder_context
        .variable_sorts
        .remove(&CoreVarId::new(2));
    assert_eq!(
        validate_core_context_shape(&missing_sort, &BTreeSet::new()),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
    );

    let mut reserved_role = context.clone();
    reserved_role.binder_context.variable_roles.insert(
        CoreVarId::new(2),
        CoreVarRole::new(NESTED_FRAENKEL_CAPTURE_CORE_ROLE),
    );
    assert_eq!(
        validate_core_context_shape(&reserved_role, &BTreeSet::new()),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
    );

    let mut undeclared_frame = context.clone();
    undeclared_frame.binder_context.frames[0].original_var = CoreVarId::new(99);
    assert_eq!(
        validate_core_context_shape(&undeclared_frame, &BTreeSet::new()),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
    );

    let mut undeclared_generated_param = context.clone();
    undeclared_generated_param
        .generated_origins
        .table
        .get_mut(GeneratedOriginId::new(0))
        .expect("existing origin")
        .params = vec![CoreVarId::new(99)];
    assert_eq!(
        validate_core_context_shape(&undeclared_generated_param, &BTreeSet::new()),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
    );

    let mut mismatched_binder_record = context;
    mismatched_binder_record
        .binder_sources
        .by_var
        .get_mut(&CoreVarId::new(2))
        .expect("binder record")
        .var = CoreVarId::new(9);
    assert_eq!(
        validate_core_context_shape(&mismatched_binder_record, &BTreeSet::new()),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
    );
}

#[test]
fn task33c4c8_owner_origin_and_item_checks_are_exact() {
    let owner_range = range(20, 30);
    let context = prepare_core_context(input_with_items(vec![CoreItemSeed::new(
        symbol("CaptureOwner"),
        CoreItemKind::Functor,
        "public",
        CoreSourceRef::direct(owner_range),
        provenance("checker:task33c4c8:owner"),
    )]))
    .expect("Task33C4C8 owner context");
    let owner_id = context
        .item_registry
        .id_for_symbol(&symbol("CaptureOwner"))
        .expect("owner item");
    let origin = SemanticOrigin::new(
        source_id(),
        module_id(),
        SourceAnchor::Range(owner_range),
        Vec::new(),
    );
    assert_eq!(validate_owner_origin(&context, &origin), Ok(owner_range));
    assert_eq!(
        validate_owner_item(&context, &symbol("CaptureOwner"), owner_range),
        Ok(owner_id)
    );

    let source_allocator = InMemorySessionIdAllocator::new();
    let source_snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "09".repeat(32)
    ))
    .expect("valid alternate snapshot id");
    let _discarded = source_allocator
        .next_source_id(source_snapshot)
        .expect("first alternate source id");
    let alternate_source = source_allocator
        .next_source_id(source_snapshot)
        .expect("second alternate source id");
    let wrong_source_origin = SemanticOrigin::new(
        alternate_source,
        module_id(),
        SourceAnchor::Range(owner_range),
        Vec::new(),
    );
    let wrong_module_origin = SemanticOrigin::new(
        source_id(),
        external_module_id(),
        SourceAnchor::Range(owner_range),
        Vec::new(),
    );
    let point_origin = SemanticOrigin::new(
        source_id(),
        module_id(),
        SourceAnchor::Point {
            source_id: source_id(),
            offset: owner_range.start,
        },
        Vec::new(),
    );
    for invalid in [
        wrong_source_origin,
        wrong_module_origin,
        point_origin,
        origin.recovered(),
    ] {
        assert_eq!(
            validate_owner_origin(&context, &invalid),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation)
        );
    }

    let mut invalid_status = context.clone();
    invalid_status
        .item_registry
        .items
        .get_mut(owner_id)
        .expect("owner")
        .status = CoreItemStatus::Partial;
    let mut wrong_range = context.clone();
    wrong_range
        .item_registry
        .items
        .get_mut(owner_id)
        .expect("owner")
        .source = direct(21, 30);
    wrong_range
        .source_map
        .item_sources
        .insert(owner_id, direct(21, 30));
    let mut missing_source_map = context.clone();
    missing_source_map.source_map.item_sources.remove(&owner_id);
    let mut near_match = context;
    near_match
        .item_registry
        .items
        .get_mut(owner_id)
        .expect("owner")
        .symbol = SymbolId::new(
        module_id(),
        LocalSymbolId::new("CaptureOwner"),
        FullyQualifiedName::new("pkg::main::CaptureOwnerDisplayNearMatch"),
    );
    for invalid in [invalid_status, wrong_range, missing_source_map, near_match] {
        assert_eq!(
            validate_owner_item(&invalid, &symbol("CaptureOwner"), owner_range),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation)
        );
    }
}

fn task33c4c8_capture_validation_fixture() -> (
    CoreContext,
    SourceNestedFraenkelCaptureCoreVariableTable,
    Vec<CaptureAssociation>,
    BTreeSet<CoreVarId>,
) {
    let mut input = CoreContextInput::new(summary());
    for index in [2, 9, 10, 11] {
        let var = CoreVarId::new(index);
        let capture = index.checked_sub(10);
        let role = if capture.is_some() {
            NESTED_FRAENKEL_CAPTURE_CORE_ROLE
        } else {
            "existing-term"
        };
        input
            .variable_seeds
            .push(task33c4c8_variable_seed(var, role));
        let (source, binder_provenance) = if let Some(capture) = capture {
            let capture_id = SourceNestedFraenkelCaptureGraphCaptureId::new(capture);
            let key = capture_provenance_key(capture_id);
            (
                CoreSourceRef::direct(range(100 + capture, 101 + capture)).with_provenance(vec![
                    CoreProvenance::new(CoreProvenancePhase::Checker, key.clone()),
                ]),
                CheckerOwnedProvenance::checker(key),
            )
        } else {
            (
                direct(70 + index, 71 + index),
                provenance(format!("checker:task33c4c8:binder:{index}").as_str()),
            )
        };
        input
            .binder_seeds
            .push(CoreBinderSeed::new(var, source, binder_provenance));
    }
    let context = prepare_core_context(input).expect("Task33C4C8 capture validation context");
    let associations = vec![
        CaptureAssociation {
            capture: SourceNestedFraenkelCaptureGraphCaptureId::new(0),
            generator: mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphGeneratorId::new(1),
            resolver_binding: FraenkelGeneratorVariableBindingId::new(1),
            binder_range: range(100, 101),
        },
        CaptureAssociation {
            capture: SourceNestedFraenkelCaptureGraphCaptureId::new(1),
            generator: mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphGeneratorId::new(2),
            resolver_binding: FraenkelGeneratorVariableBindingId::new(2),
            binder_range: range(101, 102),
        },
    ];
    let mut table = SourceNestedFraenkelCaptureCoreVariableTable::empty();
    for (association, core_var) in associations
        .iter()
        .zip([CoreVarId::new(10), CoreVarId::new(11)])
    {
        table.rows.insert(
            association.capture,
            SourceNestedFraenkelCaptureCoreVariable {
                capture: association.capture,
                generator: association.generator,
                resolver_binding: association.resolver_binding,
                core_var,
            },
        );
    }
    let allowed = capture_vars(&table);
    let used = validate_core_context_shape(&context, &allowed)
        .expect("Task33C4C8 capture validation used inventory");
    (context, table, associations, used)
}

#[test]
fn task33c4c8_capture_row_postvalidation_is_fail_closed() {
    let (context, table, associations, used) = task33c4c8_capture_validation_fixture();
    assert_eq!(
        validate_capture_rows(&context, &table, &used, &associations),
        Ok(())
    );

    let mut missing = table.clone();
    missing
        .rows
        .remove(&SourceNestedFraenkelCaptureGraphCaptureId::new(0));
    let mut extra = table.clone();
    extra.rows.insert(
        SourceNestedFraenkelCaptureGraphCaptureId::new(2),
        SourceNestedFraenkelCaptureCoreVariable {
            capture: SourceNestedFraenkelCaptureGraphCaptureId::new(2),
            generator: mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphGeneratorId::new(3),
            resolver_binding: FraenkelGeneratorVariableBindingId::new(3),
            core_var: CoreVarId::new(12),
        },
    );
    let mut reordered = table.clone();
    reordered
        .rows
        .get_mut(&SourceNestedFraenkelCaptureGraphCaptureId::new(0))
        .expect("capture row")
        .core_var = CoreVarId::new(11);
    reordered
        .rows
        .get_mut(&SourceNestedFraenkelCaptureGraphCaptureId::new(1))
        .expect("capture row")
        .core_var = CoreVarId::new(10);
    let mut mismatched = table.clone();
    mismatched
        .rows
        .get_mut(&SourceNestedFraenkelCaptureGraphCaptureId::new(1))
        .expect("capture row")
        .resolver_binding = FraenkelGeneratorVariableBindingId::new(1);
    for invalid in [missing, extra, reordered, mismatched] {
        assert_eq!(
            validate_capture_rows(&context, &invalid, &used, &associations),
            Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation)
        );
    }

    let mut duplicate_var = table.clone();
    duplicate_var
        .rows
        .get_mut(&SourceNestedFraenkelCaptureGraphCaptureId::new(1))
        .expect("capture row")
        .core_var = CoreVarId::new(10);
    assert_eq!(
        validate_capture_rows(&context, &duplicate_var, &used, &associations),
        Err(
            SourceNestedFraenkelCaptureCoreContextError::CoreVariableCollision {
                var: CoreVarId::new(10)
            }
        )
    );

    let mut bad_provenance = context.clone();
    bad_provenance
        .binder_sources
        .by_var
        .get_mut(&CoreVarId::new(10))
        .expect("capture binder")
        .source
        .provenance
        .clear();
    assert_eq!(
        validate_capture_rows(&bad_provenance, &table, &used, &associations),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation)
    );

    let mut stale_role = context;
    stale_role
        .binder_context
        .variable_roles
        .insert(CoreVarId::new(11), CoreVarRole::new("stale-capture-role"));
    assert_eq!(
        validate_capture_rows(&stale_role, &table, &used, &associations),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation)
    );
}

#[test]
fn task33c4c8_allocator_is_zero_based_checked_and_deterministic() {
    assert_eq!(
        allocate_capture_core_vars(&BTreeSet::new(), 2).expect("empty allocation"),
        [CoreVarId::new(0), CoreVarId::new(1)]
    );
    let used = BTreeSet::from([CoreVarId::new(2), CoreVarId::new(9)]);
    assert_eq!(
        allocate_capture_core_vars(&used, 2).expect("populated allocation"),
        [CoreVarId::new(10), CoreVarId::new(11)]
    );
    assert_eq!(
        allocate_capture_core_vars(&BTreeSet::from([CoreVarId::new(usize::MAX)]), 1),
        Err(SourceNestedFraenkelCaptureCoreContextError::CoreVariableAllocationOverflow)
    );
    assert_eq!(
        allocate_capture_core_vars(&BTreeSet::from([CoreVarId::new(usize::MAX - 1)]), 2),
        Err(SourceNestedFraenkelCaptureCoreContextError::CoreVariableAllocationOverflow)
    );
}

#[test]
fn source_binding_core_context_builds_exact_checker_order_and_metadata() {
    let context =
        prepare_core_context(CoreContextInput::new(summary())).expect("empty Core context");
    let first = SourceBindingCoreContextProducer::build(context, source_binding_env())
        .expect("source binding Core handoff");
    let second = SourceBindingCoreContextProducer::build(
        prepare_core_context(CoreContextInput::new(summary())).expect("empty Core context"),
        source_binding_env(),
    )
    .expect("deterministic source binding Core handoff");

    assert_eq!(first, second);
    assert_eq!(
        first.debug_text(),
        "source-binding-core-context-v1|module=pkg.main|bindings=2|variables=0:0,1:1"
    );
    let rows = first.variables().iter().collect::<Vec<_>>();
    let [(first_binding, first_row), (second_binding, second_row)] = rows.as_slice() else {
        panic!("two source binding rows expected");
    };
    assert_eq!(*first_binding, BindingId::new(0));
    assert_eq!(*second_binding, BindingId::new(1));
    assert_eq!(first_row.binding(), BindingId::new(0));
    assert_eq!(second_row.binding(), BindingId::new(1));
    assert_eq!(first_row.core_var(), CoreVarId::new(0));
    assert_eq!(second_row.core_var(), CoreVarId::new(1));
    assert_eq!(first.variables().get(BindingId::new(0)), Some(*first_row));
    assert_eq!(first.variables().get(BindingId::new(1)), Some(*second_row));

    let context = first.context();
    for (binding, row, declaration_range, role) in [
        (
            BindingId::new(0),
            *first_row,
            range(10, 11),
            SOURCE_BINDING_CORE_RESERVED_ROLE,
        ),
        (
            BindingId::new(1),
            *second_row,
            range(20, 21),
            SOURCE_BINDING_CORE_RESERVED_ROLE,
        ),
    ] {
        let var = row.core_var();
        let key = source_binding_provenance_key(binding);
        let expected_provenance = CoreProvenance::new(CoreProvenancePhase::Checker, key.clone());
        assert_eq!(
            context.binder_context().variable_classes.get(&var),
            Some(&NormalizedVarClass::Free)
        );
        assert_eq!(
            context.binder_context().variable_sorts.get(&var),
            Some(&NormalizedVarSort::Term)
        );
        assert_eq!(
            context.binder_context().variable_roles.get(&var),
            Some(&CoreVarRole::new(role))
        );
        assert_eq!(context.binder_type_facts().get(&var), Some(&Vec::new()));
        let record = context
            .binder_sources()
            .get(var)
            .expect("source binding binder source");
        assert_eq!(
            record.source,
            CoreSourceRef::direct(declaration_range).with_provenance(vec![expected_provenance])
        );
        assert_eq!(record.provenance, CheckerOwnedProvenance::checker(key));
    }

    let reserved = first
        .binding_env()
        .bindings()
        .get(BindingId::new(0))
        .expect("reserved binding");
    assert_eq!(reserved.kind, BindingKind::ReservedVariable);
    assert_eq!(reserved.status, BindingStatus::Reserved);
    assert_eq!(reserved.owner_context, BindingContextId::new(0));
    let second_reserved = first
        .binding_env()
        .bindings()
        .get(BindingId::new(1))
        .expect("second reserved binding");
    assert_eq!(second_reserved.kind, BindingKind::ReservedVariable);
    assert_eq!(second_reserved.status, BindingStatus::Reserved);
    assert_eq!(second_reserved.owner_context, BindingContextId::new(0));
    assert_eq!(
        second_reserved.identity,
        BinderIdentity::ReservedVariable {
            spelling: "y".to_owned(),
            declaration_range: range(20, 21),
        }
    );
    let module_context = first
        .binding_env()
        .contexts()
        .get(BindingContextId::new(0))
        .expect("module context");
    assert_eq!(
        module_context.bindings,
        vec![BindingId::new(0), BindingId::new(1)]
    );
    assert_eq!(module_context.visible_bindings, module_context.bindings);
}

#[test]
fn source_binding_core_context_allocates_above_complete_used_inventory() {
    let handoff = SourceBindingCoreContextProducer::build(
        task33c4c8_context_with_complete_used_inventory(),
        source_binding_env(),
    )
    .expect("source binding Core handoff above existing variables");
    let rows = handoff
        .variables()
        .iter()
        .map(|(binding, row)| (binding, row.core_var()))
        .collect::<Vec<_>>();
    assert_eq!(
        rows,
        vec![
            (BindingId::new(0), CoreVarId::new(10)),
            (BindingId::new(1), CoreVarId::new(11))
        ]
    );
    assert!(
        handoff
            .context()
            .binder_context()
            .free_variables
            .is_superset(&BTreeSet::from([
                CoreVarId::new(2),
                CoreVarId::new(9),
                CoreVarId::new(10),
                CoreVarId::new(11),
            ]))
    );
    for (binding, var, role) in [
        (
            BindingId::new(0),
            CoreVarId::new(10),
            SOURCE_BINDING_CORE_RESERVED_ROLE,
        ),
        (
            BindingId::new(1),
            CoreVarId::new(11),
            SOURCE_BINDING_CORE_RESERVED_ROLE,
        ),
    ] {
        assert_eq!(
            handoff.context().binder_sources().get(var).unwrap().var,
            var
        );
        assert_eq!(
            handoff.context().binder_context().variable_roles.get(&var),
            Some(&CoreVarRole::new(role))
        );
        assert_eq!(handoff.variables().get(binding).unwrap().core_var(), var);
    }
}

#[test]
fn source_binding_core_context_postvalidation_rejects_malformed_association() {
    let context =
        prepare_core_context(CoreContextInput::new(summary())).expect("empty Core context");
    let baseline = SourceBindingCoreContextProducer::build(context, source_binding_env())
        .expect("source binding Core handoff");

    let mut missing = baseline.clone();
    missing.variables.rows.remove(0);
    assert_eq!(
        missing.validate(),
        Err(SourceBindingCoreContextError::InvalidCoreContext)
    );

    let mut extra_orphan = baseline.clone();
    extra_orphan.variables.rows.push((
        BindingId::new(99),
        SourceBindingCoreVariable {
            binding: BindingId::new(99),
            core_var: CoreVarId::new(99),
        },
    ));
    assert_eq!(
        extra_orphan.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );

    let mut duplicate_core_var = baseline.clone();
    duplicate_core_var.variables.rows[1].1.core_var = CoreVarId::new(0);
    duplicate_core_var
        .context
        .binder_context
        .variable_roles
        .insert(CoreVarId::new(1), CoreVarRole::new("existing-term"));
    assert_eq!(
        duplicate_core_var.validate(),
        Err(SourceBindingCoreContextError::CoreVariableCollision {
            var: CoreVarId::new(0)
        })
    );

    let mut reordered = baseline.clone();
    reordered.variables.rows.swap(0, 1);
    assert_eq!(
        reordered.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );

    let mut stale = baseline.clone();
    stale.variables.rows[0].1.binding = BindingId::new(1);
    assert_eq!(
        stale.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );

    let mut mismatched_key = baseline.clone();
    mismatched_key.variables.rows[0].0 = BindingId::new(1);
    assert_eq!(
        mismatched_key.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );

    let mut wrong_role = baseline.clone();
    wrong_role
        .context
        .binder_context
        .variable_roles
        .insert(CoreVarId::new(0), CoreVarRole::new("wrong-role"));
    assert_eq!(
        wrong_role.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );

    let mut wrong_class = baseline.clone();
    wrong_class
        .context
        .binder_context
        .variable_classes
        .insert(CoreVarId::new(0), NormalizedVarClass::Schematic);
    assert_eq!(
        wrong_class.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );

    let mut wrong_sort = baseline.clone();
    wrong_sort
        .context
        .binder_context
        .variable_sorts
        .insert(CoreVarId::new(0), NormalizedVarSort::Formula);
    assert_eq!(
        wrong_sort.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );

    let mut wrong_range = baseline.clone();
    wrong_range
        .context
        .binder_sources
        .by_var
        .get_mut(&CoreVarId::new(0))
        .expect("reserved binder source")
        .source
        .anchor = CoreSourceAnchor::SourceRange(range(11, 12));
    assert_eq!(
        wrong_range.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );

    let mut wrong_provenance = baseline.clone();
    wrong_provenance
        .context
        .binder_sources
        .by_var
        .get_mut(&CoreVarId::new(0))
        .expect("reserved binder source")
        .provenance = CheckerOwnedProvenance::checker("wrong-source-binding-provenance");
    assert_eq!(
        wrong_provenance.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );

    let mut nonempty_facts = baseline;
    nonempty_facts
        .context
        .binder_type_facts
        .insert(CoreVarId::new(0), vec![TypeFactId::new(0)]);
    assert_eq!(
        nonempty_facts.validate(),
        Err(SourceBindingCoreContextError::InvalidBindingAssociation)
    );
}

#[test]
fn source_binding_core_context_rejects_mismatch_unsupported_and_overflow() {
    let source_mismatch = source_binding_env_with_one(
        alternate_source_id(),
        module_id(),
        BindingKind::ReservedVariable,
        BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: SourceRange {
                source_id: alternate_source_id(),
                start: 10,
                end: 11,
            },
        },
        BindingStatus::Reserved,
    );
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_mismatch,
        ),
        Err(SourceBindingCoreContextError::EnvironmentMismatch)
    );

    let module_mismatch = source_binding_env_with_one(
        source_id(),
        external_module_id(),
        BindingKind::ReservedVariable,
        BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: range(10, 11),
        },
        BindingStatus::Reserved,
    );
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            module_mismatch,
        ),
        Err(SourceBindingCoreContextError::EnvironmentMismatch)
    );

    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            empty_source_binding_env(),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let mut diagnostics = BindingDiagnosticTable::new();
    diagnostics.insert(BindingDiagnosticDraft {
        source_range: Some(range(1, 2)),
        class: BindingDiagnosticClass::UnsupportedSourceShape,
        severity: BindingDiagnosticSeverity::Error,
        message_key: "source-binding-test-diagnostic".to_owned(),
        recovery: BindingDiagnosticRecovery::Degraded,
    });
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Normal,
                source_binding_reserved_draft(),
                source_binding_second_reserved_draft(),
                diagnostics,
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Recovered,
                source_binding_reserved_draft(),
                source_binding_second_reserved_draft(),
                BindingDiagnosticTable::new(),
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Degraded,
                source_binding_reserved_draft(),
                source_binding_second_reserved_draft(),
                BindingDiagnosticTable::new(),
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let mut recovered_binding = source_binding_reserved_draft();
    recovered_binding.recovery = BindingRecoveryState::Recovered;
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Normal,
                recovered_binding,
                source_binding_second_reserved_draft(),
                BindingDiagnosticTable::new(),
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let mut degraded_binding = source_binding_reserved_draft();
    degraded_binding.recovery = BindingRecoveryState::Degraded;
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Normal,
                degraded_binding,
                source_binding_second_reserved_draft(),
                BindingDiagnosticTable::new(),
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let mut degraded_status = source_binding_reserved_draft();
    degraded_status.status = BindingStatus::Degraded;
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Normal,
                degraded_status,
                source_binding_second_reserved_draft(),
                BindingDiagnosticTable::new(),
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let mut captured_binding = source_binding_reserved_draft();
    captured_binding.captured =
        CapturedFreeVariables::new(vec![BinderIdentity::ReservedVariable {
            spelling: "captured".to_owned(),
            declaration_range: range(30, 31),
        }]);
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Normal,
                captured_binding,
                source_binding_second_reserved_draft(),
                BindingDiagnosticTable::new(),
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let mut status_mismatch = source_binding_reserved_draft();
    status_mismatch.status = BindingStatus::Active;
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Normal,
                status_mismatch,
                source_binding_second_reserved_draft(),
                BindingDiagnosticTable::new(),
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let mut omitted_status = source_binding_reserved_draft();
    omitted_status.status = BindingStatus::Omitted;
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Normal,
                omitted_status,
                source_binding_second_reserved_draft(),
                BindingDiagnosticTable::new(),
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let mut binding_diagnostics = BindingDiagnosticTable::new();
    let binding_diagnostic = binding_diagnostics.insert(BindingDiagnosticDraft {
        source_range: Some(range(10, 11)),
        class: BindingDiagnosticClass::UnsupportedSourceShape,
        severity: BindingDiagnosticSeverity::Error,
        message_key: "source-binding-test-row-diagnostic".to_owned(),
        recovery: BindingDiagnosticRecovery::Degraded,
    });
    let mut diagnostic_binding = source_binding_reserved_draft();
    diagnostic_binding.diagnostics.push(binding_diagnostic);
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            source_binding_env_with_options(
                BindingContextRecovery::Normal,
                diagnostic_binding,
                source_binding_second_reserved_draft(),
                binding_diagnostics,
            ),
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let unsupported = source_binding_env_with_one(
        source_id(),
        module_id(),
        BindingKind::LocalAbbreviation,
        BinderIdentity::ResolverLocal {
            scope: LocalTermScope::new(vec![0]),
            ordinal: 0,
            declaration_range: range(10, 11),
        },
        BindingStatus::Active,
    );
    assert_eq!(
        SourceBindingCoreContextProducer::build(
            prepare_core_context(CoreContextInput::new(summary())).expect("Core context"),
            unsupported,
        ),
        Err(SourceBindingCoreContextError::InvalidBindingEnvironment)
    );

    let mut invalid_existing_input = CoreContextInput::new(summary());
    invalid_existing_input
        .variable_seeds
        .push(CoreVariableSeed::new(
            CoreVarId::new(7),
            NormalizedVarClass::Free,
            "existing-term",
            NormalizedVarSort::Term,
            provenance("checker:source-binding:invalid-existing"),
        ));
    let mut invalid_existing_context =
        prepare_core_context(invalid_existing_input).expect("invalid-existing context");
    invalid_existing_context
        .binder_context
        .variable_sorts
        .remove(&CoreVarId::new(7));
    assert_eq!(
        SourceBindingCoreContextProducer::build(invalid_existing_context, source_binding_env()),
        Err(SourceBindingCoreContextError::InvalidCoreContext)
    );

    let mut unauthenticated_role_input = CoreContextInput::new(summary());
    unauthenticated_role_input
        .variable_seeds
        .push(CoreVariableSeed::new(
            CoreVarId::new(7),
            NormalizedVarClass::Free,
            SOURCE_BINDING_CORE_RESERVED_ROLE,
            NormalizedVarSort::Term,
            provenance("checker:source-binding:unauthenticated-role"),
        ));
    let unauthenticated_role_context =
        prepare_core_context(unauthenticated_role_input).expect("reserved-role context");
    assert_eq!(
        SourceBindingCoreContextProducer::build(unauthenticated_role_context, source_binding_env()),
        Err(SourceBindingCoreContextError::InvalidCoreContext)
    );

    let mut overflow_input = CoreContextInput::new(summary());
    overflow_input.variable_seeds.push(CoreVariableSeed::new(
        CoreVarId::new(usize::MAX),
        NormalizedVarClass::Free,
        "existing-term",
        NormalizedVarSort::Term,
        provenance("checker:source-binding:overflow"),
    ));
    let overflow_context =
        prepare_core_context(overflow_input).expect("overflow source binding context");
    assert_eq!(
        SourceBindingCoreContextProducer::build(overflow_context, source_binding_env()),
        Err(SourceBindingCoreContextError::CoreVariableAllocationOverflow)
    );
}
