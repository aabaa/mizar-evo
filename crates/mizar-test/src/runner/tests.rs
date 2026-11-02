include!("tests/support.rs");

include!("tests/declaration_symbol.rs");

include!("tests/proof_verification.rs");

include!("tests/parse_only.rs");

include!("tests/syntax_smoke.rs");

include!("tests/type_elaboration/source_extraction.rs");

include!("tests/type_elaboration/reserved_binary.rs");

include!("tests/type_elaboration/mode_chain.rs");

include!("tests/type_elaboration/reserved_direct.rs");

include!("tests/type_elaboration/asserted_head_base.rs");

include!("tests/type_elaboration/asserted_head_four_edge_radix.rs");

include!("tests/type_elaboration/asserted_head_three_edge_object_radix.rs");

include!("tests/type_elaboration/asserted_head_two_edge_object_radix.rs");

include!("tests/type_elaboration/asserted_head_type_assertion.rs");

include!("tests/type_elaboration/binary_route_fixtures.rs");

include!("tests/type_elaboration/reserve_object_fixtures.rs");

include!("tests/type_elaboration/formula_constant_fixture.rs");

include!("tests/type_elaboration/reserve_fixtures.rs");

include!("tests/type_elaboration/mode_chain_fixtures.rs");

include!("tests/type_elaboration/asserted_head_fixtures.rs");

include!("tests/type_elaboration/source_gap_and_equality.rs");
include!("tests/type_elaboration/source_context.rs");
include!("tests/type_elaboration/source_type.rs");
include!("tests/type_elaboration/source_attribute.rs");
include!("tests/type_elaboration/source_evidence.rs");
include!("tests/type_elaboration/source_term.rs");
include!("tests/type_elaboration/source_application.rs");
include!("tests/type_elaboration/source_set_term.rs");
include!("tests/type_elaboration/source_structure.rs");
include!("tests/type_elaboration/source_atomic_formula.rs");
include!("tests/type_elaboration/source_composite_formula.rs");
include!("tests/type_elaboration/source_formula_composition.rs");
include!("tests/type_elaboration/source_statement.rs");

include!("tests/type_elaboration/source_statement/misc.rs");

include!("tests/type_elaboration/source_statement/task258a.rs");

include!("tests/type_elaboration/source_statement/task258b1.rs");

include!("tests/type_elaboration/source_statement/task258b2.rs");

include!("tests/type_elaboration/source_statement/task258b3.rs");

include!("tests/type_elaboration/source_statement/task258b3m1.rs");

include!("tests/type_elaboration/source_statement/task258b3m2a.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b1.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2a.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2b1a.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2b1b1.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2b2a.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2b2b.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2b2c.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2b3a.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2b3c.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2b3d.rs");

include!("tests/type_elaboration/source_statement/task258b3m2b2b3e.rs");

include!("tests/type_elaboration/source_statement/task258b3n.rs");

include!("tests/type_elaboration/source_statement/task258b4a.rs");

include!("tests/type_elaboration/source_statement/task258b4b.rs");

include!("tests/type_elaboration/source_statement/task258b4c.rs");

include!("tests/type_elaboration/source_statement/task258b5a.rs");

include!("tests/type_elaboration/source_statement/task258b5b.rs");
include!("tests/type_elaboration/source_attribute_definition.rs");
include!("tests/type_elaboration/source_functor_definition.rs");
include!("tests/type_elaboration/source_mode_definition.rs");
include!("tests/type_elaboration/source_structure_definition.rs");
include!("tests/type_elaboration/source_template.rs");
include!("tests/type_elaboration/template_parameter_identity.rs");
include!("tests/type_elaboration/fraenkel_generator_variable_identity.rs");
include!("tests/type_elaboration/fraenkel_nested_capture_identity.rs");
include!("tests/type_elaboration/template_type_parameter_association.rs");
include!("tests/type_elaboration/template_fraenkel_structural_composition.rs");
include!("tests/type_elaboration/template_fraenkel_generator_binding_context.rs");
include!("tests/type_elaboration/template_fraenkel_generator_bound_use.rs");
include!("tests/type_elaboration/source_predicate_definition.rs");
include!("tests/type_elaboration/source_proof_local_declaration.rs");
include!("tests/type_elaboration/source_property_implementation.rs");
include!("tests/type_elaboration/long_chain/active.rs");

include!("tests/type_elaboration/long_chain/source_local_mode.rs");

include!("tests/type_elaboration/long_chain/source_local_object.rs");
include!("tests/type_elaboration/remaining_bridges_and_nested_isolation.rs");

fn step5c11_config() -> DiscoveryConfig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    DiscoveryConfig {
        workspace_root: root,
        tests_root: PathBuf::from("tests"),
        manifest_path: PathBuf::from("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    }
}

// Only immutable repository fixtures share a plan; mutable fixture plans stay local.
fn step5c11_plan() -> &'static crate::harness::TestPlan {
    static PLAN: std::sync::OnceLock<crate::harness::TestPlan> = std::sync::OnceLock::new();
    PLAN.get_or_init(|| build_test_plan(&step5c11_config()).unwrap())
}

#[test]
fn generic_source_transport_matches_active_frontends_and_layout_variants() {
    use mizar_checker::typed_ast::{
        NodeRecoveryState, TypedArena, TypedAst, TypedAstParts, TypedNode, TypedNodeId,
    };
    use mizar_resolve::resolved_ast::{ModuleId, SurfaceResolvedArena};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind};

    let mut config = step5c11_config();
    config.profile = TestProfile::Full;
    let root = &config.workspace_root;
    let plan = build_test_plan(&config).unwrap();
    assert_eq!(plan.error_count(), 0, "{:?}", plan.diagnostics);
    let cases = super::active_parse_only_cases(&plan)
        .chain(super::active_declaration_symbol_cases(&plan))
        .chain(super::active_type_elaboration_cases(&plan))
        .chain(super::active_formula_statement_cases(root, &plan))
        .chain(super::active_proof_verification_cases(&plan))
        .chain(plan.cases.iter().filter(|case| {
            super::step5c11_registration_admitted(root, case)
                || super::step5c13_overload_admitted(root, case)
        }));
    let mut count = 0;
    let mut recovered_cases = 0;
    for (ordinal, case) in cases.enumerate() {
        let original = run_frontend(root, case, ordinal).unwrap();
        let ast = original.ast.as_ref();
        recovered_cases +=
            usize::from(ast.is_some_and(|ast| ast.nodes().iter().any(|node| node.recovered)));
        let check = |frontend: &super::shared::FrontendRun, label: &str| {
            let Some(ast) = frontend.ast.as_ref() else {
                assert!(
                    !frontend.diagnostics.is_empty(),
                    "{} {label} rejected frontend",
                    case.id.0
                );
                assert_eq!(
                    super::shared::assertion_diagnostic_codes(case, &frontend.diagnostics),
                    case.expectation.diagnostic_codes
                );
                return;
            };
            let module = ModuleId::new(
                PackageId::new("source-transport"),
                ModulePath::new("corpus"),
            );
            let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
            let recovery = |recovered| {
                if recovered {
                    NodeRecoveryState::Recovered
                } else {
                    NodeRecoveryState::Normal
                }
            };
            let nodes = TypedArena::try_new(
                Some(TypedNodeId::new(ast.root().unwrap().index())),
                source
                    .arena()
                    .iter()
                    .map(|(id, node)| {
                        TypedNode::new("syntax", node.origin().anchor().clone())
                            .with_resolved_node(id)
                            .with_children(
                                node.children()
                                    .iter()
                                    .map(|child| TypedNodeId::new(child.index()))
                                    .collect(),
                            )
                            .with_recovery(recovery(ast.nodes()[id.index()].recovered))
                    })
                    .collect(),
            )
            .unwrap();
            let mut typed = TypedAst::try_new(TypedAstParts {
                source_id: ast.source_id,
                module_id: module,
                resolved_root: Some(source.arena().root()),
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes,
                contexts: Default::default(),
                types: Default::default(),
                facts: Default::default(),
                coercions: Default::default(),
                initial_obligations: Default::default(),
                diagnostics: Default::default(),
            })
            .unwrap();
            assert!(typed.source_node_entries().is_none());
            typed
                .install_source_nodes(&source)
                .unwrap_or_else(|error| panic!("{} {label}: {error:?}", case.id.0));
            let entries = typed.source_node_entries().unwrap();
            let root_id = ast.root().unwrap();
            let root_range = ast.node(root_id).unwrap().range;
            let mut reachable = std::collections::BTreeSet::new();
            let mut pending = vec![root_id];
            while let Some(id) = pending.pop() {
                if reachable.insert(id) {
                    pending.extend(ast.node(id).unwrap().children.iter().copied());
                }
            }
            assert_eq!(
                entries.len(),
                reachable.len(),
                "{} {label} rooted census",
                case.id.0
            );
            let mut tokens = ast.token_views().collect::<Vec<_>>();
            tokens.sort_by_key(|view| (view.range().start, view.range().end));
            assert!(
                tokens.iter().all(|token| reachable.contains(&token.id())),
                "every stored frontend token is rooted"
            );
            let recovery_points = reachable
                .iter()
                .filter(|id| {
                    let range = ast.node(**id).unwrap().range;
                    if range.start != range.end {
                        return false;
                    }
                    let mut descendants = vec![**id];
                    while let Some(id) = descendants.pop() {
                        let node = ast.node(id).unwrap();
                        if node.range != range
                            || if matches!(node.kind, K::ErrorRecovery(_)) {
                                !node.recovered
                            } else {
                                node.children.is_empty()
                            }
                        {
                            return false;
                        }
                        descendants.extend(node.children.iter().copied());
                    }
                    true
                })
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            let mut parents = vec![None; ast.nodes().len()];
            for view in ast
                .node_views()
                .filter(|view| reachable.contains(&view.id()) && view.id() != root_id)
            {
                let mut end = view.range().start;
                for (ordinal, child) in view.children().iter().enumerate() {
                    assert!(
                        parents[child.index()]
                            .replace((view.id(), ordinal))
                            .is_none()
                    );
                    let range = ast.node(*child).unwrap().range;
                    let insertion_point = tokens
                        .iter()
                        .find(|token| token.range().start >= view.range().end)
                        .map_or_else(
                            || {
                                tokens
                                    .last()
                                    .map_or(root_range.start, |token| token.range().end)
                            },
                            |token| token.range().start,
                        );
                    let contained =
                        view.range().start <= range.start && range.end <= view.range().end;
                    let insertion = recovery_points.contains(child)
                        && range.start == insertion_point
                        && root_range.start <= range.start
                        && range.end <= root_range.end;
                    assert!(
                        range.start >= end && (contained || insertion),
                        "{} {label} ordered siblings and contained/recovery range",
                        case.id.0
                    );
                    end = range.end;
                }
            }
            for (ordinal, child) in ast.node(root_id).unwrap().children.iter().enumerate() {
                let range = ast.node(*child).unwrap().range;
                assert!(root_range.start <= range.start && range.end <= root_range.end);
                if parents[child.index()].is_none() {
                    parents[child.index()] = Some((root_id, ordinal));
                } else {
                    assert!(matches!(
                        ast.node(*child).unwrap().kind,
                        K::Token(_) | K::ErrorRecovery(_)
                    ));
                }
            }
            assert!(parents[root_id.index()].is_none());
            // Lexicographic child paths give preorder independently of the producer's walk.
            let mut expected = ast
                .node_views()
                .filter(|view| reachable.contains(&view.id()))
                .map(|view| {
                    let mut path = Vec::new();
                    let mut cursor = view.id();
                    while cursor != root_id {
                        let (parent, ordinal) = parents[cursor.index()].expect("actual node owner");
                        path.push(ordinal);
                        cursor = parent;
                        assert!(path.len() < ast.nodes().len(), "cyclic frontend ownership");
                    }
                    path.reverse();
                    (path, view)
                })
                .collect::<Vec<_>>();
            expected.sort_by(|left, right| left.0.cmp(&right.0));
            for token in &tokens {
                let range = token.range();
                assert_eq!(range.source_id, ast.source_id);
                assert_eq!(
                    frontend.source_text.get(range.start..range.end),
                    Some(token.as_token().unwrap().text.as_ref())
                );
            }
            assert!(
                tokens
                    .windows(2)
                    .all(|pair| pair[0].range().end <= pair[1].range().start)
            );
            let mut token_count = 0;
            for (preorder, (entry, (_, view))) in entries.zip(expected).enumerate() {
                let parent = parents[view.id().index()];
                let spelling = tokens
                    .iter()
                    .filter(|token| {
                        token.range().start >= view.range().start
                            && token.range().end <= view.range().end
                    })
                    .map(|token| token.as_token().unwrap().text.as_ref())
                    .collect::<Vec<_>>()
                    .join(" ");
                assert_eq!(
                    entry,
                    (
                        source.resolved_node_for(view.id()).unwrap(),
                        view.kind(),
                        view.range(),
                        parent.map(|(id, _)| source.resolved_node_for(id).unwrap()),
                        parent.map_or(0, |(_, ordinal)| ordinal),
                        preorder,
                        recovery(view.is_recovered()),
                        spelling.as_str(),
                    ),
                    "{} {label} node {:?}",
                    case.id.0,
                    view.id()
                );
                token_count += usize::from(matches!(entry.1, K::Token(_)));
            }
            assert_eq!(token_count, tokens.len(), "root token aliases count once");
        };
        check(&original, "original");
        let mut tokens = ast
            .into_iter()
            .flat_map(|ast| ast.token_views())
            .collect::<Vec<_>>();
        tokens.sort_by_key(|view| (view.range().start, view.range().end));
        let gap = tokens.windows(2).find_map(|pair| {
            let left = pair[0];
            let right = pair[1];
            if left.as_token().unwrap().kind == SurfaceTokenKind::AnnotationMarker {
                return None;
            }
            let gap = original
                .source_text
                .get(left.range().end..right.range().start)?;
            (!gap.is_empty()
                && gap
                    .bytes()
                    .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n')))
            .then_some(left.range().end)
        });
        for (label, start, replacement) in [
            ("leading_line", 0, "\n"),
            gap.map_or(("leading_lines", 0, "\n\n"), |start| {
                ("intertoken_comment", start, " :: transport\n")
            }),
        ] {
            let edits = [metamorphic_edits::Edit {
                start,
                end: start,
                replacement: replacement.to_owned(),
            }];
            let text = metamorphic_edits::apply(&original.source_text, &edits);
            let variant = metamorphic_guard::with_source(case, &text, || {
                run_frontend(root, case, ordinal).unwrap()
            });
            match (ast, variant.ast.as_ref()) {
                (Some(ast), Some(variant_ast)) => {
                    assert_eq!(
                        ast.nodes().len(),
                        variant_ast.nodes().len(),
                        "{} {label}",
                        case.id.0
                    );
                    for (original, changed) in ast.nodes().iter().zip(variant_ast.nodes()) {
                        assert_eq!(
                            (&original.kind, &original.children, original.recovered),
                            (&changed.kind, &changed.children, changed.recovered),
                            "{} {label} topology/recovery/token text",
                            case.id.0
                        );
                    }
                }
                (None, None) => {
                    assert_eq!(original.diagnostics.len(), variant.diagnostics.len());
                    for (before, after) in original.diagnostics.iter().zip(&variant.diagnostics) {
                        assert_eq!((&before.code, before.class), (&after.code, after.class));
                        let mizar_frontend::orchestration::DiagnosticLocation::SourceRange(range) =
                            before.location
                        else {
                            panic!("loaded corpus diagnostic must have a source range");
                        };
                        assert_eq!(
                            after.location,
                            mizar_frontend::orchestration::DiagnosticLocation::SourceRange(
                                metamorphic_edits::mapped_range(range, &edits)
                            )
                        );
                        assert_eq!(
                            after.secondary,
                            before
                                .secondary
                                .iter()
                                .map(|anchor| match anchor {
                                    mizar_session::SourceAnchor::Range(range) =>
                                        mizar_session::SourceAnchor::Range(
                                            metamorphic_edits::mapped_range(*range, &edits)
                                        ),
                                    _ => panic!("loaded corpus secondary must have a source range"),
                                })
                                .collect::<Vec<_>>()
                        );
                    }
                }
                _ => panic!("{} {label} changed frontend AST availability", case.id.0),
            }
            check(&variant, label);
        }
        count += 1;
    }
    assert!(
        count > 0 && recovered_cases > 0,
        "corpus includes parser recovery"
    );
}
fn step5c11_source_case(family: &str) -> (crate::harness::TestCase, String) {
    let plan = step5c11_plan();
    let id = format!("pass_advanced_semantics_{family}_registration_001");
    let case = plan.cases.iter().find(|case| case.id.0 == id).cloned().unwrap();
    let source = std::fs::read_to_string(&case.source_path).unwrap();
    (case, source)
}

