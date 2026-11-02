use super::*;

#[test]
fn supplied_workspace_leaf_public_symbol_lexes_importer_and_seals_both_parents() {
    let importer = b"import alpha.leaf;\ntheorem T: a combine b = a;\n";
    let leaf =
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
    for threshold in [usize::MAX, 3] {
        for reverse in [false, true] {
            let fixture = WorkspaceLeafFixture::new(importer, leaf, None, threshold);
            let (own_output, leaf_output) = fixture.clean_parents();
            assert_eq!(
                matches!(leaf_output.placement(), StoragePlacement::Resident),
                threshold == usize::MAX
            );
            fs::remove_file(fixture.fixture.root.join("alpha/src/leaf.miz")).unwrap();
            let parents = if reverse {
                vec![leaf_output.clone(), own_output.clone()]
            } else {
                vec![own_output.clone(), leaf_output.clone()]
            };
            let importer_frontend = fixture.frontend("main", parents, None);
            assert_eq!(importer_frontend.status, PhaseStatus::Complete);
            assert!(importer_frontend.diagnostics.is_empty());
            let output = &importer_frontend.output_refs[0];
            let lineage = fixture
                .publisher
                .registry()
                .output_lineage(output.output())
                .unwrap();
            assert_eq!(lineage.parents.len(), 2);
            assert!(lineage.parents.contains(&own_output.output()));
            assert!(lineage.parents.contains(&leaf_output.output()));
            let typed = fixture
                .publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    output,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let loaded = fixture.publisher.storage().get(&typed).unwrap();
            assert_eq!(loaded.source.source_id, fixture.fixture.version.source_id);
            assert!(loaded.ast.is_some());
            let candidates =
                ImportPathCandidate::from_surface_ast(loaded.ast.as_ref().unwrap()).unwrap();
            let resolution = ImportPathResolver::new(ModuleIndexInput::new(
                fixture.submission.module_index.as_ref().unwrap(),
            ))
            .resolve(
                &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main")),
                &candidates,
            );
            assert_eq!(resolution.resolved().len(), 1);
            assert_eq!(
                resolution.resolved()[0].target(),
                &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf"))
            );
            assert!(loaded.tokens.tokens().iter().any(|token| {
                token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol
            }));
        }
    }
}

#[test]
fn supplied_workspace_leaf_rejects_absent_unused_or_wrong_source_parent() {
    let leaf =
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
    for mode in ["absent", "unused", "wrong_source"] {
        let importer = if mode == "unused" {
            b"definition\nend;\n".as_slice()
        } else {
            b"import alpha.leaf;\ndefinition\nend;\n".as_slice()
        };
        let fixture = WorkspaceLeafFixture::new(importer, leaf, None, usize::MAX);
        if mode == "wrong_source" {
            let leaf_source = fixture.source("leaf");
            assert_eq!(leaf_source.status, PhaseStatus::Complete);
            let leaf_frontend = fixture.frontend("leaf", leaf_source.output_refs.clone(), None);
            assert_eq!(leaf_frontend.status, PhaseStatus::Complete);
            let result = fixture.frontend(
                "main",
                vec![
                    leaf_source.output_refs[0].clone(),
                    leaf_frontend.output_refs[0].clone(),
                ],
                None,
            );
            assert_eq!(result.status, PhaseStatus::Blocking);
            assert!(result.output_refs.is_empty());
            continue;
        }
        let (own_source, leaf_output) = fixture.clean_parents();
        let parents = match mode {
            "absent" => vec![own_source],
            "unused" => vec![own_source, leaf_output],
            _ => unreachable!(),
        };
        let result = fixture.frontend("main", parents, None);
        assert_eq!(result.status, PhaseStatus::Blocking, "{mode}");
        assert!(result.output_refs.is_empty(), "{mode}");
    }
}

#[test]
fn supplied_workspace_leaf_duplicate_handle_is_rejected_by_ir_bundle() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ndefinition\nend;\n",
        b"definition\nend;\n",
        None,
        usize::MAX,
    );
    let (own_source, leaf_output) = fixture.clean_parents();
    let snapshot = fixture.submission.session.captured().snapshot().id;
    let seal = |output| {
        mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
            &fixture.publisher,
            snapshot,
            output,
        )
        .unwrap()
    };
    assert!(
        PhaseDispatchInputBundle::new(
            snapshot,
            source_key(fixture.version("main")),
            Vec::new(),
            vec![
                seal(own_source),
                seal(leaf_output.clone()),
                seal(leaf_output)
            ],
        )
        .is_err()
    );
}

