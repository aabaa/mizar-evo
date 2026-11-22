#[path = "support/source_overload.rs"]
mod support;

use mizar_checker::type_checker::check_source_algorithm_call_targets;
use mizar_checker::{
    overload_resolution::{
        ArgumentViabilityEvidence, CandidateDeclarationKind, InsertedViewInput, InsertedViewKind,
        InsertedViewReasonKey, InsertedViewStatus, OverloadCandidateId, OverloadResultId,
        OverloadResultStatus, SpecificityComparisonInput, SpecificityComparisonStatus,
        SpecificityGraphId, SpecificityReasonKey, ViabilityFactStatus,
    },
    source_overload::{SourceOverloadHandoff, SourceOverloadInput},
    typed_ast::TypedSiteRef,
};
use mizar_resolve::{
    env::SymbolEntry,
    resolved_ast::SymbolRef,
    symbols::{source_algorithm_recursive_groups, source_definiens_dependencies},
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceNodeId, SurfaceNodeKind};
use std::collections::BTreeMap;
use support::{SourceFixture, ordinary_overloads, single_ordinary, source_view, template_overload};

fn checked_site(
    fixture: &SourceFixture,
    kind: SurfaceNodeKind,
    start: usize,
    len: usize,
) -> (SurfaceNodeId, SourceRange) {
    let range = SourceRange {
        source_id: fixture.native.source_id(),
        start,
        end: start + len,
    };
    let (id, _) = fixture
        .native
        .arena()
        .iter()
        .find(|(_, node)| {
            node.kind() == &kind && node.origin().anchor() == &SourceAnchor::Range(range)
        })
        .expect("exact source occurrence");
    let ast =
        mizar_resolve::symbols::validated_source_symbol_ast(&fixture.native, &fixture.symbols)
            .unwrap();
    let surface = ast
        .node_views()
        .find(|node| node.id().index() == id.index())
        .unwrap()
        .id();
    (surface, range)
}

fn checked_symbol<'a>(fixture: &'a SourceFixture, text: &str, marker: &str) -> &'a SymbolEntry {
    fixture.symbols.symbols().iter().find(|entry| matches!(entry.origin().anchor(), SourceAnchor::Range(range) if range.start == text.find(marker).unwrap())).expect("real declaration identity")
}

fn checked_selection(
    fixture: &SourceFixture,
    text: &str,
    kind: SurfaceNodeKind,
    sites: &[(usize, &str, &str)],
) -> BTreeMap<SurfaceNodeId, SymbolRef> {
    sites
        .iter()
        .map(|&(start, spelling, declaration)| {
            let (id, range) = checked_site(fixture, kind.clone(), start, spelling.len());
            (
                id,
                SymbolRef::new(
                    checked_symbol(fixture, text, declaration).symbol().clone(),
                    range,
                ),
            )
        })
        .collect()
}

fn contest(input: &mut SourceOverloadInput, left: usize, right: usize, reason: &str) {
    input.comparisons.push(SpecificityComparisonInput {
        left: OverloadCandidateId::new(left),
        right: OverloadCandidateId::new(right),
        status: SpecificityComparisonStatus::LeftAtLeastRight,
        reasons: vec![SpecificityReasonKey::new(reason)],
    });
}

