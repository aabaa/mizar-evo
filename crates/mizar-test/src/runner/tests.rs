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
                    (
                        entry.site(),
                        entry.kind(),
                        entry.range(),
                        entry.parent(),
                        entry.child_ordinal(),
                        entry.preorder_ordinal(),
                        entry.recovery(),
                        entry.spelling(),
                    ),
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
                token_count += usize::from(matches!(entry.kind(), K::Token(_)));
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

#[test]
fn task270_advanced_inline_closure_requests_execute_the_active_consumer() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let id = "pass_advanced_semantics_inline_closure_requests_001";
    let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
    assert_eq!(case.expectation.stage, crate::Stage::AdvancedSemantics);
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let frontend = super::formula_statement::step5c8_test_frontend(&text);
    assert!(frontend.ast.is_some());
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let report = super::run_advanced_semantics_corpus(&config).unwrap();
    let result = report
        .results
        .iter()
        .find(|result| result.id.0 == id)
        .unwrap_or_else(|| panic!("request case missing: {:?}", report.diagnostics));
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Passed);
}

#[test]
fn task270_advanced_inline_closure_requests_build_the_native_transaction() {
    use mizar_checker::{
        binding_env::BinderIdentity,
        source_proof_local_declaration::SourceProofLocalDeclarationProducer,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::SurfaceResolvedArena,
    };
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_advanced_semantics_inline_closure_requests_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let (ast, module, _, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(&text, 2_704_001);
    assert_eq!(diagnostics, 0);
    let original =
        super::type_elaboration::source_proof_local_syntax_receiver(&ast, &module, &text).unwrap();
    let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let scope = SourceVariableScopeResolver::resolve_proof_bindings(SourceVariableScopeInput::new(
        &ast, &module, &symbols,
    ))
    .unwrap();
    assert_eq!(
        scope
            .bindings()
            .iter()
            .map(|binding| binding.spelling())
            .collect::<Vec<_>>(),
        ["m", "a", "F", "n", "k", "P", "n", "m"]
    );
    let handoff =
        SourceProofLocalDeclarationProducer::build_bindings(&original, &native, &symbols, &scope)
            .expect("native inline closure transaction");
    let outer = &scope.bindings()[0];
    let expected_capture = BinderIdentity::ResolverLocal {
        scope: outer.scope().clone(),
        ordinal: outer.ordinal(),
        declaration_range: outer.range(),
    };
    for id in [2, 5] {
        assert_eq!(
            handoff
                .binding_env()
                .bindings()
                .get(mizar_checker::binding_env::BindingId::new(id))
                .unwrap()
                .captured
                .identities(),
            std::slice::from_ref(&expected_capture)
        );
    }
}

fn task270_request_handoff(
    text: &str,
) -> mizar_checker::source_proof_local_declaration::SourceProofLocalBindingHandoff {
    let (original, native, symbols, scope) = task270_request_inputs(text);
    mizar_checker::source_proof_local_declaration::SourceProofLocalDeclarationProducer::build_bindings(&original, &native, &symbols, &scope).unwrap()
}

fn task270_request_inputs(
    text: &str,
) -> (
    mizar_checker::typed_ast::TypedAst,
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
    mizar_resolve::env::SymbolEnv,
    mizar_resolve::names::ResolvedVariableScope,
) {
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::SurfaceResolvedArena,
    };
    let (ast, module, _, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(text, 2_704_100);
    assert_eq!(diagnostics, 0);
    let original =
        super::type_elaboration::source_proof_local_syntax_receiver(&ast, &module, text).unwrap();
    let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let scope = SourceVariableScopeResolver::resolve_proof_bindings(SourceVariableScopeInput::new(
        &ast, &module, &symbols,
    ))
    .unwrap();
    (original, native, symbols, scope)
}

#[test]
fn task270_advanced_inline_closure_requests_match_original_role_and_identity_oracles() {
    use mizar_syntax::ast::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::INLINE_CLOSURE_REQUEST_ID)
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let handoff = task270_request_handoff(&text);
    let rows = handoff
        .source()
        .source_node_entries()
        .unwrap()
        .collect::<Vec<_>>();
    let child = |parent, ordinal| {
        rows.iter()
            .copied()
            .find(|row| row.parent() == Some(parent) && row.child_ordinal() == ordinal)
            .unwrap()
    };
    let f = rows
        .iter()
        .find(|row| row.kind() == &K::InlineFunctorDefinition)
        .unwrap();
    let p = rows
        .iter()
        .find(|row| row.kind() == &K::InlinePredicateDefinition)
        .unwrap();
    let fc = rows
        .iter()
        .find(|row| row.kind() == &K::ApplicationTerm)
        .unwrap();
    let pc = rows
        .iter()
        .find(|row| row.kind() == &K::InlinePredicateApplication)
        .unwrap();
    let fb = child(f.site(), 10).site().index();
    let ft = child(f.site(), 8).site().index();
    let pb = child(p.site(), 6).site().index();
    let ng = child(child(f.site(), 3).site(), 2).site().index();
    let kg = child(child(f.site(), 5).site(), 2).site().index();
    let pg = child(child(p.site(), 3).site(), 2).site().index();
    use mizar_checker::binding_env::{BindingContextLayer, BindingContextOwner, BindingId, BindingTypeSite};
    let env = handoff.binding_env();
    let binding = |id| env.bindings().get(BindingId::new(id)).unwrap();
    let proof = binding(0).owner_context;
    assert_eq!(env.contexts().get(proof).unwrap().layer, BindingContextLayer::Proof);
    for (name, formal, members, definition) in [(2, 3, vec![3, 4], *f), (5, 6, vec![6], *p)] {
        assert_eq!(binding(name).owner_context, proof);
        let context = env.contexts().get(binding(formal).owner_context).unwrap();
        assert_eq!(context.layer, BindingContextLayer::Declaration);
        assert_eq!(context.parent, Some(proof));
        assert_eq!(context.owner, BindingContextOwner::SourceStatement { source_range: definition.range() });
        assert_eq!(context.bindings, members.iter().copied().map(BindingId::new).collect::<Vec<_>>());
        for id in members {
            assert_eq!(binding(id).owner_context, context.id);
            assert_eq!(context.lexical_scope.as_ref(), Some(handoff.scope().bindings()[id].scope()));
        }
    }
    for (id, site) in [(2, ft), (3, ng), (4, kg), (6, pg)] {
        let original = rows.iter().find(|row| row.site().index() == site).unwrap();
        assert_eq!(binding(id).type_site, BindingTypeSite::Source(original.range()));
    }
    assert_eq!(binding(5).type_site, BindingTypeSite::Missing);
    let mut expected = vec![
        format!("closure.2.InlineFunctor.body={fb}.result={ft}.arity=2"),
        format!("formal.2.0.binding=3.guard={ng}.state=pending"),
        format!("formal.2.1.binding=4.guard={kg}.state=pending"),
        "capture.2.0".to_owned(),
        format!("closure.5.InlinePredicate.body={pb}.result=none.arity=1"),
        format!("formal.5.0.binding=6.guard={pg}.state=pending"),
        "capture.5.0".to_owned(),
    ];
    for (id, target) in [0, 6, 0, 5, 7, 2, 7, 1, 7].into_iter().enumerate() {
        expected.push(format!("reference.{id}.target={target}.ordinal={id}"));
        if id == 3 {
            expected.push(format!(
                "request.3.callee=5.call={}.body={pb}.arity=1.state=pending",
                pc.site().index()
            ));
            expected.push(format!(
                "argument.3.0.formal=6.actual={}.guard={pg}.state=pending",
                child(pc.site(), 2).site().index()
            ));
        } else if id == 5 {
            expected.push(format!(
                "request.5.callee=2.call={}.body={fb}.arity=2.state=pending",
                fc.site().index()
            ));
            expected.push(format!(
                "argument.5.0.formal=3.actual={}.guard={ng}.state=pending",
                child(fc.site(), 2).site().index()
            ));
            expected.push(format!(
                "argument.5.1.formal=4.actual={}.guard={kg}.state=pending",
                child(fc.site(), 4).site().index()
            ));
        }
    }
    assert_eq!(
        super::inline_closure_request_observation(&handoff).unwrap(),
        expected
    );
    let actual = super::source_inline_closure_requests(
        &config.workspace_root,
        case,
        super::run_frontend(&config.workspace_root, case, 0).unwrap(),
    )
    .unwrap();
    assert_eq!(actual, expected);
    let swapped = text.replace("F(m, a)", "F(a, m)");
    let swapped =
        super::inline_closure_request_observation(&task270_request_handoff(&swapped)).unwrap();
    assert_ne!(swapped, expected);
    assert!(swapped.contains(&"reference.6.target=1.ordinal=6".to_owned()));
    assert!(swapped.contains(&"reference.7.target=7.ordinal=7".to_owned()));
}

#[test]
fn task270_advanced_inline_closure_requests_keep_zero_roots_and_formal_body_identity() {
    let text = "theorem Zero: thesis proof let m be object; deffunc C() -> object equals m; defpred Q() means m = m; now let m be object; A: Q(); thus C() = m; end; thus thesis; end;";
    let handoff = task270_request_handoff(text);
    let actual = super::inline_closure_request_observation(&handoff).unwrap();
    let roots = actual
        .iter()
        .filter(|key| key.starts_with("request."))
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 2);
    assert!(roots[0].starts_with("request.3.callee=2."));
    assert!(roots[1].starts_with("request.4.callee=1."));
    assert!(
        roots
            .iter()
            .all(|key| key.ends_with(".arity=0.state=pending"))
    );
    assert!(
        !actual
            .iter()
            .any(|key| key.starts_with("formal.") || key.starts_with("argument."))
    );
    assert!(actual.contains(&"capture.1.0".to_owned()));
    assert!(actual.contains(&"capture.2.0".to_owned()));
    let formal = task270_request_handoff(
        "theorem FormalBody: thesis proof let m, a be object; deffunc F(n be object, k be object) -> object equals n; defpred P(n be object, k be object) means n = m & k = k; now let m be object; A: P(m, a); thus F(m, a) = m; end; thus thesis; end;",
    );
    let actual = super::inline_closure_request_observation(&formal).unwrap();
    assert!(!actual.iter().any(|key| key.starts_with("capture.2.")));
    assert!(actual.contains(&"reference.0.target=3.ordinal=0".to_owned()));
    assert!(actual.contains(&"capture.5.0".to_owned()));
    assert!(
        actual
            .iter()
            .any(|key| key.starts_with("formal.5.1.binding=7.") && key.ends_with(".state=pending"))
    );
}

#[test]
fn task270_advanced_inline_closure_requests_admit_only_the_frozen_case_and_keep_layout() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::INLINE_CLOSURE_REQUEST_ID)
        .unwrap();
    assert!(super::inline_closure_requests_admitted(
        &config.workspace_root,
        case
    ));
    for mutation in 0..7 {
        let mut bad = case.clone();
        match mutation {
            0 => bad.expectation.domain = "resolve.declaration_symbol".to_owned(),
            1 => bad.source_path = config.workspace_root.join("tests/miz/pass/other.miz"),
            2 => bad.expectation.tags.clear(),
            3 => {
                bad.expectation.expected_phase = Some(crate::expectation::PipelinePhase::TypeCheck)
            }
            4 => bad.expectation.stable_detail_key = Some("unexpected.key".to_owned()),
            5 => bad.expectation.spec_refs.clear(),
            _ => bad.expectation.profiles.push("heavy".to_owned()),
        }
        assert!(!super::inline_closure_requests_admitted(
            &config.workspace_root,
            &bad
        ));
        let mut invalid = plan.clone();
        invalid.cases.push(bad);
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-ADVANCED-SEMANTICS-ADMISSION")
        );
    }
    let mut duplicate = plan.clone();
    duplicate.cases.push(case.clone());
    let mut missing = plan.clone();
    missing.cases.retain(|candidate| candidate.id != case.id);
    for invalid in [duplicate, missing] {
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|diagnostic| diagnostic.detail_key
                    == format!(
                        "advanced_semantics.inventory.{}",
                        super::INLINE_CLOSURE_REQUEST_ID
                    ))
        );
    }
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let baseline =
        super::inline_closure_request_observation(&task270_request_handoff(&text)).unwrap();
    for source in [
        format!(
            ":: genuine layout\n{}",
            text.replace("n be object, k be object", "n\tbe object,\n k being object")
        ),
        text.replace("InlineClosureRequests", "RenamedRequests")
            .replace("m,", "outer_name,")
            .replace("m;", "outer_name;")
            .replace("m)", "outer_name)")
            .replace("m be", "outer_name be")
            .replace("n be", "formal be")
            .replace("n =", "formal ="),
    ] {
        let actual = super::source_inline_closure_requests(
            &config.workspace_root,
            case,
            super::formula_statement::step5c8_test_frontend(&source),
        )
        .unwrap();
        assert_eq!(actual, baseline);
    }
    let unused = text
        .replace("A: P(m);", "A: m = m;")
        .replace("thus F(m, a) = m;", "thus m = m;");
    let report = metamorphic_guard::with_source(case, &unused, || {
        super::run_advanced_semantics_corpus(&config)
    })
    .unwrap();
    let result = report
        .results
        .iter()
        .find(|result| result.id == case.id)
        .unwrap();
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Failed);
    assert_eq!(
        result.actual_detail_keys,
        ["inline closure request does not match original roles"]
    );
}

#[test]
fn task270_advanced_inline_closure_requests_refuse_legacy_local_capture_and_foreign_roles() {
    use mizar_checker::source_proof_local_declaration::{
        SourceProofLocalDeclarationError, SourceProofLocalDeclarationProducer,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::validate_source_proof_occurrences,
    };
    for definition in [
        "defpred P(n be object) means for q being object holds q = m; A: P(m);",
        "deffunc F(n be object) -> set equals {q where q is object : q = m}; A: F(m) = F(m);",
    ] {
        let text = format!(
            "theorem LocalCapture: thesis proof let m be object; {definition} thus thesis; end;"
        );
        let (original, native, symbols, current) = task270_request_inputs(&text);
        let (ast, module, _, _, _) =
            task253_ast_from_source_text_with_diagnostic_count(&text, 2_704_100);
        let legacy = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, &module, &symbols),
        )
        .unwrap();
        validate_source_proof_occurrences(&native, &symbols, &legacy).unwrap();
        let inline = current
            .bindings()
            .iter()
            .find(|binding| binding.arity().is_some())
            .unwrap();
        assert_eq!(
            inline
                .captures()
                .iter()
                .map(|id| current.bindings()[id.index()].spelling())
                .collect::<Vec<_>>(),
            ["m"]
        );
        assert_eq!(
            legacy.bindings()[inline.id().index()]
                .captures()
                .iter()
                .map(|id| legacy.bindings()[id.index()].spelling())
                .collect::<Vec<_>>(),
            ["m", "q"]
        );
        assert!(
            SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &current
            )
            .is_ok()
        );
        assert_eq!(
            SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &legacy
            ),
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        );
    }
    let plan = crate::harness::build_test_plan(&step5c11_config()).unwrap();
    let text = std::fs::read_to_string(
        &plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::INLINE_CLOSURE_REQUEST_ID)
            .unwrap()
            .source_path,
    )
    .unwrap();
    let (original, _, _, _) = task270_request_inputs(&text);
    for foreign in [
        text.replace("n be object, k", "n be set, k"),
        text.replace("equals m", "equals n"),
        text.replace("n be object, k be object", "k be object, n be object"),
        text.replace("F(m, a)", "F(a, m)"),
    ] {
        let (foreign_original, native, symbols, scope) = task270_request_inputs(&foreign);
        assert_eq!(foreign_original.source_id(), original.source_id());
        assert_eq!(foreign_original.module_id(), original.module_id());
        assert!(
            SourceProofLocalDeclarationProducer::build_bindings(
                &foreign_original,
                &native,
                &symbols,
                &scope
            )
            .is_ok()
        );
        assert_eq!(
            SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &scope
            ),
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        );
    }
    for (definition, bare) in [
        ("deffunc F() -> object equals m;", "F"),
        ("defpred P() means m = m;", "P"),
    ] {
        let text = format!(
            "theorem Bare: thesis proof let m be object; {definition} set a = {bare}; thus thesis; end;"
        );
        let (original, native, symbols, scope) = task270_request_inputs(&text);
        assert_eq!(
            SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &scope
            ),
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        );
    }
}

#[test]
fn task271_native_conversions_build_from_existing_formula_fixture() {
    use mizar_checker::source_proof_local_declaration::SourceProofLocalDeclarationProducer;
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::SurfaceResolvedArena,
    };
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_reconsider_builtin_widening_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let (ast, module, _, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(&text, 2_714_001);
    assert_eq!(diagnostics, 0);
    let original =
        super::type_elaboration::source_proof_local_syntax_receiver(&ast, &module, &text).unwrap();
    let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let scope = SourceVariableScopeResolver::resolve_proof_bindings(SourceVariableScopeInput::new(
        &ast, &module, &symbols,
    ))
    .unwrap();
    SourceProofLocalDeclarationProducer::build_bindings(&original, &native, &symbols, &scope)
        .expect("native reconsider transaction");
}

#[test]
fn task271_native_conversions_execute_classification_runtime() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let id = "pass_advanced_semantics_reconsider_classification_001";
    let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
    let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
    assert!(frontend.ast.is_some());
    let report = super::run_advanced_semantics_corpus(&config).unwrap();
    let result = report
        .results
        .iter()
        .find(|result| result.id.0 == id)
        .unwrap_or_else(|| panic!("classification case missing: {:?}", report.diagnostics));
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Passed);
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
}

fn task271_conversion_source(
    text: &str,
) -> (
    mizar_checker::source_proof_local_declaration::SourceProofLocalBindingHandoff,
    mizar_resolve::env::SymbolEnv,
) {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_CONVERSION_ID)
        .unwrap();
    let frontend = metamorphic_guard::with_source(case, text, || {
        super::run_frontend(&config.workspace_root, case, 0)
    })
    .unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    (
        super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap(),
        resolver.env,
    )
}

#[test]
fn task271_native_conversions_match_manual_original_requests_queries_and_statuses() {
    use mizar_checker::{
        type_checker::{CoercionRequestKind as C, TypeFactQueryStatus as Q, TypeHeadInput as H},
        typed_ast::{
            CoercionStatus as S, InitialObligationStatus, Polarity, TypeRole, TypedSiteRef,
        },
    };
    use mizar_syntax::ast::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_CONVERSION_ID)
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let (handoff, symbols) = task271_conversion_source(&text);
    assert_eq!(
        handoff
            .scope()
            .bindings()
            .iter()
            .map(|binding| binding.spelling())
            .collect::<Vec<_>>(),
        ["s", "o", "a", "again", "e", "p", "rejected", "viewed"]
    );
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [0, 0, 2, 1, 1, 0, 0]
    );
    let (requests, checked, queries) = handoff.check_conversions().unwrap();
    assert_eq!(
        (
            requests.len(),
            checked.coercions().len(),
            queries.len(),
            checked.initial_obligations().len()
        ),
        (7, 7, 6, 2)
    );
    let rows = handoff
        .source()
        .source_node_entries()
        .unwrap()
        .collect::<Vec<_>>();
    let mut items = rows
        .iter()
        .copied()
        .filter(|row| row.kind() == &K::ReconsiderItem)
        .collect::<Vec<_>>();
    items.sort_by_key(|row| row.preorder_ordinal());
    let qua = rows
        .iter()
        .copied()
        .find(|row| row.kind() == &K::QuaExpression)
        .unwrap();
    let target = |item: &mizar_checker::typed_ast::SourceNode| {
        rows.iter()
            .copied()
            .find(|row| row.parent() == item.parent() && row.kind() == &K::TypeExpression)
            .unwrap()
    };
    let targets = items.iter().map(|item| target(item)).collect::<Vec<_>>();
    assert_eq!(targets[0].site(), targets[1].site());
    let declared_type = |name: &str| {
        let binding = handoff
            .scope()
            .bindings()
            .iter()
            .find(|binding| binding.spelling() == name)
            .unwrap();
        let token = rows
            .iter()
            .copied()
            .find(|row| row.range() == binding.range() && matches!(row.kind(), K::Token(_)))
            .unwrap();
        rows.iter()
            .copied()
            .find(|row| row.parent() == token.parent() && row.kind() == &K::TypeExpression)
            .unwrap()
    };
    let st = declared_type("s");
    let ot = declared_type("o");
    let qt = rows
        .iter()
        .copied()
        .find(|row| row.parent() == Some(qua.site()) && row.kind() == &K::TypeExpression)
        .unwrap();
    let from_nodes = [st, st, targets[0], ot, ot, st, targets[1]];
    let to_nodes = [
        targets[0], targets[1], targets[2], targets[3], targets[4], targets[5], qt,
    ];
    let expected = [
        (
            C::Widening,
            H::BuiltinSet,
            H::BuiltinObject,
            S::Candidate,
            Some(Q::Satisfied),
        ),
        (
            C::Widening,
            H::BuiltinSet,
            H::BuiltinObject,
            S::Candidate,
            Some(Q::Satisfied),
        ),
        (
            C::Widening,
            H::BuiltinObject,
            H::BuiltinObject,
            S::Candidate,
            Some(Q::Satisfied),
        ),
        (
            C::Narrowing,
            H::BuiltinObject,
            H::BuiltinSet,
            S::RequiresObligation,
            Some(Q::Missing),
        ),
        (
            C::Narrowing,
            H::BuiltinObject,
            H::BuiltinSet,
            S::RequiresObligation,
            Some(Q::Missing),
        ),
        (
            C::Narrowing,
            H::BuiltinSet,
            H::BuiltinSet,
            S::Rejected,
            Some(Q::Missing),
        ),
        (
            C::SourceQua,
            H::BuiltinObject,
            H::BuiltinObject,
            S::Candidate,
            None,
        ),
    ];
    let typed_node = |row: &mizar_checker::typed_ast::SourceNode| {
        handoff
            .source()
            .nodes()
            .iter()
            .find(|(_, node)| node.resolved_node == Some(row.site()))
            .unwrap()
            .0
    };
    for (index, (request, (kind, from, to, status, query_status))) in
        requests.iter().zip(expected).enumerate()
    {
        let original = if index < 6 { items[index] } else { qua };
        assert_eq!(request.site, TypedSiteRef::Node(typed_node(original)));
        assert_eq!(request.kind, kind);
        if index < 6 {
            assert_eq!(
                request.justification,
                if index == 3 || index == 4 {
                    mizar_checker::type_checker::CoercionJustification::Explicit
                } else {
                    mizar_checker::type_checker::CoercionJustification::Omitted
                }
            );
        }
        assert_eq!(request.from_type.as_ref().unwrap().head, from);
        assert_eq!(request.to_type.head, to);
        assert_eq!(
            request.from_type.as_ref().unwrap().source_range,
            from_nodes[index].range()
        );
        assert_eq!(request.to_type.source_range, to_nodes[index].range());
        assert_eq!(
            request.from_type.as_ref().unwrap().site,
            TypedSiteRef::Role {
                node: typed_node(from_nodes[index]),
                role: TypeRole::new(format!("conversion.{}.from", typed_node(original).index()))
            }
        );
        assert_eq!(
            request.to_type.site,
            TypedSiteRef::Role {
                node: typed_node(to_nodes[index]),
                role: TypeRole::new(format!("conversion.{}.to", typed_node(original).index()))
            }
        );
        let result = checked
            .coercions()
            .iter()
            .find(|(_, result)| result.site == request.site)
            .unwrap()
            .1;
        assert_eq!(result.status, status);
        if index == 3 || index == 4 {
            assert_eq!(
                checked
                    .initial_obligations()
                    .get(result.obligation.unwrap())
                    .unwrap()
                    .status,
                InitialObligationStatus::Pending
            );
        } else {
            assert!(result.obligation.is_none());
        }
        let query = queries
            .iter()
            .find(|query| query.query().subject == request.site);
        assert_eq!(query.map(|query| query.status()), query_status);
        if let Some(query) = query {
            assert_eq!(query.query().polarity, Polarity::Positive);
            assert_eq!(
                query.query().predicate.as_str(),
                "coercion.widening.builtin_radix"
            );
            assert_eq!(query.query().source_range, original.range());
            assert_eq!(query.matched_facts().len(), usize::from(index < 3));
            for id in query.matched_facts() {
                assert_eq!(checked.facts().get(*id).unwrap().subject, request.site);
            }
        }
    }
    assert_ne!(requests[0].to_type.site, requests[1].to_type.site);
    assert_eq!(requests[5].to_type.attributes.len(), 1);
    let attribute = &requests[5].to_type.attributes[0];
    assert_eq!(
        symbols
            .symbols()
            .get(&attribute.symbol)
            .unwrap()
            .primary_spelling(),
        "marked1"
    );
    assert_eq!(
        symbols.symbols().get(&attribute.symbol).unwrap().kind(),
        mizar_resolve::env::SymbolKind::Attribute
    );
    assert_eq!(
        checked
            .diagnostics()
            .iter()
            .map(|(_, diagnostic)| diagnostic.message_key.as_str())
            .collect::<Vec<_>>(),
        ["type.narrowing_requires_proof"]
    );
    let observed = super::source_native_conversions(
        &config.workspace_root,
        case,
        super::run_frontend(&config.workspace_root, case, 0).unwrap(),
    )
    .unwrap();
    assert_eq!(
        observed,
        super::native_conversion_observation(&handoff).unwrap()
    );
    let conversions = observed
        .iter()
        .filter(|key| key.starts_with("conversion."))
        .collect::<Vec<_>>();
    assert_eq!(conversions.len(), 7);
    assert!(conversions[3].contains(".intent=by."));
    assert!(conversions[4].contains(".intent=proof."));
    assert!(conversions[5].contains(".intent=omitted."));
}

#[test]
fn task271_native_conversions_keep_view_lifetimes_and_refuse_pending_alias_laundering() {
    use mizar_checker::type_checker::TypeHeadInput as H;
    let text = "theorem Views: thesis proof let s be set; now reconsider s as object; reconsider t = s as object; thus thesis; end; reconsider u = s as object; thus thesis; end;";
    let (handoff, _) = task271_conversion_source(text);
    let (requests, _, _) = handoff.check_conversions().unwrap();
    assert_eq!(
        requests
            .iter()
            .map(|request| request.from_type.as_ref().unwrap().head.clone())
            .collect::<Vec<_>>(),
        [H::BuiltinSet, H::BuiltinObject, H::BuiltinSet]
    );
    let (handoff, _) = task271_conversion_source(
        "theorem OriginalAlias: thesis proof let s be set; reconsider a = s as object; reconsider b = a as set; thus thesis; end;",
    );
    let (requests, _, _) = handoff.check_conversions().unwrap();
    assert_eq!(requests[1].from_type.as_ref().unwrap().head, H::BuiltinSet);
    for text in [
        "theorem Pending: thesis proof let o be object; A: thesis; reconsider e = o as set by A; reconsider leaked = e as object; thus thesis; end;",
        "theorem QuaChain: thesis proof let o be object; set a = (o qua set) qua object; thus thesis; end;",
    ] {
        assert!(
            task271_conversion_source(text)
                .0
                .check_conversions()
                .is_err()
        );
    }
    let (handoff, _) = task271_conversion_source(
        "theorem PendingView: thesis proof let o be object; A: thesis; reconsider o as set by A; reconsider a = o as object; thus thesis; end;",
    );
    let (requests, checked, _) = handoff.check_conversions().unwrap();
    assert_eq!(
        requests[1].from_type.as_ref().unwrap().head,
        H::BuiltinObject
    );
    assert_eq!(
        checked
            .coercions()
            .iter()
            .find(|(_, result)| result.site == requests[1].site)
            .unwrap()
            .1
            .status,
        mizar_checker::typed_ast::CoercionStatus::Candidate
    );
}

#[test]
fn task271_native_conversions_consume_whole_inactive_seed_without_activating_it() {
    use mizar_checker::{type_checker::TypeFactQueryStatus, typed_ast::CoercionStatus};
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let old = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_types_reconsider_omitted_justification_001")
        .unwrap();
    assert!(old.expectation.tags.is_empty());
    assert_eq!(
        old.expectation.stable_detail_key.as_deref(),
        Some("conversion.reconsider.narrowing_requires_proof")
    );
    let frontend = super::run_frontend(&config.workspace_root, old, 0).unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, old, &ast);
    assert!(resolver.detail_keys.is_empty());
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    assert!(
        handoff
            .source()
            .source_node_entries()
            .unwrap()
            .any(|row| matches!(
                row.kind(),
                mizar_syntax::SurfaceNodeKind::DefinitionBlockItem
            ))
    );
    let (requests, checked, queries) = handoff.check_conversions().unwrap();
    assert_eq!(
        (requests.len(), checked.coercions().len(), queries.len()),
        (1, 1, 1)
    );
    assert_eq!(requests[0].to_type.attributes.len(), 1);
    assert_eq!(requests[0].to_type.attributes[0].spelling, "keen");
    assert_eq!(
        checked.coercions().iter().next().unwrap().1.status,
        CoercionStatus::Rejected
    );
    assert_eq!(queries[0].status(), TypeFactQueryStatus::Missing);
    assert!(checked.initial_obligations().is_empty());
    assert_eq!(
        checked
            .diagnostics()
            .iter()
            .map(|(_, diagnostic)| diagnostic.message_key.as_str())
            .collect::<Vec<_>>(),
        ["type.narrowing_requires_proof"]
    );
    assert!(
        !super::run_advanced_semantics_corpus(&config)
            .unwrap()
            .results
            .iter()
            .any(|case| case.id == old.id)
    );
    let prepared = super::run_advanced_semantics_case(&config.workspace_root, old, 0);
    assert_eq!(prepared.status, super::TypeElaborationCaseStatus::Passed);
    assert_eq!(
        prepared.actual_detail_keys,
        ["conversion.reconsider.narrowing_requires_proof"]
    );
}

#[test]
fn task271_native_conversions_guard_admission_and_actual_semantic_consumer_paths() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_CONVERSION_ID)
        .unwrap();
    assert!(super::native_conversions_admitted(
        &config.workspace_root,
        case
    ));
    for mutation in 0..6 {
        let mut bad = case.clone();
        match mutation {
            0 => bad.expectation.domain = "types.conversion".to_owned(),
            1 => bad.expectation.tags.clear(),
            2 => bad.expectation.expected_phase = Some(crate::expectation::PipelinePhase::Resolve),
            3 => bad.source_path = config.workspace_root.join("tests/miz/pass/other.miz"),
            4 => bad.expectation.spec_refs.clear(),
            _ => bad.expectation.stable_detail_key = Some("unexpected".to_owned()),
        }
        assert!(!super::native_conversions_admitted(
            &config.workspace_root,
            &bad
        ));
        let mut invalid = plan.clone();
        invalid.cases.push(bad);
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-ADVANCED-SEMANTICS-ADMISSION")
        );
    }
    let mut duplicate = plan.clone();
    duplicate.cases.push(case.clone());
    let mut missing = plan.clone();
    missing.cases.retain(|candidate| candidate.id != case.id);
    for invalid in [duplicate, missing] {
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|diagnostic| diagnostic.detail_key
                    == format!(
                        "advanced_semantics.inventory.{}",
                        super::NATIVE_CONVERSION_ID
                    ))
        );
    }
    let empty = "theorem NoConversions: thesis proof let s be set; thus thesis; end;";
    let report = metamorphic_guard::with_source(case, empty, || {
        super::run_advanced_semantics_corpus(&config)
    })
    .unwrap();
    assert_eq!(
        report
            .results
            .iter()
            .find(|result| result.id == case.id)
            .unwrap()
            .status,
        super::TypeElaborationCaseStatus::Failed
    );
    let unsupported_alias = "theorem UnsupportedAlias: thesis proof let s be set; set alias = s; reconsider a = alias as object; thus thesis; end;";
    assert!(
        task271_conversion_source(unsupported_alias)
            .0
            .check_conversions()
            .is_err()
    );
    let report = metamorphic_guard::with_source(case, unsupported_alias, || {
        super::run_advanced_semantics_corpus(&config)
    })
    .unwrap();
    assert_eq!(
        report
            .results
            .iter()
            .find(|result| result.id == case.id)
            .unwrap()
            .status,
        super::TypeElaborationCaseStatus::Failed
    );
    let fs = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_reconsider_builtin_widening_001")
        .unwrap();
    assert_eq!(
        super::formula_statement::run_formula_statement_case(&config.workspace_root, fs, 0).status,
        super::FormulaStatementCaseStatus::Passed
    );
    let source = std::fs::read_to_string(&fs.source_path)
        .unwrap()
        .replace("reconsider Y = X", "set alias = X;\n  reconsider Y = alias");
    let frontend = metamorphic_guard::with_source(fs, &source, || {
        super::run_frontend(&config.workspace_root, fs, 0)
    })
    .unwrap();
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, fs, &ast);
    assert!(
        super::formula_statement::source_variable_semantics_detail_keys(
            &ast,
            &resolver.module,
            &resolver.env
        )
        .is_empty()
    );
    let failed = metamorphic_guard::with_source(fs, &source, || {
        super::formula_statement::run_formula_statement_case(&config.workspace_root, fs, 0)
    });
    assert_eq!(failed.status, super::FormulaStatementCaseStatus::Failed);
    assert!(
        failed
            .actual_detail_keys
            .iter()
            .any(|key| key.contains("native reconsider"))
    );
}

