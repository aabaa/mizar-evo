//! Real disk source publication, native cache request preparation and dependency lexical input.
//! See [the source service contract](../../../../doc/design/mizar-driver/en/frontend_adapter.md).
use std::{path::PathBuf, sync::Arc};

use mizar_build::{
    module_index::ModuleIndexLocation,
    planner::PackagePlanSource,
    task_graph::{PipelinePhase, WorkUnit},
};
use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticNote, DiagnosticNoteKind, DiagnosticPrimaryLocation, DiagnosticSpan,
        DiagnosticSpanRole, FailureCategory, PipelinePhase as DiagnosticPhase, SpanFreshness,
    },
    registry::{DiagnosticCode, DiagnosticSeverity},
};
use mizar_frontend::{
    cache_key::{
        ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION, SOURCE_UNIT_CACHE_KEY_VERSION,
        SourceUnitCacheKey, TOKEN_STREAM_CACHE_KEY_VERSION,
    },
    lexical_env::{
        ExportedSymbolShape, LexicalEnvironmentDiagnosticCode as EnvironmentCode,
        ModuleLexicalSummary,
    },
    lexing::{LexDiagnosticCode, LexingDiagnosticKind, ScopeSkeletonDiagnosticCode},
    orchestration::{
        DiagnosticClass, DiagnosticCode as FrontendCode, DiagnosticLocation, Frontend,
        FrontendDiagnostic, FrontendOutput,
    },
    parsing::{MizarParserSeam, ParserSeam},
    preprocess::{
        ImportPrescanDiagnosticCode, PreprocessDiagnosticKind, SourcePreprocessDiagnosticCode,
    },
    source::{
        FrontendSourceLoader, SourceUnit, SourceUnitLoader, SourceUnitRequest,
        diagnostic_file_path, source_unit_from_loaded,
    },
};
use mizar_ir::{
    identity::{
        NamedInputHash, OutputKind, PipelinePhase as IrPipelinePhase, WorkUnit as IrWorkUnit,
    },
    publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
    storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion, SideTableRecord},
};
use mizar_session::{
    DiskSourceLoader, Hash, PackageId, SourceInput, SourceLoadError, SourceOrigin,
    SourceOriginInput,
};

use crate::registry::{
    PhaseCacheContext, PhaseCacheIntent, PhaseDescriptor, PhaseExecutionContext, PhaseInput,
    PhaseOwner, PhaseResult, PhaseService, PhaseStatus,
};

