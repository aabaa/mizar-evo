#[test]
fn task258b4a_real_route_publishes_exact_composite_statement_root() {
    use mizar_checker::{
        source_composite_formula::{
            SourceCompositeFormulaId, SourceCompositeFormulaKind, SourceFormulaRootId,
            SourceFormulaRootOwnership,
        },
        source_statement::{
            SourceStatementCandidateFactKind,
            SourceStatementContextId, SourceStatementFormulaTarget, SourceStatementId,
            SourceStatementKind, SourceStatementRecovery, SourceTheoremOwnerId, SourceTheoremRole,
            SourceTheoremStatus,
        },
    };

    assert_eq!(SOURCE_STATEMENT_B4A_TEXT.len(), 80);
    assert_eq!(&SOURCE_STATEMENT_B4A_TEXT.as_bytes()[78..], &[10, 10]);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B4A_TEXT),
        "826e2b93bac42772e1a99bbfaac17af19623927c5dde6578869b4b2bc4cadd56"
    );
    let (ast, module, symbols, output) = task258b4a_output(265_000);
    assert_eq!(ast.nodes().len(), 26);
    assert_eq!(ast.root().expect("surface root").index(), 25);
    assert!(task258b4a_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B4A_TEXT,
        SourceStatementB4ASurfaceMutation::None,
    ));
    for index in 0..26 {
        for mutation in [
            SourceStatementB4ASurfaceMutation::NodeKind(index),
            SourceStatementB4ASurfaceMutation::NodeRange(index),
            SourceStatementB4ASurfaceMutation::NodeRecovery(index),
            SourceStatementB4ASurfaceMutation::NodeChildren(index),
        ] {
            assert!(
                !task258b4a_surface_profile_with_mutation_for_test(
                    &ast,
                    SOURCE_STATEMENT_B4A_TEXT,
                    mutation,
                ),
                "surface mutation {mutation:?} must fail"
            );
        }
    }
    assert!(!task258b4a_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B4A_TEXT,
        SourceStatementB4ASurfaceMutation::RootIdentity,
    ));
    for index in 0..SOURCE_STATEMENT_B4A_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B4A_TEXT.as_bytes().to_vec();
        bytes[index] ^= 1;
        let changed = String::from_utf8(bytes).expect("ASCII source mutation");
        assert!(!task258b4a_surface_profile_with_mutation_for_test(
            &ast, &changed, SourceStatementB4ASurfaceMutation::None,
        ), "source byte {index}");
        assert!(source_statement_b4a_output_with_mutation(
            &ast, module.clone(), &symbols, &changed, |_| panic!("near miss reached upper capture"),
        ).is_none());
    }
    let resolver = source_statement_b4a_resolver_env_for_test(
        &ast,
        &module,
        &symbols,
        SOURCE_STATEMENT_B4A_TEXT,
    )
    .expect("Task258B4A resolver provenance");
    assert_eq!(
        (
            resolver.symbols().len(),
            resolver.labels().len(),
            resolver.definitions().len(),
            resolver.contributions().len(),
            resolver.imports().len(),
        ),
        (1, 1, 1, 1, 0)
    );
    let labels = resolver.labels().visible_candidates(
        &mizar_resolve::env::NamespacePath::new(module.path().as_str()),
        "FormulaQuantifierBoundUsePayloadBoundary",
    );
    let [label] = labels.as_slice() else {
        panic!("one Task258B4A theorem label")
    };
    assert_eq!(
        (
            label.kind(),
            label.visibility(),
            label.export_status(),
            label.contribution().index(),
            label.origin().source_id(),
            label.origin().module_id(),
            label.origin().anchor(),
            label.origin().structural_path(),
            label.origin().import_edge(),
            label.origin().is_recovered(),
        ),
        (
            mizar_resolve::resolved_ast::LabelKind::Theorem,
            mizar_resolve::env::Visibility::Public,
            mizar_resolve::env::ExportStatus::Exported,
            0,
            ast.source_id,
            &module,
            &mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 78)),
            [2, 0].as_slice(),
            None,
            false,
        )
    );
    assert_eq!(output.binding_env.contexts().len(), 2);
    assert_eq!(output.binding_env.bindings().len(), 1);
    assert_eq!(output.binding_env.diagnostics().len(), 4);
    let statement = &output.statement;
    assert_eq!(statement.source_id, ast.source_id);
    assert_eq!(statement.module_id, module);
    assert_eq!(statement.owners.len(), 1);
    assert_eq!(statement.statements.len(), 1);
    assert_eq!(statement.contexts.len(), 1);
    assert!(statement.input_facts.is_empty());
    assert_eq!(statement.candidate_facts.len(), 1);
    let owner = &statement.owners[0];
    assert_eq!(owner.contribution.index(), 0);
    assert_eq!(owner.site.node().index(), 22);
    assert_eq!(owner.source_range, range(ast.source_id, 0, 78));
    assert_eq!(owner.spelling, "FormulaQuantifierBoundUsePayloadBoundary");
    assert_eq!(owner.role, SourceTheoremRole::Theorem);
    assert_eq!(owner.status, SourceTheoremStatus::Unmodified);
    assert_eq!(owner.recovery, SourceStatementRecovery::Normal);
    let checked_owner = mizar_checker::type_checker::CheckedStatementOwner::validate_exact_local_theorem(
        &resolver, owner.symbol.clone(), ast.source_id, &module,
    ).expect("genuine checked theorem owner before raw admission");
    assert_eq!(checked_owner.origin().structural_path(), [2, 0]);
    assert_eq!(checked_owner.symbol(), &owner.symbol);
    assert_eq!(
        checked_owner.source_range(),
        range(ast.source_id, 0, 78)
    );
    assert_eq!(
        checked_owner.visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert_eq!(
        checked_owner.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(
        checked_owner.origin().source_id(),
        ast.source_id
    );
    assert_eq!(checked_owner.origin().module_id(), &module);
    assert_eq!(
        checked_owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 78))
    );
    assert!(checked_owner.origin().import_edge().is_none());
    assert!(!checked_owner.origin().is_recovered());
    let row = &statement.statements[0];
    assert_eq!(row.owner, SourceTheoremOwnerId::new(0));
    assert_eq!(row.context, SourceStatementContextId::new(0));
    assert_eq!(row.formula, SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)));
    assert_eq!(row.site.node().index(), 22);
    assert_eq!(row.source_range, range(ast.source_id, 0, 78));
    assert_eq!(row.source_ordinal, 0);
    assert_eq!(row.spelling, "theorem FormulaQuantifierBoundUsePayloadBoundary : for x being set holds x = x ;");
    assert_eq!(row.kind, SourceStatementKind::TheoremProposition);
    assert_eq!(row.recovery, SourceStatementRecovery::Normal);
    let context = &statement.contexts[0];
    assert_eq!(context.statement, SourceStatementId::new(0));
    assert_eq!(context.binding_context.index(), 0);
    assert_eq!(context.source_range, range(ast.source_id, 0, 78));
    assert!(context.visible_bindings.is_empty());
    let candidate = &statement.candidate_facts[0];
    assert_eq!(candidate.statement, SourceStatementId::new(0));
    assert_eq!(candidate.context, SourceStatementContextId::new(0));
    assert_eq!(candidate.ordinal, 0);
    assert_eq!(candidate.kind, SourceStatementCandidateFactKind::UnverifiedProposition);
    assert_eq!(candidate.formula, row.formula);
    let primary = &output.primary;
    assert_eq!(
        primary.references().iter().map(|(_, row)| row.use_ordinal()).collect::<Vec<_>>(),
        [1, 1]
    );
    let atomic = &output.atomic;
    let composite = &output.composite;
    let composition = &output.composition;
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len()
        ),
        (2, 2, 0)
    );
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_segments().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.edges().len(),
            atomic.requests().len()
        ),
        (1, 0, 0, 0, 0, 0, 2, 2)
    );
    assert_eq!(
        (
            composite.formulas().len(),
            composite.wrappers().len(),
            composite.roots().len(),
            composite.binders().len(),
            composite.type_sites().len(),
            composite.edges().len(),
            composite.requests().len()
        ),
        (1, 0, 1, 1, 1, 0, 2)
    );
    assert_eq!(
        (
            composition.atomic_edges().len(),
            composition.bound_uses().len()
        ),
        (1, 2)
    );
    let formula = composite
        .formulas()
        .get(SourceCompositeFormulaId::new(0))
        .expect("formula");
    assert_eq!(formula.kind(), SourceCompositeFormulaKind::Universal);
    assert_eq!(formula.site().node().index(), 20);
    assert_eq!(formula.source_range(), range(ast.source_id, 50, 77));
    assert_eq!(
        composite
            .roots()
            .get(SourceFormulaRootId::new(0))
            .expect("root")
            .ownership(),
        SourceFormulaRootOwnership::UnassignedStatement
    );
    let lower = source_formula_composition_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B4A_TEXT)
        .expect("independent lower selector").expect("independent lower transport");
    assert_eq!(primary, lower.typed_ast.source_term().unwrap());
    assert_eq!(atomic, lower.typed_ast.source_atomic_formula().unwrap());
    assert_eq!(composite, lower.resolved.source_composite_formula().unwrap());
    assert_eq!(composition, lower.resolved.source_formula_composition().unwrap());
    assert!(lower.typed_ast.source_statement().is_none());
    assert!(symbols.labels().is_empty());
    assert_eq!(module, lower.typed_ast.module_id().clone());
}