#[test]
fn checked_algorithm_calls_preserve_exact_self_mutual_and_repeated_sites() {
    for text in [
        "",
        "definition let n be object; algorithm quiet(n) -> object do return n; end; end;",
        "definition let n be object; algorithm recur(n) -> object do return recur(n); end; end;",
    ] {
        let fixture = SourceFixture::parse(text, "checked_algorithm_calls");
        let before = (
            fixture.native.clone(),
            fixture.symbols.clone(),
            fixture.typed.clone(),
        );
        let selected =
            check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap();
        let (expected, groups) = if text.contains("return recur(n)") {
            (
                checked_selection(
                    &fixture,
                    text,
                    SurfaceNodeKind::ApplicationTerm,
                    &[(
                        text.rfind("recur(n)").unwrap(),
                        "recur(n)",
                        "algorithm recur",
                    )],
                ),
                vec![vec![checked_symbol(&fixture, text, "algorithm recur")]],
            )
        } else {
            (BTreeMap::new(), Vec::new())
        };
        assert_eq!(selected, (BTreeMap::new(), expected, BTreeMap::new()));
        assert_eq!(
            source_algorithm_recursive_groups(
                &fixture.native,
                &fixture.symbols,
                &selected.1,
                &selected.2
            )
            .unwrap(),
            Some(groups)
        );
        assert_eq!(
            check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap(),
            selected
        );
        assert_eq!((fixture.native, fixture.symbols, fixture.typed), before);
    }
    for (a, b, c) in [("alpha", "gamma", "beta"), ("renamed", "middle", "last")] {
        let text = format!(
            "definition let n be object; algorithm tail(n) -> object do return {a}(n); end; algorithm {a}(n) -> object do return {b}(n); return {b}(n); end; algorithm {b}(n) -> object do return {c}(n); end; algorithm {c}(n) -> object do return {a}(n); end; algorithm quiet(n) -> object do return n; end; end;"
        );
        let fixture = SourceFixture::parse(&text, "checked_algorithm_mutual");
        let mut expected = BTreeMap::new();
        for target in [a, b, c] {
            let call = format!("{target}(n)");
            for (start, _) in text.match_indices(&format!("return {call}")) {
                expected.extend(checked_selection(
                    &fixture,
                    &text,
                    SurfaceNodeKind::ApplicationTerm,
                    &[(start + 7, &call, &format!("algorithm {target}"))],
                ));
            }
        }
        let selected =
            check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap();
        assert_eq!(selected, (BTreeMap::new(), expected, BTreeMap::new()));
        assert_eq!(
            source_algorithm_recursive_groups(
                &fixture.native,
                &fixture.symbols,
                &selected.1,
                &selected.2
            )
            .unwrap(),
            Some(vec![vec![
                checked_symbol(&fixture, &text, &format!("algorithm {a}")),
                checked_symbol(&fixture, &text, &format!("algorithm {b}")),
                checked_symbol(&fixture, &text, &format!("algorithm {c}"))
            ]])
        );
    }
}

#[test]
fn checked_algorithm_synonyms_keep_alias_occurrences_and_original_declarations() {
    for body in ["return chain(n);", "return n;"] {
        let text = format!(
            "definition let n be object; algorithm run(n) -> object do return n; end; synonym alias(n) for run(n); synonym chain(n) for alias(n); algorithm caller(n) -> object do {body} end; end;"
        );
        let fixture = SourceFixture::parse(&text, "checked_algorithm_synonyms");
        let mut originals = BTreeMap::new();
        for (marker, target) in [
            ("synonym alias", "algorithm run"),
            ("synonym chain", "synonym alias"),
        ] {
            let SourceAnchor::Range(range) =
                checked_symbol(&fixture, &text, marker).origin().anchor()
            else {
                panic!("source declaration range")
            };
            let (id, range) = checked_site(
                &fixture,
                SurfaceNodeKind::NotationAlias,
                range.start,
                range.end - range.start,
            );
            originals.insert(
                id,
                SymbolRef::new(
                    checked_symbol(&fixture, &text, target).symbol().clone(),
                    range,
                ),
            );
        }
        let applications = if body.contains("chain") {
            checked_selection(
                &fixture,
                &text,
                SurfaceNodeKind::ApplicationTerm,
                &[(
                    text.find("return chain(n)").unwrap() + 7,
                    "chain(n)",
                    "synonym chain",
                )],
            )
        } else {
            BTreeMap::new()
        };
        let selected =
            check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap();
        assert_eq!(selected, (BTreeMap::new(), applications, originals));
        assert_eq!(
            source_algorithm_recursive_groups(
                &fixture.native,
                &fixture.symbols,
                &selected.1,
                &selected.2
            )
            .unwrap(),
            Some(Vec::new())
        );
        let dependencies = source_definiens_dependencies(
            &fixture.native,
            &fixture.symbols,
            &selected.0,
            &selected.2,
        )
        .unwrap();
        assert!(dependencies.is_empty());
    }
}