pub(crate) fn source_input_hash(version: &mizar_session::SourceVersion) -> Hash {
    SourceUnitCacheKey {
        version: Arc::from(SOURCE_UNIT_CACHE_KEY_VERSION),
        package_id: version.package_id.clone(),
        module_path: version.module_path.clone(),
        normalized_path: version.normalized_path.clone(),
        source_hash: version.source_hash,
        edition: version.edition.clone(),
    }
    .stable_hash()
}

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
        Ok(CacheKeyRequest {
            cache_schema_version: CacheSchema::new(CACHE_KEY_SCHEMA_VERSION),
            phase: CachePhase::new(descriptor.service_name),
            work_unit: CacheWorkUnit::new(format!(
                "{:?}:{:?}",
                source.package_id.as_str(),
                source.module_path.as_str()
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
                toolchain_compatibility: vec![CompatibilityField {
                    family: "mizar-session/toolchain".to_owned(),
                    field_name: "identity".to_owned(),
                    value: self.snapshot.toolchain.identity().to_owned(),
                }],
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
            .validate_current_output(input.snapshot, handle.any())
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
        handle: &mizar_ir::storage::PhaseOutputRef<
            FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
        >,
    ) -> Result<(mizar_cache::cache_key::CacheKeyRequest, Vec<u8>, Vec<u8>), String> {
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
            || input.parent_outputs().len() != 1
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
        let parent = input.parent_outputs()[0].as_output_ref();
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_current_output(input.snapshot, parent)
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
        storage
            .validate_handle(handle.any())
            .map_err(|error| error.to_string())?;
        publisher
            .validate_current_output(input.snapshot, handle.any())
            .map_err(|error| error.to_string())?;
        if handle.phase().as_str() != "Frontend"
            || handle.work_unit().as_str() != request.work_unit.as_str()
            || handle.output_kind() != &OutputKind::new("FrontendOutput")
            || handle.schema_version() != SchemaVersion::new(1)
        {
            return Err("native Frontend encoding output binding mismatch".to_owned());
        }
        let output = storage.get(handle).map_err(|error| error.to_string())?;
        if output.source.source_id != version.source_id
            || !output
                .source
                .matches_canonical_storage(&source, &captured_input)
            || output.ast.is_none()
            || !output.diagnostics.is_empty()
            || !output.preprocessed.import_stubs.is_empty()
            || output.cache_keys.source != SourceUnitCacheKey::from_source(&source)
            || output.cache_keys.preprocessed != PreprocessedSourceCacheKey::from_source(&source)
            || handle.lineage().parents != vec![parent.output()]
            || handle.lineage().named_input_hashes
                != vec![
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
                ]
        {
            return Err("native Frontend encoding output source or lineage mismatch".to_owned());
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
        if *storage
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
        Ok((request, canonical_payload, storage_payload))
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

        let (request, canonical_payload, storage_payload) =
            self.prepare_native_disk_frontend_cache_payload(input, policy, adapter, &handle)?;
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

    /// Prepares the checked native leaf profile from a retained current Frontend oracle.
    pub fn prepare_native_disk_frontend_cache_request(
        self,
        input: &PhaseInput,
        policy: &mizar_proof::policy::VerifierPolicy,
        adapter: &mizar_ir::cache_adapter::IrCacheAdapter,
        handle: &mizar_ir::storage::PhaseOutputRef<
            FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
        >,
    ) -> Result<mizar_cache::cache_key::CacheKeyRequest, String> {
        self.prepare_native_disk_frontend_cache_payload(input, policy, adapter, handle)
            .map(|(request, _, _)| request)
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
        use mizar_cache::{
            cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
            cache_store::CacheLookupOutcome,
        };
        use mizar_ir::cache_adapter::RehydrateCacheHitInput;

        let (oracle_input, oracle_adapter, oracle_handle) = oracle;
        let (request, semantic, storage_payload) = self
            .prepare_native_disk_frontend_cache_payload(
                oracle_input,
                policy,
                oracle_adapter,
                oracle_handle,
            )?;
        if input.snapshot != oracle_input.snapshot
            || input.work_unit != oracle_input.work_unit
            || input.identities() != oracle_input.identities()
            || input.parent_outputs().len() != 1
        {
            return Err("native Frontend lookup target binding mismatch".to_owned());
        }
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
        let parent = input.parent_outputs()[0].as_output_ref();
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_current_output(input.snapshot, parent)
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
            || parent.output() != oracle_input.parent_outputs()[0].output()
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
        Ok(adapter.rehydrate_with_expected_payload(
            RehydrateCacheHitInput {
                lookup,
                snapshot: input.snapshot,
                phase: IrPipelinePhase::new(request.phase.as_str()),
                work_unit: IrWorkUnit::new(request.work_unit.as_str()),
                output_kind: OutputKind::new("FrontendOutput"),
                schema_version: SchemaVersion::new(1),
                parents: vec![parent.clone()],
                named_input_hashes: oracle_handle.lineage().named_input_hashes.clone(),
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
        ))
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

    /// Copies original written public theorem formulas from protected native capture.
    pub fn native_public_theorem_formula_text_candidates(
        self,
        module: &mizar_build::module_index::ModuleId,
    ) -> Result<
        Option<std::collections::BTreeMap<mizar_resolve::resolved_ast::SymbolId, String>>,
        String,
    > {
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem text requires one captured source version".to_owned())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem text capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem text requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem text requires a workspace package".to_owned());
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
        if !collection.diagnostics().is_empty() {
            return Ok(None);
        }
        let arena = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &resolver_module)
            .map_err(|error| error.to_string())?;
        let Some(formulas) = mizar_resolve::labels::source_public_theorem_formula_candidates(
            ast,
            &arena,
            collection.env(),
        )?
        else {
            return Ok(None);
        };
        let mut texts = std::collections::BTreeMap::new();
        for (owner, formula) in formulas {
            let Some(node) = ast.node_view(formula) else {
                return Ok(None);
            };
            let range = node.range();
            if range.source_id != output.source.source_id {
                return Ok(None);
            }
            let Some(text) = output.source.source_text.get(range.start..range.end) else {
                return Ok(None);
            };
            texts.insert(owner, text.to_owned());
        }
        Ok(Some(texts))
    }

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
            .validate_published_output(input.snapshot, parent)
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
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
        Ok(result)
    }

    /// Returns genuine Checker metadata for the bound original native theorem source.
    pub fn native_theorem_metadata_with_frontend_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<
        Option<(
            mizar_checker::binding_env::BindingEnv,
            mizar_checker::type_checker::TermFormulaInferenceOutput,
            mizar_checker::typed_ast::TypedAst,
            mizar_checker::resolved_typed_ast::ResolvedTypedAst,
        )>,
        String,
    > {
        let Some(_) =
            self.native_theorem_module_candidates_with_frontend_parent(input, publisher)?
        else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem metadata requires a module".to_owned());
        };
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
            .ok_or_else(|| "native theorem metadata requires retained syntax".to_owned())?;
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let Some((arena, env)) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(
            mizar_resolve::names::SourceVariableScopeInput::new(ast, &resolver_module, &env),
        )
        .map_err(|error| format!("{error:?}"))?;
        let result =
            mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)?;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        Ok(Some(result))
    }

    /// Returns genuine Checker TypedAst bound to a published native theorem summary.
    pub fn native_theorem_typed_ast_with_signature_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<Option<mizar_checker::typed_ast::TypedAst>, String> {
        if input.snapshot != self.snapshot.id {
            return Err("native theorem TypedAst snapshot mismatch".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem TypedAst requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem TypedAst requires one captured source".to_owned())?;
        let source_key = source_input_hash(version);
        if input.identities().input_hash() != source_key
            || !input.identities().dependency_hashes().is_empty()
            || !self.snapshot.dependency_artifacts.is_empty()
            || !self.module_index.dependency_summaries.is_empty()
            || input.parent_outputs().len() != 1
        {
            return Err("native theorem TypedAst requires the leaf input profile".to_owned());
        }
        let parent = input.parent_outputs()[0].as_output_ref();
        let storage = publisher.storage();
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
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
            || parent.phase() != &IrPipelinePhase::new("SignatureCollection")
            || parent.work_unit() != &work_unit
            || parent.output_kind() != &OutputKind::new("SourceTheoremModuleSummary")
            || parent.schema_version() != SchemaVersion::new(1)
            || parent.lineage().named_input_hashes != named_input_hashes
        {
            return Err("native theorem TypedAst parent binding mismatch".to_owned());
        }
        let [resolved_id] = parent.lineage().parents.as_slice() else {
            return Err("native theorem TypedAst requires one ModuleResolve ancestor".to_owned());
        };
        let resolved = publisher
            .registry()
            .output_lineage(*resolved_id)
            .ok_or_else(|| "native theorem TypedAst requires registered ancestry".to_owned())?;
        if resolved.snapshot != input.snapshot
            || resolved.phase != IrPipelinePhase::new("ModuleResolve")
            || resolved.work_unit != work_unit
            || resolved.output_kind != OutputKind::new("SourceTheoremModuleCandidates")
            || resolved.named_input_hashes != named_input_hashes
        {
            return Err("native theorem TypedAst ModuleResolve ancestry mismatch".to_owned());
        }
        let [frontend_id] = resolved.parents.as_slice() else {
            return Err("native theorem TypedAst requires one Frontend ancestor".to_owned());
        };
        let frontend = publisher
            .registry()
            .output_lineage(*frontend_id)
            .ok_or_else(|| "native theorem TypedAst requires registered ancestry".to_owned())?;
        if frontend.snapshot != input.snapshot
            || frontend.phase != IrPipelinePhase::new("Frontend")
            || frontend.work_unit != work_unit
            || frontend.output_kind != OutputKind::new("FrontendOutput")
        {
            return Err("native theorem TypedAst Frontend ancestry mismatch".to_owned());
        }
        let typed = storage
            .typed_handle::<mizar_artifact::module_summary::ModuleSummary>(
                parent,
                &OutputKind::new("SourceTheoremModuleSummary"),
            )
            .map_err(|error| error.to_string())?;
        let retained = storage.get(&typed).map_err(|error| error.to_string())?;
        let Some(summary) = self.native_theorem_module_summary_candidate(module)? else {
            return Ok(None);
        };
        if *retained != summary {
            return Err("native theorem TypedAst complete summary mismatch".to_owned());
        }
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem TypedAst capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem TypedAst requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem TypedAst requires a workspace package".to_owned());
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
        if mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env)?.is_none() {
            return Ok(None);
        }
        let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(
            mizar_resolve::names::SourceVariableScopeInput::new(ast, &resolver_module, &env),
        )
        .map_err(|error| format!("{error:?}"))?;
        let checked = mizar_checker::type_checker::DeclarationChecker::
            check_source_theorem_equalities_with_typed_declarations(&arena, &env, &scope)?;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        Ok(Some(checked.2))
    }

    /// Lowers genuine algorithms after binding a published Frontend to captured source.
    pub fn native_algorithm_core_with_frontend_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<Option<mizar_core::core_ir::CoreIr>, String> {
        use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
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
            .validate_published_output(input.snapshot, parent)
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
        let typed = TypedArena::try_new(
            Some(TypedNodeId::new(arena.arena().root().index())),
            arena
                .arena()
                .iter()
                .map(|(id, node)| {
                    TypedNode::new(format!("{:?}", node.kind()), node.origin().anchor().clone())
                        .with_resolved_node(id)
                        .with_children(
                            node.children()
                                .iter()
                                .map(|child| TypedNodeId::new(child.index()))
                                .collect(),
                        )
                })
                .collect(),
        )
        .map_err(|error| error.to_string())?;
        let check =
            mizar_checker::type_checker::check_source_algorithm_types(&arena, &typed, &env)?;
        let core = mizar_core::elaborator::lower_source_algorithms(&check)?;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        Ok(Some(core))
    }

    /// Lowers genuine bound native theorem metadata into owned Core IR.
    pub fn native_theorem_core_with_frontend_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<Option<mizar_core::core_ir::CoreIr>, String> {
        let Some(metadata) = self.native_theorem_metadata_with_frontend_parent(input, publisher)?
        else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem core requires a module".to_owned());
        };
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
            .ok_or_else(|| "native theorem core requires retained syntax".to_owned())?;
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let Some((arena, env)) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &resolver_module)?
        else {
            return Ok(None);
        };
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
        let core = mizar_core::elaborator::lower_source_theorem_skeletons(&check, None)?;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        Ok(Some(core))
    }

    /// Generates the unpublished raw algorithm VC graph from parent-bound native Core.
    pub fn native_algorithm_vcs_with_frontend_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        generation_schema: &mizar_vc::vc_ir::GenerationSchemaVersion,
        vc_schema: &mizar_vc::vc_ir::VcSchemaVersion,
    ) -> Result<Option<mizar_vc::vc_ir::VcSet>, String> {
        let Some(core) = self.native_algorithm_core_with_frontend_parent(input, publisher)? else {
            return Ok(None);
        };
        let vcs = mizar_vc::generator::generate_source_algorithm_postconditions(
            &core,
            input.snapshot,
            generation_schema,
            vc_schema,
        )?;
        let parent = input.parent_outputs()[0].as_output_ref();
        publisher
            .storage()
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        Ok(Some(vcs))
    }

    /// Generates the unpublished raw VC graph from genuine parent-bound native Core.
    pub fn native_theorem_vcs_with_frontend_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        generation_schema: &mizar_vc::vc_ir::GenerationSchemaVersion,
        vc_schema: &mizar_vc::vc_ir::VcSchemaVersion,
    ) -> Result<Option<mizar_vc::vc_ir::VcSet>, String> {
        let Some(core) = self.native_theorem_core_with_frontend_parent(input, publisher)? else {
            return Ok(None);
        };
        let vcs = mizar_vc::generator::generate_source_theorem_vcs(
            &core,
            input.snapshot,
            generation_schema,
            vc_schema,
        )?;
        let parent = input.parent_outputs()[0].as_output_ref();
        publisher
            .storage()
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        Ok(Some(vcs))
    }
    /// Seals genuine bounded native algorithm VCs with their content and storage streams.
    pub fn publish_native_algorithm_vcs(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        generation_schema: &mizar_vc::vc_ir::GenerationSchemaVersion,
        vc_schema: &mizar_vc::vc_ir::VcSchemaVersion,
    ) -> Result<Option<mizar_ir::storage::PhaseOutputRef<mizar_vc::vc_ir::VcSet>>, String> {
        use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
        let Some(core) = self.native_algorithm_core_with_frontend_parent(input, publisher)? else {
            return Ok(None);
        };
        let vcs = mizar_vc::generator::generate_source_algorithm_postconditions(
            &core,
            input.snapshot,
            generation_schema,
            vc_schema,
        )?;
        let parent = input.parent_outputs()[0].as_output_ref();
        let storage = publisher.storage();
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native algorithm vc publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native algorithm vc publication requires one captured source".to_owned())?;
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
            .ok_or_else(|| "native algorithm vc publication requires retained syntax".to_owned())?;
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let Some((arena, env)) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        let neutral = TypedArena::try_new(
            Some(TypedNodeId::new(arena.arena().root().index())),
            arena
                .arena()
                .iter()
                .map(|(id, node)| {
                    TypedNode::new(format!("{:?}", node.kind()), node.origin().anchor().clone())
                        .with_resolved_node(id)
                        .with_children(
                            node.children()
                                .iter()
                                .map(|child| TypedNodeId::new(child.index()))
                                .collect(),
                        )
                })
                .collect(),
        )
        .map_err(|error| error.to_string())?;
        let check =
            mizar_checker::type_checker::check_source_algorithm_types(&arena, &neutral, &env)?;
        let core_semantic_bytes =
            mizar_core::elaborator::source_algorithm_core_semantic_payload(&check, &core)?;
        let domain = b"mizar-driver/native-algorithm-vc-content/v1\0";
        let fields = [
            generation_schema.as_str().as_bytes(),
            vcs.schema_version().as_str().as_bytes(),
            core_semantic_bytes.as_slice(),
        ];
        let mut size = domain.len() as u64;
        for field in fields {
            let Some(next) = u64::try_from(field.len())
                .ok()
                .and_then(|length| size.checked_add(8)?.checked_add(length))
            else {
                return Ok(None);
            };
            if next > 32 * 1024 * 1024 {
                return Ok(None);
            }
            size = next;
        }
        let Ok(size) = usize::try_from(size) else {
            return Ok(None);
        };
        let mut semantic_bytes = Vec::with_capacity(size);
        semantic_bytes.extend_from_slice(domain);
        for field in fields {
            semantic_bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
            semantic_bytes.extend_from_slice(field);
        }
        let Some(storage_bytes) = native_algorithm_vc_storage_bytes(
            ast,
            &resolver_module,
            &core,
            &vcs,
            input.snapshot,
            generation_schema,
        )?
        else {
            return Ok(None);
        };
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-algorithm-vc-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("VcGenerate");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceAlgorithmVC");
        let source_id = version.source_id;
        let snapshot = input.snapshot;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    input.snapshot,
                    phase.clone(),
                    work_unit.clone(),
                    output_kind.clone(),
                    SchemaVersion::new(1),
                ),
                snapshot: input.snapshot,
                phase,
                work_unit,
                output_kind,
                schema_version: SchemaVersion::new(1),
                payload: vcs,
                canonical_payload: Some(semantic_bytes),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_algorithm_vc_from_storage_bytes(
                        bytes,
                        source_id,
                        &resolver_module,
                        snapshot,
                    )
                    .map_err(BlobDecodeError::new)?
                    .ok_or_else(|| BlobDecodeError::new("invalid native algorithm vc storage"))
                }),
                parents: vec![parent.clone()],
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
                side_tables: IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "generate-algorithm-vc-storage",
                        version.normalized_path.as_str(),
                        storage_hash,
                    )],
                    ..IrSideTables::default()
                },
                origin: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    OutputOrigin::OpenBuffer
                } else {
                    OutputOrigin::PackageSource
                },
                target: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    PublicationTarget::InternalOnly
                } else {
                    PublicationTarget::CurrentPackage
                },
            })
            .map(Some)
            .map_err(|error| error.to_string())
    }
    /// Seals a genuine bounded native VC graph with its content and storage streams.
    pub fn publish_native_theorem_vcs(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        generation_schema: &mizar_vc::vc_ir::GenerationSchemaVersion,
        vc_schema: &mizar_vc::vc_ir::VcSchemaVersion,
    ) -> Result<Option<mizar_ir::storage::PhaseOutputRef<mizar_vc::vc_ir::VcSet>>, String> {
        let Some(core) = self.native_theorem_core_with_frontend_parent(input, publisher)? else {
            return Ok(None);
        };
        let vcs = mizar_vc::generator::generate_source_theorem_vcs(
            &core,
            input.snapshot,
            generation_schema,
            vc_schema,
        )?;
        let parent = input.parent_outputs()[0].as_output_ref();
        let storage = publisher.storage();
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem vc publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem vc publication requires one captured source".to_owned())?;
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
            .ok_or_else(|| "native theorem vc publication requires retained syntax".to_owned())?;
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let Some(core_semantic_bytes) =
            native_theorem_core_semantic_payload(ast, &resolver_module, &core)?
        else {
            return Ok(None);
        };
        let domain = b"mizar-driver/native-theorem-vc-content/v1\0";
        let fields = [
            generation_schema.as_str().as_bytes(),
            vcs.schema_version().as_str().as_bytes(),
            core_semantic_bytes.as_slice(),
        ];
        let mut size = domain.len() as u64;
        for field in fields {
            let Some(next) = u64::try_from(field.len())
                .ok()
                .and_then(|length| size.checked_add(8)?.checked_add(length))
            else {
                return Ok(None);
            };
            if next > 32 * 1024 * 1024 {
                return Ok(None);
            }
            size = next;
        }
        let Ok(size) = usize::try_from(size) else {
            return Ok(None);
        };
        let mut semantic_bytes = Vec::with_capacity(size);
        semantic_bytes.extend_from_slice(domain);
        for field in fields {
            let Ok(length) = u64::try_from(field.len()) else {
                return Ok(None);
            };
            semantic_bytes.extend_from_slice(&length.to_le_bytes());
            semantic_bytes.extend_from_slice(field);
        }
        let Some(storage_bytes) = native_theorem_vc_storage_bytes(
            ast,
            &resolver_module,
            &core,
            &vcs,
            input.snapshot,
            generation_schema,
        )?
        else {
            return Ok(None);
        };
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-vc-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("VcGenerate");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremVC");
        let source_id = version.source_id;
        let snapshot = input.snapshot;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    input.snapshot,
                    phase.clone(),
                    work_unit.clone(),
                    output_kind.clone(),
                    SchemaVersion::new(1),
                ),
                snapshot: input.snapshot,
                phase,
                work_unit,
                output_kind,
                schema_version: SchemaVersion::new(1),
                payload: vcs,
                canonical_payload: Some(semantic_bytes),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_theorem_vc_from_storage_bytes(
                        bytes,
                        source_id,
                        &resolver_module,
                        snapshot,
                    )
                    .map_err(BlobDecodeError::new)?
                    .ok_or_else(|| BlobDecodeError::new("invalid native theorem vc storage"))
                }),
                parents: vec![parent.clone()],
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
                side_tables: IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "generate-theorem-vc-storage",
                        version.normalized_path.as_str(),
                        storage_hash,
                    )],
                    ..IrSideTables::default()
                },
                origin: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    OutputOrigin::OpenBuffer
                } else {
                    OutputOrigin::PackageSource
                },
                target: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    PublicationTarget::InternalOnly
                } else {
                    PublicationTarget::CurrentPackage
                },
            })
            .map(Some)
            .map_err(|error| error.to_string())
    }
    /// Seals genuine bounded native algorithm Core with its semantic and storage streams.
    pub fn publish_native_algorithm_core(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<Option<mizar_ir::storage::PhaseOutputRef<mizar_core::core_ir::CoreIr>>, String>
    {
        use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
        let Some(core) = self.native_algorithm_core_with_frontend_parent(input, publisher)? else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native algorithm core publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native algorithm core publication requires one captured source".to_owned()
        })?;
        let parent = input.parent_outputs()[0].as_output_ref();
        let storage = publisher.storage();
        let typed = storage
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                parent,
                &OutputKind::new("FrontendOutput"),
            )
            .map_err(|error| error.to_string())?;
        let retained = storage.get(&typed).map_err(|error| error.to_string())?;
        let ast = retained.ast.as_ref().ok_or_else(|| {
            "native algorithm core publication requires retained syntax".to_owned()
        })?;
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let Some((arena, env)) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        let neutral = TypedArena::try_new(
            Some(TypedNodeId::new(arena.arena().root().index())),
            arena
                .arena()
                .iter()
                .map(|(id, node)| {
                    TypedNode::new(format!("{:?}", node.kind()), node.origin().anchor().clone())
                        .with_resolved_node(id)
                        .with_children(
                            node.children()
                                .iter()
                                .map(|child| TypedNodeId::new(child.index()))
                                .collect(),
                        )
                })
                .collect(),
        )
        .map_err(|error| error.to_string())?;
        let check =
            mizar_checker::type_checker::check_source_algorithm_types(&arena, &neutral, &env)?;
        let semantic_bytes =
            mizar_core::elaborator::source_algorithm_core_semantic_payload(&check, &core)?;
        let Some(storage_bytes) =
            native_algorithm_core_storage_bytes(ast, &resolver_module, &core)?
        else {
            return Ok(None);
        };
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-algorithm-core-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("Elaboration");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceAlgorithmCore");
        let source_id = version.source_id;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    input.snapshot,
                    phase.clone(),
                    work_unit.clone(),
                    output_kind.clone(),
                    SchemaVersion::new(1),
                ),
                snapshot: input.snapshot,
                phase,
                work_unit,
                output_kind,
                schema_version: SchemaVersion::new(1),
                payload: core,
                canonical_payload: Some(semantic_bytes),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_algorithm_core_from_storage_bytes(bytes, source_id, &resolver_module)
                        .map_err(BlobDecodeError::new)?
                        .ok_or_else(|| {
                            BlobDecodeError::new("invalid native algorithm core storage")
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
                        "elaborate-algorithm-core-storage",
                        version.normalized_path.as_str(),
                        storage_hash,
                    )],
                    ..IrSideTables::default()
                },
                origin: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    OutputOrigin::OpenBuffer
                } else {
                    OutputOrigin::PackageSource
                },
                target: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    PublicationTarget::InternalOnly
                } else {
                    PublicationTarget::CurrentPackage
                },
            })
            .map(Some)
            .map_err(|error| error.to_string())
    }

    /// Seals genuine bounded native Core with its semantic and storage streams.
    pub fn publish_native_theorem_core(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<Option<mizar_ir::storage::PhaseOutputRef<mizar_core::core_ir::CoreIr>>, String>
    {
        let Some(core) = self.native_theorem_core_with_frontend_parent(input, publisher)? else {
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
            .ok_or_else(|| "native theorem core publication requires retained syntax".to_owned())?;
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
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    input.snapshot,
                    phase.clone(),
                    work_unit.clone(),
                    output_kind.clone(),
                    SchemaVersion::new(1),
                ),
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
                origin: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    OutputOrigin::OpenBuffer
                } else {
                    OutputOrigin::PackageSource
                },
                target: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    PublicationTarget::InternalOnly
                } else {
                    PublicationTarget::CurrentPackage
                },
            })
            .map(Some)
            .map_err(|error| error.to_string())
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
            .validate_published_output(input.snapshot, parent)
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
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    input.snapshot,
                    phase.clone(),
                    work_unit.clone(),
                    output_kind.clone(),
                    SchemaVersion::new(1),
                ),
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
                origin: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    OutputOrigin::OpenBuffer
                } else {
                    OutputOrigin::PackageSource
                },
                target: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    PublicationTarget::InternalOnly
                } else {
                    PublicationTarget::CurrentPackage
                },
            })
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
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    input.snapshot,
                    phase.clone(),
                    work_unit.clone(),
                    output_kind.clone(),
                    SchemaVersion::new(1),
                ),
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
                origin: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    OutputOrigin::OpenBuffer
                } else {
                    OutputOrigin::PackageSource
                },
                target: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                    PublicationTarget::InternalOnly
                } else {
                    PublicationTarget::CurrentPackage
                },
            })
            .map(Some)
            .map_err(|error| error.to_string())
    }

    fn native_source(
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

struct DependencyLexicalProvider<'a> {
    inputs: crate::registry::SourceLoadInputs<'a>,
    artifact_roots: &'a [(mizar_session::PackageId, PathBuf)],
    workspace_summaries: &'a [(
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

pub(crate) struct SourceTheoremSignatureCollectionService;

impl PhaseService for SourceTheoremSignatureCollectionService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremSignatureCollection",
            PhaseOwner::MizarResolve,
            vec![PipelinePhase::SignatureCollection],
            "mizar-resolve/source-theorem-summary-publication/v1",
            "SourceTheoremModuleSummary",
        )
        .expect("source theorem signature collection descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem signature collection has no complete dependency footprint"
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
        match inputs.publish_native_theorem_module_summary(&input, &publisher) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}

