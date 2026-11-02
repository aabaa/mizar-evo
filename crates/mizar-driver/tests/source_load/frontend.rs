use super::*;

#[test]
fn frontend_publishes_only_clean_loaded_source_with_parent_and_storage_maps() {
    for threshold in [usize::MAX, 3] {
        let fixture = Fixture::new(b"\xef\xbb\xbfdefinition\r\nend;\r\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, threshold);
        assert_eq!(source.status, PhaseStatus::Complete);
        fixture.write(b"replaced after source publication");

        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Complete);
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.output_refs.len(), 1);
        let output = &result.output_refs[0];
        assert_eq!(output.phase(), &IrPipelinePhase::new("Frontend"));
        assert_eq!(output.output_kind(), &OutputKind::new("FrontendOutput"));
        assert_eq!(
            output.snapshot(),
            submission.session.captured().snapshot().id
        );
        assert!(
            matches!(
                output.placement(),
                StoragePlacement::Resident if threshold == usize::MAX
            ) || matches!(
                output.placement(),
                StoragePlacement::Blob { .. } if threshold != usize::MAX
            )
        );
        let lineage = publisher
            .registry()
            .output_lineage(output.output())
            .unwrap();
        assert_eq!(lineage.parents, vec![source.output_refs[0].output()]);
        assert!(
            lineage
                .named_input_hashes
                .iter()
                .any(|input| input.name == "active-lexical-environment")
        );
        assert!(
            lineage
                .named_input_hashes
                .iter()
                .any(|input| input.name == "tokens")
        );
        let tables = publisher.storage().side_tables_by_ref(output).unwrap();
        assert_eq!(tables.source_maps.len(), 1);
        assert_eq!(tables.source_maps[0].domain, "frontend-storage");
        let typed = publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                output,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = publisher.storage().get(&typed).unwrap();
        assert_eq!(loaded.source.source_id, fixture.version.source_id);
        assert_eq!(loaded.source.source_text.as_ref(), "definition\nend;\n");
        assert!(loaded.ast.is_some());
        assert!(loaded.diagnostics.is_empty());
        let wrong_parent = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![output.clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(wrong_parent.status, PhaseStatus::Blocking);
        assert!(wrong_parent.output_refs.is_empty());
    }
}

#[test]
fn frontend_recovery_and_unrecoverable_input_emit_diagnostics_without_output() {
    for (text, status, code) in [
        (b"definition\n".as_slice(), PhaseStatus::Recoverable, 10),
        (b"end".as_slice(), PhaseStatus::Fatal, 52),
    ] {
        let fixture = Fixture::new(text);
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, status);
        assert!(result.output_refs.is_empty());
        assert_eq!(result.diagnostics.len(), 1);
        assert!(
            result.diagnostics[0]
                .drafts()
                .iter()
                .any(|draft| draft.code().number() == code)
        );
    }
}