#[test]
fn checked_definiens_references_feed_dependencies_without_body_application_leaks() {
    use mizar_resolve::env::{DeclarationDependencyKind, DependencyEndpoint};
    let text = "definition let n be object; func Base: base(n) -> object equals n; func Caller: caller(n) -> object equals base(n); algorithm run(n) -> object do return base(n); end; end;";
    let fixture = SourceFixture::parse(text, "checked_definiens");
    let start = text.find("equals base(n)").unwrap() + 7;
    let references = checked_selection(
        &fixture,
        text,
        SurfaceNodeKind::TermReference,
        &[(start, "base", "func Base:")],
    );
    let applications = checked_selection(
        &fixture,
        text,
        SurfaceNodeKind::ApplicationTerm,
        &[(
            text.find("return base(n)").unwrap() + 7,
            "base(n)",
            "func Base:",
        )],
    );
    let selected =
        check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
            .unwrap();
    assert_eq!(selected, (references, applications, BTreeMap::new()));
    let dependencies =
        source_definiens_dependencies(&fixture.native, &fixture.symbols, &selected.0, &selected.2)
            .unwrap();
    assert_eq!(
        dependencies
            .iter()
            .map(|edge| (
                edge.kind(),
                edge.source().clone(),
                edge.target().clone(),
                edge.anchor().clone()
            ))
            .collect::<Vec<_>>(),
        vec![(
            DeclarationDependencyKind::DefiniensMention,
            DependencyEndpoint::Symbol(
                checked_symbol(&fixture, text, "func Caller:")
                    .symbol()
                    .clone()
            ),
            DependencyEndpoint::Symbol(
                checked_symbol(&fixture, text, "func Base:")
                    .symbol()
                    .clone()
            ),
            SourceAnchor::Range(
                checked_site(
                    &fixture,
                    SurfaceNodeKind::TermReference,
                    start,
                    "base".len()
                )
                .1
            )
        )]
    );
    assert_eq!(
        source_algorithm_recursive_groups(
            &fixture.native,
            &fixture.symbols,
            &selected.1,
            &selected.2
        )
        .unwrap(),
        Some(Vec::new())
    );
    assert!(
        source_definiens_dependencies(&fixture.native, &fixture.symbols, &selected.1, &selected.2)
            .is_err()
    );
}

#[test]
fn checked_algorithm_unsupported_calls_never_fabricate_selected_edges() {
    for text in [
        "definition let s be set; func Pick: pick(s) -> set equals s; end; definition let n be object; algorithm run(n) -> object do return pick(n); end; end;",
        "definition let n be object; algorithm run(n) -> object do var x := n; return run(x); end; end;",
        "definition let n be object; algorithm run(n) -> object do var n := n; return run(n); end; end;",
        "definition let n be object; func Target: target(n) -> object equals n; end; definition let n be object; algorithm run(n) -> object do return Missing.target(n); end; end;",
        "definition let T be type; let n be T; func Pick: pick[T](n) -> T equals n; end; definition let n be object; algorithm run(n) -> object do return pick[object](n); end; end;",
        "definition let s be set; func PickSet: pick(s) -> set equals s; end; definition let n be object; func PickObject: pick(n) -> object equals n; algorithm run(n) -> object do return pick(n); end; end;",
        "definition let n be object; algorithm run(n) -> object do return bad(n); end; synonym bad(n) for run(0); end;",
    ] {
        let fixture = SourceFixture::parse(text, "checked_unsupported_calls");
        let before = (
            fixture.native.clone(),
            fixture.symbols.clone(),
            fixture.typed.clone(),
        );
        if text.starts_with("definition let s be set; func Pick:") {
            let (_, candidates) = mizar_resolve::symbols::source_algorithm_call_candidates(
                &fixture.native,
                &fixture.symbols,
            )
            .unwrap();
            let site = checked_site(
                &fixture,
                SurfaceNodeKind::ApplicationTerm,
                text.find("return pick(n)").unwrap() + 7,
                "pick(n)".len(),
            )
            .0;
            assert_eq!(
                candidates[&site],
                vec![checked_symbol(&fixture, text, "func Pick:")]
            );
        }
        let selected =
            check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap();
        assert_eq!(
            selected,
            (BTreeMap::new(), BTreeMap::new(), BTreeMap::new()),
            "{text}"
        );
        assert_eq!(
            source_algorithm_recursive_groups(
                &fixture.native,
                &fixture.symbols,
                &selected.1,
                &selected.2
            )
            .unwrap(),
            None,
            "{text}"
        );
        assert_eq!((fixture.native, fixture.symbols, fixture.typed), before);
    }
    for text in [
        "definition let n be object; func Base: f(n) -> object equals n; redefine func Changed: f(n) -> object equals n; coherence by Proof; func Caller: caller(n) -> object equals f(n); end;",
        "definition let n be object; algorithm run(n) -> object do return n; end; end; definition let n be object; let run be object; synonym alias(n) for run(n); end;",
        "definition let n be object; algorithm run(n) -> object do return n; end; end; definition let n be object; let run,m be object; synonym alias(n) for run(n); end;",
        "definition let n be object; algorithm run(n) -> object do return n; end; end; definition let n be object; let run,m be object; func Caller: caller(n) -> object equals run(n); end;",
        "definition let n be object; algorithm run(n) -> object do return n; end; end; definition let n be object; let run be object, m be object; synonym alias(n) for run(n); end;",
        "definition let n be object; algorithm run(n) -> object do return n; end; end; definition let n be object; let run be object, m be object; func Caller: caller(n) -> object equals run(n); end;",
        "definition let n be object; func Base: f(n) -> object equals n; end; reserve unused for object; definition let n be object; func Caller: caller(n) -> object equals f(n); synonym alias(n) for f(n); end;",
    ] {
        let fixture = SourceFixture::parse(text, "checked_scope_exclusions");
        let selected =
            check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap();
        assert_eq!(
            selected,
            (BTreeMap::new(), BTreeMap::new(), BTreeMap::new()),
            "{text}"
        );
    }
    let text = "definition let n be object; algorithm run(n) -> object do return run(n); return run(0); end; end;";
    let fixture = SourceFixture::parse(text, "checked_mixed_calls");
    let selected =
        check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
            .unwrap();
    let expected = checked_selection(
        &fixture,
        text,
        SurfaceNodeKind::ApplicationTerm,
        &[
            (
                text.find("return run(n)").unwrap() + 7,
                "run(n)",
                "algorithm run",
            ),
            (
                text.find("return run(0)").unwrap() + 7,
                "run(0)",
                "algorithm run",
            ),
        ],
    );
    assert_eq!(selected, (BTreeMap::new(), expected, BTreeMap::new()));
    assert_eq!(
        source_algorithm_recursive_groups(
            &fixture.native,
            &fixture.symbols,
            &selected.1,
            &selected.2
        )
        .unwrap(),
        Some(vec![vec![checked_symbol(&fixture, text, "algorithm run")]])
    );
}