pub(crate) struct SourceLoadService;

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
                    let (number, name) = match &error {
                        SourceLoadError::InvalidUtf8 { .. } => (601, "invalid_utf8"),
                        SourceLoadError::UnreadableSourceFile { .. } => (602, "unreadable_file"),
                        SourceLoadError::SourcePathOutsidePackageRoot { .. } => {
                            (603, "outside_package_root")
                        }
                        SourceLoadError::UnsupportedFileExtension { .. } => {
                            (600, "unsupported_file_extension")
                        }
                        SourceLoadError::DuplicateModulePath { .. } => {
                            (600, "duplicate_module_path")
                        }
                        SourceLoadError::StaleLspDocumentVersion { .. } => {
                            (600, "stale_lsp_document_version")
                        }
                        SourceLoadError::UnmappedOpenBufferUri { .. } => {
                            (600, "unmapped_open_buffer_uri")
                        }
                        SourceLoadError::GeneratedSourceWithoutMetadata { .. } => {
                            (600, "generated_source_without_metadata")
                        }
                        SourceLoadError::SourceIdAllocation { .. } => (600, "source_id_allocation"),
                        SourceLoadError::UnsupportedSourceOrigin { .. } => {
                            (600, "unsupported_source_origin")
                        }
                        SourceLoadError::InvalidSourcePath { .. } => (600, "invalid_source_path"),
                        _ => return blocking(),
                    };
                    let detail = format!("source.{name}");
                    let code = DiagnosticCode::from_parts(DiagnosticSeverity::Error, number)
                        .expect("source-load diagnostic code is valid");
                    let draft = DiagnosticDraft::new(DiagnosticDraftInput {
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
                    });
                    let Ok(draft) = draft else {
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
        let phase = IrPipelinePhase::new("SourceLoad");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            source.package_id.as_str(),
            source.module_path.as_str()
        ));
        let output_kind = OutputKind::new("SourceUnit");
        let decode_input = source_input;
        let source_id = source.source_id;
        let expected_source = source.clone();
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
            origin: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                OutputOrigin::OpenBuffer
            } else {
                OutputOrigin::PackageSource
            },
            target: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                PublicationTarget::InternalOnly
            } else {
                PublicationTarget::CurrentPackage
            },
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

pub(crate) struct FrontendService {
    pub(crate) artifact_roots: Vec<(PackageId, PathBuf)>,
}

