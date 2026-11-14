use super::*;

#[test]
fn source_reserved_statement_original_capture_transports_native_rows() -> Result<(), String> {
    use mizar_checker::{
        binding_env::BindingContextOwner, cluster_trace::ClusterFactTable, overload_resolution::*,
        resolved_typed_ast::*, source_statement::*, type_checker::TermFormulaChecker,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        resolved_ast::ModuleId,
        symbols::{
            source_proof_inputs, source_theorem_module_original_inputs,
            validate_source_proof_occurrences,
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOriginInput,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    // Spec 4.3, 14.5.2, 16.2 and 2.2: names and coherent trivia do not select profiles.
    let original =
        "reserve x for set;\ntheorem FormulaStatementReservedVariableEqualitySmoke: x = x;\n";
    let texts = [
        original.to_owned(),
        original.replace(" x", " Parcel").replace("FormulaStatementReservedVariableEqualitySmoke", "RenamedReserve"),
        "reserve\n arbitrary\tfor :: type\n set;\ntheorem Layout: :: proposition\n arbitrary\t=\n arbitrary;".to_owned(),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/reserved.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut cases = Vec::new();
    for text in texts {
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk" } else { &text }).unwrap();
            let frontend = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                super::super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot: snapshot_id(777),
                    input: SourceInput {
                        package_id: PackageId::new("native-statement-test"),
                        module_path: ModulePath::new("reserved"),
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
            let ast = frontend.ast.as_ref().unwrap();
            let module = ModuleId::new(
                frontend.source.package_id.clone(),
                frontend.source.module_path.clone(),
            );
            let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
            let (scope, _, _) = source_proof_inputs(&source, &symbols)?;
            source.validate_against(ast, &module).unwrap();
            assert_eq!(
                validate_source_proof_occurrences(&source, &symbols, &scope),
                Ok(())
            );
            let before = (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                ast.snapshot_text(),
                frontend.source.clone(),
            );
            let (bindings, inference, base) =
                TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)?;
            assert_eq!(
                (
                    base.source_term().unwrap().terms().len(),
                    base.source_atomic_formula().unwrap().formulas().len()
                ),
                (2, 1)
            );
            assert!(base.source_statement().is_none());
            cases.push((
                frontend, source, symbols, scope, before, bindings, inference, base,
            ));
        }
    }
    assert_eq!(cases.len(), 6);
    for (case_index, (frontend, source, symbols, scope, before, bindings, inference, base)) in
        cases.into_iter().enumerate()
    {
        let ast = frontend.ast.as_ref().unwrap();
        let module = source.module();
        // Runtime RED follows independent readiness of all six genuine captures.
        let installed = base
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope);
        assert!(
            installed.is_ok(),
            "native statement installation: {installed:?}"
        );
        let installed = installed.unwrap();
        let handoff = installed.source_statement().unwrap();
        assert_eq!(
            (handoff.source_id(), handoff.module_id()),
            (source.source_id(), module)
        );
        assert_eq!(handoff.binding_env(), &bindings);
        assert_eq!(handoff.binding_fingerprint(), bindings.debug_text());
        assert_eq!(
            handoff.primary_term_fingerprint(),
            base.source_term().unwrap().debug_text()
        );
        assert_eq!(
            handoff.atomic_formula_fingerprint(),
            base.source_atomic_formula().unwrap().debug_text()
        );
        assert!(
            handoff.composite_formula_fingerprint().is_none()
                && handoff.formula_composition_fingerprint().is_none()
        );
        assert_eq!(
            (
                handoff.owners().len(),
                handoff.statements().len(),
                handoff.contexts().len(),
                handoff.input_facts().len(),
                handoff.candidate_facts().len()
            ),
            (1, 1, 1, 1, 1)
        );
        let theorem = ast
            .node_views()
            .find(|view| matches!(view.kind(), K::TheoremItem))
            .unwrap();
        let label = ast.node_views().find(|view| matches!(view.kind(), K::Token(token) if token.kind == mizar_syntax::SurfaceTokenKind::Identifier && theorem.range().start <= view.range().start && view.range().end <= theorem.range().end)).unwrap();
        let (owner_id, owner) = handoff.owners().iter().next().unwrap();
        let (statement_id, statement) = handoff.statements().iter().next().unwrap();
        let (context_id, context) = handoff.contexts().iter().next().unwrap();
        let (_, guard) = handoff.input_facts().iter().next().unwrap();
        let (_, candidate) = handoff.candidate_facts().iter().next().unwrap();
        assert_eq!(owner_id.index(), 0);
        assert_eq!(
            owner.spelling(),
            ast.node(label.id()).unwrap().token_text().unwrap()
        );
        assert_eq!(owner.site().node().index(), theorem.id().index());
        assert_eq!(owner.source_range(), theorem.range());
        assert_eq!(owner.symbol(), handoff.checked_owner().symbol());
        assert_eq!(
            owner.contribution(),
            symbols
                .symbols()
                .get(owner.symbol())
                .unwrap()
                .contribution()
        );
        assert_eq!(handoff.checked_owner().source_range(), theorem.range());
        assert_eq!(
            (owner.role(), owner.status(), owner.recovery()),
            (
                SourceTheoremRole::Theorem,
                SourceTheoremStatus::Unmodified,
                SourceStatementRecovery::Normal
            )
        );
        assert_eq!((statement_id.index(), context_id.index()), (0, 0));
        assert_eq!(
            (statement.owner(), statement.context()),
            (owner_id, context_id)
        );
        assert_eq!(statement.site(), owner.site());
        assert_eq!(statement.source_range(), theorem.range());
        assert_eq!(statement.source_ordinal(), 0);
        assert_eq!(
            (statement.kind(), statement.recovery()),
            (
                SourceStatementKind::TheoremProposition,
                SourceStatementRecovery::Normal
            )
        );
        let mut tokens = ast
            .node_views()
            .filter(|view| {
                matches!(view.kind(), K::Token(_))
                    && theorem.range().start <= view.range().start
                    && view.range().end <= theorem.range().end
            })
            .collect::<Vec<_>>();
        tokens.sort_by_key(|view| view.range().start);
        let spelling = tokens
            .iter()
            .map(|view| ast.node(view.id()).unwrap().token_text().unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(statement.spelling(), spelling);
        let (formula_id, _) = base
            .source_atomic_formula()
            .unwrap()
            .formulas()
            .iter()
            .next()
            .unwrap();
        assert_eq!(
            statement.formula(),
            SourceStatementFormulaTarget::Atomic(formula_id)
        );
        assert_eq!(context.statement(), statement_id);
        assert_eq!(context.source_range(), theorem.range());
        let binding_context = bindings.contexts().get(context.binding_context()).unwrap();
        assert_eq!(binding_context.owner, BindingContextOwner::Module);
        assert_eq!(context.visible_bindings(), binding_context.visible_bindings);
        let (binding_id, _) = bindings.bindings().iter().next().unwrap();
        assert_eq!(context.visible_bindings(), &[binding_id]);
        assert_eq!(
            (
                guard.statement(),
                guard.context(),
                guard.ordinal(),
                guard.kind(),
                guard.binding()
            ),
            (
                statement_id,
                context_id,
                0,
                SourceStatementInputFactKind::ReservedTypeGuard,
                binding_id
            )
        );
        let references = base
            .source_term()
            .unwrap()
            .references()
            .iter()
            .collect::<Vec<_>>();
        assert_eq!(
            guard.uses(),
            references.iter().map(|(id, _)| *id).collect::<Vec<_>>()
        );
        assert_eq!(
            references
                .iter()
                .map(|(_, row)| row.use_ordinal())
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        for ((_, reference), occurrence) in references.iter().zip(scope.references()) {
            assert_eq!(reference.binding(), binding_id);
            assert_eq!(reference.use_ordinal(), occurrence.ordinal());
        }
        assert_eq!(
            (
                candidate.statement(),
                candidate.context(),
                candidate.ordinal(),
                candidate.kind(),
                candidate.formula()
            ),
            (
                statement_id,
                context_id,
                0,
                SourceStatementCandidateFactKind::UnverifiedProposition,
                statement.formula()
            )
        );
        assert_eq!(installed.nodes(), base.nodes());
        assert_eq!(installed.source_term(), base.source_term());
        assert_eq!(
            installed.source_atomic_formula(),
            base.source_atomic_formula()
        );
        assert!(
            installed.types().is_empty()
                && installed.facts().is_empty()
                && installed.initial_obligations().is_empty()
        );
        if case_index == 0 {
            assert_eq!(
                installed,
                base.clone()
                    .with_source_statement_from_original(&source, &symbols, &scope)
                    .unwrap()
            );
        }
        let collection = OverloadCollectionOutput::collect([], []);
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability = CandidateViabilityOutput::filter(&expansion, []);
        let graphs = SpecificityGraphOutput::build(&viability, []);
        let selection = OverloadSelectionOutput::resolve(&graphs, []);
        let cluster_facts = ClusterFactTable::new();
        let mut replay = None;
        let cloned = (case_index == 0).then(|| installed.clone());
        for typed in std::iter::once(&installed).chain(cloned.iter()) {
            let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: typed,
                cluster_facts: &cluster_facts,
                overload_collection: &collection,
                template_expansion: &expansion,
                viability: &viability,
                specificity: &graphs,
                overload_selection: &selection,
                expressions: Vec::new(),
                node_hints: Vec::new(),
                statement_semantics: None,
                statement_proofs: None,
            })
            .map_err(|error| error.to_string())?;
            assert_eq!(resolved.source_statement(), Some(handoff));
            assert_eq!(resolved.source_term(), installed.source_term());
            assert_eq!(
                resolved.source_atomic_formula(),
                installed.source_atomic_formula()
            );
            assert!(resolved.nodes().root().is_some());
            assert_eq!(resolved.nodes().len(), installed.nodes().len());
            assert!(
                resolved.expr_metadata().is_empty()
                    && resolved.checked_formulas().is_empty()
                    && resolved.statement_semantics().is_empty()
                    && resolved.checked_proofs().is_empty()
            );
            if let Some(previous) = &replay {
                assert_eq!(&resolved, previous);
            }
            replay = Some(resolved);
        }
        assert!(inference.facts().is_empty());
        assert_eq!(
            (source, symbols, scope, ast.snapshot_text(), frontend.source),
            before
        );
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_reserved_statement_original_capture_preserves_installation_boundary() -> Result<(), String>
{
    use mizar_checker::{
        cluster_trace::ClusterFactTable, overload_resolution::*, resolved_typed_ast::*,
        type_checker::TermFormulaChecker, typed_ast::*,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        env::{SymbolEnv, SymbolEnvIndexes},
        resolved_ast::ModuleId,
        symbols::{
            source_proof_inputs, source_theorem_module_original_inputs,
            validate_source_proof_occurrences,
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceAnchor,
        SourceInput, SourceOriginInput,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let text =
        "reserve x for set;\ntheorem FormulaStatementReservedVariableEqualitySmoke: x = x;\n";
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/reserved.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut captures = Vec::new();
    // Each refusal receives a coherent capture, never changed text beside an old AST.
    for current in [
        text.to_owned(),
        text.replace("Smoke", "Foreign"),
        text.replace("for set", "for object"),
        text.replace("x = x", "x ="),
        format!("{text}theorem Extra: x = x;"),
        "theorem Quantified: for x being set holds x = x proof let y be set; thus y = y; end;"
            .to_owned(),
    ] {
        fs::write(&path, &current).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(777),
                input: SourceInput {
                    package_id: PackageId::new("native-statement-boundary"),
                    module_path: ModulePath::new("reserved"),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path: path.clone() },
                },
            },
            &ids,
        )
        .unwrap();
        captures.push(frontend);
    }
    let ast = captures[0].ast.as_ref().unwrap();
    assert!(captures[0].diagnostics.is_empty());
    let module = ModuleId::new(
        captures[0].source.package_id.clone(),
        captures[0].source.module_path.clone(),
    );
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let (scope, _, _) = source_proof_inputs(&source, &symbols)?;
    assert_eq!(
        validate_source_proof_occurrences(&source, &symbols, &scope),
        Ok(())
    );
    let (_, inference, base) =
        TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)?;
    let quantified_ast = captures[5].ast.as_ref().unwrap();
    assert!(captures[5].diagnostics.is_empty());
    let (quantified_source, quantified_symbols) =
        source_theorem_module_original_inputs(quantified_ast, &module)?.unwrap();
    let (quantified_scope, _, _) = source_proof_inputs(&quantified_source, &quantified_symbols)?;
    assert_eq!(
        validate_source_proof_occurrences(
            &quantified_source,
            &quantified_symbols,
            &quantified_scope
        ),
        Ok(())
    );
    // This nonreserved profile has a genuine successful native base, including its proof body.
    let (_, _, quantified_base) = TermFormulaChecker::check_source_theorem_equalities(
        &quantified_source,
        &quantified_symbols,
        &quantified_scope,
    )?;
    let quantified_before = (
        quantified_source.clone(),
        quantified_symbols.clone(),
        quantified_scope.clone(),
        quantified_base.clone(),
    );
    let before = (source.clone(), symbols.clone(), scope.clone(), base.clone());
    let installed = base
        .clone()
        .with_source_statement_from_original(&source, &symbols, &scope);
    assert!(
        installed.is_ok(),
        "native statement installation: {installed:?}"
    );
    let installed = installed.unwrap();
    let detached = installed.source_statement().unwrap().clone();
    assert_eq!(
        quantified_base.clone().with_source_statement_from_original(
            &quantified_source,
            &quantified_symbols,
            &quantified_scope
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Ok(installed.clone())
    );
    assert_eq!(
        installed
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Ok(installed.clone())
    );
    let theorem = ast
        .node_views()
        .find(|view| matches!(view.kind(), K::TheoremItem))
        .unwrap();
    let label = ast.node_views().find(|view| matches!(view.kind(), K::Token(token) if token.text.as_ref() == "FormulaStatementReservedVariableEqualitySmoke")).unwrap();
    for control in ["canonical", "token", "child", "range", "rootless"] {
        let mut nodes = base
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        match control {
            "token" => {
                nodes[label.id().index()].kind = TypedNodeKind::new("changed-token-envelope")
            }
            "child" => nodes[theorem.id().index()].children.reverse(),
            "range" => nodes[label.id().index()].anchor = SourceAnchor::Range(theorem.range()),
            _ => {}
        }
        let reconstructed = TypedAst::try_new(TypedAstParts {
            source_id: base.source_id(),
            module_id: base.module_id().clone(),
            resolved_root: if control == "rootless" {
                None
            } else {
                base.resolved_root()
            },
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: TypedArena::try_new(
                if control == "rootless" {
                    None
                } else {
                    base.nodes().root()
                },
                nodes,
            )
            .unwrap(),
            contexts: base.contexts().clone(),
            types: base.types().clone(),
            facts: base.facts().clone(),
            coercions: base.coercions().clone(),
            initial_obligations: base.initial_obligations().clone(),
            diagnostics: base.diagnostics().clone(),
        })
        .unwrap()
        .with_source_term(base.source_term().unwrap().clone())
        .unwrap()
        .with_source_atomic_formula(base.source_atomic_formula().unwrap().clone())
        .unwrap();
        assert_eq!(reconstructed.source_term(), base.source_term());
        assert_eq!(
            reconstructed.source_atomic_formula(),
            base.source_atomic_formula()
        );
        assert!(reconstructed.source_statement().is_none());
        let result = reconstructed
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope);
        if control == "canonical" {
            assert_eq!(result, Ok(installed.clone()));
        } else {
            assert_eq!(
                result,
                Err(TypedAstError::InvalidSourceStatement),
                "{control}"
            );
        }
        if matches!(control, "canonical" | "rootless") {
            assert_eq!(
                reconstructed
                    .clone()
                    .with_source_statement(detached.clone()),
                Err(TypedAstError::InvalidSourceStatement),
                "detached {control}"
            );
        }
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
    }
    let enriched = TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
        &source, &symbols, &scope,
    )?
    .2;
    assert!(!enriched.types().is_empty());
    assert_eq!(
        enriched.with_source_statement_from_original(&source, &symbols, &scope),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Ok(installed.clone())
    );
    let (foreign_source, foreign_symbols) =
        source_theorem_module_original_inputs(captures[1].ast.as_ref().unwrap(), &module)?.unwrap();
    let (foreign_scope, _, _) = source_proof_inputs(&foreign_source, &foreign_symbols)?;
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    for (current, env, occurrences) in [
        (&foreign_source, &symbols, &scope),
        (&source, &foreign_symbols, &scope),
        (&source, &symbols, &foreign_scope),
        (&source, &empty, &scope),
    ] {
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(current, env, occurrences),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
    }
    for (control, capture) in ["unsupported type", "malformed equality", "extra theorem"]
        .into_iter()
        .zip(&captures[2..5])
    {
        let current = capture.ast.as_ref().unwrap();
        let (current_source, current_symbols) =
            source_theorem_module_original_inputs(current, &module)?
                .expect("bounded native control remains publicly constructible");
        match source_proof_inputs(&current_source, &current_symbols) {
            Ok((current_scope, _, _)) => {
                assert_eq!(
                    validate_source_proof_occurrences(
                        &current_source,
                        &current_symbols,
                        &current_scope
                    ),
                    Ok(())
                );
                assert_eq!(
                    base.clone().with_source_statement_from_original(
                        &current_source,
                        &current_symbols,
                        &current_scope
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    "{control}"
                );
            }
            Err(error) => {
                assert_eq!(control, "malformed equality", "{error}");
                assert!(!capture.diagnostics.is_empty());
                // Malformed recollection has no native scope: this is explicitly a foreign-scope boundary.
                assert!(
                    validate_source_proof_occurrences(&current_source, &current_symbols, &scope)
                        .is_err()
                );
                assert!(
                    TermFormulaChecker::check_source_theorem_equalities(
                        &current_source,
                        &current_symbols,
                        &scope
                    )
                    .is_err()
                );
                assert_eq!(
                    base.clone().with_source_statement_from_original(
                        &current_source,
                        &current_symbols,
                        &scope
                    ),
                    Err(TypedAstError::InvalidSourceStatement)
                );
            }
        }
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
    }
    let verify_final_assembly: Box<dyn FnOnce() -> Result<(), String> + '_> = Box::new(|| {
        let collection = OverloadCollectionOutput::collect([], []);
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability = CandidateViabilityOutput::filter(&expansion, []);
        let graphs = SpecificityGraphOutput::build(&viability, []);
        let selection = OverloadSelectionOutput::resolve(&graphs, []);
        let cluster_facts = ClusterFactTable::new();
        let mut final_baseline = None;
        for control in [
            "canonical",
            "metadata",
            "canonical",
            "hint",
            "canonical",
            "semantics",
            "canonical",
            "proofs",
            "canonical",
        ] {
            let replayed = base
                .clone()
                .with_source_statement_from_original(&source, &symbols, &scope)
                .unwrap();
            assert_eq!(replayed, installed);
            let handoff = replayed.source_statement().unwrap();
            let (formula_id, formula) = inference.formulas().iter().next().unwrap();
            let owner = handoff.checked_owner();
            let expressions = if control == "metadata" {
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("extra"),
                    typed_site: formula.site.clone(),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }]
            } else {
                Vec::new()
            };
            let node_hints = if control == "hint" {
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(theorem.id().index()),
                    kind: ResolvedNodeKindHintKind::ResolvedUse {
                        symbol: handoff.owners().iter().next().unwrap().1.symbol().clone(),
                    },
                }]
            } else {
                Vec::new()
            };
            let result = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: &replayed,
                cluster_facts: &cluster_facts,
                overload_collection: &collection,
                template_expansion: &expansion,
                viability: &viability,
                specificity: &graphs,
                overload_selection: &selection,
                expressions,
                node_hints,
                statement_semantics: (control == "semantics").then_some(StatementSemanticInputs {
                    owner: handoff.checked_owner(),
                    binding_env: handoff.binding_env(),
                    term_formula: &inference,
                    rows: vec![StatementSemanticInput {
                        owner: owner.symbol().clone(),
                        owner_node: TypedNodeId::new(theorem.id().index()),
                        formula: formula_id,
                        formula_node: formula.site.node(),
                    }],
                }),
                statement_proofs: (control == "proofs").then_some(StatementProofInputs {
                    owner,
                    rows: vec![StatementProofIntentInput {
                        id: StatementProofIntentId::new(0),
                        source_order: 0,
                        statement: StatementSemanticId::new(0),
                        source_id: source.source_id(),
                        module_id: module.clone(),
                        owner: owner.symbol().clone(),
                        owner_node: TypedNodeId::new(theorem.id().index()),
                        owner_range: owner.source_range(),
                        owner_origin: owner.origin().clone(),
                        owner_visibility: owner.visibility(),
                        owner_export_status: owner.export_status(),
                        formula: formula_id,
                        formula_site: formula.site.clone(),
                        formula_node: formula.site.node(),
                        formula_range: formula.source_range,
                        recovery: NodeRecoveryState::Normal,
                        policy: TheoremPolicyIntent::Unmodified,
                        justification: TheoremJustificationIntent::Omitted,
                    }],
                }),
            });
            if control == "canonical" {
                let resolved = result.map_err(|error| error.to_string())?;
                assert_eq!(resolved.source_statement(), replayed.source_statement());
                assert!(
                    resolved.checked_formulas().is_empty()
                        && resolved.statement_semantics().is_empty()
                        && resolved.checked_proofs().is_empty()
                );
                if let Some(baseline) = &final_baseline {
                    assert_eq!(&resolved, baseline);
                }
                final_baseline = Some(resolved);
            } else {
                assert!(result.is_err(), "final accepted {control}");
            }
        }
        Ok(())
    });
    verify_final_assembly()?;
    assert_eq!(
        (
            quantified_source,
            quantified_symbols,
            quantified_scope,
            quantified_base
        ),
        quantified_before
    );
    assert_eq!((source, symbols, scope, base), before);
    fs::remove_dir_all(root).unwrap();
    Ok(())
}
