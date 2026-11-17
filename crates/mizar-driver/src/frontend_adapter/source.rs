use super::*;

type NativeFrontendCachePayload = (
    mizar_cache::cache_key::CacheKeyRequest,
    Vec<u8>,
    Vec<u8>,
    Vec<NamedInputHash>,
);

impl<'a> crate::registry::SourceLoadInputs<'a> {
    /// Explicitly compares protected native capture with Build's package config content profile.
    pub fn validate_native_config_content(self) -> Result<(), String> {
        let captured = self
            .captured
            .ok_or_else(|| "native config content requires captured sources".to_owned())?;
        if captured.snapshot() != self.snapshot
            || captured.native_sources().is_none()
            || self.build_plan.toolchain != self.snapshot.toolchain
        {
            return Err("native config content context mismatch".to_owned());
        }
        if mizar_build::planner::workspace_config_content_hash(
            &self.build_plan.verifier_config,
            &self.build_plan.build_config,
        ) != self.snapshot.verifier_config_hash
        {
            return Err("native config content mismatch".to_owned());
        }
        Ok(())
    }

    /// Explicitly compares protected native capture with Build's typed lock content profile.
    pub fn validate_native_lock_content(self) -> Result<(), String> {
        let captured = self
            .captured
            .ok_or_else(|| "native lock content requires captured sources".to_owned())?;
        if captured.snapshot() != self.snapshot
            || captured.native_sources().is_none()
            || self.build_plan.toolchain != self.snapshot.toolchain
        {
            return Err("native lock content context mismatch".to_owned());
        }
        if mizar_build::planner::lockfile_content_hash(&self.build_plan.lockfile)
            != self.snapshot.lockfile_hash
        {
            return Err("native lock content mismatch".to_owned());
        }
        Ok(())
    }