impl PhaseService for FrontendService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "Frontend",
            PhaseOwner::MizarFrontend,
            vec![PipelinePhase::Frontend],
            "mizar-frontend/output-publication/v1",
            "FrontendOutput",
        )
        .expect("frontend descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "frontend inputs require a live source and dependency run".to_owned(),
        }
    }

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext<'_>) -> PhaseResult {
        let snapshot = input.snapshot;
        let Some(mut sink) = context.diagnostics else {
            return blocking();
        };
        if sink.is_sealed()
            || !sink.drafts().is_empty()
            || sink.scope().phase() != DiagnosticPhase::Frontend
            || sink.scope().source_snapshot() != snapshot
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
        let Some(inputs) = context.source_load else {
            return blocking();
        };
        if inputs.snapshot.id != snapshot
            || inputs.snapshot.workspace_root != inputs.build_plan.workspace_root
        {
            return blocking();
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return blocking();
        };
        let Some(version) = unique(inputs.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        })) else {
            return blocking();
        };
        if version.origin != SourceOrigin::Disk
            && inputs
                .captured
                .is_none_or(|capture| capture.native_sources().is_none())
        {
            return blocking();
        }
        let Some(package) = unique(
            inputs
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
            inputs
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
            inputs
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
        let workspace = PathBuf::from(inputs.snapshot.workspace_root.as_str());
        let package_root = workspace.join(root);
        if workspace.join(source_root) != package_root.join("src")
            || workspace.join(manifest_path) != package_root.join("mizar.pkg")
        {
            return blocking();
        }
        let source_key = source_input_hash(version);
        for indexed in &inputs.module_index.modules {
            let ModuleIndexLocation::DependencySummary {
                artifact,
                content_hash,
            } = &indexed.location
            else {
                continue;
            };
            let Some(reference) = unique(
                inputs
                    .module_index
                    .dependency_summaries
                    .iter()
                    .filter(|reference| reference.module == indexed.module),
            ) else {
                return blocking();
            };
            if reference.artifact != *artifact
                || reference.content_hash != *content_hash
                || unique(
                    inputs
                        .snapshot
                        .dependency_artifacts
                        .iter()
                        .filter(|captured| {
                            captured.artifact == *artifact && captured.content_hash == *content_hash
                        }),
                )
                .is_none()
            {
                return blocking();
            }
        }
        for reference in &inputs.module_index.dependency_summaries {
            let Some(indexed) = unique(
                inputs
                    .module_index
                    .modules
                    .iter()
                    .filter(|indexed| indexed.module == reference.module),
            ) else {
                return blocking();
            };
            if !matches!(&indexed.location,
                ModuleIndexLocation::DependencySummary { artifact, content_hash }
                if artifact == &reference.artifact && content_hash == &reference.content_hash)
                || unique(
                    inputs
                        .snapshot
                        .dependency_artifacts
                        .iter()
                        .filter(|captured| {
                            captured.artifact == reference.artifact
                                && captured.content_hash == reference.content_hash
                        }),
                )
                .is_none()
            {
                return blocking();
            }
        }
        let mut dependencies = inputs
            .module_index
            .dependency_summaries
            .iter()
            .map(|reference| reference.content_hash)
            .collect::<Vec<_>>();
        dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        if input.identities().input_hash() != source_key
            || input.identities().dependency_hashes() != dependencies
        {
            return blocking();
        }
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let Some(parent) = unique(
            input
                .parent_outputs()
                .iter()
                .map(|handle| handle.as_output_ref())
                .filter(|handle| {
                    handle.phase() == &IrPipelinePhase::new("SourceLoad")
                        && handle.work_unit() == &work_unit
                        && handle.output_kind() == &OutputKind::new("SourceUnit")
                        && handle.schema_version() == SchemaVersion::new(1)
                }),
        ) else {
            return blocking();
        };
        let parents_current = || {
            input.parent_outputs().iter().all(|handle| {
                let handle = handle.as_output_ref();
                (if inputs
                    .captured
                    .is_some_and(|capture| capture.native_sources().is_some())
                {
                    publisher.validate_published_output(snapshot, handle)
                } else {
                    publisher.validate_current_output(snapshot, handle)
                })
                .is_ok()
                    && publisher.storage().validate_handle(handle).is_ok()
            })
        };
        if !parents_current()
            || input.parent_outputs().iter().any(|handle| {
                let handle = handle.as_output_ref();
                handle.phase() == &IrPipelinePhase::new("SourceLoad") && handle != parent
            })
        {
            return blocking();
        }
        let Ok(typed) = publisher
            .storage()
            .typed_handle::<SourceUnit>(parent, &OutputKind::new("SourceUnit"))
        else {
            return blocking();
        };
        let Ok(source) = publisher.storage().get(&typed) else {
            return blocking();
        };
        let source_input = SourceInput {
            package_id: version.package_id.clone(),
            module_path: version.module_path.clone(),
            normalized_path: version.normalized_path.clone(),
            edition: version.edition.clone(),
            origin: SourceOriginInput::Disk {
                path: version.normalized_path.as_str().into(),
            },
        };
        let source_input = if inputs
            .captured
            .is_some_and(|capture| capture.native_sources().is_some())
        {
            let Some((expected, input)) = inputs.native_source(version) else {
                return blocking();
            };
            if source.as_ref() != &expected {
                return blocking();
            }
            input
        } else {
            source_input
        };
        if (match &source_input.origin {
            SourceOriginInput::OpenBuffer { .. } => {
                source.canonical_open_buffer_bytes(&source_input)
            }
            _ => source.canonical_disk_bytes(),
        })
        .is_none()
            || source.source_id != version.source_id
            || source.package_id != version.package_id
            || source.module_path != version.module_path
            || source.normalized_path != version.normalized_path
            || source.edition != version.edition
            || source.source_hash != version.source_hash
            || source.origin != version.origin
        {
            return blocking();
        }
        let Some(workspace_summaries) = (|| {
            use mizar_artifact::{
                module_summary::ModuleSummaryIdentity, store::canonical_json_string,
            };
            use mizar_resolve::{
                declarations::DeclarationShellCollector, env::NamespacePath,
                module_index::IndexedModuleId, resolved_ast::ModuleId,
                symbols::SignatureProjectionExtractor,
            };
            let mut summaries = Vec::new();
            for handle in input.parent_outputs() {
                let leaf = handle.as_output_ref();
                if leaf == parent {
                    continue;
                }
                if leaf.phase() != &IrPipelinePhase::new("Frontend")
                    || leaf.output_kind() != &OutputKind::new("FrontendOutput")
                    || leaf.schema_version() != SchemaVersion::new(1)
                {
                    return None;
                }
                let typed = publisher
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        leaf,
                        &OutputKind::new("FrontendOutput"),
                    )
                    .ok()?;
                let output = publisher.storage().get(&typed).ok()?;
                let leaf_source = &output.source;
                let leaf_version =
                    unique(inputs.snapshot.source_versions.iter().filter(|candidate| {
                        candidate.package_id == leaf_source.package_id
                            && candidate.module_path == leaf_source.module_path
                    }))?;
                let leaf_module = IndexedModuleId::new(
                    leaf_source.package_id.clone(),
                    leaf_source.module_path.clone(),
                );
                let leaf_entry = unique(
                    inputs
                        .module_index
                        .modules
                        .iter()
                        .filter(|candidate| candidate.module == leaf_module),
                )?;
                let ModuleIndexLocation::WorkspaceFile {
                    source_root: leaf_root,
                    normalized_path: leaf_path,
                    source_relative_path,
                } = &leaf_entry.location
                else {
                    return None;
                };
                let leaf_unit = IrWorkUnit::new(format!(
                    "{:?}:{:?}",
                    leaf_module.package.as_str(),
                    leaf_module.path.as_str()
                ));
                let parent_lineages = leaf
                    .lineage()
                    .parents
                    .iter()
                    .map(|parent| publisher.registry().output_lineage(*parent))
                    .collect::<Option<Vec<_>>>()?;
                let source_lineage = unique(
                    parent_lineages
                        .iter()
                        .filter(|parent| parent.phase == IrPipelinePhase::new("SourceLoad")),
                )?;
                let leaf_package = unique(
                    inputs
                        .build_plan
                        .packages
                        .iter()
                        .filter(|package| package.package_id == leaf_module.package),
                )?;
                let PackagePlanSource::Workspace {
                    root: target_root,
                    source_root: target_source_root,
                    manifest_path: target_manifest,
                } = &leaf_package.source
                else {
                    return None;
                };
                let leaf_index_package = unique(
                    inputs
                        .module_index
                        .packages
                        .iter()
                        .filter(|package| package.package_id == leaf_module.package),
                )?;
                let mizar_build::module_index::PackageIndexSource::Workspace {
                    package_root: indexed_root,
                    source_root: indexed_source_root,
                    manifest_path: indexed_manifest,
                } = &leaf_index_package.source
                else {
                    return None;
                };
                if target_root != indexed_root
                    || target_source_root != indexed_source_root
                    || target_manifest != indexed_manifest
                    || leaf_package.version != leaf_index_package.version
                    || leaf_package.edition != leaf_version.edition
                    || leaf_index_package.edition != leaf_version.edition
                    || !safe_relative(target_root)
                    || !safe_relative(target_source_root)
                    || !safe_relative(target_manifest)
                    || workspace.join(target_source_root) != workspace.join(target_root).join("src")
                    || workspace.join(target_manifest)
                        != workspace.join(target_root).join("mizar.pkg")
                {
                    return None;
                }
                let leaf_source_key = source_input_hash(leaf_version);
                let native_leaf = if inputs
                    .captured
                    .is_some_and(|capture| capture.native_sources().is_some())
                {
                    let (expected, raw) = inputs.native_source(leaf_version)?;
                    if leaf_source != &expected {
                        return None;
                    }
                    Some(raw)
                } else {
                    None
                };
                if leaf_module == *module
                    || leaf.work_unit() != &leaf_unit
                    || (native_leaf.is_none() && leaf_version.origin != SourceOrigin::Disk)
                    || (match native_leaf.as_ref().map(|raw| &raw.origin) {
                        Some(SourceOriginInput::OpenBuffer { .. }) => {
                            output.canonical_open_buffer_bytes(native_leaf.as_ref()?)
                        }
                        _ => output.canonical_disk_bytes(),
                    })
                    .is_none()
                    || leaf_source.source_id != leaf_version.source_id
                    || leaf_source.package_id != leaf_version.package_id
                    || leaf_source.module_path != leaf_version.module_path
                    || leaf_source.normalized_path != leaf_version.normalized_path
                    || leaf_source.edition != leaf_version.edition
                    || leaf_source.source_hash != leaf_version.source_hash
                    || leaf_source.origin != leaf_version.origin
                    || leaf_entry.package_id != leaf_module.package
                    || leaf_entry.module_path != leaf_module.path
                    || leaf_entry.edition != leaf_version.edition
                    || leaf_root != target_source_root
                    || leaf_path != leaf_version.normalized_path.as_str()
                    || leaf_path != &format!("src/{source_relative_path}")
                    || source_relative_path.is_empty()
                    || index_package.version != package.version
                    || source_lineage.snapshot != snapshot
                    || source_lineage.phase != IrPipelinePhase::new("SourceLoad")
                    || source_lineage.work_unit != leaf_unit
                    || source_lineage.output_kind != OutputKind::new("SourceUnit")
                    || !source_lineage.parents.is_empty()
                    || source_lineage.named_input_hashes.as_slice()
                        != [NamedInputHash {
                            name: "source".to_owned(),
                            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                            digest: leaf_source_key,
                        }]
                    || output.cache_keys.source.stable_hash() != leaf_source_key
                    || !output.diagnostics.is_empty()
                    || output.ast.as_ref().is_none_or(|ast| {
                        ast.node_views().any(|view| view.as_export_item().is_some())
                    })
                {
                    return None;
                }
                let index = mizar_resolve::module_index::ModuleIndexInput::new(inputs.module_index);
                let resolution = mizar_resolve::imports::ImportPathResolver::new(index)
                    .resolve_frontend(&output, leaf_source)?;
                if !resolution.unresolved().is_empty() {
                    return None;
                }
                let mut targets = Vec::new();
                for import in resolution.resolved() {
                    let target = IndexedModuleId::new(
                        import.target().package().clone(),
                        import.target().path().clone(),
                    );
                    let entry = unique(
                        inputs
                            .module_index
                            .modules
                            .iter()
                            .filter(|entry| entry.module == target),
                    )?;
                    match &entry.location {
                        ModuleIndexLocation::WorkspaceFile { .. } => {
                            if target == leaf_module {
                                return None;
                            }
                        }
                        ModuleIndexLocation::DependencySummary { .. } => continue,
                        _ => return None,
                    }
                    let unit = IrWorkUnit::new(format!(
                        "{:?}:{:?}",
                        target.package.as_str(),
                        target.path.as_str()
                    ));
                    if !targets.contains(&unit) {
                        targets.push(unit);
                    }
                }
                for parent in &parent_lineages {
                    if parent.output == source_lineage.output {
                        continue;
                    }
                    if parent.snapshot != snapshot
                        || parent.phase != IrPipelinePhase::new("Frontend")
                        || parent.output_kind != OutputKind::new("FrontendOutput")
                    {
                        return None;
                    }
                    let position = targets.iter().position(|unit| unit == &parent.work_unit)?;
                    targets.remove(position);
                }
                if !targets.is_empty() {
                    return None;
                }
                let mut named_inputs = vec![
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
                for (ordinal, digest) in dependencies.iter().enumerate() {
                    named_inputs.push(NamedInputHash {
                        name: format!("dependency.{ordinal}"),
                        domain: "mizar-frontend/dependency-summary/v1".to_owned(),
                        digest: *digest,
                    });
                }
                named_inputs.sort_by(|left, right| {
                    left.name
                        .cmp(&right.name)
                        .then_with(|| left.domain.cmp(&right.domain))
                        .then_with(|| left.digest.as_bytes().cmp(right.digest.as_bytes()))
                });
                if leaf.lineage().named_input_hashes != named_inputs {
                    return None;
                }
                let ast = output.ast.as_ref()?;
                let resolver_module =
                    ModuleId::new(leaf_module.package.clone(), leaf_module.path.clone());
                let shells = DeclarationShellCollector::new(ast, &resolver_module).collect();
                let collection = SignatureProjectionExtractor::new(
                    ast,
                    &shells,
                    NamespacePath::new(resolver_module.path().as_str()),
                )
                .collect(&resolver_module);
                if !collection.diagnostics().is_empty() {
                    return None;
                }
                let identity = ModuleSummaryIdentity {
                    package_id: leaf_module.package.as_str().to_owned(),
                    package_version: Some(leaf_package.version.to_string()),
                    lockfile_identity: None,
                    module_path: leaf_module.path.as_str().to_owned(),
                    language_edition: leaf_version.edition.as_str().to_owned(),
                };
                let module_id = canonical_json_string(&identity.canonical_json().ok()?);
                let shapes = collection
                    .export_frontend_lexical_contributions(&output, &identity)?
                    .into_iter()
                    .map(|contribution| {
                        ExportedSymbolShape::from_canonical_bytes(contribution.payload.as_bytes())
                    })
                    .collect::<Option<Vec<_>>>()?;
                let summary = ModuleLexicalSummary::from_exported_symbols(
                    mizar_frontend::lexical_env::ModuleId::new(module_id),
                    shapes,
                )?;
                summaries.push((leaf_module, summary));
            }
            Some(summaries)
        })() else {
            return blocking();
        };
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(package_root)),
            DependencyLexicalProvider {
                inputs,
                artifact_roots: &self.artifact_roots,
                workspace_summaries: &workspace_summaries,
            },
            MizarParserSeam,
        );
        let Ok(output) = frontend.run_loaded(source.as_ref().clone()) else {
            return blocking();
        };
        if output.ast.is_some() != output.cache_keys.ast.is_some() {
            return blocking();
        }
        if !parents_current() {
            return blocking();
        }
        let Some(drafts) =
            convert_frontend_diagnostics(&output.source, snapshot, &output.diagnostics)
        else {
            return blocking();
        };
        for draft in drafts {
            if sink.emit(draft).is_err() {
                return blocking();
            }
        }
        if !output.diagnostics.is_empty() {
            if !parents_current() {
                return blocking();
            }
            let import_batch = (|| {
                use mizar_diagnostics::sink::{DiagnosticProducerScope, DiagnosticSink};
                use mizar_resolve::{
                    imports::{ImportPathFailureClass as Failure, ImportPathResolver},
                    module_index::ModuleIndexInput,
                };
                if !output.diagnostics.iter().all(|diagnostic| {
                    diagnostic.code
                        == FrontendCode::LexicalEnvironment(EnvironmentCode::UnresolvedImport)
                        && diagnostic.class == DiagnosticClass::LexicalEnvironment
                }) || output.source != *source
                    || output.preprocessed.source_id != source.source_id
                {
                    return None;
                }
                let index = ModuleIndexInput::new(inputs.module_index);
                let current = index.resolver_module_id(module);
                let resolution =
                    ImportPathResolver::new(index).resolve_frontend(&output, source.as_ref())?;
                if resolution.unresolved().is_empty() {
                    return None;
                }
                let registry = mizar_diagnostics::registry::DiagnosticRegistry::builtin();
                let drafts = resolution
                    .unresolved()
                    .iter()
                    .map(|candidate| {
                        let number = match candidate.class() {
                            Failure::UnknownNamespaceOrPackage => 220,
                            Failure::UnknownModule => 221,
                            Failure::RelativePathEscapesPackage => 222,
                            Failure::DuplicateAlias => 223,
                            Failure::AliasRootConflict => 224,
                            _ => return None,
                        };
                        let alias_range = candidate
                            .alias_range()
                            .or(candidate.branch_member_range())
                            .unwrap_or(candidate.range());
                        let primary = match number {
                            220 | 222 => candidate.branch_base_range().unwrap_or(candidate.range()),
                            221 => candidate.branch_member_range().unwrap_or(candidate.range()),
                            _ => alias_range,
                        };
                        let mut secondary = match number {
                            220 | 222 => candidate.branch_member_range(),
                            _ => candidate.branch_base_range(),
                        }
                        .into_iter()
                        .collect::<Vec<_>>();
                        let mut details = vec![
                            (
                                "import.source_module",
                                DiagnosticDetailValue::List(vec![
                                    DiagnosticDetailValue::String(
                                        current.package().as_str().to_owned(),
                                    ),
                                    DiagnosticDetailValue::String(
                                        current.path().as_str().to_owned(),
                                    ),
                                ]),
                            ),
                            (
                                "import.ordinal",
                                DiagnosticDetailValue::Integer(
                                    i64::try_from(candidate.ordinal()).ok()?,
                                ),
                            ),
                            (
                                "import.path",
                                DiagnosticDetailValue::String(candidate.spelling().to_owned()),
                            ),
                        ];
                        if let Some(member) = candidate.branch_member_range() {
                            details.push((
                                "import.branch_member",
                                DiagnosticDetailValue::Source(member),
                            ));
                        }
                        if number == 223 {
                            secondary.retain(|range| *range != primary);
                            let alias = candidate
                                .alias()
                                .or_else(|| candidate.components().last().map(String::as_str))?;
                            let target = candidate.candidate_target()?;
                            details.push((
                                "import.alias",
                                DiagnosticDetailValue::String(alias.to_owned()),
                            ));
                            details.push((
                                "import.target",
                                DiagnosticDetailValue::List(vec![
                                    DiagnosticDetailValue::String(
                                        target.package().as_str().to_owned(),
                                    ),
                                    DiagnosticDetailValue::String(
                                        target.path().as_str().to_owned(),
                                    ),
                                ]),
                            ));
                            let mut peers = resolution
                                .unresolved()
                                .iter()
                                .filter(|peer| {
                                    peer.class() == Failure::DuplicateAlias
                                        && peer.ordinal() != candidate.ordinal()
                                        && peer.alias().or_else(|| {
                                            peer.components().last().map(String::as_str)
                                        }) == Some(alias)
                                })
                                .map(|peer| {
                                    Some((
                                        peer.ordinal(),
                                        peer.range().start,
                                        peer.range().end,
                                        peer.candidate_target()?,
                                        peer.alias_range()
                                            .or(peer.branch_member_range())
                                            .unwrap_or(peer.range()),
                                    ))
                                })
                                .collect::<Option<Vec<_>>>()?;
                            peers.sort_by(|a, b| {
                                (&a.0, &a.1, &a.2, &a.3).cmp(&(&b.0, &b.1, &b.2, &b.3))
                            });
                            secondary.extend(peers.into_iter().map(|peer| peer.4));
                        }
                        let span = |range, role| {
                            source.line_map.validate_range(range).ok()?;
                            DiagnosticSpan::from_anchor(
                                mizar_session::SourceAnchor::Range(range),
                                role,
                                None,
                                SpanFreshness::Current,
                                None,
                            )
                            .ok()
                        };
                        let primary = span(primary, DiagnosticSpanRole::Primary)?;
                        let secondary = secondary
                            .into_iter()
                            .map(|range| span(range, DiagnosticSpanRole::Secondary))
                            .collect::<Option<Vec<_>>>()?;
                        let descriptor = registry.lookup(
                            DiagnosticCode::from_parts(DiagnosticSeverity::Error, number).ok()?,
                        )?;
                        DiagnosticDraft::new(DiagnosticDraftInput {
                            source_snapshot: snapshot,
                            code: descriptor.code,
                            phase: DiagnosticPhase::Resolver,
                            category: FailureCategory::ResolveError,
                            stable_detail_key: descriptor.semantic_name.to_owned(),
                            message: descriptor.summary.to_owned(),
                            primary_location: DiagnosticPrimaryLocation::Span(primary),
                            secondary_spans: secondary,
                            notes: Vec::new(),
                            details: DiagnosticDetails::from_entries(details).ok()?,
                            fixes: Vec::new(),
                            explanation: None,
                        })
                        .ok()
                    })
                    .collect::<Option<Vec<_>>>()?;
                let mut resolver_sink = DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::Resolver,
                    snapshot,
                    "frontend.import_continuation",
                ));
                for draft in drafts {
                    resolver_sink.emit(draft).ok()?;
                }
                Some(resolver_sink.seal())
            })();
            if !parents_current() {
                return blocking();
            }
            let mut diagnostics = vec![sink.seal()];
            diagnostics.extend(import_batch);
            return PhaseResult {
                status: if output.ast.is_some() {
                    PhaseStatus::Recoverable
                } else {
                    PhaseStatus::Fatal
                },
                diagnostics,
                ..blocking()
            };
        }
        let index = mizar_resolve::module_index::ModuleIndexInput::new(inputs.module_index);
        let Some(resolution) = mizar_resolve::imports::ImportPathResolver::new(index)
            .resolve_frontend(&output, source.as_ref())
        else {
            return blocking();
        };
        if !resolution.unresolved().is_empty() {
            return blocking();
        }
        let mut used = Vec::new();
        for import in resolution.resolved() {
            let target = mizar_resolve::module_index::IndexedModuleId::new(
                import.target().package().clone(),
                import.target().path().clone(),
            );
            let Some(entry) = unique(
                inputs
                    .module_index
                    .modules
                    .iter()
                    .filter(|entry| entry.module == target),
            ) else {
                return blocking();
            };
            if matches!(&entry.location, ModuleIndexLocation::WorkspaceFile { .. })
                && !workspace_summaries
                    .iter()
                    .any(|(module, _)| module == &target)
            {
                return blocking();
            }
            if matches!(&entry.location, ModuleIndexLocation::WorkspaceFile { .. })
                && !used.contains(&target)
            {
                used.push(target);
            }
        }
        if used.len() != workspace_summaries.len() {
            return blocking();
        }
        let Some(ast_key) = output.cache_keys.ast.as_ref() else {
            return blocking();
        };
        let Some(storage_bytes) = (match &source_input.origin {
            SourceOriginInput::OpenBuffer { .. } => {
                output.canonical_open_buffer_bytes(&source_input)
            }
            _ => output.canonical_disk_bytes(),
        }) else {
            return blocking();
        };
        let mut semantic_bytes = b"mizar-frontend/output-publication/v1\0".to_vec();
        semantic_bytes.extend_from_slice(ast_key.stable_hash().as_bytes());
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-frontend/output-storage-side-table/v1",
            &storage_bytes,
        ));
        let mut named_input_hashes = vec![
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
        for (ordinal, digest) in dependencies.into_iter().enumerate() {
            named_input_hashes.push(NamedInputHash {
                name: format!("dependency.{ordinal}"),
                domain: "mizar-frontend/dependency-summary/v1".to_owned(),
                digest,
            });
        }
        let decode_input = source_input;
        let source_id = version.source_id;
        let expected_source = output.source.clone();
        let phase = IrPipelinePhase::new("Frontend");
        let output_kind = OutputKind::new("FrontendOutput");
        if !parents_current() {
            return blocking();
        }
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
            payload: output,
            canonical_payload: Some(semantic_bytes),
            storage_payload: Some(storage_bytes),
            decode: BlobDecoder::new(move |bytes| {
                FrontendOutput::from_canonical_storage_bytes(bytes, source_id, &decode_input)
                    .filter(|output| {
                        output
                            .source
                            .matches_canonical_storage(&expected_source, &decode_input)
                    })
                    .ok_or_else(|| BlobDecodeError::new("invalid frontend disk payload"))
            }),
            parents: input
                .parent_outputs()
                .iter()
                .map(|handle| handle.as_output_ref().clone())
                .collect(),
            named_input_hashes,
            side_tables: IrSideTables {
                source_maps: vec![SideTableRecord::new(
                    "frontend-storage",
                    version.normalized_path.as_str(),
                    storage_hash,
                )],
                ..IrSideTables::default()
            },
            origin: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                OutputOrigin::OpenBuffer
            } else {
                OutputOrigin::PackageSource
            },
            target: if matches!(version.origin, SourceOrigin::OpenBuffer { .. }) {
                PublicationTarget::InternalOnly
            } else {
                PublicationTarget::CurrentPackage
            },
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

