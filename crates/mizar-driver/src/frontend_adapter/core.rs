use super::*;

impl<'a> crate::registry::SourceLoadInputs<'a> {
    /// Lowers genuine theorem Core after whole metadata replay against captured source.
    pub fn native_theorem_core_with_metadata_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<Option<mizar_core::core_ir::CoreIr>, String> {
        if input.snapshot != self.snapshot.id {
            return Err("native theorem metadata Core snapshot mismatch".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem metadata Core requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem metadata Core requires one captured source".to_owned())?;
        let source_key = source_input_hash(version);
        if input.identities().input_hash() != source_key
            || !input.identities().dependency_hashes().is_empty()
            || !self.snapshot.dependency_artifacts.is_empty()
            || !self.module_index.dependency_summaries.is_empty()
            || input.parent_outputs().len() != 1
        {
            return Err("native theorem metadata Core requires the leaf input profile".to_owned());
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
            || parent.phase() != &IrPipelinePhase::new("OverloadResolution")
            || parent.work_unit() != &work_unit
            || parent.output_kind() != &OutputKind::new("SourceTheoremMetadata")
            || parent.schema_version() != SchemaVersion::new(1)
            || parent.lineage().named_input_hashes != named_input_hashes
        {
            return Err("native theorem metadata Core parent binding mismatch".to_owned());
        }
        if parent.lineage().parents.len() != 2 {
            return Err("native theorem metadata Core requires two ancestors".to_owned());
        }
        let ancestors = parent
            .lineage()
            .parents
            .iter()
            .map(|id| publisher.registry().output_lineage(*id))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                "native theorem metadata Core requires registered ancestry".to_owned()
            })?;
        let typed = unique(ancestors.iter().filter(|ancestor| {
            ancestor.phase == IrPipelinePhase::new("TypeChecking")
                && ancestor.output_kind == OutputKind::new("SourceTheoremTypedAst")
        }))
        .ok_or_else(|| "native theorem metadata Core requires one Type ancestor".to_owned())?;
        let registration = unique(ancestors.iter().filter(|ancestor| {
            ancestor.phase == IrPipelinePhase::new("RegistrationResolution")
                && ancestor.output_kind == OutputKind::new("SourceTheoremRegistrationClosure")
        }))
        .ok_or_else(|| {
            "native theorem metadata Core requires one Registration ancestor".to_owned()
        })?;
        if ancestors.iter().any(|ancestor| {
            ancestor.snapshot != input.snapshot
                || ancestor.work_unit != work_unit
                || ancestor.named_input_hashes != named_input_hashes
        }) || registration.parents != vec![typed.output]
        {
            return Err("native theorem metadata Core ancestry mismatch".to_owned());
        }
        let typed = storage
            .typed_handle::<(
                mizar_checker::binding_env::BindingEnv,
                mizar_checker::type_checker::TermFormulaInferenceOutput,
                mizar_checker::typed_ast::TypedAst,
                mizar_checker::resolved_typed_ast::ResolvedTypedAst,
            )>(parent, &OutputKind::new("SourceTheoremMetadata"))
            .map_err(|error| error.to_string())?;
        let retained = storage.get(&typed).map_err(|error| error.to_string())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem metadata Core capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem metadata Core requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem metadata Core requires a workspace package".to_owned());
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
        if native_theorem_metadata_storage_bytes(ast, &resolver_module, &retained)?.is_none() {
            return Ok(None);
        }
        let Some((arena, env)) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(&arena, &env)?;
        let check =
            mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena,
                &retained.2,
                &scope,
                &env,
                &labels,
                &resolved,
                None,
            )?;
        let core = mizar_core::elaborator::lower_source_theorem_skeletons(&check, None)?;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_output(input.snapshot, parent, OutputAuthorization::Published)
            .map_err(|error| error.to_string())?;
        Ok(Some(core))
    }