#[test]
fn checked_algorithm_call_selection_refuses_foreign_and_forged_inputs_atomically() {
    use mizar_checker::typed_ast::{
        NodeRecoveryState, TypedArena, TypedAst, TypedAstParts, TypedNodeId, TypedNodeKind,
    };
    use mizar_resolve::{
        env::{SymbolEnv, SymbolEnvIndexes},
        resolved_ast::SurfaceResolvedArena,
    };
    use mizar_syntax::SurfaceAstBuilder;
    let text = "definition let n be object; algorithm run(n) -> object do return run(n); end; end;";
    let fixture = SourceFixture::parse(text, "checked_associations");
    let foreign = SourceFixture::parse(text, "checked_foreign");
    let stale = SourceFixture::parse(
        "definition let n be set; algorithm run(n) -> set do return run(n); end; end;",
        "checked_associations",
    );
    let before = (
        fixture.native.clone(),
        fixture.symbols.clone(),
        fixture.typed.clone(),
    );
    let expected =
        check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
            .unwrap();
    assert!(!expected.1.is_empty());
    for other in [&foreign, &stale] {
        assert!(
            check_source_algorithm_call_targets(&other.native, &fixture.symbols, &fixture.typed)
                .is_err()
        );
        assert!(
            check_source_algorithm_call_targets(&fixture.native, &other.symbols, &fixture.typed)
                .is_err()
        );
        assert!(
            check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &other.typed)
                .is_err()
        );
    }
    let mut indexes = SymbolEnvIndexes {
        imports: fixture.symbols.imports().clone(),
        exports: fixture.symbols.exports().clone(),
        symbols: fixture.symbols.symbols().clone(),
        labels: fixture.symbols.labels().clone(),
        definitions: fixture.symbols.definitions().clone(),
        overloads: fixture.symbols.overloads().clone(),
        registrations: fixture.symbols.registrations().clone(),
        lexical_summaries: fixture.symbols.lexical_summaries().clone(),
        namespace_graph: fixture.symbols.namespace_graph().clone(),
        declaration_dependencies: fixture.symbols.declaration_dependencies().clone(),
        contributions: fixture.symbols.contributions().clone(),
        module_summaries: fixture.symbols.module_summaries().clone(),
    };
    indexes
        .symbols
        .insert(foreign.symbols.symbols().iter().next().unwrap().clone());
    let augmented_env = SymbolEnv::new(fixture.native.module().clone(), indexes);
    assert!(
        check_source_algorithm_call_targets(&fixture.native, &augmented_env, &fixture.typed)
            .is_err()
    );
    let call = checked_site(
        &fixture,
        SurfaceNodeKind::ApplicationTerm,
        text.rfind("run(n)").unwrap(),
        "run(n)".len(),
    )
    .0;
    for mutation in 0..7 {
        let mut nodes: Vec<_> = fixture
            .typed
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect();
        let original = nodes[call.index()].clone();
        match mutation {
            0 => nodes[call.index()].kind = TypedNodeKind::new("ForgedApplication"),
            1 => nodes[call.index()].resolved_node = fixture.typed.resolved_root(),
            2 => {
                nodes[call.index()].anchor = fixture
                    .typed
                    .nodes()
                    .node(fixture.typed.nodes().root().unwrap())
                    .unwrap()
                    .anchor
                    .clone()
            }
            3 => nodes[call.index()].children.clear(),
            4 => nodes[call.index()].recovery = NodeRecoveryState::Recovered,
            5 => nodes.push(original),
            6 => {
                nodes[call.index()].children.reverse();
            }
            _ => unreachable!(),
        }
        let mut typed = TypedAst::try_new(TypedAstParts {
            source_id: fixture.typed.source_id(),
            module_id: fixture.typed.module_id().clone(),
            resolved_root: fixture.typed.resolved_root(),
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: TypedArena::try_new(
                Some(TypedNodeId::new(fixture.native.arena().root().index())),
                nodes,
            )
            .unwrap(),
            contexts: Default::default(),
            types: Default::default(),
            facts: Default::default(),
            coercions: Default::default(),
            initial_obligations: Default::default(),
            diagnostics: Default::default(),
        })
        .expect("structurally valid forged typed AST");
        let _ = typed.install_source_nodes(&fixture.native);
        assert!(
            check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &typed).is_err(),
            "mutation {mutation}"
        );
    }
    let mut builder = SurfaceAstBuilder::new(fixture.native.source_id());
    let mut rebuilt = Vec::new();
    let mut marked_recovered = false;
    for (_, node) in fixture.native.arena().iter() {
        let SourceAnchor::Range(range) = node.origin().anchor() else {
            panic!("source range")
        };
        let children = node
            .children()
            .iter()
            .map(|child| rebuilt[child.index()])
            .collect();
        let id = match node.kind() {
            SurfaceNodeKind::Token(token) if !marked_recovered => {
                marked_recovered = true;
                builder.add_recovered_token(token.kind, token.text.clone(), *range)
            }
            SurfaceNodeKind::Token(token) => {
                builder.add_token(token.kind, token.text.clone(), *range)
            }
            kind => builder.add_node(kind.clone(), *range, children),
        };
        rebuilt.push(id);
    }
    assert!(marked_recovered);
    let recovered = builder.finish(Some(rebuilt[fixture.native.arena().root().index()]), None);
    let source = SurfaceResolvedArena::lower(&recovered, fixture.native.module()).unwrap();
    assert!(
        check_source_algorithm_call_targets(&source, &fixture.symbols, &fixture.typed).is_err()
    );
    assert_eq!(
        check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
            .unwrap(),
        expected
    );
    assert_eq!((fixture.native, fixture.symbols, fixture.typed), before);
}