#[test]
fn task271_native_conversions_preserve_layout_names_and_refuse_rejected_aliases() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_CONVERSION_ID)
        .unwrap();
    let source = std::fs::read_to_string(&case.source_path).unwrap();
    let baseline = super::native_conversion_observation(&task271_conversion_source(&source).0)
        .unwrap()
        .into_iter()
        .filter(|key| key.starts_with("conversion."))
        .collect::<Vec<_>>();
    let renamed = source
        .replace("NativeConversions", "RenamedConversions")
        .replace("marked1", "marked2")
        .replace("let s be", "let seed be")
        .replace("= s, s as", "= seed, seed as")
        .replace("rejected = s as", "rejected = seed as")
        .replace("viewed = s qua", "viewed = seed qua");
    for (text, expected) in [
        (
            format!(
                ":: genuine conversion layout\n{}",
                source.replace(
                    "reconsider a = s, s as object",
                    "reconsider\ta = s,\n s as object"
                )
            ),
            baseline.clone(),
        ),
        (
            renamed,
            baseline
                .iter()
                .map(|key| key.replace("marked1", "marked2"))
                .collect(),
        ),
    ] {
        let actual = super::native_conversion_observation(&task271_conversion_source(&text).0)
            .unwrap()
            .into_iter()
            .filter(|key| key.starts_with("conversion."))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
    let rejected_alias = source.replace(
        "set viewed =",
        "reconsider laundered = rejected as object;\n  set viewed =",
    );
    assert!(
        task271_conversion_source(&rejected_alias)
            .0
            .check_conversions()
            .is_err()
    );
}

#[test]
fn task271_native_conversions_qua_uses_apparent_view_while_reconsider_retains_original() {
    use mizar_checker::{
        type_checker::{CoercionRequestKind as C, TypeFactQueryStatus as Q, TypeHeadInput as H},
        typed_ast::CoercionStatus as S,
    };
    let source = "theorem Apparent: thesis proof let s be set; now reconsider s as object; set denied = s qua set; reconsider keep = s as set; thus thesis; end; set restored = s qua set; thus thesis; end;";
    let (handoff, _) = task271_conversion_source(source);
    let (requests, checked, queries) = handoff.check_conversions().unwrap();
    assert_eq!(
        (requests.len(), checked.coercions().len(), queries.len()),
        (4, 4, 2)
    );
    for (request, (kind, from, to, status, query)) in requests.iter().zip([
        (
            C::Widening,
            H::BuiltinSet,
            H::BuiltinObject,
            S::Candidate,
            Some(Q::Satisfied),
        ),
        (
            C::SourceQua,
            H::BuiltinObject,
            H::BuiltinSet,
            S::Rejected,
            None,
        ),
        (
            C::Widening,
            H::BuiltinSet,
            H::BuiltinSet,
            S::Candidate,
            Some(Q::Satisfied),
        ),
        (
            C::SourceQua,
            H::BuiltinSet,
            H::BuiltinSet,
            S::Candidate,
            None,
        ),
    ]) {
        assert_eq!(request.kind, kind);
        assert_eq!(request.from_type.as_ref().unwrap().head, from);
        assert_eq!(request.to_type.head, to);
        let result = checked
            .coercions()
            .iter()
            .find(|(_, result)| result.site == request.site)
            .unwrap()
            .1;
        assert_eq!(result.status, status);
        assert!(result.obligation.is_none());
        assert_eq!(
            queries
                .iter()
                .find(|entry| entry.query().subject == request.site)
                .map(|entry| entry.status()),
            query
        );
    }
    assert!(checked.initial_obligations().is_empty());
    assert_eq!(
        checked
            .diagnostics()
            .iter()
            .map(|(_, diagnostic)| diagnostic.message_key.as_str())
            .collect::<Vec<_>>(),
        ["checker.coercion.invalid_source_qua_target"]
    );
}

#[test]
fn task271_native_conversions_accept_infix_qua_roles_without_inventing_type() {
    use mizar_syntax::ast::SurfaceNodeKind as K;
    let source = "definition let lhs, rhs be set; func JoinDef: lhs ++ rhs -> set equals lhs; end; infix_operator(\"++\", left, 80); theorem InfixQua: thesis proof let x, y be set; set viewed = x ++ y qua object; thus thesis; end;";
    let (handoff, _) = task271_conversion_source(source);
    let rows = handoff
        .source()
        .source_node_entries()
        .unwrap()
        .collect::<Vec<_>>();
    let qua = rows
        .iter()
        .find(|row| row.kind() == &K::QuaExpression)
        .unwrap();
    let base = rows
        .iter()
        .find(|row| row.parent() == Some(qua.site()) && row.child_ordinal() == 0)
        .unwrap();
    assert!(matches!(base.kind(), K::InfixExpression(_)));
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.spelling())
            .collect::<Vec<_>>(),
        ["x", "y"]
    );
    assert!(handoff.check_conversions().is_err());
}

#[test]
fn task271_native_conversions_refuse_genuine_legacy_bare_and_self_rhs_authority() {
    use mizar_checker::source_proof_local_declaration::{
        SourceProofLocalDeclarationError, SourceProofLocalDeclarationProducer,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::SurfaceResolvedArena,
        symbols::validate_source_proof_occurrences,
    };
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_CONVERSION_ID)
        .unwrap();
    for (index, source) in [
        "theorem BareTotality: thesis proof let s be set; reconsider s as object; thus thesis; end;",
        "theorem OwnRhs: thesis proof let s be set; now reconsider s = s as object; thus thesis; end; thus thesis; end;",
    ].into_iter().enumerate() {
        let frontend = metamorphic_guard::with_source(case, source, || super::run_frontend(&config.workspace_root, case, 0)).unwrap();
        assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
        let ast = frontend.ast.unwrap();
        let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let original = super::type_elaboration::source_syntax_receiver(&ast, &resolver.module).unwrap();
        let native = SurfaceResolvedArena::lower(&ast, &resolver.module).unwrap();
        let legacy = SourceVariableScopeResolver::resolve_proof_occurrences(SourceVariableScopeInput::new(&ast, &resolver.module, &resolver.env)).unwrap();
        validate_source_proof_occurrences(&native, &resolver.env, &legacy).unwrap();
        assert_eq!(SourceProofLocalDeclarationProducer::build_bindings(&original, &native, &resolver.env, &legacy), Err(SourceProofLocalDeclarationError::DependencyMismatch));
        let current = SourceVariableScopeResolver::resolve_proof_bindings(SourceVariableScopeInput::new(&ast, &resolver.module, &resolver.env)).unwrap();
        if index == 0 {
            assert!(legacy.references().is_empty());
            assert_eq!(current.references().len(), 1);
        } else {
            assert_eq!(legacy.references()[0].binding().index(), 1);
            assert_eq!(current.references()[0].binding().index(), 0);
        }
        assert!(SourceProofLocalDeclarationProducer::build_bindings(&original, &native, &resolver.env, &current).is_ok());
    }
}

#[test]
fn task272_native_proof_intents_execute_metadata_runtime() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let id = "pass_advanced_semantics_native_proof_intents_001";
    let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
    let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.as_ref().expect("genuine parsed proof source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let native = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &resolver.module)
        .expect("genuine source lowering");
    native.validate_against(ast, &resolver.module).unwrap();
    println!(
        "Task272 test-first: frontend, symbol collection and authentic native lowering succeeded"
    );
    let report = super::run_advanced_semantics_corpus(&config).unwrap();
    let result = report
        .results
        .iter()
        .find(|result| result.id.0 == id)
        .unwrap_or_else(|| {
            panic!(
                "proof-intent metadata case missing: {:?}",
                report.diagnostics
            )
        });
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Passed);
}

fn task272_proof_intent_source(
    text: &str,
) -> (
    mizar_checker::source_proof_local_declaration::SourceProofLocalBindingHandoff,
    mizar_resolve::env::SymbolEnv,
) {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_PROOF_INTENT_ID)
        .unwrap();
    let frontend = metamorphic_guard::with_source(case, text, || {
        super::run_frontend(&config.workspace_root, case, 0)
    })
    .unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    (
        super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap(),
        resolver.env,
    )
}

#[test]
fn task272_native_proof_intents_match_manual_original_rows_bindings_and_citations() {
    use mizar_checker::{
        binding_env::{BinderIdentity, BindingId},
        source_proof_local_declaration::SourceProofIntentStatus as S,
        typed_ast::TypedSiteRef,
    };
    use mizar_resolve::resolved_ast::LabelResolution;
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_PROOF_INTENT_ID)
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let (handoff, _) = task272_proof_intent_source(&text);
    let (intents, labels) = handoff.check_proof_intents().unwrap();
    let rows = handoff
        .source()
        .source_node_entries()
        .unwrap()
        .collect::<Vec<_>>();
    let original = |site: &TypedSiteRef| {
        let id = handoff
            .source()
            .nodes()
            .node(site.node())
            .unwrap()
            .resolved_node
            .unwrap();
        rows.iter().copied().find(|row| row.site() == id).unwrap()
    };
    let expected = [
        K::TheoremItem,
        K::ProofBlock,
        K::LetStatement,
        K::SetStatement,
        K::CompactStatement,
        K::NowStatement,
        K::CompactStatement,
        K::ThenStatement,
        K::CompactStatement,
        K::ConclusionStatement,
        K::HerebyStatement,
        K::ConclusionStatement,
    ];
    assert_eq!(intents.len(), expected.len());
    let nearest_parents = [
        None,
        Some(0),
        Some(1),
        Some(1),
        Some(1),
        Some(1),
        Some(5),
        Some(5),
        Some(7),
        Some(5),
        Some(1),
        Some(10),
    ];
    for (index, ((site, justification, status), kind)) in intents.iter().zip(expected).enumerate() {
        let row = original(site);
        assert_eq!(row.kind(), &kind);
        assert_eq!(*status, S::Pending);
        let mut parent = row.parent();
        let nearest = loop {
            let Some(id) = parent else {
                break None;
            };
            if let Some(index) = intents
                .iter()
                .position(|(site, _, _)| original(site).site() == id)
            {
                break Some(index);
            }
            parent = rows.iter().find(|row| row.site() == id).unwrap().parent();
        };
        assert_eq!(nearest, nearest_parents[index]);
        match index {
            0 => assert_eq!(
                justification.as_ref().map(original).map(|row| row.site()),
                Some(original(&intents[1].0).site())
            ),
            6 | 8 | 9 | 11 => {
                let clause = original(justification.as_ref().unwrap());
                assert_eq!(clause.kind(), &K::JustificationClause);
                assert_eq!(clause.parent(), Some(row.site()));
                let spelling = match index {
                    8 => "by B",
                    11 => "by N",
                    _ => "by A",
                };
                assert_eq!(
                    text[clause.range().start..clause.range().end].trim(),
                    spelling
                );
            }
            _ => assert!(justification.is_none()),
        }
        let original_text = text[row.range().start..row.range().end].trim_end();
        if kind == K::ProofBlock {
            assert!(original_text.ends_with("end"));
        } else {
            assert!(original_text.ends_with(';'));
        }
        if index > 0 {
            assert!(original(&intents[index - 1].0).preorder_ordinal() < row.preorder_ordinal());
        }
    }
    let before = original(&intents[6].0);
    let then = original(&intents[7].0);
    assert_eq!(before.parent(), then.parent());
    assert_eq!(before.child_ordinal() + 1, then.child_ordinal());
    assert!(before.spelling().starts_with("B :"));
    assert_eq!(
        handoff
            .scope()
            .bindings()
            .iter()
            .map(|binding| (binding.id().index(), binding.spelling()))
            .collect::<Vec<_>>(),
        [(0, "x"), (1, "x"), (2, "a")]
    );
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| (reference.id().index(), reference.binding().index()))
            .collect::<Vec<_>>(),
        [
            (0, 0),
            (1, 0),
            (2, 1),
            (3, 2),
            (4, 2),
            (5, 2),
            (6, 2),
            (7, 2),
            (8, 2),
            (9, 2),
            (10, 2),
            (11, 1),
            (12, 1)
        ]
    );
    let proof_x = &handoff.scope().bindings()[1];
    assert_eq!(
        handoff
            .binding_env()
            .bindings()
            .get(BindingId::new(2))
            .unwrap()
            .captured
            .identities(),
        [BinderIdentity::ResolverLocal {
            scope: proof_x.scope().clone(),
            ordinal: proof_x.ordinal(),
            declaration_range: proof_x.range()
        }]
    );
    assert!(labels.diagnostics().is_empty());
    assert!(!labels.has_unresolved());
    let mut declared = labels
        .index()
        .iter()
        .map(|label| label.primary_spelling())
        .collect::<Vec<_>>();
    declared.sort_unstable();
    assert_eq!(declared, ["A", "B", "N", "NativeProofIntents"]);
    assert_eq!(labels.ids().len(), 4);
    for (id, name) in labels.ids().iter().zip(["A", "B", "A", "N"]) {
        let entry = labels.table().get(*id).unwrap();
        assert_eq!(entry.site().spelling(), name);
        let LabelResolution::Resolved(target) = entry.resolution() else {
            panic!("real local citation must resolve");
        };
        let declaration = labels
            .index()
            .iter()
            .find(|label| label.origin_path() == target.origin())
            .unwrap();
        assert_eq!(declaration.primary_spelling(), name);
        let mizar_session::SourceAnchor::Range(range) = declaration.origin().anchor() else {
            panic!("original declaration range");
        };
        assert_eq!(&text[range.start..range.end], name);
        assert_eq!(target.range(), entry.site().range());
    }
    let direct = super::native_proof_intent_observation(&handoff).unwrap();
    assert_eq!(
        direct
            .iter()
            .filter(|key| key.starts_with("intent."))
            .count(),
        12
    );
    assert_eq!(
        direct
            .iter()
            .filter(|key| key.starts_with("citation."))
            .count(),
        4
    );
    assert_eq!(
        direct,
        {
            let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend.ast.unwrap();
            let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let fresh = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
            super::native_proof_intent_observation(&fresh).unwrap()
        }
    );
    assert!(handoff.source().facts().is_empty());
    assert!(handoff.source().initial_obligations().is_empty());
}

#[test]
fn task272_native_proof_intents_cover_all_twelve_rules_without_goal_acceptance() {
    use mizar_checker::source_proof_local_declaration::SourceProofIntentStatus as S;
    use mizar_syntax::SurfaceNodeKind as K;
    let prefix = "theorem Roles: for x being set holds x = x proof let x be set; A: x = x;";
    for (body, required) in [
        (
            "given g being set such that GG: g = g; consider c being set such that CC: c = c by A;",
            vec![K::GivenStatement, K::ConsiderStatement],
        ),
        (
            "E: x = x by A .= x by A;",
            vec![K::IterativeEqualityStatement, K::IterativeEqualityStep],
        ),
        (
            "per cases by A; case C1: x = x; thus x = x by C1; end; case C2: not x = x; thus x = x by A; end;",
            vec![K::CaseReasoningStatement, K::CaseItem, K::CaseItem],
        ),
        (
            "per cases; suppose S1: x = x; thus x = x by S1; end; suppose S2: not x = x; thus x = x by A; end;",
            vec![K::CaseReasoningStatement, K::SupposeItem, K::SupposeItem],
        ),
        (
            "N: now B: x = x by A; then x = x by B; thus x = x by A; end; hereby thus x = x by N; end;",
            vec![K::NowStatement, K::ThenStatement, K::HerebyStatement],
        ),
    ] {
        let source = format!("{prefix} {body} thus x = x by A; end;");
        let (handoff, _) = task272_proof_intent_source(&source);
        let (intents, labels) = handoff.check_proof_intents().unwrap();
        assert!(intents.iter().all(|(_, _, status)| *status == S::Pending));
        assert!(!labels.has_unresolved());
        let kinds = intents
            .iter()
            .map(|(site, _, _)| {
                let id = handoff
                    .source()
                    .nodes()
                    .node(site.node())
                    .unwrap()
                    .resolved_node
                    .unwrap();
                handoff
                    .source()
                    .source_node_entries()
                    .unwrap()
                    .find(|row| row.site() == id)
                    .unwrap()
                    .kind()
                    .clone()
            })
            .collect::<Vec<_>>();
        for kind in &required {
            assert_eq!(
                kinds.iter().filter(|actual| *actual == kind).count(),
                required.iter().filter(|actual| *actual == kind).count()
            );
        }
        super::native_proof_intent_observation(&handoff).unwrap();
    }
    let module = "reserve module_seed for set; set module_alias = module_seed; theorem ModuleIntent: for x being set holds x = x proof let x be set; thus x = x; end;";
    let (handoff, _) = task272_proof_intent_source(module);
    let (intents, _) = handoff.check_proof_intents().unwrap();
    assert!(intents.iter().all(|(_, _, status)| *status == S::Pending));
    assert_eq!(intents.len(), 6);
    let first = intents
        .iter()
        .take(2)
        .map(|(site, just, _)| {
            let id = handoff
                .source()
                .nodes()
                .node(site.node())
                .unwrap()
                .resolved_node
                .unwrap();
            let row = handoff
                .source()
                .source_node_entries()
                .unwrap()
                .find(|row| row.site() == id)
                .unwrap();
            assert!(just.is_none());
            row.kind().clone()
        })
        .collect::<Vec<_>>();
    assert_eq!(first, [K::StatementItem, K::SetStatement]);
    let witness = "theorem WitnessIntent: for x being set ex y being set st y = x proof let x be set; take x; thus x = x; end;";
    let (handoff, _) = task272_proof_intent_source(witness);
    assert_eq!(handoff.check_proof_intents().unwrap().0.len(), 5);
    assert!(handoff.source().types().is_empty());
    assert!(handoff.source().facts().is_empty());
    assert!(handoff.source().initial_obligations().is_empty());
}

#[test]
fn task272_native_proof_intents_then_requires_adjacent_premise_and_local_branch_header() {
    use mizar_checker::source_proof_local_declaration::SourceProofIntentStatus as S;
    use mizar_syntax::SurfaceNodeKind as K;
    for (intervening, expected) in [
        ("", S::Pending),
        ("let other be set;", S::Blocked),
        ("set other = x;", S::Blocked),
    ] {
        let source = format!(
            "theorem Adjacent: for x being set holds x = x proof let x be set; A: x = x; {intervening} then x = x by A; thus x = x by A; end;"
        );
        let (handoff, _) = task272_proof_intent_source(&source);
        let (intents, labels) = handoff.check_proof_intents().unwrap();
        assert!(!labels.has_unresolved());
        for (site, _, status) in &intents {
            let id = handoff
                .source()
                .nodes()
                .node(site.node())
                .unwrap()
                .resolved_node
                .unwrap();
            let row = handoff
                .source()
                .source_node_entries()
                .unwrap()
                .find(|row| row.site() == id)
                .unwrap();
            let is_affected = matches!(
                row.kind(),
                K::TheoremItem | K::ProofBlock | K::ThenStatement
            );
            assert_eq!(
                *status,
                if is_affected { expected } else { S::Pending },
                "{:?}",
                row.kind()
            );
        }
    }
    for branch in ["case", "suppose"] {
        let source = format!(
            "theorem Branch: for x being set holds x = x proof let x be set; per cases; {branch} C: x = x; then x = x by C; thus x = x by C; end; end;"
        );
        let (handoff, _) = task272_proof_intent_source(&source);
        let (intents, labels) = handoff.check_proof_intents().unwrap();
        assert!(intents.iter().all(|(_, _, status)| *status == S::Pending));
        assert!(!labels.has_unresolved());
    }
    let assumed = "theorem Assumed: for x being set holds x = x implies x = x proof let x be set; assume A: x = x; then x = x by A; thus x = x by A; end;";
    assert!(
        task272_proof_intent_source(assumed)
            .0
            .check_proof_intents()
            .unwrap()
            .0
            .iter()
            .all(|(_, _, status)| *status == S::Pending)
    );
}

#[test]
fn task272_native_proof_intents_block_only_actual_citation_path_and_refuse_unsupported_evidence() {
    use mizar_checker::source_proof_local_declaration::SourceProofIntentStatus as S;
    use mizar_syntax::SurfaceNodeKind as K;
    for body in [
        "A: x = x by Missing; B: x = x; thus x = x by B;",
        "A: x = x by B; B: x = x; thus x = x by B;",
        "N: now B: x = x; thus x = x by B; end; A: x = x by B; thus x = x;",
        "N: now A: x = x by N; thus x = x; end; B: x = x; thus x = x by B;",
    ] {
        let source = format!(
            "theorem CitationPath: for x being set holds x = x proof let x be set; {body} end;"
        );
        let (handoff, _) = task272_proof_intent_source(&source);
        let (intents, labels) = handoff.check_proof_intents().unwrap();
        assert!(labels.has_unresolved());
        assert_eq!(intents[0].2, S::Blocked);
        assert_eq!(intents[1].2, S::Blocked);
        for (site, _, status) in &intents {
            let id = handoff
                .source()
                .nodes()
                .node(site.node())
                .unwrap()
                .resolved_node
                .unwrap();
            let row = handoff
                .source()
                .source_node_entries()
                .unwrap()
                .find(|row| row.site() == id)
                .unwrap();
            if row.kind() == &K::LetStatement || row.spelling() == "B : x = x ;" {
                assert_eq!(*status, S::Pending);
            }
        }
        assert!(
            !super::native_proof_intent_observation(&handoff)
                .unwrap()
                .iter()
                .any(|key| key == "conversion.reconsider.narrowing_requires_proof")
        );
    }
    for body in [
        "A: x = x; A: x = x; thus x = x;",
        "A: x = x; now A: x = x; thus x = x; end; thus x = x;",
        "A: x = x; thus x = x by A[set];",
    ] {
        let source = format!(
            "theorem Unsupported: for x being set holds x = x proof let x be set; {body} end;"
        );
        let (original, native, symbols, scope) = task272_proof_intent_inputs(&source);
        assert!(mizar_checker::source_proof_local_declaration::SourceProofLocalDeclarationProducer::build_bindings(&original, &native, &symbols, &scope)
            .and_then(|handoff| handoff.check_proof_intents()).is_err());
    }
}

#[test]
fn task272_native_proof_intents_guard_exact_admission_and_actual_fs_as_consumers() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_PROOF_INTENT_ID)
        .unwrap();
    assert!(super::native_proof_intents_admitted(
        &config.workspace_root,
        case
    ));
    for mutation in 0..9 {
        let mut bad = case.clone();
        match mutation {
            0 => bad.expectation.domain = "checker.native_conversions".into(),
            1 => bad.expectation.tags.clear(),
            2 => {
                bad.expectation.expected_phase = Some(crate::expectation::PipelinePhase::TypeCheck)
            }
            3 => bad.source_path = config.workspace_root.join("tests/miz/pass/other.miz"),
            4 => bad.expectation.spec_refs.clear(),
            5 => bad.expectation.stable_detail_key = Some("unexpected".into()),
            6 => bad.expectation.tags.push("extra".into()),
            7 => {
                bad.expectation_path = config
                    .workspace_root
                    .join("tests/miz/pass/other.expect.toml")
            }
            _ => bad.expectation.id.0 = "wrong_identity".into(),
        }
        assert!(!super::native_proof_intents_admitted(
            &config.workspace_root,
            &bad
        ));
        let mut invalid = plan.clone();
        invalid.cases.push(bad);
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-ADVANCED-SEMANTICS-ADMISSION")
        );
    }
    let mut duplicate = plan.clone();
    duplicate.cases.push(case.clone());
    let mut missing = plan.clone();
    missing.cases.retain(|candidate| candidate.id != case.id);
    for invalid in [duplicate, missing] {
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|diagnostic| diagnostic.detail_key
                    == format!(
                        "advanced_semantics.inventory.{}",
                        super::NATIVE_PROOF_INTENT_ID
                    ))
        );
    }
    for source in [
        "reserve seed for set; set alias = seed;",
        "theorem BadEvidence: for x being set holds x = x proof let x be set; A: x = x; thus x = x by A[set]; end;",
    ] {
        let report = metamorphic_guard::with_source(case, source, || {
            super::run_advanced_semantics_corpus(&config)
        })
        .unwrap();
        assert_eq!(
            report
                .results
                .iter()
                .find(|result| result.id == case.id)
                .unwrap()
                .status,
            super::TypeElaborationCaseStatus::Failed
        );
    }
    for id in [
        "pass_formula_statement_consider_choice_001",
        "pass_formula_statement_given_existential_assumption_001",
        "pass_formula_statement_now_diffuse_statement_001",
        "pass_formula_statement_hereby_diffuse_conclusion_001",
        "pass_formula_statement_iterative_equality_001",
        "pass_formula_statement_per_cases_suppose_001",
        "pass_formula_statement_then_hence_linking_001",
        "pass_formula_statement_set_local_constant_take_001",
    ] {
        let fs = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        assert_eq!(
            super::formula_statement::run_formula_statement_case(&config.workspace_root, fs, 0)
                .status,
            super::FormulaStatementCaseStatus::Passed,
            "{id}"
        );
        let text = std::fs::read_to_string(&fs.source_path).unwrap();
        super::native_proof_intent_observation(&task272_proof_intent_source(&text).0).unwrap();
    }
    // Preserve the existing semantic consumer and prove its genuine unused-label bypass.
    let fs = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_set_local_constant_take_001")
        .unwrap();
    let text = std::fs::read_to_string(&fs.source_path).unwrap();
    let modified = text.replace(
        "let x be object;",
        "let x be object; Unused: x = x; Unused: x = x;",
    );
    let frontend = metamorphic_guard::with_source(fs, &modified, || {
        super::run_frontend(&config.workspace_root, fs, 0)
    })
    .unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, fs, &ast);
    assert!(
        super::source_variable_semantics_detail_keys(&ast, &resolver.module, &resolver.env)
            .is_empty()
    );
    let result = metamorphic_guard::with_source(fs, &modified, || {
        super::formula_statement::run_formula_statement_case(&config.workspace_root, fs, 0)
    });
    assert_eq!(result.status, super::FormulaStatementCaseStatus::Failed);
    assert!(
        result
            .actual_detail_keys
            .iter()
            .any(|key| key.contains("native proof intent"))
    );
}

#[test]
fn task272_native_proof_intents_consume_real_inactive_narrowing_without_status_laundering() {
    use mizar_checker::source_proof_local_declaration::SourceProofIntentStatus as S;
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let old = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_types_reconsider_omitted_justification_001")
        .unwrap();
    assert!(old.expectation.tags.is_empty());
    let text = std::fs::read_to_string(&old.source_path).unwrap();
    let frontend = super::run_frontend(&config.workspace_root, old, 0).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, old, &ast);
    assert!(resolver.detail_keys.is_empty());
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    let (intents, _) = handoff.check_proof_intents().unwrap();
    let mut registration_proof = false;
    for (site, _, status) in &intents {
        let id = handoff
            .source()
            .nodes()
            .node(site.node())
            .unwrap()
            .resolved_node
            .unwrap();
        let row = handoff
            .source()
            .source_node_entries()
            .unwrap()
            .find(|row| row.site() == id)
            .unwrap();
        if row.kind() == &K::TheoremItem
            || row.kind() == &K::ReconsiderStatement
            || row.kind() == &K::ReconsiderItem
        {
            assert_eq!(*status, S::Blocked);
        }
        if row.kind() == &K::ProofBlock && row.spelling().contains("reconsider") {
            assert_eq!(*status, S::Blocked);
        }
        if row.kind() == &K::ProofBlock && !row.spelling().contains("reconsider") {
            assert_eq!(*status, S::Pending);
            registration_proof = true;
        }
    }
    assert!(registration_proof);
    assert_eq!(
        super::native_proof_intent_observation(&handoff)
            .unwrap()
            .iter()
            .filter(|key| key.as_str() == "conversion.reconsider.narrowing_requires_proof")
            .count(),
        1
    );
    assert_eq!(
        old.expectation.stable_detail_key.as_deref(),
        Some("conversion.reconsider.narrowing_requires_proof")
    );
    assert!(
        !super::run_advanced_semantics_corpus(&config)
            .unwrap()
            .results
            .iter()
            .any(|result| result.id == old.id)
    );
    let prepared = super::run_advanced_semantics_case(&config.workspace_root, old, 0);
    assert_eq!(prepared.status, super::TypeElaborationCaseStatus::Passed);
    assert_eq!(
        prepared.actual_detail_keys,
        ["conversion.reconsider.narrowing_requires_proof"]
    );
    let unrelated = text.replace(
        "reconsider k = s as keen set;",
        "reconsider k = s as object; A: s = s by Unknown;",
    );
    let (handoff, _) = task272_proof_intent_source(&unrelated);
    assert!(
        handoff
            .check_proof_intents()
            .unwrap()
            .0
            .iter()
            .any(|(_, _, status)| *status == S::Blocked)
    );
    assert!(
        !super::native_proof_intent_observation(&handoff)
            .unwrap()
            .iter()
            .any(|key| key == "conversion.reconsider.narrowing_requires_proof")
    );
    let pending = text.replace(
        "reconsider k = s as keen set;",
        "A: s = s; reconsider k = s as keen set by A;",
    );
    let (handoff, _) = task272_proof_intent_source(&pending);
    assert!(
        handoff
            .check_proof_intents()
            .unwrap()
            .0
            .iter()
            .all(|(_, _, status)| *status == S::Pending)
    );
    assert!(
        !super::native_proof_intent_observation(&handoff)
            .unwrap()
            .iter()
            .any(|key| key == "conversion.reconsider.narrowing_requires_proof")
    );
}

#[test]
fn task272_native_proof_intents_preserve_layout_and_names_without_derived_oracles() {
    use mizar_checker::source_proof_local_declaration::SourceProofIntentStatus as S;
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_proof_intents_001.miz"
    );
    for source in [
        format!(":: layout\n{}", base.replace("let x be", "let\nx\tbe")),
        base.replace("NativeProofIntents", "RenamedIntents")
            .replace("x", "seed")
            .replace("a", "alias")
            .replace("A:", "Start:")
            .replace("by A", "by Start"),
    ] {
        let (handoff, _) = task272_proof_intent_source(&source);
        let (intents, labels) = handoff.check_proof_intents().unwrap();
        assert_eq!(intents.len(), 12);
        assert!(intents.iter().all(|(_, _, status)| *status == S::Pending));
        assert_eq!(labels.ids().len(), 4);
        assert!(!labels.has_unresolved());
        assert_eq!(
            handoff
                .scope()
                .references()
                .iter()
                .map(|reference| reference.binding().index())
                .collect::<Vec<_>>(),
            [0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1]
        );
        super::native_proof_intent_observation(&handoff).unwrap();
    }
}

fn task272_proof_intent_inputs(
    source: &str,
) -> (
    mizar_checker::typed_ast::TypedAst,
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
    mizar_resolve::env::SymbolEnv,
    mizar_resolve::names::ResolvedVariableScope,
) {
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::SurfaceResolvedArena,
    };
    let (ast, module, _, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(source, 2_724_001);
    assert_eq!(diagnostics, 0);
    let original = super::type_elaboration::source_syntax_receiver(&ast, &module).unwrap();
    let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let scope = SourceVariableScopeResolver::resolve_proof_bindings(SourceVariableScopeInput::new(
        &ast, &module, &symbols,
    ))
    .unwrap();
    (original, native, symbols, scope)
}

#[test]
fn task272_native_proof_intents_refuse_foreign_roles_and_complete_legacy_case_omission() {
    use mizar_checker::source_proof_local_declaration::{
        SourceProofLocalDeclarationError, SourceProofLocalDeclarationProducer,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::validate_source_proof_occurrences,
    };
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_proof_intents_001.miz"
    );
    let (original, native, symbols, current) = task272_proof_intent_inputs(base);
    SourceProofLocalDeclarationProducer::build_bindings(&original, &native, &symbols, &current)
        .unwrap()
        .check_proof_intents()
        .unwrap();
    for foreign in [
        base.replace("then a = a by B;", "then x = x by B;"),
        base.replace("N: now", "M: now").replace("by N", "by M"),
        base.replace("A: a = a;", "A: x = x;"),
        base.replace("B: a = a by A;", "B: a = a by A; set intervening = a;"),
    ] {
        let (coherent, native, symbols, scope) = task272_proof_intent_inputs(&foreign);
        assert_eq!(coherent.source_id(), original.source_id());
        assert_eq!(coherent.module_id(), original.module_id());
        SourceProofLocalDeclarationProducer::build_bindings(&coherent, &native, &symbols, &scope)
            .unwrap()
            .check_proof_intents()
            .unwrap();
        assert_eq!(
            SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &scope
            ),
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        );
    }
    let (ast, module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(base, 2_724_001);
    assert_eq!(diagnostics, 0);
    let legacy = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &module, &symbols),
    )
    .unwrap();
    validate_source_proof_occurrences(&native, &symbols, &legacy).unwrap();
    assert_eq!(legacy, current);
    SourceProofLocalDeclarationProducer::build_bindings(&original, &native, &symbols, &legacy)
        .unwrap()
        .check_proof_intents()
        .unwrap();

    let source = "theorem CaseProfile: for x being set holds x = x proof let x be set; A: x = x; per cases; case C: x = x; thus x = x by C; end; thus x = x by A; end;";
    let (ast, module, _, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(source, 2_724_001);
    assert_eq!(diagnostics, 0);
    let native = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .unwrap();
    let legacy = mizar_resolve::labels::ProofLabelSourceCollector::new(
        &ast,
        &module,
        namespace,
        owner.contribution(),
        &native,
    )
    .unwrap()
    .collect_with_proof_organization()
    .unwrap();
    assert_eq!(
        legacy
            .projections()
            .iter()
            .map(|entry| entry.primary_spelling())
            .collect::<Vec<_>>(),
        ["A"]
    );
    assert_eq!(
        legacy
            .references()
            .iter()
            .map(|entry| entry.site().spelling())
            .collect::<Vec<_>>(),
        ["A"]
    );
    let (handoff, _) = task272_proof_intent_source(source);
    let (_, labels) = handoff.check_proof_intents().unwrap();
    assert!(!labels.has_unresolved());
    assert_eq!(labels.ids().len(), 2);
    for (id, spelling) in labels.ids().iter().zip(["C", "A"]) {
        let entry = labels.table().get(*id).unwrap();
        assert_eq!(entry.site().spelling(), spelling);
        let mizar_resolve::resolved_ast::LabelResolution::Resolved(target) = entry.resolution()
        else {
            panic!("authentic native Case citation");
        };
        assert_eq!(
            labels
                .index()
                .iter()
                .find(|entry| entry.origin_path() == target.origin())
                .unwrap()
                .primary_spelling(),
            spelling
        );
    }
}

