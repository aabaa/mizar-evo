use super::*;

#[test]
fn native_theorem_module_candidates_preserve_captured_pairs_and_original_owners() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let quantified = "101:3:for100:1:X101:5:being101:3:set101:5:holds100:1:X102:1:=100:1:X";
    let numeral = "103:1:0102:1:=103:1:0";
    for (text, public, citations) in [
        (private.to_owned(), vec![("UsePriv1", quantified)], vec![("UsePriv1", "PrivT1")]),
        (private.replace("theorem UsePriv1", "lemma UsePriv1"), vec![("UsePriv1", quantified)], vec![("UsePriv1", "PrivT1")]),
        (private.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"), vec![], vec![("UsePriv1", "PrivT1")]),
        (include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(), vec![("OpenT1", quantified), ("AxT1", quantified)], vec![]),
        (format!("{private}\ntheorem Alpha: 0 = 0\nproof\n  thus 0 = 0 by UsePriv1;\nend;\n"), vec![("UsePriv1", quantified), ("Alpha", numeral)], vec![("UsePriv1", "PrivT1"), ("Alpha", "UsePriv1")]),
        (format!("\u{feff}{}", private.replace("holds X = X", "holds X ::= 日本語 formula =:: = X").replace('\n', "\r\n")), vec![("UsePriv1", quantified)], vec![("UsePriv1", "PrivT1")]),
    ] {
        for open in [false, true] {
            let fixture = Fixture::new(text.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (mut draft, input) = fixture.submit_request(vec![], vec![]);
            draft.source_inputs.versions.clear();
            let descriptor = SourceInput {
                package_id: fixture.version.package_id.clone(),
                module_path: fixture.version.module_path.clone(),
                normalized_path: fixture.version.normalized_path.clone(),
                edition: fixture.version.edition.clone(),
                origin: if open {
                    SourceOriginInput::OpenBuffer {
                        uri: format!("file://{}", fixture.root.join("alpha/src/main.miz").display()),
                        expected_version: 7,
                        actual_version: 7,
                        text: std::sync::Arc::from(text.as_str()),
                    }
                } else {
                    SourceOriginInput::Disk { path: "src/main.miz".into() }
                },
            };
            if open {
                fixture.write(&[0xff]);
            }
            let session = draft.capture_sources(
                &ids,
                &snapshots,
                vec![(DiskSourceLoader::new(fixture.root.join("alpha")), descriptor.clone())],
            ).unwrap();
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            let submission = CompilerDriver::new(builder.build().unwrap())
                .submit_captured(session, &ids, &snapshots, input).unwrap();
            let (inputs, _) = native_disk_lookup_context(&submission, &ids);
            let module = mizar_build::module_index::ModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            let (_, loaded, captured_input) = &inputs.captured.unwrap().native_sources().unwrap()[0];
            let source = mizar_frontend::source::source_unit_from_loaded(
                loaded.clone(), mizar_frontend::source::diagnostic_file_path(captured_input),
            );
            assert_eq!(source.source_text.as_ref(), text.trim_start_matches('\u{feff}').replace("\r\n", "\n"));
            let output = mizar_frontend::orchestration::Frontend::new(
                mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
                inputs.dependency_lexical_provider(&[]), MizarParserSeam,
            ).run_loaded(source.clone()).unwrap();
            assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
            assert!(output.cache_keys.ast.is_some());
            let ast = output.ast.as_ref().unwrap();
            let resolver_module = mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
            let shells = mizar_resolve::declarations::DeclarationShellCollector::new(ast, &resolver_module).collect();
            let collection = mizar_resolve::symbols::SignatureProjectionExtractor::new(
                ast, &shells, mizar_resolve::env::NamespacePath::new("main"),
            ).collect(&resolver_module);
            assert!(collection.diagnostics().is_empty());
            let arena = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &resolver_module).unwrap();
            let expected = mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, collection.env()).unwrap().unwrap();
            assert_eq!(expected.0.source_id(), loaded.source_id);
            assert_eq!(expected.0.module_id(), &resolver_module);
            assert_eq!(expected.1.module_id(), &resolver_module);
            let public_keys = public.iter().map(|(name, payload)| {
                let owners = collection.env().symbols().iter().filter(|owner| owner.primary_spelling() == *name).collect::<Vec<_>>();
                let [owner] = owners.as_slice() else { panic!("one original owner for {name}") };
                assert_eq!(owner.visibility(), mizar_resolve::env::Visibility::Public);
                assert_eq!(owner.origin().source_id(), loaded.source_id);
                assert_eq!(owner.symbol().module(), &resolver_module);
                let signature = mizar_resolve::env::SignatureShell::Opaque {
                    schema: "source-theorem-statement-tokens-v1".to_owned(), payload: (*payload).to_owned(),
                };
                assert_eq!(expected.1.symbols().get(owner.symbol()).unwrap().signature(), Some(&signature));
                assert_eq!(expected.1.definitions().by_symbol(owner.symbol()).unwrap().signature(), Some(&signature));
                owner.symbol().clone()
            }).collect::<std::collections::BTreeSet<_>>();
            assert_eq!(expected.1.symbols().iter().filter(|owner| owner.visibility() == mizar_resolve::env::Visibility::Public).map(|owner| owner.symbol().clone()).collect::<std::collections::BTreeSet<_>>(), public_keys);
            assert_eq!(expected.1.definitions().iter().filter(|owner| owner.visibility() == mizar_resolve::env::Visibility::Public).map(|owner| owner.symbol().clone()).collect::<std::collections::BTreeSet<_>>(), public_keys);
            assert_eq!(expected.0.label_refs().len(), citations.len());
            let edges = expected.1.declaration_dependencies().iter().collect::<Vec<_>>();
            assert_eq!(edges.len(), citations.len());
            for (edge, (source_name, target_name)) in edges.iter().zip(&citations) {
                let source_owner = collection.env().symbols().iter().find(|owner| owner.primary_spelling() == *source_name).unwrap();
                let target_owner = collection.env().symbols().iter().find(|owner| owner.primary_spelling() == *target_name).unwrap();
                assert_eq!(edge.source(), &mizar_resolve::env::DependencyEndpoint::Symbol(source_owner.symbol().clone()));
                let target = expected.1.labels().iter().find(|label| label.origin() == target_owner.origin()).unwrap();
                assert_eq!(edge.target(), &mizar_resolve::env::DependencyEndpoint::Label(target.origin_path().clone()));
            }
            let before = (inputs.snapshot.clone(), inputs.build_plan.clone(), inputs.module_index.clone(), loaded.clone(), captured_input.clone(), ast.clone(), arena.clone(), collection.env().clone());
            for stage in 0..3 {
                if stage == 1 {
                    fixture.write(b"open theorem Changed: 0 = 1;\n");
                } else if stage == 2 {
                    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                }
                assert_eq!(inputs.native_theorem_module_candidates(&module).unwrap(), Some(expected.clone()), "open={open}, stage={stage}");
            }
            mizar_resolve::symbols::validate_source_symbol_env(&arena, collection.env()).unwrap();
            assert_eq!((inputs.snapshot.clone(), inputs.build_plan.clone(), inputs.module_index.clone(), loaded.clone(), captured_input.clone(), ast.clone(), arena.clone(), collection.env().clone()), before);
            assert_eq!(captured_input, &descriptor);
        }
    }
}