fn convert_frontend_diagnostics(
    source: &SourceUnit,
    snapshot: mizar_session::BuildSnapshotId,
    diagnostics: &[FrontendDiagnostic],
) -> Option<Vec<DiagnosticDraft>> {
    let registry = mizar_diagnostics::registry::DiagnosticRegistry::builtin();
    diagnostics
        .iter()
        .map(|diagnostic| {
            let class = match diagnostic.class {
                DiagnosticClass::LexicalPrecondition => "lexical_precondition",
                DiagnosticClass::CommentStructure => "comment_structure",
                DiagnosticClass::ImportPrescan => "import_prescan",
                DiagnosticClass::LexicalEnvironment => "lexical_environment",
                DiagnosticClass::ScopeSkeleton => "scope_skeleton",
                DiagnosticClass::Tokenization => "tokenization",
                DiagnosticClass::Syntax => "syntax",
                DiagnosticClass::AnnotationSyntax => "annotation_syntax",
                _ => return None,
            };
            let (number, prefix, leaf, expected_class) = match &diagnostic.code {
                FrontendCode::Preprocess(PreprocessDiagnosticKind::SourcePrecondition(code)) => {
                    match code {
                        SourcePreprocessDiagnosticCode::CarriageReturn => (
                            13,
                            "preprocess.source_precondition",
                            "carriage_return",
                            "lexical_precondition",
                        ),
                        SourcePreprocessDiagnosticCode::NonAsciiCode => (
                            14,
                            "preprocess.source_precondition",
                            "non_ascii_code",
                            "lexical_precondition",
                        ),
                        SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment => (
                            15,
                            "preprocess.source_precondition",
                            "unterminated_multi_line_comment",
                            "comment_structure",
                        ),
                        _ => return None,
                    }
                }
                FrontendCode::Preprocess(PreprocessDiagnosticKind::ImportPrescan(code)) => {
                    let (number, leaf) = match code {
                        ImportPrescanDiagnosticCode::MissingModulePath => {
                            (16, "missing_module_path")
                        }
                        ImportPrescanDiagnosticCode::EmptyModulePathComponent => {
                            (17, "empty_module_path_component")
                        }
                        ImportPrescanDiagnosticCode::MissingAlias => (18, "missing_alias"),
                        ImportPrescanDiagnosticCode::MissingSemicolon => (19, "missing_semicolon"),
                        ImportPrescanDiagnosticCode::UnexpectedToken => (20, "unexpected_token"),
                        _ => return None,
                    };
                    (number, "preprocess.import_prescan", leaf, "import_prescan")
                }
                FrontendCode::Preprocess(PreprocessDiagnosticKind::RawImportScan) => {
                    (21, "preprocess", "raw_import_scan", "import_prescan")
                }
                FrontendCode::LexicalEnvironment(code) => {
                    let (number, leaf) = match code {
                        EnvironmentCode::UnresolvedImport => (22, "unresolved_import"),
                        EnvironmentCode::MissingSummary => (23, "missing_summary"),
                        EnvironmentCode::UserSymbolImportConflict => {
                            (24, "user_symbol_import_conflict")
                        }
                        EnvironmentCode::InvalidUserSymbolSpelling => {
                            (25, "invalid_user_symbol_spelling")
                        }
                        EnvironmentCode::InvalidUserSymbolArity => {
                            (26, "invalid_user_symbol_arity")
                        }
                        EnvironmentCode::ReservedWordCollision => (27, "reserved_word_collision"),
                        EnvironmentCode::ReservedSymbolCollision => {
                            (28, "reserved_symbol_collision")
                        }
                        _ => return None,
                    };
                    (number, "lexical_environment", leaf, "lexical_environment")
                }
                FrontendCode::Lexing(LexingDiagnosticKind::RawScan) => {
                    (29, "lexing", "raw_scan", "tokenization")
                }
                FrontendCode::Lexing(LexingDiagnosticKind::ScopeSkeleton(code)) => {
                    let (number, leaf) = match code {
                        ScopeSkeletonDiagnosticCode::MalformedBinderList => {
                            (30, "malformed_binder_list")
                        }
                        ScopeSkeletonDiagnosticCode::UnsupportedBinderShape => {
                            (31, "unsupported_binder_shape")
                        }
                        ScopeSkeletonDiagnosticCode::DuplicateBindingName => {
                            (32, "duplicate_binding_name")
                        }
                        ScopeSkeletonDiagnosticCode::UnmatchedEnd => (33, "unmatched_end"),
                        ScopeSkeletonDiagnosticCode::MissingEnd => (10, "missing_end"),
                        _ => return None,
                    };
                    (number, "lexing.scope_skeleton", leaf, "scope_skeleton")
                }
                FrontendCode::Lexing(LexingDiagnosticKind::Lexer(code)) => {
                    let (number, leaf) = match code {
                        LexDiagnosticCode::NoValidTokenCandidate => {
                            (34, "no_valid_token_candidate")
                        }
                        LexDiagnosticCode::ParserContextRejectedCandidate => {
                            (35, "parser_context_rejected_candidate")
                        }
                        LexDiagnosticCode::AmbiguousUserSymbol => (36, "ambiguous_user_symbol"),
                        LexDiagnosticCode::MalformedStringLiteral => {
                            (2, "malformed_string_literal")
                        }
                        LexDiagnosticCode::UnsupportedRawToken => (37, "unsupported_raw_token"),
                        _ => return None,
                    };
                    (number, "lexing.lexer", leaf, "tokenization")
                }
                FrontendCode::Syntax(key) => {
                    let number = match key.as_ref() {
                        "unexpected_error_token" => 38,
                        "dangling_operator" => 39,
                        "non_associative_operator_chain" => 40,
                        "missing_end" => 10,
                        "missing_semicolon" => 41,
                        "missing_string_literal" => 42,
                        "malformed_import" => 43,
                        "malformed_export" => 44,
                        "malformed_visibility" => 45,
                        "malformed_type_expression" => 46,
                        "malformed_term_expression" => 47,
                        "malformed_formula_expression" => 48,
                        "malformed_justification" => 49,
                        "malformed_annotation" => 50,
                        "unexpected_top_level_token" => 51,
                        "unrecoverable_input" => 52,
                        _ => return None,
                    };
                    (number, "syntax", key.as_ref(), "syntax")
                }
                _ => return None,
            };
            if class != expected_class
                && !(expected_class == "syntax" && class == "annotation_syntax")
            {
                return None;
            }
            let DiagnosticLocation::SourceRange(primary) = &diagnostic.location else {
                return None;
            };
            source.line_map.validate_range(*primary).ok()?;
            let primary = DiagnosticSpan::from_anchor(
                mizar_session::SourceAnchor::Range(*primary),
                DiagnosticSpanRole::Primary,
                None,
                SpanFreshness::Current,
                None,
            )
            .ok()?;
            let secondary = diagnostic
                .secondary
                .iter()
                .map(|anchor| {
                    let span = DiagnosticSpan::from_anchor(
                        anchor.clone(),
                        DiagnosticSpanRole::Secondary,
                        None,
                        SpanFreshness::Current,
                        None,
                    )
                    .ok()?;
                    source.line_map.validate_range(span.range()).ok()?;
                    Some(span)
                })
                .collect::<Option<Vec<_>>>()?;
            let code = DiagnosticCode::from_parts(DiagnosticSeverity::Error, number).ok()?;
            let descriptor = registry.lookup(code)?;
            let local_code = format!("{prefix}.{leaf}");
            let details = DiagnosticDetails::from_entries([
                (
                    "frontend.class",
                    DiagnosticDetailValue::String(class.to_owned()),
                ),
                ("frontend.code", DiagnosticDetailValue::String(local_code)),
            ])
            .ok()?;
            let notes = diagnostic
                .recovery_note
                .as_ref()
                .map_or_else(Vec::new, |note| {
                    vec![DiagnosticNote::new(
                        DiagnosticNoteKind::Note,
                        note.clone(),
                        None,
                    )]
                });
            DiagnosticDraft::new(DiagnosticDraftInput {
                source_snapshot: snapshot,
                code,
                phase: DiagnosticPhase::Frontend,
                category: FailureCategory::ParseError,
                stable_detail_key: descriptor.semantic_name.to_owned(),
                message: diagnostic.message.to_string(),
                primary_location: DiagnosticPrimaryLocation::Span(primary),
                secondary_spans: secondary,
                notes,
                details,
                fixes: Vec::new(),
                explanation: None,
            })
            .ok()
        })
        .collect()
}