    /// Seals genuine bounded Core after authenticating its Metadata parent.
    pub fn publish_native_theorem_core_with_metadata_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<Option<mizar_ir::storage::PhaseOutputRef<mizar_core::core_ir::CoreIr>>, String>
    {
        let Some(core) = self.native_theorem_core_with_metadata_parent(input, publisher)? else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem core publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem core publication requires one captured source".to_owned())?;
        let parent = input.parent_outputs()[0].as_output_ref();
        let storage = publisher.storage();
        let (source, _) = self.native_source(version).ok_or_else(|| {
            "native theorem metadata Core publication capture binding failed".to_owned()
        })?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| {
                "native theorem metadata Core publication requires a bound package".to_owned()
            })?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(
                "native theorem metadata Core publication requires a workspace package".to_owned(),
            );
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
        let Some(semantic_bytes) =
            native_theorem_core_semantic_payload(ast, &resolver_module, &core)?
        else {
            return Ok(None);
        };
        let Some(storage_bytes) = native_theorem_core_storage_bytes(ast, &resolver_module, &core)?
        else {
            return Ok(None);
        };
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-core-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("Elaboration");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremCore");
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
                payload: core,
                canonical_payload: Some(semantic_bytes),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_theorem_core_from_storage_bytes(bytes, source_id, &resolver_module)
                        .map_err(BlobDecodeError::new)?
                        .ok_or_else(|| BlobDecodeError::new("invalid native theorem core storage"))
                }),
                parents: vec![parent.clone()],
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
                side_tables: IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "elaborate-theorem-core-storage",
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

    /// Derives theorem-only flow and seed handoff from an authenticated Core parent.
    pub fn native_theorem_preparation_with_core_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<
        Option<(
            mizar_core::control_flow::ControlFlowOutput,
            mizar_core::control_flow::ObligationSeedHandoff,
        )>,
        String,
    > {
        if input.snapshot != self.snapshot.id {
            return Err("native theorem preparation snapshot mismatch".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem preparation requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem preparation requires one captured source".to_owned())?;
        let source_key = source_input_hash(version);
        if input.identities().input_hash() != source_key
            || !input.identities().dependency_hashes().is_empty()
            || !self.snapshot.dependency_artifacts.is_empty()
            || !self.module_index.dependency_summaries.is_empty()
            || input.parent_outputs().len() != 1
        {
            return Err("native theorem preparation requires the leaf input profile".to_owned());
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
            || parent.phase() != &IrPipelinePhase::new("Elaboration")
            || parent.work_unit() != &work_unit
            || parent.output_kind() != &OutputKind::new("SourceTheoremCore")
            || parent.schema_version() != SchemaVersion::new(1)
            || parent.lineage().named_input_hashes != named_input_hashes
        {
            return Err("native theorem preparation parent binding mismatch".to_owned());
        }
        let [ancestor] = parent.lineage().parents.as_slice() else {
            return Err("native theorem preparation requires one Metadata ancestor".to_owned());
        };
        let ancestor = publisher
            .registry()
            .output_lineage(*ancestor)
            .ok_or_else(|| "native theorem preparation requires registered ancestry".to_owned())?;
        if ancestor.phase != IrPipelinePhase::new("OverloadResolution")
            || ancestor.output_kind != OutputKind::new("SourceTheoremMetadata")
            || ancestor.snapshot != input.snapshot
            || ancestor.work_unit != work_unit
            || ancestor.named_input_hashes != named_input_hashes
        {
            return Err("native theorem preparation ancestry mismatch".to_owned());
        }
        let typed = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                parent,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let retained = storage.get(&typed).map_err(|error| error.to_string())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem preparation capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem preparation requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem preparation requires a workspace package".to_owned());
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
        if native_theorem_core_storage_bytes(ast, &resolver_module, &retained)?.is_none() {
            return Ok(None);
        }
        if !retained.algorithms().is_empty() {
            return Ok(None);
        }
        let flow = mizar_core::control_flow::build_control_flow_ir(&retained);
        let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&retained, &flow);
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_output(input.snapshot, parent, OutputAuthorization::Published)
            .map_err(|error| error.to_string())?;
        Ok(Some((flow, handoff)))
    }

    /// Seals genuine theorem preparation with coherent semantic and storage streams.
    pub fn publish_native_theorem_preparation(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<
        Option<
            mizar_ir::storage::PhaseOutputRef<(
                mizar_core::control_flow::ControlFlowOutput,
                mizar_core::control_flow::ObligationSeedHandoff,
            )>,
        >,
        String,
    > {
        let Some(preparation) =
            self.native_theorem_preparation_with_core_parent(input, publisher)?
        else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem preparation publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem preparation publication requires one captured source".to_owned()
        })?;
        let parent = input.parent_outputs()[0].as_output_ref();
        let storage = publisher.storage();
        let typed = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                parent,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&typed).map_err(|error| error.to_string())?;
        let (source, _) = self.native_source(version).ok_or_else(|| {
            "native theorem preparation publication capture binding failed".to_owned()
        })?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| {
                "native theorem preparation publication requires a bound package".to_owned()
            })?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(
                "native theorem preparation publication requires a workspace package".to_owned(),
            );
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
        let Some(core_semantics) =
            native_theorem_core_semantic_payload(ast, &resolver_module, &core)?
        else {
            return Ok(None);
        };
        let Some(storage_bytes) =
            native_theorem_preparation_storage_bytes(ast, &resolver_module, &core, &preparation)?
        else {
            return Ok(None);
        };
        let domain = b"mizar-driver/native-theorem-preparation-content/v1\0";
        let Some(size) = u64::try_from(core_semantics.len())
            .ok()
            .and_then(|_| domain.len().checked_add(8))
            .and_then(|size| size.checked_add(core_semantics.len()))
            .filter(|size| *size <= 32 * 1024 * 1024)
        else {
            return Ok(None);
        };
        let mut semantic_bytes = Vec::with_capacity(size);
        semantic_bytes.extend_from_slice(domain);
        semantic_bytes.extend_from_slice(&(core_semantics.len() as u64).to_le_bytes());
        semantic_bytes.extend_from_slice(&core_semantics);
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-preparation-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("AlgorithmPreparation");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremPreparation");
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
                payload: preparation,
                canonical_payload: Some(semantic_bytes),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_theorem_preparation_from_storage_bytes(
                        bytes,
                        source_id,
                        &resolver_module,
                    )
                    .map_err(BlobDecodeError::new)?
                    .ok_or_else(|| {
                        BlobDecodeError::new("invalid native theorem preparation storage")
                    })
                }),
                parents: vec![parent.clone()],
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
                side_tables: IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "prepare-theorem-storage",
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
}
pub(crate) struct SourceTheoremElaborationService;