#[test]
fn native_theorem_module_candidates_preserve_real_refusals_errors_and_binding_guards() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "unsupported"),
        (private.replace("by PrivT1", "by Missing"), "unresolved"),
        ("theorem Broken: ;\n".to_owned(), "diagnostic"),
        (private.replace("thus X = X;", "thus Y = Y;"), "error"),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let foreign = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, _, _, _) =
            native_disk_record_submission(&fixture, &ids, &snapshots, usize::MAX);
        let (_, foreign_submission, _, _, _) =
            native_disk_record_submission(&foreign, &ids, &snapshots, usize::MAX);
        let (inputs, _) = native_disk_lookup_context(&submission, &ids);
        let (foreign_inputs, _) = native_disk_lookup_context(&foreign_submission, &ids);
        let module = mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        );
        let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let output = mizar_frontend::orchestration::Frontend::new(
            mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(
                fixture.root.join("alpha"),
            )),
            inputs.dependency_lexical_provider(&[]),
            MizarParserSeam,
        )
        .run_loaded(mizar_frontend::source::source_unit_from_loaded(
            loaded.clone(),
            mizar_frontend::source::diagnostic_file_path(descriptor),
        ))
        .unwrap();
        assert_eq!(!output.diagnostics.is_empty(), case == "diagnostic");
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            descriptor.clone(),
        );
        if case == "diagnostic" {
            assert_eq!(
                inputs.native_theorem_module_candidates(&module).unwrap(),
                None
            );
        } else {
            let ast = output.ast.as_ref().unwrap();
            assert!(output.cache_keys.ast.is_some());
            let resolver_module = mizar_resolve::resolved_ast::ModuleId::new(
                module.package.clone(),
                module.path.clone(),
            );
            let shells =
                mizar_resolve::declarations::DeclarationShellCollector::new(ast, &resolver_module)
                    .collect();
            let collection = mizar_resolve::symbols::SignatureProjectionExtractor::new(
                ast,
                &shells,
                mizar_resolve::env::NamespacePath::new("main"),
            )
            .collect(&resolver_module);
            assert!(collection.diagnostics().is_empty());
            let arena =
                mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &resolver_module)
                    .unwrap();
            let lower = mizar_resolve::symbols::source_theorem_module_candidates(
                ast,
                &arena,
                collection.env(),
            );
            if case == "error" {
                assert_eq!(
                    mizar_resolve::names::SourceVariableScopeResolver::resolve_occurrences(
                        mizar_resolve::names::SourceVariableScopeInput::new(
                            ast,
                            &resolver_module,
                            collection.env()
                        )
                    )
                    .unwrap_err(),
                    mizar_resolve::names::SourceVariableScopeError::UnresolvedReference
                );
                let error = mizar_resolve::resolved_ast::source_theorem_resolved_ast(
                    ast,
                    &arena,
                    collection.env(),
                )
                .unwrap_err();
                assert_eq!(lower, Err(error.clone()));
                assert_eq!(inputs.native_theorem_module_candidates(&module), Err(error));
            } else {
                if case == "unresolved" {
                    let semantic = mizar_resolve::resolved_ast::source_theorem_resolved_ast(
                        ast,
                        &arena,
                        collection.env(),
                    )
                    .unwrap()
                    .unwrap();
                    assert!(semantic.label_refs().iter().any(|(_, row)| matches!(
                        row.resolution(),
                        mizar_resolve::resolved_ast::LabelResolution::Unresolved(_)
                    )));
                } else {
                    assert_eq!(
                        mizar_resolve::resolved_ast::source_theorem_resolved_ast(
                            ast,
                            &arena,
                            collection.env()
                        )
                        .unwrap(),
                        None
                    );
                }
                assert_eq!(lower.unwrap(), None);
                assert_eq!(
                    inputs.native_theorem_module_candidates(&module).unwrap(),
                    None
                );
            }
        }
        for requested in [
            mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("missing"),
            ),
            mizar_build::module_index::ModuleId::new(
                PackageId::new("foreign"),
                ModulePath::new("main"),
            ),
        ] {
            assert!(inputs.native_theorem_module_candidates(&requested).is_err());
        }
        let mut ambiguous = inputs.snapshot.clone();
        ambiguous
            .source_versions
            .push(ambiguous.source_versions[0].clone());
        let mut absent_index = inputs.module_index.clone();
        absent_index.modules.clear();
        let mut ambiguous_index = inputs.module_index.clone();
        ambiguous_index
            .modules
            .push(ambiguous_index.modules[0].clone());
        let mut foreign_index = foreign_inputs.module_index.clone();
        foreign_index.modules[0].module.package = PackageId::new("foreign");
        assert_ne!(&foreign_index, inputs.module_index);
        for invalid in [
            SourceLoadInputs {
                captured: None,
                ..inputs
            },
            SourceLoadInputs {
                captured: Some(foreign_submission.session.captured()),
                ..inputs
            },
            SourceLoadInputs {
                snapshot: &ambiguous,
                ..inputs
            },
            SourceLoadInputs {
                snapshot: foreign_inputs.snapshot,
                ..inputs
            },
            SourceLoadInputs {
                module_index: &absent_index,
                ..inputs
            },
            SourceLoadInputs {
                module_index: &ambiguous_index,
                ..inputs
            },
            SourceLoadInputs {
                module_index: &foreign_index,
                ..inputs
            },
        ] {
            let error = invalid
                .native_theorem_module_candidates(&module)
                .unwrap_err();
            assert!(
                error.contains("captured source version") || error.contains("capture binding"),
                "{error}"
            );
        }
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                descriptor.clone()
            ),
            before
        );
    }
}