fn unique<T>(mut items: impl Iterator<Item = T>) -> Option<T> {
    let item = items.next()?;
    items.next().is_none().then_some(item)
}

pub(crate) fn safe_relative(path: &str) -> bool {
    path == "."
        || (!path.is_empty()
            && !path.contains(['\\', ':'])
            && path
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != ".."))
}

fn blocking() -> PhaseResult {
    PhaseResult {
        status: PhaseStatus::Blocking,
        ..PhaseResult::complete()
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
    let mut size = domain.len();
    for field in fields {
        let Some(next) = u64::try_from(field.len())
            .ok()
            .and_then(|_| size.checked_add(8))
            .and_then(|size| size.checked_add(field.len()))
        else {
            return Ok(None);
        };
        if next > 32 * 1024 * 1024 {
            return Ok(None);
        }
        size = next;
    }
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(domain);
    for field in fields {
        let Ok(length) = u64::try_from(field.len()) else {
            return Ok(None);
        };
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(field);
    }
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
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) = bytes.strip_prefix(domain) else {
        return Ok(None);
    };
    let mut fields = [&[][..]; 3];
    for field in &mut fields {
        let Some(length) = remaining
            .get(..8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())
        else {
            return Ok(None);
        };
        remaining = &remaining[8..];
        let Some(value) = remaining.get(..length) else {
            return Ok(None);
        };
        *field = value;
        remaining = &remaining[length..];
    }
    if !remaining.is_empty()
        || fields[0] != module.package().as_str().as_bytes()
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

/// Stores genuine raw VCs through lossless Core storage and deterministic regeneration.
pub fn native_theorem_vc_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    core: &mizar_core::core_ir::CoreIr,
    vcs: &mizar_vc::vc_ir::VcSet,
    snapshot: mizar_session::BuildSnapshotId,
    generation_schema: &mizar_vc::vc_ir::GenerationSchemaVersion,
) -> Result<Option<Vec<u8>>, String> {
    let Some(core_bytes) = native_theorem_core_storage_bytes(ast, module, core)? else {
        return Ok(None);
    };
    let domain = b"mizar-driver/native-theorem-vc-storage/v1\0";
    let fields = [
        generation_schema.as_str().as_bytes(),
        vcs.schema_version().as_str().as_bytes(),
        core_bytes.as_slice(),
    ];
    let mut size = domain.len() as u64;
    for field in fields {
        let Some(next) = u64::try_from(field.len())
            .ok()
            .and_then(|length| size.checked_add(8)?.checked_add(length))
        else {
            return Ok(None);
        };
        if next > 32 * 1024 * 1024 {
            return Ok(None);
        }
        size = next;
    }
    let Ok(size) = usize::try_from(size) else {
        return Ok(None);
    };
    let expected = mizar_vc::generator::generate_source_theorem_vcs(
        core,
        snapshot,
        generation_schema,
        vcs.schema_version(),
    )?;
    if expected != *vcs {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(domain);
    for field in fields {
        let Ok(length) = u64::try_from(field.len()) else {
            return Ok(None);
        };
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(field);
    }
    Ok(Some(bytes))
}

/// Replays raw VCs in the caller's existing source, module and snapshot context.
pub fn native_theorem_vc_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
    snapshot: mizar_session::BuildSnapshotId,
) -> Result<Option<mizar_vc::vc_ir::VcSet>, String> {
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) = bytes.strip_prefix(b"mizar-driver/native-theorem-vc-storage/v1\0")
    else {
        return Ok(None);
    };
    let mut fields = [&[][..]; 3];
    for field in &mut fields {
        let Some(length) = remaining
            .get(..8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())
        else {
            return Ok(None);
        };
        remaining = &remaining[8..];
        let Some(value) = remaining.get(..length) else {
            return Ok(None);
        };
        *field = value;
        remaining = &remaining[length..];
    }
    if !remaining.is_empty() {
        return Ok(None);
    }
    let (Ok(generation), Ok(schema)) = (
        std::str::from_utf8(fields[0]),
        std::str::from_utf8(fields[1]),
    ) else {
        return Ok(None);
    };
    let generation = mizar_vc::vc_ir::GenerationSchemaVersion::new(generation);
    let schema = mizar_vc::vc_ir::VcSchemaVersion::new(schema);
    let Some(core) = native_theorem_core_from_storage_bytes(fields[2], source_id, module)? else {
        return Ok(None);
    };
    mizar_vc::generator::generate_source_theorem_vcs(&core, snapshot, &generation, &schema)
        .map(Some)
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
    let mut size = domain.len();
    for field in fields {
        let Some(next) = u64::try_from(field.len())
            .ok()
            .and_then(|_| size.checked_add(8))
            .and_then(|size| size.checked_add(field.len()))
        else {
            return Ok(None);
        };
        if next > 32 * 1024 * 1024 {
            return Ok(None);
        }
        size = next;
    }
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(domain);
    for field in fields {
        let Ok(length) = u64::try_from(field.len()) else {
            return Ok(None);
        };
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(field);
    }
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
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) = bytes.strip_prefix(domain) else {
        return Ok(None);
    };
    let mut fields = [&[][..]; 3];
    for field in &mut fields {
        let Some(length) = remaining
            .get(..8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())
        else {
            return Ok(None);
        };
        remaining = &remaining[8..];
        let Some(value) = remaining.get(..length) else {
            return Ok(None);
        };
        *field = value;
        remaining = &remaining[length..];
    }
    if !remaining.is_empty()
        || fields[0] != module.package().as_str().as_bytes()
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