fn step5c11_check_source(
    case: &crate::harness::TestCase,
    source: &str,
) -> Result<
    (
        mizar_checker::registration_resolution::RegistrationDatabase,
        mizar_checker::type_checker::TermFormulaInferenceOutput,
    ),
    String,
> {
    super::source_registration_intake(
        &step5c11_config().workspace_root,
        case,
        super::formula_statement::step5c8_test_frontend(source),
    )
}

mod metamorphic_edits {
    use std::collections::HashSet;

    use mizar_session::SourceRange;
    use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind, SurfaceTokenKind};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(super) struct Edit {
        pub(super) start: usize,
        pub(super) end: usize,
        pub(super) replacement: String,
    }

    pub(super) fn variants(ast: &SurfaceAst, text: &str) -> Vec<(&'static str, Vec<Edit>)> {
        let tokens = verified_tokens(ast, text);
        let mut out = vec![(
            "leading_line",
            vec![Edit {
                start: 0,
                end: 0,
                replacement: "\n".to_owned(),
            }],
        )];
        let gap = tokens.windows(2).find_map(|pair| {
            let (left, left_kind) = pair[0];
            let (right, _) = pair[1];
            if matches!(left_kind, SurfaceTokenKind::AnnotationMarker) || left.end >= right.start {
                return None;
            }
            let gap = text.get(left.end..right.start)?;
            (!gap.is_empty()
                && gap
                    .bytes()
                    .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n')))
            .then_some(left.end)
        });
        let gap = gap.unwrap_or_else(|| {
            panic!("metamorphic generator error: no verified intertoken whitespace gap")
        });
        out.push((
            "intertoken_layout",
            vec![Edit {
                start: gap,
                end: gap,
                replacement: " :: metamorphic\n".to_owned(),
            }],
        ));
        if let Some(edits) = alpha_rename(ast, text) {
            out.push(("rename", edits));
        }
        out
    }

    fn verified_tokens(ast: &SurfaceAst, text: &str) -> Vec<(SourceRange, SurfaceTokenKind)> {
        let mut tokens = ast
            .token_views()
            .filter_map(|view| {
                let token = view.as_token()?;
                let range = view.range();
                assert_eq!(
                    range.source_id, ast.source_id,
                    "token source identity drift"
                );
                assert!(range.start <= range.end, "reversed AST token range");
                assert_eq!(
                    text.get(range.start..range.end),
                    Some(token.text.as_ref()),
                    "AST token does not match source bytes"
                );
                Some((range, token.kind))
            })
            .collect::<Vec<_>>();
        tokens.sort_by_key(|(range, _)| (range.start, range.end));
        assert!(
            tokens
                .windows(2)
                .all(|pair| pair[0].0.end <= pair[1].0.start),
            "overlapping AST token ranges"
        );
        tokens
    }

    fn descendants(ast: &SurfaceAst, root: SurfaceNodeId) -> Vec<SurfaceNodeId> {
        let mut found = Vec::new();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            found.push(id);
            if let Some(node) = ast.node(id) {
                pending.extend(node.children.iter().rev().copied());
            }
        }
        found
    }

    fn alpha_rename(ast: &SurfaceAst, text: &str) -> Option<Vec<Edit>> {
        if ast.node_views().any(|view| {
            matches!(
                view.kind(),
                SurfaceNodeKind::ImportItem | SurfaceNodeKind::ModuleBranchImport
            )
        }) {
            return None;
        }
        let mut parents = vec![None; ast.nodes().len()];
        let mut ambiguous_parent = vec![false; parents.len()];
        for (parent_index, node) in ast.nodes().iter().enumerate() {
            if matches!(node.kind, SurfaceNodeKind::Root) {
                continue;
            }
            for child in &node.children {
                let slot = &mut parents[child.index()];
                if slot.replace(parent_index).is_some() {
                    ambiguous_parent[child.index()] = true;
                }
            }
        }
        let spellings = ast
            .token_views()
            .filter_map(|view| view.as_token().map(|token| token.text.as_ref()))
            .collect::<HashSet<_>>();
        for quantified in ast
            .node_views()
            .filter(|view| matches!(view.kind(), SurfaceNodeKind::QuantifiedFormula(_)))
        {
            let quantified_id = quantified.id();
            let subtree = descendants(ast, quantified_id);
            if subtree
                .iter()
                .any(|id| ast.node(*id).is_some_and(|node| node.recovered))
            {
                continue;
            }
            let segments = ast
                .node(quantified_id)?
                .children
                .iter()
                .copied()
                .filter(|id| {
                    ast.node(*id).is_some_and(|node| {
                        matches!(node.kind, SurfaceNodeKind::QuantifierVariableSegment)
                    })
                })
                .collect::<Vec<_>>();
            if segments.len() != 1 {
                continue;
            }
            let segment = segments[0];
            let segment_node = ast.node(segment)?;
            let direct_tokens = segment_node
                .children
                .iter()
                .filter_map(|id| ast.node(*id).map(|node| (*id, &node.kind)))
                .filter_map(|(id, kind)| match kind {
                    SurfaceNodeKind::Token(token) => Some((id, token.kind, token.text.as_ref())),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let binders = direct_tokens
                .iter()
                .filter(|(_, kind, _)| matches!(kind, SurfaceTokenKind::Identifier))
                .collect::<Vec<_>>();
            let typed = direct_tokens
                .iter()
                .filter(|(_, _, spelling)| matches!(*spelling, "be" | "being"))
                .count()
                == 1
                && segment_node
                    .children
                    .iter()
                    .filter(|id| {
                        ast.node(**id).is_some_and(|node| {
                            matches!(node.kind, SurfaceNodeKind::TypeExpression)
                        })
                    })
                    .count()
                    == 1;
            if binders.len() != 1 || !typed {
                continue;
            }
            let binder_id = binders[0].0;
            let spelling = binders[0].2;
            if ambiguous_parent[binder_id.index()] {
                continue;
            }
            let children = &ast.node(quantified_id)?.children;
            let segment_position = children.iter().position(|id| *id == segment)?;
            let scope = children[segment_position + 1..]
                .iter()
                .flat_map(|id| descendants(ast, *id))
                .collect::<HashSet<_>>();
            let mut occurrences = vec![binder_id];
            let mut safe = true;
            for view in ast.token_views() {
                let Some(token) = view.as_token() else {
                    continue;
                };
                if token.text.as_ref() != spelling {
                    continue;
                }
                let id = view.id();
                if id == binder_id {
                    continue;
                }
                let parent = parents[id.index()].and_then(|index| ast.nodes().get(index));
                let is_bound_use = scope.contains(&id)
                    && matches!(token.kind, SurfaceTokenKind::Identifier)
                    && parent
                        .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::TermReference))
                    && !ambiguous_parent[id.index()];
                if is_bound_use {
                    occurrences.push(id);
                } else {
                    safe = false;
                    break;
                }
            }
            if !safe {
                continue;
            }
            let fresh = (0usize..)
                .map(|index| format!("metamorphic_bound_variable_{index}"))
                .find(|candidate| !spellings.contains(candidate.as_str()))?;
            let mut edits = occurrences
                .into_iter()
                .map(|id| {
                    let range = ast.node(id).expect("token node exists").range;
                    Edit {
                        start: range.start,
                        end: range.end,
                        replacement: fresh.clone(),
                    }
                })
                .collect::<Vec<_>>();
            edits.sort_by_key(|edit| edit.start);
            assert!(
                edits
                    .iter()
                    .all(|edit| text.get(edit.start..edit.end) == Some(spelling)),
                "rename edit does not cover its AST identifier"
            );
            return Some(edits);
        }
        proof_label_rename(ast, text, &parents, &ambiguous_parent, &spellings)
    }

    fn proof_label_rename(
        ast: &SurfaceAst,
        text: &str,
        parents: &[Option<usize>],
        ambiguous_parent: &[bool],
        spellings: &HashSet<&str>,
    ) -> Option<Vec<Edit>> {
        for statement in ast
            .node_views()
            .filter(|view| matches!(view.kind(), SurfaceNodeKind::CompactStatement))
        {
            if statement.is_recovered() {
                continue;
            }
            let propositions = statement
                .child_views()
                .filter(|view| matches!(view.kind(), SurfaceNodeKind::Proposition))
                .collect::<Vec<_>>();
            if propositions.len() != 1 || propositions[0].is_recovered() {
                continue;
            }
            let children = propositions[0].child_views().collect::<Vec<_>>();
            if children.len() < 2
                || children[0].is_recovered()
                || children[1].is_recovered()
                || !children[0]
                    .as_token()
                    .is_some_and(|token| token.kind == SurfaceTokenKind::Identifier)
                || !children[1].as_token().is_some_and(|token| {
                    token.kind == SurfaceTokenKind::ReservedSymbol && token.text.as_ref() == ":"
                })
            {
                continue;
            }
            let declaration = children[0].id();
            if ambiguous_parent[declaration.index()] {
                continue;
            }
            let spelling = children[0].as_token()?.text.as_ref();
            let proof_index = enclosing_proof_index(ast, declaration, parents)?;
            if !scope_path_is_safe(ast, declaration, proof_index, parents) {
                continue;
            }
            let proof_id = ast
                .node_views()
                .find(|view| view.id().index() == proof_index)?
                .id();
            let proof_nodes = descendants(ast, proof_id)
                .into_iter()
                .collect::<HashSet<_>>();
            let mut occurrences = vec![declaration];
            let mut safe = true;
            for view in ast.token_views() {
                let Some(token) = view.as_token() else {
                    continue;
                };
                if token.text.as_ref() != spelling {
                    continue;
                }
                let id = view.id();
                if id == declaration {
                    continue;
                }
                let in_scope = proof_nodes.contains(&id)
                    && scope_path_is_safe(ast, id, proof_index, parents)
                    && view.range().start > statement.range().end;
                let citation = exact_citation_identifier(ast, id, parents, ambiguous_parent);
                if in_scope && citation {
                    occurrences.push(id);
                } else {
                    safe = false;
                    break;
                }
            }
            if !safe {
                continue;
            }
            let fresh = (0usize..)
                .map(|index| format!("metamorphic_bound_variable_{index}"))
                .find(|candidate| !spellings.contains(candidate.as_str()))?;
            let mut edits = occurrences
                .into_iter()
                .map(|id| {
                    let range = ast.node(id).expect("label token node exists").range;
                    Edit {
                        start: range.start,
                        end: range.end,
                        replacement: fresh.clone(),
                    }
                })
                .collect::<Vec<_>>();
            edits.sort_by_key(|edit| edit.start);
            assert!(
                edits
                    .iter()
                    .all(|edit| text.get(edit.start..edit.end) == Some(spelling)),
                "label edit does not cover its AST identifier"
            );
            return Some(edits);
        }
        None
    }

    fn enclosing_proof_index(
        ast: &SurfaceAst,
        id: SurfaceNodeId,
        parents: &[Option<usize>],
    ) -> Option<usize> {
        let mut current = id.index();
        while let Some(parent) = parents.get(current).copied().flatten() {
            current = parent;
            if matches!(ast.nodes().get(current)?.kind, SurfaceNodeKind::ProofBlock) {
                return Some(current);
            }
        }
        None
    }

    fn scope_path_is_safe(
        ast: &SurfaceAst,
        id: SurfaceNodeId,
        proof_index: usize,
        parents: &[Option<usize>],
    ) -> bool {
        let mut current = id.index();
        while current != proof_index {
            let Some(parent) = parents.get(current).copied().flatten() else {
                return false;
            };
            current = parent;
            if matches!(
                ast.nodes()[current].kind,
                SurfaceNodeKind::NowStatement
                    | SurfaceNodeKind::HerebyStatement
                    | SurfaceNodeKind::SupposeItem
                    | SurfaceNodeKind::CaseItem
                    | SurfaceNodeKind::CaseReasoningStatement
            ) {
                return false;
            }
        }
        true
    }

    fn exact_citation_identifier(
        ast: &SurfaceAst,
        id: SurfaceNodeId,
        parents: &[Option<usize>],
        ambiguous_parent: &[bool],
    ) -> bool {
        if ambiguous_parent[id.index()]
            || !matches!(ast.nodes()[id.index()].kind, SurfaceNodeKind::Token(_))
        {
            return false;
        }
        let Some(reference_index) = parents[id.index()] else {
            return false;
        };
        let reference = &ast.nodes()[reference_index];
        if !matches!(reference.kind, SurfaceNodeKind::Reference)
            || reference.recovered
            || reference.children.as_slice() != [id]
        {
            return false;
        }
        let Some(list_index) = parents[reference_index] else {
            return false;
        };
        let list = &ast.nodes()[list_index];
        if !matches!(list.kind, SurfaceNodeKind::ReferenceList) || list.recovered {
            return false;
        }
        let Some(justification_index) = parents[list_index] else {
            return false;
        };
        let justification = &ast.nodes()[justification_index];
        if !matches!(justification.kind, SurfaceNodeKind::JustificationClause)
            || justification.recovered
        {
            return false;
        }
        let Some(first_id) = justification.children.first() else {
            return false;
        };
        ast.node(*first_id).is_some_and(|node| matches!(&node.kind, SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::ReservedWord && token.text.as_ref() == "by"))
    }

    pub(super) fn apply(text: &str, edits: &[Edit]) -> String {
        let mut edits = edits.to_vec();
        edits.sort_by_key(|edit| (edit.start, edit.end));
        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        for edit in edits {
            assert!(
                edit.start >= cursor && edit.start <= edit.end,
                "overlapping or reversed edits"
            );
            output.push_str(
                text.get(cursor..edit.start)
                    .expect("edit starts on UTF-8 boundary"),
            );
            output.push_str(&edit.replacement);
            cursor = edit.end;
        }
        output.push_str(text.get(cursor..).expect("edit ends on UTF-8 boundary"));
        output
    }

    pub(super) fn mapped_range(mut range: SourceRange, edits: &[Edit]) -> SourceRange {
        if range.start == range.end {
            let point = map_endpoint(range.start, true, edits);
            range.start = point;
            range.end = point;
        } else {
            range.start = map_endpoint(range.start, true, edits);
            range.end = map_endpoint(range.end, false, edits);
        }
        range
    }

    fn map_endpoint(point: usize, start_affinity: bool, edits: &[Edit]) -> usize {
        let mut edits = edits.to_vec();
        edits.sort_by_key(|edit| (edit.start, edit.end));
        let mut shift: isize = 0;
        for edit in edits {
            let mapped_start = (edit.start as isize + shift) as usize;
            if edit.start == edit.end {
                if point < edit.start {
                    break;
                }
                if point == edit.start {
                    return mapped_start
                        + if start_affinity {
                            edit.replacement.len()
                        } else {
                            0
                        };
                }
                shift += edit.replacement.len() as isize;
            } else {
                if point < edit.start {
                    break;
                }
                if point <= edit.end {
                    return if point == edit.end {
                        mapped_start + edit.replacement.len()
                    } else {
                        mapped_start
                    };
                }
                shift += edit.replacement.len() as isize - (edit.end - edit.start) as isize;
            }
        }
        (point as isize + shift) as usize
    }

    #[test]
    fn layouts_and_alpha_rename_preserve_unicode_and_token_boundaries() {
        let source = "@[module_note(\"π\")]\ntheorem T: for x being set holds x = x; :: λ note\n";
        let ast = super::task257c4c1_frontend_output(source, 620).ast.unwrap();
        let variants = variants(&ast, source);
        assert_eq!(variants[0].0, "leading_line");
        assert_eq!(apply(source, &variants[0].1), format!("\n{source}"));
        assert_eq!(variants[1].0, "intertoken_layout");
        let comment_edit = &variants[1].1[0];
        assert!(source.as_bytes()[comment_edit.start].is_ascii_whitespace());
        let layout = apply(source, &variants[1].1);
        assert!(layout.contains("@[module_note(\"π\")]"));
        assert!(layout.contains(":: λ note"));
        let rename = variants
            .iter()
            .find(|(name, _)| *name == "rename")
            .expect("explicit, isolated binder is renameable");
        assert_eq!(rename.1.len(), 3);
        let renamed = apply(source, &rename.1);
        assert!(renamed.contains("metamorphic_bound_variable_0"));
        assert!(renamed.contains("\"π\""));
        assert!(renamed.contains(":: λ note"));
    }

    #[test]
    fn shadowing_and_selector_names_make_alpha_rename_inapplicable() {
        for source in [
            "theorem T: for x being set holds (for x being set holds x = x) & x = x;",
            "theorem T: for x being set holds x.x = x;",
            "import parser.type_fixtures;\ntheorem T: for x being object holds x = x;",
        ] {
            let ast = super::task257c4c1_frontend_output(source, 621).ast.unwrap();
            assert!(
                variants(&ast, source)
                    .iter()
                    .all(|(name, _)| *name != "rename")
            );
        }
    }

    #[test]
    fn binderless_duplicate_and_confined_cases_keep_layouts_without_partial_renames() {
        let sources = [
            "theorem T: contradiction;",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/miz/fail/variables/fail_formula_statement_duplicate_generalization_001.miz"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/miz/fail/resolve/fail_resolve_proof_label_inner_to_outer_confinement_001.miz"
            )),
        ];
        for (index, source) in sources.into_iter().enumerate() {
            let ast = super::task257c4c1_frontend_output(source, 622 + index)
                .ast
                .unwrap();
            let variants = variants(&ast, source);
            assert_eq!(variants.len(), 2, "case {index} must retain both layouts");
            assert_eq!(variants[0].0, "leading_line");
            assert_eq!(variants[1].0, "intertoken_layout");
            assert!(!apply(source, &variants[1].1).is_empty());
        }
    }

    #[test]
    fn isolated_proof_local_label_and_citation_are_renamed_together() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/miz/fail/types/fail_type_elaboration_statement_proof_gap_001.miz"
        ));
        let ast = super::task257c4c1_frontend_output(source, 626).ast.unwrap();
        let rename = variants(&ast, source)
            .into_iter()
            .find(|(name, _)| *name == "rename")
            .expect("unique proof-local label has one citation family");
        assert_eq!(rename.1.len(), 2);
        let renamed = apply(source, &rename.1);
        assert_eq!(renamed.matches("metamorphic_bound_variable_0").count(), 2);
        assert!(renamed.contains("by metamorphic_bound_variable_0;"));
        assert!(!renamed.contains("by A;"));
    }

    #[test]
    fn byte_map_keeps_insertions_outside_neighboring_token_spans() {
        let source_id = super::source_id(9);
        let edit = Edit {
            start: 1,
            end: 1,
            replacement: "comment".to_owned(),
        };
        assert_eq!(
            mapped_range(
                SourceRange {
                    source_id,
                    start: 0,
                    end: 1
                },
                std::slice::from_ref(&edit)
            ),
            SourceRange {
                source_id,
                start: 0,
                end: 1
            }
        );
        assert_eq!(
            mapped_range(
                SourceRange {
                    source_id,
                    start: 2,
                    end: 3
                },
                std::slice::from_ref(&edit)
            ),
            SourceRange {
                source_id,
                start: 9,
                end: 10
            }
        );
        assert_eq!(
            mapped_range(
                SourceRange {
                    source_id,
                    start: 1,
                    end: 1
                },
                std::slice::from_ref(&edit)
            ),
            SourceRange {
                source_id,
                start: 8,
                end: 8
            }
        );
        let replacement = Edit {
            start: 2,
            end: 3,
            replacement: "fresh_name".to_owned(),
        };
        assert_eq!(
            mapped_range(
                SourceRange {
                    source_id,
                    start: 2,
                    end: 3
                },
                &[replacement]
            ),
            SourceRange {
                source_id,
                start: 2,
                end: 12
            }
        );
    }
}