#[test]
fn native_theorem_module_publication_seals_genuine_pair_and_streams() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let mixed = format!(
        "{}\ntheorem Numeral: 0 = 12\nproof\n  thus 0 = 12 by UsePriv1, PrivT1, PrivT1;\nend;\n",
        private.replace("by PrivT1", "by PrivT1, PrivT1")
    );
    let mut neutral = None;
    for (profile, text) in [
        mixed.clone(),
        private.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
        private.replace("theorem UsePriv1", "lemma UsePriv1"),
        include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(),
        format!("\u{feff}{}", mixed.replace("holds X = X", "holds X ::= 日本語 formula =:: = X").replace('\n', "\r\n")),
    ].into_iter().enumerate() {
        for open in [false, true] {
            for threshold in [usize::MAX, 0] {
                let fixture = Fixture::new(text.as_bytes());
                let ids = InMemorySessionIdAllocator::new();
                let snapshots = SnapshotRegistry::new();
                let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
                draft.source_inputs.versions.clear();
                let descriptor = SourceInput {
                    package_id: fixture.version.package_id.clone(),
                    module_path: fixture.version.module_path.clone(),
                    normalized_path: fixture.version.normalized_path.clone(),
                    edition: fixture.version.edition.clone(),
                    origin: if open {
                        SourceOriginInput::OpenBuffer {
                            uri: format!("file://{}", fixture.root.join("alpha/src/main.miz").display()),
                            expected_version: 7, actual_version: 7,
                            text: std::sync::Arc::from(text.as_str()),
                        }
                    } else { SourceOriginInput::Disk { path: "src/main.miz".into() } },
                };
                if open { fixture.write(&[0xff]); }
                let session = draft.capture_sources(&ids, &snapshots,
                    vec![(DiskSourceLoader::new(fixture.root.join("alpha")), descriptor.clone())]).unwrap();
                let snapshot = session.captured().snapshot().id;
                let module = mizar_build::module_index::ModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                let unit = WorkUnit::Module { module: module.clone() };
                let ir_unit = IrWorkUnit::new(format!("{:?}:{:?}", module.package.as_str(), module.path.as_str()));
                let owner = publisher(snapshot, threshold, &unit);
                allow_frontend(&owner, &unit);
                owner.allow_work_unit(AllowedWorkUnit::new(IrPipelinePhase::new("ModuleResolve"),
                    OutputKind::new("SourceTheoremModuleCandidates"), ir_unit.clone()));
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_load();
                builder.register_frontend(vec![]);
                let submission = CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone())
                    .submit_captured(session, &ids, &snapshots, submit).unwrap();
                let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
                let parent = submission.scheduler_run.as_ref().unwrap().phase_results.values().flatten()
                    .flat_map(|result| &result.output_refs)
                    .find(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput")).unwrap();
                let input = PhaseInput::new(unit, PhaseDispatchInputBundle::new(snapshot,
                    source_input.identities().input_hash(), vec![],
                    vec![SealedParentOutputHandle::from_published_output(&owner, snapshot, parent.clone()).unwrap()]).unwrap());
                let version = &inputs.snapshot.source_versions[0];
                let (_, loaded, captured_input) = &inputs.captured.unwrap().native_sources().unwrap()[0];
                let output = mizar_frontend::orchestration::Frontend::new(
                    mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
                    inputs.dependency_lexical_provider(&[]), MizarParserSeam,
                ).run_loaded(mizar_frontend::source::source_unit_from_loaded(loaded.clone(),
                    mizar_frontend::source::diagnostic_file_path(captured_input))).unwrap();
                assert!(output.diagnostics.is_empty());
                let typed = owner.storage().typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    parent, &OutputKind::new("FrontendOutput")).unwrap();
                assert_eq!(*owner.storage().get(&typed).unwrap(), output);
                let ast = output.ast.as_ref().unwrap();
                let resolver_module = ResolverModuleId::new(module.package.clone(), module.path.clone());
                let shells = DeclarationShellCollector::new(ast, &resolver_module).collect();
                let collection = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main")).collect(&resolver_module);
                assert!(collection.diagnostics().is_empty());
                let arena = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &resolver_module).unwrap();
                let expected = mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, collection.env()).unwrap().unwrap();
                assert_eq!(inputs.native_theorem_module_candidates_with_frontend_parent(&input, &owner).unwrap(), Some(expected.clone()));
                let (resolved, env, semantic) = mizar_resolve::symbols::source_theorem_module_semantic_payload(ast, &resolver_module).unwrap().unwrap();
                assert_eq!((resolved, env), expected);
                let bytes = mizar_resolve::symbols::source_theorem_module_candidates_canonical_bytes(ast, &resolver_module).unwrap().unwrap();
                assert_eq!(mizar_resolve::symbols::source_theorem_module_candidates_from_canonical_bytes(&bytes, loaded.source_id, &resolver_module), Some(expected.clone()));
                assert_eq!(expected.0.source_id(), loaded.source_id);
                assert_eq!(expected.0.module_id(), &resolver_module);
                assert_eq!(expected.1.module_id(), &resolver_module);
                let public = expected.1.symbols().iter().filter(|symbol| symbol.visibility() == mizar_resolve::env::Visibility::Public).collect::<Vec<_>>();
                assert_eq!(public.len(), match profile { 1 => 0, 2 => 1, _ => 2 });
                for symbol in public {
                    let payload = if symbol.primary_spelling() == "Numeral" { "103:1:0102:1:=103:2:12" }
                        else { "101:3:for100:1:X101:5:being101:3:set101:5:holds100:1:X102:1:=100:1:X" };
                    assert_eq!(symbol.signature(), Some(&mizar_resolve::env::SignatureShell::Opaque {
                        schema: "source-theorem-statement-tokens-v1".to_owned(), payload: payload.to_owned(),
                    }));
                    assert_eq!(expected.1.definitions().by_symbol(symbol.symbol()).unwrap().signature(), symbol.signature());
                }
                let targets = match profile {
                    0 | 4 => vec!["PrivT1", "PrivT1", "UsePriv1", "PrivT1", "PrivT1"],
                    1 | 2 => vec!["PrivT1"], _ => vec![],
                };
                assert_eq!(expected.0.label_refs().len(), targets.len());
                assert_eq!(expected.1.declaration_dependencies().iter().count(), targets.len());
                for ((_, row), target) in expected.0.label_refs().iter().zip(targets) {
                    assert_eq!(row.site().spelling(), target);
                    let symbol = expected.1.symbols().iter().find(|symbol| symbol.primary_spelling() == target).unwrap();
                    let mizar_resolve::resolved_ast::LabelResolution::Resolved(reference) = row.resolution() else { panic!("genuine citation must resolve") };
                    assert_eq!(reference.origin().as_str(), symbol.symbol().fqn().as_str());
                    assert_eq!(reference.range(), row.site().range());
                    assert_eq!(expected.1.labels().get(reference.origin()).unwrap().origin(), symbol.origin());
                }
                // Independent framing of the IR content formula, with the real lower semantic bytes.
                let mut hasher = blake3::Hasher::new();
                for field in [b"mizar-ir/publisher-content-hash/v1".as_slice(), semantic.as_slice()] {
                    hasher.update(&(field.len() as u64).to_le_bytes()); hasher.update(field);
                }
                hasher.update(&1_u64.to_le_bytes());
                hasher.update(&("parent_content".len() as u64).to_le_bytes());
                hasher.update(b"parent_content"); hasher.update(parent.content_hash().as_bytes());
                hasher.update(&1_u64.to_le_bytes());
                for field in ["source", SOURCE_UNIT_CACHE_KEY_VERSION, "mizar-ir/publisher-named-input-hash/v1"] {
                    hasher.update(&(field.len() as u64).to_le_bytes()); hasher.update(field.as_bytes());
                }
                hasher.update(source_key(version).as_bytes());
                let expected_content = Hash::from_bytes(*hasher.finalize().as_bytes());
                let mut blob_hasher = blake3::Hasher::new();
                let domain = b"mizar-ir/content-blob/v1";
                blob_hasher.update(&(domain.len() as u64).to_le_bytes()); blob_hasher.update(domain);
                blob_hasher.update(&1_u32.to_le_bytes());
                blob_hasher.update(&(bytes.len() as u64).to_le_bytes()); blob_hasher.update(&bytes);
                let expected_blob = Hash::from_bytes(*blob_hasher.finalize().as_bytes());
                if profile == 0 { neutral = Some((semantic.clone(), bytes.clone())); }
                else if profile == 4 {
                    let (neutral_semantic, neutral_storage) = neutral.as_ref().unwrap();
                    assert_eq!(&semantic, neutral_semantic);
                    assert_ne!(&bytes, neutral_storage);
                }
                let before = (inputs.snapshot.clone(), inputs.build_plan.clone(), inputs.module_index.clone(),
                    loaded.clone(), captured_input.clone(), output.clone(), arena.clone(), collection.env().clone(), input.clone(), parent.clone());
                let sealed = inputs.publish_native_theorem_module_candidates(&input, &owner).unwrap().unwrap();
                let original_handle = sealed.erase();
                for stage in 0..3 {
                    if stage == 1 { fixture.write(b"open theorem Changed: 0 = 1;\n"); }
                    else if stage == 2 { fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap(); }
                    assert_eq!(inputs.native_theorem_module_candidates_with_frontend_parent(&input, &owner).unwrap(), Some(expected.clone()));
                    assert_eq!(mizar_resolve::symbols::source_theorem_module_semantic_payload(ast, &resolver_module).unwrap(),
                        Some((expected.0.clone(), expected.1.clone(), semantic.clone())));
                    assert_eq!(mizar_resolve::symbols::source_theorem_module_candidates_canonical_bytes(ast, &resolver_module).unwrap(), Some(bytes.clone()));
                    let duplicate = mizar_ir::publisher::PublishError::Storage {
                        error: Box::new(mizar_ir::storage::StorageError::OutputAlreadyStored { output: sealed.output() }),
                    }.to_string();
                    assert_eq!(inputs.publish_native_theorem_module_candidates(&input, &owner).unwrap_err(), duplicate);
                    assert_eq!(sealed.any(), &original_handle);
                    assert_eq!(*owner.storage().get(&sealed).unwrap(), expected);
                    assert_eq!(sealed.snapshot(), snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("ModuleResolve"));
                    assert_eq!(sealed.output_kind(), &OutputKind::new("SourceTheoremModuleCandidates"));
                    assert_eq!(sealed.work_unit(), &ir_unit);
                    assert_eq!(sealed.schema_version(), mizar_ir::storage::SchemaVersion::new(1));
                    assert_eq!(sealed.lineage().parents, vec![parent.output()]);
                    assert_eq!(sealed.lineage().named_input_hashes, vec![mizar_ir::identity::NamedInputHash {
                        name: "source".to_owned(), domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(), digest: source_key(version),
                    }]);
                    assert_eq!(sealed.content_hash(), expected_content);
                    match sealed.placement() {
                        StoragePlacement::Resident => assert_eq!(threshold, usize::MAX),
                        StoragePlacement::Blob { blob, len } => {
                            assert_eq!(threshold, 0); assert_eq!(*len, bytes.len()); assert_eq!(blob.hash(), expected_blob);
                        }
                        _ => panic!("unexpected storage placement"),
                    }
                    assert_eq!(*owner.storage().side_tables(&sealed).unwrap(), mizar_ir::storage::IrSideTables {
                        source_maps: vec![mizar_ir::storage::SideTableRecord::new("resolve-theorem-module-storage",
                            version.normalized_path.as_str(), Hash::from_bytes(blake3::derive_key(
                                "mizar-driver/native-theorem-module-storage-side-table/v1", &bytes)))],
                        ..mizar_ir::storage::IrSideTables::default()
                    });
                    assert!(owner.validate_published_output(snapshot, sealed.any()).is_ok());
                    assert_eq!(owner.validate_current_output(snapshot, sealed.any()).is_ok(), !open);
                }
                assert_eq!((inputs.snapshot.clone(), inputs.build_plan.clone(), inputs.module_index.clone(),
                    loaded.clone(), captured_input.clone(), output.clone(), arena.clone(), collection.env().clone(), input.clone(), parent.clone()), before);
                assert_eq!(*owner.storage().get(&typed).unwrap(), output);
                assert_eq!(captured_input, &descriptor);
                if profile == 0 { assert_ne!(semantic, bytes); }
            }
        }
    }
}

