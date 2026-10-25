use super::*;
use crate::env::{ContributionKind, SourceContributionIndex, SymbolEntry, SymbolEnvIndexes};
use crate::resolved_ast::{
    FullyQualifiedName, LocalSymbolId, ResolvedArenaBuilder, ResolvedNode, SemanticOrigin, SymbolId,
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    SourceAnchor, SourceId,
};
use mizar_syntax::{
    SurfaceAst, SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceFormulaConstant, SurfaceNodeKind,
    SurfaceTokenKind, SyntaxRecoveryKind,
};

struct NoImportedSummaries;

impl mizar_frontend::lexical_env::LexicalSummaryProvider for NoImportedSummaries {
    fn resolve_imports(
        &self,
        _request: &mizar_frontend::lexical_env::LexicalEnvironmentRequest<'_>,
    ) -> Result<
        mizar_frontend::lexical_env::ResolvedImports,
        mizar_frontend::lexical_env::FrontendLexicalEnvironmentError,
    > {
        Ok(mizar_frontend::lexical_env::ResolvedImports {
            imports: Vec::new(),
            summaries: Vec::new(),
            diagnostics: Vec::new(),
        })
    }
}

fn parsed_private_theorem_fixture(
    text: &str,
) -> (
    SurfaceAst,
    ModuleId,
    NamespacePath,
    SymbolEnv,
    SurfaceResolvedArena,
) {
    use mizar_frontend::{
        orchestration::Frontend, parsing::MizarParserSeam, source::FrontendSourceLoader,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, SourceInput, SourceOriginInput, normalize_path,
    };

    let root = std::env::temp_dir().join(format!(
        "mizar-resolve-private-theorem-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = root.join("src/main.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    let output = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        NoImportedSummaries,
        MizarParserSeam,
    )
    .run(
        mizar_frontend::source::SourceUnitRequest {
            snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                "mizar-session-build-snapshot-v1:{}",
                "31".repeat(Hash::BYTE_LEN)
            ))
            .unwrap(),
            input: SourceInput {
                package_id: PackageId::new("pkg"),
                module_path: ModulePath::new("main"),
                normalized_path: normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk { path },
            },
        },
        &InMemorySessionIdAllocator::new(),
    )
    .unwrap();
    std::fs::remove_dir_all(root).unwrap();
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.expect("private theorem source has a parser AST");
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let shells = crate::declarations::DeclarationShellCollector::new(&ast, &module).collect();
    let symbols =
        crate::symbols::SignatureProjectionExtractor::new(&ast, &shells, namespace.clone())
            .collect(&module);
    assert!(symbols.diagnostics().is_empty());
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    (ast, module, namespace, symbols.into_env(), resolved)
}

#[test]
fn private_theorem_owner_opt_in_collects_real_source_label_and_citation() {
    let (ast, module, namespace, env, resolved) = parsed_private_theorem_fixture(include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    ));
    let private = env
        .symbols()
        .iter()
        .find(|entry| entry.primary_spelling() == "PrivT1")
        .unwrap();
    assert_eq!(private.visibility(), Visibility::Private);
    assert_eq!(private.export_status(), ExportStatus::LocalOnly);
    let collector = ProofLabelSourceCollector::new(
        &ast,
        &module,
        namespace.clone(),
        private.contribution(),
        &resolved,
    )
    .unwrap();
    let collection = collector.collect_with_theorem_owners(&env).unwrap();
    let projection = collection
        .projections()
        .iter()
        .find(|projection| projection.primary_spelling() == "PrivT1")
        .unwrap();
    assert_eq!(projection.visibility(), Visibility::Private);
    assert_eq!(projection.export_status(), ExportStatus::LocalOnly);
    assert_eq!(
        projection.origin_path().as_str(),
        private.symbol().fqn().as_str()
    );
    assert_eq!(projection.origin().source_id(), ast.source_id);
    assert_eq!(collection.references().len(), 1);
    assert_eq!(collection.references()[0].site().spelling(), "PrivT1");
    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert_resolved_label(&result, 0, private.symbol().fqn().as_str());
}

#[test]
fn theorem_owner_requires_marker_visibility_and_export_correspondence() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for public in [false, true] {
        let text = if public {
            fixture.replacen("private theorem", "public theorem", 1)
        } else {
            fixture.to_owned()
        };
        let (ast, module, namespace, env, resolved) = parsed_private_theorem_fixture(&text);
        let private = env
            .symbols()
            .iter()
            .find(|entry| entry.primary_spelling() == "PrivT1")
            .unwrap();
        for (visibility, export) in [
            (Visibility::Private, ExportStatus::LocalOnly),
            (Visibility::Public, ExportStatus::LocalOnly),
            (Visibility::Private, ExportStatus::Exported),
            (Visibility::Public, ExportStatus::Exported),
        ] {
            let mut indexes = SymbolEnvIndexes::default();
            indexes.symbols.insert(
                private
                    .clone()
                    .with_visibility(visibility)
                    .with_export_status(export),
            );
            let checked = SymbolEnv::new(module.clone(), indexes);
            let collection = ProofLabelSourceCollector::new(
                &ast,
                &module,
                namespace.clone(),
                private.contribution(),
                &resolved,
            )
            .unwrap()
            .collect_with_theorem_owners(&checked)
            .unwrap();
            assert_eq!(
                collection
                    .projections()
                    .iter()
                    .any(|projection| projection.primary_spelling() == "PrivT1"),
                if public {
                    visibility == Visibility::Public && export == ExportStatus::Exported
                } else {
                    visibility == Visibility::Private && export == ExportStatus::LocalOnly
                },
                "{visibility:?}/{export:?}"
            );
        }
    }
}

#[test]
fn theorem_visibility_wrappers_remain_opt_in_with_exact_owner_and_local_labels() {
    let source = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    ).replacen("thus X = X;", "Inner: X = X;\n  thus X = X by Inner;", 1);
    for public in [false, true] {
        let text = if public {
            source.replacen("private theorem", "public theorem", 1)
        } else {
            source.to_owned()
        };
        let (ast, module, namespace, env, resolved) = parsed_private_theorem_fixture(&text);
        let owner = env
            .symbols()
            .iter()
            .find(|entry| entry.primary_spelling() == "PrivT1")
            .unwrap();
        assert_eq!(
            owner.visibility(),
            if public {
                Visibility::Public
            } else {
                Visibility::Private
            }
        );
        assert_eq!(
            owner.export_status(),
            if public {
                ExportStatus::Exported
            } else {
                ExportStatus::LocalOnly
            }
        );
        assert_eq!(
            source_range_from_anchor(owner.origin().anchor()),
            Some(range(
                ast.source_id,
                if public { 7 } else { 8 },
                if public { 116 } else { 117 }
            ))
        );
        let collector = ProofLabelSourceCollector::new(
            &ast,
            &module,
            namespace,
            owner.contribution(),
            &resolved,
        )
        .unwrap();
        for collection in [
            collector.collect().unwrap(),
            collector.collect_with_let_conditions().unwrap(),
            collector.collect_with_proof_organization().unwrap(),
        ] {
            assert!(collection.projections().is_empty());
            assert_eq!(
                collection
                    .references()
                    .iter()
                    .map(|reference| reference.site().spelling())
                    .collect::<Vec<_>>(),
                ["PrivT1"],
            );
        }
        let collection = collector.collect_with_theorem_owners(&env).unwrap();
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(|projection| (
                    projection.primary_spelling(),
                    projection.visibility(),
                    projection.export_status()
                ))
                .collect::<Vec<_>>(),
            [
                (
                    "PrivT1",
                    if public {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    },
                    if public {
                        ExportStatus::Exported
                    } else {
                        ExportStatus::LocalOnly
                    }
                ),
                ("Inner", Visibility::Private, ExportStatus::LocalOnly),
                ("UsePriv1", Visibility::Public, ExportStatus::Exported),
            ]
        );
        let projection = &collection.projections()[0];
        assert_eq!(
            projection.origin_path().as_str(),
            owner.symbol().fqn().as_str()
        );
        assert_eq!(projection.origin(), owner.origin());
        let start = if public { 15 } else { 16 };
        assert_eq!(
            projection.declaration_range(),
            range(ast.source_id, start, start + 6)
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| reference.site().spelling())
                .collect::<Vec<_>>(),
            ["Inner", "PrivT1"]
        );
        let result = LabelResolver::new(collection.projections()).resolve(
            &module,
            &NamespacePath::new("main"),
            collection.references(),
        );
        assert_resolved_label(
            &result,
            0,
            collection.projections()[1].origin_path().as_str(),
        );
        assert_resolved_label(&result, 1, owner.symbol().fqn().as_str());
    }
}

#[test]
fn private_theorem_owner_rejects_malformed_and_recovered_wrappers() {
    let source_id = source_id();
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    for shape in 0..9 {
        let mut builder = CollectorAstBuilder::new(source_id);
        let marker_token = if shape == 4 {
            builder.token(SurfaceTokenKind::ReservedWord, "public")
        } else if shape == 5 {
            builder.recovered_token(SurfaceTokenKind::ReservedWord, "private")
        } else {
            builder.token(SurfaceTokenKind::ReservedWord, "private")
        };
        let marker = if shape == 3 {
            builder.node(SurfaceNodeKind::Annotation, vec![marker_token])
        } else if shape == 7 {
            builder.builder.add_recovery(
                SyntaxRecoveryKind::MissingItem,
                range(source_id, 0, builder.cursor),
                vec![marker_token],
            )
        } else {
            builder.node(SurfaceNodeKind::VisibilityMarker, vec![marker_token])
        };
        let theorem = if shape == 8 {
            let role = builder.recovered_token(SurfaceTokenKind::ReservedWord, "theorem");
            let owner = builder.token(SurfaceTokenKind::Identifier, "PrivateOwner");
            let colon = builder.token(SurfaceTokenKind::ReservedSymbol, ":");
            let formula = builder.formula();
            let proof = builder.proof(vec![], ("proof", false), ("end", false));
            let semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
            builder.node(
                SurfaceNodeKind::TheoremItem,
                vec![role, owner, colon, formula, proof, semicolon],
            )
        } else {
            builder.valid_theorem("PrivateOwner", vec![])
        };
        let wrapper_children = match shape {
            1 => vec![theorem],
            2 => {
                let extra = builder.node(SurfaceNodeKind::Annotation, vec![]);
                vec![marker, theorem, extra]
            }
            _ => vec![marker, theorem],
        };
        let wrapper = if shape == 6 {
            builder.builder.add_recovery(
                SyntaxRecoveryKind::MissingItem,
                range(source_id, 0, builder.cursor),
                wrapper_children,
            )
        } else {
            builder.node(SurfaceNodeKind::VisibleItem, wrapper_children)
        };
        let ast = builder.finish_items(vec![wrapper]);
        let theorem = ast
            .node_views()
            .find(|node| matches!(node.kind(), SurfaceNodeKind::TheoremItem))
            .unwrap();
        let mut indexes = SymbolEnvIndexes::default();
        indexes.symbols.insert(
            SymbolEntry::new(
                SymbolId::new(
                    module.clone(),
                    LocalSymbolId::new("private-owner"),
                    FullyQualifiedName::new("pkg::main::private-owner"),
                ),
                SymbolKind::Theorem,
                namespace.clone(),
                "PrivateOwner",
                origin(
                    source_id,
                    module.clone(),
                    theorem.range(),
                    theorem.id().index(),
                ),
                contribution,
            )
            .with_visibility(Visibility::Private)
            .with_export_status(ExportStatus::LocalOnly),
        );
        let env = SymbolEnv::new(module.clone(), indexes);
        let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
        let collection = ProofLabelSourceCollector::new(
            &ast,
            &module,
            namespace.clone(),
            contribution,
            &resolved,
        )
        .unwrap()
        .collect_with_theorem_owners(&env)
        .unwrap();
        assert_eq!(
            collection
                .projections()
                .iter()
                .any(|projection| projection.primary_spelling() == "PrivateOwner"),
            shape == 0,
            "wrapper shape {shape}"
        );
    }
}

#[test]
fn unqualified_citation_respects_proof_block_visibility_and_confinement() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let local_contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let imported_contribution = contribution(&mut contributions, dep.clone(), source_id, 100);
    let current_fixture = ProjectionFixture::new(
        source_id,
        current.clone(),
        namespace.clone(),
        local_contribution,
    );
    let dep_fixture = ProjectionFixture::new(
        source_id,
        dep.clone(),
        namespace.clone(),
        imported_contribution,
    );
    let outer_scope = LabelScopePath::new(vec![0]);
    let inner_scope = LabelScopePath::new(vec![0, 1]);

    let projections = vec![
        proof_step_projection(&current_fixture, "A", 10, 1, outer_scope.clone()),
        proof_step_projection(&current_fixture, "B", 20, 2, inner_scope.clone()),
        current_theorem_projection(&current_fixture, "T", 30, 3),
        imported_theorem_projection(&dep_fixture, "Lib", 40),
    ];
    let references = vec![
        unqualified_ref(
            source_id,
            current.clone(),
            2,
            80,
            "T",
            Some(inner_scope.clone()),
        ),
        unqualified_ref(
            source_id,
            current.clone(),
            4,
            60,
            "A",
            Some(inner_scope.clone()),
        ),
        unqualified_ref(source_id, current.clone(), 5, 70, "B", Some(outer_scope)),
        unqualified_ref(source_id, current.clone(), 6, 90, "Lib", Some(inner_scope)),
    ];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "T");
    assert_resolved_label(&resolved, 1, "app::main::proof::A");
    assert_unresolved_label(&resolved, 2, LabelExpectation::ProofOrTheorem, "B");
    assert_resolved_label(&resolved, 3, "dep::logic::theorem::Lib");
    assert!(resolved.has_unresolved());
}

#[test]
fn theorem_owner_opt_in_authenticates_theorem_and_lemma_labels() {
    let (source_id, foreign_source) = distinct_source_ids();
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut builder = CollectorAstBuilder::new(source_id);
    let lemma_statement = builder.valid_statement("L", None, false);
    let lemma_proof = builder.proof(vec![lemma_statement], ("proof", false), ("end", false));
    let lemma_children = vec![
        builder.token(SurfaceTokenKind::ReservedWord, "lemma"),
        builder.token(SurfaceTokenKind::Identifier, "LemmaOwner"),
        builder.token(SurfaceTokenKind::ReservedSymbol, ":"),
        builder.formula(),
        lemma_proof,
        builder.token(SurfaceTokenKind::ReservedSymbol, ";"),
    ];
    let lemma = builder.node(SurfaceNodeKind::LemmaItem, lemma_children);
    let theorem_statement = builder.valid_statement("T", Some("LemmaOwner"), false);
    let theorem = builder.valid_theorem("TheoremOwner", vec![theorem_statement]);
    let ast = builder.finish_items(vec![lemma, theorem]);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let item_list = exact_proof_label_item_list(&ast).unwrap();
    let items = item_list.child_views().collect::<Vec<_>>();
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let mut indexes = SymbolEnvIndexes::default();
    for (item, spelling, kind, local) in [
        (&items[0], "LemmaOwner", SymbolKind::Lemma, "lemma-owner"),
        (
            &items[1],
            "TheoremOwner",
            SymbolKind::Theorem,
            "theorem-owner",
        ),
    ] {
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::main::{local}")),
        );
        let origin = origin(source_id, module.clone(), item.range(), item.id().index());
        indexes.symbols.insert(SymbolEntry::new(
            symbol,
            kind,
            namespace.clone(),
            spelling,
            origin,
            contribution,
        ));
    }
    let symbols = SymbolEnv::new(module.clone(), indexes);
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect_with_theorem_owners(&symbols)
            .unwrap();
    assert_eq!(
        collection
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["LemmaOwner", "L", "TheoremOwner", "T"]
    );
    assert_eq!(collection.references()[0].ordinal(), 2);
    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert_resolved_label(&result, 0, "pkg::main::lemma-owner");
    let collect_owner = |ast: &SurfaceAst, spelling, kind, origin_source| {
        let item = exact_proof_label_item_list(ast)
            .unwrap()
            .child_views()
            .next()
            .unwrap();
        let mut indexes = SymbolEnvIndexes::default();
        indexes.symbols.insert(SymbolEntry::new(
            SymbolId::new(
                module.clone(),
                LocalSymbolId::new("owner"),
                FullyQualifiedName::new("owner"),
            ),
            kind,
            namespace.clone(),
            spelling,
            origin(
                origin_source,
                module.clone(),
                range(origin_source, item.range().start, item.range().end),
                item.id().index(),
            ),
            contribution,
        ));
        let symbols = SymbolEnv::new(module.clone(), indexes);
        let resolved = SurfaceResolvedArena::lower(ast, &module).unwrap();
        ProofLabelSourceCollector::new(ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect_with_theorem_owners(&symbols)
            .unwrap()
    };
    let malformed = malformed_theorem_owner_ast(source_id, TheoremShapeMutation::WrongRole);
    let mut wrong_kind_builder = CollectorAstBuilder::new(source_id);
    let wrong_kind_item = wrong_kind_builder.valid_theorem("Owner", Vec::new());
    let wrong_kind = wrong_kind_builder.finish_items(vec![wrong_kind_item]);
    let mut wrong_source_builder = CollectorAstBuilder::new(source_id);
    let wrong_source_item = wrong_source_builder.valid_theorem("Owner", Vec::new());
    let wrong_source = wrong_source_builder.finish_items(vec![wrong_source_item]);
    for (index, collection) in [
        collect_owner(&malformed, "BadOwner", SymbolKind::Theorem, source_id),
        collect_owner(&wrong_kind, "Owner", SymbolKind::Lemma, source_id),
        collect_owner(&wrong_source, "Owner", SymbolKind::Theorem, foreign_source),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(collection.projections().is_empty(), "case {index}");
        assert!(collection.references().is_empty(), "case {index}");
    }
}

#[test]
fn duplicate_and_visible_nested_labels_are_internal_diagnostics() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let outer_scope = LabelScopePath::new(vec![0]);
    let inner_scope = LabelScopePath::new(vec![0, 1]);
    let sibling_scope = LabelScopePath::new(vec![0, 2]);
    let outer = proof_step_projection(&fixture, "A", 10, 1, outer_scope.clone());
    let duplicate = proof_step_projection(&fixture, "A", 20, 2, outer_scope);
    let inner_conflict = proof_step_projection(&fixture, "A", 30, 3, inner_scope);
    let sibling_conflict = proof_step_projection(&fixture, "A", 40, 4, sibling_scope);
    let projections = vec![sibling_conflict, inner_conflict, duplicate, outer];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &[]);

    let diagnostics = resolved.diagnostics();
    assert_eq!(diagnostics.len(), 3);
    assert_eq!(
        diagnostics
            .iter()
            .map(LabelDiagnostic::kind)
            .collect::<Vec<_>>(),
        vec![
            LabelDiagnosticKind::DuplicateLabel,
            LabelDiagnosticKind::ConflictingVisibleLabel,
            LabelDiagnosticKind::ConflictingVisibleLabel,
        ]
    );
    assert_eq!(
        diagnostics
            .iter()
            .map(LabelDiagnostic::primary_range)
            .collect::<Vec<_>>(),
        vec![
            range(source_id, 20, 21),
            range(source_id, 30, 31),
            range(source_id, 40, 41),
        ]
    );
    let related_ranges = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.related_ranges().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        related_ranges,
        vec![
            vec![range(source_id, 10, 11)],
            vec![range(source_id, 10, 11), range(source_id, 20, 21)],
            vec![range(source_id, 10, 11), range(source_id, 20, 21)],
        ]
    );
}

#[test]
fn proof_organization_branch_labels_are_visible_only_in_their_branch() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);

    let first_label = builder.proposition(Some("A"), Some(":"), true);
    let first_reference = builder.reference("A", false, false);
    let first_justification = builder.justification(
        ("by", SurfaceTokenKind::ReservedWord),
        vec![first_reference],
        false,
    );
    let first_body_proposition = builder.proposition(None, None, true);
    let first_body = builder.compact(first_body_proposition, Some(first_justification));
    let first_suppose = builder.token(SurfaceTokenKind::ReservedWord, "suppose");
    let first_semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
    let first_end = builder.token(SurfaceTokenKind::ReservedWord, "end");
    let first_trailing_semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
    let first_branch = builder.node(
        SurfaceNodeKind::SupposeItem,
        vec![
            first_suppose,
            first_label,
            first_semicolon,
            first_body,
            first_end,
            first_trailing_semicolon,
        ],
    );

    let second_reference = builder.reference("A", false, false);
    let second_justification = builder.justification(
        ("by", SurfaceTokenKind::ReservedWord),
        vec![second_reference],
        false,
    );
    let second_body_proposition = builder.proposition(None, None, true);
    let second_body = builder.compact(second_body_proposition, Some(second_justification));
    let second_suppose = builder.token(SurfaceTokenKind::ReservedWord, "suppose");
    let second_proposition = builder.proposition(None, None, true);
    let second_semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
    let second_end = builder.token(SurfaceTokenKind::ReservedWord, "end");
    let second_trailing_semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
    let second_branch = builder.node(
        SurfaceNodeKind::SupposeItem,
        vec![
            second_suppose,
            second_proposition,
            second_semicolon,
            second_body,
            second_end,
            second_trailing_semicolon,
        ],
    );
    let per = builder.token(SurfaceTokenKind::ReservedWord, "per");
    let cases = builder.token(SurfaceTokenKind::ReservedWord, "cases");
    let reasoning_semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
    let reasoning = builder.node(
        SurfaceNodeKind::CaseReasoningStatement,
        vec![per, cases, reasoning_semicolon, first_branch, second_branch],
    );
    let theorem = builder.valid_theorem("Owner", vec![reasoning]);
    let ast = builder.finish_items(vec![theorem]);
    let collection = collect_proof_fixture(&ast);
    assert_eq!(
        collection
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["A"]
    );

    let module = module_id("pkg", "main");
    let projection = &collection.projections()[0];
    assert_eq!(projection.module(), &module);
    assert_eq!(projection.declaration_range().source_id, source_id);
    assert!(
        matches!(projection.source(), LabelProjectionSource::CurrentModule {
        proof_scope: Some(scope), ..
    } if matches!(collection.references()[0].scope(), LabelReferenceScope::Unqualified {
        proof_scope: Some(reference_scope)
    } if reference_scope == scope))
    );
    assert!(collection.references()[0].ordinal() < collection.references()[1].ordinal());
    let resolved = LabelResolver::new(collection.projections()).resolve(
        &module,
        &NamespacePath::new("main"),
        collection.references(),
    );
    for (id, reference) in resolved.ids().iter().zip(collection.references()) {
        let entry = resolved.table().get(*id).unwrap();
        assert_eq!(entry.site(), reference.site());
        assert_eq!(entry.origin(), reference.origin());
        assert_eq!(reference.site().range().source_id, source_id);
    }
    assert!(matches!(
        resolved
            .table()
            .get(resolved.ids()[0])
            .unwrap()
            .resolution(),
        LabelResolution::Resolved(_)
    ));
    assert!(matches!(
        resolved
            .table()
            .get(resolved.ids()[1])
            .unwrap()
            .resolution(),
        LabelResolution::Unresolved(_)
    ));
}

#[test]
fn forward_references_to_later_theorem_labels_are_unresolved() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let projections = vec![current_theorem_projection(&fixture, "Later", 20, 5)];
    let references = vec![
        unqualified_ref(source_id, current.clone(), 4, 10, "Later", None),
        unqualified_ref(source_id, current.clone(), 6, 30, "Later", None),
    ];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "Later");
    assert_resolved_label(&resolved, 1, "app::main::theorem::Later");
}

#[test]
fn forward_references_to_later_proof_step_labels_are_unresolved() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let scope = LabelScopePath::new(vec![0]);
    let projections = vec![proof_step_projection(
        &fixture,
        "LaterStep",
        20,
        5,
        scope.clone(),
    )];
    let references = vec![
        unqualified_ref(
            source_id,
            current.clone(),
            4,
            10,
            "LaterStep",
            Some(scope.clone()),
        ),
        unqualified_ref(source_id, current.clone(), 6, 40, "LaterStep", Some(scope)),
    ];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "LaterStep");
    assert_resolved_label(&resolved, 1, "app::main::proof::LaterStep");
}

#[test]
fn qualified_and_lowered_grouped_item_citations_use_module_label_projections() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let namespace = NamespacePath::new("logic");
    let current_namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let imported_contribution = contribution(&mut contributions, dep.clone(), source_id, 0);
    let dep_fixture = ProjectionFixture::new(
        source_id,
        dep.clone(),
        namespace.clone(),
        imported_contribution,
    );
    let projections = vec![
        imported_theorem_projection(&dep_fixture, "Th1", 10),
        imported_theorem_projection(&dep_fixture, "G1", 20),
        imported_theorem_projection(&dep_fixture, "G2", 30),
    ];
    let references = vec![
        qualified_ref(
            source_id,
            current.clone(),
            dep.clone(),
            namespace.clone(),
            2,
            50,
            "Th1",
        ),
        qualified_ref(
            source_id,
            current.clone(),
            dep.clone(),
            namespace.clone(),
            3,
            60,
            "G1",
        ),
        qualified_ref(
            source_id,
            current.clone(),
            dep.clone(),
            namespace,
            4,
            70,
            "G2",
        ),
    ];

    let resolved =
        LabelResolver::new(&projections).resolve(&current, &current_namespace, &references);

    assert_resolved_label(&resolved, 0, "dep::logic::theorem::Th1");
    assert_resolved_label(&resolved, 1, "dep::logic::theorem::G1");
    assert_resolved_label(&resolved, 2, "dep::logic::theorem::G2");
}

#[test]
fn imported_local_only_labels_are_not_visible_to_citations() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let namespace = NamespacePath::new("logic");
    let current_namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let imported_contribution = contribution(&mut contributions, dep.clone(), source_id, 0);
    let dep_fixture = ProjectionFixture::new(
        source_id,
        dep.clone(),
        namespace.clone(),
        imported_contribution,
    );
    let projections = vec![
        imported_theorem_projection(&dep_fixture, "Hidden", 10)
            .with_export_status(ExportStatus::LocalOnly),
    ];
    let references = vec![
        unqualified_ref(source_id, current.clone(), 2, 30, "Hidden", None),
        qualified_ref(
            source_id,
            current.clone(),
            dep,
            namespace.clone(),
            3,
            40,
            "Hidden",
        ),
    ];

    let resolved =
        LabelResolver::new(&projections).resolve(&current, &current_namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "Hidden");
    assert_unresolved_label(&resolved, 1, LabelExpectation::Theorem, "Hidden");
    assert!(
        resolved
            .index()
            .visible_candidates(&namespace, "Hidden")
            .is_empty()
    );
}

#[test]
fn recovered_empty_and_failed_namespace_references_are_unresolved() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let scope = LabelScopePath::new(vec![0]);
    let (recovered_site, recovered_origin) = reference_site(source_id, current.clone(), 10, "A", 1);
    let (empty_site, empty_origin) = reference_site(source_id, current.clone(), 20, "", 2);
    let (failed_site, failed_origin) = reference_site(source_id, current.clone(), 30, "B", 3);
    let references = vec![
        LabelReferenceCandidate::unqualified_citation(
            recovered_site,
            recovered_origin.recovered(),
            1,
            Some(scope),
        ),
        LabelReferenceCandidate::unqualified_citation(empty_site, empty_origin, 2, None),
        LabelReferenceCandidate::failed_namespace(
            failed_site,
            failed_origin,
            3,
            LabelExpectation::Theorem,
        ),
    ];

    let resolved = LabelResolver::new(&[]).resolve(&current, &namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "A");
    assert_unresolved_label(&resolved, 1, LabelExpectation::ProofOrTheorem, "");
    assert_unresolved_label(&resolved, 2, LabelExpectation::Theorem, "B");
}

#[test]
fn recovered_label_projections_do_not_emit_conflict_diagnostics() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let clean = current_theorem_projection(&fixture, "A", 10, 1);
    let mut recovered_data = fixture.data("A", LabelKind::Theorem, "theorem", 20, 2);
    recovered_data.origin = recovered_data.origin.recovered();
    let recovered = LabelProjection::current_module(recovered_data, 2)
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported);
    let projections = vec![clean, recovered];
    let references = vec![unqualified_ref(
        source_id,
        current.clone(),
        3,
        30,
        "A",
        None,
    )];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert!(resolved.diagnostics().is_empty());
    assert_resolved_label(&resolved, 0, "app::main::theorem::A");
}

#[test]
fn ambiguous_cross_family_citations_keep_sorted_candidates() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let scope = LabelScopePath::new(vec![0]);
    let projections = vec![
        current_theorem_projection(&fixture, "A", 20, 1),
        proof_step_projection(&fixture, "A", 10, 2, scope.clone()),
    ];
    let references = vec![unqualified_ref(
        source_id,
        current.clone(),
        3,
        30,
        "A",
        Some(scope),
    )];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    let entry = resolved.table().get(resolved.ids()[0]).unwrap();
    let LabelResolution::Ambiguous(ambiguous) = entry.resolution() else {
        panic!("expected ambiguous label reference");
    };
    assert_eq!(
        ambiguous
            .candidates()
            .iter()
            .map(|candidate| candidate.origin().as_str())
            .collect::<Vec<_>>(),
        vec!["app::main::proof::A", "app::main::theorem::A"]
    );
}

#[test]
fn label_index_and_reference_table_order_are_deterministic() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let projections = vec![
        current_theorem_projection(&fixture, "Z", 30, 3),
        current_theorem_projection(&fixture, "A", 10, 1),
        current_theorem_projection(&fixture, "M", 20, 2),
    ];
    let references = vec![
        unqualified_ref(source_id, current.clone(), 6, 60, "Z", None),
        unqualified_ref(source_id, current.clone(), 4, 40, "A", None),
        unqualified_ref(source_id, current.clone(), 5, 50, "M", None),
    ];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert_eq!(
        resolved
            .index()
            .iter()
            .map(|entry| entry.origin_path().as_str())
            .collect::<Vec<_>>(),
        vec![
            "app::main::theorem::A",
            "app::main::theorem::M",
            "app::main::theorem::Z",
        ]
    );
    assert_eq!(
        resolved
            .ids()
            .iter()
            .map(|id| resolved.table().get(*id).unwrap().site().spelling())
            .collect::<Vec<_>>(),
        vec!["A", "M", "Z"]
    );
}

#[test]
fn source_collector_reproduces_exact_inner_to_outer_b5c_profile() {
    let source_id = source_id();
    let ast = b5c_inner_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    assert_eq!(collection.projections().len(), 1);
    assert_eq!(collection.references().len(), 1);
    let projection = &collection.projections()[0];
    assert_eq!(projection.primary_spelling(), "A");
    assert_eq!(projection.kind(), LabelKind::ProofStep);
    assert_eq!(projection.declaration_range(), range(source_id, 80, 81));
    assert_eq!(
        projection.origin_path().as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=33:ProofLabelInnerToOuterConfinement|owner-occurrence=0|proof-path=1:0|label=1:A|label-occurrence=0"
    );
    assert_eq!(projection.origin().source_id(), source_id);
    assert_eq!(projection.origin().module_id(), &module);
    assert_eq!(
        projection.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 80, 81))
    );
    assert_eq!(projection.origin().structural_path(), &[57, 42, 8]);
    match projection.source() {
        LabelProjectionSource::CurrentModule {
            visible_after_ordinal,
            proof_scope: Some(scope),
        } => {
            assert_eq!(*visible_after_ordinal, 3);
            assert_eq!(scope.path(), &[0, 0]);
        }
        other => panic!("unexpected projection source: {other:?}"),
    }

    let reference = &collection.references()[0];
    assert_eq!(reference.site().node().index(), 52);
    assert_eq!(reference.site().range(), range(source_id, 165, 166));
    assert_eq!(reference.site().spelling(), "A");
    assert_eq!(reference.ordinal(), 5);
    assert_eq!(reference.origin().source_id(), source_id);
    assert_eq!(reference.origin().module_id(), &module);
    assert_eq!(
        reference.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 165, 166))
    );
    assert_eq!(reference.origin().structural_path(), &[57, 55, 52]);
    match reference.scope() {
        LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } => assert_eq!(scope.path(), &[0]),
        other => panic!("unexpected reference scope: {other:?}"),
    }

    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert_eq!(result.index().len(), 1);
    assert_eq!(result.table().len(), 1);
    assert!(result.diagnostics().is_empty());
    assert!(result.has_unresolved());
    assert_unresolved_label(&result, 0, LabelExpectation::ProofOrTheorem, "A");
}

#[test]
fn source_collector_reproduces_exact_sibling_b5c_profile() {
    let source_id = source_id();
    let ast = b5c_sibling_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    let [projection] = collection.projections() else {
        panic!("expected one projection");
    };
    assert_eq!(projection.declaration_range(), range(source_id, 75, 76));
    assert_eq!(projection.origin().source_id(), source_id);
    assert_eq!(projection.origin().module_id(), &module);
    assert_eq!(
        projection.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 75, 76))
    );
    assert_eq!(projection.origin().structural_path(), &[67, 47, 8]);
    assert_eq!(
        projection.origin_path().as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=28:ProofLabelSiblingConfinement|owner-occurrence=0|proof-path=1:0|label=1:A|label-occurrence=0"
    );
    match projection.source() {
        LabelProjectionSource::CurrentModule {
            visible_after_ordinal,
            proof_scope: Some(scope),
        } => {
            assert_eq!(*visible_after_ordinal, 3);
            assert_eq!(scope.path(), &[0, 0]);
        }
        other => panic!("unexpected projection source: {other:?}"),
    }

    let [reference] = collection.references() else {
        panic!("expected one reference");
    };
    assert_eq!(reference.site().node().index(), 60);
    assert_eq!(reference.site().range(), range(source_id, 182, 183));
    assert_eq!(reference.site().spelling(), "A");
    assert_eq!(reference.ordinal(), 6);
    assert_eq!(reference.origin().source_id(), source_id);
    assert_eq!(reference.origin().module_id(), &module);
    assert_eq!(
        reference.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 182, 183))
    );
    assert_eq!(reference.origin().structural_path(), &[67, 63, 60]);
    match reference.scope() {
        LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } => assert_eq!(scope.path(), &[0, 1]),
        other => panic!("unexpected reference scope: {other:?}"),
    }

    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert!(result.diagnostics().is_empty());
    assert_unresolved_label(&result, 0, LabelExpectation::ProofOrTheorem, "A");

    let inner = b5c_inner_ast(source_id);
    let inner_resolved = SurfaceResolvedArena::lower(&inner, &module).unwrap();
    let inner_collection =
        ProofLabelSourceCollector::new(&inner, &module, namespace, contribution, &inner_resolved)
            .unwrap()
            .collect()
            .unwrap();
    assert_ne!(
        inner_collection.projections()[0].origin_path(),
        projection.origin_path(),
        "the two B5C owners must retain distinct canonical label identities"
    );
}

#[test]
fn source_collector_defers_own_proof_visibility_and_allows_post_completion_use() {
    let source_id = source_id();
    let ast = own_proof_and_post_completion_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    let [projection] = collection.projections() else {
        panic!("expected one projection");
    };
    assert_eq!(projection.visible_after_ordinal(), Some(2));
    assert_eq!(
        collection
            .references()
            .iter()
            .map(LabelReferenceCandidate::ordinal)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert_unresolved_label(&result, 0, LabelExpectation::ProofOrTheorem, "A");
    assert_resolved_label(&result, 1, projection.origin_path().as_str());
}

#[test]
fn source_collector_allows_enclosing_label_in_child_and_skips_unlisted_proof_children() {
    let source_id = source_id();
    let ast = enclosing_to_child_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    assert_eq!(collection.projections().len(), 1);
    assert_eq!(collection.references().len(), 1);
    assert_eq!(
        collection.projections()[0].proof_scope().unwrap().path(),
        &[0]
    );
    assert_eq!(collection.references()[0].ordinal(), 3);
    match collection.references()[0].scope() {
        LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } => assert_eq!(scope.path(), &[0, 0]),
        other => panic!("unexpected reference scope: {other:?}"),
    }
    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert_resolved_label(
        &result,
        0,
        collection.projections()[0].origin_path().as_str(),
    );
}

#[test]
fn source_collector_mixed_reference_list_is_default_deny_and_ordered() {
    let source_id = source_id();
    let ast = mixed_reference_ast(source_id);
    let module = module_id("pkg", "main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection = ProofLabelSourceCollector::new(
        &ast,
        &module,
        NamespacePath::new("main"),
        contribution,
        &resolved,
    )
    .unwrap()
    .collect()
    .unwrap();

    assert!(collection.projections().is_empty());
    assert_eq!(
        collection
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("A", 1), ("B", 1)]
    );
}

#[test]
fn source_collector_keeps_global_ordinals_and_theorem_roots_isolated() {
    let source_id = source_id();
    let ast = cross_theorem_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    assert_eq!(collection.projections().len(), 2);
    assert_eq!(
        collection
            .projections()
            .iter()
            .map(|projection| projection.proof_scope().unwrap().path())
            .collect::<Vec<_>>(),
        vec![&[0][..], &[1][..]]
    );
    assert!(
        collection.projections()[0]
            .origin_path()
            .as_str()
            .contains("|owner-occurrence=0|")
    );
    assert!(
        collection.projections()[1]
            .origin_path()
            .as_str()
            .contains("|owner-occurrence=1|")
    );
    assert_eq!(collection.projections()[0].visible_after_ordinal(), Some(1));
    assert!(
        collection.projections()[0]
            .origin_path()
            .as_str()
            .ends_with("|label=1:A|label-occurrence=0")
    );
    assert!(
        collection.projections()[1]
            .origin_path()
            .as_str()
            .ends_with("|label=1:A|label-occurrence=0"),
        "same-spelling label occurrence resets in the later theorem proof scope"
    );
    assert_eq!(collection.references()[0].ordinal(), 2);
    assert!(
        collection.projections()[0].visible_after_ordinal().unwrap()
            < collection.references()[0].ordinal(),
        "the earlier theorem projection must be ordinal-eligible and fail only confinement"
    );
    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert!(result.diagnostics().is_empty());
    assert_unresolved_label(&result, 0, LabelExpectation::ProofOrTheorem, "A");
}