    /// Projects protected native Disk capture into plain cache request data.
    /// Root binding performs filesystem metadata I/O; no source body is reopened.
    pub fn prepare_native_disk_cache_request(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
    ) -> Result<mizar_cache::cache_key::CacheKeyRequest, String> {
        use mizar_cache::cache_key::{
            CACHE_KEY_SCHEMA_VERSION, CacheKeyRequest, CacheValidationInputs, CompatibilityField,
            FootprintCompleteness, NamedHash, NamedSchemaVersion, PipelinePhase as CachePhase,
            PolicyFingerprint, SchemaVersion as CacheSchema, SourceIdentity,
            WorkUnit as CacheWorkUnit,
        };
        use mizar_proof::policy::{DEFAULT_CHECKER_SCHEMA_VERSION, POLICY_SCHEMA_VERSION};

        if input.snapshot != self.snapshot.id
            || self.build_plan.toolchain != self.snapshot.toolchain
            || !input.parent_outputs().is_empty()
            || !input.identities().dependency_hashes().is_empty()
        {
            return Err("native Disk request context mismatch".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native Disk request requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native Disk request requires one captured source version".to_owned())?;
        if version.origin != SourceOrigin::Disk
            || input.identities().input_hash() != source_input_hash(version)
        {
            return Err("native Disk request source dispatch mismatch".to_owned());
        }
        let (source, source_input) = self
            .native_source(version)
            .ok_or_else(|| "native Disk request capture binding failed".to_owned())?;
        if !matches!(source_input.origin, SourceOriginInput::Disk { .. }) {
            return Err("native Disk request requires Disk capture".to_owned());
        }
        let storage = source
            .canonical_disk_bytes()
            .ok_or_else(|| "native Disk request storage is unsupported".to_owned())?;
        if policy.profile_id().trim().is_empty()
            || policy.schema_version() != POLICY_SCHEMA_VERSION
            || policy.checker_schema_version() != DEFAULT_CHECKER_SCHEMA_VERSION
        {
            return Err("native Disk request policy metadata is unsupported".to_owned());
        }
        let fingerprint = policy.policy_fingerprint().hash();
        let descriptor = SourceLoadService.phase();
        let mut toolchain_compatibility = vec![CompatibilityField {
            family: "mizar-session/toolchain".to_owned(),
            field_name: "identity".to_owned(),
            value: self.snapshot.toolchain.identity().to_owned(),
        }];
        if let Ok(image) = SourceLoadService::running_native_image() {
            toolchain_compatibility.push(image.clone());
        }
        Ok(CacheKeyRequest {
            cache_schema_version: CacheSchema::new(CACHE_KEY_SCHEMA_VERSION),
            phase: CachePhase::new(descriptor.service_name),
            work_unit: CacheWorkUnit::new(module_work_unit_label(
                source.package_id.as_str(),
                source.module_path.as_str(),
            )),
            source_identity: Some(SourceIdentity {
                package_id: source.package_id,
                module_path: source.module_path,
                normalized_source_path: source.normalized_path,
                source_hash: source.source_hash,
                language_edition: source.edition,
            }),
            input_hashes: vec![
                NamedHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                },
                NamedHash {
                    name: "source-storage".to_owned(),
                    domain: "mizar-frontend/source-unit-storage-side-table/v1".to_owned(),
                    digest: Hash::from_bytes(blake3::derive_key(
                        "mizar-frontend/source-unit-storage-side-table/v1",
                        &storage,
                    )),
                },
                NamedHash {
                    name: "captured-lockfile".to_owned(),
                    domain: "mizar-driver/captured-lockfile-identity/v1".to_owned(),
                    digest: self.snapshot.lockfile_hash,
                },
            ],
            dependency_hashes: Vec::new(),
            dependency_slices: Vec::new(),
            config_hash: self.snapshot.verifier_config_hash,
            schema_versions: vec![
                NamedSchemaVersion {
                    schema_family: "phase-output".to_owned(),
                    name: descriptor.output_kind,
                    version: CacheSchema::new(descriptor.schema_version),
                },
                NamedSchemaVersion {
                    schema_family: "source-key".to_owned(),
                    name: "SourceUnit".to_owned(),
                    version: CacheSchema::new(SOURCE_UNIT_CACHE_KEY_VERSION),
                },
                NamedSchemaVersion {
                    schema_family: "source-storage".to_owned(),
                    name: "Disk".to_owned(),
                    version: CacheSchema::new("mizar-frontend-source-disk-v1"),
                },
            ],
            policy_fingerprint: PolicyFingerprint::new(fingerprint),
            validation_inputs: CacheValidationInputs {
                cache_schema_compatibility: CacheSchema::new(CACHE_KEY_SCHEMA_VERSION),
                toolchain_compatibility,
                dependency_artifacts: Vec::new(),
                footprint_completeness: FootprintCompleteness::Complete,
                uncacheable: false,
                policy_compatibility: vec![
                    CompatibilityField {
                        family: "mizar-proof/verifier-policy".to_owned(),
                        field_name: "schema_version".to_owned(),
                        value: policy.schema_version().to_string(),
                    },
                    CompatibilityField {
                        family: "mizar-proof/verifier-policy".to_owned(),
                        field_name: "profile_id".to_owned(),
                        value: policy.profile_id().to_owned(),
                    },
                    CompatibilityField {
                        family: "mizar-proof/verifier-policy".to_owned(),
                        field_name: "checker_schema_version".to_owned(),
                        value: policy.checker_schema_version().to_string(),
                    },
                    CompatibilityField {
                        family: "mizar-proof/verifier-policy".to_owned(),
                        field_name: "fingerprint".to_owned(),
                        value: blake3::Hash::from(*fingerprint.as_bytes())
                            .to_hex()
                            .to_string(),
                    },
                ],
                canonical_vc_fingerprint: None,
                local_context_fingerprint: None,
                dependency_slice_fingerprints: Vec::new(),
                obligation_anchor_fingerprint: None,
                selected_proof_witness_hash: None,
                deterministic_discharge_hash: None,
                proof_reuse_schema_versions: Vec::new(),
                proof_reuse_validation_hash: None,
                proof_reuse_evidence_identities: Vec::new(),
                diagnostic_refs: Vec::new(),
            },
        })
    }

    /// Encodes a current SourceLoad output bound to protected native Disk capture.
    pub fn encode_native_disk_cache_record(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
        handle: mizar_ir::storage::PhaseOutputRef<SourceUnit>,
    ) -> Result<mizar_ir::cache_adapter::EncodeCacheRecordOutcome, String> {
        use mizar_cache::cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder};
        use mizar_ir::cache_adapter::{CacheAdapterCacheability, EncodeCacheRecordInput};

        let request = self.prepare_native_disk_cache_request(input, policy)?;
        let key = match CacheKeyBuilder::new(request.clone()).build() {
            CacheKeyBuildOutcome::Cacheable(key) => key,
            _ => return Err("native Disk encoding requires a cacheable key".to_owned()),
        };
        let publisher = adapter.publisher();
        let storage = publisher.storage();
        storage
            .validate_handle(handle.any())
            .map_err(|error| error.to_string())?;
        publisher
            .validate_output(
                input.snapshot,
                handle.any(),
                OutputAuthorization::CurrentPackage,
            )
            .map_err(|error| error.to_string())?;
        if handle.phase().as_str() != request.phase.as_str()
            || handle.work_unit().as_str() != request.work_unit.as_str()
            || handle.output_kind() != &OutputKind::new("SourceUnit")
            || handle.schema_version() != SchemaVersion::new(1)
        {
            return Err("native Disk encoding output binding mismatch".to_owned());
        }
        let lineage = publisher
            .registry()
            .output_lineage(handle.output())
            .ok_or_else(|| "native Disk encoding requires output lineage".to_owned())?;
        if !lineage.parents.is_empty()
            || lineage.named_input_hashes
                != vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: input.identities().input_hash(),
                }]
        {
            return Err("native Disk encoding lineage mismatch".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native Disk encoding requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native Disk encoding requires one captured source version".to_owned())?;
        let (source, source_input) = self
            .native_source(version)
            .ok_or_else(|| "native Disk encoding capture binding failed".to_owned())?;
        let payload = storage.get(&handle).map_err(|error| error.to_string())?;
        if payload.source_id != version.source_id
            || !payload.matches_canonical_storage(&source, &source_input)
        {
            return Err("native Disk encoding source storage mismatch".to_owned());
        }
        let storage_input = unique(
            request
                .input_hashes
                .iter()
                .filter(|entry| entry.name == "source-storage"),
        )
        .ok_or_else(|| "native Disk encoding requires source storage identity".to_owned())?;
        let side_tables = storage
            .side_tables(&handle)
            .map_err(|error| error.to_string())?;
        if *side_tables
            != (IrSideTables {
                source_maps: vec![SideTableRecord::new(
                    "source-storage",
                    source.normalized_path.as_str(),
                    storage_input.digest,
                )],
                ..IrSideTables::default()
            })
        {
            return Err("native Disk encoding side tables mismatch".to_owned());
        }
        let storage_payload = payload
            .canonical_disk_bytes()
            .ok_or_else(|| "native Disk encoding storage is unsupported".to_owned())?;
        Ok(adapter.encode(EncodeCacheRecordInput {
            handle,
            cache_key: key,
            produced_by: request.validation_inputs.toolchain_compatibility,
            canonical_payload: source_input_hash(version).as_bytes().to_vec(),
            storage_payload,
            cacheability: CacheAdapterCacheability::Cacheable,
        }))
    }

    fn prepare_native_disk_frontend_cache_payload(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
        handle: Option<
            &mizar_ir::storage::PhaseOutputRef<
                FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
            >,
        >,
        frontend: Option<&FrontendService>,
        leaf_sources: &[mizar_ir::storage::AnyPhaseOutputRef],
    ) -> Result<NativeFrontendCachePayload, String> {
        use mizar_cache::cache_key::{
            NamedHash, NamedSchemaVersion, PipelinePhase as CachePhase,
            SchemaVersion as CacheSchema,
        };
        use mizar_frontend::cache_key::PreprocessedSourceCacheKey;
        use mizar_ir::dispatch_input::PhaseDispatchInputBundle;

        self.validate_native_lock_content()?;
        self.validate_native_config_content()?;
        if !input.identities().dependency_hashes().is_empty()
            || !self.snapshot.dependency_artifacts.is_empty()
            || !self.module_index.dependency_summaries.is_empty()
            || self
                .module_index
                .modules
                .iter()
                .any(|entry| !matches!(entry.location, ModuleIndexLocation::WorkspaceFile { .. }))
            || !(input.parent_outputs().len() == 1
                || (frontend.is_some()
                    && !leaf_sources.is_empty()
                    && input.parent_outputs().len() == 1 + leaf_sources.len()))
        {
            return Err("native Frontend encoding requires the leaf profile".to_owned());
        }
        let source_input = PhaseInput::new(
            input.work_unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                input.snapshot,
                input.identities().input_hash(),
                Vec::new(),
            ),
        );
        let mut request = self.prepare_native_disk_cache_request(&source_input, policy)?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native Frontend encoding requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native Frontend encoding requires one captured source".to_owned())?;
        let (source, captured_input) = self
            .native_source(version)
            .ok_or_else(|| "native Frontend encoding capture binding failed".to_owned())?;
        let publisher = adapter.publisher();
        let storage = publisher.storage();
        let parent = unique(
            input
                .parent_outputs()
                .iter()
                .map(|parent| parent.as_output_ref())
                .filter(|parent| {
                    parent.phase().as_str() == "SourceLoad"
                        && parent.work_unit().as_str() == request.work_unit.as_str()
                }),
        )
        .ok_or_else(|| "native Frontend requires one own SourceLoad parent".to_owned())?;
        let imported = input.parent_outputs().len() > 1;
        if !imported && !leaf_sources.is_empty() {
            return Err("native Frontend surplus leaf source parents".to_owned());
        }
        let mut physical_units = std::collections::BTreeSet::new();
        for physical in input
            .parent_outputs()
            .iter()
            .map(|parent| parent.as_output_ref())
            .chain(leaf_sources)
        {
            if !physical_units.insert((physical.phase().as_str(), physical.work_unit().as_str())) {
                return Err("native Frontend duplicate physical parent".to_owned());
            }
            storage
                .validate_handle(physical)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    physical,
                    OutputAuthorization::CurrentPackage,
                )
                .map_err(|error| error.to_string())?;
        }
        let workspace_summaries =
            FrontendService::authenticated_workspace_summaries(self, input, publisher, parent)
                .ok_or_else(|| "native Frontend dependency authentication failed".to_owned())?;
        if imported {
            use mizar_ir::dispatch_input::SealedParentOutputHandle;
            if workspace_summaries.len() != leaf_sources.len() {
                return Err("native Frontend leaf SourceLoad correspondence mismatch".to_owned());
            }
            let mut targets = std::collections::BTreeSet::new();
            for (target, _) in &workspace_summaries {
                if !targets.insert((target.package.as_str(), target.path.as_str())) {
                    return Err("native Frontend duplicate leaf target".to_owned());
                }
                let unit = mizar_session::module_work_unit_label(
                    target.package.as_str(),
                    target.path.as_str(),
                );
                let leaf = unique(
                    input
                        .parent_outputs()
                        .iter()
                        .map(|parent| parent.as_output_ref())
                        .filter(|candidate| {
                            candidate.phase().as_str() == "Frontend"
                                && candidate.work_unit().as_str() == unit
                        }),
                )
                .ok_or_else(|| "native Frontend requires one Frontend per target".to_owned())?;
                let leaf_source = unique(leaf_sources.iter().filter(|candidate| {
                    candidate.phase().as_str() == "SourceLoad"
                        && candidate.work_unit().as_str() == unit
                }))
                .ok_or_else(|| "native Frontend requires one SourceLoad per target".to_owned())?;
                if leaf.lineage().parents != [leaf_source.output()] {
                    return Err(
                        "native Frontend leaf SourceLoad correspondence mismatch".to_owned()
                    );
                }
                let leaf_version = unique(self.snapshot.source_versions.iter().filter(|version| {
                    version.package_id == target.package && version.module_path == target.path
                }))
                .ok_or_else(|| "native Frontend leaf capture is unavailable".to_owned())?;
                let leaf_input = PhaseInput::new(
                    WorkUnit::Module {
                        module: target.clone(),
                    },
                    PhaseDispatchInputBundle::new(
                        input.snapshot,
                        source_input_hash(leaf_version),
                        Vec::new(),
                        vec![
                            SealedParentOutputHandle::from_current_output(
                                publisher,
                                input.snapshot,
                                leaf_source.clone(),
                            )
                            .map_err(|error| error.to_string())?,
                        ],
                    )
                    .map_err(|error| error.to_string())?,
                );
                let leaf_handle = storage
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        leaf,
                        &OutputKind::new("FrontendOutput"),
                    )
                    .map_err(|error| error.to_string())?;
                self.prepare_native_disk_frontend_cache_payload(
                    &leaf_input,
                    policy,
                    adapter,
                    Some(&leaf_handle),
                    None,
                    &[],
                )?;
            }
        }

        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_output(input.snapshot, parent, OutputAuthorization::CurrentPackage)
            .map_err(|error| error.to_string())?;
        if parent.phase().as_str() != "SourceLoad"
            || parent.work_unit().as_str() != request.work_unit.as_str()
            || parent.output_kind() != &OutputKind::new("SourceUnit")
            || parent.schema_version() != SchemaVersion::new(1)
        {
            return Err("native Frontend encoding source parent mismatch".to_owned());
        }
        let typed_parent = storage
            .typed_handle::<SourceUnit>(parent, &OutputKind::new("SourceUnit"))
            .map_err(|error| error.to_string())?;
        let parent_source = storage
            .get(&typed_parent)
            .map_err(|error| error.to_string())?;
        if parent_source.source_id != version.source_id
            || !parent_source.matches_canonical_storage(&source, &captured_input)
        {
            return Err("native Frontend encoding parent storage mismatch".to_owned());
        }
        let source_storage_hash = request
            .input_hashes
            .iter()
            .find(|entry| entry.name == "source-storage")
            .ok_or_else(|| "native Frontend encoding requires source storage identity".to_owned())?
            .digest;
        if !typed_parent.lineage().parents.is_empty()
            || typed_parent.lineage().named_input_hashes
                != vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: input.identities().input_hash(),
                }]
            || *storage
                .side_tables(&typed_parent)
                .map_err(|error| error.to_string())?
                != (IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "source-storage",
                        source.normalized_path.as_str(),
                        source_storage_hash,
                    )],
                    ..IrSideTables::default()
                })
        {
            return Err("native Frontend encoding source parent lineage mismatch".to_owned());
        }
        let retained = if let Some(handle) = handle {
            storage
                .validate_handle(handle.any())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    handle.any(),
                    OutputAuthorization::CurrentPackage,
                )
                .map_err(|error| error.to_string())?;
            if handle.phase().as_str() != "Frontend"
                || handle.work_unit().as_str() != request.work_unit.as_str()
                || handle.output_kind() != &OutputKind::new("FrontendOutput")
                || handle.schema_version() != SchemaVersion::new(1)
            {
                return Err("native Frontend encoding output binding mismatch".to_owned());
            }
            Some(storage.get(handle).map_err(|error| error.to_string())?)
        } else {
            None
        };
        let output = match &retained {
            Some(output) if !imported => output.clone(),
            _ => {
                let frontend =
                    frontend.ok_or_else(|| "native Frontend producer unavailable".to_owned())?;
                let package = unique(
                    self.build_plan
                        .packages
                        .iter()
                        .filter(|package| package.package_id == module.package),
                )
                .ok_or_else(|| "native Frontend producer requires one package".to_owned())?;
                let PackagePlanSource::Workspace { root, .. } = &package.source else {
                    return Err("native Frontend producer requires a workspace package".to_owned());
                };
                Arc::new(frontend.analyze_loaded(
                    self,
                    PathBuf::from(self.snapshot.workspace_root.as_str()).join(root),
                    &workspace_summaries,
                    source.clone(),
                )?)
            }
        };
        if imported
            && retained.as_ref().is_some_and(|retained| {
                retained.canonical_disk_bytes() != output.canonical_disk_bytes()
            })
        {
            return Err("native Frontend output differs from independent analysis".to_owned());
        }
        let named_input_hashes = vec![
            NamedInputHash {
                name: "active-lexical-environment".to_owned(),
                domain: ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION.to_owned(),
                digest: output.cache_keys.active_lexical_environment.stable_hash(),
            },
            NamedInputHash {
                name: "tokens".to_owned(),
                domain: TOKEN_STREAM_CACHE_KEY_VERSION.to_owned(),
                digest: output.cache_keys.tokens.stable_hash(),
            },
        ];
        if output.source.source_id != version.source_id
            || !output
                .source
                .matches_canonical_storage(&source, &captured_input)
            || output.ast.is_none()
            || !output.diagnostics.is_empty()
            || (!imported && !output.preprocessed.import_stubs.is_empty())
            || output.cache_keys.source != SourceUnitCacheKey::from_source(&source)
            || output.cache_keys.preprocessed != PreprocessedSourceCacheKey::from_source(&source)
            || handle.is_some_and(|handle| {
                handle.lineage().parents.len() != input.parent_outputs().len()
                    || input
                        .parent_outputs()
                        .iter()
                        .any(|parent| !handle.lineage().parents.contains(&parent.output()))
                    || handle.lineage().named_input_hashes != named_input_hashes
            })
        {
            return Err("native Frontend encoding output source or lineage mismatch".to_owned());
        }
        if imported {
            let domain = "mizar-driver/native-disk-frontend-dependencies/v1";
            let dependency_bytes = {
                use mizar_resolve::{imports::ImportPathResolver, module_index::ModuleIndexInput};
                use serde_json::json;
                let unavailable = || "native Frontend dependency closure is unsupported".to_owned();
                if output
                    .ast
                    .as_ref()
                    .is_none_or(|ast| ast.node_views().any(|view| view.as_export_item().is_some()))
                {
                    return Err(unavailable());
                }
                let resolution = ImportPathResolver::new(ModuleIndexInput::new(self.module_index))
                    .resolve_frontend(&output, &output.source)
                    .ok_or_else(unavailable)?;
                if !resolution.unresolved().is_empty() || resolution.resolved().is_empty() {
                    return Err(unavailable());
                }
                let range = |span: mizar_session::SourceRange| json!([span.start, span.end]);
                let mut resolved = resolution.resolved().iter().collect::<Vec<_>>();
                resolved.sort_by_key(|import| import.ordinal());
                let mut imports = Vec::new();
                let mut resolved_targets = std::collections::BTreeSet::new();
                for import in resolved {
                    let target = (import.target().package(), import.target().path());
                    if !workspace_summaries
                        .iter()
                        .any(|(module, _)| (&module.package, &module.path) == target)
                    {
                        return Err(unavailable());
                    }
                    resolved_targets.insert((target.0.as_str(), target.1.as_str()));
                    imports.push(json!([
                        import.target().package().as_str(),
                        import.target().path().as_str(),
                        import.ordinal(),
                        range(import.range()),
                        import.alias_range().map(range),
                        import.branch_base_range().map(range),
                        import.branch_member_range().map(range)
                    ]));
                }
                if resolved_targets.len() != workspace_summaries.len() {
                    return Err(unavailable());
                }
                let mut ordered_workspace_summaries =
                    workspace_summaries.iter().collect::<Vec<_>>();
                ordered_workspace_summaries.sort_by(|left, right| {
                    (left.0.package.as_str(), left.0.path.as_str())
                        .cmp(&(right.0.package.as_str(), right.0.path.as_str()))
                });
                let mut summary_bytes = Vec::new();
                for (target, summary) in ordered_workspace_summaries {
                    let shapes = summary
                        .exported_symbols
                        .iter()
                        .map(ExportedSymbolShape::canonical_bytes)
                        .collect::<Option<Vec<_>>>()
                        .ok_or_else(unavailable)?;
                    summary_bytes.push(json!([
                        target.package.as_str(),
                        target.path.as_str(),
                        summary.module_id.as_str(),
                        summary.fingerprint.get(),
                        shapes
                    ]));
                }
                let mut parents = input
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref())
                    .chain(leaf_sources)
                    .collect::<Vec<_>>();
                parents.sort_by(|left, right| {
                    (left.phase().as_str(), left.work_unit().as_str())
                        .cmp(&(right.phase().as_str(), right.work_unit().as_str()))
                });
                let parents = parents
                    .into_iter()
                    .map(|parent| {
                        json!([
                            parent.phase().as_str(),
                            parent.work_unit().as_str(),
                            parent.content_hash().as_bytes(),
                            parent.side_table_hash().as_bytes()
                        ])
                    })
                    .collect::<Vec<_>>();
                serde_json::to_vec(&json!([
                    domain,
                    output
                        .preprocessed
                        .canonical_bytes()
                        .ok_or_else(unavailable)?,
                    output.tokens.canonical_bytes().ok_or_else(unavailable)?,
                    imports,
                    summary_bytes,
                    parents
                ]))
                .map_err(|error| error.to_string())?
            };
            request.input_hashes.push(NamedHash {
                name: "frontend-dependencies".to_owned(),
                domain: domain.to_owned(),
                digest: Hash::from_bytes(blake3::derive_key(domain, &dependency_bytes)),
            });
        }
        let ast_key = output
            .cache_keys
            .ast
            .as_ref()
            .ok_or_else(|| "native Frontend encoding requires an AST key".to_owned())?;
        let key_bytes = output
            .cache_keys
            .canonical_bytes()
            .ok_or_else(|| "native Frontend encoding requires canonical keys".to_owned())?;
        let storage_payload = output
            .canonical_disk_bytes()
            .ok_or_else(|| "native Frontend encoding storage is unsupported".to_owned())?;
        if let Some(handle) = handle
            && *storage
                .side_tables(handle)
                .map_err(|error| error.to_string())?
                != (IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "frontend-storage",
                        source.normalized_path.as_str(),
                        Hash::from_bytes(blake3::derive_key(
                            "mizar-frontend/output-storage-side-table/v1",
                            &storage_payload,
                        )),
                    )],
                    ..IrSideTables::default()
                })
        {
            return Err("native Frontend encoding side tables mismatch".to_owned());
        }
        let descriptor = FrontendService {
            artifact_roots: Vec::new(),
        }
        .phase();
        request.phase = CachePhase::new(descriptor.service_name);
        let schema = request
            .schema_versions
            .iter_mut()
            .find(|entry| entry.schema_family == "phase-output")
            .ok_or_else(|| "native Frontend encoding requires phase schema".to_owned())?;
        schema.name = descriptor.output_kind;
        schema.version = CacheSchema::new(descriptor.schema_version);
        let mut parent_bytes = parent.content_hash().as_bytes().to_vec();
        parent_bytes.extend_from_slice(parent.side_table_hash().as_bytes());
        for (name, domain, bytes) in [
            (
                "frontend-cache-keys",
                "mizar-driver/native-disk-frontend-cache-keys/v1",
                key_bytes,
            ),
            (
                "source-parent",
                "mizar-driver/native-disk-frontend-source-parent/v1",
                parent_bytes,
            ),
        ] {
            request.input_hashes.push(NamedHash {
                name: name.to_owned(),
                domain: domain.to_owned(),
                digest: Hash::from_bytes(blake3::derive_key(domain, &bytes)),
            });
        }
        for (schema_family, name, version) in [
            (
                "frontend-keys",
                "FrontendCacheKeys",
                "mizar-frontend/cache-keys/v1",
            ),
            ("frontend-storage", "Disk", "mizar-frontend-output-disk-v1"),
        ] {
            request.schema_versions.push(NamedSchemaVersion {
                schema_family: schema_family.to_owned(),
                name: name.to_owned(),
                version: CacheSchema::new(version),
            });
        }
        let mut canonical_payload = b"mizar-frontend/output-publication/v1\0".to_vec();
        canonical_payload.extend_from_slice(ast_key.stable_hash().as_bytes());
        for physical in input
            .parent_outputs()
            .iter()
            .map(|parent| parent.as_output_ref())
            .chain(leaf_sources)
            .chain(handle.map(|handle| handle.any()))
        {
            storage
                .validate_handle(physical)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    physical,
                    OutputAuthorization::CurrentPackage,
                )
                .map_err(|error| error.to_string())?;
        }
        Ok((
            request,
            canonical_payload,
            storage_payload,
            named_input_hashes,
        ))
    }

    /// Encodes a current zero-import Frontend output bound to native Disk capture.
    pub fn encode_native_disk_frontend_cache_record(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
        handle: mizar_ir::storage::PhaseOutputRef<
            FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
        >,
    ) -> Result<mizar_ir::cache_adapter::EncodeCacheRecordOutcome, String> {
        use mizar_cache::cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder};
        use mizar_ir::cache_adapter::{CacheAdapterCacheability, EncodeCacheRecordInput};

        let (request, canonical_payload, storage_payload, _): NativeFrontendCachePayload = self
            .prepare_native_disk_frontend_cache_payload(
                input,
                policy,
                adapter,
                Some(&handle),
                None,
                &[],
            )?;
        let key = match CacheKeyBuilder::new(request.clone()).build() {
            CacheKeyBuildOutcome::Cacheable(key) => key,
            _ => return Err("native Frontend encoding requires a cacheable key".to_owned()),
        };
        Ok(adapter.encode(EncodeCacheRecordInput {
            handle,
            cache_key: key,
            produced_by: request.validation_inputs.toolchain_compatibility,
            canonical_payload,
            storage_payload,
            cacheability: CacheAdapterCacheability::Cacheable,
        }))
    }

    pub(crate) fn encode_native_disk_frontend_cache_record_with_frontend(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
        handle: mizar_ir::storage::PhaseOutputRef<
            FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
        >,
        frontend: &FrontendService,
        leaf_sources: &[mizar_ir::storage::AnyPhaseOutputRef],
    ) -> Result<mizar_ir::cache_adapter::EncodeCacheRecordOutcome, String> {
        use mizar_cache::cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder};
        use mizar_ir::cache_adapter::{CacheAdapterCacheability, EncodeCacheRecordInput};

        let (request, canonical_payload, storage_payload, _): NativeFrontendCachePayload = self
            .prepare_native_disk_frontend_cache_payload(
                input,
                policy,
                adapter,
                Some(&handle),
                Some(frontend),
                leaf_sources,
            )?;
        let key = match CacheKeyBuilder::new(request.clone()).build() {
            CacheKeyBuildOutcome::Cacheable(key) => key,
            _ => return Err("native Frontend encoding requires a cacheable key".to_owned()),
        };
        Ok(adapter.encode(EncodeCacheRecordInput {
            handle,
            cache_key: key,
            produced_by: request.validation_inputs.toolchain_compatibility,
            canonical_payload,
            storage_payload,
            cacheability: CacheAdapterCacheability::Cacheable,
        }))
    }

    /// Reconstructs a native leaf Frontend output against a retained current oracle.
    pub fn rehydrate_native_disk_frontend_cache(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
        store: &mizar_cache::cache_store::CacheStoreRoot,
        oracle: (
            &PhaseInput,
            &mizar_ir::cache_adapter::IrCacheAdapter,
            &mizar_ir::storage::PhaseOutputRef<
                FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
            >,
        ),
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
    ) -> Result<
        mizar_ir::cache_adapter::CacheRehydrateOutcome<
            FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
        >,
        String,
    > {
        let (oracle_input, oracle_adapter, oracle_handle) = oracle;
        let payload = self.prepare_native_disk_frontend_cache_payload(
            oracle_input,
            policy,
            oracle_adapter,
            Some(oracle_handle),
            None,
            &[],
        )?;
        if input.snapshot != oracle_input.snapshot
            || input.work_unit != oracle_input.work_unit
            || input.identities() != oracle_input.identities()
            || input.parent_outputs().len() != 1
        {
            return Err("native Frontend lookup target binding mismatch".to_owned());
        }
        self.rehydrate_native_disk_frontend_payload(
            input,
            store,
            adapter,
            payload,
            Some(oracle_input.parent_outputs()[0].output()),
            &[],
        )
    }

    pub(crate) fn rehydrate_native_disk_frontend_cache_cold(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
        store: &mizar_cache::cache_store::CacheStoreRoot,
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
        frontend: &FrontendService,
    ) -> Result<
        mizar_ir::cache_adapter::CacheRehydrateOutcome<
            FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
        >,
        String,
    > {
        let payload = self.prepare_native_disk_frontend_cache_payload(
            input,
            policy,
            adapter,
            None,
            Some(frontend),
            &[],
        )?;
        self.rehydrate_native_disk_frontend_payload(input, store, adapter, payload, None, &[])
    }

    pub(crate) fn rehydrate_native_disk_frontend_cache_with_dependencies(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
        store: &mizar_cache::cache_store::CacheStoreRoot,
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
        frontend: &FrontendService,
        leaf_sources: &[mizar_ir::storage::AnyPhaseOutputRef],
    ) -> Result<
        mizar_ir::cache_adapter::CacheRehydrateOutcome<
            FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
        >,
        String,
    > {
        let payload = self.prepare_native_disk_frontend_cache_payload(
            input,
            policy,
            adapter,
            None,
            Some(frontend),
            leaf_sources,
        )?;
        self.rehydrate_native_disk_frontend_payload(
            input,
            store,
            adapter,
            payload,
            None,
            leaf_sources,
        )
    }

    fn rehydrate_native_disk_frontend_payload(
        self,
        input: &PhaseInput,
        store: &mizar_cache::cache_store::CacheStoreRoot,
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
        payload: NativeFrontendCachePayload,
        oracle_parent: Option<mizar_ir::identity::PhaseOutputId>,
        leaf_sources: &[mizar_ir::storage::AnyPhaseOutputRef],
    ) -> Result<
        mizar_ir::cache_adapter::CacheRehydrateOutcome<
            FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
        >,
        String,
    > {
        use mizar_cache::{
            cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
            cache_store::CacheLookupOutcome,
        };
        use mizar_ir::cache_adapter::RehydrateCacheHitInput;
        let (request, semantic, storage_payload, named_input_hashes) = payload;
        let publisher = adapter.publisher();
        publisher
            .validate_current_snapshot(input.snapshot)
            .map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native Frontend lookup requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native Frontend lookup requires one captured source".to_owned())?;
        let (source, source_input) = self
            .native_source(version)
            .ok_or_else(|| "native Frontend lookup capture binding failed".to_owned())?;
        let storage = publisher.storage();
        let parent = unique(
            input
                .parent_outputs()
                .iter()
                .map(|parent| parent.as_output_ref())
                .filter(|parent| {
                    parent.phase().as_str() == "SourceLoad"
                        && parent.work_unit().as_str() == request.work_unit.as_str()
                }),
        )
        .ok_or_else(|| "native Frontend lookup requires one own SourceLoad parent".to_owned())?;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_output(input.snapshot, parent, OutputAuthorization::CurrentPackage)
            .map_err(|error| error.to_string())?;
        if parent.phase().as_str() != "SourceLoad"
            || parent.work_unit().as_str() != request.work_unit.as_str()
            || parent.output_kind() != &OutputKind::new("SourceUnit")
            || parent.schema_version() != SchemaVersion::new(1)
        {
            return Err("native Frontend lookup source parent mismatch".to_owned());
        }
        let typed_parent = storage
            .typed_handle::<SourceUnit>(parent, &OutputKind::new("SourceUnit"))
            .map_err(|error| error.to_string())?;
        let parent_source = storage
            .get(&typed_parent)
            .map_err(|error| error.to_string())?;
        if parent_source.source_id != version.source_id
            || !parent_source.matches_canonical_storage(&source, &source_input)
        {
            return Err("native Frontend lookup parent storage mismatch".to_owned());
        }
        let source_storage_hash = request
            .input_hashes
            .iter()
            .find(|entry| entry.name == "source-storage")
            .ok_or_else(|| "native Frontend lookup requires source storage identity".to_owned())?
            .digest;
        if !typed_parent.lineage().parents.is_empty()
            || typed_parent.lineage().named_input_hashes
                != vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: input.identities().input_hash(),
                }]
            || *storage
                .side_tables(&typed_parent)
                .map_err(|error| error.to_string())?
                != (IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "source-storage",
                        source.normalized_path.as_str(),
                        source_storage_hash,
                    )],
                    ..IrSideTables::default()
                })
            || oracle_parent.is_some_and(|output| parent.output() != output)
        {
            return Err("native Frontend lookup source parent lineage mismatch".to_owned());
        }
        let key = match CacheKeyBuilder::new(request.clone()).build() {
            CacheKeyBuildOutcome::Cacheable(key) => key,
            _ => return Err("native Frontend lookup requires a cacheable key".to_owned()),
        };
        let side_tables = IrSideTables {
            source_maps: vec![SideTableRecord::new(
                "frontend-storage",
                source.normalized_path.as_str(),
                Hash::from_bytes(blake3::derive_key(
                    "mizar-frontend/output-storage-side-table/v1",
                    &storage_payload,
                )),
            )],
            ..IrSideTables::default()
        };
        let lookup = store.lookup(&key);
        if let CacheLookupOutcome::Hit(record) = &lookup
            && record.header.produced_by != request.validation_inputs.toolchain_compatibility
        {
            return Err("native Frontend lookup producer binding mismatch".to_owned());
        }
        for physical in input
            .parent_outputs()
            .iter()
            .map(|parent| parent.as_output_ref())
            .chain(leaf_sources)
        {
            storage
                .validate_handle(physical)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    physical,
                    OutputAuthorization::CurrentPackage,
                )
                .map_err(|error| error.to_string())?;
        }
        let outcome = adapter.rehydrate_with_expected_payload(
            RehydrateCacheHitInput {
                lookup,
                snapshot: input.snapshot,
                phase: IrPipelinePhase::new(request.phase.as_str()),
                work_unit: IrWorkUnit::new(request.work_unit.as_str()),
                output_kind: OutputKind::new("FrontendOutput"),
                schema_version: SchemaVersion::new(1),
                parents: input
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref().clone())
                    .collect(),
                named_input_hashes,
                decode: BlobDecoder::new(move |bytes| {
                    if bytes != storage_payload {
                        return Err(BlobDecodeError::new(
                            "native Frontend lookup storage mismatch",
                        ));
                    }
                    FrontendOutput::from_canonical_storage_bytes(
                        bytes,
                        source.source_id,
                        &source_input,
                    )
                    .filter(|decoded| {
                        decoded
                            .source
                            .matches_canonical_storage(&source, &source_input)
                    })
                    .ok_or_else(|| BlobDecodeError::new("native Frontend lookup storage mismatch"))
                }),
            },
            &semantic,
            &side_tables,
        );
        for physical in input
            .parent_outputs()
            .iter()
            .map(|parent| parent.as_output_ref())
            .chain(leaf_sources)
        {
            storage
                .validate_handle(physical)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    physical,
                    OutputAuthorization::CurrentPackage,
                )
                .map_err(|error| error.to_string())?;
        }
        Ok(outcome)
    }

    /// Reconstructs an authorized current source from its genuine native Disk key.
    pub fn rehydrate_native_disk_cache(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
        store: &mizar_cache::cache_store::CacheStoreRoot,
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
    ) -> Result<mizar_ir::cache_adapter::CacheRehydrateOutcome<SourceUnit>, String> {
        use mizar_cache::{
            cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
            cache_store::CacheLookupOutcome,
        };
        use mizar_ir::cache_adapter::RehydrateCacheHitInput;

        let request = self.prepare_native_disk_cache_request(input, policy)?;
        let key = match CacheKeyBuilder::new(request.clone()).build() {
            CacheKeyBuildOutcome::Cacheable(key) => key,
            _ => return Err("native Disk lookup requires a cacheable key".to_owned()),
        };
        adapter
            .publisher()
            .validate_current_snapshot(input.snapshot)
            .map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native Disk lookup requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native Disk lookup requires one captured source version".to_owned())?;
        let (source, source_input) = self
            .native_source(version)
            .ok_or_else(|| "native Disk lookup capture binding failed".to_owned())?;
        let storage_input = unique(
            request
                .input_hashes
                .iter()
                .filter(|entry| entry.name == "source-storage"),
        )
        .ok_or_else(|| "native Disk lookup requires source storage identity".to_owned())?;
        let side_tables = IrSideTables {
            source_maps: vec![SideTableRecord::new(
                "source-storage",
                source.normalized_path.as_str(),
                storage_input.digest,
            )],
            ..IrSideTables::default()
        };
        let lookup = store.lookup(&key);
        if let CacheLookupOutcome::Hit(record) = &lookup
            && record.header.produced_by != request.validation_inputs.toolchain_compatibility
        {
            return Err("native Disk lookup producer binding mismatch".to_owned());
        }
        let source_hash = input.identities().input_hash();
        Ok(adapter.rehydrate_with_expected_payload(
            RehydrateCacheHitInput {
                lookup,
                snapshot: input.snapshot,
                phase: IrPipelinePhase::new(request.phase.as_str()),
                work_unit: IrWorkUnit::new(request.work_unit.as_str()),
                output_kind: OutputKind::new("SourceUnit"),
                schema_version: SchemaVersion::new(1),
                parents: Vec::new(),
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_hash,
                }],
                decode: BlobDecoder::new(move |bytes| {
                    SourceUnit::from_canonical_storage_bytes(bytes, source.source_id, &source_input)
                        .filter(|decoded| decoded.matches_canonical_storage(&source, &source_input))
                        .ok_or_else(|| BlobDecodeError::new("native Disk lookup storage mismatch"))
                }),
            },
            source_hash.as_bytes(),
            &side_tables,
        ))
    }

    pub(super) fn native_source(
        self,
        version: &mizar_session::SourceVersion,
    ) -> Option<(SourceUnit, SourceInput)> {
        let captured = self.captured?;
        if captured.snapshot() != self.snapshot
            || self.build_plan.toolchain != self.snapshot.toolchain
        {
            return None;
        }
        let (loader, loaded, input) =
            unique(captured.native_sources()?.iter().filter(|(_, loaded, _)| {
                loaded.package_id == version.package_id && loaded.module_path == version.module_path
            }))?;
        if loaded.source_id != version.source_id
            || loaded.normalized_path != version.normalized_path
            || loaded.edition != version.edition
            || loaded.source_hash != version.source_hash
            || loaded.origin != version.origin
            || input.package_id != loaded.package_id
            || input.module_path != loaded.module_path
            || input.normalized_path != loaded.normalized_path
            || input.edition != loaded.edition
        {
            return None;
        }
        let package = unique(
            self.build_plan
                .packages
                .iter()
                .filter(|package| package.package_id == version.package_id),
        )?;
        let PackagePlanSource::Workspace {
            root,
            source_root,
            manifest_path,
        } = &package.source
        else {
            return None;
        };
        let indexed = unique(
            self.module_index
                .packages
                .iter()
                .filter(|package| package.package_id == version.package_id),
        )?;
        let mizar_build::module_index::PackageIndexSource::Workspace {
            package_root: index_root,
            source_root: index_source_root,
            manifest_path: index_manifest,
        } = &indexed.source
        else {
            return None;
        };
        let entry = unique(self.module_index.modules.iter().filter(|entry| {
            entry.package_id == version.package_id && entry.module_path == version.module_path
        }))?;
        let ModuleIndexLocation::WorkspaceFile {
            source_root: entry_root,
            normalized_path,
            source_relative_path,
        } = &entry.location
        else {
            return None;
        };
        let workspace = PathBuf::from(self.snapshot.workspace_root.as_str())
            .canonicalize()
            .ok()?;
        let expected_root = workspace.join(root).canonicalize().ok()?;
        if self.snapshot.workspace_root != self.build_plan.workspace_root
            || !safe_relative(root)
            || !safe_relative(source_root)
            || !safe_relative(manifest_path)
            || !expected_root.starts_with(&workspace)
            || loader.package_root().canonicalize().ok()? != expected_root
            || root != index_root
            || source_root != index_source_root
            || manifest_path != index_manifest
            || package.version != indexed.version
            || package.edition != version.edition
            || indexed.edition != version.edition
            || entry.edition != version.edition
            || entry.module.package != version.package_id
            || entry.module.path != version.module_path
            || entry_root != source_root
            || normalized_path != version.normalized_path.as_str()
            || normalized_path != &format!("src/{source_relative_path}")
            || source_relative_path.is_empty()
            || workspace.join(source_root) != workspace.join(root).join("src")
            || workspace.join(manifest_path) != workspace.join(root).join("mizar.pkg")
        {
            return None;
        }
        let source = source_unit_from_loaded(loaded.clone(), diagnostic_file_path(input));
        match &input.origin {
            SourceOriginInput::Disk { .. } => {
                source.canonical_disk_bytes()?;
            }
            SourceOriginInput::OpenBuffer { .. } => {
                source.canonical_open_buffer_bytes(input)?;
            }
            _ => return None,
        }
        Some((source, input.clone()))
    }

    /// Acquires provisional lexical ordering without publishing source or phase outputs.
    pub(crate) fn discover_import_overlay(
        self,
    ) -> Option<mizar_build::task_graph::ModuleDependencyOverlay> {
        use mizar_build::{
            module_index::{ModuleId, ModuleIndexLocation},
            planner::PackagePlanSource,
            task_graph::{ModuleDependencyCoverage, ModuleDependencyEdge, ModuleDependencyKind},
        };
        use mizar_frontend::{
            lexical_env::LexicalEnvironmentRequest,
            preprocess::preprocess,
            source::{
                FrontendSourceLoader, SourceUnitLoader, SourceUnitRequest, register_source_unit,
            },
            span_bridge::SpanBridge,
        };
        use mizar_resolve::{
            imports::{ImportPathCandidate, ImportPathResolver},
            module_index::ModuleIndexInput,
        };
        use mizar_session::{
            DiskSourceLoader, InMemorySessionIdAllocator, SourceInput, SourceOrigin,
            SourceOriginInput,
        };
        let snapshot = self.snapshot;
        let build_plan = self.build_plan;
        let module_index = self.module_index;
        if snapshot.workspace_root != build_plan.workspace_root {
            return None;
        }
        let workspace = std::path::Path::new(snapshot.workspace_root.as_str())
            .canonicalize()
            .ok()?;
        let local_ids = InMemorySessionIdAllocator::new();
        let index = ModuleIndexInput::new(module_index);
        let mut edges = Vec::new();
        let mut source_count = 0;
        for entry in &module_index.modules {
            let ModuleIndexLocation::WorkspaceFile {
                source_root,
                normalized_path,
                source_relative_path,
            } = &entry.location
            else {
                continue;
            };
            source_count += 1;
            let mut versions = snapshot.source_versions.iter().filter(|version| {
                version.package_id == entry.module.package
                    && version.module_path == entry.module.path
            });
            let version = versions.next()?;
            if versions.next().is_some()
                || (version.origin != SourceOrigin::Disk
                    && self
                        .captured
                        .is_none_or(|capture| capture.native_sources().is_none()))
            {
                return None;
            }
            let package = build_plan
                .packages
                .iter()
                .find(|package| package.package_id == entry.module.package)?;
            let PackagePlanSource::Workspace {
                root,
                source_root: plan_source_root,
                ..
            } = &package.source
            else {
                return None;
            };
            let package_root = workspace.join(root).canonicalize().ok()?;
            if !package_root.starts_with(&workspace)
                || source_root != plan_source_root
                || workspace.join(source_root) != workspace.join(root).join("src")
                || entry.package_id != version.package_id
                || entry.module_path != version.module_path
                || entry.edition != version.edition
                || package.edition != version.edition
                || normalized_path != version.normalized_path.as_str()
                || normalized_path != &format!("src/{source_relative_path}")
            {
                return None;
            }
            let loaded = if self
                .captured
                .is_some_and(|capture| capture.native_sources().is_some())
            {
                self.native_source(version)?.0
            } else {
                FrontendSourceLoader::new(DiskSourceLoader::new(package_root))
                    .load_source_unit(
                        SourceUnitRequest {
                            snapshot: snapshot.id,
                            input: SourceInput {
                                package_id: version.package_id.clone(),
                                module_path: version.module_path.clone(),
                                normalized_path: version.normalized_path.clone(),
                                edition: version.edition.clone(),
                                origin: SourceOriginInput::Disk {
                                    path: normalized_path.into(),
                                },
                            },
                        },
                        &local_ids,
                    )
                    .ok()?
            };
            if loaded.package_id != version.package_id
                || loaded.module_path != version.module_path
                || loaded.normalized_path != version.normalized_path
                || loaded.edition != version.edition
                || loaded.source_hash != version.source_hash
                || loaded.origin != version.origin
            {
                return None;
            }
            let mut bridge = SpanBridge::new();
            register_source_unit(&mut bridge, &loaded).ok()?;
            let preprocessed = preprocess(&loaded, &mut bridge).ok()?;
            if !preprocessed.diagnostics.is_empty() {
                return None;
            }
            let candidates =
                ImportPathCandidate::from_frontend_imports(&LexicalEnvironmentRequest {
                    source_id: loaded.source_id,
                    import_stubs: &preprocessed.import_stubs,
                    edition: loaded.edition.clone(),
                })?;
            let resolution = ImportPathResolver::new(index)
                .resolve(&index.resolver_module_id(&entry.module), &candidates);
            if !resolution.unresolved().is_empty() {
                return None;
            }
            for import in resolution.resolved() {
                let target = ModuleId::new(
                    import.target().package().clone(),
                    import.target().path().clone(),
                );
                let target_entry = module_index
                    .modules
                    .iter()
                    .find(|candidate| candidate.module == target)?;
                match &target_entry.location {
                    ModuleIndexLocation::WorkspaceFile { .. } => {}
                    ModuleIndexLocation::DependencySummary { .. } => continue,
                    _ => return None,
                }
                let edge = ModuleDependencyEdge::new(
                    entry.module.clone(),
                    target,
                    ModuleDependencyKind::ImportSummary,
                );
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
            }
        }
        if source_count != snapshot.source_versions.len() {
            return None;
        }
        edges.sort_by(|a, b| {
            (
                a.dependent.package.as_str(),
                a.dependent.path.as_str(),
                a.dependency.package.as_str(),
                a.dependency.path.as_str(),
            )
                .cmp(&(
                    b.dependent.package.as_str(),
                    b.dependent.path.as_str(),
                    b.dependency.package.as_str(),
                    b.dependency.path.as_str(),
                ))
        });
        Some(mizar_build::task_graph::ModuleDependencyOverlay {
            coverage: ModuleDependencyCoverage::ImportsOnly,
            edges,
        })
    }

    /// Resolves real dependency lexical summaries under explicit caller-owned roots.
    /// Requires build-validated indexes and stubs from the captured source version.
    pub fn dependency_lexical_provider(
        self,
        artifact_roots: &'a [(mizar_session::PackageId, PathBuf)],
    ) -> impl mizar_frontend::lexical_env::LexicalSummaryProvider + 'a {
        DependencyLexicalProvider {
            inputs: self,
            artifact_roots,
            workspace_summaries: &[],
        }
    }
}
pub(super) struct DependencyLexicalProvider<'a> {
    pub(super) inputs: crate::registry::SourceLoadInputs<'a>,
    pub(super) artifact_roots: &'a [(mizar_session::PackageId, PathBuf)],
    pub(super) workspace_summaries: &'a [(
        mizar_resolve::module_index::IndexedModuleId,
        ModuleLexicalSummary,
    )],
}