fn step5c11_false_coherence_case() -> (crate::harness::TestCase, String) {
    let plan = step5c11_plan();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_proof_verification_functorial_false_coherence_001")
        .cloned()
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    (case, text)
}

fn step5c11_functorial_core(
    case: &crate::harness::TestCase,
    text: &str,
) -> Result<mizar_core::core_ir::CoreIr, String> {
    let (source, nodes, symbols) = super::source_registration_inputs(
        &step5c11_config().workspace_root,
        case,
        super::formula_statement::step5c8_test_frontend(text),
    )?;
    let checked = mizar_checker::registration_resolution::check_source_registration_intake(
        &source, &nodes, &symbols,
    )?;
    mizar_core::elaborator::lower_source_functorial_registration(&checked)
}

fn step5c13_case() -> crate::harness::TestCase {
    step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == super::STEP5C13_OVERLOAD_IDS[0])
        .cloned()
        .unwrap()
}

fn step5c13_inputs(
    text: &str,
) -> Result<
    (
        mizar_resolve::resolved_ast::SurfaceResolvedArena,
        mizar_checker::typed_ast::TypedArena,
        mizar_resolve::env::SymbolEnv,
    ),
    String,
> {
    super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &step5c13_case(),
        super::formula_statement::step5c8_test_frontend(text),
    )
}