#[test]
fn native_theorem_module_publication_preserves_refusals_and_publisher_boundaries() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (format!("{private}\ndefinition\nend;\n"), "mixed"),
        (private.replace("by PrivT1", "by Missing"), "unresolved"),
        (private.replace("thus X = X;", "thus Y = Y;"), "error"),
        ("theorem Broken: ;\n".to_owned(), "diagnostic"),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptor = SourceInput {
            package_id: fixture.version.package_id.clone(),
            module_path: fixture.version.module_path.clone(),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: fixture.version.edition.clone(),
            origin: SourceOriginInput::Disk {
                path: "src/main.miz".into(),
            },
        };
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    descriptor,
                )],
            )
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, 0, &unit);
        allow_frontend(&owner, &unit);
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("ModuleResolve"),
            OutputKind::new("SourceTheoremModuleCandidates"),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let submission = CompilerDriver::new(builder.build().unwrap())
            .with_output_publisher(owner.clone())
            .submit_captured(session, &ids, &snapshots, submit)
            .unwrap();
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let parents = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .filter(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
            .map(|parent| {
                SealedParentOutputHandle::from_published_output(&owner, snapshot, parent.clone())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let input = PhaseInput::new(
            unit,
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                parents,
            )
            .unwrap(),
        );
        let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let output = mizar_frontend::orchestration::Frontend::new(
            mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(
                fixture.root.join("alpha"),
            )),
            inputs.dependency_lexical_provider(&[]),
            MizarParserSeam,
        )
        .run_loaded(mizar_frontend::source::source_unit_from_loaded(
            loaded.clone(),
            mizar_frontend::source::diagnostic_file_path(descriptor),
        ))
        .unwrap();
        let old = inputs.native_theorem_module_candidates_with_frontend_parent(&input, &owner);
        if case == "diagnostic" {
            assert!(!output.diagnostics.is_empty());
            assert!(input.parent_outputs().is_empty());
            assert!(old.is_err());
            assert!(
                submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .any(|result| result.status == PhaseStatus::Recoverable)
            );
        } else {
            assert!(output.diagnostics.is_empty());
            let ast = output.ast.as_ref().unwrap();
            let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            let shells = DeclarationShellCollector::new(ast, &module).collect();
            let collection =
                SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main"))
                    .collect(&module);
            let arena =
                mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &module).unwrap();
            let lower = mizar_resolve::symbols::source_theorem_module_candidates(
                ast,
                &arena,
                collection.env(),
            );
            assert_eq!(old, lower);
            if case == "error" {
                assert!(old.is_err());
            } else {
                assert_eq!(old, Ok(None));
            }
        }
        let actual = inputs
            .publish_native_theorem_module_candidates(&input, &owner)
            .map(|output| output.map(|output| (*owner.storage().get(&output).unwrap()).clone()));
        assert_eq!(actual, old, "{case}");
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            collected.outputs_dropped,
            if case == "diagnostic" { 1 } else { 2 }
        );
        assert_eq!(collected.abandoned_slots_dropped, 0);
    }
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let foreign = Fixture::new(private.as_bytes());
    let (_, foreign_submission, foreign_adapter, foreign_handle, _) =
        native_disk_frontend_record_submission(&foreign, &ids, &snapshots, 0);
    for case in [
        "missing",
        "extra",
        "foreign",
        "key",
        "capture",
        "collected",
        "obsolete",
        "wrong-storage",
        "denied-kind",
        "denied-phase",
        "nonproducer",
    ] {
        let fixture = Fixture::new(private.as_bytes());
        let (_, submission, adapter, handle, frontend_input) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (mut inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let snapshot = source_input.snapshot;
        let ir_unit = handle.work_unit().clone();
        if !case.starts_with("denied-") {
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("ModuleResolve"),
                OutputKind::new("SourceTheoremModuleCandidates"),
                ir_unit.clone(),
            ));
        } else {
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new(if case == "denied-kind" {
                    "ModuleResolve"
                } else {
                    "Frontend"
                }),
                OutputKind::new(if case == "denied-kind" {
                    "WrongKind"
                } else {
                    "SourceTheoremModuleCandidates"
                }),
                ir_unit,
            ));
        }
        let mut parents = vec![
            SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                .unwrap(),
        ];
        let mut input_hash = source_input.identities().input_hash();
        let wrong_storage = PhaseOutputPublisher::new(
            std::sync::Arc::new(IrStorageService::new()),
            owner.registry().clone(),
        );
        let alternative;
        let mut selected_owner: &PhaseOutputPublisher = owner;
        match case {
            "missing" => parents.clear(),
            "extra" => parents.push(frontend_input.parent_outputs()[0].clone()),
            "foreign" => {
                parents = vec![
                    SealedParentOutputHandle::from_published_output(
                        foreign_adapter.publisher(),
                        foreign_submission.session.captured().snapshot().id,
                        foreign_handle.erase(),
                    )
                    .unwrap(),
                ]
            }
            "key" => input_hash = Hash::from_bytes([0x49; 32]),
            "capture" => inputs.captured = None,
            "collected" => {
                owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
            }
            "obsolete" => owner.mark_obsolete(snapshot).unwrap(),
            "wrong-storage" => selected_owner = &wrong_storage,
            "nonproducer" => {
                // A negative-only derivative of the real aggregate tests the inherited full comparison.
                alternative = PhaseOutputPublisher::new(
                    owner.storage().clone(),
                    std::sync::Arc::new(SnapshotHandleRegistry::new()),
                );
                alternative.register_current_snapshot(snapshot);
                let ancestor = frontend_input.parent_outputs()[0].as_output_ref().lineage();
                alternative
                    .registry()
                    .register_output(mizar_ir::identity::OutputIdentityInput {
                        snapshot: ancestor.snapshot,
                        phase: ancestor.phase.clone(),
                        work_unit: ancestor.work_unit.clone(),
                        output_kind: ancestor.output_kind.clone(),
                        content_hash: ancestor.content_hash,
                        side_table_hash: ancestor.side_table_hash,
                        parents: ancestor.parents.clone(),
                        named_input_hashes: ancestor.named_input_hashes.clone(),
                    })
                    .unwrap();
                alternative.allow_work_unit(AllowedWorkUnit::new(
                    handle.phase().clone(),
                    handle.output_kind().clone(),
                    handle.work_unit().clone(),
                ));
                selected_owner = &alternative;
                let mut actual = (*owner.storage().get(&handle).unwrap()).clone();
                actual.tokens.tokens[0].text = "negative-only-token".into();
                let raw = actual.canonical_disk_bytes().unwrap();
                let source_id = actual.source.source_id;
                let descriptor = inputs.captured.unwrap().native_sources().unwrap()[0]
                    .2
                    .clone();
                let negative = selected_owner
                    .publish(mizar_ir::publisher::PublishOutputInput {
                        slot: selected_owner.allocate(
                            snapshot,
                            handle.phase().clone(),
                            handle.work_unit().clone(),
                            handle.output_kind().clone(),
                            handle.schema_version(),
                        ),
                        snapshot,
                        phase: handle.phase().clone(),
                        work_unit: handle.work_unit().clone(),
                        output_kind: handle.output_kind().clone(),
                        schema_version: handle.schema_version(),
                        payload: actual,
                        canonical_payload: Some(raw.clone()),
                        storage_payload: Some(raw),
                        decode: mizar_ir::storage::BlobDecoder::new(move |bytes| {
                            FrontendOutput::from_canonical_storage_bytes(
                                bytes,
                                source_id,
                                &descriptor,
                            )
                            .ok_or_else(|| {
                                mizar_ir::storage::BlobDecodeError::new("negative aggregate")
                            })
                        }),
                        parents: vec![frontend_input.parent_outputs()[0].as_output_ref().clone()],
                        named_input_hashes: handle.lineage().named_input_hashes.clone(),
                        side_tables: (*owner.storage().side_tables(&handle).unwrap()).clone(),
                        origin: mizar_ir::publisher::OutputOrigin::PackageSource,
                        target: mizar_ir::publisher::PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                parents = vec![
                    SealedParentOutputHandle::from_published_output(
                        selected_owner,
                        snapshot,
                        negative.erase(),
                    )
                    .unwrap(),
                ];
            }
            _ => {}
        }
        let bundle_snapshot = if case == "foreign" {
            foreign_submission.session.captured().snapshot().id
        } else {
            snapshot
        };
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(bundle_snapshot, input_hash, vec![], parents).unwrap(),
        );
        let old =
            inputs.native_theorem_module_candidates_with_frontend_parent(&input, selected_owner);
        let error = inputs
            .publish_native_theorem_module_candidates(&input, selected_owner)
            .unwrap_err();
        if case.starts_with("denied-") {
            assert!(old.unwrap().is_some());
            assert!(error.contains("not allowed"), "{error}");
        } else {
            assert_eq!(error, old.unwrap_err(), "{case}");
            if case == "nonproducer" {
                assert_eq!(error, "native Frontend resolver complete output mismatch");
            }
        }
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            collected.outputs_dropped,
            match case {
                "collected" => 0,
                "nonproducer" => 3,
                _ => 2,
            }
        );
    }
}