impl mizar_frontend::lexical_env::LexicalSummaryProvider for DependencyLexicalProvider<'_> {
    fn resolve_imports(
        &self,
        request: &mizar_frontend::lexical_env::LexicalEnvironmentRequest<'_>,
    ) -> Result<
        mizar_frontend::lexical_env::ResolvedImports,
        mizar_frontend::lexical_env::FrontendLexicalEnvironmentError,
    > {
        use mizar_frontend::lexical_env::{
            FrontendLexicalEnvironmentError, ModuleId, ResolvedImport, ResolvedImportEntry,
            ResolvedImports,
        };
        use mizar_resolve::{
            imports::{ImportPathCandidate, ImportPathResolver},
            module_index::{IndexedModuleId, ModuleIndexInput},
            module_summary_reuse::{ModuleSummaryReuse, ModuleSummaryReuseRequest},
        };
        let unavailable = || FrontendLexicalEnvironmentError::ProviderUnavailable {
            message: "dependency lexical provider binding or payload unavailable".to_owned(),
        };
        let version = unique(
            self.inputs
                .snapshot
                .source_versions
                .iter()
                .filter(|version| version.source_id == request.source_id),
        )
        .ok_or_else(unavailable)?;
        let current = unique(self.inputs.module_index.modules.iter().filter(|entry| {
            entry.module.package == version.package_id && entry.module.path == version.module_path
        }))
        .ok_or_else(unavailable)?;
        if version.edition != request.edition
            || current.edition != request.edition
            || current.package_id != version.package_id
            || current.module_path != version.module_path
        {
            return Err(unavailable());
        }
        let index = ModuleIndexInput::new(self.inputs.module_index);
        let candidates =
            ImportPathCandidate::from_frontend_imports(request).ok_or_else(unavailable)?;
        let resolution = ImportPathResolver::new(index)
            .resolve(&index.resolver_module_id(&current.module), &candidates);
        let mut result = ResolvedImports {
            imports: Vec::new(),
            summaries: Vec::new(),
            diagnostics: Vec::new(),
        };
        for import in resolution.resolved() {
            let module = IndexedModuleId::new(
                import.target().package().clone(),
                import.target().path().clone(),
            );
            let entry = unique(
                self.inputs
                    .module_index
                    .modules
                    .iter()
                    .filter(|entry| entry.module == module),
            )
            .ok_or_else(unavailable)?;
            if let ModuleIndexLocation::WorkspaceFile { .. } = &entry.location {
                let (_, summary) = unique(
                    self.workspace_summaries
                        .iter()
                        .filter(|(target, _)| target == &module),
                )
                .ok_or_else(unavailable)?;
                if entry.package_id != module.package || entry.module_path != module.path {
                    return Err(unavailable());
                }
                let module_id = summary.module_id.clone();
                result.summaries.push(summary.clone());
                result.imports.push(ResolvedImportEntry {
                    stub_ordinal: import.ordinal(),
                    stub_span: import.range(),
                    import: ResolvedImport { module_id },
                });
                continue;
            }
            let ModuleIndexLocation::DependencySummary {
                artifact,
                content_hash,
            } = &entry.location
            else {
                return Err(unavailable());
            };
            let reference = unique(
                self.inputs
                    .module_index
                    .dependency_summaries
                    .iter()
                    .filter(|reference| reference.module == module),
            )
            .ok_or_else(unavailable)?;
            if entry.package_id != module.package
                || entry.module_path != module.path
                || artifact != &reference.artifact
                || content_hash != &reference.content_hash
            {
                return Err(unavailable());
            }
            let (_, root) = unique(
                self.artifact_roots
                    .iter()
                    .filter(|(package, _)| package == &module.package),
            )
            .ok_or_else(unavailable)?;
            let module_id = if let Some(value) = reference.read_current_summary(root) {
                let summary = ModuleSummaryReuse::new(index)
                    .read_lexical_summary(
                        ModuleSummaryReuseRequest::new(
                            &module,
                            mizar_session::SourceAnchor::Range(import.range()),
                        ),
                        &value,
                    )
                    .ok_or_else(unavailable)?;
                let id = summary.module_id.clone();
                result.summaries.push(summary);
                id
            } else {
                ModuleId::new(format!(
                    "{:?}:{:?}",
                    module.package.as_str(),
                    module.path.as_str()
                ))
            };
            result.imports.push(ResolvedImportEntry {
                stub_ordinal: import.ordinal(),
                stub_span: import.range(),
                import: ResolvedImport { module_id },
            });
        }
        Ok(result)
    }
}