fn resolved_root(handoff: &SourceOverloadHandoff) -> OverloadCandidateId {
    let row = handoff
        .selection()
        .results()
        .get(OverloadResultId::new(0))
        .expect("one selected overload result");
    match row.status {
        OverloadResultStatus::Resolved { root, .. } => root,
        ref status => panic!("expected an ordinary root, found {status:?}"),
    }
}

fn add_object_widening_evidence(
    input: &mut SourceOverloadInput,
    fact: mizar_checker::typed_ast::TypeFactId,
) {
    let ArgumentViabilityEvidence::Exact { actual } = &input.viability[1].arguments[0] else {
        panic!("source intake starts with exact argument evidence");
    };
    input.viability[1].arguments[0] = ArgumentViabilityEvidence::FactWidening {
        actual: *actual,
        target: input.candidates[1].parameters[0],
        facts: vec![fact],
        status: ViabilityFactStatus::Consumable,
    };
}

fn source_anchored_overloads() -> (
    SourceFixture,
    mizar_checker::typed_ast::TypedAst,
    mizar_checker::typed_ast::TypeFactId,
) {
    let fixture = ordinary_overloads();
    let (initial, _) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
            .expect("derive source call and argument");
    let (typed, fact) = fixture.with_known_fact(initial.sites[0].arguments[0].clone(), "object");
    (fixture, typed, fact)
}

