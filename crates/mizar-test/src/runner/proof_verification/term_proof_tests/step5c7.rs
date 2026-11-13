use super::*;

#[test]
fn step5c7_proof_corpus() {
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let cases = plan
        .cases
        .iter()
        .filter(|case| step5c7_proof_candidate(case))
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 2);
    for case in cases {
        let result = run_proof_verification_case(root, &root.join("tests"), case, 1);
        assert_eq!(
            result.status,
            ProofVerificationCaseStatus::Passed,
            "{:?}",
            result.failure
        );
    }
}

#[test]
fn step5c7_membership_proof_mutations_are_not_credit() {
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    for (id, path) in STEP5C7_PROOF_CASES {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let source = fs::read_to_string(root.join(path)).unwrap();
        let check = |text: &str| {
            let ast = parse(text);
            let symbols = resolver_symbol_collection(root, case, &ast);
            normalize_term_ast(&ast, &symbols.module, &symbols.env)
        };
        // Renaming is semantically irrelevant, including proof-local alpha names.
        let renamed = source
            .replace("A1", "Premise")
            .replace("CompMember1", "Guarded")
            .replace("EnumMember1", "Listed");
        assert_eq!(check(&renamed), Ok(2));
        if id.contains("comprehension") {
            for (label, changed) in [
                ("missing citation", source.replace(" by A1", "")),
                ("wrong citation", source.replace("by A1", "by Missing")),
                ("missing guard", source.replace(" : z in A", "")),
                (
                    "non-set bound",
                    source.replace("reserve A for set", "reserve A for object"),
                ),
                (
                    "shadowed set reservation",
                    source.replace(
                        "reserve A for set;",
                        "reserve A for set;\nreserve A for object;",
                    ),
                ),
                ("dependent bound", source.replace("z in A", "z in z")),
                ("wrong mapper", source.replace("{ z where", "{ x where")),
                (
                    "wrong proof premise",
                    source.replace("A1: x in A", "A1: A in A"),
                ),
                (
                    "altered conclusion",
                    source.replace("thus x in", "thus A in"),
                ),
                ("unjustified premise", source.replace(" st x in A", "")),
            ] {
                assert!(check(&changed).is_err(), "{label} was accepted");
            }
            let alpha = source
                .replace("let x be", "let u be")
                .replace("A1: x in", "A1: u in")
                .replace("thus x in", "thus u in");
            assert_eq!(check(&alpha), Ok(2));
        } else {
            for changed in [
                source.replace("thus x in", "thus y in"),
                source.replace("{x, y}", "{y}"),
                source.replace("thus x in {x, y}", "thus x in {x}"),
                source.replace("let x, y be object", "let x, y be set"),
            ] {
                assert!(
                    check(&changed).is_err(),
                    "altered enumeration proof accepted: {changed}"
                );
            }
        }
    }
}

#[test]
fn step5c7_citation_results_cannot_be_substituted() {
    use mizar_checker::{
        source_set_term::SourceSetTermProducer, type_checker::SourceVariableSemanticsChecker,
    };
    use mizar_resolve::{
        labels::{
            LabelReferenceCandidate, LabelReferenceScope, LabelResolver, ProofLabelSourceCollector,
        },
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ReferenceSite, SurfaceResolvedArena},
    };
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
        .unwrap();
    let source = fs::read_to_string(root.join(STEP5C7_PROOF_CASES[0].1)).unwrap();
    for citation in ["A1", "Missing"] {
        let ast = parse(&source.replace("by A1", &format!("by {citation}")));
        let symbols = resolver_symbol_collection(root, case, &ast);
        let module = &symbols.module;
        let scope = SourceVariableScopeResolver::resolve_occurrences(
            SourceVariableScopeInput::new(&ast, module, &symbols.env),
        )
        .unwrap();
        let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        let typed = super::super::super::type_elaboration::step5c7_membership_typed_ast(
            &ast,
            module,
            &symbols.env,
            &scope,
            &bindings,
        )
        .unwrap();
        let arena = SurfaceResolvedArena::lower(&ast, module).unwrap();
        let owner = symbols
            .env
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
            .unwrap();
        let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
        let labels = ProofLabelSourceCollector::new(
            &ast,
            module,
            namespace.clone(),
            owner.contribution(),
            &arena,
        )
        .unwrap()
        .collect_with_let_conditions()
        .unwrap();
        let resolve = |references: &[LabelReferenceCandidate]| {
            LabelResolver::new(labels.projections()).resolve(module, &namespace, references)
        };
        let check = |resolved| {
            SourceSetTermProducer::check_membership_proof(
                &typed,
                &scope,
                &bindings,
                &labels,
                resolved,
                &symbols.env,
            )
            .is_ok()
        };
        let original = resolve(labels.references());
        assert_eq!(check(&original), citation == "A1");
        let reference = &labels.references()[0];
        let LabelReferenceScope::Unqualified { proof_scope } = reference.scope() else {
            panic!("local citation")
        };
        let forged = LabelReferenceCandidate::unqualified_citation(
            ReferenceSite::new(reference.site().node(), reference.site().range(), "A1"),
            if citation == "A1" {
                labels.projections()[0].origin().clone()
            } else {
                reference.origin().clone()
            },
            reference.ordinal(),
            proof_scope.clone(),
        );
        let substituted = resolve(&[forged]);
        assert!(!substituted.has_unresolved());
        assert!(
            !check(&substituted),
            "substituted site/origin for {citation}"
        );
    }
}

