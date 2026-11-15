use super::*;

impl<'a> crate::registry::SourceLoadInputs<'a> {
    /// Returns unpublished theorem module metadata from protected native capture.
    pub fn native_theorem_module_candidates(
        self,
        module: &mizar_build::module_index::ModuleId,
    ) -> Result<
        Option<(
            mizar_resolve::resolved_ast::ResolvedAst,
            mizar_resolve::env::SymbolEnv,
        )>,
        String,
    > {
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem module requires one captured source version".to_owned())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem module capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem module requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem module requires a workspace package".to_owned());
        };
        let output = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(
                PathBuf::from(self.snapshot.workspace_root.as_str()).join(root),
            )),
            self.dependency_lexical_provider(&[]),
            MizarParserSeam,
        )
        .run_loaded(source)
        .map_err(|error| error.to_string())?;
        if !output.diagnostics.is_empty() || output.cache_keys.ast.is_none() {
            return Ok(None);
        }
        let Some(ast) = output.ast.as_ref() else {
            return Ok(None);
        };
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let Some((arena, env)) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env)
    }

    /// Associates a published Frontend parent with a fresh protected-source execution.
    pub fn native_theorem_module_candidates_with_frontend_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<
        Option<(
            mizar_resolve::resolved_ast::ResolvedAst,
            mizar_resolve::env::SymbolEnv,
        )>,
        String,
    > {
        if input.snapshot != self.snapshot.id {
            return Err("native Frontend resolver snapshot mismatch".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native Frontend resolver requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native Frontend resolver requires one captured source".to_owned())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native Frontend resolver capture binding failed".to_owned())?;
        if input.identities().input_hash() != source_input_hash(version)
            || !input.identities().dependency_hashes().is_empty()
            || !self.snapshot.dependency_artifacts.is_empty()
            || !self.module_index.dependency_summaries.is_empty()
            || input.parent_outputs().len() != 1
        {
            return Err("native Frontend resolver requires the leaf input profile".to_owned());
        }
        let parent = input.parent_outputs()[0].as_output_ref();
        let storage = publisher.storage();
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_output(input.snapshot, parent, OutputAuthorization::Published)
            .map_err(|error| error.to_string())?;
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        if parent.snapshot() != input.snapshot
            || parent.phase() != &IrPipelinePhase::new("Frontend")
            || parent.work_unit() != &work_unit
            || parent.output_kind() != &OutputKind::new("FrontendOutput")
            || parent.schema_version() != SchemaVersion::new(1)
        {
            return Err("native Frontend resolver parent binding mismatch".to_owned());
        }
        let typed = storage
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                parent,
                &OutputKind::new("FrontendOutput"),
            )
            .map_err(|error| error.to_string())?;
        let [ancestor] = typed.lineage().parents.as_slice() else {
            return Err("native Frontend resolver requires one SourceLoad ancestor".to_owned());
        };
        let ancestor = publisher
            .registry()
            .output_lineage(*ancestor)
            .ok_or_else(|| "native Frontend resolver requires registered ancestry".to_owned())?;
        if ancestor.snapshot != input.snapshot
            || ancestor.phase != IrPipelinePhase::new("SourceLoad")
            || ancestor.work_unit != work_unit
            || ancestor.output_kind != OutputKind::new("SourceUnit")
            || !ancestor.parents.is_empty()
            || ancestor.named_input_hashes
                != vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: input.identities().input_hash(),
                }]
        {
            return Err("native Frontend resolver SourceLoad ancestry mismatch".to_owned());
        }
        let retained = storage.get(&typed).map_err(|error| error.to_string())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native Frontend resolver requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native Frontend resolver requires a workspace package".to_owned());
        };
        let output = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(
                PathBuf::from(self.snapshot.workspace_root.as_str()).join(root),
            )),
            self.dependency_lexical_provider(&[]),
            MizarParserSeam,
        )
        .run_loaded(source)
        .map_err(|error| error.to_string())?;
        if *retained != output {
            return Err("native Frontend resolver complete output mismatch".to_owned());
        }
        if !output.diagnostics.is_empty() || output.cache_keys.ast.is_none() {
            return Ok(None);
        }
        let Some(ast) = output.ast.as_ref() else {
            return Ok(None);
        };
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let Some((arena, env)) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        let result = mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env)?;
        if result.is_some() {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
                .map_err(|error| error.to_string())?;
        }
        Ok(result)
    }

    /// Seals a genuine native theorem summary bound to its resolved parent.
    pub fn publish_native_theorem_module_summary(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<
        Option<mizar_ir::storage::PhaseOutputRef<mizar_artifact::module_summary::ModuleSummary>>,
        String,
    > {
        use mizar_artifact::{
            module_summary::{ModuleSummaryReadOptions, read_module_summary, write_module_summary},
            store::parse_canonical_json_bytes,
        };

        const STORAGE_CONTEXT: &str = "native theorem module summary storage";
        if input.snapshot != self.snapshot.id {
            return Err("native theorem summary publication snapshot mismatch".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem summary publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem summary publication requires one captured source".to_owned()
        })?;
        let source_key = source_input_hash(version);
        if input.identities().input_hash() != source_key
            || !input.identities().dependency_hashes().is_empty()
            || !self.snapshot.dependency_artifacts.is_empty()
            || !self.module_index.dependency_summaries.is_empty()
            || input.parent_outputs().len() != 1
        {
            return Err(
                "native theorem summary publication requires the leaf input profile".to_owned(),
            );
        }
        let parent = input.parent_outputs()[0].as_output_ref();
        let storage = publisher.storage();
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_output(input.snapshot, parent, OutputAuthorization::Published)
            .map_err(|error| error.to_string())?;
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let named_input_hashes = vec![NamedInputHash {
            name: "source".to_owned(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
            digest: source_key,
        }];
        if parent.snapshot() != input.snapshot
            || parent.phase() != &IrPipelinePhase::new("ModuleResolve")
            || parent.work_unit() != &work_unit
            || parent.output_kind() != &OutputKind::new("SourceTheoremModuleCandidates")
            || parent.schema_version() != SchemaVersion::new(1)
            || parent.lineage().named_input_hashes != named_input_hashes
        {
            return Err("native theorem summary publication parent binding mismatch".to_owned());
        }
        let [ancestor] = parent.lineage().parents.as_slice() else {
            return Err(
                "native theorem summary publication requires one Frontend ancestor".to_owned(),
            );
        };
        let ancestor = publisher
            .registry()
            .output_lineage(*ancestor)
            .ok_or_else(|| {
                "native theorem summary publication requires registered ancestry".to_owned()
            })?;
        if ancestor.snapshot != input.snapshot
            || ancestor.phase != IrPipelinePhase::new("Frontend")
            || ancestor.work_unit != work_unit
            || ancestor.output_kind != OutputKind::new("FrontendOutput")
        {
            return Err("native theorem summary publication Frontend ancestry mismatch".to_owned());
        }
        let typed = storage
            .typed_handle::<(
                mizar_resolve::resolved_ast::ResolvedAst,
                mizar_resolve::env::SymbolEnv,
            )>(parent, &OutputKind::new("SourceTheoremModuleCandidates"))
            .map_err(|error| error.to_string())?;
        let retained = storage.get(&typed).map_err(|error| error.to_string())?;
        let Some(pair) = self.native_theorem_module_candidates(module)? else {
            return Ok(None);
        };
        if *retained != pair {
            return Err("native theorem summary publication resolved pair mismatch".to_owned());
        }
        let Some(summary) = self.native_theorem_module_summary_candidate(module)? else {
            return Ok(None);
        };
        let bytes = write_module_summary(&summary).map_err(|error| error.to_string())?;
        let value = parse_canonical_json_bytes(&bytes, STORAGE_CONTEXT)
            .map_err(|error| error.to_string())?;
        let decoded = read_module_summary(
            &value,
            ModuleSummaryReadOptions {
                artifact_path: Some(STORAGE_CONTEXT),
                expected_module: Some(&summary.module),
                expected_interface_hash: Some(summary.interface_hash),
            },
        )
        .map_err(|error| error.to_string())?;
        if decoded != summary {
            return Err("native theorem summary publication storage mismatch".to_owned());
        }
        let identity = summary.module.clone();
        let interface_hash = summary.interface_hash;
        let phase = IrPipelinePhase::new("SignatureCollection");
        let output_kind = OutputKind::new("SourceTheoremModuleSummary");
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_output(input.snapshot, parent, OutputAuthorization::Published)
            .map_err(|error| error.to_string())?;
        let (origin, target) = source_publication_route(&version.origin);
        publish_native_output(
            publisher,
            input.snapshot,
            (phase, work_unit, output_kind),
            |slot, phase, work_unit, output_kind| PublishOutputInput {
                slot,
                snapshot: input.snapshot,
                phase,
                work_unit,
                output_kind,
                schema_version: SchemaVersion::new(1),
                payload: summary,
                canonical_payload: Some(bytes.clone()),
                storage_payload: Some(bytes),
                decode: BlobDecoder::new(move |bytes| {
                    let value = parse_canonical_json_bytes(bytes, STORAGE_CONTEXT)
                        .map_err(|error| BlobDecodeError::new(error.to_string()))?;
                    read_module_summary(
                        &value,
                        ModuleSummaryReadOptions {
                            artifact_path: Some(STORAGE_CONTEXT),
                            expected_module: Some(&identity),
                            expected_interface_hash: Some(interface_hash),
                        },
                    )
                    .map_err(|error| BlobDecodeError::new(error.to_string()))
                }),
                parents: vec![parent.clone()],
                named_input_hashes,
                side_tables: IrSideTables::default(),
                origin,
                target,
            },
        )
        .map(Some)
        .map_err(|error| error.to_string())
    }

    /// Seals the coherent bounded native theorem pair and its genuine streams.
    pub fn publish_native_theorem_module_candidates(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<
        Option<
            mizar_ir::storage::PhaseOutputRef<(
                mizar_resolve::resolved_ast::ResolvedAst,
                mizar_resolve::env::SymbolEnv,
            )>,
        >,
        String,
    > {
        let Some(pair) =
            self.native_theorem_module_candidates_with_frontend_parent(input, publisher)?
        else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem publication requires one captured source".to_owned())?;
        let parent = input.parent_outputs()[0].as_output_ref();
        let storage = publisher.storage();
        let typed = storage
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                parent,
                &OutputKind::new("FrontendOutput"),
            )
            .map_err(|error| error.to_string())?;
        let retained = storage.get(&typed).map_err(|error| error.to_string())?;
        let ast = retained
            .ast
            .as_ref()
            .ok_or_else(|| "native theorem publication requires retained syntax".to_owned())?;
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let Some((resolved, env, semantic_bytes)) =
            mizar_resolve::symbols::source_theorem_module_semantic_payload(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        if (resolved, env) != pair {
            return Err("native theorem publication semantic pair mismatch".to_owned());
        }
        let Some(storage_bytes) =
            mizar_resolve::symbols::source_theorem_module_candidates_canonical_bytes(
                ast,
                &resolver_module,
            )?
        else {
            return Ok(None);
        };
        if mizar_resolve::symbols::source_theorem_module_candidates_from_canonical_bytes(
            &storage_bytes,
            version.source_id,
            &resolver_module,
        )
        .as_ref()
            != Some(&pair)
        {
            return Err("native theorem publication storage pair mismatch".to_owned());
        }
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-module-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("ModuleResolve");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremModuleCandidates");
        let source_id = version.source_id;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_output(input.snapshot, parent, OutputAuthorization::Published)
            .map_err(|error| error.to_string())?;
        let (origin, target) = source_publication_route(&version.origin);
        publish_native_output(
            publisher,
            input.snapshot,
            (phase, work_unit, output_kind),
            |slot, phase, work_unit, output_kind| PublishOutputInput {
                slot,
                snapshot: input.snapshot,
                phase,
                work_unit,
                output_kind,
                schema_version: SchemaVersion::new(1),
                payload: pair,
                canonical_payload: Some(semantic_bytes),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    mizar_resolve::symbols::source_theorem_module_candidates_from_canonical_bytes(
                        bytes,
                        source_id,
                        &resolver_module,
                    )
                    .ok_or_else(|| BlobDecodeError::new("invalid native theorem module storage"))
                }),
                parents: vec![parent.clone()],
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
                side_tables: IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "resolve-theorem-module-storage",
                        version.normalized_path.as_str(),
                        storage_hash,
                    )],
                    ..IrSideTables::default()
                },
                origin,
                target,
            },
        )
        .map(Some)
        .map_err(|error| error.to_string())
    }
    /// Returns an unpublished source theorem summary from protected native capture.
    pub fn native_theorem_module_summary_candidate(
        self,
        module: &mizar_build::module_index::ModuleId,
    ) -> Result<Option<mizar_artifact::module_summary::ModuleSummary>, String> {
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem summary requires one captured source version".to_owned())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem summary capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem summary requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem summary requires a workspace package".to_owned());
        };
        let output = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(
                PathBuf::from(self.snapshot.workspace_root.as_str()).join(root),
            )),
            self.dependency_lexical_provider(&[]),
            MizarParserSeam,
        )
        .run_loaded(source)
        .map_err(|error| error.to_string())?;
        if !output.diagnostics.is_empty() || output.cache_keys.ast.is_none() {
            return Ok(None);
        }
        let Some(ast) = output.ast.as_ref() else {
            return Ok(None);
        };
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let shells =
            mizar_resolve::declarations::DeclarationShellCollector::new(ast, &resolver_module)
                .collect();
        let collection = mizar_resolve::symbols::SignatureProjectionExtractor::new(
            ast,
            &shells,
            mizar_resolve::env::NamespacePath::new(module.path.as_str()),
        )
        .collect(&resolver_module);
        let identity = mizar_artifact::module_summary::ModuleSummaryIdentity {
            package_id: module.package.as_str().to_owned(),
            package_version: Some(package.version.to_string()),
            lockfile_identity: None,
            module_path: module.path.as_str().to_owned(),
            language_edition: version.edition.as_str().to_owned(),
        };
        collection.source_theorem_module_summary_candidate(&output, &identity)
    }
}
pub(crate) struct SourceTheoremModuleResolverService;

impl PhaseService for SourceTheoremModuleResolverService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremModuleResolver",
            PhaseOwner::MizarResolve,
            vec![PipelinePhase::ModuleResolve],
            "mizar-resolve/source-theorem-module-publication/v1",
            "SourceTheoremModuleCandidates",
        )
        .expect("source theorem module resolver descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem resolution has no complete dependency footprint".to_owned(),
        }
    }

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext<'_>) -> PhaseResult {
        if context.common.snapshot != input.snapshot || context.common.work_unit != input.work_unit
        {
            return blocking();
        }
        if let Some(token) = context.cancellation {
            return if token.snapshot == input.snapshot {
                PhaseResult {
                    status: PhaseStatus::Cancelled,
                    ..blocking()
                }
            } else {
                blocking()
            };
        }
        let Some(publisher) = context.output_publisher else {
            return blocking();
        };
        let Some(inputs) = context.source_load else {
            return blocking();
        };
        if inputs.snapshot.id != input.snapshot
            || publisher.validate_current_snapshot(input.snapshot).is_err()
        {
            return blocking();
        }
        match inputs.publish_native_theorem_module_candidates(&input, &publisher) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}