#[test]
fn task272_native_proof_intents_preserve_annotated_compact_then_and_now_ownership() {
    use mizar_checker::source_proof_local_declaration::SourceProofIntentStatus as S;
    use mizar_syntax::SurfaceNodeKind as K;
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_proof_intents_001.miz"
    );
    for (compact, now, then) in [
        ("@trace(\"compact\")", "@trace(\"now\")", "@trace(\"then\")"),
        ("@[simp]", "@[simp]", "@[simp]"),
    ] {
        let source = base
            .replace("A: a = a;", &format!("{compact} A: a = a;"))
            .replace("N: now", &format!("{now} N: now"))
            .replace("then a = a by B;", &format!("{then} then a = a by B;"));

        let (handoff, _) = task272_proof_intent_source(&source);
        let rows = handoff
            .source()
            .source_node_entries()
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(
            rows.iter()
                .filter(|row| row.kind() == &K::AnnotatedStatement)
                .count(),
            3
        );
        let (intents, labels) = handoff.check_proof_intents().unwrap();
        assert_eq!(intents.len(), 12);
        assert!(intents.iter().all(|(_, _, status)| *status == S::Pending));
        assert!(!labels.has_unresolved());
        assert_eq!(
            labels
                .ids()
                .iter()
                .map(|id| labels.table().get(*id).unwrap().site().spelling())
                .collect::<Vec<_>>(),
            ["A", "B", "A", "N"]
        );
        for ordinal in [4, 5, 7] {
            let node = handoff
                .source()
                .nodes()
                .node(intents[ordinal].0.node())
                .unwrap()
                .resolved_node
                .unwrap();
            let row = rows.iter().find(|row| row.site() == node).unwrap();
            let parent = rows
                .iter()
                .find(|candidate| Some(candidate.site()) == row.parent())
                .unwrap();
            assert_eq!(parent.kind(), &K::AnnotatedStatement);
            assert!(intents[ordinal].1.is_none());
        }
        assert_eq!(
            handoff
                .scope()
                .references()
                .iter()
                .map(|reference| reference.binding().index())
                .collect::<Vec<_>>(),
            [0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1]
        );
        let observed = super::native_proof_intent_observation(&handoff).unwrap();
        assert!(
            observed
                .iter()
                .find(|key| key.starts_with("then.7."))
                .unwrap()
                .contains("previous=Some((CompactStatement,")
        );
    }
}

#[test]
fn task272b1_native_witness_goals_execute_metadata_runtime() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let id = "pass_advanced_semantics_native_witness_goals_001";
    let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
    let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend
        .ast
        .as_ref()
        .expect("genuine parsed witness source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let native = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &resolver.module)
        .expect("genuine witness source lowering");
    native.validate_against(ast, &resolver.module).unwrap();
    println!(
        "Task272B1 test-first: frontend, symbol collection and authentic native lowering succeeded"
    );
    let report = super::run_advanced_semantics_corpus(&config).unwrap();
    let result = report
        .results
        .iter()
        .find(|result| result.id.0 == id)
        .unwrap_or_else(|| panic!("witness-goal case missing: {:?}", report.diagnostics));
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Passed);
}

fn task272b1_witness_source(
    source: &str,
) -> mizar_checker::source_proof_local_declaration::SourceProofLocalBindingHandoff {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_WITNESS_GOAL_ID)
        .unwrap();
    let frontend = metamorphic_guard::with_source(case, source, || {
        super::run_frontend(&config.workspace_root, case, 0)
    })
    .unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap()
}

fn task272b1_binding(
    handoff: &mizar_checker::source_proof_local_declaration::SourceProofLocalBindingHandoff,
    index: usize,
) -> mizar_checker::binding_env::BindingId {
    use mizar_checker::binding_env::BinderIdentity;
    let binding = &handoff.scope().bindings()[index];
    handoff
        .binding_env()
        .bindings()
        .iter()
        .find(|(_, actual)| {
            actual.identity
                == BinderIdentity::ResolverLocal {
                    scope: binding.scope().clone(),
                    ordinal: binding.ordinal(),
                    declaration_range: binding.range(),
                }
        })
        .unwrap()
        .0
}

fn task272b1_goal_text(
    goal: &mizar_checker::source_proof_local_declaration::SourceProofGoal,
) -> String {
    use mizar_checker::source_proof_local_declaration::SourceProofGoal as G;
    match goal {
        G::Equality { left, right, .. } => format!("eq({},{})", left.index(), right.index()),
        G::Membership { left, right, .. } => format!("member({},{})", left.index(), right.index()),
        G::Disjunction { left, right, .. } => format!(
            "or({},{})",
            task272b1_goal_text(left),
            task272b1_goal_text(right)
        ),
        G::Negation { body, .. } => format!("not({})", task272b1_goal_text(body)),
        G::Conjunction { left, right, .. } => format!(
            "and({},{})",
            task272b1_goal_text(left),
            task272b1_goal_text(right)
        ),
        G::Implication {
            antecedent,
            consequent,
            ..
        } => format!(
            "implies({},{})",
            task272b1_goal_text(antecedent),
            task272b1_goal_text(consequent)
        ),
        G::Universal {
            binding,
            radix,
            body,
            ..
        } => format!(
            "forall({}:{radix:?},{})",
            binding.index(),
            task272b1_goal_text(body)
        ),
        G::Existential {
            binding,
            radix,
            body,
            ..
        } => format!(
            "exists({}:{radix:?},{})",
            binding.index(),
            task272b1_goal_text(body)
        ),
        _ => panic!("unsupported derived goal"),
    }
}

#[test]
fn task272b1_native_witness_goals_match_manual_trees_steps_types_and_guards() {
    use mizar_checker::{
        type_checker::{
            CoercionEvidence, CoercionJustification, CoercionRequestKind, TermReference,
            TypeHeadInput,
        },
        typed_ast::{CoercionStatus, FactStatus, TypedSiteRef},
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let source = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_witness_goals_001.miz"
    );
    let handoff = task272b1_witness_source(source);
    assert_eq!(
        handoff
            .scope()
            .bindings()
            .iter()
            .map(|binding| binding.spelling())
            .collect::<Vec<_>>(),
        ["s", "o", "a", "b", "s", "o", "k"]
    );
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [2, 0, 3, 1, 4, 5]
    );
    let [s0, o1, a2, b3, s4, o5, k6] =
        std::array::from_fn::<_, 7, _>(|index| task272b1_binding(&handoff, index));
    let (s, o, a, b, x, y) = (
        s0.index(),
        o1.index(),
        a2.index(),
        b3.index(),
        s4.index(),
        o5.index(),
    );
    let g0 = format!(
        "forall({s}:Set,forall({o}:Object,exists({a}:Set,exists({b}:Object,and(eq({a},{s}),eq({b},{o}))))))"
    );
    let g1 = format!(
        "forall({o}:Object,exists({a}:Set,exists({b}:Object,and(eq({a},{x}),eq({b},{o})))))"
    );
    let g2 = format!("exists({a}:Set,exists({b}:Object,and(eq({a},{x}),eq({b},{y}))))");
    let g3 = format!("exists({b}:Object,and(eq({x},{x}),eq({b},{y})))");
    let g4 = format!("and(eq({x},{x}),eq({y},{y}))");
    let expected = [
        (&g0, &g0),
        (&g0, &g1),
        (&g1, &g2),
        (&g2, &g3),
        (&g3, &g4),
        (&g4, &g4),
    ];
    let (steps, inputs, pending, checked, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 6);
    assert_eq!(
        steps.iter().map(|step| step.matched()).collect::<Vec<_>>(),
        [
            None,
            Some((s0, s4)),
            Some((o1, o5)),
            Some((a2, s4)),
            Some((b3, o5)),
            None
        ]
    );
    let expected_free = [
        vec![],
        vec![s4],
        vec![s4, o5],
        vec![s4, o5],
        vec![s4, o5],
        vec![s4, o5],
    ];
    let expected_avoided = [
        vec![],
        vec![o1, a2, b3],
        vec![a2, b3],
        vec![b3],
        vec![],
        vec![],
    ];
    let rows = handoff
        .source()
        .source_node_entries()
        .unwrap()
        .collect::<Vec<_>>();
    let original = |site: &TypedSiteRef| {
        let id = handoff
            .source()
            .nodes()
            .node(site.node())
            .unwrap()
            .resolved_node
            .unwrap();
        rows.iter().copied().find(|row| row.site() == id).unwrap()
    };
    for (index, step) in steps.iter().enumerate() {
        assert_eq!(task272b1_goal_text(step.before()), *expected[index].0);
        assert_eq!(task272b1_goal_text(step.after()), *expected[index].1);
        assert_eq!(step.free(), expected_free[index]);
        assert_eq!(step.avoided(), expected_avoided[index]);
        assert_eq!(step.owner(), steps[0].site());
        assert_eq!(step.context(), steps[0].context());
        let row = original(step.site());
        match index {
            0 => assert_eq!(row.kind(), &K::TheoremItem),
            1 | 2 => {
                assert!(matches!(row.kind(), K::Token(_)));
                assert_eq!(row.range(), handoff.scope().bindings()[index + 3].range());
            }
            3 | 4 => assert_eq!(row.kind(), &K::Witness),
            5 => assert_eq!(row.kind(), &K::ConclusionStatement),
            _ => unreachable!(),
        }
        assert!(!step.free().contains(&k6));
    }
    assert_eq!(inputs.len(), 2);
    assert!(pending.is_empty());
    assert!(checked.initial_obligations().is_empty());
    for (input, head) in inputs
        .iter()
        .zip([TypeHeadInput::BuiltinSet, TypeHeadInput::BuiltinObject])
    {
        assert_eq!(input.kind, CoercionRequestKind::Widening);
        assert_eq!(input.justification, CoercionJustification::Omitted);
        assert_eq!(input.evidence, CoercionEvidence::BuiltinRadix);
        assert_eq!(input.from_type.as_ref().unwrap().head, head);
        assert_eq!(input.to_type.head, head);
        let step = steps
            .iter()
            .find(|step| step.site() == &input.site)
            .unwrap();
        assert_eq!(input.source_range, original(step.site()).range());
        let result = checked
            .coercions()
            .iter()
            .find(|(_, result)| result.site == input.site)
            .unwrap()
            .1;
        assert_eq!(result.status, CoercionStatus::Candidate);
        assert!(result.obligation.is_none());
        assert!(
            checked
                .facts()
                .iter()
                .any(|(_, fact)| fact.subject == input.site && fact.status == FactStatus::Known)
        );
    }
    assert!(checked.diagnostics().is_empty());
    for reference in handoff.scope().references() {
        let binding = task272b1_binding(&handoff, reference.binding().index());
        assert!(inference.terms().iter().any(|(_, term)| term.reference
            == Some(TermReference::Binding(binding))
            && original(&term.site).range() == reference.range()));
    }
    for binding in [s4, o5] {
        assert!(inference.terms().iter().any(|(_,term)| term.reference==Some(TermReference::Binding(binding)) && matches!(&term.site,TypedSiteRef::Role {node,role} if role.as_str()=="witness-goal.declaration-value" && handoff.source().nodes().node(*node).unwrap().resolved_node.is_some_and(|id| rows.iter().any(|row| row.site()==id && row.range()==handoff.binding_env().bindings().get(binding).unwrap().declaration_range)))));
    }
    let observed = super::native_witness_goal_observation(&handoff).unwrap();
    assert_eq!(
        observed
            .iter()
            .filter(|key| key.starts_with("step."))
            .count(),
        6
    );
    assert_eq!(
        observed
            .iter()
            .filter(|key| key.starts_with("guard."))
            .count(),
        2
    );
    assert!(
        observed
            .iter()
            .any(|key| key.starts_with("remaining.") && key.ends_with("pending=[]"))
    );
    assert!(handoff.source().types().is_empty());
    assert!(handoff.source().facts().is_empty());
    assert!(handoff.source().initial_obligations().is_empty());
}

#[test]
fn task272b1_native_witness_goals_keep_pending_guard_and_changed_body_in_order() {
    use mizar_checker::{
        type_checker::{InitialRequirementKind, TypeHeadInput},
        typed_ast::{CoercionStatus, FactStatus, InitialObligationStatus},
    };
    let source = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_witness_goals_001.miz"
    )
    .replace("take k = s, o;", "take k = o, s;");
    let handoff = task272b1_witness_source(&source);
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [2, 0, 3, 1, 5, 4]
    );
    let (steps, success, pending, checked, _) = handoff.check_witness_goals().unwrap();
    let (a, b, s, o) = (
        task272b1_binding(&handoff, 2),
        task272b1_binding(&handoff, 3),
        task272b1_binding(&handoff, 4),
        task272b1_binding(&handoff, 5),
    );
    assert_eq!(steps[3].matched(), Some((a, o)));
    assert_eq!(steps[4].matched(), Some((b, s)));
    assert_eq!(
        task272b1_goal_text(steps[3].after()),
        format!(
            "exists({}:Object,and(eq({},{}),eq({},{})))",
            b.index(),
            o.index(),
            s.index(),
            b.index(),
            o.index()
        )
    );
    let body = format!(
        "and(eq({},{}),eq({},{}))",
        o.index(),
        s.index(),
        s.index(),
        o.index()
    );
    assert_eq!(task272b1_goal_text(steps[4].after()), body);
    assert_eq!(steps[5].before(), steps[4].after());
    assert_eq!(steps[5].after(), steps[4].after());
    assert_eq!(
        (
            success.len(),
            pending.len(),
            checked.coercions().len(),
            checked.initial_obligations().len()
        ),
        (1, 1, 1, 1)
    );
    assert_eq!(pending[0].site, *steps[3].site());
    assert_eq!(pending[0].kind, InitialRequirementKind::Narrowing);
    assert_eq!(pending[0].target_type.head, TypeHeadInput::BuiltinSet);
    assert_eq!(success[0].site, *steps[4].site());
    assert_eq!(
        success[0].from_type.as_ref().unwrap().head,
        TypeHeadInput::BuiltinSet
    );
    assert_eq!(success[0].to_type.head, TypeHeadInput::BuiltinObject);
    assert_eq!(
        checked.coercions().iter().next().unwrap().1.status,
        CoercionStatus::Candidate
    );
    let obligation = checked.initial_obligations().iter().next().unwrap().1;
    assert_eq!(obligation.owner, *steps[3].site());
    assert_eq!(obligation.status, InitialObligationStatus::Pending);
    assert!(
        checked
            .facts()
            .iter()
            .any(|(_, fact)| fact.status == FactStatus::PendingObligation
                && !fact.status.is_unconditionally_consumable())
    );
    assert!(checked.diagnostics().is_empty());
    let observed = super::native_witness_goal_observation(&handoff).unwrap();
    assert!(
        observed
            .iter()
            .any(|key| key.starts_with("guard.0.") && key.contains("status=Pending"))
    );
    assert!(observed.iter().any(|key| key.starts_with("remaining.")
        && key.contains(&format!("pending=[{:?}]", obligation.id))));
}

#[test]
fn task272b1_native_witness_goals_cover_lists_segments_unused_and_retained_binders() {
    use mizar_checker::type_checker::TermReference;
    use mizar_checker::typed_ast::TypedSiteRef;
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_witness_goals_001.miz"
    );
    for source in [
        base.replace(
            "let s be set;\n  let o be object;",
            "let s be set, o be object;",
        ),
        base.replace("take k = s, o;", "take k = s; take o;"),
        base.replace("take k = s, o;", "take s, o;"),
        base.replace("thus thesis;", "thus s = s & o = o;"),
        base.replace("thus thesis;", "thus k = s & o = o;"),
    ] {
        let handoff = task272b1_witness_source(&source);
        let (steps, success, pending, _, _) = handoff.check_witness_goals().unwrap();
        assert_eq!(steps.len(), 6);
        assert_eq!(success.len(), 2);
        assert!(pending.is_empty());
        let (s, o) = (
            task272b1_binding(&handoff, 4),
            task272b1_binding(&handoff, 5),
        );
        assert_eq!(
            task272b1_goal_text(steps[5].after()),
            format!("and(eq({0},{0}),eq({1},{1}))", s.index(), o.index())
        );
        super::native_witness_goal_observation(&handoff).unwrap();
    }
    let multi = "theorem Multi: for s, t being set ex a, b being set st a = s & b = t proof let s, t be set; take s, t; thus thesis; end;";
    let handoff = task272b1_witness_source(multi);
    let (steps, success, pending, _, _) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 6);
    assert_eq!(success.len(), 2);
    assert!(pending.is_empty());
    assert_eq!(
        steps[1].matched(),
        Some((
            task272b1_binding(&handoff, 0),
            task272b1_binding(&handoff, 4)
        ))
    );
    assert_eq!(
        steps[2].matched(),
        Some((
            task272b1_binding(&handoff, 1),
            task272b1_binding(&handoff, 5)
        ))
    );
    assert_ne!(steps[1].site(), steps[2].site());
    let unused = "theorem Unused: for unused being object, s being set ex a being set st a = s proof let unused be object; let s be set; take s; thus thesis; end;";
    let handoff = task272b1_witness_source(unused);
    let (steps, _, _, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [2, 1, 4]
    );
    let local = task272b1_binding(&handoff, 3);
    assert_eq!(
        steps[1].matched(),
        Some((task272b1_binding(&handoff, 0), local))
    );
    assert!(steps[1].free().is_empty());
    assert_eq!(
        steps[1].avoided(),
        [
            task272b1_binding(&handoff, 1),
            task272b1_binding(&handoff, 2)
        ]
    );
    assert!(inference.terms().iter().any(|(_, term)| term.reference
        == Some(TermReference::Binding(local))
        && matches!(term.site, TypedSiteRef::Role { .. })));
    super::native_witness_goal_observation(&handoff).unwrap();
    for terminal in ["thesis", "for z being set holds y = z"] {
        let source = format!(
            "theorem Retained: for y being set ex x being set st for y being set holds x = y proof let y be set; take y; thus {terminal}; end;"
        );
        let handoff = task272b1_witness_source(&source);
        let (steps, _, _, _, _) = handoff.check_witness_goals().unwrap();
        assert_eq!(steps.len(), 4);
        let (inner, local) = (
            task272b1_binding(&handoff, 2),
            task272b1_binding(&handoff, 3),
        );
        assert_eq!(
            task272b1_goal_text(steps[2].after()),
            format!(
                "forall({}:Set,eq({},{}))",
                inner.index(),
                local.index(),
                inner.index()
            )
        );
        assert_eq!(steps[2].free(), [local]);
        assert_eq!(steps[2].avoided(), [inner]);
        assert!(steps[3].avoided().is_empty());
        assert_eq!(steps[3].before(), steps[3].after());
        super::native_witness_goal_observation(&handoff).unwrap();
    }
    for body in ["thus thesis;", "let x be set; thus thesis;"] {
        let source = format!(
            "theorem Unfinished: for x being set ex y being set st y = x proof {body} end;"
        );
        let handoff = task272b1_witness_source(&source);
        let (steps, success, pending, _, _) = handoff.check_witness_goals().unwrap();
        assert_eq!(steps.len(), if body.starts_with("let") { 3 } else { 2 });
        assert!(success.is_empty());
        assert!(pending.is_empty());
        assert_eq!(
            steps.last().unwrap().before(),
            steps.last().unwrap().after()
        );
        assert!(task272b1_goal_text(steps.last().unwrap().after()).contains("exists("));
        super::native_witness_goal_observation(&handoff).unwrap();
    }
}

#[test]
fn task272b1_native_witness_goals_refuse_wrong_events_terminal_and_foreign_authority() {
    use mizar_checker::source_proof_local_declaration::{
        SourceProofLocalDeclarationError, SourceProofLocalDeclarationProducer,
    };
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_witness_goals_001.miz"
    );
    for source in [
        base.replace("let o be object;", "let o be set;"),
        base.replace("let o be object;\n  take k = s, o;", "take k = s; let o be object; take o;"),
        base.replace("take k = s, o;", "take k = s, o, s;"),
        base.replace("thus thesis;", "thus s = o & o = s;"),
        "theorem NotExistential: for s being set holds s = s proof let s be set; take s; thus thesis; end;".to_owned(),
        "theorem WrongAlpha: for y being set ex x being set st for y being set holds x = y proof let y be set; take y; thus for y being set holds y = y; end;".to_owned(),
        "theorem WrongGuard: for y being set ex x being set st for y being set holds x = y proof let y be set; take y; thus for z being object holds y = z; end;".to_owned(),
    ] { assert!(task272b1_witness_source(&source).check_witness_goals().is_err(),"{source}"); }
    let (original, _, _, _) = task272_proof_intent_inputs(base);
    for source in [
        base.replace("take k = s, o;", "take k = o, s;"),
        base.replace("a = s & b = o", "a = o & b = s"),
        base.replace("let s be set;", "let s be object;"),
        base.replace("thus thesis;", "thus s = s & o = o;"),
    ] {
        let (coherent, native, symbols, scope) = task272_proof_intent_inputs(&source);
        assert_eq!(coherent.source_id(), original.source_id());
        assert_eq!(coherent.module_id(), original.module_id());
        assert!(
            SourceProofLocalDeclarationProducer::build_bindings(
                &coherent, &native, &symbols, &scope
            )
            .is_ok()
        );
        assert!(matches!(
            SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &scope
            ),
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        ));
    }
}

#[test]
fn task272b1_native_witness_goals_keep_owner_local_pending_and_real_fs_consumer() {
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_witness_goals_001.miz"
    );
    let source = format!(
        "{}\n{}",
        base.replace("NativeWitnessGoals", "FirstWitnessGoals")
            .replace("take k = s, o;", "take k = o, s;"),
        base.replace("NativeWitnessGoals", "SecondWitnessGoals")
    );
    let handoff = task272b1_witness_source(&source);
    let (steps, success, pending, checked, _) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 12);
    assert_ne!(steps[0].owner(), steps[6].owner());
    assert_ne!(steps[0].context(), steps[6].context());
    assert_eq!(
        (
            success.len(),
            pending.len(),
            checked.initial_obligations().len()
        ),
        (3, 1, 1)
    );
    assert_eq!(pending[0].site, *steps[3].site());
    let observed = super::native_witness_goal_observation(&handoff).unwrap();
    let remaining = observed
        .iter()
        .filter(|key| key.starts_with("remaining."))
        .collect::<Vec<_>>();
    assert_eq!(remaining.len(), 2);
    assert!(!remaining[0].ends_with("pending=[]"));
    assert!(remaining[1].ends_with("pending=[]"));
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let fs = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_existential_multi_witness_001")
        .unwrap();
    assert_eq!(
        super::formula_statement::run_formula_statement_case(&config.workspace_root, fs, 0).status,
        super::FormulaStatementCaseStatus::Passed
    );
    let frontend = super::run_frontend(&config.workspace_root, fs, 0).unwrap();
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, fs, &ast);
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    let (steps, success, pending, _, _) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 5);
    assert_eq!(success.len(), 2);
    assert!(pending.is_empty());
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [1, 0, 2, 0, 3, 3, 3, 3, 3, 3]
    );
    let local = task272b1_binding(&handoff, 3);
    assert_eq!(
        task272b1_goal_text(steps[4].after()),
        format!("and(eq({0},{0}),eq({0},{0}))", local.index())
    );
    let text = std::fs::read_to_string(&fs.source_path).unwrap();
    let modified = text.replace("theorem ExMulti1:", "open theorem ExMulti1:");
    let frontend = metamorphic_guard::with_source(fs, &modified, || {
        super::run_frontend(&config.workspace_root, fs, 0)
    })
    .unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, fs, &ast);
    assert!(
        super::formula_statement::check_formula_ast_with_organization(
            &ast,
            &resolver.module,
            &resolver.env,
            false
        )
        .unwrap()
    );
    let result = metamorphic_guard::with_source(fs, &modified, || {
        super::formula_statement::run_formula_statement_case(&config.workspace_root, fs, 0)
    });
    assert_eq!(result.status, super::FormulaStatementCaseStatus::Failed);
    assert!(
        result
            .actual_detail_keys
            .iter()
            .any(|key| key.contains("native witness goal"))
    );
}

#[test]
fn task272b1_native_witness_goals_guard_exact_admission_and_actual_as_paths() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_WITNESS_GOAL_ID)
        .unwrap();
    assert!(super::native_witness_goals_admitted(
        &config.workspace_root,
        case
    ));
    for mutation in 0..9 {
        let mut bad = case.clone();
        match mutation {
            0 => bad.expectation.domain = "checker.native_proof_intents".into(),
            1 => bad.expectation.tags.clear(),
            2 => {
                bad.expectation.expected_phase = Some(crate::expectation::PipelinePhase::TypeCheck)
            }
            3 => bad.source_path = config.workspace_root.join("tests/miz/pass/other.miz"),
            4 => bad.expectation.spec_refs.clear(),
            5 => bad.expectation.stable_detail_key = Some("unexpected".into()),
            6 => bad.expectation.tags.push("extra".into()),
            7 => {
                bad.expectation_path = config
                    .workspace_root
                    .join("tests/miz/pass/other.expect.toml")
            }
            _ => bad.expectation.id.0 = "wrong_identity".into(),
        };
        assert!(!super::native_witness_goals_admitted(
            &config.workspace_root,
            &bad
        ));
        let mut invalid = plan.clone();
        invalid.cases.push(bad);
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-ADVANCED-SEMANTICS-ADMISSION")
        );
    }
    let mut duplicate = plan.clone();
    duplicate.cases.push(case.clone());
    let mut missing = plan.clone();
    missing.cases.retain(|candidate| candidate.id != case.id);
    for invalid in [duplicate, missing] {
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|diagnostic| diagnostic.detail_key
                    == format!(
                        "advanced_semantics.inventory.{}",
                        super::NATIVE_WITNESS_GOAL_ID
                    ))
        );
    }
    for source in ["reserve seed for set; set alias = seed;".to_owned(),include_str!("../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_witness_goals_001.miz").replace("thus thesis;","thus s = o & o = s;")] {
        let report=metamorphic_guard::with_source(case,&source,||super::run_advanced_semantics_corpus(&config)).unwrap();assert_eq!(report.results.iter().find(|result|result.id==case.id).unwrap().status,super::TypeElaborationCaseStatus::Failed);
    }
}

#[test]
fn task272b2_blocked_goals_execute_whole_inactive_case() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let old = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_types_reconsider_omitted_justification_001")
        .unwrap();
    assert!(old.expectation.tags.is_empty());
    let frontend = super::run_frontend(&config.workspace_root, old, 0).unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.expect("genuine whole old source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root, old, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    println!(
        "Task272B2 test-first: authentic unchanged old source, symbols, native transaction and Typed/Resolved replay succeeded"
    );
    let (steps, _, _, _, _) = handoff.check_witness_goals().expect(
        "actual whole inactive case must return typed conversion checkpoint and remaining goal",
    );
    assert_eq!(steps.len(), 4);
}

fn task272b2_old_source_handoff(
    source: &str,
) -> mizar_checker::source_proof_local_declaration::SourceProofLocalBindingHandoff {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let old = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_types_reconsider_omitted_justification_001")
        .unwrap();
    let frontend = metamorphic_guard::with_source(old, source, || {
        super::run_frontend(&config.workspace_root, old, 0)
    })
    .unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, old, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap()
}

#[test]
fn task272b2_blocked_goals_match_manual_checkpoint_owner_and_type_oracles() {
    use mizar_checker::{
        source_proof_local_declaration::SourceProofIntentStatus as S,
        type_checker::{FormulaKind, TermReference, TermStatus, TypeHeadRef},
        typed_ast::{TypeEntryActual, TypeStatus},
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let old = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_types_reconsider_omitted_justification_001")
        .unwrap();
    let source = std::fs::read_to_string(&old.source_path).unwrap();
    let handoff = task272b2_old_source_handoff(&source);
    assert_eq!(
        handoff
            .scope()
            .bindings()
            .iter()
            .map(|binding| binding.spelling())
            .collect::<Vec<_>>(),
        ["s", "s", "k"]
    );
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [0, 0, 1]
    );
    let (h, p, k) = (
        task272b1_binding(&handoff, 0),
        task272b1_binding(&handoff, 1),
        task272b1_binding(&handoff, 2),
    );
    let (steps, witness_inputs, witness_pending, witness_checked, inference) =
        handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 4);
    assert!(witness_inputs.is_empty());
    assert!(witness_pending.is_empty());
    assert!(witness_checked.coercions().is_empty());
    assert!(witness_checked.initial_obligations().is_empty());
    let initial = format!("forall({0}:Set,eq({0},{0}))", h.index());
    let remaining = format!("eq({0},{0})", p.index());
    assert_eq!(
        steps.iter().map(|step| step.matched()).collect::<Vec<_>>(),
        [None, Some((h, p)), None, None]
    );
    let rows = handoff
        .source()
        .source_node_entries()
        .unwrap()
        .collect::<Vec<_>>();
    let original = |site: &mizar_checker::typed_ast::TypedSiteRef| {
        let id = handoff
            .source()
            .nodes()
            .node(site.node())
            .unwrap()
            .resolved_node
            .unwrap();
        rows.iter().copied().find(|row| row.site() == id).unwrap()
    };
    for (index, step) in steps.iter().enumerate() {
        assert_eq!(
            task272b1_goal_text(step.before()),
            if index < 2 {
                initial.clone()
            } else {
                remaining.clone()
            }
        );
        assert_eq!(
            task272b1_goal_text(step.after()),
            if index == 0 {
                initial.clone()
            } else {
                remaining.clone()
            }
        );
        assert_eq!(step.free(), if index == 0 { vec![] } else { vec![p] });
        assert!(step.avoided().is_empty());
        assert_eq!(step.owner(), steps[0].site());
        assert_eq!(step.context(), steps[0].context());
        match index {
            0 => assert_eq!(original(step.site()).kind(), &K::TheoremItem),
            1 => assert_eq!(
                original(step.site()).range(),
                handoff.scope().bindings()[1].range()
            ),
            2 => assert_eq!(original(step.site()).kind(), &K::ReconsiderItem),
            3 => assert_eq!(original(step.site()).kind(), &K::ConclusionStatement),
            _ => unreachable!(),
        }
    }
    assert_eq!(inference.terms().len(), 5);
    assert_eq!(inference.formulas().len(), 1);
    for (_, term) in inference.terms().iter() {
        assert_eq!(term.status, TermStatus::Inferred);
        assert_ne!(term.reference, Some(TermReference::Binding(k)));
        let entry = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(entry.status, TypeStatus::Known);
        let TypeEntryActual::Known(id) = entry.actual else {
            panic!("genuine known declaration type");
        };
        assert_eq!(
            inference.normalized_types().get(id).unwrap().head,
            TypeHeadRef::BuiltinSet
        );
    }
    assert_eq!(
        inference.formulas().iter().next().unwrap().1.kind,
        FormulaKind::Equality
    );
    let (intents, _) = handoff.check_proof_intents().unwrap();
    for index in [0, 2] {
        assert!(
            intents
                .iter()
                .any(|(site, _, status)| site == steps[index].site() && *status == S::Blocked)
        );
    }
    assert!(
        intents
            .iter()
            .any(|(site, _, status)| site == steps[3].site() && *status == S::Pending)
    );
    let registration = rows
        .iter()
        .copied()
        .find(|row| {
            row.kind() == &K::ProofBlock
                && row.parent().is_some_and(|id| {
                    rows.iter().any(|parent| {
                        parent.site() == id && parent.kind() == &K::CorrectnessCondition
                    })
                })
        })
        .unwrap();
    assert!(intents.iter().any(
        |(site, _, status)| original(site).site() == registration.site() && *status == S::Pending
    ));
    assert!(
        steps
            .iter()
            .all(|step| original(step.site()).range().start > registration.range().end)
    );
    let observed = super::native_witness_goal_observation(&handoff).unwrap();
    let terminals = observed
        .iter()
        .filter(|key| key.starts_with("remaining."))
        .collect::<Vec<_>>();
    assert_eq!(terminals.len(), 1);
    assert!(terminals[0].contains(&format!("terminal={:?}", steps[3].site())));
    assert!(!terminals[0].contains(&format!("terminal={:?}", steps[2].site())));
    let result = super::run_advanced_semantics_case(&config.workspace_root, old, 0);
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Passed);
    assert_eq!(
        result.actual_detail_keys,
        ["conversion.reconsider.narrowing_requires_proof"]
    );
    assert!(old.expectation.tags.is_empty());
    assert!(
        !super::run_advanced_semantics_corpus(&config)
            .unwrap()
            .results
            .iter()
            .any(|case| case.id == old.id)
    );
    assert!(handoff.source().facts().is_empty());
    assert!(handoff.source().types().is_empty());
    assert!(handoff.source().initial_obligations().is_empty());
}

#[test]
fn task272b2_prepared_case_refuses_metadata_drift_and_error_key_matching() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let old = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_types_reconsider_omitted_justification_001")
        .unwrap();
    for mutation in 0..20 {
        let mut bad = old.clone();
        match mutation {
            0 => bad.id.0 = "other".into(),
            1 => bad.expectation.id.0 = "other".into(),
            2 => bad.expectation.schema_version = 2,
            3 => bad.source_path = config.workspace_root.join("tests/miz/fail/types/other.miz"),
            4 => {
                bad.expectation_path = config
                    .workspace_root
                    .join("tests/miz/fail/types/other.expect.toml")
            }
            5 => bad.expectation.stage = crate::staged_model::Stage::TypeElaboration,
            6 => bad.expectation.domain = "checker.native_witness_goals".into(),
            7 => bad.expectation.kind = crate::expectation::TestKind::Pass,
            8 => bad.expectation.expected_outcome = crate::expectation::ExpectedOutcome::Pass,
            9 => {
                bad.expectation.expected_phase =
                    Some(crate::expectation::PipelinePhase::Elaboration)
            }
            10 => bad.expectation.failure_category = None,
            11 => bad.expectation.rejection_reason = None,
            12 => bad.expectation.stable_detail_key = None,
            13 => bad.expectation.spec_refs.clear(),
            14 => bad.expectation.profiles.clear(),
            15 => bad
                .expectation
                .tags
                .push("active_advanced_semantics".into()),
            16 => bad.expectation.diagnostic_codes.push("unexpected".into()),
            17 => bad.expectation.snapshots = Some("unexpected".into()),
            18 => bad.expectation.source = "other.miz".into(),
            _ => bad
                .expectation
                .declaration_symbol_payloads
                .push("unexpected".into()),
        }
        let result = super::run_advanced_semantics_case(&config.workspace_root, &bad, 0);
        assert_eq!(result.status, super::TypeElaborationCaseStatus::Failed);
        assert_eq!(
            result.actual_detail_keys,
            ["advanced_semantics.invalid_prepared_case"]
        );
    }
    let mut counterfeit = old.clone();
    counterfeit.expectation.stable_detail_key =
        Some("advanced_semantics.invalid_prepared_case".into());
    let result = super::run_advanced_semantics_case(&config.workspace_root, &counterfeit, 0);
    assert_eq!(
        result.actual_detail_keys,
        ["advanced_semantics.invalid_prepared_case"]
    );
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Failed);
}

