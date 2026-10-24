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
pub(crate) use mizar_frontend::cache_key::SOURCE_UNIT_CACHE_KEY_VERSION;
use mizar_frontend::{
    cache_key::{
        ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION, SourceUnitCacheKey,
        TOKEN_STREAM_CACHE_KEY_VERSION,
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

    /// Seals a genuine native theorem TypedAst with source-associated and storage bytes.
    pub fn publish_native_theorem_typed_ast(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<Option<mizar_ir::storage::PhaseOutputRef<mizar_checker::typed_ast::TypedAst>>, String>
    {
        let Some(typed) = self.native_theorem_typed_ast_with_signature_parent(input, publisher)?
        else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem TypedAst publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem TypedAst publication requires one captured source".to_owned()
        })?;
        let (source, _) = self.native_source(version).ok_or_else(|| {
            "native theorem TypedAst publication capture binding failed".to_owned()
        })?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| {
                "native theorem TypedAst publication requires a bound package".to_owned()
            })?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(
                "native theorem TypedAst publication requires a workspace package".to_owned(),
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
        let Some(storage_bytes) =
            native_theorem_typed_ast_storage_bytes(ast, &resolver_module, &typed)?
        else {
            return Ok(None);
        };
        let Some((_, _, semantic)) =
            mizar_resolve::symbols::source_theorem_module_semantic_payload(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        let domain = b"mizar-driver/native-theorem-typed-ast-content/v1\0";
        let Some(length) = u64::try_from(semantic.len()).ok() else {
            return Ok(None);
        };
        let Some(size) = domain
            .len()
            .checked_add(8)
            .and_then(|size| size.checked_add(semantic.len()))
        else {
            return Ok(None);
        };
        if size > 32 * 1024 * 1024 {
            return Ok(None);
        }
        let mut content = Vec::with_capacity(size);
        content.extend_from_slice(domain);
        content.extend_from_slice(&length.to_le_bytes());
        content.extend_from_slice(&semantic);
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-typed-ast-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("TypeChecking");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremTypedAst");
        let source_id = version.source_id;
        let parent = input.parent_outputs()[0].as_output_ref();
        publisher
            .storage()
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
                payload: typed,
                canonical_payload: Some(content),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_theorem_typed_ast_from_storage_bytes(bytes, source_id, &resolver_module)
                        .map_err(BlobDecodeError::new)?
                        .ok_or_else(|| {
                            BlobDecodeError::new("invalid native theorem TypedAst storage")
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
                        "typecheck-theorem-typed-ast-storage",
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

    /// Returns the real native registration closure bound to a published TypedAst.
    pub fn native_theorem_registration_closure_with_type_parent(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<Option<mizar_checker::cluster_trace::ClusterClosureOutput>, String> {
        if input.snapshot != self.snapshot.id {
            return Err("native theorem registration closure snapshot mismatch".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem registration closure requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem registration closure requires one captured source".to_owned()
        })?;
        let source_key = source_input_hash(version);
        if input.identities().input_hash() != source_key
            || !input.identities().dependency_hashes().is_empty()
            || !self.snapshot.dependency_artifacts.is_empty()
            || !self.module_index.dependency_summaries.is_empty()
            || input.parent_outputs().len() != 1
        {
            return Err(
                "native theorem registration closure requires the leaf input profile".to_owned(),
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
            || parent.phase() != &IrPipelinePhase::new("TypeChecking")
            || parent.work_unit() != &work_unit
            || parent.output_kind() != &OutputKind::new("SourceTheoremTypedAst")
            || parent.schema_version() != SchemaVersion::new(1)
            || parent.lineage().named_input_hashes != named_input_hashes
        {
            return Err("native theorem registration closure parent binding mismatch".to_owned());
        }
        let mut ancestry = parent.lineage().parents.clone();
        for (phase, kind, source_bound) in [
            ("SignatureCollection", "SourceTheoremModuleSummary", true),
            ("ModuleResolve", "SourceTheoremModuleCandidates", true),
            ("Frontend", "FrontendOutput", false),
        ] {
            let [ancestor] = ancestry.as_slice() else {
                return Err(
                    "native theorem registration closure requires unique ancestry".to_owned(),
                );
            };
            let ancestor = publisher
                .registry()
                .output_lineage(*ancestor)
                .ok_or_else(|| {
                    "native theorem registration closure requires registered ancestry".to_owned()
                })?;
            if ancestor.snapshot != input.snapshot
                || ancestor.phase != IrPipelinePhase::new(phase)
                || ancestor.work_unit != work_unit
                || ancestor.output_kind != OutputKind::new(kind)
                || (source_bound && ancestor.named_input_hashes != named_input_hashes)
            {
                return Err("native theorem registration closure ancestry mismatch".to_owned());
            }
            ancestry = ancestor.parents;
        }
        let typed = storage
            .typed_handle::<mizar_checker::typed_ast::TypedAst>(
                parent,
                &OutputKind::new("SourceTheoremTypedAst"),
            )
            .map_err(|error| error.to_string())?;
        let retained = storage.get(&typed).map_err(|error| error.to_string())?;
        let (source, _) = self.native_source(version).ok_or_else(|| {
            "native theorem registration closure capture binding failed".to_owned()
        })?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| {
                "native theorem registration closure requires a bound package".to_owned()
            })?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(
                "native theorem registration closure requires a workspace package".to_owned(),
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
        let closure = mizar_checker::type_checker::check_source_theorem_registration_closure(
            &arena, &env, &scope, &retained,
        )?;
        storage
            .validate_handle(parent)
            .map_err(|error| error.to_string())?;
        publisher
            .validate_published_output(input.snapshot, parent)
            .map_err(|error| error.to_string())?;
        Ok(Some(closure))
    }

    /// Seals the authenticated native theorem closure and its genuine byte streams.
    pub fn publish_native_theorem_registration_closure(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<
        Option<
            mizar_ir::storage::PhaseOutputRef<mizar_checker::cluster_trace::ClusterClosureOutput>,
        >,
        String,
    > {
        let Some(closure) =
            self.native_theorem_registration_closure_with_type_parent(input, publisher)?
        else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err(
                "native theorem registration closure publication requires a module".to_owned(),
            );
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem registration closure publication requires one captured source"
                .to_owned()
        })?;
        let (source, _) = self.native_source(version).ok_or_else(|| {
            "native theorem registration closure publication capture binding failed".to_owned()
        })?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| {
                "native theorem registration closure publication requires a bound package"
                    .to_owned()
            })?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(
                "native theorem registration closure publication requires a workspace package"
                    .to_owned(),
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
        let Some(storage_bytes) =
            native_theorem_registration_closure_storage_bytes(ast, &resolver_module, &closure)?
        else {
            return Ok(None);
        };
        let Some((_, _, semantic)) =
            mizar_resolve::symbols::source_theorem_module_semantic_payload(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        let domain = b"mizar-driver/native-theorem-registration-closure-content/v1\0";
        let fields = [
            semantic.as_slice(),
            closure
                .trace()
                .traversal_profile()
                .cache_key_material()
                .as_str()
                .as_bytes(),
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
        let mut content = Vec::with_capacity(size);
        content.extend_from_slice(domain);
        for field in fields {
            content.extend_from_slice(&(field.len() as u64).to_le_bytes());
            content.extend_from_slice(field);
        }
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-registration-closure-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("RegistrationResolution");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremRegistrationClosure");
        let source_id = version.source_id;
        let parent = input.parent_outputs()[0].as_output_ref();
        publisher
            .storage()
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
                payload: closure,
                canonical_payload: Some(content),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_theorem_registration_closure_from_storage_bytes(
                        bytes,
                        source_id,
                        &resolver_module,
                    )
                    .map_err(BlobDecodeError::new)?
                    .ok_or_else(|| {
                        BlobDecodeError::new("invalid native theorem registration closure storage")
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
                        "registration-theorem-closure-storage",
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

    /// Returns whole native metadata bound to actual Type and Registration parents.
    pub fn native_theorem_metadata_with_type_registration_parents(
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
        if input.parent_outputs().len() != 2 {
            return Err("native theorem metadata requires two parents".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem metadata requires a module".to_owned());
        };
        let typed = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("TypeChecking")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremTypedAst")
        }))
        .ok_or_else(|| "native theorem metadata requires one Type parent".to_owned())?;
        let registration = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("RegistrationResolution")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremRegistrationClosure")
        }))
        .ok_or_else(|| "native theorem metadata requires one Registration parent".to_owned())?;
        let type_ref = typed.as_output_ref();
        let registration_ref = registration.as_output_ref();
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let named = vec![NamedInputHash {
            name: "source".to_owned(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
            digest: input.identities().input_hash(),
        }];
        for parent in [type_ref, registration_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
            if parent.snapshot() != input.snapshot
                || parent.work_unit() != &work_unit
                || parent.schema_version() != SchemaVersion::new(1)
                || parent.lineage().named_input_hashes != named
            {
                return Err("native theorem metadata parent binding mismatch".to_owned());
            }
        }
        if registration_ref.lineage().parents != vec![type_ref.output()] {
            return Err("native theorem metadata Registration parent link mismatch".to_owned());
        }
        let sole_type = PhaseInput::new(
            input.work_unit.clone(),
            mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                input.identities().dependency_hashes().to_vec(),
                vec![typed.clone()],
            )
            .map_err(|error| error.to_string())?,
        );
        let Some(closure) =
            self.native_theorem_registration_closure_with_type_parent(&sole_type, publisher)?
        else {
            return Ok(None);
        };
        let registration = storage
            .typed_handle::<mizar_checker::cluster_trace::ClusterClosureOutput>(
                registration_ref,
                &OutputKind::new("SourceTheoremRegistrationClosure"),
            )
            .map_err(|error| error.to_string())?;
        let retained = storage
            .get(&registration)
            .map_err(|error| error.to_string())?;
        if *retained != closure {
            return Err("native theorem metadata complete Registration mismatch".to_owned());
        }
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem metadata requires one captured source".to_owned())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem metadata capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem metadata requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem metadata requires a workspace package".to_owned());
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
        let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(
            mizar_resolve::names::SourceVariableScopeInput::new(ast, &resolver_module, &env),
        )
        .map_err(|error| format!("{error:?}"))?;
        let metadata =
            mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)?;
        for parent in [type_ref, registration_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(metadata))
    }

    /// Seals complete authenticated native metadata with both actual parents.
    #[expect(
        clippy::type_complexity,
        reason = "preserve the existing whole metadata tuple in the sealed handle"
    )]
    pub fn publish_native_theorem_metadata(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Result<
        Option<
            mizar_ir::storage::PhaseOutputRef<(
                mizar_checker::binding_env::BindingEnv,
                mizar_checker::type_checker::TermFormulaInferenceOutput,
                mizar_checker::typed_ast::TypedAst,
                mizar_checker::resolved_typed_ast::ResolvedTypedAst,
            )>,
        >,
        String,
    > {
        let Some(metadata) =
            self.native_theorem_metadata_with_type_registration_parents(input, publisher)?
        else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem metadata publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem metadata publication requires one captured source".to_owned()
        })?;
        let (source, _) = self.native_source(version).ok_or_else(|| {
            "native theorem metadata publication capture binding failed".to_owned()
        })?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| {
                "native theorem metadata publication requires a bound package".to_owned()
            })?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(
                "native theorem metadata publication requires a workspace package".to_owned(),
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
        let Some(storage_bytes) =
            native_theorem_metadata_storage_bytes(ast, &resolver_module, &metadata)?
        else {
            return Ok(None);
        };
        let Some((_, _, semantic)) =
            mizar_resolve::symbols::source_theorem_module_semantic_payload(ast, &resolver_module)?
        else {
            return Ok(None);
        };
        let domain = b"mizar-driver/native-theorem-metadata-content/v1\0";
        let Some(length) = u64::try_from(semantic.len()).ok() else {
            return Ok(None);
        };
        let Some(size) = domain
            .len()
            .checked_add(8)
            .and_then(|size| size.checked_add(semantic.len()))
        else {
            return Ok(None);
        };
        if size > 32 * 1024 * 1024 {
            return Ok(None);
        }
        let mut content = Vec::with_capacity(size);
        content.extend_from_slice(domain);
        content.extend_from_slice(&length.to_le_bytes());
        content.extend_from_slice(&semantic);
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-metadata-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("OverloadResolution");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremMetadata");
        let source_id = version.source_id;
        let parents = input
            .parent_outputs()
            .iter()
            .map(|parent| parent.as_output_ref().clone())
            .collect::<Vec<_>>();
        for parent in &parents {
            publisher
                .storage()
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
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
                payload: metadata,
                canonical_payload: Some(content),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_theorem_metadata_from_storage_bytes(bytes, source_id, &resolver_module)
                        .map_err(BlobDecodeError::new)?
                        .ok_or_else(|| {
                            BlobDecodeError::new("invalid native theorem metadata storage")
                        })
                }),
                parents,
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
                side_tables: IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "overload-theorem-metadata-storage",
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
            .validate_published_output(input.snapshot, parent)
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
            .validate_published_output(input.snapshot, parent)
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

    /// Generates whole native VCs after authenticating Core and Preparation parents.
    pub fn native_theorem_vcs_with_core_preparation_parents(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        generation_schema: &mizar_vc::vc_ir::GenerationSchemaVersion,
        vc_schema: &mizar_vc::vc_ir::VcSchemaVersion,
    ) -> Result<Option<mizar_vc::vc_ir::VcSet>, String> {
        if input.parent_outputs().len() != 2 {
            return Err("native theorem vc requires two parents".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem vc requires a module".to_owned());
        };
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem vc requires one Core parent".to_owned())?;
        let preparation_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("AlgorithmPreparation")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremPreparation")
        }))
        .ok_or_else(|| "native theorem vc requires one Preparation parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let preparation_ref = preparation_parent.as_output_ref();
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let named = vec![NamedInputHash {
            name: "source".to_owned(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
            digest: input.identities().input_hash(),
        }];
        for parent in [core_ref, preparation_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
            if parent.snapshot() != input.snapshot
                || parent.work_unit() != &work_unit
                || parent.schema_version() != SchemaVersion::new(1)
                || parent.lineage().named_input_hashes != named
            {
                return Err("native theorem vc parent binding mismatch".to_owned());
            }
        }
        if preparation_ref.lineage().parents != vec![core_ref.output()] {
            return Err("native theorem vc Preparation parent link mismatch".to_owned());
        }
        let sole_core = PhaseInput::new(
            input.work_unit.clone(),
            mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                input.identities().dependency_hashes().to_vec(),
                vec![core_parent.clone()],
            )
            .map_err(|error| error.to_string())?,
        );
        let Some(actual_preparation) =
            self.native_theorem_preparation_with_core_parent(&sole_core, publisher)?
        else {
            return Ok(None);
        };
        let preparation = storage
            .typed_handle::<(
                mizar_core::control_flow::ControlFlowOutput,
                mizar_core::control_flow::ObligationSeedHandoff,
            )>(
                preparation_ref,
                &OutputKind::new("SourceTheoremPreparation"),
            )
            .map_err(|error| error.to_string())?;
        let retained = storage
            .get(&preparation)
            .map_err(|error| error.to_string())?;
        if *retained != actual_preparation {
            return Err("native theorem vc complete Preparation mismatch".to_owned());
        }
        let core = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                core_ref,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&core).map_err(|error| error.to_string())?;
        let vcs = mizar_vc::generator::generate_source_theorem_vcs(
            &core,
            input.snapshot,
            generation_schema,
            vc_schema,
        )?;
        for parent in [core_ref, preparation_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(vcs))
    }

    /// Seals bounded VCs with their authenticated Core and Preparation parents.
    pub fn publish_native_theorem_vcs_with_core_preparation_parents(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        generation_schema: &mizar_vc::vc_ir::GenerationSchemaVersion,
        vc_schema: &mizar_vc::vc_ir::VcSchemaVersion,
    ) -> Result<Option<mizar_ir::storage::PhaseOutputRef<mizar_vc::vc_ir::VcSet>>, String> {
        let Some(vcs) = self.native_theorem_vcs_with_core_preparation_parents(
            input,
            publisher,
            generation_schema,
            vc_schema,
        )?
        else {
            return Ok(None);
        };
        let parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem vc publication requires one Core parent".to_owned())?
        .as_output_ref();
        let storage = publisher.storage();
        let typed = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                parent,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&typed).map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem vc publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem vc publication requires one captured source".to_owned())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem vc publication capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem vc publication requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem vc publication requires a workspace package".to_owned());
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
            semantic_bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
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
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent.as_output_ref())
                .map_err(|error| error.to_string())?;
        }
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
                parents: input
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref().clone())
                    .collect(),
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
    /// Associates source-authenticated Core and VCs with an untrusted local handoff.
    pub fn native_theorem_kernel_handoff_with_core_vc_parents(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
    ) -> Result<Option<mizar_vc::kernel_evidence_handoff::VcKernelEvidenceHandoff>, String> {
        if input.parent_outputs().len() != 2 {
            return Err("native theorem kernel handoff requires two parents".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem kernel handoff requires a module".to_owned());
        };
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem kernel handoff requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem kernel handoff requires one VC parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let vc_ref = vc_parent.as_output_ref();
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let named = vec![NamedInputHash {
            name: "source".to_owned(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
            digest: input.identities().input_hash(),
        }];
        for parent in [core_ref, vc_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
            if parent.snapshot() != input.snapshot
                || parent.work_unit() != &work_unit
                || parent.schema_version() != SchemaVersion::new(1)
                || parent.lineage().named_input_hashes != named
            {
                return Err("native theorem kernel handoff parent binding mismatch".to_owned());
            }
        }
        if vc_ref.lineage().parents.len() != 2
            || !vc_ref.lineage().parents.contains(&core_ref.output())
        {
            return Err(
                "native theorem kernel handoff requires Core and Preparation ancestry".to_owned(),
            );
        }
        let preparation = vc_ref
            .lineage()
            .parents
            .iter()
            .find(|id| **id != core_ref.output())
            .and_then(|id| publisher.registry().output_lineage(*id))
            .ok_or_else(|| {
                "native theorem kernel handoff requires registered Preparation ancestry".to_owned()
            })?;
        if preparation.phase != IrPipelinePhase::new("AlgorithmPreparation")
            || preparation.output_kind != OutputKind::new("SourceTheoremPreparation")
            || preparation.snapshot != input.snapshot
            || preparation.work_unit != work_unit
            || preparation.named_input_hashes != named
            || preparation.parents != vec![core_ref.output()]
        {
            return Err("native theorem kernel handoff Preparation ancestry mismatch".to_owned());
        }
        let sole_core = PhaseInput::new(
            input.work_unit.clone(),
            mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                input.identities().dependency_hashes().to_vec(),
                vec![core_parent.clone()],
            )
            .map_err(|error| error.to_string())?,
        );
        if self
            .native_theorem_preparation_with_core_parent(&sole_core, publisher)?
            .is_none()
        {
            return Ok(None);
        }
        let core = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                core_ref,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&core).map_err(|error| error.to_string())?;
        let vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, &OutputKind::new("SourceTheoremVC"))
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&vcs).map_err(|error| error.to_string())?;
        if vcs.snapshot() != input.snapshot {
            return Err("native theorem kernel handoff VC snapshot mismatch".to_owned());
        }
        let handoff = mizar_vc::kernel_evidence_handoff::build_source_theorem_kernel_handoff(
            &core, &vcs, vc_id,
        )?;
        for parent in [core_ref, vc_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(handoff))
    }

    /// Associates one genuine fresh reflexivity check with both retained original graphs.
    pub fn native_theorem_reflexivity_with_core_vc_parents(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        policy: &mizar_proof::policy::VerifierPolicy,
    ) -> Result<
        Option<(
            mizar_kernel::checker::KernelCheckResult,
            mizar_proof::policy::PolicyDecision,
        )>,
        String,
    > {
        if self
            .native_theorem_kernel_handoff_with_core_vc_parents(input, publisher, vc_id)?
            .is_none()
        {
            return Ok(None);
        }
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem reflexivity requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem reflexivity requires one VC parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let vc_ref = vc_parent.as_output_ref();
        let storage = publisher.storage();
        let core = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                core_ref,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&core).map_err(|error| error.to_string())?;
        let vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, &OutputKind::new("SourceTheoremVC"))
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&vcs).map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem reflexivity requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem reflexivity requires one captured source".to_owned())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem reflexivity capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem reflexivity requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem reflexivity requires a workspace package".to_owned());
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
        let (checked_core, checked_vcs, result, decision) =
            mizar_proof::status::check_source_theorem_reflexivity_with_original_graph(
                &arena,
                &check,
                &env,
                input.snapshot,
                vc_id,
                policy,
            )?;
        if checked_core != *core || checked_vcs != *vcs {
            return Err("native theorem reflexivity original Core/VC mismatch".to_owned());
        }
        for parent in [core_ref, vc_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some((result, decision)))
    }

    /// Runs actual deterministic discharge after authenticating both original parents.
    pub fn native_theorem_discharge_with_core_vc_parents(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        policy: &mizar_vc::discharge::DischargePolicy,
    ) -> Result<Option<mizar_vc::discharge::DischargeOutput>, String> {
        if self
            .native_theorem_kernel_handoff_with_core_vc_parents(input, publisher, vc_id)?
            .is_none()
        {
            return Ok(None);
        }
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem discharge requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem discharge requires one VC parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let vc_ref = vc_parent.as_output_ref();
        let storage = publisher.storage();
        let vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, &OutputKind::new("SourceTheoremVC"))
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&vcs).map_err(|error| error.to_string())?;
        let output = mizar_vc::discharge::try_discharge(mizar_vc::discharge::DischargeInput {
            vc_set: &vcs,
            policy,
        })
        .map_err(|error| error.to_string())?;
        for parent in [core_ref, vc_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(output))
    }

    /// Publishes genuine module discharge data with its actual Core and original VC parents.
    pub fn publish_native_theorem_discharge(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        policy: &mizar_vc::discharge::DischargePolicy,
    ) -> Result<
        Option<mizar_ir::storage::PhaseOutputRef<mizar_vc::discharge::DischargeOutput>>,
        String,
    > {
        let Some(discharge) =
            self.native_theorem_discharge_with_core_vc_parents(input, publisher, vc_id, policy)?
        else {
            return Ok(None);
        };
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem discharge publication requires one Core parent".to_owned())?
        .as_output_ref();
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem discharge publication requires one VC parent".to_owned())?
        .as_output_ref();
        let storage = publisher.storage();
        let typed = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                core_parent,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&typed).map_err(|error| error.to_string())?;
        let typed_vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_parent, &OutputKind::new("SourceTheoremVC"))
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&typed_vcs).map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem discharge publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem discharge publication requires one captured source".to_owned()
        })?;
        let (source, _) = self.native_source(version).ok_or_else(|| {
            "native theorem discharge publication capture binding failed".to_owned()
        })?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| {
                "native theorem discharge publication requires a bound package".to_owned()
            })?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(
                "native theorem discharge publication requires a workspace package".to_owned(),
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
        let Some(core_semantic_bytes) =
            native_theorem_core_semantic_payload(ast, &resolver_module, &core)?
        else {
            return Ok(None);
        };
        let generation_schema = &vcs
            .vc(vc_id)
            .ok_or_else(|| {
                "native theorem discharge publication requires the selected VC".to_owned()
            })?
            .anchor
            .generation_schema_version;
        let limit = policy.computation_limit.max_steps.to_le_bytes();
        let domain = b"mizar-driver/native-theorem-discharge-content/v1\0";
        let fields = [
            core_semantic_bytes.as_slice(),
            generation_schema.as_str().as_bytes(),
            vcs.schema_version().as_str().as_bytes(),
            policy.computation_limit.policy.as_str().as_bytes(),
            limit.as_slice(),
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
        let mut semantic_bytes = Vec::with_capacity(size);
        semantic_bytes.extend_from_slice(domain);
        for field in fields {
            semantic_bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
            semantic_bytes.extend_from_slice(field);
        }
        let Some(storage_bytes) = native_theorem_discharge_storage_bytes(
            ast,
            &resolver_module,
            &core,
            mizar_vc::discharge::DischargeInput {
                vc_set: &vcs,
                policy,
            },
            &discharge,
            generation_schema,
        )?
        else {
            return Ok(None);
        };
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-discharge-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("VcDischarge");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremDischarge");
        let source_id = version.source_id;
        let snapshot = input.snapshot;
        let decode_policy = policy.clone();
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent.as_output_ref())
                .map_err(|error| error.to_string())?;
        }
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
                payload: discharge,
                canonical_payload: Some(semantic_bytes),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_theorem_discharge_from_storage_bytes(
                        bytes,
                        source_id,
                        &resolver_module,
                        snapshot,
                        &decode_policy,
                    )
                    .map_err(BlobDecodeError::new)?
                    .ok_or_else(|| BlobDecodeError::new("invalid native theorem discharge storage"))
                }),
                parents: input
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref().clone())
                    .collect(),
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
                side_tables: IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "discharge-theorem-storage",
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

    /// Produces untrusted theorem candidates after authenticating all three retained parents.
    pub fn native_theorem_candidate_with_core_vc_discharge_parents(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        policy: &mizar_vc::discharge::DischargePolicy,
        logic_profile: mizar_atp::problem::LogicProfile,
        concrete_format: mizar_atp::problem::ConcreteFormat,
    ) -> Result<
        Option<(
            mizar_atp::backend::EncodedBackendProblem,
            mizar_atp::backend::BackendCandidateEvidence,
        )>,
        String,
    > {
        if input.parent_outputs().len() != 3 {
            return Err("native theorem candidate requires three parents".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem candidate requires a module".to_owned());
        };
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem candidate requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem candidate requires one VC parent".to_owned())?;
        let discharge_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcDischarge")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremDischarge")
        }))
        .ok_or_else(|| "native theorem candidate requires one Discharge parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let vc_ref = vc_parent.as_output_ref();
        let discharge_ref = discharge_parent.as_output_ref();
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let named = vec![NamedInputHash {
            name: "source".to_owned(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
            digest: input.identities().input_hash(),
        }];
        for parent in [core_ref, vc_ref, discharge_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
            if parent.snapshot() != input.snapshot
                || parent.work_unit() != &work_unit
                || parent.schema_version() != SchemaVersion::new(1)
                || parent.lineage().named_input_hashes != named
            {
                return Err("native theorem candidate parent binding mismatch".to_owned());
            }
        }
        if discharge_ref.lineage().parents.len() != 2
            || !discharge_ref.lineage().parents.contains(&core_ref.output())
            || !discharge_ref.lineage().parents.contains(&vc_ref.output())
        {
            return Err("native theorem candidate Discharge parent link mismatch".to_owned());
        }
        let originals = PhaseInput::new(
            input.work_unit.clone(),
            mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                input.identities().dependency_hashes().to_vec(),
                vec![core_parent.clone(), vc_parent.clone()],
            )
            .map_err(|error| error.to_string())?,
        );
        let Some(actual_discharge) = self
            .native_theorem_discharge_with_core_vc_parents(&originals, publisher, vc_id, policy)?
        else {
            return Ok(None);
        };
        let discharge = storage
            .typed_handle::<mizar_vc::discharge::DischargeOutput>(
                discharge_ref,
                &OutputKind::new("SourceTheoremDischarge"),
            )
            .map_err(|error| error.to_string())?;
        let discharge = storage.get(&discharge).map_err(|error| error.to_string())?;
        if *discharge != actual_discharge {
            return Err("native theorem candidate complete Discharge mismatch".to_owned());
        }
        let core = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                core_ref,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&core).map_err(|error| error.to_string())?;
        let vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, &OutputKind::new("SourceTheoremVC"))
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&vcs).map_err(|error| error.to_string())?;
        let pair = mizar_atp::backend::build_source_theorem_candidate_inputs(
            &core,
            &vcs,
            vc_id,
            &discharge,
            policy,
            logic_profile,
            concrete_format,
        )?;
        for parent in [core_ref, vc_ref, discharge_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(pair))
    }

    /// Seals the actual untrusted candidate pair with coherent semantic and storage streams.
    pub fn publish_native_theorem_candidate(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        policy: &mizar_vc::discharge::DischargePolicy,
        logic_profile: mizar_atp::problem::LogicProfile,
        concrete_format: mizar_atp::problem::ConcreteFormat,
    ) -> Result<
        Option<
            mizar_ir::storage::PhaseOutputRef<(
                mizar_atp::backend::EncodedBackendProblem,
                mizar_atp::backend::BackendCandidateEvidence,
            )>,
        >,
        String,
    > {
        let Some(pair) = self.native_theorem_candidate_with_core_vc_discharge_parents(
            input,
            publisher,
            vc_id,
            policy,
            logic_profile.clone(),
            concrete_format,
        )?
        else {
            return Ok(None);
        };
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem candidate publication requires one Core parent".to_owned())?
        .as_output_ref();
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem candidate publication requires one VC parent".to_owned())?
        .as_output_ref();
        let storage = publisher.storage();
        let typed = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                core_parent,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&typed).map_err(|error| error.to_string())?;
        let typed_vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_parent, &OutputKind::new("SourceTheoremVC"))
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&typed_vcs).map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem candidate publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem candidate publication requires one captured source".to_owned()
        })?;
        let (source, _) = self.native_source(version).ok_or_else(|| {
            "native theorem candidate publication capture binding failed".to_owned()
        })?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| {
                "native theorem candidate publication requires a bound package".to_owned()
            })?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(
                "native theorem candidate publication requires a workspace package".to_owned(),
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
        let Some(core_semantic_bytes) =
            native_theorem_core_semantic_payload(ast, &resolver_module, &core)?
        else {
            return Ok(None);
        };
        let generation_schema = &vcs
            .vc(vc_id)
            .ok_or_else(|| {
                "native theorem candidate publication requires the selected VC".to_owned()
            })?
            .anchor
            .generation_schema_version;
        let Some(storage_bytes) = native_theorem_candidate_storage_bytes(
            ast,
            &resolver_module,
            &core,
            (
                input.snapshot,
                generation_schema,
                vcs.schema_version(),
                vc_id,
            ),
            (policy, &logic_profile, concrete_format),
            &pair,
        )?
        else {
            return Ok(None);
        };
        let Some(remaining) =
            storage_bytes.strip_prefix(b"mizar-driver/native-theorem-candidate-storage/v1\0")
        else {
            return Ok(None);
        };
        let Some(core_length) = remaining
            .get(..8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())
        else {
            return Ok(None);
        };
        let Some(config) = remaining
            .get(8..)
            .and_then(|bytes| bytes.get(core_length..))
        else {
            return Ok(None);
        };
        let domain = b"mizar-driver/native-theorem-candidate-content/v1\0";
        let fields = [core_semantic_bytes.as_slice(), config];
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
        let mut semantic_bytes = Vec::with_capacity(size);
        semantic_bytes.extend_from_slice(domain);
        for field in fields {
            semantic_bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
            semantic_bytes.extend_from_slice(field);
        }
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-candidate-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("AtpSolve");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremCandidateInputs");
        let source_id = version.source_id;
        let snapshot = input.snapshot;
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent.as_output_ref())
                .map_err(|error| error.to_string())?;
        }
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
                    native_theorem_candidate_from_storage_bytes(
                        bytes,
                        source_id,
                        &resolver_module,
                        snapshot,
                    )
                    .map_err(BlobDecodeError::new)?
                    .ok_or_else(|| BlobDecodeError::new("invalid native theorem candidate storage"))
                }),
                parents: input
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref().clone())
                    .collect(),
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
                side_tables: IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "candidate-theorem-storage",
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

    /// Associates one fresh source candidate check with all four retained parents.
    pub fn native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
    ) -> Result<
        Option<(
            mizar_kernel::checker::KernelCheckResult,
            mizar_proof::policy::PolicyDecision,
        )>,
        String,
    > {
        if input.parent_outputs().len() != 4 {
            return Err("native theorem candidate check requires four parents".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem candidate check requires a module".to_owned());
        };
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem candidate check requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem candidate check requires one VC parent".to_owned())?;
        let discharge_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcDischarge")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremDischarge")
        }))
        .ok_or_else(|| "native theorem candidate check requires one Discharge parent".to_owned())?;
        let atp_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("AtpSolve")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremCandidateInputs")
        }))
        .ok_or_else(|| "native theorem candidate check requires one ATP parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let vc_ref = vc_parent.as_output_ref();
        let discharge_ref = discharge_parent.as_output_ref();
        let atp_ref = atp_parent.as_output_ref();
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let named = vec![NamedInputHash {
            name: "source".to_owned(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
            digest: input.identities().input_hash(),
        }];
        for parent in [core_ref, vc_ref, discharge_ref, atp_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
            if parent.snapshot() != input.snapshot
                || parent.work_unit() != &work_unit
                || parent.schema_version() != SchemaVersion::new(1)
                || parent.lineage().named_input_hashes != named
            {
                return Err("native theorem candidate check parent binding mismatch".to_owned());
            }
        }
        if atp_ref.lineage().parents.len() != 3
            || !atp_ref.lineage().parents.contains(&core_ref.output())
            || !atp_ref.lineage().parents.contains(&vc_ref.output())
            || !atp_ref.lineage().parents.contains(&discharge_ref.output())
        {
            return Err("native theorem candidate check ATP parent link mismatch".to_owned());
        }
        let originals = PhaseInput::new(
            input.work_unit.clone(),
            mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                input.identities().dependency_hashes().to_vec(),
                vec![
                    core_parent.clone(),
                    vc_parent.clone(),
                    discharge_parent.clone(),
                ],
            )
            .map_err(|error| error.to_string())?,
        );
        let Some(expected_pair) = self.native_theorem_candidate_with_core_vc_discharge_parents(
            &originals,
            publisher,
            vc_id,
            policies.0,
            encoding.0.clone(),
            encoding.1,
        )?
        else {
            return Ok(None);
        };
        let pair = storage
            .typed_handle::<(
                mizar_atp::backend::EncodedBackendProblem,
                mizar_atp::backend::BackendCandidateEvidence,
            )>(atp_ref, &OutputKind::new("SourceTheoremCandidateInputs"))
            .map_err(|error| error.to_string())?;
        let pair = storage.get(&pair).map_err(|error| error.to_string())?;
        if *pair != expected_pair {
            return Err("native theorem candidate check complete ATP mismatch".to_owned());
        }
        let core = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                core_ref,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&core).map_err(|error| error.to_string())?;
        let vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, &OutputKind::new("SourceTheoremVC"))
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&vcs).map_err(|error| error.to_string())?;
        let discharge = storage
            .typed_handle::<mizar_vc::discharge::DischargeOutput>(
                discharge_ref,
                &OutputKind::new("SourceTheoremDischarge"),
            )
            .map_err(|error| error.to_string())?;
        let discharge = storage.get(&discharge).map_err(|error| error.to_string())?;
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| "native theorem candidate check requires one captured source".to_owned())?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| "native theorem candidate check capture binding failed".to_owned())?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| "native theorem candidate check requires a bound package".to_owned())?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err("native theorem candidate check requires a workspace package".to_owned());
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
        let generation = &vcs
            .vc(vc_id)
            .ok_or_else(|| "native theorem candidate check requires the selected VC".to_owned())?
            .anchor
            .generation_schema_version;
        let (checked_core, checked_vcs, checked_discharge, encoded, candidate, result, decision) =
            mizar_proof::status::check_source_theorem_candidate(
                &arena,
                &check,
                &env,
                input.snapshot,
                vc_id,
                (generation, vcs.schema_version(), encoding.0, encoding.1),
                policies,
            )?;
        if checked_core != *core
            || checked_vcs != *vcs
            || checked_discharge != *discharge
            || (encoded, candidate) != *pair
        {
            return Err("native theorem candidate check complete pipeline mismatch".to_owned());
        }
        for parent in [core_ref, vc_ref, discharge_ref, atp_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some((result, decision)))
    }

    /// Seals actual native theorem checking data with owned replay context.
    pub fn publish_native_theorem_candidate_check(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
    ) -> Result<
        Option<
            mizar_ir::storage::PhaseOutputRef<(
                mizar_kernel::checker::KernelCheckResult,
                mizar_proof::policy::PolicyDecision,
            )>,
        >,
        String,
    > {
        let Some(checked) = self
            .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                input,
                publisher,
                vc_id,
                (encoding.0.clone(), encoding.1),
                policies,
            )?
        else {
            return Ok(None);
        };
        let (logic_profile, concrete_format) = encoding;
        let (policy, verifier) = policies;
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| {
            "native theorem candidate check publication requires one Core parent".to_owned()
        })?
        .as_output_ref();
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| {
            "native theorem candidate check publication requires one VC parent".to_owned()
        })?
        .as_output_ref();
        let atp_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("AtpSolve")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremCandidateInputs")
        }))
        .ok_or_else(|| {
            "native theorem candidate check publication requires one ATP parent".to_owned()
        })?
        .as_output_ref();
        let storage = publisher.storage();
        let typed = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                core_parent,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&typed).map_err(|error| error.to_string())?;
        let typed_vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_parent, &OutputKind::new("SourceTheoremVC"))
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&typed_vcs).map_err(|error| error.to_string())?;
        let typed_pair = storage
            .typed_handle::<(
                mizar_atp::backend::EncodedBackendProblem,
                mizar_atp::backend::BackendCandidateEvidence,
            )>(atp_parent, &OutputKind::new("SourceTheoremCandidateInputs"))
            .map_err(|error| error.to_string())?;
        let pair = storage
            .get(&typed_pair)
            .map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem candidate check publication requires a module".to_owned());
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem candidate check publication requires one captured source".to_owned()
        })?;
        let (source, _) = self.native_source(version).ok_or_else(|| {
            "native theorem candidate check publication capture binding failed".to_owned()
        })?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| {
                "native theorem candidate check publication requires a bound package".to_owned()
            })?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(
                "native theorem candidate check publication requires a workspace package"
                    .to_owned(),
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
        let Some(core_semantic_bytes) =
            native_theorem_core_semantic_payload(ast, &resolver_module, &core)?
        else {
            return Ok(None);
        };
        let generation_schema = &vcs
            .vc(vc_id)
            .ok_or_else(|| {
                "native theorem candidate check publication requires the selected VC".to_owned()
            })?
            .anchor
            .generation_schema_version;
        let Some(storage_bytes) = native_theorem_candidate_check_storage_bytes(
            ast,
            &resolver_module,
            &core,
            (
                input.snapshot,
                generation_schema,
                vcs.schema_version(),
                vc_id,
            ),
            (policy, &logic_profile, concrete_format),
            verifier,
            (&pair, &checked),
        )?
        else {
            return Ok(None);
        };
        let Some(remaining) =
            storage_bytes.strip_prefix(b"mizar-driver/native-theorem-candidate-check-storage/v1\0")
        else {
            return Ok(None);
        };
        let Some(candidate_length) = remaining
            .get(..8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())
        else {
            return Ok(None);
        };
        let Some(candidate_bytes) = remaining
            .get(8..)
            .and_then(|bytes| bytes.get(..candidate_length))
        else {
            return Ok(None);
        };
        let Some(remaining) =
            candidate_bytes.strip_prefix(b"mizar-driver/native-theorem-candidate-storage/v1\0")
        else {
            return Ok(None);
        };
        let Some(core_length) = remaining
            .get(..8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())
        else {
            return Ok(None);
        };
        let Some(config) = remaining
            .get(8..)
            .and_then(|bytes| bytes.get(core_length..))
        else {
            return Ok(None);
        };
        let domain = b"mizar-driver/native-theorem-candidate-check-content/v1\0";
        let fingerprint = verifier.policy_fingerprint();
        let fields = [
            core_semantic_bytes.as_slice(),
            config,
            fingerprint.as_bytes().as_slice(),
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
        let mut semantic_bytes = Vec::with_capacity(size);
        semantic_bytes.extend_from_slice(domain);
        for field in fields {
            semantic_bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
            semantic_bytes.extend_from_slice(field);
        }
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-candidate-check-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("KernelCheck");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let output_kind = OutputKind::new("SourceTheoremCandidateCheck");
        let decode_ast = ast.clone();
        let decode_generation = generation_schema.clone();
        let decode_schema = vcs.schema_version().clone();
        let decode_policy = policy.clone();
        let decode_verifier = verifier.clone();
        let snapshot = input.snapshot;
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent.as_output_ref())
                .map_err(|error| error.to_string())?;
        }
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
                payload: checked,
                canonical_payload: Some(semantic_bytes),
                storage_payload: Some(storage_bytes),
                decode: BlobDecoder::new(move |bytes| {
                    native_theorem_candidate_check_from_storage_bytes(
                        bytes,
                        &decode_ast,
                        &resolver_module,
                        (snapshot, &decode_generation, &decode_schema, vc_id),
                        (&decode_policy, &logic_profile, concrete_format),
                        &decode_verifier,
                    )
                    .map_err(BlobDecodeError::new)?
                    .ok_or_else(|| {
                        BlobDecodeError::new("invalid native theorem candidate check storage")
                    })
                }),
                parents: input
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref().clone())
                    .collect(),
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
                side_tables: IrSideTables {
                    source_maps: vec![SideTableRecord::new(
                        "candidate-check-theorem-storage",
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

    /// Associates fresh source selection with four current Core, VC, discharge and Kernel outputs.
    pub fn native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
    ) -> Result<Option<mizar_proof::selection::ProofSelection>, String> {
        if input.parent_outputs().len() != 4 {
            return Err("native theorem candidate selection requires four parents".to_owned());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem candidate selection requires a module".to_owned());
        };
        let core_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("Elaboration")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem candidate selection requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem candidate selection requires one VC parent".to_owned())?;
        let discharge_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcDischarge")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremDischarge")
        }))
        .ok_or_else(|| {
            "native theorem candidate selection requires one Discharge parent".to_owned()
        })?;
        let kernel_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("KernelCheck")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremCandidateCheck")
        }))
        .ok_or_else(|| {
            "native theorem candidate selection requires one Kernel parent".to_owned()
        })?;
        let core_ref = core_parent.as_output_ref();
        let vc_ref = vc_parent.as_output_ref();
        let discharge_ref = discharge_parent.as_output_ref();
        let kernel_ref = kernel_parent.as_output_ref();
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let named = vec![NamedInputHash {
            name: "source".to_owned(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
            digest: input.identities().input_hash(),
        }];
        for parent in [core_ref, vc_ref, discharge_ref, kernel_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
            if parent.snapshot() != input.snapshot
                || parent.work_unit() != &work_unit
                || parent.schema_version() != SchemaVersion::new(1)
                || parent.lineage().named_input_hashes != named
            {
                return Err("native theorem candidate selection parent binding mismatch".to_owned());
            }
        }
        let retained = [core_ref.output(), vc_ref.output(), discharge_ref.output()];
        if kernel_ref.lineage().parents.len() != 4
            || !retained
                .iter()
                .all(|id| kernel_ref.lineage().parents.contains(id))
        {
            return Err(
                "native theorem candidate selection Kernel parent link mismatch".to_owned(),
            );
        }
        let extra = unique(
            kernel_ref
                .lineage()
                .parents
                .iter()
                .filter(|id| !retained.contains(id)),
        )
        .ok_or_else(|| {
            "native theorem candidate selection Kernel parent link mismatch".to_owned()
        })?;
        let lineage = publisher
            .registry()
            .output_lineage(*extra)
            .ok_or_else(|| "native theorem candidate selection ATP lineage mismatch".to_owned())?;
        if lineage.snapshot != input.snapshot
            || lineage.phase != IrPipelinePhase::new("AtpSolve")
            || lineage.output_kind != OutputKind::new("SourceTheoremCandidateInputs")
            || lineage.work_unit != work_unit
            || lineage.named_input_hashes != named
            || lineage.parents.len() != 3
            || !retained.iter().all(|id| lineage.parents.contains(id))
        {
            return Err("native theorem candidate selection ATP lineage mismatch".to_owned());
        }
        let originals = PhaseInput::new(
            input.work_unit.clone(),
            mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                input.identities().dependency_hashes().to_vec(),
                vec![
                    core_parent.clone(),
                    vc_parent.clone(),
                    discharge_parent.clone(),
                ],
            )
            .map_err(|error| error.to_string())?,
        );
        let Some(expected_pair) = self.native_theorem_candidate_with_core_vc_discharge_parents(
            &originals,
            publisher,
            vc_id,
            policies.0,
            encoding.0.clone(),
            encoding.1,
        )?
        else {
            return Ok(None);
        };
        let checked = storage
            .typed_handle::<(
                mizar_kernel::checker::KernelCheckResult,
                mizar_proof::policy::PolicyDecision,
            )>(kernel_ref, &OutputKind::new("SourceTheoremCandidateCheck"))
            .map_err(|error| error.to_string())?;
        let checked = storage.get(&checked).map_err(|error| error.to_string())?;
        let core = storage
            .typed_handle::<mizar_core::core_ir::CoreIr>(
                core_ref,
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&core).map_err(|error| error.to_string())?;
        let vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, &OutputKind::new("SourceTheoremVC"))
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&vcs).map_err(|error| error.to_string())?;
        let discharge = storage
            .typed_handle::<mizar_vc::discharge::DischargeOutput>(
                discharge_ref,
                &OutputKind::new("SourceTheoremDischarge"),
            )
            .map_err(|error| error.to_string())?;
        let discharge = storage.get(&discharge).map_err(|error| error.to_string())?;
        let Some((arena, env, _, typed, _)) =
            self.native_theorem_source_metadata(module, "native theorem candidate selection")?
        else {
            return Ok(None);
        };
        let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(&arena, &env)?;
        let check =
            mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena, &typed, &scope, &env, &labels, &resolved, None,
            )?;
        let generation = &vcs
            .vc(vc_id)
            .ok_or_else(|| {
                "native theorem candidate selection requires the selected VC".to_owned()
            })?
            .anchor
            .generation_schema_version;
        let (
            checked_core,
            checked_vcs,
            checked_discharge,
            encoded,
            candidate,
            result,
            decision,
            selection,
        ) = mizar_proof::status::select_source_theorem_candidate(
            &arena,
            &check,
            &env,
            input.snapshot,
            vc_id,
            (generation, vcs.schema_version(), encoding.0, encoding.1),
            policies,
        )?;
        if checked_core != *core
            || checked_vcs != *vcs
            || checked_discharge != *discharge
            || (encoded, candidate) != expected_pair
            || (result, decision) != *checked
        {
            return Err("native theorem candidate selection complete pipeline mismatch".to_owned());
        }
        for parent in [core_ref, vc_ref, discharge_ref, kernel_ref] {
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(selection))
    }

    /// Associates a genuine witness file with current physical theorem inputs.
    pub fn native_theorem_candidate_witness_file_with_core_vc_discharge_check_parents(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
        witness_path: &str,
    ) -> Result<
        Option<(
            mizar_artifact::store::CanonicalJson,
            mizar_artifact::proof_witness::ProofWitnessRef,
        )>,
        String,
    > {
        self.native_theorem_candidate_witness_payload(
            input,
            publisher,
            vc_id,
            encoding,
            policies,
            witness_path,
        )
        .map(|value| value.map(|(_, (_, file, reference))| (file, reference)))
    }

    /// Publishes genuine witness file data as a child of the actual Kernel output.
    pub fn publish_native_theorem_candidate_witness_file(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
        witness_path: &str,
    ) -> Result<
        Option<mizar_ir::storage::PhaseOutputRef<mizar_artifact::store::CanonicalJson>>,
        String,
    > {
        let Some((_, (_, file, reference))) = self.native_theorem_candidate_witness_payload(
            input,
            publisher,
            vc_id,
            encoding,
            policies,
            witness_path,
        )?
        else {
            return Ok(None);
        };
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem candidate witness file requires a module".to_owned());
        };
        let kernel = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("KernelCheck")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremCandidateCheck")
        }))
        .ok_or_else(|| {
            "native theorem candidate witness file requires one Kernel parent".to_owned()
        })?;
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| {
            "native theorem candidate witness file requires one captured source".to_owned()
        })?;
        let bytes = mizar_artifact::store::canonical_json_bytes(&file);
        let expected = reference.witness_artifact_hash;
        let domain = mizar_artifact::store::artifact_hash_domain(
            expected.schema_family,
            expected.schema_version,
        );
        let expected = expected.digest;
        let path = witness_path.to_owned();
        let phase = IrPipelinePhase::new("KernelCheck");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let kind = OutputKind::new("SourceTheoremCandidateWitnessFile");
        for parent in input.parent_outputs() {
            publisher
                .storage()
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent.as_output_ref())
                .map_err(|error| error.to_string())?;
        }
        publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    input.snapshot,
                    phase.clone(),
                    work_unit.clone(),
                    kind.clone(),
                    SchemaVersion::new(1),
                ),
                snapshot: input.snapshot,
                phase,
                work_unit,
                output_kind: kind,
                schema_version: SchemaVersion::new(1),
                payload: file,
                canonical_payload: Some(bytes.clone()),
                storage_payload: Some(bytes),
                decode: BlobDecoder::new(move |bytes| {
                    let value = mizar_artifact::store::parse_canonical_json_bytes(bytes, &path)
                        .map_err(|error| BlobDecodeError::new(error.to_string()))?;
                    if domain.hash(&value, &[]) != expected {
                        return Err(BlobDecodeError::new(
                            "invalid native theorem candidate witness file storage",
                        ));
                    }
                    Ok(value)
                }),
                parents: vec![kernel.as_output_ref().clone()],
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }],
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

    #[expect(
        clippy::type_complexity,
        reason = "keeps actual source and checker products together"
    )]
    fn native_theorem_source_metadata(
        self,
        module: &mizar_build::module_index::ModuleId,
        prefix: &str,
    ) -> Result<
        Option<(
            mizar_resolve::resolved_ast::SurfaceResolvedArena,
            mizar_resolve::env::SymbolEnv,
            mizar_checker::type_checker::TermFormulaInferenceOutput,
            mizar_checker::typed_ast::TypedAst,
            mizar_checker::resolved_typed_ast::ResolvedTypedAst,
        )>,
        String,
    > {
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| format!("{prefix} requires one captured source"))?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| format!("{prefix} capture binding failed"))?;
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id == module.package)
            .ok_or_else(|| format!("{prefix} requires a bound package"))?;
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return Err(format!("{prefix} requires a workspace package"));
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
        let metadata =
            mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)?;
        Ok(Some((arena, env, metadata.1, metadata.2, metadata.3)))
    }

    #[expect(
        clippy::type_complexity,
        reason = "retains genuine selection and complete witness products"
    )]
    fn native_theorem_candidate_witness_payload(
        self,
        input: &PhaseInput,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
        witness_path: &str,
    ) -> Result<
        Option<(
            mizar_proof::selection::ProofSelection,
            (
                (
                    mizar_proof::status::ProofStatusProjection,
                    mizar_vc::dependency_slice::DependencySlice,
                    mizar_proof::witness_store::ProofWitnessDraft,
                ),
                mizar_artifact::store::CanonicalJson,
                mizar_artifact::proof_witness::ProofWitnessRef,
            ),
        )>,
        String,
    > {
        use mizar_proof::{
            policy::{KernelEvidenceOrigin, KernelPolicyInput},
            selection::ProofWinnerClass,
            status::TrustedUsedAxiomsRef,
        };
        let Some(selection) = self
            .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                input,
                publisher,
                vc_id,
                (encoding.0.clone(), encoding.1),
                policies,
            )?
        else {
            return Ok(None);
        };
        if selection.selected_class() != ProofWinnerClass::KernelVerified {
            return Ok(None);
        }
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem candidate witness file requires one VC parent".to_owned())?;
        let kernel_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("KernelCheck")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremCandidateCheck")
        }))
        .ok_or_else(|| {
            "native theorem candidate witness file requires one Kernel parent".to_owned()
        })?;
        let storage = publisher.storage();
        let typed = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(
                vc_parent.as_output_ref(),
                &OutputKind::new("SourceTheoremVC"),
            )
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&typed).map_err(|error| error.to_string())?;
        let typed = storage
            .typed_handle::<(
                mizar_kernel::checker::KernelCheckResult,
                mizar_proof::policy::PolicyDecision,
            )>(
                kernel_parent.as_output_ref(),
                &OutputKind::new("SourceTheoremCandidateCheck"),
            )
            .map_err(|error| error.to_string())?;
        let checked = storage.get(&typed).map_err(|error| error.to_string())?;
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err("native theorem candidate witness file requires a module".to_owned());
        };
        let Some((arena, env, _, typed, _)) =
            self.native_theorem_source_metadata(module, "native theorem candidate witness file")?
        else {
            return Ok(None);
        };
        let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(&arena, &env)?;
        let check =
            mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena, &typed, &scope, &env, &labels, &resolved, None,
            )?;
        let generation = &vcs
            .vc(vc_id)
            .ok_or_else(|| {
                "native theorem candidate witness file requires the selected VC".to_owned()
            })?
            .anchor
            .generation_schema_version;
        let (products, file, reference) =
            mizar_proof::status::source_theorem_candidate_witness_file_candidate(
                &arena,
                &check,
                &env,
                self.snapshot,
                vc_id,
                (generation, vcs.schema_version(), encoding.0, encoding.1),
                policies,
                witness_path,
            )?;
        let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
        let kernel = KernelPolicyInput::from_kernel_result(&checked.0, origin);
        let trusted = TrustedUsedAxiomsRef::from_kernel_result(&checked.0, origin)
            .map_err(|error| error.to_string())?;
        let projection = &products.0;
        let reuse = projection.reuse_metadata();
        let selected = selection.reuse_metadata();
        if projection.selected_class() != selection.selected_class()
            || reuse.selected_candidate_id() != selection.selected_candidate_id()
            || reuse.policy_fingerprint() != selected.policy_fingerprint()
            || reuse.selected_evidence_hash() != selected.selected_evidence_hash()
            || reuse.accepted_goal_polarity() != selected.accepted_goal_polarity()
            || reuse.selected_candidate_provenance_hash()
                != selected.selected_candidate_provenance_hash()
            || reuse.selected_evidence_hash() != kernel.accepted_evidence_hash()
            || reuse.accepted_goal_polarity() != kernel.accepted_goal_polarity()
            || projection.trusted_used_axioms() != Some(&trusted)
            || Some(reference.kernel_acceptance.accepted_result_hash.digest)
                != kernel.accepted_evidence_hash()
        {
            return Err(
                "native theorem candidate witness file checked evidence mismatch".to_owned(),
            );
        }
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_published_output(input.snapshot, parent.as_output_ref())
                .map_err(|error| error.to_string())?;
        }
        Ok(Some((selection, (products, file, reference))))
    }

    /// Projects real source rows and staged witness bytes into an unpublished module draft.
    pub fn project_native_theorem_artifact_draft(
        self,
        input: &PhaseInput,
        publisher: &Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
        staging: (&str, &std::path::Path),
    ) -> Result<Option<mizar_ir::projection::VerifiedArtifactDraft>, String> {
        use mizar_artifact::{
            module_summary::{MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummary, SourceRangeSummary},
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
            store::{
                CanonicalHashDomain, CanonicalJson, HashClass, PublishedArtifactPath,
                PublishedArtifactReadOptions, artifact_hash_domain, read_published_artifact,
                write_published_artifact,
            },
            verified_artifact::{
                BuildProvenance, ExportVisibility, ExpressionMetadata, ObligationMetadata,
                ObligationStatus, VerifiedExport,
            },
        };
        use mizar_checker::{
            source_atomic_formula::{SourceAtomicFormulaKind, SourceAtomicFormulaRecovery},
            source_statement::{SourceStatementRecovery, SourceTheoremRole, SourceTheoremStatus},
            source_term::{SourcePrimaryTermKind, SourcePrimaryTermRecovery},
            type_checker::{NormalizedTypeStatus, TypeHeadRef},
            typed_ast::NodeRecoveryState,
        };
        use mizar_ir::{
            dispatch_input::PhaseDispatchInputBundle,
            projection::{ArtifactProjectionService, ProjectVerifiedArtifactInput},
        };
        use mizar_proof::{
            policy::{KernelEvidenceOrigin, KernelPolicyInput},
            selection::{
                ProofEvidenceCandidate, ProofEvidenceSet, ProofWinnerClass, VcProofSelection,
                merge_artifact_proof_selections, select_winner,
            },
            status::{
                ArtifactStatusPublication, CurrentArtifactObligationStatus, ProjectedProofStatus,
                ProofObligationIdentity, ProofStatusProjectionInput, TrustedUsedAxiomsRef,
                project_status,
            },
        };
        let fail = |suffix: &str| format!("native theorem artifact draft {suffix}");
        if input.snapshot != self.snapshot.id {
            return Err(fail("snapshot mismatch"));
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err(fail("requires a module"));
        };
        if input.parent_outputs().len() != 6 {
            return Err(fail("requires six parents"));
        }
        let roles = [
            ("SignatureCollection", "SourceTheoremModuleSummary"),
            ("Elaboration", "SourceTheoremCore"),
            ("VcGenerate", "SourceTheoremVC"),
            ("VcDischarge", "SourceTheoremDischarge"),
            ("KernelCheck", "SourceTheoremCandidateCheck"),
            ("KernelCheck", "SourceTheoremCandidateWitnessFile"),
        ];
        let parents = roles.map(|(phase, kind)| {
            unique(input.parent_outputs().iter().filter(|parent| {
                parent.as_output_ref().phase() == &IrPipelinePhase::new(phase)
                    && parent.as_output_ref().output_kind() == &OutputKind::new(kind)
            }))
        });
        let [
            Some(summary_parent),
            Some(core_parent),
            Some(vc_parent),
            Some(discharge_parent),
            Some(kernel_parent),
            Some(file_parent),
        ] = parents
        else {
            return Err(fail("requires unique source roles"));
        };
        let original = PhaseInput::new(
            input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                input.identities().dependency_hashes().to_vec(),
                vec![
                    core_parent.clone(),
                    vc_parent.clone(),
                    discharge_parent.clone(),
                    kernel_parent.clone(),
                ],
            )
            .map_err(|error| error.to_string())?,
        );
        let Some((selection, ((projection, slice, witness_draft), file, reference))) = self
            .native_theorem_candidate_witness_payload(
                &original, publisher, vc_id, encoding, policies, staging.0,
            )?
        else {
            return Ok(None);
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| fail("requires one captured source"))?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| fail("capture binding failed"))?;
        if !matches!(version.origin, SourceOrigin::Disk) {
            return Ok(None);
        }
        self.validate_native_lock_content()?;
        self.validate_native_config_content()?;
        if input.identities().input_hash() != source_input_hash(version)
            || !input.identities().dependency_hashes().is_empty()
            || !self.snapshot.dependency_artifacts.is_empty()
            || !self.module_index.dependency_summaries.is_empty()
        {
            return Ok(None);
        }
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let named = vec![NamedInputHash {
            name: "source".to_owned(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
            digest: source_input_hash(version),
        }];
        publisher
            .validate_current_snapshot(input.snapshot)
            .map_err(|error| error.to_string())?;
        for parent in input.parent_outputs() {
            let parent = parent.as_output_ref();
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_current_output(input.snapshot, parent)
                .map_err(|error| error.to_string())?;
            if parent.snapshot() != input.snapshot
                || parent.work_unit() != &work_unit
                || parent.schema_version() != SchemaVersion::new(1)
                || parent.lineage().named_input_hashes != named
            {
                return Err(fail("parent binding mismatch"));
            }
        }
        let file_ref = file_parent.as_output_ref();
        if file_ref.lineage().parents != vec![kernel_parent.as_output_ref().output()] {
            return Err(fail("FILE Kernel parent mismatch"));
        }
        let typed_file = storage
            .typed_handle::<CanonicalJson>(
                file_ref,
                &OutputKind::new("SourceTheoremCandidateWitnessFile"),
            )
            .map_err(|error| error.to_string())?;
        if *storage
            .get(&typed_file)
            .map_err(|error| error.to_string())?
            != file
        {
            return Err(fail("complete witness file mismatch"));
        }
        let typed_summary = storage
            .typed_handle::<ModuleSummary>(
                summary_parent.as_output_ref(),
                &OutputKind::new("SourceTheoremModuleSummary"),
            )
            .map_err(|error| error.to_string())?;
        let stored_summary = storage
            .get(&typed_summary)
            .map_err(|error| error.to_string())?;
        let Some(summary) = self.native_theorem_module_summary_candidate(module)? else {
            return Ok(None);
        };
        if *stored_summary != summary {
            return Err(fail("complete summary mismatch"));
        }
        if !summary.reexports.is_empty()
            || !summary.dependency_interfaces.is_empty()
            || summary.exported_symbols.len() != 1
        {
            return Ok(None);
        }
        let Some((arena, env, inference, typed, metadata)) =
            self.native_theorem_source_metadata(module, "native theorem artifact draft")?
        else {
            return Ok(None);
        };
        let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(&arena, &env)?;
        let check =
            mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena, &typed, &scope, &env, &labels, &resolved, None,
            )?;
        let [(owner, _)] = check.owners() else {
            return Ok(None);
        };
        if owner.role != SourceTheoremRole::Theorem
            || owner.status != SourceTheoremStatus::Unmodified
            || owner.recovery != SourceStatementRecovery::Normal
            || !env.imports().is_empty()
            || !env.exports().is_empty()
            || !typed.diagnostics().is_empty()
            || !inference.diagnostics().is_empty()
            || !metadata.diagnostics().is_empty()
        {
            return Ok(None);
        }
        let symbol = env
            .symbols()
            .get(&owner.symbol)
            .ok_or_else(|| fail("checked owner symbol missing"))?;
        if symbol.kind() != mizar_resolve::env::SymbolKind::Theorem
            || symbol.visibility() != mizar_resolve::env::Visibility::Public
            || symbol.export_status() != mizar_resolve::env::ExportStatus::Exported
        {
            return Ok(None);
        }
        let export = &summary.exported_symbols[0];
        if owner.symbol.module() != arena.module()
            || owner.source_range.source_id != version.source_id
            || arena.source_id() != version.source_id
            || symbol.origin().source_id() != version.source_id
            || symbol.origin().module_id() != arena.module()
            || symbol.origin().is_recovered()
            || symbol.origin().import_edge().is_some()
            || export.origin_id != owner.symbol.fqn().as_str()
            || export.fully_qualified_name != owner.symbol.fqn().as_str()
            || export.namespace_path
                != symbol
                    .namespace()
                    .as_str()
                    .split('.')
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            || export.visibility != "public"
            || export.declaration_kind != "theorem"
            || export.proof_status.is_some()
            || export.rendered_signature.is_empty()
            || export.source_range.start_byte != owner.source_range.start as u64
            || export.source_range.end_byte != owner.source_range.end as u64
            || summary.source_hash != version.source_hash
        {
            return Err(fail("checked owner export mismatch"));
        }
        let exports = vec![VerifiedExport {
            origin_id: export.origin_id.clone(),
            fully_qualified_name: export.fully_qualified_name.clone(),
            namespace_path: export.namespace_path.clone(),
            visibility: ExportVisibility::Public,
            export_kind: export.declaration_kind.clone(),
            source_range: export.source_range,
            rendered_signature: export.rendered_signature.clone(),
            interface_fingerprint: ArtifactHashRef::new(
                ArtifactHashClass::Interface,
                MODULE_SUMMARY_SCHEMA_FAMILY,
                summary.schema_version,
                export.interface_fingerprint,
            ),
            proof_status: None,
            documentation_ref: None,
        }];
        let Some(primary) = typed.source_term() else {
            return Ok(None);
        };
        let Some(atomic) = typed.source_atomic_formula() else {
            return Ok(None);
        };
        if metadata.expr_metadata().is_empty()
            || metadata.expr_metadata().len() != primary.terms().len() + atomic.formulas().len()
            || !metadata.collection_candidates().is_empty()
            || !metadata.expanded_candidates().is_empty()
            || !metadata.template_expansions().is_empty()
            || !metadata.viable_candidates().is_empty()
            || !metadata.viability_decisions().is_empty()
            || !metadata.specificity_graphs().is_empty()
            || !metadata.resolved_overloads().is_empty()
            || !metadata.inserted_coercions().is_empty()
            || !atomic.predicate_heads().is_empty()
            || !atomic.candidates().is_empty()
        {
            return Ok(None);
        }
        let mut sites = std::collections::BTreeSet::new();
        for (_, term) in primary.terms().iter() {
            if term.kind() != SourcePrimaryTermKind::VariableReference
                || term.recovery() != SourcePrimaryTermRecovery::Normal
                || !sites.insert(term.site().clone())
                || metadata
                    .expr_metadata()
                    .get_by_site(term.site())
                    .is_none_or(|row| {
                        row.source_range != term.source_range() || row.final_type.is_none()
                    })
            {
                return Ok(None);
            }
        }
        for (_, formula) in atomic.formulas().iter() {
            if formula.kind() != SourceAtomicFormulaKind::Equality
                || formula.recovery() != SourceAtomicFormulaRecovery::Normal
                || !sites.insert(formula.site().clone())
                || metadata
                    .expr_metadata()
                    .get_by_site(formula.site())
                    .is_none_or(|row| row.source_range != formula.source_range())
            {
                return Ok(None);
            }
        }
        let mut expressions = Vec::with_capacity(sites.len());
        for (_, row) in metadata.expr_metadata().canonical_iter() {
            let range = row.source_range;
            let Some(node) = typed.nodes().node(row.typed_site.node()) else {
                return Ok(None);
            };
            if !sites.remove(&row.typed_site)
                || range.source_id != version.source_id
                || node.anchor != mizar_session::SourceAnchor::Range(range)
                || node.recovery != NodeRecoveryState::Normal
                || row.overload.is_some()
                || !row.inserted_views.is_empty()
                || !row.diagnostics.is_empty()
                || range.start >= range.end
            {
                return Ok(None);
            }
            let Some(surface) = source.source_text.get(range.start..range.end) else {
                return Ok(None);
            };
            let inferred_type = if let Some(id) = row.final_type {
                let Some(ty) = inference.normalized_types().get(id) else {
                    return Ok(None);
                };
                if ty.status != NormalizedTypeStatus::Known
                    || ty.head != TypeHeadRef::BuiltinSet
                    || !ty.args.is_empty()
                    || !ty.attributes.positive().is_empty()
                    || !ty.attributes.negative().is_empty()
                {
                    return Ok(None);
                }
                Some("set".to_owned())
            } else {
                None
            };
            expressions.push(ExpressionMetadata {
                expression_id: row.expr.as_str().to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: range.start as u64,
                    end_byte: range.end as u64,
                },
                expression_kind: node.kind.as_str().to_owned(),
                rendered_surface: surface.to_owned(),
                inferred_type,
                resolved_symbol: None,
                inserted_coercions: Vec::new(),
                active_thesis: None,
                overload_resolution: None,
            });
        }
        if !sites.is_empty() {
            return Ok(None);
        }
        let vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(
                vc_parent.as_output_ref(),
                &OutputKind::new("SourceTheoremVC"),
            )
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&vcs).map_err(|error| error.to_string())?;
        let [vc] = vcs.vcs() else {
            return Ok(None);
        };
        if vc.id != vc_id
            || vc.kind != mizar_vc::vc_ir::VcKind::TheoremProofStep
            || slice.vc() != vc_id
            || slice.kind() != &vc.kind
            || slice.fingerprint().hash()
                != projection.reuse_metadata().dependency_slice_fingerprint()
        {
            return Ok(None);
        }
        let range = vc
            .anchor
            .source_range
            .ok_or_else(|| fail("requires original VC source range"))?;
        if range.source_id != version.source_id
            || range.start >= range.end
            || range.start < owner.source_range.start
            || range.end > owner.source_range.end
        {
            return Err(fail("original VC source range mismatch"));
        }
        let statement = source
            .source_text
            .get(range.start..range.end)
            .ok_or_else(|| fail("original VC source range invalid"))?;
        let reuse = projection.reuse_metadata();
        let identity = ProofObligationIdentity::new(
            projection.obligation_id(),
            reuse.obligation_anchor().clone(),
            reuse.obligation_fingerprint(),
            reuse.vc_fingerprint(),
            reuse.local_context_fingerprint(),
            reuse.dependency_slice_fingerprint(),
        )
        .map_err(|error| error.to_string())?;
        let kernel = storage
            .typed_handle::<(
                mizar_kernel::checker::KernelCheckResult,
                mizar_proof::policy::PolicyDecision,
            )>(
                kernel_parent.as_output_ref(),
                &OutputKind::new("SourceTheoremCandidateCheck"),
            )
            .map_err(|error| error.to_string())?;
        let kernel = storage.get(&kernel).map_err(|error| error.to_string())?;
        let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
        let kernel_input = KernelPolicyInput::from_kernel_result(&kernel.0, origin);
        let trusted = TrustedUsedAxiomsRef::from_kernel_result(&kernel.0, origin)
            .map_err(|error| error.to_string())?;
        let selected = selection.reuse_metadata();
        let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
            selection
                .selected_candidate_id()
                .cloned()
                .ok_or_else(|| fail("requires selected candidate"))?,
            &kernel_input,
        )
        .ok_or_else(|| fail("requires trusted candidate"))?
        .with_provenance_hash(
            selected
                .selected_candidate_provenance_hash()
                .ok_or_else(|| fail("requires candidate provenance"))?,
        );
        PublishedArtifactPath::new(version.normalized_path.as_str())
            .map_err(|_| fail("source path mismatch"))?;
        let path = PublishedArtifactPath::new(staging.0).map_err(|error| error.to_string())?;
        if path.as_str() != reference.witness_path {
            return Err(fail("witness path mismatch"));
        }
        let raw_refs = [
            (
                ArtifactHashClass::Interface,
                HashClass::Interface,
                "mizar-vc/source-theorem-canonical-vc/v1",
                reuse.vc_fingerprint(),
            ),
            (
                ArtifactHashClass::Interface,
                HashClass::Interface,
                "mizar-vc/source-theorem-canonical-context/v1",
                reuse.local_context_fingerprint(),
            ),
            (
                ArtifactHashClass::Interface,
                HashClass::Interface,
                mizar_vc::dependency_slice::DEPENDENCY_SLICE_SCHEMA_VERSION,
                reuse.dependency_slice_fingerprint(),
            ),
            (
                ArtifactHashClass::Artifact,
                HashClass::Artifact,
                if self.build_plan.lockfile.toolchain_version.is_some() {
                    "mizar-build/parsed-lock-content/v2"
                } else {
                    "mizar-build/parsed-lock-content/v1"
                },
                self.snapshot.lockfile_hash,
            ),
            (
                ArtifactHashClass::Interface,
                HashClass::Interface,
                "mizar-build/package-config-content/v1",
                self.snapshot.verifier_config_hash,
            ),
        ]
        .map(|(class, hash_class, producer, digest)| {
            let value = CanonicalJson::Object(std::collections::BTreeMap::from([
                ("producer".to_owned(), CanonicalJson::string(producer)),
                (
                    "digest".to_owned(),
                    CanonicalJson::string(
                        digest
                            .as_bytes()
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>(),
                    ),
                ),
            ]));
            let family = "mizar-driver/source-artifact-owner-fingerprint";
            let version = mizar_artifact::store::SchemaVersion::new(1, 0);
            ArtifactHashRef::new(
                class,
                family,
                version,
                CanonicalHashDomain::new(hash_class, family, version).hash(&value, &[]),
            )
        });
        let [
            vc_fingerprint,
            local_context_fingerprint,
            dependency_slice_fingerprint,
            lockfile_hash,
            verifier_config_hash,
        ] = raw_refs;
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_current_output(input.snapshot, parent.as_output_ref())
                .map_err(|error| error.to_string())?;
        }
        std::fs::create_dir(staging.1).map_err(|error| error.to_string())?;
        let domain = artifact_hash_domain(
            reference.witness_artifact_hash.schema_family.clone(),
            reference.witness_artifact_hash.schema_version,
        );
        let written = write_published_artifact(staging.1, &path, &file, &domain, &[])
            .map_err(|error| error.to_string())?;
        if written.artifact_hash != reference.witness_artifact_hash.digest {
            return Err(fail("witness file hash mismatch"));
        }
        let read = read_published_artifact(
            staging.1,
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &[],
                expected_artifact_hash: Some(reference.witness_artifact_hash.digest),
            },
        )
        .map_err(|error| error.to_string())?;
        mizar_proof::witness_store::read_source_theorem_witness_file(&read, &reference)?;
        let staged = witness_draft.stage_source_theorem_file(&read, &reference)?;
        let evidence = ProofEvidenceSet::new(
            kernel.0.target_vc_fingerprint().sort_bytes(),
            selected.encoded_problem_hash(),
            policies.1.clone(),
        )
        .with_candidates([
            candidate.with_selected_proof_witness_hash(staged.staged_payload_hash().digest)
        ]);
        let merged = merge_artifact_proof_selections(
            [VcProofSelection::new(vc_id, select_winner(&evidence))],
            [],
        )
        .map_err(|error| error.to_string())?;
        let [accepted] =
            <[_; 1]>::try_from(merged).map_err(|_| fail("requires one accepted selection"))?;
        let accepted = project_status(
            ProofStatusProjectionInput::new(accepted, policies.1.clone(), identity)
                .with_trusted_used_axioms(trusted),
        )
        .map_err(|error| error.to_string())?;
        if accepted.selected_class() != ProofWinnerClass::KernelVerified
            || accepted.projected_status() != ProjectedProofStatus::Accepted
            || accepted.artifact_publication()
                != ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Accepted)
            || accepted.accepted_witness_obligation_id() != Some(reference.obligation_id.as_str())
            || !accepted.diagnostic_refs().is_empty()
            || accepted.explanation_ref().is_some()
            || staged.artifact_reference_candidate() != Some(&reference)
        {
            return Err(fail("witness-bound accepted candidate mismatch"));
        }
        let obligations = vec![ObligationMetadata {
            obligation_id: accepted.obligation_id().to_owned(),
            obligation_anchor: Some(
                accepted
                    .reuse_metadata()
                    .obligation_anchor()
                    .as_str()
                    .to_owned(),
            ),
            owner_origin_id: Some(export.origin_id.clone()),
            source_range: SourceRangeSummary {
                start_byte: range.start as u64,
                end_byte: range.end as u64,
            },
            obligation_kind: "theorem-proof-step".to_owned(),
            statement_summary: statement.to_owned(),
            obligation_fingerprint: reference.obligation_fingerprint.clone(),
            vc_fingerprint,
            local_context_fingerprint,
            dependency_slice_fingerprint,
            verifier_policy_fingerprint: reference
                .kernel_acceptance
                .verifier_policy_fingerprint
                .clone(),
            status: match accepted.artifact_publication() {
                ArtifactStatusPublication::Publishable(
                    CurrentArtifactObligationStatus::Accepted,
                ) => ObligationStatus::Accepted,
                _ => return Err(fail("accepted status unavailable")),
            },
            accepted_witness_obligation_id: accepted
                .accepted_witness_obligation_id()
                .map(str::to_owned),
            deterministic_discharge_hash: None,
            diagnostic_ref: None,
        }];
        let draft = ArtifactProjectionService::new(Arc::clone(publisher))
            .project_module(ProjectVerifiedArtifactInput {
                snapshot: input.snapshot,
                required_outputs: input
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref().clone())
                    .collect(),
                module: summary.module,
                source_file: version.normalized_path.as_str().to_owned(),
                source_hash: version.source_hash,
                verified_at: None,
                exports,
                expressions,
                obligations,
                proof_witnesses: vec![reference],
                diagnostics: Vec::new(),
                provenance: BuildProvenance {
                    toolchain: self.snapshot.toolchain.identity().to_owned(),
                    language_edition: version.edition.as_str().to_owned(),
                    lockfile_hash,
                    verifier_config_hash,
                    dependency_artifact_hashes: Vec::new(),
                    cache_key: None,
                },
            })
            .map_err(|error| error.to_string())?;
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_current_output(input.snapshot, parent.as_output_ref())
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(draft))
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