#[test]
fn step5c7_native_predicate_condition_uses_scoped_context() {
    use mizar_checker::{
        binding_env::{BindingContextId, BindingContextLayer, BindingContextOwner, BindingId},
        source_atomic_formula::{
            SourceAtomicEdgeRole, SourceAtomicFormulaKind, SourceAtomicFormulaRecovery,
            SourceAtomicRequestKind, SourceAtomicTermTarget, SourcePredicateSegmentPolarityInput,
        },
        source_set_term::{
            SourceSetGeneratorId, SourceSetRequestKind, SourceSetTermKind, SourceSetTypeOwner,
        },
        source_term::{
            SourcePrimaryTermKind, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceRole,
            SourcePrimaryTermRole,
        },
        type_checker::SourceVariableSemanticsChecker,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::SurfaceResolvedArena,
    };

    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
        .unwrap();
    let likes = "definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; ";
    let relation = ":: alpha/layout\ndefinition\n let parcel, needle be object; pred Rel: parcel knows needle means parcel = needle; end;\n";
    let positive = "{ z where z is set : z likes A }";
    let positive_chain = "{ z where z is set : z likes A likes z }";
    let positive_three_segment_chain = "{ z where z is set : z likes A likes z likes A }";
    let positive_long_chain = "{ z where z is set : z likes A likes z likes A likes z }";
    let negative = "{ z where z is set : z does not likes A }";
    let does_not_first = "{ z where z is set : z does not likes A likes z likes A }";
    let does_not_middle = "{ z where z is set : z likes A does not likes z likes A }";
    let does_not_last = "{ z where z is set : z likes A likes z does not likes A }";
    let do_not_first =
        "{ point where point is set : point do not knows item knows point knows item }";
    let do_not_middle =
        "{ point where point is set : point knows item do not knows point knows item }";
    let do_not_last =
        "{ point where point is set : point knows item knows point do not knows item }";
    let cases = vec![
        (
            "positive header",
            format!("{likes}theorem T: for A being set holds A in {positive};"),
            1,
            1,
        ),
        (
            "positive proof",
            format!(
                "{likes}theorem T: for A being set holds A = A proof let A be set; thus A in {positive}; end;"
            ),
            1,
            1,
        ),
        (
            "positive both and shadowed proof binding",
            format!(
                "{likes}theorem T: for A being set holds A in {positive} proof let A be set; thus A in {positive}; end;"
            ),
            2,
            2,
        ),
        (
            "does not header",
            format!("{likes}theorem T: for A being set holds A in {negative};"),
            1,
            1,
        ),
        (
            "does not proof",
            format!(
                "{likes}theorem T: for A being set holds A = A proof let A be set; thus A in {negative}; end;"
            ),
            1,
            1,
        ),
        (
            "does not both and shadowed proof binding",
            format!(
                "{likes}theorem T: for A being set holds A in {negative} proof let A be set; thus A in {negative}; end;"
            ),
            2,
            2,
        ),
        (
            "mixed disjoint conditions",
            format!(
                "{likes}theorem T: for A being set holds A in {positive} & A in {{ w where w is set : w does not likes A }};"
            ),
            2,
            2,
        ),
        (
            "do not alpha/layout and shadowed proof binding",
            format!(
                "{relation}theorem T: for item being set holds item in {{ point where point is set : point do not knows item }} proof\nlet item be set; thus item in {{ point where point is set : point do not knows item }}; end;"
            ),
            2,
            2,
        ),
        (
            "unused outer quantifier and proof Let",
            format!(
                "{likes}theorem T: for A, spare being set holds A in {positive} proof let A, spare be set; thus A in {positive}; end;"
            ),
            2,
            2,
        ),
        (
            "positive two-segment chain header",
            format!("{likes}theorem T: for A being set holds A in {positive_chain};"),
            1,
            1,
        ),
        (
            "positive three-segment chain proof with shadowed binding",
            format!(
                "{likes}theorem T: for A being set holds A = A proof let A be set; thus A in {positive_three_segment_chain}; end;"
            ),
            1,
            1,
        ),
        (
            "positive four-segment chain header and proof with shadowed binding",
            format!(
                "{likes}theorem T: for A being set holds A in {positive_long_chain} proof let A be set; thus A in {positive_long_chain}; end;"
            ),
            2,
            2,
        ),
        (
            "does not first chain segment in header",
            format!("{likes}theorem T: for A being set holds A in {does_not_first};"),
            1,
            1,
        ),
        (
            "does not middle chain segment in proof",
            format!(
                "{likes}theorem T: for A being set holds A = A proof let A be set; thus A in {does_not_middle}; end;"
            ),
            1,
            1,
        ),
        (
            "does not last chain segment in both with shadowed binding",
            format!(
                "{likes}theorem T: for A being set holds A in {does_not_last} proof let A be set; thus A in {does_not_last}; end;"
            ),
            2,
            2,
        ),
        (
            "do not first chain segment in alpha/layout header",
            format!("{relation}theorem T: for item being set holds item in {do_not_first};"),
            1,
            1,
        ),
        (
            "do not middle chain segment in proof",
            format!(
                "{relation}theorem T: for item being set holds item = item proof let item be set; thus item in {do_not_middle}; end;"
            ),
            1,
            1,
        ),
        (
            "do not last chain segment in both with shadowed binding",
            format!(
                "{relation}theorem T: for item being set holds item in {do_not_last} proof let item be set; thus item in {do_not_last}; end;"
            ),
            2,
            2,
        ),
        (
            "mixed disjoint chain then singleton",
            format!(
                "{likes}theorem T: for A being set holds A in {positive_chain} & A in {{ w where w is set : w does not likes A }};"
            ),
            2,
            2,
        ),
        (
            "mixed disjoint singleton then chain",
            format!(
                "{likes}theorem T: for A being set holds A in {{ w where w is set : w does not likes A }} & A in {positive_chain};"
            ),
            2,
            2,
        ),
        (
            "unused declarations with four-segment chain",
            format!(
                "{likes}theorem T: for A, spare being set holds A in {positive_long_chain} proof let A, spare be set; thus A in {positive_long_chain}; end;"
            ),
            2,
            2,
        ),
    ];
    for (case_index, (label, source, expected_sets, expected_memberships)) in
        cases.into_iter().enumerate()
    {
        let ast = parse(&source);
        let ast_before = ast.clone();
        let symbols = resolver_symbol_collection(root, case, &ast);
        assert!(
            symbols.detail_keys.is_empty(),
            "{label}: {:?}",
            symbols.detail_keys
        );
        let module = &symbols.module;
        let resolved = SurfaceResolvedArena::lower(&ast, module).unwrap();
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, module, &symbols.env),
        )
        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
        let scope_before = scope.clone();
        let bindings = SourceVariableSemanticsChecker::source_theorem_binding_env(
            &resolved,
            &symbols.env,
            &scope,
        )
        .unwrap_or_else(|error| panic!("{label}: {error}"));
        let bindings_before = bindings.clone();
        let (scopes, parents) =
            mizar_resolve::symbols::source_proof_scope_maps(&resolved, &symbols.env, &scope)
                .unwrap_or_else(|error| panic!("{label}: {error}"));
        let mut boundaries = scopes
            .iter()
            .filter(|(id, local)| {
                parents
                    .get(id)
                    .is_none_or(|parent| scopes.get(parent) != Some(local))
            })
            .collect::<Vec<_>>();
        boundaries.sort_by_key(|(_, local)| (local.path().len(), local.path()));
        let expected_context = |site| {
            let local = scopes.get(&site).expect("source local scope");
            BindingContextId::new(
                boundaries
                    .iter()
                    .position(|(_, boundary)| *boundary == local)
                    .expect("source scope boundary"),
            )
        };
        assert_eq!(
            bindings.contexts().len(),
            boundaries.len(),
            "{label}: complete contexts"
        );
        assert_eq!(
            bindings.bindings().len(),
            scope.bindings().len(),
            "{label}: complete declarations"
        );
        for (index, (surface, local)) in boundaries.iter().enumerate() {
            let view = ast.node_view(**surface).unwrap();
            let row = bindings
                .contexts()
                .get(BindingContextId::new(index))
                .unwrap();
            let (owner, layer) = match view.kind() {
                mizar_syntax::SurfaceNodeKind::QuantifiedFormula(_) => (
                    BindingContextOwner::SourceFormula {
                        source_range: view.range(),
                    },
                    BindingContextLayer::Expression,
                ),
                mizar_syntax::SurfaceNodeKind::ProofBlock => (
                    BindingContextOwner::SourceStatement {
                        source_range: view.range(),
                    },
                    BindingContextLayer::Proof,
                ),
                mizar_syntax::SurfaceNodeKind::DefinitionBlockItem => (
                    BindingContextOwner::SourceStatement {
                        source_range: view.range(),
                    },
                    BindingContextLayer::Declaration,
                ),
                mizar_syntax::SurfaceNodeKind::SetComprehension => (
                    BindingContextOwner::SourceComprehension {
                        source_range: view.range(),
                    },
                    BindingContextLayer::Expression,
                ),
                _ if index == 0 => (BindingContextOwner::Module, BindingContextLayer::Module),
                kind => panic!("{label}: unexpected context owner {kind:?}"),
            };
            assert_eq!(
                row.id,
                BindingContextId::new(index),
                "{label}: dense context ID"
            );
            assert_eq!(row.owner, owner, "{label}: actual context owner");
            assert_eq!(row.layer, layer, "{label}: actual context layer");
            let own = scope
                .bindings()
                .iter()
                .filter(|binding| binding.scope() == *local)
                .map(|binding| BindingId::new(binding.id().index()))
                .collect::<Vec<_>>();
            let visible = scope
                .bindings()
                .iter()
                .filter(|binding| local.path().starts_with(binding.scope().path()))
                .map(|binding| BindingId::new(binding.id().index()))
                .collect::<Vec<_>>();
            assert_eq!(row.bindings, own, "{label}: complete local declarations");
            assert_eq!(
                row.visible_bindings, visible,
                "{label}: complete visible declarations"
            );
        }
        if label == "unused outer quantifier and proof Let" {
            assert_eq!(
                scope
                    .bindings()
                    .iter()
                    .filter(|binding| binding.spelling() == "spare")
                    .count(),
                2,
                "unused declarations remain in scope census"
            );
        }
        let mut written_sets = ast
            .node_views()
            .filter(|view| view.kind() == &mizar_syntax::SurfaceNodeKind::SetComprehension)
            .collect::<Vec<_>>();
        written_sets.sort_by_key(|view| (view.range().start, view.range().end, view.id().index()));
        assert_eq!(
            written_sets.len(),
            expected_sets,
            "{label}: literal comprehension roots"
        );
        let mut memberships = ast
            .node_views()
            .filter(|view| {
                let node = ast.node(view.id()).unwrap();
                matches!(
                    node.kind,
                    mizar_syntax::SurfaceNodeKind::BuiltinPredicateApplication
                ) && node
                    .children
                    .iter()
                    .filter_map(|child| ast.node(*child).unwrap().token_text())
                    .collect::<Vec<_>>()
                    == ["in"]
            })
            .collect::<Vec<_>>();
        memberships.sort_by_key(|view| (view.range().start, view.range().end, view.id().index()));
        assert_eq!(
            memberships.len(),
            expected_memberships,
            "{label}: literal Membership roots"
        );
        let condition_sites = written_sets
            .iter()
            .map(|view| {
                ast.node(view.id())
                    .unwrap()
                    .children
                    .iter()
                    .copied()
                    .find(|child| {
                        matches!(
                            ast.node(*child).unwrap().kind,
                            mizar_syntax::SurfaceNodeKind::FormulaExpression
                        )
                    })
                    .expect("literal comprehension condition")
            })
            .collect::<Vec<_>>();

        let typed = super::super::super::type_elaboration::step5c7_membership_typed_ast(
            &ast,
            module,
            &symbols.env,
            &scope,
            &bindings,
        )
        .unwrap_or_else(|error| panic!("{label}: {error}"));
        let primary = typed.source_term().unwrap();
        let set = typed.source_set_term().unwrap();
        let atomic = typed.source_atomic_formula().unwrap();
        assert_eq!(
            primary.references().len(),
            scope.references().len(),
            "{label}: complete Primary references"
        );
        assert_eq!(
            primary.terms().len(),
            scope.references().len(),
            "{label}: complete Primary rows"
        );
        let mut references = scope.references().iter().collect::<Vec<_>>();
        references.sort_by_key(|reference| (reference.range().start, reference.range().end));
        for (ordinal, actual) in references.iter().enumerate() {
            let (id, term) = primary.terms().iter().nth(ordinal).unwrap();
            assert_eq!(id.index(), ordinal, "{label}: dense Primary id");
            assert_eq!(term.source_ordinal(), ordinal, "{label}: Primary ordinal");
            assert_eq!(
                term.site().node().index(),
                actual.node().index(),
                "{label}: Primary source site"
            );
            assert_eq!(
                term.source_range(),
                actual.range(),
                "{label}: Primary source range"
            );
            assert_eq!(
                term.spelling(),
                actual.spelling(),
                "{label}: Primary spelling"
            );
            assert_eq!(
                term.kind(),
                SourcePrimaryTermKind::VariableReference,
                "{label}: Primary kind"
            );
            assert_eq!(
                term.role(),
                SourcePrimaryTermRole::Value,
                "{label}: Primary role"
            );
            assert_eq!(
                term.recovery(),
                SourcePrimaryTermRecovery::Normal,
                "{label}: Primary recovery"
            );
            assert!(term.parent().is_none(), "{label}: bare Primary parent");
            assert_eq!(
                term.context(),
                expected_context(actual.node()),
                "{label}: Primary lexical context"
            );
            let (reference_id, row) = primary.references().iter().nth(ordinal).unwrap();
            assert_eq!(reference_id.index(), ordinal, "{label}: dense reference id");
            assert_eq!(row.term(), id, "{label}: reference/term link");
            assert_eq!(
                row.binding().index(),
                actual.binding().index(),
                "{label}: resolver winner"
            );
            assert_eq!(
                row.lexical_scope(),
                Some(actual.scope()),
                "{label}: reference lexical scope"
            );
            assert_eq!(
                row.use_ordinal(),
                actual.ordinal(),
                "{label}: reference ordinal"
            );
            assert_eq!(
                row.role(),
                SourcePrimaryTermReferenceRole::Variable,
                "{label}: reference role"
            );
        }
        assert_eq!(
            set.terms().len(),
            expected_sets,
            "{label}: complete Set rows"
        );
        assert_eq!(
            set.generators().len(),
            expected_sets,
            "{label}: complete generators"
        );
        assert_eq!(
            set.type_sites().len(),
            expected_sets,
            "{label}: complete generator types"
        );
        assert_eq!(
            set.conditions().len(),
            expected_sets,
            "{label}: complete conditions"
        );
        assert_eq!(
            set.requests().len(),
            expected_sets * 2,
            "{label}: unanswered Set requests"
        );
        for (ordinal, view) in written_sets.iter().enumerate() {
            let (id, term) = set.terms().iter().nth(ordinal).unwrap();
            assert_eq!(id.index(), ordinal, "{label}: dense Set id");
            assert_eq!(term.source_ordinal(), ordinal, "{label}: Set ordinal");
            assert_eq!(
                term.site().node().index(),
                view.id().index(),
                "{label}: Set owner"
            );
            assert_eq!(term.kind(), SourceSetTermKind::Comprehension, "{label}");
            assert_eq!(
                term.context(),
                expected_context(view.id()),
                "{label}: Set context"
            );
            let generator_id = SourceSetGeneratorId::new(ordinal);
            let generator = set.generators().get(generator_id).unwrap();
            assert_eq!(generator.term(), id, "{label}: generator owner");
            assert_eq!(generator.ordinal(), 0, "{label}: generator ordinal");
            assert_eq!(
                generator.context(),
                term.context(),
                "{label}: generator context"
            );
            let type_site = set.type_sites().get(generator.type_site()).unwrap();
            assert_eq!(
                type_site.owner(),
                SourceSetTypeOwner::Generator(generator_id),
                "{label}: generator type owner"
            );
            assert_eq!(
                type_site.context(),
                term.context(),
                "{label}: generator type context"
            );
            let condition = set
                .conditions()
                .get(mizar_checker::source_set_term::SourceSetConditionId::new(
                    ordinal,
                ))
                .unwrap();
            assert_eq!(condition.term(), id, "{label}: condition owner");
            assert_eq!(
                condition.condition_site().node().index(),
                condition_sites[ordinal].index(),
                "{label}: condition site"
            );
            assert_eq!(
                condition.spelling(),
                &source[condition.source_range().start..condition.source_range().end],
                "{label}: condition spelling"
            );
        }
        assert!(
            set.requests().iter().all(|(_, request)| matches!(
                request.kind(),
                SourceSetRequestKind::GeneratorSethood | SourceSetRequestKind::ResultType
            )),
            "{label}: Set requests remain unanswered"
        );

        let mut expected_atoms = ast
            .node_views()
            .filter(|view| {
                matches!(
                    view.kind(),
                    mizar_syntax::SurfaceNodeKind::PredicateApplication
                ) || memberships
                    .iter()
                    .any(|membership| membership.id() == view.id())
            })
            .collect::<Vec<_>>();
        expected_atoms
            .sort_by_key(|view| (view.range().start, view.range().end, view.id().index()));
        let mut expected_segments = Vec::new();
        for formula in &expected_atoms {
            if formula.kind() != &mizar_syntax::SurfaceNodeKind::PredicateApplication {
                continue;
            }
            let mut ordinal = 0;
            for child in &ast.node(formula.id()).unwrap().children {
                if ast.node(*child).unwrap().kind == mizar_syntax::SurfaceNodeKind::PredicateSegment
                {
                    expected_segments.push((formula.id(), *child, ordinal));
                    ordinal += 1;
                }
            }
            assert!(ordinal > 0, "{label}: predicate root has written segments");
        }
        assert_eq!(
            atomic.formulas().len(),
            expected_atoms.len(),
            "{label}: complete Atomic formula census"
        );
        assert_eq!(
            atomic.formulas().len(),
            expected_memberships + expected_sets,
            "{label}: formula rows"
        );
        assert_eq!(
            typed.nodes().len(),
            ast.node_views().count(),
            "{label}: complete typed syntax arena"
        );
        for (typed_id, node) in typed.nodes().iter() {
            let surface = ast
                .node_views()
                .find(|view| view.id().index() == typed_id.index())
                .unwrap();
            assert_eq!(
                node.resolved_node,
                resolved.resolved_node_for(surface.id()),
                "{label}: immutable resolved link"
            );
        }
        for (ordinal, (id, formula)) in atomic.formulas().iter().enumerate() {
            assert_eq!(id.index(), ordinal, "{label}: dense Atomic id");
            assert_eq!(formula.source_ordinal(), ordinal, "{label}: Atomic ordinal");
            assert_eq!(
                formula.site().node().index(),
                expected_atoms[ordinal].id().index(),
                "{label}: Atomic site"
            );
            assert_eq!(
                formula.context(),
                expected_context(expected_atoms[ordinal].id()),
                "{label}: Atomic context"
            );
            assert_eq!(
                formula.kind(),
                if matches!(
                    expected_atoms[ordinal].kind(),
                    mizar_syntax::SurfaceNodeKind::PredicateApplication
                ) {
                    SourceAtomicFormulaKind::PredicateApplication
                } else {
                    SourceAtomicFormulaKind::Membership
                },
                "{label}: Atomic kind"
            );
        }
        assert_eq!(
            atomic.predicate_heads().len(),
            expected_segments.len(),
            "{label}: predicate heads"
        );
        assert_eq!(
            atomic.predicate_segments().len(),
            expected_segments.len(),
            "{label}: predicate segments"
        );
        assert_eq!(
            atomic.candidates().len(),
            expected_segments.len(),
            "{label}: predicate candidates"
        );
        let declared = symbols
            .env
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Predicate)
            .collect::<Vec<_>>();
        assert_eq!(
            declared.len(),
            1,
            "{label}: one source predicate definition"
        );
        for (ordinal, ((root_site, segment_site, segment_ordinal), (candidate_id, candidate))) in
            expected_segments
                .iter()
                .zip(atomic.candidates().iter())
                .enumerate()
        {
            let (head_id, head) = atomic.predicate_heads().iter().nth(ordinal).unwrap();
            let (segment_id, segment) = atomic.predicate_segments().iter().nth(ordinal).unwrap();
            let formula_id = atomic
                .formulas()
                .iter()
                .find(|(_, formula)| formula.site().node().index() == root_site.index())
                .expect("source predicate root has one Atomic formula")
                .0;
            let segment_node = ast.node(*segment_site).unwrap();
            let head_site = segment_node
                .children
                .iter()
                .copied()
                .find(|child| {
                    ast.node(*child).unwrap().kind == mizar_syntax::SurfaceNodeKind::PredicateHead
                })
                .expect("source segment has a written head");
            assert_eq!(candidate_id.index(), ordinal, "{label}: dense candidate id");
            assert_eq!(segment_id.index(), ordinal, "{label}: dense segment id");
            assert_eq!(
                segment.formula(),
                formula_id,
                "{label}: segment root formula"
            );
            assert_eq!(
                *segment_ordinal,
                segment.ordinal(),
                "{label}: local segment order"
            );
            assert_eq!(segment.site().node().index(), segment_site.index());
            assert_eq!(segment.source_range(), segment_node.range);
            assert_eq!(
                segment.spelling(),
                &source[segment_node.range.start..segment_node.range.end],
                "{label}: exact segment source"
            );
            assert_eq!(segment.recovery(), SourceAtomicFormulaRecovery::Normal);
            assert_eq!(segment.context(), expected_context(*segment_site));
            assert_eq!(
                candidate.head(),
                head_id,
                "{label}: candidate/head association"
            );
            assert_eq!(candidate.ordinal(), 0, "{label}: candidate ordinal");
            assert_eq!(
                candidate.symbol(),
                declared[0].symbol(),
                "{label}: source collector candidate"
            );
            assert_eq!(
                candidate.contribution(),
                declared[0].contribution(),
                "{label}: candidate contribution"
            );
            assert_eq!(head.formula(), formula_id, "{label}: head root formula");
            assert_eq!(head.site().node().index(), head_site.index());
            let head_node = ast.node(head_site).unwrap();
            assert_eq!(head.source_range(), head_node.range);
            assert_eq!(
                head.spelling(),
                &source[head_node.range.start..head_node.range.end],
                "{label}: exact predicate head source"
            );
            assert_eq!(head.context(), expected_context(head_site));
            assert_eq!(head.left_arity(), 1, "{label}: binary predicate left arity");
            assert_eq!(
                head.right_arity(),
                1,
                "{label}: binary predicate right arity"
            );
            assert_eq!(segment.head(), head_id, "{label}: segment head");
            assert_eq!(
                segment.context(),
                head.context(),
                "{label}: segment context"
            );
            let tokens = segment_node
                .children
                .iter()
                .filter_map(|child| {
                    ast.node(*child)
                        .unwrap()
                        .token_text()
                        .map(|text| (*child, text))
                })
                .collect::<Vec<_>>();
            if tokens
                .iter()
                .any(|(_, token)| matches!(*token, "does" | "do"))
            {
                let (verb, verb_text) = tokens
                    .iter()
                    .find(|(_, token)| matches!(*token, "does" | "do"))
                    .unwrap();
                let (not, _) = tokens.iter().find(|(_, token)| *token == "not").unwrap();
                assert!(
                    matches!(segment.polarity(), SourcePredicateSegmentPolarityInput::Negative { verb_site, verb_range, verb_spelling, verb_recovery, not_site, not_range, not_spelling, not_recovery } if verb_site.node().index() == verb.index() && *verb_range == ast.node(*verb).unwrap().range && verb_spelling == verb_text && *verb_recovery == SourceAtomicFormulaRecovery::Normal && not_site.node().index() == not.index() && *not_range == ast.node(*not).unwrap().range && not_spelling == "not" && *not_recovery == SourceAtomicFormulaRecovery::Normal),
                    "{label}: actual polarity sites"
                );
            } else {
                assert!(
                    matches!(
                        segment.polarity(),
                        SourcePredicateSegmentPolarityInput::Positive
                    ),
                    "{label}: positive polarity"
                );
            }
        }

        let mut expected_edge_count = 0;
        for formula_site in expected_atoms.iter().filter(|formula| {
            formula.kind() == &mizar_syntax::SurfaceNodeKind::PredicateApplication
        }) {
            let formula_id = atomic
                .formulas()
                .iter()
                .find(|(_, formula)| formula.site().node().index() == formula_site.id().index())
                .expect("predicate root Atomic row")
                .0;
            let formula_segments = expected_segments
                .iter()
                .filter(|(root_site, _, _)| *root_site == formula_site.id())
                .collect::<Vec<_>>();
            let mut operand_sites = Vec::new();
            for (ordinal, (_, segment_site, _)) in formula_segments.iter().enumerate() {
                let written_terms = ast
                    .node(*segment_site)
                    .unwrap()
                    .children
                    .iter()
                    .copied()
                    .filter(|child| {
                        ast.node(*child).unwrap().kind
                            == mizar_syntax::SurfaceNodeKind::TermExpression
                    })
                    .collect::<Vec<_>>();
                if ordinal == 0 {
                    assert_eq!(
                        written_terms.len(),
                        2,
                        "{label}: first segment owns left and right operands"
                    );
                    operand_sites.extend(written_terms);
                } else {
                    assert_eq!(
                        written_terms.len(),
                        1,
                        "{label}: continuation owns only its written right operand"
                    );
                    operand_sites.push(written_terms[0]);
                }
            }
            assert_eq!(
                operand_sites.len(),
                formula_segments.len() + 1,
                "{label}: chain edge/operand census"
            );
            let actual_edges = atomic
                .edges()
                .iter()
                .filter(|(_, edge)| edge.formula() == formula_id)
                .collect::<Vec<_>>();
            assert_eq!(
                actual_edges.len(),
                operand_sites.len(),
                "{label}: chain edges"
            );
            expected_edge_count += operand_sites.len();
            for (ordinal, ((_, edge), term_site)) in
                actual_edges.iter().zip(&operand_sites).enumerate()
            {
                let role = if ordinal == 0 {
                    SourceAtomicEdgeRole::PredicateLeftArgument
                } else if ordinal + 1 == operand_sites.len() {
                    SourceAtomicEdgeRole::PredicateRightArgument
                } else {
                    SourceAtomicEdgeRole::PredicateChainBoundary
                };
                let source_term = ast.node(*term_site).unwrap();
                let reference_sites = source_term
                    .children
                    .iter()
                    .copied()
                    .filter(|child| {
                        ast.node(*child).unwrap().kind
                            == mizar_syntax::SurfaceNodeKind::TermReference
                    })
                    .collect::<Vec<_>>();
                let [reference_site] = reference_sites.as_slice() else {
                    panic!("{label}: written chain operand has one TermReference")
                };
                let primary_id = primary
                    .terms()
                    .iter()
                    .find(|(_, term)| term.site().node().index() == reference_site.index())
                    .expect("written chain operand has one Primary row")
                    .0;
                assert_eq!(edge.ordinal(), ordinal, "{label}: dense chain edge ordinal");
                assert_eq!(edge.role(), role, "{label}: written chain edge role");
                assert_eq!(
                    edge.target(),
                    SourceAtomicTermTarget::Primary(primary_id),
                    "{label}: chain edge follows its source operand"
                );
            }
        }
        for membership in &memberships {
            let formula_id = atomic
                .formulas()
                .iter()
                .find(|(_, formula)| formula.site().node().index() == membership.id().index())
                .expect("Membership root Atomic row")
                .0;
            let membership_node = ast.node(membership.id()).unwrap();
            let written_terms = membership_node
                .children
                .iter()
                .copied()
                .filter(|child| {
                    ast.node(*child).unwrap().kind == mizar_syntax::SurfaceNodeKind::TermExpression
                })
                .collect::<Vec<_>>();
            assert_eq!(
                written_terms.len(),
                2,
                "{label}: Membership has two written operands"
            );
            let set_site = written_sets
                .iter()
                .find(|set_view| {
                    ast.node(written_terms[1]).unwrap().range.start <= set_view.range().start
                        && set_view.range().end <= ast.node(written_terms[1]).unwrap().range.end
                })
                .expect("Membership right operand is its source comprehension")
                .id();
            let set_id = set
                .terms()
                .iter()
                .find(|(_, term)| term.site().node().index() == set_site.index())
                .expect("Membership comprehension has one Set row")
                .0;
            let left_term = ast.node(written_terms[0]).unwrap();
            let left_references = left_term
                .children
                .iter()
                .copied()
                .filter(|child| {
                    ast.node(*child).unwrap().kind == mizar_syntax::SurfaceNodeKind::TermReference
                })
                .collect::<Vec<_>>();
            let [left_reference] = left_references.as_slice() else {
                panic!("{label}: Membership left operand has one TermReference")
            };
            let left_primary = primary
                .terms()
                .iter()
                .find(|(_, term)| term.site().node().index() == left_reference.index())
                .expect("Membership left operand has one Primary row")
                .0;
            let actual_edges = atomic
                .edges()
                .iter()
                .filter(|(_, edge)| edge.formula() == formula_id)
                .collect::<Vec<_>>();
            assert_eq!(actual_edges.len(), 2, "{label}: Membership edge count");
            expected_edge_count += 2;
            assert_eq!(actual_edges[0].1.ordinal(), 0);
            assert_eq!(
                actual_edges[0].1.role(),
                SourceAtomicEdgeRole::BuiltinLeftOperand
            );
            assert_eq!(
                actual_edges[0].1.target(),
                SourceAtomicTermTarget::Primary(left_primary)
            );
            assert_eq!(actual_edges[1].1.ordinal(), 1);
            assert_eq!(
                actual_edges[1].1.role(),
                SourceAtomicEdgeRole::BuiltinRightOperand
            );
            assert_eq!(
                actual_edges[1].1.target(),
                SourceAtomicTermTarget::SetTerm(set_id)
            );
        }
        assert_eq!(
            atomic.edges().len(),
            expected_edge_count,
            "{label}: complete source edge census"
        );
        assert_eq!(
            atomic.requests().len(),
            expected_memberships + expected_segments.len(),
            "{label}: complete unresolved Atomic requests"
        );
        assert_eq!(
            atomic
                .requests()
                .iter()
                .filter(
                    |(_, row)| row.kind() == SourceAtomicRequestKind::PredicateCandidateSignature
                )
                .count(),
            expected_segments.len(),
            "{label}: candidate requests"
        );
        for (candidate_ordinal, (root_site, _, segment_ordinal)) in
            expected_segments.iter().enumerate()
        {
            let formula_id = atomic
                .formulas()
                .iter()
                .find(|(_, formula)| formula.site().node().index() == root_site.index())
                .expect("predicate root request owner")
                .0;
            let candidate_id =
                mizar_checker::source_atomic_formula::SourcePredicateCandidateId::new(
                    candidate_ordinal,
                );
            let requests = atomic
                .requests()
                .iter()
                .filter(|(_, request)| request.formula() == formula_id)
                .collect::<Vec<_>>();
            let request = requests
                .iter()
                .find(|(_, request)| request.candidate() == Some(candidate_id))
                .expect("each written segment has one unresolved candidate request")
                .1;
            assert_eq!(request.ordinal(), *segment_ordinal);
            assert_eq!(
                request.kind(),
                SourceAtomicRequestKind::PredicateCandidateSignature
            );
            assert_eq!(request.edge(), None);
            assert_eq!(request.type_site(), None);
            assert_eq!(request.attribute(), None);
        }
        for membership in &memberships {
            let formula_id = atomic
                .formulas()
                .iter()
                .find(|(_, formula)| formula.site().node().index() == membership.id().index())
                .expect("Membership request owner")
                .0;
            let requests = atomic
                .requests()
                .iter()
                .filter(|(_, request)| request.formula() == formula_id)
                .collect::<Vec<_>>();
            assert_eq!(
                requests.len(),
                1,
                "{label}: one unanswered Membership request"
            );
            let (edge_id, edge) = atomic
                .edges()
                .iter()
                .find(|(_, edge)| edge.formula() == formula_id && edge.ordinal() == 1)
                .unwrap();
            assert_eq!(requests[0].1.ordinal(), 0);
            assert_eq!(
                requests[0].1.kind(),
                SourceAtomicRequestKind::OperandExpectedType
            );
            assert_eq!(requests[0].1.edge(), Some(edge_id));
            assert_eq!(edge.role(), SourceAtomicEdgeRole::BuiltinRightOperand);
            assert_eq!(requests[0].1.candidate(), None);
            assert_eq!(requests[0].1.type_site(), None);
            assert_eq!(requests[0].1.attribute(), None);
        }

        let route = super::super::super::type_elaboration::source_condition_formula_composition_output_with_source(&ast, module.clone(), &symbols.env, &source)
            .unwrap_or_else(|| panic!("{label}: native scoped C2 route missing"))
            .unwrap_or_else(|error| panic!("{label}: {error}"));
        let composition = route
            .typed_ast
            .source_condition_formula_composition()
            .unwrap();
        assert_eq!(
            composition.edges().len(),
            expected_sets,
            "{label}: C2 association census"
        );
        for (ordinal, ((condition_id, condition), (edge_id, edge))) in set
            .conditions()
            .iter()
            .zip(composition.edges().iter())
            .enumerate()
        {
            assert_eq!(edge_id.index(), ordinal, "{label}: dense association id");
            assert_eq!(
                edge.condition(),
                condition_id,
                "{label}: source condition association"
            );
            assert_eq!(
                edge.ordinal(),
                condition.ordinal(),
                "{label}: association ordinal"
            );
            let wrapper_view = ast
                .node_views()
                .find(|view| view.id().index() == condition.condition_site().node().index())
                .unwrap();
            let wrapper = ast.node(wrapper_view.id()).unwrap();
            let condition_children = wrapper
                .children
                .iter()
                .copied()
                .filter(|child| {
                    !matches!(
                        ast.node(*child).unwrap().kind,
                        mizar_syntax::SurfaceNodeKind::Token(_)
                    )
                })
                .collect::<Vec<_>>();
            let [atom] = condition_children.as_slice() else {
                panic!("{label}: singleton source condition")
            };
            let formula_id = atomic
                .formulas()
                .iter()
                .find(|(_, formula)| formula.site().node().index() == atom.index())
                .unwrap()
                .0;
            assert_eq!(
                edge.formula(),
                formula_id,
                "{label}: paired condition/atom association"
            );
            let whole_root = atomic.formulas().get(formula_id).unwrap();
            assert_eq!(whole_root.site().node().index(), atom.index());
            let root_segments = atomic
                .predicate_segments()
                .iter()
                .filter(|(_, segment)| segment.formula() == formula_id)
                .map(|(_, segment)| segment.site().node().index())
                .collect::<Vec<_>>();
            if !root_segments.is_empty() {
                assert!(
                    root_segments.iter().all(|site| *site != atom.index()),
                    "{label}: C2 points to the whole chain root"
                );
                let source_segment_count = ast
                    .node(*atom)
                    .unwrap()
                    .children
                    .iter()
                    .filter(|child| {
                        ast.node(**child).unwrap().kind
                            == mizar_syntax::SurfaceNodeKind::PredicateSegment
                    })
                    .count();
                assert_eq!(root_segments.len(), source_segment_count);
            }
        }
        assert_eq!(
            route.typed_ast.source_term(),
            typed.source_term(),
            "{label}: Primary install"
        );
        assert_eq!(
            route.typed_ast.source_set_term(),
            typed.source_set_term(),
            "{label}: Set install"
        );
        assert_eq!(
            route.typed_ast.source_atomic_formula(),
            typed.source_atomic_formula(),
            "{label}: Atomic install"
        );
        assert_eq!(
            route.typed_ast.source_condition_formula_composition(),
            route.resolved.source_condition_formula_composition(),
            "{label}: final replay"
        );
        assert!(
            route.resolved.checked_formulas().is_empty()
                && route.resolved.statement_semantics().is_empty()
                && route.resolved.checked_proofs().is_empty(),
            "{label}: no semantic credit"
        );
        assert_eq!(
            super::super::super::type_elaboration::source_formula_composition_transport_detail_keys(
                &ast,
                module.clone(),
                &symbols.env,
                &source
            ),
            Some(vec![
                "type_elaboration.external_dependency.ast_payload_extraction".to_owned()
            ]),
            "{label}: existing coverage boundary"
        );
        if case_index == 0 || label == "positive two-segment chain header" {
            let pristine = route.typed_ast.clone();
            assert!(
                route
                    .typed_ast
                    .clone()
                    .with_source_condition_formula_composition(composition.clone())
                    .is_err(),
                "duplicate C2 install"
            );
            assert_eq!(route.typed_ast, pristine, "failed install preserves input");
            let base = mizar_checker::typed_ast::TypedAst::try_new(
                mizar_checker::typed_ast::TypedAstParts {
                    source_id: typed.source_id(),
                    module_id: typed.module_id().clone(),
                    resolved_root: typed.resolved_root(),
                    source_context: typed.source_context().cloned(),
                    source_type: typed.source_type().cloned(),
                    source_attribute: typed.source_attribute().cloned(),
                    nodes: typed.nodes().clone(),
                    contexts: typed.contexts().clone(),
                    types: typed.types().clone(),
                    facts: typed.facts().clone(),
                    coercions: typed.coercions().clone(),
                    initial_obligations: typed.initial_obligations().clone(),
                    diagnostics: typed.diagnostics().clone(),
                },
            )
            .unwrap()
            .with_source_term(primary.clone())
            .unwrap();
            let application = route.typed_ast.source_application().unwrap().clone();
            let install_application_first = base
                .clone()
                .with_source_application(application.clone())
                .unwrap()
                .with_source_set_term(set.clone())
                .unwrap()
                .with_source_atomic_formula(atomic.clone())
                .unwrap()
                .with_source_condition_formula_composition(composition.clone())
                .unwrap();
            let install_set_first = base
                .with_source_set_term(set.clone())
                .unwrap()
                .with_source_application(application)
                .unwrap()
                .with_source_atomic_formula(atomic.clone())
                .unwrap()
                .with_source_condition_formula_composition(composition.clone())
                .unwrap();
            assert_eq!(
                install_application_first.source_condition_formula_composition(),
                Some(composition),
                "application-first install"
            );
            assert_eq!(
                install_set_first.source_condition_formula_composition(),
                Some(composition),
                "Set-first install"
            );
            let final_application_first =
                super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                    &install_application_first,
                    vec![],
                )
                .unwrap();
            let final_set_first =
                super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                    &install_set_first,
                    vec![],
                )
                .unwrap();
            assert_eq!(
                final_application_first.source_condition_formula_composition(),
                Some(composition)
            );
            assert_eq!(
                final_set_first.source_condition_formula_composition(),
                Some(composition)
            );
            let replay = super::super::super::type_elaboration::source_condition_formula_composition_output_with_source(&ast, module.clone(), &symbols.env, &source).unwrap().unwrap();
            assert_eq!(
                route.typed_ast, replay.typed_ast,
                "deterministic Typed replay"
            );
            assert_eq!(
                route.resolved, replay.resolved,
                "deterministic final replay"
            );
            let appended = format!("{source} theorem U: for A being set holds A = A;");
            let gap_token = source.replacen("end; theorem", "end;;theorem", 1);
            let wrong = source.replace("LikesDef", "OtherDef");
            for changed in [&appended, &gap_token, &wrong] {
                assert!(super::super::super::type_elaboration::source_condition_formula_composition_output_with_source(&ast, module.clone(), &symbols.env, changed).is_some_and(|result| result.is_err()), "source mismatch was accepted: {changed}");
            }
            let reject_profile = |changed: String, reason: &str| {
                let changed_ast = parse(&changed);
                let changed_symbols = resolver_symbol_collection(root, case, &changed_ast);
                assert!(
                    changed_symbols.detail_keys.is_empty(),
                    "{reason}: {:?}",
                    changed_symbols.detail_keys
                );
                assert!(super::super::super::type_elaboration::source_condition_formula_composition_output_with_source(
                    &changed_ast,
                    changed_symbols.module,
                    &changed_symbols.env,
                    &changed,
                ).is_some_and(|result| result.is_err()), "{reason}: native profile refusal was lost");
            };
            let grouped_condition = if source.contains(": z likes A likes z") {
                source.replace(": z likes A likes z", ": (z likes A likes z)")
            } else {
                source.replace(": z likes A", ": (z likes A)")
            };
            if case_index == 0 {
                let grouped_frontend =
                    super::super::super::formula_statement::step5c8_test_frontend(
                        &grouped_condition,
                    );
                assert!(
                    grouped_frontend.diagnostics.is_empty(),
                    "grouped condition frontend"
                );
                let grouped_ast = grouped_frontend.ast.unwrap();
                let grouped_symbols = resolver_symbol_collection(root, case, &grouped_ast);
                assert!(
                    grouped_symbols.detail_keys.is_empty(),
                    "grouped condition resolver"
                );
                let grouped_output = super::super::super::type_elaboration::source_condition_formula_composition_output_with_source(
                    &grouped_ast,
                    grouped_symbols.module.clone(),
                    &grouped_symbols.env,
                    &grouped_condition,
                )
                .expect("grouped singleton native C2 selector")
                .expect("grouped singleton native C2 route");
                assert_eq!(
                    grouped_output
                        .typed_ast
                        .source_condition_formula_composition()
                        .unwrap()
                        .edges()
                        .len(),
                    1,
                    "grouped singleton keeps one whole-condition C2 association"
                );
                assert_eq!(
                    grouped_output
                        .typed_ast
                        .source_atomic_formula()
                        .unwrap()
                        .wrappers()
                        .len(),
                    1,
                    "grouped singleton retains its Atomic wrapper"
                );
                assert!(
                    grouped_output
                        .typed_ast
                        .source_predicate_chain_composition()
                        .is_none(),
                    "grouped singleton has no C3"
                );
                let grouped_replay = super::super::super::type_elaboration::source_condition_formula_composition_output_with_source(
                    &grouped_ast,
                    grouped_symbols.module,
                    &grouped_symbols.env,
                    &grouped_condition,
                )
                .unwrap()
                .unwrap();
                assert_eq!(grouped_replay.typed_ast, grouped_output.typed_ast);
                assert_eq!(grouped_replay.resolved, grouped_output.resolved);
            }
            reject_profile(
                source.replace(": z likes A", ": z likes A & z likes A"),
                "compound condition",
            );
            if case_index == 0 {
                let wrapped_compound = source.replace(": z likes A", ": (z likes A & A likes z)");
                let wrapped_compound_frontend =
                    super::super::super::formula_statement::step5c8_test_frontend(
                        &wrapped_compound,
                    );
                assert!(
                    wrapped_compound_frontend.diagnostics.is_empty(),
                    "wrapped compound condition frontend"
                );
                reject_profile(wrapped_compound, "wrapped compound condition");
                let wrapped_formula_negation = source.replace(": z likes A", ": (not z likes A)");
                let wrapped_formula_negation_frontend =
                    super::super::super::formula_statement::step5c8_test_frontend(
                        &wrapped_formula_negation,
                    );
                assert!(
                    wrapped_formula_negation_frontend.diagnostics.is_empty(),
                    "wrapped formula-level negation frontend"
                );
                reject_profile(wrapped_formula_negation, "wrapped formula-level negation");
            }
            reject_profile(
                source.replace(": z likes A", ": (z) likes A"),
                "grouped predicate operand",
            );
            let wrapped_source = if source.contains(": z likes A likes z") {
                source.replace(
                    "in { z where z is set : z likes A likes z }",
                    "in ({ z where z is set : z likes A likes z })",
                )
            } else {
                source.replace(
                    "in { z where z is set : z likes A }",
                    "in ({ z where z is set : z likes A })",
                )
            };
            reject_profile(wrapped_source, "wrapped comprehension");
            reject_profile(
                source.replace("z is set", "z is object"),
                "Object generator",
            );
            reject_profile(
                source.replace("z where z is set", "z where z is set, w is set"),
                "multiple generators",
            );
            reject_profile(
                source.replace(
                    "z where z is set",
                    "{ z where z is set : z likes A } where z is set",
                ),
                "nested comprehension",
            );
        }
        if label.starts_with("mixed disjoint") {
            let cross_pair = super::super::super::type_elaboration::source_condition_formula_composition_output_with_source_and_mutation(
                &ast,
                module.clone(),
                &symbols.env,
                &source,
                |inputs| inputs.composition.edges[0].formula = inputs.composition.edges[1].formula,
            );
            assert!(
                cross_pair.is_some_and(|result| result.is_err()),
                "cross-pair C2 association was admitted"
            );
        }
        if label == "does not header" {
            let disconnected = super::super::super::type_elaboration::source_condition_formula_composition_output_with_source_and_mutation(
                &ast,
                module.clone(),
                &symbols.env,
                &source,
                |inputs| {
                    let segment_site = inputs.atomic.predicate_segments[0].site.clone();
                    if let SourcePredicateSegmentPolarityInput::Negative { verb_site, .. } =
                        &mut inputs.atomic.predicate_segments[0].polarity
                    {
                        *verb_site = segment_site;
                    }
                },
            );
            assert!(
                disconnected.is_some_and(|result| result.is_err()),
                "disconnected negative polarity was admitted"
            );
        }
        assert_eq!(ast, ast_before, "{label}: AST purity");
        assert_eq!(scope, scope_before, "{label}: scope purity");
        assert_eq!(
            bindings, bindings_before,
            "{label}: acquired binding purity"
        );
    }
}