pub(crate) struct SourceLoadService;

impl SourceLoadService {
    // Only the opened executing artifact and OS executable mappings confer image provenance.
    pub(crate) fn running_native_image()
    -> Result<&'static mizar_cache::cache_key::CompatibilityField, String> {
        static IMAGE: std::sync::OnceLock<mizar_cache::cache_key::CompatibilityField> =
            std::sync::OnceLock::new();
        if let Some(image) = IMAGE.get() {
            return Ok(image);
        }
        #[cfg(target_os = "linux")]
        {
            use mizar_cache::{cache_key::CacheKeyBuilder, cache_store::CacheStoreRoot};
            use mizar_frontend::source::SourceUnit;
            use mizar_ir::cache_adapter::IrCacheAdapter;
            use std::{io::Read, os::unix::fs::MetadataExt};
            let unavailable = || "running native image provenance is unavailable".to_owned();
            let mut executable =
                std::fs::File::open("/proc/self/exe").map_err(|_| unavailable())?;
            let metadata = executable.metadata().map_err(|_| unavailable())?;
            if !metadata.is_file() || metadata.ino() == 0 || metadata.len() == 0 {
                return Err(unavailable());
            }
            let device = metadata.dev();
            let major = ((device >> 8) & 0xfff) | ((device >> 32) & 0xfffff000);
            let minor = (device & 0xff) | ((device >> 12) & 0xffffff00);
            let mappings = std::fs::read_to_string("/proc/self/maps").map_err(|_| unavailable())?;
            let anchors = [
                Self::running_native_image as *const () as usize,
                crate::registry::assemble_registry as *const () as usize,
                <crate::frontend_adapter::SourceLoadService as PhaseService>::execute as *const ()
                    as usize,
                <FrontendService as PhaseService>::execute as *const () as usize,
                FrontendService::analyze_loaded as *const () as usize,
                FrontendService::authenticated_workspace_summaries as *const () as usize,
                <DependencyLexicalProvider<'static> as mizar_frontend::lexical_env::LexicalSummaryProvider>::resolve_imports as *const () as usize,
                mizar_resolve::imports::ImportPathResolver::resolve_frontend as *const () as usize,
                mizar_resolve::imports::ImportPathResolver::resolve as *const () as usize,
                mizar_resolve::declarations::DeclarationShellCollector::collect as *const () as usize,
                mizar_resolve::symbols::SignatureProjectionExtractor::collect as *const () as usize,
                mizar_resolve::symbols::SymbolCollectionResult::export_frontend_lexical_contributions as *const () as usize,
                mizar_artifact::module_summary::ModuleSummaryIdentity::canonical_json as *const () as usize,
                mizar_artifact::store::canonical_json_string as *const () as usize,
                ModuleLexicalSummary::from_exported_symbols as *const () as usize,
                ExportedSymbolShape::canonical_bytes as *const () as usize,
                ExportedSymbolShape::from_canonical_bytes as *const () as usize,
                mizar_frontend::preprocess::PreprocessedSource::canonical_bytes as *const () as usize,
                mizar_frontend::lexing::TokenStream::canonical_bytes as *const () as usize,
                Frontend::<
                    FrontendSourceLoader<DiskSourceLoader>,
                    DependencyLexicalProvider<'static>,
                    MizarParserSeam,
                >::run_loaded as *const () as usize,
                <MizarParserSeam as ParserSeam>::parse as *const () as usize,
                mizar_parser::parse as *const () as usize,
                mizar_frontend::preprocess::preprocess as *const () as usize,
                mizar_frontend::lexing::tokenize as *const () as usize,
                FrontendOutput::<<MizarParserSeam as ParserSeam>::Ast>::canonical_disk_bytes
                    as *const () as usize,
                FrontendOutput::<<MizarParserSeam as ParserSeam>::Ast>::from_canonical_storage_bytes
                    as *const () as usize,
                mizar_frontend::cache_key::FrontendCacheKeys::canonical_bytes as *const () as usize,
                mizar_frontend::cache_key::FrontendCacheKeys::from_canonical_bytes as *const ()
                    as usize,
                IrCacheAdapter::encode::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>
                    as *const () as usize,
                IrCacheAdapter::rehydrate_with_expected_payload::<
                    FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
                > as *const () as usize,
                SourceUnit::canonical_disk_bytes as *const () as usize,
                SourceUnit::from_canonical_storage_bytes as *const () as usize,
                CacheKeyBuilder::build as *const () as usize,
                CacheStoreRoot::lookup as *const () as usize,
                CacheStoreRoot::insert as *const () as usize,
                IrCacheAdapter::new as *const () as usize,
                IrCacheAdapter::encode::<SourceUnit> as *const () as usize,
                IrCacheAdapter::rehydrate_with_expected_payload::<SourceUnit> as *const () as usize,
                mizar_proof::policy::VerifierPolicy::policy_fingerprint as *const () as usize,
                mizar_build::planner::workspace_config_content_hash as *const () as usize,
                mizar_session::source::normalize_path as *const () as usize,
            ];
            let mut found = vec![false; anchors.len()];
            for line in mappings.lines() {
                let mut fields = line.split_whitespace();
                let Some(range) = fields.next() else { continue };
                let Some(permissions) = fields.next() else {
                    continue;
                };
                let _ = fields.next(); // file offset
                let Some(device) = fields.next() else {
                    continue;
                };
                let Some(inode) = fields.next() else { continue };
                if !permissions.contains('x') || inode.parse::<u64>().ok() != Some(metadata.ino()) {
                    continue;
                }
                let Some((map_major, map_minor)) = device.split_once(':') else {
                    continue;
                };
                if u64::from_str_radix(map_major, 16).ok() != Some(major)
                    || u64::from_str_radix(map_minor, 16).ok() != Some(minor)
                {
                    continue;
                }
                let Some((start, end)) = range.split_once('-') else {
                    continue;
                };
                let (Ok(start), Ok(end)) = (
                    usize::from_str_radix(start, 16),
                    usize::from_str_radix(end, 16),
                ) else {
                    continue;
                };
                for (address, present) in anchors.iter().zip(&mut found) {
                    *present |= (start..end).contains(address);
                }
            }
            if !found.into_iter().all(|present| present) {
                return Err(unavailable());
            }
            let mut hasher = blake3::Hasher::new_derive_key("mizar-driver/running-native-image/v1");
            let mut buffer = [0; 64 * 1024];
            let mut length = 0;
            loop {
                let read = executable.read(&mut buffer).map_err(|_| unavailable())?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
                length += read as u64;
            }
            if length != metadata.len() {
                return Err(unavailable());
            }
            let image = mizar_cache::cache_key::CompatibilityField {
                family: "mizar-driver/running-native-image/v1".to_owned(),
                field_name: "identity".to_owned(),
                value: hasher.finalize().to_hex().to_string(),
            };
            let _ = IMAGE.set(image);
            Ok(IMAGE
                .get()
                .expect("successful running image acquisition initializes identity"))
        }
        #[cfg(not(target_os = "linux"))]
        Err("running native image provenance is unsupported on this platform".to_owned())
    }

    pub(crate) fn failure_draft(
        snapshot: mizar_session::BuildSnapshotId,
        source_input: &SourceInput,
        error: &SourceLoadError,
    ) -> Option<DiagnosticDraft> {
        let (number, name) = match error {
            SourceLoadError::InvalidUtf8 { .. } => (601, "invalid_utf8"),
            SourceLoadError::UnreadableSourceFile { .. } => (602, "unreadable_file"),
            SourceLoadError::SourcePathOutsidePackageRoot { .. } => (603, "outside_package_root"),
            SourceLoadError::UnsupportedFileExtension { .. } => (600, "unsupported_file_extension"),
            SourceLoadError::DuplicateModulePath { .. } => (600, "duplicate_module_path"),
            SourceLoadError::StaleLspDocumentVersion { .. } => (600, "stale_lsp_document_version"),
            SourceLoadError::UnmappedOpenBufferUri { .. } => (600, "unmapped_open_buffer_uri"),
            SourceLoadError::GeneratedSourceWithoutMetadata { .. } => {
                (600, "generated_source_without_metadata")
            }
            SourceLoadError::SourceIdAllocation { .. } => (600, "source_id_allocation"),
            SourceLoadError::UnsupportedSourceOrigin { .. } => (600, "unsupported_source_origin"),
            SourceLoadError::InvalidSourcePath { .. } => (600, "invalid_source_path"),
            _ => return None,
        };
        let detail = format!("source.{name}");
        let code = DiagnosticCode::from_parts(DiagnosticSeverity::Error, number)
            .expect("source-load diagnostic code is valid");
        DiagnosticDraft::new(DiagnosticDraftInput {
            source_snapshot: snapshot,
            code,
            phase: DiagnosticPhase::SourceLoad,
            category: FailureCategory::SourceLoadError,
            stable_detail_key: detail,
            message: error.to_string(),
            primary_location: DiagnosticPrimaryLocation::SourceLoad {
                package_id: source_input.package_id.clone(),
                path: source_input.normalized_path.clone(),
            },
            secondary_spans: Vec::new(),
            notes: Vec::new(),
            details: DiagnosticDetails::new(),
            fixes: Vec::new(),
            explanation: None,
        })
        .ok()
    }
}