impl PhaseService for SourceTheoremElaborationService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremElaboration",
            PhaseOwner::MizarCore,
            vec![PipelinePhase::Elaboration],
            "mizar-core/source-theorem-core-publication/v1",
            "SourceTheoremCore",
        )
        .expect("source theorem elaboration descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem elaboration has no complete dependency footprint".to_owned(),
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
        match inputs.publish_native_theorem_core_with_metadata_parent(&input, &publisher) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}

pub(crate) struct SourceTheoremAlgorithmPreparationService;

impl PhaseService for SourceTheoremAlgorithmPreparationService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremAlgorithmPreparation",
            PhaseOwner::MizarCore,
            vec![PipelinePhase::AlgorithmPreparation],
            "mizar-core/source-theorem-preparation-publication/v1",
            "SourceTheoremPreparation",
        )
        .expect("source theorem algorithm preparation descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem algorithm preparation has no complete dependency footprint"
                .to_owned(),
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
        match inputs.publish_native_theorem_preparation(&input, &publisher) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}

/// Associates retained source syntax with genuine Core semantic content.
pub fn native_theorem_core_semantic_payload(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    core: &mizar_core::core_ir::CoreIr,
) -> Result<Option<Vec<u8>>, String> {
    let Some((arena, env)) =
        mizar_resolve::symbols::source_theorem_module_original_inputs(ast, module)?
    else {
        return Ok(None);
    };
    if mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env)?.is_none() {
        return Ok(None);
    }
    let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(
        mizar_resolve::names::SourceVariableScopeInput::new(ast, module, &env),
    )
    .map_err(|error| format!("{error:?}"))?;
    let metadata =
        mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)?;
    let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(&arena, &env)?;
    let check =
        mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
            &arena,
            &metadata.2,
            &scope,
            &env,
            &labels,
            &resolved,
            None,
        )?;
    mizar_core::elaborator::source_theorem_core_semantic_payload(&check, core).map(Some)
}