pub(crate) struct SourceTheoremTypeCheckingService;

impl PhaseService for SourceTheoremTypeCheckingService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremTypeChecking",
            PhaseOwner::MizarChecker,
            vec![PipelinePhase::TypeChecking],
            "mizar-checker/source-theorem-typed-ast-publication/v1",
            "SourceTheoremTypedAst",
        )
        .expect("source theorem type checking descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem type checking has no complete dependency footprint".to_owned(),
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
        match inputs.publish_native_theorem_typed_ast(&input, &publisher) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}

pub(crate) struct SourceTheoremRegistrationResolutionService;

impl PhaseService for SourceTheoremRegistrationResolutionService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremRegistrationResolution",
            PhaseOwner::MizarChecker,
            vec![PipelinePhase::RegistrationResolution],
            "mizar-checker/source-theorem-registration-closure-publication/v1",
            "SourceTheoremRegistrationClosure",
        )
        .expect("source theorem registration resolution descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem registration resolution has no complete dependency footprint"
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
        match inputs.publish_native_theorem_registration_closure(&input, &publisher) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}

pub(crate) struct SourceTheoremOverloadResolutionService;

impl PhaseService for SourceTheoremOverloadResolutionService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremOverloadResolution",
            PhaseOwner::MizarChecker,
            vec![PipelinePhase::OverloadResolution],
            "mizar-checker/source-theorem-metadata-publication/v1",
            "SourceTheoremMetadata",
        )
        .expect("source theorem overload resolution descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem overload resolution has no complete dependency footprint"
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
        match inputs.publish_native_theorem_metadata(&input, &publisher) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
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

pub(crate) struct SourceTheoremVcGenerationService {
    pub(crate) generation_schema: mizar_vc::vc_ir::GenerationSchemaVersion,
    pub(crate) vc_schema: mizar_vc::vc_ir::VcSchemaVersion,
}

impl PhaseService for SourceTheoremVcGenerationService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremVcGeneration",
            PhaseOwner::MizarVc,
            vec![PipelinePhase::VcGenerate],
            format!(
                "mizar-vc/source-theorem-vc-publication/v1;generation={}:{};vc={}:{}",
                self.generation_schema.as_str().len(),
                self.generation_schema.as_str(),
                self.vc_schema.as_str().len(),
                self.vc_schema.as_str(),
            ),
            "SourceTheoremVC",
        )
        .expect("source theorem VC generation descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem VC generation has no complete dependency footprint".to_owned(),
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
        match inputs.publish_native_theorem_vcs_with_core_preparation_parents(
            &input,
            &publisher,
            &self.generation_schema,
            &self.vc_schema,
        ) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}