#[test]
fn supplied_workspace_leaf_rejects_tampered_captured_source_and_index_identity() {
    for mode in [
        "source_id",
        "source_hash",
        "source_path",
        "edition",
        "index_edition",
        "index_version",
    ] {
        let mut fixture = WorkspaceLeafFixture::new(
            b"import alpha.leaf;\ndefinition\nend;\n",
            b"definition\nend;\n",
            None,
            usize::MAX,
        );
        let (own_source, leaf_output) = fixture.clean_parents();
        let mut forged_snapshot = fixture.submission.session.captured().snapshot().clone();
        match mode {
            "source_id" => {
                forged_snapshot
                    .source_versions
                    .iter_mut()
                    .find(|version| version.module_path.as_str() == "leaf")
                    .unwrap()
                    .source_id = fixture.ids.next_source_id(snapshot_id(0x53)).unwrap()
            }
            "source_hash" => {
                forged_snapshot
                    .source_versions
                    .iter_mut()
                    .find(|version| version.module_path.as_str() == "leaf")
                    .unwrap()
                    .source_hash = hash(0xee)
            }
            "source_path" => {
                forged_snapshot
                    .source_versions
                    .iter_mut()
                    .find(|version| version.module_path.as_str() == "leaf")
                    .unwrap()
                    .normalized_path = fixture.fixture.version.normalized_path.clone()
            }
            "edition" => {
                forged_snapshot
                    .source_versions
                    .iter_mut()
                    .find(|version| version.module_path.as_str() == "leaf")
                    .unwrap()
                    .edition = Edition::new("2026")
            }
            "index_edition" => {
                fixture
                    .submission
                    .module_index
                    .as_mut()
                    .unwrap()
                    .modules
                    .iter_mut()
                    .find(|entry| entry.module.path.as_str() == "leaf")
                    .unwrap()
                    .edition = Edition::new("2026")
            }
            "index_version" => {
                fixture
                    .submission
                    .module_index
                    .as_mut()
                    .unwrap()
                    .packages
                    .iter_mut()
                    .find(|entry| entry.package_id.as_str() == "alpha")
                    .unwrap()
                    .version = "9.9.9".parse().unwrap()
            }
            _ => unreachable!(),
        }
        let result = fixture.frontend(
            "main",
            vec![own_source, leaf_output],
            Some(&forged_snapshot),
        );
        assert_eq!(result.status, PhaseStatus::Blocking, "{mode}");
        assert!(result.output_refs.is_empty(), "{mode}");
    }
}

