use mizar_core::{
    binder_normalization::{
        BinderContext, NormalizedVarClass, NormalizedVarSort, alpha_equivalent_formulas,
        canonical_formula, normalize_core_formula,
    },
    control_flow::{
        ControlFlowOutput, ObligationSeedHandoff, build_control_flow_ir,
        build_obligation_seed_handoff,
    },
    core_ir::*,
};
use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SymbolId};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    SourceId, SourceRange,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CoreFixture {
    core: CoreIr,
    quantified: CoreFormulaId,
    renamed_quantified: CoreFormulaId,
    binder_context_vars: [CoreVarId; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PipelineObservation {
    core: CoreIr,
    flow: ControlFlowOutput,
    handoff: ObligationSeedHandoff,
    core_debug: String,
    flow_debug: String,
    handoff_debug: String,
}

fn source_id() -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "20".repeat(32)
    ))
    .expect("valid snapshot id");
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id")
}

fn source(source_id: SourceId, start: usize) -> CoreSourceRef {
    CoreSourceRef::direct(SourceRange {
        source_id,
        start,
        end: start + 1,
    })
}

fn module_id() -> ModuleId {
    ModuleId::new(PackageId::new("pkg"), ModulePath::new("determinism"))
}

fn symbol(name: &str) -> SymbolId {
    SymbolId::new(
        module_id(),
        LocalSymbolId::new(name),
        FullyQualifiedName::new(format!("pkg::determinism::{name}")),
    )
}

fn role(name: &str) -> CoreVarRole {
    CoreVarRole::new(name)
}

fn ty(name: &str) -> CoreTypePredicate {
    CoreTypePredicate::new(name)
}

fn binder(
    source_id: SourceId,
    var: CoreVarId,
    role_name: &str,
    source_name: &str,
    start: usize,
    guard: Option<CoreFormulaId>,
) -> CoreBinder {
    CoreBinder {
        var,
        role: role(role_name),
        ty_guard: guard,
        source_name: Some(source_name.to_owned()),
        source: source(source_id, start),
    }
}

fn push_item(
    parts: &mut CoreIrParts,
    symbol_name: &str,
    kind: CoreItemKind,
    start: usize,
) -> CoreItemId {
    let item_source = source(parts.source_id, start);
    let item = parts.items.insert(CoreItem::new(
        symbol(symbol_name),
        kind,
        "public",
        item_source.clone(),
    ));
    parts.source_map.item_sources.insert(item, item_source);
    item
}

fn push_term(parts: &mut CoreIrParts, kind: CoreTermKind, start: usize) -> CoreTermId {
    let term_source = source(parts.source_id, start);
    let term = parts.terms.insert(CoreTerm::new(kind, term_source.clone()));
    parts.source_map.term_sources.insert(term, term_source);
    term
}

fn push_formula(parts: &mut CoreIrParts, kind: CoreFormulaKind, start: usize) -> CoreFormulaId {
    let formula_source = source(parts.source_id, start);
    let formula = parts
        .formulas
        .insert(CoreFormula::new(kind, formula_source.clone()));
    parts
        .source_map
        .formula_sources
        .insert(formula, formula_source);
    formula
}

fn push_stmt(
    parts: &mut CoreIrParts,
    owner: CoreAlgorithmId,
    kind: CoreAlgorithmStmtKind,
    start: usize,
) -> CoreAlgorithmStmtId {
    let statement_source = source(parts.source_id, start);
    let statement = parts.algorithm_statements.insert(CoreAlgorithmStmt {
        owner,
        kind,
        source: statement_source.clone(),
        diagnostics: Vec::new(),
    });
    parts
        .source_map
        .algorithm_sources
        .insert(statement, statement_source);
    statement
}