/// Stores genuine raw algorithm VCs through lossless Core storage and regeneration.
pub fn native_algorithm_vc_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    core: &mizar_core::core_ir::CoreIr,
    vcs: &mizar_vc::vc_ir::VcSet,
    snapshot: mizar_session::BuildSnapshotId,
    generation_schema: &mizar_vc::vc_ir::GenerationSchemaVersion,
) -> Result<Option<Vec<u8>>, String> {
    let Some(core_bytes) = native_algorithm_core_storage_bytes(ast, module, core)? else {
        return Ok(None);
    };
    let domain = b"mizar-driver/native-algorithm-vc-storage/v1\0";
    let fields = [
        generation_schema.as_str().as_bytes(),
        vcs.schema_version().as_str().as_bytes(),
        core_bytes.as_slice(),
    ];
    let mut size = domain.len() as u64;
    for field in fields {
        let Some(next) = u64::try_from(field.len())
            .ok()
            .and_then(|length| size.checked_add(8)?.checked_add(length))
        else {
            return Ok(None);
        };
        if next > 32 * 1024 * 1024 {
            return Ok(None);
        }
        size = next;
    }
    let Ok(size) = usize::try_from(size) else {
        return Ok(None);
    };
    let expected = mizar_vc::generator::generate_source_algorithm_postconditions(
        core,
        snapshot,
        generation_schema,
        vcs.schema_version(),
    )?;
    if expected != *vcs {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(domain);
    for field in fields {
        bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
        bytes.extend_from_slice(field);
    }
    Ok(Some(bytes))
}

/// Rebinds stored algorithm syntax to the supplied source, module and snapshot.
pub fn native_algorithm_vc_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
    snapshot: mizar_session::BuildSnapshotId,
) -> Result<Option<mizar_vc::vc_ir::VcSet>, String> {
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) = bytes.strip_prefix(b"mizar-driver/native-algorithm-vc-storage/v1\0")
    else {
        return Ok(None);
    };
    let mut fields = [&[][..]; 3];
    for field in &mut fields {
        let Some(length) = remaining
            .get(..8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())
        else {
            return Ok(None);
        };
        remaining = &remaining[8..];
        let Some(value) = remaining.get(..length) else {
            return Ok(None);
        };
        *field = value;
        remaining = &remaining[length..];
    }
    if !remaining.is_empty() {
        return Ok(None);
    }
    let (Ok(generation), Ok(schema)) = (
        std::str::from_utf8(fields[0]),
        std::str::from_utf8(fields[1]),
    ) else {
        return Ok(None);
    };
    let generation = mizar_vc::vc_ir::GenerationSchemaVersion::new(generation);
    let schema = mizar_vc::vc_ir::VcSchemaVersion::new(schema);
    let Some(core) = native_algorithm_core_from_storage_bytes(fields[2], source_id, module)? else {
        return Ok(None);
    };
    mizar_vc::generator::generate_source_algorithm_postconditions(
        &core,
        snapshot,
        &generation,
        &schema,
    )
    .map(Some)
}

/// Stores lossless source syntax only when genuine checker replay reproduces the supplied TypedAst.
pub fn native_theorem_typed_ast_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    typed: &mizar_checker::typed_ast::TypedAst,
) -> Result<Option<Vec<u8>>, String> {
    let Some(syntax) = ast.canonical_bytes() else {
        return Ok(None);
    };
    let domain = b"mizar-driver/native-theorem-typed-ast-storage/v1\0";
    let fields = [
        module.package().as_str().as_bytes(),
        module.path().as_str().as_bytes(),
        syntax.as_slice(),
    ];
    let mut size = domain.len();
    for field in fields {
        let Some(next) = u64::try_from(field.len())
            .ok()
            .and_then(|_| size.checked_add(8))
            .and_then(|size| size.checked_add(field.len()))
        else {
            return Ok(None);
        };
        if next > 32 * 1024 * 1024 {
            return Ok(None);
        }
        size = next;
    }
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(domain);
    for field in fields {
        let Ok(length) = u64::try_from(field.len()) else {
            return Ok(None);
        };
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(field);
    }
    let Some(replayed) =
        native_theorem_typed_ast_from_storage_bytes(&bytes, ast.source_id, module)?
    else {
        return Ok(None);
    };
    Ok((replayed == *typed).then_some(bytes))
}