#[test]
fn ordinary_multicandidate_site_uses_explicit_evidence_through_all_stages() {
    let (fixture, typed, fact) = source_anchored_overloads();
    let (mut input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &typed)
            .expect("derive authenticated source overload rows");
    assert_eq!(input.sites.len(), 1);
    assert_eq!(input.candidates.len(), 2);
    add_object_widening_evidence(&mut input, fact);
    contest(&mut input, 0, 1, "source-specificity-set-over-object");

    let handoff = SourceOverloadHandoff::build(
        input.clone(),
        &fixture.native,
        &fixture.symbols,
        &typed,
        &normalization,
    )
    .expect("build source-authenticated selection pipeline");

    assert_eq!(handoff.source_id(), fixture.native.source_id());
    assert_eq!(handoff.module_id(), fixture.native.module());
    assert_eq!(handoff.collection().sites().len(), 1);
    assert_eq!(handoff.collection().candidates().len(), 2);
    assert_eq!(handoff.expansion().candidates().len(), 2);
    assert_eq!(handoff.viability().decisions().len(), 2);
    assert_eq!(handoff.specificity().graphs().len(), 1);
    assert_eq!(
        handoff
            .specificity()
            .graphs()
            .get(SpecificityGraphId::new(0))
            .unwrap()
            .comparisons
            .len(),
        1
    );
    let root = resolved_root(&handoff);
    assert_eq!(root, OverloadCandidateId::new(0));
    let selected = handoff.collection().candidates().get(root).unwrap();
    assert_eq!(selected.symbol, input.candidates[root.index()].symbol);
    assert_eq!(selected.ordinary_root, selected.symbol);
    assert!(input.candidates.iter().all(|candidate| {
        candidate
            .provenance
            .source_range
            .is_some_and(|range| range.source_id == fixture.native.source_id())
    }));

    let installed = typed
        .clone()
        .with_source_overload(handoff.clone())
        .expect("install immutable handoff in TypedAst");
    assert_eq!(installed.source_overload(), Some(&handoff));
    assert_eq!(
        handoff.debug_text(),
        SourceOverloadHandoff::build(
            input,
            &fixture.native,
            &fixture.symbols,
            &typed,
            &normalization,
        )
        .unwrap()
        .debug_text()
    );
}

