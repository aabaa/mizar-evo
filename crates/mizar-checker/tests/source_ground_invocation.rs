#[path = "support/source_overload.rs"]
// reason: This binary reuses parser setup; overload-only fixture helpers belong to other binaries.
#[allow(dead_code)]
mod support;

use mizar_checker::{
    binding_env::{BindingContextLayer, BindingContextOwner},
    type_checker::{
        SourceGroundInvocationCheck, TermKind, TypeHeadRef, check_source_algorithm_call_targets,
        check_source_ground_invocations,
    },
    typed_ast::{TypeEntryActual, TypeStatus, TypedNodeId, TypedSiteRef},
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::SurfaceNodeKind;
use support::SourceFixture;

fn range(fixture: &SourceFixture, site: mizar_syntax::SurfaceNodeId) -> SourceRange {
    let node = fixture
        .native
        .arena()
        .node(fixture.native.resolved_node_for(site).unwrap())
        .unwrap();
    let SourceAnchor::Range(range) = node.origin().anchor() else {
        panic!("source range")
    };
    *range
}

fn assert_actuals(
    fixture: &SourceFixture,
    text: &str,
    checked: &SourceGroundInvocationCheck<'_>,
    application: mizar_syntax::SurfaceNodeId,
    expected: &[&str],
) {
    let caller = checked
        .inventory()
        .iter()
        .find(|owner| {
            owner
                .calls()
                .iter()
                .any(|call| call.application() == application)
        })
        .unwrap();
    let call = caller
        .calls()
        .iter()
        .find(|call| call.application() == application)
        .unwrap();
    assert_eq!(call.arguments().len(), expected.len());
    let app = range(fixture, application);
    let mut start = app.start + text[app.start..app.end].find('(').unwrap() + 1;
    for (&argument, &payload) in call.arguments().iter().zip(expected) {
        let actual = range(fixture, argument);
        assert_eq!(
            actual,
            SourceRange {
                source_id: fixture.native.source_id(),
                start,
                end: start + payload.len()
            }
        );
        assert_eq!(&text[actual.start..actual.end], payload);
        start = actual.end + 1;
        let resolved = fixture.native.resolved_node_for(argument).unwrap();
        let original = checked
            .typed()
            .source_node_entries()
            .unwrap()
            .find(|node| node.site() == resolved)
            .unwrap();
        assert_eq!(original.spelling(), payload);
        let terms: Vec<_> = checked
            .inference()
            .terms()
            .iter()
            .filter(|(_, term)| term.site.node().index() == resolved.index())
            .map(|(_, term)| term)
            .collect();
        assert_eq!(terms.len(), 1, "one inference row per actual occurrence");
        let term = terms[0];
        assert_eq!(
            term.site,
            TypedSiteRef::Node(TypedNodeId::new(resolved.index()))
        );
        assert_eq!(term.kind, TermKind::Numeral);
        assert!(
            term.reference.is_none() && term.candidate_set.is_none() && term.deferred.is_empty()
        );
        let ty = checked
            .inference()
            .type_entries()
            .get(term.type_entry)
            .unwrap();
        assert_eq!(ty.owner, term.site);
        assert_eq!(ty.status, TypeStatus::Known);
        let TypeEntryActual::Known(normalized) = ty.actual else {
            panic!("Known object actual")
        };
        assert_eq!(
            checked
                .inference()
                .normalized_types()
                .get(normalized)
                .unwrap()
                .head,
            TypeHeadRef::BuiltinObject
        );
        let context = checked.bindings().contexts().get(term.context).unwrap();
        assert_eq!(
            context.owner,
            BindingContextOwner::SourceStatement {
                source_range: range(fixture, caller.body())
            }
        );
        assert_eq!(context.layer, BindingContextLayer::Block);
        assert!(
            context.bindings.is_empty()
                && context.visible_bindings.is_empty()
                && context.lexical_scope.is_none()
        );
    }
}

#[test]
fn genuine_ground_invocation_retains_target_caller_and_original_numeral() {
    let text =
        include_str!("../../../tests/miz/pass/parser/pass_parser_typed_ground_invocation_001.miz");
    let fixture = SourceFixture::parse(text, "ground_invocation");
    let before = (
        fixture.native.clone(),
        fixture.symbols.clone(),
        fixture.typed.clone(),
    );
    let checked =
        check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed).unwrap();
    assert!(std::ptr::eq(checked.source(), &fixture.native));
    assert!(std::ptr::eq(checked.typed(), &fixture.typed));
    assert_eq!(checked.invocations().len(), 1);
    let (&application, selected) = checked.invocations().iter().next().unwrap();
    let application_range = range(&fixture, application);
    let start = text.find("idalgo(0)").unwrap();
    assert_eq!(
        application_range,
        SourceRange {
            source_id: fixture.native.source_id(),
            start,
            end: start + "idalgo(0)".len()
        }
    );
    assert_eq!(selected.range(), application_range);
    let caller = checked
        .inventory()
        .iter()
        .find(|owner| {
            owner
                .calls()
                .iter()
                .any(|call| call.application() == application)
        })
        .unwrap();
    let target = checked
        .inventory()
        .iter()
        .find(|owner| owner.owner().symbol() == selected.symbol())
        .unwrap();
    assert_eq!(
        range(&fixture, target.declaration()).start,
        text.find("terminating algorithm idalgo").unwrap()
    );
    assert_eq!(
        range(&fixture, caller.declaration()).start,
        text.find("algorithm ground_caller").unwrap()
    );
    assert_eq!(
        target.owner().origin().anchor(),
        &SourceAnchor::Range(range(&fixture, target.declaration()))
    );
    let call = caller
        .calls()
        .iter()
        .find(|call| call.application() == application)
        .unwrap();
    assert_eq!(call.region(), caller.body());
    let actual = call.arguments()[0];
    let actual_node = fixture.native.resolved_node_for(actual).unwrap();
    let original = checked
        .typed()
        .source_node_entries()
        .unwrap()
        .find(|node| node.site() == actual_node)
        .unwrap();
    assert_eq!(original.kind(), &SurfaceNodeKind::TermExpression);
    let node = fixture.native.arena().node(actual_node).unwrap();
    assert!(
        node.children()
            .iter()
            .any(|child| fixture.native.arena().node(*child).unwrap().kind()
                == &SurfaceNodeKind::NumeralTerm)
    );
    assert_eq!(checked.inference().terms().len(), 1);
    assert_actuals(&fixture, text, &checked, application, &["0"]);
    let selected_edges =
        check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
            .unwrap();
    assert_eq!(&selected_edges.1, checked.invocations());
    assert_eq!(
        (
            fixture.native.clone(),
            fixture.symbols.clone(),
            fixture.typed.clone()
        ),
        before
    );
}