fn push_seed(
    parts: &mut CoreIrParts,
    owner: CoreItemId,
    goal: CoreFormulaId,
    context: Vec<CoreFormulaId>,
    path: &str,
    start: usize,
) -> ObligationSeedId {
    let seed_source = source(parts.source_id, start);
    let seed = parts.obligation_seeds.insert(ObligationSeed {
        owner,
        kind: ObligationSeedKind::TheoremProof,
        goal: Some(goal),
        context,
        local_path: LocalProofOrProgramPath::new(path),
        label: None,
        semantic_origin: NormalizedSemanticOrigin::new(format!("pkg::determinism::{path}")),
        provenance: vec![
            CoreProvenance::new(CoreProvenancePhase::Generated, "z-after"),
            CoreProvenance::new(CoreProvenancePhase::ProofSkeleton, "a-before"),
        ],
        source: seed_source.clone(),
        core_refs: vec![CoreNodeRef::Formula(goal)],
        status: ObligationSeedStatus::Active,
        diagnostics: Vec::new(),
    });
    parts
        .source_map
        .obligation_sources
        .insert(seed, seed_source);
    seed
}

fn empty_parts() -> CoreIrParts {
    let source_id = source_id();
    CoreIrParts {
        source_id,
        module_id: module_id(),
        items: CoreItemTable::new(),
        terms: CoreTermTable::new(),
        formulas: CoreFormulaTable::new(),
        definitions: CoreDefinitionTable::new(),
        proofs: CoreProofTable::new(),
        proof_nodes: CoreProofNodeTable::new(),
        algorithms: CoreAlgorithmTable::new(),
        algorithm_statements: CoreAlgorithmStmtTable::new(),
        generated: GeneratedOriginTable::new(),
        obligation_seeds: ObligationSeedTable::new(),
        source_map: CoreSourceMap::new(),
        diagnostics: CoreDiagnosticTable::new(),
    }
}