#[test]
fn source_collector_rejects_malformed_upper_chains_and_recovered_boundary() {
    let source_id = source_id();
    let module = module_id("pkg", "main");
    for ast in [
        malformed_upper_ast(source_id, UpperShape::MissingCompilation),
        malformed_upper_ast(source_id, UpperShape::AdditionalCompilation),
        malformed_upper_ast(source_id, UpperShape::WrongCompilationChild),
        malformed_upper_ast(source_id, UpperShape::RootWrongStructuralChild),
        malformed_upper_ast(source_id, UpperShape::TheoremDirectlyUnderRoot),
        malformed_upper_ast(source_id, UpperShape::MissingItemList),
        malformed_upper_ast(source_id, UpperShape::AdditionalItemList),
        relocated_theorem_ast(source_id, true),
        relocated_theorem_ast(source_id, false),
        recovered_proof_boundary_ast(source_id),
    ] {
        let mut contributions = SourceContributionIndex::new();
        let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
        let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
        let collection = ProofLabelSourceCollector::new(
            &ast,
            &module,
            NamespacePath::new("main"),
            contribution,
            &resolved,
        )
        .unwrap()
        .collect()
        .unwrap();
        assert!(collection.projections().is_empty());
        assert!(collection.references().is_empty());
    }
}

#[test]
fn source_collector_rejects_wrong_module_and_reports_checked_overflow() {
    let source_id = source_id();
    let ast = b5c_inner_ast(source_id);
    let module = module_id("pkg", "main");
    let wrong_module = module_id("pkg", "other");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();

    assert!(matches!(
        ProofLabelSourceCollector::new(&ast, &wrong_module, namespace, contribution, &resolved),
        Err(ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::ModuleMismatch
        ))
    ));
    let stale_ast = b5c_sibling_ast(source_id);
    assert!(matches!(
        ProofLabelSourceCollector::new(
            &stale_ast,
            &module,
            NamespacePath::new("main"),
            contribution,
            &resolved
        ),
        Err(ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::NodeCountMismatch
        ))
    ));
    #[cfg(target_pointer_width = "64")]
    {
        let node = ast.root().unwrap();
        let overflowing = usize::try_from(u64::from(u32::MAX) + 1).unwrap();
        assert!(matches!(
            checked_scope_component(node, overflowing),
            Err(ProofLabelSourceCollectionError::ScopeComponentOverflow {
                node: error_node
            }) if error_node == node
        ));
        assert!(matches!(
            checked_structural_component_value(node, overflowing),
            Err(
                ProofLabelSourceCollectionError::StructuralPathComponentOverflow {
                    node: error_node
                }
            ) if error_node == node
        ));
    }
}

#[test]
fn source_collector_is_deterministic_and_identity_is_length_framed() {
    let source_id = source_id();
    let ast = b5c_inner_ast(source_id);
    let module = module_id("på", "mód");
    let namespace = NamespacePath::new("mód");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collector =
        ProofLabelSourceCollector::new(&ast, &module, namespace, contribution, &resolved).unwrap();

    let first = collector.collect().unwrap();
    let second = collector.collect().unwrap();
    assert_eq!(first, second);
    assert_eq!(first.clone(), first);
    assert!(
        first.projections()[0]
            .origin_path()
            .as_str()
            .starts_with("proof-step-v1|package=3:på|module=4:mód|contribution=0|")
    );

    let empty_path = proof_label_origin_path(&module, contribution, "Ow|ner", 2, &[], "Å:|", 3);
    assert_eq!(
        empty_path.as_str(),
        "proof-step-v1|package=3:på|module=4:mód|contribution=0|owner-kind=theorem|owner=6:Ow|ner|owner-occurrence=2|proof-path=0:|label=4:Å:||label-occurrence=3"
    );
}

#[test]
fn proof_step_identity_mutates_each_framed_component_independently() {
    let source_id = source_id();
    let base_module = module_id("pkg", "main");
    let mut contributions = SourceContributionIndex::new();
    let base_contribution = contribution(&mut contributions, base_module.clone(), source_id, 0);
    let other_contribution = contribution(&mut contributions, base_module.clone(), source_id, 10);
    let base = proof_label_origin_path(
        &base_module,
        base_contribution,
        "Owner",
        0,
        &[0, 2],
        "Label",
        0,
    );
    assert_eq!(
        base.as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=5:Owner|owner-occurrence=0|proof-path=2:0,2|label=5:Label|label-occurrence=0"
    );

    let mutations = [
        proof_label_origin_path(
            &module_id("other", "main"),
            base_contribution,
            "Owner",
            0,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &module_id("pkg", "other"),
            base_contribution,
            "Owner",
            0,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            other_contribution,
            "Owner",
            0,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Other",
            0,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Owner",
            1,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Owner",
            0,
            &[0, 3],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Owner",
            0,
            &[0, 2],
            "Other",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Owner",
            0,
            &[0, 2],
            "Label",
            1,
        ),
    ];
    for mutation in mutations {
        assert_ne!(mutation, base);
    }

    let exact_bytes = proof_label_origin_path(
        &module_id("på", "mód"),
        base_contribution,
        "Öw|ner",
        2,
        &[],
        "Å:|",
        3,
    );
    assert_eq!(
        exact_bytes.as_str(),
        "proof-step-v1|package=3:på|module=4:mód|contribution=0|owner-kind=theorem|owner=7:Öw|ner|owner-occurrence=2|proof-path=0:|label=4:Å:||label-occurrence=3"
    );

    let composed =
        proof_label_origin_path(&base_module, base_contribution, "Owner", 0, &[], "Å", 0);
    let decomposed = proof_label_origin_path(
        &base_module,
        base_contribution,
        "Owner",
        0,
        &[],
        "A\u{030a}",
        0,
    );
    assert_eq!(
        composed.as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=5:Owner|owner-occurrence=0|proof-path=0:|label=2:Å|label-occurrence=0"
    );
    assert_eq!(
        decomposed.as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=5:Owner|owner-occurrence=0|proof-path=0:|label=3:A\u{030a}|label-occurrence=0"
    );
    assert_ne!(
        composed, decomposed,
        "identity must preserve exact parser bytes without Unicode normalization"
    );
}

#[test]
fn source_collector_accepts_root_to_compilation_with_direct_token_siblings() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let statement = builder.valid_statement("A", Some("A"), false);
    let theorem = builder.valid_theorem("Owner", vec![statement]);
    let ast = builder.finish_items(vec![theorem]);

    let root = ast.root_view().unwrap();
    assert!(
        root.child_views()
            .filter(|child| matches!(child.kind(), SurfaceNodeKind::Token(_)))
            .count()
            > 1
    );
    assert_eq!(
        root.child_views()
            .filter(|child| matches!(child.kind(), SurfaceNodeKind::CompilationUnit))
            .count(),
        1
    );
    let collection = collect_fixture(&ast);
    assert_eq!(collection.projections().len(), 1);
    assert_eq!(collection.references().len(), 1);
}

#[test]
fn source_collector_accepts_compilation_to_exact_item_list() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let statement = builder.valid_statement("A", None, false);
    let theorem = builder.valid_theorem("Owner", vec![statement]);
    let ast = builder.finish_items(vec![theorem]);

    let item_list = exact_proof_label_item_list(&ast).expect("exact upper chain");
    assert!(matches!(item_list.kind(), SurfaceNodeKind::ItemList));
    assert_eq!(collect_fixture(&ast).projections().len(), 1);
}

#[test]
fn source_collector_scans_only_direct_item_list_theorems() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let hidden_theorem = builder.valid_theorem("HiddenOwner", vec![hidden_statement]);
    let wrapper = builder.node(SurfaceNodeKind::VisibleItem, vec![hidden_theorem]);
    let direct_statement = builder.valid_statement("Direct", Some("Direct"), false);
    let direct_theorem = builder.valid_theorem("DirectOwner", vec![direct_statement]);
    let ast = builder.finish_items(vec![wrapper, direct_theorem]);

    let collection = collect_fixture(&ast);
    assert_eq!(
        collection
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["Direct"]
    );
    assert_eq!(
        collection
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("Direct", 1)]
    );
}

#[test]
fn source_collector_default_denies_every_representative_unsupported_item_owner() {
    let source_id = source_id();
    for kind in [
        SurfaceNodeKind::LemmaItem,
        SurfaceNodeKind::VisibleItem,
        SurfaceNodeKind::StatementItem,
        SurfaceNodeKind::DefinitionBlockItem,
        SurfaceNodeKind::RegistrationBlockItem,
        SurfaceNodeKind::ClaimBlockItem,
        SurfaceNodeKind::Annotation,
        SurfaceNodeKind::PlaceholderItem,
    ] {
        let collection = collect_fixture(&unsupported_item_wrapper_ast(source_id, kind.clone()));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "unsupported item kind {kind:?} must not be descended"
        );
        assert_eq!(
            collection.projections()[0].proof_scope().unwrap().path(),
            &[0],
            "unsupported item kind {kind:?} must not allocate a theorem root"
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("Sentinel", 1)],
            "unsupported item kind {kind:?} must consume no ordinal"
        );
    }

    let recovered = collect_fixture(&recovered_item_wrapper_ast(source_id));
    assert_eq!(
        recovered
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["Sentinel"]
    );
    assert_eq!(recovered.references()[0].ordinal(), 1);
}

#[test]
fn source_collector_default_denies_every_representative_unlisted_proof_child() {
    let source_id = source_id();
    for kind in [
        SurfaceNodeKind::LetStatement,
        SurfaceNodeKind::AssumptionStatement,
        SurfaceNodeKind::GivenStatement,
        SurfaceNodeKind::TakeStatement,
        SurfaceNodeKind::SetStatement,
        SurfaceNodeKind::ConsiderStatement,
        SurfaceNodeKind::ReconsiderStatement,
        SurfaceNodeKind::CaseReasoningStatement,
        SurfaceNodeKind::CaseItem,
        SurfaceNodeKind::SupposeItem,
        SurfaceNodeKind::NowStatement,
        SurfaceNodeKind::HerebyStatement,
        SurfaceNodeKind::IterativeEqualityStatement,
    ] {
        let collection = collect_fixture(&unsupported_proof_child_ast(source_id, kind.clone()));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "unlisted proof child {kind:?} must not be descended"
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("Sentinel", 1)],
            "unlisted proof child {kind:?} must consume no ordinal"
        );
    }
}

#[test]
fn source_collector_rejects_every_required_theorem_owner_shape_mutation() {
    let source_id = source_id();
    for mutation in [
        TheoremShapeMutation::WrongRole,
        TheoremShapeMutation::WrongOwnerKind,
        TheoremShapeMutation::WrongColon,
        TheoremShapeMutation::MissingProof,
        TheoremShapeMutation::AdditionalProof,
        TheoremShapeMutation::WrappedProof,
        TheoremShapeMutation::RecoveredRole,
        TheoremShapeMutation::RecoveredOwner,
        TheoremShapeMutation::RecoveredColon,
    ] {
        let collection = collect_fixture(&malformed_theorem_owner_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "malformed theorem mutation {mutation:?} must not be descended"
        );
        assert_eq!(
            collection.projections()[0].proof_scope().unwrap().path(),
            &[0],
            "malformed theorem mutation {mutation:?} must not allocate a root"
        );
        assert_eq!(
            collection.references()[0].ordinal(),
            1,
            "malformed theorem mutation {mutation:?} must consume no ordinal"
        );
    }
}

#[test]
fn source_collector_rejects_every_malformed_or_recovered_proof_boundary() {
    let source_id = source_id();
    for mutation in [
        ProofBoundaryMutation::MissingEnd,
        ProofBoundaryMutation::WrongStart,
        ProofBoundaryMutation::WrongEnd,
        ProofBoundaryMutation::RecoveredStart,
        ProofBoundaryMutation::RecoveredEnd,
        ProofBoundaryMutation::RecoveredInterior,
    ] {
        let collection = collect_fixture(&malformed_proof_boundary_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "malformed proof mutation {mutation:?} must not be descended"
        );
        assert_eq!(
            collection.projections()[0].proof_scope().unwrap().path(),
            &[0],
            "malformed proof mutation {mutation:?} must not allocate a root"
        );
        assert_eq!(
            collection.references()[0].ordinal(),
            1,
            "malformed proof mutation {mutation:?} must consume no ordinal"
        );
    }
}

#[test]
fn source_collector_rejects_every_malformed_compact_label_edge_but_keeps_statement_ordinal() {
    let source_id = source_id();
    for mutation in [
        CompactLabelMutation::MissingIdentifier,
        CompactLabelMutation::WrongIdentifierKind,
        CompactLabelMutation::MissingColon,
        CompactLabelMutation::WrongColon,
        CompactLabelMutation::RecoveredIdentifier,
        CompactLabelMutation::RecoveredColon,
        CompactLabelMutation::AdditionalProposition,
    ] {
        let collection = collect_fixture(&malformed_compact_label_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "malformed compact label {mutation:?} must emit no projection"
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("MutRef", 1), ("Sentinel", 2)],
            "malformed label {mutation:?} must not reject its supported statement"
        );
    }
}

#[test]
fn source_collector_rejects_every_malformed_simple_reference_edge() {
    let source_id = source_id();
    for mutation in [
        ReferenceMutation::MissingIdentifier,
        ReferenceMutation::WrongIdentifierKind,
        ReferenceMutation::AdditionalChild,
        ReferenceMutation::RecoveredIdentifier,
    ] {
        let collection = collect_fixture(&malformed_reference_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["MutLabel", "Sentinel"]
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("Sentinel", 2)],
            "malformed reference {mutation:?} must emit no row"
        );
    }
}

#[test]
fn source_collector_default_denies_relocated_wrapped_and_computation_edges() {
    let source_id = source_id();
    for mutation in [
        JustificationMutation::WrongFirstToken,
        JustificationMutation::RecoveredFirstToken,
        JustificationMutation::AdditionalReferenceList,
        JustificationMutation::ComputationInstead,
        JustificationMutation::WrappedJustification,
        JustificationMutation::WrappedReferenceList,
        JustificationMutation::RelocatedIntoProposition,
    ] {
        let collection = collect_fixture(&malformed_justification_edge_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["MutLabel", "Sentinel"]
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("Sentinel", 2)],
            "rejected edge {mutation:?} must emit no reference and consume no extra ordinal"
        );
    }

    let relocated_proof = collect_fixture(&malformed_justification_edge_ast(
        source_id,
        JustificationMutation::ProofRelocatedIntoProposition,
    ));
    assert_eq!(
        relocated_proof
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["MutLabel", "Sentinel"]
    );
    assert_eq!(
        relocated_proof.projections()[0].visible_after_ordinal(),
        Some(1),
        "a proof below Proposition must not be descended or delay visibility"
    );
    assert_eq!(
        relocated_proof
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("MutRef", 1), ("Sentinel", 2)]
    );

    let conclusion = collect_fixture(&malformed_justification_edge_ast(
        source_id,
        JustificationMutation::ConclusionLabel,
    ));
    assert_eq!(
        conclusion
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["Sentinel"],
        "ConclusionStatement proposition labels are excluded"
    );
    assert_eq!(
        conclusion
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("MutRef", 1), ("Sentinel", 2)]
    );
}

#[test]
fn source_collector_consumes_supported_unlabelled_statement_ordinals_only() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let compact_proposition = builder.proposition(None, None, true);
    let compact = builder.compact(compact_proposition, None);
    let conclusion_proposition = builder.proposition(None, None, true);
    let conclusion = builder.conclusion(conclusion_proposition, None, None);
    let sentinel = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let theorem = builder.valid_theorem("Owner", vec![compact, conclusion, sentinel]);
    let collection = collect_fixture(&builder.finish_items(vec![theorem]));

    assert_eq!(collection.projections().len(), 1);
    assert_eq!(collection.projections()[0].visible_after_ordinal(), Some(3));
    assert_eq!(
        collection
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("Sentinel", 3)]
    );
}

#[test]
fn source_collector_accepts_two_child_label_and_never_descends_semantic_children() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let two_child = builder.proposition(Some("TwoChild"), Some(":"), false);
    let statement = builder.compact(two_child, None);
    let theorem = builder.valid_theorem("Owner", vec![statement]);
    let ast = builder.finish_items(vec![theorem]);
    let collection = collect_fixture(&ast);
    assert_eq!(collection.projections()[0].primary_spelling(), "TwoChild");
    assert!(collection.references().is_empty());

    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_reference = builder.reference("Hidden", false, false);
    let semantic = builder.node(SurfaceNodeKind::FormulaExpression, vec![hidden_reference]);
    let label = builder.token(SurfaceTokenKind::Identifier, "Visible");
    let colon = builder.token(SurfaceTokenKind::ReservedSymbol, ":");
    let proposition = builder.node(SurfaceNodeKind::Proposition, vec![label, colon, semantic]);
    let statement = builder.compact(proposition, None);
    let theorem = builder.valid_theorem("Owner", vec![statement]);
    let ast = builder.finish_items(vec![theorem]);
    let collection = collect_fixture(&ast);
    assert_eq!(collection.projections()[0].primary_spelling(), "Visible");
    assert!(collection.references().is_empty());
}

#[test]
fn source_collector_identity_is_stable_under_formatting_owner_and_order_mutations() {
    let source_id = source_id();
    let base = collect_fixture(&two_theorem_identity_ast(
        source_id,
        0,
        "FirstOwner",
        "StableOwner",
        false,
    ));
    let formatted = collect_fixture(&two_theorem_identity_ast(
        source_id,
        500,
        "FirstOwner",
        "StableOwner",
        false,
    ));
    assert_eq!(
        base.projections()
            .iter()
            .map(|projection| projection.origin_path().as_str())
            .collect::<Vec<_>>(),
        formatted
            .projections()
            .iter()
            .map(|projection| projection.origin_path().as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        base.references()
            .iter()
            .map(|reference| reference.site().spelling())
            .collect::<Vec<_>>(),
        vec!["FirstRef", "SecondRef"]
    );

    let renamed = collect_fixture(&two_theorem_identity_ast(
        source_id,
        0,
        "RenamedFirstOwner",
        "StableOwner",
        false,
    ));
    let base_stable = base
        .projections()
        .iter()
        .find(|projection| projection.primary_spelling() == "SecondLabel")
        .unwrap();
    let renamed_stable = renamed
        .projections()
        .iter()
        .find(|projection| projection.primary_spelling() == "SecondLabel")
        .unwrap();
    assert_eq!(base_stable.origin_path(), renamed_stable.origin_path());

    let reordered = collect_fixture(&two_theorem_identity_ast(
        source_id,
        0,
        "FirstOwner",
        "StableOwner",
        true,
    ));
    assert_eq!(
        reordered
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["SecondLabel", "FirstLabel"]
    );
    assert_eq!(
        reordered
            .references()
            .iter()
            .map(|reference| reference.site().spelling())
            .collect::<Vec<_>>(),
        vec!["SecondRef", "FirstRef"]
    );
    for spelling in ["FirstLabel", "SecondLabel"] {
        let before = base
            .projections()
            .iter()
            .find(|projection| projection.primary_spelling() == spelling)
            .unwrap();
        let after = reordered
            .projections()
            .iter()
            .find(|projection| projection.primary_spelling() == spelling)
            .unwrap();
        assert_eq!(before.origin_path(), after.origin_path());
    }
}

#[test]
fn source_collector_identity_tracks_proof_topology_and_occurrence_only() {
    let source_id = source_id();
    let root = collect_fixture(&label_topology_ast(source_id, false));
    let nested = collect_fixture(&label_topology_ast(source_id, true));
    assert!(
        root.projections()[0]
            .origin_path()
            .as_str()
            .contains("|proof-path=0:|")
    );
    assert!(
        nested.projections()[0]
            .origin_path()
            .as_str()
            .contains("|proof-path=1:0|")
    );
    assert_ne!(
        root.projections()[0].origin_path(),
        nested.projections()[0].origin_path()
    );

    let repeated = collect_fixture(&same_scope_labels_ast(source_id));
    assert_eq!(repeated.projections().len(), 2);
    assert!(
        repeated.projections()[0]
            .origin_path()
            .as_str()
            .ends_with("|label-occurrence=0")
    );
    assert!(
        repeated.projections()[1]
            .origin_path()
            .as_str()
            .ends_with("|label-occurrence=1")
    );
    assert_ne!(
        repeated.projections()[0].origin_path(),
        repeated.projections()[1].origin_path()
    );
}

#[derive(Clone, Copy)]
enum ValidationAstMutation {
    None,
    Kind,
    Children,
    Range,
    Recovery,
    Root,
}

fn validation_ast(source_id: SourceId, mutation: ValidationAstMutation) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let token = if matches!(mutation, ValidationAstMutation::Recovery) {
        builder.add_recovered_token(
            SurfaceTokenKind::ReservedWord,
            "token",
            range(source_id, 0, 5),
        )
    } else {
        builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "token",
            range(source_id, 0, 5),
        )
    };
    let item_kind = if matches!(mutation, ValidationAstMutation::Kind) {
        SurfaceNodeKind::PlaceholderItem
    } else {
        SurfaceNodeKind::ItemList
    };
    let item_children = if matches!(mutation, ValidationAstMutation::Children) {
        vec![token]
    } else {
        Vec::new()
    };
    let item_range = if matches!(mutation, ValidationAstMutation::Range) {
        range(source_id, 10, 11)
    } else {
        range(source_id, 0, 5)
    };
    let item_list = builder.add_node(item_kind, item_range, item_children);
    let compilation = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 5),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 5),
        vec![token, compilation],
    );
    let root = if matches!(mutation, ValidationAstMutation::Root) {
        compilation
    } else {
        root
    };
    builder.finish(Some(root), None)
}

fn proof_label_surface_error_class(error: ProofLabelSourceCollectionError) -> &'static str {
    match error {
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::SourceMismatch,
        ) => "source",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::ModuleMismatch,
        ) => "module",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::NodeCountMismatch,
        ) => "count",
        ProofLabelSourceCollectionError::SurfaceArena(SurfaceResolvedArenaError::RootMismatch) => {
            "root"
        }
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::NodeKindMismatch { .. },
        ) => "kind",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::ChildListMismatch { .. },
        ) => "children",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::RangeMismatch { .. },
        ) => "range",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::RecoveryMismatch { .. },
        ) => "recovery",
        _ => "other",
    }
}

fn assert_collector_rejects_stale_arena(
    ast: &SurfaceAst,
    module: &ModuleId,
    contribution: SourceContributionId,
    resolved: &SurfaceResolvedArena,
    expected: &str,
) {
    let constructor_error = ProofLabelSourceCollector::new(
        ast,
        module,
        NamespacePath::new("main"),
        contribution,
        resolved,
    )
    .err()
    .expect("constructor must reject stale arena");
    assert_eq!(proof_label_surface_error_class(constructor_error), expected);

    let collector = ProofLabelSourceCollector {
        ast,
        namespace: NamespacePath::new("main"),
        contribution,
        resolved,
    };
    let collect_error = collector
        .collect()
        .expect_err("collect must revalidate stale arena");
    assert_eq!(proof_label_surface_error_class(collect_error), expected);
}

#[test]
fn source_collector_forwards_r032a_source_node_shape_range_and_recovery_errors() {
    let (source_id, other_source_id) = distinct_source_ids();
    let module = module_id("pkg", "main");
    let base = validation_ast(source_id, ValidationAstMutation::None);
    let resolved = SurfaceResolvedArena::lower(&base, &module).unwrap();
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);

    let cases = [
        (
            validation_ast(other_source_id, ValidationAstMutation::None),
            "source",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Kind),
            "kind",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Children),
            "children",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Range),
            "range",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Recovery),
            "recovery",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Root),
            "root",
        ),
    ];
    for (ast, expected) in &cases {
        assert_collector_rejects_stale_arena(ast, &module, contribution, &resolved, expected);
    }

    let wrong_module = module_id("pkg", "other");
    let error = ProofLabelSourceCollector::new(
        &base,
        &wrong_module,
        NamespacePath::new("main"),
        contribution,
        &resolved,
    )
    .err()
    .expect("wrong module must be rejected");
    assert!(
        error
            .to_string()
            .contains("invalid proof-label surface arena")
    );
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(proof_label_surface_error_class(error), "module");

    let larger = b5c_inner_ast(source_id);
    assert_collector_rejects_stale_arena(&larger, &module, contribution, &resolved, "count");
}

fn b5c_inner_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (
            SurfaceTokenKind::Identifier,
            "ProofLabelInnerToOuterConfinement",
            8,
            41,
        ),
        (SurfaceTokenKind::ReservedSymbol, ":", 41, 42),
        (SurfaceTokenKind::ReservedWord, "thesis", 43, 49),
        (SurfaceTokenKind::ReservedWord, "proof", 50, 55),
        (SurfaceTokenKind::ReservedWord, "thus", 58, 62),
        (SurfaceTokenKind::ReservedWord, "thesis", 63, 69),
        (SurfaceTokenKind::ReservedWord, "proof", 70, 75),
        (SurfaceTokenKind::Identifier, "A", 80, 81),
        (SurfaceTokenKind::ReservedSymbol, ":", 81, 82),
        (SurfaceTokenKind::ReservedWord, "thesis", 83, 89),
        (SurfaceTokenKind::ReservedWord, "proof", 90, 95),
        (SurfaceTokenKind::ReservedWord, "thus", 102, 106),
        (SurfaceTokenKind::ReservedWord, "thesis", 107, 113),
        (SurfaceTokenKind::ReservedSymbol, ";", 113, 114),
        (SurfaceTokenKind::ReservedWord, "end", 119, 122),
        (SurfaceTokenKind::ReservedSymbol, ";", 122, 123),
        (SurfaceTokenKind::ReservedWord, "thus", 128, 132),
        (SurfaceTokenKind::ReservedWord, "thesis", 133, 139),
        (SurfaceTokenKind::ReservedSymbol, ";", 139, 140),
        (SurfaceTokenKind::ReservedWord, "end", 143, 146),
        (SurfaceTokenKind::ReservedSymbol, ";", 146, 147),
        (SurfaceTokenKind::ReservedWord, "thus", 150, 154),
        (SurfaceTokenKind::ReservedWord, "thesis", 155, 161),
        (SurfaceTokenKind::ReservedWord, "by", 162, 164),
        (SurfaceTokenKind::Identifier, "A", 165, 166),
        (SurfaceTokenKind::ReservedSymbol, ";", 166, 167),
        (SurfaceTokenKind::ReservedWord, "end", 168, 171),
        (SurfaceTokenKind::ReservedSymbol, ";", 171, 172),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            43,
            49,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 43, 49, &[29]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            63,
            69,
            &[6],
        ),
        node(SurfaceNodeKind::FormulaExpression, 63, 69, &[31]),
        node(SurfaceNodeKind::Proposition, 63, 69, &[32]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            83,
            89,
            &[10],
        ),
        node(SurfaceNodeKind::FormulaExpression, 83, 89, &[34]),
        node(SurfaceNodeKind::Proposition, 80, 89, &[8, 9, 35]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            107,
            113,
            &[13],
        ),
        node(SurfaceNodeKind::FormulaExpression, 107, 113, &[37]),
        node(SurfaceNodeKind::Proposition, 107, 113, &[38]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            102,
            114,
            &[12, 39, 14],
        ),
        node(SurfaceNodeKind::ProofBlock, 90, 122, &[11, 40, 15]),
        node(SurfaceNodeKind::CompactStatement, 80, 123, &[36, 41, 16]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            133,
            139,
            &[18],
        ),
        node(SurfaceNodeKind::FormulaExpression, 133, 139, &[43]),
        node(SurfaceNodeKind::Proposition, 133, 139, &[44]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            128,
            140,
            &[17, 45, 19],
        ),
        node(SurfaceNodeKind::ProofBlock, 70, 146, &[7, 42, 46, 20]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            58,
            147,
            &[5, 33, 47, 21],
        ),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            155,
            161,
            &[23],
        ),
        node(SurfaceNodeKind::FormulaExpression, 155, 161, &[49]),
        node(SurfaceNodeKind::Proposition, 155, 161, &[50]),
        node(SurfaceNodeKind::Reference, 165, 166, &[25]),
        node(SurfaceNodeKind::ReferenceList, 165, 166, &[52]),
        node(SurfaceNodeKind::JustificationClause, 162, 166, &[24, 53]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            150,
            167,
            &[22, 51, 54, 26],
        ),
        node(SurfaceNodeKind::ProofBlock, 50, 171, &[4, 48, 55, 27]),
        node(SurfaceNodeKind::TheoremItem, 0, 172, &[0, 1, 2, 30, 56, 28]),
        node(SurfaceNodeKind::ItemList, 0, 172, &[57]),
        node(SurfaceNodeKind::CompilationUnit, 0, 172, &[58]),
        node(
            SurfaceNodeKind::Root,
            0,
            172,
            &(0..=28).chain(std::iter::once(59)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn b5c_sibling_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (
            SurfaceTokenKind::Identifier,
            "ProofLabelSiblingConfinement",
            8,
            36,
        ),
        (SurfaceTokenKind::ReservedSymbol, ":", 36, 37),
        (SurfaceTokenKind::ReservedWord, "thesis", 38, 44),
        (SurfaceTokenKind::ReservedWord, "proof", 45, 50),
        (SurfaceTokenKind::ReservedWord, "thus", 53, 57),
        (SurfaceTokenKind::ReservedWord, "thesis", 58, 64),
        (SurfaceTokenKind::ReservedWord, "proof", 65, 70),
        (SurfaceTokenKind::Identifier, "A", 75, 76),
        (SurfaceTokenKind::ReservedSymbol, ":", 76, 77),
        (SurfaceTokenKind::ReservedWord, "thesis", 78, 84),
        (SurfaceTokenKind::ReservedWord, "proof", 85, 90),
        (SurfaceTokenKind::ReservedWord, "thus", 97, 101),
        (SurfaceTokenKind::ReservedWord, "thesis", 102, 108),
        (SurfaceTokenKind::ReservedSymbol, ";", 108, 109),
        (SurfaceTokenKind::ReservedWord, "end", 114, 117),
        (SurfaceTokenKind::ReservedSymbol, ";", 117, 118),
        (SurfaceTokenKind::ReservedWord, "thus", 123, 127),
        (SurfaceTokenKind::ReservedWord, "thesis", 128, 134),
        (SurfaceTokenKind::ReservedSymbol, ";", 134, 135),
        (SurfaceTokenKind::ReservedWord, "end", 138, 141),
        (SurfaceTokenKind::ReservedSymbol, ";", 141, 142),
        (SurfaceTokenKind::ReservedWord, "thus", 145, 149),
        (SurfaceTokenKind::ReservedWord, "thesis", 150, 156),
        (SurfaceTokenKind::ReservedWord, "proof", 157, 162),
        (SurfaceTokenKind::ReservedWord, "thus", 167, 171),
        (SurfaceTokenKind::ReservedWord, "thesis", 172, 178),
        (SurfaceTokenKind::ReservedWord, "by", 179, 181),
        (SurfaceTokenKind::Identifier, "A", 182, 183),
        (SurfaceTokenKind::ReservedSymbol, ";", 183, 184),
        (SurfaceTokenKind::ReservedWord, "end", 187, 190),
        (SurfaceTokenKind::ReservedSymbol, ";", 190, 191),
        (SurfaceTokenKind::ReservedWord, "end", 192, 195),
        (SurfaceTokenKind::ReservedSymbol, ";", 195, 196),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            38,
            44,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 38, 44, &[34]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            58,
            64,
            &[6],
        ),
        node(SurfaceNodeKind::FormulaExpression, 58, 64, &[36]),
        node(SurfaceNodeKind::Proposition, 58, 64, &[37]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            78,
            84,
            &[10],
        ),
        node(SurfaceNodeKind::FormulaExpression, 78, 84, &[39]),
        node(SurfaceNodeKind::Proposition, 75, 84, &[8, 9, 40]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            102,
            108,
            &[13],
        ),
        node(SurfaceNodeKind::FormulaExpression, 102, 108, &[42]),
        node(SurfaceNodeKind::Proposition, 102, 108, &[43]),
        node(SurfaceNodeKind::ConclusionStatement, 97, 109, &[12, 44, 14]),
        node(SurfaceNodeKind::ProofBlock, 85, 117, &[11, 45, 15]),
        node(SurfaceNodeKind::CompactStatement, 75, 118, &[41, 46, 16]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            128,
            134,
            &[18],
        ),
        node(SurfaceNodeKind::FormulaExpression, 128, 134, &[48]),
        node(SurfaceNodeKind::Proposition, 128, 134, &[49]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            123,
            135,
            &[17, 50, 19],
        ),
        node(SurfaceNodeKind::ProofBlock, 65, 141, &[7, 47, 51, 20]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            53,
            142,
            &[5, 38, 52, 21],
        ),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            150,
            156,
            &[23],
        ),
        node(SurfaceNodeKind::FormulaExpression, 150, 156, &[54]),
        node(SurfaceNodeKind::Proposition, 150, 156, &[55]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            172,
            178,
            &[26],
        ),
        node(SurfaceNodeKind::FormulaExpression, 172, 178, &[57]),
        node(SurfaceNodeKind::Proposition, 172, 178, &[58]),
        node(SurfaceNodeKind::Reference, 182, 183, &[28]),
        node(SurfaceNodeKind::ReferenceList, 182, 183, &[60]),
        node(SurfaceNodeKind::JustificationClause, 179, 183, &[27, 61]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            167,
            184,
            &[25, 59, 62, 29],
        ),
        node(SurfaceNodeKind::ProofBlock, 157, 190, &[24, 63, 30]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            145,
            191,
            &[22, 56, 64, 31],
        ),
        node(SurfaceNodeKind::ProofBlock, 45, 195, &[4, 53, 65, 32]),
        node(SurfaceNodeKind::TheoremItem, 0, 196, &[0, 1, 2, 35, 66, 33]),
        node(SurfaceNodeKind::ItemList, 0, 196, &[67]),
        node(SurfaceNodeKind::CompilationUnit, 0, 196, &[68]),
        node(
            SurfaceNodeKind::Root,
            0,
            196,
            &(0..=33).chain(std::iter::once(69)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn own_proof_and_post_completion_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (SurfaceTokenKind::Identifier, "Owner", 8, 13),
        (SurfaceTokenKind::ReservedSymbol, ":", 13, 14),
        (SurfaceTokenKind::ReservedWord, "thesis", 15, 21),
        (SurfaceTokenKind::ReservedWord, "proof", 22, 27),
        (SurfaceTokenKind::Identifier, "A", 28, 29),
        (SurfaceTokenKind::ReservedSymbol, ":", 29, 30),
        (SurfaceTokenKind::ReservedWord, "thesis", 31, 37),
        (SurfaceTokenKind::ReservedWord, "proof", 38, 43),
        (SurfaceTokenKind::ReservedWord, "thus", 44, 48),
        (SurfaceTokenKind::ReservedWord, "thesis", 49, 55),
        (SurfaceTokenKind::ReservedWord, "by", 56, 58),
        (SurfaceTokenKind::Identifier, "A", 59, 60),
        (SurfaceTokenKind::ReservedSymbol, ";", 60, 61),
        (SurfaceTokenKind::ReservedWord, "end", 62, 65),
        (SurfaceTokenKind::ReservedSymbol, ";", 65, 66),
        (SurfaceTokenKind::ReservedWord, "thus", 67, 71),
        (SurfaceTokenKind::ReservedWord, "thesis", 72, 78),
        (SurfaceTokenKind::ReservedWord, "by", 79, 81),
        (SurfaceTokenKind::Identifier, "A", 82, 83),
        (SurfaceTokenKind::ReservedSymbol, ";", 83, 84),
        (SurfaceTokenKind::ReservedWord, "end", 85, 88),
        (SurfaceTokenKind::ReservedSymbol, ";", 88, 89),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            15,
            21,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 15, 21, &[23]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            31,
            37,
            &[7],
        ),
        node(SurfaceNodeKind::FormulaExpression, 31, 37, &[25]),
        node(SurfaceNodeKind::Proposition, 28, 37, &[5, 6, 26]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            49,
            55,
            &[10],
        ),
        node(SurfaceNodeKind::FormulaExpression, 49, 55, &[28]),
        node(SurfaceNodeKind::Proposition, 49, 55, &[29]),
        node(SurfaceNodeKind::Reference, 59, 60, &[12]),
        node(SurfaceNodeKind::ReferenceList, 59, 60, &[31]),
        node(SurfaceNodeKind::JustificationClause, 56, 60, &[11, 32]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            44,
            61,
            &[9, 30, 33, 13],
        ),
        node(SurfaceNodeKind::ProofBlock, 38, 65, &[8, 34, 14]),
        node(SurfaceNodeKind::CompactStatement, 28, 66, &[27, 35, 15]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            72,
            78,
            &[17],
        ),
        node(SurfaceNodeKind::FormulaExpression, 72, 78, &[37]),
        node(SurfaceNodeKind::Proposition, 72, 78, &[38]),
        node(SurfaceNodeKind::Reference, 82, 83, &[19]),
        node(SurfaceNodeKind::ReferenceList, 82, 83, &[40]),
        node(SurfaceNodeKind::JustificationClause, 79, 83, &[18, 41]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            67,
            84,
            &[16, 39, 42, 20],
        ),
        node(SurfaceNodeKind::ProofBlock, 22, 88, &[4, 36, 43, 21]),
        node(SurfaceNodeKind::TheoremItem, 0, 89, &[0, 1, 2, 24, 44, 22]),
        node(SurfaceNodeKind::ItemList, 0, 89, &[45]),
        node(SurfaceNodeKind::CompilationUnit, 0, 89, &[46]),
        node(
            SurfaceNodeKind::Root,
            0,
            89,
            &(0..=22).chain(std::iter::once(47)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn enclosing_to_child_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (SurfaceTokenKind::Identifier, "Owner", 8, 13),
        (SurfaceTokenKind::ReservedSymbol, ":", 13, 14),
        (SurfaceTokenKind::ReservedWord, "thesis", 15, 21),
        (SurfaceTokenKind::ReservedWord, "proof", 22, 27),
        (SurfaceTokenKind::Identifier, "A", 28, 29),
        (SurfaceTokenKind::ReservedSymbol, ":", 29, 30),
        (SurfaceTokenKind::ReservedWord, "thesis", 31, 37),
        (SurfaceTokenKind::ReservedSymbol, ";", 37, 38),
        (SurfaceTokenKind::ReservedWord, "thus", 39, 43),
        (SurfaceTokenKind::ReservedWord, "thesis", 44, 50),
        (SurfaceTokenKind::ReservedWord, "proof", 51, 56),
        (SurfaceTokenKind::ReservedWord, "thus", 57, 61),
        (SurfaceTokenKind::ReservedWord, "thesis", 62, 68),
        (SurfaceTokenKind::ReservedWord, "by", 69, 71),
        (SurfaceTokenKind::Identifier, "A", 72, 73),
        (SurfaceTokenKind::ReservedSymbol, ";", 73, 74),
        (SurfaceTokenKind::ReservedWord, "end", 75, 78),
        (SurfaceTokenKind::ReservedSymbol, ";", 78, 79),
        (SurfaceTokenKind::ReservedWord, "noise", 80, 85),
        (SurfaceTokenKind::ReservedWord, "end", 86, 89),
        (SurfaceTokenKind::ReservedSymbol, ";", 89, 90),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            15,
            21,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 15, 21, &[22]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            31,
            37,
            &[7],
        ),
        node(SurfaceNodeKind::FormulaExpression, 31, 37, &[24]),
        node(SurfaceNodeKind::Proposition, 28, 37, &[5, 6, 25]),
        node(SurfaceNodeKind::CompactStatement, 28, 38, &[26, 8]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            44,
            50,
            &[10],
        ),
        node(SurfaceNodeKind::FormulaExpression, 44, 50, &[28]),
        node(SurfaceNodeKind::Proposition, 44, 50, &[29]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            62,
            68,
            &[13],
        ),
        node(SurfaceNodeKind::FormulaExpression, 62, 68, &[31]),
        node(SurfaceNodeKind::Proposition, 62, 68, &[32]),
        node(SurfaceNodeKind::Reference, 72, 73, &[15]),
        node(SurfaceNodeKind::ReferenceList, 72, 73, &[34]),
        node(SurfaceNodeKind::JustificationClause, 69, 73, &[14, 35]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            57,
            74,
            &[12, 33, 36, 16],
        ),
        node(SurfaceNodeKind::ProofBlock, 51, 78, &[11, 37, 17]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            39,
            79,
            &[9, 30, 38, 18],
        ),
        node(SurfaceNodeKind::ProofBlock, 22, 89, &[4, 27, 19, 39, 20]),
        node(SurfaceNodeKind::TheoremItem, 0, 90, &[0, 1, 2, 23, 40, 21]),
        node(SurfaceNodeKind::ItemList, 0, 90, &[41]),
        node(SurfaceNodeKind::CompilationUnit, 0, 90, &[42]),
        node(
            SurfaceNodeKind::Root,
            0,
            90,
            &(0..=21).chain(std::iter::once(43)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn mixed_reference_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (SurfaceTokenKind::Identifier, "Owner", 8, 13),
        (SurfaceTokenKind::ReservedSymbol, ":", 13, 14),
        (SurfaceTokenKind::ReservedWord, "thesis", 15, 21),
        (SurfaceTokenKind::ReservedWord, "proof", 22, 27),
        (SurfaceTokenKind::ReservedWord, "thus", 28, 32),
        (SurfaceTokenKind::ReservedWord, "thesis", 33, 39),
        (SurfaceTokenKind::ReservedWord, "by", 40, 42),
        (SurfaceTokenKind::Identifier, "A", 43, 44),
        (SurfaceTokenKind::ReservedSymbol, ",", 44, 45),
        (SurfaceTokenKind::Identifier, "Q", 46, 47),
        (SurfaceTokenKind::ReservedSymbol, ",", 47, 48),
        (SurfaceTokenKind::Identifier, "G", 49, 50),
        (SurfaceTokenKind::ReservedSymbol, ",", 50, 51),
        (SurfaceTokenKind::Identifier, "Bulk", 52, 56),
        (SurfaceTokenKind::ReservedSymbol, ",", 56, 57),
        (SurfaceTokenKind::Identifier, "Templated", 58, 67),
        (SurfaceTokenKind::Identifier, "Arg", 68, 71),
        (SurfaceTokenKind::ReservedSymbol, ",", 71, 72),
        (SurfaceTokenKind::Identifier, "B", 73, 74),
        (SurfaceTokenKind::ReservedSymbol, ";", 74, 75),
        (SurfaceTokenKind::ReservedWord, "noise", 76, 81),
        (SurfaceTokenKind::ReservedWord, "end", 82, 85),
        (SurfaceTokenKind::ReservedSymbol, ";", 85, 86),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            15,
            21,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 15, 21, &[24]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            33,
            39,
            &[6],
        ),
        node(SurfaceNodeKind::FormulaExpression, 33, 39, &[26]),
        node(SurfaceNodeKind::Proposition, 33, 39, &[27]),
        node(SurfaceNodeKind::Reference, 43, 44, &[8]),
        node(SurfaceNodeKind::QualifiedReference, 46, 47, &[10]),
        node(SurfaceNodeKind::GroupedReference, 49, 50, &[12]),
        node(SurfaceNodeKind::BulkReference, 52, 56, &[14]),
        node(SurfaceNodeKind::TemplateArgument, 68, 71, &[17]),
        node(SurfaceNodeKind::TemplateArguments, 68, 71, &[33]),
        node(SurfaceNodeKind::Reference, 58, 71, &[16, 34]),
        node(SurfaceNodeKind::Reference, 73, 74, &[19]),
        node(
            SurfaceNodeKind::ReferenceList,
            43,
            74,
            &[29, 9, 30, 11, 31, 13, 32, 15, 35, 18, 36],
        ),
        node(SurfaceNodeKind::JustificationClause, 40, 74, &[7, 37]),
        node(SurfaceNodeKind::CompactStatement, 28, 75, &[5, 28, 38, 20]),
        node(SurfaceNodeKind::NowStatement, 76, 81, &[]),
        node(SurfaceNodeKind::ProofBlock, 22, 85, &[4, 39, 21, 40, 22]),
        node(SurfaceNodeKind::TheoremItem, 0, 86, &[0, 1, 2, 25, 41, 23]),
        node(SurfaceNodeKind::ItemList, 0, 86, &[42]),
        node(SurfaceNodeKind::CompilationUnit, 0, 86, &[43]),
        node(
            SurfaceNodeKind::Root,
            0,
            86,
            &(0..=23).chain(std::iter::once(44)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn cross_theorem_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (SurfaceTokenKind::Identifier, "Same", 8, 12),
        (SurfaceTokenKind::ReservedSymbol, ":", 12, 13),
        (SurfaceTokenKind::ReservedWord, "thesis", 14, 20),
        (SurfaceTokenKind::ReservedWord, "proof", 21, 26),
        (SurfaceTokenKind::Identifier, "A", 27, 28),
        (SurfaceTokenKind::ReservedSymbol, ":", 28, 29),
        (SurfaceTokenKind::ReservedWord, "thesis", 30, 36),
        (SurfaceTokenKind::ReservedSymbol, ";", 36, 37),
        (SurfaceTokenKind::ReservedWord, "end", 38, 41),
        (SurfaceTokenKind::ReservedSymbol, ";", 41, 42),
        (SurfaceTokenKind::ReservedWord, "theorem", 43, 50),
        (SurfaceTokenKind::Identifier, "Same", 51, 55),
        (SurfaceTokenKind::ReservedSymbol, ":", 55, 56),
        (SurfaceTokenKind::ReservedWord, "thesis", 57, 63),
        (SurfaceTokenKind::ReservedWord, "proof", 64, 69),
        (SurfaceTokenKind::ReservedWord, "thus", 70, 74),
        (SurfaceTokenKind::ReservedWord, "thesis", 75, 81),
        (SurfaceTokenKind::ReservedWord, "by", 82, 84),
        (SurfaceTokenKind::Identifier, "A", 85, 86),
        (SurfaceTokenKind::ReservedSymbol, ";", 86, 87),
        (SurfaceTokenKind::Identifier, "A", 88, 89),
        (SurfaceTokenKind::ReservedSymbol, ":", 89, 90),
        (SurfaceTokenKind::ReservedWord, "thesis", 91, 97),
        (SurfaceTokenKind::ReservedSymbol, ";", 97, 98),
        (SurfaceTokenKind::ReservedWord, "end", 99, 102),
        (SurfaceTokenKind::ReservedSymbol, ";", 102, 103),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            14,
            20,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 14, 20, &[27]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            30,
            36,
            &[7],
        ),
        node(SurfaceNodeKind::FormulaExpression, 30, 36, &[29]),
        node(SurfaceNodeKind::Proposition, 27, 36, &[5, 6, 30]),
        node(SurfaceNodeKind::CompactStatement, 27, 37, &[31, 8]),
        node(SurfaceNodeKind::ProofBlock, 21, 41, &[4, 32, 9]),
        node(SurfaceNodeKind::TheoremItem, 0, 42, &[0, 1, 2, 28, 33, 10]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            57,
            63,
            &[14],
        ),
        node(SurfaceNodeKind::FormulaExpression, 57, 63, &[35]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            75,
            81,
            &[17],
        ),
        node(SurfaceNodeKind::FormulaExpression, 75, 81, &[37]),
        node(SurfaceNodeKind::Proposition, 75, 81, &[38]),
        node(SurfaceNodeKind::Reference, 85, 86, &[19]),
        node(SurfaceNodeKind::ReferenceList, 85, 86, &[40]),
        node(SurfaceNodeKind::JustificationClause, 82, 86, &[18, 41]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            70,
            87,
            &[16, 39, 42, 20],
        ),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            91,
            97,
            &[23],
        ),
        node(SurfaceNodeKind::FormulaExpression, 91, 97, &[44]),
        node(SurfaceNodeKind::Proposition, 88, 97, &[21, 22, 45]),
        node(SurfaceNodeKind::CompactStatement, 88, 98, &[46, 24]),
        node(SurfaceNodeKind::ProofBlock, 64, 102, &[15, 43, 47, 25]),
        node(
            SurfaceNodeKind::TheoremItem,
            43,
            103,
            &[11, 12, 13, 36, 48, 26],
        ),
        node(SurfaceNodeKind::ItemList, 0, 103, &[34, 49]),
        node(SurfaceNodeKind::CompilationUnit, 0, 103, &[50]),
        node(
            SurfaceNodeKind::Root,
            0,
            103,
            &(0..=26).chain(std::iter::once(51)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

#[derive(Clone, Copy)]
enum UpperShape {
    MissingCompilation,
    AdditionalCompilation,
    WrongCompilationChild,
    RootWrongStructuralChild,
    TheoremDirectlyUnderRoot,
    MissingItemList,
    AdditionalItemList,
}

fn malformed_upper_ast(source_id: SourceId, shape: UpperShape) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 0, 7),
    );
    let root = match shape {
        UpperShape::MissingCompilation => {
            builder.add_node(SurfaceNodeKind::Root, range(source_id, 0, 7), vec![token])
        }
        UpperShape::AdditionalCompilation => {
            let first_list =
                builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let first = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![first_list],
            );
            let second_list =
                builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let second = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![second_list],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, first, second],
            )
        }
        UpperShape::WrongCompilationChild => {
            let theorem =
                builder.add_node(SurfaceNodeKind::TheoremItem, range(source_id, 0, 7), vec![]);
            let compilation = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![theorem],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, compilation],
            )
        }
        UpperShape::RootWrongStructuralChild => {
            let item_list =
                builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let compilation = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![item_list],
            );
            let wrong = builder.add_node(
                SurfaceNodeKind::PlaceholderItem,
                range(source_id, 0, 7),
                vec![],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, compilation, wrong],
            )
        }
        UpperShape::TheoremDirectlyUnderRoot => {
            let theorem =
                builder.add_node(SurfaceNodeKind::TheoremItem, range(source_id, 0, 7), vec![]);
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, theorem],
            )
        }
        UpperShape::MissingItemList => {
            let compilation = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, compilation],
            )
        }
        UpperShape::AdditionalItemList => {
            let first = builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let second =
                builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let compilation = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![first, second],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, compilation],
            )
        }
    };
    builder.finish(Some(root), None)
}