fn step5c14_static_case(name: &str) -> crate::harness::TestCase {
    let id = format!("fail_type_elaboration_algorithm_{name}_001");
    step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == id)
        .cloned()
        .unwrap()
}

fn step5c14_static_core(
    case: &crate::harness::TestCase,
    text: &str,
) -> Result<mizar_core::core_ir::CoreIr, String> {
    let (source, typed, symbols) = super::source_registration_inputs(
        &step5c11_config().workspace_root,
        case,
        super::formula_statement::step5c8_test_frontend(text),
    )?;
    let checked =
        mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols)?;
    mizar_core::elaborator::lower_source_algorithms(&checked)
}

fn step5c14_return_case() -> crate::harness::TestCase {
    step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .cloned()
        .unwrap()
}

fn step5c14_return_vcs(
    core: &mizar_core::core_ir::CoreIr,
) -> Result<mizar_vc::vc_ir::VcSet, String> {
    mizar_vc::generator::generate_source_algorithm_postconditions(
        core,
        super::shared::snapshot_id(0),
        &mizar_vc::vc_ir::GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1"),
        &mizar_vc::vc_ir::VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1"),
    )
}

fn step5c14_state_case() -> crate::harness::TestCase {
    step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
        .cloned()
        .unwrap()
}

fn step5c14_state_vcs(
    core: &mizar_core::core_ir::CoreIr,
) -> Result<mizar_vc::vc_ir::VcSet, String> {
    mizar_vc::generator::generate_source_algorithm_postconditions(
        core,
        super::shared::snapshot_id(0),
        &mizar_vc::vc_ir::GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1"),
        &mizar_vc::vc_ir::VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1"),
    )
}

fn step5c11_reduce_case() -> (crate::harness::TestCase, String) {
    let case = step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_proof_verification_reduce_false_reducibility_001")
        .cloned()
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    (case, text)
}

fn step5c14_claim_case() -> crate::harness::TestCase {
    step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_claim_block_theorem_001")
        .cloned()
        .unwrap()
}

fn step5c14_claim_core(
    case: &crate::harness::TestCase,
    text: &str,
) -> Result<mizar_core::core_ir::CoreIr, String> {
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_resolve::{
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
    };
    let frontend = super::formula_statement::step5c8_test_frontend(text);
    let ast = frontend.ast.clone().ok_or("missing AST")?;
    let (source, nodes, symbols) =
        super::source_registration_inputs(&step5c11_config().workspace_root, case, frontend)?;
    let algorithm =
        mizar_checker::type_checker::check_source_algorithm_types(&source, &nodes, &symbols)?;
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, source.module(), &symbols),
    )
    .map_err(|error| format!("scope: {error:?}"))?;
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::type_elaboration::step5c8_formula_typed_ast(
        &ast,
        source.module(),
        &symbols,
        &scope,
        &bindings,
        true,
    )?;
    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .ok_or("theorem missing")?;
    let namespace = mizar_resolve::env::NamespacePath::new(source.module().path().as_str());
    let labels = ProofLabelSourceCollector::new(
        &ast,
        source.module(),
        namespace.clone(),
        owner.contribution(),
        &source,
    )
    .and_then(|collector| collector.collect_with_theorem_owners(&symbols))
    .map_err(|error| error.to_string())?;
    let resolved = LabelResolver::new(labels.projections()).resolve(
        source.module(),
        &namespace,
        labels.references(),
    );
    let checked = SourceVariableSemanticsChecker::check_theorem_skeletons(
        &source,
        &typed,
        &scope,
        &symbols,
        &labels,
        &resolved,
        Some(&algorithm),
    )?;
    assert!(
        SourceVariableSemanticsChecker::check_theorem_skeletons(
            &source, &typed, &scope, &symbols, &labels, &resolved, None
        )
        .is_err()
    );
    assert!(mizar_core::elaborator::lower_source_theorem_skeletons(&checked, None).is_err());
    mizar_core::elaborator::lower_source_theorem_skeletons(&checked, Some(&algorithm))
}

fn step5c14_claim_vcs(
    core: &mizar_core::core_ir::CoreIr,
) -> Result<mizar_vc::vc_ir::VcSet, String> {
    mizar_vc::generator::generate_source_void_claim(
        core,
        super::shared::snapshot_id(0),
        &mizar_vc::vc_ir::GenerationSchemaVersion::new("mizar-vc-generation-step5c14-claim-v1"),
        &mizar_vc::vc_ir::VcSchemaVersion::new("mizar-vc-vcset-step5c14-claim-v1"),
    )
}

fn step5c14_assert_failure_case() -> crate::harness::TestCase {
    step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_proof_verification_algorithm_assert_unprovable_001")
        .cloned()
        .unwrap()
}

fn step5c14_assert_failure_vcs(
    core: &mizar_core::core_ir::CoreIr,
) -> Result<mizar_vc::vc_ir::VcSet, String> {
    mizar_vc::generator::generate_source_algorithm_postconditions(
        core,
        super::shared::snapshot_id(0),
        &mizar_vc::vc_ir::GenerationSchemaVersion::new(
            "mizar-vc-generation-step5c14-assert-failure-v1",
        ),
        &mizar_vc::vc_ir::VcSchemaVersion::new("mizar-vc-vcset-step5c14-assert-failure-v1"),
    )
}

fn step5c5_functor_property_case() -> (crate::harness::TestCase, String) {
    let case = step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_func_commutativity_property_001")
        .cloned()
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    (case, text)
}

fn step5c6_antonym_case() -> crate::harness::TestCase {
    step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_antonym_predicate_001")
        .cloned()
        .unwrap()
}

fn step5c3_registration_case() -> crate::harness::TestCase {
    step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_argument_attribute_widening_001")
        .cloned()
        .unwrap()
}

fn step5c3_registration_inputs(
    text: &str,
) -> Result<
    (
        mizar_resolve::resolved_ast::SurfaceResolvedArena,
        mizar_checker::typed_ast::TypedArena,
        mizar_resolve::env::SymbolEnv,
    ),
    String,
> {
    super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &step5c3_registration_case(),
        super::formula_statement::step5c8_test_frontend(text),
    )
}

fn step5c3_registration_vcs(
    core: &mizar_core::core_ir::CoreIr,
) -> Result<mizar_vc::vc_ir::VcSet, String> {
    mizar_vc::generator::generate_source_existential_registration(
        core,
        super::shared::snapshot_id(0),
        &mizar_vc::vc_ir::GenerationSchemaVersion::new("source-existential-registration-v1"),
        &mizar_vc::vc_ir::VcSchemaVersion::new("vc-v1"),
    )
}

