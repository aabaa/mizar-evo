#[test]
fn task251_real_routes_publish_exact_missing_requests_and_preserve_outcomes() {
    let cases = [
        (
            "fail_type_elaboration_source_type_application_payload_001",
            SourceEvidenceRouteKind::SourceType,
            (10, 13, 6),
            (0, 0, 0, 0, 0),
            [5, 3, 0],
            "type_elaboration.checker.source_evidence.dependency_input_missing",
        ),
        (
            "fail_type_elaboration_imported_attribute_gap_001",
            SourceEvidenceRouteKind::AttributedType,
            (1, 1, 0),
            (1, 1, 0, 0, 0),
            [0, 0, 1],
            "type_elaboration.checker.checker.declaration.deferred.evidence_query",
        ),
        (
            "fail_type_elaboration_attributed_reserve_gap_001",
            SourceEvidenceRouteKind::AttributedType,
            (1, 1, 0),
            (1, 1, 0, 0, 0),
            [0, 0, 1],
            "type_elaboration.checker.checker.declaration.deferred.evidence_query",
        ),
    ];
    let mut source_type_totals = [0_usize; 3];
    let mut source_attribute_totals = [0_usize; 5];
    let mut request_kinds = [0_usize; 3];

    for (id, route, type_counts, attribute_counts, kinds, detail) in cases {
        let (ast, module, symbols) = task251_real_ast(id);
        assert_eq!(
            source_evidence_detail_keys(&ast, module.clone(), &symbols),
            Some(vec![detail.to_owned()]),
            "{id}"
        );
        let first = source_evidence_output(&ast, module.clone(), &symbols)
            .unwrap_or_else(|| panic!("{id} should select Task 251"))
            .unwrap_or_else(|error| panic!("{id} Task 251 failed: {error}"));
        let second = source_evidence_output(&ast, module, &symbols)
            .unwrap_or_else(|| panic!("{id} should remain selected"))
            .unwrap_or_else(|error| panic!("{id} repeated Task 251 failed: {error}"));
        assert_eq!(first.kind, route, "{id}");

        let source_type = first
            .typed_ast
            .source_type()
            .expect("Task 251 requires Task 249");
        let actual_type_counts = (
            source_type.applications().len(),
            source_type.expressions().len(),
            source_type.arguments().len(),
        );
        assert_eq!(actual_type_counts, type_counts, "{id}");
        for (total, count) in source_type_totals
            .iter_mut()
            .zip([type_counts.0, type_counts.1, type_counts.2])
        {
            *total += count;
        }

        let actual_attribute_counts = first
            .typed_ast
            .source_attribute()
            .map_or((0, 0, 0, 0, 0), |handoff| {
                (
                    handoff.chains().len(),
                    handoff.attributes().len(),
                    handoff.qualifiers().len(),
                    handoff.argument_groups().len(),
                    handoff.arguments().len(),
                )
            });
        assert_eq!(actual_attribute_counts, attribute_counts, "{id}");
        for (total, count) in source_attribute_totals.iter_mut().zip([
            attribute_counts.0,
            attribute_counts.1,
            attribute_counts.2,
            attribute_counts.3,
            attribute_counts.4,
        ]) {
            *total += count;
        }

        let handoff = first
            .typed_ast
            .source_evidence()
            .expect("Task 251 handoff should be installed");
        assert_eq!(handoff.requests().len(), kinds.iter().sum(), "{id}");
        assert!(handoff.responses().is_empty(), "{id}");
        let mut actual_kinds = [0_usize; 3];
        for (_, request) in handoff.requests().iter() {
            assert_eq!(
                request.state(),
                mizar_checker::source_evidence::SourceEvidenceInputState::Missing,
                "{id}"
            );
            match request.kind() {
                mizar_checker::source_evidence::SourceEvidenceRequestKind::ModeExpansion => {
                    actual_kinds[0] += 1;
                }
                mizar_checker::source_evidence::SourceEvidenceRequestKind::StructureInhabitation => {
                    actual_kinds[1] += 1;
                }
                mizar_checker::source_evidence::SourceEvidenceRequestKind::AttributedTypeInhabitation => {
                    actual_kinds[2] += 1;
                }
                kind => panic!("{id} emitted an unexpected Task 251 kind: {kind:?}"),
            }
        }
        assert_eq!(actual_kinds, kinds, "{id}");
        for (total, count) in request_kinds.iter_mut().zip(kinds) {
            *total += count;
        }
        assert_eq!(
            first.typed_ast.source_evidence(),
            first.resolved.source_evidence(),
            "{id}"
        );
        assert_eq!(
            first.typed_ast.debug_text(),
            second.typed_ast.debug_text(),
            "{id}"
        );
        assert_eq!(
            first.resolved.debug_text(),
            second.resolved.debug_text(),
            "{id}"
        );
        assert!(first
            .typed_ast
            .debug_text()
            .contains("source-evidence-debug-v1"));
    }

    assert_eq!(source_type_totals, [12, 15, 6]);
    assert_eq!(source_attribute_totals, [2, 2, 0, 0, 0]);
    assert_eq!(request_kinds, [5, 3, 2]);
}