fn recovered_proof_boundary_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let theorem = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 0, 7),
    );
    let owner = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Owner",
        range(source_id, 8, 13),
    );
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 13, 14),
    );
    let thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 15, 21),
    );
    let proof = builder.add_recovered_token(
        SurfaceTokenKind::ReservedWord,
        "proof",
        range(source_id, 22, 27),
    );
    let label = builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 28, 29));
    let label_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 29, 30),
    );
    let statement_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 31, 37),
    );
    let statement_end = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 37, 38),
    );
    let end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 39, 42),
    );
    let theorem_end = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 42, 43),
    );
    let theorem_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 15, 21),
        vec![thesis],
    );
    let theorem_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 15, 21),
        vec![theorem_constant],
    );
    let statement_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 31, 37),
        vec![statement_thesis],
    );
    let statement_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 31, 37),
        vec![statement_constant],
    );
    let proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 28, 37),
        vec![label, label_colon, statement_formula],
    );
    let statement = builder.add_node(
        SurfaceNodeKind::CompactStatement,
        range(source_id, 28, 38),
        vec![proposition, statement_end],
    );
    let proof_block = builder.add_node(
        SurfaceNodeKind::ProofBlock,
        range(source_id, 22, 42),
        vec![proof, statement, end],
    );
    let theorem_item = builder.add_node(
        SurfaceNodeKind::TheoremItem,
        range(source_id, 0, 43),
        vec![
            theorem,
            owner,
            colon,
            theorem_formula,
            proof_block,
            theorem_end,
        ],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 43),
        vec![theorem_item],
    );
    let compilation = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 43),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 43),
        vec![
            theorem,
            owner,
            colon,
            thesis,
            proof,
            label,
            label_colon,
            statement_thesis,
            statement_end,
            end,
            theorem_end,
            compilation,
        ],
    );
    builder.finish(Some(root), None)
}

struct TestNode {
    kind: SurfaceNodeKind,
    start: usize,
    end: usize,
    children: Vec<usize>,
}

fn node(kind: SurfaceNodeKind, start: usize, end: usize, children: &[usize]) -> TestNode {
    TestNode {
        kind,
        start,
        end,
        children: children.to_vec(),
    }
}

fn build_surface_ast(
    source_id: SourceId,
    tokens: Vec<(SurfaceTokenKind, &'static str, usize, usize)>,
    nodes: Vec<TestNode>,
) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let mut ids = Vec::new();
    for (kind, text, start, end) in tokens {
        ids.push(builder.add_token(kind, text, range(source_id, start, end)));
    }
    for node in nodes {
        let children = node.children.iter().map(|index| ids[*index]).collect();
        ids.push(builder.add_node(node.kind, range(source_id, node.start, node.end), children));
    }
    builder.finish(ids.last().copied(), None)
}

struct CollectorAstBuilder {
    source_id: SourceId,
    builder: SurfaceAstBuilder,
    base: usize,
    cursor: usize,
}

impl CollectorAstBuilder {
    fn new(source_id: SourceId) -> Self {
        Self::with_offset(source_id, 0)
    }

    fn with_offset(source_id: SourceId, offset: usize) -> Self {
        Self {
            source_id,
            builder: SurfaceAstBuilder::new(source_id),
            base: offset,
            cursor: offset,
        }
    }

    fn token(&mut self, kind: SurfaceTokenKind, text: &str) -> SurfaceBuilderNodeId {
        let start = self.cursor;
        let width = text.len().max(1);
        self.cursor += width + 1;
        self.builder
            .add_token(kind, text, range(self.source_id, start, start + width))
    }

    fn recovered_token(&mut self, kind: SurfaceTokenKind, text: &str) -> SurfaceBuilderNodeId {
        let start = self.cursor;
        let width = text.len().max(1);
        self.cursor += width + 1;
        self.builder
            .add_recovered_token(kind, text, range(self.source_id, start, start + width))
    }

    fn node(
        &mut self,
        kind: SurfaceNodeKind,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        self.builder.add_node(
            kind,
            range(self.source_id, self.base, self.cursor.max(self.base + 1)),
            children,
        )
    }

    fn formula(&mut self) -> SurfaceBuilderNodeId {
        let thesis = self.token(SurfaceTokenKind::ReservedWord, "thesis");
        let constant = self.node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            vec![thesis],
        );
        self.node(SurfaceNodeKind::FormulaExpression, vec![constant])
    }

    fn proposition(
        &mut self,
        label: Option<&str>,
        colon: Option<&str>,
        include_semantic_child: bool,
    ) -> SurfaceBuilderNodeId {
        let mut children = Vec::new();
        if let Some(label) = label {
            children.push(self.token(SurfaceTokenKind::Identifier, label));
        }
        if let Some(colon) = colon {
            children.push(self.token(SurfaceTokenKind::ReservedSymbol, colon));
        }
        if include_semantic_child {
            let formula = self.formula();
            children.push(formula);
        }
        self.node(SurfaceNodeKind::Proposition, children)
    }

    fn reference(
        &mut self,
        spelling: &str,
        additional_child: bool,
        recovered_identifier: bool,
    ) -> SurfaceBuilderNodeId {
        let identifier = if recovered_identifier {
            self.recovered_token(SurfaceTokenKind::Identifier, spelling)
        } else {
            self.token(SurfaceTokenKind::Identifier, spelling)
        };
        let mut children = vec![identifier];
        if additional_child {
            let arguments = self.node(SurfaceNodeKind::TemplateArguments, Vec::new());
            children.push(arguments);
        }
        self.node(SurfaceNodeKind::Reference, children)
    }

    fn justification(
        &mut self,
        first: (&str, SurfaceTokenKind),
        reference_children: Vec<SurfaceBuilderNodeId>,
        additional_list: bool,
    ) -> SurfaceBuilderNodeId {
        let first = self.token(first.1, first.0);
        let list = self.node(SurfaceNodeKind::ReferenceList, reference_children);
        let mut children = vec![first, list];
        if additional_list {
            let second = self.node(SurfaceNodeKind::ReferenceList, Vec::new());
            children.push(second);
        }
        self.node(SurfaceNodeKind::JustificationClause, children)
    }

    fn compact(
        &mut self,
        proposition: SurfaceBuilderNodeId,
        justification: Option<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let mut children = vec![proposition];
        if let Some(justification) = justification {
            children.push(justification);
        }
        let semicolon = self.token(SurfaceTokenKind::ReservedSymbol, ";");
        children.push(semicolon);
        self.node(SurfaceNodeKind::CompactStatement, children)
    }

    fn conclusion(
        &mut self,
        proposition: SurfaceBuilderNodeId,
        justification: Option<SurfaceBuilderNodeId>,
        proof: Option<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let thus = self.token(SurfaceTokenKind::ReservedWord, "thus");
        let mut children = vec![thus, proposition];
        if let Some(justification) = justification {
            children.push(justification);
        }
        if let Some(proof) = proof {
            children.push(proof);
        }
        let semicolon = self.token(SurfaceTokenKind::ReservedSymbol, ";");
        children.push(semicolon);
        self.node(SurfaceNodeKind::ConclusionStatement, children)
    }

    fn proof(
        &mut self,
        statements: Vec<SurfaceBuilderNodeId>,
        first: (&str, bool),
        last: (&str, bool),
    ) -> SurfaceBuilderNodeId {
        let first = if first.1 {
            self.recovered_token(SurfaceTokenKind::ReservedWord, first.0)
        } else {
            self.token(SurfaceTokenKind::ReservedWord, first.0)
        };
        let last = if last.1 {
            self.recovered_token(SurfaceTokenKind::ReservedWord, last.0)
        } else {
            self.token(SurfaceTokenKind::ReservedWord, last.0)
        };
        let mut children = vec![first];
        children.extend(statements);
        children.push(last);
        self.node(SurfaceNodeKind::ProofBlock, children)
    }

    fn theorem(
        &mut self,
        owner: &str,
        proof: Option<SurfaceBuilderNodeId>,
        role: &str,
        colon: &str,
        additional_proof: Option<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let role = self.token(SurfaceTokenKind::ReservedWord, role);
        let owner = self.token(SurfaceTokenKind::Identifier, owner);
        let colon = self.token(SurfaceTokenKind::ReservedSymbol, colon);
        let formula = self.formula();
        let mut children = vec![role, owner, colon, formula];
        if let Some(proof) = proof {
            children.push(proof);
        }
        if let Some(proof) = additional_proof {
            children.push(proof);
        }
        let semicolon = self.token(SurfaceTokenKind::ReservedSymbol, ";");
        children.push(semicolon);
        self.node(SurfaceNodeKind::TheoremItem, children)
    }

    fn valid_statement(
        &mut self,
        label: &str,
        reference: Option<&str>,
        include_semantic_child: bool,
    ) -> SurfaceBuilderNodeId {
        let proposition = self.proposition(Some(label), Some(":"), include_semantic_child);
        let justification = reference.map(|reference| {
            let reference = self.reference(reference, false, false);
            self.justification(
                ("by", SurfaceTokenKind::ReservedWord),
                vec![reference],
                false,
            )
        });
        self.compact(proposition, justification)
    }

    fn valid_theorem(
        &mut self,
        owner: &str,
        statements: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let proof = self.proof(statements, ("proof", false), ("end", false));
        self.theorem(owner, Some(proof), "theorem", ":", None)
    }

    fn finish_items(mut self, items: Vec<SurfaceBuilderNodeId>) -> SurfaceAst {
        let item_list = self.node(SurfaceNodeKind::ItemList, items);
        let compilation = self.node(SurfaceNodeKind::CompilationUnit, vec![item_list]);
        let mut root_children = self.builder.token_node_ids().to_vec();
        root_children.push(compilation);
        let root = self.node(SurfaceNodeKind::Root, root_children);
        self.builder.finish(Some(root), None)
    }
}

fn collect_fixture(ast: &SurfaceAst) -> ProofLabelSourceCollection {
    let module = module_id("pkg", "main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), ast.source_id, 0);
    let resolved = SurfaceResolvedArena::lower(ast, &module).unwrap();
    ProofLabelSourceCollector::new(
        ast,
        &module,
        NamespacePath::new("main"),
        contribution,
        &resolved,
    )
    .unwrap()
    .collect()
    .unwrap()
}

fn collect_proof_fixture(ast: &SurfaceAst) -> ProofLabelSourceCollection {
    let module = module_id("pkg", "main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), ast.source_id, 0);
    let resolved = SurfaceResolvedArena::lower(ast, &module).unwrap();
    ProofLabelSourceCollector::new(
        ast,
        &module,
        NamespacePath::new("main"),
        contribution,
        &resolved,
    )
    .unwrap()
    .collect_with_proof_organization()
    .unwrap()
}

fn two_theorem_identity_ast(
    source_id: SourceId,
    offset: usize,
    first_owner: &str,
    second_owner: &str,
    reverse_items: bool,
) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::with_offset(source_id, offset);
    let first_statement = builder.valid_statement("FirstLabel", Some("FirstRef"), false);
    let first = builder.valid_theorem(first_owner, vec![first_statement]);
    let second_statement = builder.valid_statement("SecondLabel", Some("SecondRef"), false);
    let second = builder.valid_theorem(second_owner, vec![second_statement]);
    let items = if reverse_items {
        vec![second, first]
    } else {
        vec![first, second]
    };
    builder.finish_items(items)
}

fn label_topology_ast(source_id: SourceId, nested: bool) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let labelled = builder.valid_statement("Topology", None, false);
    let statements = if nested {
        let child_proof = builder.proof(vec![labelled], ("proof", false), ("end", false));
        let proposition = builder.proposition(None, None, true);
        let owner = builder.conclusion(proposition, None, Some(child_proof));
        vec![owner]
    } else {
        vec![labelled]
    };
    let theorem = builder.valid_theorem("Owner", statements);
    builder.finish_items(vec![theorem])
}

fn same_scope_labels_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let first = builder.valid_statement("Repeated", None, false);
    let second = builder.valid_statement("Repeated", None, false);
    let theorem = builder.valid_theorem("Owner", vec![first, second]);
    builder.finish_items(vec![theorem])
}

fn relocated_theorem_ast(source_id: SourceId, directly_under_root: bool) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let theorem = builder.valid_theorem("HiddenOwner", vec![statement]);
    let mut root_children = builder.builder.token_node_ids().to_vec();
    if directly_under_root {
        root_children.push(theorem);
    } else {
        let compilation = builder.node(SurfaceNodeKind::CompilationUnit, vec![theorem]);
        root_children.push(compilation);
    }
    let root = builder.node(SurfaceNodeKind::Root, root_children);
    builder.builder.finish(Some(root), None)
}

fn unsupported_item_wrapper_ast(source_id: SourceId, kind: SurfaceNodeKind) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let hidden_theorem = builder.valid_theorem("HiddenOwner", vec![hidden_statement]);
    let unsupported = builder.node(kind, vec![hidden_theorem]);
    let sentinel_statement = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let sentinel_theorem = builder.valid_theorem("SentinelOwner", vec![sentinel_statement]);
    builder.finish_items(vec![unsupported, sentinel_theorem])
}

fn recovered_item_wrapper_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let hidden_theorem = builder.valid_theorem("HiddenOwner", vec![hidden_statement]);
    let recovered = builder.builder.add_recovery(
        SyntaxRecoveryKind::MissingItem,
        range(source_id, 0, 1),
        vec![hidden_theorem],
    );
    let sentinel_statement = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let sentinel_theorem = builder.valid_theorem("SentinelOwner", vec![sentinel_statement]);
    builder.finish_items(vec![recovered, sentinel_theorem])
}

fn unsupported_proof_child_ast(source_id: SourceId, kind: SurfaceNodeKind) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let hidden_proof = builder.proof(vec![hidden_statement], ("proof", false), ("end", false));
    let unsupported = builder.node(kind, vec![hidden_proof]);
    let sentinel = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let theorem = builder.valid_theorem("Owner", vec![unsupported, sentinel]);
    builder.finish_items(vec![theorem])
}

#[derive(Clone, Copy, Debug)]
enum TheoremShapeMutation {
    WrongRole,
    WrongOwnerKind,
    WrongColon,
    MissingProof,
    AdditionalProof,
    WrappedProof,
    RecoveredRole,
    RecoveredOwner,
    RecoveredColon,
}

fn malformed_theorem_owner_ast(source_id: SourceId, mutation: TheoremShapeMutation) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let role = if matches!(mutation, TheoremShapeMutation::RecoveredRole) {
        builder.recovered_token(SurfaceTokenKind::ReservedWord, "theorem")
    } else {
        builder.token(
            SurfaceTokenKind::ReservedWord,
            if matches!(mutation, TheoremShapeMutation::WrongRole) {
                "lemma"
            } else {
                "theorem"
            },
        )
    };
    let owner = if matches!(mutation, TheoremShapeMutation::RecoveredOwner) {
        builder.recovered_token(SurfaceTokenKind::Identifier, "BadOwner")
    } else {
        builder.token(
            if matches!(mutation, TheoremShapeMutation::WrongOwnerKind) {
                SurfaceTokenKind::ReservedWord
            } else {
                SurfaceTokenKind::Identifier
            },
            "BadOwner",
        )
    };
    let colon = if matches!(mutation, TheoremShapeMutation::RecoveredColon) {
        builder.recovered_token(SurfaceTokenKind::ReservedSymbol, ":")
    } else {
        builder.token(
            SurfaceTokenKind::ReservedSymbol,
            if matches!(mutation, TheoremShapeMutation::WrongColon) {
                ";"
            } else {
                ":"
            },
        )
    };
    let formula = builder.formula();
    let hidden = builder.valid_statement("Hidden", Some("Hidden"), false);
    let proof = builder.proof(vec![hidden], ("proof", false), ("end", false));
    let mut children = vec![role, owner, colon, formula];
    match mutation {
        TheoremShapeMutation::MissingProof => {}
        TheoremShapeMutation::AdditionalProof => {
            children.push(proof);
            let second = builder.proof(Vec::new(), ("proof", false), ("end", false));
            children.push(second);
        }
        TheoremShapeMutation::WrappedProof => {
            let wrapper = builder.node(SurfaceNodeKind::VisibleItem, vec![proof]);
            children.push(wrapper);
        }
        _ => children.push(proof),
    }
    let end = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
    children.push(end);
    let malformed = builder.node(SurfaceNodeKind::TheoremItem, children);

    let sentinel_statement = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let sentinel = builder.valid_theorem("SentinelOwner", vec![sentinel_statement]);
    builder.finish_items(vec![malformed, sentinel])
}

#[derive(Clone, Copy, Debug)]
enum ProofBoundaryMutation {
    MissingEnd,
    WrongStart,
    WrongEnd,
    RecoveredStart,
    RecoveredEnd,
    RecoveredInterior,
}

fn malformed_proof_boundary_ast(
    source_id: SourceId,
    mutation: ProofBoundaryMutation,
) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden = builder.valid_statement("Hidden", Some("Hidden"), false);
    let proof = match mutation {
        ProofBoundaryMutation::MissingEnd => {
            let start = builder.token(SurfaceTokenKind::ReservedWord, "proof");
            builder.node(SurfaceNodeKind::ProofBlock, vec![start, hidden])
        }
        ProofBoundaryMutation::WrongStart => {
            builder.proof(vec![hidden], ("begin", false), ("end", false))
        }
        ProofBoundaryMutation::WrongEnd => {
            builder.proof(vec![hidden], ("proof", false), ("stop", false))
        }
        ProofBoundaryMutation::RecoveredStart => {
            builder.proof(vec![hidden], ("proof", true), ("end", false))
        }
        ProofBoundaryMutation::RecoveredEnd => {
            builder.proof(vec![hidden], ("proof", false), ("end", true))
        }
        ProofBoundaryMutation::RecoveredInterior => {
            let start = builder.token(SurfaceTokenKind::ReservedWord, "proof");
            let recovery = builder.recovered_token(SurfaceTokenKind::ErrorRecovery, "?");
            let end = builder.token(SurfaceTokenKind::ReservedWord, "end");
            builder.node(
                SurfaceNodeKind::ProofBlock,
                vec![start, hidden, recovery, end],
            )
        }
    };
    let malformed = builder.theorem("BadOwner", Some(proof), "theorem", ":", None);
    let sentinel_statement = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let sentinel = builder.valid_theorem("SentinelOwner", vec![sentinel_statement]);
    builder.finish_items(vec![malformed, sentinel])
}

fn finish_statement_mutation(
    mut builder: CollectorAstBuilder,
    mutated: SurfaceBuilderNodeId,
) -> SurfaceAst {
    let sentinel = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let theorem = builder.valid_theorem("Owner", vec![mutated, sentinel]);
    builder.finish_items(vec![theorem])
}

#[derive(Clone, Copy, Debug)]
enum CompactLabelMutation {
    MissingIdentifier,
    WrongIdentifierKind,
    MissingColon,
    WrongColon,
    RecoveredIdentifier,
    RecoveredColon,
    AdditionalProposition,
}

fn malformed_compact_label_ast(source_id: SourceId, mutation: CompactLabelMutation) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let identifier = match mutation {
        CompactLabelMutation::MissingIdentifier => None,
        CompactLabelMutation::WrongIdentifierKind => {
            Some(builder.token(SurfaceTokenKind::ReservedWord, "MutLabel"))
        }
        CompactLabelMutation::RecoveredIdentifier => {
            Some(builder.recovered_token(SurfaceTokenKind::Identifier, "MutLabel"))
        }
        _ => Some(builder.token(SurfaceTokenKind::Identifier, "MutLabel")),
    };
    let colon = match mutation {
        CompactLabelMutation::MissingColon => None,
        CompactLabelMutation::WrongColon => {
            Some(builder.token(SurfaceTokenKind::ReservedSymbol, ";"))
        }
        CompactLabelMutation::RecoveredColon => {
            Some(builder.recovered_token(SurfaceTokenKind::ReservedSymbol, ":"))
        }
        _ => Some(builder.token(SurfaceTokenKind::ReservedSymbol, ":")),
    };
    let formula = builder.formula();
    let mut proposition_children = Vec::new();
    if let Some(identifier) = identifier {
        proposition_children.push(identifier);
    }
    if let Some(colon) = colon {
        proposition_children.push(colon);
    }
    proposition_children.push(formula);
    let proposition = builder.node(SurfaceNodeKind::Proposition, proposition_children);
    let reference = builder.reference("MutRef", false, false);
    let justification = builder.justification(
        ("by", SurfaceTokenKind::ReservedWord),
        vec![reference],
        false,
    );
    let mutated = if matches!(mutation, CompactLabelMutation::AdditionalProposition) {
        let additional = builder.proposition(Some("Other"), Some(":"), false);
        let semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
        builder.node(
            SurfaceNodeKind::CompactStatement,
            vec![proposition, additional, justification, semicolon],
        )
    } else {
        builder.compact(proposition, Some(justification))
    };
    finish_statement_mutation(builder, mutated)
}

#[derive(Clone, Copy, Debug)]
enum ReferenceMutation {
    MissingIdentifier,
    WrongIdentifierKind,
    AdditionalChild,
    RecoveredIdentifier,
}

fn malformed_reference_ast(source_id: SourceId, mutation: ReferenceMutation) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let proposition = builder.proposition(Some("MutLabel"), Some(":"), false);
    let reference = match mutation {
        ReferenceMutation::MissingIdentifier => {
            builder.node(SurfaceNodeKind::Reference, Vec::new())
        }
        ReferenceMutation::WrongIdentifierKind => {
            let child = builder.token(SurfaceTokenKind::ReservedWord, "MutRef");
            builder.node(SurfaceNodeKind::Reference, vec![child])
        }
        ReferenceMutation::AdditionalChild => builder.reference("MutRef", true, false),
        ReferenceMutation::RecoveredIdentifier => builder.reference("MutRef", false, true),
    };
    let justification = builder.justification(
        ("by", SurfaceTokenKind::ReservedWord),
        vec![reference],
        false,
    );
    let mutated = builder.compact(proposition, Some(justification));
    finish_statement_mutation(builder, mutated)
}

#[derive(Clone, Copy, Debug)]
enum JustificationMutation {
    WrongFirstToken,
    RecoveredFirstToken,
    AdditionalReferenceList,
    ComputationInstead,
    WrappedJustification,
    WrappedReferenceList,
    RelocatedIntoProposition,
    ProofRelocatedIntoProposition,
    ConclusionLabel,
}

fn malformed_justification_edge_ast(
    source_id: SourceId,
    mutation: JustificationMutation,
) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let reference = builder.reference("MutRef", false, false);
    let justification = match mutation {
        JustificationMutation::WrongFirstToken => builder.justification(
            ("from", SurfaceTokenKind::ReservedWord),
            vec![reference],
            false,
        ),
        JustificationMutation::RecoveredFirstToken => {
            let by = builder.recovered_token(SurfaceTokenKind::ReservedWord, "by");
            let list = builder.node(SurfaceNodeKind::ReferenceList, vec![reference]);
            builder.node(SurfaceNodeKind::JustificationClause, vec![by, list])
        }
        JustificationMutation::AdditionalReferenceList => builder.justification(
            ("by", SurfaceTokenKind::ReservedWord),
            vec![reference],
            true,
        ),
        JustificationMutation::ComputationInstead => {
            let by = builder.token(SurfaceTokenKind::ReservedWord, "by");
            let list = builder.node(SurfaceNodeKind::ReferenceList, vec![reference]);
            builder.node(SurfaceNodeKind::ComputationJustification, vec![by, list])
        }
        JustificationMutation::WrappedReferenceList => {
            let by = builder.token(SurfaceTokenKind::ReservedWord, "by");
            let list = builder.node(SurfaceNodeKind::ReferenceList, vec![reference]);
            let wrapper = builder.node(SurfaceNodeKind::VisibleItem, vec![list]);
            builder.node(SurfaceNodeKind::JustificationClause, vec![by, wrapper])
        }
        _ => builder.justification(
            ("by", SurfaceTokenKind::ReservedWord),
            vec![reference],
            false,
        ),
    };

    let mutated = match mutation {
        JustificationMutation::WrappedJustification => {
            let proposition = builder.proposition(Some("MutLabel"), Some(":"), false);
            let wrapper = builder.node(SurfaceNodeKind::VisibleItem, vec![justification]);
            let semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
            builder.node(
                SurfaceNodeKind::CompactStatement,
                vec![proposition, wrapper, semicolon],
            )
        }
        JustificationMutation::RelocatedIntoProposition => {
            let label = builder.token(SurfaceTokenKind::Identifier, "MutLabel");
            let colon = builder.token(SurfaceTokenKind::ReservedSymbol, ":");
            let proposition = builder.node(
                SurfaceNodeKind::Proposition,
                vec![label, colon, justification],
            );
            builder.compact(proposition, None)
        }
        JustificationMutation::ProofRelocatedIntoProposition => {
            let hidden = builder.valid_statement("Hidden", Some("Hidden"), false);
            let hidden_proof = builder.proof(vec![hidden], ("proof", false), ("end", false));
            let label = builder.token(SurfaceTokenKind::Identifier, "MutLabel");
            let colon = builder.token(SurfaceTokenKind::ReservedSymbol, ":");
            let proposition = builder.node(
                SurfaceNodeKind::Proposition,
                vec![label, colon, hidden_proof],
            );
            builder.compact(proposition, Some(justification))
        }
        JustificationMutation::ConclusionLabel => {
            let proposition = builder.proposition(Some("MutLabel"), Some(":"), false);
            builder.conclusion(proposition, Some(justification), None)
        }
        _ => {
            let proposition = builder.proposition(Some("MutLabel"), Some(":"), false);
            builder.compact(proposition, Some(justification))
        }
    };
    finish_statement_mutation(builder, mutated)
}

fn current_theorem_projection(
    fixture: &ProjectionFixture,
    spelling: &str,
    start: usize,
    visible_after: usize,
) -> LabelProjection {
    LabelProjection::current_module(
        fixture.data(
            spelling,
            LabelKind::Theorem,
            "theorem",
            start,
            visible_after,
        ),
        visible_after,
    )
    .with_visibility(Visibility::Public)
    .with_export_status(ExportStatus::Exported)
}

fn imported_theorem_projection(
    fixture: &ProjectionFixture,
    spelling: &str,
    start: usize,
) -> LabelProjection {
    LabelProjection::imported(fixture.data(spelling, LabelKind::Theorem, "theorem", start, start))
}

fn proof_step_projection(
    fixture: &ProjectionFixture,
    spelling: &str,
    start: usize,
    visible_after: usize,
    scope: LabelScopePath,
) -> LabelProjection {
    LabelProjection::proof_step(
        fixture.data(
            spelling,
            LabelKind::ProofStep,
            "proof",
            start,
            visible_after,
        ),
        visible_after,
        scope,
    )
}

#[derive(Clone)]
struct ProjectionFixture {
    source_id: SourceId,
    module: ModuleId,
    namespace: NamespacePath,
    contribution: SourceContributionId,
}

impl ProjectionFixture {
    fn new(
        source_id: SourceId,
        module: ModuleId,
        namespace: NamespacePath,
        contribution: SourceContributionId,
    ) -> Self {
        Self {
            source_id,
            module,
            namespace,
            contribution,
        }
    }