#[test]
fn supplied_workspace_leaf_rejects_stale_parent() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ndefinition\nend;\n",
        b"definition\nend;\n",
        None,
        usize::MAX,
    );
    let (own_source, leaf_output) = fixture.clean_parents();
    let snapshot = fixture.submission.session.captured().snapshot().id;
    let sealed = [own_source, leaf_output]
        .into_iter()
        .map(|output| {
            mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
                &fixture.publisher,
                snapshot,
                output,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let bundle = PhaseDispatchInputBundle::new(
        snapshot,
        source_key(fixture.version("main")),
        Vec::new(),
        sealed,
    )
    .unwrap();
    fixture.publisher.mark_obsolete(snapshot).unwrap();
    let result = fixture.frontend_bundle("main", bundle, None);
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
}

#[test]
fn supplied_workspace_leaf_private_only_exports_match_empty_lexical_environment() {
    let importer = b"import alpha.leaf;\ndefinition\nend;\n";
    let mut fingerprints = Vec::new();
    for leaf in [
        b"definition\nend;\n".as_slice(),
        b"definition\nlet x, y be set;\nprivate pred Hidden: x hidden y means thesis;\nend;\n"
            .as_slice(),
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n"
            .as_slice(),
    ] {
        let fixture = WorkspaceLeafFixture::new(importer, leaf, None, usize::MAX);
        let (own_source, leaf_output) = fixture.clean_parents();
        let result = fixture.frontend("main", vec![own_source, leaf_output], None);
        assert_eq!(result.status, PhaseStatus::Complete);
        assert!(result.diagnostics.is_empty());
        let typed = fixture
            .publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                &result.output_refs[0],
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = fixture.publisher.storage().get(&typed).unwrap();
        fingerprints.push(loaded.cache_keys.active_lexical_environment.stable_hash());
    }
    assert_eq!(fingerprints[0], fingerprints[1]);
    assert_ne!(fingerprints[0], fingerprints[2]);
}

#[test]
fn supplied_workspace_leaf_preserves_importer_diagnostics() {
    for (importer, expected) in [
        (
            b"import alpha.leaf, mml.no_such;\ndefinition\nend;\n".as_slice(),
            &[22, 220][..],
        ),
        (b"import alpha.leaf;\ndefinition\n".as_slice(), &[10][..]),
    ] {
        let fixture = WorkspaceLeafFixture::new(importer, b"definition\nend;\n", None, usize::MAX);
        let (own_source, leaf_output) = fixture.clean_parents();
        let result = fixture.frontend("main", vec![own_source, leaf_output], None);
        assert_eq!(result.status, PhaseStatus::Recoverable);
        assert!(result.output_refs.is_empty());
        let codes = result
            .diagnostics
            .iter()
            .flat_map(|batch| batch.drafts())
            .map(|draft| draft.code().number())
            .collect::<Vec<_>>();
        for code in expected {
            assert!(codes.contains(code), "missing E{code:04}: {codes:?}");
        }
    }
}

#[test]
fn supplied_workspace_leaf_rejects_reexports_and_unsupported_lexical_shapes() {
    for (case, leaf) in [
        ("reexport", b"export Demo;\ndefinition\nend;\n".as_slice()),
        ("alias", b"definition\nlet X, Y be set;\nfunc Base: X \\+\\ Y -> set equals X;\nsynonym X <+> Y for X \\+\\ Y;\nend;\n".as_slice()),
        ("operator", b"definition\ninfix_operator(\"+\", left, 80);\nend;\n".as_slice()),
    ] {
        let fixture = WorkspaceLeafFixture::new(
            b"import alpha.leaf;\ndefinition\nend;\n", leaf, None, usize::MAX);
        let (own_source, leaf_output) = fixture.clean_parents();
        let result = fixture.frontend("main", vec![own_source, leaf_output], None);
        assert_eq!(result.status, PhaseStatus::Blocking, "{case}");
        assert!(result.output_refs.is_empty(), "{case}");
    }
}

#[test]
fn supplied_workspace_intermediate_rejects_reexport_and_changed_captured_import_index() {
    for mode in ["reexport", "missing_import_index"] {
        let intermediate = if mode == "reexport" {
            b"import alpha.base;\nexport alpha.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n".as_slice()
        } else {
            b"import alpha.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n".as_slice()
        };
        let mut fixture = WorkspaceLeafFixture::new(
            b"import alpha.leaf;\ntheorem T: a arrange b = a;\n",
            intermediate,
            Some(b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n"),
            usize::MAX,
        );
        let base_source = fixture.source("base");
        assert_eq!(base_source.status, PhaseStatus::Complete);
        let base_frontend = fixture.frontend("base", base_source.output_refs, None);
        assert_eq!(base_frontend.status, PhaseStatus::Complete);
        let leaf_source = fixture.source("leaf");
        assert_eq!(leaf_source.status, PhaseStatus::Complete);
        let leaf_frontend = fixture.frontend(
            "leaf",
            vec![
                leaf_source.output_refs[0].clone(),
                base_frontend.output_refs[0].clone(),
            ],
            None,
        );
        assert_eq!(leaf_frontend.status, PhaseStatus::Complete, "{mode}");
        assert!(leaf_frontend.diagnostics.is_empty(), "{mode}");
        let main_source = fixture.source("main");
        assert_eq!(main_source.status, PhaseStatus::Complete);
        if mode == "missing_import_index" {
            fixture
                .submission
                .module_index
                .as_mut()
                .unwrap()
                .modules
                .retain(|entry| entry.module.path.as_str() != "base");
        }
        let result = fixture.frontend(
            "main",
            vec![
                main_source.output_refs[0].clone(),
                leaf_frontend.output_refs[0].clone(),
            ],
            None,
        );
        assert_eq!(result.status, PhaseStatus::Blocking, "{mode}");
        assert!(result.output_refs.is_empty(), "{mode}");
    }
}
