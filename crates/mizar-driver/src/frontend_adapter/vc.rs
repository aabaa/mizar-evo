use super::*;

impl<'a> crate::registry::SourceLoadInputs<'a> {
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem kernel handoff requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem kernel handoff requires one VC parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let vc_ref = vc_parent.as_output_ref();
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
        if preparation.phase != IrPipelinePhase::from(PipelinePhase::AlgorithmPreparation)
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(handoff))
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem discharge requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem discharge publication requires one Core parent".to_owned())?
        .as_output_ref();
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
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
        let Some(semantic_bytes) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
            return Ok(None);
        };
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
        let phase = IrPipelinePhase::from(PipelinePhase::VcDischarge);
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
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
                .validate_output(
                    input.snapshot,
                    parent.as_output_ref(),
                    OutputAuthorization::Published,
                )
                .map_err(|error| error.to_string())?;
        }
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
                origin,
                target,
            },
        )
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem vc requires one Core parent".to_owned())?;
        let preparation_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase()
                == &IrPipelinePhase::from(PipelinePhase::AlgorithmPreparation)
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremPreparation")
        }))
        .ok_or_else(|| "native theorem vc requires one Preparation parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let preparation_ref = preparation_parent.as_output_ref();
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
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
        let Some(semantic_bytes) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
            return Ok(None);
        };
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
        let phase = IrPipelinePhase::from(PipelinePhase::VcGenerate);
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
        ));
        let output_kind = OutputKind::new("SourceTheoremVC");
        let source_id = version.source_id;
        let snapshot = input.snapshot;
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    parent.as_output_ref(),
                    OutputAuthorization::Published,
                )
                .map_err(|error| error.to_string())?;
        }
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
                origin,
                target,
            },
        )
        .map(Some)
        .map_err(|error| error.to_string())
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
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
        let Some(vc) = vcs.vcs().first() else {
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
    let Some(size) = framing::checked_size(domain, &fields, 32 * 1024 * 1024) else {
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
    let Some(bytes) = framing::encode(domain, &fields, size) else {
        return Ok(None);
    };
    Ok(Some(bytes))
}

/// Replays raw VCs in the caller's existing source, module and snapshot context.
pub fn native_theorem_vc_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
    snapshot: mizar_session::BuildSnapshotId,
) -> Result<Option<mizar_vc::vc_ir::VcSet>, String> {
    let Some(fields) = framing::decode::<3>(
        bytes,
        b"mizar-driver/native-theorem-vc-storage/v1\0",
        32 * 1024 * 1024,
    ) else {
        return Ok(None);
    };
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
    let Some(size) = framing::checked_size(domain, &fields, 32 * 1024 * 1024) else {
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
    let Some(bytes) = framing::encode(domain, &fields, size) else {
        return Ok(None);
    };
    Ok(Some(bytes))
}

/// Rebinds stored algorithm syntax to the supplied source, module and snapshot.
pub fn native_algorithm_vc_from_storage_bytes(
    bytes: &[u8],
    source_id: mizar_session::SourceId,
    module: &mizar_resolve::resolved_ast::ModuleId,
    snapshot: mizar_session::BuildSnapshotId,
) -> Result<Option<mizar_vc::vc_ir::VcSet>, String> {
    let Some(fields) = framing::decode::<3>(
        bytes,
        b"mizar-driver/native-algorithm-vc-storage/v1\0",
        32 * 1024 * 1024,
    ) else {
        return Ok(None);
    };
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
    let Some(size) = framing::checked_size(domain, &fields, 32 * 1024 * 1024) else {
        return Ok(None);
    };
    let actual = mizar_vc::discharge::try_discharge(input).map_err(|error| error.to_string())?;
    if actual != *output {
        return Ok(None);
    }
    let Some(bytes) = framing::encode(domain, &fields, size) else {
        return Ok(None);
    };
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
    let Some(fields) = framing::decode::<3>(
        bytes,
        b"mizar-driver/native-theorem-discharge-storage/v1\0",
        32 * 1024 * 1024,
    ) else {
        return Ok(None);
    };
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

pub(crate) struct SourceTheoremReflexivityKernelCheckService {
    pub(crate) verifier_policy: mizar_proof::policy::VerifierPolicy,
}

impl PhaseService for SourceTheoremReflexivityKernelCheckService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremReflexivityKernelCheck",
            PhaseOwner::MizarKernelProof,
            vec![PipelinePhase::KernelCheck],
            format!(
                "mizar-proof/source-theorem-reflexivity-kernel-status/v1;verifier_policy={}",
                self.verifier_policy.policy_fingerprint().to_lower_hex(),
            ),
            "SourceTheoremReflexivityKernelStatus",
        )
        .expect("source theorem reflexivity Kernel descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem reflexivity Kernel check has no complete dependency footprint"
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
        let result = (|| -> Option<PhaseResult> {
            let WorkUnit::EvidenceCandidate { module, .. } = &input.work_unit else {
                return None;
            };
            if input.parent_outputs().len() != 3 {
                return None;
            }
            let mut parents = Vec::new();
            for (phase, kind) in [
                (PipelinePhase::Elaboration, "SourceTheoremCore"),
                (
                    PipelinePhase::AlgorithmPreparation,
                    "SourceTheoremPreparation",
                ),
                (PipelinePhase::VcGenerate, "SourceTheoremVC"),
            ] {
                parents.push(unique(input.parent_outputs().iter().filter(|parent| {
                    parent.as_output_ref().phase() == &IrPipelinePhase::from(phase)
                        && parent.as_output_ref().output_kind() == &OutputKind::new(kind)
                }))?);
            }
            let work_unit = IrWorkUnit::new(module_work_unit_label(
                module.package.as_str(),
                module.path.as_str(),
            ));
            let named = vec![NamedInputHash {
                name: "source".to_owned(),
                domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                digest: input.identities().input_hash(),
            }];
            let storage = publisher.storage();
            for parent in &parents {
                let parent = parent.as_output_ref();
                storage.validate_handle(parent).ok()?;
                publisher
                    .validate_output(input.snapshot, parent, OutputAuthorization::Published)
                    .ok()?;
                if parent.snapshot() != input.snapshot
                    || parent.work_unit() != &work_unit
                    || parent.schema_version() != SchemaVersion::new(1)
                    || parent.lineage().named_input_hashes != named
                {
                    return None;
                }
            }
            let core_ref = parents[0].as_output_ref();
            let preparation_ref = parents[1].as_output_ref();
            let vc_ref = parents[2].as_output_ref();
            if preparation_ref.lineage().parents != vec![core_ref.output()]
                || vc_ref.lineage().parents.len() != 2
                || !vc_ref.lineage().parents.contains(&core_ref.output())
                || !vc_ref.lineage().parents.contains(&preparation_ref.output())
            {
                return None;
            }
            let core = storage
                .typed_handle::<mizar_core::core_ir::CoreIr>(
                    core_ref,
                    &OutputKind::new("SourceTheoremCore"),
                )
                .ok()?;
            let core = storage.get(&core).ok()?;
            let vcs = storage
                .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, &OutputKind::new("SourceTheoremVC"))
                .ok()?;
            let vcs = storage.get(&vcs).ok()?;
            let vc = vcs.vcs().first()?;
            let module_input = PhaseInput::new(
                WorkUnit::Module {
                    module: module.clone(),
                },
                mizar_ir::dispatch_input::PhaseDispatchInputBundle::new(
                    input.snapshot,
                    input.identities().input_hash(),
                    input.identities().dependency_hashes().to_vec(),
                    vec![parents[0].clone(), parents[1].clone()],
                )
                .ok()?,
            );
            let actual_vcs = inputs
                .native_theorem_vcs_with_core_preparation_parents(
                    &module_input,
                    &publisher,
                    &vc.anchor.generation_schema_version,
                    vcs.schema_version(),
                )
                .ok()??;
            if actual_vcs != *vcs {
                return None;
            }
            let version = unique(inputs.snapshot.source_versions.iter().filter(|version| {
                version.package_id == module.package && version.module_path == module.path
            }))?;
            if source_input_hash(version) != input.identities().input_hash() {
                return None;
            }
            let (source, _) = inputs.native_source(version)?;
            let package = inputs
                .build_plan
                .packages
                .iter()
                .find(|package| package.package_id == module.package)?;
            let PackagePlanSource::Workspace { root, .. } = &package.source else {
                return None;
            };
            let output = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(
                    PathBuf::from(inputs.snapshot.workspace_root.as_str()).join(root),
                )),
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(source)
            .ok()?;
            if !output.diagnostics.is_empty() || output.cache_keys.ast.is_none() {
                return None;
            }
            let ast = output.ast.as_ref()?;
            let resolver_module = mizar_resolve::resolved_ast::ModuleId::new(
                module.package.clone(),
                module.path.clone(),
            );
            let (arena, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(
                ast,
                &resolver_module,
            )
            .ok()??;
            mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env).ok()??;
            let scope =
                mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(
                    mizar_resolve::names::SourceVariableScopeInput::new(
                        ast,
                        &resolver_module,
                        &env,
                    ),
                )
                .ok()?;
            let metadata =
                mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)
                    .ok()?;
            let (scope, labels, resolved) =
                mizar_resolve::symbols::source_proof_inputs(&arena, &env).ok()?;
            let check = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena, &metadata.2, &scope, &env, &labels, &resolved, None,
            ).ok()?;
            let checked = mizar_proof::status::check_source_theorem_reflexivity_with_status(
                &arena,
                &check,
                &env,
                input.snapshot,
                vc.id,
                &self.verifier_policy,
            )
            .ok()?;
            if checked.0 != *core || checked.1 != *vcs {
                return None;
            }
            for parent in &parents {
                storage.validate_handle(parent.as_output_ref()).ok()?;
                publisher
                    .validate_output(
                        input.snapshot,
                        parent.as_output_ref(),
                        OutputAuthorization::Published,
                    )
                    .ok()?;
            }
            publisher.validate_current_snapshot(input.snapshot).ok()?;
            let bytes = format!(
                "{}:{checked:?}",
                self.verifier_policy.policy_fingerprint().to_lower_hex()
            )
            .into_bytes();
            let retained_bytes = bytes.clone();
            let retained = checked.clone();
            let (origin, target) = source_publication_route(&version.origin);
            let handle = publish_native_output(
                &publisher,
                input.snapshot,
                (
                    IrPipelinePhase::from(PipelinePhase::KernelCheck),
                    work_unit,
                    OutputKind::new("SourceTheoremReflexivityKernelStatus"),
                ),
                |slot, phase, work_unit, output_kind| PublishOutputInput {
                    slot,
                    snapshot: input.snapshot,
                    phase,
                    work_unit,
                    output_kind,
                    schema_version: SchemaVersion::new(1),
                    payload: checked,
                    canonical_payload: Some(bytes.clone()),
                    storage_payload: Some(bytes),
                    decode: BlobDecoder::new(move |bytes| {
                        if bytes == retained_bytes {
                            Ok(retained.clone())
                        } else {
                            Err(BlobDecodeError::new(
                                "invalid retained reflexivity Kernel status",
                            ))
                        }
                    }),
                    parents: parents
                        .iter()
                        .map(|parent| parent.as_output_ref().clone())
                        .collect(),
                    named_input_hashes: named,
                    side_tables: IrSideTables::default(),
                    origin,
                    target,
                },
            )
            .ok()?;
            Some(PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            })
        })();
        result.unwrap_or_else(blocking)
    }
}