impl PhaseService for SourceLoadService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceLoad",
            PhaseOwner::MizarFrontend,
            vec![PipelinePhase::SourceLoad],
            "mizar-frontend/source-unit-publication/v1",
            "SourceUnit",
        )
        .expect("source-load descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "current disk source maps require a real source load".to_owned(),
        }
    }

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext<'_>) -> PhaseResult {
        let snapshot = input.snapshot;
        let Some(mut diagnostics) = context.diagnostics else {
            return blocking();
        };
        if diagnostics.is_sealed()
            || !diagnostics.drafts().is_empty()
            || diagnostics.scope().phase() != DiagnosticPhase::SourceLoad
            || diagnostics.scope().source_snapshot() != snapshot
        {
            return blocking();
        }
        if let Some(token) = context.cancellation {
            return if token.snapshot == snapshot {
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
        if publisher.validate_current_snapshot(snapshot).is_err() {
            return blocking();
        }
        let Some(source_inputs) = context.source_load else {
            return blocking();
        };
        if source_inputs.snapshot.id != snapshot
            || source_inputs.snapshot.workspace_root != source_inputs.build_plan.workspace_root
        {
            return blocking();
        }
        if !input.parent_outputs().is_empty() || !input.identities().dependency_hashes().is_empty()
        {
            return blocking();
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return blocking();
        };
        let Some(version) = unique(source_inputs.snapshot.source_versions.iter().filter(
            |version| version.package_id == module.package && version.module_path == module.path,
        )) else {
            return blocking();
        };
        if !matches!(version.origin, SourceOrigin::Disk)
            && source_inputs
                .captured
                .is_none_or(|capture| capture.native_sources().is_none())
        {
            return blocking();
        }
        let Some(package) = unique(
            source_inputs
                .build_plan
                .packages
                .iter()
                .filter(|package| package.package_id == module.package),
        ) else {
            return blocking();
        };
        let PackagePlanSource::Workspace {
            root,
            source_root,
            manifest_path,
        } = &package.source
        else {
            return blocking();
        };
        let Some(index_package) = unique(
            source_inputs
                .module_index
                .packages
                .iter()
                .filter(|entry| entry.package_id == module.package),
        ) else {
            return blocking();
        };
        let mizar_build::module_index::PackageIndexSource::Workspace {
            package_root: index_root,
            source_root: index_source_root,
            manifest_path: index_manifest,
        } = &index_package.source
        else {
            return blocking();
        };
        if root != index_root
            || source_root != index_source_root
            || manifest_path != index_manifest
            || package.edition != version.edition
            || index_package.edition != version.edition
            || !safe_relative(root)
            || !safe_relative(source_root)
            || !safe_relative(manifest_path)
        {
            return blocking();
        }
        let Some(entry) = unique(
            source_inputs
                .module_index
                .modules
                .iter()
                .filter(|entry| entry.module == *module),
        ) else {
            return blocking();
        };
        let ModuleIndexLocation::WorkspaceFile {
            source_root: entry_source_root,
            normalized_path,
            source_relative_path,
        } = &entry.location
        else {
            return blocking();
        };
        if entry.package_id != module.package
            || entry.module_path != module.path
            || entry.edition != version.edition
            || entry_source_root != source_root
            || normalized_path != version.normalized_path.as_str()
            || normalized_path != &format!("src/{source_relative_path}")
            || source_relative_path.is_empty()
        {
            return blocking();
        }
        let workspace = PathBuf::from(source_inputs.snapshot.workspace_root.as_str());
        let package_root = workspace.join(root);
        let source_root_path = workspace.join(source_root);
        if source_root_path != package_root.join("src")
            || workspace.join(manifest_path) != package_root.join("mizar.pkg")
        {
            return blocking();
        }

        let source_key = source_input_hash(version);
        if input.identities().input_hash() != source_key {
            return blocking();
        }
        let mut source_input = SourceInput {
            package_id: version.package_id.clone(),
            module_path: version.module_path.clone(),
            normalized_path: version.normalized_path.clone(),
            edition: version.edition.clone(),
            origin: SourceOriginInput::Disk {
                path: version.normalized_path.as_str().into(),
            },
        };
        let bound_root = match (workspace.canonicalize(), package_root.canonicalize()) {
            (Ok(workspace), Ok(root)) if root.starts_with(&workspace) => Some(root),
            (Ok(_), Ok(_)) => return blocking(),
            _ => None,
        };
        let loader = FrontendSourceLoader::new(DiskSourceLoader::new(
            bound_root.as_ref().unwrap_or(&package_root),
        ));
        let loaded = if source_inputs
            .captured
            .is_some_and(|capture| capture.native_sources().is_some())
        {
            let Some((source, input)) = source_inputs.native_source(version) else {
                return blocking();
            };
            source_input = input;
            source
        } else {
            match loader.load_source_unit(
                SourceUnitRequest {
                    snapshot,
                    input: source_input.clone(),
                },
                source_inputs.allocator,
            ) {
                Ok(loaded) => loaded,
                Err(error) => {
                    let Some(draft) = Self::failure_draft(snapshot, &source_input, &error) else {
                        return blocking();
                    };
                    if diagnostics.emit(draft).is_err() {
                        return blocking();
                    }
                    return PhaseResult {
                        status: PhaseStatus::Fatal,
                        diagnostics: vec![diagnostics.seal()],
                        ..blocking()
                    };
                }
            }
        };
        if bound_root.is_none()
            || loaded.package_id != version.package_id
            || loaded.module_path != version.module_path
            || loaded.normalized_path != version.normalized_path
            || loaded.edition != version.edition
            || loaded.source_hash != version.source_hash
            || loaded.origin != version.origin
        {
            return blocking();
        }
        let Some(storage_bytes) = (match &source_input.origin {
            SourceOriginInput::OpenBuffer { .. } => {
                loaded.canonical_open_buffer_bytes(&source_input)
            }
            _ => loaded.canonical_disk_bytes(),
        }) else {
            return blocking();
        };
        let Some(source) = SourceUnit::from_canonical_storage_bytes(
            &storage_bytes,
            version.source_id,
            &source_input,
        ) else {
            return blocking();
        };
        if !source.matches_canonical_storage(&loaded, &source_input) {
            return blocking();
        }
        let semantic_bytes = source_key.as_bytes().to_vec();
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-frontend/source-unit-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::from(PipelinePhase::SourceLoad);
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            source.package_id.as_str(),
            source.module_path.as_str(),
        ));
        let output_kind = OutputKind::new("SourceUnit");
        let decode_input = source_input;
        let source_id = source.source_id;
        let expected_source = source.clone();
        let (origin, target) = source_publication_route(&version.origin);
        let handle = publisher.publish(PublishOutputInput {
            slot: publisher.allocate(
                snapshot,
                phase.clone(),
                work_unit.clone(),
                output_kind.clone(),
                SchemaVersion::new(1),
            ),
            snapshot,
            phase,
            work_unit,
            output_kind,
            schema_version: SchemaVersion::new(1),
            payload: source,
            canonical_payload: Some(semantic_bytes),
            storage_payload: Some(storage_bytes),
            decode: BlobDecoder::new(move |bytes| {
                SourceUnit::from_canonical_storage_bytes(bytes, source_id, &decode_input)
                    .filter(|source| {
                        source.matches_canonical_storage(&expected_source, &decode_input)
                    })
                    .ok_or_else(|| BlobDecodeError::new("invalid source-unit disk payload"))
            }),
            parents: Vec::new(),
            named_input_hashes: vec![NamedInputHash {
                name: "source".to_owned(),
                domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                digest: source_key,
            }],
            side_tables: IrSideTables {
                source_maps: vec![SideTableRecord::new(
                    "source-storage",
                    version.normalized_path.as_str(),
                    storage_hash,
                )],
                ..IrSideTables::default()
            },
            origin,
            target,
        });
        match handle {
            Ok(handle) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Err(_) => blocking(),
        }
    }
}