#[test]
fn frontend_diagnostic_only_import_reports_five_path_failures_from_live_source() {
    for (text, code, dependency_modules, needs_dependency, paths, primary_text) in [
        (
            "import mml.no_such;\ndefinition\nend;\n",
            220,
            &[][..],
            false,
            &["mml.no_such"][..],
            "mml.no_such",
        ),
        (
            "import dep.missing;\ndefinition\nend;\n",
            221,
            &["present"][..],
            true,
            &["dep.missing"][..],
            "dep.missing",
        ),
        (
            "import ..common;\ndefinition\nend;\n",
            222,
            &[][..],
            false,
            &["..common"][..],
            "..common",
        ),
        (
            "import dep.core as shared, dep.core as shared, dep.other as shared;\ndefinition\nend;\n",
            223,
            &["core", "other"][..],
            true,
            &["dep.core", "dep.core", "dep.other"][..],
            "shared",
        ),
        (
            "import dep.core as mml;\ndefinition\nend;\n",
            224,
            &["core"][..],
            true,
            &["dep.core"][..],
            "mml",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let references = dependency_modules
            .iter()
            .enumerate()
            .map(|(ordinal, path)| {
                let artifact = format!("dep/{path}.summary.json");
                let digest = hash(0x60 + ordinal as u8);
                (
                    DependencyModuleSummaryRef {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("dep"),
                            ModulePath::new(*path),
                        ),
                        artifact: artifact.clone(),
                        content_hash: digest,
                    },
                    DependencyArtifactRef::new(artifact, digest),
                )
            })
            .collect::<Vec<_>>();
        let mut dependency_hashes = references
            .iter()
            .map(|(indexed, _)| indexed.content_hash)
            .collect::<Vec<_>>();
        dependency_hashes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        let submission = fixture.submit_with_dependencies(
            &ids,
            &snapshots,
            references
                .iter()
                .map(|(_, captured)| captured.clone())
                .collect(),
            if needs_dependency {
                vec![DependencyArtifactIndex::new(
                    PackageId::new("dep"),
                    Vec::new(),
                    references.into_iter().map(|(indexed, _)| indexed).collect(),
                )]
            } else {
                Vec::new()
            },
        );
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        assert_eq!(source.status, PhaseStatus::Complete, "{text}");
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), dependency_hashes),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Recoverable, "{text}");
        assert!(result.output_refs.is_empty(), "{text}");
        assert_eq!(result.diagnostics.len(), 2, "{text}");
        let frontend = &result.diagnostics[0];
        assert_eq!(frontend.scope().phase(), DiagnosticPhase::Frontend);
        assert_eq!(frontend.drafts().len(), paths.len());
        assert!(
            frontend
                .drafts()
                .iter()
                .all(|draft| draft.code().number() == 22)
        );
        let resolver = &result.diagnostics[1];
        assert_eq!(resolver.scope().phase(), DiagnosticPhase::Resolver);
        assert_eq!(resolver.drafts().len(), paths.len(), "{text}");
        assert!(resolver.drafts().iter().all(|draft| {
            draft.code().number() == code
                && draft.phase() == DiagnosticPhase::Resolver
                && draft.category() == FailureCategory::ResolveError
        }));
        for (ordinal, draft) in resolver.drafts().iter().enumerate() {
            assert_eq!(
                draft.stable_detail_key(),
                mizar_diagnostics::registry::DiagnosticRegistry::builtin()
                    .lookup(draft.code())
                    .unwrap()
                    .semantic_name
            );
            let details = draft.details().entries();
            assert_eq!(
                details.get("import.source_module"),
                Some(&DiagnosticDetailValue::List(vec![
                    DiagnosticDetailValue::String("alpha".to_owned()),
                    DiagnosticDetailValue::String("main".to_owned()),
                ])),
                "{text}"
            );
            assert_eq!(
                details.get("import.ordinal"),
                Some(&DiagnosticDetailValue::Integer(ordinal as i64)),
                "{text}"
            );
            assert_eq!(
                details.get("import.path"),
                Some(&DiagnosticDetailValue::String(paths[ordinal].to_owned()))
            );
            let DiagnosticPrimaryLocation::Span(primary) = draft.primary_location() else {
                panic!("semantic failure has a loaded-source primary span");
            };
            let SourceAnchor::Range(primary_range) = primary.anchor() else {
                panic!("semantic failure retains Range shape");
            };
            assert_eq!(primary_range.source_id, fixture.version.source_id);
            assert_eq!(&text[primary_range.start..primary_range.end], primary_text);
            if code == 223 {
                assert_eq!(
                    details.get("import.alias"),
                    Some(&DiagnosticDetailValue::String("shared".to_owned()))
                );
                assert_eq!(
                    details.get("import.target"),
                    Some(&DiagnosticDetailValue::List(vec![
                        DiagnosticDetailValue::String("dep".to_owned()),
                        DiagnosticDetailValue::String(
                            if ordinal == 2 { "other" } else { "core" }.to_owned(),
                        ),
                    ]))
                );
                let alias_offsets = text
                    .match_indices("shared")
                    .map(|(start, _)| start)
                    .collect::<Vec<_>>();
                assert_eq!(primary.range().start, alias_offsets[ordinal]);
                assert_eq!(
                    draft
                        .secondary_spans()
                        .iter()
                        .map(|span| match span.anchor() {
                            SourceAnchor::Range(range) => *range,
                            _ => panic!("peer retains Range shape"),
                        })
                        .collect::<Vec<_>>(),
                    alias_offsets
                        .into_iter()
                        .enumerate()
                        .filter_map(|(peer, start)| {
                            (peer != ordinal).then_some(mizar_session::SourceRange {
                                source_id: fixture.version.source_id,
                                start,
                                end: start + "shared".len(),
                            })
                        })
                        .collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
fn frontend_diagnostic_only_import_keeps_distinct_branch_members() {
    let text = "import dep.{missing, absent};\ndefinition\nend;\n";
    let fixture = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let digest = hash(0x60);
    let artifact = "dep/present.summary.json";
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        vec![DependencyArtifactRef::new(artifact, digest)],
        vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            vec![DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new("present"),
                ),
                artifact: artifact.to_owned(),
                content_hash: digest,
            }],
        )],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(source.status, PhaseStatus::Complete);
    let result = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), vec![digest]),
        Vec::new(),
        None,
    );
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert_eq!(result.diagnostics.len(), 2);
    let [first, second] = result.diagnostics[1].drafts() else {
        panic!("one E0221 draft per branch member");
    };
    for (ordinal, (draft, member)) in [(first, "missing"), (second, "absent")]
        .into_iter()
        .enumerate()
    {
        assert_eq!(draft.code().number(), 221);
        let DiagnosticPrimaryLocation::Span(primary) = draft.primary_location() else {
            panic!("branch member source span");
        };
        let SourceAnchor::Range(member_range) = primary.anchor() else {
            panic!("retained range anchor");
        };
        assert_eq!(&text[member_range.start..member_range.end], member);
        assert_eq!(draft.secondary_spans().len(), 1);
        assert_eq!(
            &text[draft.secondary_spans()[0].range().start..draft.secondary_spans()[0].range().end],
            "dep"
        );
        let details = draft.details().entries();
        assert_eq!(
            details.get("import.branch_member"),
            Some(&DiagnosticDetailValue::Source(*member_range))
        );
        assert_eq!(
            details.get("import.path"),
            Some(&DiagnosticDetailValue::String(format!("dep.{member}")))
        );
        assert_eq!(
            details.get("import.ordinal"),
            Some(&DiagnosticDetailValue::Integer(ordinal as i64))
        );
    }
}