pub(crate) struct SourceTheoremVcDischargeService {
    pub(crate) policy: mizar_vc::discharge::DischargePolicy,
}

impl PhaseService for SourceTheoremVcDischargeService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremVcDischarge",
            PhaseOwner::MizarVc,
            vec![PipelinePhase::VcDischarge],
            format!(
                "mizar-vc/source-theorem-discharge-publication/v1;computation_policy={}:{};max_steps={}",
                self.policy.computation_limit.policy.as_str().len(),
                self.policy.computation_limit.policy.as_str(),
                self.policy.computation_limit.max_steps,
            ),
            "SourceTheoremDischarge",
        )
        .expect("source theorem VC discharge descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem VC discharge has no complete dependency footprint".to_owned(),
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
        let WorkUnit::Vc { module, .. } = &input.work_unit else {
            return blocking();
        };
        let Some(parent) = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        })) else {
            return blocking();
        };
        let Ok(handle) = publisher.storage().typed_handle::<mizar_vc::vc_ir::VcSet>(
            parent.as_output_ref(),
            &OutputKind::new("SourceTheoremVC"),
        ) else {
            return blocking();
        };
        let Ok(vcs) = publisher.storage().get(&handle) else {
            return blocking();
        };
        let [vc] = vcs.vcs() else {
            return blocking();
        };
        let Ok(bundle) = mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
            input.snapshot,
            input.identities().input_hash(),
            input.identities().dependency_hashes().to_vec(),
            input.parent_outputs().to_vec(),
        ) else {
            return blocking();
        };
        let module_input = PhaseInput::new(
            WorkUnit::Module {
                module: module.clone(),
            },
            bundle,
        );
        match inputs.publish_native_theorem_discharge(
            &module_input,
            &publisher,
            vc.id,
            &self.policy,
        ) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}

