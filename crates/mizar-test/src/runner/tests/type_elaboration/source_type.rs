#[test]
fn active_source_type_application_fixture_preserves_exact_flat_handoff() {
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
    let plan = build_test_plan(&config).expect("Task 249 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "fail_type_elaboration_source_type_application_payload_001"
        })
        .expect("Task 249 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 249 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.expect("Task 249 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    assert!(
        !symbols.imports().is_empty(),
        "Task 249 resolver imports: {:#?}",
        symbols.imports()
    );
    let first = source_type_application_output(&ast, resolver.module.clone(), &symbols)
        .expect("Task 249 exact source should select the bounded route")
        .expect("Task 249 exact source payload should be valid");
    let second = source_type_application_output(&ast, resolver.module.clone(), &symbols)
        .expect("Task 249 repeated source should remain selected")
        .expect("Task 249 repeated source payload should remain valid");

    assert!(first.typed_ast.source_context().is_none());
    let handoff = first
        .typed_ast
        .source_type()
        .expect("TypedAst should own the final Task 249 handoff");
    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &resolver.module);
    assert_eq!(handoff.applications().len(), 10);
    assert_eq!(handoff.expressions().len(), 13);
    assert_eq!(handoff.arguments().len(), 6);
    assert_eq!(first.resolved.source_type(), Some(handoff));

    let mut outer_forms = [0_usize; 4];
    for (_, application) in handoff.applications().iter() {
        let expression = handoff
            .expressions()
            .get(application.root())
            .expect("application root expression");
        increment_source_type_form(&mut outer_forms, expression.form());
    }
    assert_eq!(outer_forms, [4, 2, 1, 3]);

    let mut all_forms = [0_usize; 4];
    let mut heads = [0_usize; 6];
    for (_, expression) in handoff.expressions().iter() {
        increment_source_type_form(&mut all_forms, expression.form());
        match expression.head() {
            mizar_checker::source_type::SourceTypeHead::BuiltinSet => heads[0] += 1,
            mizar_checker::source_type::SourceTypeHead::BuiltinObject => heads[1] += 1,
            mizar_checker::source_type::SourceTypeHead::Symbol { symbol, .. } => {
                let entry = symbols
                    .symbols()
                    .get(symbol)
                    .expect("authenticated Task 249 symbol");
                let local = symbol.module() == &resolver.module;
                match (local, entry.kind()) {
                    (true, mizar_resolve::env::SymbolKind::Mode) => heads[2] += 1,
                    (true, mizar_resolve::env::SymbolKind::Structure) => heads[3] += 1,
                    (false, mizar_resolve::env::SymbolKind::Mode) => heads[4] += 1,
                    (false, mizar_resolve::env::SymbolKind::Structure) => heads[5] += 1,
                    _ => panic!("Task 249 head must be a mode or structure"),
                }
            }
            _ => panic!("Task 249 head variant is outside the frozen contract"),
        }
    }
    assert_eq!(all_forms, [7, 2, 1, 3]);
    assert_eq!(heads, [4, 1, 4, 2, 1, 1]);

    let mut arguments = [0_usize; 3];
    for (_, argument) in handoff.arguments().iter() {
        match argument.argument() {
            mizar_checker::source_type::SourceTypeArgument::TermSite { .. } => arguments[0] += 1,
            mizar_checker::source_type::SourceTypeArgument::TypeSite { .. } => arguments[1] += 1,
            mizar_checker::source_type::SourceTypeArgument::QuaSite { radix, .. } => {
                arguments[2] += 1;
                assert_eq!(radix.len(), 1);
            }
            _ => panic!("Task 249 argument variant is outside the frozen contract"),
        }
    }
    assert_eq!(arguments, [3, 2, 1]);

    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.initial_obligations().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.collection_candidates().is_empty());
    assert!(first.resolved.expanded_candidates().is_empty());
    assert!(first.resolved.template_expansions().is_empty());
    assert!(first.resolved.viable_candidates().is_empty());
    assert!(first.resolved.viability_decisions().is_empty());
    assert!(first.resolved.specificity_graphs().is_empty());
    assert!(first.resolved.resolved_overloads().is_empty());
    assert!(first.resolved.inserted_coercions().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    assert!(first
        .typed_ast
        .debug_text()
        .contains("source-type-application-debug-v1"));

    let import = symbols
        .imports()
        .iter()
        .next()
        .expect("Task 249 imported heads require a real import projection")
        .import();
    let mut corrupted = source_type_input_from_handoff(handoff);
    let qua = corrupted
        .arguments
        .iter_mut()
        .find_map(|argument| match &mut argument.argument {
            mizar_checker::source_type::SourceTypeArgument::QuaSite { provenance, .. } => {
                Some(provenance)
            }
            _ => None,
        })
        .expect("Task 249 broad handoff should contain one qua site");
    *qua = qua.clone().with_import_edge(import);
    assert!(matches!(
        mizar_checker::source_type::SourceTypeProducer::build(
            corrupted,
            &first.binding_env,
            &symbols,
            first.typed_ast.nodes(),
        ),
        Err(mizar_checker::source_type::SourceTypeError::InvalidProvenance { .. })
    ));

    let mut mismatched_import = source_type_input_from_handoff(handoff);
    let imported_mode = mismatched_import
        .expressions
        .iter_mut()
        .find(|expression| {
            let mizar_checker::source_type::SourceTypeHead::Symbol { symbol, .. } =
                &expression.head
            else {
                return false;
            };
            symbol.module() != &resolver.module
                && symbols
                    .symbols()
                    .get(symbol)
                    .is_some_and(|entry| {
                        entry.kind() == mizar_resolve::env::SymbolKind::Mode
                    })
        })
        .expect("Task 249 broad handoff should contain one imported mode");
    let wrong_module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task249-wrong-import"),
        mizar_session::ModulePath::new("task249.wrong.import"),
    );
    let mut wrong_indexes =
        super::import_fixtures::clone_symbol_env_indexes(&symbols);
    let wrong_contribution = wrong_indexes.contributions.insert(
        wrong_module.clone(),
        mizar_resolve::env::ContributionKind::ImportedSource {
            source_id: ast.source_id,
        },
        mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 0,
            end: 1,
        }),
    );
    wrong_indexes
        .contributions
        .add_import(wrong_contribution, import);
    let wrong_symbol = mizar_resolve::resolved_ast::SymbolId::new(
        wrong_module.clone(),
        mizar_resolve::resolved_ast::LocalSymbolId::new("mismatched-import-mode"),
        mizar_resolve::resolved_ast::FullyQualifiedName::new(
            "task249.wrong.import::MismatchedImportMode",
        ),
    );
    let primary_spelling = match imported_mode.form {
        mizar_checker::source_type::SourceTypeApplicationForm::Bare => {
            imported_mode.head_spelling.clone()
        }
        mizar_checker::source_type::SourceTypeApplicationForm::Of => {
            format!("{} of p", imported_mode.head_spelling)
        }
        mizar_checker::source_type::SourceTypeApplicationForm::Over => {
            format!("{} over p", imported_mode.head_spelling)
        }
        mizar_checker::source_type::SourceTypeApplicationForm::Bracket => {
            format!("{} [ p ]", imported_mode.head_spelling)
        }
        _ => panic!("Task 249 form is outside the frozen contract"),
    };
    wrong_indexes.symbols.insert(
        mizar_resolve::env::SymbolEntry::new(
            wrong_symbol.clone(),
            mizar_resolve::env::SymbolKind::Mode,
            mizar_resolve::env::NamespacePath::new(resolver.module.path().as_str()),
            primary_spelling,
            mizar_resolve::resolved_ast::SemanticOrigin::new(
                ast.source_id,
                wrong_module,
                mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                    source_id: ast.source_id,
                    start: 0,
                    end: 1,
                }),
                vec![0],
            ),
            wrong_contribution,
        )
        .with_visibility(mizar_resolve::env::Visibility::Public)
        .with_export_status(mizar_resolve::env::ExportStatus::Exported),
    );
    wrong_indexes
        .contributions
        .add_symbol(wrong_contribution, wrong_symbol.clone());
    imported_mode.head = mizar_checker::source_type::SourceTypeHead::Symbol {
        symbol: wrong_symbol,
        contribution: wrong_contribution,
    };
    let wrong_symbols =
        mizar_resolve::env::SymbolEnv::new(resolver.module.clone(), wrong_indexes);
    assert!(matches!(
        mizar_checker::source_type::SourceTypeProducer::build(
            mismatched_import,
            &first.binding_env,
            &wrong_symbols,
            first.typed_ast.nodes(),
        ),
        Err(mizar_checker::source_type::SourceTypeError::InvalidSymbolHead { .. })
    ));

    for mutation in [
        Task249ImportedHeadMutation::ContributionKind,
        Task249ImportedHeadMutation::ContributionSource,
        Task249ImportedHeadMutation::ContributionAnchor,
        Task249ImportedHeadMutation::ContributionAnchorSource,
        Task249ImportedHeadMutation::ContributionAnchorOrder,
        Task249ImportedHeadMutation::OriginModule,
        Task249ImportedHeadMutation::Visibility,
        Task249ImportedHeadMutation::ExportStatus,
    ] {
        let mut corrupted = source_type_input_from_handoff(handoff);
        let imported_mode = corrupted
            .expressions
            .iter_mut()
            .find(|expression| {
                let mizar_checker::source_type::SourceTypeHead::Symbol { symbol, .. } =
                    &expression.head
                else {
                    return false;
                };
                symbol.module() != &resolver.module
                    && symbols
                        .symbols()
                        .get(symbol)
                        .is_some_and(|entry| {
                            entry.kind() == mizar_resolve::env::SymbolKind::Mode
                        })
            })
            .expect("Task 249 broad handoff should contain one imported mode");
        let mizar_checker::source_type::SourceTypeHead::Symbol {
            symbol: original_symbol,
            ..
        } = &imported_mode.head
        else {
            unreachable!()
        };
        let imported_module = original_symbol.module().clone();
        let mut indexes =
            super::import_fixtures::clone_symbol_env_indexes(&symbols);
        let contribution_kind = match mutation {
            Task249ImportedHeadMutation::ContributionKind => {
                mizar_resolve::env::ContributionKind::LocalSource {
                    source_id: ast.source_id,
                }
            }
            Task249ImportedHeadMutation::ContributionSource => {
                mizar_resolve::env::ContributionKind::ImportedSource {
                    source_id: task248_other_source_id(),
                }
            }
            _ => mizar_resolve::env::ContributionKind::ImportedSource {
                source_id: ast.source_id,
            },
        };
        let contribution_anchor = match mutation {
            Task249ImportedHeadMutation::ContributionAnchor => {
                mizar_session::SourceAnchor::Point {
                    source_id: ast.source_id,
                    offset: 0,
                }
            }
            Task249ImportedHeadMutation::ContributionAnchorSource => {
                mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                    source_id: task248_other_source_id(),
                    start: 0,
                    end: 1,
                })
            }
            Task249ImportedHeadMutation::ContributionAnchorOrder => {
                mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                    source_id: ast.source_id,
                    start: imported_mode.head_range.end,
                    end: imported_mode.head_range.end + 1,
                })
            }
            _ => mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                source_id: ast.source_id,
                start: 0,
                end: 1,
            }),
        };
        let contribution = indexes.contributions.insert(
            imported_module.clone(),
            contribution_kind,
            contribution_anchor,
        );
        indexes.contributions.add_import(contribution, import);
        let symbol = mizar_resolve::resolved_ast::SymbolId::new(
            imported_module.clone(),
            mizar_resolve::resolved_ast::LocalSymbolId::new(format!(
                "task249-import-corruption-{mutation:?}"
            )),
            mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                "{}::Task249ImportCorruption{mutation:?}",
                imported_module.path().as_str()
            )),
        );
        let origin_module = if matches!(mutation, Task249ImportedHeadMutation::OriginModule) {
            resolver.module.clone()
        } else {
            imported_module
        };
        let entry = mizar_resolve::env::SymbolEntry::new(
            symbol.clone(),
            mizar_resolve::env::SymbolKind::Mode,
            mizar_resolve::env::NamespacePath::new(resolver.module.path().as_str()),
            source_type_primary_spelling(
                &imported_mode.head_spelling,
                imported_mode.form,
            ),
            mizar_resolve::resolved_ast::SemanticOrigin::new(
                ast.source_id,
                origin_module,
                mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                    source_id: ast.source_id,
                    start: 0,
                    end: 1,
                }),
                vec![0],
            ),
            contribution,
        );
        let entry = if matches!(mutation, Task249ImportedHeadMutation::Visibility) {
            entry.with_export_status(mizar_resolve::env::ExportStatus::Exported)
        } else if matches!(mutation, Task249ImportedHeadMutation::ExportStatus) {
            entry.with_visibility(mizar_resolve::env::Visibility::Public)
        } else {
            entry
                .with_visibility(mizar_resolve::env::Visibility::Public)
                .with_export_status(mizar_resolve::env::ExportStatus::Exported)
        };
        indexes.symbols.insert(entry);
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        imported_mode.head = mizar_checker::source_type::SourceTypeHead::Symbol {
            symbol,
            contribution,
        };
        let corrupted_symbols =
            mizar_resolve::env::SymbolEnv::new(resolver.module.clone(), indexes);
        assert!(
            matches!(
                mizar_checker::source_type::SourceTypeProducer::build(
                    corrupted,
                    &first.binding_env,
                    &corrupted_symbols,
                    first.typed_ast.nodes(),
                ),
                Err(mizar_checker::source_type::SourceTypeError::InvalidSymbolHead { .. })
            ),
            "Task 249 accepted imported-head mutation {mutation:?}"
        );
    }
}