#[test]
fn frontend_mixed_or_malformed_import_diagnostics_do_not_continue_resolution() {
    for text in [
        "import std., pkg.math as ;\ndefinition\nend;\n",
        "import mml.no_such;\ndefinition\n",
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        assert_eq!(source.status, PhaseStatus::Complete);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert!(matches!(
            result.status,
            PhaseStatus::Recoverable | PhaseStatus::Fatal
        ));
        assert!(result.output_refs.is_empty());
        assert_eq!(result.diagnostics.len(), 1, "{text}");
        let drafts = result.diagnostics[0].drafts();
        assert_eq!(
            result.diagnostics[0].scope().phase(),
            DiagnosticPhase::Frontend
        );
        assert!(
            drafts.iter().any(|draft| draft.code().number() == 22),
            "{text}"
        );
        assert!(
            drafts.iter().any(|draft| draft.code().number() != 22),
            "{text}"
        );
    }

    let fixture = Fixture::new(b"import mml.no_such, dep.core;\ndefinition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let artifact = "dep/core.summary.json";
    let digest = hash(0x60);
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        vec![DependencyArtifactRef::new(artifact, digest)],
        vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            vec![DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new("core"),
                ),
                artifact: artifact.to_owned(),
                content_hash: digest,
            }],
        )],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(source.status, PhaseStatus::Complete);
    let artifact_root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&artifact_root).unwrap();
    let result = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), vec![digest]),
        vec![(PackageId::new("dep"), artifact_root)],
        None,
    );
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    let codes = result.diagnostics[0]
        .drafts()
        .iter()
        .map(|draft| draft.code().number())
        .collect::<Vec<_>>();
    assert!(
        codes.contains(&22),
        "unresolved path keeps E0022: {codes:?}"
    );
    assert!(
        codes.contains(&23),
        "missing summary keeps E0023: {codes:?}"
    );
}