#[test]
fn task272b2_prepared_case_consumes_near_misses_registration_and_goal_bypass() {
    use mizar_checker::{
        source_proof_local_declaration::SourceProofIntentStatus as S,
        type_checker::TypeFactQueryStatus, typed_ast::CoercionStatus,
    };
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let old = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_types_reconsider_omitted_justification_001")
        .unwrap();
    let source = std::fs::read_to_string(&old.source_path).unwrap();
    let prior =
        "theorem Prior: for p being set holds p = p proof let p be set; thus thesis; end;\n";
    let pending = format!(
        "{prior}{}",
        source.replace(
            "reconsider k = s as keen set;",
            "reconsider k = s as keen set by Prior;"
        )
    );
    let widening = source.replace(
        "reconsider k = s as keen set;",
        "reconsider k = s as object;",
    );
    let unrelated = source.replace(
        "reconsider k = s as keen set;",
        "reconsider k = s as object by Unknown;",
    );
    for (text, expected) in [
        (&pending, CoercionStatus::RequiresObligation),
        (&widening, CoercionStatus::Candidate),
        (&unrelated, CoercionStatus::Candidate),
    ] {
        let handoff = task272b2_old_source_handoff(text);
        let (_, checked, _) = handoff.check_conversions().unwrap();
        assert_eq!(
            checked.coercions().iter().next().unwrap().1.status,
            expected
        );
        handoff.check_witness_goals().unwrap();
        assert!(checked.diagnostics().is_empty());
        if text == &unrelated {
            assert!(
                handoff
                    .check_proof_intents()
                    .unwrap()
                    .0
                    .iter()
                    .any(|(_, _, status)| *status == S::Blocked)
            );
        }
        let result = metamorphic_guard::with_source(old, text, || {
            super::run_advanced_semantics_case(&config.workspace_root, old, 0)
        });
        assert_eq!(result.status, super::TypeElaborationCaseStatus::Failed);
        assert!(result.actual_detail_keys.is_empty());
    }
    let registration = source.replace(
        "existence proof thus thesis; end;",
        "existence proof thus contradiction; end;",
    );
    let handoff = task272b2_old_source_handoff(&registration);
    let (steps, _, _, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 4);
    assert_eq!(inference.terms().len(), 5);
    assert_eq!(inference.formulas().len(), 1);
    let result = metamorphic_guard::with_source(old, &registration, || {
        super::run_advanced_semantics_case(&config.workspace_root, old, 0)
    });
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Passed);
    assert_eq!(
        result.actual_detail_keys,
        ["conversion.reconsider.narrowing_requires_proof"]
    );
    let unsupported = source.replace(
        "for s being set holds s = s",
        "for s being set holds s = s implies s = s",
    );
    let handoff = task272b2_old_source_handoff(&unsupported);
    let (_, checked, queries) = handoff.check_conversions().unwrap();
    assert_eq!(
        checked.coercions().iter().next().unwrap().1.status,
        CoercionStatus::Rejected
    );
    assert_eq!(queries[0].status(), TypeFactQueryStatus::Missing);
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(old, &unsupported, || {
        super::run_advanced_semantics_case(&config.workspace_root, old, 0)
    });
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Failed);
    assert_ne!(
        result.actual_detail_keys,
        ["conversion.reconsider.narrowing_requires_proof"]
    );
    let alias = "theorem AliasUse: for s being set ex a being object st a = s proof let s be set; reconsider k = s as object; take k; thus thesis; end;";
    let handoff = task272b2_old_source_handoff(alias);
    assert_eq!(
        handoff
            .check_conversions()
            .unwrap()
            .1
            .coercions()
            .iter()
            .next()
            .unwrap()
            .1
            .status,
        CoercionStatus::Candidate
    );
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(old, alias, || {
        super::run_advanced_semantics_case(&config.workspace_root, old, 0)
    });
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Failed);
}

#[test]
fn task272b2_blocked_checkpoints_stay_with_their_actual_theorem_owner() {
    use mizar_checker::{
        source_proof_local_declaration::SourceProofIntentStatus as S, typed_ast::CoercionStatus,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let old = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_types_reconsider_omitted_justification_001")
        .unwrap();
    let source = std::fs::read_to_string(&old.source_path).unwrap();
    let later = "\ntheorem OtherOwner: for s being set holds s = s proof let s be set; reconsider k = s as object; thus thesis; end;";
    let source = format!("{source}{later}");
    let handoff = task272b2_old_source_handoff(&source);
    let (steps, _, _, _, _) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 8);
    assert_ne!(steps[0].owner(), steps[4].owner());
    assert_ne!(steps[0].context(), steps[4].context());
    assert_ne!(steps[3].after(), steps[7].after());
    let (requests, checked, _) = handoff.check_conversions().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        checked
            .coercions()
            .iter()
            .find(|(_, row)| row.site == *steps[2].site())
            .unwrap()
            .1
            .status,
        CoercionStatus::Rejected
    );
    assert_eq!(
        checked
            .coercions()
            .iter()
            .find(|(_, row)| row.site == *steps[6].site())
            .unwrap()
            .1
            .status,
        CoercionStatus::Candidate
    );
    let (intents, _) = handoff.check_proof_intents().unwrap();
    for (owner, status) in [
        (steps[0].owner(), S::Blocked),
        (steps[4].owner(), S::Pending),
    ] {
        assert!(
            intents
                .iter()
                .any(|(site, _, actual)| site == owner && *actual == status)
        );
    }
    let observations = super::native_witness_goal_observation(&handoff).unwrap();
    assert_eq!(
        observations
            .iter()
            .filter(|key| key.starts_with("remaining."))
            .count(),
        2
    );
    for step in [steps[2].site(), steps[6].site()] {
        let id = handoff
            .source()
            .nodes()
            .node(step.node())
            .unwrap()
            .resolved_node
            .unwrap();
        assert!(
            handoff
                .source()
                .source_node_entries()
                .unwrap()
                .any(|row| row.site() == id && row.kind() == &K::ReconsiderItem)
        );
    }
    let result = metamorphic_guard::with_source(old, &source, || {
        super::run_advanced_semantics_case(&config.workspace_root, old, 0)
    });
    assert_eq!(result.status, super::TypeElaborationCaseStatus::Passed);
    assert_eq!(
        result.actual_detail_keys,
        ["conversion.reconsider.narrowing_requires_proof"]
    );
}

#[test]
fn task272b3_local_alias_goals_execute_existing_set_take_fs() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_set_local_constant_take_001")
        .unwrap();
    assert_eq!(
        super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0)
            .status,
        super::FormulaStatementCaseStatus::Passed
    );
    let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.expect("genuine existing Set/Take source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    println!(
        "Task272B3 test-first: unchanged FS semantic engine passes and actual source/native/freshTyped/Resolved succeed"
    );
    let (steps, success, pending, checked, inference) = handoff
        .check_witness_goals()
        .expect("actual Set/Take FS source must materialize alias-expanded goals");
    assert_eq!(steps.len(), 5);
    assert_eq!(inference.terms().len(), 9);
    assert_eq!(inference.formulas().len(), 2);
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [1, 0, 2, 3, 3, 2]
    );
    let (header, formal, value) = (
        task272b1_binding(&handoff, 0),
        task272b1_binding(&handoff, 1),
        task272b1_binding(&handoff, 2),
    );
    let alias = task272b3_assert_alias_type(
        &handoff,
        &inference,
        3,
        mizar_checker::type_checker::TypeHeadRef::BuiltinObject,
    );
    assert_eq!(
        handoff
            .binding_env()
            .bindings()
            .get(alias)
            .unwrap()
            .captured
            .identities(),
        [handoff
            .binding_env()
            .bindings()
            .get(value)
            .unwrap()
            .identity
            .clone()]
    );
    assert_eq!(
        steps.iter().map(|step| step.matched()).collect::<Vec<_>>(),
        [
            None,
            Some((header, value)),
            None,
            Some((formal, alias)),
            None
        ]
    );
    let g0 = format!(
        "forall({}:Object,exists({}:Object,eq({},{})))",
        header.index(),
        formal.index(),
        formal.index(),
        header.index()
    );
    let g1 = format!(
        "exists({}:Object,eq({},{}))",
        formal.index(),
        formal.index(),
        value.index()
    );
    let g2 = format!("eq({0},{0})", value.index());
    for (index, step) in steps.iter().enumerate() {
        assert_eq!(
            task272b1_goal_text(step.before()),
            if index < 2 {
                g0.clone()
            } else if index < 4 {
                g1.clone()
            } else {
                g2.clone()
            }
        );
        assert_eq!(
            task272b1_goal_text(step.after()),
            if index == 0 {
                g0.clone()
            } else if index < 3 {
                g1.clone()
            } else {
                g2.clone()
            }
        );
        assert_eq!(step.free(), if index == 0 { vec![] } else { vec![value] });
        assert_eq!(
            step.avoided(),
            if index == 1 { vec![formal] } else { vec![] }
        );
        assert_eq!(step.owner(), steps[0].site());
        assert_eq!(step.context(), steps[0].context());
    }
    let item = handoff
        .source()
        .nodes()
        .node(steps[2].site().node())
        .unwrap()
        .resolved_node
        .unwrap();
    assert!(handoff.source().source_node_entries().unwrap().any(|row|row.site()==item && row.kind()==&mizar_syntax::SurfaceNodeKind::Equating));
    assert_eq!(success.len(), 1);
    assert!(pending.is_empty());
    assert!(checked.initial_obligations().is_empty());
    assert_eq!(
        checked.coercions().iter().next().unwrap().1.status,
        mizar_checker::typed_ast::CoercionStatus::Candidate
    );
    assert!(handoff.source().types().is_empty());
    assert!(handoff.source().facts().is_empty());
    super::native_witness_goal_observation(&handoff).unwrap();
}

fn task272b3_assert_alias_type(
    handoff: &mizar_checker::source_proof_local_declaration::SourceProofLocalBindingHandoff,
    inference: &mizar_checker::type_checker::TermFormulaInferenceOutput,
    index: usize,
    head: mizar_checker::type_checker::TypeHeadRef,
) -> mizar_checker::binding_env::BindingId {
    use mizar_checker::{
        binding_env::BindingTypeSite,
        type_checker::{TermReference, TermStatus},
        typed_ast::{TypeEntryActual, TypeStatus},
    };
    use mizar_resolve::names::SourceVariableBindingKind as B;
    let source = &handoff.scope().bindings()[index];
    let alias = task272b1_binding(handoff, index);
    assert!(matches!(source.kind(), B::Set | B::NamedWitness));
    assert!(source.declared_type().is_none());
    assert!(source.captures().is_empty());
    let entry = handoff.binding_env().bindings().get(alias).unwrap();
    assert_eq!(entry.type_site, BindingTypeSite::Missing);
    if source.kind() == B::NamedWitness {
        assert!(entry.captured.identities().is_empty());
    }
    let terms = inference
        .terms()
        .iter()
        .filter(|(_, term)| term.reference == Some(TermReference::Binding(alias)))
        .map(|(_, term)| term)
        .collect::<Vec<_>>();
    assert_eq!(
        terms.len(),
        handoff
            .scope()
            .references()
            .iter()
            .filter(|reference| reference.binding() == source.id())
            .count()
    );
    assert!(!terms.is_empty());
    for term in terms {
        assert_eq!(term.status, TermStatus::Inferred);
        let ty = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(ty.status, TypeStatus::Known);
        let TypeEntryActual::Known(id) = ty.actual else {
            panic!("actual alias value type");
        };
        assert_eq!(inference.normalized_types().get(id).unwrap().head, head);
        let node = handoff
            .source()
            .nodes()
            .node(term.site.node())
            .unwrap()
            .resolved_node
            .unwrap();
        let row = handoff
            .source()
            .source_node_entries()
            .unwrap()
            .find(|row| row.site() == node)
            .unwrap();
        assert!(
            handoff
                .scope()
                .references()
                .iter()
                .any(|reference| reference.binding() == source.id()
                    && reference.range() == row.range())
        );
    }
    alias
}

#[test]
fn task272b3_local_alias_goals_preserve_named_source_ids_and_transitive_captures() {
    use mizar_checker::{type_checker::TypeHeadRef, typed_ast::CoercionStatus};
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_witness_goals_001.miz"
    );
    let named = base.replace("thus thesis;", "thus k = s & o = o;");
    let handoff = task272b1_witness_source(&named);
    let (steps, _, pending, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [2, 0, 3, 1, 4, 5, 6, 4, 5, 5]
    );
    assert!(pending.is_empty());
    assert_eq!(steps.len(), 6);
    let (s, o, k) = (
        task272b1_binding(&handoff, 4),
        task272b1_binding(&handoff, 5),
        task272b3_assert_alias_type(&handoff, &inference, 6, TypeHeadRef::BuiltinSet),
    );
    assert_ne!(k, s);
    assert_eq!(
        task272b1_goal_text(steps[5].after()),
        format!("and(eq({0},{0}),eq({1},{1}))", s.index(), o.index())
    );
    assert_eq!(steps[5].free(), [s, o]);
    super::native_witness_goal_observation(&handoff).unwrap();
    let same_list = base.replace("take k = s, o;", "take k = s, k;");
    let handoff = task272b1_witness_source(&same_list);
    let (steps, success, pending, checked, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [2, 0, 3, 1, 4, 6]
    );
    assert_eq!(success.len(), 2);
    assert!(pending.is_empty());
    assert!(
        checked
            .coercions()
            .iter()
            .all(|(_, row)| row.status == CoercionStatus::Candidate)
    );
    let (s, o, k) = (
        task272b1_binding(&handoff, 4),
        task272b1_binding(&handoff, 5),
        task272b3_assert_alias_type(&handoff, &inference, 6, TypeHeadRef::BuiltinSet),
    );
    assert_eq!(
        steps[3].matched(),
        Some((task272b1_binding(&handoff, 2), s))
    );
    assert_eq!(
        steps[4].matched(),
        Some((task272b1_binding(&handoff, 3), k))
    );
    assert_eq!(
        task272b1_goal_text(steps[5].after()),
        format!("and(eq({0},{0}),eq({0},{1}))", s.index(), o.index())
    );
    super::native_witness_goal_observation(&handoff).unwrap();
    let chain = base
        .replace("take k = s, o;", "set a = s, b = a;\n  take k = b, o;")
        .replace("thus thesis;", "thus k = s & o = o;");
    let handoff = task272b1_witness_source(&chain);
    let (steps, success, pending, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [2, 0, 3, 1, 4, 6, 7, 5, 8, 4, 5, 5]
    );
    assert_eq!(steps.len(), 8);
    assert_eq!(inference.terms().len(), 18);
    assert_eq!(inference.formulas().len(), 4);
    assert_eq!(success.len(), 2);
    assert!(pending.is_empty());
    let (s, o) = (
        task272b1_binding(&handoff, 4),
        task272b1_binding(&handoff, 5),
    );
    let a = task272b3_assert_alias_type(&handoff, &inference, 6, TypeHeadRef::BuiltinSet);
    let b = task272b3_assert_alias_type(&handoff, &inference, 7, TypeHeadRef::BuiltinSet);
    let k = task272b3_assert_alias_type(&handoff, &inference, 8, TypeHeadRef::BuiltinSet);
    assert_eq!(
        handoff
            .binding_env()
            .bindings()
            .get(a)
            .unwrap()
            .captured
            .identities(),
        [handoff
            .binding_env()
            .bindings()
            .get(s)
            .unwrap()
            .identity
            .clone()]
    );
    assert_eq!(
        handoff
            .binding_env()
            .bindings()
            .get(b)
            .unwrap()
            .captured
            .identities(),
        [handoff
            .binding_env()
            .bindings()
            .get(a)
            .unwrap()
            .identity
            .clone()]
    );
    assert!(
        handoff
            .binding_env()
            .bindings()
            .get(k)
            .unwrap()
            .captured
            .identities()
            .is_empty()
    );
    for index in [3, 4] {
        assert!(steps[index].matched().is_none());
        assert_eq!(steps[index].before(), steps[index].after());
        assert!(steps[index].avoided().is_empty());
    }
    assert_eq!(
        steps[5].matched(),
        Some((task272b1_binding(&handoff, 2), b))
    );
    assert_eq!(
        task272b1_goal_text(steps[7].after()),
        format!("and(eq({0},{0}),eq({1},{1}))", s.index(), o.index())
    );
    super::native_witness_goal_observation(&handoff).unwrap();
}

#[test]
fn task272b3_local_alias_goals_keep_object_pending_and_child_occurrence_context() {
    use mizar_checker::{
        type_checker::{TermReference, TypeHeadInput, TypeHeadRef},
        typed_ast::{InitialObligationStatus, TypeEntryActual},
    };
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_witness_goals_001.miz"
    );
    let pending = base
        .replace("take k = s, o;", "take k = o, s;")
        .replace("thus thesis;", "thus k = s & s = o;");
    let handoff = task272b1_witness_source(&pending);
    let (steps, success, pending, checked, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [2, 0, 3, 1, 5, 4, 6, 4, 4, 5]
    );
    assert_eq!((success.len(), pending.len()), (1, 1));
    let (s, o) = (
        task272b1_binding(&handoff, 4),
        task272b1_binding(&handoff, 5),
    );
    task272b3_assert_alias_type(&handoff, &inference, 6, TypeHeadRef::BuiltinObject);
    assert_eq!(
        task272b1_goal_text(steps[5].after()),
        format!("and(eq({1},{0}),eq({0},{1}))", s.index(), o.index())
    );
    let obligation = checked.initial_obligations().iter().next().unwrap().1;
    assert_eq!(obligation.status, InitialObligationStatus::Pending);
    assert!(
        super::native_witness_goal_observation(&handoff)
            .unwrap()
            .iter()
            .any(|line| line.starts_with("remaining.")
                && line.contains(&format!("pending=[{:?}]", obligation.id)))
    );
    let object_alias = "theorem ObjectAlias: for o being object ex s being set st s = o proof let o be object; set a = o; take a; thus thesis; end;";
    let handoff = task272b1_witness_source(object_alias);
    let (steps, success, pending, checked, inference) = handoff.check_witness_goals().unwrap();
    let alias = task272b3_assert_alias_type(&handoff, &inference, 3, TypeHeadRef::BuiltinObject);
    let value = task272b1_binding(&handoff, 2);
    assert!(success.is_empty());
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].target_type.head, TypeHeadInput::BuiltinSet);
    assert_eq!(
        steps[3].matched(),
        Some((task272b1_binding(&handoff, 1), alias))
    );
    assert_eq!(
        task272b1_goal_text(steps[4].after()),
        format!("eq({0},{0})", value.index())
    );
    assert_eq!(
        checked
            .initial_obligations()
            .iter()
            .next()
            .unwrap()
            .1
            .status,
        InitialObligationStatus::Pending
    );
    super::native_witness_goal_observation(&handoff).unwrap();
    let retained = "theorem RetainedAlias: for y being set ex x being set st for y being set holds x = y proof let y be set; set a = y; take k = a; thus for z being set holds k = z; end;";
    let handoff = task272b1_witness_source(retained);
    let (steps, _, _, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| reference.binding().index())
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6]
    );
    let a = task272b3_assert_alias_type(&handoff, &inference, 4, TypeHeadRef::BuiltinSet);
    let k = task272b3_assert_alias_type(&handoff, &inference, 5, TypeHeadRef::BuiltinSet);
    let (bound, value) = (
        task272b1_binding(&handoff, 2),
        task272b1_binding(&handoff, 3),
    );
    assert_eq!(
        steps[3].matched(),
        Some((task272b1_binding(&handoff, 1), a))
    );
    assert_eq!(
        task272b1_goal_text(steps[4].after()),
        format!(
            "forall({}:Set,eq({},{}))",
            bound.index(),
            value.index(),
            bound.index()
        )
    );
    assert_eq!(steps[3].free(), [value]);
    assert_eq!(steps[3].avoided(), [bound]);
    let term = inference
        .terms()
        .iter()
        .find(|(_, term)| term.reference == Some(TermReference::Binding(k)))
        .unwrap()
        .1;
    let declaration = handoff.binding_env().bindings().get(k).unwrap();
    assert_ne!(term.context, declaration.owner_context);
    assert_eq!(
        term.context,
        handoff
            .binding_env()
            .bindings()
            .get(task272b1_binding(&handoff, 6))
            .unwrap()
            .owner_context
    );
    let ty = inference.type_entries().get(term.type_entry).unwrap();
    assert!(matches!(ty.actual, TypeEntryActual::Known(_)));
    super::native_witness_goal_observation(&handoff).unwrap();
}

#[test]
fn task272b3_local_alias_goals_preserve_liveness_owner_and_actual_fs_gate() {
    use mizar_checker::source_proof_local_declaration::{
        SourceProofLocalDeclarationError, SourceProofLocalDeclarationProducer,
    };
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_witness_goals_001.miz"
    );
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let native = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::NATIVE_WITNESS_GOAL_ID)
        .unwrap();
    for source in [
        base.replace("take k = s, o;", "set a = a; take k = s, o;"),
        base.replace("take k = s, o;", "set a = b, b = s; take k = s, o;"),
        base.replace("take k = s, o;", "take k = k, o;"),
        base.replace("take k = s, o;", "set unused = {s}; take k = s, o;"),
    ] {
        let frontend = metamorphic_guard::with_source(native, &source, || {
            super::run_frontend(&config.workspace_root, native, 0)
        })
        .unwrap();
        assert!(
            frontend.diagnostics.is_empty(),
            "{:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.unwrap();
        let resolver = super::resolver_symbol_collection(&config.workspace_root, native, &ast);
        assert!(resolver.detail_keys.is_empty());
        assert!(
            super::native_conversion_handoff(&ast, &resolver.module, &resolver.env)
                .and_then(|handoff| handoff
                    .check_witness_goals()
                    .map_err(|_| "unsupported alias".to_owned()))
                .is_err()
        );
    }
    let original_source = "theorem Capture: for x, z being set ex y being set st y = x proof let x, z be set; set a = x; take a; thus a = x; end;";
    let (original, _, _, _) = task272_proof_intent_inputs(original_source);
    let changed = original_source.replace("set a = x;", "set a = z;");
    let (coherent, native, symbols, scope) = task272_proof_intent_inputs(&changed);
    SourceProofLocalDeclarationProducer::build_bindings(&coherent, &native, &symbols, &scope)
        .unwrap()
        .check_witness_goals()
        .unwrap();
    assert!(matches!(
        SourceProofLocalDeclarationProducer::build_bindings(&original, &native, &symbols, &scope),
        Err(SourceProofLocalDeclarationError::DependencyMismatch)
    ));
    let fs_source = include_str!(
        "../../../../tests/miz/pass/variables/pass_formula_statement_set_local_constant_take_001.miz"
    );
    let two = format!(
        "{}\n{}",
        fs_source.replace("SetTake1", "FirstAliases"),
        fs_source.replace("SetTake1", "SecondAliases")
    );
    let handoff = task272b1_witness_source(&two);
    let (steps, _, _, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 10);
    assert_ne!(steps[0].owner(), steps[5].owner());
    assert_ne!(steps[0].context(), steps[5].context());
    for (terminal, proof, alias) in [(4, 2, 3), (9, 6, 7)] {
        let value = task272b1_binding(&handoff, proof);
        task272b3_assert_alias_type(
            &handoff,
            &inference,
            alias,
            mizar_checker::type_checker::TypeHeadRef::BuiltinObject,
        );
        assert_eq!(
            task272b1_goal_text(steps[terminal].after()),
            format!("eq({0},{0})", value.index())
        );
    }
    let fs = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_set_local_constant_take_001")
        .unwrap();
    let modified = fs_source.replace(
        "let x be object;",
        "let x be object; deffunc Unused() -> object equals x;",
    );
    let frontend = metamorphic_guard::with_source(fs, &modified, || {
        super::run_frontend(&config.workspace_root, fs, 0)
    })
    .unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, fs, &ast);
    assert!(
        super::source_variable_semantics_detail_keys(&ast, &resolver.module, &resolver.env)
            .is_empty()
    );
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(fs, &modified, || {
        super::formula_statement::run_formula_statement_case(&config.workspace_root, fs, 0)
    });
    assert_eq!(result.status, super::FormulaStatementCaseStatus::Failed);
    assert!(
        result
            .actual_detail_keys
            .iter()
            .any(|key| key.contains("native witness goal"))
    );
}

#[test]
fn task272b4_implication_goals_execute_existing_then_hence_fs() {
    use mizar_checker::{
        source_proof_local_declaration::SourceProofIntentStatus,
        type_checker::{FormulaKind, FormulaStatus, TermReference, TermStatus, TypeHeadRef},
        typed_ast::{TypeEntryActual, TypeStatus, TypedSiteRef},
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_then_hence_linking_001")
        .unwrap();
    assert_eq!(
        super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0)
            .status,
        super::FormulaStatementCaseStatus::Passed
    );
    let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.expect("genuine existing ThenChain source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    assert_eq!(
        super::formula_statement::check_formula_ast_with_organization(
            &ast,
            &resolver.module,
            &resolver.env,
            true,
        ),
        Ok(true)
    );
    println!(
        "Task272B4 test-first: unchanged FS semantic engine passes and genuine frontend/symbol/native/scope handoff succeeds"
    );
    let (steps, success, pending, checked, inference) = handoff
        .check_witness_goals()
        .expect("actual ThenChain FS source must materialize implication/assumption goals");
    assert_eq!(steps.len(), 5);
    assert_eq!(inference.terms().len(), 12);
    assert_eq!(inference.formulas().len(), 5);
    let (header, local) = (
        task272b1_binding(&handoff, 0),
        task272b1_binding(&handoff, 1),
    );
    assert_ne!(header, local);
    assert_eq!(handoff.scope().bindings().len(), 2);
    assert_eq!(
        handoff
            .scope()
            .references()
            .iter()
            .map(|reference| task272b1_binding(&handoff, reference.binding().index()))
            .collect::<Vec<_>>(),
        [
            header, header, header, header, local, local, local, local, local, local
        ]
    );
    let g0 = format!(
        "forall({}:Set,implies(eq({0},{0}),eq({0},{0})))",
        header.index()
    );
    let g1 = format!("implies(eq({0},{0}),eq({0},{0}))", local.index());
    let g2 = format!("eq({0},{0})", local.index());
    let expected = [(&g0, &g0), (&g0, &g1), (&g1, &g2), (&g2, &g2), (&g2, &g2)];
    let rows = handoff
        .source()
        .source_node_entries()
        .unwrap()
        .collect::<Vec<_>>();
    let original = |site: &TypedSiteRef| {
        let id = handoff
            .source()
            .nodes()
            .node(site.node())
            .unwrap()
            .resolved_node
            .unwrap();
        rows.iter().copied().find(|row| row.site() == id).unwrap()
    };
    use mizar_checker::source_proof_local_declaration::SourceProofGoal as G;
    let G::Universal { origin, body, .. } = steps[0].after() else {
        panic!("typed universal")
    };
    assert!(matches!(original(origin).kind(), K::QuantifiedFormula(_)));
    let G::Implication {
        origin: implication_origin,
        antecedent,
        consequent,
    } = body.as_ref()
    else {
        panic!("restricted universal implication")
    };
    assert_eq!(implication_origin, origin);
    let G::Equality {
        origin: antecedent_origin,
        ..
    } = antecedent.as_ref()
    else {
        panic!("original antecedent equality")
    };
    let G::Equality {
        origin: consequent_origin,
        ..
    } = consequent.as_ref()
    else {
        panic!("original consequent equality")
    };
    assert_ne!(antecedent_origin, consequent_origin);
    for site in [antecedent_origin, consequent_origin] {
        assert_eq!(original(site).kind(), &K::BuiltinPredicateApplication);
    }
    for step in &steps[2..] {
        let G::Equality {
            origin,
            left,
            right,
        } = step.after()
        else {
            panic!("remaining equality")
        };
        assert_eq!(origin, consequent_origin);
        assert_eq!((*left, *right), (local, local));
    }
    let (intents, _) = handoff.check_proof_intents().unwrap();
    for (index, step) in steps.iter().enumerate() {
        assert_eq!(task272b1_goal_text(step.before()), *expected[index].0);
        assert_eq!(task272b1_goal_text(step.after()), *expected[index].1);
        assert_eq!(step.free(), if index == 0 { vec![] } else { vec![local] });
        assert!(step.avoided().is_empty());
        assert_eq!(
            step.matched(),
            if index == 1 {
                Some((header, local))
            } else {
                None
            }
        );
        assert_eq!(step.owner(), steps[0].site());
        assert_eq!(step.context(), steps[0].context());
        let row = original(step.site());
        if index == 1 {
            assert_eq!(row.range(), handoff.scope().bindings()[1].range());
        } else {
            assert_eq!(
                row.kind(),
                &[
                    K::TheoremItem,
                    K::TheoremItem,
                    K::AssumptionStatement,
                    K::CompactStatement,
                    K::ConclusionStatement
                ][index]
            );
        }
        if index >= 2 {
            assert!(intents.iter().any(|(site, _, status)| site == step.site()
                && *status == SourceProofIntentStatus::Pending));
        }
    }
    assert!(success.is_empty());
    assert!(pending.is_empty());
    assert!(checked.coercions().is_empty());
    assert!(checked.initial_obligations().is_empty());
    assert!(checked.facts().is_empty());
    assert!(checked.diagnostics().is_empty());
    for (_, term) in inference.terms().iter() {
        assert_eq!(term.status, TermStatus::Inferred);
        let entry = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(entry.status, TypeStatus::Known);
        let TypeEntryActual::Known(ty) = entry.actual else {
            panic!("known Set")
        };
        assert_eq!(
            inference.normalized_types().get(ty).unwrap().head,
            TypeHeadRef::BuiltinSet
        );
    }
    for reference in handoff.scope().references() {
        let binding = task272b1_binding(&handoff, reference.binding().index());
        assert!(inference.terms().iter().any(|(_, term)| term.reference
            == Some(TermReference::Binding(binding))
            && original(&term.site).range() == reference.range()));
    }
    for binding in [header, local] {
        assert!(inference.terms().iter().any(|(_, term)| {
            term.reference == Some(TermReference::Binding(binding))
                && matches!(&term.site, TypedSiteRef::Role { role, .. }
                    if role.as_str() == "witness-goal.declaration-value")
                && original(&term.site).range()
                    == handoff
                        .binding_env()
                        .bindings()
                        .get(binding)
                        .unwrap()
                        .declaration_range
        }));
    }
    for (_, formula) in inference.formulas().iter() {
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(original(&formula.site).range(), formula.source_range);
        assert_eq!(formula.terms.len(), 2);
    }
    assert!(inference.diagnostics().is_empty());
    assert!(handoff.source().types().is_empty());
    assert!(handoff.source().facts().is_empty());
    assert!(handoff.source().initial_obligations().is_empty());
    super::native_proof_intent_observation(&handoff).unwrap();
    super::native_witness_goal_observation(&handoff).unwrap();
}