#[test]
fn task258b4a_lower_and_upper_mutations_fail_at_owning_stage_and_replay() {
    use mizar_checker::{
        source_atomic_formula::SourceAtomicFormulaId,
        source_statement::{SourceStatementFormulaTarget, SourceStatementRecovery},
        typed_ast::{TypedArena, TypedNodeId},
    };

    let (ast, module, symbols, baseline) = task258b4a_output(265_100);
    let baseline_debug = format!("{baseline:?}");
    let lower = source_formula_composition_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B4A_TEXT)
        .expect("lower selector").expect("lower baseline");
    let baseline_typed = lower.typed_ast.debug_text();
    let baseline_resolved = lower.resolved.debug_text();

    let foreign_source = task258b3m2b2b3a_foreign_source_id(ast.source_id);
    let wrong_module = ResolverModuleId::new(
        PackageId::new("pkg"),
        ModulePath::new("statement.b4a.wrong"),
    );
    for mutation in 0..142 {
        let run = || {
            match source_formula_composition_output_with_source_and_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4A_TEXT,
                |input| {
                    let before = format!("{input:?}");
                    match mutation {
                    0 => input.primary.source_id = foreign_source,
                    1 => input.primary.module_id = wrong_module.clone(),
                    2 => input.primary.terms.clear(),
                    3 => input.primary.terms.push(input.primary.terms[0].clone()),
                    4 => input.primary.terms[0].site = input.primary.terms[1].site.clone(),
                    5 => input.primary.terms[0].source_range.end -= 1,
                    6 => input.primary.terms[0].source_ordinal = 1,
                    7 => input.primary.terms[0].context = BindingContextId::new(0),
                    8 => {
                        input.primary.terms[0].recovery =
                            mizar_checker::source_term::SourcePrimaryTermRecovery::Degraded
                    }
                    9 => input.primary.terms[0].spelling = "y".to_owned(),
                    10 => {
                        input.primary.terms[0].kind =
                            mizar_checker::source_term::SourcePrimaryTermKind::Numeral
                    }
                    11 => {
                        input.primary.terms[0].role =
                            mizar_checker::source_term::SourcePrimaryTermRole::CurrentDefinitionResult
                    }
                    12 => {
                        input.primary.terms[0].parent =
                            Some(mizar_checker::source_term::SourcePrimaryTermId::new(1))
                    }
                    13 => input.primary.terms[1].site = input.primary.terms[0].site.clone(),
                    14 => input.primary.terms[1].source_range.start += 1,
                    15 => input.primary.terms[1].source_ordinal = 0,
                    16 => input.primary.terms[1].context = BindingContextId::new(0),
                    17 => {
                        input.primary.terms[1].recovery =
                            mizar_checker::source_term::SourcePrimaryTermRecovery::Degraded
                    }
                    18 => input.primary.terms[1].spelling = "y".to_owned(),
                    19 => {
                        input.primary.terms[1].kind =
                            mizar_checker::source_term::SourcePrimaryTermKind::Numeral
                    }
                    20 => {
                        input.primary.terms[1].role =
                            mizar_checker::source_term::SourcePrimaryTermRole::CurrentDefinitionResult
                    }
                    21 => {
                        input.primary.terms[1].parent =
                            Some(mizar_checker::source_term::SourcePrimaryTermId::new(0))
                    }
                    22 => input.primary.references.clear(),
                    23 => input
                        .primary
                        .references
                        .push(input.primary.references[0].clone()),
                    24 => {
                        input.primary.references[0].term =
                            mizar_checker::source_term::SourcePrimaryTermId::new(1)
                    }
                    25 => input.primary.references[0].binding = BindingId::new(1),
                    26 => {
                        input.primary.references[0].role =
                            mizar_checker::source_term::SourcePrimaryTermReferenceRole::LocalConstant
                    }
                    27 => {
                        input.primary.references[1].term =
                            mizar_checker::source_term::SourcePrimaryTermId::new(0)
                    }
                    28 => input.primary.references[1].binding = BindingId::new(1),
                    29 => {
                        input.primary.references[1].role =
                            mizar_checker::source_term::SourcePrimaryTermReferenceRole::LocalConstant
                    }
                    30 => input.primary.numeric_type_requests.push(
                        mizar_checker::source_term::SourceNumericTypeRequestInput {
                            term: mizar_checker::source_term::SourcePrimaryTermId::new(0),
                            owner: input.primary.terms[0].site.clone(),
                            source_range: input.primary.terms[0].source_range,
                            spelling: "x".to_owned(),
                            request_ordinal: 0,
                        },
                    ),
                    31 => input.atomic.source_id = foreign_source,
                    32 => input.atomic.module_id = wrong_module.clone(),
                    33 => input.atomic.formulas.clear(),
                    34 => input.atomic.formulas.push(input.atomic.formulas[0].clone()),
                    35 => input.atomic.formulas[0].site = input.primary.terms[0].site.clone(),
                    36 => input.atomic.formulas[0].source_range.end -= 1,
                    37 => input.atomic.formulas[0].source_ordinal = 1,
                    38 => input.atomic.formulas[0].context = BindingContextId::new(0),
                    39 => {
                        input.atomic.formulas[0].recovery =
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Degraded
                    }
                    40 => input.atomic.formulas[0].spelling = "x <> x".to_owned(),
                    41 => {
                        input.atomic.formulas[0].kind =
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Inequality
                    }
                    42 => input.atomic.edges.clear(),
                    43 => input.atomic.edges.push(input.atomic.edges[0].clone()),
                    44 => {
                        input.atomic.edges[0].formula =
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1)
                    }
                    45 => input.atomic.edges[0].ordinal = 1,
                    46 => {
                        input.atomic.edges[0].role =
                            mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand
                    }
                    47 => {
                        input.atomic.edges[0].target =
                            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                                mizar_checker::source_term::SourcePrimaryTermId::new(1),
                            )
                    }
                    48 => {
                        input.atomic.edges[1].formula =
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1)
                    }
                    49 => input.atomic.edges[1].ordinal = 0,
                    50 => {
                        input.atomic.edges[1].role =
                            mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand
                    }
                    51 => {
                        input.atomic.edges[1].target =
                            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                                mizar_checker::source_term::SourcePrimaryTermId::new(0),
                            )
                    }
                    52 => input.atomic.requests.clear(),
                    53 => input.atomic.requests.push(input.atomic.requests[0].clone()),
                    54 => {
                        input.atomic.requests[0].formula =
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1)
                    }
                    55 => input.atomic.requests[0].ordinal = 1,
                    56 => {
                        input.atomic.requests[0].kind =
                            mizar_checker::source_atomic_formula::SourceAtomicRequestKind::PredicateCandidateSignature
                    }
                    57 => {
                        input.atomic.requests[0].edge =
                            Some(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(1))
                    }
                    58 => {
                        input.atomic.requests[0].candidate = Some(
                            mizar_checker::source_atomic_formula::SourcePredicateCandidateId::new(0),
                        )
                    }
                    59 => {
                        input.atomic.requests[0].type_site = Some(
                            mizar_checker::source_atomic_formula::SourceAssertionTypeSiteId::new(0),
                        )
                    }
                    60 => {
                        input.atomic.requests[0].attribute = Some(
                            mizar_checker::source_atomic_formula::SourceAssertionAttributeId::new(0),
                        )
                    }
                    61 => {
                        input.atomic.requests[1].formula =
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1)
                    }
                    62 => input.atomic.requests[1].ordinal = 0,
                    63 => {
                        input.atomic.requests[1].kind =
                            mizar_checker::source_atomic_formula::SourceAtomicRequestKind::PredicateCandidateSignature
                    }
                    64 => {
                        input.atomic.requests[1].edge =
                            Some(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(0))
                    }
                    65 => {
                        input.atomic.requests[1].candidate = Some(
                            mizar_checker::source_atomic_formula::SourcePredicateCandidateId::new(0),
                        )
                    }
                    66 => {
                        input.atomic.requests[1].type_site = Some(
                            mizar_checker::source_atomic_formula::SourceAssertionTypeSiteId::new(0),
                        )
                    }
                    67 => {
                        input.atomic.requests[1].attribute = Some(
                            mizar_checker::source_atomic_formula::SourceAssertionAttributeId::new(0),
                        )
                    }
                    68 => input.composite.source_id = foreign_source,
                    69 => input.composite.module_id = wrong_module.clone(),
                    70 => input.composite.formulas.clear(),
                    71 => input
                        .composite
                        .formulas
                        .push(input.composite.formulas[0].clone()),
                    72 => input.composite.formulas[0].site = input.composite.binders[0].segment_site.clone(),
                    73 => input.composite.formulas[0].source_range.start += 1,
                    74 => input.composite.formulas[0].source_ordinal = 1,
                    75 => input.composite.formulas[0].context = BindingContextId::new(1),
                    76 => {
                        input.composite.formulas[0].recovery =
                            mizar_checker::source_composite_formula::SourceCompositeFormulaRecovery::Degraded
                    }
                    77 => input.composite.formulas[0].spelling = "for st".to_owned(),
                    78 => {
                        input.composite.formulas[0].kind =
                            mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Existential
                    }
                    79 => input.composite.roots.clear(),
                    80 => input.composite.roots.push(input.composite.roots[0].clone()),
                    81 => {
                        input.composite.roots[0].formula =
                            mizar_checker::source_composite_formula::SourceCompositeFormulaId::new(1)
                    }
                    82 => input.composite.roots[0].ordinal = 1,
                    83 => input.composite.binders.clear(),
                    84 => input
                        .composite
                        .binders
                        .push(input.composite.binders[0].clone()),
                    85 => {
                        input.composite.binders[0].formula =
                            mizar_checker::source_composite_formula::SourceCompositeFormulaId::new(1)
                    }
                    86 => input.composite.binders[0].ordinal = 1,
                    87 => {
                        input.composite.binders[0].segment_site =
                            input.composite.binders[0].identifier_site.clone()
                    }
                    88 => input.composite.binders[0].segment_range.end -= 1,
                    89 => input.composite.binders[0].segment_spelling = "x".to_owned(),
                    90 => {
                        input.composite.binders[0].identifier_site =
                            input.composite.binders[0].segment_site.clone()
                    }
                    91 => input.composite.binders[0].identifier_range.end += 1,
                    92 => input.composite.binders[0].identifier_spelling = "y".to_owned(),
                    93 => {
                        input.composite.binders[0].local =
                            mizar_resolve::names::LocalTermBinding::new(
                                "y",
                                mizar_resolve::names::LocalTermScope::new(vec![1]),
                                input.composite.binders[0].identifier_range,
                                1,
                            )
                    }
                    94 => input.composite.binders[0].binding = BindingId::new(1),
                    95 => input.composite.binders[0].body_context = BindingContextId::new(0),
                    96 => {
                        input.composite.binders[0].type_site =
                            mizar_checker::source_composite_formula::SourceBinderTypeSiteId::new(1)
                    }
                    97 => {
                        input.composite.binders[0].recovery =
                            mizar_checker::source_composite_formula::SourceCompositeFormulaRecovery::Degraded
                    }
                    98 => input.composite.type_sites.clear(),
                    99 => input
                        .composite
                        .type_sites
                        .push(input.composite.type_sites[0].clone()),
                    100 => {
                        input.composite.type_sites[0].binder =
                            mizar_checker::source_composite_formula::SourceQuantifierBinderId::new(1)
                    }
                    101 => {
                        input.composite.type_sites[0].site =
                            input.composite.type_sites[0].head_site.clone()
                    }
                    102 => input.composite.type_sites[0].source_range.start += 1,
                    103 => input.composite.type_sites[0].spelling = "object".to_owned(),
                    104 => {
                        input.composite.type_sites[0].head_site =
                            input.composite.type_sites[0].site.clone()
                    }
                    105 => input.composite.type_sites[0].head_range.end -= 1,
                    106 => input.composite.type_sites[0].head_spelling = "object".to_owned(),
                    107 => input.composite.type_sites[0].context = BindingContextId::new(1),
                    108 => {
                        input.composite.type_sites[0].recovery =
                            mizar_checker::source_composite_formula::SourceCompositeFormulaRecovery::Degraded
                    }
                    109 => input.composite.edges.push(
                        mizar_checker::source_composite_formula::SourceFormulaEdgeInput {
                            parent: mizar_checker::source_composite_formula::SourceCompositeFormulaId::new(0),
                            ordinal: 0,
                            role: mizar_checker::source_composite_formula::SourceFormulaEdgeRole::UniversalBody,
                            child: mizar_checker::source_composite_formula::SourceCompositeFormulaId::new(0),
                        },
                    ),
                    110 => input.composite.requests.clear(),
                    111 => input
                        .composite
                        .requests
                        .push(input.composite.requests[0].clone()),
                    112 => {
                        input.composite.requests[0].formula =
                            mizar_checker::source_composite_formula::SourceCompositeFormulaId::new(1)
                    }
                    113 => input.composite.requests[0].ordinal = 1,
                    114 => {
                        input.composite.requests[0].kind =
                            mizar_checker::source_composite_formula::SourceFormulaRequestKind::ConstantSemantics
                    }
                    115 => {
                        input.composite.requests[0].binder =
                            Some(mizar_checker::source_composite_formula::SourceQuantifierBinderId::new(0))
                    }
                    116 => {
                        input.composite.requests[0].type_site =
                            Some(mizar_checker::source_composite_formula::SourceBinderTypeSiteId::new(0))
                    }
                    117 => {
                        input.composite.requests[1].formula =
                            mizar_checker::source_composite_formula::SourceCompositeFormulaId::new(1)
                    }
                    118 => input.composite.requests[1].ordinal = 0,
                    119 => {
                        input.composite.requests[1].kind =
                            mizar_checker::source_composite_formula::SourceFormulaRequestKind::QuantifierSemantics
                    }
                    120 => input.composite.requests[1].binder = None,
                    121 => input.composite.requests[1].type_site = None,
                    122 => input.composition.source_id = foreign_source,
                    123 => input.composition.module_id = wrong_module.clone(),
                    124 => input.composition.atomic_edges.clear(),
                    125 => input
                        .composition
                        .atomic_edges
                        .push(input.composition.atomic_edges[0].clone()),
                    126 => {
                        input.composition.atomic_edges[0].formula =
                            mizar_checker::source_composite_formula::SourceCompositeFormulaId::new(1)
                    }
                    127 => input.composition.atomic_edges[0].ordinal = 1,
                    128 => {
                        input.composition.atomic_edges[0].role =
                            mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::UniversalRestriction
                    }
                    129 => {
                        input.composition.atomic_edges[0].child =
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1)
                    }
                    130 => input.composition.bound_uses.clear(),
                    131 => input
                        .composition
                        .bound_uses
                        .push(input.composition.bound_uses[0].clone()),
                    132 => {
                        input.composition.bound_uses[0].binder =
                            mizar_checker::source_composite_formula::SourceQuantifierBinderId::new(1)
                    }
                    133 => input.composition.bound_uses[0].ordinal = 1,
                    134 => {
                        input.composition.bound_uses[0].body_edge =
                            mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeId::new(1)
                    }
                    135 => {
                        input.composition.bound_uses[0].term =
                            mizar_checker::source_term::SourcePrimaryTermId::new(1)
                    }
                    136 => {
                        input.composition.bound_uses[0].reference =
                            mizar_checker::source_term::SourcePrimaryTermReferenceId::new(1)
                    }
                    137 => {
                        input.composition.bound_uses[1].binder =
                            mizar_checker::source_composite_formula::SourceQuantifierBinderId::new(1)
                    }
                    138 => input.composition.bound_uses[1].ordinal = 0,
                    139 => {
                        input.composition.bound_uses[1].body_edge =
                            mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeId::new(1)
                    }
                    140 => {
                        input.composition.bound_uses[1].term =
                            mizar_checker::source_term::SourcePrimaryTermId::new(0)
                    }
                    141 => {
                        input.composition.bound_uses[1].reference =
                            mizar_checker::source_term::SourcePrimaryTermReferenceId::new(0)
                    }
                    _ => unreachable!(),
                    }
                    assert_ne!(format!("{input:?}"), before, "lower mutation {mutation} actual delta");
                },
            )
            .expect("lower selector")
            {
                Ok(_) => panic!("lower mutation {mutation} unexpectedly passed"),
                Err(error) => error,
            }
        };
        let error = run();
        assert_eq!(error, run(), "lower mutation {mutation}");
        let owning_stage_errors: &[&str] = match mutation {
            0..=30 => &["source primary", "source numeric-type", "source binding event"],
            31..=67 => &["source atomic"],
            68..=121 => &["source composite", "source formula root", "source quantifier binder", "source binder type site", "source formula edge", "source formula request"],
            122..=141 => &["source formula composition", "source formula atomic edge", "source quantifier bound use"],
            _ => unreachable!(),
        };
        assert!(owning_stage_errors.iter().any(|prefix| error.starts_with(prefix)), "lower owning stage mutation {mutation}: {error}");
        assert_eq!(format!("{baseline:?}"), baseline_debug, "lower purity {mutation}");
        let pristine = source_formula_composition_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B4A_TEXT)
            .expect("pristine lower selector").expect("pristine lower replay");
        assert_eq!(pristine.typed_ast, lower.typed_ast, "lower pristine {mutation}");
        assert_eq!(pristine.resolved, lower.resolved, "lower pristine final {mutation}");
    }

    // Typed owning-stage causes: Checker task_258b4a_composite_root_corruption_matrix_fails_atomically.
    // Runner 0..3 -> Checker 0..3; 4..31 -> 5..32; 32 -> 34; 33 -> 36.
    for mutation in 0..34 {
        let run = || {
            let mut captured = None;
            let error =
            source_statement_b4a_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4A_TEXT,
                |input| {
                    let before = format!("{input:?}");
                    match mutation {
                    0 => input.statement.source_id = foreign_source,
                    1 => input.statement.module_id = wrong_module.clone(),
                    2 => input.statement.owners.clear(),
                    3 => input
                        .statement
                        .owners
                        .push(input.statement.owners[0].clone()),
                    4 => {
                        input.statement.owners[0].site =
                            mizar_checker::typed_ast::TypedSiteRef::Node(TypedNodeId::new(21))
                    }
                    5 => input.statement.owners[0].source_range.end -= 1,
                    6 => input.statement.owners[0].spelling.push_str("Mutated"),
                    7 => {
                        input.statement.owners[0].recovery = SourceStatementRecovery::Degraded
                    }
                    8 => input.statement.statements.clear(),
                    9 => input
                        .statement
                        .statements
                        .push(input.statement.statements[0].clone()),
                    10 => {
                        input.statement.statements[0].owner =
                            mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
                    }
                    11 => {
                        input.statement.statements[0].context =
                            mizar_checker::source_statement::SourceStatementContextId::new(1)
                    }
                    12 => {
                        input.statement.statements[0].formula =
                            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
                    }
                    13 => {
                        input.statement.statements[0].site =
                            mizar_checker::typed_ast::TypedSiteRef::Node(TypedNodeId::new(21))
                    }
                    14 => input.statement.statements[0].source_range.end -= 1,
                    15 => input.statement.statements[0].source_ordinal = 1,
                    16 => input.statement.statements[0].spelling.push('!'),
                    17 => {
                        input.statement.statements[0].kind =
                            mizar_checker::source_statement::SourceStatementKind::Conclusion
                    }
                    18 => {
                        input.statement.statements[0].recovery =
                            SourceStatementRecovery::Degraded
                    }
                    19 => input.statement.contexts.clear(),
                    20 => input
                        .statement
                        .contexts
                        .push(input.statement.contexts[0].clone()),
                    21 => {
                        input.statement.contexts[0].statement =
                            mizar_checker::source_statement::SourceStatementId::new(1)
                    }
                    22 => {
                        input.statement.contexts[0].binding_context = BindingContextId::new(1)
                    }
                    23 => input.statement.contexts[0].source_range.end -= 1,
                    24 => input.statement.contexts[0].visible_bindings.push(BindingId::new(0)),
                    25 => input.statement.input_facts.push(
                        mizar_checker::source_statement::SourceStatementInputFactInput {
                            statement:
                                mizar_checker::source_statement::SourceStatementId::new(0),
                            context:
                                mizar_checker::source_statement::SourceStatementContextId::new(0),
                            ordinal: 0,
                            kind: mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                            binding: BindingId::new(0),
                            uses: vec![
                                mizar_checker::source_term::SourcePrimaryTermReferenceId::new(0),
                                mizar_checker::source_term::SourcePrimaryTermReferenceId::new(1),
                            ],
                        },
                    ),
                    26 => input.statement.candidate_facts.clear(),
                    27 => input
                        .statement
                        .candidate_facts
                        .push(input.statement.candidate_facts[0].clone()),
                    28 => {
                        input.statement.candidate_facts[0].statement =
                            mizar_checker::source_statement::SourceStatementId::new(1)
                    }
                    29 => {
                        input.statement.candidate_facts[0].context =
                            mizar_checker::source_statement::SourceStatementContextId::new(1)
                    }
                    30 => input.statement.candidate_facts[0].ordinal = 1,
                    31 => {
                        input.statement.candidate_facts[0].formula =
                            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
                    }
                    32 => {
                        input.statement.statements[0].formula =
                            SourceStatementFormulaTarget::Composite(
                                mizar_checker::source_composite_formula::SourceCompositeFormulaId::new(1),
                            );
                        input.statement.candidate_facts[0].formula =
                            input.statement.statements[0].formula;
                    }
                    33 => {
                        input.statement.owners.clear();
                        input.statement.candidate_facts.clear();
                    }
                    _ => unreachable!(),
                    }
                    let changed = format!("{input:?}");
                    assert_ne!(changed, before, "upper extraction mutation {mutation} actual delta");
                    captured = Some(changed);
                },
            )
            .expect("upper selector")
            .expect_err("retired raw upper");
            assert_eq!(error, "source statement dependency mismatch");
            captured.expect("changed upper inputs captured before refusal")
        };
        let changed = run();
        assert_eq!(changed, run(), "upper extraction replay {mutation}");
        assert_eq!(format!("{baseline:?}"), baseline_debug, "upper extraction purity {mutation}");
    }
    // The exact 26-node Surface profile is authenticated separately above. At
    // the checker boundary only nodes owned by the Task 252/256/257/258
    // handoffs are normative; unrelated lower-stage nodes remain unassigned.
    // Capture actual arena deltas here; typed constructor/installer refusals
    // for each of these owned causes remain in the private Checker suite.
    for node_index in [4, 12, 13, 14, 15, 17, 19, 20, 22] {
        for mutate_kind in [false, true] {
            let run = || {
                let mut captured = None;
                let error = source_statement_b4a_output_with_mutation(
                    &ast,
                    module.clone(),
                    &symbols,
                    SOURCE_STATEMENT_B4A_TEXT,
                    |input| {
                        let before = format!("{:?}", input.arena);
                        let mut nodes = input
                            .arena
                            .iter()
                            .map(|(_, node)| node.clone())
                            .collect::<Vec<_>>();
                        if mutate_kind {
                            nodes[node_index].kind = "source.statement.corrupted".into();
                        } else {
                            nodes[node_index].anchor =
                                mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 0));
                        }
                        input.arena =
                            TypedArena::try_new(input.arena.root(), nodes).expect("mutated arena");
                        let changed = format!("{:?}", input.arena);
                        assert_ne!(changed, before, "owned arena {node_index} kind={mutate_kind} actual delta");
                        captured = Some(changed);
                    },
                )
                .expect("upper arena selector")
                .expect_err("retired raw upper");
                assert_eq!(error, "source statement dependency mismatch");
                captured.expect("actual owned arena extraction")
            };
            let changed = run();
            assert_eq!(
                changed,
                run(),
                "arena node {node_index} kind={mutate_kind} capture replay"
            );
        }
    }
    let rooted_lower_run = || {
        let mut captured = None;
        let error = source_statement_b4a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B4A_TEXT,
            |input| {
                assert_ne!(input.arena.root(), Some(TypedNodeId::new(25)));
                input.arena = TypedArena::try_new(
                    Some(TypedNodeId::new(25)),
                    input.arena.iter().map(|(_, node)| node.clone()).collect(),
                )
                .expect("coherent rooted lower arena");
                captured = Some(format!("{:?}", input.arena));
            },
        )
        .expect("rooted lower selector")
        .expect_err("retired raw upper");
        assert_eq!(error, "source statement dependency mismatch");
        captured.expect("actual rooted lower extraction")
    };
    assert_eq!(
        rooted_lower_run(),
        rooted_lower_run(),
        "rooted lower replay"
    );

    let replay =
        source_formula_composition_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B4A_TEXT)
            .expect("replay selector")
            .expect("replay");
    assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
    assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    assert_eq!(format!("{baseline:?}"), baseline_debug);
    let (_, _, _, pristine) = task258b4a_output(265_100);
    assert_eq!(format!("{pristine:?}"), baseline_debug);
}