#[test]
fn frontend_same_normalized_source_separates_semantic_and_storage_identity() {
    let mut hashes = Vec::new();
    for bytes in [
        b"\xef\xbb\xbfdefinition\r\nend;\r\n".as_slice(),
        b"definition\nend;\n".as_slice(),
    ] {
        let fixture = Fixture::new(bytes);
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Complete);
        hashes.push((
            result.output_refs[0].content_hash(),
            result.output_refs[0].side_table_hash(),
        ));
    }
    assert_eq!(hashes[0].0, hashes[1].0);
    assert_ne!(hashes[0].1, hashes[1].1);
}

#[test]
fn frontend_rejects_missing_wrong_or_mismatched_parent_identities() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    let source_ref = source.output_refs[0].clone();
    for (parents, input_hash, dependencies) in [
        (Vec::new(), source_key(&fixture.version), Vec::new()),
        (vec![source_ref.clone()], hash(7), Vec::new()),
        (
            vec![source_ref.clone()],
            source_key(&fixture.version),
            vec![hash(8)],
        ),
    ] {
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            parents,
            (input_hash, dependencies),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Blocking);
        assert!(result.output_refs.is_empty());
    }
}

#[test]
fn frontend_imports_require_one_captured_summary_artifact_identity() {
    use mizar_artifact::{
        module_summary::{
            ModuleLexicalSummary, ModuleSummary, ModuleSummaryIdentity, current_schema_version,
            module_summary_json,
        },
        store::{PublishedArtifactPath, artifact_hash_domain, write_published_artifact},
    };

    let fixture = Fixture::new(b"import dep.core;\ndefinition\nend;\n");
    let root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&root).unwrap();
    let artifact = "dep/core.summary.json";
    let mut summary = ModuleSummary {
        schema_version: current_schema_version(),
        module: ModuleSummaryIdentity {
            package_id: "dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("dependency-lock".to_owned()),
            module_path: "core".to_owned(),
            language_edition: "2025".to_owned(),
        },
        source_hash: hash(9),
        interface_hash: hash(8),
        exported_symbols: Vec::new(),
        exported_labels: Vec::new(),
        lexical_summary: ModuleLexicalSummary {
            schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
            fingerprint: None,
            contributions: Vec::new(),
        },
        reexports: Vec::new(),
        dependency_interfaces: Vec::new(),
    };
    summary.refresh_interface_hash().unwrap();
    let content_hash = write_published_artifact(
        &root,
        &PublishedArtifactPath::new(artifact).unwrap(),
        &module_summary_json(&summary).unwrap(),
        &artifact_hash_domain(
            mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
            summary.schema_version,
        ),
        &[],
    )
    .unwrap()
    .artifact_hash;
    let indexed = DependencyArtifactIndex::new(
        PackageId::new("dep"),
        Vec::new(),
        vec![DependencyModuleSummaryRef {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("dep"),
                ModulePath::new("core"),
            ),
            artifact: artifact.to_owned(),
            content_hash,
        }],
    );
    let matching = DependencyArtifactRef::new(artifact, content_hash);
    for (captured, expected) in [
        (vec![matching.clone()], PhaseStatus::Complete),
        (Vec::new(), PhaseStatus::Blocking),
        (
            vec![DependencyArtifactRef::new(artifact, hash(0x98))],
            PhaseStatus::Blocking,
        ),
        (
            vec![matching.clone(), matching.clone()],
            PhaseStatus::Blocking,
        ),
        (
            vec![
                matching.clone(),
                DependencyArtifactRef::new(artifact, hash(0x98)),
            ],
            PhaseStatus::Complete,
        ),
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission =
            fixture.submit_with_dependencies(&ids, &snapshots, captured, vec![indexed.clone()]);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        assert_eq!(source.status, PhaseStatus::Complete);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), vec![content_hash]),
            vec![(PackageId::new("dep"), root.clone())],
            None,
        );
        assert_eq!(result.status, expected);
        assert_eq!(
            result.output_refs.len(),
            usize::from(expected == PhaseStatus::Complete)
        );
        if expected == PhaseStatus::Complete {
            assert!(result.diagnostics.is_empty());
            let typed = publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    &result.output_refs[0],
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let loaded = publisher.storage().get(&typed).unwrap();
            let candidates = ImportPathCandidate::from_surface_ast(
                loaded.ast.as_ref().expect("sealed import AST"),
            )
            .expect("parsed import candidates");
            let [candidate] = candidates.as_slice() else {
                panic!("one parsed import candidate");
            };
            assert_eq!(candidate.components(), ["dep", "core"]);
            assert_eq!(candidate.ordinal(), 0);
            assert_eq!(candidate.range().source_id, loaded.source.source_id);
            assert_eq!(
                &loaded.source.source_text[candidate.range().start..candidate.range().end],
                "dep.core"
            );
            let current = ResolverModuleId::new(
                loaded.source.package_id.clone(),
                loaded.source.module_path.clone(),
            );
            let resolution = ImportPathResolver::new(ModuleIndexInput::new(
                submission.module_index.as_ref().unwrap(),
            ))
            .resolve(&current, &candidates);
            assert!(resolution.unresolved().is_empty());
            let [resolved] = resolution.resolved() else {
                panic!("one resolved import");
            };
            assert_eq!(
                resolved.target(),
                &ResolverModuleId::new(PackageId::new("dep"), ModulePath::new("core"))
            );
            assert_eq!(resolved.range(), candidate.range());
            assert_eq!(resolved.ordinal(), candidate.ordinal());
            let lineage = publisher
                .registry()
                .output_lineage(result.output_refs[0].output())
                .unwrap();
            assert!(
                lineage
                    .named_input_hashes
                    .iter()
                    .any(|input| { input.name == "dependency.0" && input.digest == content_hash })
            );
        }
    }
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        vec![matching.clone()],
        vec![indexed.clone()],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    for mutation in 0..4 {
        let mut changed = submission.module_index.as_ref().unwrap().clone();
        let dependency_module = changed
            .modules
            .iter()
            .position(|entry| entry.module.package.as_str() == "dep")
            .unwrap();
        match mutation {
            0 => {
                let ModuleIndexLocation::DependencySummary { artifact, .. } =
                    &mut changed.modules[dependency_module].location
                else {
                    unreachable!()
                };
                *artifact = "dep/other.summary.json".to_owned();
            }
            1 => changed.dependency_summaries[0].artifact = "dep/other.summary.json".to_owned(),
            2 => changed
                .dependency_summaries
                .push(changed.dependency_summaries[0].clone()),
            3 => changed
                .modules
                .push(changed.modules[dependency_module].clone()),
            _ => unreachable!(),
        }
        let dependencies = changed
            .dependency_summaries
            .iter()
            .map(|reference| reference.content_hash)
            .collect();
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), dependencies),
            vec![(PackageId::new("dep"), root.clone())],
            Some(&changed),
        );
        assert_eq!(
            result.status,
            PhaseStatus::Blocking,
            "index mutation {mutation}"
        );
        assert!(result.output_refs.is_empty());
    }
    fs::remove_file(root.join(artifact)).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission =
        fixture.submit_with_dependencies(&ids, &snapshots, vec![matching], vec![indexed]);
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    let missing_summary = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), vec![content_hash]),
        vec![(PackageId::new("dep"), root)],
        None,
    );
    assert_eq!(missing_summary.status, PhaseStatus::Recoverable);
    assert!(missing_summary.output_refs.is_empty());
    assert!(
        missing_summary.diagnostics[0]
            .drafts()
            .iter()
            .any(|draft| draft.code().number() == 23)
    );
}