#[test]
fn template_site_uses_native_direct_template_roles_and_selects_template_root() {
    let fixture = template_overload();
    let (input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
            .expect("derive template candidate from direct source roles");
    assert_eq!(input.sites.len(), 1);
    assert_eq!(input.candidates.len(), 1);
    let candidate = &input.candidates[0];
    assert!(candidate.template.is_some());
    assert_eq!(
        candidate.declaration_kind,
        CandidateDeclarationKind::Template
    );
    let template_argument = &candidate.template.as_ref().unwrap().arguments;
    assert_eq!(template_argument.len(), 1);
    assert!(matches!(
        template_argument[0],
        mizar_checker::overload_resolution::TemplateArgument::Explicit(_)
    ));

    let handoff = SourceOverloadHandoff::build(
        input,
        &fixture.native,
        &fixture.symbols,
        &fixture.typed,
        &normalization,
    )
    .expect("build template overload pipeline");
    assert_eq!(handoff.expansion().expansions().len(), 1);
    assert_eq!(handoff.viability().decisions().len(), 1);
    assert_eq!(resolved_root(&handoff), OverloadCandidateId::new(0));
    assert!(
        handoff
            .collection()
            .candidates()
            .get(OverloadCandidateId::new(0))
            .unwrap()
            .template
            .is_some()
    );
}

#[test]
fn one_source_candidate_keeps_its_full_payload_and_root() {
    let fixture = single_ordinary();
    let (input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
            .expect("derive source overload rows");
    assert_eq!(input.candidates.len(), 1);
    let source_symbol = input.candidates[0].symbol.clone();
    let source_result = input.candidates[0].result;
    let source_parameters = input.candidates[0].parameters.clone();
    let handoff = SourceOverloadHandoff::build(
        input,
        &fixture.native,
        &fixture.symbols,
        &fixture.typed,
        &normalization,
    )
    .expect("build single-candidate selection pipeline");

    assert_eq!(resolved_root(&handoff), OverloadCandidateId::new(0));
    let candidate = handoff
        .collection()
        .candidates()
        .get(OverloadCandidateId::new(0))
        .unwrap();
    assert_eq!(candidate.symbol, source_symbol);
    assert_eq!(candidate.result, source_result);
    assert_eq!(candidate.parameters, source_parameters);
}

#[test]
fn explicit_source_qua_is_consumed_by_inserted_view_selection() {
    let fixture = source_view();
    let (initial, _) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
            .expect("derive source call and argument");
    let (typed, fact) = fixture.with_known_fact(initial.sites[0].arguments[0].clone(), "object");
    let (mut input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &typed)
            .expect("derive authenticated source view");
    let ArgumentViabilityEvidence::Exact { actual } = &input.viability[0].arguments[0] else {
        panic!("source intake starts with exact argument evidence");
    };
    input.viability[0].arguments[0] = ArgumentViabilityEvidence::FactWidening {
        actual: *actual,
        target: input.candidates[0].parameters[0],
        facts: vec![fact],
        status: ViabilityFactStatus::Consumable,
    };
    let source_qua = input.sites[0]
        .source_qua
        .first()
        .expect("written qua basis")
        .clone();
    let resolution = input.resolution.first_mut().expect("site resolution");
    resolution.inserted_views.push(InsertedViewInput {
        argument: source_qua.argument.clone(),
        target: source_qua.target,
        selected_candidate: OverloadCandidateId::new(0),
        kind: InsertedViewKind::SourceQua,
        status: InsertedViewStatus::Accepted,
        reason: InsertedViewReasonKey::new("written-source-qua"),
        evidence_facts: source_qua.evidence_facts.clone(),
        path: source_qua.path.clone(),
    });
    let handoff = SourceOverloadHandoff::build(
        input,
        &fixture.native,
        &fixture.symbols,
        &typed,
        &normalization,
    )
    .expect("build explicit source-view selection");

    let result = handoff
        .selection()
        .results()
        .get(OverloadResultId::new(0))
        .unwrap();
    let OverloadResultStatus::Resolved {
        root,
        inserted_views,
        ..
    } = &result.status
    else {
        panic!(
            "expected source-view candidate to resolve: {:?}",
            result.status
        );
    };
    assert_eq!(*root, OverloadCandidateId::new(0));
    assert_eq!(inserted_views.len(), 1);
    let view = handoff
        .selection()
        .inserted_views()
        .get(inserted_views[0])
        .expect("selected inserted view");
    assert_eq!(view.argument, source_qua.argument);
    assert_eq!(view.target, source_qua.target);
    assert_eq!(view.kind, InsertedViewKind::SourceQua);
    assert_eq!(view.reason.as_str(), "written-source-qua");
    assert!(matches!(source_qua.argument, TypedSiteRef::Node(_)));
}

#[test]
fn contested_candidates_keep_source_order_and_deterministic_winner() {
    let (fixture, typed, fact) = source_anchored_overloads();
    let (mut input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &typed)
            .expect("derive authenticated source overload rows");
    let source_order: Vec<_> = input
        .candidates
        .iter()
        .map(|candidate| candidate.provenance.source_range.unwrap().start)
        .collect();
    assert!(source_order.windows(2).all(|pair| pair[0] < pair[1]));
    add_object_widening_evidence(&mut input, fact);
    contest(&mut input, 0, 1, "source-order-contest");

    let first = SourceOverloadHandoff::build(
        input.clone(),
        &fixture.native,
        &fixture.symbols,
        &typed,
        &normalization,
    )
    .expect("build first contested resolution");
    let second = SourceOverloadHandoff::build(
        input,
        &fixture.native,
        &fixture.symbols,
        &typed,
        &normalization,
    )
    .expect("build deterministic rerun");
    assert_eq!(resolved_root(&first), OverloadCandidateId::new(0));
    assert_eq!(first.selection(), second.selection());
    assert_eq!(first.debug_text(), second.debug_text());
}