    fn data(
        &self,
        spelling: &str,
        kind: LabelKind,
        origin_role: &str,
        start: usize,
        ordinal: usize,
    ) -> LabelProjectionData {
        let range = range(self.source_id, start, start + spelling.len());
        LabelProjectionData {
            origin_path: LabelOriginPath::new(format!(
                "{}::{}::{origin_role}::{spelling}",
                self.module.package().as_str(),
                self.module.path().as_str()
            )),
            module: self.module.clone(),
            namespace: self.namespace.clone(),
            primary_spelling: spelling.to_owned(),
            kind,
            declaration_range: range,
            origin: origin(self.source_id, self.module.clone(), range, ordinal),
            contribution: self.contribution,
        }
    }
}

fn unqualified_ref(
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
    spelling: &str,
    scope: Option<LabelScopePath>,
) -> LabelReferenceCandidate {
    let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
    LabelReferenceCandidate::unqualified_citation(site, origin, ordinal, scope)
}

fn qualified_ref(
    source_id: SourceId,
    current: ModuleId,
    target: ModuleId,
    namespace: NamespacePath,
    ordinal: usize,
    start: usize,
    spelling: &str,
) -> LabelReferenceCandidate {
    let (site, origin) = reference_site(source_id, current, start, spelling, ordinal);
    LabelReferenceCandidate::qualified_citation(site, origin, ordinal, target, namespace)
}

fn reference_site(
    source_id: SourceId,
    module: ModuleId,
    start: usize,
    spelling: &str,
    ordinal: usize,
) -> (ReferenceSite, SemanticOrigin) {
    let range = range(source_id, start, start + spelling.len());
    let origin = origin(source_id, module, range, ordinal);
    let mut arena = ResolvedArenaBuilder::new();
    let node = arena
        .push(ResolvedNode::new(
            SurfaceNodeKind::Reference,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    (ReferenceSite::new(node, range, spelling), origin)
}

fn origin(
    source_id: SourceId,
    module: ModuleId,
    range: SourceRange,
    ordinal: usize,
) -> SemanticOrigin {
    SemanticOrigin::new(
        source_id,
        module,
        SourceAnchor::Range(range),
        vec![ordinal as u32],
    )
}

fn contribution(
    index: &mut SourceContributionIndex,
    module: ModuleId,
    source_id: SourceId,
    start: usize,
) -> SourceContributionId {
    index.insert(
        module,
        ContributionKind::LocalSource { source_id },
        SourceAnchor::Range(range(source_id, start, start + 1)),
    )
}

fn assert_resolved_label(resolution: &LabelResolutionResult, index: usize, expected_origin: &str) {
    let entry = resolution.table().get(resolution.ids()[index]).unwrap();
    let LabelResolution::Resolved(label) = entry.resolution() else {
        panic!("expected resolved label at index {index}");
    };
    assert_eq!(label.origin().as_str(), expected_origin);
}

fn assert_unresolved_label(
    resolution: &LabelResolutionResult,
    index: usize,
    expected_expectation: LabelExpectation,
    expected_spelling: &str,
) {
    let entry = resolution.table().get(resolution.ids()[index]).unwrap();
    let LabelResolution::Unresolved(unresolved) = entry.resolution() else {
        panic!("expected unresolved label at index {index}");
    };
    assert_eq!(unresolved.expectation(), expected_expectation);
    assert_eq!(unresolved.spelling(), expected_spelling);
}

fn module_id(package: &str, path: &str) -> ModuleId {
    ModuleId::new(PackageId::new(package), ModulePath::new(path))
}

fn source_id() -> SourceId {
    let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "34".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    allocator.next_source_id(snapshot_id).unwrap()
}

fn distinct_source_ids() -> (SourceId, SourceId) {
    let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "35".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    (
        allocator.next_source_id(snapshot_id).unwrap(),
        allocator.next_source_id(snapshot_id).unwrap(),
    )
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

#[test]
fn source_public_theorem_label_candidates_project_exact_source_rows() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let alpha = "theorem Alpha: for X being set holds X = X\nproof\n  thus thesis;\nend;\n";
    for (text, expected) in [
        (fixture.to_owned(), vec![("UsePriv1", 102, 110)]),
        (fixture.replacen("private theorem", "public theorem", 1), vec![("PrivT1", 15, 21), ("UsePriv1", 101, 109)]),
        (
            fixture.replace("UsePriv1", "Renamed1"),
            vec![("Renamed1", 102, 110)],
        ),
        (
            fixture.replacen("thus X = X;", "Inner: X = X;\n  thus X = X by Inner;", 1),
            vec![("UsePriv1", 127, 135)],
        ),
        (
            fixture.replace("theorem UsePriv1", "lemma UsePriv1"),
            vec![("UsePriv1", 100, 108)],
        ),
        (
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
            )
            .to_owned(),
            vec![("AxT1", 67, 71), ("OpenT1", 13, 19)],
        ),
        (
            format!("{fixture}\n{alpha}"),
            vec![("Alpha", 200, 205), ("UsePriv1", 102, 110)],
        ),
        (
            format!("{alpha}\n{fixture}"),
            vec![("Alpha", 8, 13), ("UsePriv1", 172, 180)],
        ),
    ] {
        let (ast, module, namespace, env, resolved) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), resolved.clone(), env.clone());
        let rows = source_public_theorem_label_candidates(&ast, &resolved, &env)
            .unwrap()
            .unwrap();
        let expected_rows = expected
            .iter()
            .map(|(label, start_byte, end_byte)| {
                let owner = env
                    .symbols()
                    .iter()
                    .find(|entry| entry.primary_spelling() == *label)
                    .unwrap();
                assert_eq!(owner.visibility(), Visibility::Public);
                assert_eq!(owner.export_status(), ExportStatus::Exported);
                assert_eq!(owner.symbol().module(), &module);
                assert_eq!(owner.namespace(), &namespace);
                assert_ne!(
                    source_range_from_anchor(owner.origin().anchor()).unwrap(),
                    SourceRange { source_id: ast.source_id, start: *start_byte as usize, end: *end_byte as usize }
                );
                mizar_artifact::module_summary::ExportedLabelSummary {
                    origin_id: owner.symbol().fqn().as_str().to_owned(),
                    label: (*label).to_owned(),
                    owner_fully_qualified_name: owner.symbol().fqn().as_str().to_owned(),
                    visibility: "public".to_owned(),
                    source_range: mizar_artifact::module_summary::SourceRangeSummary {
                        start_byte: *start_byte,
                        end_byte: *end_byte,
                    },
                    target_kind: "theorem".to_owned(),
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(rows, expected_rows, "{text}");
        assert_eq!(
            source_public_theorem_label_candidates(&ast, &resolved, &env).unwrap(),
            Some(rows)
        );
        assert_eq!((ast, resolved, env), before);
    }
    let all_private = fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1");
    let (ast, _, _, env, resolved) = parsed_private_theorem_fixture(&all_private);
    assert_eq!(
        source_public_theorem_label_candidates(&ast, &resolved, &env).unwrap(),
        Some(Vec::new())
    );
}

#[test]
fn source_public_theorem_label_candidates_refuse_profiles_after_correspondence_checks() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for text in [
        fixture.replacen("private theorem", "public lemma", 1),
        format!("{fixture}\nreserve Y for set;\n"),
        format!("{fixture}\ntheorem Plain: 0 = 0;\n"),
        format!("{fixture}\ntheorem Direct: 0 = 0 by UsePriv1;\n"),
        format!("{fixture}\nopen theorem OpenDirect: 0 = 0 by UsePriv1;\n"),
        format!("{fixture}\nopen theorem OpenComp: 0 = 0 by computation(steps: 1);\n"),
        format!(
            "{fixture}\n{}",
            include_str!(
                "../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.miz"
            )
        ),
    ] {
        let (ast, _, _, env, resolved) = parsed_private_theorem_fixture(&text);
        assert!(env.symbols().iter().any(|entry| {
            entry.primary_spelling() == "UsePriv1"
                && entry.visibility() == Visibility::Public
                && entry.export_status() == ExportStatus::Exported
        }));
        assert_eq!(
            source_public_theorem_label_candidates(&ast, &resolved, &env).unwrap(),
            None,
            "{text}"
        );
    }
    let text = format!("{fixture}\ntheorem Plain: 0 = 0;\n");
    let (ast, module, namespace, env, resolved) = parsed_private_theorem_fixture(&text);
    let (foreign_ast, _, _, foreign_env, foreign_arena) =
        parsed_private_theorem_fixture(&text.replace("UsePriv1", "Changed1"));
    assert_eq!(
        source_public_theorem_label_candidates(&foreign_ast, &resolved, &env).unwrap_err(),
        resolved
            .validate_against(&foreign_ast, &module)
            .unwrap_err()
            .to_string()
    );
    assert_eq!(
        source_public_theorem_label_candidates(&ast, &foreign_arena, &env).unwrap_err(),
        foreign_arena
            .validate_against(&ast, &module)
            .unwrap_err()
            .to_string()
    );
    assert_eq!(
        source_public_theorem_label_candidates(&ast, &resolved, &foreign_env).unwrap_err(),
        crate::symbols::validate_source_symbol_env(&resolved, &foreign_env).unwrap_err()
    );
    let mut mutated_ast = ast.clone();
    mutated_ast.source_id = distinct_source_ids().1;
    assert_ne!(mutated_ast.source_id, ast.source_id);
    assert_eq!(
        source_public_theorem_label_candidates(&mutated_ast, &resolved, &env).unwrap_err(),
        resolved
            .validate_against(&mutated_ast, &module)
            .unwrap_err()
            .to_string()
    );
    let foreign_module_arena =
        SurfaceResolvedArena::lower(&ast, &module_id("pkg", "other")).unwrap();
    assert_eq!(
        source_public_theorem_label_candidates(&ast, &foreign_module_arena, &env).unwrap_err(),
        crate::symbols::validate_source_symbol_env(&foreign_module_arena, &env).unwrap_err()
    );
    let shells = crate::declarations::DeclarationShellCollector::new(&ast, &module).collect();
    let mutated_env = crate::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new(format!("{}.other", namespace.as_str())),
    )
    .collect(&module)
    .into_env();
    assert_eq!(
        source_public_theorem_label_candidates(&ast, &resolved, &mutated_env).unwrap_err(),
        crate::symbols::validate_source_symbol_env(&resolved, &mutated_env).unwrap_err()
    );
}

#[test]
fn source_public_theorem_formula_candidates_associate_original_written_nodes() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, expected) in [
        (fixture.to_owned(), vec![("UsePriv1", 112, 139)]),
        (fixture.replacen("private theorem", "public theorem", 1), vec![("PrivT1", 23, 50), ("UsePriv1", 111, 138)]),
        (
            fixture.replace("theorem UsePriv1", "lemma UsePriv1"),
            vec![("UsePriv1", 110, 137)],
        ),
        (
            fixture.replacen(
                "thus X = X by PrivT1;",
                "Inner: X = X;\n  thus X = X by Inner;",
                1,
            ),
            vec![("UsePriv1", 112, 139)],
        ),
        (
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
            )
            .to_owned(),
            vec![("OpenT1", 21, 48), ("AxT1", 73, 100)],
        ),
    ] {
        let (ast, _, _, env, resolved) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), resolved.clone(), env.clone());
        let mut expected_formulas = BTreeMap::new();
        for (label, start, end) in expected {
            let owner = env.symbols().iter().find(|entry| entry.primary_spelling() == label).unwrap();
            assert_eq!(owner.visibility(), Visibility::Public);
            assert_eq!(owner.export_status(), ExportStatus::Exported);
            let expected_range = range(ast.source_id, start, end);
            let nodes = ast.node_views().filter(|node| {
                node.as_formula_expression().is_some() && node.range() == expected_range
            }).collect::<Vec<_>>();
            let [formula] = nodes.as_slice() else { panic!("unique written formula for {label}") };
            assert_eq!(&text[start..end], "for X being set holds X = X");
            let item = ast.node_views().find(|item| {
                matches!(item.kind(), SurfaceNodeKind::TheoremItem | SurfaceNodeKind::LemmaItem)
                    && source_range_from_anchor(owner.origin().anchor()) == Some(item.range())
            }).unwrap();
            assert!(item.child_views().any(|child| child.id() == formula.id()));
            assert!(expected_formulas.insert(owner.symbol().clone(), formula.id()).is_none());
        }
        let candidates = source_public_theorem_formula_candidates(&ast, &resolved, &env).unwrap();
        assert_eq!(candidates, Some(expected_formulas));
        assert_eq!(source_public_theorem_formula_candidates(&ast, &resolved, &env).unwrap(), candidates);
        assert_eq!((ast, resolved, env), before);
    }
    let all_private = fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1");
    let (ast, _, _, env, resolved) = parsed_private_theorem_fixture(&all_private);
    assert_eq!(
        source_public_theorem_formula_candidates(&ast, &resolved, &env).unwrap(),
        Some(BTreeMap::new())
    );
}

#[test]
fn source_public_theorem_formula_candidates_refuse_malformed_written_nodes_after_base_admission() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let unsupported = format!("{fixture}\ntheorem Plain: 0 = 0;\n");
    let (unsupported_ast, module, _, unsupported_env, unsupported_arena) =
        parsed_private_theorem_fixture(&unsupported);
    assert_eq!(
        source_public_theorem_formula_candidates(
            &unsupported_ast,
            &unsupported_arena,
            &unsupported_env
        )
        .unwrap(),
        None
    );
    let (foreign_ast, _, _, foreign_env, _) =
        parsed_private_theorem_fixture(&unsupported.replace("UsePriv1", "Changed1"));
    assert_eq!(
        source_public_theorem_formula_candidates(
            &foreign_ast,
            &unsupported_arena,
            &unsupported_env
        )
        .unwrap_err(),
        unsupported_arena
            .validate_against(&foreign_ast, &module)
            .unwrap_err()
            .to_string()
    );
    assert_eq!(
        source_public_theorem_formula_candidates(
            &unsupported_ast,
            &unsupported_arena,
            &foreign_env
        )
        .unwrap_err(),
        crate::symbols::validate_source_symbol_env(&unsupported_arena, &foreign_env).unwrap_err()
    );
    let (ast, module, namespace, _, _) = parsed_private_theorem_fixture(fixture);
    let owner = exact_proof_label_item_list(&ast)
        .unwrap()
        .child_views()
        .find(|item| {
            exact_theorem_owner(*item, true).is_some_and(|owner| owner.spelling == "UsePriv1")
        })
        .unwrap();
    let children = owner.child_views().collect::<Vec<_>>();
    let formula = children[3];
    assert_eq!(formula.range(), range(ast.source_id, 112, 139));
    let descendant = formula
        .child_views()
        .find(|node| node.as_token().is_none())
        .unwrap();
    for mutation in [
        "kind",
        "missing",
        "multiple",
        "misplaced",
        "colon_overlap",
        "proof_overlap",
        "empty",
        "foreign_formula",
        "child_escape",
        "foreign_child",
        "proof_body",
        "justification",
        "computation",
    ] {
        // Replay structured parser output; this mutation is not source-text authentication.
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut mapped = BTreeMap::new();
        for node in ast.node_views() {
            let mut kind = node.kind().clone();
            let mut node_range = node.range();
            let mut mapped_children = node
                .children()
                .iter()
                .map(|child| mapped[child])
                .collect::<Vec<_>>();
            if node.id() == formula.id() {
                match mutation {
                    "kind" => kind = SurfaceNodeKind::Proposition,
                    "colon_overlap" => node_range.start = children[2].range().end - 1,
                    "proof_overlap" => node_range.end = children[4].range().start + 1,
                    "empty" => node_range.end = node_range.start,
                    "foreign_formula" => node_range.source_id = distinct_source_ids().1,
                    _ => {}
                }
            }
            if node.id() == descendant.id() {
                match mutation {
                    "child_escape" => node_range.end = formula.range().end + 1,
                    "foreign_child" => node_range.source_id = distinct_source_ids().1,
                    "proof_body" => kind = SurfaceNodeKind::ProofBlock,
                    "justification" => kind = SurfaceNodeKind::JustificationClause,
                    "computation" => kind = SurfaceNodeKind::ComputationJustification,
                    _ => {}
                }
            }
            if node.id() == owner.id() {
                match mutation {
                    "missing" => {
                        mapped_children.remove(3);
                    }
                    "multiple" => {
                        let extra =
                            builder.add_node(formula.kind().clone(), formula.range(), Vec::new());
                        mapped_children.insert(4, extra);
                    }
                    "misplaced" => mapped_children.swap(3, 4),
                    _ => {}
                }
            }
            let id = match &kind {
                SurfaceNodeKind::Token(token) => {
                    builder.add_token(token.kind, token.text.clone(), node_range)
                }
                _ => builder.add_node(kind, node_range, mapped_children),
            };
            mapped.insert(node.id(), id);
        }
        let rebuilt = builder.finish(Some(mapped[&ast.root().unwrap()]), None);
        let shells =
            crate::declarations::DeclarationShellCollector::new(&rebuilt, &module).collect();
        let collected =
            crate::symbols::SignatureProjectionExtractor::new(&rebuilt, &shells, namespace.clone())
                .collect(&module);
        assert!(
            collected.diagnostics().is_empty(),
            "{mutation}: {:?}",
            collected.diagnostics()
        );
        let env = collected.into_env();
        let arena = SurfaceResolvedArena::lower(&rebuilt, &module).unwrap();
        let before = (rebuilt.clone(), arena.clone(), env.clone());
        assert!(
            source_public_theorem_label_candidates(&rebuilt, &arena, &env)
                .unwrap()
                .is_some(),
            "{mutation}: inherited admission"
        );
        assert_eq!(
            source_public_theorem_formula_candidates(&rebuilt, &arena, &env).unwrap(),
            None,
            "{mutation}"
        );
        assert_eq!((rebuilt, arena, env), before);
    }
}

#[test]
fn source_theorem_label_reference_candidates_preserve_original_batch() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, expected) in [
        (fixture.to_owned(), vec![(178, 184, "PrivT1")]),
        (
            fixture.replace("theorem UsePriv1", "lemma UsePriv1"),
            vec![(176, 182, "PrivT1")],
        ),
        (
            fixture.replace("thus X = X by PrivT1;", "Inner: X = X by PrivT1;\n  thus X = X by Inner, PrivT1, Missing;"),
            vec![(180, 186, "PrivT1"), (204, 209, "Inner"), (211, 217, "PrivT1"), (219, 226, "Missing")],
        ),
        (
            fixture.replacen("thus X = X;", "Inner: X = X\n  proof\n    thus X = X;\n  end;\n  thus X = X by Inner;", 1),
            vec![(136, 141, "Inner"), (233, 239, "PrivT1")],
        ),
        (fixture.replace(" by PrivT1", ""), vec![]),
        (
            include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(),
            vec![],
        ),
    ] {
        let (ast, module, namespace, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), arena.clone(), env.clone());
        let contribution = env.contributions().by_source(ast.source_id)[0].id();
        let collection = ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &arena)
            .unwrap().collect_with_theorem_owners(&env).unwrap();
        let result = source_theorem_label_reference_candidates(&ast, &arena, &env).unwrap().unwrap();
        assert_eq!(result.ids().len(), expected.len());
        for (id, (start, end, spelling)) in result.ids().iter().zip(expected) {
            assert_eq!(&text[start..end], spelling);
            let reference = ast.node_views().find(|node| {
                matches!(node.kind(), SurfaceNodeKind::Reference)
                    && node.range() == range(ast.source_id, start, end)
            }).unwrap();
            let entry = result.table().get(*id).unwrap();
            assert_eq!(entry.site(), &ReferenceSite::new(arena.resolved_node_for(reference.id()).unwrap(), reference.range(), spelling));
            let original = collection.references().iter().find(|candidate| candidate.site() == entry.site()).unwrap();
            assert_eq!(entry.origin(), original.origin());
            if spelling == "Missing" {
                assert!(matches!(entry.resolution(), LabelResolution::Unresolved(_)));
                assert!(result.has_unresolved());
            } else {
                let projection = collection.projections().iter().find(|projection| projection.primary_spelling() == spelling).unwrap();
                if spelling == "PrivT1" {
                    let owner = env.symbols().iter().find(|owner| owner.primary_spelling() == spelling).unwrap();
                    assert_eq!(projection.origin_path().as_str(), owner.symbol().fqn().as_str());
                    assert_eq!(projection.declaration_range(), range(ast.source_id, 16, 22));
                    assert_eq!(projection.origin(), owner.origin());
                    assert_eq!(projection.visibility(), Visibility::Private);
                } else {
                    assert_eq!(projection.kind(), LabelKind::ProofStep);
                    assert!(matches!(projection.declaration_range().start, 76 | 164));
                    assert_eq!(&text[projection.declaration_range().start..projection.declaration_range().end], "Inner");
                }
                assert_eq!(entry.resolution(), &LabelResolution::Resolved(LabelRef::new(projection.origin_path().clone(), projection.kind(), reference.range())));
            }
        }
        assert_eq!(result, LabelResolver::new(collection.projections()).resolve(&module, &namespace, collection.references()));
        assert_eq!(source_theorem_label_reference_candidates(&ast, &arena, &env).unwrap(), Some(result));
        assert_eq!((ast, arena, env), before);
    }
}

#[test]
fn source_theorem_label_reference_candidates_refuse_incomplete_coverage() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for text in [
        fixture.replace("by PrivT1;", "by PrivT1, main.PrivT1;"),
        fixture.replace("by PrivT1;", "by PrivT1, PrivT1[X];"),
        fixture.replace("by PrivT1;", "by PrivT1, main.{PrivT1};"),
        fixture.replacen("thus X = X;", "assume X = X;\n  thus X = X;", 1),
        fixture.replacen("let X be set;", "let X be set such that A1: X = X;", 1),
    ] {
        let (ast, module, namespace, env, arena) = parsed_private_theorem_fixture(&text);
        assert!(
            source_public_theorem_label_candidates(&ast, &arena, &env)
                .unwrap()
                .is_some()
        );
        let collection = ProofLabelSourceCollector::new(
            &ast,
            &module,
            namespace,
            env.contributions().by_source(ast.source_id)[0].id(),
            &arena,
        )
        .unwrap()
        .collect_with_theorem_owners(&env)
        .unwrap();
        assert!(
            !collection.references().is_empty(),
            "supported citation retained: {text}"
        );
        assert_eq!(
            source_theorem_label_reference_candidates(&ast, &arena, &env).unwrap(),
            None,
            "{text}"
        );
    }
    let unsupported = format!("{fixture}\ntheorem Plain: 0 = 0;\n");
    let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&unsupported);
    assert_eq!(
        source_theorem_label_reference_candidates(&ast, &arena, &env).unwrap(),
        None
    );
    let (foreign_ast, _, _, foreign_env, _) =
        parsed_private_theorem_fixture(&unsupported.replace("UsePriv1", "Changed1"));
    assert_eq!(
        source_theorem_label_reference_candidates(&foreign_ast, &arena, &env).unwrap_err(),
        arena
            .validate_against(&foreign_ast, &module)
            .unwrap_err()
            .to_string()
    );
    assert_eq!(
        source_theorem_label_reference_candidates(&ast, &arena, &foreign_env).unwrap_err(),
        crate::symbols::validate_source_symbol_env(&arena, &foreign_env).unwrap_err()
    );

    let (ast, module, namespace, _, _) = parsed_private_theorem_fixture(fixture);
    let reference = ast
        .node_views()
        .find(|node| matches!(node.kind(), SurfaceNodeKind::Reference))
        .unwrap();
    let list = ast
        .node_views()
        .find(|node| matches!(node.kind(), SurfaceNodeKind::ReferenceList))
        .unwrap();
    let owner = exact_proof_label_item_list(&ast)
        .unwrap()
        .child_views()
        .find(|item| {
            exact_theorem_owner(*item, true).is_some_and(|owner| owner.spelling == "UsePriv1")
        })
        .unwrap();
    assert_eq!(owner.range().source_id, ast.source_id);
    let foreign_source_id = distinct_source_ids().1;
    assert_ne!(foreign_source_id, ast.source_id);
    for mutation in [
        "range",
        "child_range",
        "duplicate",
        "orphan",
        "hidden",
        "parent_original_source",
        "parent_foreign_source",
    ] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut mapped = BTreeMap::new();
        for node in ast.node_views() {
            let mut kind = node.kind().clone();
            let mut node_range = node.range();
            let mut children = node
                .children()
                .iter()
                .map(|child| mapped[child])
                .collect::<Vec<_>>();
            if node.id() == owner.id()
                && matches!(mutation, "parent_original_source" | "parent_foreign_source")
            {
                node_range.source_id = if mutation == "parent_foreign_source" {
                    foreign_source_id
                } else {
                    ast.source_id
                };
            }
            if node.id() == reference.id() && mutation == "range" {
                node_range.start -= 1;
            }
            if reference.children().contains(&node.id()) && mutation == "child_range" {
                node_range.end += 1;
            }
            if node.id() == list.id() && mutation == "duplicate" {
                let identifier = exact_simple_reference_identifier(reference).unwrap();
                let token = identifier.as_token().unwrap();
                let cloned = builder.add_token(token.kind, token.text.clone(), identifier.range());
                children.push(builder.add_node(
                    reference.kind().clone(),
                    reference.range(),
                    vec![cloned],
                ));
            }
            if node.id() == list.id() && mutation == "hidden" {
                // A retained citation under a skipped list wrapper is not an empty batch.
                kind = SurfaceNodeKind::FormulaExpression;
            }
            let id = match &kind {
                SurfaceNodeKind::Token(token) => {
                    builder.add_token(token.kind, token.text.clone(), node_range)
                }
                _ => builder.add_node(kind, node_range, children),
            };
            mapped.insert(node.id(), id);
        }
        if mutation == "orphan" {
            builder.add_node(SurfaceNodeKind::ReferenceList, list.range(), Vec::new());
        }
        let rebuilt = builder.finish(Some(mapped[&ast.root().unwrap()]), None);
        let shells =
            crate::declarations::DeclarationShellCollector::new(&rebuilt, &module).collect();
        let collected =
            crate::symbols::SignatureProjectionExtractor::new(&rebuilt, &shells, namespace.clone())
                .collect(&module);
        assert!(collected.diagnostics().is_empty(), "{mutation}");
        let env = collected.into_env();
        let arena = SurfaceResolvedArena::lower(&rebuilt, &module).unwrap();
        let before = (rebuilt.clone(), arena.clone(), env.clone());
        assert!(
            source_public_theorem_label_candidates(&rebuilt, &arena, &env)
                .unwrap()
                .is_some(),
            "{mutation}: inherited admission"
        );
        let collection = ProofLabelSourceCollector::new(
            &rebuilt,
            &module,
            namespace.clone(),
            env.contributions().by_source(rebuilt.source_id)[0].id(),
            &arena,
        )
        .unwrap()
        .collect_with_theorem_owners(&env)
        .unwrap();
        assert_eq!(
            collection.projections().len(),
            2,
            "{mutation}: genuine owners"
        );
        assert_eq!(
            collection.references().len(),
            match mutation {
                "duplicate" => 2,
                "hidden" => 0,
                _ => 1,
            },
            "{mutation}: actual collector"
        );
        let result = source_theorem_label_reference_candidates(&rebuilt, &arena, &env).unwrap();
        if mutation == "parent_original_source" {
            assert!(
                result.is_some(),
                "original same-source parent remains admitted"
            );
        } else {
            assert_eq!(result, None, "{mutation}");
        }
        assert_eq!((rebuilt, arena, env), before);
    }
}
#[test]
fn source_theorem_builtin_type_reference_candidates_preserve_original_batch() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, expected) in [
        (fixture.to_owned(), vec![(36, 39, "set"), (69, 72, "set"), (124, 127, "set"), (157, 160, "set")]),
        (fixture.replace("private ", ""), vec![(28, 31, "set"), (61, 64, "set"), (116, 119, "set"), (149, 152, "set")]),
        (include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz").to_owned(), vec![(24, 27, "set"), (57, 60, "set"), (111, 114, "set"), (144, 147, "set")]),
        (include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(), vec![(33, 36, "set"), (85, 88, "set")]),
        (fixture.replace("set", "object"), vec![(36, 42, "object"), (72, 78, "object"), (130, 136, "object"), (166, 172, "object")]),
        (fixture.replacen("set", "object", 1), vec![(36, 42, "object"), (72, 75, "set"), (127, 130, "set"), (160, 163, "set")]),
        ("theorem Plain: 0 = 0\nproof\n  thus 0 = 0;\nend;\n".to_owned(), vec![]),
    ] {
        let (ast, module, namespace, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), arena.clone(), env.clone());
        let result = crate::names::source_theorem_builtin_type_reference_candidates(&ast, &arena, &env).unwrap().unwrap();
        assert_eq!(result.ids().len(), expected.len());
        assert_eq!(result.table().len(), expected.len());
        assert!(!result.has_unresolved());
        assert_eq!(ast.node_views().filter(|node| matches!(node.kind(), SurfaceNodeKind::TypeHead)).count(), expected.len());
        let mut candidates = Vec::new();
        for (index, (id, (start, end, spelling))) in result.ids().iter().zip(expected).enumerate() {
            assert_eq!(id.index(), index);
            assert_eq!(&text[start..end], spelling);
            let head = ast.node_views().find(|node| matches!(node.kind(), SurfaceNodeKind::TypeHead) && node.range() == range(ast.source_id, start, end)).unwrap();
            let mapped = arena.resolved_node_for(head.id()).unwrap();
            let entry = result.table().get(*id).unwrap();
            assert_eq!(entry.site(), &ReferenceSite::new(mapped, head.range(), spelling));
            assert_eq!(entry.origin(), arena.arena().node(mapped).unwrap().origin());
            let builtin = match spelling {
                "set" => "builtin:type:set",
                "object" => "builtin:type:object",
                _ => unreachable!(),
            };
            assert_eq!(entry.resolution(), &crate::resolved_ast::NameResolution::ResolvedBuiltin(crate::resolved_ast::BuiltinRef::new(crate::resolved_ast::BuiltinId::new(builtin), range(ast.source_id, start, end), spelling)));
            candidates.push(crate::names::NameReferenceCandidate::unqualified(entry.site().clone(), entry.origin().clone(), start));
        }
        let builtins = [
            crate::names::BuiltinNameProjection::new(crate::resolved_ast::BuiltinId::new("builtin:type:set"), "set"),
            crate::names::BuiltinNameProjection::new(crate::resolved_ast::BuiltinId::new("builtin:type:object"), "object"),
        ];
        assert_eq!(result, crate::names::SymbolNameResolver::new(&[], &builtins).resolve(&module, &namespace, &candidates));
        assert_eq!(crate::names::source_theorem_builtin_type_reference_candidates(&ast, &arena, &env).unwrap(), Some(result));
        assert_eq!((ast, arena, env), before);
    }
}

#[test]
fn source_theorem_builtin_type_reference_candidates_refuse_incomplete_coverage() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let unsupported = fixture.replace("by PrivT1;", "by PrivT1, main.PrivT1;");
    let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&unsupported);
    assert_eq!(
        source_theorem_label_reference_candidates(&ast, &arena, &env).unwrap(),
        None
    );
    assert_eq!(
        crate::names::source_theorem_builtin_type_reference_candidates(&ast, &arena, &env).unwrap(),
        None
    );
    let (foreign_ast, _, _, foreign_env, _) =
        parsed_private_theorem_fixture(&unsupported.replace("UsePriv1", "Changed1"));
    assert_eq!(
        crate::names::source_theorem_builtin_type_reference_candidates(&foreign_ast, &arena, &env)
            .unwrap_err(),
        arena
            .validate_against(&foreign_ast, &module)
            .unwrap_err()
            .to_string()
    );
    assert_eq!(
        crate::names::source_theorem_builtin_type_reference_candidates(&ast, &arena, &foreign_env)
            .unwrap_err(),
        crate::symbols::validate_source_symbol_env(&arena, &foreign_env).unwrap_err()
    );

    let (ast, module, namespace, _, _) = parsed_private_theorem_fixture(fixture);
    let expression = ast
        .node_views()
        .find(|node| matches!(node.kind(), SurfaceNodeKind::TypeExpression))
        .unwrap();
    assert_eq!(expression.range(), range(ast.source_id, 36, 39));
    let head = expression.child_views().next().unwrap();
    let token = head.child_views().next().unwrap();
    for mutation in [
        "attribute",
        "extra_token",
        "missing_head",
        "missing_token",
        "non_token",
        "named",
        "token_kind",
        "token_text",
        "expression_range",
        "head_range",
        "token_range",
        "empty",
        "uncovered",
        "orphan",
        "foreign_range",
    ] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut mapped = BTreeMap::new();
        for node in ast.node_views() {
            let mut kind = node.kind().clone();
            let mut node_range = node.range();
            let mut children = node
                .children()
                .iter()
                .map(|child| mapped[child])
                .collect::<Vec<_>>();
            if node.id() == token.id() {
                if mutation == "token_kind" {
                    kind = SurfaceNodeKind::Token(mizar_syntax::SurfaceToken {
                        kind: SurfaceTokenKind::Identifier,
                        text: "set".into(),
                    });
                }
                if mutation == "token_text" {
                    kind = SurfaceNodeKind::Token(mizar_syntax::SurfaceToken {
                        kind: SurfaceTokenKind::ReservedWord,
                        text: "Set".into(),
                    });
                }
                if mutation == "token_range" {
                    node_range.end -= 1;
                }
                if mutation == "foreign_range" {
                    node_range.source_id = distinct_source_ids().1;
                }
            }
            if node.id() == head.id() {
                match mutation {
                    "extra_token" => {
                        let original = token.as_token().unwrap();
                        children.push(builder.add_token(
                            original.kind,
                            original.text.clone(),
                            token.range(),
                        ));
                    }
                    "missing_head" => kind = SurfaceNodeKind::FormulaExpression,
                    "missing_token" => children.clear(),
                    "non_token" | "named" => {
                        children = vec![builder.add_node(
                            if mutation == "named" {
                                SurfaceNodeKind::QualifiedSymbol
                            } else {
                                SurfaceNodeKind::FormulaExpression
                            },
                            token.range(),
                            children,
                        )];
                    }
                    "head_range" => node_range.start -= 1,
                    _ => {}
                }
            }
            if node.id() == expression.id() {
                match mutation {
                    "attribute" => {
                        let original = token.as_token().unwrap();
                        let cloned =
                            builder.add_token(original.kind, original.text.clone(), token.range());
                        children.insert(
                            0,
                            builder.add_node(
                                SurfaceNodeKind::AttributeChain,
                                token.range(),
                                vec![cloned],
                            ),
                        );
                    }
                    "expression_range" => node_range.start -= 1,
                    "uncovered" => kind = SurfaceNodeKind::FormulaExpression,
                    _ => {}
                }
            }
            if mutation == "empty" && [expression.id(), head.id(), token.id()].contains(&node.id())
            {
                node_range.end = node_range.start;
            }
            let id = match &kind {
                SurfaceNodeKind::Token(token) => {
                    builder.add_token(token.kind, token.text.clone(), node_range)
                }
                _ => builder.add_node(kind, node_range, children),
            };
            mapped.insert(node.id(), id);
        }
        if mutation == "orphan" {
            let original = token.as_token().unwrap();
            let cloned = builder.add_token(original.kind, original.text.clone(), token.range());
            builder.add_node(head.kind().clone(), head.range(), vec![cloned]);
        }
        let rebuilt = builder.finish(Some(mapped[&ast.root().unwrap()]), None);
        let shells =
            crate::declarations::DeclarationShellCollector::new(&rebuilt, &module).collect();
        let collected =
            crate::symbols::SignatureProjectionExtractor::new(&rebuilt, &shells, namespace.clone())
                .collect(&module);
        assert!(collected.diagnostics().is_empty(), "{mutation}");
        let env = collected.into_env();
        let arena = SurfaceResolvedArena::lower(&rebuilt, &module).unwrap();
        let before = (rebuilt.clone(), arena.clone(), env.clone());
        assert!(
            source_theorem_label_reference_candidates(&rebuilt, &arena, &env)
                .unwrap()
                .is_some(),
            "{mutation}: inherited admission"
        );
        assert!(
            rebuilt
                .node_views()
                .any(|node| matches!(node.kind(), SurfaceNodeKind::TypeHead)
                    && node.range() == range(ast.source_id, 69, 72)),
            "{mutation}: supported head retained"
        );
        let result =
            crate::names::source_theorem_builtin_type_reference_candidates(&rebuilt, &arena, &env);
        match mutation {
            "missing_token" | "head_range" | "orphan" => assert_eq!(
                result.unwrap_err(),
                format!("{:?}", crate::names::SourceVariableScopeError::InvalidShape),
                "{mutation}"
            ),
            "foreign_range" => assert_eq!(
                result.unwrap_err(),
                format!(
                    "{:?}",
                    crate::names::SourceVariableScopeError::SourceMismatch
                ),
                "{mutation}"
            ),
            _ => assert_eq!(result.unwrap(), None, "{mutation}"),
        }
        assert_eq!((rebuilt, arena, env), before);
    }
}
#[test]
fn source_theorem_resolved_ast_preserves_complete_semantic_snapshot() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, types, citations) in [
        (fixture.to_owned(), vec![(36,39,"set"),(69,72,"set"),(124,127,"set"),(157,160,"set")], vec![(178,184,"PrivT1")]),
        (fixture.replace("private ", ""), vec![(28,31,"set"),(61,64,"set"),(116,119,"set"),(149,152,"set")], vec![(170,176,"PrivT1")]),
        (include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz").to_owned(), vec![(24,27,"set"),(57,60,"set"),(111,114,"set"),(144,147,"set")], vec![(165,169,"Lem1")]),
        (include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(), vec![(33,36,"set"),(85,88,"set")], vec![]),
        (fixture.replace("set", "object"), vec![(36,42,"object"),(72,78,"object"),(130,136,"object"),(166,172,"object")], vec![(190,196,"PrivT1")]),
        (fixture.replacen("set", "object", 1), vec![(36,42,"object"),(72,75,"set"),(127,130,"set"),(160,163,"set")], vec![(181,187,"PrivT1")]),
        ("theorem Plain: 0 = 0\nproof\n  thus 0 = 0;\nend;\n".to_owned(), vec![], vec![]),
        (fixture.replace("by PrivT1", "by Missing"), vec![(36,39,"set"),(69,72,"set"),(124,127,"set"),(157,160,"set")], vec![(178,185,"Missing")]),
    ] {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), arena.clone(), env.clone());
        let result = crate::resolved_ast::source_theorem_resolved_ast(&ast, &arena, &env).unwrap().unwrap();
        assert_eq!(result.source_id(), ast.source_id);
        assert_eq!(result.module_id(), &module);
        assert_eq!(result.nodes().root(), arena.arena().root());
        assert_eq!(result.nodes().len(), ast.nodes().len());
        assert_eq!(result.imports(), &crate::resolved_ast::ResolvedImports::new());
        let mut expected_names = crate::resolved_ast::NameRefTable::new();
        let mut expected_labels = crate::resolved_ast::LabelRefTable::new();
        let mut attachments = BTreeMap::new();
        for (start, end, spelling) in types {
            assert_eq!(&text[start..end], spelling);
            let head = ast.node_views().find(|node| matches!(node.kind(), SurfaceNodeKind::TypeHead) && node.range() == range(ast.source_id, start, end)).unwrap();
            let mapped = arena.resolved_node_for(head.id()).unwrap();
            let origin = SemanticOrigin::new(ast.source_id, module.clone(), SourceAnchor::Range(head.range()), vec![u32::try_from(head.id().index()).unwrap()]);
            let id = expected_names.insert(crate::resolved_ast::NameRefEntry::new(
                ReferenceSite::new(mapped, head.range(), spelling),
                crate::resolved_ast::NameResolution::ResolvedBuiltin(crate::resolved_ast::BuiltinRef::new(crate::resolved_ast::BuiltinId::new(format!("builtin:type:{spelling}")), head.range(), spelling)), origin));
            attachments.insert(mapped, (crate::resolved_ast::NodeReferenceKey::Name(id), crate::resolved_ast::NodeResolutionState::Resolved));
        }
        for (start, end, spelling) in citations {
            assert_eq!(&text[start..end], spelling);
            let reference = ast.node_views().find(|node| matches!(node.kind(), SurfaceNodeKind::Reference) && node.range() == range(ast.source_id, start, end)).unwrap();
            let statement = ast.node_views().find(|node| matches!(node.kind(), SurfaceNodeKind::ConclusionStatement) && node.range().start <= start && node.range().end >= end).unwrap();
            let theorem = ast.node_views().find(|node| matches!(node.kind(), SurfaceNodeKind::TheoremItem | SurfaceNodeKind::LemmaItem) && node.range().start <= start && node.range().end >= end).unwrap();
            let origin = SemanticOrigin::new(ast.source_id, module.clone(), SourceAnchor::Range(reference.range()), [theorem.id(), statement.id(), reference.id()].map(|id| u32::try_from(id.index()).unwrap()).to_vec());
            let mapped = arena.resolved_node_for(reference.id()).unwrap();
            assert_ne!(&origin, arena.arena().node(mapped).unwrap().origin());
            let (resolution, state) = if spelling == "Missing" {
                (LabelResolution::Unresolved(crate::resolved_ast::UnresolvedLabelRef::new(spelling, reference.range(), crate::resolved_ast::LabelExpectation::ProofOrTheorem)), crate::resolved_ast::NodeResolutionState::Unresolved)
            } else {
                let owner = env.symbols().iter().find(|entry| entry.primary_spelling() == spelling).unwrap();
                (LabelResolution::Resolved(LabelRef::new(crate::resolved_ast::LabelOriginPath::new(owner.symbol().fqn().as_str()), LabelKind::Theorem, reference.range())), crate::resolved_ast::NodeResolutionState::Resolved)
            };
            let id = expected_labels.insert(crate::resolved_ast::LabelRefEntry::new(ReferenceSite::new(mapped, reference.range(), spelling), resolution, origin));
            attachments.insert(mapped, (crate::resolved_ast::NodeReferenceKey::Label(id), state));
        }
        assert_eq!(result.name_refs(), &expected_names);
        assert_eq!(result.label_refs(), &expected_labels);
        for (id, original) in arena.arena().iter() {
            let expected = if let Some((key, state)) = attachments.get(&id) { original.clone().with_reference_key(*key).with_resolution(*state) } else { original.clone() };
            assert_eq!(result.nodes().node(id).unwrap(), &expected);
            assert_eq!(original.resolution(), crate::resolved_ast::NodeResolutionState::NotApplicable);
            assert_eq!(original.reference_key(), None);
        }
        assert_eq!(result.name_refs(), crate::names::source_theorem_builtin_type_reference_candidates(&ast, &arena, &env).unwrap().unwrap().table());
        assert_eq!(result.label_refs(), source_theorem_label_reference_candidates(&ast, &arena, &env).unwrap().unwrap().table());
        assert_eq!(crate::resolved_ast::source_theorem_resolved_ast(&ast, &arena, &env).unwrap(), Some(result));
        assert_eq!((ast, arena, env), before);
    }
}