#[test]
fn frontend_imported_symbol_conflict_uses_real_captured_summaries() {
    use mizar_artifact::{
        module_summary::{
            ExportedSymbolSummary, LexicalContributionSummary, ModuleLexicalSummary, ModuleSummary,
            ModuleSummaryIdentity, SourceRangeSummary, current_schema_version, module_summary_json,
        },
        store::{
            CanonicalJson, PublishedArtifactPath, artifact_hash_domain, canonical_json_string,
            write_published_artifact,
        },
    };
    use mizar_frontend::lexical_env::{
        ExportRank, ExportedSymbolShape, ModuleId as LexerModuleId, SymbolId, UserSymbolArity,
        UserSymbolKind,
    };

    let fixture = Fixture::new(b"import dep.core, dep.other;\ndefinition\nend;\n");
    let root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&root).unwrap();
    let mut indexed = Vec::new();
    let mut captured = Vec::new();
    for module_path in ["core", "other"] {
        let module = ModuleSummaryIdentity {
            package_id: "dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("dependency-lock".to_owned()),
            module_path: module_path.to_owned(),
            language_edition: "2025".to_owned(),
        };
        let origin_id = "symbol:combine";
        let shape = ExportedSymbolShape {
            spelling: "combine".to_owned(),
            symbol_id: SymbolId::new(canonical_json_string(&CanonicalJson::array([
                module.canonical_json().unwrap(),
                CanonicalJson::string(origin_id),
            ]))),
            source_module: LexerModuleId::new(canonical_json_string(
                &module.canonical_json().unwrap(),
            )),
            export_rank: ExportRank::new(0),
            kind: UserSymbolKind::Functor,
            arity: UserSymbolArity::exact(2),
            operator: None,
        };
        let mut summary = ModuleSummary {
            schema_version: current_schema_version(),
            module,
            source_hash: hash(9),
            interface_hash: hash(8),
            exported_symbols: vec![ExportedSymbolSummary {
                origin_id: origin_id.to_owned(),
                fully_qualified_name: format!("{module_path}.combine"),
                namespace_path: vec![module_path.to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "functor".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 0,
                    end_byte: 1,
                },
                rendered_signature: format!("functor {module_path}.combine"),
                interface_fingerprint: hash(8),
                proof_status: None,
            }],
            exported_labels: Vec::new(),
            lexical_summary: ModuleLexicalSummary {
                schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
                fingerprint: None,
                contributions: vec![LexicalContributionSummary {
                    kind: "exported-symbol".to_owned(),
                    key: origin_id.to_owned(),
                    payload: String::from_utf8(shape.canonical_bytes().unwrap()).unwrap(),
                }],
            },
            reexports: Vec::new(),
            dependency_interfaces: Vec::new(),
        };
        summary.refresh_interface_hash().unwrap();
        let artifact = format!("dep/{module_path}.summary.json");
        let content_hash = write_published_artifact(
            &root,
            &PublishedArtifactPath::new(&artifact).unwrap(),
            &module_summary_json(&summary).unwrap(),
            &artifact_hash_domain(
                mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
                summary.schema_version,
            ),
            &[],
        )
        .unwrap()
        .artifact_hash;
        indexed.push(DependencyModuleSummaryRef {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("dep"),
                ModulePath::new(module_path),
            ),
            artifact: artifact.clone(),
            content_hash,
        });
        captured.push(DependencyArtifactRef::new(artifact, content_hash));
    }
    let mut dependencies = captured
        .iter()
        .map(|item| item.content_hash)
        .collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        captured,
        vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            indexed,
        )],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(source.status, PhaseStatus::Complete);
    let result = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), dependencies),
        vec![(PackageId::new("dep"), root)],
        None,
    );
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert!(
        result.diagnostics[0]
            .drafts()
            .iter()
            .any(|draft| draft.code().number() == 24)
    );
}