#[test]
fn step5c7_native_condition_composition_replays_source_membership() {
    use mizar_checker::{
        source_application::{
            SourceFunctorApplicationHandoffInput, SourceFunctorApplicationProducer,
        },
        source_atomic_formula::{
            SourceAtomicEdgeRole, SourceAtomicFormulaKind, SourceAtomicFormulaRecovery,
            SourceAtomicRequestKind, SourceAtomicTermTarget,
        },
        source_formula_composition::{
            SourceConditionFormulaCompositionHandoffInput,
            SourceConditionFormulaCompositionProducer, SourceConditionFormulaEdgeInput,
        },
        source_set_term::{
            SourceSetConditionId, SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget,
            SourceSetTermKind, SourceSetTermRecovery, SourceSetTypeHead, SourceSetTypeOwner,
        },
        type_checker::SourceVariableSemanticsChecker,
    };
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    use mizar_syntax::SurfaceNodeKind;

    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
        .unwrap();
    let original = fs::read_to_string(root.join(STEP5C7_PROOF_CASES[0].1)).unwrap();
    let one = original.replacen(" : z in A", "", 1);
    let mut header_only = original.clone();
    let last_condition = header_only.rfind(" : z in A").unwrap();
    header_only.replace_range(last_condition..last_condition + " : z in A".len(), "");
    let zero = original.replace(" : z in A", "");
    let three = format!(
        "{original}\ntheorem CompMemberExtra: for A for x being object st x in A holds x in {{ z where z is object : z in A }};\n"
    );
    let alpha = original.replace("z", "q");
    let layout = original.replace(": z in A", ":  z in A");
    let set_heads = original.replace("z is object", "z is set");
    let equality = original.replace(": z in A", ": z = z");
    let header_equality = original.replacen(": z in A", ": z = z", 1);
    let mut proof_equality = original.clone();
    let proof_condition = proof_equality.rfind(": z in A").unwrap();
    proof_equality.replace_range(
        proof_condition..proof_condition + ": z in A".len(),
        ": z = z",
    );
    let noncondition_equality = original.replacen("st x in A holds", "st x = x holds", 1);
    let equality_layout = original.replace(": z in A", ":  z = z");
    let inequality = original.replace(": z in A", ": z <> z");
    let header_inequality = original.replacen(": z in A", ": z <> z", 1);
    let mut proof_inequality = original.clone();
    proof_inequality.replace_range(
        proof_condition..proof_condition + ": z in A".len(),
        ": z <> z",
    );
    let mixed_conditions = format!(
        "{}\ntheorem CompMemberMixed: for A for x being object st x in A holds x in {{ z where z is object : z in A }};\n",
        proof_inequality.replacen(": z in A", ": z = z", 1)
    );
    let noncondition_inequality = inequality.replacen("st x in A holds", "st x <> x holds", 1);
    let composite_inequality = inequality.replace(": z <> z", ": z <> z & z in A");
    let set_literal = "{ z where z is object : z in A }";
    let multiple = original.replace("z where z is object", "z where z is object, w is object");
    let three_generators = original.replace(
        "z where z is object",
        "z where z is object, w is object, v is object",
    );
    let unused_set = original.replace("z where z is object", "z where z is object, w is set");
    let mut enumeration_sibling = original.clone();
    let last_set = enumeration_sibling.rfind(set_literal).unwrap();
    enumeration_sibling.replace_range(last_set..last_set + set_literal.len(), "{ x }");
    let enumeration_mapper = original.replace(set_literal, "{ { x } where z is object : z in A }");
    let enumeration_tree =
        original.replace(set_literal, "{ { { x } } where z is object : z in A }");
    let wrapped_mapper = original.replace(set_literal, "{ ({ x }) where z is object : z in A }");
    let wrapped_element =
        original.replace(set_literal, "{ { ({ x }) } where z is object : z in A }");
    let double_wrapped_mapper =
        original.replace(set_literal, "{ (({ x })) where z is object : z in A }");
    let grouped_set = original.replace(set_literal, &format!("({set_literal})"));
    let grouped_header = original.replacen(set_literal, &format!("({set_literal})"), 1);
    let mut grouped_proof = original.clone();
    let proof_set = grouped_proof.rfind(set_literal).unwrap();
    grouped_proof.replace_range(
        proof_set..proof_set + set_literal.len(),
        &format!("({set_literal})"),
    );
    let double_grouped_set = original.replace(set_literal, &format!("(({set_literal}))"));
    let grouped_layout = original.replace(set_literal, &format!("(  {set_literal}  )"));
    let grouped_primary = original.replacen("st x in A holds", "st (x) in A holds", 1);
    let grouped_left = original.replace(
        &format!("x in {set_literal}"),
        &format!("({set_literal}) in A"),
    );
    let mut grouped_enumeration = original.clone();
    let proof_set = grouped_enumeration.rfind(set_literal).unwrap();
    grouped_enumeration.replace_range(proof_set..proof_set + set_literal.len(), "({ x })");
    let inner_comprehension = original.replace(
        set_literal,
        "{ { x where w is object } where z is object : z in A }",
    );
    let mut cases = [
        ("one", one, 1, &["z in A"][..]),
        ("header only", header_only, 1, &["z in A"][..]),
        ("original", original.clone(), 2, &["z in A"][..]),
        ("three", three, 3, &["z in A"][..]),
        ("alpha", alpha, 2, &["q in A"][..]),
        ("layout", layout, 2, &["z in A"][..]),
        ("set heads", set_heads, 2, &["z in A"][..]),
        ("equality", equality.clone(), 2, &["z = z"][..]),
        (
            "header equality",
            header_equality,
            2,
            &["z = z", "z in A"][..],
        ),
        (
            "proof equality",
            proof_equality,
            2,
            &["z in A", "z = z"][..],
        ),
        (
            "equality alpha",
            equality.replace('z', "q"),
            2,
            &["q = q"][..],
        ),
        ("equality layout", equality_layout, 2, &["z = z"][..]),
        ("inequality", inequality.clone(), 2, &["z <> z"][..]),
        (
            "header inequality",
            header_inequality,
            2,
            &["z <> z", "z in A"][..],
        ),
        (
            "proof inequality",
            proof_inequality,
            2,
            &["z in A", "z <> z"][..],
        ),
        (
            "mixed conditions",
            mixed_conditions,
            3,
            &["z = z", "z <> z", "z in A"][..],
        ),
        (
            "inequality alpha",
            inequality.replace('z', "q"),
            2,
            &["q <> q"][..],
        ),
        (
            "inequality layout",
            inequality.replace(": z <> z", ":  z <> z"),
            2,
            &["z <> z"][..],
        ),
        (
            "inequality distinct operands",
            inequality.replace("z <> z", "z <> A"),
            2,
            &["z <> A"][..],
        ),
        (
            "inequality condition-free sibling",
            inequality.replacen(" : z <> z", "", 1),
            1,
            &["z <> z"][..],
        ),
        ("multiple generators", multiple.clone(), 2, &["z in A"][..]),
        ("three generators", three_generators, 2, &["z in A"][..]),
        (
            "multiple generators alpha",
            multiple.replace('z', "q"),
            2,
            &["q in A"][..],
        ),
        ("unused set generator", unused_set, 2, &["z in A"][..]),
        (
            "enumeration sibling",
            enumeration_sibling,
            1,
            &["z in A"][..],
        ),
        (
            "enumeration mapper",
            enumeration_mapper.clone(),
            2,
            &["z in A"][..],
        ),
        (
            "enumeration mapper layout",
            enumeration_mapper.replace("{ x }", "{  x }"),
            2,
            &["z in A"][..],
        ),
        ("enumeration tree", enumeration_tree, 2, &["z in A"][..]),
        ("wrapped mapper", wrapped_mapper.clone(), 2, &["z in A"][..]),
        (
            "wrapped mapper alpha",
            wrapped_mapper.replace('z', "q"),
            2,
            &["q in A"][..],
        ),
        (
            "wrapped element",
            wrapped_element.clone(),
            2,
            &["z in A"][..],
        ),
        (
            "wrapped element layout",
            wrapped_element.replace("({ x })", "( {  x  } )"),
            2,
            &["z in A"][..],
        ),
        (
            "double wrapped mapper",
            double_wrapped_mapper,
            2,
            &["z in A"][..],
        ),
        ("grouped set", grouped_set.clone(), 2, &["z in A"][..]),
        ("grouped header", grouped_header, 2, &["z in A"][..]),
        ("grouped proof", grouped_proof, 2, &["z in A"][..]),
        ("double grouped set", double_grouped_set, 2, &["z in A"][..]),
        (
            "grouped alpha",
            grouped_set.replace('z', "q"),
            2,
            &["q in A"][..],
        ),
        ("grouped layout", grouped_layout, 2, &["z in A"][..]),
        ("grouped left", grouped_left, 2, &["z in A"][..]),
        (
            "grouped enumeration",
            grouped_enumeration,
            1,
            &["z in A"][..],
        ),
        (
            "inner comprehension boundary",
            inner_comprehension,
            2,
            &["z in A"][..],
        ),
        (
            "noncondition equality",
            noncondition_equality,
            2,
            &["z in A"][..],
        ),
        (
            "noncondition inequality",
            noncondition_inequality,
            2,
            &["z <> z"][..],
        ),
        ("zero", zero, 0, &["z in A"][..]),
    ]
    .into_iter()
    .map(|(label, source, count, literals)| (label, source, count, literals.to_vec()))
    .collect::<Vec<_>>();
    // EN14 14.1 grouping is transparent condition metadata, without proof credit.
    for (header_label, proof_label, both_label, nested_label, direct, grouped, nested) in [
        (
            "grouped equality header",
            "grouped equality proof",
            "grouped equality",
            "nested equality",
            "z = z",
            "(z = z)",
            "((z = z))",
        ),
        (
            "grouped inequality header",
            "grouped inequality proof",
            "grouped inequality",
            "nested inequality",
            "z <> A",
            "(z <> A)",
            "((z <> A))",
        ),
        (
            "grouped membership header",
            "grouped membership proof",
            "grouped condition",
            "nested membership",
            "z in A",
            "(z in A)",
            "((z in A))",
        ),
    ] {
        let base = original.replace(": z in A", &format!(": {direct}"));
        cases.push((
            header_label,
            base.replacen(&format!(": {direct}"), &format!(": {grouped}"), 1),
            2,
            vec![grouped, direct],
        ));
        let mut proof = base.clone();
        let last = proof.rfind(&format!(": {direct}")).unwrap();
        proof.replace_range(last..last + direct.len() + 2, &format!(": {grouped}"));
        cases.push((proof_label, proof, 2, vec![direct, grouped]));
        cases.push((
            both_label,
            base.replace(&format!(": {direct}"), &format!(": {grouped}")),
            2,
            vec![grouped],
        ));
        cases.push((
            nested_label,
            base.replace(&format!(": {direct}"), &format!(": {nested}")),
            2,
            vec![nested],
        ));
    }
    cases.extend([
        (
            "outside grouped membership",
            original.replace(": z in A", ": (z in A)").replacen(
                "st x in A holds",
                "st (x in A) holds",
                1,
            ),
            2,
            vec!["(z in A)"],
        ),
        (
            "grouped condition alpha",
            original
                .replace(": z in A", ": ((q in A))")
                .replace("z where z", "q where q"),
            2,
            vec!["((q in A))"],
        ),
        (
            "grouped condition layout",
            original.replace(": z in A", ": (  (z in A)  )"),
            2,
            vec!["(  (z in A)  )"],
        ),
        (
            "grouped condition-free sibling",
            original
                .replacen(" : z in A", "", 1)
                .replace(": z in A", ": (z in A)"),
            1,
            vec!["(z in A)"],
        ),
    ]);
    for (header_label, proof_label, both_label, grouped) in [
        (
            "primary condition left header",
            "primary condition left proof",
            "primary condition left",
            "(z) in A",
        ),
        (
            "primary condition right header",
            "primary condition right proof",
            "primary condition right",
            "z in (A)",
        ),
        (
            "primary condition inequality header",
            "primary condition inequality proof",
            "primary condition inequality",
            "((z)) <> (A)",
        ),
        (
            "primary condition equality header",
            "primary condition equality proof",
            "primary condition equality",
            "(z) = (A)",
        ),
        (
            "primary condition formula header",
            "primary condition formula proof",
            "primary condition formula",
            "((z) in (A))",
        ),
    ] {
        cases.push((
            header_label,
            original.replacen(": z in A", &format!(": {grouped}"), 1),
            2,
            vec![grouped, "z in A"],
        ));
        let mut proof = original.clone();
        let last = proof.rfind(": z in A").unwrap();
        proof.replace_range(last..last + ": z in A".len(), &format!(": {grouped}"));
        cases.push((proof_label, proof, 2, vec!["z in A", grouped]));
        cases.push((
            both_label,
            original.replace(": z in A", &format!(": {grouped}")),
            2,
            vec![grouped],
        ));
    }
    cases.push((
        "primary condition Set",
        original
            .replace(set_literal, &format!("({set_literal})"))
            .replace(": z in A", ": (z) in A"),
        2,
        vec!["(z) in A"],
    ));
    cases.extend([
        (
            "primary condition alpha",
            original
                .replace(": z in A", ": ((q)) in (A)")
                .replace("z where z", "q where q"),
            2,
            vec!["((q)) in (A)"],
        ),
        (
            "primary condition layout",
            original.replace(": z in A", ": (  z  ) in ( A )"),
            2,
            vec!["(  z  ) in ( A )"],
        ),
        (
            "primary condition-free sibling",
            original
                .replacen(" : z in A", "", 1)
                .replace(": z in A", ": (z) in A"),
            1,
            vec!["(z) in A"],
        ),
    ]);
    // EN13.1.3/13.4: sealed variable groups retain their actual Set slots.
    for (header_label, proof_label, both_label, mapper) in [
        (
            "primary Set mapper header",
            "primary Set mapper proof",
            "primary Set mapper",
            "(z)",
        ),
        (
            "primary Set nested header",
            "primary Set nested proof",
            "primary Set nested",
            "{ { ((x)) } }",
        ),
    ] {
        let grouped = format!("{{ {mapper} where z is object : z in A }}");
        cases.push((
            header_label,
            original.replacen(set_literal, &grouped, 1),
            2,
            vec!["z in A"],
        ));
        let mut proof = original.clone();
        let last = proof.rfind(set_literal).unwrap();
        proof.replace_range(last..last + set_literal.len(), &grouped);
        cases.push((proof_label, proof, 2, vec!["z in A"]));
        cases.push((
            both_label,
            original.replace(set_literal, &grouped),
            2,
            vec!["z in A"],
        ));
    }
    let mut primary_enumeration_sibling = original.clone();
    let last = primary_enumeration_sibling.rfind(set_literal).unwrap();
    primary_enumeration_sibling.replace_range(last..last + set_literal.len(), "{ (x) }");
    cases.push((
        "primary Set enumeration sibling",
        primary_enumeration_sibling,
        1,
        vec!["z in A"],
    ));
    cases.extend([
        (
            "primary Set elements",
            original.replace(
                set_literal,
                "{ { x, (z), { A }, ((A)) } where z is object : z in A }",
            ),
            2,
            vec!["z in A"],
        ),
        (
            "primary Set alpha",
            original
                .replace("z where z", "((q)) where q")
                .replace(": z in A", ": q in A"),
            2,
            vec!["q in A"],
        ),
        (
            "primary Set layout",
            original.replace("z where z", "(  ( z )  ) where z"),
            2,
            vec!["z in A"],
        ),
        (
            "primary Set mixed conditions",
            original
                .replace("z where z", "(z) where z")
                .replacen(": z in A", ": ((z) in (A))", 1),
            2,
            vec!["((z) in (A))", "z in A"],
        ),
        (
            "primary Set condition-free mapper",
            original
                .replace("z where z", "((z)) where z")
                .replacen(" : z in A", "", 1),
            1,
            vec!["z in A"],
        ),
    ]);
    // EN13.1.3/13.4 and EN14.2 permit grouped Enumeration condition operands.
    cases.push((
        "primary condition Enumeration membership",
        original.replace(": z in A", ": (z) in { A }"),
        2,
        vec!["(z) in { A }"],
    ));
    for (header_label, proof_label, both_label, condition) in [
        (
            "primary condition Enumeration in left header",
            "primary condition Enumeration in left proof",
            "primary condition Enumeration in left",
            "{ A } in z",
        ),
        (
            "primary condition Enumeration in right header",
            "primary condition Enumeration in right proof",
            "primary condition Enumeration in right",
            "z in { A }",
        ),
        (
            "primary condition Enumeration eq left header",
            "primary condition Enumeration eq left proof",
            "primary condition Enumeration eq left",
            "{ A } = z",
        ),
        (
            "primary condition Enumeration eq right header",
            "primary condition Enumeration eq right proof",
            "primary condition Enumeration eq right",
            "z = { A }",
        ),
        (
            "primary condition Enumeration ne left header",
            "primary condition Enumeration ne left proof",
            "primary condition Enumeration ne left",
            "{ A } <> z",
        ),
        (
            "primary condition Enumeration ne right header",
            "primary condition Enumeration ne right proof",
            "primary condition Enumeration ne right",
            "z <> { A }",
        ),
        (
            "primary condition Enumeration grouped in left header",
            "primary condition Enumeration grouped in left proof",
            "primary condition Enumeration grouped in left",
            "(({ A })) in (z)",
        ),
        (
            "primary condition Enumeration grouped in right header",
            "primary condition Enumeration grouped in right proof",
            "primary condition Enumeration grouped in right",
            "(z) in (({ A }))",
        ),
        (
            "primary condition Enumeration grouped eq left header",
            "primary condition Enumeration grouped eq left proof",
            "primary condition Enumeration grouped eq left",
            "(({ A })) = (z)",
        ),
        (
            "primary condition Enumeration grouped eq right header",
            "primary condition Enumeration grouped eq right proof",
            "primary condition Enumeration grouped eq right",
            "(z) = (({ A }))",
        ),
        (
            "primary condition Enumeration grouped ne left header",
            "primary condition Enumeration grouped ne left proof",
            "primary condition Enumeration grouped ne left",
            "(({ A })) <> (z)",
        ),
        (
            "primary condition Enumeration grouped ne right header",
            "primary condition Enumeration grouped ne right proof",
            "primary condition Enumeration grouped ne right",
            "(z) <> (({ A }))",
        ),
    ] {
        cases.push((
            header_label,
            original.replacen(": z in A", &format!(": {condition}"), 1),
            2,
            vec![condition, "z in A"],
        ));
        let mut proof = original.clone();
        let last = proof.rfind(": z in A").unwrap();
        proof.replace_range(last..last + ": z in A".len(), &format!(": {condition}"));
        cases.push((proof_label, proof, 2, vec!["z in A", condition]));
        cases.push((
            both_label,
            original.replace(": z in A", &format!(": {condition}")),
            2,
            vec![condition],
        ));
    }
    cases.extend([
        (
            "primary condition Enumeration empty",
            original.replace(": z in A", ": z in { }"),
            2,
            vec!["z in { }"],
        ),
        (
            "primary condition Enumeration nested",
            original.replace(": z in A", ": z in { A , { ((x)) } , z }"),
            2,
            vec!["z in { A , { ((x)) } , z }"],
        ),
        (
            "primary condition Enumeration both",
            original.replace(": z in A", ": { (z) } = { A }"),
            2,
            vec!["{ (z) } = { A }"],
        ),
        (
            "primary condition Enumeration alpha",
            original
                .replace("z where z", "q where q")
                .replace(": z in A", ": ((q) in { A })"),
            2,
            vec!["((q) in { A })"],
        ),
        (
            "primary condition Enumeration layout",
            original.replace(": z in A", ":  ( z ) in {  A  }"),
            2,
            vec!["( z ) in {  A  }"],
        ),
        (
            "primary condition Enumeration mapper",
            original
                .replace("z where z", "(z) where z")
                .replace(": z in A", ": z in { (A) }"),
            2,
            vec!["z in { (A) }"],
        ),
        (
            "primary condition Enumeration condition-free sibling",
            original
                .replacen(" : z in A", "", 1)
                .replace(": z in A", ": z in { A }"),
            1,
            vec!["z in { A }"],
        ),
    ]);
    for (label, condition) in [
        (
            "primary condition Enumeration grouped membership",
            "(z) in ({ A })",
        ),
        (
            "primary condition Enumeration grouped empty",
            "(z) in (({ }))",
        ),
        (
            "primary condition Enumeration grouped nested",
            "z in (({ A , { ((x)) } , z }))",
        ),
        (
            "primary condition Enumeration grouped both",
            "({ (z) }) = (({ A }))",
        ),
        (
            "primary condition Enumeration grouped alpha",
            "((q) in (({ A })))",
        ),
        (
            "primary condition Enumeration grouped layout",
            "( z ) in ( ( {  A  } ) )",
        ),
        (
            "primary condition Enumeration grouped depth",
            "z in (((({ A }))))",
        ),
    ] {
        let source = if label.ends_with("alpha") {
            original.replace("z where z", "q where q")
        } else {
            original.clone()
        };
        cases.push((
            label,
            source.replace(": z in A", &format!(": {condition}")),
            2,
            vec![condition],
        ));
    }
    let mut foreign_lower = None;
    let mut foreign_composition: Option<
        mizar_checker::source_formula_composition::SourceConditionFormulaCompositionHandoff,
    > = None;
    for (label, source, count, condition_texts) in cases {
        let ast = if label.contains("inequality")
            || label == "mixed conditions"
            || label.starts_with("grouped condition")
            || label.starts_with("grouped equality")
            || label.starts_with("grouped membership")
            || label.starts_with("nested ")
            || label == "outside grouped membership"
            || label.starts_with("primary ")
        {
            let frontend = super::super::super::formula_statement::step5c8_test_frontend(&source);
            assert!(
                frontend.diagnostics.is_empty(),
                "{label}: {:?}",
                frontend.diagnostics
            );
            frontend.ast.unwrap()
        } else {
            parse(&source)
        };
        let symbols = resolver_symbol_collection(root, case, &ast);
        assert!(
            symbols.detail_keys.is_empty(),
            "{label}: {:?}",
            symbols.detail_keys
        );
        let module = &symbols.module;
        let scope = SourceVariableScopeResolver::resolve_occurrences(
            SourceVariableScopeInput::new(&ast, module, &symbols.env),
        )
        .unwrap();
        let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        let typed = super::super::super::type_elaboration::step5c7_membership_typed_ast(
            &ast,
            module,
            &symbols.env,
            &scope,
            &bindings,
        )
        .unwrap_or_else(|error| panic!("{label}: {error}"));
        let primary = typed.source_term().unwrap();
        let sets = typed.source_set_term().unwrap();
        let atomic = typed.source_atomic_formula().unwrap();
        if label.starts_with("primary ") {
            use mizar_checker::source_term::{
                SourcePrimaryTermKind, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceRole,
                SourcePrimaryTermRole,
            };
            assert!(primary.numeric_type_requests().is_empty());
            assert_eq!(primary.references().len(), scope.references().len());
            for ((_, reference), actual) in primary.references().iter().zip(scope.references()) {
                let leaf = primary.terms().get(reference.term()).unwrap();
                assert_eq!(leaf.kind(), SourcePrimaryTermKind::VariableReference);
                assert_eq!(leaf.site().node().index(), actual.node().index());
                assert_eq!(leaf.source_range(), actual.range());
                assert_eq!(leaf.spelling(), actual.spelling());
                assert_eq!(reference.binding().index(), actual.binding().index());
                assert_eq!(reference.lexical_scope(), Some(actual.scope()));
                assert_eq!(reference.use_ordinal(), actual.ordinal());
                assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
            }
            let groups = ast
                .node_views()
                .filter(|view| matches!(view.kind(), SurfaceNodeKind::ParenthesizedTerm))
                .filter(|view| {
                    let mut node = ast.node(view.id()).unwrap();
                    while matches!(
                        node.kind,
                        SurfaceNodeKind::ParenthesizedTerm | SurfaceNodeKind::TermExpression
                    ) {
                        let index =
                            usize::from(matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm));
                        assert_eq!(node.children.len(), if index == 1 { 3 } else { 1 });
                        node = ast.node(node.children[index]).unwrap();
                    }
                    matches!(node.kind, SurfaceNodeKind::TermReference)
                })
                .filter(|view| {
                    ((label.starts_with("primary Set")
                        || label.starts_with("primary condition Enumeration"))
                        && ast.node_views().any(|owner| {
                            matches!(
                                owner.kind(),
                                SurfaceNodeKind::SetComprehension | SurfaceNodeKind::SetEnumeration
                            ) && ast.node(owner.id()).unwrap().children.iter().any(|child| {
                                let slot = ast.node(*child).unwrap();
                                matches!(slot.kind, SurfaceNodeKind::TermExpression)
                                    && slot.range.start <= view.range().start
                                    && view.range().end <= slot.range.end
                            })
                        }))
                        || condition_texts.iter().any(|text| {
                            source.match_indices(*text).any(|(start, _)| {
                                source[..start].trim_end().ends_with(':')
                                    && start <= view.range().start
                                    && view.range().end <= start + text.len()
                            })
                        })
                })
                .collect::<Vec<_>>();
            assert_eq!(
                primary.terms().len(),
                scope.references().len() + groups.len()
            );
            for (id, term) in primary.terms().iter() {
                assert_eq!(id.index(), term.source_ordinal());
                assert_eq!(term.role(), SourcePrimaryTermRole::Value);
                assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
                assert_eq!(
                    term.context(),
                    mizar_checker::binding_env::BindingContextId::new(0)
                );
                if let Some(parent) = term.parent() {
                    assert!(parent.index() < id.index());
                }
            }
            for group in groups {
                let shell = ast.node(group.id()).unwrap();
                assert!(!shell.recovered);
                assert_eq!(
                    shell
                        .children
                        .iter()
                        .filter_map(|child| ast.node(*child).unwrap().token_text())
                        .collect::<Vec<_>>(),
                    ["(", ")"]
                );
                let bridges = shell
                    .children
                    .iter()
                    .filter_map(|child| {
                        let node = ast.node(*child).unwrap();
                        (!matches!(node.kind, SurfaceNodeKind::Token(_))).then_some(node)
                    })
                    .collect::<Vec<_>>();
                let [bridge] = bridges.as_slice() else {
                    panic!("actual Primary sole expression bridge")
                };
                assert!(matches!(bridge.kind, SurfaceNodeKind::TermExpression));
                assert!(!bridge.recovered);
                let [child_id] = bridge.children.as_slice() else {
                    panic!("actual Primary sole child")
                };
                let child = ast.node(*child_id).unwrap();
                assert_eq!(bridge.range, child.range);
                let (parent_id, parent) = primary
                    .terms()
                    .iter()
                    .find(|(_, term)| term.site().node().index() == group.id().index())
                    .expect("actual Primary parent row");
                let (child_id, child_term) = primary
                    .terms()
                    .iter()
                    .find(|(_, term)| term.site().node().index() == child_id.index())
                    .expect("actual Primary child row");
                assert_eq!(parent.kind(), SourcePrimaryTermKind::Parenthesized);
                assert_eq!(parent.source_range(), shell.range);
                assert_eq!(
                    parent.spelling(),
                    source[shell.range.start..shell.range.end]
                        .replace('(', " ( ")
                        .replace(')', " ) ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                assert_eq!(child_term.parent(), Some(parent_id));
                assert!(parent_id.index() < child_id.index());
                assert_eq!(child_term.context(), parent.context());
                assert!(
                    primary
                        .references()
                        .iter()
                        .all(|(_, reference)| reference.term() != parent_id)
                );
                assert_eq!(
                    primary
                        .terms()
                        .iter()
                        .filter(|(_, term)| term.parent() == Some(parent_id))
                        .count(),
                    1
                );
            }
        }
        if matches!(
            label,
            "multiple generators"
                | "three generators"
                | "multiple generators alpha"
                | "unused set generator"
                | "enumeration sibling"
                | "enumeration mapper"
                | "enumeration mapper layout"
                | "enumeration tree"
                | "wrapped mapper"
                | "wrapped mapper alpha"
                | "wrapped element"
                | "wrapped element layout"
                | "double wrapped mapper"
                | "grouped set"
                | "grouped header"
                | "grouped proof"
                | "double grouped set"
                | "grouped alpha"
                | "grouped layout"
                | "grouped left"
                | "grouped enumeration"
                | "inner comprehension boundary"
        ) || label.starts_with("primary Set")
            || label.starts_with("primary condition Enumeration")
        {
            // Compare the lower Set rows with actual syntax, independently of C2.
            let written_sets = ast
                .node_views()
                .filter_map(|view| {
                    let node = ast.node(view.id())?;
                    match &node.kind {
                        SurfaceNodeKind::SetComprehension => {
                            Some((view.id(), node, SourceSetTermKind::Comprehension))
                        }
                        SurfaceNodeKind::SetEnumeration => {
                            Some((view.id(), node, SourceSetTermKind::Enumeration))
                        }
                        _ => None,
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(
                sets.terms().len(),
                written_sets.len(),
                "{label}: Set census"
            );
            let (expected_terms, expected_generators, expected_nested) = match label {
                "multiple generators" | "multiple generators alpha" | "unused set generator" => {
                    (2, 4, 0)
                }
                "three generators" => (2, 6, 0),
                "enumeration sibling" => (2, 1, 0),
                "enumeration mapper" | "enumeration mapper layout" => (4, 2, 2),
                "enumeration tree" => (6, 2, 4),
                "wrapped mapper" | "wrapped mapper alpha" | "double wrapped mapper" => (4, 2, 2),
                "wrapped element" | "wrapped element layout" => (6, 2, 4),
                "grouped enumeration" => (2, 1, 0),
                "grouped set" | "grouped header" | "grouped proof" | "double grouped set"
                | "grouped alpha" | "grouped layout" | "grouped left" => (2, 2, 0),
                "inner comprehension boundary" => (4, 4, 2),
                "primary Set nested" | "primary Set elements" => (6, 2, 4),
                "primary Set nested header" | "primary Set nested proof" => (4, 2, 2),
                "primary Set enumeration sibling" => (2, 1, 0),
                _ if label.starts_with("primary Set") => (2, 2, 0),
                "primary condition Enumeration nested"
                | "primary condition Enumeration grouped nested" => (6, 2, 2),
                "primary condition Enumeration both"
                | "primary condition Enumeration grouped both" => (6, 2, 0),
                "primary condition Enumeration condition-free sibling" => (3, 2, 0),
                _ if label.starts_with("primary condition Enumeration")
                    && (label.ends_with("header") || label.ends_with("proof")) =>
                {
                    (3, 2, 0)
                }
                _ if label.starts_with("primary condition Enumeration") => (4, 2, 0),
                _ => unreachable!(),
            };
            assert_eq!(
                written_sets.len(),
                expected_terms,
                "{label}: written Set count"
            );
            for (site, node, kind) in &written_sets {
                let matching = sets
                    .terms()
                    .iter()
                    .filter(|(_, row)| row.site().node().index() == site.index())
                    .collect::<Vec<_>>();
                let [(term_id, term)] = matching.as_slice() else {
                    panic!("{label}: missing or duplicate written Set")
                };
                assert_eq!(term.source_range(), node.range);
                assert_eq!(term.kind(), *kind);
                if *kind == SourceSetTermKind::Enumeration {
                    assert!(
                        sets.conditions()
                            .iter()
                            .all(|(_, row)| row.term() != *term_id)
                    );
                    assert!(
                        sets.generators()
                            .iter()
                            .all(|(_, row)| row.term() != *term_id)
                    );
                }
            }
            let written_generators = ast
                .node_views()
                .filter(|view| {
                    matches!(
                        ast.node(view.id()).unwrap().kind,
                        SurfaceNodeKind::ComprehensionVariableSegment
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                sets.generators().len(),
                written_generators.len(),
                "{label}: generator census"
            );
            assert_eq!(
                written_generators.len(),
                expected_generators,
                "{label}: written generator count"
            );
            assert_eq!(sets.type_sites().len(), written_generators.len());
            for segment in written_generators {
                let node = ast.node(segment.id()).unwrap();
                let identifier = node
                    .children
                    .iter()
                    .copied()
                    .find(|child| {
                        ast.node(*child)
                            .unwrap()
                            .token_text()
                            .is_some_and(|text| text != "is")
                    })
                    .unwrap();
                let type_expression = node
                    .children
                    .iter()
                    .copied()
                    .find(|child| {
                        matches!(
                            ast.node(*child).unwrap().kind,
                            SurfaceNodeKind::TypeExpression
                        )
                    })
                    .unwrap();
                let (parent_site, parent, _) = written_sets
                    .iter()
                    .find(|(_, parent, kind)| {
                        *kind == SourceSetTermKind::Comprehension
                            && parent.children.contains(&segment.id())
                    })
                    .unwrap();
                let parent_id = sets
                    .terms()
                    .iter()
                    .find(|(_, row)| row.site().node().index() == parent_site.index())
                    .unwrap()
                    .0;
                let (generator_id, generator) = sets
                    .generators()
                    .iter()
                    .find(|(_, row)| row.site().node().index() == identifier.index())
                    .unwrap();
                assert_eq!(generator.term(), parent_id);
                assert_eq!(
                    generator.source_range(),
                    ast.node(identifier).unwrap().range
                );
                let written_ordinal = parent
                    .children
                    .iter()
                    .take_while(|child| **child != segment.id())
                    .filter(|child| {
                        matches!(
                            ast.node(**child).unwrap().kind,
                            SurfaceNodeKind::ComprehensionVariableSegment
                        )
                    })
                    .count();
                assert_eq!(generator.ordinal(), written_ordinal);
                let type_site = sets.type_sites().get(generator.type_site()).unwrap();
                assert_eq!(
                    type_site.owner(),
                    SourceSetTypeOwner::Generator(generator_id)
                );
                assert_eq!(type_site.site().node().index(), type_expression.index());
                assert_eq!(
                    type_site.source_range(),
                    ast.node(type_expression).unwrap().range
                );
                let head = ast
                    .node(type_expression)
                    .unwrap()
                    .children
                    .iter()
                    .copied()
                    .find(|child| {
                        matches!(ast.node(*child).unwrap().kind, SurfaceNodeKind::TypeHead)
                    })
                    .unwrap();
                assert_eq!(type_site.head_site().node().index(), head.index());
                assert_eq!(type_site.head_range(), ast.node(head).unwrap().range);
                let written_tokens = ast
                    .node(head)
                    .unwrap()
                    .children
                    .iter()
                    .filter_map(|child| ast.node(*child).unwrap().token_text())
                    .collect::<Vec<_>>();
                let [written_head] = written_tokens.as_slice() else {
                    panic!("{label}: generator type has no single written head")
                };
                assert_eq!(type_site.spelling(), *written_head);
                assert_eq!(type_site.head_spelling(), *written_head);
                assert_eq!(
                    type_site.head(),
                    match *written_head {
                        "set" => SourceSetTypeHead::BuiltinSet,
                        "object" => SourceSetTypeHead::BuiltinObject,
                        _ => panic!("{label}: unsupported written generator type"),
                    }
                );
                let requests = sets
                    .requests()
                    .iter()
                    .filter(|(_, row)| {
                        row.kind() == SourceSetRequestKind::GeneratorSethood
                            && row.generator() == Some(generator_id)
                    })
                    .collect::<Vec<_>>();
                let [(_, request)] = requests.as_slice() else {
                    panic!("{label}: missing or duplicate generator request")
                };
                assert_eq!(request.term(), parent_id);
                assert_eq!(request.type_site(), Some(generator.type_site()));
            }
            assert_eq!(
                sets.requests()
                    .iter()
                    .filter(|(_, row)| row.kind() == SourceSetRequestKind::ResultType)
                    .count(),
                written_sets.len(),
                "{label}: result request census"
            );
            if label.starts_with("primary Set")
                || label.starts_with("primary condition Enumeration")
            {
                if label.starts_with("primary Set") {
                    assert!(sets.wrappers().is_empty());
                }
                assert_eq!(
                    sets.requests().len(),
                    written_sets.len() + expected_generators
                );
                for (site, node, kind) in &written_sets {
                    assert!(!node.recovered);
                    let (owner_id, _) = sets
                        .terms()
                        .iter()
                        .find(|(_, term)| term.site().node().index() == site.index())
                        .unwrap();
                    let slots = node
                        .children
                        .iter()
                        .filter_map(|child| {
                            let slot = ast.node(*child).unwrap();
                            matches!(slot.kind, SurfaceNodeKind::TermExpression).then_some(slot)
                        })
                        .collect::<Vec<_>>();
                    let edges = sets
                        .edges()
                        .iter()
                        .filter(|(_, edge)| edge.term() == owner_id)
                        .map(|(_, edge)| edge)
                        .collect::<Vec<_>>();
                    assert_eq!(
                        edges.len(),
                        slots.len(),
                        "{label}: complete Set slot census"
                    );
                    for (ordinal, (slot, edge)) in slots.iter().zip(edges).enumerate() {
                        assert!(!slot.recovered);
                        let [root] = slot.children.as_slice() else {
                            panic!("{label}: actual Set slot has no sole root")
                        };
                        assert_eq!(slot.range, ast.node(*root).unwrap().range);
                        assert_eq!(edge.ordinal(), ordinal);
                        assert_eq!(
                            edge.role(),
                            if *kind == SourceSetTermKind::Comprehension {
                                SourceSetEdgeRole::ComprehensionMapper
                            } else {
                                SourceSetEdgeRole::EnumerationElement
                            }
                        );
                        match edge.target() {
                            SourceSetTarget::Primary(id) => {
                                let term = primary.terms().get(id).unwrap();
                                assert!(term.parent().is_none());
                                assert_eq!(term.site().node().index(), root.index());
                                assert_eq!(term.source_range(), slot.range);
                            }
                            SourceSetTarget::SetTerm(id) => {
                                let term = sets.terms().get(id).unwrap();
                                assert_eq!(term.site().node().index(), root.index());
                                assert_eq!(term.source_range(), slot.range);
                            }
                            _ => panic!("{label}: unexpected Set slot target"),
                        }
                    }
                }
            }
            let mut nested_count = 0;
            for (inner_site, inner, _) in &written_sets {
                let parent = written_sets
                    .iter()
                    .filter(|(outer_site, outer, _)| {
                        outer_site != inner_site
                            && outer.range.start <= inner.range.start
                            && inner.range.end <= outer.range.end
                    })
                    .min_by_key(|(_, outer, _)| outer.range.end - outer.range.start);
                if let Some((outer_site, outer, outer_kind)) = parent {
                    if label.starts_with("primary condition Enumeration")
                        && *outer_kind == SourceSetTermKind::Comprehension
                        && outer.children.iter().any(|id| {
                            let child = ast.node(*id).unwrap();
                            matches!(child.kind, SurfaceNodeKind::FormulaExpression)
                                && child.range.start <= inner.range.start
                                && inner.range.end <= child.range.end
                        })
                    {
                        let (inner_id, row) = sets
                            .terms()
                            .iter()
                            .find(|(_, row)| row.site().node().index() == inner_site.index())
                            .unwrap();
                        let owner = sets
                            .terms()
                            .iter()
                            .find(|(_, row)| row.site().node().index() == outer_site.index())
                            .unwrap()
                            .1;
                        assert_eq!(
                            row.context(),
                            owner.context(),
                            "{label}: condition forest context"
                        );
                        assert!(
                            sets.edges().iter().all(
                                |(_, edge)| edge.target() != SourceSetTarget::SetTerm(inner_id)
                            ),
                            "{label}: condition forest root has a Set parent"
                        );
                        continue;
                    }
                    nested_count += 1;
                    let inner_id = sets
                        .terms()
                        .iter()
                        .find(|(_, row)| row.site().node().index() == inner_site.index())
                        .unwrap()
                        .0;
                    let outer_id = sets
                        .terms()
                        .iter()
                        .find(|(_, row)| row.site().node().index() == outer_site.index())
                        .unwrap()
                        .0;
                    let role = if *outer_kind == SourceSetTermKind::Comprehension {
                        SourceSetEdgeRole::ComprehensionMapper
                    } else {
                        SourceSetEdgeRole::EnumerationElement
                    };
                    assert_eq!(
                        sets.edges()
                            .iter()
                            .filter(|(_, edge)| {
                                edge.term() == outer_id
                                    && edge.role() == role
                                    && edge.target() == SourceSetTarget::SetTerm(inner_id)
                            })
                            .count(),
                        1,
                        "{label}: nested mapper ownership"
                    );
                }
            }
            assert_eq!(
                sets.edges()
                    .iter()
                    .filter(|(_, edge)| matches!(edge.target(), SourceSetTarget::SetTerm(_)))
                    .count(),
                nested_count,
                "{label}: nested Set edge census"
            );
            assert_eq!(
                nested_count, expected_nested,
                "{label}: written nested Set count"
            );
            if matches!(
                label,
                "wrapped mapper"
                    | "wrapped mapper alpha"
                    | "wrapped element"
                    | "wrapped element layout"
                    | "double wrapped mapper"
                    | "grouped set"
                    | "grouped header"
                    | "grouped proof"
                    | "double grouped set"
                    | "grouped alpha"
                    | "grouped layout"
                    | "grouped left"
                    | "grouped enumeration"
            ) || label.starts_with("primary condition Enumeration")
            {
                let written_wrappers = ast
                    .node_views()
                    .filter(|view| {
                        matches!(
                            ast.node(view.id()).unwrap().kind,
                            SurfaceNodeKind::ParenthesizedTerm
                        )
                    })
                    .filter(|view| {
                        let mut node = ast.node(view.id()).unwrap();
                        while matches!(
                            node.kind,
                            SurfaceNodeKind::ParenthesizedTerm | SurfaceNodeKind::TermExpression
                        ) {
                            let index = usize::from(matches!(
                                node.kind,
                                SurfaceNodeKind::ParenthesizedTerm
                            ));
                            assert_eq!(node.children.len(), if index == 1 { 3 } else { 1 });
                            node = ast.node(node.children[index]).unwrap();
                        }
                        matches!(
                            node.kind,
                            SurfaceNodeKind::SetEnumeration | SurfaceNodeKind::SetComprehension
                        )
                    })
                    .collect::<Vec<_>>();
                let expected = match label {
                    _ if label.starts_with("primary condition Enumeration") => {
                        written_wrappers.len()
                    }
                    "double wrapped mapper" | "double grouped set" => 4,
                    "grouped header" | "grouped proof" | "grouped enumeration" => 1,
                    _ => 2,
                };
                assert_eq!(
                    written_wrappers.len(),
                    expected,
                    "{label}: written wrappers"
                );
                assert_eq!(sets.wrappers().len(), expected, "{label}: lower wrappers");
                for (inner_id, term) in sets.terms().iter() {
                    let mut physical = written_wrappers
                        .iter()
                        .filter_map(|view| {
                            let node = ast.node(view.id())?;
                            let mut core_site = view.id();
                            let mut core = node;
                            while matches!(
                                core.kind,
                                SurfaceNodeKind::ParenthesizedTerm
                                    | SurfaceNodeKind::TermExpression
                            ) {
                                let index = usize::from(matches!(
                                    core.kind,
                                    SurfaceNodeKind::ParenthesizedTerm
                                ));
                                assert_eq!(core.children.len(), if index == 1 { 3 } else { 1 });
                                core_site = core.children[index];
                                core = ast.node(core_site).unwrap();
                            }
                            (core_site.index() == term.site().node().index())
                                .then_some((view.id(), node))
                        })
                        .collect::<Vec<_>>();
                    physical
                        .sort_by_key(|(_, node)| (node.range.start, usize::MAX - node.range.end));
                    let owned = sets
                        .wrappers()
                        .iter()
                        .filter(|(_, row)| row.term() == inner_id)
                        .collect::<Vec<_>>();
                    assert_eq!(owned.len(), physical.len(), "{label}: wrapper owner census");
                    for (ordinal, ((_, wrapper), (site, node))) in
                        owned.iter().zip(&physical).enumerate()
                    {
                        let tokens = ast
                            .token_views()
                            .filter_map(|view| {
                                let token = ast.node(view.id())?;
                                token.token_text().filter(|_| {
                                    node.range.start <= token.range.start
                                        && token.range.end <= node.range.end
                                })
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(wrapper.term(), inner_id);
                        assert_eq!(wrapper.ordinal(), ordinal);
                        assert_eq!(wrapper.site().node().index(), site.index());
                        assert_eq!(wrapper.source_range(), node.range);
                        assert_eq!(wrapper.spelling(), tokens.join(" "));
                        assert_eq!(wrapper.context(), term.context());
                        assert_eq!(wrapper.recovery(), SourceSetTermRecovery::Normal);
                        assert!(!node.recovered);
                    }
                    if let Some((outer_site, outermost)) = physical.first() {
                        let parent = written_sets
                            .iter()
                            .filter(|(_, parent, _)| {
                                parent.range.start < outermost.range.start
                                    && outermost.range.end < parent.range.end
                                    && !parent.children.iter().any(|id| {
                                        let slot = ast.node(*id).unwrap();
                                        matches!(slot.kind, SurfaceNodeKind::FormulaExpression)
                                            && slot.range.start <= outermost.range.start
                                            && outermost.range.end <= slot.range.end
                                    })
                            })
                            .min_by_key(|(_, parent, _)| parent.range.end - parent.range.start);
                        if let Some((parent_site, _, parent_kind)) = parent {
                            let parent_id = sets
                                .terms()
                                .iter()
                                .find(|(_, row)| row.site().node().index() == parent_site.index())
                                .unwrap()
                                .0;
                            let role = if *parent_kind == SourceSetTermKind::Comprehension {
                                SourceSetEdgeRole::ComprehensionMapper
                            } else {
                                SourceSetEdgeRole::EnumerationElement
                            };
                            assert_eq!(
                                sets.edges()
                                    .iter()
                                    .filter(|(_, edge)| {
                                        edge.term() == parent_id
                                            && edge.role() == role
                                            && edge.target() == SourceSetTarget::SetTerm(inner_id)
                                    })
                                    .count(),
                                1,
                                "{label}: wrapped effective parent edge"
                            );
                        } else {
                            let formula = ast
                                .node_views()
                                .find(|view| {
                                    let node = ast.node(view.id()).unwrap();
                                    matches!(
                                        node.kind,
                                        SurfaceNodeKind::BuiltinPredicateApplication
                                    ) && node.children.iter().any(|operand| {
                                        ast.node(*operand).unwrap().children.contains(outer_site)
                                    })
                                })
                                .expect("wrapped root has an actual Atomic parent");
                            let written = ast.node(formula.id()).unwrap();
                            let side = written
                                .children
                                .iter()
                                .filter(|child| {
                                    !matches!(
                                        ast.node(**child).unwrap().kind,
                                        SurfaceNodeKind::Token(_)
                                    )
                                })
                                .position(|operand| {
                                    ast.node(*operand).unwrap().children.contains(outer_site)
                                })
                                .expect("outermost Set shell is a direct Atomic operand");
                            let formula_id = atomic
                                .formulas()
                                .iter()
                                .find(|(_, row)| row.site().node().index() == formula.id().index())
                                .unwrap()
                                .0;
                            let role = if side == 0 {
                                SourceAtomicEdgeRole::BuiltinLeftOperand
                            } else {
                                SourceAtomicEdgeRole::BuiltinRightOperand
                            };
                            assert_eq!(
                                atomic
                                    .edges()
                                    .iter()
                                    .filter(|(_, edge)| {
                                        edge.formula() == formula_id
                                            && edge.ordinal() == side
                                            && edge.role() == role
                                            && edge.target()
                                                == SourceAtomicTermTarget::SetTerm(inner_id)
                                    })
                                    .count(),
                                1,
                                "{label}: grouped effective Atomic operand"
                            );
                        }
                    }
                }
            }
        }
        if matches!(label, "noncondition equality" | "noncondition inequality") {
            let (operator, typed_kind) = if label == "noncondition equality" {
                ("=", "source.formula.atomic.equality")
            } else {
                ("<>", "source.formula.atomic.inequality")
            };
            let outside = ast
                .node_views()
                .find(|view| {
                    let node = ast.node(view.id()).unwrap();
                    matches!(node.kind, SurfaceNodeKind::BuiltinPredicateApplication)
                        && node
                            .children
                            .iter()
                            .filter_map(|child| ast.node(*child).unwrap().token_text())
                            .collect::<Vec<_>>()
                            == [operator]
                })
                .expect("actual noncondition builtin");
            assert!(
                atomic
                    .formulas()
                    .iter()
                    .all(|(_, row)| row.site().node().index() != outside.id().index())
            );
            assert_ne!(
                typed
                    .nodes()
                    .node(mizar_checker::typed_ast::TypedNodeId::new(
                        outside.id().index()
                    ))
                    .unwrap()
                    .kind
                    .as_str(),
                typed_kind
            );
        }
        let application = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: ast.source_id,
                module_id: module.clone(),
                applications: vec![],
                wrappers: vec![],
                candidates: vec![],
                arguments: vec![],
                type_requests: vec![],
            },
            &symbols.env,
            &bindings,
            primary,
            typed.nodes(),
        )
        .unwrap();
        assert!(application.applications().is_empty());
        let typed_with_application = typed
            .clone()
            .with_source_application(application.clone())
            .unwrap();
        let baseline = super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
            &typed_with_application,
            vec![],
        )
        .unwrap();
        assert!(baseline.source_condition_formula_composition().is_none());
        assert!(baseline.checked_formulas().is_empty());
        assert!(baseline.statement_semantics().is_empty());
        assert!(baseline.checked_proofs().is_empty());

        // The source literal fixes the condition census. Resolve every literal
        // to its actual transparent chain and core atom before looking up
        // lower IDs; no expected association comes from C2 output.
        let mut literal_positions = condition_texts
            .iter()
            .flat_map(|condition_text| {
                source
                    .match_indices(*condition_text)
                    .filter_map(|(start, _)| {
                        source[..start].trim_end().ends_with(':').then_some((
                            start,
                            start + condition_text.len(),
                            *condition_text,
                        ))
                    })
            })
            .collect::<Vec<_>>();
        literal_positions.sort_by_key(|(start, _, _)| *start);
        assert_eq!(literal_positions.len(), count, "{label}");
        assert_eq!(sets.conditions().len(), count, "{label}");
        let mut edges = Vec::new();
        let mut expected_wrappers = Vec::new();
        for (start, end, condition_text) in literal_positions {
            let (wrapper_id, wrapper) = ast
                .node_views()
                .find_map(|view| {
                    let node = ast.node(view.id())?;
                    (matches!(node.kind, SurfaceNodeKind::FormulaExpression)
                        && node.range.start == start
                        && node.range.end == end)
                        .then_some((view.id(), node))
                })
                .expect("literal condition wrapper");
            let children = wrapper
                .children
                .iter()
                .copied()
                .filter(|id| !matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::Token(_)))
                .collect::<Vec<_>>();
            let [first] = children.as_slice() else {
                panic!("{label}: condition has no single effective occurrence")
            };
            let mut formula_id = *first;
            let mut shells = Vec::new();
            while matches!(
                ast.node(formula_id).unwrap().kind,
                SurfaceNodeKind::ParenthesizedFormula
            ) {
                let shell = ast.node(formula_id).unwrap();
                assert!(!shell.recovered);
                assert_eq!(
                    shell
                        .children
                        .iter()
                        .filter_map(|child| ast.node(*child).unwrap().token_text())
                        .collect::<Vec<_>>(),
                    ["(", ")"]
                );
                shells.push(formula_id);
                let bridges = shell
                    .children
                    .iter()
                    .copied()
                    .filter(|child| {
                        !matches!(ast.node(*child).unwrap().kind, SurfaceNodeKind::Token(_))
                    })
                    .collect::<Vec<_>>();
                let [bridge_id] = bridges.as_slice() else {
                    panic!("{label}: grouped condition bridge census")
                };
                let bridge = ast.node(*bridge_id).unwrap();
                assert!(matches!(bridge.kind, SurfaceNodeKind::FormulaExpression));
                assert!(!bridge.recovered);
                let inner = bridge
                    .children
                    .iter()
                    .copied()
                    .filter(|child| {
                        !matches!(ast.node(*child).unwrap().kind, SurfaceNodeKind::Token(_))
                    })
                    .collect::<Vec<_>>();
                let [child] = inner.as_slice() else {
                    panic!("{label}: grouped condition inner census")
                };
                assert_eq!(bridge.range, ast.node(*child).unwrap().range);
                formula_id = *child;
            }
            let formula = ast.node(formula_id).unwrap();
            assert!(matches!(
                formula.kind,
                SurfaceNodeKind::BuiltinPredicateApplication
            ));
            assert_eq!(ast.node(*first).unwrap().range, wrapper.range);
            let (expected_kind, expected_operator) = if condition_text.contains(" <> ") {
                (SourceAtomicFormulaKind::Inequality, "<>")
            } else if condition_text.contains(" = ") {
                (SourceAtomicFormulaKind::Equality, "=")
            } else {
                (SourceAtomicFormulaKind::Membership, "in")
            };
            let written_operator = formula
                .children
                .iter()
                .filter_map(|child| ast.node(*child).unwrap().token_text())
                .collect::<Vec<_>>();
            assert_eq!(written_operator, [expected_operator]);
            let (condition_id, condition) = sets
                .conditions()
                .iter()
                .find(|(_, row)| row.condition_site().node().index() == wrapper_id.index())
                .expect("actual lower condition wrapper");
            let set_parent = ast
                .node_views()
                .find_map(|view| {
                    let node = ast.node(view.id())?;
                    (matches!(node.kind, SurfaceNodeKind::SetComprehension)
                        && node.children.contains(&wrapper_id))
                    .then_some(view.id())
                })
                .expect("actual comprehension parent");
            let set_id = sets
                .terms()
                .iter()
                .find(|(_, row)| row.site().node().index() == set_parent.index())
                .map(|(id, _)| id)
                .expect("actual lower set owner");
            let (atomic_id, atomic_row) = atomic
                .formulas()
                .iter()
                .find(|(_, row)| row.site().node().index() == formula_id.index())
                .expect("actual lower direct condition atom");
            assert_eq!(condition.source_range(), wrapper.range);
            assert_eq!(atomic_row.source_range(), formula.range);
            let canonical = condition_text
                .replace('(', " ( ")
                .replace(')', " ) ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            assert_eq!(condition.spelling(), canonical);
            assert_eq!(
                atomic_row.spelling(),
                source[formula.range.start..formula.range.end]
                    .replace('(', " ( ")
                    .replace(')', " ) ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            assert_eq!(atomic_row.recovery(), SourceAtomicFormulaRecovery::Normal);
            for (ordinal, shell_id) in shells.into_iter().enumerate() {
                let shell = ast.node(shell_id).unwrap();
                let literal = &source[shell.range.start..shell.range.end];
                let spelling = literal
                    .replace('(', " ( ")
                    .replace(')', " ) ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                expected_wrappers.push((
                    atomic_id,
                    ordinal,
                    shell_id,
                    shell.range,
                    atomic_row.context(),
                    spelling,
                ));
            }
            assert_eq!(atomic_row.kind(), expected_kind);
            assert_eq!(condition.term(), set_id);
            assert_eq!(condition.ordinal(), 0);
            assert_eq!(
                atomic_row.context(),
                sets.terms().get(condition.term()).unwrap().context()
            );
            let formula_edges = atomic
                .edges()
                .iter()
                .filter(|(_, row)| row.formula() == atomic_id)
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            assert_eq!(formula_edges.len(), 2);
            if label.starts_with("primary condition Enumeration") {
                let slots = formula
                    .children
                    .iter()
                    .filter_map(|id| {
                        let node = ast.node(*id).unwrap();
                        (!matches!(node.kind, SurfaceNodeKind::Token(_))).then_some(node)
                    })
                    .collect::<Vec<_>>();
                assert_eq!(slots.len(), 2);
                let mut roots = Vec::new();
                for (ordinal, slot) in slots.iter().enumerate() {
                    assert!(matches!(slot.kind, SurfaceNodeKind::TermExpression));
                    assert!(!slot.recovered);
                    let [root] = slot.children.as_slice() else {
                        panic!("{label}: condition operand has no sole bare root")
                    };
                    let mut root = *root;
                    let mut node = ast.node(root).unwrap();
                    assert_eq!(slot.range, node.range);
                    while matches!(
                        node.kind,
                        SurfaceNodeKind::ParenthesizedTerm | SurfaceNodeKind::TermExpression
                    ) {
                        let index =
                            usize::from(matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm));
                        assert_eq!(node.children.len(), if index == 1 { 3 } else { 1 });
                        root = node.children[index];
                        node = ast.node(root).unwrap();
                    }
                    if matches!(node.kind, SurfaceNodeKind::SetEnumeration) {
                        let (id, term) = sets
                            .terms()
                            .iter()
                            .find(|(_, term)| term.site().node().index() == root.index())
                            .unwrap();
                        assert_eq!(term.context(), atomic_row.context());
                        assert_eq!(
                            atomic.edges().get(formula_edges[ordinal]).unwrap().target(),
                            SourceAtomicTermTarget::SetTerm(id)
                        );
                        roots.push(id);
                    }
                }
                let forest = sets
                    .terms()
                    .iter()
                    .filter(|(id, term)| {
                        term.kind() == SourceSetTermKind::Enumeration
                            && wrapper.range.start <= term.source_range().start
                            && term.source_range().end <= wrapper.range.end
                            && sets
                                .edges()
                                .iter()
                                .all(|(_, edge)| edge.target() != SourceSetTarget::SetTerm(*id))
                    })
                    .map(|(id, _)| id)
                    .collect::<Vec<_>>();
                assert_eq!(
                    forest, roots,
                    "{label}: complete condition Enumeration forest census"
                );
            }
            if expected_kind == SourceAtomicFormulaKind::Inequality
                || label.starts_with("grouped condition")
                || label.starts_with("grouped equality")
                || label.starts_with("grouped membership")
                || label.starts_with("nested ")
                || label.starts_with("primary ")
            {
                let operands = formula
                    .children
                    .iter()
                    .filter_map(|child| {
                        let node = ast.node(*child).unwrap();
                        (!matches!(node.kind, SurfaceNodeKind::Token(_))).then_some(node)
                    })
                    .collect::<Vec<_>>();
                assert_eq!(operands.len(), 2);
                for (ordinal, operand) in operands.into_iter().enumerate() {
                    let mut core_site = operand.children[0];
                    while matches!(
                        ast.node(core_site).unwrap().kind,
                        SurfaceNodeKind::ParenthesizedTerm | SurfaceNodeKind::TermExpression
                    ) {
                        let node = ast.node(core_site).unwrap();
                        let index =
                            usize::from(matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm));
                        core_site = node.children[index];
                    }
                    let target = primary
                        .terms()
                        .iter()
                        .find(|(_, term)| {
                            term.parent().is_none() && term.source_range() == operand.range
                        })
                        .map(|(id, _)| SourceAtomicTermTarget::Primary(id))
                        .or_else(|| {
                            sets.terms()
                                .iter()
                                .find(|(_, term)| term.site().node().index() == core_site.index())
                                .map(|(id, _)| SourceAtomicTermTarget::SetTerm(id))
                        })
                        .expect("actual written operand root");
                    let edge = atomic.edges().get(formula_edges[ordinal]).unwrap();
                    assert_eq!(edge.ordinal(), ordinal);
                    assert_eq!(
                        edge.role(),
                        if ordinal == 0 {
                            SourceAtomicEdgeRole::BuiltinLeftOperand
                        } else {
                            SourceAtomicEdgeRole::BuiltinRightOperand
                        }
                    );
                    assert_eq!(edge.target(), target);
                }
            }
            let formula_requests = atomic
                .requests()
                .iter()
                .filter(|(_, row)| row.formula() == atomic_id)
                .map(|(_, row)| row)
                .collect::<Vec<_>>();
            let expected_request_edges = if expected_kind == SourceAtomicFormulaKind::Membership {
                &formula_edges[1..]
            } else {
                formula_edges.as_slice()
            };
            assert_eq!(formula_requests.len(), expected_request_edges.len());
            for (ordinal, (request, edge)) in formula_requests
                .into_iter()
                .zip(expected_request_edges)
                .enumerate()
            {
                assert_eq!(request.ordinal(), ordinal);
                assert_eq!(request.kind(), SourceAtomicRequestKind::OperandExpectedType);
                assert_eq!(request.edge(), Some(*edge));
                assert!(request.candidate().is_none());
                assert!(request.type_site().is_none());
                assert!(request.attribute().is_none());
            }
            edges.push(SourceConditionFormulaEdgeInput {
                condition: condition_id,
                ordinal: 0,
                formula: atomic_id,
            });
        }
        assert_eq!(
            atomic.wrappers().len(),
            expected_wrappers.len(),
            "{label}: complete wrapper census"
        );
        for ((_, actual), (formula, ordinal, site, range, context, spelling)) in
            atomic.wrappers().iter().zip(&expected_wrappers)
        {
            assert_eq!(actual.formula(), *formula);
            assert_eq!(actual.ordinal(), *ordinal);
            assert_eq!(actual.site().node().index(), site.index());
            assert_eq!(actual.source_range(), *range);
            assert_eq!(actual.context(), *context);
            assert_eq!(actual.spelling(), spelling);
            assert_eq!(actual.recovery(), SourceAtomicFormulaRecovery::Normal);
        }
        let input = SourceConditionFormulaCompositionHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            edges,
        };
        let build = |candidate| {
            SourceConditionFormulaCompositionProducer::build(
                candidate,
                primary,
                &application,
                sets,
                atomic,
                typed.nodes(),
            )
        };
        if count == 0 {
            assert!(build(input).is_err(), "zero-condition C2 admitted");
            continue;
        }
        if label == "inner comprehension boundary" {
            assert!(
                build(input).is_err(),
                "nested comprehension mapper admitted"
            );
            continue;
        }
        if matches!(
            label,
            "noncondition inequality" | "outside grouped membership"
        ) {
            use mizar_checker::{
                source_atomic_formula::{
                    SourceAtomicEdgeId, SourceAtomicEdgeInput, SourceAtomicFormulaHandoffInput,
                    SourceAtomicFormulaId, SourceAtomicFormulaInput, SourceAtomicFormulaProducer,
                    SourceAtomicRequestInput, SourceAtomicWrapperInput,
                },
                typed_ast::{TypedArena, TypedNodeId, TypedSiteRef},
            };
            let comparison = label == "noncondition inequality";
            let shift = usize::from(comparison);
            let outside = ast
                .node_views()
                .find(|view| {
                    let node = ast.node(view.id()).unwrap();
                    matches!(node.kind, SurfaceNodeKind::BuiltinPredicateApplication)
                        && &source[node.range.start..node.range.end]
                            == if comparison { "x <> x" } else { "x in A" }
                        && (comparison || node.range.start == source.find("(x in A)").unwrap() + 1)
                })
                .unwrap();
            let outside_node = ast.node(outside.id()).unwrap();
            let mut formulas = Vec::new();
            let mut receiver_edges = Vec::new();
            let mut requests = Vec::new();
            if comparison {
                formulas.push(SourceAtomicFormulaInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(outside.id().index())),
                    source_range: outside_node.range,
                    source_ordinal: 0,
                    context: mizar_checker::binding_env::BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: "x <> x".into(),
                    kind: SourceAtomicFormulaKind::Inequality,
                });
                receiver_edges.extend(
                    outside_node
                        .children
                        .iter()
                        .filter_map(|child| {
                            let operand = ast.node(*child).unwrap();
                            (!matches!(operand.kind, SurfaceNodeKind::Token(_)))
                                .then_some(operand.range)
                        })
                        .enumerate()
                        .map(|(ordinal, range)| SourceAtomicEdgeInput {
                            formula: SourceAtomicFormulaId::new(0),
                            ordinal,
                            role: if ordinal == 0 {
                                SourceAtomicEdgeRole::BuiltinLeftOperand
                            } else {
                                SourceAtomicEdgeRole::BuiltinRightOperand
                            },
                            target: SourceAtomicTermTarget::Primary(
                                primary
                                    .terms()
                                    .iter()
                                    .find(|(_, row)| row.source_range() == range)
                                    .unwrap()
                                    .0,
                            ),
                        }),
                );
                assert_eq!(receiver_edges.len(), 2);
                requests.extend((0..2).map(|ordinal| SourceAtomicRequestInput {
                    formula: SourceAtomicFormulaId::new(0),
                    ordinal,
                    kind: SourceAtomicRequestKind::OperandExpectedType,
                    edge: Some(SourceAtomicEdgeId::new(ordinal)),
                    candidate: None,
                    type_site: None,
                    attribute: None,
                }));
            }
            formulas.extend(
                atomic
                    .formulas()
                    .iter()
                    .map(|(_, row)| SourceAtomicFormulaInput {
                        site: row.site().clone(),
                        source_range: row.source_range(),
                        source_ordinal: row.source_ordinal() + shift,
                        context: row.context(),
                        recovery: row.recovery(),
                        spelling: row.spelling().into(),
                        kind: row.kind(),
                    }),
            );
            receiver_edges.extend(atomic.edges().iter().map(|(_, row)| SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(row.formula().index() + shift),
                ordinal: row.ordinal(),
                role: row.role(),
                target: row.target(),
            }));
            requests.extend(atomic.requests().iter().map(|(_, row)| {
                SourceAtomicRequestInput {
                    formula: SourceAtomicFormulaId::new(row.formula().index() + shift),
                    ordinal: row.ordinal(),
                    kind: row.kind(),
                    edge: row
                        .edge()
                        .map(|edge| SourceAtomicEdgeId::new(edge.index() + 2 * shift)),
                    candidate: row.candidate(),
                    type_site: row.type_site(),
                    attribute: row.attribute(),
                }
            }));
            let mut wrappers = atomic
                .wrappers()
                .iter()
                .map(|(_, row)| SourceAtomicWrapperInput {
                    formula: SourceAtomicFormulaId::new(row.formula().index() + shift),
                    ordinal: row.ordinal(),
                    site: row.site().clone(),
                    source_range: row.source_range(),
                    context: row.context(),
                    recovery: row.recovery(),
                    spelling: row.spelling().into(),
                })
                .collect::<Vec<_>>();
            let mut nodes = typed
                .nodes()
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            if comparison {
                nodes[outside.id().index()].kind = "source.formula.atomic.inequality".into();
            } else {
                let shell = ast
                    .node_views()
                    .find(|view| {
                        let node = ast.node(view.id()).unwrap();
                        matches!(node.kind, SurfaceNodeKind::ParenthesizedFormula)
                            && &source[node.range.start..node.range.end] == "(x in A)"
                    })
                    .unwrap();
                let shell_node = ast.node(shell.id()).unwrap();
                let bridge = shell_node
                    .children
                    .iter()
                    .filter_map(|child| {
                        let node = ast.node(*child).unwrap();
                        matches!(node.kind, SurfaceNodeKind::FormulaExpression).then_some(node)
                    })
                    .next()
                    .unwrap();
                assert!(bridge.children.contains(&outside.id()));
                let (formula, row) = atomic
                    .formulas()
                    .iter()
                    .find(|(_, row)| row.site().node().index() == outside.id().index())
                    .unwrap();
                assert_eq!(row.kind(), SourceAtomicFormulaKind::Membership);
                nodes[shell.id().index()].kind = "source.formula.atomic.parenthesized".into();
                wrappers.insert(
                    0,
                    SourceAtomicWrapperInput {
                        formula,
                        ordinal: 0,
                        site: TypedSiteRef::Node(TypedNodeId::new(shell.id().index())),
                        source_range: shell_node.range,
                        context: row.context(),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "( x in A )".into(),
                    },
                );
            }
            let receiver_arena = TypedArena::try_new(typed.nodes().root(), nodes).unwrap();
            let receiver_atomic = SourceAtomicFormulaProducer::build(
                SourceAtomicFormulaHandoffInput {
                    source_id: ast.source_id,
                    module_id: module.clone(),
                    formulas,
                    wrappers,
                    predicate_segments: vec![],
                    predicate_heads: vec![],
                    candidates: vec![],
                    type_sites: vec![],
                    attributes: vec![],
                    edges: receiver_edges,
                    requests,
                },
                &bindings,
                &symbols.env,
                primary,
                None,
                None,
                Some(sets),
                &receiver_arena,
            )
            .expect("genuine independently valid outside-condition Atomic input");
            let mut receiver_input = input.clone();
            for edge in &mut receiver_input.edges {
                edge.formula = SourceAtomicFormulaId::new(edge.formula.index() + shift);
            }
            assert!(
                SourceConditionFormulaCompositionProducer::build(
                    receiver_input,
                    primary,
                    &application,
                    sets,
                    &receiver_atomic,
                    &receiver_arena,
                )
                .is_err(),
                "{label}: outside-condition atom/wrapper admitted by C2"
            );
        }
        let composition = build(input.clone()).expect("RED native C2 associations");
        assert_eq!(composition.edges().len(), count);
        for (index, (id, row)) in composition.edges().iter().enumerate() {
            assert_eq!(id.index(), index);
            assert_eq!(row.condition(), input.edges[index].condition);
            assert_eq!(row.ordinal(), input.edges[index].ordinal);
            assert_eq!(row.formula(), input.edges[index].formula);
        }
        assert_eq!(composition.primary_term_fingerprint(), primary.debug_text());
        assert_eq!(
            composition.application_fingerprint(),
            application.debug_text()
        );
        assert_eq!(composition.set_term_fingerprint(), sets.debug_text());
        assert_eq!(
            composition.atomic_formula_fingerprint(),
            atomic.debug_text()
        );
        if label == "nested inequality" {
            use mizar_checker::typed_ast::{NodeRecoveryState, TypedArena, TypedNodeId};
            let (_, _, outer, _, _, _) = expected_wrappers.first().unwrap();
            let condition = sets.conditions().get(input.edges[0].condition).unwrap();
            let bridge = ast
                .node(*outer)
                .unwrap()
                .children
                .iter()
                .copied()
                .find(|child| {
                    matches!(
                        ast.node(*child).unwrap().kind,
                        SurfaceNodeKind::FormulaExpression
                    )
                })
                .unwrap();
            let core = atomic
                .formulas()
                .get(input.edges[0].formula)
                .unwrap()
                .site()
                .node();
            for mutation in 0..5 {
                let mut nodes = typed
                    .nodes()
                    .iter()
                    .map(|(_, node)| node.clone())
                    .collect::<Vec<_>>();
                match mutation {
                    0 => nodes[condition.condition_site().node().index()].children = vec![core],
                    1 => nodes[bridge.index()].children = vec![core],
                    2 => nodes[bridge.index()].recovery = NodeRecoveryState::Degraded,
                    3 => nodes[bridge.index()].children.push(core),
                    4 => nodes[outer.index()].children[0] = TypedNodeId::new(bridge.index()),
                    _ => unreachable!(),
                }
                let changed = TypedArena::try_new(typed.nodes().root(), nodes).unwrap();
                assert!(
                    SourceConditionFormulaCompositionProducer::build(
                        input.clone(),
                        primary,
                        &application,
                        sets,
                        atomic,
                        &changed,
                    )
                    .is_err(),
                    "source chain mutation {mutation} admitted"
                );
            }
            assert_eq!(
                build(input.clone()).unwrap(),
                composition,
                "source-chain pristine replay"
            );
        }
        if matches!(
            label,
            "primary condition inequality"
                | "primary Set elements"
                | "primary condition Enumeration membership"
                | "primary condition Enumeration grouped in right"
        ) {
            use mizar_checker::{
                source_term::SourcePrimaryTermKind,
                typed_ast::{NodeRecoveryState, TypedArena, TypedAst, TypedAstParts},
            };
            let grouped_set = label == "primary condition Enumeration grouped in right";
            let (group_id, group_site) = if grouped_set {
                (
                    None,
                    sets.wrappers()
                        .iter()
                        .find(|(_, row)| row.ordinal() == 0)
                        .unwrap()
                        .1
                        .site()
                        .node(),
                )
            } else {
                let (id, group) = primary
                    .terms()
                    .iter()
                    .find(|(_, term)| term.kind() == SourcePrimaryTermKind::Parenthesized)
                    .unwrap();
                (Some(id), group.site().node())
            };
            let shell = ast.nodes().get(group_site.index()).unwrap();
            let bridge_id = shell
                .children
                .iter()
                .copied()
                .find(|child| {
                    matches!(
                        ast.node(*child).unwrap().kind,
                        SurfaceNodeKind::TermExpression
                    )
                })
                .unwrap();
            let child = ast.node(bridge_id).unwrap().children[0];
            let set_slot = if label == "primary condition Enumeration membership" || grouped_set {
                let owner = atomic
                    .formulas()
                    .get(input.edges[0].formula)
                    .unwrap()
                    .site()
                    .node();
                let slots = ast.nodes()[owner.index()]
                    .children
                    .iter()
                    .copied()
                    .filter(|id| {
                        matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TermExpression)
                    })
                    .collect::<Vec<_>>();
                Some((owner, slots, 1))
            } else {
                sets.edges().iter().find_map(|(_, edge)| {
                    (group_id.is_some_and(|id| edge.target() == SourceSetTarget::Primary(id))).then(
                        || {
                            let owner = sets.terms().get(edge.term()).unwrap().site().node();
                            let slots = ast.nodes()[owner.index()]
                                .children
                                .iter()
                                .copied()
                                .filter(|id| {
                                    matches!(
                                        ast.node(*id).unwrap().kind,
                                        SurfaceNodeKind::TermExpression
                                    )
                                })
                                .collect::<Vec<_>>();
                            (owner, slots, edge.ordinal())
                        },
                    )
                })
            };
            let inner = ast.node(child).unwrap();
            let inner_bridge = grouped_set.then(|| {
                inner
                    .children
                    .iter()
                    .copied()
                    .find(|id| {
                        matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TermExpression)
                    })
                    .unwrap()
            });
            for mutation in 0..if grouped_set {
                33
            } else if set_slot.is_some() {
                14
            } else {
                4
            } {
                let mut nodes = typed
                    .nodes()
                    .iter()
                    .map(|(_, node)| node.clone())
                    .collect::<Vec<_>>();
                match mutation {
                    0 => nodes[bridge_id.index()].children.clear(),
                    1 => nodes[bridge_id.index()]
                        .children
                        .push(mizar_checker::typed_ast::TypedNodeId::new(child.index())),
                    2 => nodes[bridge_id.index()].recovery = NodeRecoveryState::Degraded,
                    3 => {
                        nodes[shell.children[2].index()].kind =
                            nodes[shell.children[0].index()].kind.clone()
                    }
                    4..=13 => {
                        let (owner, slots, ordinal) = set_slot.as_ref().unwrap();
                        let owner_id = owner.index();
                        let slot = slots[*ordinal].index();
                        match mutation {
                            4 => nodes[owner_id].children.retain(|id| id.index() != slot),
                            5 => {
                                let foreign = ast
                                    .node_views()
                                    .filter(|view| {
                                        matches!(
                                            view.kind(),
                                            SurfaceNodeKind::SetComprehension
                                                | SurfaceNodeKind::SetEnumeration
                                        )
                                    })
                                    .flat_map(|view| {
                                        ast.node(view.id()).unwrap().children.iter().copied()
                                    })
                                    .find(|id| {
                                        !slots.contains(id)
                                            && matches!(
                                                ast.node(*id).unwrap().kind,
                                                SurfaceNodeKind::TermExpression
                                            )
                                    })
                                    .unwrap();
                                nodes[owner_id].children.push(
                                    mizar_checker::typed_ast::TypedNodeId::new(foreign.index()),
                                );
                            }
                            6 => nodes[owner_id]
                                .children
                                .push(mizar_checker::typed_ast::TypedNodeId::new(slot)),
                            7 => {
                                let first = nodes[owner_id]
                                    .children
                                    .iter()
                                    .position(|id| id.index() == slots[0].index())
                                    .unwrap();
                                let grouped = nodes[owner_id]
                                    .children
                                    .iter()
                                    .position(|id| id.index() == slot)
                                    .unwrap();
                                nodes[owner_id].children.swap(first, grouped);
                            }
                            8 => nodes[slot].children = nodes[slots[0].index()].children.clone(),
                            9 => nodes[slot].children.clear(),
                            10 => nodes[slot].children.push(group_site),
                            11 => nodes[slot].recovery = NodeRecoveryState::Degraded,
                            12 => nodes[slot].anchor = nodes[slots[0].index()].anchor.clone(),
                            13 => nodes[slot].kind = nodes[owner_id].kind.clone(),
                            _ => unreachable!(),
                        }
                    }
                    14..=32 => {
                        let outer = group_site.index();
                        let bridge = bridge_id.index();
                        let inner_id = child.index();
                        let inner_bridge = inner_bridge.unwrap().index();
                        let core = nodes[inner_bridge].children[0];
                        match mutation {
                            14 => {
                                nodes[bridge].children = vec![
                                    primary
                                        .terms()
                                        .iter()
                                        .find(|(_, row)| {
                                            row.kind() == SourcePrimaryTermKind::VariableReference
                                                && sets.conditions().iter().any(|(_, condition)| {
                                                    condition.source_range().start
                                                        <= row.source_range().start
                                                        && row.source_range().end
                                                            <= condition.source_range().end
                                                        && condition.source_range().start
                                                            <= shell.range.start
                                                        && shell.range.end
                                                            <= condition.source_range().end
                                                })
                                        })
                                        .unwrap()
                                        .1
                                        .site()
                                        .node(),
                                ]
                            }
                            15 => nodes[outer].children.swap(0, 2),
                            16 => nodes[bridge].children = nodes[inner_bridge].children.clone(),
                            17 => nodes[bridge].kind = nodes[outer].kind.clone(),
                            18 => nodes[bridge].anchor = nodes[outer].anchor.clone(),
                            19 => nodes[outer].kind = nodes[bridge].kind.clone(),
                            20 => nodes[outer].recovery = NodeRecoveryState::Degraded,
                            21 => nodes[outer].anchor = nodes[bridge].anchor.clone(),
                            22 => nodes[shell.children[0].index()].children.push(core),
                            23 => {
                                nodes[shell.children[0].index()].anchor =
                                    nodes[shell.children[2].index()].anchor.clone()
                            }
                            24 => nodes[inner_bridge].children.clear(),
                            25 => nodes[inner_id].kind = nodes[bridge].kind.clone(),
                            26 => nodes[inner_bridge].anchor = nodes[inner_id].anchor.clone(),
                            27 => nodes[inner_id].recovery = NodeRecoveryState::Degraded,
                            28 => {
                                nodes[outer].children.remove(1);
                            }
                            29 => nodes[outer]
                                .children
                                .push(mizar_checker::typed_ast::TypedNodeId::new(bridge)),
                            30 => {
                                nodes[outer].children[1] =
                                    mizar_checker::typed_ast::TypedNodeId::new(inner_bridge)
                            }
                            31 => {
                                nodes[inner.children[2].index()].kind =
                                    nodes[inner.children[0].index()].kind.clone()
                            }
                            32 => {
                                nodes[shell.children[0].index()].recovery =
                                    NodeRecoveryState::Degraded
                            }
                            _ => unreachable!(),
                        };
                    }
                    _ => unreachable!(),
                }
                let changed = TypedArena::try_new(typed.nodes().root(), nodes).unwrap();
                assert!(
                    SourceConditionFormulaCompositionProducer::build(
                        input.clone(),
                        primary,
                        &application,
                        sets,
                        atomic,
                        &changed,
                    )
                    .is_err(),
                    "Primary/Set source chain mutation {mutation} admitted by C2"
                );
                let changed_typed = TypedAst::try_new(TypedAstParts {
                    source_id: typed.source_id(),
                    module_id: typed.module_id().clone(),
                    resolved_root: typed.resolved_root(),
                    source_context: typed.source_context().cloned(),
                    source_type: typed.source_type().cloned(),
                    source_attribute: typed.source_attribute().cloned(),
                    nodes: changed,
                    contexts: typed.contexts().clone(),
                    types: typed.types().clone(),
                    facts: typed.facts().clone(),
                    coercions: typed.coercions().clone(),
                    initial_obligations: typed.initial_obligations().clone(),
                    diagnostics: typed.diagnostics().clone(),
                })
                .unwrap()
                .with_source_term(primary.clone())
                .expect("standalone Primary retains metadata-only installation")
                .with_source_application(application.clone())
                .unwrap()
                .with_source_set_term(sets.clone());
                if grouped_set && matches!(mutation, 19 | 20 | 21 | 25 | 27) {
                    assert!(
                        changed_typed.is_err(),
                        "Set wrapper metadata mutation {mutation} admitted by Set lower"
                    );
                    continue;
                }
                let changed_typed = changed_typed
                    .unwrap_or_else(|error| panic!("source chain mutation {mutation}: {error:?}"))
                    .with_source_atomic_formula(atomic.clone())
                    .unwrap();
                assert!(
                    changed_typed
                        .clone()
                        .with_source_condition_formula_composition(composition.clone())
                        .is_err(),
                    "Primary/Set source chain mutation {mutation} admitted by Typed C2 replay"
                );
                assert!(
                    changed_typed
                        .source_condition_formula_composition()
                        .is_none()
                );
            }
            assert_eq!(
                build(input.clone()).unwrap(),
                composition,
                "Primary/Set source-chain pristine replay"
            );
        }
        let installed = typed_with_application
            .clone()
            .with_source_condition_formula_composition(composition.clone())
            .unwrap();
        assert_eq!(
            installed.source_term(),
            typed_with_application.source_term()
        );
        assert_eq!(
            installed.source_application(),
            typed_with_application.source_application()
        );
        assert_eq!(
            installed.source_set_term(),
            typed_with_application.source_set_term()
        );
        assert_eq!(
            installed.source_atomic_formula(),
            typed_with_application.source_atomic_formula()
        );
        let final_ast = super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
            &installed,
            vec![],
        )
        .unwrap();
        assert_eq!(
            final_ast.source_condition_formula_composition(),
            Some(&composition)
        );
        assert_eq!(final_ast.source_term(), baseline.source_term());
        assert_eq!(
            final_ast.source_application(),
            baseline.source_application()
        );
        assert_eq!(final_ast.source_set_term(), baseline.source_set_term());
        assert_eq!(
            final_ast.source_atomic_formula(),
            baseline.source_atomic_formula()
        );
        assert!(final_ast.checked_formulas().is_empty());
        assert!(final_ast.statement_semantics().is_empty());
        assert!(final_ast.checked_proofs().is_empty());
        if label.starts_with("primary ")
            || !expected_wrappers.is_empty()
            || matches!(
                label,
                "original"
                    | "equality"
                    | "inequality"
                    | "header inequality"
                    | "proof inequality"
                    | "inequality distinct operands"
                    | "inequality condition-free sibling"
                    | "multiple generators"
                    | "unused set generator"
                    | "enumeration sibling"
                    | "enumeration mapper"
                    | "wrapped mapper"
                    | "wrapped element"
                    | "double wrapped mapper"
                    | "grouped set"
                    | "grouped header"
                    | "grouped proof"
                    | "double grouped set"
                    | "grouped alpha"
                    | "grouped layout"
                    | "grouped left"
                    | "grouped enumeration"
            )
        {
            use mizar_checker::source_set_term::SourceSetTermProducer;
            use mizar_resolve::{
                labels::{LabelResolver, ProofLabelSourceCollector},
                resolved_ast::SurfaceResolvedArena,
            };
            let source_arena = SurfaceResolvedArena::lower(&ast, module).unwrap();
            let owner = symbols
                .env
                .symbols()
                .iter()
                .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
                .unwrap();
            let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
            let labels = ProofLabelSourceCollector::new(
                &ast,
                module,
                namespace.clone(),
                owner.contribution(),
                &source_arena,
            )
            .unwrap()
            .collect_with_let_conditions()
            .unwrap();
            let resolved = LabelResolver::new(labels.projections()).resolve(
                module,
                &namespace,
                labels.references(),
            );
            assert!(!resolved.has_unresolved());
            assert_eq!(
                SourceSetTermProducer::check_membership_proof(
                    &installed,
                    &scope,
                    &bindings,
                    &labels,
                    &resolved,
                    &symbols.env,
                )
                .is_ok(),
                matches!(
                    label,
                    "original"
                        | "grouped set"
                        | "grouped header"
                        | "grouped proof"
                        | "double grouped set"
                        | "grouped alpha"
                        | "grouped layout"
                ),
                "{label}: wrong proof credit"
            );
        }
        assert!(
            installed
                .clone()
                .with_source_condition_formula_composition(composition.clone())
                .is_err()
        );
        if let Some(stale) = &foreign_composition {
            assert!(
                typed_with_application
                    .clone()
                    .with_source_condition_formula_composition(stale.clone())
                    .is_err(),
                "{label}: stale C2 installation"
            );
            assert!(
                typed_with_application
                    .source_condition_formula_composition()
                    .is_none(),
                "{label}: failed installation changed its input"
            );
        }

        let mut missing = input.clone();
        missing.edges.pop();
        assert!(build(missing).is_err(), "{label}: missing condition");
        let mut duplicated = input.clone();
        duplicated.edges.push(input.edges[0].clone());
        assert!(build(duplicated).is_err(), "{label}: duplicate condition");
        let mut extra = input.clone();
        extra.edges.push(SourceConditionFormulaEdgeInput {
            condition: SourceSetConditionId::new(count),
            ordinal: 0,
            formula: input.edges[0].formula,
        });
        assert!(build(extra).is_err(), "{label}: extra condition");
        let mut wrong_ordinal = input.clone();
        wrong_ordinal.edges[0].ordinal = 1;
        assert!(build(wrong_ordinal).is_err(), "{label}: wrong ordinal");
        let unrelated = atomic
            .formulas()
            .iter()
            .find(|(id, _)| *id != input.edges[0].formula)
            .map(|(id, _)| id)
            .unwrap();
        let mut nondirect = input.clone();
        nondirect.edges[0].formula = unrelated;
        assert!(build(nondirect).is_err(), "{label}: nondirect formula");
        if count > 1 {
            let mut reordered = input.clone();
            reordered.edges.swap(0, 1);
            assert!(build(reordered).is_err(), "{label}: reordered conditions");
            let mut wrong_condition = input.clone();
            wrong_condition.edges[0].condition = input.edges[1].condition;
            assert!(build(wrong_condition).is_err(), "{label}: wrong condition");
            let mut cross_owner = input.clone();
            cross_owner.edges[0].formula = input.edges[1].formula;
            assert!(build(cross_owner).is_err(), "{label}: cross-owner formula");
        }
        let mut foreign_source = input.clone();
        use mizar_session::SessionIdAllocator;
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(778)).unwrap();
        foreign_source.source_id = ids.next_source_id(snapshot_id(778)).unwrap();
        assert_ne!(foreign_source.source_id, input.source_id);
        assert!(build(foreign_source).is_err(), "{label}: foreign source");
        let mut foreign_module = input.clone();
        foreign_module.module_id = mizar_resolve::resolved_ast::ModuleId::new(
            module.package().clone(),
            mizar_session::ModulePath::new("foreign.condition-composition"),
        );
        assert!(build(foreign_module).is_err(), "{label}: foreign module");
        if let Some((other_primary, other_arena)) = &foreign_lower {
            assert_ne!(other_arena, typed.nodes(), "{label}: foreign arena setup");
            assert!(
                SourceConditionFormulaCompositionProducer::build(
                    input.clone(),
                    other_primary,
                    &application,
                    sets,
                    atomic,
                    typed.nodes(),
                )
                .is_err(),
                "{label}: foreign primary"
            );
            assert!(
                SourceConditionFormulaCompositionProducer::build(
                    input.clone(),
                    primary,
                    &application,
                    sets,
                    atomic,
                    other_arena,
                )
                .is_err(),
                "{label}: foreign arena"
            );
        }
        assert_eq!(
            build(input).unwrap(),
            composition,
            "{label}: pristine replay"
        );
        if foreign_lower.is_none() {
            foreign_lower = Some((primary.clone(), typed.nodes().clone()));
        }
        foreign_composition = Some(composition);
    }
    for (label, source) in [
        ("grouped Primary", grouped_primary),
        ("composite inequality", composite_inequality),
        (
            "condition Comprehension operand",
            original.replace(": z in A", ": (z) in { A where w is object }"),
        ),
    ] {
        let ast = parse(&source);
        let symbols = resolver_symbol_collection(root, case, &ast);
        assert!(symbols.detail_keys.is_empty(), "{label}");
        let scope = SourceVariableScopeResolver::resolve_occurrences(
            SourceVariableScopeInput::new(&ast, &symbols.module, &symbols.env),
        )
        .unwrap();
        let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        if label == "composite inequality" {
            let positions = source.match_indices("z <> z & z in A").collect::<Vec<_>>();
            assert_eq!(positions.len(), 2);
            for (start, spelling) in positions {
                let wrapper = ast
                    .nodes()
                    .iter()
                    .find(|node| {
                        matches!(node.kind, SurfaceNodeKind::FormulaExpression)
                            && node.range.start == start
                            && node.range.end == start + spelling.len()
                    })
                    .expect("actual unsupported composite condition wrapper");
                assert!(wrapper.children.iter().any(|child| matches!(
                    ast.node(*child).unwrap().kind,
                    SurfaceNodeKind::BinaryFormula(_)
                )));
            }
        }
        let lower = super::super::super::type_elaboration::step5c7_membership_typed_ast(
            &ast,
            &symbols.module,
            &symbols.env,
            &scope,
            &bindings,
        );
        assert!(lower.is_err(), "{label}: unsupported lower input admitted");
    }
}