pub(crate) struct SourceTheoremAtpSolveService {
    pub(crate) policy: mizar_vc::discharge::DischargePolicy,
    pub(crate) logic_profile: mizar_atp::problem::LogicProfile,
    pub(crate) concrete_format: mizar_atp::problem::ConcreteFormat,
}

impl SourceTheoremAtpSolveService {
    pub(crate) fn configuration_tags(&self) -> Option<[u8; 7]> {
        use mizar_atp::problem::{
            ConcreteFormat, EqualitySupport, LogicFragment, NativePropertySupport,
            QuantifierPolicy, SoftTypeStrategy,
        };
        let mut formats = 0;
        for format in self.logic_profile.concrete_formats() {
            formats |= match format {
                ConcreteFormat::Tptp => 1,
                ConcreteFormat::SmtLib => 2,
                _ => return None,
            };
        }
        if formats == 0 {
            return None;
        }
        Some([
            match self.logic_profile.fragment() {
                LogicFragment::Fof => 0,
                LogicFragment::TffLike => 1,
                LogicFragment::SmtLibUninterpreted => 2,
                _ => return None,
            },
            match self.logic_profile.equality() {
                EqualitySupport::Unsupported => 0,
                EqualitySupport::Supported => 1,
                _ => return None,
            },
            match self.logic_profile.quantifiers() {
                QuantifierPolicy::PropositionalOnly => 0,
                QuantifierPolicy::FirstOrder => 1,
                _ => return None,
            },
            match self.logic_profile.soft_types() {
                SoftTypeStrategy::BackendSorts => 0,
                SoftTypeStrategy::GuardPredicates => 1,
                SoftTypeStrategy::SortsAndGuards => 2,
                SoftTypeStrategy::ScopedGuardPredicates => 3,
                _ => return None,
            },
            match self.logic_profile.native_properties() {
                NativePropertySupport::Unsupported => 0,
                NativePropertySupport::Supported => 1,
                _ => return None,
            },
            formats,
            match self.concrete_format {
                ConcreteFormat::Tptp => 0,
                ConcreteFormat::SmtLib => 1,
                _ => return None,
            },
        ])
    }
}