#[test]
fn native_theorem_module_summary_candidate_preserves_captured_rows_and_identity() {
    use mizar_build::module_index::ModuleId as BuildModuleId;
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let statuses = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
    );
    let quantified = "for X being set holds X = X";
    for (text, public) in [
        (private.into(), vec![("UsePriv1", "theorem", quantified)]),
        (
            private.replace("theorem UsePriv1", "lemma UsePriv1"),
            vec![("UsePriv1", "lemma", quantified)],
        ),
        (
            private.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
            vec![],
        ),
        (
            statuses.to_owned(),
            vec![
                ("OpenT1", "open theorem", quantified),
                ("AxT1", "assumed theorem", quantified),
            ],
        ),
        (String::new(), vec![]),
        (
            format!("{private}\ntheorem Numeral: 0 = 0\nproof\n thus 0 = 0 by UsePriv1;\nend;\n"),
            vec![
                ("UsePriv1", "theorem", quantified),
                ("Numeral", "theorem", "0 = 0"),
            ],
        ),
        (
            format!(
                "\u{feff}{}",
                private
                    .replace("holds X = X", "holds X ::= 日本語 formula =:: = X")
                    .replace('\n', "\r\n")
            ),
            vec![("UsePriv1", "theorem", quantified)],
        ),
    ] {
        for open in [false, true] {
            let fixture = Fixture::new(text.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (mut draft, input) = fixture.submit_request(vec![], vec![]);
            draft.source_inputs.versions.clear();
            let descriptor = SourceInput {
                package_id: PackageId::new("alpha"),
                module_path: ModulePath::new("main"),
                normalized_path: fixture.version.normalized_path.clone(),
                edition: Edition::new("2025"),
                origin: if open {
                    SourceOriginInput::OpenBuffer {
                        uri: format!(
                            "file://{}",
                            fixture.root.join("alpha/src/main.miz").display()
                        ),
                        expected_version: 7,
                        actual_version: 7,
                        text: std::sync::Arc::from(text.as_str()),
                    }
                } else {
                    SourceOriginInput::Disk {
                        path: "src/main.miz".into(),
                    }
                },
            };
            if open {
                fixture.write(&[0xff]);
            }
            let session = draft
                .capture_sources(
                    &ids,
                    &snapshots,
                    vec![(
                        DiskSourceLoader::new(fixture.root.join("alpha")),
                        descriptor.clone(),
                    )],
                )
                .unwrap();
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            let submission = CompilerDriver::new(builder.build().unwrap())
                .submit_captured(session, &ids, &snapshots, input)
                .unwrap();
            let (inputs, _) = native_disk_lookup_context(&submission, &ids);
            let module = BuildModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            let (_, loaded, captured_input) =
                &inputs.captured.unwrap().native_sources().unwrap()[0];
            let output = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(source_unit_from_loaded(
                loaded.clone(),
                diagnostic_file_path(captured_input),
            ))
            .unwrap();
            assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
            assert!(output.cache_keys.ast.is_some());
            let ast = output.ast.as_ref().unwrap();
            let resolver_module =
                ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            let shells = DeclarationShellCollector::new(ast, &resolver_module).collect();
            let collection =
                SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main"))
                    .collect(&resolver_module);
            let identity = ModuleSummaryIdentity {
                package_id: "alpha".to_owned(),
                package_version: Some("0.1.0".to_owned()),
                lockfile_identity: None,
                module_path: "main".to_owned(),
                language_edition: "2025".to_owned(),
            };
            let expected = collection.source_theorem_module_summary_candidate(&output, &identity);
            let summary = expected.as_ref().unwrap().as_ref().unwrap();
            assert_eq!(summary.module, identity);
            assert_eq!(summary.source_hash, loaded.source_hash);
            assert_eq!(
                summary.source_hash,
                inputs.snapshot.source_versions[0].source_hash
            );
            assert_eq!(
                summary.source_hash,
                mizar_session::hash_text(&output.source.source_text)
            );
            assert_eq!(output.source.source_id, loaded.source_id);
            assert_eq!(
                output.source.source_text.as_ref(),
                text.trim_start_matches('\u{feff}').replace("\r\n", "\n")
            );
            assert_eq!(summary.exported_symbols.len(), public.len());
            assert_eq!(summary.exported_labels.len(), public.len());
            for (name, prefix, formula) in &public {
                let owner = collection
                    .env()
                    .symbols()
                    .iter()
                    .find(|owner| owner.primary_spelling() == *name)
                    .unwrap();
                let row = summary
                    .exported_symbols
                    .iter()
                    .find(|row| row.fully_qualified_name == owner.symbol().fqn().as_str())
                    .unwrap();
                assert_eq!(row.visibility, "public");
                assert_eq!(
                    row.declaration_kind,
                    prefix.split_whitespace().last().unwrap()
                );
                assert_eq!(
                    row.rendered_signature,
                    format!("{prefix} {name} : {formula}")
                );
                assert_eq!(row.proof_status, None);
                let label = summary
                    .exported_labels
                    .iter()
                    .find(|row| row.label == *name)
                    .unwrap();
                assert_eq!(label.owner_fully_qualified_name, row.fully_qualified_name);
            }
            assert!(summary.lexical_summary.contributions.is_empty());
            let before = (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                captured_input.clone(),
                output.clone(),
                collection.clone(),
            );
            for stage in 0..3 {
                if stage == 1 {
                    fixture.write(b"open theorem Changed: 0 = 1;\n");
                }
                if stage == 2 {
                    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                }
                assert_eq!(
                    inputs.native_theorem_module_summary_candidate(&module),
                    expected,
                    "open={open}, stage={stage}"
                );
            }
            assert_eq!(
                (
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inputs.module_index.clone(),
                    loaded.clone(),
                    captured_input.clone(),
                    output.clone(),
                    collection.clone()
                ),
                before
            );
            assert_eq!(captured_input, &descriptor);
        }
    }
}