#[test]
fn task249_exact_selector_does_not_capture_tasks68_through_71() {
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
    let plan = build_test_plan(&config).expect("Task 249 isolation plan should build");
    for id in [
        "fail_type_elaboration_argument_bearing_mode_gap_001",
        "fail_type_elaboration_argument_bearing_structure_gap_001",
        "fail_type_elaboration_bracket_mode_argument_gap_001",
        "fail_type_elaboration_bracket_structure_argument_gap_001",
    ] {
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
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        assert!(
            source_type_application_output(&ast, resolver.module, &symbols).is_none(),
            "{id} must not be captured by Task 249"
        );
    }
}

fn increment_source_type_form(
    counts: &mut [usize; 4],
    form: mizar_checker::source_type::SourceTypeApplicationForm,
) {
    match form {
        mizar_checker::source_type::SourceTypeApplicationForm::Bare => counts[0] += 1,
        mizar_checker::source_type::SourceTypeApplicationForm::Of => counts[1] += 1,
        mizar_checker::source_type::SourceTypeApplicationForm::Over => counts[2] += 1,
        mizar_checker::source_type::SourceTypeApplicationForm::Bracket => counts[3] += 1,
        _ => panic!("Task 249 form is outside the frozen contract"),
    }
}

#[derive(Clone, Copy, Debug)]
enum Task249ImportedHeadMutation {
    ContributionKind,
    ContributionSource,
    ContributionAnchor,
    ContributionAnchorSource,
    ContributionAnchorOrder,
    OriginModule,
    Visibility,
    ExportStatus,
}