fn build_core_fixture() -> CoreFixture {
    let mut parts = empty_parts();
    let source_id = parts.source_id;
    let theorem_item = push_item(&mut parts, "ThDeterministic", CoreItemKind::Theorem, 0);
    let algorithm_item = push_item(&mut parts, "AlgDeterministic", CoreItemKind::Algorithm, 4);

    let param_var = CoreVarId::new(0);
    let local_var = CoreVarId::new(1);
    let result_var = CoreVarId::new(2);
    let ghost_var = CoreVarId::new(3);
    let quantified_var = CoreVarId::new(10);
    let renamed_quantified_var = CoreVarId::new(11);

    let param_term = push_term(&mut parts, CoreTermKind::Var(param_var), 10);
    let local_term = push_term(&mut parts, CoreTermKind::Var(local_var), 11);
    let result_term = push_term(&mut parts, CoreTermKind::Var(result_var), 12);
    let zero_term = push_term(&mut parts, CoreTermKind::Const(symbol("Zero")), 13);
    let bound_term = push_term(&mut parts, CoreTermKind::Var(quantified_var), 14);
    let renamed_bound_term = push_term(&mut parts, CoreTermKind::Var(renamed_quantified_var), 15);
    let tuple_term = push_term(
        &mut parts,
        CoreTermKind::Tuple(vec![param_term, local_term, zero_term]),
        16,
    );

    let condition = push_formula(
        &mut parts,
        CoreFormulaKind::Atom {
            predicate: symbol("P"),
            args: vec![param_term],
        },
        20,
    );
    let invariant = push_formula(
        &mut parts,
        CoreFormulaKind::TypePred {
            subject: local_term,
            ty: ty("Nat"),
        },
        21,
    );
    let equality = push_formula(
        &mut parts,
        CoreFormulaKind::Equals {
            left: local_term,
            right: result_term,
        },
        22,
    );
    let bound_atom = push_formula(
        &mut parts,
        CoreFormulaKind::Atom {
            predicate: symbol("P"),
            args: vec![bound_term],
        },
        23,
    );
    let renamed_bound_atom = push_formula(
        &mut parts,
        CoreFormulaKind::Atom {
            predicate: symbol("P"),
            args: vec![renamed_bound_term],
        },
        24,
    );
    let bound_guard = push_formula(
        &mut parts,
        CoreFormulaKind::TypePred {
            subject: bound_term,
            ty: ty("Nat"),
        },
        25,
    );
    let renamed_bound_guard = push_formula(
        &mut parts,
        CoreFormulaKind::TypePred {
            subject: renamed_bound_term,
            ty: ty("Nat"),
        },
        26,
    );
    let quantified = push_formula(
        &mut parts,
        CoreFormulaKind::Forall {
            binders: vec![binder(
                source_id,
                quantified_var,
                "term",
                "x",
                27,
                Some(bound_guard),
            )],
            body: bound_atom,
        },
        28,
    );
    let renamed_quantified = push_formula(
        &mut parts,
        CoreFormulaKind::Forall {
            binders: vec![binder(
                source_id,
                renamed_quantified_var,
                "term",
                "renamed",
                29,
                Some(renamed_bound_guard),
            )],
            body: renamed_bound_atom,
        },
        30,
    );

    let algorithm_source = source(source_id, 40);
    let contracts = CoreContractSet {
        requires: vec![condition],
        ensures: vec![equality],
        invariants: vec![quantified],
        assertions: vec![renamed_quantified],
        decreasing: vec![tuple_term],
    };
    let algorithm = parts.algorithms.insert(CoreAlgorithm {
        item: algorithm_item,
        symbol: symbol("AlgDeterministic"),
        params: vec![binder(
            source_id,
            param_var,
            "local:const",
            "input",
            41,
            None,
        )],
        result: Some(binder(
            source_id,
            result_var,
            "local:var",
            "result",
            42,
            None,
        )),
        contracts,
        statements: Vec::new(),
        ghost_effects: vec![GhostEffectKey::new("ghost:erased-temp")],
        source: algorithm_source,
        diagnostics: Vec::new(),
    });

    let let_stmt = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::Let {
            binder: binder(source_id, local_var, "local:var", "local", 50, None),
            value: Some(param_term),
            ghost: false,
        },
        51,
    );
    let then_assign = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::Assign {
            target: CorePlace::new("result"),
            value: local_term,
        },
        52,
    );
    let else_pick = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::Pick {
            binder: binder(source_id, ghost_var, "local:ghost-var", "ghost", 53, None),
            witness_ty: Some(invariant),
            ghost: true,
        },
        54,
    );
    let if_stmt = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::If {
            condition,
            then_body: vec![then_assign],
            else_body: vec![else_pick],
        },
        55,
    );
    let loop_assign = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::Assign {
            target: CorePlace::new("result"),
            value: zero_term,
        },
        56,
    );
    let loop_assert = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::Assert { formula: equality },
        57,
    );
    let while_stmt = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::While {
            condition,
            invariants: vec![invariant],
            decreasing: vec![tuple_term],
            body: vec![loop_assign, loop_assert],
        },
        58,
    );
    let first_arm_assign = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::Assign {
            target: CorePlace::new("result"),
            value: param_term,
        },
        59,
    );
    let second_arm_assign = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::Assign {
            target: CorePlace::new("result"),
            value: local_term,
        },
        60,
    );
    let match_stmt = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::Match {
            scrutinee: tuple_term,
            arms: vec![
                CoreAlgorithmMatchArm {
                    pattern: CoreProvenanceKey::new("first-arm"),
                    body: vec![first_arm_assign],
                },
                CoreAlgorithmMatchArm {
                    pattern: CoreProvenanceKey::new("second-arm"),
                    body: vec![second_arm_assign],
                },
            ],
        },
        61,
    );
    let return_stmt = push_stmt(
        &mut parts,
        algorithm,
        CoreAlgorithmStmtKind::Return(Some(result_term)),
        62,
    );
    parts
        .algorithms
        .get_mut(algorithm)
        .expect("algorithm row")
        .statements = vec![let_stmt, if_stmt, while_stmt, match_stmt, return_stmt];

    push_seed(
        &mut parts,
        theorem_item,
        quantified,
        vec![condition],
        "proof/z",
        70,
    );
    push_seed(
        &mut parts,
        theorem_item,
        renamed_quantified,
        vec![condition],
        "proof/a",
        70,
    );

    CoreFixture {
        core: CoreIr::try_new(parts).expect("deterministic core fixture validates"),
        quantified,
        renamed_quantified,
        binder_context_vars: [local_var, quantified_var, renamed_quantified_var],
    }
}

fn binder_context(vars: [CoreVarId; 3]) -> BinderContext {
    let mut context = BinderContext::new();
    for var in vars {
        context.declare_variable(
            var,
            NormalizedVarClass::Free,
            role("term"),
            NormalizedVarSort::Term,
        );
    }
    context
}