impl PhaseService for SourceTheoremAtpSolveService {
    fn phase(&self) -> PhaseDescriptor {
        let schema = match self.configuration_tags() {
            Some(
                [
                    fragment,
                    equality,
                    quantifiers,
                    soft_types,
                    native_properties,
                    formats,
                    format,
                ],
            ) => {
                format!(
                    "mizar-atp/source-theorem-candidate-publication/v1;computation_policy={}:{};max_steps={};profile={}:{};fragment={fragment};equality={equality};quantifiers={quantifiers};soft_types={soft_types};native_properties={native_properties};formats={formats};format={format}",
                    self.policy.computation_limit.policy.as_str().len(),
                    self.policy.computation_limit.policy.as_str(),
                    self.policy.computation_limit.max_steps,
                    self.logic_profile.name().as_str().len(),
                    self.logic_profile.name().as_str(),
                )
            }
            None => {
                "mizar-atp/source-theorem-candidate-publication/unsupported-transport".to_owned()
            }
        };
        PhaseDescriptor::new(
            "SourceTheoremAtpSolve",
            PhaseOwner::MizarAtp,
            vec![PipelinePhase::AtpSolve],
            schema,
            "SourceTheoremCandidateInputs",
        )
        .expect("source theorem ATP solve descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem ATP solve has no complete dependency footprint".to_owned(),
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
        if self.configuration_tags().is_none() {
            return blocking();
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
        let WorkUnit::Vc { module, .. } = &input.work_unit else {
            return blocking();
        };
        let Some(parent) = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        })) else {
            return blocking();
        };
        let Ok(handle) = publisher.storage().typed_handle::<mizar_vc::vc_ir::VcSet>(
            parent.as_output_ref(),
            &OutputKind::new("SourceTheoremVC"),
        ) else {
            return blocking();
        };
        let Ok(vcs) = publisher.storage().get(&handle) else {
            return blocking();
        };
        let [vc] = vcs.vcs() else {
            return blocking();
        };
        let Ok(bundle) = mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
            input.snapshot,
            input.identities().input_hash(),
            input.identities().dependency_hashes().to_vec(),
            input.parent_outputs().to_vec(),
        ) else {
            return blocking();
        };
        let module_input = PhaseInput::new(
            WorkUnit::Module {
                module: module.clone(),
            },
            bundle,
        );
        match inputs.publish_native_theorem_candidate(
            &module_input,
            &publisher,
            vc.id,
            &self.policy,
            self.logic_profile.clone(),
            self.concrete_format,
        ) {
            Ok(Some(handle)) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}

