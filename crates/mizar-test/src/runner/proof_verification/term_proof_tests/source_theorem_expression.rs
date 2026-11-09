use super::*;

#[test]
fn source_theorem_expression_metadata_retains_genuine_sites_and_consumers() -> Result<(), String> {
    use mizar_checker::{
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::*,
        source_atomic_formula::SourceAtomicFormulaKind,
        source_term::SourcePrimaryTermKind,
        type_checker::{TermFormulaChecker as Checker, *},
        typed_ast::*,
    };
    use mizar_core::{
        core_ir::{CoreProofStatus, ObligationSeedStatus},
        elaborator::lower_source_theorem_skeletons,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOrigin, SourceOriginInput, hash_text,
    };
    let fixture = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let sources = [
        fixture.to_owned(),
        fixture.replace("Lem1", "Seed").replace("let X be set;", "let Local be set;").replace("thus X = X", "thus Local = Local"),
        "theorem Pair: for A, Spare, B being set holds A = B proof let L, Unused, R be set; thus L = R; end;".into(),
        "theorem Nested: for A being set for Unused being set holds for B being set holds A = B proof let L be set; let Idle be set; let R be set; thus L = R; end;".into(),
        "theorem Groups: for A being set, Unused, B being set holds A = B proof let L be set, Idle, R be set; thus L = R; end;".into(),
        "theorem Different: for A, B being set holds A = B proof let L, R be set; thus L = R; end;".into(),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/expression_metadata.miz");
    let ids = InMemorySessionIdAllocator::new();
    for (case, text) in sources.iter().enumerate() {
        let mut disk_capture = None;
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk content" } else { text }).unwrap();
            let frontend = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                super::super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot: snapshot_id(777),
                    input: SourceInput {
                        package_id: PackageId::new("expression-metadata-test"),
                        module_path: ModulePath::new("owners"),
                        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                        edition: Edition::new("2026"),
                        origin: if buffer {
                            SourceOriginInput::OpenBuffer {
                                uri: format!("file://{}", path.display()),
                                expected_version: 3,
                                actual_version: 3,
                                text: text.clone().into(),
                            }
                        } else {
                            SourceOriginInput::Disk { path: path.clone() }
                        },
                    },
                },
                &ids,
            )
            .unwrap();
            assert!(
                frontend.diagnostics.is_empty(),
                "{:?}",
                frontend.diagnostics
            );
            assert_eq!(frontend.source.source_text.as_ref(), text);
            assert_eq!(frontend.source.source_hash, hash_text(text));
            assert_eq!(
                matches!(
                    frontend.source.origin,
                    SourceOrigin::OpenBuffer { version: 3 }
                ),
                buffer
            );
            let ast = frontend.ast.as_ref().unwrap();
            let module = ModuleId::new(
                frontend.source.package_id.clone(),
                frontend.source.module_path.clone(),
            );
            let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
            let shells = DeclarationShellCollector::new(ast, &module).collect();
            let collected = SignatureProjectionExtractor::new(
                ast,
                &shells,
                NamespacePath::new(module.path().as_str()),
            )
            .collect(&module);
            assert!(collected.diagnostics().is_empty());
            let symbols = collected.env();
            let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(ast, &module, symbols),
            )
            .unwrap();
            let before = (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                ast.snapshot_text(),
                frontend.source.clone(),
            );
            let baseline = Checker::check_source_theorem_equalities_with_typed_tables(
                &source, symbols, &scope,
            )?;
            let (bindings, inference, typed, expressions) =
                Checker::check_source_theorem_equalities_with_expression_metadata(
                    &source, symbols, &scope,
                )?;
            assert_eq!(
                (&bindings, &inference, &typed),
                (&baseline.0, &baseline.1, &baseline.2)
            );
            assert_eq!(typed.source_id(), ast.source_id);
            let primary = baseline.2.source_term().unwrap();
            let atomic = baseline.2.source_atomic_formula().unwrap();
            let expected = primary
                .terms()
                .iter()
                .map(|(_, term)| ("term", term.source_ordinal(), term.site()))
                .chain(
                    atomic
                        .formulas()
                        .iter()
                        .map(|(_, formula)| ("formula", formula.source_ordinal(), formula.site())),
                )
                .map(|(kind, ordinal, site)| ExpressionMetadataInput {
                    expr: ExprId::new(format!("source.theorem.{kind}.{ordinal}")),
                    typed_site: site.clone(),
                    local_context: None,
                    cluster_facts: Vec::new(),
                })
                .collect::<Vec<_>>();
            assert_eq!(expressions, expected);
            assert_eq!(primary.terms().len(), scope.references().len());
            let ids = expressions
                .iter()
                .map(|row| &row.expr)
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(ids.len(), expressions.len());
            for (ordinal, (id, term)) in primary.terms().iter().enumerate() {
                assert_eq!(term.source_ordinal(), ordinal);
                assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
                let occurrence = scope
                    .references()
                    .iter()
                    .find(|reference| reference.node().index() == term.site().node().index())
                    .unwrap();
                assert_eq!(term.source_range(), occurrence.range());
                let reference = primary
                    .references()
                    .iter()
                    .find(|(_, reference)| reference.term() == id)
                    .unwrap()
                    .1;
                assert_eq!(reference.binding().index(), occurrence.binding().index());
                assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
                let checked = inference
                    .terms()
                    .iter()
                    .find(|(_, checked)| &checked.site == term.site())
                    .unwrap()
                    .1;
                assert_eq!(checked.status, TermStatus::Inferred);
                assert_eq!(
                    checked.reference,
                    Some(TermReference::Binding(reference.binding()))
                );
                let entry = typed.types().get(checked.type_entry).unwrap();
                assert_eq!(entry.owner, *term.site());
                assert_eq!(entry.status, TypeStatus::Known);
                let node = typed.nodes().node(term.site().node()).unwrap();
                assert_eq!(node.links.type_entry, Some(checked.type_entry));
                let TypeEntryActual::Known(actual) = entry.actual else {
                    panic!("missing real normalized type")
                };
                let normalized = inference.normalized_types().get(actual).unwrap();
                assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
                assert_eq!(
                    &text[term.source_range().start..term.source_range().end],
                    term.spelling()
                );
            }
            for (ordinal, (_, formula)) in atomic.formulas().iter().enumerate() {
                assert_eq!(formula.source_ordinal(), ordinal);
                assert_eq!(formula.kind(), SourceAtomicFormulaKind::Equality);
                let checked = inference
                    .formulas()
                    .iter()
                    .find(|(_, checked)| &checked.site == formula.site())
                    .unwrap()
                    .1;
                assert_eq!(checked.status, FormulaStatus::Checked);
                assert_eq!(checked.kind, FormulaKind::Equality);
                let node = typed.nodes().node(formula.site().node()).unwrap();
                assert_eq!(
                    node.anchor,
                    mizar_session::SourceAnchor::Range(formula.source_range())
                );
                assert!(node.links.type_entry.is_none());
                assert_eq!(
                    &text[formula.source_range().start..formula.source_range().end],
                    formula.spelling()
                );
            }
            let roles = typed
                .types()
                .iter()
                .filter(|(_, entry)| matches!(entry.owner, TypedSiteRef::Role { .. }))
                .collect::<Vec<_>>();
            assert_eq!(roles.len(), primary.terms().len());
            for (id, entry) in roles {
                assert_eq!(baseline.2.types().get(id), Some(entry));
                assert!(!expressions.iter().any(|row| row.typed_site == entry.owner));
                assert_eq!(entry.status, TypeStatus::Known);
                let TypeEntryActual::Known(actual) = entry.actual else {
                    panic!("missing written type")
                };
                assert_eq!(
                    inference.normalized_types().get(actual).unwrap().head,
                    TypeHeadRef::BuiltinSet
                );
            }
            assert!(primary.numeric_type_requests().is_empty());
            assert!(atomic.candidates().is_empty());
            assert!(
                typed.source_application().is_none()
                    && typed.source_template().is_none()
                    && typed.source_set_term().is_none()
                    && typed.source_structure().is_none()
            );
            let collection = OverloadCollectionOutput::collect([], []);
            let expansion = TemplateExpansionOutput::expand(&collection);
            let viability = CandidateViabilityOutput::filter(&expansion, []);
            let graphs = SpecificityGraphOutput::build(&viability, []);
            let selection = OverloadSelectionOutput::resolve(&graphs, []);
            let cluster_facts = ClusterFactTable::new();
            let mut assembled = None;
            for reverse in [false, true] {
                let mut rows = expressions.clone();
                if reverse {
                    rows.reverse();
                }
                let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                    typed_ast: &typed,
                    cluster_facts: &cluster_facts,
                    overload_collection: &collection,
                    template_expansion: &expansion,
                    viability: &viability,
                    specificity: &graphs,
                    overload_selection: &selection,
                    expressions: rows,
                    node_hints: Vec::new(),
                    statement_semantics: None,
                    statement_proofs: None,
                })
                .map_err(|error| error.to_string())?;
                assert_eq!(resolved.source_id(), typed.source_id());
                assert_eq!(resolved.source_term(), typed.source_term());
                assert_eq!(
                    resolved.source_atomic_formula(),
                    typed.source_atomic_formula()
                );
                assert_eq!(resolved.expr_metadata().len(), expressions.len());
                for row in &expressions {
                    let metadata = resolved.expr_metadata().get_by_expr(&row.expr).unwrap();
                    assert_eq!(
                        resolved.expr_metadata().get_by_site(&row.typed_site),
                        Some(metadata)
                    );
                    assert_eq!(metadata.expr, row.expr);
                    assert_eq!(metadata.typed_site, row.typed_site);
                    let node = typed.nodes().node(row.typed_site.node()).unwrap();
                    assert_eq!(
                        node.anchor,
                        mizar_session::SourceAnchor::Range(metadata.source_range)
                    );
                    let actual_type = typed
                        .types()
                        .iter()
                        .find(|(_, entry)| entry.owner == row.typed_site)
                        .map(|(_, entry)| {
                            let TypeEntryActual::Known(actual) = entry.actual else {
                                panic!("missing final type")
                            };
                            actual
                        });
                    assert_eq!(metadata.final_type, actual_type);
                    assert_eq!(metadata.local_context, row.local_context);
                    assert_eq!(metadata.cluster_facts, row.cluster_facts);
                    assert!(
                        metadata.visible_facts.is_empty()
                            && metadata.overload.is_none()
                            && metadata.inserted_views.is_empty()
                            && metadata.diagnostics.is_empty()
                    );
                }
                if let Some(previous) = &assembled {
                    assert_eq!(previous, &resolved);
                }
                assembled = Some(resolved);
            }
            let signature = assembled
                .unwrap()
                .expr_metadata()
                .canonical_iter()
                .map(|(_, row)| {
                    (
                        row.expr.clone(),
                        row.typed_site.clone(),
                        (row.source_range.start, row.source_range.end),
                        row.final_type,
                        row.local_context,
                        row.cluster_facts.clone(),
                    )
                })
                .collect::<Vec<_>>();
            if let Some((source_id, previous)) = &disk_capture {
                assert_ne!(*source_id, typed.source_id());
                assert_eq!(previous, &signature);
            } else {
                disk_capture = Some((typed.source_id(), signature));
            }
            let owner = symbols.symbols().iter().next().unwrap();
            let namespace = NamespacePath::new(module.path().as_str());
            let labels = ProofLabelSourceCollector::new(
                ast,
                &module,
                namespace.clone(),
                owner.contribution(),
                &source,
            )
            .and_then(|collector| collector.collect_with_theorem_owners(symbols))
            .unwrap();
            let resolved = LabelResolver::new(labels.projections()).resolve(
                &module,
                &namespace,
                labels.references(),
            );
            let label_before = (labels.clone(), resolved.clone());
            let base = &baseline.2;
            let old_check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source, base, &scope, symbols, &labels, &resolved, None,
            );
            let new_check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source, &typed, &scope, symbols, &labels, &resolved, None,
            );
            if case < 2 {
                let old_check = old_check?;
                let new_check = new_check?;
                assert_eq!(new_check.owners(), old_check.owners());
                assert_eq!(new_check.scope(), old_check.scope());
                assert_eq!(new_check.labels(), old_check.labels());
                assert_eq!(new_check.resolved_labels(), old_check.resolved_labels());
                let expected = lower_source_theorem_skeletons(&old_check, None)?;
                let core = lower_source_theorem_skeletons(&new_check, None)?;
                assert_eq!(core, expected);
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert_eq!(core.obligation_seeds().len(), 2);
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, seed)| seed.status == ObligationSeedStatus::Active
                            && seed.goal.is_some())
                );
                let vcs = generate_core_vcs(&core, snapshot_id(777))?;
                assert_eq!(vcs, generate_core_vcs(&expected, snapshot_id(777))?);
                assert_eq!(vcs.vcs().len(), 2);
                assert!(
                    vcs.vcs()
                        .iter()
                        .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
                );
                assert_ne!(vcs.vcs()[0].local_context, vcs.vcs()[1].local_context);
            } else {
                assert_eq!(new_check.unwrap_err(), old_check.unwrap_err());
                assert!(inference.formulas().iter().any(|(_, formula)| {
                    let references = formula
                        .terms
                        .iter()
                        .map(|site| {
                            inference
                                .terms()
                                .iter()
                                .find(|(_, term)| &term.site == site)
                                .unwrap()
                                .1
                                .reference
                                .clone()
                        })
                        .collect::<Vec<_>>();
                    references[0] != references[1]
                }));
                if case < 5 {
                    assert!(scope.bindings().iter().any(|binding| {
                        !scope
                            .references()
                            .iter()
                            .any(|reference| reference.binding() == binding.id())
                    }));
                }
            }
            let baseline_replay = Checker::check_source_theorem_equalities_with_typed_tables(
                &source, symbols, &scope,
            )?;
            assert_eq!(baseline_replay, baseline);
            let replay = Checker::check_source_theorem_equalities_with_expression_metadata(
                &source, symbols, &scope,
            )?;
            assert_eq!(replay, (bindings, inference, typed, expressions));
            assert_eq!(label_before, (labels, resolved));
            assert_eq!(
                before,
                (
                    source,
                    symbols.clone(),
                    scope,
                    ast.snapshot_text(),
                    frontend.source.clone()
                )
            );
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_expression_metadata_preserves_upstream_errors() {
    use mizar_checker::type_checker::TermFormulaChecker as Checker;
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv, SymbolEnvIndexes},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, validate_source_proof_occurrences},
    };
    use mizar_session::{ModulePath, PackageId};
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let text = "theorem T: for X being set, Unused being set holds X = X proof let X be set; thus X = X; end;";
    let ast = parse(text);
    let module = ModuleId::new(
        PackageId::new("expression-metadata-test"),
        ModulePath::new("owners"),
    );
    let source = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    let symbols = collected.env();
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &module, symbols),
    )
    .unwrap();
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    let foreign_module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("other"));
    let mut controls = vec![
        (
            "foreign environment",
            source.clone(),
            empty.clone(),
            scope.clone(),
        ),
        (
            "foreign source",
            SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap(),
            symbols.clone(),
            scope.clone(),
        ),
    ];
    for control in ["foreign scope", "recovered source"] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        for view in ast.node_views() {
            let children = view
                .children()
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect();
            rebuilt.push(match view.kind() {
                K::Token(token)
                    if control == "foreign scope" && token.text.as_ref() == "Unused" =>
                {
                    builder.add_token(token.kind, "Hidden", view.range())
                }
                K::Token(token) if control == "recovered source" && token.text.as_ref() == "=" => {
                    builder.add_recovered_token(token.kind, token.text.clone(), view.range())
                }
                K::Token(token) => builder.add_token(token.kind, token.text.clone(), view.range()),
                kind => builder.add_node(kind.clone(), view.range(), children),
            });
        }
        let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
        if control == "foreign scope" {
            let foreign = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(&changed, &module, symbols),
            )
            .unwrap();
            assert_eq!(foreign.source_id(), scope.source_id());
            assert_eq!(foreign.references(), scope.references());
            assert_ne!(foreign.bindings(), scope.bindings());
            controls.push((control, source.clone(), symbols.clone(), foreign));
        } else {
            controls.push((
                control,
                SurfaceResolvedArena::lower(&changed, &module).unwrap(),
                symbols.clone(),
                scope.clone(),
            ));
        }
    }
    let unsupported = parse(&text.replacen("thus X = X;", "thus X = 0;", 1));
    let unsupported_source = SurfaceResolvedArena::lower(&unsupported, &module).unwrap();
    let shells = DeclarationShellCollector::new(&unsupported, &module).collect();
    let collected = SignatureProjectionExtractor::new(
        &unsupported,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    assert!(collected.diagnostics().is_empty());
    let unsupported_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&unsupported, &module, collected.env()),
    )
    .unwrap();
    assert_eq!(
        mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(
            &unsupported_source,
            collected.env(),
            &unsupported_scope,
        )
        .unwrap_err(),
        "theorems.equality.unsupported_source_types"
    );
    controls.push((
        "unsupported type",
        unsupported_source.clone(),
        collected.env().clone(),
        unsupported_scope.clone(),
    ));
    controls.push(("combined precedence", unsupported_source, empty, scope));
    for (control, source, symbols, scope) in controls {
        let before = (source.clone(), symbols.clone(), scope.clone());
        let expected =
            Checker::check_source_theorem_equalities_with_typed_tables(&source, &symbols, &scope)
                .unwrap_err();
        if control == "combined precedence" {
            assert_eq!(
                expected,
                validate_source_proof_occurrences(&source, &symbols, &scope).unwrap_err()
            );
        }
        assert_eq!(
            Checker::check_source_theorem_equalities_with_expression_metadata(
                &source, &symbols, &scope
            )
            .unwrap_err(),
            expected,
            "{control}"
        );
        let expected_replay =
            Checker::check_source_theorem_equalities_with_typed_tables(&source, &symbols, &scope)
                .unwrap_err();
        assert_eq!(
            Checker::check_source_theorem_equalities_with_expression_metadata(
                &source, &symbols, &scope
            )
            .unwrap_err(),
            expected_replay,
            "{control}"
        );
        assert_eq!(before, (source, symbols, scope));
    }
}