#[test]
fn task258b4a_source_resolver_and_active_b1_near_misses_stay_isolated() {
    let active_source = &SOURCE_STATEMENT_B4A_TEXT[..79];
    assert_eq!(active_source.as_bytes().last(), Some(&b'\n'));
    assert_eq!(
        sha256_text(active_source),
        "757872ac21c2a924c7c47f23328f5d76a8504255c195c17f113041c81bae5f3c"
    );
    let (active_ast, active_module, _, active_symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(active_source, 265_200);
    assert_eq!(diagnostics, 0);
    let active_symbols =
        augment_type_elaboration_import_summaries(&active_ast, &active_module, active_symbols);
    assert!(!task258b4a_surface_profile_with_mutation_for_test(
        &active_ast, active_source, SourceStatementB4ASurfaceMutation::None,
    ));
    assert!(source_statement_b4a_output_with_mutation(
        &active_ast, active_module.clone(), &active_symbols, active_source,
        |_| panic!("active lower-only source reached retired upper capture"),
    ).is_none());
    assert!(
        source_statement_output_with_source(
            &active_ast,
            active_module.clone(),
            &active_symbols,
            active_source,
        )
        .is_none(),
        "the active one-LF source must remain lower-only"
    );
    let lower = source_formula_composition_output_with_source(
        &active_ast,
        active_module,
        &active_symbols,
        active_source,
    )
    .expect("active lower selector")
    .expect("active lower route");
    assert!(lower.typed_ast.source_statement().is_none());
    assert!(lower.typed_ast.source_composite_formula().is_some());
    assert!(lower.typed_ast.source_formula_composition().is_some());

    let (ast, module, symbols, baseline) = task258b4a_output(265_210);
    let symbols_before = format!("{symbols:?}");
    let pristine_lower = source_formula_composition_output_with_source(
        &ast, module.clone(), &symbols, SOURCE_STATEMENT_B4A_TEXT,
    ).expect("private lower selector").expect("genuine private lower");
    // Exact typed owner-label refusals stay in the private Checker producer suite.
    // The retired raw route still exposes each real changed resolver environment.
    for mutation in [
        Task258B2ResolverMutation::Imported,
        Task258B2ResolverMutation::Missing,
        Task258B2ResolverMutation::Duplicate,
        Task258B2ResolverMutation::WrongPath,
        Task258B2ResolverMutation::WrongKind,
        Task258B2ResolverMutation::Private,
        Task258B2ResolverMutation::Recovered,
    ] {
        let run = || {
            let mut captured = None;
            let error = source_statement_b4a_output_with_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4A_TEXT,
                |symbols| {
                    let before = format!("{symbols:?}");
                    let symbols = task258b2_mutate_resolver(symbols, mutation);
                    let changed = format!("{symbols:?}");
                    assert_ne!(changed, before, "resolver {mutation:?} actual delta");
                    captured = Some(changed);
                    symbols
                },
            )
            .expect("resolver selector")
            .expect_err("retired raw upper");
            assert_eq!(error, "source statement dependency mismatch");
            captured.expect("actual mutated resolver environment")
        };
        assert_eq!(run(), run(), "{mutation:?} resolver capture replay");
        assert_eq!(format!("{symbols:?}"), symbols_before, "{mutation:?} resolver purity");
        let replay = source_formula_composition_output_with_source(
            &ast, module.clone(), &symbols, SOURCE_STATEMENT_B4A_TEXT,
        ).expect("pristine lower selector").expect("pristine lower replay");
        assert_eq!(replay.typed_ast, pristine_lower.typed_ast, "{mutation:?} pristine typed");
        assert_eq!(replay.resolved, pristine_lower.resolved, "{mutation:?} pristine final");
    }
    let (_, _, _, replay) = task258b4a_output(265_210);
    assert_eq!(format!("{replay:?}"), format!("{baseline:?}"));
}