#[test]
fn source_theorem_resolved_ast_refuses_incomplete_roles_and_preserves_errors() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let (ast, module, namespace, _, _) = parsed_private_theorem_fixture(fixture);
    let term = ast
        .node_views()
        .find(|node| {
            matches!(node.kind(), SurfaceNodeKind::TermReference)
                && node.range() == range(ast.source_id, 46, 47)
        })
        .unwrap();
    let terminal = term.child_views().next().unwrap();
    let wrapper = ast
        .node_views()
        .find(|node| {
            matches!(node.kind(), SurfaceNodeKind::TermExpression) && node.children() == [term.id()]
        })
        .unwrap();
    let formula = ast
        .node_views()
        .find(|node| {
            matches!(node.kind(), SurfaceNodeKind::BuiltinPredicateApplication)
                && node.range().start == 46
        })
        .unwrap();
    let segment = ast
        .node_views()
        .find(|node| matches!(node.kind(), SurfaceNodeKind::QuantifierVariableSegment))
        .unwrap();
    for mutation in [
        "qualified",
        "template",
        "other_formula",
        "other_term",
        "extra_formula_child",
        "extra_segment_child",
        "hidden_identifier",
        "misclassified_identifier",
        "term_range",
        "wrapper_range",
        "numeral_kind",
        "application",
        "extra_owner_child",
        "extra_proposition_child",
        "extra_quantifier_child",
        "extra_formula_wrapper_child",
        "wrong_let_keyword",
        "wrong_conclusion_keyword",
    ] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut mapped = BTreeMap::new();
        for node in ast.node_views() {
            let mut kind = node.kind().clone();
            let mut node_range = node.range();
            let mut children = node
                .children()
                .iter()
                .map(|child| mapped[child])
                .collect::<Vec<_>>();
            if node.id() == term.id() {
                match mutation {
                    "qualified" => {
                        children = vec![builder.add_node(
                            SurfaceNodeKind::QualifiedSymbol,
                            node_range,
                            children,
                        )]
                    }
                    "template" => children.push(builder.add_node(
                        SurfaceNodeKind::TemplateArguments,
                        range(ast.source_id, 47, 47),
                        vec![],
                    )),
                    "other_term" => kind = SurfaceNodeKind::ParenthesizedTerm,
                    "application" => kind = SurfaceNodeKind::ApplicationTerm,
                    "misclassified_identifier" | "numeral_kind" => {
                        kind = SurfaceNodeKind::NumeralTerm
                    }
                    "term_range" => node_range.end += 1,
                    _ => {}
                }
            }
            if node.id() == wrapper.id() && matches!(mutation, "wrapper_range" | "term_range") {
                node_range.end += 1;
            }
            if node.id() == formula.id() {
                match mutation {
                    "other_formula" => {
                        kind = SurfaceNodeKind::BinaryFormula(
                            mizar_syntax::SurfaceFormulaBinaryOperator {
                                connective: mizar_syntax::SurfaceFormulaConnective::And,
                                repeated: false,
                            },
                        )
                    }
                    "extra_formula_child" | "hidden_identifier" => {
                        children.insert(
                            1,
                            builder.add_token(
                                if mutation == "hidden_identifier" {
                                    SurfaceTokenKind::Identifier
                                } else {
                                    SurfaceTokenKind::ReservedSymbol
                                },
                                "extra",
                                range(ast.source_id, 47, 47),
                            ),
                        );
                    }
                    _ => {}
                }
            }
            if node.id() == segment.id() && mutation == "extra_segment_child" {
                children.push(builder.add_token(
                    SurfaceTokenKind::ReservedSymbol,
                    ",",
                    range(ast.source_id, 39, 39),
                ));
            }
            if node.id() == terminal.id() && mutation == "numeral_kind" {
                kind = SurfaceNodeKind::Token(mizar_syntax::SurfaceToken {
                    kind: SurfaceTokenKind::ReservedWord,
                    text: "X".into(),
                });
            }
            if mutation == "wrong_let_keyword"
                && node
                    .as_token()
                    .is_some_and(|token| token.text.as_ref() == "let")
            {
                kind = SurfaceNodeKind::Token(mizar_syntax::SurfaceToken {
                    kind: SurfaceTokenKind::ReservedWord,
                    text: "thus".into(),
                });
            }
            if mutation == "wrong_conclusion_keyword"
                && node
                    .as_token()
                    .is_some_and(|token| token.text.as_ref() == "thus")
            {
                kind = SurfaceNodeKind::Token(mizar_syntax::SurfaceToken {
                    kind: SurfaceTokenKind::ReservedWord,
                    text: "let".into(),
                });
            }
            if (mutation == "extra_owner_child"
                && matches!(node.kind(), SurfaceNodeKind::TheoremItem))
                || (mutation == "extra_proposition_child"
                    && matches!(node.kind(), SurfaceNodeKind::Proposition))
                || (mutation == "extra_quantifier_child"
                    && matches!(node.kind(), SurfaceNodeKind::QuantifiedFormula(_)))
                || (mutation == "extra_formula_wrapper_child"
                    && matches!(node.kind(), SurfaceNodeKind::FormulaExpression))
            {
                children.push(builder.add_token(
                    SurfaceTokenKind::ReservedSymbol,
                    "",
                    range(ast.source_id, node_range.end, node_range.end),
                ));
            }
            let id = if let SurfaceNodeKind::Token(token) = &kind {
                builder.add_token(token.kind, token.text.clone(), node_range)
            } else {
                builder.add_node(kind, node_range, children)
            };
            mapped.insert(node.id(), id);
        }
        let rebuilt = builder.finish(Some(mapped[&ast.root().unwrap()]), None);
        let shells =
            crate::declarations::DeclarationShellCollector::new(&rebuilt, &module).collect();
        let collected =
            crate::symbols::SignatureProjectionExtractor::new(&rebuilt, &shells, namespace.clone())
                .collect(&module);
        assert!(collected.diagnostics().is_empty(), "{mutation}");
        let env = collected.into_env();
        let arena = SurfaceResolvedArena::lower(&rebuilt, &module).unwrap();
        let before = (rebuilt.clone(), arena.clone(), env.clone());
        assert!(
            crate::names::source_theorem_builtin_type_reference_candidates(&rebuilt, &arena, &env)
                .unwrap_or_else(|error| panic!("{mutation}: {error}"))
                .is_some(),
            "{mutation}: inherited admission"
        );
        assert!(
            rebuilt
                .node_views()
                .any(|node| matches!(node.kind(), SurfaceNodeKind::TermReference)
                    && node.range() == range(ast.source_id, 50, 51)),
            "{mutation}: supported use retained"
        );
        assert_eq!(
            crate::resolved_ast::source_theorem_resolved_ast(&rebuilt, &arena, &env).unwrap(),
            None,
            "{mutation}"
        );
        assert_eq!((rebuilt, arena, env), before);
    }
    let template_source = fixture.replacen("holds X = X", "holds X[set] = X", 1);
    let (template_ast, template_module, _, template_env, template_arena) =
        parsed_private_theorem_fixture(&template_source);
    let template_names = crate::names::source_theorem_builtin_type_reference_candidates(
        &template_ast,
        &template_arena,
        &template_env,
    )
    .unwrap()
    .unwrap();
    assert_eq!(template_names.table().len(), 5);
    let template_scope = crate::names::SourceVariableScopeResolver::resolve_occurrences(
        crate::names::SourceVariableScopeInput::new(&template_ast, &template_module, &template_env),
    )
    .unwrap();
    assert_eq!(template_scope.references().len(), 8);
    let template_reference = template_ast
        .node_views()
        .find(|node| {
            matches!(node.kind(), SurfaceNodeKind::TermReference)
                && node.range() == range(template_ast.source_id, 46, 52)
        })
        .unwrap();
    assert_eq!(template_reference.children().len(), 2);
    assert!(
        template_scope
            .references()
            .iter()
            .any(|reference| reference.node() == template_reference.id())
    );
    assert_eq!(
        crate::resolved_ast::source_theorem_resolved_ast(
            &template_ast,
            &template_arena,
            &template_env
        )
        .unwrap(),
        None
    );
    let (unsupported_ast, unsupported_module, _, unsupported_env, unsupported_arena) =
        parsed_private_theorem_fixture(&template_source.replacen("X[set] = X", "X[set] = Y", 1));
    assert!(
        crate::names::source_theorem_builtin_type_reference_candidates(
            &unsupported_ast,
            &unsupported_arena,
            &unsupported_env
        )
        .unwrap()
        .is_some()
    );
    assert!(matches!(
        crate::names::SourceVariableScopeResolver::resolve_occurrences(
            crate::names::SourceVariableScopeInput::new(
                &unsupported_ast,
                &unsupported_module,
                &unsupported_env
            )
        ),
        Err(crate::names::SourceVariableScopeError::UnresolvedReference)
    ));
    assert_eq!(
        crate::resolved_ast::source_theorem_resolved_ast(
            &unsupported_ast,
            &unsupported_arena,
            &unsupported_env
        )
        .unwrap(),
        None
    );
    let unsupported = fixture.replace("by PrivT1;", "by main.PrivT1;");
    let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&unsupported);
    assert_eq!(
        crate::names::source_theorem_builtin_type_reference_candidates(&ast, &arena, &env).unwrap(),
        None
    );
    assert_eq!(
        crate::resolved_ast::source_theorem_resolved_ast(&ast, &arena, &env).unwrap(),
        None
    );
    let (foreign_ast, _, _, foreign_env, _) =
        parsed_private_theorem_fixture(&unsupported.replace("UsePriv1", "Changed1"));
    assert_eq!(
        crate::resolved_ast::source_theorem_resolved_ast(&foreign_ast, &arena, &env).unwrap_err(),
        arena
            .validate_against(&foreign_ast, &module)
            .unwrap_err()
            .to_string()
    );
    assert_eq!(
        crate::resolved_ast::source_theorem_resolved_ast(&ast, &arena, &foreign_env).unwrap_err(),
        crate::symbols::validate_source_symbol_env(&arena, &foreign_env).unwrap_err()
    );
    assert_eq!(
        crate::resolved_ast::source_theorem_resolved_ast(&foreign_ast, &arena, &foreign_env)
            .unwrap_err(),
        arena
            .validate_against(&foreign_ast, &module)
            .unwrap_err()
            .to_string()
    );
    let (ast, module, _, env, arena) =
        parsed_private_theorem_fixture(&fixture.replace("thus X = X;", "thus Y = X;"));
    assert!(
        crate::names::source_theorem_builtin_type_reference_candidates(&ast, &arena, &env)
            .unwrap()
            .is_some()
    );
    let error = crate::names::SourceVariableScopeResolver::resolve_occurrences(
        crate::names::SourceVariableScopeInput::new(&ast, &module, &env),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        crate::names::SourceVariableScopeError::UnresolvedReference
    ));
    assert_eq!(
        crate::resolved_ast::source_theorem_resolved_ast(&ast, &arena, &env).unwrap_err(),
        format!("{error:?}")
    );
}

#[test]
fn source_public_theorem_signature_candidates_frame_original_public_statements() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let quantified = "101:3:for100:1:X101:5:being101:3:set101:5:holds100:1:X102:1:=100:1:X";
    let numeral = "103:1:0102:1:=103:1:0";
    let alpha = "theorem Alpha: 0 = 0\nproof\n  thus 0 = 0;\nend;\n";
    for (text, expected) in [
        (fixture.to_owned(), vec![("UsePriv1", quantified)]),
        (fixture.replacen("private theorem", "public theorem", 1), vec![("PrivT1", quantified), ("UsePriv1", quantified)]),
        (fixture.replace("theorem UsePriv1", "lemma UsePriv1"), vec![("UsePriv1", quantified)]),
        (fixture.replace("private ", ""), vec![("PrivT1", quantified), ("UsePriv1", quantified)]),
        (fixture.replace("UsePriv1", "Renamed1"), vec![("Renamed1", quantified)]),
        (fixture.replace("thus X = X by PrivT1;", "Inner: X = X;\n  thus X = X by Inner;"), vec![("UsePriv1", quantified)]),
        (fixture.replace("holds X = X", "holds\n X  =  X :: \u{03bb} comment\n"), vec![("UsePriv1", quantified)]),
        (format!("{fixture}\n{alpha}"), vec![("UsePriv1", quantified), ("Alpha", numeral)]),
        (format!("{alpha}\n{fixture}"), vec![("UsePriv1", quantified), ("Alpha", numeral)]),
        (include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(), vec![("OpenT1", quantified), ("AxT1", quantified)]),
        (fixture.replace('X', "Y"), vec![("UsePriv1", "101:3:for100:1:Y101:5:being101:3:set101:5:holds100:1:Y102:1:=100:1:Y")]),
        (fixture.replace("holds X = X", "holds X <> X"), vec![("UsePriv1", "101:3:for100:1:X101:5:being101:3:set101:5:holds100:1:X102:2:<>100:1:X")]),
        (alpha.replace("0 = 0", "0 = 12"), vec![("Alpha", "103:1:0102:1:=103:2:12")]),
        (alpha.replace("0 = 0", "12 = 0"), vec![("Alpha", "103:2:12102:1:=103:1:0")]),
        (fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"), vec![]),
    ] {
        let (ast, _, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), arena.clone(), env.clone());
        let mut expected_map = BTreeMap::new();
        for (label, payload) in expected {
            let owner = env.symbols().iter().find(|entry| entry.primary_spelling() == label).unwrap();
            assert_eq!(owner.visibility(), Visibility::Public);
            assert_eq!(owner.export_status(), ExportStatus::Exported);
            assert!(expected_map.insert(owner.symbol().clone(), crate::env::SignatureShell::Opaque {
                schema: "source-theorem-statement-tokens-v1".to_owned(),
                payload: payload.to_owned(),
            }).is_none());
        }
        let result = crate::symbols::source_public_theorem_signature_candidates(&ast, &arena, &env).unwrap();
        assert_eq!(result, Some(expected_map), "{text}");
        assert_eq!(crate::symbols::source_public_theorem_signature_candidates(&ast, &arena, &env).unwrap(), result);
        assert_eq!((ast, arena, env), before);
    }
}

#[test]
fn source_public_theorem_signature_candidates_refuse_whole_maps_and_preserve_errors() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for text in [
        fixture.replacen("private theorem", "public lemma", 1),
        format!("{fixture}\nreserve Y for set;\n"),
        format!("{fixture}\ntheorem Plain: 0 = 0;\n"),
        format!("{fixture}\ntheorem Direct: 0 = 0 by UsePriv1;\n"),
        format!("{fixture}\nopen theorem OpenDirect: 0 = 0 by UsePriv1;\n"),
        format!("{fixture}\nopen theorem OpenComp: 0 = 0 by computation(steps: 1);\n"),
        format!(
            "{fixture}\n{}",
            include_str!(
                "../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.miz"
            )
        ),
    ] {
        let (ast, _, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), arena.clone(), env.clone());
        assert_eq!(
            source_public_theorem_formula_candidates(&ast, &arena, &env).unwrap(),
            None
        );
        assert_eq!(
            crate::symbols::source_public_theorem_signature_candidates(&ast, &arena, &env).unwrap(),
            None
        );
        assert_eq!((ast, arena, env), before);
    }
    let unsupported = format!("{fixture}\ntheorem Plain: 0 = 0;\n");
    let (ast, _, _, env, arena) = parsed_private_theorem_fixture(&unsupported);
    let (foreign_ast, _, _, foreign_env, foreign_arena) =
        parsed_private_theorem_fixture(&unsupported.replace("UsePriv1", "Changed1"));
    for (ast, arena, env) in [
        (&foreign_ast, &arena, &env),
        (&ast, &foreign_arena, &env),
        (&ast, &arena, &foreign_env),
        (&foreign_ast, &arena, &foreign_env),
    ] {
        let before = (ast.clone(), arena.clone(), env.clone());
        assert_eq!(
            crate::symbols::source_public_theorem_signature_candidates(ast, arena, env)
                .unwrap_err(),
            source_public_theorem_formula_candidates(ast, arena, env).unwrap_err()
        );
        assert_eq!((ast.clone(), arena.clone(), env.clone()), before);
    }
    let text = format!("{fixture}\ntheorem Alpha: 0 = 0\nproof\n  thus 0 = 0;\nend;\n");
    let (ast, module, namespace, env, arena) = parsed_private_theorem_fixture(&text);
    let original = source_public_theorem_formula_candidates(&ast, &arena, &env)
        .unwrap()
        .unwrap();
    let target_owner = env
        .symbols()
        .iter()
        .find(|entry| entry.primary_spelling() == "UsePriv1")
        .unwrap();
    let supported_owner = env
        .symbols()
        .iter()
        .find(|entry| entry.primary_spelling() == "Alpha")
        .unwrap();
    let formula = ast.node_view(original[target_owner.symbol()]).unwrap();
    let target = ast
        .node_views()
        .find(|node| {
            node.as_token()
                .is_some_and(|token| token.text.as_ref() == "X")
                && node.range().start >= formula.range().start
                && node.range().end <= formula.range().end
        })
        .unwrap();
    for mutation in [
        "unknown",
        "annotation",
        "string",
        "empty",
        "token_free",
        "error_token",
        "recovery",
    ] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut mapped = BTreeMap::new();
        for node in ast.node_views() {
            let mut children = node
                .children()
                .iter()
                .map(|child| mapped[child])
                .collect::<Vec<_>>();
            if node.id() == formula.id() && mutation == "token_free" {
                children.clear();
            }
            let id = if let Some(token) = node.as_token() {
                let mut kind = token.kind;
                let mut text = token.text.clone();
                if node.id() == target.id() {
                    match mutation {
                        "unknown" => kind = SurfaceTokenKind::Unknown,
                        "annotation" => kind = SurfaceTokenKind::AnnotationMarker,
                        "string" => kind = SurfaceTokenKind::StringLiteral,
                        "empty" => text = "".into(),
                        "error_token" | "recovery" => kind = SurfaceTokenKind::ErrorRecovery,
                        _ => {}
                    }
                }
                if node.id() == target.id() && mutation == "recovery" {
                    builder.add_recovered_token(kind, text, node.range())
                } else {
                    builder.add_token(kind, text, node.range())
                }
            } else {
                builder.add_node(node.kind().clone(), node.range(), children)
            };
            mapped.insert(node.id(), id);
        }
        let rebuilt = builder.finish(Some(mapped[&ast.root().unwrap()]), None);
        let shells =
            crate::declarations::DeclarationShellCollector::new(&rebuilt, &module).collect();
        let collected =
            crate::symbols::SignatureProjectionExtractor::new(&rebuilt, &shells, namespace.clone())
                .collect(&module);
        let env = collected.into_env();
        let arena = SurfaceResolvedArena::lower(&rebuilt, &module).unwrap();
        let before = (rebuilt.clone(), arena.clone(), env.clone());
        let upstream = source_public_theorem_formula_candidates(&rebuilt, &arena, &env);
        let result =
            crate::symbols::source_public_theorem_signature_candidates(&rebuilt, &arena, &env);
        if mutation == "recovery" {
            assert_eq!(result.unwrap_err(), upstream.unwrap_err());
        } else {
            let formulas = upstream.unwrap().unwrap();
            assert_eq!(formulas.len(), 2, "{mutation}");
            assert!(
                formulas.contains_key(supported_owner.symbol()),
                "{mutation}: supported public owner retained"
            );
            let supported = ast.node_view(original[supported_owner.symbol()]).unwrap();
            assert_eq!(
                rebuilt
                    .node_view(formulas[supported_owner.symbol()])
                    .unwrap()
                    .range(),
                supported.range()
            );
            assert_eq!(result.unwrap(), None, "{mutation}");
        }
        assert_eq!((rebuilt, arena, env), before);
    }
}

#[test]
fn source_theorem_module_candidates_compose_original_metadata_and_identities() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let quantified = "101:3:for100:1:X101:5:being101:3:set101:5:holds100:1:X102:1:=100:1:X";
    let numeral = "103:1:0102:1:=103:1:0";
    let many = (0..12)
        .map(|i| format!("theorem T{i}: 0 = 0\nproof\n  thus 0 = 0;\nend;\n"))
        .collect::<String>();
    for (text, public, citations, nonallocation_order) in [
        (fixture.to_owned(), vec![("UsePriv1".to_owned(), quantified)], vec![("UsePriv1", "PrivT1")], false),
        (format!("{fixture}\ntheorem Alpha: 0 = 0\nproof\n  thus 0 = 0 by UsePriv1;\nend;\n"), vec![("UsePriv1".to_owned(), quantified), ("Alpha".to_owned(), numeral)], vec![("UsePriv1", "PrivT1"), ("Alpha", "UsePriv1")], false),
        (fixture.replace("by PrivT1;", "by PrivT1, PrivT1;"), vec![("UsePriv1".to_owned(), quantified)], vec![("UsePriv1", "PrivT1"), ("UsePriv1", "PrivT1")], false),
        (fixture.replace("theorem UsePriv1", "lemma UsePriv1").replace("private ", ""), vec![("PrivT1".to_owned(), quantified), ("UsePriv1".to_owned(), quantified)], vec![("UsePriv1", "PrivT1")], false),
        (fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"), vec![], vec![("UsePriv1", "PrivT1")], false),
        (include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(), vec![("OpenT1".to_owned(), quantified), ("AxT1".to_owned(), quantified)], vec![], false),
        (many, (0..12).map(|i| (format!("T{i}"), numeral)).collect(), vec![], true),
    ] {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), arena.clone(), env.clone());
        if nonallocation_order {
            let ids = env.definitions().iter().map(|entry| entry.id().index()).collect::<Vec<_>>();
            assert_eq!(ids.len(), 12);
            assert!(ids.windows(2).any(|pair| pair[0] > pair[1]), "genuine iterator differs from allocation order");
        }
        let semantic = crate::resolved_ast::source_theorem_resolved_ast(&ast, &arena, &env).unwrap().unwrap();
        let labels = source_theorem_label_reference_candidates(&ast, &arena, &env).unwrap().unwrap();
        let mut signatures = BTreeMap::new();
        for (name, payload) in public {
            let owner = env.symbols().iter().find(|entry| entry.primary_spelling() == name).unwrap();
            assert_eq!(owner.visibility(), Visibility::Public);
            assert!(signatures.insert(owner.symbol().clone(), crate::env::SignatureShell::Opaque {
                schema: "source-theorem-statement-tokens-v1".to_owned(), payload: payload.to_owned(),
            }).is_none());
        }
        assert_eq!(crate::symbols::source_public_theorem_signature_candidates(&ast, &arena, &env).unwrap(), Some(signatures.clone()));
        crate::symbols::validate_source_symbol_env(&arena, &env).unwrap();
        let (actual, composed) = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env).unwrap().unwrap();
        assert_eq!(actual, semantic);
        assert_eq!(composed.module_id(), &module);
        assert_eq!(composed.labels(), labels.index());
        assert_eq!(actual.label_refs(), labels.table());
        assert_eq!(composed.imports(), env.imports());
        assert_eq!(composed.exports(), env.exports());
        assert_eq!(composed.overloads(), env.overloads());
        assert_eq!(composed.registrations(), env.registrations());
        assert_eq!(composed.lexical_summaries(), env.lexical_summaries());
        assert_eq!(composed.namespace_graph(), env.namespace_graph());
        assert_eq!(composed.module_summaries(), env.module_summaries());
        let mut expected_symbols = crate::env::SymbolIndex::new();
        for original in env.symbols().iter() {
            let expected = if let Some(signature) = signatures.get(original.symbol()) {
                original.clone().with_signature(signature.clone())
            } else {
                assert_eq!(original.visibility(), Visibility::Private);
                original.clone()
            };
            expected_symbols.insert(expected);
        }
        assert_eq!(composed.symbols(), &expected_symbols);
        let mut contributions = env.contributions().clone();
        for label in labels.index().iter() {
            let original = env.contributions().get(label.contribution()).unwrap();
            assert_eq!(original.module(), &module);
            contributions.add_label(label.contribution(), label.origin_path().clone());
        }
        let mut dependencies = env.declaration_dependencies().clone();
        let mut by_owner: BTreeMap<SymbolId, Vec<crate::env::DeclarationDependencyId>> = BTreeMap::new();
        assert_eq!(actual.label_refs().len(), citations.len());
        for ((_, row), (source_name, target_name)) in actual.label_refs().iter().zip(citations) {
            assert_eq!(row.site().spelling(), target_name);
            let source_owner = env.symbols().iter().find(|entry| entry.primary_spelling() == source_name).unwrap();
            let target_owner = env.symbols().iter().find(|entry| entry.primary_spelling() == target_name).unwrap();
            let reference = ast.node_views().find(|node| matches!(node.kind(), SurfaceNodeKind::Reference) && node.range() == row.site().range()).unwrap();
            let theorem = ast.node_views().find(|node| matches!(node.kind(), SurfaceNodeKind::TheoremItem | SurfaceNodeKind::LemmaItem) && node.range().start <= reference.range().start && node.range().end >= reference.range().end).unwrap();
            let statement = ast.node_views().find(|node| matches!(node.kind(), SurfaceNodeKind::ConclusionStatement) && node.range().start <= reference.range().start && node.range().end >= reference.range().end).unwrap();
            assert_eq!(row.origin().structural_path(), [theorem.id(), statement.id(), reference.id()].map(|node| u32::try_from(node.index()).unwrap()));
            assert_eq!(source_owner.origin().anchor(), &SourceAnchor::Range(theorem.range()));
            assert_eq!(row.site().node(), arena.resolved_node_for(reference.id()).unwrap());
            let target = crate::resolved_ast::LabelOriginPath::new(target_owner.symbol().fqn().as_str());
            assert_eq!(row.resolution(), &LabelResolution::Resolved(LabelRef::new(target.clone(), LabelKind::Theorem, reference.range())));
            assert_eq!(composed.labels().get(&target).unwrap().origin(), target_owner.origin());
            let edge = dependencies.insert(
                crate::env::DependencyEndpoint::Symbol(source_owner.symbol().clone()),
                crate::env::DependencyEndpoint::Label(target),
                crate::env::DeclarationDependencyKind::LabelCitation,
                SourceAnchor::Range(reference.range()), source_owner.contribution(),
            );
            contributions.add_declaration_dependency(source_owner.contribution(), edge);
            by_owner.entry(source_owner.symbol().clone()).or_default().push(edge);
            assert_eq!(composed.declaration_dependencies().get(edge), dependencies.get(edge));
        }
        assert_eq!(composed.declaration_dependencies(), &dependencies);
        assert_eq!(composed.contributions(), &contributions);
        assert_eq!(composed.definitions().len(), env.definitions().len());
        for original in env.definitions().iter() {
            let retained = composed.definitions().get(original.id()).unwrap();
            assert_eq!(retained.symbol(), original.symbol());
            assert_eq!(retained.kind(), original.kind());
            assert_eq!(retained.visibility(), original.visibility());
            assert_eq!(retained.parameters(), original.parameters());
            assert_eq!(retained.binders(), original.binders());
            assert_eq!(retained.arity(), original.arity());
            assert_eq!(retained.notation_shape(), original.notation_shape());
            assert_eq!(retained.doc_attachment(), original.doc_attachment());
            assert_eq!(retained.origin(), original.origin());
            assert_eq!(retained.contribution(), original.contribution());
            assert_eq!(retained.conflict(), original.conflict());
            assert_eq!(retained.signature(), signatures.get(original.symbol()).or(original.signature()));
            let mut expected = original.dependencies().to_vec();
            expected.extend(by_owner.remove(original.symbol()).unwrap_or_default());
            expected.sort();
            assert_eq!(retained.dependencies(), expected);
        }
        assert!(by_owner.is_empty());
        assert_eq!(crate::symbols::source_theorem_module_candidates(&ast, &arena, &env).unwrap(), Some((actual, composed.clone())));
        assert!(crate::symbols::validate_source_symbol_env(&arena, &composed).is_err());
        crate::symbols::validate_source_symbol_env(&arena, &env).unwrap();
        assert_eq!((ast, arena, env), before);
    }
}

#[test]
fn source_theorem_module_candidates_refuse_whole_pairs_and_preserve_lower_order() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for text in [
        fixture.replace("by PrivT1", "by Missing"),
        fixture.replace("by PrivT1", "by main.PrivT1"),
        format!("{fixture}\nreserve Y for set;\n"),
        format!("{fixture}\ntheorem Plain: 0 = 0;\n"),
    ] {
        let (ast, _, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), arena.clone(), env.clone());
        let lower = crate::resolved_ast::source_theorem_resolved_ast(&ast, &arena, &env).unwrap();
        if text.contains("by Missing") {
            let lower = lower.unwrap();
            assert!(
                lower
                    .label_refs()
                    .iter()
                    .any(|(_, row)| matches!(row.resolution(), LabelResolution::Unresolved(_)))
            );
            assert!(
                crate::symbols::source_public_theorem_signature_candidates(&ast, &arena, &env)
                    .unwrap()
                    .is_some()
            );
        } else {
            assert_eq!(lower, None);
        }
        assert_eq!(
            crate::symbols::source_theorem_module_candidates(&ast, &arena, &env).unwrap(),
            None
        );
        assert_eq!((ast, arena, env), before);
    }
    let (ast, _, _, env, arena) = parsed_private_theorem_fixture(fixture);
    let (foreign_ast, _, _, foreign_env, foreign_arena) =
        parsed_private_theorem_fixture(&fixture.replace("UsePriv1", "Changed1"));
    for (ast, arena, env) in [
        (&foreign_ast, &arena, &env),
        (&ast, &foreign_arena, &env),
        (&ast, &arena, &foreign_env),
    ] {
        let before = (ast.clone(), arena.clone(), env.clone());
        assert_eq!(
            crate::symbols::source_theorem_module_candidates(ast, arena, env).unwrap_err(),
            crate::resolved_ast::source_theorem_resolved_ast(ast, arena, env).unwrap_err()
        );
        assert_eq!((ast.clone(), arena.clone(), env.clone()), before);
    }
    let text = "theorem Numeral: 0 = 0\nproof\n  thus 0 = 0;\nend;\n";
    let (ast, module, namespace, _, _) = parsed_private_theorem_fixture(text);
    let target = ast
        .node_views()
        .find(|node| {
            node.as_token()
                .is_some_and(|token| token.kind == SurfaceTokenKind::Numeral)
        })
        .unwrap()
        .id();
    let mut builder = SurfaceAstBuilder::new(ast.source_id);
    let mut mapped = BTreeMap::new();
    for node in ast.node_views() {
        let children = node.children().iter().map(|child| mapped[child]).collect();
        let id = if let Some(token) = node.as_token() {
            builder.add_token(
                token.kind,
                if node.id() == target {
                    "".into()
                } else {
                    token.text.clone()
                },
                node.range(),
            )
        } else {
            builder.add_node(node.kind().clone(), node.range(), children)
        };
        mapped.insert(node.id(), id);
    }
    let rebuilt = builder.finish(Some(mapped[&ast.root().unwrap()]), None);
    let shells = crate::declarations::DeclarationShellCollector::new(&rebuilt, &module).collect();
    let collected = crate::symbols::SignatureProjectionExtractor::new(&rebuilt, &shells, namespace)
        .collect(&module);
    assert!(collected.diagnostics().is_empty());
    let env = collected.into_env();
    let arena = SurfaceResolvedArena::lower(&rebuilt, &module).unwrap();
    let before = (rebuilt.clone(), arena.clone(), env.clone());
    assert!(
        crate::resolved_ast::source_theorem_resolved_ast(&rebuilt, &arena, &env)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        crate::symbols::source_public_theorem_signature_candidates(&rebuilt, &arena, &env).unwrap(),
        None
    );
    assert_eq!(
        crate::symbols::source_theorem_module_candidates(&rebuilt, &arena, &env).unwrap(),
        None
    );
    assert_eq!((rebuilt, arena, env), before);
}

#[test]
fn source_theorem_module_storage_roundtrips_complete_pairs_and_rebinds() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let many = (0..12)
        .map(|i| format!("theorem T{i}: 0 = 0\nproof\n  thus 0 = 0;\nend;\n"))
        .collect::<String>();
    let mixed = format!(
        "{fixture}\ntheorem Alpha: 0 = 0\nproof\n  thus 0 = 0 by UsePriv1, PrivT1, PrivT1;\nend;\n"
    );
    let trivia = format!(
        "\u{feff}{}",
        fixture
            .replacen("proof\n", "proof\n  :: 日本語 trivia\n", 1)
            .replace('\n', "\r\n")
    );
    for text in [
        fixture.to_owned(),
        mixed,
        fixture.replace("theorem UsePriv1", "lemma UsePriv1"),
        fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
        include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(),
        many,
        trivia,
    ] {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), env.clone(), arena.clone());
        let expected = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env).unwrap().unwrap();
        if text.contains("theorem T11:") {
            let ids = env.definitions().iter().map(|entry| entry.id().index()).collect::<Vec<_>>();
            assert_eq!(ids.len(), 12);
            assert!(ids.windows(2).any(|pair| pair[0] > pair[1]));
        }
        let nested = ast.canonical_bytes().unwrap();
        let bytes = crate::symbols::source_theorem_module_candidates_canonical_bytes(&ast, &module).unwrap().unwrap();
        let mut literal = b"mizar-resolve/source-theorem-module-storage/v1\0\x03\0\0\0\0\0\0\0pkg\x04\0\0\0\0\0\0\0main".to_vec();
        literal.extend_from_slice(&u64::try_from(nested.len()).unwrap().to_le_bytes());
        literal.extend_from_slice(&nested);
        assert_eq!(bytes, literal);
        assert_eq!(crate::symbols::source_theorem_module_candidates_from_canonical_bytes(&bytes, ast.source_id, &module), Some(expected.clone()));
        assert_eq!(crate::symbols::source_theorem_module_candidates_canonical_bytes(&ast, &module).unwrap(), Some(bytes.clone()));
        let rebound_id = distinct_source_ids().1;
        assert_ne!(rebound_id, ast.source_id);
        // This is storage reconstruction under a genuine id, not a Frontend execution receipt.
        let rebound = SurfaceAst::from_canonical_bytes(&nested, rebound_id).unwrap();
        let shells = crate::declarations::DeclarationShellCollector::new(&rebound, &module).collect();
        let collected = crate::symbols::SignatureProjectionExtractor::new(&rebound, &shells, NamespacePath::new(module.path().as_str())).collect(&module);
        assert!(collected.diagnostics().is_empty());
        let rebound_arena = SurfaceResolvedArena::lower(&rebound, &module).unwrap();
        let rebound_expected = crate::symbols::source_theorem_module_candidates(&rebound, &rebound_arena, collected.env()).unwrap().unwrap();
        assert_ne!(rebound_expected, expected);
        assert_eq!(crate::symbols::source_theorem_module_candidates_from_canonical_bytes(&bytes, rebound_id, &module), Some(rebound_expected));
        assert_eq!(crate::symbols::source_theorem_module_candidates_canonical_bytes(&rebound, &module).unwrap(), Some(bytes));
        assert_eq!((ast, env, arena), before);
    }
}