// Independent encoder: structurally parse Formula v1, then name actual choices in v2.
fn step5c3_check_registration_handoff(
    handoff: &mizar_vc::kernel_evidence_handoff::VcKernelEvidenceHandoff,
    mutation: &str,
) -> Result<mizar_kernel::checker::KernelCheckResult, String> {
    use std::collections::BTreeSet;
    use mizar_kernel::{
        clause::{SymbolKind, Term},
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, KernelVcGeneratedFormulaId,
            check_kernel_evidence,
        },
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::TargetVcFingerprint,
    };
    use mizar_vc::{
        kernel_evidence_handoff::{
            KernelContextIdentitySource as VcSource, KernelEvidenceFingerprint, KernelFormulaSource,
        },
        vc_ir::VcFormulaRef,
    };
    fn bytes(value: &[u8], target: &mut Vec<u8>) {
        target.extend(u32::try_from(value.len()).unwrap().to_be_bytes());
        target.extend(value);
    }
    fn fingerprint(value: &KernelEvidenceFingerprint, target: &mut Vec<u8>) {
        target.push(value.algorithm_id);
        bytes(&value.digest, target);
    }
    let envelope = handoff.canonical_evidence();
    let mut sections: [Vec<Vec<u8>>; 7] = Default::default();
    sections[0] = envelope
        .symbol_manifest()
        .iter()
        .map(|entry| entry.payload.clone())
        .collect();
    sections[1] = envelope
        .variable_manifest()
        .iter()
        .map(|entry| entry.payload.clone())
        .collect();
    for formula in envelope.formula_evidence() {
        let (kind, context) = match formula.source() {
            KernelFormulaSource::GeneratedVcFact { vc_fact_id } => (3, *vc_fact_id),
            KernelFormulaSource::CitedPremise { local_context_id } => (2, *local_context_id),
            KernelFormulaSource::LocalHypothesis { local_context_id } => (1, *local_context_id),
            _ => panic!("unexpected source"),
        };
        let mut item = formula.formula_id().to_be_bytes().to_vec();
        item.push(kind);
        fingerprint(formula.formula_fingerprint(), &mut item);
        item.extend(formula.provenance_id().to_be_bytes());
        item.extend(context.to_be_bytes());
        item.extend(formula.formula_bytes());
        sections[2].push(item);
    }
    for row in envelope.provenance() {
        let mut item = row.provenance_id.to_be_bytes().to_vec();
        fingerprint(&row.target_vc, &mut item);
        fingerprint(&row.formula_fingerprint, &mut item);
        bytes(&row.payload, &mut item);
        sections[4].push(item);
    }
    let goal = envelope.final_goal();
    let mut item = vec![if mutation == "polarity" { 2 } else { 1 }];
    fingerprint(&goal.formula_fingerprint, &mut item);
    item.extend(goal.provenance_id.to_be_bytes());
    item.extend(&goal.formula_bytes);
    sections[5].push(item);
    if mutation == "goal-bytes" { *sections[5][0].last_mut().unwrap() ^= 1; }
    let profile = envelope.kernel_profile();
    let target = envelope.target_vc();
    let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
    let mut schema = 1u16;
    let parsed = loop {
        sections[3].clear();
        for substitution in envelope.substitutions() {
            let mut item = substitution.substitution_id.to_be_bytes().to_vec();
            item.extend(
                if schema == 2 && mutation == "substitution-source" {
                    0u32
                } else {
                    substitution.source_formula_id
                }
                .to_be_bytes(),
            );
            item.extend(substitution.provenance_id.to_be_bytes());
            bytes(&substitution.binder_context_encoding, &mut item);
            if schema == 2 && mutation == "substitution-actual" {
                let mut payload = substitution.payload[..17].to_vec();
                payload.push(1);
                payload.extend(&substitution.payload[13..17]);
                payload.push(1);
                item.extend(payload);
            } else {
                item.extend(&substitution.payload);
            }
            item.extend(
                u32::try_from(substitution.freshness_witnesses.len())
                    .unwrap()
                    .to_be_bytes(),
            );
            for row in &substitution.freshness_witnesses {
                item.extend(row);
            }
            item.extend(
                u32::try_from(substitution.free_variable_constraints.len())
                    .unwrap()
                    .to_be_bytes(),
            );
            for row in &substitution.free_variable_constraints {
                item.extend(row);
            }
            sections[3].push(item);
        }
        if schema == 2 && mutation == "missing-substitution" {
            sections[3].clear();
        }
        if schema == 2 && mutation == "missing-second-substitution" {
            assert_eq!(sections[3].len(), 2);
            sections[3].pop();
        }
        if schema == 2 && mutation == "swapped-substitution-source" {
            assert_eq!(sections[3].len(), 2);
            let first = sections[3][0][4..8].to_vec();
            let second = sections[3][1][4..8].to_vec();
            sections[3][0][4..8].copy_from_slice(&second);
            sections[3][1][4..8].copy_from_slice(&first);
        }
        let mut wire = b"MIZAR_KERNEL_EVIDENCE\0".to_vec();
        for value in [schema, 1, profile.profile_id, 1, 1] {
            wire.extend(value.to_be_bytes());
        }
        wire.extend([1, 1]);
        fingerprint(envelope.target_vc(), &mut wire);
        let section_count = if schema == 1 { 6 } else { 7 };
        wire.extend((section_count as u32).to_be_bytes());
        let mut payload = Vec::new();
        for (index, rows) in sections.iter().take(section_count).enumerate() {
            let tag = u8::try_from(index + 1).unwrap();
            let start = payload.len();
            for row in rows {
                payload.extend([tag, 1]);
                bytes(row, &mut payload);
            }
            wire.push(tag);
            for value in [rows.len(), start, payload.len() - start] {
                wire.extend(u32::try_from(value).unwrap().to_be_bytes());
            }
        }
        wire.extend(payload);
        if mutation == "wire" {
            wire[0] ^= 1;
        }
        let mut parse_context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(
                    envelope.target_vc().algorithm_id,
                    envelope.target_vc().digest.clone(),
                ),
                KernelProfileRecord::v1(profile.profile_id, ClauseTautologyPolicy::Reject),
            );
        if schema == 2 { parse_context.accepted_schema_versions.insert(2); }
        let parsed = parse_formula_evidence(&wire, &parse_context).map_err(|error| format!("{error:?}"))?;
        if schema == 2 { break parsed; }
        let equalities = parsed.symbol_manifest().iter().filter(|entry| entry.symbol.kind == SymbolKind::Equality).collect::<Vec<_>>();
        assert_eq!(equalities.len(), 1);
        let mut choices = BTreeSet::new();
        for substitution in parsed.substitutions() {
            if substitution.payload.payload_kind != 1 || !substitution.payload.rewrite_path.segments.is_empty() {
                return Err("unexpected choice substitution".into());
            }
            let [replacement] = substitution.payload.replacements.as_slice() else { return Err("unexpected replacement count".into()); };
            let Term::Application { symbol, arguments } = &replacement.actual_term else { return Err("expected closed choice".into()); };
            if symbol.kind != SymbolKind::FunctorPredicate || !arguments.is_empty() || replacement.replacement_role != 1 {
                return Err("expected closed choice".into());
            }
            choices.insert(*symbol);
        }
        for (index, choice) in choices.into_iter().enumerate() {
            let mut item = (index as u32 + 1).to_be_bytes().to_vec();
            item.push(1);
            item.extend(equalities[0].symbol.id.0.to_be_bytes());
            item.extend([2, 2]); item.extend(choice.id.0.to_be_bytes()); item.extend(0u32.to_be_bytes());
            sections[6].push(item);
        }
        if mutation == "missing-equality-instance" { sections[6].clear(); }
        schema = 2;
    };
    let entries = handoff
        .context_identity()
        .entries()
        .iter()
        .skip(usize::from(mutation == "context"))
        .map(|entry| {
            let source = match entry.source() {
                VcSource::GeneratedVcFact { vc_fact_id } => {
                    KernelContextIdentitySource::GeneratedVcFact { vc_fact_id }
                }
                VcSource::CitedPremise { local_context_id } => {
                    KernelContextIdentitySource::CitedPremise { local_context_id }
                }
                VcSource::LocalHypothesis { local_context_id } => {
                    KernelContextIdentitySource::LocalHypothesis { local_context_id }
                }
                _ => panic!("unexpected context"),
            };
            let producer = match entry.producer_formula_ref() {
                VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                VcFormulaRef::Generated(id) => {
                    KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(id.index()))
                }
                _ => panic!("unexpected formula"),
            };
            KernelContextIdentityEntry::new(
                source,
                entry.formula_id(),
                Fingerprint::new(
                    entry.formula_fingerprint().algorithm_id,
                    entry.formula_fingerprint().digest.clone(),
                ),
                producer,
            )
        })
        .collect();
    let context = FormulaEvidenceContext::with_context_identity(
        (mutation != "provenance").then(|| handoff.context_identity_hash().as_bytes().to_vec()),
        Vec::new(),
        Vec::new(),
        Some(KernelContextIdentityPayload::new(
            target.clone(),
            handoff.canonical_hash(),
            handoff.context_identity_hash(),
            entries,
        )),
        ImportedFactContextLimits::default(),
    )
    .map_err(|error| format!("{error:?}"))?;
    let mut limits = KernelEvidenceCheckLimits::default();
    if mutation == "resource" {
        limits.max_pipeline_steps = 0;
    }
    let result = check_kernel_evidence(KernelEvidenceCheckInput {
        target_vc_fingerprint: &target,
        evidence: &parsed,
        formula_context: Some(&context),
        check_kind: KernelEvidenceCheckKind::ProofObligation,
        policy: KernelCheckPolicy::default(),
        limits,
    });
    if result.status() != KernelCheckStatus::Accepted {
        return Err(format!("{result:?}"));
    }
    assert!(!result.policy_taint());
    Ok(result)
}

fn step5c13_registration_case() -> crate::harness::TestCase {
    step5c11_plan().cases.iter()
        .find(|case| case.id.0 == "fail_advanced_semantics_overload_ambiguous_candidates_001")
        .cloned()
        .unwrap()
}

fn step5c13_registration_inputs(text: &str) -> Result<(
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
    mizar_checker::typed_ast::TypedArena,
    mizar_resolve::env::SymbolEnv,
), String> {
    super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &step5c13_registration_case(),
        super::formula_statement::step5c8_test_frontend(text),
    )
}

fn step5c5_dependent_return_case() -> (crate::harness::TestCase, String) {
    let case = step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_func_dependent_return_type_001")
        .cloned()
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    (case, text)
}

fn step5c5_narrower_inputs(text: &str) -> Result<(
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
    mizar_checker::typed_ast::TypedArena,
    mizar_resolve::env::SymbolEnv,
), String> {
    let config = step5c11_config();
    let case = build_test_plan(&config).unwrap().cases.into_iter()
        .find(|case| case.id.0 == "pass_type_elaboration_pred_redefine_narrower_loci_001").unwrap();
    super::source_registration_inputs(&config.workspace_root, &case,
        super::formula_statement::step5c8_test_frontend(text))
}

fn source_candidate_core_and_vcs(multiple: bool, text: &str) -> (mizar_core::core_ir::CoreIr, mizar_vc::vc_ir::VcSet, mizar_vc::vc_ir::VcSet) {
    use mizar_vc::discharge::{DischargeInput, DischargePolicy, try_discharge};
    let (source, nodes, symbols) = if multiple { step5c13_registration_inputs(text) }
        else { step5c3_registration_inputs(text) }.unwrap();
    let checked = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
    let open = step5c3_registration_vcs(&core).unwrap();
    let discharge = try_discharge(DischargeInput { vc_set: &open, policy: &DischargePolicy::default() }).unwrap();
    (core, open, discharge.vc_set().clone())
}

fn source_candidate_profile(name: &str, format: mizar_atp::problem::ConcreteFormat) -> mizar_atp::problem::LogicProfile {
    use mizar_atp::problem::*;
    LogicProfile::try_new(name, if format == ConcreteFormat::Tptp { LogicFragment::Fof }
        else { LogicFragment::SmtLibUninterpreted }, EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates,
        NativePropertySupport::Unsupported, [format].into_iter().collect()).unwrap()
}