#[test]
fn task258b4a_cross_family_orders_and_ownership_are_atomic() {
    use mizar_checker::{
        source_composite_formula::{SourceFormulaRootId, SourceFormulaRootOwnership},
        typed_ast::TypedAstError,
    };

    let (composite_ast, composite_module, composite_symbols, composite_output) =
        task258b4a_output(265_300);
    let (atomic_ast, atomic_module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_STATEMENT_TEXT, 265_301);
    assert_eq!(diagnostics, 0);
    let (atomic_output_typed, _) = task258_native_output(&atomic_ast, &atomic_module);
    let composite_before = format!("{composite_output:?}");
    let lower = source_formula_composition_output_with_source(
        &composite_ast, composite_module.clone(), &composite_symbols, SOURCE_STATEMENT_B4A_TEXT,
    ).expect("lower selector").expect("genuine composite lower");
    let atomic_statement = atomic_output_typed
        .source_statement()
        .expect("atomic statement")
        .clone();
    assert_eq!(atomic_statement.composite_formula_fingerprint(), None);
    assert_eq!(atomic_statement.formula_composition_fingerprint(), None);
    assert!(
        !atomic_statement
            .debug_text()
            .contains("composite-formula-fingerprint:")
    );
    assert!(
        !atomic_statement
            .debug_text()
            .contains("formula-composition-fingerprint:")
    );
    let atomic_before = atomic_output_typed.debug_text();
    assert_eq!(
        lower
            .typed_ast
            .clone()
            .with_source_statement(atomic_statement.clone())
            .expect_err("composite then atomic must fail"),
        TypedAstError::InvalidSourceStatement
    );
    assert_eq!(format!("{composite_output:?}"), composite_before);
    assert_eq!(atomic_output_typed.debug_text(), atomic_before);
    assert_eq!(
        lower
            .typed_ast
            .source_composite_formula()
            .expect("composite")
            .roots()
            .get(SourceFormulaRootId::new(0))
            .expect("root")
            .ownership(),
        SourceFormulaRootOwnership::UnassignedStatement
    );
    let (_, _, _, replay) = task258b4a_output(265_300);
    assert_eq!(format!("{replay:?}"), composite_before);
}