#[test]
fn source_theorem_module_storage_rejects_lower_and_storage_failures() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for text in [
        fixture.replace("by PrivT1", "by Missing"),
        fixture.replace("by PrivT1", "by main.PrivT1"),
        format!("{fixture}\nreserve Y for set;\n"),
        fixture.replace("thus X = X;", "thus Y = X;"),
    ] {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), env.clone(), arena.clone());
        let lower = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env);
        if text.contains("thus Y = X;") {
            assert_eq!(lower.as_ref().unwrap_err(), "UnresolvedReference");
            // Oversize framing must refuse before replaying even an error-producing AST.
            let oversized = module_id(&"p".repeat(32 * 1024 * 1024), "main");
            assert_eq!(
                crate::symbols::source_theorem_module_candidates_canonical_bytes(&ast, &oversized)
                    .unwrap(),
                None
            );
        } else {
            assert_eq!(lower, Ok(None));
        }
        assert_eq!(
            crate::symbols::source_theorem_module_candidates_canonical_bytes(&ast, &module),
            lower.map(|pair| pair.map(|_| unreachable!()))
        );
        let nested = ast.canonical_bytes().unwrap();
        let mut bytes = b"mizar-resolve/source-theorem-module-storage/v1\0".to_vec();
        for field in [b"pkg".as_slice(), b"main".as_slice(), nested.as_slice()] {
            bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            bytes.extend_from_slice(field);
        }
        assert_eq!(
            crate::symbols::source_theorem_module_candidates_from_canonical_bytes(
                &bytes,
                ast.source_id,
                &module
            ),
            None
        );
        assert_eq!((ast, env, arena), before);
    }
    let (ast, module, _, env, arena) = parsed_private_theorem_fixture(fixture);
    let before = (ast.clone(), env.clone(), arena.clone());
    let bytes = crate::symbols::source_theorem_module_candidates_canonical_bytes(&ast, &module)
        .unwrap()
        .unwrap();
    for end in 0..bytes.len() {
        assert_eq!(
            crate::symbols::source_theorem_module_candidates_from_canonical_bytes(
                &bytes[..end],
                ast.source_id,
                &module
            ),
            None,
            "truncation {end}"
        );
    }
    for expected in [module_id("other", "main"), module_id("pkg", "other")] {
        assert_eq!(
            crate::symbols::source_theorem_module_candidates_from_canonical_bytes(
                &bytes,
                ast.source_id,
                &expected
            ),
            None
        );
    }
    let domain_len = b"mizar-resolve/source-theorem-module-storage/v1\0".len();
    for offset in [0, domain_len - 1, domain_len + 8, domain_len + 8 + 3 + 8] {
        let mut malformed = bytes.clone();
        malformed[offset] ^= 1;
        assert_eq!(
            crate::symbols::source_theorem_module_candidates_from_canonical_bytes(
                &malformed,
                ast.source_id,
                &module
            ),
            None
        );
    }
    for offset in [domain_len, domain_len + 8 + 3, domain_len + 8 + 3 + 8 + 4] {
        let mut malformed = bytes.clone();
        malformed[offset..offset + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(
            crate::symbols::source_theorem_module_candidates_from_canonical_bytes(
                &malformed,
                ast.source_id,
                &module
            ),
            None
        );
    }
    for suffix in [b"x".as_slice(), &[0; 8]] {
        let mut malformed = bytes.clone();
        malformed.extend_from_slice(suffix);
        assert_eq!(
            crate::symbols::source_theorem_module_candidates_from_canonical_bytes(
                &malformed,
                ast.source_id,
                &module
            ),
            None
        );
    }
    let nested = ast.canonical_bytes().unwrap();
    let schema = String::from_utf8(nested.clone())
        .unwrap()
        .replacen(
            "mizar-syntax/surface-ast/v1",
            "mizar-syntax/surface-ast/v2",
            1,
        )
        .into_bytes();
    let mut noncanonical = nested.clone();
    noncanonical.push(b' ');
    for nested in [vec![0xff], b"[]".to_vec(), schema, noncanonical] {
        assert!(SurfaceAst::from_canonical_bytes(&nested, ast.source_id).is_none());
        let mut malformed = bytes[..domain_len + 8 + 3 + 8 + 4].to_vec();
        malformed.extend_from_slice(&u64::try_from(nested.len()).unwrap().to_le_bytes());
        malformed.extend_from_slice(&nested);
        assert_eq!(
            crate::symbols::source_theorem_module_candidates_from_canonical_bytes(
                &malformed,
                ast.source_id,
                &module
            ),
            None
        );
    }
    let oversized_module = module_id("pkg", &"m".repeat(32 * 1024 * 1024));
    let mut oversized = b"mizar-resolve/source-theorem-module-storage/v1\0".to_vec();
    for field in [
        oversized_module.package().as_str().as_bytes(),
        oversized_module.path().as_str().as_bytes(),
        nested.as_slice(),
    ] {
        oversized.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
        oversized.extend_from_slice(field);
    }
    assert!(oversized.len() > 32 * 1024 * 1024);
    assert_eq!(
        crate::symbols::source_theorem_module_candidates_from_canonical_bytes(
            &oversized,
            ast.source_id,
            &oversized_module
        ),
        None
    );
    let mut inconsistent = ast.clone();
    inconsistent.source_id = distinct_source_ids().1;
    assert_ne!(inconsistent.source_id, ast.source_id);
    assert_eq!(inconsistent.canonical_bytes(), None);
    assert_eq!(
        crate::symbols::source_theorem_module_candidates_canonical_bytes(&inconsistent, &module)
            .unwrap(),
        None
    );
    assert_eq!((ast, env, arena), before);
}

#[test]
fn source_theorem_module_original_inputs_match_real_original_context() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let many = (0..12)
        .map(|i| format!("theorem T{i}: 0 = 0\nproof\n  thus 0 = 0;\nend;\n"))
        .collect::<String>();
    let trivia = format!(
        "\u{feff}{}",
        fixture
            .replacen("proof\n", "proof\n  :: 日本語 trivia\n", 1)
            .replace('\n', "\r\n")
    );
    for text in [
        fixture.to_owned(),
        fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
        fixture.replace("theorem UsePriv1", "lemma UsePriv1").replace("private ", ""),
        include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(),
        format!("{fixture}\ntheorem Alpha: 0 = 0\nproof\n  thus 0 = 0 by UsePriv1, PrivT1, PrivT1;\nend;\n"),
        many,
        trivia,
    ] {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), module.clone(), arena.clone(), env.clone());
        if text.contains("theorem T11:") {
            let ids = env.definitions().iter().map(|entry| entry.id().index()).collect::<Vec<_>>();
            assert_eq!(ids.len(), 12);
            assert!(ids.windows(2).any(|pair| pair[0] > pair[1]));
        }
        for expected_module in [module.clone(), module_id("other", "main"), module_id("pkg", "nested.main")] {
            let (expected_arena, expected_env) = if expected_module == module {
                (arena.clone(), env.clone())
            } else {
                let shells = crate::declarations::DeclarationShellCollector::new(&ast, &expected_module).collect();
                let collected = crate::symbols::SignatureProjectionExtractor::new(&ast, &shells, NamespacePath::new(expected_module.path().as_str())).collect(&expected_module);
                assert!(collected.diagnostics().is_empty());
                (SurfaceResolvedArena::lower(&ast, &expected_module).unwrap(), collected.into_env())
            };
            let expected_pair = crate::symbols::source_theorem_module_candidates(&ast, &expected_arena, &expected_env).unwrap().unwrap();
            if text == fixture {
                assert_ne!(&expected_pair.1, &expected_env);
            }
            let module_before = expected_module.clone();
            let (actual_arena, actual_env) = crate::symbols::source_theorem_module_original_inputs(&ast, &expected_module).unwrap().unwrap();
            assert_eq!(actual_arena, expected_arena);
            assert_eq!(actual_env, expected_env);
            assert_eq!(actual_arena.source_id(), ast.source_id);
            assert_eq!(actual_arena.module(), &expected_module);
            assert_eq!(actual_env.module_id(), &expected_module);
            assert_eq!(crate::symbols::source_theorem_module_candidates(&ast, &actual_arena, &actual_env).unwrap(), Some(expected_pair));
            assert_eq!(crate::symbols::source_theorem_module_original_inputs(&ast, &expected_module).unwrap(), Some((expected_arena, expected_env)));
            assert_eq!(expected_module, module_before);
        }
        assert_eq!((ast, module, arena, env), before);
    }
}

#[test]
fn source_theorem_module_original_inputs_preserve_constructor_and_admission_boundaries() {
    use mizar_frontend::{
        orchestration::Frontend, parsing::MizarParserSeam, source::FrontendSourceLoader,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, SourceInput, SourceOriginInput, normalize_path,
    };

    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for text in [
        fixture.replace("by PrivT1", "by Missing"),
        fixture.replace("by PrivT1", "by main.PrivT1"),
        format!("{fixture}\nreserve Y for set;\n"),
        format!("{fixture}\ntheorem Plain: 0 = 0;\n"),
        fixture.replace("thus X = X;", "thus Y = X;"),
    ] {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), module.clone(), arena.clone(), env.clone());
        let expected_pair = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env);
        if text.contains("thus Y = X;") {
            assert_eq!(expected_pair.as_ref().unwrap_err(), "UnresolvedReference");
        } else {
            assert_eq!(expected_pair, Ok(None));
        }
        let (actual_arena, actual_env) =
            crate::symbols::source_theorem_module_original_inputs(&ast, &module)
                .unwrap()
                .unwrap();
        assert_eq!(actual_arena, arena);
        assert_eq!(actual_env, env);
        assert_eq!(
            crate::symbols::source_theorem_module_candidates(&ast, &actual_arena, &actual_env),
            expected_pair
        );
        assert_eq!((ast, module, arena, env), before);
    }
    let root = std::env::temp_dir().join(format!(
        "mizar-resolve-original-inputs-duplicate-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = root.join("src/main.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        include_str!(
            "../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz"
        ),
    )
    .unwrap();
    let output = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        NoImportedSummaries,
        MizarParserSeam,
    )
    .run(
        mizar_frontend::source::SourceUnitRequest {
            snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                "mizar-session-build-snapshot-v1:{}",
                "31".repeat(Hash::BYTE_LEN)
            ))
            .unwrap(),
            input: SourceInput {
                package_id: PackageId::new("pkg"),
                module_path: ModulePath::new("main"),
                normalized_path: normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk { path },
            },
        },
        &InMemorySessionIdAllocator::new(),
    )
    .unwrap();
    std::fs::remove_dir_all(root).unwrap();
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output
        .ast
        .expect("duplicate theorem source has a parser AST");
    let module = module_id("pkg", "main");
    let before = (ast.clone(), module.clone());
    let shells = crate::declarations::DeclarationShellCollector::new(&ast, &module).collect();
    let collected = crate::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    assert!(!collected.diagnostics().is_empty());
    assert_eq!(
        crate::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap(),
        None
    );
    assert_eq!((ast, module), before);
}

#[test]
fn source_theorem_module_semantic_payload_matches_real_pair_and_exact_profile() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let quantified = "101:3:for100:1:X101:5:being101:3:set101:5:holds100:1:X102:1:=100:1:X";
    let many = (0..12)
        .map(|i| format!("theorem T{i}: 0 = 0\nproof\n  thus 0 = 0;\nend;\n"))
        .collect::<String>();
    let mixed = format!(
        "{fixture}\ntheorem Alpha: 0 = 12\nproof\n  thus 0 = 12 by UsePriv1, PrivT1, PrivT1;\nend;\n"
    );
    let trivia = format!(
        "\u{feff}{}",
        fixture
            .replacen("proof\n", "proof\n  :: 日本語 trivia\n", 1)
            .replace('\n', "\r\n")
    );
    for text in [fixture.to_owned(), fixture.replace("private ", ""), fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
        fixture.replace("theorem UsePriv1", "lemma UsePriv1"),
        include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(), mixed, many, trivia] {
        let (ast, module, _, original, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), module.clone(), original.clone(), arena.clone());
        if text.contains("theorem T11:") {
            let ids = original.definitions().iter().map(|row| row.id().index()).collect::<Vec<_>>();
            assert_eq!(ids.len(), 12);
            assert!(ids.windows(2).any(|pair| pair[0] > pair[1]));
        }
        let mut home_bytes = None;
        let mut home_fqns = None;
        for actual_module in [module.clone(), module_id("other", "main"), module_id("pkg", "nested.main")] {
            // Independent original collectors and the whole old pair precede the new API.
            let shells = crate::declarations::DeclarationShellCollector::new(&ast, &actual_module).collect();
            let collected = crate::symbols::SignatureProjectionExtractor::new(&ast, &shells, NamespacePath::new(actual_module.path().as_str())).collect(&actual_module);
            assert!(collected.diagnostics().is_empty());
            let old_arena = SurfaceResolvedArena::lower(&ast, &actual_module).unwrap();
            let expected = crate::symbols::source_theorem_module_candidates(&ast, &old_arena, collected.env()).unwrap().unwrap();
            let (resolved, env, bytes) = crate::symbols::source_theorem_module_semantic_payload(&ast, &actual_module).unwrap().unwrap();
            assert_eq!((&resolved, &env), (&expected.0, &expected.1));
            assert_eq!(crate::symbols::source_theorem_module_semantic_payload(&ast, &actual_module).unwrap(), Some((resolved.clone(), env.clone(), bytes.clone())));
            let fqns = env.symbols().iter().map(|row| row.symbol().fqn().as_str().to_owned()).collect::<Vec<_>>();
            if actual_module == module {
                home_bytes = Some(bytes.clone());
                home_fqns = Some(fqns);
            } else {
                assert_ne!(Some(&bytes), home_bytes.as_ref());
                assert_ne!(Some(&fqns), home_fqns.as_ref());
            }
            assert!(bytes.starts_with(b"mizar-resolve/source-theorem-module-semantics/v1\0"));
            if actual_module == module {
                assert!(bytes.starts_with(b"mizar-resolve/source-theorem-module-semantics/v1\0\x03\0\0\0\0\0\0\0pkg\x04\0\0\0\0\0\0\0main"));
            }
            let mut rest = &bytes[b"mizar-resolve/source-theorem-module-semantics/v1\0".len()..];
            let number = |input: &mut &[u8]| -> usize {
                let (word, tail) = input.split_at(8);
                *input = tail;
                usize::try_from(u64::from_le_bytes(word.try_into().unwrap())).unwrap()
            };
            let string = |input: &mut &[u8]| -> String {
                let length = number(input);
                let (value, tail) = input.split_at(length);
                *input = tail;
                String::from_utf8(value.to_vec()).unwrap()
            };
            let path = |input: &mut &[u8]| -> Vec<u32> {
                (0..number(input)).map(|_| u32::try_from(number(input)).unwrap()).collect()
            };
            assert_eq!(string(&mut rest), actual_module.package().as_str());
            assert_eq!(string(&mut rest), actual_module.path().as_str());
            assert_eq!(number(&mut rest), expected.0.nodes().root().index());
            assert_eq!(number(&mut rest), expected.0.nodes().len());
            let mut seen = Vec::new();
            for _ in 0..expected.0.nodes().len() {
                let id = number(&mut rest);
                let (_, row) = expected.0.nodes().iter().find(|(key, _)| key.index() == id).unwrap();
                seen.push(id);
                let tag = number(&mut rest);
                assert_eq!(tag, row.kind().syntax_kind() as usize);
                match row.kind() {
                    SurfaceNodeKind::Root => assert_eq!(tag, 1),
                    SurfaceNodeKind::TheoremItem => assert_eq!(tag, 96),
                    SurfaceNodeKind::QuantifiedFormula(mizar_syntax::SurfaceQuantifierKind::Universal) => assert_eq!(tag, 55),
                    SurfaceNodeKind::Token(token) => {
                        let token_tag = number(&mut rest);
                        assert_eq!(token_tag, token.kind.syntax_kind() as usize);
                        if token.kind == SurfaceTokenKind::Identifier { assert_eq!(token_tag, 100); }
                        assert_eq!(string(&mut rest), token.text.as_ref());
                    }
                    _ => {}
                }
                let children = (0..number(&mut rest)).map(|_| number(&mut rest)).collect::<Vec<_>>();
                assert_eq!(children, row.children().iter().map(|id| id.index()).collect::<Vec<_>>());
                let state = number(&mut rest);
                assert_eq!(state, match row.resolution() { crate::resolved_ast::NodeResolutionState::NotApplicable => 0, crate::resolved_ast::NodeResolutionState::Resolved => 1, _ => panic!("old pair state") });
                match number(&mut rest) {
                    0 => assert_eq!(row.reference_key(), None),
                    1 => assert!(matches!(row.reference_key(), Some(crate::resolved_ast::NodeReferenceKey::Name(key)) if key.index() == number(&mut rest))),
                    2 => assert!(matches!(row.reference_key(), Some(crate::resolved_ast::NodeReferenceKey::Label(key)) if key.index() == number(&mut rest))),
                    tag => panic!("unexpected reference tag {tag}"),
                }
                assert_eq!(path(&mut rest), row.origin().structural_path());
            }
            assert_eq!(seen, expected.0.nodes().iter().map(|(id, _)| id.index()).collect::<Vec<_>>());
            assert_eq!(number(&mut rest), expected.0.name_refs().len());
            let mut seen = Vec::new();
            for _ in 0..expected.0.name_refs().len() {
                let id = number(&mut rest);
                let (_, row) = expected.0.name_refs().iter().find(|(key, _)| key.index() == id).unwrap();
                seen.push(id);
                let crate::resolved_ast::NameResolution::ResolvedBuiltin(target) = row.resolution() else { panic!("old builtin") };
                assert_eq!(number(&mut rest), row.site().node().index());
                assert_eq!(string(&mut rest), row.site().spelling());
                assert_eq!(string(&mut rest), target.builtin().as_str());
                assert_eq!(string(&mut rest), target.spelling());
                assert_eq!(path(&mut rest), row.origin().structural_path());
            }
            assert_eq!(seen, expected.0.name_refs().iter().map(|(id, _)| id.index()).collect::<Vec<_>>());
            assert_eq!(number(&mut rest), expected.0.label_refs().len());
            let mut seen = Vec::new();
            for _ in 0..expected.0.label_refs().len() {
                let id = number(&mut rest);
                let (_, row) = expected.0.label_refs().iter().find(|(key, _)| key.index() == id).unwrap();
                seen.push(id);
                let LabelResolution::Resolved(target) = row.resolution() else { panic!("old label") };
                assert_eq!(number(&mut rest), row.site().node().index());
                assert_eq!(string(&mut rest), row.site().spelling());
                assert_eq!(string(&mut rest), target.origin().as_str());
                assert_eq!(path(&mut rest), row.origin().structural_path());
            }
            assert_eq!(seen, expected.0.label_refs().iter().map(|(id, _)| id.index()).collect::<Vec<_>>());
            assert_eq!(number(&mut rest), expected.1.symbols().len());
            let mut seen = Vec::new();
            for _ in 0..expected.1.symbols().len() {
                let fqn = string(&mut rest);
                let row = expected.1.symbols().iter().find(|row| row.symbol().fqn().as_str() == fqn).unwrap();
                seen.push(fqn);
                assert_eq!(number(&mut rest), usize::from(row.kind() == crate::env::SymbolKind::Lemma));
                assert_eq!(number(&mut rest), usize::from(row.visibility() == Visibility::Public));
                assert_eq!(number(&mut rest), usize::from(row.export_status() == ExportStatus::Exported));
                assert_eq!(string(&mut rest), row.namespace().as_str());
                assert_eq!(string(&mut rest), row.primary_spelling());
                assert_eq!(number(&mut rest), row.contribution().index());
                let Some(crate::env::SignatureShell::Opaque { schema, payload }) = row.signature() else { panic!("old signature") };
                assert_eq!(string(&mut rest), *schema);
                assert_eq!(string(&mut rest), *payload);
                if row.visibility() == Visibility::Public {
                    assert_eq!(schema, "source-theorem-statement-tokens-v1");
                    assert_eq!(payload, if row.primary_spelling() == "Alpha" { "103:1:0102:1:=103:2:12" } else if row.primary_spelling().starts_with('T') { "103:1:0102:1:=103:1:0" } else { quantified });
                } else { assert_eq!(schema, "parser-signature-v1"); }
                assert_eq!(path(&mut rest), row.origin().structural_path());
            }
            assert_eq!(seen, expected.1.symbols().iter().map(|row| row.symbol().fqn().as_str().to_owned()).collect::<Vec<_>>());
            assert_eq!(number(&mut rest), expected.1.labels().len());
            let mut seen = Vec::new();
            for _ in 0..expected.1.labels().len() {
                let key = string(&mut rest);
                let row = expected.1.labels().iter().find(|row| row.origin_path().as_str() == key).unwrap();
                seen.push(key);
                assert_eq!(number(&mut rest), usize::from(row.visibility() == Visibility::Public));
                assert_eq!(number(&mut rest), usize::from(row.export_status() == ExportStatus::Exported));
                assert_eq!(string(&mut rest), row.namespace().as_str());
                assert_eq!(string(&mut rest), row.primary_spelling());
                assert_eq!(number(&mut rest), row.contribution().index());
                assert_eq!(path(&mut rest), row.origin().structural_path());
            }
            assert_eq!(seen, expected.1.labels().iter().map(|row| row.origin_path().as_str().to_owned()).collect::<Vec<_>>());
            assert_eq!(number(&mut rest), expected.1.definitions().len());
            let mut seen = Vec::new();
            for _ in 0..expected.1.definitions().len() {
                let id = number(&mut rest);
                let row = expected.1.definitions().iter().find(|row| row.id().index() == id).unwrap();
                seen.push(id);
                assert_eq!(string(&mut rest), row.symbol().fqn().as_str());
                assert_eq!(number(&mut rest), usize::from(row.kind() == crate::env::DefinitionKind::Lemma));
                assert_eq!(number(&mut rest), usize::from(row.visibility() == Visibility::Public));
                assert_eq!(number(&mut rest), row.contribution().index());
                let Some(crate::env::SignatureShell::Opaque { schema, payload }) = row.signature() else { panic!("old definition signature") };
                assert_eq!(string(&mut rest), *schema);
                assert_eq!(string(&mut rest), *payload);
                assert_eq!(path(&mut rest), row.origin().structural_path());
                let dependencies = (0..number(&mut rest)).map(|_| number(&mut rest)).collect::<Vec<_>>();
                assert_eq!(dependencies, row.dependencies().iter().map(|key| key.index()).collect::<Vec<_>>());
            }
            assert_eq!(seen, expected.1.definitions().iter().map(|row| row.id().index()).collect::<Vec<_>>());
            assert_eq!(number(&mut rest), expected.1.declaration_dependencies().len());
            let mut seen = Vec::new();
            for _ in 0..expected.1.declaration_dependencies().len() {
                let id = number(&mut rest);
                let row = expected.1.declaration_dependencies().iter().find(|row| row.id().index() == id).unwrap();
                seen.push(id);
                let (crate::env::DependencyEndpoint::Symbol(owner), crate::env::DependencyEndpoint::Label(target)) = (row.source(), row.target()) else { panic!("old citation") };
                assert_eq!(string(&mut rest), owner.fqn().as_str());
                assert_eq!(string(&mut rest), target.as_str());
                assert_eq!(number(&mut rest), row.contribution().index());
                assert_eq!(row.kind(), crate::env::DeclarationDependencyKind::LabelCitation);
            }
            assert_eq!(seen, expected.1.declaration_dependencies().iter().map(|row| row.id().index()).collect::<Vec<_>>());
            assert_eq!(number(&mut rest), expected.1.contributions().iter().count());
            let mut seen = Vec::new();
            for _ in 0..expected.1.contributions().iter().count() {
                let id = number(&mut rest);
                let row = expected.1.contributions().iter().find(|row| row.id().index() == id).unwrap();
                seen.push(id);
                let effects = row.effects();
                let symbols = (0..number(&mut rest)).map(|_| string(&mut rest)).collect::<Vec<_>>();
                let definitions = (0..number(&mut rest)).map(|_| number(&mut rest)).collect::<Vec<_>>();
                let labels = (0..number(&mut rest)).map(|_| string(&mut rest)).collect::<Vec<_>>();
                let dependencies = (0..number(&mut rest)).map(|_| number(&mut rest)).collect::<Vec<_>>();
                assert_eq!(symbols, effects.symbols().iter().map(|key| key.fqn().as_str().to_owned()).collect::<Vec<_>>());
                assert_eq!(definitions, effects.definitions().iter().map(|key| key.index()).collect::<Vec<_>>());
                assert_eq!(labels, effects.labels().iter().map(|key| key.as_str().to_owned()).collect::<Vec<_>>());
                assert_eq!(dependencies, effects.declaration_dependencies().iter().map(|key| key.index()).collect::<Vec<_>>());
                assert!(matches!(row.kind(), ContributionKind::LocalSource { source_id } if *source_id == ast.source_id));
            }
            assert_eq!(seen, expected.1.contributions().iter().map(|row| row.id().index()).collect::<Vec<_>>());
            assert!(rest.is_empty());
            if let Some(storage) = crate::symbols::source_theorem_module_candidates_canonical_bytes(&ast, &actual_module).unwrap() {
                assert_eq!(crate::symbols::source_theorem_module_candidates_from_canonical_bytes(&storage, ast.source_id, &actual_module), Some(expected.clone()));
            }
            let rebound_id = distinct_source_ids().1;
            assert_ne!(rebound_id, ast.source_id);
            let rebound = SurfaceAst::from_canonical_bytes(&ast.canonical_bytes().unwrap(), rebound_id).unwrap();
            let rebound_shells = crate::declarations::DeclarationShellCollector::new(&rebound, &actual_module).collect();
            let rebound_collected = crate::symbols::SignatureProjectionExtractor::new(&rebound, &rebound_shells, NamespacePath::new(actual_module.path().as_str())).collect(&actual_module);
            assert!(rebound_collected.diagnostics().is_empty());
            let rebound_arena = SurfaceResolvedArena::lower(&rebound, &actual_module).unwrap();
            let rebound_expected = crate::symbols::source_theorem_module_candidates(&rebound, &rebound_arena, rebound_collected.env()).unwrap().unwrap();
            assert_ne!(rebound_expected, expected);
            let (rebound_resolved, rebound_env, rebound_bytes) = crate::symbols::source_theorem_module_semantic_payload(&rebound, &actual_module).unwrap().unwrap();
            assert_eq!((rebound_resolved, rebound_env), rebound_expected);
            assert_eq!(rebound_bytes, bytes);
        }
        assert_eq!((ast, module, original, arena), before);
    }
}

#[test]
fn source_theorem_module_semantic_payload_preserves_errors_and_participation() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for text in [
        fixture.replace("by PrivT1", "by Missing"),
        fixture.replace("by PrivT1", "by main.PrivT1"),
        format!("{fixture}\nreserve Y for set;\n"),
        format!("{fixture}\ntheorem Plain: 0 = 0;\n"),
        fixture.replace("thus X = X;", "thus Y = X;"),
    ] {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), module.clone(), env.clone(), arena.clone());
        let old = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env);
        let new = crate::symbols::source_theorem_module_semantic_payload(&ast, &module);
        if text.contains("thus Y = X;") {
            assert_eq!(old.as_ref().unwrap_err(), "UnresolvedReference");
            assert_eq!(new.as_ref().unwrap_err(), old.as_ref().unwrap_err());
            let oversized = module_id(&"p".repeat(32 * 1024 * 1024), "main");
            assert_eq!(
                crate::symbols::source_theorem_module_semantic_payload(&ast, &oversized).unwrap(),
                None
            );
        } else {
            assert_eq!(old, Ok(None));
            assert_eq!(new, Ok(None));
        }
        assert_eq!((ast, module, env, arena), before);
    }
    // Negative only: genuine parsed nodes with the second same-width owner restored.
    let duplicate = include_str!(
        "../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz"
    );
    let collision_free = duplicate.replacen("\ntheorem Clash", "\ntheorem Other", 1);
    let (ast, module, _, _, _) = parsed_private_theorem_fixture(&collision_free);
    let mut builder = SurfaceAstBuilder::new(ast.source_id);
    let mut rebuilt = Vec::new();
    let mut replacements = 0;
    for node in ast.node_views() {
        let children = node
            .children()
            .iter()
            .map(|id| rebuilt[id.index()])
            .collect::<Vec<_>>();
        let id = if let Some(token) = node.as_token() {
            if token.text.as_ref() == "Other" {
                replacements += 1;
                builder.add_token(token.kind, "Clash", node.range())
            } else {
                builder.add_token(token.kind, token.text.clone(), node.range())
            }
        } else {
            builder.add_node(node.kind().clone(), node.range(), children)
        };
        rebuilt.push(id);
    }
    assert_eq!(replacements, 1);
    let negative = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
    let before = (negative.clone(), module.clone());
    let shells = crate::declarations::DeclarationShellCollector::new(&negative, &module).collect();
    let collected = crate::symbols::SignatureProjectionExtractor::new(
        &negative,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    assert!(!collected.diagnostics().is_empty());
    assert_eq!(
        crate::symbols::source_theorem_module_semantic_payload(&negative, &module).unwrap(),
        None
    );
    assert_eq!((negative, module), before);
    let (base_ast, module, _, base_env, base_arena) = parsed_private_theorem_fixture(fixture);
    let baseline =
        crate::symbols::source_theorem_module_candidates(&base_ast, &base_arena, &base_env)
            .unwrap()
            .unwrap();
    let baseline_owner = baseline
        .1
        .symbols()
        .iter()
        .find(|row| row.primary_spelling() == "UsePriv1")
        .unwrap();
    assert_eq!(baseline_owner.kind(), crate::env::SymbolKind::Theorem);
    assert_eq!(baseline_owner.visibility(), Visibility::Public);
    let baseline_bytes = crate::symbols::source_theorem_module_semantic_payload(&base_ast, &module)
        .unwrap()
        .unwrap()
        .2;
    for text in [
        fixture.replace("proof\n", "proof\n :: 日本語\n"),
        fixture.replace("X = X", "X  =  X"),
        format!("\u{feff}{}", fixture.replace('\n', "\r\n")),
    ] {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let old = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env)
            .unwrap()
            .unwrap();
        let (resolved, symbols, bytes) =
            crate::symbols::source_theorem_module_semantic_payload(&ast, &module)
                .unwrap()
                .unwrap();
        assert_eq!((resolved, symbols), old);
        assert_eq!(bytes, baseline_bytes);
    }
    for (case, text) in [
        ("owner", fixture.replace("PrivT1", "PrivT2")),
        (
            "role",
            fixture.replace("theorem UsePriv1", "lemma UsePriv1"),
        ),
        (
            "visibility",
            fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
        ),
        ("binder", fixture.replace('X', "Y")),
        (
            "proof",
            fixture
                .replace("let X be set;", "let Y be set;")
                .replace("thus X = X", "thus Y = Y"),
        ),
    ] {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let old = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env)
            .unwrap()
            .unwrap();
        let use_owner = old
            .1
            .symbols()
            .iter()
            .find(|row| row.primary_spelling() == "UsePriv1")
            .unwrap();
        match case {
            "owner" => assert!(
                old.1
                    .labels()
                    .iter()
                    .any(|row| row.primary_spelling() == "PrivT2")
            ),
            "role" => assert_eq!(use_owner.kind(), crate::env::SymbolKind::Lemma),
            "visibility" => assert_eq!(use_owner.visibility(), Visibility::Private),
            "binder" | "proof" => assert!(old.0.nodes().iter().any(|(_, row)| matches!(
                row.kind(), SurfaceNodeKind::Token(token)
                    if token.kind == SurfaceTokenKind::Identifier && token.text.as_ref() == "Y"
            ))),
            _ => unreachable!(),
        }
        let (resolved, symbols, bytes) =
            crate::symbols::source_theorem_module_semantic_payload(&ast, &module)
                .unwrap()
                .unwrap();
        assert_eq!((resolved, symbols), old);
        assert_ne!(bytes, baseline_bytes, "{case}");
    }
    let alpha = "theorem Alpha: 0 = 0\nproof\n thus 0 = 0;\nend;\n";
    let beta = "theorem Beta: 0 = 0\nproof\n thus 0 = 0;\nend;\n";
    let citing =
        format!("{alpha}{beta}theorem Use: 0 = 0\nproof\n thus 0 = 0 by Alpha, Beta;\nend;\n");
    for (left, right, case) in [
        (
            alpha.to_owned(),
            alpha.replace("0 = 0", "0 = 12"),
            "numeral",
        ),
        (
            format!("{alpha}{beta}"),
            format!("{beta}{alpha}"),
            "declaration order",
        ),
        (
            citing.clone(),
            citing.replace("by Alpha, Beta", "by Beta, Alpha"),
            "citation order",
        ),
    ] {
        let mut pairs = Vec::new();
        let mut streams = Vec::new();
        for text in [left, right] {
            let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
            let old = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env)
                .unwrap()
                .unwrap();
            let (resolved, symbols, bytes) =
                crate::symbols::source_theorem_module_semantic_payload(&ast, &module)
                    .unwrap()
                    .unwrap();
            assert_eq!((resolved, symbols), old);
            pairs.push(old);
            streams.push(bytes);
        }
        match case {
            "numeral" => assert_ne!(
                pairs[0].1.symbols().iter().next().unwrap().signature(),
                pairs[1].1.symbols().iter().next().unwrap().signature()
            ),
            "declaration order" => {
                for (pair, expected_id) in pairs.iter().zip([0, 1]) {
                    let (_, env) = pair;
                    let owner = env
                        .symbols()
                        .iter()
                        .find(|row| row.primary_spelling() == "Alpha")
                        .unwrap();
                    let definition = env.definitions().by_symbol(owner.symbol()).unwrap();
                    assert_eq!(definition.id().index(), expected_id);
                }
            }
            "citation order" => {
                for (pair, names) in pairs.iter().zip([["Alpha", "Beta"], ["Beta", "Alpha"]]) {
                    let actual = pair
                        .0
                        .label_refs()
                        .iter()
                        .map(|(_, row)| row.site().spelling())
                        .collect::<Vec<_>>();
                    assert_eq!(actual, names);
                }
                assert_ne!(
                    pairs[0].1.declaration_dependencies(),
                    pairs[1].1.declaration_dependencies()
                );
            }
            _ => unreachable!(),
        }
        assert_ne!(streams[0], streams[1], "{case}");
    }
}