fn source_type_primary_spelling(
    head: &str,
    form: mizar_checker::source_type::SourceTypeApplicationForm,
) -> String {
    match form {
        mizar_checker::source_type::SourceTypeApplicationForm::Bare => head.to_owned(),
        mizar_checker::source_type::SourceTypeApplicationForm::Of => {
            format!("{head} of p")
        }
        mizar_checker::source_type::SourceTypeApplicationForm::Over => {
            format!("{head} over p")
        }
        mizar_checker::source_type::SourceTypeApplicationForm::Bracket => {
            format!("{head} [ p ]")
        }
        _ => panic!("Task 249 form is outside the frozen contract"),
    }
}

fn source_type_input_from_handoff(
    handoff: &mizar_checker::source_type::SourceTypeApplicationHandoff,
) -> mizar_checker::source_type::SourceTypeHandoffInput {
    mizar_checker::source_type::SourceTypeHandoffInput {
        source_id: handoff.source_id(),
        module_id: handoff.module_id().clone(),
        applications: handoff
            .applications()
            .iter()
            .map(
                |(_, application)| mizar_checker::source_type::SourceTypeApplicationInput {
                    binding: application.binding(),
                    source_ordinal: application.source_ordinal(),
                    root: application.root(),
                },
            )
            .collect(),
        expressions: handoff
            .expressions()
            .iter()
            .map(
                |(_, expression)| mizar_checker::source_type::SourceTypeExpressionInput {
                    source_id: expression.source_id(),
                    module_id: expression.module_id().clone(),
                    site: expression.site().clone(),
                    source_range: expression.source_range(),
                    spelling: expression.spelling().to_owned(),
                    head_site: expression.head_site().clone(),
                    head_range: expression.head_range(),
                    head_spelling: expression.head_spelling().to_owned(),
                    form: expression.form(),
                    head: expression.head().clone(),
                    recovery: expression.recovery(),
                },
            )
            .collect(),
        arguments: handoff
            .arguments()
            .iter()
            .map(
                |(_, argument)| mizar_checker::source_type::SourceTypeArgumentInput {
                    parent: argument.parent(),
                    ordinal: argument.ordinal(),
                    argument: argument.argument().clone(),
                },
            )
            .collect(),
    }
}