#[test]
fn step5c7_proof_admission_is_exact() {
    let config = config();
    let root = &config.workspace_root;
    let mut plan = crate::harness::build_test_plan(&config).unwrap();
    let diagnostics = validate_active_proof_verification_tags(root, &plan);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let original = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
        .unwrap()
        .clone();
    for mutate in [
        |case: &mut TestCase| case.expectation.tags.clear(),
        |case: &mut TestCase| case.expectation.tags.push("extra".into()),
        |case: &mut TestCase| case.expectation.expected_phase = Some(PipelinePhase::TypeCheck),
        |case: &mut TestCase| case.expectation.expected_outcome = ExpectedOutcome::Fail,
        |case: &mut TestCase| case.expectation.stable_detail_key = Some("forged".into()),
        |case: &mut TestCase| case.expectation.source = "wrong.miz".into(),
        |case: &mut TestCase| case.id.0 = "unlisted".into(),
        |case: &mut TestCase| case.expectation.id.0 = "forged".into(),
        |case: &mut TestCase| case.expectation_path.set_file_name("wrong.expect.toml"),
    ] {
        let mut altered = original.clone();
        mutate(&mut altered);
        assert!(!step5c7_proof_workspace_member(root, &altered));
        assert_eq!(
            run_proof_verification_case(root, &root.join("tests"), &altered, 0).status,
            ProofVerificationCaseStatus::Failed
        );
    }
    let mut wrong_root = original.clone();
    wrong_root.source_path = root.join("alias").join(STEP5C7_PROOF_CASES[0].1);
    assert!(!step5c7_proof_workspace_member(root, &wrong_root));
    plan.cases.push(original);
    assert!(!validate_active_proof_verification_tags(root, &plan).is_empty());
    plan.cases
        .retain(|case| case.id.0 != STEP5C7_PROOF_CASES[1].0);
    assert!(
        validate_active_proof_verification_tags(root, &plan)
            .iter()
            .any(|diagnostic| diagnostic.detail_key.ends_with(STEP5C7_PROOF_CASES[1].0))
    );
}