#[test]
fn task272b4_implication_goals_preserve_assumption_identity_and_residual_boundary() {
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_then_hence_linking_001.miz"
    );
    for source in [
        base.replace("st X = X holds X = X", "holds X = X implies X = X"),
        base.replace("hence X = X;", "thus thesis;"),
        base.replace("hence X = X;", "hence X = X by A1;"),
        base.replace("assume A1:", "assume"),
        base.replace("A1", "Premise").replace("X", "Value"),
        "theorem Conjunction: for X being set st X=X & X=X holds X=X proof let X be set; assume A:X=X & X=X; thus X=X; end;".to_owned(),
        "theorem Antecedent: for X being set st (X=X implies X=X) holds X=X proof let X be set; assume A:X=X implies X=X; thus X=X; end;".to_owned(),
        "theorem Repeated: for X being set holds X=X implies X=X implies X=X proof let X be set; assume A:X=X; assume B:X=X; thus X=X; end;".to_owned(),
        "theorem Alpha: for X being set st (for y being set holds y=y) holds X=X proof let X be set; assume A:for z being set holds z=z; thus X=X; end;".to_owned(),
        "theorem Residual: for X being set st X=X holds for y being set holds y=X proof let X be set; assume A:X=X; thus thesis; end;".to_owned(),
    ] {
        let handoff = task272b1_witness_source(&source);
        let (steps, _, pending, _, _) = handoff.check_witness_goals().unwrap_or_else(|error|
            panic!("{source}: {error:?}"));
        assert!(!steps.is_empty());
        assert!(pending.is_empty());
        assert!(steps.iter().all(|step| step.owner() == steps[0].owner()));
        if source.contains("hence X = X by A1;") {
            use mizar_resolve::resolved_ast::{LabelKind, LabelResolution};
            let (_, labels) = handoff.check_proof_intents().unwrap();
            let [id] = labels.ids() else { panic!("one actual local citation") };
            let LabelResolution::Resolved(label) = labels.table().get(*id).unwrap().resolution()
                else { panic!("resolved actual Assume label") };
            assert_eq!(label.kind(), LabelKind::ProofStep);
            assert_eq!(labels.index().get(label.origin()).unwrap().primary_spelling(), "A1");
            assert_eq!(&source[label.range().start..label.range().end], "A1");
            let terminal = steps.last().unwrap();
            assert_eq!(terminal.before(), terminal.after());
            assert_eq!(task272b1_goal_text(terminal.after()),
                format!("eq({0},{0})", task272b1_binding(&handoff, 1).index()));
        }
    }
    for source in [
        base.replace("assume A1: X = X;", ""),
        base.replace("assume A1: X = X;", "assume A1: X = X & X = X;"),
        base.replace("then X = X;", "then X = X & X = X;"),
        base.replace("then X = X;", "then X = X by A1;"),
        base.replace("assume A1: X = X;", "assume that A1: X = X and A2: X = X;"),
        base.replace("then X = X;", "set K = X; then X = X;"),
        base.replace("hence X = X;", "hence X = X; then X = X;"),
        "theorem WrongFree: for X,Y being set st X=X holds X=X proof let X,Y be set; assume A:Y=Y; thus X=X; end;".to_owned(),
        "theorem Shadow: for X being set st (for y being set holds y=X) holds X=X proof let X be set; assume A:for X being set holds X=X; thus X=X; end;".to_owned(),
        "theorem Unconsumed: for X being set holds X=X implies X=X proof let X be set; thus thesis; end;".to_owned(),
        "theorem Nested: for X being set st X=X holds (X=X implies X=X) & X=X proof let X be set; assume A:X=X; thus thesis; end;".to_owned(),
        "theorem Retained: for X being set st X=X holds for y being set holds y=y implies X=X proof let X be set; assume A:X=X; thus thesis; end;".to_owned(),
        "theorem Order: for X being set st X=X holds X=X proof let X be set; thus X=X; assume A:X=X; end;".to_owned(),
    ] {
        let handoff = task272b1_witness_source(&source);
        if source.contains("set K = X;") {
            use mizar_checker::source_proof_local_declaration::SourceProofIntentStatus as S;
            use mizar_syntax::SurfaceNodeKind as K;
            let (intents, _) = handoff.check_proof_intents().unwrap();
            for (kind, status) in [(K::ThenStatement, S::Blocked), (K::CompactStatement, S::Pending)] {
                assert!(intents.iter().any(|(site, _, actual)| {
                    let id = handoff.source().nodes().node(site.node()).unwrap().resolved_node.unwrap();
                    *actual == status && handoff.source().source_node_entries().unwrap()
                        .any(|row| row.site() == id && row.kind() == &kind)
                }));
            }
        }
        assert!(handoff.check_witness_goals().is_err(), "{source}");
    }
    let nonreflexive = task272b1_witness_source(
        "theorem Nonreflexive: for X,Y being set st X=X holds X=Y proof let X,Y be set; assume A:X=X; thus X=Y; end;",
    );
    let (steps, success, pending, checked, _) = nonreflexive.check_witness_goals().unwrap();
    assert_eq!(
        task272b1_goal_text(steps.last().unwrap().after()),
        format!(
            "eq({},{})",
            task272b1_binding(&nonreflexive, 2).index(),
            task272b1_binding(&nonreflexive, 3).index()
        )
    );
    assert!(success.is_empty() && pending.is_empty() && checked.facts().is_empty());
    let dropped = task272b1_witness_source(
        "theorem Dropped: for X,Y being set st X=X holds Y=Y proof let X,Y be set; assume A:X=X; thus Y=Y; end;",
    );
    let (steps, _, _, _, _) = dropped.check_witness_goals().unwrap();
    assert_eq!(
        steps[2].free(),
        [
            task272b1_binding(&dropped, 2),
            task272b1_binding(&dropped, 3)
        ]
    );
    assert_eq!(steps[3].free(), [task272b1_binding(&dropped, 3)]);
    let two = format!(
        "{}\n{}",
        base.replace("ThenChain1", "First"),
        base.replace("ThenChain1", "Second")
    );
    let handoff = task272b1_witness_source(&two);
    let (steps, _, _, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 10);
    assert_eq!(inference.terms().len(), 24);
    assert_eq!(inference.formulas().len(), 10);
    assert_ne!(steps[0].owner(), steps[5].owner());
    assert_ne!(steps[0].context(), steps[5].context());
    assert!(
        steps[..5]
            .iter()
            .all(|step| step.owner() == steps[0].owner())
    );
    assert!(
        steps[5..]
            .iter()
            .all(|step| step.owner() == steps[5].owner())
    );
    let wrong_owner = two.replacen("assume A1: X = X;", "assume A1: X = X & X = X;", 1);
    assert!(
        task272b1_witness_source(&wrong_owner)
            .check_witness_goals()
            .is_err()
    );
}

#[test]
fn task272b4_implication_goals_are_required_by_actual_fs_consumer() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_then_hence_linking_001")
        .unwrap();
    let source = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_then_hence_linking_001.miz"
    )
    .replace("then X = X;", "Inner: now thus X = X by A1; end;")
    .replace("hence X = X;", "thus X = X by Inner;");
    let frontend = metamorphic_guard::with_source(case, &source, || {
        super::run_frontend(&config.workspace_root, case, 0)
    })
    .unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    assert_eq!(
        super::formula_statement::check_formula_ast_with_organization(
            &ast,
            &resolver.module,
            &resolver.env,
            true
        ),
        Ok(true)
    );
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(case, &source, || {
        super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0)
    });
    assert_eq!(result.status, super::FormulaStatementCaseStatus::Failed);
    assert!(
        result
            .actual_detail_keys
            .iter()
            .any(|key| key.contains("native witness goal"))
    );
}

#[test]
fn task272b5_conditioned_let_goals_execute_existing_fs() {
    use mizar_checker::{
        source_proof_local_declaration::SourceProofIntentStatus as S,
        type_checker::{FormulaKind, FormulaStatus, TermReference, TermStatus, TypeHeadRef},
        typed_ast::{TypeEntryActual, TypeStatus, TypedSiteRef},
    };
    use mizar_resolve::resolved_ast::{LabelKind, LabelResolution};
    use mizar_syntax::SurfaceNodeKind as K;
    let source = include_str!(
        "../../../../tests/miz/pass/variables/pass_formula_statement_let_such_that_assumption_001.miz"
    );
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_let_such_that_assumption_001")
        .unwrap();
    assert_eq!(
        super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0)
            .status,
        super::FormulaStatementCaseStatus::Passed
    );
    let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    let ast = frontend.ast.expect("genuine existing LetSuchThat source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty(), "{:?}", resolver.detail_keys);
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    println!(
        "Task272B5 test-first: unchanged FS semantic engine passes and genuine frontend/symbol/native/scope handoff succeeds"
    );
    let (steps, success, pending, checked, inference) = handoff
        .check_witness_goals()
        .expect("actual LetSuchThat FS source must materialize condition/citation goals");
    assert_eq!(steps.len(), 4);
    assert_eq!(inference.terms().len(), 10);
    assert_eq!(inference.formulas().len(), 4);
    let (header, local) = (task272b1_binding(&handoff, 0), task272b1_binding(&handoff, 1));
    assert_ne!(header, local);
    assert_eq!(handoff.scope().bindings().len(), 2);
    assert_eq!(handoff.scope().references().iter().map(|reference|
        task272b1_binding(&handoff, reference.binding().index())).collect::<Vec<_>>(),
        [header, header, header, header, local, local, local, local]);
    let g0 = format!("forall({}:Set,implies(eq({0},{0}),eq({0},{0})))", header.index());
    let g1 = format!("implies(eq({0},{0}),eq({0},{0}))", local.index());
    let g2 = format!("eq({0},{0})", local.index());
    let expected = [(&g0, &g0), (&g0, &g1), (&g1, &g2), (&g2, &g2)];
    let rows = handoff.source().source_node_entries().unwrap().collect::<Vec<_>>();
    let original = |site: &TypedSiteRef| {
        let id = handoff.source().nodes().node(site.node()).unwrap().resolved_node.unwrap();
        rows.iter().copied().find(|row| row.site() == id).unwrap()
    };
    let (intents, labels) = handoff.check_proof_intents().unwrap();
    for (index, step) in steps.iter().enumerate() {
        assert_eq!(task272b1_goal_text(step.before()), *expected[index].0);
        assert_eq!(task272b1_goal_text(step.after()), *expected[index].1);
        assert_eq!(step.matched(), if index == 1 { Some((header, local)) } else { None });
        assert_eq!(step.free(), if index == 0 { vec![] } else { vec![local] });
        assert!(step.avoided().is_empty());
        assert_eq!(step.owner(), steps[0].site());
        assert_eq!(step.context(), steps[0].context());
        let row = original(step.site());
        if index == 1 {
            assert_eq!(row.range(), handoff.scope().bindings()[1].range());
        } else {
            assert_eq!(row.kind(), &[K::TheoremItem, K::TheoremItem,
                K::ConditionList, K::ConclusionStatement][index]);
        }
    }
    assert!(!intents.iter().any(|(site, _, _)| site == steps[2].site()));
    assert!(intents.iter().any(|(site, _, status)| site == steps[3].site() && *status == S::Pending));
    let condition = original(steps[2].site());
    assert!(rows.iter().any(|row| Some(row.site()) == condition.parent() && row.kind() == &K::LetStatement));
    assert_eq!(labels.index().iter().filter(|label| label.kind() == LabelKind::ProofStep).count(), 1);
    assert!(labels.index().iter().any(|label| label.kind() == LabelKind::Theorem));
    let [id] = labels.ids() else { panic!("one genuine By reference") };
    let reference = labels.table().get(*id).unwrap();
    let LabelResolution::Resolved(label) = reference.resolution() else { panic!("resolved local condition") };
    assert_eq!(label.kind(), LabelKind::ProofStep);
    let declaration = labels.index().get(label.origin()).unwrap();
    assert_eq!(declaration.primary_spelling(), "A1");
    let declaration_start = source.find("A1:").unwrap();
    let use_start = source.rfind("A1;").unwrap();
    let mizar_session::SourceAnchor::Range(declaration_range) = declaration.origin().anchor()
        else { panic!("original declaration token") };
    assert_eq!((declaration_range.start, declaration_range.end), (declaration_start, declaration_start + 2));
    assert_eq!((label.range().start, label.range().end), (use_start, use_start + 2));
    assert_eq!(declaration.origin().source_id(), reference.origin().source_id());
    assert!(!declaration.origin().is_recovered() && !reference.origin().is_recovered());
    let use_row = rows.iter().copied().find(|row| row.kind() == &K::Reference && row.range() == label.range()).unwrap();
    let list = rows.iter().copied().find(|row| Some(row.site()) == use_row.parent()).unwrap();
    assert_eq!(list.kind(), &K::ReferenceList);
    let by = rows.iter().copied().find(|row| Some(row.site()) == list.parent()).unwrap();
    assert_eq!(by.kind(), &K::JustificationClause);
    assert_eq!(by.parent(), Some(original(steps[3].site()).site()));
    assert!(success.is_empty() && pending.is_empty());
    assert!(checked.coercions().is_empty() && checked.initial_obligations().is_empty() && checked.facts().is_empty());
    assert!(checked.diagnostics().is_empty() && inference.diagnostics().is_empty());
    for (_, term) in inference.terms().iter() {
        assert_eq!(term.status, TermStatus::Inferred);
        let entry = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(entry.status, TypeStatus::Known);
        let TypeEntryActual::Known(ty) = entry.actual else { panic!("known Set") };
        assert_eq!(inference.normalized_types().get(ty).unwrap().head, TypeHeadRef::BuiltinSet);
    }
    for reference in handoff.scope().references() {
        let binding = task272b1_binding(&handoff, reference.binding().index());
        assert!(inference.terms().iter().any(|(_, term)| term.reference == Some(TermReference::Binding(binding))
            && original(&term.site).range() == reference.range()));
    }
    for binding in [header, local] {
        assert!(inference.terms().iter().any(|(_, term)| term.reference == Some(TermReference::Binding(binding))
            && matches!(&term.site, TypedSiteRef::Role { role, .. } if role.as_str() == "witness-goal.declaration-value")
            && original(&term.site).range() == handoff.binding_env().bindings().get(binding).unwrap().declaration_range));
    }
    for (_, formula) in inference.formulas().iter() {
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(original(&formula.site).range(), formula.source_range);
    }
    assert!(handoff.source().types().is_empty() && handoff.source().facts().is_empty()
        && handoff.source().initial_obligations().is_empty());
    super::native_proof_intent_observation(&handoff).unwrap();
    super::native_witness_goal_observation(&handoff).unwrap();
}

#[test]
fn task272b5_conditions_and_citations_preserve_source_owner_and_frontier() {
    let base = include_str!(
        "../../../../tests/miz/pass/variables/pass_formula_statement_let_such_that_assumption_001.miz"
    );
    let direct_by = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_then_hence_linking_001.miz"
    ).replace("then X = X;\n  hence X = X;", "thus X = X by A1;");
    for source in [
        base.replace("A1: ", "").replace(" by A1", ""),
        base.replace("A1", "Premise"),
        base.replace("thus X = X by A1", "thus thesis by A1"),
        direct_by,
        "theorem Conjunction: for X being set st X=X & X=X holds X=X proof let X be set such that A:X=X & X=X; thus X=X by A; end;".to_owned(),
        "theorem Segments: for X being set, Y being object st X=Y holds X=Y proof let X be set, Y be object such that A:X=Y; thus X=Y by A; end;".to_owned(),
        "theorem Alpha: for X being set st (for y being set holds y=y) holds X=X proof let X be set such that A:for z being set holds z=z; thus X=X by A; end;".to_owned(),
        "theorem Unproved: for X,Y being set st X=X holds X=Y proof let X,Y be set; assume A:X=X; thus X=Y by A; end;".to_owned(),
        "theorem Frontier: for X,Y being set st X=X holds Y=Y proof let X,Y be set such that A:X=X; set K=X; thus Y=Y by A; end;".to_owned(),
    ] {
        let handoff = task272b1_witness_source(&source);
        let (steps, _, _, checked, _) = handoff.check_witness_goals()
            .unwrap_or_else(|error| panic!("{source}: {error:?}"));
        let terminal = steps.last().unwrap();
        assert_eq!(terminal.before(), terminal.after());
        assert!(checked.facts().is_empty());
        let (_, labels) = handoff.check_proof_intents().unwrap();
        for id in labels.ids() {
            use mizar_resolve::resolved_ast::{LabelKind, LabelResolution};
            let LabelResolution::Resolved(label) = labels.table().get(*id).unwrap().resolution()
                else { panic!("authentic prior label") };
            assert_eq!(label.kind(), LabelKind::ProofStep);
            let mizar_session::SourceAnchor::Range(range) = labels.index().get(label.origin()).unwrap().origin().anchor()
                else { panic!("actual label declaration") };
            assert!(range.end < label.range().start);
        }
        if source.contains("theorem Unproved:") {
            assert_eq!(task272b1_goal_text(terminal.after()), format!("eq({},{})",
                task272b1_binding(&handoff, 2).index(), task272b1_binding(&handoff, 3).index()));
        }
        if source.contains("theorem Frontier:") {
            assert_eq!(steps[2].free(), [task272b1_binding(&handoff, 2), task272b1_binding(&handoff, 3)]);
            assert_eq!(steps[3].free(), [task272b1_binding(&handoff, 3)]);
            assert_eq!(terminal.free(), [task272b1_binding(&handoff, 3)]);
        }
    }
    for source in [
        base.replace("such that A1: X = X", "such that A1: X = X & X = X"),
        base.replace("let X be set", "let X be object"),
        base.replace("such that A1: X = X", "such that A1: X = X and A2: X = X"),
        base.replace("such that A1: X = X", "such that A1: X = X by A1"),
        base.replace("thus X = X by A1;", "thus X = X by Missing;"),
        base.replace("A1: ", ""),
        base.replace("thus X = X by A1;", "thus X = X by LetSuchThat1;"),
        base.replace("thus X = X by A1;", "thus X = X by A1, A1;"),
        base.replace("thus X = X by A1;", "thus X = X by Imported.A1;"),
        base.replace("thus X = X by A1;", "thus X = X by Later; assume Later: X = X;"),
        base.replace("thus X = X by A1;", "assume A1: X = X; thus X = X by A1;"),
        base.replace("thus X = X by A1;", "thus X = X; set K = X;"),
        "theorem WrongFree: for X,Y being set st X=X holds Y=Y proof let X,Y be set such that A:Y=Y; thus Y=Y by A; end;".to_owned(),
        "theorem WrongKind: for X being set st X=X holds X=X proof let X be set such that A:X=X; Other:X=X; thus X=X by Other; end;".to_owned(),
    ] {
        assert!(task272b1_witness_source(&source).check_witness_goals().is_err(), "{source}");
    }
    let two = format!("{}\n{}", base.replace("LetSuchThat1", "First"), base.replace("LetSuchThat1", "Second"));
    let handoff = task272b1_witness_source(&two);
    let (steps, _, _, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 8);
    assert_eq!(inference.terms().len(), 20);
    assert_eq!(inference.formulas().len(), 8);
    assert_ne!(steps[0].owner(), steps[4].owner());
    assert_ne!(steps[0].context(), steps[4].context());
    let (_, labels) = handoff.check_proof_intents().unwrap();
    assert_eq!(labels.ids().len(), 2);
    let origins = labels.ids().iter().map(|id| {
        let mizar_resolve::resolved_ast::LabelResolution::Resolved(label) = labels.table().get(*id).unwrap().resolution()
            else { panic!("owner-local reference") };
        label.origin()
    }).collect::<Vec<_>>();
    assert_ne!(origins[0], origins[1]);
    let foreign = format!("{}\n{}", base.replace("LetSuchThat1", "First"),
        base.replace("LetSuchThat1", "Second").replace("A1: ", ""));
    assert!(task272b1_witness_source(&foreign).check_witness_goals().is_err());
    let theorem_citation = format!("{}\n{}", base.replace("LetSuchThat1", "First"),
        base.replace("LetSuchThat1", "Second").replace("by A1", "by First"));
    let handoff = task272b1_witness_source(&theorem_citation);
    let (_, labels) = handoff.check_proof_intents().unwrap();
    let mizar_resolve::resolved_ast::LabelResolution::Resolved(label) = labels.table()
        .get(*labels.ids().last().unwrap()).unwrap().resolution()
        else { panic!("genuine module theorem citation") };
    assert_eq!(label.kind(), mizar_resolve::resolved_ast::LabelKind::Theorem);
    assert!(handoff.check_witness_goals().is_err());
}

#[test]
fn task272b5_conditioned_let_native_gate_controls_actual_fs_result() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|
        case.id.0 == "pass_formula_statement_let_such_that_assumption_001").unwrap();
    let source = include_str!(
        "../../../../tests/miz/pass/variables/pass_formula_statement_let_such_that_assumption_001.miz"
    ).replace("thus X = X by A1;", "Inner: now thus X = X by A1; end; thus X = X by Inner;");
    let frontend = metamorphic_guard::with_source(case, &source, ||
        super::run_frontend(&config.workspace_root, case, 0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    assert!(super::source_variable_semantics_detail_keys(&ast, &resolver.module, &resolver.env).is_empty());
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(case, &source, ||
        super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0));
    assert_eq!(result.status, super::FormulaStatementCaseStatus::Failed);
    assert!(result.actual_detail_keys.iter().any(|key| key.contains("native witness goal")));
}

#[test]
fn task272b6_given_goals_execute_existing_fs() {
    use mizar_checker::{
        binding_env::{BindingKind, BindingTypeSite},
        source_proof_local_declaration::SourceProofIntentStatus as S,
        type_checker::{CoercionEvidence, CoercionJustification, CoercionRequestKind,
            FormulaKind, FormulaStatus, TermReference, TermStatus, TypeHeadInput, TypeHeadRef},
        typed_ast::{CoercionStatus, FactStatus, TypeEntryActual, TypeStatus, TypedSiteRef},
    };
    use mizar_resolve::{names::{SourceVariableBindingKind as B, SourceVariableTypeRadix as R},
        resolved_ast::{LabelKind, LabelResolution}};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_given_existential_assumption_001")
        .unwrap();
    assert_eq!(
        super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0)
            .status,
        super::FormulaStatementCaseStatus::Passed
    );
    let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    let ast = frontend.ast.expect("genuine existing Given1 source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty(), "{:?}", resolver.detail_keys);
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    println!(
        "Task272B6 test-first: unchanged FS semantic engine and native A pass; genuine frontend/symbol/native/scope handoff succeeds"
    );
    let (steps, success, pending, checked, inference) = handoff
        .check_witness_goals()
        .expect("actual Given1 FS source must materialize Given/membership goals");
    assert_eq!(steps.len(), 6);
    assert_eq!(inference.terms().len(), 14);
    assert_eq!(inference.formulas().len(), 4);
    let [h, e, z, p, w] = std::array::from_fn::<_, 5, _>(|index| task272b1_binding(&handoff, index));
    assert_eq!(handoff.scope().bindings().len(), 5);
    assert_eq!(handoff.scope().references().iter().map(|reference|
        task272b1_binding(&handoff, reference.binding().index())).collect::<Vec<_>>(),
        [e, h, z, h, w, p, w, w, p]);
    let member = |left: mizar_checker::binding_env::BindingId, right: mizar_checker::binding_env::BindingId|
        format!("member({},{})", left.index(), right.index());
    let g0 = format!("forall({}:Set,implies(exists({}:Object,{}),exists({}:Object,{})))",
        h.index(), e.index(), member(e,h), z.index(), member(z,h));
    let g1 = format!("implies(exists({}:Object,{}),exists({}:Object,{}))",
        e.index(), member(e,p), z.index(), member(z,p));
    let g2 = format!("implies({},exists({}:Object,{}))", member(w,p), z.index(), member(z,p));
    let g3 = format!("exists({}:Object,{})", z.index(), member(z,p));
    let g4 = member(w,p);
    let expected = [(&g0,&g0),(&g0,&g1),(&g1,&g2),(&g2,&g3),(&g3,&g4),(&g4,&g4)];
    let matched = [None,Some((h,p)),Some((e,w)),None,Some((z,w)),None];
    let free = [vec![],vec![p],vec![p,w],vec![p],vec![p,w],vec![p,w]];
    use mizar_checker::source_proof_local_declaration::SourceProofGoal as G;
    let G::Implication { origin: before_origin, consequent: before_consequent, .. } = steps[2].before()
        else { panic!("Given opens the existential antecedent of an implication") };
    let G::Implication { origin: after_origin, consequent: after_consequent, .. } = steps[2].after()
        else { panic!("same unproved implication after Given name") };
    assert_eq!(before_origin, after_origin);
    assert_eq!(before_consequent, after_consequent);
    assert_eq!(steps[3].after(), before_consequent.as_ref());
    let rows = handoff.source().source_node_entries().unwrap().collect::<Vec<_>>();
    let original = |site: &TypedSiteRef| {
        let id = handoff.source().nodes().node(site.node()).unwrap().resolved_node.unwrap();
        rows.iter().copied().find(|row| row.site() == id).unwrap()
    };
    for (index, step) in steps.iter().enumerate() {
        assert_eq!(task272b1_goal_text(step.before()), *expected[index].0);
        assert_eq!(task272b1_goal_text(step.after()), *expected[index].1);
        assert_eq!(step.matched(), matched[index]);
        assert_eq!(step.free(), free[index]);
        assert_eq!(step.avoided(), if index == 1 { vec![e,z] } else { vec![] });
        assert_eq!(step.owner(), steps[0].site());
        assert_eq!(step.context(), steps[0].context());
        let row = original(step.site());
        match index {
            1 => assert_eq!(row.range(), handoff.scope().bindings()[3].range()),
            2 => assert_eq!(row.range(), handoff.scope().bindings()[4].range()),
            _ => assert_eq!(row.kind(), &[K::TheoremItem,K::TheoremItem,K::TheoremItem,
                K::ConditionList,K::Witness,K::ConclusionStatement][index]),
        }
    }
    let given = &handoff.scope().bindings()[4];
    assert_eq!(given.kind(), B::GivenWitness);
    assert_eq!(given.declared_type().unwrap().radix(), R::Object);
    assert!(given.captures().is_empty());
    let actual = handoff.binding_env().bindings().get(w).unwrap();
    assert_eq!(actual.kind, BindingKind::GivenWitness);
    assert_eq!(actual.type_site, BindingTypeSite::Missing);
    assert!(actual.captured.identities().is_empty());
    let condition = original(steps[3].site());
    assert!(rows.iter().any(|row| Some(row.site()) == condition.parent() && row.kind() == &K::GivenStatement));
    let (intents, labels) = handoff.check_proof_intents().unwrap();
    assert!(!intents.iter().any(|(site,_,_)| site == steps[3].site()));
    assert!(intents.iter().any(|(site,_,status)| site == steps[5].site() && *status == S::Pending));
    assert_eq!(labels.index().iter().filter(|label| label.kind() == LabelKind::ProofStep).count(), 1);
    assert!(labels.index().iter().any(|label| label.kind() == LabelKind::Theorem));
    let [id] = labels.ids() else { panic!("one authentic Given condition citation") };
    let LabelResolution::Resolved(label) = labels.table().get(*id).unwrap().resolution()
        else { panic!("resolved Given condition") };
    assert_eq!(label.kind(), LabelKind::ProofStep);
    let declaration = labels.index().get(label.origin()).unwrap();
    assert_eq!(declaration.primary_spelling(), "A1");
    let mizar_session::SourceAnchor::Range(range) = declaration.origin().anchor()
        else { panic!("original label token") };
    assert!(condition.range().start <= range.start && range.end <= condition.range().end);
    assert!(range.end < label.range().start);
    assert!(rows.iter().any(|row| row.kind() == &K::Reference && row.range() == label.range()));
    assert_eq!(success.len(), 1);
    assert!(pending.is_empty() && checked.initial_obligations().is_empty());
    let input = &success[0];
    assert_eq!(&input.site, steps[4].site());
    assert_eq!(input.source_range, original(steps[4].site()).range());
    assert_eq!(input.kind, CoercionRequestKind::Widening);
    assert_eq!(input.justification, CoercionJustification::Omitted);
    assert_eq!(input.evidence, CoercionEvidence::BuiltinRadix);
    assert_eq!(input.from_type.as_ref().unwrap().head, TypeHeadInput::BuiltinObject);
    assert_eq!(input.to_type.head, TypeHeadInput::BuiltinObject);
    assert_eq!(checked.coercions().len(), 1);
    assert_eq!(checked.coercions().iter().next().unwrap().1.status, CoercionStatus::Candidate);
    assert_eq!(checked.facts().len(), 1);
    assert!(checked.facts().iter().all(|(_,fact)| fact.subject == input.site && fact.status == FactStatus::Known));
    for (_,term) in inference.terms().iter() {
        let Some(TermReference::Binding(binding)) = term.reference else { panic!("real variable") };
        assert_eq!(term.status, TermStatus::Inferred);
        let ty = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(ty.status, TypeStatus::Known);
        let TypeEntryActual::Known(id) = ty.actual else { panic!("actual declaration type") };
        assert_eq!(inference.normalized_types().get(id).unwrap().head,
            if [h,p].contains(&binding) { TypeHeadRef::BuiltinSet } else { TypeHeadRef::BuiltinObject });
    }
    for reference in handoff.scope().references() {
        let binding = task272b1_binding(&handoff, reference.binding().index());
        assert!(inference.terms().iter().any(|(_,term)| term.reference == Some(TermReference::Binding(binding))
            && original(&term.site).range() == reference.range()));
    }
    for binding in [h,e,z,p,w] {
        assert!(inference.terms().iter().any(|(_,term)| term.reference == Some(TermReference::Binding(binding))
            && matches!(&term.site,TypedSiteRef::Role {role,..} if role.as_str() == "witness-goal.declaration-value")
            && original(&term.site).range() == handoff.binding_env().bindings().get(binding).unwrap().declaration_range));
    }
    for (_,formula) in inference.formulas().iter() {
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.terms.len(), 2);
        assert_eq!(original(&formula.site).range(), formula.source_range);
        assert_eq!(original(&formula.site).kind(), &K::BuiltinPredicateApplication);
    }
    assert!(checked.diagnostics().is_empty() && inference.diagnostics().is_empty());
    assert!(handoff.source().types().is_empty() && handoff.source().facts().is_empty()
        && handoff.source().initial_obligations().is_empty());
    super::native_witness_goal_observation(&handoff).unwrap();
}

#[test]
fn task272b6_given_values_keep_alias_identity_actual_types_and_pending_body() {
    use mizar_checker::{type_checker::{TermReference, TypeHeadInput, TypeHeadRef},
        typed_ast::{FactStatus, InitialObligationStatus, TypeEntryActual}};
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_given_existential_assumption_001.miz"
    );
    for source in [
        base.replace("take y0;", "set alias = y0; take alias;")
            .replace("thus y0 in A", "thus alias in A"),
        base.replace("take y0;", "take alias = y0;")
            .replace("thus y0 in A", "thus alias in A"),
    ] {
        let handoff = task272b1_witness_source(&source);
        let (steps, success, pending, _, inference) = handoff.check_witness_goals().unwrap();
        let alias = task272b3_assert_alias_type(&handoff, &inference, 5, TypeHeadRef::BuiltinObject);
        let witness = task272b1_binding(&handoff, 4);
        let entry = handoff.binding_env().bindings().get(alias).unwrap();
        let taken = if source.contains("set alias") {
            assert_eq!(entry.captured.identities(), &[handoff.binding_env().bindings().get(witness).unwrap().identity.clone()]);
            alias
        } else {
            assert!(entry.captured.identities().is_empty());
            witness
        };
        assert_eq!(success.len(), 1);
        assert!(pending.is_empty());
        assert!(steps.iter().any(|step| step.matched() == Some((task272b1_binding(&handoff, 2), taken))));
        assert_eq!(task272b1_goal_text(steps.last().unwrap().after()),
            format!("member({},{})", witness.index(), task272b1_binding(&handoff, 3).index()));
    }
    let source = base.replace("ex z being object", "ex z being set");
    let handoff = task272b1_witness_source(&source);
    let (steps, success, pending, checked, inference) = handoff.check_witness_goals().unwrap();
    let witness = task272b1_binding(&handoff, 4);
    assert!(success.is_empty());
    assert_eq!(inference.terms().len(), 14);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].site, *steps[4].site());
    assert_eq!(pending[0].target_type.head, TypeHeadInput::BuiltinSet);
    assert_eq!(steps[4].after(), steps[5].before());
    assert_eq!(steps[4].after(), steps[5].after());
    assert_eq!(task272b1_goal_text(steps[5].after()),
        format!("member({},{})", witness.index(), task272b1_binding(&handoff, 3).index()));
    for (_,term) in inference.terms().iter().filter(|(_,term)| term.reference == Some(TermReference::Binding(witness))) {
        let TypeEntryActual::Known(id) = inference.type_entries().get(term.type_entry).unwrap().actual
            else { panic!("Given remains actual Object") };
        assert_eq!(inference.normalized_types().get(id).unwrap().head, TypeHeadRef::BuiltinObject);
    }
    let obligation = checked.initial_obligations().iter().next().unwrap().1;
    assert_eq!(obligation.owner, *steps[4].site());
    assert_eq!(obligation.status, InitialObligationStatus::Pending);
    assert_eq!(checked.facts().len(), 1);
    assert!(checked.facts().iter().all(|(_,fact)| fact.status == FactStatus::PendingObligation
        && !fact.status.is_unconditionally_consumable()));
    assert!(super::native_witness_goal_observation(&handoff).unwrap().iter().any(|line|
        line.starts_with("remaining.") && line.contains(&format!("pending=[{:?}]", obligation.id))));
}

#[test]
fn task272b6_given_and_membership_enforce_freshness_domains_and_whole_owners() {
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_given_existential_assumption_001.miz"
    );
    for source in [
        base.replace("y0", "y").replace("A1", "Premise"),
        base.replace("ex y being object", "ex y being set").replace("given y0 being object", "given y0 being set"),
        base.replace("A1: ", "").replace(" by A1", ""),
        "theorem Retained: for A being set st (ex y being object st for q being object holds y in A) holds ex z being object st z in A proof let A be set; given y0 being object such that H:for r being object holds y0 in A; take y0; thus y0 in A by H; end;".to_owned(),
        base.replace("ex z being object st z in A", "ex z being object st A in A")
            .replace("thus y0 in A", "thus A in A"),
    ] {
        let handoff = task272b1_witness_source(&source);
        let (steps, _, _, _, _) = handoff.check_witness_goals().unwrap_or_else(|error| panic!("{source}: {error:?}"));
        if source.contains("theorem Retained:") {
            assert_eq!(steps[2].avoided(), [task272b1_binding(&handoff, 2)]);
        }
        if source.contains("given y being") {
            assert_ne!(steps[2].matched().unwrap().0, steps[2].matched().unwrap().1);
        }
        assert_eq!(steps.last().unwrap().before(), steps.last().unwrap().after());
    }
    for source in [
        base.replace("given y0 being object", "given y0 being set"),
        base.replace("ex y being object", "ex y being set"),
        base.replace("such that A1: y0 in A", "such that A1: y0 = A"),
        base.replace("such that A1: y0 in A", "such that A1: y0 in A and A2: y0 in A"),
        base.replace("given y0 being object", "given y0, y1 being object"),
        base.replace("for A being set", "for A being object").replace("let A be set", "let A be object"),
        base.replace("thus y0 in A by A1;", "thus y0 in A by Missing;"),
        base.replace("take y0;", "set unused = {y0}; take y0;"),
        "theorem WrongFree: for A,B being set st (ex y being object st y in A) holds ex z being object st z in A proof let A,B be set; given y0 being object such that H:y0 in B; take y0; thus y0 in A by H; end;".to_owned(),
        "theorem Context: for A being set holds ex z being object st z in A proof let A be set; given y0 being object such that H:y0 in A; take y0; thus y0 in A by H; end;".to_owned(),
        "theorem DerivedRight: for A being set st (ex y being object st y in A) holds ex z being set st A in z proof let A be set; given y0 being object such that H:y0 in A; take y0; thus thesis by H; end;".to_owned(),
    ] {
        assert!(task272b1_witness_source(&source).check_witness_goals().is_err(), "{source}");
    }
    let two = format!("{}\n{}", base.replace("Given1", "First"), base.replace("Given1", "Second"));
    let handoff = task272b1_witness_source(&two);
    let (steps, success, pending, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 12);
    assert_eq!(inference.terms().len(), 28);
    assert_eq!(inference.formulas().len(), 8);
    assert_eq!(success.len(), 2);
    assert!(pending.is_empty());
    assert_ne!(steps[0].owner(), steps[6].owner());
    assert_ne!(steps[0].context(), steps[6].context());
    assert!(steps[..6].iter().all(|step| step.owner() == steps[0].owner()));
    assert!(steps[6..].iter().all(|step| step.owner() == steps[6].owner()));
    assert!(task272b1_witness_source(&two.replacen("such that A1: y0 in A", "such that A1: y0 = A", 1))
        .check_witness_goals().is_err());
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == super::NATIVE_WITNESS_GOAL_ID).unwrap();
    for source in [
        base.replace("given y0 being object such that A1: y0 in A;\n  take y0;",
            "take y0; given y0 being object such that A1: y0 in A;"),
        format!("{}\n{}", base.replace("Given1", "First"), base.replace("Given1", "Second")
            .replace("given y0 being object such that A1: y0 in A;", "")),
    ] {
        let frontend = metamorphic_guard::with_source(case, &source, || super::run_frontend(&config.workspace_root, case, 0)).unwrap();
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend.ast.unwrap();
        let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        assert!(super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).is_err(), "{source}");
    }
}