#[test]
fn frontend_requires_current_resources_and_scopes_cancellation() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    let snapshot = submission.session.captured().snapshot().id;
    let task = submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    let parent = mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
        &publisher,
        snapshot,
        source.output_refs[0].clone(),
    )
    .unwrap();
    let source_load = SourceLoadInputs {
        captured: None,
        snapshot: submission.session.captured().snapshot(),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(Vec::new());
    let registry = builder.build().unwrap();
    let sink = || {
        DiagnosticSink::new(DiagnosticProducerScope::new(
            DiagnosticPhase::Frontend,
            snapshot,
            "frontend.run_loaded",
        ))
    };
    let run = |resources: PhaseExecutionResources<'_>| {
        registry
            .execute_phase_with_resources(
                PipelinePhase::Frontend,
                PhaseInput::new(
                    task.unit.clone(),
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        source_key(&fixture.version),
                        Vec::new(),
                        vec![parent.clone()],
                    )
                    .unwrap(),
                ),
                resources,
            )
            .unwrap()
            .result
    };
    for resources in [
        PhaseExecutionResources {
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        },
        PhaseExecutionResources {
            diagnostics: Some(sink()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        },
        PhaseExecutionResources {
            diagnostics: Some(sink()),
            output_publisher: Some(publisher.clone()),
            ..PhaseExecutionResources::default()
        },
        PhaseExecutionResources {
            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                DiagnosticPhase::SourceLoad,
                snapshot,
                "wrong.phase",
            ))),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        },
    ] {
        let result = run(resources);
        assert_eq!(result.status, PhaseStatus::Blocking);
        assert!(result.output_refs.is_empty());
    }
    let cancelled = run(PhaseExecutionResources {
        cancellation: Some(CancellationToken {
            snapshot,
            generation: CancellationGeneration::new(1),
            reason: CancellationReason::ExplicitRequest,
        }),
        diagnostics: Some(sink()),
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
    });
    assert_eq!(cancelled.status, PhaseStatus::Cancelled);
    assert!(cancelled.output_refs.is_empty());
    let foreign = run(PhaseExecutionResources {
        cancellation: Some(CancellationToken {
            snapshot: snapshot_id(0xaa),
            generation: CancellationGeneration::new(1),
            reason: CancellationReason::ExplicitRequest,
        }),
        diagnostics: Some(sink()),
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
    });
    assert_eq!(foreign.status, PhaseStatus::Blocking);
    assert!(foreign.output_refs.is_empty());
    let denied = run(PhaseExecutionResources {
        diagnostics: Some(sink()),
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(denied.status, PhaseStatus::Blocking);
    assert!(denied.output_refs.is_empty());
    publisher.mark_obsolete(snapshot).unwrap();
    let stale = run(PhaseExecutionResources {
        diagnostics: Some(sink()),
        output_publisher: Some(publisher),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(stale.status, PhaseStatus::Blocking);
    assert!(stale.output_refs.is_empty());
}

#[test]
fn frontend_real_diagnostic_producer_matrix() {
    let cases: &[(&str, &[u16])] = &[
        ("definition\rend;", &[13, 21, 29, 38]),
        ("α\n::=\n", &[14, 15]),
        ("import ;\ndefinition\nend;", &[16, 43]),
        (
            "import std., pkg.math as ;\ndefinition\nend;",
            &[17, 18, 22, 41, 51],
        ),
        ("import std.core\ndefinition\nend;", &[19]),
        ("import @;\ndefinition\nend;", &[20]),
        ("let , x be set;\nfor + y holds thesis;", &[30, 31, 34, 47]),
        ("definition\nlet x,x be set;\nend;", &[32]),
        ("end;\ndefinition\nlet x be set;", &[33, 10, 52]),
        ("\"abc\"", &[35]),
        ("infix_operator(\"bad\\n\",left,80);", &[2, 42]),
        ("theorem T: x = y &;\n", &[48]),
        ("private ;", &[45]),
        ("export 123;", &[44]),
        ("theorem T: thesis by ;", &[49]),
        ("@[ label theorem T: thesis;", &[50]),
        ("theorem T: thesis;\nexport foo;", &[51]),
        ("definition\nlet x be ;\nend;", &[46]),
        (
            "func Plus: +(x,y) -> set; infix_operator(\"+\",left,80); theorem T: a + ;",
            &[39],
        ),
        (
            "func Plus: +(x,y) -> set; infix_operator(\"+\",none,80); theorem T: a + b + c;",
            &[40],
        ),
    ];
    for &(text, expected) in cases {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert!(matches!(
            result.status,
            PhaseStatus::Recoverable | PhaseStatus::Fatal
        ));
        assert!(result.output_refs.is_empty());
        let actual = result
            .diagnostics
            .iter()
            .flat_map(|batch| batch.drafts())
            .map(|draft| draft.code().number())
            .collect::<Vec<_>>();
        for &code in expected {
            assert!(
                actual.contains(&code),
                "{text:?} must emit E{code:04}, got {actual:?}"
            );
        }
    }
}