#[test]
fn task251_exact_selector_excludes_frozen_siblings() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 251 isolation plan should build");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if source_evidence_output(&ast, resolver.module, &symbols).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(
        selected,
        [
            "fail_type_elaboration_attributed_reserve_gap_001",
            "fail_type_elaboration_imported_attribute_gap_001",
            "fail_type_elaboration_source_type_application_payload_001",
        ]
    );
}

#[test]
fn task251_real_ast_injection_distinguishes_all_transport_states() {
    use mizar_checker::{
        registration_resolution::{
            ExistentialGateInput, RegistrationAttributeKey,
        },
        source_evidence::{
            SourceEvidenceDependencyRecord, SourceEvidenceInputState,
            SourceEvidenceResponseDisposition, SourceEvidenceResponseInput,
            SourceEvidenceResponseKey, SourceEvidenceResponsePayload,
            SourceEvidenceResponseProvenance, SourceEvidenceRequestId,
        },
    };

    let (ast, module, symbols) =
        task251_real_ast("fail_type_elaboration_imported_attribute_gap_001");
    let mut baseline_facts = None;
    for state in [
        SourceEvidenceInputState::Requested,
        SourceEvidenceInputState::Missing,
        SourceEvidenceInputState::Rejected,
        SourceEvidenceInputState::Supplied,
    ] {
        let first = source_evidence_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            |input, records| {
                input.requests[0].state = state;
                match state {
                    SourceEvidenceInputState::Requested | SourceEvidenceInputState::Missing => {}
                    SourceEvidenceInputState::Rejected => {
                        let request = SourceEvidenceRequestId::new(0);
                        let key = SourceEvidenceResponseKey::new("task251.rejected");
                        input.responses.push(SourceEvidenceResponseInput {
                            request,
                            ordinal: 0,
                            key: key.clone(),
                        });
                        records.push(SourceEvidenceDependencyRecord::new(
                            key,
                            request,
                            SourceEvidenceResponseDisposition::Rejected,
                            SourceEvidenceResponseProvenance::ExplicitInput,
                            None,
                        ));
                    }
                    SourceEvidenceInputState::Supplied => {
                        let request = SourceEvidenceRequestId::new(0);
                        let key = SourceEvidenceResponseKey::new("task251.supplied");
                        let gate = ExistentialGateInput::new(
                            input.requests[0].owner.clone(),
                            input.requests[0].source_range,
                            "task251.pattern",
                            "task251.trigger",
                            Vec::<RegistrationAttributeKey>::new(),
                        );
                        input.responses.push(SourceEvidenceResponseInput {
                            request,
                            ordinal: 0,
                            key: key.clone(),
                        });
                        records.push(SourceEvidenceDependencyRecord::new(
                            key,
                            request,
                            SourceEvidenceResponseDisposition::Supplied,
                            SourceEvidenceResponseProvenance::ExplicitInput,
                            Some(SourceEvidenceResponsePayload::ExistentialGate(gate)),
                        ));
                    }
                    _ => panic!("Task 251 state is outside the frozen contract"),
                }
            },
        )
        .expect("real Task 84 AST should select Task 251")
        .unwrap_or_else(|error| panic!("{state:?} injection failed: {error}"));
        let second = source_evidence_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            |input, records| {
                input.requests[0].state = state;
                match state {
                    SourceEvidenceInputState::Requested | SourceEvidenceInputState::Missing => {}
                    SourceEvidenceInputState::Rejected => {
                        let request = SourceEvidenceRequestId::new(0);
                        let key = SourceEvidenceResponseKey::new("task251.rejected");
                        input.responses.push(SourceEvidenceResponseInput {
                            request,
                            ordinal: 0,
                            key: key.clone(),
                        });
                        records.push(SourceEvidenceDependencyRecord::new(
                            key,
                            request,
                            SourceEvidenceResponseDisposition::Rejected,
                            SourceEvidenceResponseProvenance::ExplicitInput,
                            None,
                        ));
                    }
                    SourceEvidenceInputState::Supplied => {
                        let request = SourceEvidenceRequestId::new(0);
                        let key = SourceEvidenceResponseKey::new("task251.supplied");
                        let gate = ExistentialGateInput::new(
                            input.requests[0].owner.clone(),
                            input.requests[0].source_range,
                            "task251.pattern",
                            "task251.trigger",
                            Vec::<RegistrationAttributeKey>::new(),
                        );
                        input.responses.push(SourceEvidenceResponseInput {
                            request,
                            ordinal: 0,
                            key: key.clone(),
                        });
                        records.push(SourceEvidenceDependencyRecord::new(
                            key,
                            request,
                            SourceEvidenceResponseDisposition::Supplied,
                            SourceEvidenceResponseProvenance::ExplicitInput,
                            Some(SourceEvidenceResponsePayload::ExistentialGate(gate)),
                        ));
                    }
                    _ => panic!("Task 251 state is outside the frozen contract"),
                }
            },
        )
        .expect("repeated real Task 84 AST should select Task 251")
        .unwrap_or_else(|error| panic!("repeated {state:?} injection failed: {error}"));

        let handoff = first
            .typed_ast
            .source_evidence()
            .expect("injected handoff");
        let request = handoff
            .requests()
            .get(SourceEvidenceRequestId::new(0))
            .expect("one attributed request");
        assert_eq!(request.state(), state);
        assert_eq!(
            handoff.responses().len(),
            usize::from(matches!(
                state,
                SourceEvidenceInputState::Rejected | SourceEvidenceInputState::Supplied
            ))
        );
        assert_eq!(
            first.typed_ast.source_evidence(),
            first.resolved.source_evidence()
        );
        let fact_count = first.typed_ast.facts().len();
        assert_eq!(*baseline_facts.get_or_insert(fact_count), fact_count);
        assert_eq!(
            first.typed_ast.debug_text(),
            second.typed_ast.debug_text()
        );
        assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    }
}