#[test]
fn source_public_theorem_export_candidates_preserve_real_surface_and_identity() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let identity = mizar_artifact::module_summary::ModuleSummaryIdentity {
        package_id: "pkg".to_owned(),
        package_version: Some("1.2.3".to_owned()),
        lockfile_identity: Some("lock:source-test".to_owned()),
        module_path: "main".to_owned(),
        language_edition: "2025".to_owned(),
    };
    let quantified = "theorem UsePriv1 : for X being set holds X = X";
    let statuses = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
    );
    let mixed = "lemma Zed: 12 = 0\nproof\n  thus 12 = 0;\nend;\ntheorem Alpha: 0 = 12\nproof\n  thus 0 = 12 by Zed, Zed;\nend;\nprivate theorem Hidden: 0 = 0\nproof\n  thus 0 = 0;\nend;\n";
    let schema = mizar_artifact::module_summary::current_schema_version();
    let mut baseline = None;
    let mut baseline_source = None;
    for case in 0..13 {
        let text = match case {
            12 => fixture.replacen("private theorem", "public theorem", 1),
            1 => fixture.replace("theorem UsePriv1", "lemma UsePriv1"),
            2 => fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
            3 => mixed.to_owned(),
            4 => statuses.to_owned(),
            5 => fixture.replace("by PrivT1;", "by PrivT1, PrivT1;"),
            6 => format!(
                ":: λ\n{}",
                fixture.replace("holds X = X", "holds\n X  =  X :: trivia\n")
            ),
            7 => fixture.replace("holds X = X", "holds X <> X"),
            8 => statuses.replace("open theorem", "assumed theorem"),
            _ => fixture.to_owned(),
        };
        let expected = match case {
            12 => vec![
                ("PrivT1", "theorem PrivT1 : for X being set holds X = X"),
                ("UsePriv1", quantified),
            ],
            1 => vec![("UsePriv1", "lemma UsePriv1 : for X being set holds X = X")],
            2 => vec![],
            3 => vec![
                ("Zed", "lemma Zed : 12 = 0"),
                ("Alpha", "theorem Alpha : 0 = 12"),
            ],
            4 => vec![
                (
                    "OpenT1",
                    "open theorem OpenT1 : for X being set holds X = X",
                ),
                ("AxT1", "assumed theorem AxT1 : for X being set holds X = X"),
            ],
            8 => vec![
                (
                    "OpenT1",
                    "assumed theorem OpenT1 : for X being set holds X = X",
                ),
                ("AxT1", "assumed theorem AxT1 : for X being set holds X = X"),
            ],
            7 => vec![(
                "UsePriv1",
                "theorem UsePriv1 : for X being set holds X <> X",
            )],
            _ => vec![("UsePriv1", quantified)],
        };
        let (package, path) = match case {
            9 => ("other-package", "main"),
            10 => ("pkg", "Nested.Main"),
            _ => ("pkg", "main"),
        };
        let (mut ast, module, namespace, env, arena) = parsed_private_theorem_fixture(&text);
        if case == 11 {
            let root = std::env::temp_dir()
                .join(format!("mizar-export-relocation-{}", std::process::id()));
            let path = root.join("src/main.miz");
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &text).unwrap();
            let allocator = InMemorySessionIdAllocator::new();
            let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
                "mizar-session-build-snapshot-v1:{}",
                "31".repeat(Hash::BYTE_LEN)
            ))
            .unwrap();
            let skipped = allocator.next_source_id(snapshot).unwrap();
            let output = mizar_frontend::orchestration::Frontend::new(
                mizar_frontend::source::FrontendSourceLoader::new(
                    mizar_session::DiskSourceLoader::new(&root),
                ),
                NoImportedSummaries,
                mizar_frontend::parsing::MizarParserSeam,
            )
            .run(
                mizar_frontend::source::SourceUnitRequest {
                    snapshot,
                    input: mizar_session::SourceInput {
                        package_id: PackageId::new("pkg"),
                        module_path: ModulePath::new("main"),
                        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                        edition: mizar_session::Edition::new("2025"),
                        origin: mizar_session::SourceOriginInput::Disk { path },
                    },
                },
                &allocator,
            )
            .unwrap();
            std::fs::remove_dir_all(root).unwrap();
            assert!(output.diagnostics.is_empty());
            let relocated = output.ast.unwrap();
            assert_ne!(relocated.source_id, skipped);
            assert_ne!(relocated.source_id, ast.source_id);
            ast = relocated;
        }
        // Alternate package/path are honest supplied metadata with real recollection.
        let (module, namespace, env, arena) = if matches!(case, 9..=11) {
            let module = module_id(package, path);
            let namespace = NamespacePath::new(path);
            let shells =
                crate::declarations::DeclarationShellCollector::new(&ast, &module).collect();
            let env =
                crate::symbols::SignatureProjectionExtractor::new(&ast, &shells, namespace.clone())
                    .collect(&module)
                    .into_env();
            let arena = SurfaceResolvedArena::lower(&ast, &module).unwrap();
            (module, namespace, env, arena)
        } else {
            (module, namespace, env, arena)
        };
        let identity = mizar_artifact::module_summary::ModuleSummaryIdentity {
            package_id: package.to_owned(),
            module_path: path.to_owned(),
            ..identity.clone()
        };
        let before = (ast.clone(), arena.clone(), env.clone(), identity.clone());
        let formulas = source_public_theorem_formula_candidates(&ast, &arena, &env)
            .unwrap()
            .unwrap();
        assert_eq!(formulas.len(), expected.len(), "{case}");
        let mut expected_rows = Vec::new();
        for (name, header) in expected {
            let kind = if header.starts_with("lemma ") {
                "lemma"
            } else {
                "theorem"
            };
            let owner = env
                .symbols()
                .iter()
                .find(|owner| owner.primary_spelling() == name)
                .unwrap();
            assert_eq!(owner.symbol().module(), &module);
            assert_eq!(owner.namespace(), &namespace);
            assert_eq!(namespace.as_str(), path);
            assert_eq!(owner.visibility(), Visibility::Public);
            assert_eq!(owner.export_status(), ExportStatus::Exported);
            let expected_kind = match kind {
                "lemma" => crate::env::SymbolKind::Lemma,
                _ => crate::env::SymbolKind::Theorem,
            };
            assert_eq!(owner.kind(), expected_kind);
            let formula = ast.node_view(formulas[owner.symbol()]).unwrap();
            assert!(formula.as_formula_expression().is_some());
            // Expected byte boundaries come from the literal source, not a row traversal.
            let prefix = format!("{}:", header.split(" : ").next().unwrap());
            let start = text.find(&prefix).unwrap();
            let end = if header.starts_with("open ") || header.starts_with("assumed ") {
                start + text[start..].find(';').unwrap() + 1
            } else {
                start + text[start..].find("end;").unwrap() + 4
            };
            assert_eq!(
                owner.origin().anchor(),
                &SourceAnchor::Range(SourceRange {
                    source_id: ast.source_id,
                    start,
                    end
                })
            );
            assert!(formula.range().start > start && formula.range().end < end);
            let fqn = owner.symbol().fqn().as_str();
            let symbol_json = mizar_artifact::store::CanonicalJson::Object(
                [
                    ("origin_id", fqn),
                    ("fully_qualified_name", fqn),
                    ("visibility", "public"),
                    ("declaration_kind", kind),
                    ("rendered_signature", header),
                ]
                .into_iter()
                .map(|(k, v)| (k.into(), mizar_artifact::store::CanonicalJson::string(v)))
                .chain([
                    (
                        "namespace_path".to_owned(),
                        mizar_artifact::store::CanonicalJson::array(
                            path.split('.')
                                .map(mizar_artifact::store::CanonicalJson::string),
                        ),
                    ),
                    (
                        "proof_status".to_owned(),
                        mizar_artifact::store::CanonicalJson::Null,
                    ),
                ])
                .collect(),
            );
            let module_json = mizar_artifact::store::CanonicalJson::Object(
                [
                    ("package_id", package),
                    ("package_version", "1.2.3"),
                    ("lockfile_identity", "lock:source-test"),
                    ("module_path", path),
                    ("language_edition", "2025"),
                ]
                .into_iter()
                .map(|(k, v)| (k.into(), mizar_artifact::store::CanonicalJson::string(v)))
                .collect(),
            );
            let projection = mizar_artifact::store::CanonicalJson::object([
                ("module", module_json),
                ("symbol", symbol_json),
            ])
            .unwrap();
            let fingerprint = mizar_artifact::store::CanonicalHashDomain::new(
                mizar_artifact::store::HashClass::Interface,
                mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
                schema,
            )
            .hash(&projection, &[]);
            expected_rows.push(mizar_artifact::module_summary::ExportedSymbolSummary {
                origin_id: fqn.to_owned(),
                fully_qualified_name: fqn.to_owned(),
                namespace_path: path.split('.').map(str::to_owned).collect(),
                visibility: "public".to_owned(),
                declaration_kind: kind.to_owned(),
                source_range: mizar_artifact::module_summary::SourceRangeSummary {
                    start_byte: start as u64,
                    end_byte: end as u64,
                },
                rendered_signature: header.to_owned(),
                interface_fingerprint: fingerprint,
                proof_status: None,
            });
        }
        let rows =
            crate::symbols::source_public_theorem_export_candidates(&ast, &arena, &env, &identity)
                .unwrap()
                .unwrap();
        assert_eq!(rows, expected_rows, "{case}");
        assert_eq!(
            crate::symbols::source_public_theorem_export_candidates(&ast, &arena, &env, &identity)
                .unwrap(),
            Some(rows.clone())
        );
        if case == 0 {
            baseline = Some(rows.clone());
            baseline_source = Some(ast.source_id);
        }
        if case == 11 {
            assert_eq!(&rows, &baseline.as_ref().unwrap()[..1]);
            assert_ne!(ast.source_id, baseline_source.unwrap());
        }
        if matches!(case, 9 | 10) {
            assert_ne!(
                rows[0].interface_fingerprint,
                baseline.as_ref().unwrap()[0].interface_fingerprint
            );
        }
        if matches!(case, 5..=7) {
            let original = &baseline.as_ref().unwrap()[0];
            assert_ne!(rows[0].source_range, original.source_range);
            if case == 7 {
                assert_ne!(rows[0].rendered_signature, original.rendered_signature);
                assert_ne!(
                    rows[0].interface_fingerprint,
                    original.interface_fingerprint
                );
            } else {
                let mut relocated = rows[0].clone();
                relocated.source_range = original.source_range;
                assert_eq!(&relocated, original);
            }
        }
        if case == 4 || case == 8 {
            let open = rows
                .iter()
                .find(|row| row.rendered_signature.contains("OpenT1"))
                .unwrap();
            let stored = baseline.as_mut().unwrap();
            if case == 4 {
                stored.push(open.clone());
            } else {
                assert_ne!(open.rendered_signature, stored[1].rendered_signature);
                assert_ne!(open.interface_fingerprint, stored[1].interface_fingerprint);
            }
        }
        for field in 0..3 {
            let mut supplied = identity.clone();
            match field {
                0 => supplied.package_version = Some("2.0.0".to_owned()),
                1 => supplied.lockfile_identity = Some("lock:other".to_owned()),
                2 => supplied.language_edition = "2027".to_owned(),
                _ => unreachable!(),
            }
            let changed = crate::symbols::source_public_theorem_export_candidates(
                &ast, &arena, &env, &supplied,
            )
            .unwrap()
            .unwrap();
            assert_eq!(changed.len(), rows.len());
            for (actual, original) in changed.iter().zip(&rows) {
                assert_ne!(actual.interface_fingerprint, original.interface_fingerprint);
                let mut without_hash = actual.clone();
                without_hash.interface_fingerprint = original.interface_fingerprint;
                assert_eq!(&without_hash, original);
            }
        }
        assert_eq!((ast, arena, env, identity.clone()), before);
    }
}

#[test]
fn source_public_theorem_export_candidates_preserve_refusals_and_boundaries() {
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let identity = mizar_artifact::module_summary::ModuleSummaryIdentity {
        package_id: "pkg".to_owned(),
        package_version: None,
        lockfile_identity: None,
        module_path: "main".to_owned(),
        language_edition: "2025".to_owned(),
    };
    let invalid = mizar_artifact::module_summary::ModuleSummaryIdentity {
        language_edition: String::new(),
        ..identity.clone()
    };
    for text in [
        format!("{fixture}\nreserve Y for set;\n"),
        format!("{fixture}\ntheorem Plain: 0 = 0;\n"),
        fixture.replacen("private theorem", "public lemma", 1),
    ] {
        let (ast, _, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), arena.clone(), env.clone());
        let lower = source_public_theorem_formula_candidates(&ast, &arena, &env);
        assert_eq!(lower, Ok(None));
        for supplied in [&identity, &invalid] {
            assert_eq!(
                crate::symbols::source_public_theorem_export_candidates(
                    &ast, &arena, &env, supplied
                ),
                Ok(None)
            );
        }
        assert_eq!((ast, arena, env), before);
    }
    let (ast, module, namespace, env, arena) = parsed_private_theorem_fixture(fixture);
    let (foreign, _, _, foreign_env, foreign_arena) =
        parsed_private_theorem_fixture(&fixture.replace("UsePriv1", "Changed1"));
    let augmented = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env)
        .unwrap()
        .unwrap()
        .1;
    for (ast, arena, env) in [
        (&foreign, &arena, &env),
        (&ast, &foreign_arena, &env),
        (&ast, &arena, &foreign_env),
        (&ast, &arena, &augmented),
    ] {
        let before = (ast.clone(), arena.clone(), env.clone());
        let lower = source_public_theorem_formula_candidates(ast, arena, env).unwrap_err();
        for supplied in [&identity, &invalid] {
            assert_eq!(
                crate::symbols::source_public_theorem_export_candidates(ast, arena, env, supplied)
                    .unwrap_err(),
                lower
            );
        }
        assert_eq!((ast.clone(), arena.clone(), env.clone()), before);
    }
    // Altered tokens are negative-only, with a real valid Alpha row ordered first.
    let text = format!("theorem Alpha: 0 = 0\nproof\n thus 0 = 0;\nend;\n{fixture}");
    let (ast, _, _, env, arena) = parsed_private_theorem_fixture(&text);
    let original = source_public_theorem_formula_candidates(&ast, &arena, &env)
        .unwrap()
        .unwrap();
    let owners = original.keys().collect::<Vec<_>>();
    assert_eq!(owners.len(), 2);
    for (id, name) in owners.iter().zip(["Alpha", "UsePriv1"]) {
        assert_eq!(env.symbols().get(id).unwrap().primary_spelling(), name);
    }
    let formula = ast.node_view(original[owners[1]]).unwrap();
    let target = ast
        .node_views()
        .find(|node| {
            node.as_token()
                .is_some_and(|token| token.text.as_ref() == "X")
                && node.range().start >= formula.range().start
                && node.range().end <= formula.range().end
        })
        .unwrap()
        .id();
    for mutation in 0..3 {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut mapped = BTreeMap::new();
        for node in ast.node_views() {
            let children = node.children().iter().map(|child| mapped[child]).collect();
            let id = if let Some(token) = node.as_token() {
                let (kind, text) = match (node.id() == target, mutation) {
                    (true, 0) => (SurfaceTokenKind::Unknown, token.text.clone()),
                    (true, 1) => (token.kind, "".into()),
                    _ => (token.kind, token.text.clone()),
                };
                if node.id() == target && mutation == 2 {
                    builder.add_recovered_token(kind, text, node.range())
                } else {
                    builder.add_token(kind, text, node.range())
                }
            } else {
                builder.add_node(node.kind().clone(), node.range(), children)
            };
            mapped.insert(node.id(), id);
        }
        let rebuilt = builder.finish(Some(mapped[&ast.root().unwrap()]), None);
        let shells =
            crate::declarations::DeclarationShellCollector::new(&rebuilt, &module).collect();
        let env =
            crate::symbols::SignatureProjectionExtractor::new(&rebuilt, &shells, namespace.clone())
                .collect(&module)
                .into_env();
        let arena = SurfaceResolvedArena::lower(&rebuilt, &module).unwrap();
        let before = (rebuilt.clone(), arena.clone(), env.clone());
        let identities = (identity.clone(), invalid.clone());
        let lower = source_public_theorem_formula_candidates(&rebuilt, &arena, &env);
        if mutation == 2 {
            assert!(lower.is_err());
            for supplied in [&identity, &invalid] {
                assert_eq!(
                    crate::symbols::source_public_theorem_export_candidates(
                        &rebuilt, &arena, &env, supplied
                    )
                    .unwrap_err(),
                    lower.clone().unwrap_err()
                );
            }
        } else {
            let formulas = lower.unwrap().unwrap();
            assert_eq!(formulas.keys().collect::<Vec<_>>(), owners);
            assert_eq!(
                crate::symbols::source_public_theorem_export_candidates(
                    &rebuilt, &arena, &env, &identity
                ),
                Ok(None)
            );
        }
        assert_eq!((rebuilt, arena, env), before);
        assert_eq!((identity.clone(), invalid.clone()), identities);
    }
    for text in [
        fixture.to_owned(),
        fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
    ] {
        let (ast, _, _, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), arena.clone(), env.clone(), identity.clone());
        let lower = source_public_theorem_formula_candidates(&ast, &arena, &env)
            .unwrap()
            .unwrap();
        assert_eq!(
            lower.len(),
            usize::from(!text.contains("private theorem UsePriv1"))
        );
        for field in 0..7 {
            let mut supplied = identity.clone();
            match field {
                0 => supplied.package_id = String::new(),
                1 => supplied.module_path = String::new(),
                2 => supplied.language_edition = String::new(),
                3 => supplied.package_version = Some(String::new()),
                4 => supplied.lockfile_identity = Some(String::new()),
                5 => supplied.package_id = "wrong-package".to_owned(),
                6 => supplied.module_path = "wrong.path".to_owned(),
                _ => unreachable!(),
            }
            let result = crate::symbols::source_public_theorem_export_candidates(
                &ast, &arena, &env, &supplied,
            )
            .unwrap_err();
            if field < 5 {
                assert_eq!(result, supplied.canonical_json().unwrap_err().to_string());
            } else {
                assert_eq!(result, "export identity does not match source module");
            }
        }
        assert_eq!((ast, arena, env, identity.clone()), before);
    }
}

#[test]
fn source_theorem_module_summary_candidate_preserves_real_rows_and_hashes() {
    use mizar_artifact::{
        module_summary::*,
        store::{CanonicalHashDomain, CanonicalJson as Json, HashClass},
    };
    use mizar_frontend::{
        orchestration::Frontend, parsing::MizarParserSeam, source::FrontendSourceLoader,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, SourceInput, SourceOriginInput, hash_text, normalize_path,
    };
    let simple = "theorem Base: 0 = 0\nproof\n thus 0 = 0;\nend;\n";
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let statuses = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
    );
    let mut baseline = None;
    for case in 0..13 {
        let text = match case {
            1 => format!(":: λ relocation\n{simple}"),
            2 => simple.replace(" thus 0 = 0;", " thus 0 = 0;\n thus 0 = 0;"),
            3 => simple.replacen("0 = 0", "0 = 1", 1),
            4 => simple.replace("Base", "Changed"),
            5 => simple.replace("theorem", "lemma"),
            6 => fixture.to_owned(),
            7 => fixture.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
            8 => statuses.to_owned(),
            9 => String::new(),
            _ => simple.to_owned(),
        };
        let (package, module_path) = match case {
            10 => ("other-package", "main"),
            11 => ("pkg", "Nested.Main"),
            _ => ("pkg", "main"),
        };
        let root = std::env::temp_dir().join(format!(
            "mizar-summary-positive-{}-{case}",
            std::process::id()
        ));
        let path = root.join(if case == 12 {
            "src/relocated/source.miz"
        } else {
            "src/main.miz"
        });
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &text).unwrap();
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "31".repeat(Hash::BYTE_LEN)
        ))
        .unwrap();
        let allocator = InMemorySessionIdAllocator::new();
        if case == 12 {
            allocator.next_source_id(snapshot).unwrap();
        }
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            NoImportedSummaries,
            MizarParserSeam,
        )
        .run(
            mizar_frontend::source::SourceUnitRequest {
                snapshot,
                input: SourceInput {
                    package_id: PackageId::new(package),
                    module_path: ModulePath::new(module_path),
                    normalized_path: normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &allocator,
        )
        .unwrap();
        std::fs::remove_dir_all(root).unwrap();
        assert!(
            frontend.diagnostics.is_empty(),
            "{case}: {:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.as_ref().unwrap();
        let module = module_id(package, module_path);
        let shells = crate::declarations::DeclarationShellCollector::new(ast, &module).collect();
        let collected = crate::symbols::SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module_path),
        )
        .collect(&module);
        assert!(collected.diagnostics().is_empty());
        let identity = ModuleSummaryIdentity {
            package_id: package.to_owned(),
            package_version: Some("1.2.3".to_owned()),
            lockfile_identity: Some("lock:source-test".to_owned()),
            module_path: module_path.to_owned(),
            language_edition: "2025".to_owned(),
        };
        let before = (frontend.clone(), collected.clone(), identity.clone());
        let lexical = collected
            .export_frontend_lexical_contributions(&frontend, &identity)
            .unwrap();
        assert!(lexical.is_empty());
        let locals = frontend.tokens.local_declarations();
        assert!(locals.user_symbols.is_empty() && locals.operator_declarations.is_empty());
        assert!(frontend.preprocessed.import_stubs.is_empty());
        let arena = SurfaceResolvedArena::lower(ast, &module).unwrap();
        let env = collected.env();
        let semantic = crate::resolved_ast::source_theorem_resolved_ast(ast, &arena, env)
            .unwrap()
            .unwrap();
        let imports = semantic.imports();
        assert!(imports.imports().next().is_none() && imports.exports().next().is_none());
        assert!(imports.canonical_import_modules().is_empty());
        assert!(env.imports().is_empty() && env.exports().is_empty() && env.overloads().is_empty());
        assert!(env.registrations().is_empty() && env.lexical_summaries().is_empty());
        assert!(env.namespace_graph().is_empty() && env.module_summaries().is_empty());
        let symbols =
            crate::symbols::source_public_theorem_export_candidates(ast, &arena, env, &identity)
                .unwrap()
                .unwrap();
        let labels = source_public_theorem_label_candidates(ast, &arena, env)
            .unwrap()
            .unwrap();
        let summary = collected
            .source_theorem_module_summary_candidate(&frontend, &identity)
            .unwrap()
            .unwrap();
        assert_eq!(summary.schema_version, current_schema_version());
        assert_eq!(summary.module, identity);
        assert_eq!(summary.source_hash, frontend.source.source_hash);
        assert_eq!(summary.source_hash, hash_text(&frontend.source.source_text));
        assert_eq!(summary.exported_symbols, symbols);
        assert!(
            summary
                .exported_symbols
                .iter()
                .all(|row| row.proof_status.is_none())
        );
        assert_eq!(summary.exported_labels, labels);
        assert_eq!(
            summary.lexical_summary,
            ModuleLexicalSummary {
                schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
                fingerprint: None,
                contributions: lexical
            }
        );
        assert!(summary.reexports.is_empty() && summary.dependency_interfaces.is_empty());
        assert_eq!(
            collected
                .source_theorem_module_summary_candidate(&frontend, &identity)
                .unwrap(),
            Some(summary.clone())
        );
        let json = module_summary_json(&summary).unwrap();
        assert_eq!(
            write_module_summary(&summary).unwrap(),
            mizar_artifact::store::canonical_json_bytes(&json)
        );
        assert_eq!(
            read_module_summary(
                &json,
                ModuleSummaryReadOptions {
                    artifact_path: Some("source-summary.json"),
                    expected_module: Some(&identity),
                    expected_interface_hash: Some(summary.interface_hash)
                }
            )
            .unwrap(),
            summary
        );
        if case == 0 {
            let owner = env
                .symbols()
                .iter()
                .find(|owner| owner.primary_spelling() == "Base")
                .unwrap();
            let fqn = owner.symbol().fqn().as_str();
            let module_json = Json::object([
                ("package_id", Json::string("pkg")),
                ("package_version", Json::string("1.2.3")),
                ("lockfile_identity", Json::string("lock:source-test")),
                ("module_path", Json::string("main")),
                ("language_edition", Json::string("2025")),
            ])
            .unwrap();
            let mut symbol = Json::object([
                ("origin_id", Json::string(fqn)),
                ("fully_qualified_name", Json::string(fqn)),
                ("namespace_path", Json::array([Json::string("main")])),
                ("visibility", Json::string("public")),
                ("declaration_kind", Json::string("theorem")),
                ("rendered_signature", Json::string("theorem Base : 0 = 0")),
                ("proof_status", Json::Null),
            ])
            .unwrap();
            let domain = CanonicalHashDomain::new(
                HashClass::Interface,
                MODULE_SUMMARY_SCHEMA_FAMILY,
                current_schema_version(),
            );
            let fingerprint = domain.hash(
                &Json::object([("module", module_json.clone()), ("symbol", symbol.clone())])
                    .unwrap(),
                &[],
            );
            let hash_hex = fingerprint
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            let Json::Object(fields) = &mut symbol else {
                unreachable!()
            };
            fields.insert(
                "interface_fingerprint".to_owned(),
                Json::string(format!(
                    "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/module-summary:1.0:{hash_hex}"
                )),
            );
            let label = Json::object([
                ("origin_id", Json::string(fqn)),
                ("label", Json::string("Base")),
                ("owner_fully_qualified_name", Json::string(fqn)),
                ("visibility", Json::string("public")),
                ("target_kind", Json::string("theorem")),
            ])
            .unwrap();
            let projection = Json::object([
                ("schema_version", Json::string("1.0")),
                ("module", module_json),
                ("exported_symbols", Json::array([symbol])),
                ("exported_labels", Json::array([label])),
                (
                    "lexical_summary",
                    Json::object([
                        (
                            "schema_version",
                            Json::string("mizar-resolve/exported-lexical/v1"),
                        ),
                        ("fingerprint", Json::Null),
                        ("contributions", Json::array([])),
                    ])
                    .unwrap(),
                ),
                ("reexports", Json::array([])),
                ("dependency_interfaces", Json::array([])),
            ])
            .unwrap();
            assert_eq!(summary.interface_hash, domain.hash(&projection, &[]));
            baseline = Some((
                summary.clone(),
                ast.source_id,
                frontend.source.normalized_path.clone(),
            ));
        }
        let original = &baseline.as_ref().unwrap().0;
        if matches!(case, 1 | 2 | 12) {
            assert_eq!(summary.interface_hash, original.interface_hash);
            if case == 12 {
                assert_eq!(summary.source_hash, original.source_hash);
                assert_ne!(ast.source_id, baseline.as_ref().unwrap().1);
                assert_ne!(
                    frontend.source.normalized_path,
                    baseline.as_ref().unwrap().2
                );
            } else {
                assert_ne!(summary.source_hash, original.source_hash);
            }
        }
        if matches!(case, 3..=5 | 10 | 11) {
            assert_ne!(summary.interface_hash, original.interface_hash);
        }
        if matches!(case, 7 | 9) {
            assert!(symbols.is_empty() && labels.is_empty());
        }
        for field in 0..2 {
            let mut metadata = identity.clone();
            if field == 0 {
                metadata.package_version = Some("2.0.0".to_owned());
            } else {
                metadata.lockfile_identity = Some("lock:other".to_owned());
            }
            let changed = collected
                .source_theorem_module_summary_candidate(&frontend, &metadata)
                .unwrap()
                .unwrap();
            assert_eq!(changed.module, metadata);
            assert_eq!(changed.source_hash, summary.source_hash);
            assert_ne!(changed.interface_hash, summary.interface_hash);
        }
        for field in 0..8 {
            let mut invalid = identity.clone();
            match field {
                0 => invalid.package_id.clear(),
                1 => invalid.module_path.clear(),
                2 => invalid.language_edition.clear(),
                3 => invalid.package_version = Some(String::new()),
                4 => invalid.lockfile_identity = Some(String::new()),
                5 => invalid.package_id = "wrong-package".to_owned(),
                6 => invalid.module_path = "wrong.path".to_owned(),
                7 => invalid.language_edition = "2027".to_owned(),
                _ => unreachable!(),
            }
            assert_eq!(
                collected.export_frontend_lexical_contributions(&frontend, &invalid),
                None
            );
            assert_eq!(
                collected.source_theorem_module_summary_candidate(&frontend, &invalid),
                Ok(None)
            );
        }
        assert_eq!((frontend.clone(), collected, identity), before);
    }
}

#[test]
fn source_theorem_module_summary_candidate_refuses_missing_or_unsupported_inputs() {
    use mizar_artifact::module_summary::ModuleSummaryIdentity;
    use mizar_frontend::{
        orchestration::Frontend, parsing::MizarParserSeam, source::FrontendSourceLoader,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, SourceInput, SourceOriginInput, hash_text, normalize_path,
    };
    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let simple = "theorem Base: 0 = 0\nproof\n thus 0 = 0;\nend;\n";
    let identity = ModuleSummaryIdentity {
        package_id: "pkg".to_owned(),
        package_version: None,
        lockfile_identity: None,
        module_path: "main".to_owned(),
        language_edition: "2025".to_owned(),
    };
    let mut original = None;
    for (case, text) in [
        fixture.to_owned(), format!("import std.core;\n{simple}"), format!("export std.core;\n{simple}"),
        format!("{simple}\ndefinition\n let X, Y be set;\n private pred Hidden: X secret Y means X = Y;\nend;\n"),
        format!("{simple}\nprivate infix_operator(\"+\", left, 80);\n"),
        format!("{simple}\ndefinition\n let X, Y be set;\n synonym X alias Y for X base Y;\nend;\n"),
        format!("{simple}\nreserve Y for set;\n"), format!("{simple}\ntheorem Plain: 0 = 0;\n"),
        format!("{simple}\ntheorem Direct: 0 = 0 by Base;\n"),
        simple.replace("0 = 0", "f(0) = 0"),
        include_str!("../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz").to_owned(),
        "theorem Broken: 0 = ;\n".to_owned(), fixture.replace("UsePriv1", "Changed1"),
    ].into_iter().enumerate() {
        let root = std::env::temp_dir().join(format!("mizar-summary-negative-{}-{case}", std::process::id()));
        let path = root.join("src/main.miz");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &text).unwrap();
        let frontend = Frontend::new(FrontendSourceLoader::new(DiskSourceLoader::new(&root)), NoImportedSummaries, MizarParserSeam).run(
            mizar_frontend::source::SourceUnitRequest {
                snapshot: BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{}", "31".repeat(Hash::BYTE_LEN))).unwrap(),
                input: SourceInput { package_id: PackageId::new("pkg"), module_path: ModulePath::new("main"),
                    normalized_path: normalize_path(&root, &path).unwrap(), edition: Edition::new("2025"), origin: SourceOriginInput::Disk { path }, },
            }, &InMemorySessionIdAllocator::new()).unwrap();
        std::fs::remove_dir_all(root).unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let module = module_id("pkg", "main");
        let shells = crate::declarations::DeclarationShellCollector::new(ast, &module).collect();
        let collected = crate::symbols::SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main")).collect(&module);
        let arena = SurfaceResolvedArena::lower(ast, &module).unwrap();
        let lexical = collected.export_frontend_lexical_contributions(&frontend, &identity);
        let semantic = crate::resolved_ast::source_theorem_resolved_ast(ast, &arena, collected.env());
        let symbol_rows = crate::symbols::source_public_theorem_export_candidates(ast, &arena, collected.env(), &identity);
        let label_rows = source_public_theorem_label_candidates(ast, &arena, collected.env());
        let before = (frontend.clone(), collected.clone(), identity.clone());
        if case == 0 || case == 12 {
            assert_eq!(lexical, Some(Vec::new()));
            assert!(semantic.unwrap().is_some());
            assert!(symbol_rows.unwrap().is_some() && label_rows.unwrap().is_some());
            if case == 0 {
                original = Some((frontend.clone(), collected.clone()));
                let augmented = crate::symbols::source_theorem_module_candidates(ast, &arena, collected.env()).unwrap().unwrap().1;
                assert!(crate::resolved_ast::source_theorem_resolved_ast(ast, &arena, &augmented).is_err());
            } else {
                let (first, first_collection) = original.as_ref().unwrap();
                assert_eq!(first_collection.export_frontend_lexical_contributions(&frontend, &identity), None);
                assert_eq!(collected.export_frontend_lexical_contributions(first, &identity), None);
                assert_eq!(first_collection.source_theorem_module_summary_candidate(&frontend, &identity), Ok(None));
                assert_eq!(collected.source_theorem_module_summary_candidate(first, &identity), Ok(None));
            }
        } else {
            if case == 10 { assert!(!collected.diagnostics().is_empty()); }
            if case == 11 { assert!(!frontend.diagnostics.is_empty()); }
            if case == 3 {
                assert!(!frontend.tokens.local_declarations().user_symbols.is_empty());
                assert_eq!(lexical, Some(Vec::new()), "private lexical omission is not absence");
            }
            let expected = if !collected.diagnostics().is_empty() || lexical.is_none()
                || !frontend.tokens.local_declarations().user_symbols.is_empty()
                || !frontend.tokens.local_declarations().operator_declarations.is_empty()
                || !frontend.preprocessed.import_stubs.is_empty() { Ok(None) }
                else { match semantic { Ok(None) => Ok(None), Err(error) => Err(error), Ok(Some(_)) => panic!("unsupported profile admitted: {case}") } };
            assert_eq!(collected.source_theorem_module_summary_candidate(&frontend, &identity), expected, "{case}");
        }
        let (first, first_collection) = original.as_ref().unwrap();
        if case == 1 {
            assert!(!frontend.preprocessed.import_stubs.is_empty());
            let mut omitted = frontend.clone();
            omitted.preprocessed.import_stubs.clear();
            assert_eq!(collected.export_frontend_lexical_contributions(&omitted, &identity), None);
            assert_eq!(collected.source_theorem_module_summary_candidate(&omitted, &identity), Ok(None));
        }
        if case == 3 {
            let mut stale = first.clone();
            stale.tokens.local_declarations = frontend.tokens.local_declarations.clone();
            assert_eq!(first_collection.export_frontend_lexical_contributions(&stale, &identity), None);
            assert_eq!(first_collection.source_theorem_module_summary_candidate(&stale, &identity), Ok(None));
        }
        if case == 11 {
            let mut diagnostic = first.clone();
            diagnostic.diagnostics = frontend.diagnostics.clone();
            assert_eq!(first_collection.export_frontend_lexical_contributions(&diagnostic, &identity), None);
            assert_eq!(first_collection.source_theorem_module_summary_candidate(&diagnostic, &identity), Ok(None));
        }
        assert_eq!((frontend, collected, identity.clone()), before);
    }
    let ((first, c), id) = (original.unwrap(), identity);
    for mutation in 0..5 {
        let mut f = first.clone();
        match mutation {
            0 => f.ast = None,
            1 => f.source.source_hash = hash_text("stale"),
            2 => f.source.source_text = "stale".into(),
            3 => f.preprocessed.lexical_hash = hash_text("stale lexical text"),
            4 => f.preprocessed.source_map.lexical_text_len += 1,
            _ => unreachable!(),
        }
        assert_eq!(c.export_frontend_lexical_contributions(&f, &id), None);
        assert_eq!(c.source_theorem_module_summary_candidate(&f, &id), Ok(None));
    }
}

#[test]
fn source_proof_occurrence_validation_preserves_real_rows_and_inputs() {
    use crate::names::{
        SourceVariableBindingKind, SourceVariableScopeInput, SourceVariableScopeResolver,
    };
    use crate::symbols::validate_source_proof_occurrences;

    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let local = "theorem Local: for X, Y being set holds X = X\nproof\n let X, Y be set;\n thus X = X;\nend;\n";
    for (case, text) in [
        private.to_owned(),
        private.replace("private ", ""),
        local.to_owned(),
        local.replace(" thus X = X;", " now\n let Z, W be object;\n thus Z = Z;\n end;\n thus X = X;"),
        include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_given_existential_assumption_001.miz").to_owned(),
        include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_consider_choice_001.miz").to_owned(),
    ].into_iter().enumerate() {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(&text);
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, &module, &env),
        ).unwrap();
        assert_eq!(scope.source_id(), ast.source_id);
        assert_eq!(scope.module_id(), &module);
        assert!(!scope.bindings().is_empty() && !scope.references().is_empty());
        if case == 3 {
            for name in ["Y", "W"] {
                let unused = scope.bindings().iter().find(|row| row.spelling() == name && row.kind() == SourceVariableBindingKind::Let).unwrap();
                assert!(!scope.references().iter().any(|row| row.binding() == unused.id()));
            }
            assert_eq!(scope.bindings().iter().filter(|row| row.kind() == SourceVariableBindingKind::Let).count(), 4);
        }
        if case >= 4 {
            let (kind, owner) = if case == 4 {
                (SourceVariableBindingKind::GivenWitness, SurfaceNodeKind::GivenStatement)
            } else {
                (SourceVariableBindingKind::ConsiderWitness, SurfaceNodeKind::ConsiderStatement)
            };
            let binding = scope.bindings().iter().find(|row| row.kind() == kind).unwrap();
            assert_eq!(ast.node_view(binding.node()).unwrap().as_token().unwrap().text.as_ref(), binding.spelling());
            assert!(ast.node_views().any(|node| node.kind() == &owner && node.range().start <= binding.range().start && binding.range().end <= node.range().end));
            assert!(scope.references().iter().any(|row| row.binding() == binding.id()));
        }
        let before = (ast.clone(), module.clone(), env.clone(), arena.clone(), scope.clone());
        validate_source_proof_occurrences(&arena, &env, &scope).unwrap();
        let replay = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, &module, &env),
        ).unwrap();
        assert_eq!(scope, replay);
        validate_source_proof_occurrences(&arena, &env, &replay).unwrap();
        if case == 2 {
            let legacy = SourceVariableScopeResolver::resolve(SourceVariableScopeInput::new(&ast, &module, &env)).unwrap();
            assert_eq!(scope.source_id(), legacy.source_id());
            assert_eq!(scope.module_id(), legacy.module_id());
            assert_eq!(scope.bindings(), legacy.bindings());
            assert_eq!(scope.references(), legacy.references());
            assert!(scope.thesis().is_none() && scope.statements().is_empty());
            assert!(legacy.thesis().is_some() && !legacy.statements().is_empty());
            let legacy_before = legacy.clone();
            validate_source_proof_occurrences(&arena, &env, &legacy).unwrap();
            assert_eq!(legacy, legacy_before);
        }
        assert_eq!((ast, module, env, arena, scope), before);
    }
}