pub(crate) struct SourceTheoremCandidateCheckService {
    pub(crate) candidate: SourceTheoremAtpSolveService,
    pub(crate) verifier_policy: mizar_proof::policy::VerifierPolicy,
    pub(crate) witness_path_prefix: Option<String>,
}

impl PhaseService for SourceTheoremCandidateCheckService {
    fn phase(&self) -> PhaseDescriptor {
        let schema = if self.candidate.configuration_tags().is_some() {
            let candidate = self.candidate.phase().schema_version;
            let mut schema = format!(
                "mizar-proof/source-theorem-candidate-check-publication/v1;candidate={}:{};verifier_policy={}",
                candidate.len(),
                candidate,
                self.verifier_policy.policy_fingerprint().to_lower_hex(),
            );
            if let Some(prefix) = &self.witness_path_prefix {
                schema.push_str(&format!(
                    ";witness_file_path_prefix={}:{}",
                    prefix.len(),
                    prefix,
                ));
            }
            schema
        } else {
            "mizar-proof/source-theorem-candidate-check-publication/unsupported-transport"
                .to_owned()
        };
        PhaseDescriptor::new(
            "SourceTheoremCandidateCheck",
            PhaseOwner::MizarKernelProof,
            vec![PipelinePhase::KernelCheck],
            schema,
            "SourceTheoremCandidateCheck",
        )
        .expect("source theorem candidate check descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem candidate check has no complete dependency footprint"
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
        if self.candidate.configuration_tags().is_none() {
            return blocking();
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
        let WorkUnit::EvidenceCandidate { module, .. } = &input.work_unit else {
            return blocking();
        };
        if input.parent_outputs().len() != 3
            || [
                ("Elaboration", "SourceTheoremCore"),
                ("VcDischarge", "SourceTheoremDischarge"),
            ]
            .into_iter()
            .any(|(phase, kind)| {
                unique(input.parent_outputs().iter().filter(|parent| {
                    parent.as_output_ref().phase() == &IrPipelinePhase::new(phase)
                        && parent.as_output_ref().output_kind() == &OutputKind::new(kind)
                }))
                .is_none()
            })
        {
            return blocking();
        }
        let Some(parent) = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        })) else {
            return blocking();
        };
        let Ok(handle) = publisher.storage().typed_handle::<mizar_vc::vc_ir::VcSet>(
            parent.as_output_ref(),
            &OutputKind::new("SourceTheoremVC"),
        ) else {
            return blocking();
        };
        let Ok(vcs) = publisher.storage().get(&handle) else {
            return blocking();
        };
        let [vc] = vcs.vcs() else {
            return blocking();
        };
        let Ok(bundle) = mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
            input.snapshot,
            input.identities().input_hash(),
            input.identities().dependency_hashes().to_vec(),
            input.parent_outputs().to_vec(),
        ) else {
            return blocking();
        };
        let module_input = PhaseInput::new(
            WorkUnit::Module {
                module: module.clone(),
            },
            bundle,
        );
        let Ok(Some(candidate)) = inputs.publish_native_theorem_candidate(
            &module_input,
            &publisher,
            vc.id,
            &self.candidate.policy,
            self.candidate.logic_profile.clone(),
            self.candidate.concrete_format,
        ) else {
            return blocking();
        };
        let Ok(candidate) =
            mizar_ir::dispatch_input::SealedParentOutputHandle::from_published_output(
                &publisher,
                input.snapshot,
                candidate.erase(),
            )
        else {
            return blocking();
        };
        let mut parents = input.parent_outputs().to_vec();
        parents.push(candidate);
        let Ok(bundle) = mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
            input.snapshot,
            input.identities().input_hash(),
            input.identities().dependency_hashes().to_vec(),
            parents,
        ) else {
            return blocking();
        };
        let module_input = PhaseInput::new(module_input.work_unit, bundle);
        match inputs.publish_native_theorem_candidate_check(
            &module_input,
            &publisher,
            vc.id,
            (
                self.candidate.logic_profile.clone(),
                self.candidate.concrete_format,
            ),
            (&self.candidate.policy, &self.verifier_policy),
        ) {
            Ok(Some(handle)) => {
                let mut result = PhaseResult {
                    output_refs: vec![handle.erase()],
                    ..PhaseResult::complete()
                };
                let Some(prefix) = &self.witness_path_prefix else {
                    return result;
                };
                let Ok(kernel) =
                    mizar_ir::dispatch_input::SealedParentOutputHandle::from_published_output(
                        &publisher,
                        input.snapshot,
                        result.output_refs[0].clone(),
                    )
                else {
                    result.status = PhaseStatus::Blocking;
                    return result;
                };
                let mut parents = input.parent_outputs().to_vec();
                parents.push(kernel);
                let Ok(bundle) = mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
                    input.snapshot,
                    input.identities().input_hash(),
                    input.identities().dependency_hashes().to_vec(),
                    parents,
                ) else {
                    result.status = PhaseStatus::Blocking;
                    return result;
                };
                let mut path = format!("{prefix}/");
                for byte in result.output_refs[0].content_hash().as_bytes() {
                    use std::fmt::Write as _;
                    write!(path, "{byte:02x}").expect("writing to string cannot fail");
                }
                path.push_str(".json");
                match inputs.publish_native_theorem_candidate_witness_file(
                    &PhaseInput::new(module_input.work_unit, bundle),
                    &publisher,
                    vc.id,
                    (
                        self.candidate.logic_profile.clone(),
                        self.candidate.concrete_format,
                    ),
                    (&self.candidate.policy, &self.verifier_policy),
                    &path,
                ) {
                    Ok(Some(file)) => result.output_refs.push(file.erase()),
                    Ok(None) => {}
                    Err(_) => result.status = PhaseStatus::Blocking,
                }
                result
            }
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