#[test]
fn task258b4a_final_clone_debug_and_semantic_outputs_are_stable() {
    let (ast, module, symbols, captured) = task258b4a_output(265_400);
    assert_eq!(format!("{:?}", captured.clone()), format!("{captured:?}"));
    let output = source_formula_composition_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B4A_TEXT)
        .expect("lower selector").expect("independent lower final");
    assert_eq!(output.typed_ast.clone(), output.typed_ast);
    assert_eq!(output.resolved.clone(), output.resolved);
    assert_eq!(
        output.typed_ast.clone().debug_text(),
        output.typed_ast.debug_text()
    );
    assert_eq!(
        output.resolved.clone().debug_text(),
        output.resolved.debug_text()
    );
    assert!(output.typed_ast.source_statement().is_none());
    assert!(output.resolved.source_statement().is_none());
    assert_eq!(output.typed_ast.source_term(), Some(&captured.primary));
    assert_eq!(output.typed_ast.source_atomic_formula(), Some(&captured.atomic));
    assert_eq!(output.typed_ast.source_composite_formula(), Some(&captured.composite));
    assert_eq!(output.typed_ast.source_formula_composition(), Some(&captured.composition));
    assert!(output.typed_ast.source_statement_references().is_none());
    assert!(output.typed_ast.source_statement_witnesses().is_none());
    assert!(output.typed_ast.source_context().is_none());
    assert!(output.typed_ast.source_type().is_none());
    assert!(output.typed_ast.source_attribute().is_none());
    assert!(output.typed_ast.source_evidence().is_none());
    assert!(output.typed_ast.source_application().is_none());
    assert!(output.typed_ast.source_structure().is_none());
    assert!(output.typed_ast.source_set_term().is_none());
    assert!(
        output
            .typed_ast
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        output
            .typed_ast
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(output.typed_ast.contexts().is_empty());
    assert!(output.typed_ast.types().is_empty());
    assert!(output.typed_ast.facts().is_empty());
    assert!(output.typed_ast.coercions().is_empty());
    assert!(output.typed_ast.initial_obligations().is_empty());
    assert!(output.typed_ast.diagnostics().is_empty());
    assert!(output.resolved.expr_metadata().is_empty());
    assert!(output.resolved.collection_candidates().is_empty());
    assert!(output.resolved.expanded_candidates().is_empty());
    assert!(output.resolved.template_expansions().is_empty());
    assert!(output.resolved.viable_candidates().is_empty());
    assert!(output.resolved.viability_decisions().is_empty());
    assert!(output.resolved.specificity_graphs().is_empty());
    assert!(output.resolved.resolved_overloads().is_empty());
    assert!(output.resolved.inserted_coercions().is_empty());
    assert!(output.resolved.cluster_facts().is_empty());
    assert!(output.resolved.diagnostics().is_empty());
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_proof_nodes().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());
    assert!(
        source_statement_transport_detail_keys(&ast, module, &symbols, SOURCE_STATEMENT_B4A_TEXT,)
            .is_none()
    );
}