#[test]
fn step5c7_qua_checks_both_source_scopes_and_rejects_unrelated_errors() {
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_term_qua_invalid_narrowing_001")
        .unwrap();
    let source = fs::read_to_string(&case.source_path).unwrap();
    let keys = |source: &str| {
        let ast = parse(source);
        let symbols = resolver_symbol_collection(root, case, &ast);
        if !symbols.detail_keys.is_empty() {
            return symbols.detail_keys;
        }
        super::super::super::type_elaboration::step5c7_term_detail_keys(
            &ast,
            &symbols.module,
            &symbols.env,
        )
    };
    let invalid = vec!["terms.qua.invalid_narrowing".to_owned()];
    assert_eq!(keys(&source), invalid);
    let legal = source.replace("qua QuaBox", "qua object");
    assert!(keys(&legal).is_empty());
    assert!(keys(&source.replace("qua QuaBox", "qua set")).is_empty());
    assert_eq!(
        keys(&source.replacen("qua QuaBox", "qua object", 1)),
        invalid
    );
    assert_eq!(
        keys(&legal.replacen("qua object", "qua QuaBox", 1)),
        invalid
    );
    let renamed = source
        .replace("QuaBox", "Crate")
        .replace("field d", "field payload")
        .replace("X", "Y");
    assert_eq!(keys(&renamed), invalid);
    assert!(keys(&renamed.replace("qua Crate", "qua object")).is_empty());
    let (_, theorem) = source.split_once("theorem").unwrap();
    let definition = source.split_once("theorem").unwrap().0;
    for changed in [
        source.replace("qua QuaBox", "qua Unknown"),
        source.replacen("X qua", "Z qua", 1),
        source.replace("being set", "being object"),
        source.replace("be set", "be object"),
        source.replace("field d -> set;", "field d -> object;"),
        source.replace("field d -> set;", "field d -> set; field d -> set;"),
        source.replace("field d -> set;", "field d -> ;"),
        source.replace("struct QuaBox where", "struct QuaBox -> Missing where"),
        source.replace("qua QuaBox", "qua empty QuaBox"),
        source.replace(
            "struct QuaBox where\n    field d -> set;\n  end;",
            "mode QuaBox is set;",
        ),
        format!("theorem{theorem}{definition}"),
        source.replace("thus (X qua QuaBox)", "thus (Z qua object)"),
    ] {
        assert_ne!(keys(&changed), invalid, "{changed}");
    }
    let ast = parse(&source);
    let symbols = resolver_symbol_collection(root, case, &ast);
    let mut pending = vec![ast.root().unwrap()];
    let mut reachable = std::collections::BTreeSet::new();
    while let Some(id) = pending.pop() {
        assert!(
            reachable.insert(id),
            "duplicate structural parent for {id:?}"
        );
        let node = ast.node(id).unwrap();
        pending.extend(node.children.iter().copied().filter(|child| {
            node.kind != mizar_syntax::SurfaceNodeKind::Root
                || !matches!(
                    ast.node(*child).unwrap().kind,
                    mizar_syntax::SurfaceNodeKind::Token(_)
                )
        }));
    }
    assert_eq!(reachable.len(), ast.nodes().len(), "orphan source nodes");
    let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_occurrences(
        mizar_resolve::names::SourceVariableScopeInput::new(&ast, &symbols.module, &symbols.env),
    )
    .unwrap();
    let bindings = scope
        .references()
        .iter()
        .map(|reference| reference.binding())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(bindings.len(), 2);
    let qua_sites = ast
        .nodes()
        .iter()
        .filter(|node| node.kind == mizar_syntax::SurfaceNodeKind::QuaExpression)
        .map(|node| node.range)
        .collect::<Vec<_>>();
    assert_eq!(qua_sites.len(), 2);
    assert_ne!(qua_sites[0], qua_sites[1]);
    let renamed_ast = parse(&renamed);
    let stale = super::super::super::type_elaboration::step5c7_term_detail_keys(
        &renamed_ast,
        &symbols.module,
        &symbols.env,
    );
    assert_ne!(stale, invalid);
    let mut foreign_ast = ast.clone();
    use mizar_session::SessionIdAllocator;
    let ids = mizar_session::InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(777)).unwrap();
    foreign_ast.source_id = ids.next_source_id(snapshot_id(777)).unwrap();
    assert_ne!(
        super::super::super::type_elaboration::step5c7_term_detail_keys(
            &foreign_ast,
            &symbols.module,
            &symbols.env
        ),
        invalid
    );
}