// Independent names oracle: use actual parsed numeric keys, never ATP names or the factory.
fn source_candidate_wire(handoff: &mizar_vc::kernel_evidence_handoff::VcKernelEvidenceHandoff) -> Vec<u8> {
    use mizar_kernel::{certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        clause::{SymbolKind, Term}, formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence}};
    let envelope = handoff.canonical_evidence();
    let target = envelope.target_vc();
    let context = FormulaEvidenceParseContext::v1(Fingerprint::new(target.algorithm_id, target.digest.clone()),
        KernelProfileRecord::v1(envelope.kernel_profile().profile_id, ClauseTautologyPolicy::Reject));
    let parsed = parse_formula_evidence(&handoff.encode_local_evidence(None).unwrap(), &context).unwrap();
    let equality = parsed.symbol_manifest().iter().find(|row| row.symbol.kind == SymbolKind::Equality).unwrap().symbol.id.0;
    let choices = parsed.substitutions().iter().map(|row| {
        let [replacement] = row.payload.replacements.as_slice() else { panic!("actual replacement") };
        let Term::Application { symbol, arguments } = &replacement.actual_term else { panic!("actual closed choice") };
        assert!(arguments.is_empty());
        symbol.id.0
    }).collect::<std::collections::BTreeSet<_>>();
    let names = choices.into_iter().enumerate().map(|(index, choice)| {
        let mut item = u32::try_from(index + 1).unwrap().to_be_bytes().to_vec();
        item.push(1); item.extend(equality.to_be_bytes()); item.extend([2, 2]);
        item.extend(choice.to_be_bytes()); item.extend(0u32.to_be_bytes()); item
    }).collect::<Vec<_>>();
    handoff.encode_local_evidence(Some(&names)).unwrap()
}

fn source_candidate_check_bytes(
    handoff: &mizar_vc::kernel_evidence_handoff::VcKernelEvidenceHandoff,
    bytes: &[u8], target: &mizar_atp::problem::AtpTargetBinding,
) -> Result<mizar_kernel::checker::KernelCheckStatus, String> {
    use mizar_kernel::{certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::*, formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence}, rejection::TargetVcFingerprint};
    use mizar_vc::{kernel_evidence_handoff::KernelContextIdentitySource as VcSource, vc_ir::VcFormulaRef};
    let fingerprint = target.fingerprint();
    let target = TargetVcFingerprint::new(fingerprint.algorithm_id(), fingerprint.digest().to_vec());
    let mut parse_context = FormulaEvidenceParseContext::v1(Fingerprint::new(target.algorithm_id, target.digest.clone()),
        KernelProfileRecord::v1(handoff.canonical_evidence().kernel_profile().profile_id, ClauseTautologyPolicy::Reject));
    parse_context.accepted_schema_versions.insert(2);
    let parsed = parse_formula_evidence(bytes, &parse_context).map_err(|error| format!("{error:?}"))?;
    let entries = handoff.context_identity().entries().iter().map(|entry| {
        let source = match entry.source() {
            VcSource::LocalHypothesis { local_context_id } => KernelContextIdentitySource::LocalHypothesis { local_context_id },
            VcSource::CitedPremise { local_context_id } => KernelContextIdentitySource::CitedPremise { local_context_id },
            VcSource::GeneratedVcFact { vc_fact_id } => KernelContextIdentitySource::GeneratedVcFact { vc_fact_id },
            _ => panic!("actual context source"),
        };
        let producer = match entry.producer_formula_ref() {
            VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
            VcFormulaRef::Generated(id) => KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(id.index())),
            _ => panic!("actual producer"),
        };
        KernelContextIdentityEntry::new(source, entry.formula_id(),
            Fingerprint::new(entry.formula_fingerprint().algorithm_id, entry.formula_fingerprint().digest.clone()), producer)
    }).collect();
    let context = FormulaEvidenceContext::with_context_identity(Some(handoff.context_identity_hash().as_bytes().to_vec()),
        Vec::new(), Vec::new(), Some(KernelContextIdentityPayload::new(target.clone(), handoff.canonical_hash(),
            handoff.context_identity_hash(), entries)), ImportedFactContextLimits::default()).unwrap();
    let result = check_kernel_evidence(KernelEvidenceCheckInput { target_vc_fingerprint: &target, evidence: &parsed,
        formula_context: Some(&context), check_kind: KernelEvidenceCheckKind::ProofObligation,
        policy: KernelCheckPolicy::default(), limits: KernelEvidenceCheckLimits::default() });
    assert!(!result.policy_taint());
    Ok(result.status())
}