/// Stores lossless source syntax only when genuine checker replay reproduces the supplied closure.
pub fn native_theorem_registration_closure_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    closure: &mizar_checker::cluster_trace::ClusterClosureOutput,
) -> Result<Option<Vec<u8>>, String> {
    let Some(syntax) = ast.canonical_bytes() else {
        return Ok(None);
    };
    let domain = b"mizar-driver/native-theorem-registration-closure-storage/v1\0";
    let fields = [
        module.package().as_str().as_bytes(),
        module.path().as_str().as_bytes(),
        syntax.as_slice(),
        closure
            .trace()
            .traversal_profile()
            .cache_key_material()
            .as_str()
            .as_bytes(),
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
        bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
        bytes.extend_from_slice(field);
    }
    let Some(replayed) =
        native_theorem_registration_closure_from_storage_bytes(&bytes, ast.source_id, module)?
    else {
        return Ok(None);
    };
    Ok((replayed == *closure).then_some(bytes))
}

/// Rebinds stored syntax to an existing source id and replays the genuine Checker owners.
pub fn native_theorem_registration_closure_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
) -> Result<Option<mizar_checker::cluster_trace::ClusterClosureOutput>, String> {
    let domain = b"mizar-driver/native-theorem-registration-closure-storage/v1\0";
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) = bytes.strip_prefix(domain) else {
        return Ok(None);
    };
    let mut fields = [&[][..]; 4];
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
    let closure = mizar_checker::type_checker::check_source_theorem_registration_closure(
        &arena, &env, &scope, &checked.2,
    )?;
    Ok((fields[3]
        == closure
            .trace()
            .traversal_profile()
            .cache_key_material()
            .as_str()
            .as_bytes())
    .then_some(closure))
}

/// Stores lossless source syntax only when genuine checker replay reproduces the whole supplied metadata.
pub fn native_theorem_metadata_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    metadata: &(
        mizar_checker::binding_env::BindingEnv,
        mizar_checker::type_checker::TermFormulaInferenceOutput,
        mizar_checker::typed_ast::TypedAst,
        mizar_checker::resolved_typed_ast::ResolvedTypedAst,
    ),
) -> Result<Option<Vec<u8>>, String> {
    let Some(syntax) = ast.canonical_bytes() else {
        return Ok(None);
    };
    let domain = b"mizar-driver/native-theorem-metadata-storage/v1\0";
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
        bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
        bytes.extend_from_slice(field);
    }
    let Some(replayed) = native_theorem_metadata_from_storage_bytes(&bytes, ast.source_id, module)?
    else {
        return Ok(None);
    };
    Ok((replayed == *metadata).then_some(bytes))
}

/// Rebinds stored syntax to an existing source id and replays the genuine Checker owners.
pub fn native_theorem_metadata_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
) -> Result<
    Option<(
        mizar_checker::binding_env::BindingEnv,
        mizar_checker::type_checker::TermFormulaInferenceOutput,
        mizar_checker::typed_ast::TypedAst,
        mizar_checker::resolved_typed_ast::ResolvedTypedAst,
    )>,
    String,
> {
    let domain = b"mizar-driver/native-theorem-metadata-storage/v1\0";
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
    mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope).map(Some)
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