#[test]
fn task272b6_given_native_gate_controls_actual_fs_result() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|
        case.id.0 == "pass_formula_statement_given_existential_assumption_001").unwrap();
    let source = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_given_existential_assumption_001.miz"
    ).replace("thus y0 in A by A1;", "Inner: now thus y0 in A by A1; end; thus y0 in A by Inner;");
    let frontend = metamorphic_guard::with_source(case, &source, || super::run_frontend(&config.workspace_root, case, 0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    assert_eq!(super::formula_statement::check_formula_ast_with_organization(&ast, &resolver.module, &resolver.env, true), Ok(true));
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(case, &source, ||
        super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0));
    assert_eq!(result.status, super::FormulaStatementCaseStatus::Failed);
    assert!(result.actual_detail_keys.iter().any(|key| key.contains("native witness goal")));
}

#[test]
fn task272b7_hereby_goals_execute_existing_fs() {
    use mizar_checker::{
        binding_env::{BindingContextLayer as L, BindingContextOwner, BindingContextRecovery},
        source_proof_local_declaration::SourceProofIntentStatus as S,
        type_checker::{FormulaKind, FormulaStatus, TermReference, TermStatus, TypeHeadRef},
        typed_ast::{TypeEntryActual, TypeStatus, TypedSiteRef},
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_formula_statement_hereby_diffuse_conclusion_001")
        .unwrap();
    assert_eq!(
        super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0)
            .status,
        super::FormulaStatementCaseStatus::Passed
    );
    let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    let ast = frontend.ast.expect("genuine existing Hereby1 source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty(), "{:?}", resolver.detail_keys);
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    println!(
        "Task272B7 test-first: unchanged FS semantic engine and native A pass; genuine frontend/symbol/native/scope handoff succeeds"
    );
    let (steps, success, pending, checked, inference) = handoff
        .check_witness_goals()
        .expect("actual Hereby1 FS source must materialize inherited block goals");
    assert_eq!(steps.len(), 4);
    assert_eq!(inference.terms().len(), 6);
    assert_eq!(inference.formulas().len(), 2);
    let (header, local) = (task272b1_binding(&handoff, 0), task272b1_binding(&handoff, 1));
    assert_ne!(header, local);
    assert_eq!(handoff.scope().bindings().len(), 2);
    assert_eq!(handoff.scope().references().iter().map(|reference|
        task272b1_binding(&handoff, reference.binding().index())).collect::<Vec<_>>(),
        [header, header, local, local]);
    let g0 = format!("forall({}:Set,eq({0},{0}))", header.index());
    let g1 = format!("eq({0},{0})", local.index());
    let expected = [(&g0,&g0),(&g0,&g1),(&g1,&g1),(&g1,&g1)];
    let rows = handoff.source().source_node_entries().unwrap().collect::<Vec<_>>();
    let original = |site: &TypedSiteRef| {
        let id = handoff.source().nodes().node(site.node()).unwrap().resolved_node.unwrap();
        rows.iter().copied().find(|row| row.site() == id).unwrap()
    };
    let outer = steps[0].context();
    let inner = steps[2].context();
    assert_ne!(outer, inner);
    let contexts = handoff.binding_env().contexts();
    assert_eq!(contexts.get(outer).unwrap().layer, L::Proof);
    let block = contexts.get(inner).unwrap();
    assert_eq!(block.layer, L::Block);
    assert_eq!(block.parent, Some(outer));
    assert_eq!(block.owner, BindingContextOwner::SourceStatement { source_range: original(steps[2].site()).range() });
    assert_eq!(block.recovery, BindingContextRecovery::Normal);
    assert!(block.bindings.is_empty());
    assert!(block.visible_bindings.contains(&local));
    let native = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &resolver.module).unwrap();
    let (scopes, _) = mizar_resolve::symbols::source_proof_scope_maps(&native, &resolver.env, handoff.scope()).unwrap();
    let hereby = ast.node_views().find(|node| node.kind() == &K::HerebyStatement).unwrap();
    assert_eq!(block.lexical_scope.as_ref(), Some(scopes.get(&hereby.id()).unwrap()));
    assert_eq!(contexts.iter().filter(|(_, context)| context.lexical_scope == block.lexical_scope).count(), 1);
    for (index,step) in steps.iter().enumerate() {
        assert_eq!(task272b1_goal_text(step.before()), *expected[index].0);
        assert_eq!(task272b1_goal_text(step.after()), *expected[index].1);
        assert_eq!(step.owner(), steps[0].site());
        assert_eq!(step.context(), if index < 2 { outer } else { inner });
        assert_eq!(step.matched(), if index == 1 { Some((header,local)) } else { None });
        assert_eq!(step.free(), if index == 0 { vec![] } else { vec![local] });
        assert!(step.avoided().is_empty());
        let row = original(step.site());
        if index == 1 { assert_eq!(row.range(), handoff.scope().bindings()[1].range()); }
        else { assert_eq!(row.kind(), &[K::TheoremItem,K::TheoremItem,K::HerebyStatement,K::ConclusionStatement][index]); }
    }
    assert_eq!(original(steps[3].site()).parent(), Some(original(steps[2].site()).site()));
    assert_eq!(steps[2].before(), steps[2].after());
    assert_eq!(steps[3].before(), steps[3].after());
    let (intents, labels) = handoff.check_proof_intents().unwrap();
    assert!(labels.ids().is_empty());
    for step in &steps[2..] {
        assert!(intents.iter().any(|(site,_,status)| site == step.site() && *status == S::Pending));
    }
    assert!(success.is_empty() && pending.is_empty());
    assert!(checked.coercions().is_empty() && checked.initial_obligations().is_empty() && checked.facts().is_empty());
    assert!(checked.diagnostics().is_empty() && inference.diagnostics().is_empty());
    for (_,term) in inference.terms().iter() {
        assert_eq!(term.status, TermStatus::Inferred);
        let ty = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(ty.status, TypeStatus::Known);
        let TypeEntryActual::Known(id) = ty.actual else { panic!("genuine known Set") };
        assert_eq!(inference.normalized_types().get(id).unwrap().head, TypeHeadRef::BuiltinSet);
    }
    for reference in handoff.scope().references() {
        let binding = task272b1_binding(&handoff, reference.binding().index());
        let context = if binding == local { inner } else { handoff.binding_env().bindings().get(header).unwrap().owner_context };
        assert!(inference.terms().iter().any(|(_,term)| term.reference == Some(TermReference::Binding(binding))
            && original(&term.site).range() == reference.range() && term.context == context));
    }
    for binding in [header,local] {
        let entry = handoff.binding_env().bindings().get(binding).unwrap();
        assert!(inference.terms().iter().any(|(_,term)| term.reference == Some(TermReference::Binding(binding))
            && matches!(&term.site,TypedSiteRef::Role {role,..} if role.as_str() == "witness-goal.declaration-value")
            && original(&term.site).range() == entry.declaration_range && term.context == entry.owner_context));
    }
    for (_,formula) in inference.formulas().iter() {
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(original(&formula.site).range(), formula.source_range);
    }
    assert!(handoff.source().types().is_empty() && handoff.source().facts().is_empty()
        && handoff.source().initial_obligations().is_empty());
    super::native_witness_goal_observation(&handoff).unwrap();
}

#[test]
fn task272b7_hereby_inherits_supported_goals_aliases_and_occurrence_contexts() {
    use mizar_checker::{binding_env::BindingContextLayer as L,
        type_checker::{TermReference, TypeHeadRef}, typed_ast::{InitialObligationStatus, TypeEntryActual}};
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_hereby_diffuse_conclusion_001.miz"
    );
    for source in [
        base.replace("thus X = X;", "thus thesis;"),
        base.replace("X", "Value"),
        base.replace("let X be set;", "let X be set; set alias = X;").replace("thus X = X;", "thus alias = X;"),
        "theorem Whole: for X being set holds X=X & X=X proof let X be set; hereby thus X=X & X=X; end; end;".to_owned(),
        "theorem Residual: for X being set holds ex y being object st y=X proof let X be set; hereby thus thesis; end; end;".to_owned(),
        "theorem Quantified: for X being set holds for y being set holds y=X proof let X be set; hereby thus for z being set holds z=X; end; end;".to_owned(),
    ] {
        let handoff = task272b1_witness_source(&source);
        let (steps, _, _, _, inference) = handoff.check_witness_goals().unwrap_or_else(|error| panic!("{source}: {error:?}"));
        let block = steps[steps.len()-2].context();
        assert_eq!(handoff.binding_env().contexts().get(block).unwrap().layer, L::Block);
        assert_eq!(steps.last().unwrap().context(), block);
        assert_eq!(steps.last().unwrap().before(), steps.last().unwrap().after());
        if source.contains("set alias") {
            task272b3_assert_alias_type(&handoff, &inference, 2, TypeHeadRef::BuiltinSet);
        }
        if source.contains("theorem Quantified:") {
            let child = task272b1_binding(&handoff, 3);
            let terms = inference.terms().iter().filter(|(_,term)| term.reference == Some(TermReference::Binding(child))).collect::<Vec<_>>();
            assert_eq!(terms.len(), 2);
            for (_,term) in terms {
                let context = handoff.binding_env().contexts().get(term.context).unwrap();
                assert_eq!(context.layer, L::Expression);
                assert_eq!(context.parent, Some(block));
            }
        }
    }
    let given = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_given_existential_assumption_001.miz"
    );
    let alias_source = given.replace("take y0;", "set alias = y0; take alias;")
        .replace("thus y0 in A by A1;", "hereby thus alias in A; end;");
    let handoff = task272b1_witness_source(&alias_source);
    let (steps, _, _, _, inference) = handoff.check_witness_goals().unwrap();
    let alias = task272b3_assert_alias_type(&handoff, &inference, 5, TypeHeadRef::BuiltinObject);
    let block = handoff.binding_env().contexts().get(steps.last().unwrap().context()).unwrap();
    assert!(block.visible_bindings.contains(&alias));
    assert!(block.visible_bindings.contains(&task272b1_binding(&handoff, 4)));
    let pending_source = given.replace("ex z being object", "ex z being set")
        .replace("thus y0 in A by A1;", "hereby thus y0 in A; end;");
    let handoff = task272b1_witness_source(&pending_source);
    let (steps, success, pending, checked, inference) = handoff.check_witness_goals().unwrap();
    assert!(success.is_empty());
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].site, *steps[4].site());
    assert_ne!(steps[4].context(), steps.last().unwrap().context());
    assert_eq!(steps[4].owner(), steps.last().unwrap().owner());
    assert_eq!(steps[4].after(), steps.last().unwrap().after());
    let obligation = checked.initial_obligations().iter().next().unwrap().1;
    assert_eq!(obligation.owner, *steps[4].site());
    assert_eq!(obligation.status, InitialObligationStatus::Pending);
    let witness = task272b1_binding(&handoff, 4);
    for (_,term) in inference.terms().iter().filter(|(_,term)| term.reference == Some(TermReference::Binding(witness))) {
        let TypeEntryActual::Known(id) = inference.type_entries().get(term.type_entry).unwrap().actual
            else { panic!("Hereby preserves actual Object") };
        assert_eq!(inference.normalized_types().get(id).unwrap().head, TypeHeadRef::BuiltinObject);
    }
    assert!(super::native_witness_goal_observation(&handoff).unwrap().iter().any(|line|
        line.starts_with("remaining.") && line.contains(&format!("pending=[{:?}]", obligation.id))));
}

#[test]
fn task272b7_hereby_refuses_partial_extra_and_foreign_source_effects() {
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_hereby_diffuse_conclusion_001.miz"
    );
    for source in [
        base.replace("holds X = X", "holds X = X & X = X"),
        base.replace("thus X = X;", "thus X in X;"),
        base.replace("thus X = X;", "hence X = X;"),
        base.replace("thus X = X;", "then X = X;"),
        base.replace("thus X = X;", "thus A1: X = X;"),
        base.replace("thus X = X;", "let Y be set; thus X = X;"),
        base.replace("thus X = X;", "assume A1: X = X; thus X = X;"),
        base.replace("thus X = X;", "take X; thus X = X;"),
        base.replace("thus X = X;", "hereby thus X = X; end;"),
        base.replace("thus X = X;", "thus X = X; thus X = X;"),
        base.replace("  end;\nend;", "  end;\n  thus X = X;\nend;"),
        "theorem WrongFree: for X,Y being set holds X=X proof let X,Y be set; hereby thus Y=Y; end; end;".to_owned(),
        "theorem ResidualImp: for X being set holds X=X implies X=X proof let X be set; hereby thus thesis; end; end;".to_owned(),
        "theorem NestedImp: for X being set holds for y being set holds y=y implies X=X proof let X be set; hereby thus thesis; end; end;".to_owned(),
    ] {
        assert!(task272b1_witness_source(&source).check_witness_goals().is_err(), "{source}");
    }
    let two = format!("{}\n{}", base.replace("Hereby1", "First"), base.replace("Hereby1", "Second"));
    let handoff = task272b1_witness_source(&two);
    let (steps, _, _, _, inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 8);
    assert_eq!(inference.terms().len(), 12);
    assert_eq!(inference.formulas().len(), 4);
    assert_ne!(steps[0].owner(), steps[4].owner());
    assert_ne!(steps[2].context(), steps[6].context());
    assert!(steps[..4].iter().all(|step| step.owner() == steps[0].owner()));
    assert!(steps[4..].iter().all(|step| step.owner() == steps[4].owner()));
    assert!(task272b1_witness_source(&two.replacen("thus X = X;", "thus X in X;", 1)).check_witness_goals().is_err());
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == super::NATIVE_WITNESS_GOAL_ID).unwrap();
    let source = "theorem Future: for X being set holds X=X proof let X be set; hereby thus Y=Y; end; let Y be set; end;";
    let frontend = metamorphic_guard::with_source(case, source, || super::run_frontend(&config.workspace_root, case, 0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    assert!(super::native_conversion_handoff(&ast, &resolver.module, &resolver.env)
        .and_then(|handoff| handoff.check_witness_goals().map_err(|_| "future Hereby value".to_owned())).is_err());
}

#[test]
fn task272b7_hereby_native_gate_controls_actual_fs_result() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|
        case.id.0 == "pass_formula_statement_hereby_diffuse_conclusion_001").unwrap();
    let source = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_hereby_diffuse_conclusion_001.miz"
    ).replace("for X being set holds X = X", "for X being set st X = X holds X = X")
        .replace("let X be set;", "let X be set; assume A1: X = X;")
        .replace("thus X = X;", "thus X = X by A1;");
    let frontend = metamorphic_guard::with_source(case, &source, || super::run_frontend(&config.workspace_root, case, 0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    assert_eq!(super::formula_statement::check_formula_ast_with_organization(&ast, &resolver.module, &resolver.env, true), Ok(true));
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(case, &source, ||
        super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0));
    assert_eq!(result.status, super::FormulaStatementCaseStatus::Failed);
    assert!(result.actual_detail_keys.iter().any(|key| key.contains("native witness goal")));
}

#[test]
fn task272b8_now_goals_execute_existing_fs() {
    use mizar_checker::{binding_env::{BindingContextLayer as L, BindingContextOwner, BindingContextRecovery},
        source_proof_local_declaration::SourceProofIntentStatus as S,
        type_checker::{FormulaKind, FormulaStatus, TermReference, TermStatus, TypeHeadRef},
        typed_ast::{TypeEntryActual, TypeStatus, TypedSiteRef}};
    use mizar_resolve::resolved_ast::{LabelKind, LabelResolution};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|
        case.id.0 == "pass_formula_statement_now_diffuse_statement_001").unwrap();
    assert_eq!(super::formula_statement::run_formula_statement_case(&config.workspace_root, case, 0).status,
        super::FormulaStatementCaseStatus::Passed);
    let frontend = super::run_frontend(&config.workspace_root, case, 0).unwrap();
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    let ast = frontend.ast.expect("genuine existing Diffuse1 source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty(), "{:?}", resolver.detail_keys);
    let handoff = super::native_conversion_handoff(&ast, &resolver.module, &resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    println!("Task272B8 test-first: unchanged FS semantic engine and native A pass; genuine frontend/symbol/native/scope handoff succeeds");
    let (steps, success, pending, checked, inference) = handoff.check_witness_goals()
        .expect("actual Diffuse1 FS source must materialize Now/citation goals");
    assert_eq!(steps.len(), 5);
    assert_eq!(inference.terms().len(), 8);
    assert_eq!(inference.formulas().len(), 3);
    let (header, local) = (task272b1_binding(&handoff, 0), task272b1_binding(&handoff, 1));
    assert_ne!(header, local);
    assert_eq!(handoff.scope().bindings().len(), 2);
    assert_eq!(handoff.scope().references().iter().map(|reference|
        task272b1_binding(&handoff, reference.binding().index())).collect::<Vec<_>>(),
        [header,header,local,local,local,local]);
    let g0 = format!("forall({}:Set,eq({0},{0}))", header.index());
    let g1 = format!("eq({0},{0})", local.index());
    let expected = [(&g0,&g0),(&g0,&g1),(&g1,&g1),(&g1,&g1),(&g1,&g1)];
    let rows = handoff.source().source_node_entries().unwrap().collect::<Vec<_>>();
    let original = |site: &TypedSiteRef| {
        let id = handoff.source().nodes().node(site.node()).unwrap().resolved_node.unwrap();
        rows.iter().copied().find(|row| row.site() == id).unwrap()
    };
    let outer = steps[0].context();
    let inner = steps[2].context();
    assert_ne!(outer, inner);
    assert_eq!(handoff.binding_env().contexts().get(outer).unwrap().layer, L::Proof);
    let block = handoff.binding_env().contexts().get(inner).unwrap();
    assert_eq!(block.layer, L::Block);
    assert_eq!(block.parent, Some(outer));
    assert_eq!(block.owner, BindingContextOwner::SourceStatement {source_range: original(steps[2].site()).range()});
    assert_eq!(block.recovery, BindingContextRecovery::Normal);
    assert!(block.bindings.is_empty() && block.visible_bindings.contains(&local));
    let native = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &resolver.module).unwrap();
    let (scopes, _) = mizar_resolve::symbols::source_proof_scope_maps(&native, &resolver.env, handoff.scope()).unwrap();
    let now = ast.node_views().find(|node| node.kind() == &K::NowStatement).unwrap();
    assert_eq!(block.lexical_scope.as_ref(), Some(scopes.get(&now.id()).unwrap()));
    assert_eq!(handoff.binding_env().contexts().iter().filter(|(_,context)| context.lexical_scope == block.lexical_scope).count(), 1);
    for (index,step) in steps.iter().enumerate() {
        assert_eq!(task272b1_goal_text(step.before()), *expected[index].0);
        assert_eq!(task272b1_goal_text(step.after()), *expected[index].1);
        assert_eq!(step.owner(), steps[0].site());
        assert_eq!(step.context(), if [2,3].contains(&index) { inner } else { outer });
        assert_eq!(step.matched(), if index == 1 { Some((header,local)) } else { None });
        assert_eq!(step.free(), if index == 0 { vec![] } else { vec![local] });
        assert!(step.avoided().is_empty());
        let row = original(step.site());
        if index == 1 { assert_eq!(row.range(), handoff.scope().bindings()[1].range()); }
        else { assert_eq!(row.kind(), &[K::TheoremItem,K::TheoremItem,K::NowStatement,K::ConclusionStatement,K::ConclusionStatement][index]); }
    }
    assert_eq!(original(steps[3].site()).parent(), Some(original(steps[2].site()).site()));
    let (intents, labels) = handoff.check_proof_intents().unwrap();
    for step in &steps[2..] {
        assert!(intents.iter().any(|(site,_,status)| site == step.site() && *status == S::Pending));
    }
    assert_eq!(labels.index().iter().filter(|label| label.kind() == LabelKind::ProofStep).count(), 1);
    assert!(labels.index().iter().any(|label| label.kind() == LabelKind::Theorem));
    let [id] = labels.ids() else { panic!("one genuine Now reference") };
    let LabelResolution::Resolved(label) = labels.table().get(*id).unwrap().resolution()
        else { panic!("resolved authentic Now claim") };
    assert_eq!(label.kind(), LabelKind::ProofStep);
    let declaration = labels.index().get(label.origin()).unwrap();
    assert_eq!(declaration.primary_spelling(), "A1");
    let mizar_session::SourceAnchor::Range(range) = declaration.origin().anchor()
        else { panic!("actual Now label token") };
    assert_eq!(range.start, original(steps[2].site()).range().start);
    assert!(original(steps[2].site()).range().end <= label.range().start);
    assert!(rows.iter().any(|row| row.kind() == &K::Reference && row.range() == label.range()));
    assert_eq!(declaration.origin().source_id(), labels.table().get(*id).unwrap().origin().source_id());
    assert!(success.is_empty() && pending.is_empty());
    assert!(checked.coercions().is_empty() && checked.initial_obligations().is_empty() && checked.facts().is_empty());
    for (_,term) in inference.terms().iter() {
        assert_eq!(term.status, TermStatus::Inferred);
        let ty = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(ty.status, TypeStatus::Known);
        let TypeEntryActual::Known(id) = ty.actual else { panic!("actual known Set") };
        assert_eq!(inference.normalized_types().get(id).unwrap().head, TypeHeadRef::BuiltinSet);
    }
    for (index,reference) in handoff.scope().references().iter().enumerate() {
        let binding = task272b1_binding(&handoff, reference.binding().index());
        let context = if index < 2 { handoff.binding_env().bindings().get(header).unwrap().owner_context }
            else if index < 4 { inner } else { outer };
        assert!(inference.terms().iter().any(|(_,term)| term.reference == Some(TermReference::Binding(binding))
            && original(&term.site).range() == reference.range() && term.context == context));
    }
    for binding in [header,local] {
        let entry = handoff.binding_env().bindings().get(binding).unwrap();
        assert!(inference.terms().iter().any(|(_,term)| term.reference == Some(TermReference::Binding(binding))
            && matches!(&term.site,TypedSiteRef::Role {role,..} if role.as_str() == "witness-goal.declaration-value")
            && original(&term.site).range() == entry.declaration_range && term.context == entry.owner_context));
    }
    for (_,formula) in inference.formulas().iter() {
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(original(&formula.site).range(), formula.source_range);
    }
    assert!(checked.diagnostics().is_empty() && inference.diagnostics().is_empty());
    assert!(handoff.source().types().is_empty() && handoff.source().facts().is_empty()
        && handoff.source().initial_obligations().is_empty());
    super::native_witness_goal_observation(&handoff).unwrap();
}

#[test]
fn task272b8_pending_guard_observation_uses_owner_and_prior_step_prefix() {
    use mizar_checker::typed_ast::InitialObligationStatus;
    let before = "theorem Before: for o being object ex z being set st z=o proof let o be object; N: now thus thesis; end; take o; thus thesis by N; end;";
    let prior = "theorem Prior: for o being object ex z being set st z=o proof let o be object; take o; N: now thus thesis; end; thus thesis by N; end;";
    for (source,take,terminals,expected_pending) in [
        (before,4,[3,5],[false,true]),
        (prior,2,[4,5],[true,true]),
    ] {
        let handoff = task272b1_witness_source(source);
        let (steps,success,pending,checked,_) = handoff.check_witness_goals().unwrap();
        assert_eq!(steps.len(), 6);
        assert!(success.is_empty() && checked.coercions().is_empty());
        assert_eq!(pending.len(), 1);
        assert_eq!(checked.initial_obligations().len(), 1);
        assert_eq!(pending[0].site, *steps[take].site());
        let obligation = checked.initial_obligations().iter().next().unwrap().1;
        assert_eq!(obligation.owner, *steps[take].site());
        assert_eq!(obligation.status, InitialObligationStatus::Pending);
        let local = task272b1_binding(&handoff, 2);
        assert_eq!(task272b1_goal_text(steps[take].after()), format!("eq({0},{0})",local.index()));
        if take == 4 {
            assert_eq!(task272b1_goal_text(steps[3].after()), format!("exists({}:Set,eq({},{}))",
                task272b1_binding(&handoff, 1).index(),task272b1_binding(&handoff, 1).index(),local.index()));
        }
        let observed = super::native_witness_goal_observation(&handoff).unwrap();
        for (index,expected) in terminals.into_iter().zip(expected_pending) {
            let line = observed.iter().find(|line| line.starts_with("remaining.")
                && line.contains(&format!("terminal={:?}",steps[index].site()))).unwrap();
            assert!(line.ends_with(&if expected { format!("pending=[{:?}]",obligation.id) }
                else { "pending=[]".to_owned() }), "{line}");
        }
    }
    let later = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_now_diffuse_statement_001.miz"
    ).replace("Diffuse1", "Later");
    let handoff = task272b1_witness_source(&format!("{before}\n{later}"));
    let (steps,_,pending,checked,_) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(), 11);
    assert_eq!(pending.len(), 1);
    assert_eq!(checked.initial_obligations().len(), 1);
    assert_ne!(steps[5].owner(), steps[9].owner());
    let observed = super::native_witness_goal_observation(&handoff).unwrap();
    for index in [9,10] {
        let line = observed.iter().find(|line| line.starts_with("remaining.")
            && line.contains(&format!("terminal={:?}",steps[index].site()))).unwrap();
        assert!(line.ends_with("pending=[]"), "{line}");
    }
}

#[test]
fn task272b8_now_keeps_candidate_claim_contexts_aliases_and_parent_continuation() {
    use mizar_checker::{binding_env::BindingContextLayer as L, type_checker::{TermReference,TypeHeadRef}};
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_now_diffuse_statement_001.miz"
    );
    for source in [
        base.replace("X", "Value").replace("A1", "Claim"),
        base.replace("thus X = X;", "thus thesis;"),
        base.replace("let X be set;", "let X be set; set alias = X;").replace("thus X = X", "thus alias = X"),
        base.replace("thus X = X by A1;", "hereby thus X = X; end;"),
        "theorem Quantified: for X being set holds for y being set holds y=X proof let X be set; N:now thus for z being set holds z=X; end; thus for r being set holds r=X by N; end;".to_owned(),
    ] {
        let handoff = task272b1_witness_source(&source);
        let (steps,_,_,checked,inference) = handoff.check_witness_goals().unwrap_or_else(|error| panic!("{source}: {error:?}"));
        assert_eq!(steps[2].before(),steps[2].after());
        assert_eq!(steps[3].before(),steps[3].after());
        assert!(checked.facts().is_empty());
        if source.contains("set alias") {
            task272b3_assert_alias_type(&handoff,&inference,2,TypeHeadRef::BuiltinSet);
        }
        if source.contains("theorem Quantified:") {
            let child = task272b1_binding(&handoff,3);
            let terms = inference.terms().iter().filter(|(_,term)| term.reference == Some(TermReference::Binding(child))).collect::<Vec<_>>();
            assert_eq!(terms.len(),2);
            for (_,term) in terms {
                let context = handoff.binding_env().contexts().get(term.context).unwrap();
                assert_eq!(context.layer,L::Expression);
                assert_eq!(context.parent,Some(steps[2].context()));
            }
        }
    }
    let source = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_given_existential_assumption_001.miz"
    ).replace("take y0;", "set alias = y0; take alias;")
        .replace("thus y0 in A by A1;", "N: now thus alias in A; end; thus alias in A by N;");
    let handoff = task272b1_witness_source(&source);
    let (steps,_,_,_,inference) = handoff.check_witness_goals().unwrap();
    let alias = task272b3_assert_alias_type(&handoff,&inference,5,TypeHeadRef::BuiltinObject);
    let terminal = steps.last().unwrap();
    let child = &steps[steps.len()-2];
    assert_eq!(handoff.binding_env().contexts().get(terminal.context()).unwrap().layer,L::Proof);
    assert_eq!(handoff.binding_env().contexts().get(child.context()).unwrap().parent,Some(terminal.context()));
    assert!(handoff.binding_env().contexts().get(child.context()).unwrap().visible_bindings.contains(&alias));
    assert_eq!(child.after(),terminal.after());
}

#[test]
fn task272b8_now_refuses_unsupported_claims_labels_and_whole_owner_effects() {
    let base = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_now_diffuse_statement_001.miz"
    );
    for source in [
        base.replace("holds X = X", "holds X = X & X = X"),
        base.replace("    thus X = X;", "    thus X in X;"),
        base.replace("    thus X = X;", "    hence X = X;"),
        base.replace("    thus X = X;", "    then X = X;"),
        base.replace("    thus X = X;", "    thus Inner: X = X;"),
        base.replace("    thus X = X;", "    let Y be set; thus X = X;"),
        base.replace("    thus X = X;", "    assume H: X = X; thus X = X;"),
        base.replace("    thus X = X;", "    take X; thus X = X;"),
        base.replace("    thus X = X;", "    now thus X = X; end;"),
        base.replace("    thus X = X;", "    thus X = X; thus X = X;"),
        base.replace("A1: now", "now").replace(" by A1", ""),
        base.replace("  thus X = X by A1;\n", ""),
        base.replace("by A1", "by Missing"),
        base.replace("by A1", "by Diffuse1"),
        base.replace("by A1", "by A1, A1"),
        base.replace("by A1", "by Imported.A1"),
        base.replace("  thus X = X by A1;", "  thus X = X by Later; Later: now thus X = X; end; thus X = X;"),
        "theorem WrongFree: for X,Y being set holds X=X proof let X,Y be set; N: now thus Y=Y; end; thus X=X by N; end;".to_owned(),
        "theorem ResidualImp: for X being set holds X=X implies X=X proof let X be set; N:now thus thesis; end; thus thesis by N; end;".to_owned(),
    ] {
        assert!(task272b1_witness_source(&source).check_witness_goals().is_err(),"{source}");
    }
    let two = format!("{}\n{}",base.replace("Diffuse1","First"),base.replace("Diffuse1","Second"));
    let handoff = task272b1_witness_source(&two);
    let (steps,_,_,_,inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(),10);
    assert_eq!(inference.terms().len(),16);
    assert_eq!(inference.formulas().len(),6);
    assert_ne!(steps[0].owner(),steps[5].owner());
    assert_ne!(steps[2].context(),steps[7].context());
    assert!(task272b1_witness_source(&two.replacen("    thus X = X;","    thus X in X;",1)).check_witness_goals().is_err());
    let foreign = format!("{}\n{}",base.replace("Diffuse1","First"),base.replace("Diffuse1","Second").replace("A1: now","Other: now"));
    assert!(task272b1_witness_source(&foreign).check_witness_goals().is_err());
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|case.id.0==super::NATIVE_WITNESS_GOAL_ID).unwrap();
    let source = "theorem Future: for X being set holds X=X proof let X be set; N: now thus Y=Y; end; let Y be set; thus X=X by N; end;";
    let frontend = metamorphic_guard::with_source(case,source,||super::run_frontend(&config.workspace_root,case,0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root,case,&ast);
    assert!(resolver.detail_keys.is_empty());
    assert!(super::native_conversion_handoff(&ast,&resolver.module,&resolver.env)
        .and_then(|handoff|handoff.check_witness_goals().map_err(|_|"future Now value".to_owned())).is_err());
}