#[test]
fn step5c7_term_evidence_is_source_derived() {
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
        .unwrap();
    let keys = |source: &str| {
        let ast = parse(source);
        let symbols = resolver_symbol_collection(root, case, &ast);
        super::super::super::type_elaboration::step5c7_term_detail_keys(
            &ast,
            &symbols.module,
            &symbols.env,
        )
    };
    let qua = fs::read_to_string(
        root.join("tests/miz/pass/terms/pass_type_elaboration_term_qua_widening_001.miz"),
    )
    .unwrap();
    assert!(keys(&qua).is_empty());
    assert!(
        !keys(
            &qua.replace("being set", "being object")
                .replace("be set", "be object")
                .replace("qua object", "qua set")
        )
        .is_empty()
    );
    let wrong_base = "theorem T: for x being object for y being set holds (x qua set) = y proof let x be object; let y be set; thus (x qua set) = y; end;";
    assert!(!keys(wrong_base).is_empty());
    let choice = fs::read_to_string(
        root.join("tests/miz/fail/terms/fail_type_elaboration_term_choice_uninhabited_001.miz"),
    )
    .unwrap();
    let mixed = choice.replacen(
        "the emptyish set = the emptyish set",
        "the set = the emptyish set",
        1,
    );
    assert_eq!(keys(&mixed), ["terms.choice.missing_inhabitation"]);
    assert_eq!(
        keys(&choice.replace("emptyish", "barren")),
        ["terms.choice.missing_inhabitation"]
    );
    assert!(!keys("theorem T: 1 = 1 proof thus 1 = 1; end;").is_empty());
    let mut ast = parse(&fs::read_to_string(&case.source_path).unwrap());
    let symbols = resolver_symbol_collection(root, case, &ast);
    use mizar_session::SessionIdAllocator;
    let ids = mizar_session::InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(777)).unwrap();
    ast.source_id = ids.next_source_id(snapshot_id(777)).unwrap();
    assert!(normalize_term_ast(&ast, &symbols.module, &symbols.env).is_err());
}