#[test]
fn repeated_multi_actual_calls_keep_formal_order_and_distinct_occurrences() {
    for target in ["pair", "renamed_pair"] {
        let text = format!(
            "definition let a,b be object; algorithm {target}(a,b) -> object do return a; end; algorithm caller(a) -> object do return {target}(7,7); return {target}(2,31); return {target}(7,7); end; end;"
        );
        let fixture = SourceFixture::parse(&text, "ground_multi");
        let before = (
            fixture.native.clone(),
            fixture.symbols.clone(),
            fixture.typed.clone(),
        );
        let checked =
            check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap();
        assert_eq!(checked.invocations().len(), 3);
        assert_eq!(checked.inference().terms().len(), 6);
        let mut applications: Vec<_> = checked.invocations().keys().copied().collect();
        applications.sort_by_key(|&id| range(&fixture, id).start);
        for (&application, expected) in
            applications
                .iter()
                .zip([&["7", "7"][..], &["2", "31"], &["7", "7"]])
        {
            assert_actuals(&fixture, &text, &checked, application, expected);
            let selected = &checked.invocations()[&application];
            let declaration = checked
                .inventory()
                .iter()
                .find(|owner| owner.owner().symbol() == selected.symbol())
                .unwrap();
            assert_eq!(
                range(&fixture, declaration.declaration()).start,
                text.find(&format!("algorithm {target}")).unwrap()
            );
        }
        let sites: std::collections::BTreeSet<_> = checked
            .inference()
            .terms()
            .iter()
            .map(|(_, term)| term.site.clone())
            .collect();
        assert_eq!(
            sites.len(),
            6,
            "equal numeral values retain separate occurrences"
        );
        let repeated =
            check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap();
        assert_eq!(checked.invocations(), repeated.invocations());
        assert_eq!(checked.inference(), repeated.inference());
        assert_eq!(checked.bindings(), repeated.bindings());
        assert_eq!(
            (
                fixture.native.clone(),
                fixture.symbols.clone(),
                fixture.typed.clone()
            ),
            before
        );
    }
}