fn observe_pipeline() -> PipelineObservation {
    let fixture = build_core_fixture();
    let first_flow = build_control_flow_ir(&fixture.core);
    let second_flow = build_control_flow_ir(&fixture.core);
    assert_eq!(
        first_flow, second_flow,
        "CFG construction is stable in-process"
    );
    assert_eq!(first_flow.debug_text(), second_flow.debug_text());

    let first_handoff = build_obligation_seed_handoff(&fixture.core, &first_flow);
    let second_handoff = build_obligation_seed_handoff(&fixture.core, &second_flow);
    assert_eq!(
        first_handoff, second_handoff,
        "obligation seed handoff is stable in-process"
    );
    assert_eq!(first_handoff.debug_text(), second_handoff.debug_text());

    let core_debug = fixture.core.debug_text();
    let flow_debug = first_flow.debug_text();
    let handoff_debug = first_handoff.debug_text();

    PipelineObservation {
        core: fixture.core,
        flow: first_flow,
        handoff: first_handoff,
        core_debug,
        flow_debug,
        handoff_debug,
    }
}

fn canonicalized_quantifiers() -> (String, String, bool) {
    let fixture = build_core_fixture();
    let context = binder_context(fixture.binder_context_vars);
    let direct_source = source(fixture.core.source_id(), 80);

    let first = normalize_core_formula(&fixture.core, fixture.quantified, &context, &direct_source)
        .expect("first formula normalizes");
    let second =
        normalize_core_formula(&fixture.core, fixture.quantified, &context, &direct_source)
            .expect("same formula normalizes again");
    assert_eq!(
        first, second,
        "raw core normalization is stable across repeated runs"
    );

    let renamed = normalize_core_formula(
        &fixture.core,
        fixture.renamed_quantified,
        &context,
        &direct_source,
    )
    .expect("renamed formula normalizes");
    let first_canonical =
        canonical_formula(&first, &context, &direct_source).expect("first canonicalizes");
    let second_canonical =
        canonical_formula(&second, &context, &direct_source).expect("second canonicalizes");
    let renamed_canonical =
        canonical_formula(&renamed, &context, &direct_source).expect("renamed canonicalizes");

    assert_eq!(
        first_canonical, second_canonical,
        "canonical binder numbering is stable across repeated runs"
    );
    let alpha_equivalent = alpha_equivalent_formulas(&first, &renamed, &context, &direct_source)
        .expect("alpha equivalence check succeeds");

    (
        format!("{first_canonical:?}"),
        format!("{renamed_canonical:?}"),
        alpha_equivalent,
    )
}

fn assert_ordered(text: &str, first: &str, second: &str) {
    let first_index = text
        .find(first)
        .unwrap_or_else(|| panic!("missing {first}"));
    let second_index = text
        .find(second)
        .unwrap_or_else(|| panic!("missing {second}"));
    assert!(
        first_index < second_index,
        "{first} should appear before {second}"
    );
}

#[test]
fn public_api_pipeline_is_deterministic_across_fresh_runs() {
    let first = observe_pipeline();
    let second = observe_pipeline();

    assert_eq!(
        first, second,
        "fresh public-API fixture runs are byte-stable"
    );
    assert!(first.core_debug.starts_with("core-ir-debug-v1\n"));
    assert!(
        first
            .flow_debug
            .starts_with("control-flow-output-debug-v1\n")
    );
    assert!(
        first
            .handoff_debug
            .starts_with("obligation-seed-handoff-debug-v1\n")
    );
    assert!(first.flow_debug.contains("flow-map"));
    assert!(first.flow_debug.contains("LoopInvariant"));
    assert!(first.handoff_debug.contains("AlgorithmContract"));
    assert!(first.handoff_debug.contains("GhostErasure"));
    assert_ordered(&first.handoff_debug, "proof/a", "proof/z");
}

#[test]
fn binder_canonicalization_is_deterministic_across_fresh_core_rebuilds() {
    let first = canonicalized_quantifiers();
    let second = canonicalized_quantifiers();

    assert_eq!(
        first, second,
        "fresh CoreIr rebuilds produce identical canonical quantifier output"
    );
    assert_eq!(
        first.0, first.1,
        "renamed quantified formulas share the same canonical form"
    );
    assert!(first.2, "renamed quantified formulas are alpha-equivalent");
}