/// Rebinds stored syntax to an existing source id and replays the genuine Checker owners.
pub fn native_theorem_typed_ast_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
) -> Result<Option<mizar_checker::typed_ast::TypedAst>, String> {
    let domain = b"mizar-driver/native-theorem-typed-ast-storage/v1\0";
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) = bytes.strip_prefix(domain) else {
        return Ok(None);
    };
    let mut fields = [&[][..]; 3];
    for field in &mut fields {
        let Some(length) = remaining
            .get(..8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())
        else {
            return Ok(None);
        };
        remaining = &remaining[8..];
        let Some(value) = remaining.get(..length) else {
            return Ok(None);
        };
        *field = value;
        remaining = &remaining[length..];
    }
    if !remaining.is_empty()
        || fields[0] != module.package().as_str().as_bytes()
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
    let checked = mizar_checker::type_checker::DeclarationChecker::
        check_source_theorem_equalities_with_typed_declarations(&arena, &env, &scope)?;
    Ok(Some(checked.2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_frontend::lexing::{LexDiagnosticCode as L, ScopeSkeletonDiagnosticCode as S};
    use mizar_frontend::orchestration::{DiagnosticClass as C, DiagnosticCode as D};
    use mizar_frontend::preprocess::{
        ImportPrescanDiagnosticCode as I, SourcePreprocessDiagnosticCode as P,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, GeneratedSpanAnchor, GeneratedSpanOrigin,
        InMemorySessionIdAllocator, ModulePath, SessionIdAllocator, SourceAnchor, normalize_path,
    };
    use std::{
        fs,
        path::Path,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT_SOURCE: AtomicUsize = AtomicUsize::new(0);

    fn source() -> (BuildSnapshotId, SourceUnit) {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "28".repeat(32)
        ))
        .unwrap();
        let root = std::env::temp_dir().join(format!(
            "mizar-driver-frontend-converter-{}-{}",
            std::process::id(),
            NEXT_SOURCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/test.miz"), "éx\n").unwrap();
        let input = SourceInput {
            package_id: PackageId::new("test"),
            module_path: ModulePath::new("test"),
            normalized_path: normalize_path(&root, Path::new("src/test.miz")).unwrap(),
            edition: Edition::new("2025"),
            origin: SourceOriginInput::Disk {
                path: "src/test.miz".into(),
            },
        };
        let source = FrontendSourceLoader::new(DiskSourceLoader::new(&root))
            .load_source_unit(
                SourceUnitRequest { snapshot, input },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();
        fs::remove_dir_all(root).unwrap();
        (snapshot, source)
    }

    fn diagnostic(source: &SourceUnit, code: D, class: C) -> FrontendDiagnostic {
        FrontendDiagnostic {
            code,
            message: Arc::from("exact message"),
            class,
            location: DiagnosticLocation::SourceRange(mizar_session::SourceRange {
                source_id: source.source_id,
                start: 0,
                end: 2,
            }),
            secondary: Vec::new(),
            recovery_note: None,
        }
    }

    #[test]
    fn typed_frontend_categories_map_to_allocated_codes() {
        use mizar_frontend::lexical_env::LexicalEnvironmentDiagnosticCode as E;
        use mizar_frontend::lexing::LexingDiagnosticKind as K;
        use mizar_frontend::preprocess::PreprocessDiagnosticKind as R;
        let (snapshot, source) = source();
        let cases = vec![
            (
                D::Preprocess(R::SourcePrecondition(P::CarriageReturn)),
                C::LexicalPrecondition,
                13,
            ),
            (
                D::Preprocess(R::SourcePrecondition(P::NonAsciiCode)),
                C::LexicalPrecondition,
                14,
            ),
            (
                D::Preprocess(R::SourcePrecondition(P::UnterminatedMultiLineComment)),
                C::CommentStructure,
                15,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::MissingModulePath)),
                C::ImportPrescan,
                16,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::EmptyModulePathComponent)),
                C::ImportPrescan,
                17,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::MissingAlias)),
                C::ImportPrescan,
                18,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::MissingSemicolon)),
                C::ImportPrescan,
                19,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::UnexpectedToken)),
                C::ImportPrescan,
                20,
            ),
            (D::Preprocess(R::RawImportScan), C::ImportPrescan, 21),
            (
                D::LexicalEnvironment(E::UnresolvedImport),
                C::LexicalEnvironment,
                22,
            ),
            (
                D::LexicalEnvironment(E::MissingSummary),
                C::LexicalEnvironment,
                23,
            ),
            (
                D::LexicalEnvironment(E::UserSymbolImportConflict),
                C::LexicalEnvironment,
                24,
            ),
            (
                D::LexicalEnvironment(E::InvalidUserSymbolSpelling),
                C::LexicalEnvironment,
                25,
            ),
            (
                D::LexicalEnvironment(E::InvalidUserSymbolArity),
                C::LexicalEnvironment,
                26,
            ),
            (
                D::LexicalEnvironment(E::ReservedWordCollision),
                C::LexicalEnvironment,
                27,
            ),
            (
                D::LexicalEnvironment(E::ReservedSymbolCollision),
                C::LexicalEnvironment,
                28,
            ),
            (D::Lexing(K::RawScan), C::Tokenization, 29),
            (
                D::Lexing(K::ScopeSkeleton(S::MalformedBinderList)),
                C::ScopeSkeleton,
                30,
            ),
            (
                D::Lexing(K::ScopeSkeleton(S::UnsupportedBinderShape)),
                C::ScopeSkeleton,
                31,
            ),
            (
                D::Lexing(K::ScopeSkeleton(S::DuplicateBindingName)),
                C::ScopeSkeleton,
                32,
            ),
            (
                D::Lexing(K::ScopeSkeleton(S::UnmatchedEnd)),
                C::ScopeSkeleton,
                33,
            ),
            (
                D::Lexing(K::ScopeSkeleton(S::MissingEnd)),
                C::ScopeSkeleton,
                10,
            ),
            (
                D::Lexing(K::Lexer(L::NoValidTokenCandidate)),
                C::Tokenization,
                34,
            ),
            (
                D::Lexing(K::Lexer(L::ParserContextRejectedCandidate)),
                C::Tokenization,
                35,
            ),
            (
                D::Lexing(K::Lexer(L::AmbiguousUserSymbol)),
                C::Tokenization,
                36,
            ),
            (
                D::Lexing(K::Lexer(L::MalformedStringLiteral)),
                C::Tokenization,
                2,
            ),
            (
                D::Lexing(K::Lexer(L::UnsupportedRawToken)),
                C::Tokenization,
                37,
            ),
            (
                D::Syntax(Arc::from("unexpected_error_token")),
                C::Syntax,
                38,
            ),
            (D::Syntax(Arc::from("dangling_operator")), C::Syntax, 39),
            (
                D::Syntax(Arc::from("non_associative_operator_chain")),
                C::Syntax,
                40,
            ),
            (D::Syntax(Arc::from("missing_end")), C::Syntax, 10),
            (D::Syntax(Arc::from("missing_semicolon")), C::Syntax, 41),
            (
                D::Syntax(Arc::from("missing_string_literal")),
                C::Syntax,
                42,
            ),
            (D::Syntax(Arc::from("malformed_import")), C::Syntax, 43),
            (D::Syntax(Arc::from("malformed_export")), C::Syntax, 44),
            (D::Syntax(Arc::from("malformed_visibility")), C::Syntax, 45),
            (
                D::Syntax(Arc::from("malformed_type_expression")),
                C::Syntax,
                46,
            ),
            (
                D::Syntax(Arc::from("malformed_term_expression")),
                C::Syntax,
                47,
            ),
            (
                D::Syntax(Arc::from("malformed_formula_expression")),
                C::Syntax,
                48,
            ),
            (
                D::Syntax(Arc::from("malformed_justification")),
                C::Syntax,
                49,
            ),
            (
                D::Syntax(Arc::from("malformed_annotation")),
                C::AnnotationSyntax,
                50,
            ),
            (
                D::Syntax(Arc::from("unexpected_top_level_token")),
                C::Syntax,
                51,
            ),
            (D::Syntax(Arc::from("unrecoverable_input")), C::Syntax, 52),
        ];
        assert_eq!(cases.len(), 43);
        for (code, class, number) in cases {
            let drafts = convert_frontend_diagnostics(
                &source,
                snapshot,
                &[diagnostic(&source, code, class)],
            )
            .expect("typed category converts");
            assert_eq!(drafts.len(), 1);
            assert_eq!(drafts[0].code().number(), number);
            assert_eq!(drafts[0].phase(), DiagnosticPhase::Frontend);
            assert_eq!(drafts[0].message(), "exact message");
            assert!(drafts[0].details().entries().contains_key("frontend.class"));
            assert!(drafts[0].details().entries().contains_key("frontend.code"));
            if number == 15 {
                assert_eq!(
                    drafts[0].details().entries().get("frontend.code"),
                    Some(&DiagnosticDetailValue::String(
                        "preprocess.source_precondition.unterminated_multi_line_comment".to_owned()
                    ))
                );
            }
            if number == 17 {
                assert_eq!(
                    drafts[0].details().entries().get("frontend.code"),
                    Some(&DiagnosticDetailValue::String(
                        "preprocess.import_prescan.empty_module_path_component".to_owned()
                    ))
                );
            }
        }
    }

    #[test]
    fn converter_preserves_anchors_notes_and_rejects_invalid_batches_atomically() {
        let (snapshot, source) = source();
        let range = mizar_session::SourceRange {
            source_id: source.source_id,
            start: 2,
            end: 2,
        };
        let reason = "  生成 \"exact\"\n";
        let anchor = SourceAnchor::Generated(
            GeneratedSpanOrigin::new(
                GeneratedSpanAnchor::Point {
                    source_id: source.source_id,
                    offset: 2,
                },
                reason,
            )
            .unwrap(),
        );
        let generated_range = SourceAnchor::Generated(
            GeneratedSpanOrigin::new(GeneratedSpanAnchor::Range(range), reason).unwrap(),
        );
        let mut first = diagnostic(
            &source,
            D::Syntax(Arc::from("missing_semicolon")),
            C::Syntax,
        );
        first.secondary = vec![
            SourceAnchor::Range(range),
            SourceAnchor::Point {
                source_id: source.source_id,
                offset: 2,
            },
            generated_range.clone(),
            anchor.clone(),
            anchor.clone(),
        ];
        first.recovery_note = Some(String::new());
        let drafts = convert_frontend_diagnostics(&source, snapshot, &[first.clone()]).unwrap();
        assert_eq!(drafts[0].secondary_spans().len(), 5);
        assert_eq!(
            drafts[0].secondary_spans()[0].anchor(),
            &SourceAnchor::Range(range)
        );
        assert!(matches!(
            drafts[0].secondary_spans()[1].anchor(),
            SourceAnchor::Point { .. }
        ));
        assert_eq!(drafts[0].secondary_spans()[2].anchor(), &generated_range);
        assert_eq!(drafts[0].secondary_spans()[3].anchor(), &anchor);
        assert_eq!(drafts[0].secondary_spans()[4].anchor(), &anchor);
        assert_eq!(drafts[0].secondary_spans()[3].zero_width(), None);
        assert_eq!(drafts[0].notes()[0].message(), "");
        first.recovery_note = None;
        assert!(
            convert_frontend_diagnostics(&source, snapshot, &[first.clone()]).unwrap()[0]
                .notes()
                .is_empty()
        );
        let mut zero_primary = first.clone();
        zero_primary.location = DiagnosticLocation::SourceRange(range);
        let converted = convert_frontend_diagnostics(&source, snapshot, &[zero_primary]).unwrap();
        let DiagnosticPrimaryLocation::Span(span) = converted[0].primary_location() else {
            panic!("expected span");
        };
        assert_eq!(span.range(), range);
        assert_eq!(span.zero_width(), None);

        let mut invalid = first.clone();
        invalid.secondary = vec![SourceAnchor::Point {
            source_id: source.source_id,
            offset: 1,
        }];
        assert!(
            convert_frontend_diagnostics(&source, snapshot, &[first.clone(), invalid]).is_none()
        );
        let mut invalid = first.clone();
        invalid.location = DiagnosticLocation::SourceRange(mizar_session::SourceRange {
            source_id: source.source_id,
            start: 5,
            end: 5,
        });
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let ids = InMemorySessionIdAllocator::new();
        let _ = ids.next_source_id(snapshot).unwrap();
        let foreign = ids.next_source_id(snapshot).unwrap();
        let mut invalid = first.clone();
        invalid.secondary = vec![SourceAnchor::Generated(
            GeneratedSpanOrigin::new(
                GeneratedSpanAnchor::Range(mizar_session::SourceRange {
                    source_id: foreign,
                    start: 0,
                    end: 2,
                }),
                "foreign",
            )
            .unwrap(),
        )];
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let mut invalid = first.clone();
        invalid.location = DiagnosticLocation::SourceRange(mizar_session::SourceRange {
            source_id: source.source_id,
            start: 3,
            end: 2,
        });
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let mut invalid = first.clone();
        invalid.class = C::Tokenization;
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let mut invalid = first.clone();
        invalid.code = D::Syntax(Arc::from("unknown_parser_key"));
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let mut invalid = first;
        invalid.code = D::SourceLoad;
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
    }
}