#[test]
fn native_theorem_module_summary_candidate_preserves_lower_results_and_binding_refusals() {
    use mizar_build::{module_index::ModuleId as BuildModuleId, planner::PackagePlanSource};
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_resolve::resolved_ast::{
        LabelResolution, SurfaceResolvedArena, source_theorem_resolved_ast,
    };
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (case, text) in [
        ("unsupported", format!("{private}\nreserve Y for set;\n")),
        ("citation", private.replace("by PrivT1", "by Missing")),
        ("diagnostic", "theorem Broken: 0 = ;\n".to_owned()),
        ("variable", private.replace("thus X = X;", "thus Y = Y;")),
        ("import", format!("import std.core;\n{private}")),
        (
            "private-lexical",
            format!(
                "{private}\ndefinition\n let X, Y be set;\n private pred Hidden: X secret Y means X = Y;\nend;\n"
            ),
        ),
        (
            "duplicate",
            include_str!(
                "../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz"
            )
            .to_owned(),
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let foreign = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let mut submissions = Vec::new();
        for fixture in [&fixture, &foreign] {
            let (mut draft, input) = fixture.submit_request(vec![], vec![]);
            draft.source_inputs.versions.clear();
            let descriptor = SourceInput {
                package_id: PackageId::new("alpha"),
                module_path: ModulePath::new("main"),
                normalized_path: fixture.version.normalized_path.clone(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk {
                    path: "src/main.miz".into(),
                },
            };
            let session = draft
                .capture_sources(
                    &ids,
                    &snapshots,
                    vec![(
                        DiskSourceLoader::new(fixture.root.join("alpha")),
                        descriptor,
                    )],
                )
                .unwrap();
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            submissions.push(
                CompilerDriver::new(builder.build().unwrap())
                    .submit_captured(session, &ids, &snapshots, input)
                    .unwrap(),
            );
        }
        let (inputs, _) = native_disk_lookup_context(&submissions[0], &ids);
        let (foreign_inputs, _) = native_disk_lookup_context(&submissions[1], &ids);
        let module = BuildModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
        let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let output = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
            inputs.dependency_lexical_provider(&[]),
            MizarParserSeam,
        )
        .run_loaded(source_unit_from_loaded(
            loaded.clone(),
            diagnostic_file_path(descriptor),
        ))
        .unwrap();
        let diagnostic = ["diagnostic", "import"].contains(&case);
        assert_eq!(!output.diagnostics.is_empty(), diagnostic);
        let ast = output.ast.as_ref().unwrap();
        let resolver_module =
            ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
        let shells = DeclarationShellCollector::new(ast, &resolver_module).collect();
        let collection =
            SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main"))
                .collect(&resolver_module);
        let identity = ModuleSummaryIdentity {
            package_id: "alpha".to_owned(),
            package_version: Some("0.1.0".to_owned()),
            lockfile_identity: None,
            module_path: "main".to_owned(),
            language_edition: "2025".to_owned(),
        };
        let lower = collection.source_theorem_module_summary_candidate(&output, &identity);
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            descriptor.clone(),
            output.clone(),
            collection.clone(),
        );
        match case {
            "variable" => {
                let arena = SurfaceResolvedArena::lower(ast, &resolver_module).unwrap();
                let error = source_theorem_resolved_ast(ast, &arena, collection.env()).unwrap_err();
                assert_eq!(lower, Err(error));
            }
            "citation" => {
                let arena = SurfaceResolvedArena::lower(ast, &resolver_module).unwrap();
                let semantic = source_theorem_resolved_ast(ast, &arena, collection.env())
                    .unwrap()
                    .unwrap();
                assert!(
                    semantic
                        .label_refs()
                        .iter()
                        .any(|(_, row)| matches!(row.resolution(), LabelResolution::Unresolved(_)))
                );
                assert_eq!(
                    mizar_resolve::symbols::source_theorem_module_candidates(
                        ast,
                        &arena,
                        collection.env()
                    ),
                    Ok(None)
                );
                assert!(
                    lower
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .exported_symbols
                        .iter()
                        .all(|row| row.proof_status.is_none())
                );
            }
            "private-lexical" => {
                assert!(!output.tokens.local_declarations().user_symbols.is_empty());
                assert_eq!(
                    collection.export_frontend_lexical_contributions(&output, &identity),
                    Some(Vec::new())
                );
                assert_eq!(lower, Ok(None));
            }
            "duplicate" => {
                assert!(!collection.diagnostics().is_empty());
                assert_eq!(lower, Ok(None));
            }
            _ => assert_eq!(lower, Ok(None)),
        }
        assert_eq!(
            inputs.native_theorem_module_summary_candidate(&module),
            lower,
            "{case}"
        );
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                descriptor.clone(),
                output.clone(),
                collection.clone()
            ),
            before
        );
        if case != "citation" {
            continue;
        }
        for requested in [
            BuildModuleId::new(PackageId::new("alpha"), ModulePath::new("missing")),
            BuildModuleId::new(PackageId::new("foreign"), ModulePath::new("main")),
        ] {
            assert_eq!(
                inputs
                    .native_theorem_module_summary_candidate(&requested)
                    .unwrap_err(),
                "native theorem summary requires one captured source version"
            );
        }
        let mut ambiguous = inputs.snapshot.clone();
        ambiguous
            .source_versions
            .push(ambiguous.source_versions[0].clone());
        let mut absent = inputs.snapshot.clone();
        absent.source_versions.clear();
        for snapshot in [&ambiguous, &absent] {
            assert_eq!(
                SourceLoadInputs {
                    snapshot,
                    captured: None,
                    ..inputs
                }
                .native_theorem_module_summary_candidate(&module)
                .unwrap_err(),
                "native theorem summary requires one captured source version"
            );
        }
        let mut ambiguous_plan = inputs.build_plan.clone();
        ambiguous_plan
            .packages
            .push(ambiguous_plan.packages[0].clone());
        let mut wrong_root = inputs.build_plan.clone();
        let PackagePlanSource::Workspace { root, .. } = &mut wrong_root.packages[0].source else {
            panic!("workspace")
        };
        *root = "foreign".to_owned();
        let mut wrong_edition = inputs.build_plan.clone();
        wrong_edition.packages[0].edition = Edition::new("1900");
        let mut foreign_index = foreign_inputs.module_index.clone();
        foreign_index.modules[0].module.package = PackageId::new("foreign");
        let mut ambiguous_index = inputs.module_index.clone();
        ambiguous_index
            .modules
            .push(ambiguous_index.modules[0].clone());
        for invalid in [
            SourceLoadInputs {
                captured: None,
                ..inputs
            },
            SourceLoadInputs {
                captured: foreign_inputs.captured,
                ..inputs
            },
            SourceLoadInputs {
                build_plan: foreign_inputs.build_plan,
                ..inputs
            },
            SourceLoadInputs {
                build_plan: &ambiguous_plan,
                ..inputs
            },
            SourceLoadInputs {
                build_plan: &wrong_root,
                ..inputs
            },
            SourceLoadInputs {
                build_plan: &wrong_edition,
                ..inputs
            },
            SourceLoadInputs {
                module_index: &foreign_index,
                ..inputs
            },
            SourceLoadInputs {
                module_index: &ambiguous_index,
                ..inputs
            },
        ] {
            assert_eq!(
                invalid
                    .native_theorem_module_summary_candidate(&module)
                    .unwrap_err(),
                "native theorem summary capture binding failed"
            );
        }
    }
}