/// Stores lossless source syntax only when genuine replay reproduces the supplied Core.
pub fn native_theorem_core_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    core: &mizar_core::core_ir::CoreIr,
) -> Result<Option<Vec<u8>>, String> {
    let Some(syntax) = ast.canonical_bytes() else {
        return Ok(None);
    };
    let domain = b"mizar-driver/native-theorem-core-storage/v1\0";
    let fields = [
        module.package().as_str().as_bytes(),
        module.path().as_str().as_bytes(),
        syntax.as_slice(),
    ];
    let Some(bytes) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
        return Ok(None);
    };
    let Some(replayed) = native_theorem_core_from_storage_bytes(&bytes, ast.source_id, module)?
    else {
        return Ok(None);
    };
    Ok((replayed == *core).then_some(bytes))
}

/// Rebinds stored syntax to an existing source id and replays the genuine lower owners.
pub fn native_theorem_core_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
) -> Result<Option<mizar_core::core_ir::CoreIr>, String> {
    let domain = b"mizar-driver/native-theorem-core-storage/v1\0";
    let Some(fields) = framing::decode::<3>(bytes, domain, 32 * 1024 * 1024) else {
        return Ok(None);
    };
    if fields[0] != module.package().as_str().as_bytes()
        || fields[1] != module.path().as_str().as_bytes()
    {
        return Ok(None);
    }
    let Some(ast) =
        <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(fields[2], source_id)
    else {
        return Ok(None);
    };
    let Some((arena, env)) =
        mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, module)?
    else {
        return Ok(None);
    };
    if mizar_resolve::symbols::source_theorem_module_candidates(&ast, &arena, &env)?.is_none() {
        return Ok(None);
    }
    let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(
        mizar_resolve::names::SourceVariableScopeInput::new(&ast, module, &env),
    )
    .map_err(|error| format!("{error:?}"))?;
    let metadata =
        mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)?;
    let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(&arena, &env)?;
    let check =
        mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
            &arena,
            &metadata.2,
            &scope,
            &env,
            &labels,
            &resolved,
            None,
        )?;
    Ok(Some(
        mizar_core::elaborator::lower_source_theorem_skeletons(&check, None)?,
    ))
}

/// Stores lossless source syntax only when genuine replay reproduces the supplied Core.
pub fn native_algorithm_core_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    core: &mizar_core::core_ir::CoreIr,
) -> Result<Option<Vec<u8>>, String> {
    let Some(syntax) = ast.canonical_bytes() else {
        return Ok(None);
    };
    let domain = b"mizar-driver/native-algorithm-core-storage/v1\0";
    let fields = [
        module.package().as_str().as_bytes(),
        module.path().as_str().as_bytes(),
        syntax.as_slice(),
    ];
    let Some(bytes) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
        return Ok(None);
    };
    let Some(replayed) = native_algorithm_core_from_storage_bytes(&bytes, ast.source_id, module)?
    else {
        return Ok(None);
    };
    Ok((replayed == *core).then_some(bytes))
}