pub(super) fn source_registration_owner_inputs(
    text: &str,
    package: &str,
    module: &str,
    source_skip: usize,
) -> (
    super::shared::FrontendRun,
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
    mizar_resolve::symbols::SymbolCollectionResult,
) {
    use mizar_resolve::{declarations::DeclarationShellCollector, symbols::SignatureProjectionExtractor};
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_session::{DiskSourceLoader, Edition, InMemorySessionIdAllocator, SessionIdAllocator,
        SourceInput, SourceOriginInput};
    let temporary = std::process::Command::new("mktemp").arg("-d").output().unwrap();
    assert!(temporary.status.success());
    let root = PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    std::fs::create_dir(root.join("src")).unwrap();
    let path = root
        .join("src")
        .join(module.split('.').collect::<PathBuf>())
        .with_extension("miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    let snapshot = super::shared::snapshot_id(5835);
    let ids = InMemorySessionIdAllocator::new();
    for _ in 0..source_skip {
        ids.next_source_id(snapshot).unwrap();
    }
    let output = Frontend::new(FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        super::ParseOnlyImportProvider, MizarParserSeam).run(SourceUnitRequest {
            snapshot,
            input: SourceInput {
                package_id: PackageId::new(package),
                module_path: ModulePath::new(module),
                normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2026"),
                origin: SourceOriginInput::Disk { path: path.clone() },
            },
        }, &ids).unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let frontend = super::shared::FrontendRun { source_text: output.source.source_text,
        ast: output.ast, ast_snapshot: None, diagnostics: output.diagnostics };
    let ast = frontend.ast.as_ref().unwrap();
    let module = ResolverModuleId::new(PackageId::new(package), ModulePath::new(module));
    let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(ast, &shells,
        NamespacePath::new(module.path().as_str())).collect(&module);
    (frontend, source, collected)
}

fn source_registration_formula_inputs(
    text: &str, package: &str, module: &str, skip: usize,
) -> (
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
    mizar_checker::typed_ast::TypedArena,
    mizar_resolve::symbols::SymbolCollectionResult,
    std::collections::BTreeMap<mizar_resolve::resolved_ast::SymbolId, Vec<u8>>,
) {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
    let (frontend, source, collected) = source_registration_owner_inputs(text, package, module, skip);
    assert!(frontend.diagnostics.is_empty());
    assert!(collected.diagnostics().is_empty());
    let nodes = TypedArena::try_new(
        Some(TypedNodeId::new(source.arena().root().index())),
        source.arena().iter().map(|(id, node)| {
            TypedNode::new(format!("{:?}", node.kind()), node.origin().anchor().clone())
                .with_resolved_node(id)
                .with_children(node.children().iter().map(|child| TypedNodeId::new(child.index())).collect())
        }).collect(),
    ).unwrap();
    let owners = mizar_resolve::symbols::source_registration_owner_payloads(
        frontend.ast.as_ref().unwrap(), &source, collected.env(),
    ).unwrap();
    (source, nodes, collected, owners)
}

fn source_registration_vc_fingerprint_inputs(
    text: &str, module: &str, skip: usize,
) -> (
    mizar_core::core_ir::CoreIr,
    mizar_vc::vc_ir::VcSet,
    std::collections::BTreeMap<mizar_core::core_ir::CoreFormulaId, Vec<u8>>,
    std::collections::BTreeMap<mizar_vc::vc_ir::VcId, String>,
) {
    use mizar_vc::vc_ir::{AnchorOwner, GenerationSchemaVersion, PremiseRef, SeedVcMapping,
        VcFormulaRef, VcGeneratedFormulaShape as S, VcSchemaVersion};
    let (source, nodes, collected, owners) = source_registration_formula_inputs(
        text, "vc-fingerprint-test", module, skip,
    );
    let check = mizar_checker::registration_resolution::check_source_existential_registration_proof(
        &source, &nodes, collected.env(),
    ).unwrap_or_else(|error| panic!("{error}: {text}"));
    let core = mizar_core::elaborator::lower_source_existential_registration(&check).unwrap();
    let vcs = mizar_vc::generator::generate_source_existential_registration(
        &core, super::shared::snapshot_id(0),
        &GenerationSchemaVersion::new("source-vc-fingerprint-test"),
        &VcSchemaVersion::new("source-vc-fingerprint-v1"),
    ).unwrap();
    let mut references = Vec::new();
    let mut associations = std::collections::BTreeMap::new();
    for vc in vcs.vcs() {
        references.push(vc.goal);
        for premise in &vc.premises {
            let PremiseRef::GeneratedFact { formula } = premise else { unreachable!() };
            references.push(*formula);
        }
        let AnchorOwner::Registration(item) = vc.anchor.owner else { unreachable!() };
        let symbol = &core.items().get(item).unwrap().symbol;
        let name = collected.env().symbols().iter().find(|entry| entry.symbol() == symbol)
            .unwrap().primary_spelling().to_owned();
        let SeedVcMapping::Expanded { vcs: leaves, .. } = &vcs.seed_accounting_for(vc.seed.handoff).unwrap().mapping else { unreachable!() };
        let leaf = leaves.iter().find(|leaf| leaf.vc == vc.id).unwrap().expansion_index;
        assert!(associations.insert(vc.id, format!("{name}:{leaf}")).is_none());
    }
    let mut roots = std::collections::BTreeSet::new();
    while let Some(reference) = references.pop() {
        match reference {
            VcFormulaRef::Core(id) => { roots.insert(id); }
            VcFormulaRef::Generated(id) => match &vcs.generated_formula(id).unwrap().shape {
                S::Ref(child) | S::Not(child) => references.push(*child),
                S::And(children) | S::Or(children) => references.extend(children),
                other => panic!("unexpected genuine shape {other:?}"),
            },
            other => panic!("unexpected genuine reference {other:?}"),
        }
    }
    let formulas = mizar_core::elaborator::source_registration_formula_payloads(
        &check, &core, &owners, &roots.into_iter().collect::<Vec<_>>(),
    ).unwrap();
    (core, vcs, formulas, associations)
}

// Independent grammar construction: literal Syntax schema tags, never actual AST traversal.
fn source_registration_syntax_expected(label: &str, attrs: &[&str], refs: &[&str]) -> Vec<u8> {
    let frame = |bytes: &[u8]| {
        [u32::try_from(bytes.len()).unwrap().to_be_bytes().as_slice(), bytes].concat()
    };
    let n = |tag: u16, children: Vec<Vec<u8>>| {
        [tag.to_be_bytes().as_slice(),
            u32::try_from(children.len()).unwrap().to_be_bytes().as_slice(),
            children.concat().as_slice()].concat()
    };
    let t = |kind: u16, text: &str| {
        [2u16.to_be_bytes().as_slice(), kind.to_be_bytes().as_slice(),
            frame(text.as_bytes()).as_slice(), &[0; 4]].concat()
    };
    let a = |name: &str| n(23, vec![n(7, vec![n(8, vec![t(105, name)])])]);
    let set_type = n(21, vec![n(25, vec![t(101, "set")])]);
    let choice = n(28, vec![n(33, vec![t(101, "the"), set_type])]);
    let rhs = if attrs.len() == 1 {
        n(47, vec![a(attrs[0])])
    } else {
        assert_eq!(attrs.len(), 2);
        // The parser stores the second adjective as this TypeHead in the aggregate assertion.
        n(21, vec![n(22, vec![a(attrs[0])]), n(25, vec![n(7, vec![n(8, vec![t(105, attrs[1])])])])])
    };
    let mut references = Vec::new();
    for (index, reference) in refs.iter().enumerate() {
        if index != 0 { references.push(t(102, ",")); }
        references.push(n(74, vec![t(100, reference)]));
    }
    let take = n(67, vec![t(101, "take"), n(68, vec![choice.clone()]), t(102, ";")]);
    let proposition = n(64, vec![n(44, vec![n(46, vec![choice, t(101, "is"), rhs])])]);
    let by = n(72, vec![t(101, "by"), n(73, references)]);
    let conclusion = n(84, vec![t(101, "thus"), proposition, by, t(102, ";")]);
    let proof = n(98, vec![t(101, "proof"), take, conclusion, t(101, "end")]);
    let correctness = n(115, vec![t(101, "existence"), proof, t(102, ";")]);
    let declared_type = n(21, vec![n(22, attrs.iter().map(|name| a(name)).collect()), n(25, vec![t(101, "set")])]);
    let root = n(142, vec![t(101, "cluster"), t(100, label), t(102, ":"), declared_type, t(102, ";"), correctness]);
    let owner = [b"mizar-resolve/source-registration-owner/v1\0\x02".as_slice(),
        frame(b"owner-test").as_slice(), frame(b"owners").as_slice(), frame(label.as_bytes()).as_slice()].concat();
    [b"mizar-resolve/source-registration-syntax/v1\0".as_slice(), frame(&owner).as_slice(), &root].concat()
}

// Rationale: retain the existing producer tuple plus current-item syntax map without a one-use abstraction.
#[allow(clippy::type_complexity)]
fn source_registration_source_shape_inputs(
    text: &str,
    module: &str,
    skip: usize,
) -> (
    mizar_core::core_ir::CoreIr,
    mizar_vc::vc_ir::VcSet,
    std::collections::BTreeMap<mizar_core::core_ir::CoreFormulaId, Vec<u8>>,
    std::collections::BTreeMap<mizar_vc::vc_ir::VcId, String>,
    std::collections::BTreeMap<mizar_core::core_ir::CoreItemId, Vec<u8>>,
) {
    use mizar_core::core_ir::CoreItemKind;
    use mizar_resolve::{env::SymbolKind, symbols::source_registration_syntax_payloads_from_arena};
    let (core, vcs, formulas, owners) = source_registration_vc_fingerprint_inputs(text, module, skip);
    // Repeat genuine producers from the same immutable text; this is test plumbing,
    // not the future authenticated same-call Proof assembly.
    let (frontend, source, collected) = source_registration_owner_inputs(text, "vc-fingerprint-test", module, skip);
    assert!(frontend.diagnostics.is_empty());
    assert!(collected.diagnostics().is_empty());
    assert_eq!(core.source_id(), source.source_id());
    assert_eq!(core.module_id(), source.module());
    let payloads = source_registration_syntax_payloads_from_arena(&source, collected.env()).unwrap();
    let mut syntax = std::collections::BTreeMap::new();
    for (id, item) in core.items().iter().filter(|(_, item)| item.kind == CoreItemKind::Registration) {
        let actual = collected.env().symbols().iter().find(|entry| entry.symbol() == &item.symbol).unwrap();
        assert_eq!(actual.kind(), SymbolKind::Registration);
        syntax.insert(id, payloads[actual.symbol()].clone());
    }
    assert_eq!(syntax.len(), payloads.len());
    (core, vcs, formulas, owners, syntax)
}

// Caller-owned envelope coordinates exercise structure only; acceptance and witness
// bytes below come from the original source projectors, not an enclosing producer.
fn builtin_enclosing_fixture(multiple: bool) -> (mizar_artifact::verified_artifact::VerifiedArtifact, Vec<Vec<u8>>) {
    use mizar_artifact::{module_summary::{ModuleSummaryIdentity, SourceRangeSummary},
        registration_summary::{ArtifactHashClass, ArtifactHashRef}, store::SchemaVersion,
        verified_artifact::{BuildProvenance, ObligationMetadata, ObligationStatus, VerifiedArtifact}};
    use mizar_proof::{policy::VerifierPolicy, selection::ProofWinnerClass, status::{
        project_source_registration_candidate_witness, project_source_registration_discharge_witness}};
    use mizar_vc::vc_ir::VcStatus;
    let case = if multiple { step5c13_registration_case() } else { step5c3_registration_case() };
    let text = std::fs::read_to_string(case.source_path).unwrap();
    let (source, nodes, symbols) = if multiple { step5c13_registration_inputs(&text) }
        else { step5c3_registration_inputs(&text) }.unwrap();
    let before = (source.clone(), nodes.clone(), symbols.clone());
    let (_, _, vcs) = source_candidate_core_and_vcs(multiple, &text);
    let policy = VerifierPolicy::release();
    let mut references = Vec::new();
    let mut payloads = Vec::new();
    for vc in vcs.vcs() {
        let (selection, reference, bytes) = match vc.status {
            VcStatus::Discharged { .. } => {
                let result = project_source_registration_discharge_witness(&source, &nodes, &symbols,
                    super::shared::snapshot_id(0), vc.id, &policy).unwrap();
                assert_eq!(result.0.selected_class(), ProofWinnerClass::DischargedBuiltin);
                result
            }
            VcStatus::NeedsAtp => {
                let format = mizar_atp::problem::ConcreteFormat::Tptp;
                let result = project_source_registration_candidate_witness(&source, &nodes, &symbols,
                    super::shared::snapshot_id(0), vc.id,
                    (source_candidate_profile("enclosing-source", format), format), &policy).unwrap();
                assert_eq!(result.0.selected_class(), ProofWinnerClass::KernelVerified);
                result
            }
            _ => panic!("unexpected source obligation"),
        };
        assert!(selection.selected_candidate_id().is_some());
        assert!(!bytes.is_empty());
        references.push(reference);
        payloads.push(bytes);
    }
    assert_eq!(references.len(), if multiple { 6 } else { 2 });
    assert_eq!((source, nodes, symbols), before);
    let fingerprint = references[0].obligation_fingerprint.clone();
    let obligations = references.iter().map(|w| ObligationMetadata {
        obligation_id: w.obligation_id.clone(), obligation_anchor: None, owner_origin_id: None,
        source_range: SourceRangeSummary { start_byte: 0, end_byte: text.len() as u64 },
        obligation_kind: "structural-container".into(), statement_summary: "source witness reference".into(),
        obligation_fingerprint: w.obligation_fingerprint.clone(), vc_fingerprint: fingerprint.clone(),
        local_context_fingerprint: fingerprint.clone(), dependency_slice_fingerprint: fingerprint.clone(),
        verifier_policy_fingerprint: w.kernel_acceptance.verifier_policy_fingerprint.clone(),
        status: ObligationStatus::Accepted, accepted_witness_obligation_id: Some(w.obligation_id.clone()),
        deterministic_discharge_hash: None, diagnostic_ref: None,
    }).collect();
    let mut artifact = VerifiedArtifact {
        schema_version: SchemaVersion::new(2, 0),
        module: ModuleSummaryIdentity { package_id: "enclosing-fixture".into(), package_version: None,
            lockfile_identity: None, module_path: "source".into(), language_edition: "2026".into() },
        source_file: "src/source.miz".into(), source_hash: mizar_session::hash_text(&text), verified_at: None,
        interface_hash: mizar_session::hash_text("unrefreshed interface"),
        implementation_hash: mizar_session::hash_text("unrefreshed implementation"),
        exports: Vec::new(), statements: Vec::new(), expressions: Vec::new(), obligations, proof_witnesses: references,
        diagnostics: Vec::new(), provenance: BuildProvenance { toolchain: "structural-test".into(),
            language_edition: "2026".into(), lockfile_hash: ArtifactHashRef::new(ArtifactHashClass::Artifact,
                "fixture/lock", SchemaVersion::new(1, 0), mizar_session::hash_text("lock")),
            verifier_config_hash: fingerprint, dependency_artifact_hashes: Vec::new(), cache_key: None },
    };
    artifact.refresh_hashes().unwrap();
    (artifact, payloads)
}

fn source_dependency_inputs(
    text: &str, module: &str, skip: usize, snapshot: mizar_session::BuildSnapshotId,
) -> (
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
    mizar_checker::typed_ast::TypedArena,
    mizar_resolve::symbols::SymbolCollectionResult,
    mizar_vc::vc_ir::VcSet,
) {
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let (source, nodes, collected, _) = source_registration_formula_inputs(text, "vc-fingerprint-test", module, skip);
    let check = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, collected.env()).unwrap();
    let core = mizar_core::elaborator::lower_source_existential_registration(&check).unwrap();
    let vcs = mizar_vc::generator::generate_source_existential_registration(&core, snapshot,
        &GenerationSchemaVersion::new("source-existential-registration-v1"), &VcSchemaVersion::new("vc-v1")).unwrap();
    (source, nodes, collected, vcs)
}