#[test]
fn task272b8_now_native_gate_controls_actual_fs_result() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|case.id.0=="pass_formula_statement_now_diffuse_statement_001").unwrap();
    let source = include_str!(
        "../../../../tests/miz/pass/theorems/pass_formula_statement_now_diffuse_statement_001.miz"
    ).replace("for X being set holds X = X","for X being set st X = X holds X = X")
        .replace("let X be set;","let X be set; assume Premise: X = X;")
        .replace("    thus X = X;","    thus X = X by Premise;");
    let frontend = metamorphic_guard::with_source(case,&source,||super::run_frontend(&config.workspace_root,case,0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root,case,&ast);
    assert!(resolver.detail_keys.is_empty());
    assert_eq!(super::formula_statement::check_formula_ast_with_organization(&ast,&resolver.module,&resolver.env,true),Ok(true));
    let handoff = super::native_conversion_handoff(&ast,&resolver.module,&resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(case,&source,||super::formula_statement::run_formula_statement_case(&config.workspace_root,case,0));
    assert_eq!(result.status,super::FormulaStatementCaseStatus::Failed);
    assert!(result.actual_detail_keys.iter().any(|key|key.contains("native witness goal")));
}

#[test]
fn task272b9_conjunction_goals_execute_new_source() {
    use mizar_checker::{source_proof_local_declaration::{SourceProofGoal as G,SourceProofIntentStatus as S},
        type_checker::{FormulaKind,FormulaStatus,TermReference,TermStatus,TypeHeadRef},
        typed_ast::{TypeEntryActual,TypeStatus,TypedSiteRef}};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|
        case.id.0 == "pass_advanced_semantics_native_conjunction_goals_001").unwrap();
    let frontend = super::run_frontend(&config.workspace_root,case,0).unwrap();
    assert!(frontend.diagnostics.is_empty(),"{:?}",frontend.diagnostics);
    let ast = frontend.ast.expect("genuine new conjunction source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root,case,&ast);
    assert!(resolver.detail_keys.is_empty(),"{:?}",resolver.detail_keys);
    let handoff = super::native_conversion_handoff(&ast,&resolver.module,&resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    println!("Task272B9 test-first: genuine frontend/symbol/native/scope and native A succeed before goal replay; no admission failure used as RED");
    let (steps,success,pending,checked,inference) = handoff.check_witness_goals()
        .expect("new conjunction source must materialize sequential unproved goals");
    assert_eq!(steps.len(),5);
    assert_eq!(inference.terms().len(),12);
    assert_eq!(inference.formulas().len(),4);
    let [h0,h1,p0,p1] = std::array::from_fn::<_,4,_>(|index|task272b1_binding(&handoff,index));
    assert_eq!(handoff.scope().bindings().len(),4);
    assert_ne!(h0,p0);
    assert_ne!(h1,p1);
    assert_eq!(handoff.scope().references().iter().map(|reference|
        task272b1_binding(&handoff,reference.binding().index())).collect::<Vec<_>>(),
        [h0,h0,h1,h1,p0,p0,p1,p1]);
    let g0 = format!("forall({}:Set,forall({}:Set,and(eq({0},{0}),eq({1},{1}))))",h0.index(),h1.index());
    let g1 = format!("forall({}:Set,and(eq({1},{1}),eq({0},{0})))",h1.index(),p0.index());
    let g2 = format!("and(eq({0},{0}),eq({1},{1}))",p0.index(),p1.index());
    let g3 = format!("eq({0},{0})",p1.index());
    let expected = [(&g0,&g0),(&g0,&g1),(&g1,&g2),(&g2,&g3),(&g3,&g3)];
    let matched = [None,Some((h0,p0)),Some((h1,p1)),None,None];
    let free = [vec![],vec![p0],vec![p0,p1],vec![p1],vec![p1]];
    let rows = handoff.source().source_node_entries().unwrap().collect::<Vec<_>>();
    let original = |site:&TypedSiteRef| {
        let id = handoff.source().nodes().node(site.node()).unwrap().resolved_node.unwrap();
        rows.iter().copied().find(|row|row.site()==id).unwrap()
    };
    for (index,step) in steps.iter().enumerate() {
        assert_eq!(task272b1_goal_text(step.before()),*expected[index].0);
        assert_eq!(task272b1_goal_text(step.after()),*expected[index].1);
        assert_eq!(step.owner(),steps[0].site());
        assert_eq!(step.context(),steps[0].context());
        assert_eq!(step.matched(),matched[index]);
        assert_eq!(step.free(),free[index]);
        assert_eq!(step.avoided(),if index==1 {vec![h1]} else {vec![]});
        let row = original(step.site());
        match index {
            0 => assert_eq!(row.kind(),&K::TheoremItem),
            1|2 => assert_eq!(row.range(),handoff.scope().bindings()[index+1].range()),
            3|4 => assert_eq!(row.kind(),&K::ConclusionStatement),
            _ => unreachable!(),
        }
    }
    assert_eq!(handoff.binding_env().contexts().get(steps[0].context()).unwrap().layer,
        mizar_checker::binding_env::BindingContextLayer::Proof);
    assert_eq!(original(steps[3].site()).parent(),original(steps[4].site()).parent());
    assert!(rows.iter().any(|row|Some(row.site())==original(steps[3].site()).parent() && row.kind()==&K::ProofBlock));
    let G::Conjunction {right,..} = steps[3].before() else {panic!("whole conjunction before first Thus")};
    assert_eq!(steps[3].after(),right.as_ref());
    assert_eq!(steps[4].before(),right.as_ref());
    assert_eq!(steps[4].after(),right.as_ref());
    let G::Equality {origin,..} = right.as_ref() else {panic!("original typed right leaf")};
    assert_eq!(original(origin).kind(),&K::BuiltinPredicateApplication);
    assert!(original(origin).range().end < original(steps[3].site()).range().start);
    let (intents,labels) = handoff.check_proof_intents().unwrap();
    assert!(labels.ids().is_empty());
    for step in &steps[3..] {
        assert!(intents.iter().any(|(site,_,status)|site==step.site() && *status==S::Pending));
    }
    assert!(success.is_empty() && pending.is_empty());
    assert!(checked.coercions().is_empty() && checked.initial_obligations().is_empty() && checked.facts().is_empty());
    for (_,term) in inference.terms().iter() {
        assert_eq!(term.status,TermStatus::Inferred);
        let ty = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(ty.status,TypeStatus::Known);
        let TypeEntryActual::Known(id) = ty.actual else {panic!("actual known Set")};
        assert_eq!(inference.normalized_types().get(id).unwrap().head,TypeHeadRef::BuiltinSet);
    }
    for reference in handoff.scope().references() {
        let binding = task272b1_binding(&handoff,reference.binding().index());
        let context = handoff.binding_env().bindings().get(binding).unwrap().owner_context;
        assert!(inference.terms().iter().any(|(_,term)|term.reference==Some(TermReference::Binding(binding))
            && original(&term.site).range()==reference.range() && term.context==context));
    }
    for binding in [h0,h1,p0,p1] {
        let entry = handoff.binding_env().bindings().get(binding).unwrap();
        assert!(inference.terms().iter().any(|(_,term)|term.reference==Some(TermReference::Binding(binding))
            && matches!(&term.site,TypedSiteRef::Role {role,..} if role.as_str()=="witness-goal.declaration-value")
            && original(&term.site).range()==entry.declaration_range && term.context==entry.owner_context));
    }
    for (_,formula) in inference.formulas().iter() {
        assert_eq!(formula.kind,FormulaKind::Equality);
        assert_eq!(formula.status,FormulaStatus::Checked);
        assert_eq!(formula.terms.len(),2);
        assert_eq!(original(&formula.site).range(),formula.source_range);
    }
    assert!(checked.diagnostics().is_empty() && inference.diagnostics().is_empty());
    assert!(handoff.source().types().is_empty() && handoff.source().facts().is_empty()
        && handoff.source().initial_obligations().is_empty());
}

#[test]
fn task272b9_exact_admission_inventory_and_actual_selected_as_reject_crossed_pairs() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|case.id.0=="pass_advanced_semantics_native_conjunction_goals_001").unwrap();
    let old = plan.cases.iter().find(|case|case.id.0==super::NATIVE_WITNESS_GOAL_ID).unwrap();
    assert!(super::native_witness_goals_admitted(&config.workspace_root,case));
    assert!(super::native_witness_goals_admitted(&config.workspace_root,old));
    let result = super::run_advanced_semantics_case(&config.workspace_root,case,0);
    assert_eq!(result.status,super::TypeElaborationCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());
    let report = super::run_advanced_semantics_corpus(&config).unwrap();
    assert_eq!(report.error_count(),0,"{:?}",report.diagnostics);
    for selected in [case,old] {
        let matches = report.results.iter().filter(|result|result.id==selected.id).collect::<Vec<_>>();
        assert_eq!(matches.len(),1);
        assert_eq!(matches[0].status,super::TypeElaborationCaseStatus::Passed);
    }
    for mutation in 0..14 {
        let mut bad = case.clone();
        match mutation {
            0 => bad.expectation.domain=old.expectation.domain.clone(),
            1 => bad.expectation.spec_refs=old.expectation.spec_refs.clone(),
            2 => bad.source_path=old.source_path.clone(),
            3 => bad.expectation_path=old.expectation_path.clone(),
            4 => bad.expectation.id=old.id.clone(),
            5 => bad.id=old.id.clone(),
            6 => bad.expectation.tags.clear(),
            7 => bad.expectation.tags.push("extra".into()),
            8 => bad.expectation.stage=crate::Stage::FormulaStatement,
            9 => bad.expectation.expected_phase=Some(crate::expectation::PipelinePhase::StatementCheck),
            10 => bad.expectation.profiles.push("other".into()),
            11 => bad.expectation.source="other.miz".into(),
            12|13 => {
                bad.id.0="counterfeit_native_conjunction".into();
                bad.expectation.id=bad.id.clone();
                bad.expectation.tags.clear();
                if mutation==12 {bad.expectation_path=config.workspace_root.join("other.expect.toml");}
                else {bad.source_path=config.workspace_root.join("other.miz");}
            }
            _ => unreachable!(),
        }
        assert!(!super::native_witness_goals_admitted(&config.workspace_root,&bad));
        let mut invalid = plan.clone();
        invalid.cases.push(bad);
        assert!(super::validate_step5c11_registration_inventory(&config.workspace_root,&invalid).iter()
            .any(|diagnostic|diagnostic.code.0=="E-ADVANCED-SEMANTICS-ADMISSION"));
    }
    for selected in [case,old] {
        let mut missing = plan.clone();
        missing.cases.retain(|candidate|candidate.id!=selected.id);
        let mut duplicate = plan.clone();
        duplicate.cases.push(selected.clone());
        for invalid in [missing,duplicate] {
            assert!(super::validate_step5c11_registration_inventory(&config.workspace_root,&invalid).iter()
                .any(|diagnostic|diagnostic.detail_key==format!("advanced_semantics.inventory.{}",selected.id.0)));
        }
    }
    let mut pooled = plan.clone();
    pooled.cases.retain(|candidate|candidate.id!=case.id);
    pooled.cases.push(old.clone());
    let diagnostics = super::validate_step5c11_registration_inventory(&config.workspace_root,&pooled);
    for selected in [case,old] {
        assert!(diagnostics.iter().any(|diagnostic|diagnostic.detail_key==format!("advanced_semantics.inventory.{}",selected.id.0)));
    }
    let source = include_str!("../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_conjunction_goals_001.miz")
        .replace("thus a=a;","thus b=b;");
    let frontend = metamorphic_guard::with_source(case,&source,||super::run_frontend(&config.workspace_root,case,0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root,case,&ast);
    assert!(resolver.detail_keys.is_empty());
    let handoff = super::native_conversion_handoff(&ast,&resolver.module,&resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(case,&source,||super::run_advanced_semantics_case(&config.workspace_root,case,0));
    assert_eq!(result.status,super::TypeElaborationCaseStatus::Failed);
    assert!(result.actual_detail_keys.iter().any(|key|key.contains("native witness goal")));
    let mut error_key = case.clone();
    error_key.expectation.stable_detail_key=Some("advanced_semantics.invalid_prepared_case".into());
    let result = super::run_advanced_semantics_case(&config.workspace_root,&error_key,0);
    assert_eq!(result.actual_detail_keys,["advanced_semantics.invalid_prepared_case"]);
    assert_eq!(result.status,super::TypeElaborationCaseStatus::Failed);
}

#[test]
fn task272b9_sequential_claims_keep_equal_leaves_membership_aliases_and_pending_guards() {
    use mizar_checker::type_checker::TypeHeadRef;
    let base = include_str!("../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_conjunction_goals_001.miz");
    for source in [
        base.replace("b=b","a=a"),
        base.replace("a,b","AlphaValue,BetaValue").replace("a=a","AlphaValue=AlphaValue").replace("b=b","BetaValue=BetaValue"),
        format!("\n{}",base.replace(";",";\n")),
        base.replace("thus b=b;","thus thesis;"),
        base.replace("let a,b be set;","let a,b be set; set alias=a;").replace("thus a=a;","thus alias=a;"),
        base.replace("thus a=a;\n  thus b=b;","thus a=a & b=b;"),
        "theorem Mixed: for A being set, x being object holds x in A & A=A proof let A be set; let x be object; thus x in A; thus A=A; end;".to_owned(),
        "theorem MixedRight: for A being set, x being object holds A=A & x in A proof let A be set; let x be object; thus A=A; thus x in A; end;".to_owned(),
    ] {
        let handoff = task272b1_witness_source(&source);
        let (steps,_,_,checked,inference) = handoff.check_witness_goals().unwrap_or_else(|error|panic!("{source}: {error:?}"));
        assert_eq!(steps.last().unwrap().before(),steps.last().unwrap().after());
        assert!(checked.facts().is_empty());
        if source.contains("set alias") {
            task272b3_assert_alias_type(&handoff,&inference,4,TypeHeadRef::BuiltinSet);
        }
    }
    let pending = "theorem Pending: for o being object, s being set ex z being set st z=o & s=s proof let o be object; let s be set; take o; thus o=o; thus s=s; end;";
    for source in [pending.to_owned(),format!("{pending}\n{}",base.replace("NativeConjunctionGoals","Other"))] {
        let handoff = task272b1_witness_source(&source);
        let (steps,success,pending,checked,_) = handoff.check_witness_goals().unwrap();
        assert!(success.is_empty());
        assert_eq!(pending.len(),1);
        assert_eq!(pending[0].site,*steps[3].site());
        let obligation = checked.initial_obligations().iter().next().unwrap().1;
        assert_eq!(obligation.owner,*steps[3].site());
        assert_eq!(obligation.status,mizar_checker::typed_ast::InitialObligationStatus::Pending);
        let object = task272b1_binding(&handoff,3);
        assert!(steps[3].free().contains(&object));
        assert!(!steps[4].free().contains(&object));
        let observed = super::native_witness_goal_observation(&handoff).unwrap();
        for index in [4,5] {
            let line = observed.iter().find(|line|line.starts_with("remaining.")
                && line.contains(&format!("terminal={:?}",steps[index].site()))).unwrap();
            assert!(line.ends_with(&format!("pending=[{:?}]",obligation.id)),"{line}");
        }
        if steps.len()>6 {
            assert_ne!(steps[5].owner(),steps[9].owner());
            for index in [9,10] {
                let line = observed.iter().find(|line|line.starts_with("remaining.")
                    && line.contains(&format!("terminal={:?}",steps[index].site()))).unwrap();
                assert!(line.ends_with("pending=[]"),"{line}");
            }
        }
    }
}

#[test]
fn task272b9_sequential_claims_refuse_wrong_order_frames_and_partial_blocks() {
    let base = include_str!("../../../../tests/miz/pass/theorems/pass_advanced_semantics_native_conjunction_goals_001.miz");
    let claims = "thus a=a;\n  thus b=b;";
    for source in [
        base.replace(claims,"thus b=b; thus a=a;"),
        base.replace(claims,"thus a=a; thus a=a;"),
        base.replace("thus a=a;",""),
        base.replace("thus b=b;",""),
        base.replace(claims,"thus a=a; thus b=b; thus b=b;"),
        base.replace(claims,"thus a=a; set unused=a; thus b=b;"),
        base.replace("thus a=a;","thus thesis;"),
        base.replace("thus a=a;","thus a=a & b=b;"),
        base.replace("thus a=a;","thus L:a=a;"),
        base.replace("thus b=b;","thus R:b=b;"),
        base.replace("thus a=a;","hence a=a;"),
        base.replace("thus b=b;","hence b=b;"),
        base.replace("thus a=a;","then a=a;"),
        base.replace("thus b=b;","then b=b;"),
        base.replace(claims,"hereby thus a=a; end;"),
        base.replace(claims,"N:now thus a=a; end; thus b=b by N;"),
        "theorem Nested: for a,b being set holds (a=a & b=b) & a=a proof let a,b be set; thus a=a & b=b; thus a=a; end;".to_owned(),
        "theorem NestedRight: for a,b being set holds a=a & (b=b & a=a) proof let a,b be set; thus a=a; thus b=b & a=a; end;".to_owned(),
    ] {
        assert!(task272b1_witness_source(&source).check_witness_goals().is_err(),"{source}");
    }
    let by = "theorem By: for a,b being set st a=a holds a=a & b=b proof let a,b be set; assume H:a=a; thus a=a; thus b=b; end;";
    for source in [by.replace("thus a=a;","thus a=a by H;"),by.replace("thus b=b;","thus b=b by H;")] {
        let handoff = task272b1_witness_source(&source);
        super::native_proof_intent_observation(&handoff).unwrap();
        assert!(handoff.check_witness_goals().is_err());
    }
    let two = format!("{}\n{}",base.replace("NativeConjunctionGoals","First"),base.replace("NativeConjunctionGoals","Second"));
    let handoff = task272b1_witness_source(&two);
    let (steps,_,_,_,inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(),10);
    assert_eq!(inference.terms().len(),24);
    assert_eq!(inference.formulas().len(),8);
    assert_ne!(steps[0].owner(),steps[5].owner());
    assert_ne!(steps[0].context(),steps[5].context());
    assert!(task272b1_witness_source(&two.replacen("thus a=a;","thus b=b;",1)).check_witness_goals().is_err());
}

#[test]
fn task272b10_compact_consider_goals_execute_existing_fs() {
    use mizar_checker::{binding_env::{BindingContextLayer as L,BindingContextOwner,BindingContextRecovery,BindingKind,BindingTypeSite},
        source_proof_local_declaration::{SourceProofGoal as G,SourceProofIntentStatus as S},
        type_checker::{CoercionEvidence,CoercionJustification,CoercionRequestKind,FormulaKind,FormulaStatus,TermReference,TermStatus,TypeHeadInput,TypeHeadRef},
        typed_ast::{CoercionStatus,FactStatus,TypeEntryActual,TypeStatus,TypedSiteRef}};
    use mizar_resolve::{names::SourceVariableBindingKind as B,resolved_ast::{LabelKind,LabelResolution}};
    use mizar_syntax::SurfaceNodeKind as K;
    let source = include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_consider_choice_001.miz");
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|case.id.0=="pass_formula_statement_consider_choice_001").unwrap();
    let result = super::formula_statement::run_formula_statement_case(&config.workspace_root,case,0);
    assert_eq!(result.status,super::FormulaStatementCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());
    let frontend = super::run_frontend(&config.workspace_root,case,0).unwrap();
    assert!(frontend.diagnostics.is_empty(),"{:?}",frontend.diagnostics);
    let ast = frontend.ast.expect("genuine existing Consider1 source");
    let resolver = super::resolver_symbol_collection(&config.workspace_root,case,&ast);
    assert!(resolver.detail_keys.is_empty(),"{:?}",resolver.detail_keys);
    assert_eq!(super::formula_statement::check_formula_ast_with_organization(&ast,&resolver.module,&resolver.env,true),Ok(true));
    let handoff = super::native_conversion_handoff(&ast,&resolver.module,&resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    let (intents,labels) = handoff.check_proof_intents().unwrap();
    assert!(intents.iter().all(|(_,_,status)|*status==S::Pending));
    println!("Task272B10 test-first: existing FS/legacy organization/native A and genuine frontend/symbol/native/scope pass before goal replay");
    let (steps,success,pending,checked,inference) = handoff.check_witness_goals()
        .expect("actual Consider1 source must materialize compact and Consider hypothesis owners");
    assert_eq!(steps.len(),9);
    assert_eq!(inference.terms().len(),15);
    assert_eq!(inference.formulas().len(),5);
    let [h,p,e,w] = std::array::from_fn::<_,4,_>(|index|task272b1_binding(&handoff,index));
    assert_eq!(handoff.scope().bindings().len(),4);
    assert_eq!(handoff.scope().references().iter().map(|reference|
        task272b1_binding(&handoff,reference.binding().index())).collect::<Vec<_>>(),
        [h,h,e,p,p,p,p,w,p,p,p]);
    let g0 = format!("forall({}:Object,eq({0},{0}))",h.index());
    let gp = format!("eq({0},{0})",p.index());
    let ge = format!("exists({}:Object,eq({0},{}))",e.index(),p.index());
    let gw = format!("eq({},{})",w.index(),p.index());
    let expected = [(&g0,&g0),(&g0,&gp),(&ge,&ge),(&ge,&gp),(&gp,&gp),
        (&ge,&ge),(&ge,&gw),(&gw,&gw),(&gp,&gp)];
    let owners = [0,0,2,2,2,5,5,5,0];
    let matched = [None,Some((h,p)),None,Some((e,p)),None,None,Some((e,w)),None,None];
    let free = [vec![],vec![p],vec![p],vec![p],vec![p],vec![p],vec![p,w],vec![p,w],vec![p]];
    let rows = handoff.source().source_node_entries().unwrap().collect::<Vec<_>>();
    let original = |site:&TypedSiteRef| {
        let id = handoff.source().nodes().node(site.node()).unwrap().resolved_node.unwrap();
        rows.iter().copied().find(|row|row.site()==id).unwrap()
    };
    let outer = steps[0].context();
    let inner = steps[2].context();
    assert_ne!(outer,inner);
    assert_eq!(handoff.binding_env().contexts().get(outer).unwrap().layer,L::Proof);
    let child = handoff.binding_env().contexts().get(inner).unwrap();
    assert_eq!(child.layer,L::Proof);
    assert_eq!(child.parent,Some(outer));
    assert_eq!(child.recovery,BindingContextRecovery::Normal);
    assert!(child.bindings.is_empty() && child.visible_bindings.contains(&p));
    let compact = original(steps[2].site());
    let consider = original(steps[5].site());
    let proof = rows.iter().copied().find(|row|row.parent()==Some(compact.site()) && row.kind()==&K::ProofBlock).unwrap();
    assert_eq!(child.owner,BindingContextOwner::SourceStatement {source_range:proof.range()});
    assert_eq!(compact.parent(),consider.parent());
    assert!(rows.iter().any(|row|Some(row.site())==compact.parent() && row.kind()==&K::ProofBlock));
    let native = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast,&resolver.module).unwrap();
    let (scopes,_) = mizar_resolve::symbols::source_proof_scope_maps(&native,&resolver.env,handoff.scope()).unwrap();
    let compact_node = ast.node_views().find(|node|node.kind()==&K::CompactStatement && node.range()==compact.range()).unwrap();
    let proof_node = ast.node_views().find(|node|node.kind()==&K::ProofBlock && node.range()==proof.range()).unwrap();
    assert_eq!(scopes.get(&compact_node.id()),handoff.binding_env().contexts().get(outer).unwrap().lexical_scope.as_ref());
    assert_eq!(scopes.get(&proof_node.id()),child.lexical_scope.as_ref());
    for (index,step) in steps.iter().enumerate() {
        assert_eq!(task272b1_goal_text(step.before()),*expected[index].0);
        assert_eq!(task272b1_goal_text(step.after()),*expected[index].1);
        assert_eq!(step.owner(),steps[owners[index]].site());
        assert_eq!(step.context(),if (2..=4).contains(&index) {inner} else {outer});
        assert_eq!(step.matched(),matched[index]);
        assert_eq!(step.free(),free[index]);
        assert!(step.avoided().is_empty());
        let row = original(step.site());
        match index {
            0 => assert_eq!(row.kind(),&K::TheoremItem),
            1 => assert_eq!(row.range(),handoff.scope().bindings()[1].range()),
            2 => assert_eq!(row.kind(),&K::CompactStatement),
            3 => assert_eq!(row.kind(),&K::Witness),
            4|8 => assert_eq!(row.kind(),&K::ConclusionStatement),
            5 => assert_eq!(row.kind(),&K::ConsiderStatement),
            6 => assert_eq!(row.range(),handoff.scope().bindings()[3].range()),
            7 => assert_eq!(row.kind(),&K::ConditionList),
            _ => unreachable!(),
        }
    }
    assert_ne!(steps[0].site(),steps[2].site());
    assert_ne!(steps[2].site(),steps[5].site());
    assert_eq!(steps[5].before(),steps[2].before());
    assert_ne!(steps[5].before(),steps[4].after());
    assert_eq!(steps[7].before(),steps[6].after());
    assert_eq!(steps[7].after(),steps[6].after());
    assert_eq!(steps[8].before(),steps[1].after());
    assert_eq!(steps[8].after(),steps[1].after());
    let G::Existential {body,..} = steps[5].before() else {panic!("original cited existential")};
    let G::Equality {origin,..} = body.as_ref() else {panic!("original candidate atom")};
    let G::Equality {origin:instantiated,..} = steps[6].after() else {panic!("instantiated hypothesis")};
    assert_eq!(origin,instantiated);
    assert_eq!(original(steps[7].site()).parent(),Some(consider.site()));
    assert_eq!(handoff.scope().bindings()[3].kind(),B::ConsiderWitness);
    assert_eq!(handoff.binding_env().bindings().get(w).unwrap().kind,BindingKind::GivenWitness);
    for binding in [h,p,e,w] {
        let entry = handoff.binding_env().bindings().get(binding).unwrap();
        assert_eq!(entry.type_site,BindingTypeSite::Missing);
        assert!(entry.captured.identities().is_empty());
        assert!(inference.terms().iter().any(|(_,term)|term.reference==Some(TermReference::Binding(binding))
            && matches!(&term.site,TypedSiteRef::Role {role,..} if role.as_str()=="witness-goal.declaration-value")
            && original(&term.site).range()==entry.declaration_range && term.context==entry.owner_context));
    }
    let header_expression = handoff.binding_env().bindings().get(h).unwrap().owner_context;
    let compact_expression = handoff.binding_env().bindings().get(e).unwrap().owner_context;
    assert_eq!(handoff.binding_env().contexts().get(compact_expression).unwrap().layer,L::Expression);
    assert_ne!(compact_expression,outer);
    for (index,reference) in handoff.scope().references().iter().enumerate() {
        let binding = task272b1_binding(&handoff,reference.binding().index());
        let context = match index {0|1=>header_expression,2|3=>compact_expression,4..=6=>inner,_=>outer};
        assert!(inference.terms().iter().any(|(_,term)|term.reference==Some(TermReference::Binding(binding))
            && original(&term.site).range()==reference.range() && term.context==context));
    }
    assert_eq!(labels.index().iter().filter(|label|label.kind()==LabelKind::ProofStep).count(),2);
    assert!(labels.index().iter().any(|label|label.kind()==LabelKind::Theorem));
    for (spelling,range) in [("A1",compact.range()),("A2",original(steps[7].site()).range())] {
        let declaration = labels.index().iter().find(|label|label.kind()==LabelKind::ProofStep && label.primary_spelling()==spelling).unwrap();
        let mizar_session::SourceAnchor::Range(anchor) = declaration.origin().anchor() else {panic!("real label token")};
        assert!(range.start<=anchor.start && anchor.end<=range.end);
        assert_eq!(&source[anchor.start..anchor.end],spelling);
    }
    let [id] = labels.ids() else {panic!("single Consider By reference")};
    let LabelResolution::Resolved(label) = labels.table().get(*id).unwrap().resolution() else {panic!("resolved compact candidate")};
    assert_eq!(label.kind(),LabelKind::ProofStep);
    assert_eq!(labels.index().get(label.origin()).unwrap().primary_spelling(),"A1");
    assert!(compact.range().end<=label.range().start);
    assert!(consider.range().start<=label.range().start && label.range().end<=consider.range().end);
    assert_eq!(&source[label.range().start..label.range().end],"A1");
    assert_eq!(success.len(),2);
    assert!(pending.is_empty() && checked.initial_obligations().is_empty());
    assert_eq!(checked.coercions().len(),2);
    assert_eq!(checked.facts().len(),2);
    for (input,index) in success.iter().zip([3,6]) {
        assert_eq!(&input.site,steps[index].site());
        assert_eq!(input.source_range,original(steps[index].site()).range());
        assert_eq!(input.kind,CoercionRequestKind::Widening);
        assert_eq!(input.justification,CoercionJustification::Omitted);
        assert_eq!(input.evidence,CoercionEvidence::BuiltinRadix);
        assert_eq!(input.from_type.as_ref().unwrap().head,TypeHeadInput::BuiltinObject);
        assert_eq!(input.to_type.head,TypeHeadInput::BuiltinObject);
        let result = checked.coercions().iter().find(|(_,result)|result.site==input.site).unwrap().1;
        assert_eq!(result.status,CoercionStatus::Candidate);
        assert!(result.obligation.is_none());
        assert!(checked.facts().iter().any(|(_,fact)|fact.subject==input.site && fact.status==FactStatus::Known));
    }
    for (_,term) in inference.terms().iter() {
        assert_eq!(term.status,TermStatus::Inferred);
        let ty = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(ty.status,TypeStatus::Known);
        let TypeEntryActual::Known(id) = ty.actual else {panic!("actual declared Object")};
        assert_eq!(inference.normalized_types().get(id).unwrap().head,TypeHeadRef::BuiltinObject);
    }
    for (_,formula) in inference.formulas().iter() {
        assert_eq!(formula.kind,FormulaKind::Equality);
        assert_eq!(formula.status,FormulaStatus::Checked);
        assert_eq!(formula.terms.len(),2);
        assert_eq!(original(&formula.site).range(),formula.source_range);
    }
    assert!(checked.diagnostics().is_empty() && inference.diagnostics().is_empty());
    assert!(handoff.source().types().is_empty() && handoff.source().facts().is_empty()
        && handoff.source().initial_obligations().is_empty());
}

#[test]
fn task272b10_compact_and_consider_preserve_distinct_parent_goals_and_typed_aliases() {
    use mizar_checker::{type_checker::TypeHeadRef,binding_env::BindingKind};
    let distinct = "theorem Distinct: for x,u being object holds x=u proof let x,u be object; A1: ex y being object st y=x proof take x; thus x=x; end; consider z being object such that A2: z=x by A1; thus x=u; end;";
    let handoff = task272b1_witness_source(distinct);
    let (steps,_,_,_,_) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(),10);
    assert_eq!(task272b1_goal_text(steps[9].after()),format!("eq({},{})",
        task272b1_binding(&handoff,2).index(),task272b1_binding(&handoff,3).index()));
    assert_ne!(steps[5].after(),steps[9].after());
    assert_ne!(steps[8].after(),steps[9].after());
    assert_eq!(steps[9].before(),steps[2].after());
    let restore = "theorem RestoreTake: for x being object ex q being object st q=x proof let x be object; A1: ex y being object st y=x proof take x; thus x=x; end; consider z being object such that A2: z=x by A1; take z; thus z=x; end;";
    for source in [
        restore.to_owned(),
        restore.replace("take z; thus z=x;","set alias=z; take alias; thus alias=x;"),
        restore.replace("take z; thus z=x;","set firstAlias=z; set alias=firstAlias; take alias; thus alias=x;"),
        restore.replace("take z; thus z=x;","take alias=z; thus alias=x;"),
    ] {
        let handoff = task272b1_witness_source(&source);
        let (steps,success,pending,checked,inference) = handoff.check_witness_goals().unwrap_or_else(|error|panic!("{source}: {error:?}"));
        let parent = task272b1_binding(&handoff,2);
        let witness = task272b1_binding(&handoff,4);
        assert_eq!(handoff.binding_env().bindings().get(witness).unwrap().kind,BindingKind::GivenWitness);
        let alias = if source.contains("firstAlias") {
            let first = task272b3_assert_alias_type(&handoff,&inference,5,TypeHeadRef::BuiltinObject);
            let alias = task272b3_assert_alias_type(&handoff,&inference,6,TypeHeadRef::BuiltinObject);
            assert_eq!(handoff.binding_env().bindings().get(alias).unwrap().captured.identities(),
                [handoff.binding_env().bindings().get(first).unwrap().identity.clone()]);
            Some(alias)
        } else if source.contains("alias") {
            Some(task272b3_assert_alias_type(&handoff,&inference,5,TypeHeadRef::BuiltinObject))
        } else {None};
        let actual = if source.contains("set ") {alias.unwrap()} else {witness};
        let take = &steps[steps.len()-2];
        assert_eq!(take.owner(),steps[0].site());
        assert_eq!(take.before(),steps[1].after());
        assert_eq!(take.matched(),Some((task272b1_binding(&handoff,1),actual)));
        assert_eq!(task272b1_goal_text(take.after()),format!("eq({},{})",witness.index(),parent.index()));
        assert_eq!(success.len(),3);
        assert!(pending.is_empty() && checked.initial_obligations().is_empty());
        if source.contains("set alias=z") {
            assert_eq!(handoff.binding_env().bindings().get(alias.unwrap()).unwrap().captured.identities(),
                [handoff.binding_env().bindings().get(witness).unwrap().identity.clone()]);
        }
        if source.contains("take alias=z") {
            assert!(handoff.binding_env().bindings().get(alias.unwrap()).unwrap().captured.identities().is_empty());
        }
    }
    let base = include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_consider_choice_001.miz");
    for source in [
        base.replace("object","set")
            .replace("A1: ex y being set st y = x","A1: (ex y being set st y = x)"),
        base.replace("    thus x = x;","    thus thesis;"),
        base.replace("A2: ",""),
        base.replace("let x be object;","let x be object; set parentAlias=x;")
            .replace("ex y being object st y = x","ex y being object st y = parentAlias")
            .replace("take x;","take parentAlias;").replace("    thus x = x;","    thus parentAlias = parentAlias;"),
        "theorem Member: for A being set, x being object holds x=x proof let A be set; let x be object; A1: ex y being object st y in A proof take x; thus x in A; end; consider z being object such that A2: z in A by A1; thus x=x; end;".to_owned(),
    ] {
        let handoff = task272b1_witness_source(&source);
        super::native_proof_intent_observation(&handoff).unwrap();
        let (_,_,_,_,inference) = handoff.check_witness_goals().unwrap_or_else(|error|panic!("{source}: {error:?}"));
        if source.contains("parentAlias") {
            task272b3_assert_alias_type(&handoff,&inference,2,TypeHeadRef::BuiltinObject);
        }
    }
}