/// Rebinds stored syntax to an existing source id and replays the genuine lower owners.
pub fn native_algorithm_core_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
) -> Result<Option<mizar_core::core_ir::CoreIr>, String> {
    let domain = b"mizar-driver/native-algorithm-core-storage/v1\0";
    let Some(fields) = framing::decode::<3>(bytes, domain, 32 * 1024 * 1024) else {
        return Ok(None);
    };
    if fields[0] != module.package().as_str().as_bytes()
        || fields[1] != module.path().as_str().as_bytes()
    {
        return Ok(None);
    }
    let Some(ast) =
        <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(fields[2], source_id)
    else {
        return Ok(None);
    };
    let Some((arena, env)) =
        mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, module)?
    else {
        return Ok(None);
    };
    let typed = mizar_checker::typed_ast::TypedArena::try_new(
        Some(mizar_checker::typed_ast::TypedNodeId::new(
            arena.arena().root().index(),
        )),
        arena
            .arena()
            .iter()
            .map(|(id, node)| {
                mizar_checker::typed_ast::TypedNode::new(
                    format!("{:?}", node.kind()),
                    node.origin().anchor().clone(),
                )
                .with_resolved_node(id)
                .with_children(
                    node.children()
                        .iter()
                        .map(|child| mizar_checker::typed_ast::TypedNodeId::new(child.index()))
                        .collect(),
                )
            })
            .collect(),
    )
    .map_err(|error| error.to_string())?;
    let check = mizar_checker::type_checker::check_source_algorithm_types(&arena, &typed, &env)?;
    Ok(Some(mizar_core::elaborator::lower_source_algorithms(
        &check,
    )?))
}

/// Stores a deterministic replay of the genuine theorem preparation pair.
pub fn native_theorem_preparation_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    core: &mizar_core::core_ir::CoreIr,
    preparation: &(
        mizar_core::control_flow::ControlFlowOutput,
        mizar_core::control_flow::ObligationSeedHandoff,
    ),
) -> Result<Option<Vec<u8>>, String> {
    let Some(core_bytes) = native_theorem_core_storage_bytes(ast, module, core)? else {
        return Ok(None);
    };
    if !core.algorithms().is_empty() {
        return Ok(None);
    }
    let domain = b"mizar-driver/native-theorem-preparation-storage/v1\0";
    let Ok(length) = u64::try_from(core_bytes.len()) else {
        return Ok(None);
    };
    let Some(size) = domain
        .len()
        .checked_add(8)
        .and_then(|size| size.checked_add(core_bytes.len()))
    else {
        return Ok(None);
    };
    if size > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let flow = mizar_core::control_flow::build_control_flow_ir(core);
    let handoff = mizar_core::control_flow::build_obligation_seed_handoff(core, &flow);
    if &(flow, handoff) != preparation {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(domain);
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(&core_bytes);
    Ok(Some(bytes))
}

/// Rebuilds the genuine theorem preparation pair from opaque existing Core bytes.
pub fn native_theorem_preparation_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
) -> Result<
    Option<(
        mizar_core::control_flow::ControlFlowOutput,
        mizar_core::control_flow::ObligationSeedHandoff,
    )>,
    String,
> {
    let domain = b"mizar-driver/native-theorem-preparation-storage/v1\0";
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) = bytes.strip_prefix(domain) else {
        return Ok(None);
    };
    let Some(length) = remaining
        .get(..8)
        .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
        .map(u64::from_le_bytes)
        .and_then(|length| usize::try_from(length).ok())
    else {
        return Ok(None);
    };
    remaining = &remaining[8..];
    if remaining.len() != length {
        return Ok(None);
    }
    let Some(core) = native_theorem_core_from_storage_bytes(remaining, source_id, module)? else {
        return Ok(None);
    };
    if !core.algorithms().is_empty() {
        return Ok(None);
    }
    let flow = mizar_core::control_flow::build_control_flow_ir(&core);
    let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&core, &flow);
    Ok(Some((flow, handoff)))
}