#[test]
fn task251_real_ast_corrupt_injection_fails_closed() {
    use mizar_checker::source_evidence::{
        SourceEvidenceDependencyRecord, SourceEvidenceInputState,
        SourceEvidenceResponseDisposition, SourceEvidenceResponseInput,
        SourceEvidenceResponseKey, SourceEvidenceResponseProvenance,
        SourceEvidenceRequestId,
    };

    let (attribute_ast, attribute_module, attribute_symbols) =
        task251_real_ast("fail_type_elaboration_imported_attribute_gap_001");
    let missing_catalog = || {
        source_evidence_output_with_mutation(
            &attribute_ast,
            attribute_module.clone(),
            &attribute_symbols,
            |input, _| {
                input.requests[0].state = SourceEvidenceInputState::Rejected;
                input.responses.push(SourceEvidenceResponseInput {
                    request: SourceEvidenceRequestId::new(0),
                    ordinal: 0,
                    key: SourceEvidenceResponseKey::new("task251.missing"),
                });
            },
        )
        .expect("Task 84 should select Task 251")
        .expect_err("missing catalog key must fail")
    };
    assert_eq!(missing_catalog(), missing_catalog());

    let stale = source_evidence_output_with_mutation(
        &attribute_ast,
        attribute_module,
        &attribute_symbols,
        |_, records| {
            records.push(SourceEvidenceDependencyRecord::new(
                SourceEvidenceResponseKey::new("task251.stale"),
                SourceEvidenceRequestId::new(0),
                SourceEvidenceResponseDisposition::Rejected,
                SourceEvidenceResponseProvenance::ExplicitInput,
                None,
            ));
        },
    )
    .expect("Task 84 should remain selected")
    .expect_err("unconsumed catalog row must fail");
    assert!(stale.contains("catalog") || stale.contains("stale"));

    let (broad_ast, broad_module, broad_symbols) =
        task251_real_ast("fail_type_elaboration_source_type_application_payload_001");
    let cross_request = source_evidence_output_with_mutation(
        &broad_ast,
        broad_module.clone(),
        &broad_symbols,
        |input, records| {
            input.requests[0].state = SourceEvidenceInputState::Rejected;
            let key = SourceEvidenceResponseKey::new("task251.cross-request");
            input.responses.push(SourceEvidenceResponseInput {
                request: SourceEvidenceRequestId::new(0),
                ordinal: 0,
                key: key.clone(),
            });
            records.push(SourceEvidenceDependencyRecord::new(
                key,
                SourceEvidenceRequestId::new(1),
                SourceEvidenceResponseDisposition::Rejected,
                SourceEvidenceResponseProvenance::ExplicitInput,
                None,
            ));
        },
    )
    .expect("broad route should select Task 251")
    .expect_err("cross-request catalog row must fail");
    assert!(cross_request.contains("request") || cross_request.contains("catalog"));

    let wrong_site = source_evidence_output_with_mutation(
        &broad_ast,
        broad_module,
        &broad_symbols,
        |input, _| {
            input.requests[0].site = input.requests[0].owner.clone();
        },
    )
    .expect("broad route should remain selected")
    .expect_err("wrong request site must fail");
    assert!(wrong_site.contains("request") || wrong_site.contains("source"));
}

fn task251_real_ast(id: &str) -> (SurfaceAst, ResolverModuleId, SymbolEnv) {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 251 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == id)
        .unwrap_or_else(|| panic!("{id} should remain active"));
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .unwrap_or_else(|error| panic!("{id} frontend failed: {error}"));
    assert!(frontend.diagnostics.is_empty(), "{id}");
    let ast = frontend.ast.unwrap_or_else(|| panic!("{id} AST"));
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty(), "{id}");
    let module = resolver.module;
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
    (ast, module, symbols)
}