#[test]
fn task272b10_inner_and_outer_pending_guards_keep_their_original_owners_and_actual_types() {
    use mizar_checker::{type_checker::{TermReference,TypeHeadInput,TypeHeadRef},
        typed_ast::{InitialObligationStatus,TypeEntryActual}};
    let inner = "theorem InnerPending: for x being object holds x=x proof let x be object; A1: (ex y being set st y=x) proof take x; thus x=x; end; consider z being set such that A2: z=x by A1; thus x=x; end;";
    let both = "theorem BothPending: for x being object ex q being set st q=x proof let x be object; take x; A1: (ex y being set st y=x) proof take x; thus x=x; end; consider z being set such that A2: z=x by A1; thus x=x; end;";
    for (source,child_take,outer_take,child_terminal,final_terminal) in [
        (inner,3,None,4,8),(both,4,Some(2),5,9),
    ] {
        let handoff = task272b1_witness_source(source);
        super::native_proof_intent_observation(&handoff).unwrap();
        let (steps,success,pending,checked,inference) = handoff.check_witness_goals().unwrap();
        let expected = if outer_take.is_some() {2} else {1};
        assert_eq!(pending.len(),expected);
        assert_eq!(checked.initial_obligations().len(),expected);
        assert_eq!(success.len(),1);
        assert_eq!(success[0].site,*steps[final_terminal-2].site());
        assert_eq!(success[0].from_type.as_ref().unwrap().head,TypeHeadInput::BuiltinSet);
        assert_eq!(success[0].to_type.head,TypeHeadInput::BuiltinSet);
        let child = checked.initial_obligations().iter().find(|(_,obligation)|obligation.owner==*steps[child_take].site()).unwrap().1;
        assert_eq!(child.status,InitialObligationStatus::Pending);
        assert_ne!(steps[child_take].owner(),steps[final_terminal].owner());
        let observed = super::native_witness_goal_observation(&handoff).unwrap();
        let line = observed.iter().find(|line|line.starts_with("remaining.")
            && line.contains(&format!("terminal={:?}",steps[child_terminal].site()))).unwrap();
        assert!(line.ends_with(&format!("pending=[{:?}]",child.id)),"{line}");
        let line = observed.iter().find(|line|line.starts_with("remaining.")
            && line.contains(&format!("terminal={:?}",steps[final_terminal].site()))).unwrap();
        let expected = if let Some(index) = outer_take {
            let outer = checked.initial_obligations().iter().find(|(_,obligation)|obligation.owner==*steps[index].site()).unwrap().1;
            assert_ne!(outer.id,child.id);
            assert_eq!(outer.status,InitialObligationStatus::Pending);
            format!("pending=[{:?}]",outer.id)
        } else {"pending=[]".to_owned()};
        assert!(line.ends_with(&expected),"{line}");
        assert_eq!(observed.iter().filter(|line|line.starts_with("remaining.")).count(),2);
        let parent = task272b1_binding(&handoff,if outer_take.is_some() {2} else {1});
        let witness = task272b1_binding(&handoff,if outer_take.is_some() {4} else {3});
        for (_,term) in inference.terms().iter() {
            if ![Some(TermReference::Binding(parent)),Some(TermReference::Binding(witness))].contains(&term.reference) {continue;}
            let TypeEntryActual::Known(id) = inference.type_entries().get(term.type_entry).unwrap().actual else {panic!("actual declared type")};
            assert_eq!(inference.normalized_types().get(id).unwrap().head,
                if term.reference==Some(TermReference::Binding(parent)) {TypeHeadRef::BuiltinObject} else {TypeHeadRef::BuiltinSet});
        }
        assert!(handoff.source().facts().is_empty() && handoff.source().types().is_empty());
    }
}

#[test]
fn task272b10_compact_consider_refuses_wrong_sources_guards_captures_and_citations() {
    let base = include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_consider_choice_001.miz");
    for source in [
        base.replace("consider z being object","consider z being set"),
        base.replace("ex y being object","ex y being set")
            .replace("A1: ex y being set st y = x","A1: (ex y being set st y = x)"),
        base.replace("A2: z = x","A2: x = x"),
        base.replace("consider z being object","consider z,u being object"),
        base.replace("such that A2: z = x","such that A2: z = x and A3: z = x"),
        base.replace("by A1","by Missing"),
        base.replace("by A1","by A2"),
        base.replace("by A1","by A1, A1"),
        base.replace("by A1","by Consider1"),
        base.replace("  thus x = x;\nend;","  thus x = x by A2;\nend;"),
        base.replace("  thus x = x;\nend;","  thus x = x by A1;\nend;"),
        base.replace("consider z being object such that A2: z = x by A1;","given z being object such that A2: z = x;"),
        base.replace("take x;","let u be object; take x;"),
        base.replace("take x;","take x; take x;"),
        base.replace("take x;","take alias=x;"),
        base.replace("    thus x = x;","    hence x = x;"),
        base.replace("    thus x = x;","    then x = x;"),
        base.replace("    thus x = x;","    thus Inner:x = x;"),
        base.replace("    thus x = x;","    hereby thus x = x; end;"),
        base.replace("    thus x = x;","    thus x = x; thus x = x;"),
        base.replace("A1: ex y being object st y = x","A1: x = x"),
        base.replace("A1: ex y being object st y = x","A1: ex y being object st for q being object holds y = x")
            .replace("    thus x = x;","    thus thesis;"),
        "theorem WrongFree: for x,u being object holds x=u proof let x,u be object; A1: ex y being object st y=x proof take x; thus x=x; end; consider z being object such that A2: z=u by A1; thus x=u; end;".to_owned(),
        "theorem GivenCapture: for x being object st (ex y being object st y=x) holds x=x proof let x be object; given g being object such that H:g=x; A1: ex v being object st v=g proof take g; thus g=g; end; consider z being object such that A2: z=g by A1; thus x=x; end;".to_owned(),
        "theorem GivenAliasCapture: for x being object st (ex y being object st y=x) holds x=x proof let x be object; given g being object such that H:g=x; set alias=g; A1: ex v being object st v=alias proof take alias; thus alias=alias; end; consider z being object such that A2: z=alias by A1; thus x=x; end;".to_owned(),
        base.replace("  thus x = x;\nend;","  A3: ex t being object st t=z proof take z; thus z=z; end; thus x = x;\nend;"),
        "theorem MemberRight: for x being object holds x=x proof let x be object; A1: (ex y being set st x in y) proof take x; thus thesis; end; consider z being set such that A2: x in z by A1; thus x=x; end;".to_owned(),
    ] {
        let handoff = task272b1_witness_source(&source);
        if source.contains("(ex y being set") {
            super::native_proof_intent_observation(&handoff).unwrap();
        }
        assert!(handoff.check_witness_goals().is_err(),"{source}");
    }
    let two = format!("{}\n{}",base.replace("Consider1","First"),base.replace("Consider1","Second"));
    let handoff = task272b1_witness_source(&two);
    let (steps,success,_,_,inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(),18);
    assert_eq!(success.len(),4);
    assert_eq!(inference.terms().len(),30);
    assert_eq!(inference.formulas().len(),10);
    assert_ne!(steps[0].owner(),steps[9].owner());
    assert_ne!(steps[2].owner(),steps[11].owner());
    assert_ne!(steps[5].owner(),steps[14].owner());
    assert!(task272b1_witness_source(&two.replacen("A2: z = x","A2: x = x",1)).check_witness_goals().is_err());
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|case.id.0==super::NATIVE_WITNESS_GOAL_ID).unwrap();
    let future = "theorem Future: for x,future being object holds x=x proof let x be object; A1: ex y being object st y=future proof take future; thus future=future; end; let future be object; consider z being object such that A2: z=future by A1; thus x=x; end;";
    let frontend = metamorphic_guard::with_source(case,future,||super::run_frontend(&config.workspace_root,case,0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root,case,&ast);
    assert!(resolver.detail_keys.is_empty());
    assert!(super::native_conversion_handoff(&ast,&resolver.module,&resolver.env)
        .and_then(|handoff|handoff.check_witness_goals().map_err(|_|"future compact capture".to_owned())).is_err());
}

#[test]
fn task272b10_compact_consider_native_gate_controls_actual_fs_result() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|case.id.0=="pass_formula_statement_consider_choice_001").unwrap();
    let source = include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_consider_choice_001.miz")
        .replace("for x being object holds x = x","for x being object st x = x holds x = x")
        .replace("let x be object;","let x be object; assume Premise:x = x;")
        .replace("    thus x = x;","    thus x = x by Premise;");
    let frontend = metamorphic_guard::with_source(case,&source,||super::run_frontend(&config.workspace_root,case,0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root,case,&ast);
    assert!(resolver.detail_keys.is_empty());
    assert_eq!(super::formula_statement::check_formula_ast_with_organization(&ast,&resolver.module,&resolver.env,true),Ok(true));
    let handoff = super::native_conversion_handoff(&ast,&resolver.module,&resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(case,&source,||super::formula_statement::run_formula_statement_case(&config.workspace_root,case,0));
    assert_eq!(result.status,super::FormulaStatementCaseStatus::Failed);
    assert!(result.actual_detail_keys.iter().any(|key|key.contains("native witness goal")));
}

#[test]
fn task272b11_case_goals_execute_existing_fs() {
    use mizar_checker::{binding_env::{BindingContextLayer as L,BindingContextOwner,BindingContextRecovery,BindingTypeSite},
        source_proof_local_declaration::SourceProofIntentStatus as S,
        type_checker::{FormulaKind,FormulaStatus,TermReference,TermStatus,TypeHeadRef},
        typed_ast::{TypeEntryActual,TypeStatus,TypedSiteRef}};
    use mizar_resolve::resolved_ast::{LabelKind,LabelResolution};
    use mizar_syntax::{SurfaceNodeKind as K,SurfaceFormulaConnective as C,SurfaceFormulaPrefixOperator as N};
    let source = include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_per_cases_suppose_001.miz");
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|case.id.0=="pass_formula_statement_per_cases_suppose_001").unwrap();
    let result = super::formula_statement::run_formula_statement_case(&config.workspace_root,case,0);
    assert_eq!(result.status,super::FormulaStatementCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());
    let frontend = super::run_frontend(&config.workspace_root,case,0).unwrap();
    assert!(frontend.diagnostics.is_empty(),"{:?}",frontend.diagnostics);
    let ast = frontend.ast.expect("genuine existing PerCases1 source");
    assert_eq!(ast.node_views().filter(|node|matches!(node.kind(),K::BinaryFormula(op) if op.connective==C::Or)).count(),3);
    assert_eq!(ast.node_views().filter(|node|matches!(node.kind(),K::PrefixFormula(N::Not))).count(),4);
    assert_eq!(ast.node_views().filter(|node|node.kind()==&K::BuiltinPredicateApplication).count(),8);
    let resolver = super::resolver_symbol_collection(&config.workspace_root,case,&ast);
    assert!(resolver.detail_keys.is_empty(),"{:?}",resolver.detail_keys);
    assert_eq!(super::formula_statement::check_formula_ast_with_organization(&ast,&resolver.module,&resolver.env,true),Ok(true));
    let handoff = super::native_conversion_handoff(&ast,&resolver.module,&resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    let (intents,labels) = handoff.check_proof_intents().unwrap();
    assert!(intents.iter().all(|(_,_,status)|*status==S::Pending));
    println!("Task272B11 test-first: existing FS/legacy organization/native A and genuine frontend/symbol/native/scope pass; original source has 3 Or/4 Not/8 Eq before goal replay");
    let (steps,success,pending,checked,inference) = handoff.check_witness_goals()
        .expect("actual PerCases1 source must materialize unproved whole-goal branches");
    assert_eq!(steps.len(),7);
    assert_eq!(inference.terms().len(),18);
    assert_eq!(inference.formulas().len(),8);
    let (header,local) = (task272b1_binding(&handoff,0),task272b1_binding(&handoff,1));
    assert_ne!(header,local);
    assert_eq!(handoff.scope().bindings().len(),2);
    assert_eq!(handoff.scope().references().iter().map(|reference|
        task272b1_binding(&handoff,reference.binding().index())).collect::<Vec<_>>(),
        [header,header,header,header,local,local,local,local,local,local,local,local,local,local,local,local]);
    let g0 = format!("forall({}:Set,or(eq({0},{0}),not(eq({0},{0}))))",header.index());
    let gp = format!("or(eq({0},{0}),not(eq({0},{0})))",local.index());
    let rows = handoff.source().source_node_entries().unwrap().collect::<Vec<_>>();
    let original = |site:&TypedSiteRef| {
        let id = handoff.source().nodes().node(site.node()).unwrap().resolved_node.unwrap();
        rows.iter().copied().find(|row|row.site()==id).unwrap()
    };
    assert_eq!(rows.iter().filter(|row|matches!(row.kind(),K::BinaryFormula(op) if op.connective==C::Or)).count(),3);
    assert_eq!(rows.iter().filter(|row|matches!(row.kind(),K::PrefixFormula(N::Not))).count(),4);
    let outer = steps[0].context();
    let branches = [steps[3].context(),steps[5].context()];
    assert_ne!(branches[0],branches[1]);
    assert_eq!(handoff.binding_env().contexts().get(outer).unwrap().layer,L::Proof);
    let cases = original(steps[2].site());
    assert_eq!(cases.kind(),&K::CaseReasoningStatement);
    let proof = rows.iter().copied().find(|row|Some(row.site())==cases.parent()).unwrap();
    assert_eq!(proof.kind(),&K::ProofBlock);
    assert_eq!(proof.parent(),Some(original(steps[0].site()).site()));
    let native = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast,&resolver.module).unwrap();
    let (scopes,_) = mizar_resolve::symbols::source_proof_scope_maps(&native,&resolver.env,handoff.scope()).unwrap();
    let cases_node = ast.node_views().find(|node|node.kind()==&K::CaseReasoningStatement).unwrap();
    assert_eq!(scopes.get(&cases_node.id()),handoff.binding_env().contexts().get(outer).unwrap().lexical_scope.as_ref());
    for (index,step) in steps.iter().enumerate() {
        assert_eq!(step.owner(),steps[0].site());
        assert_eq!(step.context(),match index {0..=2=>outer,3|4=>branches[0],_=>branches[1]});
        assert_eq!(task272b1_goal_text(step.before()),if index<2 {g0.clone()} else {gp.clone()});
        assert_eq!(task272b1_goal_text(step.after()),if index==0 {g0.clone()} else {gp.clone()});
        assert_eq!(step.matched(),if index==1 {Some((header,local))} else {None});
        assert_eq!(step.free(),if index==0 {vec![]} else {vec![local]});
        assert!(step.avoided().is_empty());
        if index>=2 {
            assert_eq!(step.before(),steps[1].after());
            assert_eq!(step.after(),steps[1].after());
        }
        let row = original(step.site());
        match index {
            0=>assert_eq!(row.kind(),&K::TheoremItem),
            1=>assert_eq!(row.range(),handoff.scope().bindings()[1].range()),
            2=>assert_eq!(row.kind(),&K::CaseReasoningStatement),
            3|5=>assert_eq!(row.kind(),&K::SupposeItem),
            4|6=>assert_eq!(row.kind(),&K::ConclusionStatement),
            _=>unreachable!(),
        }
    }
    for (branch,conclusion,context,spelling) in [(3,4,branches[0],"A1"),(5,6,branches[1],"A2")] {
        let row = original(steps[branch].site());
        assert_eq!(row.parent(),Some(cases.site()));
        assert_eq!(original(steps[conclusion].site()).parent(),Some(row.site()));
        let block = handoff.binding_env().contexts().get(context).unwrap();
        assert_ne!(context,outer);
        assert_eq!(block.layer,L::Block);
        assert_eq!(block.parent,Some(outer));
        assert_eq!(block.owner,BindingContextOwner::SourceStatement {source_range:row.range()});
        assert_eq!(block.recovery,BindingContextRecovery::Normal);
        assert!(block.bindings.is_empty() && block.visible_bindings.contains(&local));
        let node = ast.node_views().find(|node|node.kind()==&K::SupposeItem && node.range()==row.range()).unwrap();
        assert_eq!(block.lexical_scope.as_ref(),scopes.get(&node.id()));
        assert_eq!(handoff.binding_env().contexts().iter().filter(|(_,candidate)|candidate.lexical_scope==block.lexical_scope).count(),1);
        let declaration = labels.index().iter().find(|label|label.kind()==LabelKind::ProofStep && label.primary_spelling()==spelling).unwrap();
        let mizar_session::SourceAnchor::Range(anchor) = declaration.origin().anchor() else {panic!("original hypothesis label token")};
        assert_eq!(&source[anchor.start..anchor.end],spelling);
        let hypothesis = rows.iter().copied().find(|candidate|candidate.kind()==&K::Proposition && candidate.parent()==Some(row.site())).unwrap();
        assert!(hypothesis.range().start<=anchor.start && anchor.end<=hypothesis.range().end);
        let references = labels.ids().iter().map(|id|labels.table().get(*id).unwrap()).filter_map(|entry|
            match entry.resolution() {LabelResolution::Resolved(label) if label.origin()==declaration.origin_path()=>Some(label),_=>None}).collect::<Vec<_>>();
        let [reference] = references.as_slice() else {panic!("one real own-branch By")};
        assert_eq!(reference.kind(),LabelKind::ProofStep);
        assert_ne!(*anchor,reference.range());
        assert!(hypothesis.range().end<=reference.range().start);
        assert_eq!(&source[reference.range().start..reference.range().end],spelling);
        let use_row = rows.iter().copied().find(|candidate|candidate.kind()==&K::Reference && candidate.range()==reference.range()).unwrap();
        let list = rows.iter().copied().find(|candidate|Some(candidate.site())==use_row.parent()).unwrap();
        let by = rows.iter().copied().find(|candidate|Some(candidate.site())==list.parent()).unwrap();
        assert_eq!(list.kind(),&K::ReferenceList);
        assert_eq!(by.kind(),&K::JustificationClause);
        assert_eq!(by.parent(),Some(original(steps[conclusion].site()).site()));
    }
    assert_eq!(labels.index().iter().filter(|label|label.kind()==LabelKind::ProofStep).count(),2);
    assert!(labels.index().iter().any(|label|label.kind()==LabelKind::Theorem));
    assert_eq!(labels.ids().len(),2);
    assert!(success.is_empty() && pending.is_empty());
    assert!(checked.coercions().is_empty() && checked.initial_obligations().is_empty() && checked.facts().is_empty());
    let header_context = handoff.binding_env().bindings().get(header).unwrap().owner_context;
    assert_eq!(handoff.binding_env().contexts().get(header_context).unwrap().layer,L::Expression);
    for (index,reference) in handoff.scope().references().iter().enumerate() {
        let binding = task272b1_binding(&handoff,reference.binding().index());
        let context = if index<4 {header_context} else if index<10 {branches[0]} else {branches[1]};
        assert!(inference.terms().iter().any(|(_,term)|term.reference==Some(TermReference::Binding(binding))
            && original(&term.site).range()==reference.range() && term.context==context));
    }
    for binding in [header,local] {
        let entry = handoff.binding_env().bindings().get(binding).unwrap();
        assert_eq!(entry.type_site,BindingTypeSite::Missing);
        assert!(inference.terms().iter().any(|(_,term)|term.reference==Some(TermReference::Binding(binding))
            && matches!(&term.site,TypedSiteRef::Role {role,..} if role.as_str()=="witness-goal.declaration-value")
            && original(&term.site).range()==entry.declaration_range && term.context==entry.owner_context));
    }
    for (_,term) in inference.terms().iter() {
        assert_eq!(term.status,TermStatus::Inferred);
        let ty = inference.type_entries().get(term.type_entry).unwrap();
        assert_eq!(ty.status,TypeStatus::Known);
        let TypeEntryActual::Known(id) = ty.actual else {panic!("actual known Set")};
        assert_eq!(inference.normalized_types().get(id).unwrap().head,TypeHeadRef::BuiltinSet);
    }
    for (_,formula) in inference.formulas().iter() {
        assert_eq!(formula.kind,FormulaKind::Equality);
        assert_eq!(formula.status,FormulaStatus::Checked);
        assert_eq!(formula.terms.len(),2);
        assert_eq!(original(&formula.site).range(),formula.source_range);
        assert_eq!(original(&formula.site).kind(),&K::BuiltinPredicateApplication);
        let context = if original(steps[3].site()).range().start<=formula.source_range.start
            && formula.source_range.end<=original(steps[3].site()).range().end {branches[0]}
            else if original(steps[5].site()).range().start<=formula.source_range.start
            && formula.source_range.end<=original(steps[5].site()).range().end {branches[1]} else {header_context};
        assert_eq!(formula.context,context);
    }
    assert!(checked.diagnostics().is_empty() && inference.diagnostics().is_empty());
    assert!(handoff.source().types().is_empty() && handoff.source().facts().is_empty()
        && handoff.source().initial_obligations().is_empty());
}

#[test]
fn task272b11_cases_keep_actual_thesis_inputs_own_label_identity_and_member_types() {
    use mizar_checker::type_checker::{FormulaKind,TypeHeadRef};
    use mizar_resolve::resolved_ast::LabelResolution;
    use mizar_syntax::{SurfaceNodeKind as K,SurfaceFormulaConnective as C,SurfaceFormulaPrefixOperator as N};
    let base = include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_per_cases_suppose_001.miz");
    for source in [
        base.replace("suppose","case"),
        base.replace("X","Value").replace("A1","FirstHyp").replace("A2","SecondHyp"),
        format!("\n{}",base.replace(";",";\n")),
        base.replace("X = X or not X = X","(X = X) or (not (X = X))"),
        base.replace("A2","A1"),
        base.replace("thus X = X or not X = X by","thus thesis by"),
        base.replace("let X be set;","let X be set; set Alias=X;")
            .replace("thus X = X or not X = X","thus Alias = X or not Alias = X")
            .replace("suppose A1: X = X","suppose A1: Alias = X")
            .replace("suppose A2: not X = X","suppose A2: not Alias = X"),
        "theorem MemberCases: for A being set, x being object holds x in A or not x in A proof let A be set; let x be object; per cases; suppose H1: x in A; thus x in A or not x in A by H1; end; suppose H2: not x in A; thus x in A or not x in A by H2; end; end;".to_owned(),
    ] {
        let handoff = task272b1_witness_source(&source);
        super::native_proof_intent_observation(&handoff).unwrap();
        let (steps,success,pending,checked,inference) = handoff.check_witness_goals().unwrap_or_else(|error|panic!("{source}: {error:?}"));
        assert!(steps.iter().all(|step|step.owner()==steps[0].site()));
        assert!(success.is_empty() && pending.is_empty() && checked.facts().is_empty());
        if source.contains("thus thesis") {
            assert_eq!(steps.len(),7);
            assert_eq!(inference.terms().len(),10);
            assert_eq!(inference.formulas().len(),4);
            let rows = handoff.source().source_node_entries().unwrap().collect::<Vec<_>>();
            assert_eq!(rows.iter().filter(|row|matches!(row.kind(),K::BinaryFormula(op) if op.connective==C::Or)).count(),1);
            assert_eq!(rows.iter().filter(|row|matches!(row.kind(),K::PrefixFormula(N::Not))).count(),2);
            assert_eq!(steps[4].after(),steps[1].after());
            assert_eq!(steps[6].after(),steps[1].after());
        }
        if source.contains("set Alias") {
            task272b3_assert_alias_type(&handoff,&inference,2,TypeHeadRef::BuiltinSet);
        }
        if source.contains("theorem MemberCases:") {
            assert_eq!(inference.formulas().len(),8);
            assert!(inference.formulas().iter().all(|(_,formula)|formula.kind==FormulaKind::Membership));
        }
        let (_,labels) = handoff.check_proof_intents().unwrap();
        let origins = labels.ids().iter().map(|id| {
            let LabelResolution::Resolved(label) = labels.table().get(*id).unwrap().resolution() else {panic!("real own hypothesis")};
            label.origin()
        }).collect::<Vec<_>>();
        assert_eq!(origins.len(),2);
        assert_ne!(origins[0],origins[1]);
        assert!(handoff.source().facts().is_empty() && handoff.source().types().is_empty());
    }
}

#[test]
fn task272b11_or_not_projection_reuses_existing_effects_and_preserves_closed_caps() {
    for source in [
        "theorem Whole: for x being set holds x=x or not x=x proof let x be set; thus x=x or not x=x; end;",
        "theorem Negative: for x being set holds not x=x proof let x be set; thus not x=x; end;",
        "theorem Double: for x being set holds not not x=x proof let x be set; thus not not x=x; end;",
        "theorem Alpha: for x being set holds for y being object holds y=x or not y=x proof let x be set; thus for z being object holds z=x or not z=x; end;",
        "theorem BothEdges: for x,y being set holds y=y or not x=x proof let x,y be set; thus y=y or not x=x; end;",
        "theorem Take: for o being object ex z being object st z=o or not z=o proof let o be object; take o; thus o=o or not o=o; end;",
        "theorem Assume: for x being set st x=x holds x=x or not x=x proof let x be set; assume H:x=x; thus x=x or not x=x by H; end;",
        "theorem NowBool: for x being set holds x=x or not x=x proof let x be set; N: now thus x=x or not x=x; end; thus x=x or not x=x by N; end;",
        "theorem HerebyBool: for x being set holds x=x or not x=x proof let x be set; hereby thus x=x or not x=x; end; end;",
    ] {
        let handoff = task272b1_witness_source(source);
        let (steps,_,_,_,_) = handoff.check_witness_goals().unwrap_or_else(|error|panic!("{source}: {error:?}"));
        assert_eq!(steps.last().unwrap().before(),steps.last().unwrap().after());
        if source.contains("theorem BothEdges:") {
            assert_eq!(steps[1].free(),[task272b1_binding(&handoff,2)]);
            assert_eq!(steps[2].free(),[task272b1_binding(&handoff,2),task272b1_binding(&handoff,3)]);
            assert_eq!(task272b1_goal_text(steps[2].after()),format!("or(eq({1},{1}),not(eq({0},{0})))",
                task272b1_binding(&handoff,2).index(),task272b1_binding(&handoff,3).index()));
        }
        if source.contains("theorem Alpha:") {
            assert_eq!(steps[1].avoided(),[task272b1_binding(&handoff,1)]);
        }
    }
    for source in [
        "theorem LeftImp: for x being set holds (x=x implies x=x) or x=x proof let x be set; thus thesis; end;",
        "theorem RightImp: for x being set holds x=x or (x=x implies x=x) proof let x be set; thus thesis; end;",
        "theorem NotImp: for x being set holds not (x=x implies x=x) proof let x be set; thus thesis; end;",
        "theorem QuantImp: for x being set holds for y being set holds x=x or not (y=y implies x=x) proof let x be set; thus thesis; end;",
        "theorem NotQuantImp: for x being set holds not (for y being set holds y=y implies x=x) proof let x be set; thus thesis; end;",
        "theorem HerebyImp: for x being set holds x=x or not (x=x implies x=x) proof let x be set; hereby thus thesis; end; end;",
        "theorem NowImp: for x being set holds x=x or not (x=x implies x=x) proof let x be set; N: now thus thesis; end; thus thesis by N; end;",
        "theorem PartialCap: for x being set holds (x=x or not x=x) & x=x proof let x be set; thus x=x or not x=x; thus x=x; end;",
        "theorem CompactCap: for x being object holds x=x proof let x be object; A1: (ex y being object st y=x or not y=x) proof take x; thus x=x or not x=x; end; consider z being object such that A2:z=x or not z=x by A1; thus x=x; end;",
        "theorem BadRight: for o being object ex A being set st o=o or not (o in A) proof let o be object; take o; thus thesis; end;",
    ] {
        assert!(task272b1_witness_source(source).check_witness_goals().is_err(),"{source}");
    }
}

#[test]
fn task272b11_case_conclusions_keep_prior_pending_and_theorem_isolation() {
    use mizar_checker::{type_checker::{TermReference,TypeHeadRef},typed_ast::{InitialObligationStatus,TypeEntryActual}};
    let pending = "theorem PendingCases: for o being object ex z being set st z=o or not z=o proof let o be object; take o; per cases; suppose A1:o=o; thus o=o or not o=o by A1; end; suppose A2:not o=o; thus o=o or not o=o by A2; end; end;";
    let other = include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_per_cases_suppose_001.miz").replace("PerCases1","OtherCases");
    for source in [pending.to_owned(),format!("{pending}\n{other}")] {
        let handoff = task272b1_witness_source(&source);
        let (steps,success,pending,checked,inference) = handoff.check_witness_goals().unwrap();
        assert!(success.is_empty());
        assert_eq!(pending.len(),1);
        assert_eq!(pending[0].site,*steps[2].site());
        assert_eq!(checked.initial_obligations().len(),1);
        let obligation = checked.initial_obligations().iter().next().unwrap().1;
        assert_eq!(obligation.owner,*steps[2].site());
        assert_eq!(obligation.status,InitialObligationStatus::Pending);
        let observed = super::native_witness_goal_observation(&handoff).unwrap();
        for index in [5,7] {
            assert_eq!(steps[index].owner(),steps[2].owner());
            assert_ne!(steps[index].context(),steps[2].context());
            let line = observed.iter().find(|line|line.starts_with("remaining.")
                && line.contains(&format!("terminal={:?}",steps[index].site()))).unwrap();
            assert!(line.ends_with(&format!("pending=[{:?}]",obligation.id)),"{line}");
        }
        if steps.len()>8 {
            assert_eq!(steps.len(),15);
            assert_ne!(steps[12].owner(),steps[2].owner());
            for index in [12,14] {
                let line = observed.iter().find(|line|line.starts_with("remaining.")
                    && line.contains(&format!("terminal={:?}",steps[index].site()))).unwrap();
                assert!(line.ends_with("pending=[]"),"{line}");
            }
        }
        let object = task272b1_binding(&handoff,2);
        for (_,term) in inference.terms().iter().filter(|(_,term)|term.reference==Some(TermReference::Binding(object))) {
            let TypeEntryActual::Known(id) = inference.type_entries().get(term.type_entry).unwrap().actual else {panic!("actual Object retained")};
            assert_eq!(inference.normalized_types().get(id).unwrap().head,TypeHeadRef::BuiltinObject);
        }
        assert!(handoff.source().facts().is_empty() && handoff.source().types().is_empty());
    }
}

#[test]
fn task272b11_case_profile_refuses_other_hypotheses_citations_and_source_effects() {
    let base = include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_per_cases_suppose_001.miz");
    for source in [
        base.replace(" by A1",""),
        base.replace(" by A2",""),
        base.replace("by A1","by A2"),
        base.replace("by A2","by A1"),
        base.replace("by A1","by Missing"),
        base.replace("by A1","by PerCases1"),
        base.replace("by A1","by A1, A1"),
        base.replace("by A1","by Imported.A1"),
        base.replace("suppose A1: X = X;","suppose A1: not X = X;"),
        base.replace("suppose A2: not X = X;","suppose A2: X = X;"),
        base.replace("thus X = X or not X = X by A1;","thus X = X by A1;"),
        base.replace("thus X = X or not X = X by A1;","hence X = X or not X = X by A1;"),
        base.replace("thus X = X or not X = X by A1;","then X = X or not X = X by A1;"),
        base.replace("thus X = X or not X = X by A1;","thus Inner:X = X or not X = X by A1;"),
        base.replace("suppose A1: X = X;","suppose A1: X = X; let Y be set;"),
        base.replace("suppose A1: X = X;","suppose A1: X = X; take X;"),
        base.replace("suppose A1: X = X;","suppose A1: X = X; set Alias=X;"),
        base.replace("thus X = X or not X = X by A1;","hereby thus X = X or not X = X by A1; end;"),
        base.replace("thus X = X or not X = X by A1;","thus X = X or not X = X by A1; thus thesis by A1;"),
        base.replace("suppose A1: X = X;","suppose that A1: X = X and Extra: X = X;"),
        base.replace("per cases;","then per cases;"),
        base.replace("  end;\nend;","  end;\n  thus X = X or not X = X;\nend;"),
        base.replace("  suppose A2: not X = X;\n    thus X = X or not X = X by A2;\n  end;\n",""),
        base.replace("  end;\nend;","  end;\n  suppose A3: X = X; thus X = X or not X = X by A3; end;\nend;"),
        "theorem WrongUID: for X,Y being set holds X=Y or not X=Y proof let X,Y be set; per cases; suppose A:Y=X; thus X=Y or not X=Y by A; end; suppose B:not X=Y; thus X=Y or not X=Y by B; end; end;".to_owned(),
        "theorem ObjectRight: for A,x being object holds x in A or not x in A proof let A,x be object; per cases; suppose H1:x in A; thus x in A or not x in A by H1; end; suppose H2:not x in A; thus x in A or not x in A by H2; end; end;".to_owned(),
    ] {
        assert!(task272b1_witness_source(&source).check_witness_goals().is_err(),"{source}");
    }
    let mixed = super::formula_statement::step5c8_test_frontend(&base.replacen("suppose","case",1));
    assert!(mixed.diagnostics.iter().any(|diagnostic|diagnostic.message.contains("must not mix")));
    let outer = base.replace("for X being set holds","for X being set st X = X holds")
        .replace("let X be set;","let X be set; assume Outer: X = X;")
        .replace("by A1","by Outer");
    let handoff = task272b1_witness_source(&outer);
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let two = format!("{}\n{}",base.replace("PerCases1","First"),base.replace("PerCases1","Second"));
    let handoff = task272b1_witness_source(&two);
    let (steps,_,_,_,inference) = handoff.check_witness_goals().unwrap();
    assert_eq!(steps.len(),14);
    assert_eq!(inference.terms().len(),36);
    assert_eq!(inference.formulas().len(),16);
    assert_ne!(steps[0].owner(),steps[7].owner());
    assert_ne!(steps[3].context(),steps[10].context());
    assert!(task272b1_witness_source(&two.replacen("by A1","by A2",1)).check_witness_goals().is_err());
}

#[test]
fn task272b11_case_native_gate_controls_actual_fs_result() {
    let config = step5c11_config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case|case.id.0=="pass_formula_statement_per_cases_suppose_001").unwrap();
    let source = include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_per_cases_suppose_001.miz")
        .replacen(" by A1","",1);
    let frontend = metamorphic_guard::with_source(case,&source,||super::run_frontend(&config.workspace_root,case,0)).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.unwrap();
    let resolver = super::resolver_symbol_collection(&config.workspace_root,case,&ast);
    assert!(resolver.detail_keys.is_empty());
    assert_eq!(super::formula_statement::check_formula_ast_with_organization(&ast,&resolver.module,&resolver.env,true),Ok(true));
    let handoff = super::native_conversion_handoff(&ast,&resolver.module,&resolver.env).unwrap();
    super::native_proof_intent_observation(&handoff).unwrap();
    assert!(handoff.check_witness_goals().is_err());
    let result = metamorphic_guard::with_source(case,&source,||super::formula_statement::run_formula_statement_case(&config.workspace_root,case,0));
    assert_eq!(result.status,super::FormulaStatementCaseStatus::Failed);
    assert!(result.actual_detail_keys.iter().any(|key|key.contains("native witness goal")));
}