#[test]
fn incompatible_arity_types_and_overloaded_roots_never_choose_by_result() {
    for text in [
        "definition let a be object; algorithm target(a) -> object do return a; end; algorithm caller(a) -> object do return target(0,1); end; end;",
        "definition let a be set; algorithm target(a) -> object do return a; end; end; definition let a be object; algorithm caller(a) -> object do return target(0); end; end;",
        "definition let a be object; algorithm target(a) -> set do return a; end; algorithm caller(a) -> object do return target(0); end; end;",
        "definition let a be set; func TargetSet: target(a) -> set equals a; end; definition let a be object; algorithm target(a) -> object do return a; end; algorithm caller(a) -> object do return target(0); end; end;",
    ] {
        let fixture = SourceFixture::parse(text, "ground_mismatch");
        let checked =
            check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap_or_else(|error| panic!("{text}: {error}"));
        assert!(checked.invocations().is_empty(), "{text}");
        assert!(
            checked.inference().terms().is_empty(),
            "refused sites emit no actual rows"
        );
    }
    let conflicting_results = SourceFixture::parse(
        "definition let a be object; algorithm target(a) -> set do return a; end; end; definition let a be object; algorithm target(a) -> object do return a; end; algorithm caller(a) -> object do return target(0); end; end;",
        "ground_conflicting_results",
    );
    assert!(
        check_source_ground_invocations(
            &conflicting_results.native,
            &conflicting_results.symbols,
            &conflicting_results.typed
        )
        .is_err(),
        "equal parameter signatures cannot be distinguished by result types"
    );
}

#[test]
fn unsupported_sites_preserve_variable_edges_and_only_admit_ground_inner_sites() {
    let variable = "definition let a be object; algorithm target(a) -> object do return a; end; algorithm caller(a) -> object do return target(a); end; end;";
    let fixture = SourceFixture::parse(variable, "ground_variable");
    let selected =
        check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
            .unwrap();
    assert_eq!(
        selected.1.len(),
        1,
        "existing variable edge remains selected"
    );
    assert!(
        check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed)
            .unwrap()
            .invocations()
            .is_empty()
    );
    assert_eq!(
        check_source_algorithm_call_targets(&fixture.native, &fixture.symbols, &fixture.typed)
            .unwrap(),
        selected
    );
    let nested = variable.replace("target(a); end; end;", "target(target(0)); end; end;");
    let fixture = SourceFixture::parse(&nested, "ground_nested");
    let checked =
        check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed).unwrap();
    assert_eq!(checked.invocations().len(), 1);
    let application = *checked.invocations().keys().next().unwrap();
    assert_eq!(
        range(&fixture, application).start,
        nested.rfind("target(0)").unwrap()
    );
    assert_actuals(&fixture, &nested, &checked, application, &["0"]);
    for text in [
        "definition let a be object; algorithm target(a) -> object do return a; end; algorithm caller(a) -> object do return Missing.target(0); end; end;",
        "definition let a be object; algorithm target(a) -> object do return a; end; algorithm caller(a) -> object do return alias(0); end; synonym alias(a) for target(a); end;",
        "definition let T be type; let a be object; algorithm target[T](a) -> object do return a; end; algorithm caller(a) -> object do return target[object](0); end; end;",
        "definition let a be object; algorithm target(a) -> object do return a; end; end; definition let T be type; let a be object; algorithm caller[T](a) -> object do return target(0); end; end;",
        "definition let a be object; algorithm target(a) -> object do return a; end; algorithm caller(a) -> object do var target := a; return target(0); end; end;",
        "definition let a be object; algorithm target(a) -> object do return a; end; algorithm caller(a) -> object requires target(0) = a do return a; end; end;",
        "definition let a be object; attr Marked: a is marked means a = a; end; definition let a be marked object; algorithm target(a) -> object do return a; end; end; definition let a be object; algorithm caller(a) -> object do return target(0); end; end;",
        "definition let a be object; attr Marked: a is marked means a = a; algorithm target(a) -> marked object do return a; end; algorithm caller(a) -> object do return target(0); end; end;",
    ] {
        let fixture = SourceFixture::parse(text, "ground_unsupported");
        let checked =
            check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap();
        assert!(checked.invocations().is_empty(), "{text}");
        assert!(checked.inference().terms().is_empty());
    }
}