/// Stores genuine discharge through one opaque original-VC stream and explicit policy.
pub fn native_theorem_discharge_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    core: &mizar_core::core_ir::CoreIr,
    input: mizar_vc::discharge::DischargeInput<'_>,
    output: &mizar_vc::discharge::DischargeOutput,
    generation_schema: &mizar_vc::vc_ir::GenerationSchemaVersion,
) -> Result<Option<Vec<u8>>, String> {
    let Some(original) = native_theorem_vc_storage_bytes(
        ast,
        module,
        core,
        input.vc_set,
        input.vc_set.snapshot(),
        generation_schema,
    )?
    else {
        return Ok(None);
    };
    let domain = b"mizar-driver/native-theorem-discharge-storage/v1\0";
    let limit = input.policy.computation_limit.max_steps.to_le_bytes();
    let fields = [
        original.as_slice(),
        input.policy.computation_limit.policy.as_str().as_bytes(),
        limit.as_slice(),
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
    let actual = mizar_vc::discharge::try_discharge(input).map_err(|error| error.to_string())?;
    if actual != *output {
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

/// Replays all discharge rows in an existing source, module, snapshot and policy context.
pub fn native_theorem_discharge_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
    snapshot: mizar_session::BuildSnapshotId,
    policy: &mizar_vc::discharge::DischargePolicy,
) -> Result<Option<mizar_vc::discharge::DischargeOutput>, String> {
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) =
        bytes.strip_prefix(b"mizar-driver/native-theorem-discharge-storage/v1\0")
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
    let (Ok(key), Ok(limit)) = (
        std::str::from_utf8(fields[1]),
        <[u8; 4]>::try_from(fields[2]),
    ) else {
        return Ok(None);
    };
    let Some(vcs) = native_theorem_vc_from_storage_bytes(fields[0], source_id, module, snapshot)?
    else {
        return Ok(None);
    };
    if key != policy.computation_limit.policy.as_str()
        || u32::from_le_bytes(limit) != policy.computation_limit.max_steps
    {
        return Ok(None);
    }
    mizar_vc::discharge::try_discharge(mizar_vc::discharge::DischargeInput {
        vc_set: &vcs,
        policy,
    })
    .map(Some)
    .map_err(|error| error.to_string())
}

/// Stores an authentic candidate pair through opaque Core bytes and complete configuration.
pub fn native_theorem_candidate_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    core: &mizar_core::core_ir::CoreIr,
    vc_context: (
        mizar_session::BuildSnapshotId,
        &mizar_vc::vc_ir::GenerationSchemaVersion,
        &mizar_vc::vc_ir::VcSchemaVersion,
        mizar_vc::vc_ir::VcId,
    ),
    atp_context: (
        &mizar_vc::discharge::DischargePolicy,
        &mizar_atp::problem::LogicProfile,
        mizar_atp::problem::ConcreteFormat,
    ),
    pair: &(
        mizar_atp::backend::EncodedBackendProblem,
        mizar_atp::backend::BackendCandidateEvidence,
    ),
) -> Result<Option<Vec<u8>>, String> {
    use mizar_atp::problem::{
        ConcreteFormat, EqualitySupport, LogicFragment, NativePropertySupport, QuantifierPolicy,
        SoftTypeStrategy,
    };
    let Some(core_bytes) = native_theorem_core_storage_bytes(ast, module, core)? else {
        return Ok(None);
    };
    let Ok(selected) = u64::try_from(vc_context.3.index()) else {
        return Ok(None);
    };
    let profile = atp_context.1;
    let mut formats = 0;
    for format in profile.concrete_formats() {
        formats |= match format {
            ConcreteFormat::Tptp => 1,
            ConcreteFormat::SmtLib => 2,
            _ => return Ok(None),
        };
    }
    let mut record = [0; 19];
    record[..8].copy_from_slice(&selected.to_le_bytes());
    record[8..12].copy_from_slice(&atp_context.0.computation_limit.max_steps.to_le_bytes());
    record[12..].copy_from_slice(&[
        match profile.fragment() {
            LogicFragment::Fof => 0,
            LogicFragment::TffLike => 1,
            LogicFragment::SmtLibUninterpreted => 2,
            _ => return Ok(None),
        },
        match profile.equality() {
            EqualitySupport::Unsupported => 0,
            EqualitySupport::Supported => 1,
            _ => return Ok(None),
        },
        match profile.quantifiers() {
            QuantifierPolicy::PropositionalOnly => 0,
            QuantifierPolicy::FirstOrder => 1,
            _ => return Ok(None),
        },
        match profile.soft_types() {
            SoftTypeStrategy::BackendSorts => 0,
            SoftTypeStrategy::GuardPredicates => 1,
            SoftTypeStrategy::SortsAndGuards => 2,
            SoftTypeStrategy::ScopedGuardPredicates => 3,
            _ => return Ok(None),
        },
        match profile.native_properties() {
            NativePropertySupport::Unsupported => 0,
            NativePropertySupport::Supported => 1,
            _ => return Ok(None),
        },
        formats,
        match atp_context.2 {
            ConcreteFormat::Tptp => 0,
            ConcreteFormat::SmtLib => 1,
            _ => return Ok(None),
        },
    ]);
    let domain = b"mizar-driver/native-theorem-candidate-storage/v1\0";
    let fields = [
        core_bytes.as_slice(),
        vc_context.1.as_str().as_bytes(),
        vc_context.2.as_str().as_bytes(),
        atp_context.0.computation_limit.policy.as_str().as_bytes(),
        profile.name().as_str().as_bytes(),
        record.as_slice(),
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
    let original = mizar_vc::generator::generate_source_theorem_vcs(
        core,
        vc_context.0,
        vc_context.1,
        vc_context.2,
    )?;
    let discharge = mizar_vc::discharge::try_discharge(mizar_vc::discharge::DischargeInput {
        vc_set: &original,
        policy: atp_context.0,
    })
    .map_err(|error| error.to_string())?;
    let actual = mizar_atp::backend::build_source_theorem_candidate_inputs(
        core,
        &original,
        vc_context.3,
        &discharge,
        atp_context.0,
        profile.clone(),
        atp_context.2,
    )?;
    if actual != *pair {
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

/// Replays an untrusted candidate pair from recorded configuration in an existing context.
pub fn native_theorem_candidate_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
    snapshot: mizar_session::BuildSnapshotId,
) -> Result<
    Option<(
        mizar_atp::backend::EncodedBackendProblem,
        mizar_atp::backend::BackendCandidateEvidence,
    )>,
    String,
> {
    use mizar_atp::problem::{
        ConcreteFormat, EqualitySupport, LogicFragment, LogicProfile, NativePropertySupport,
        QuantifierPolicy, SoftTypeStrategy,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion},
    };
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) =
        bytes.strip_prefix(b"mizar-driver/native-theorem-candidate-storage/v1\0")
    else {
        return Ok(None);
    };
    let mut fields = [&[][..]; 6];
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
    let (Ok(generation), Ok(schema), Ok(key), Ok(name), Ok(record)) = (
        std::str::from_utf8(fields[1]),
        std::str::from_utf8(fields[2]),
        std::str::from_utf8(fields[3]),
        std::str::from_utf8(fields[4]),
        <[u8; 19]>::try_from(fields[5]),
    ) else {
        return Ok(None);
    };
    let Some(selected) = record
        .get(..8)
        .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
        .map(u64::from_le_bytes)
        .and_then(|index| usize::try_from(index).ok())
    else {
        return Ok(None);
    };
    let fragment = match record[12] {
        0 => LogicFragment::Fof,
        1 => LogicFragment::TffLike,
        2 => LogicFragment::SmtLibUninterpreted,
        _ => return Ok(None),
    };
    let equality = match record[13] {
        0 => EqualitySupport::Unsupported,
        1 => EqualitySupport::Supported,
        _ => return Ok(None),
    };
    let quantifiers = match record[14] {
        0 => QuantifierPolicy::PropositionalOnly,
        1 => QuantifierPolicy::FirstOrder,
        _ => return Ok(None),
    };
    let soft_types = match record[15] {
        0 => SoftTypeStrategy::BackendSorts,
        1 => SoftTypeStrategy::GuardPredicates,
        2 => SoftTypeStrategy::SortsAndGuards,
        3 => SoftTypeStrategy::ScopedGuardPredicates,
        _ => return Ok(None),
    };
    let native = match record[16] {
        0 => NativePropertySupport::Unsupported,
        1 => NativePropertySupport::Supported,
        _ => return Ok(None),
    };
    if record[17] == 0 || record[17] & !3 != 0 {
        return Ok(None);
    }
    let chosen = match record[18] {
        0 => ConcreteFormat::Tptp,
        1 => ConcreteFormat::SmtLib,
        _ => return Ok(None),
    };
    let Some(core) = native_theorem_core_from_storage_bytes(fields[0], source_id, module)? else {
        return Ok(None);
    };
    let formats = [(1, ConcreteFormat::Tptp), (2, ConcreteFormat::SmtLib)]
        .into_iter()
        .filter_map(|(bit, format)| (record[17] & bit != 0).then_some(format))
        .collect();
    let Ok(profile) = LogicProfile::try_new(
        name,
        fragment,
        equality,
        quantifiers,
        soft_types,
        native,
        formats,
    ) else {
        return Ok(None);
    };
    let original = mizar_vc::generator::generate_source_theorem_vcs(
        &core,
        snapshot,
        &GenerationSchemaVersion::new(generation),
        &VcSchemaVersion::new(schema),
    )?;
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new(key),
            max_steps: u32::from_le_bytes([record[8], record[9], record[10], record[11]]),
        },
    };
    let discharge = try_discharge(DischargeInput {
        vc_set: &original,
        policy: &policy,
    })
    .map_err(|error| error.to_string())?;
    mizar_atp::backend::build_source_theorem_candidate_inputs(
        &core,
        &original,
        VcId::new(selected),
        &discharge,
        &policy,
        profile,
        chosen,
    )
    .map(Some)
}

/// Stores replay inputs after fresh checking in an explicit source and policy context.
pub fn native_theorem_candidate_check_storage_bytes(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    core: &mizar_core::core_ir::CoreIr,
    vc_context: (
        mizar_session::BuildSnapshotId,
        &mizar_vc::vc_ir::GenerationSchemaVersion,
        &mizar_vc::vc_ir::VcSchemaVersion,
        mizar_vc::vc_ir::VcId,
    ),
    atp_context: (
        &mizar_vc::discharge::DischargePolicy,
        &mizar_atp::problem::LogicProfile,
        mizar_atp::problem::ConcreteFormat,
    ),
    policy: &mizar_proof::policy::VerifierPolicy,
    expected: (
        &(
            mizar_atp::backend::EncodedBackendProblem,
            mizar_atp::backend::BackendCandidateEvidence,
        ),
        &(
            mizar_kernel::checker::KernelCheckResult,
            mizar_proof::policy::PolicyDecision,
        ),
    ),
) -> Result<Option<Vec<u8>>, String> {
    let Some(candidate_bytes) = native_theorem_candidate_storage_bytes(
        ast,
        module,
        core,
        vc_context,
        atp_context,
        expected.0,
    )?
    else {
        return Ok(None);
    };
    let fingerprint = policy.policy_fingerprint();
    let domain = b"mizar-driver/native-theorem-candidate-check-storage/v1\0";
    let fields = [
        candidate_bytes.as_slice(),
        fingerprint.as_bytes().as_slice(),
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
    let (actual_core, _, _, encoded, candidate, result, decision) =
        mizar_proof::status::check_source_theorem_candidate(
            &arena,
            &check,
            &env,
            vc_context.0,
            vc_context.3,
            (
                vc_context.1,
                vc_context.2,
                atp_context.1.clone(),
                atp_context.2,
            ),
            (atp_context.0, policy),
        )?;
    if actual_core != *core
        || (encoded, candidate) != *expected.0
        || (result, decision) != *expected.1
    {
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

/// Freshly recomputes checking data under the explicitly supplied replay context.
pub fn native_theorem_candidate_check_from_storage_bytes(
    bytes: &[u8],
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &mizar_resolve::resolved_ast::ModuleId,
    vc_context: (
        mizar_session::BuildSnapshotId,
        &mizar_vc::vc_ir::GenerationSchemaVersion,
        &mizar_vc::vc_ir::VcSchemaVersion,
        mizar_vc::vc_ir::VcId,
    ),
    atp_context: (
        &mizar_vc::discharge::DischargePolicy,
        &mizar_atp::problem::LogicProfile,
        mizar_atp::problem::ConcreteFormat,
    ),
    policy: &mizar_proof::policy::VerifierPolicy,
) -> Result<
    Option<(
        mizar_kernel::checker::KernelCheckResult,
        mizar_proof::policy::PolicyDecision,
    )>,
    String,
> {
    let domain = b"mizar-driver/native-theorem-candidate-check-storage/v1\0";
    if bytes.len() > 32 * 1024 * 1024 {
        return Ok(None);
    }
    let Some(mut remaining) = bytes.strip_prefix(domain) else {
        return Ok(None);
    };
    let mut fields = [&[][..]; 2];
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
    if !remaining.is_empty() || fields[1].len() != 32 {
        return Ok(None);
    }
    let Some(decoded) = native_theorem_candidate_from_storage_bytes(
        fields[0],
        ast.source_id,
        module,
        vc_context.0,
    )?
    else {
        return Ok(None);
    };
    if fields[1] != policy.policy_fingerprint().as_bytes() {
        return Ok(None);
    }
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
    let (core, _, _, encoded, candidate, result, decision) =
        mizar_proof::status::check_source_theorem_candidate(
            &arena,
            &check,
            &env,
            vc_context.0,
            vc_context.3,
            (
                vc_context.1,
                vc_context.2,
                atp_context.1.clone(),
                atp_context.2,
            ),
            (atp_context.0, policy),
        )?;
    let pair = (encoded, candidate);
    if pair != decoded {
        return Ok(None);
    }
    let Some(regenerated) =
        native_theorem_candidate_storage_bytes(ast, module, &core, vc_context, atp_context, &pair)?
    else {
        return Ok(None);
    };
    if regenerated != fields[0] {
        return Ok(None);
    }
    Ok(Some((result, decision)))
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