#[cfg(test)]
pub(crate) mod imported_native_frontend_cache_tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;
    use crate::{
        driver::{
            CompilerDriver, DriverSubmitInput, NativeCacheMode, NativeCacheOptions, SubmitOptions,
        },
        registry::{PhaseInput, SourceLoadInputs, StageConfig, assemble_registry},
        request::{
            BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
            BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
            SourceInputSet, VerifierConfigInput,
        },
    };
    use mizar_build::{
        module_index::{ModuleId, StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage},
        planner::{
            DependencySelection, PlanRequest, WorkspacePackage, lockfile_content_hash,
            parse_lockfile, parse_package_manifest, produce_build_plan,
            workspace_config_content_hash,
        },
        scheduler::CacheSchedulingPolicy,
        task_graph::{
            ModuleDependencyEdge, ModuleDependencyKind, ModuleDependencyOverlay, TaskKind, WorkUnit,
        },
    };
    use mizar_ir::{
        dispatch_input::{PhaseDispatchInputBundle, SealedParentOutputHandle},
        identity::{
            OutputKind, PipelinePhase as IrPhase, SnapshotHandleRegistry, WorkUnit as IrUnit,
        },
        publisher::{AllowedWorkUnit, PhaseOutputPublisher},
        storage::{CollectInput, IrStorageService, StoragePolicy},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SnapshotRegistry, SourceInput, SourceOriginInput, ToolchainInfo, WorkspaceRoot,
        normalize_path,
    };

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    pub(crate) struct ImportedFixture {
        pub(crate) root: PathBuf,
    }

    impl ImportedFixture {
        pub(crate) fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "mizar-driver-imported-frontend-unit-{}-{}",
                std::process::id(),
                NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
            ));
            fs::create_dir_all(root.join("alpha/src")).unwrap();
            fs::write(
                root.join("alpha/src/main.miz"),
                b"import alpha.leaf;\nimport alpha.leaf;\nimport alpha.twig;\ndefinition\nend;\n",
            )
            .unwrap();
            fs::write(root.join("alpha/src/leaf.miz"), b"definition\nend;\n").unwrap();
            fs::write(root.join("alpha/src/twig.miz"), b"definition\nend;\n").unwrap();
            Self { root }
        }

        pub(crate) fn capture(
            &self,
            ids: &InMemorySessionIdAllocator,
            snapshots: &SnapshotRegistry<InMemorySessionIdAllocator>,
        ) -> (
            crate::request::BuildSession,
            DriverSubmitInput<StaticSourceLayout>,
        ) {
            let root = WorkspaceRoot::new(self.root.to_string_lossy().into_owned());
            let toolchain = ToolchainInfo::new("mizar-evo-test");
            let workspace_packages = vec![WorkspacePackage {
                member_path: "alpha".to_owned(),
                manifest: parse_package_manifest(
                    "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
                )
                .unwrap(),
            }];
            let lockfile = parse_lockfile(
                "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n",
            )
            .unwrap();
            let plan_request = PlanRequest {
                workspace_root: root.clone(),
                dependency_selection: DependencySelection::Normal,
                toolchain: toolchain.clone(),
            };
            let mut input = DriverSubmitInput::new(
                plan_request.clone(),
                workspace_packages.clone(),
                lockfile.clone(),
                StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                    package_id: PackageId::new("alpha"),
                    files: ["leaf", "main", "twig"]
                        .into_iter()
                        .map(|module| {
                            WorkspaceSourceFile::new(
                                format!("src/{module}.miz"),
                                format!("{module}.miz"),
                            )
                        })
                        .collect(),
                }]),
            );
            input.dependency_overlay = ModuleDependencyOverlay::complete(vec![
                ModuleDependencyEdge::new(
                    ModuleId::new(PackageId::new("alpha"), ModulePath::new("main")),
                    ModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf")),
                    ModuleDependencyKind::ImportSummary,
                ),
                ModuleDependencyEdge::new(
                    ModuleId::new(PackageId::new("alpha"), ModulePath::new("main")),
                    ModuleId::new(PackageId::new("alpha"), ModulePath::new("twig")),
                    ModuleDependencyKind::ImportSummary,
                ),
            ]);
            input.cache_policy = CacheSchedulingPolicy::Enabled;
            let plan =
                produce_build_plan(plan_request.clone(), workspace_packages, lockfile.clone())
                    .unwrap();
            let draft = BuildRequestDraft {
                lane: BuildLaneId::new(1),
                origin: BuildRequestOrigin::Batch(BatchRequest {
                    invocation: BatchInvocation::default(),
                }),
                generation: BuildRequestGeneration::new(0),
                workspace_root: root,
                profile: BuildProfile::new("check"),
                targets: BuildTargets::default(),
                source_inputs: SourceInputSet { versions: vec![] },
                dependency_inputs: DependencyInputSet::new(
                    vec![],
                    lockfile_content_hash(&plan.lockfile),
                    toolchain,
                ),
                verifier_config: VerifierConfigInput::new(workspace_config_content_hash(
                    &plan.verifier_config,
                    &plan.build_config,
                )),
            };
            let descriptors = ["leaf", "main", "twig"]
                .into_iter()
                .map(|module| {
                    let path = format!("src/{module}.miz");
                    (
                        DiskSourceLoader::new(self.root.join("alpha")),
                        SourceInput {
                            package_id: PackageId::new("alpha"),
                            module_path: ModulePath::new(module),
                            normalized_path: normalize_path(
                                &self.root.join("alpha"),
                                Path::new(&path),
                            )
                            .unwrap(),
                            edition: Edition::new("2025"),
                            origin: SourceOriginInput::Disk { path: path.into() },
                        },
                    )
                })
                .collect();
            (
                draft.capture_sources(ids, snapshots, descriptors).unwrap(),
                input,
            )
        }
    }

    impl Drop for ImportedFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn allow(publisher: &PhaseOutputPublisher, unit: &WorkUnit) {
        let WorkUnit::Module { module } = unit else {
            unreachable!();
        };
        for (phase, kind) in [("SourceLoad", "SourceUnit"), ("Frontend", "FrontendOutput")] {
            publisher.allow_work_unit(AllowedWorkUnit::new(
                IrPhase::new(phase),
                OutputKind::new(kind),
                IrUnit::new(format!(
                    "{:?}:{:?}",
                    module.package.as_str(),
                    module.path.as_str()
                )),
            ));
        }
    }

    #[derive(Clone, Copy)]
    enum ImportedAttack {
        WrongSource,
        SwappedPhase,
        CollectedSource,
        MissingLeafSource,
        SurplusLeafSource,
        DuplicateLeafSource,
        MissingLeafFrontend,
        DuplicateLeafFrontend,
        SwappedLeafLineage,
        CoherentPayloadCorruption,
    }

    fn run_imported_cache_case(attack: ImportedAttack) {
        use mizar_cache::{
            cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
            cache_store::{CacheLookupOutcome, CacheStoreRoot},
        };
        use mizar_ir::cache_adapter::{CacheRehydrateOutcome, IrCacheAdapter};

        let fixture = ImportedFixture::new();
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (session, input) = fixture.capture(&ids, &snapshots);
        let snapshot = session.captured().snapshot().id;
        let policy = mizar_proof::policy::VerifierPolicy::release();
        let store = CacheStoreRoot::new(fixture.root.join("alpha/.mizar-cache"));
        let mut config = StageConfig::frontend(vec![]);
        config.verifier_policy = policy.clone();
        let registry = assemble_registry(&config).unwrap();
        let importer = WorkUnit::Module {
            module: ModuleId::new(PackageId::new("alpha"), ModulePath::new("main")),
        };
        let owner = std::sync::Arc::new(PhaseOutputPublisher::new(
            std::sync::Arc::new(IrStorageService::with_policy(StoragePolicy::default())),
            std::sync::Arc::new(SnapshotHandleRegistry::new()),
        ));
        owner.register_current_snapshot(snapshot);
        for module in ["leaf", "main", "twig"] {
            allow(
                &owner,
                &WorkUnit::Module {
                    module: ModuleId::new(PackageId::new("alpha"), ModulePath::new(module)),
                },
            );
        }
        let mut driver = CompilerDriver::new(registry).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(
                session,
                &ids,
                &snapshots,
                input,
                SubmitOptions {
                    native_cache: Some(NativeCacheOptions {
                        policy: &policy,
                        store: &store,
                        mode: NativeCacheMode::PersistentPrefix,
                    }),
                    ..SubmitOptions::default()
                },
            )
            .unwrap();
        let captured = submission.session.captured();
        let source_inputs = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &ids,
        };
        let run = submission.scheduler_run.as_ref().unwrap();
        let graph = submission.task_graph.as_ref().unwrap();
        let output = |kind: TaskKind, module: &str| {
            let task = graph
                .tasks()
                .iter()
                .find(|task| {
                    task.kind == kind
                        && matches!(&task.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)
                })
                .unwrap();
            run.phase_results[&task.id][0].output_refs[0].clone()
        };
        let main_source = output(TaskKind::SourceLoad, "main");
        let leaf_source = output(TaskKind::SourceLoad, "leaf");
        let twig_source = output(TaskKind::SourceLoad, "twig");
        let leaf_frontend = output(TaskKind::Frontend, "leaf");
        let twig_frontend = output(TaskKind::Frontend, "twig");
        let importer_frontend = output(TaskKind::Frontend, "main");
        let main_version = captured
            .snapshot()
            .source_versions
            .iter()
            .find(|version| version.module_path == ModulePath::new("main"))
            .unwrap();
        let make_input = |outputs: Vec<mizar_ir::storage::AnyPhaseOutputRef>| {
            let parents = outputs
                .into_iter()
                .map(|parent| {
                    SealedParentOutputHandle::from_published_output(&owner, snapshot, parent)
                        .unwrap()
                })
                .collect();
            PhaseInput::new(
                importer.clone(),
                PhaseDispatchInputBundle::new(
                    snapshot,
                    source_input_hash(main_version),
                    vec![],
                    parents,
                )
                .unwrap(),
            )
        };
        let input = make_input(vec![
            main_source.clone(),
            leaf_frontend.clone(),
            twig_frontend.clone(),
        ]);
        let adapter = IrCacheAdapter::new(owner.clone());
        let frontend = driver.registry().native_frontend_service().unwrap();
        let handle = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                &importer_frontend,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let expected_bytes = owner.storage().get(&handle).unwrap().canonical_disk_bytes();
        let leaf_sources = [leaf_source.clone(), twig_source.clone()];
        let authenticated = FrontendService::authenticated_workspace_summaries(
            source_inputs,
            &input,
            owner.as_ref(),
            &main_source,
        )
        .expect("genuine importer parents authenticate both direct leaf summaries");
        assert_eq!(authenticated.len(), 2);
        assert_ne!(authenticated[0].0, authenticated[1].0);
        assert_eq!(
            authenticated[0].1.exported_symbols, authenticated[1].1.exported_symbols,
            "equal exported content does not collapse distinct leaf targets"
        );
        let (request, _, _, _) = source_inputs
            .prepare_native_disk_frontend_cache_payload(
                &input,
                &policy,
                &adapter,
                None,
                Some(frontend),
                &leaf_sources,
            )
            .unwrap();
        let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(request).build() else {
            panic!("captured imported Frontend cache key");
        };
        assert!(matches!(store.lookup(&key), CacheLookupOutcome::Hit(_)));
        let (reordered, _, _, _) = source_inputs
            .prepare_native_disk_frontend_cache_payload(
                &make_input(vec![
                    twig_frontend.clone(),
                    main_source.clone(),
                    leaf_frontend.clone(),
                ]),
                &policy,
                &adapter,
                None,
                Some(frontend),
                &[twig_source.clone(), leaf_source.clone()],
            )
            .unwrap();
        assert_eq!(
            CacheKeyBuilder::new(reordered).build(),
            CacheKeyBuildOutcome::Cacheable(key.clone())
        );
        let protected = vec![
            main_source.output(),
            leaf_source.output(),
            twig_source.output(),
            leaf_frontend.output(),
            twig_frontend.output(),
        ];
        assert!(
            owner
                .storage()
                .collect(CollectInput {
                    snapshot,
                    protected_outputs: protected.clone()
                })
                .outputs_dropped
                > 0
        );
        let CacheRehydrateOutcome::Rehydrated(rehydrated) = source_inputs
            .rehydrate_native_disk_frontend_cache_with_dependencies(
                &input,
                &policy,
                &store,
                &adapter,
                frontend,
                &leaf_sources,
            )
            .unwrap()
        else {
            panic!("genuine imported cache record rehydrates against its captured leaf");
        };
        assert_eq!(adapter.successful_rehydrations(), 1);
        assert_eq!(
            owner
                .storage()
                .get(&rehydrated)
                .unwrap()
                .canonical_disk_bytes(),
            expected_bytes
        );

        owner.storage().collect(CollectInput {
            snapshot,
            protected_outputs: protected.clone(),
        });
        let refusal_adapter = IrCacheAdapter::new(owner.clone());
        let (refusal_input, refusal_sources) = match attack {
            ImportedAttack::WrongSource => (
                make_input(vec![
                    main_source.clone(),
                    leaf_frontend.clone(),
                    twig_frontend.clone(),
                ]),
                vec![main_source.clone(), twig_source.clone()],
            ),
            ImportedAttack::SwappedPhase => (
                make_input(vec![
                    main_source.clone(),
                    leaf_frontend.clone(),
                    twig_frontend.clone(),
                ]),
                vec![leaf_frontend.clone(), twig_source.clone()],
            ),
            ImportedAttack::CollectedSource => {
                owner.storage().collect(CollectInput {
                    snapshot,
                    protected_outputs: vec![
                        main_source.output(),
                        leaf_frontend.output(),
                        twig_frontend.output(),
                    ],
                });
                (
                    make_input(vec![
                        main_source.clone(),
                        leaf_frontend.clone(),
                        twig_frontend.clone(),
                    ]),
                    leaf_sources.to_vec(),
                )
            }
            ImportedAttack::MissingLeafSource => (
                make_input(vec![
                    main_source.clone(),
                    leaf_frontend.clone(),
                    twig_frontend.clone(),
                ]),
                vec![leaf_source.clone()],
            ),
            ImportedAttack::SurplusLeafSource => (
                make_input(vec![
                    main_source.clone(),
                    leaf_frontend.clone(),
                    twig_frontend.clone(),
                ]),
                vec![
                    leaf_source.clone(),
                    twig_source.clone(),
                    main_source.clone(),
                ],
            ),
            ImportedAttack::DuplicateLeafSource => (
                make_input(vec![
                    main_source.clone(),
                    leaf_frontend.clone(),
                    twig_frontend.clone(),
                ]),
                vec![
                    leaf_source.clone(),
                    leaf_source.clone(),
                    twig_source.clone(),
                ],
            ),
            ImportedAttack::MissingLeafFrontend => (
                make_input(vec![main_source.clone(), leaf_frontend.clone()]),
                leaf_sources.to_vec(),
            ),
            ImportedAttack::DuplicateLeafFrontend => {
                let parents = [
                    main_source.clone(),
                    leaf_frontend.clone(),
                    leaf_frontend.clone(),
                ]
                .into_iter()
                .map(|output| {
                    SealedParentOutputHandle::from_published_output(&owner, snapshot, output)
                        .unwrap()
                })
                .collect();
                assert!(
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        source_input_hash(main_version),
                        vec![],
                        parents
                    )
                    .is_err()
                );
                assert_eq!(refusal_adapter.successful_rehydrations(), 0);
                return;
            }
            ImportedAttack::SwappedLeafLineage => {
                use mizar_ir::{
                    publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
                    storage::{BlobDecodeError, BlobDecoder},
                };
                let typed = owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        &leaf_frontend,
                        &OutputKind::new("FrontendOutput"),
                    )
                    .unwrap();
                let payload = owner.storage().get(&typed).unwrap();
                let sides = (*owner.storage().side_tables(&typed).unwrap()).clone();
                let descriptor = captured
                    .native_sources()
                    .unwrap()
                    .iter()
                    .find(|(_, _, source)| source.module_path.as_str() == "leaf")
                    .unwrap()
                    .2
                    .clone();
                let source_id = payload.source.source_id;
                let mut semantic = b"mizar-frontend/output-publication/v1\0".to_vec();
                semantic.extend_from_slice(
                    payload
                        .cache_keys
                        .ast
                        .as_ref()
                        .unwrap()
                        .stable_hash()
                        .as_bytes(),
                );
                owner.storage().collect(CollectInput {
                    snapshot,
                    protected_outputs: protected
                        .iter()
                        .copied()
                        .filter(|output| *output != leaf_frontend.output())
                        .collect(),
                });
                let swapped = owner.publish(PublishOutputInput {
                    slot: owner.allocate(
                        snapshot,
                        leaf_frontend.phase().clone(),
                        leaf_frontend.work_unit().clone(),
                        OutputKind::new("FrontendOutput"),
                        SchemaVersion::new(1),
                    ),
                    snapshot,
                    phase: leaf_frontend.phase().clone(),
                    work_unit: leaf_frontend.work_unit().clone(),
                    output_kind: OutputKind::new("FrontendOutput"),
                    schema_version: SchemaVersion::new(1),
                    payload: (*payload).clone(),
                    canonical_payload: Some(semantic),
                    storage_payload: payload.canonical_disk_bytes(),
                    decode: BlobDecoder::new(move |bytes| {
                        FrontendOutput::from_canonical_storage_bytes(bytes, source_id, &descriptor)
                            .ok_or_else(|| BlobDecodeError::new("invalid swapped leaf bytes"))
                    }),
                    parents: vec![twig_source.clone()],
                    named_input_hashes: leaf_frontend.lineage().named_input_hashes.clone(),
                    side_tables: sides,
                    origin: OutputOrigin::PackageSource,
                    target: PublicationTarget::CurrentPackage,
                });
                let swapped = match swapped {
                    Ok(swapped) => swapped,
                    Err(mizar_ir::publisher::PublishError::Identity { .. }) => {
                        assert_eq!(refusal_adapter.successful_rehydrations(), 0);
                        return;
                    }
                    Err(error) => panic!("unexpected swapped-lineage publication failure: {error}"),
                };
                assert!(owner.storage().validate_handle(swapped.any()).is_ok());
                assert!(
                    owner
                        .validate_output(
                            snapshot,
                            swapped.any(),
                            OutputAuthorization::CurrentPackage
                        )
                        .is_ok()
                );
                (
                    make_input(vec![
                        main_source.clone(),
                        swapped.erase(),
                        twig_frontend.clone(),
                    ]),
                    leaf_sources.to_vec(),
                )
            }
            ImportedAttack::CoherentPayloadCorruption => (
                make_input(vec![
                    main_source.clone(),
                    leaf_frontend.clone(),
                    twig_frontend.clone(),
                ]),
                leaf_sources.to_vec(),
            ),
        };
        let corrupt_store = CacheStoreRoot::new(fixture.root.join("alpha/.mizar-corrupt-cache"));
        let refusal_store = if matches!(attack, ImportedAttack::CoherentPayloadCorruption) {
            let CacheLookupOutcome::Hit(record) = store.lookup(&key) else {
                panic!("the genuine multi-leaf record remains available for corruption");
            };
            let mut corrupted = record.output.clone();
            let last = corrupted
                .last_mut()
                .expect("encoded cache payload is nonempty");
            *last ^= 0x01;
            let rewritten = mizar_cache::cache_store::CacheRecord::new_inline(
                key.clone(),
                record.header.produced_by.clone(),
                corrupted,
            );
            corrupt_store.insert(&rewritten).unwrap();
            &corrupt_store
        } else {
            &store
        };
        let refusal = source_inputs.rehydrate_native_disk_frontend_cache_with_dependencies(
            &refusal_input,
            &policy,
            refusal_store,
            &refusal_adapter,
            frontend,
            &refusal_sources,
        );
        assert!(match refusal {
            Err(_) => true,
            Ok(CacheRehydrateOutcome::Miss(_)) => true,
            Ok(_) => false,
        });
        assert_eq!(refusal_adapter.successful_rehydrations(), 0);
        assert!(owner.storage().validate_handle(&importer_frontend).is_err());
    }

    #[test]
    fn imported_cache_refuses_a_wrong_leaf_source_association() {
        run_imported_cache_case(ImportedAttack::WrongSource);
    }

    #[test]
    fn imported_cache_refuses_a_swapped_leaf_phase() {
        run_imported_cache_case(ImportedAttack::SwappedPhase);
    }

    #[test]
    fn imported_cache_refuses_a_collected_leaf_source() {
        run_imported_cache_case(ImportedAttack::CollectedSource);
    }

    #[test]
    fn imported_cache_refuses_missing_leaf_source_associations() {
        run_imported_cache_case(ImportedAttack::MissingLeafSource);
    }

    #[test]
    fn imported_cache_refuses_surplus_leaf_source_associations() {
        run_imported_cache_case(ImportedAttack::SurplusLeafSource);
    }

    #[test]
    fn imported_cache_refuses_duplicate_leaf_source_associations() {
        run_imported_cache_case(ImportedAttack::DuplicateLeafSource);
    }

    #[test]
    fn imported_cache_refuses_missing_leaf_frontend_associations() {
        run_imported_cache_case(ImportedAttack::MissingLeafFrontend);
    }

    #[test]
    fn imported_cache_refuses_duplicate_leaf_frontend_associations() {
        run_imported_cache_case(ImportedAttack::DuplicateLeafFrontend);
    }

    #[test]
    fn imported_cache_refuses_cross_leaf_source_lineage_with_equal_exports() {
        run_imported_cache_case(ImportedAttack::SwappedLeafLineage);
    }

    #[test]
    fn imported_cache_refuses_a_coherently_rehashed_corrupt_payload() {
        run_imported_cache_case(ImportedAttack::CoherentPayloadCorruption);
    }
}