#[test]
fn foreign_and_changed_associations_refuse_atomically() {
    use mizar_checker::typed_ast::{
        NodeRecoveryState, TypedArena, TypedAst, TypedAstParts, TypedNodeId, TypedNodeKind,
    };
    use mizar_resolve::{
        env::{SymbolEnv, SymbolEnvIndexes},
        resolved_ast::SurfaceResolvedArena,
    };
    use mizar_syntax::SurfaceAstBuilder;
    let text = "definition let a be object; algorithm target(a) -> object do return a; end; algorithm caller(a) -> object do return target(0); return target(1); end; end;";
    let fixture = SourceFixture::parse(text, "ground_authentication");
    let foreign = SourceFixture::parse(text, "ground_foreign");
    let original =
        check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed).unwrap();
    assert_eq!(original.invocations().len(), 2);
    assert!(
        check_source_ground_invocations(&foreign.native, &fixture.symbols, &fixture.typed).is_err()
    );
    assert!(
        check_source_ground_invocations(&fixture.native, &foreign.symbols, &fixture.typed).is_err()
    );
    assert!(
        check_source_ground_invocations(&fixture.native, &fixture.symbols, &foreign.typed).is_err()
    );
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
    let augmented = SymbolEnv::new(fixture.native.module().clone(), indexes);
    assert!(check_source_ground_invocations(&fixture.native, &augmented, &fixture.typed).is_err());
    let application = original
        .invocations()
        .keys()
        .max_by_key(|&&site| range(&fixture, site).start)
        .unwrap();
    for mutation in 0..6 {
        let mut nodes: Vec<_> = fixture
            .typed
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect();
        let node = &mut nodes[application.index()];
        match mutation {
            0 => node.kind = TypedNodeKind::new("ForgedGroundApplication"),
            1 => node.resolved_node = fixture.typed.resolved_root(),
            2 => {
                if let SourceAnchor::Range(span) = &mut node.anchor {
                    span.start += 1;
                }
            }
            3 => node.children.clear(),
            4 => node.children.reverse(),
            5 => node.recovery = NodeRecoveryState::Recovered,
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
        .unwrap();
        let _ = typed.install_source_nodes(&fixture.native);
        assert!(
            check_source_ground_invocations(&fixture.native, &fixture.symbols, &typed).is_err(),
            "mutation {mutation} after an earlier valid call"
        );
    }
    let mut builder = SurfaceAstBuilder::new(fixture.native.source_id());
    let mut rebuilt = Vec::new();
    let mut recovered = false;
    for (_, node) in fixture.native.arena().iter() {
        let SourceAnchor::Range(span) = node.origin().anchor() else {
            panic!("source range")
        };
        let children = node
            .children()
            .iter()
            .map(|child| rebuilt[child.index()])
            .collect();
        rebuilt.push(match node.kind() {
            SurfaceNodeKind::Token(token) if !recovered => {
                recovered = true;
                builder.add_recovered_token(token.kind, token.text.clone(), *span)
            }
            SurfaceNodeKind::Token(token) => {
                builder.add_token(token.kind, token.text.clone(), *span)
            }
            kind => builder.add_node(kind.clone(), *span, children),
        });
    }
    let ast = builder.finish(Some(rebuilt[fixture.native.arena().root().index()]), None);
    let recovered_source = SurfaceResolvedArena::lower(&ast, fixture.native.module()).unwrap();
    assert!(
        check_source_ground_invocations(&recovered_source, &fixture.symbols, &fixture.typed)
            .is_err()
    );
    assert_eq!(
        check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed)
            .unwrap()
            .invocations(),
        original.invocations()
    );
}

#[test]
fn numeral_payload_and_caller_context_stay_bound_to_each_original_source() {
    let mut source_ids = Vec::new();
    for (target, payload) in [
        ("identity", "17"),
        ("renamed", "99999999999999999999999999999999999999"),
    ] {
        let text = format!(
            "definition let a be object; algorithm {target}(a) -> object do return a; end; algorithm first(a) -> object do return {target}({payload}); end; algorithm second(a) -> object do return {target}({payload}); end; end;"
        );
        let fixture = SourceFixture::parse(&text, "ground_payload");
        let before = fixture.typed.clone();
        assert!(!source_ids.contains(&fixture.native.source_id()));
        source_ids.push(fixture.native.source_id());
        let checked =
            check_source_ground_invocations(&fixture.native, &fixture.symbols, &fixture.typed)
                .unwrap();
        assert_eq!(checked.invocations().len(), 2);
        assert_eq!(checked.inference().terms().len(), 2);
        for &application in checked.invocations().keys() {
            assert_actuals(&fixture, &text, &checked, application, &[payload]);
        }
        let contexts: std::collections::BTreeSet<_> = checked
            .inference()
            .terms()
            .iter()
            .map(|(_, term)| term.context)
            .collect();
        assert_eq!(
            contexts.len(),
            2,
            "each caller owns a separate provenance context"
        );
        assert!(checked.bindings().bindings().is_empty());
        assert!(
            checked.inference().facts().is_empty()
                && checked.inference().formulas().is_empty()
                && checked.inference().candidate_sets().is_empty()
                && checked.inference().diagnostics().is_empty()
        );
        assert!(
            checked.typed().facts().is_empty()
                && checked.typed().coercions().is_empty()
                && checked.typed().initial_obligations().is_empty()
        );
        assert!(checked.typed().source_evidence().is_none());
        assert_eq!(checked.typed(), &before);
    }
}