#[test]
fn source_proof_occurrence_validation_rejects_foreign_rows_and_preserves_errors() {
    use crate::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    use crate::symbols::{validate_source_proof_occurrences, validate_source_symbol_env};
    use mizar_frontend::{
        orchestration::Frontend, parsing::MizarParserSeam, source::FrontendSourceLoader,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, SourceInput, SourceOriginInput, normalize_path,
    };

    let invalid = "source proof occurrences do not match the original source";
    let text = "theorem Local: for X, Y being set holds X = X\nproof\n let X, Y be set;\n thus X = X;\nend;\n";
    let (ast, module, _, env, arena) = parsed_private_theorem_fixture(text);
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &module, &env),
    )
    .unwrap();
    validate_source_proof_occurrences(&arena, &env, &scope).unwrap();
    let before = (
        ast.clone(),
        module.clone(),
        env.clone(),
        arena.clone(),
        scope.clone(),
    );
    for (case, foreign_text) in [
        text.replace("thus X = X", "thus Y = X"),
        text.replace("let X, Y", "let X, Z"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(text.len(), foreign_text.len());
        let (foreign_ast, foreign_module, _, foreign_env, foreign_arena) =
            parsed_private_theorem_fixture(&foreign_text);
        let foreign = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&foreign_ast, &foreign_module, &foreign_env),
        )
        .unwrap();
        assert_eq!(foreign.source_id(), scope.source_id());
        assert_eq!(foreign.module_id(), scope.module_id());
        assert_eq!(foreign.bindings().len(), scope.bindings().len());
        assert_eq!(foreign.references().len(), scope.references().len());
        if case == 0 {
            assert_eq!(foreign.bindings(), scope.bindings());
            assert_ne!(foreign.references(), scope.references());
        } else {
            assert_eq!(foreign.references(), scope.references());
            assert_ne!(foreign.bindings(), scope.bindings());
        }
        let foreign_before = (
            foreign_ast.clone(),
            foreign_env.clone(),
            foreign_arena.clone(),
            foreign.clone(),
        );
        validate_source_proof_occurrences(&foreign_arena, &foreign_env, &foreign).unwrap();
        assert_eq!(
            validate_source_proof_occurrences(&arena, &env, &foreign).unwrap_err(),
            invalid
        );
        assert_eq!(
            (foreign_ast, foreign_env, foreign_arena, foreign),
            foreign_before
        );
    }
    let other_module = module_id("other", "main");
    let shells = crate::declarations::DeclarationShellCollector::new(&ast, &other_module).collect();
    let collected = crate::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new("main"),
    )
    .collect(&other_module);
    assert!(collected.diagnostics().is_empty());
    let other_arena = SurfaceResolvedArena::lower(&ast, &other_module).unwrap();
    let other = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &other_module, collected.env()),
    )
    .unwrap();
    assert_eq!(other.source_id(), scope.source_id());
    assert_ne!(other.module_id(), scope.module_id());
    validate_source_proof_occurrences(&other_arena, collected.env(), &other).unwrap();
    assert_eq!(
        validate_source_proof_occurrences(&arena, &env, &other).unwrap_err(),
        invalid
    );
    let (foreign_ast, _, _, foreign_env, foreign_arena) =
        parsed_private_theorem_fixture(&text.replace("Local", "Other"));
    assert_ne!(foreign_env, env);
    validate_source_symbol_env(&foreign_arena, &foreign_env).unwrap();
    let lower = validate_source_symbol_env(&arena, &foreign_env).unwrap_err();
    assert_eq!(
        validate_source_proof_occurrences(&arena, &foreign_env, &other).unwrap_err(),
        lower
    );
    assert_eq!(foreign_ast.source_id, ast.source_id);

    let fixture = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let (original_ast, original_module, _, original_env, original_arena) =
        parsed_private_theorem_fixture(fixture);
    let original = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&original_ast, &original_module, &original_env),
    )
    .unwrap();
    validate_source_proof_occurrences(&original_arena, &original_env, &original).unwrap();
    let pair = crate::symbols::source_theorem_module_candidates(
        &original_ast,
        &original_arena,
        &original_env,
    )
    .unwrap()
    .unwrap();
    assert_ne!(pair.1, original_env);
    let lower = validate_source_symbol_env(&original_arena, &pair.1).unwrap_err();
    assert_eq!(
        validate_source_proof_occurrences(&original_arena, &pair.1, &other).unwrap_err(),
        lower
    );

    let (unresolved, unresolved_module, _, unresolved_env, unresolved_arena) =
        parsed_private_theorem_fixture(&text.replace("thus X = X", "thus Q = X"));
    validate_source_symbol_env(&unresolved_arena, &unresolved_env).unwrap();
    assert!(
        SourceVariableScopeResolver::resolve_proof_occurrences(SourceVariableScopeInput::new(
            &unresolved,
            &unresolved_module,
            &unresolved_env
        ))
        .is_err()
    );
    assert_eq!(
        validate_source_proof_occurrences(&unresolved_arena, &unresolved_env, &scope).unwrap_err(),
        invalid
    );

    let (empty_ast, empty_module, _, empty_env, empty_arena) =
        parsed_private_theorem_fixture("theorem Empty: 0 = 0\nproof\n thus 0 = 0;\nend;\n");
    let empty = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&empty_ast, &empty_module, &empty_env),
    )
    .unwrap();
    assert!(empty.bindings().is_empty() && empty.references().is_empty());
    validate_source_proof_occurrences(&empty_arena, &empty_env, &empty).unwrap();
    let rebound_id = distinct_source_ids().1;
    let rebound_ast =
        SurfaceAst::from_canonical_bytes(&empty_ast.canonical_bytes().unwrap(), rebound_id)
            .unwrap();
    let shells =
        crate::declarations::DeclarationShellCollector::new(&rebound_ast, &empty_module).collect();
    let rebound_env = crate::symbols::SignatureProjectionExtractor::new(
        &rebound_ast,
        &shells,
        NamespacePath::new("main"),
    )
    .collect(&empty_module);
    assert!(rebound_env.diagnostics().is_empty());
    let rebound_arena = SurfaceResolvedArena::lower(&rebound_ast, &empty_module).unwrap();
    let rebound = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&rebound_ast, &empty_module, rebound_env.env()),
    )
    .unwrap();
    assert!(rebound.bindings().is_empty() && rebound.references().is_empty());
    assert_eq!(empty.module_id(), rebound.module_id());
    assert_ne!(empty.source_id(), rebound.source_id());
    validate_source_proof_occurrences(&rebound_arena, rebound_env.env(), &rebound).unwrap();
    assert_eq!(
        validate_source_proof_occurrences(&empty_arena, &empty_env, &rebound).unwrap_err(),
        invalid
    );

    let root = std::env::temp_dir().join(format!(
        "mizar-proof-validation-recovery-{}",
        std::process::id()
    ));
    let path = root.join("src/main.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "theorem Broken: 0 = ;\n").unwrap();
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        NoImportedSummaries,
        MizarParserSeam,
    )
    .run(
        mizar_frontend::source::SourceUnitRequest {
            snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                "mizar-session-build-snapshot-v1:{}",
                "31".repeat(Hash::BYTE_LEN)
            ))
            .unwrap(),
            input: SourceInput {
                package_id: PackageId::new("pkg"),
                module_path: ModulePath::new("main"),
                normalized_path: normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk { path },
            },
        },
        &InMemorySessionIdAllocator::new(),
    )
    .unwrap();
    std::fs::remove_dir_all(root).unwrap();
    assert!(!frontend.diagnostics.is_empty());
    let recovered = frontend.ast.as_ref().unwrap();
    assert!(recovered.node_views().any(
        |node| node.is_recovered() || matches!(node.kind(), SurfaceNodeKind::ErrorRecovery(_))
    ));
    let shells = crate::declarations::DeclarationShellCollector::new(recovered, &module).collect();
    let recovered_env = crate::symbols::SignatureProjectionExtractor::new(
        recovered,
        &shells,
        NamespacePath::new("main"),
    )
    .collect(&module);
    let recovered_arena = SurfaceResolvedArena::lower(recovered, &module).unwrap();
    let recovered_before = (
        frontend.clone(),
        recovered_env.clone(),
        recovered_arena.clone(),
    );
    let lower = validate_source_symbol_env(&recovered_arena, recovered_env.env()).unwrap_err();
    assert_eq!(
        validate_source_proof_occurrences(&recovered_arena, recovered_env.env(), &other)
            .unwrap_err(),
        lower
    );
    assert_eq!((frontend, recovered_env, recovered_arena), recovered_before);
    assert_eq!((ast, module, env, arena, scope), before);
}

#[test]
fn source_proof_inputs_match_actual_collectors_and_preserve_inputs() {
    use crate::names::{
        SourceVariableBindingKind, SourceVariableScopeInput, SourceVariableScopeResolver,
    };
    use crate::symbols::source_proof_inputs;

    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let nested = "theorem Local: for X, Y being set holds X = X\nproof\n let X, Y be set;\n A: X = X\n proof\n let Z, W be object;\n thus Z = Z;\n end;\n thus X = X by A;\nend;\n";
    for (case, text) in [
        private.to_owned(), private.replace("private ", ""), nested.to_owned(),
        include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_given_existential_assumption_001.miz").to_owned(),
        include_str!("../../../../tests/miz/pass/theorems/pass_formula_statement_consider_choice_001.miz").to_owned(),
        "theorem Empty: 0 = 0\nproof\n thus 0 = 0;\nend;\n".to_owned(), "".to_owned(),
        "theorem Unknown: 0 = 0\nproof\n thus 0 = 0 by Missing;\nend;\n".to_owned(),
        "theorem Duplicates: 0 = 0\nproof\n D: 0 = 0;\n D: 0 = 0;\n thus 0 = 0 by D;\nend;\n".to_owned(),
    ].into_iter().enumerate() {
        let (ast, module, namespace, env, arena) = parsed_private_theorem_fixture(&text);
        let before = (ast.clone(), module.clone(), namespace.clone(), env.clone(), arena.clone());
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(SourceVariableScopeInput::new(&ast, &module, &env)).unwrap();
        assert_eq!(scope.source_id(), ast.source_id);
        assert_eq!(scope.module_id(), &module);
        assert!(scope.thesis().is_none() && scope.statements().is_empty());
        let lets: Vec<_> = scope.bindings().iter().filter(|row| row.kind() == SourceVariableBindingKind::Let).map(|row| row.spelling()).collect();
        assert_eq!(lets, match case { 0 | 1 => vec!["X", "X"], 2 => vec!["X", "Y", "Z", "W"], 3 => vec!["A"], 4 => vec!["x"], _ => vec![] });
        for row in scope.bindings() {
            assert_eq!(&text[row.range().start..row.range().end], row.spelling());
            assert_eq!(ast.node_view(row.node()).unwrap().as_token().unwrap().text.as_ref(), row.spelling());
        }
        for row in scope.references() {
            assert_eq!(&text[row.range().start..row.range().end], scope.bindings()[row.binding().index()].spelling());
        }
        if case == 2 {
            assert_eq!(scope.references().len(), 8);
            for name in ["Y", "W"] {
                let unused = scope.bindings().iter().find(|row| row.spelling() == name && row.kind() == SourceVariableBindingKind::Let).unwrap();
                assert!(!scope.references().iter().any(|row| row.binding() == unused.id()));
            }
        }
        if case == 3 || case == 4 {
            let (name, kind) = if case == 3 { ("y0", SourceVariableBindingKind::GivenWitness) } else { ("z", SourceVariableBindingKind::ConsiderWitness) };
            let witness = scope.bindings().iter().find(|row| row.spelling() == name && row.kind() == kind).unwrap();
            assert!(scope.references().iter().any(|row| row.binding() == witness.id()));
        }
        if case >= 5 { assert!(scope.bindings().is_empty() && scope.references().is_empty()); }
        let contributions: Vec<_> = env.contributions().iter().filter(|entry| entry.module() == &module && matches!(entry.kind(), ContributionKind::LocalSource { source_id } if *source_id == ast.source_id)).collect();
        assert_eq!(contributions.len(), 1);
        let collection = ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contributions[0].id(), &arena).unwrap().collect_with_theorem_owners(&env).unwrap();
        let names: Vec<_> = collection.projections().iter().map(|row| row.primary_spelling()).collect();
        assert_eq!(names, match case { 0 | 1 => vec!["PrivT1", "UsePriv1"], 2 => vec!["Local", "A"], 3 => vec!["Given1"], 4 => vec!["Consider1", "A1"], 5 => vec!["Empty"], 6 => vec![], 7 => vec!["Unknown"], _ => vec!["Duplicates", "D", "D"] });
        let citations: Vec<_> = collection.references().iter().map(|row| row.site().spelling()).collect();
        assert_eq!(citations, match case { 0 | 1 => vec!["PrivT1"], 2 => vec!["A"], 3 => vec!["A1"], 7 => vec!["Missing"], 8 => vec!["D"], _ => vec![] });
        for row in collection.projections() {
            assert_eq!(&text[row.declaration_range().start..row.declaration_range().end], row.primary_spelling());
            assert_eq!(row.origin().source_id(), ast.source_id);
            assert_eq!(row.origin().module_id(), &module);
            assert_eq!(row.contribution(), contributions[0].id());
        }
        if case <= 1 {
            assert_eq!(collection.projections()[0].visibility(), if case == 0 { Visibility::Private } else { Visibility::Public });
            assert_eq!(collection.projections()[0].origin_path().as_str(), env.symbols().iter().find(|row| row.primary_spelling() == "PrivT1").unwrap().symbol().fqn().as_str());
        }
        let labels = LabelResolver::new(collection.projections()).resolve(&module, &namespace, collection.references());
        assert_eq!(labels.ids().len(), citations.len());
        assert_eq!(labels.has_unresolved(), case == 3 || case == 7);
        if case == 8 {
            let LabelResolution::Ambiguous(ambiguous) = labels.table().get(labels.ids()[0]).unwrap().resolution() else { panic!("expected genuine duplicate-label ambiguity"); };
            assert_eq!(ambiguous.spelling(), "D");
            assert_eq!(ambiguous.range(), collection.references()[0].site().range());
            assert_eq!(contributions[0].id().index(), 0);
            assert_eq!(ambiguous.candidates().iter().map(|row| row.origin().as_str()).collect::<Vec<_>>(), vec![
                "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=10:Duplicates|owner-occurrence=0|proof-path=0:|label=1:D|label-occurrence=0",
                "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=10:Duplicates|owner-occurrence=0|proof-path=0:|label=1:D|label-occurrence=1",
            ]);
            for (candidate, projection) in ambiguous.candidates().iter().zip(&collection.projections()[1..]) {
                assert_eq!(candidate.origin(), projection.origin_path());
                assert_eq!(candidate.kind(), LabelKind::ProofStep);
                assert_eq!(candidate.range(), projection.declaration_range());
                assert_eq!(projection.origin().anchor(), &SourceAnchor::Range(candidate.range()));
            }
            assert_eq!(labels.diagnostics().len(), 1);
            let diagnostic = &labels.diagnostics()[0];
            assert_eq!(diagnostic.kind(), LabelDiagnosticKind::DuplicateLabel);
            assert_eq!(diagnostic.spelling(), "D");
            assert_eq!(diagnostic.primary_range(), collection.projections()[2].declaration_range());
            assert_eq!(diagnostic.related_ranges(), &[collection.projections()[1].declaration_range()]);
        } else { assert!(labels.diagnostics().is_empty()); }
        let lower = (scope, collection, labels);
        assert_eq!(source_proof_inputs(&arena, &env).unwrap(), lower);
        assert_eq!(source_proof_inputs(&arena, &env).unwrap(), lower);
        assert_eq!((ast, module, namespace, env, arena), before);
    }
}

#[test]
fn source_proof_inputs_preserve_original_errors_and_boundaries() {
    use crate::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    use crate::symbols::{source_proof_inputs, validate_source_symbol_env};
    use mizar_frontend::{
        orchestration::Frontend, parsing::MizarParserSeam, source::FrontendSourceLoader,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, SourceInput, SourceOriginInput, normalize_path,
    };

    let text =
        "theorem Local: for X being set holds X = X\nproof\n let X be set;\n thus X = X;\nend;\n";
    let (ast, module, namespace, env, arena) = parsed_private_theorem_fixture(text);
    let actual = source_proof_inputs(&arena, &env).unwrap();
    let before = (ast.clone(), env.clone(), arena.clone(), actual.clone());
    let (_, _, _, foreign_env, foreign_arena) =
        parsed_private_theorem_fixture(&text.replace("Local", "Other"));
    source_proof_inputs(&foreign_arena, &foreign_env).unwrap();
    let foreign_before = (foreign_env.clone(), foreign_arena.clone());
    for (source, symbols) in [(&arena, &foreign_env), (&foreign_arena, &env)] {
        let lower = validate_source_symbol_env(source, symbols).unwrap_err();
        assert_eq!(source_proof_inputs(source, symbols).unwrap_err(), lower);
    }
    assert_eq!((foreign_env.clone(), foreign_arena.clone()), foreign_before);
    let other_module = module_id("other", "Nested.Main");
    let shells = crate::declarations::DeclarationShellCollector::new(&ast, &other_module).collect();
    let other_env = crate::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new("Nested.Main"),
    )
    .collect(&other_module);
    assert!(other_env.diagnostics().is_empty());
    let other_arena = SurfaceResolvedArena::lower(&ast, &other_module).unwrap();
    let other = source_proof_inputs(&other_arena, other_env.env()).unwrap();
    assert_eq!(other.0.module_id(), &other_module);
    let other_before = (other_env.clone(), other_arena.clone(), other.clone());
    for (source, symbols) in [(&arena, other_env.env()), (&other_arena, &env)] {
        let lower = validate_source_symbol_env(source, symbols).unwrap_err();
        assert_eq!(source_proof_inputs(source, symbols).unwrap_err(), lower);
    }
    assert_eq!((other_env, other_arena, other), other_before);
    let (private_ast, _, _, private_env, private_arena) = parsed_private_theorem_fixture(
        include_str!(
            "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
        ),
    );
    source_proof_inputs(&private_arena, &private_env).unwrap();
    let pair = crate::symbols::source_theorem_module_candidates(
        &private_ast,
        &private_arena,
        &private_env,
    )
    .unwrap()
    .unwrap();
    assert_ne!(pair.1, private_env);
    let private_before = (
        private_ast.clone(),
        private_env.clone(),
        private_arena.clone(),
        pair.clone(),
    );
    let lower = validate_source_symbol_env(&private_arena, &pair.1).unwrap_err();
    assert_eq!(
        source_proof_inputs(&private_arena, &pair.1).unwrap_err(),
        lower
    );
    assert_eq!(
        (private_ast, private_env, private_arena, pair),
        private_before
    );
    let (unresolved, unresolved_module, _, unresolved_env, unresolved_arena) =
        parsed_private_theorem_fixture(&text.replace("thus X = X", "thus Q = X"));
    validate_source_symbol_env(&unresolved_arena, &unresolved_env).unwrap();
    assert!(
        SourceVariableScopeResolver::resolve_proof_occurrences(SourceVariableScopeInput::new(
            &unresolved,
            &unresolved_module,
            &unresolved_env
        ))
        .is_err()
    );
    let unresolved_before = (
        unresolved.clone(),
        unresolved_env.clone(),
        unresolved_arena.clone(),
    );
    assert_eq!(
        source_proof_inputs(&unresolved_arena, &unresolved_env).unwrap_err(),
        "source proof occurrences do not match the original source"
    );
    let lower = validate_source_symbol_env(&unresolved_arena, &foreign_env).unwrap_err();
    assert_eq!(
        source_proof_inputs(&unresolved_arena, &foreign_env).unwrap_err(),
        lower
    );
    assert_eq!(
        (unresolved, unresolved_env, unresolved_arena),
        unresolved_before
    );
    let root = std::env::temp_dir().join(format!(
        "mizar-proof-inputs-recovery-{}",
        std::process::id()
    ));
    let path = root.join("src/main.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "theorem Broken: 0 = ;\n").unwrap();
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        NoImportedSummaries,
        MizarParserSeam,
    )
    .run(
        mizar_frontend::source::SourceUnitRequest {
            snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                "mizar-session-build-snapshot-v1:{}",
                "31".repeat(Hash::BYTE_LEN)
            ))
            .unwrap(),
            input: SourceInput {
                package_id: PackageId::new("pkg"),
                module_path: ModulePath::new("main"),
                normalized_path: normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk { path },
            },
        },
        &InMemorySessionIdAllocator::new(),
    )
    .unwrap();
    std::fs::remove_dir_all(root).unwrap();
    assert!(!frontend.diagnostics.is_empty());
    let recovered = frontend.ast.as_ref().unwrap();
    assert!(recovered.node_views().any(
        |node| node.is_recovered() || matches!(node.kind(), SurfaceNodeKind::ErrorRecovery(_))
    ));
    let shells = crate::declarations::DeclarationShellCollector::new(recovered, &module).collect();
    let recovered_env =
        crate::symbols::SignatureProjectionExtractor::new(recovered, &shells, namespace)
            .collect(&module);
    let recovered_arena = SurfaceResolvedArena::lower(recovered, &module).unwrap();
    let recovered_before = (
        frontend.clone(),
        recovered_env.clone(),
        recovered_arena.clone(),
    );
    let lower = validate_source_symbol_env(&recovered_arena, recovered_env.env()).unwrap_err();
    assert_eq!(
        source_proof_inputs(&recovered_arena, recovered_env.env()).unwrap_err(),
        lower
    );
    assert_eq!((frontend, recovered_env, recovered_arena), recovered_before);
    assert_eq!((ast, env, arena, actual), before);
}

#[test]
fn source_declaration_shells_match_original_collectors() {
    use crate::declarations::{
        DeclarationShellCollector, DeclarationShellKind as K, DeclarationShellVisibilityState as V,
    };
    use crate::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    use crate::symbols::source_declaration_shells;

    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for (case, text) in [
        private,
        lemma,
        "",
        "export std.core, .facade;\n",
        "definition\n let X, Y be set;\n private pred Hidden: X secret Y means X = Y;\nend;\n",
        "theorem Local: for X being set holds X = X\nproof\n let X be set;\n thus Q = X;\nend;\n",
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, env, arena) = parsed_private_theorem_fixture(text);
        let before = (ast.clone(), env.clone(), arena.clone());
        let expected = DeclarationShellCollector::new(&ast, &module).collect();
        let roles = expected
            .declarations()
            .iter()
            .map(|row| (row.kind(), row.visibility().state()))
            .collect::<Vec<_>>();
        assert_eq!(
            roles,
            match case {
                0 => vec![(K::Theorem, V::Private), (K::Theorem, V::Unspecified)],
                1 => vec![(K::Lemma, V::Unspecified), (K::Theorem, V::Unspecified)],
                4 => vec![
                    (K::DefinitionBlock, V::Unspecified),
                    (K::PredicateDefinition, V::Private)
                ],
                5 => vec![(K::Theorem, V::Unspecified)],
                _ => vec![],
            }
        );
        let mut nodes = ast
            .node_views()
            .filter(|node| {
                matches!(
                    node.kind(),
                    SurfaceNodeKind::TheoremItem
                        | SurfaceNodeKind::LemmaItem
                        | SurfaceNodeKind::DefinitionBlockItem
                        | SurfaceNodeKind::PredicateDefinition
                )
            })
            .collect::<Vec<_>>();
        nodes.sort_by_key(|node| (node.range().start, std::cmp::Reverse(node.range().end)));
        assert_eq!(nodes.len(), roles.len());
        for (row, node) in expected.declarations().iter().zip(nodes) {
            assert_eq!(row.module(), &module);
            assert_eq!(row.node_id(), node.id());
            assert_eq!(row.range(), node.range());
            assert!(!row.recovered());
            assert_eq!(
                row.parent(),
                if row.kind() == K::PredicateDefinition {
                    Some(expected.declarations()[0].id())
                } else {
                    None
                }
            );
            if let Some(marker) = row.visibility().marker_range() {
                assert_eq!(&text[marker.start..marker.end], "private");
            } else {
                assert_eq!(row.visibility().state(), V::Unspecified);
            }
        }
        assert_eq!(expected.is_empty(), case == 2);
        assert_eq!(expected.exports().len(), usize::from(case == 3));
        if case == 3 {
            let export = &expected.exports()[0];
            let node = ast
                .node_views()
                .find(|node| matches!(node.kind(), SurfaceNodeKind::ExportItem))
                .unwrap();
            assert_eq!(export.node_id(), node.id());
            assert_eq!(export.range(), node.range());
            assert_eq!(export.module(), &module);
            assert!(!export.recovered());
            assert_eq!(
                export
                    .paths()
                    .iter()
                    .map(|path| path.spelling())
                    .collect::<Vec<_>>(),
                vec!["std.core", ".facade"]
            );
            for path in export.paths() {
                assert_eq!(&text[path.range().start..path.range().end], path.spelling());
            }
        }
        if case == 5 {
            assert!(
                SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(&ast, &module, &env)
                )
                .is_err()
            );
        }
        assert_eq!(source_declaration_shells(&arena, &env).unwrap(), expected);
        assert_eq!(source_declaration_shells(&arena, &env).unwrap(), expected);
        assert_eq!((ast, env, arena), before);
    }
}

#[test]
fn source_declaration_shells_preserve_upstream_errors() {
    use crate::symbols::{source_declaration_shells, validate_source_symbol_env};
    use mizar_frontend::{
        orchestration::Frontend, parsing::MizarParserSeam, source::FrontendSourceLoader,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, SourceInput, SourceOriginInput, normalize_path,
    };

    let text = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let (ast, module, namespace, env, arena) = parsed_private_theorem_fixture(text);
    let actual = source_declaration_shells(&arena, &env).unwrap();
    let before = (ast.clone(), env.clone(), arena.clone(), actual.clone());
    let (_, _, _, foreign_env, foreign_arena) =
        parsed_private_theorem_fixture(&text.replace("PrivT1", "Other"));
    source_declaration_shells(&foreign_arena, &foreign_env).unwrap();
    let foreign_before = (foreign_env.clone(), foreign_arena.clone());
    for (source, symbols) in [(&arena, &foreign_env), (&foreign_arena, &env)] {
        let lower = validate_source_symbol_env(source, symbols).unwrap_err();
        assert_eq!(
            source_declaration_shells(source, symbols).unwrap_err(),
            lower
        );
    }
    assert_eq!((foreign_env, foreign_arena), foreign_before);
    let other_module = module_id("other", "Nested.Main");
    let shells = crate::declarations::DeclarationShellCollector::new(&ast, &other_module).collect();
    let other_env = crate::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new("Nested.Main"),
    )
    .collect(&other_module);
    let other_arena = SurfaceResolvedArena::lower(&ast, &other_module).unwrap();
    source_declaration_shells(&other_arena, other_env.env()).unwrap();
    let other_before = (other_env.clone(), other_arena.clone());
    for (source, symbols) in [(&arena, other_env.env()), (&other_arena, &env)] {
        let lower = validate_source_symbol_env(source, symbols).unwrap_err();
        assert_eq!(
            source_declaration_shells(source, symbols).unwrap_err(),
            lower
        );
    }
    assert_eq!((other_env, other_arena), other_before);
    let pair = crate::symbols::source_theorem_module_candidates(&ast, &arena, &env)
        .unwrap()
        .unwrap();
    let pair_before = pair.clone();
    let lower = validate_source_symbol_env(&arena, &pair.1).unwrap_err();
    assert_eq!(
        source_declaration_shells(&arena, &pair.1).unwrap_err(),
        lower
    );
    assert_eq!(pair, pair_before);
    let root = std::env::temp_dir().join(format!(
        "mizar-declaration-shells-recovery-{}",
        std::process::id()
    ));
    let path = root.join("src/main.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "theorem Broken: 0 = ;\n").unwrap();
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        NoImportedSummaries,
        MizarParserSeam,
    )
    .run(
        mizar_frontend::source::SourceUnitRequest {
            snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                "mizar-session-build-snapshot-v1:{}",
                "31".repeat(Hash::BYTE_LEN)
            ))
            .unwrap(),
            input: SourceInput {
                package_id: PackageId::new("pkg"),
                module_path: ModulePath::new("main"),
                normalized_path: normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk { path },
            },
        },
        &InMemorySessionIdAllocator::new(),
    )
    .unwrap();
    std::fs::remove_dir_all(root).unwrap();
    assert!(!frontend.diagnostics.is_empty());
    let recovered = frontend.ast.as_ref().unwrap();
    assert!(recovered.node_views().any(
        |node| node.is_recovered() || matches!(node.kind(), SurfaceNodeKind::ErrorRecovery(_))
    ));
    let shells = crate::declarations::DeclarationShellCollector::new(recovered, &module).collect();
    let recovered_env =
        crate::symbols::SignatureProjectionExtractor::new(recovered, &shells, namespace)
            .collect(&module);
    let recovered_arena = SurfaceResolvedArena::lower(recovered, &module).unwrap();
    let recovered_before = (
        frontend.clone(),
        recovered_env.clone(),
        recovered_arena.clone(),
    );
    let lower = validate_source_symbol_env(&recovered_arena, recovered_env.env()).unwrap_err();
    assert_eq!(
        source_declaration_shells(&recovered_arena, recovered_env.env()).unwrap_err(),
        lower
    );
    assert_eq!((frontend, recovered_env, recovered_arena), recovered_before);
    assert_eq!((ast, env, arena, actual), before);
}

#[test]
fn source_algorithm_module_summary_candidate_preserves_original_projection_and_refusals() {
    use crate::symbols::source_public_algorithm_export_candidates as rows;
    use mizar_artifact::{
        module_summary::*,
        store::{CanonicalHashDomain, CanonicalJson as J, HashClass, canonical_json_bytes},
    };
    use mizar_frontend::{
        orchestration::Frontend, parsing::MizarParserSeam, source::FrontendSourceLoader,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, SourceInput, SourceOriginInput, hash_text, normalize_path,
    };
    let fixture = include_str!(
        "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
    );
    let mut original = None;
    for case in 0..22 {
        let text = match case {
            1 => fixture.replace("x := c;", "x := a;"),
            2 => format!(":: λ relocation\n{fixture}"),
            3 => fixture.replace("terminating algorithm", "private terminating algorithm"),
            4 => String::new(),
            5 => fixture.replace("terminating ", ""),
            6 => fixture.replace("terminating algorithm", "public terminating algorithm"),
            11 => fixture.replace("swapalgo", "renamed"),
            12 => format!("import std.core;\n{fixture}"),
            13 => format!("{fixture}\ndefinition\n let X, Y be set;\n private pred Hidden: X secret Y means X = Y;\nend;\n"),
            14 => format!("{fixture}\nprivate infix_operator(\"+\", left, 80);\n"),
            15 => format!("{fixture}\ndefinition\n let X, Y be set;\n synonym X alias Y for X base Y;\nend;\n"),
            16 => format!("{fixture}\nreserve Extra for object;\n"),
            17 => fixture.replace("-> object", "-> set"),
            18 => include_str!("../../../../tests/miz/fail/parser/fail_parser_algorithm_verification_recovery_001.miz").to_owned(),
            19 => format!("export std.core;\n{fixture}"),
            20 => format!("{fixture}\n{}", fixture.replace("swapalgo", "second")),
            21 => include_str!("../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz").to_owned(),
            _ => fixture.to_owned(),
        };
        let package = if case == 7 { "other-package" } else { "pkg" };
        let module_path = if case == 8 { "Nested.Main" } else { "main" };
        let edition = if case == 9 { "2026" } else { "2025" };
        let root = std::env::temp_dir().join(format!(
            "mizar-algorithm-summary-{}-{case}",
            std::process::id()
        ));
        let path = root.join("src/main.miz");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            NoImportedSummaries,
            MizarParserSeam,
        )
        .run(
            mizar_frontend::source::SourceUnitRequest {
                snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                    "mizar-session-build-snapshot-v1:{}",
                    "31".repeat(Hash::BYTE_LEN)
                ))
                .unwrap(),
                input: SourceInput {
                    package_id: PackageId::new(package),
                    module_path: ModulePath::new(module_path),
                    normalized_path: normalize_path(&root, &path).unwrap(),
                    edition: Edition::new(edition),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();
        std::fs::remove_dir_all(root).unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let module = module_id(package, module_path);
        let shells = crate::declarations::DeclarationShellCollector::new(ast, &module).collect();
        let collected = crate::symbols::SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module_path),
        )
        .collect(&module);
        let arena = SurfaceResolvedArena::lower(ast, &module).unwrap();
        let identity = ModuleSummaryIdentity {
            package_id: package.into(),
            package_version: Some(if case == 10 { "2.0" } else { "1.0" }.into()),
            lockfile_identity: Some("caller-lock".into()),
            module_path: module_path.into(),
            language_edition: edition.into(),
        };
        let before = (frontend.clone(), collected.clone(), identity.clone());
        let lexical = collected.export_frontend_lexical_contributions(&frontend, &identity);
        let exported = rows(&arena, collected.env(), &identity);
        let locals = frontend.tokens.local_declarations();
        if case <= 11 {
            assert!(
                frontend.diagnostics.is_empty() && collected.diagnostics().is_empty(),
                "{case}"
            );
            assert_eq!(lexical, Some(vec![]));
            assert!(locals.user_symbols.is_empty() && locals.operator_declarations.is_empty());
            assert!(frontend.preprocessed.import_stubs.is_empty());
            assert!(collected.env().imports().is_empty() && collected.env().exports().is_empty());
            assert_eq!(frontend.source.source_hash, hash_text(&text));
            let exported = exported.unwrap().unwrap();
            assert_eq!(exported.len(), usize::from(!matches!(case, 3 | 4)));
            assert!(exported.iter().all(|row| row.proof_status.is_none()));
            // Ordinary expected data comes from the two existing lower producers.
            let mut expected = ModuleSummary {
                schema_version: current_schema_version(),
                module: identity.clone(),
                source_hash: frontend.source.source_hash,
                interface_hash: frontend.source.source_hash,
                exported_symbols: exported,
                exported_labels: vec![],
                lexical_summary: ModuleLexicalSummary {
                    schema_version: "mizar-resolve/exported-lexical/v1".into(),
                    fingerprint: None,
                    contributions: lexical.unwrap(),
                },
                reexports: vec![],
                dependency_interfaces: vec![],
            };
            expected.refresh_interface_hash().unwrap();
            let json = module_summary_json(&expected).unwrap();
            assert_eq!(
                write_module_summary(&expected).unwrap(),
                canonical_json_bytes(&json)
            );
            assert_eq!(
                read_module_summary(
                    &json,
                    ModuleSummaryReadOptions {
                        artifact_path: Some("algorithm-summary.json"),
                        expected_module: Some(&identity),
                        expected_interface_hash: Some(expected.interface_hash),
                    }
                )
                .unwrap(),
                expected
            );
            if case == 0 {
                let row = &expected.exported_symbols[0];
                assert_eq!(
                    row.rendered_signature,
                    "let a be object ; terminating algorithm swapalgo ( a ) -> object ensures result = a"
                );
                let fingerprint = row
                    .interface_fingerprint
                    .as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                let symbol = J::object([
                    ("origin_id", J::string(&row.origin_id)),
                    ("fully_qualified_name", J::string(&row.fully_qualified_name)),
                    ("namespace_path", J::array([J::string("main")])),
                    ("visibility", J::string("public")),
                    ("declaration_kind", J::string("algorithm")),
                    ("rendered_signature", J::string("let a be object ; terminating algorithm swapalgo ( a ) -> object ensures result = a")),
                    ("interface_fingerprint", J::string(format!("mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/module-summary:1.0:{fingerprint}"))),
                    ("proof_status", J::Null),
                ]).unwrap();
                // This displayed interface projection excludes source hash and ranges.
                let projection = J::object([
                    ("schema_version", J::string("1.0")),
                    (
                        "module",
                        J::object([
                            ("package_id", J::string("pkg")),
                            ("package_version", J::string("1.0")),
                            ("lockfile_identity", J::string("caller-lock")),
                            ("module_path", J::string("main")),
                            ("language_edition", J::string("2025")),
                        ])
                        .unwrap(),
                    ),
                    ("exported_symbols", J::array([symbol])),
                    ("exported_labels", J::array([])),
                    (
                        "lexical_summary",
                        J::object([
                            (
                                "schema_version",
                                J::string("mizar-resolve/exported-lexical/v1"),
                            ),
                            ("fingerprint", J::Null),
                            ("contributions", J::array([])),
                        ])
                        .unwrap(),
                    ),
                    ("reexports", J::array([])),
                    ("dependency_interfaces", J::array([])),
                ])
                .unwrap();
                let domain = CanonicalHashDomain::new(
                    HashClass::Interface,
                    MODULE_SUMMARY_SCHEMA_FAMILY,
                    current_schema_version(),
                );
                assert_eq!(expected.interface_hash, domain.hash(&projection, &[]));
                original = Some((
                    frontend.clone(),
                    collected.clone(),
                    identity.clone(),
                    expected.clone(),
                ));
            }
            let baseline = &original.as_ref().unwrap().3;
            if matches!(case, 1 | 2) {
                assert_eq!(expected.interface_hash, baseline.interface_hash);
                assert_ne!(expected.source_hash, baseline.source_hash);
            }
            if matches!(case, 5 | 7..=11) {
                assert_ne!(expected.interface_hash, baseline.interface_hash);
            }
            let got = collected.source_algorithm_module_summary_candidate(&frontend, &identity);
            assert_eq!(got, Ok(Some(expected.clone())));
            assert_eq!(
                collected.source_algorithm_module_summary_candidate(&frontend, &identity),
                got
            );
            let mut invalid = identity.clone();
            invalid.package_version = Some(String::new());
            if case == 11 {
                let (first, first_collection, first_id, _) = original.as_ref().unwrap();
                assert_eq!(
                    first_collection.export_frontend_lexical_contributions(&frontend, &identity),
                    None
                );
                assert_eq!(
                    collected.export_frontend_lexical_contributions(first, first_id),
                    None
                );
                assert_eq!(
                    first_collection
                        .source_algorithm_module_summary_candidate(&frontend, &identity),
                    Ok(None)
                );
                assert_eq!(
                    first_collection.source_algorithm_module_summary_candidate(&frontend, &invalid),
                    Ok(None)
                );
                assert_eq!(
                    collected.source_algorithm_module_summary_candidate(first, first_id),
                    Ok(None)
                );
                let error = rows(&arena, first_collection.env(), &identity).unwrap_err();
                assert_eq!(
                    rows(&arena, first_collection.env(), &invalid).unwrap_err(),
                    error
                );
                let first_arena =
                    SurfaceResolvedArena::lower(first.ast.as_ref().unwrap(), &module).unwrap();
                assert!(rows(&first_arena, collected.env(), first_id).is_err());
            }
            assert_eq!(
                collected.export_frontend_lexical_contributions(&frontend, &invalid),
                None
            );
            assert_eq!(
                collected.source_algorithm_module_summary_candidate(&frontend, &invalid),
                Ok(None)
            );
            assert_eq!(
                rows(&arena, collected.env(), &invalid).unwrap_err(),
                invalid.canonical_json().unwrap_err().to_string()
            );
        } else {
            assert_eq!(
                collected.source_algorithm_module_summary_candidate(&frontend, &identity),
                Ok(None),
                "{case}"
            );
            match case {
                12 | 19 => {
                    assert!(
                        !frontend.preprocessed.import_stubs.is_empty()
                            || !shells.exports().is_empty()
                    );
                    assert_eq!(exported.unwrap(), None);
                }
                13 => {
                    assert!(!locals.user_symbols.is_empty());
                    assert_eq!(
                        lexical,
                        Some(vec![]),
                        "private lexical omission is not absence"
                    );
                    assert_eq!(
                        exported.unwrap(),
                        None,
                        "extra source also fails the row profile"
                    );
                    let (first, first_collection, first_id, _) = original.as_ref().unwrap();
                    let mut stale = first.clone();
                    stale.tokens.local_declarations = frontend.tokens.local_declarations.clone();
                    assert_eq!(
                        first_collection.export_frontend_lexical_contributions(&stale, first_id),
                        None
                    );
                    assert_eq!(
                        first_collection
                            .source_algorithm_module_summary_candidate(&stale, first_id),
                        Ok(None)
                    );
                }
                14 => {
                    assert!(!locals.operator_declarations.is_empty());
                    assert_eq!(lexical, None);
                }
                15 => assert_eq!(lexical, None),
                16 | 17 | 20 => assert_eq!(exported.unwrap(), None),
                18 => {
                    assert!(!frontend.diagnostics.is_empty());
                    assert_eq!(lexical, None);
                    let (first, first_collection, first_id, _) = original.as_ref().unwrap();
                    let mut diagnostic = first.clone();
                    diagnostic.diagnostics = frontend.diagnostics.clone();
                    assert_eq!(
                        first_collection
                            .export_frontend_lexical_contributions(&diagnostic, first_id),
                        None
                    );
                    assert_eq!(
                        first_collection
                            .source_algorithm_module_summary_candidate(&diagnostic, first_id),
                        Ok(None)
                    );
                }
                21 => assert!(!collected.diagnostics().is_empty()),
                _ => unreachable!(),
            }
        }
        assert_eq!((frontend, collected, identity), before);
    }
    let (first, collected, identity, _) = original.unwrap();
    for mutation in 0..5 {
        let mut stale = first.clone();
        match mutation {
            0 => stale.ast = None,
            1 => stale.source.source_hash = hash_text("stale"),
            2 => stale.source.source_text = "stale".into(),
            3 => stale.preprocessed.lexical_hash = hash_text("stale lexical text"),
            4 => stale.preprocessed.source_map.lexical_text_len += 1,
            _ => unreachable!(),
        }
        assert_eq!(
            collected.export_frontend_lexical_contributions(&stale, &identity),
            None
        );
        assert_eq!(
            collected.source_algorithm_module_summary_candidate(&stale, &identity),
            Ok(None)
        );
    }
}