// These are caller-owned structural publication envelopes. Only the witness
// references/payloads come from existing genuine source projectors; envelope
// coordinates and metadata make no producer, currentness, or acceptance claim.
fn doc_published_fixture() -> (
    PathBuf,
    mizar_artifact::manifest::ArtifactManifest,
    Vec<mizar_artifact::verified_artifact::VerifiedArtifact>,
) {
    use mizar_artifact::{
        manifest::*,
        module_summary::SourceRangeSummary,
        proof_witness::ProofStatus,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{
            PublishedArtifactPath, SchemaVersion, artifact_hash_domain, write_published_artifact,
        },
        verified_artifact::*,
    };
    let temp = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temp.status.success());
    let root = PathBuf::from(String::from_utf8(temp.stdout).unwrap().trim());
    let (explicit, payloads) = builtin_enclosing_fixture(true);
    for (reference, bytes) in explicit.proof_witnesses.iter().zip(payloads) {
        let path = root.join(&reference.witness_path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    let mut normal = explicit.clone();
    normal.schema_version = SchemaVersion::new(1, 0);
    normal
        .proof_witnesses
        .retain(|w| w.proof_status == ProofStatus::KernelVerified);
    normal.obligations.retain(|o| {
        normal
            .proof_witnesses
            .iter()
            .any(|w| w.obligation_id == o.obligation_id)
    });
    let mut artifacts = vec![normal, explicit];
    let mut entries = Vec::new();
    for (index, artifact) in artifacts.iter_mut().enumerate() {
        artifact.module.module_path = format!("module_{index}");
        artifact.verified_at = Some("2026-10-16T01:02:03Z".into());
        artifact.provenance.cache_key = Some("structural-local-metadata".into());
        let range = SourceRangeSummary {
            start_byte: 0,
            end_byte: 1,
        };
        let diagnostic = ArtifactHashRef::new(
            ArtifactHashClass::Diagnostic,
            "fixture/diagnostic",
            SchemaVersion::new(1, 0),
            mizar_session::hash_text("diagnostic"),
        );
        artifact.exports = vec![VerifiedExport {
            origin_id: "structural-export".into(),
            fully_qualified_name: "fixture::export".into(),
            namespace_path: vec!["fixture".into()],
            visibility: ExportVisibility::Public,
            export_kind: "structural".into(),
            source_range: range,
            rendered_signature: "metadata".into(),
            interface_fingerprint: ArtifactHashRef::new(
                ArtifactHashClass::Interface,
                "fixture/export",
                SchemaVersion::new(1, 0),
                mizar_session::hash_text("export"),
            ),
            proof_status: Some(ExportProofStatus::NotRequired),
            documentation_ref: Some(diagnostic.clone()),
        }];
        artifact.expressions = vec![ExpressionMetadata {
            expression_id: "structural-expression".into(),
            source_range: range,
            expression_kind: "structural".into(),
            rendered_surface: "metadata".into(),
            inferred_type: Some("metadata-type".into()),
            resolved_symbol: Some("fixture::symbol".into()),
            inserted_coercions: vec!["metadata-coercion".into()],
            active_thesis: Some("metadata-thesis".into()),
            overload_resolution: Some(OverloadMetadata {
                root_symbol: "root".into(),
                selected_candidate: "candidate".into(),
                active_refinements: vec!["refinement".into()],
                coercion_summary: Some("coercion".into()),
            }),
        }];
        artifact.diagnostics = vec![ArtifactDiagnostic {
            diagnostic_id: "structural-diagnostic".into(),
            code: "fixture".into(),
            severity: DiagnosticSeverity::Info,
            primary_range: Some(range),
            message_key: "fixture-message".into(),
            rendered_message: "metadata".into(),
            related: vec![DiagnosticRelated {
                source_range: range,
                message_key: "related".into(),
                rendered_message: "related metadata".into(),
            }],
            explanation_ref: Some(diagnostic),
        }];
        artifact.provenance.dependency_artifact_hashes = vec![DependencyArtifactHash {
            module: artifact.module.clone(),
            interface_hash: artifact.exports[0].interface_fingerprint.clone(),
            implementation_hash: Some(ArtifactHashRef::new(
                ArtifactHashClass::Implementation,
                "fixture/dependency",
                SchemaVersion::new(1, 0),
                mizar_session::hash_text("dependency implementation"),
            )),
            artifact_hash: Some(artifact.provenance.lockfile_hash.clone()),
        }];
        artifact
            .obligations
            .sort_by(|a, b| a.obligation_id.cmp(&b.obligation_id));
        artifact
            .proof_witnesses
            .sort_by(|a, b| a.obligation_id.cmp(&b.obligation_id));
        artifact.refresh_hashes().unwrap();
        let json = if index == 0 {
            verified_artifact_json(artifact)
        } else {
            builtin_verified_artifact_json(artifact)
        }
        .unwrap();
        let path = PublishedArtifactPath::new(format!("modules/module_{index}.json")).unwrap();
        let write = write_published_artifact(
            &root,
            &path,
            &json,
            &artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version),
            &artifact_hash_excluded_paths(),
        )
        .unwrap();
        entries.push(ModuleArtifactEntry {
            module: artifact.module.clone(),
            source_file: artifact.source_file.clone(),
            source_hash: artifact.source_hash,
            artifact_file: path.as_str().into(),
            artifact_hash: ArtifactHashRef::new(
                ArtifactHashClass::Artifact,
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
                write.artifact_hash,
            ),
            interface_hash: ArtifactHashRef::new(
                ArtifactHashClass::Interface,
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
                artifact.interface_hash,
            ),
            implementation_hash: ArtifactHashRef::new(
                ArtifactHashClass::Implementation,
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
                artifact.implementation_hash,
            ),
            module_summary_file: None,
            module_summary_hash: None,
            module_summary_interface_hash: None,
            registration_summary_file: None,
            registration_summary_hash: None,
            registration_interface_hash: None,
            proof_witnesses: artifact
                .proof_witnesses
                .iter()
                .map(|w| ManifestProofWitnessEntry {
                    obligation_id: w.obligation_id.clone(),
                    obligation_fingerprint: w.obligation_fingerprint.clone(),
                    witness_path: w.witness_path.clone(),
                    witness_artifact_hash: w.witness_artifact_hash.clone(),
                })
                .collect(),
            diagnostics_hash: None,
        });
    }
    let manifest = ArtifactManifest {
        schema_version: SchemaVersion::new(1, 0),
        package: PackageIdentity {
            package_id: artifacts[0].module.package_id.clone(),
            package_version: None,
            lockfile_identity: None,
        },
        artifact_root: "artifacts".into(),
        lockfile_hash: artifacts[0].provenance.lockfile_hash.clone(),
        toolchain: artifacts[0].provenance.toolchain.clone(),
        language_edition: "2026".into(),
        verifier_config_hash: artifacts[0].provenance.verifier_config_hash.clone(),
        modules: entries,
        // Deliberately absent development file must not be loaded by this facade.
        development_artifacts: vec![DevelopmentArtifactEntry {
            kind: "structural".into(),
            path: "development/absent.json".into(),
            artifact_hash: Some(artifacts[0].provenance.lockfile_hash.clone()),
            diagnostic_hash: None,
            related_module: None,
        }],
        provenance: ManifestProvenance {
            generated_by: "caller-structural-fixture".into(),
            manifest_policy: "test".into(),
            transaction_format: "test".into(),
        },
    };
    write_manifest_file(&root, &manifest).unwrap();
    (root, manifest, artifacts)
}

fn doc_export_index_publish(
    root: &std::path::Path,
    manifest: &mut mizar_artifact::manifest::ArtifactManifest,
    artifacts: &mut [mizar_artifact::verified_artifact::VerifiedArtifact],
) -> mizar_session::Hash {
    use mizar_artifact::{manifest::write_manifest_file, store::*, verified_artifact::*};
    manifest.modules = artifacts
        .iter_mut()
        .enumerate()
        .map(|(index, artifact)| {
            artifact.refresh_hashes().unwrap();
            let json = if artifact.schema_version == SchemaVersion::new(2, 0) {
                builtin_verified_artifact_json(artifact)
            } else {
                verified_artifact_json(artifact)
            }
            .unwrap();
            let path = PublishedArtifactPath::new(format!("modules/index_{index}.json")).unwrap();
            let write = write_published_artifact(
                root,
                &path,
                &json,
                &artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version),
                &artifact_hash_excluded_paths(),
            )
            .unwrap();
            let mut entry = manifest
                .modules
                .iter()
                .find(|entry| entry.artifact_hash.schema_version == artifact.schema_version)
                .unwrap()
                .clone();
            entry.module = artifact.module.clone();
            entry.artifact_file = path.as_str().into();
            entry.artifact_hash.digest = write.artifact_hash;
            entry.interface_hash.digest = artifact.interface_hash;
            entry.implementation_hash.digest = artifact.implementation_hash;
            entry
        })
        .collect();
    write_manifest_file(root, manifest).unwrap().artifact_hash
}

fn doc_module_summaries_fixture() -> (
    PathBuf,
    mizar_artifact::manifest::ArtifactManifest,
    Vec<mizar_artifact::module_summary::ModuleSummary>,
) {
    use mizar_artifact::{
        manifest::write_manifest_file,
        module_summary::*,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::*,
    };
    let (root, mut manifest, artifacts) = doc_published_fixture();
    let mut summaries = Vec::new();
    for (index, (entry, artifact)) in manifest.modules.iter_mut().zip(artifacts).enumerate() {
        // Sidecar records are caller-owned structural metadata, not producer evidence.
        let mut summary = ModuleSummary {
            schema_version: current_schema_version(),
            module: artifact.module,
            source_hash: artifact.source_hash,
            interface_hash: mizar_session::hash_text("unrefreshed"),
            exported_symbols: Vec::new(),
            exported_labels: Vec::new(),
            lexical_summary: ModuleLexicalSummary {
                schema_version: "caller-lexical-v1".into(),
                fingerprint: (index == 0).then(|| mizar_session::hash_text("lexical")),
                contributions: Vec::new(),
            },
            reexports: Vec::new(),
            dependency_interfaces: Vec::new(),
        };
        for (ordinal, name) in ["Alpha", "Zulu"].into_iter().enumerate() {
            let range = SourceRangeSummary {
                start_byte: ordinal as u64,
                end_byte: ordinal as u64 + 1,
            };
            let mut dependency = summary.module.clone();
            dependency.module_path = format!("dependency_{ordinal}");
            summary.exported_symbols.push(ExportedSymbolSummary {
                origin_id: format!("origin-{}", 1 - ordinal),
                fully_qualified_name: format!("caller::{name}"),
                namespace_path: vec!["caller".into(), name.into()],
                visibility: "public".into(),
                declaration_kind: "structural".into(),
                source_range: range,
                rendered_signature: format!("uninterpreted {name}"),
                interface_fingerprint: mizar_session::hash_text(name),
                proof_status: (ordinal == 0).then_some(ProofStatusSummary::NotRequired),
            });
            summary.exported_labels.push(ExportedLabelSummary {
                origin_id: format!("label-{ordinal}"),
                label: "same-label".into(),
                owner_fully_qualified_name: "caller::unjoined-owner".into(),
                visibility: "public".into(),
                source_range: range,
                target_kind: format!("caller-{name}"),
            });
            summary
                .lexical_summary
                .contributions
                .push(LexicalContributionSummary {
                    kind: "notation".into(),
                    key: name.into(),
                    payload: format!("caller payload {name}"),
                });
            summary.reexports.push(ModuleReexportSummary {
                target_module: dependency.clone(),
                target_item_origin_id: (ordinal == 0).then(|| "caller-target".into()),
                exported_name: (ordinal == 0).then(|| "caller-name".into()),
                provenance_origin_id: (ordinal == 0).then(|| "caller-provenance".into()),
            });
            summary.dependency_interfaces.push(DependencyInterfaceRef {
                module: dependency,
                interface_hash: mizar_session::hash_text(name),
            });
        }
        summary.refresh_interface_hash().unwrap();
        let path = PublishedArtifactPath::new(format!("summaries/module_{index}.json")).unwrap();
        let write = write_published_artifact(
            &root,
            &path,
            &module_summary_json(&summary).unwrap(),
            &artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version),
            &[],
        )
        .unwrap();
        entry.module_summary_file = Some(path.as_str().into());
        entry.module_summary_hash = Some(ArtifactHashRef::new(
            ArtifactHashClass::Artifact,
            MODULE_SUMMARY_SCHEMA_FAMILY,
            summary.schema_version,
            write.artifact_hash,
        ));
        entry.module_summary_interface_hash = Some(ArtifactHashRef::new(
            ArtifactHashClass::Interface,
            MODULE_SUMMARY_SCHEMA_FAMILY,
            summary.schema_version,
            summary.interface_hash,
        ));
        summaries.push(summary);
    }
    write_manifest_file(&root, &manifest).unwrap();
    (root, manifest, summaries)
}

mod algorithm;
mod attribute;
mod bare;
mod binder;
mod computation;
mod doc;
mod match_cases;
mod misc;
mod source_algorithm;
mod source_candidate;
mod source_original;
mod source_public;
mod source_registration;
mod step5c11;
mod step5c13;
mod step5c14;
mod step5c3;
mod step5c4;
mod step5c5;
mod step5c6;
mod terminating;
