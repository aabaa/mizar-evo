use super::*;

impl<'a> crate::registry::SourceLoadInputs<'a> {
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem candidate requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem candidate requires one VC parent".to_owned())?;
        let discharge_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcDischarge)
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremDischarge")
        }))
        .ok_or_else(|| "native theorem candidate requires one Discharge parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let vc_ref = vc_parent.as_output_ref();
        let discharge_ref = discharge_parent.as_output_ref();
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
        for parent in [core_ref, vc_ref, discharge_ref] {
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem candidate publication requires one Core parent".to_owned())?
        .as_output_ref();
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
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
        let Some(semantic_bytes) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
            return Ok(None);
        };
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-candidate-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::from(PipelinePhase::AtpSolve);
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
        ));
        let output_kind = OutputKind::new("SourceTheoremCandidateInputs");
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
                origin,
                target,
            },
        )
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem candidate check requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem candidate check requires one VC parent".to_owned())?;
        let discharge_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcDischarge)
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremDischarge")
        }))
        .ok_or_else(|| "native theorem candidate check requires one Discharge parent".to_owned())?;
        let atp_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::AtpSolve)
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremCandidateInputs")
        }))
        .ok_or_else(|| "native theorem candidate check requires one ATP parent".to_owned())?;
        let core_ref = core_parent.as_output_ref();
        let vc_ref = vc_parent.as_output_ref();
        let discharge_ref = discharge_parent.as_output_ref();
        let atp_ref = atp_parent.as_output_ref();
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
        for parent in [core_ref, vc_ref, discharge_ref, atp_ref] {
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| {
            "native theorem candidate check publication requires one Core parent".to_owned()
        })?
        .as_output_ref();
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| {
            "native theorem candidate check publication requires one VC parent".to_owned()
        })?
        .as_output_ref();
        let atp_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::AtpSolve)
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
        let Some(semantic_bytes) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
            return Ok(None);
        };
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-driver/native-theorem-candidate-check-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::from(PipelinePhase::KernelCheck);
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
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
                origin,
                target,
            },
        )
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::Elaboration)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremCore")
        }))
        .ok_or_else(|| "native theorem candidate selection requires one Core parent".to_owned())?;
        let vc_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem candidate selection requires one VC parent".to_owned())?;
        let discharge_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcDischarge)
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremDischarge")
        }))
        .ok_or_else(|| {
            "native theorem candidate selection requires one Discharge parent".to_owned()
        })?;
        let kernel_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::KernelCheck)
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
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
            || lineage.phase != IrPipelinePhase::from(PipelinePhase::AtpSolve)
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
        let Some(source_metadata) =
            self.native_theorem_source_metadata(module, "native theorem candidate selection")?
        else {
            return Ok(None);
        };
        let (arena, env, _, typed, _) = source_metadata.as_ref();
        let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(arena, env)?;
        let check =
            mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                arena, typed, &scope, env, &labels, &resolved, None,
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
            arena,
            &check,
            env,
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
                .map_err(|error| error.to_string())?;
        }
        Ok(Some(selection))
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::KernelCheck)
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
        let phase = IrPipelinePhase::from(PipelinePhase::KernelCheck);
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
        ));
        let kind = OutputKind::new("SourceTheoremCandidateWitnessFile");
        for parent in input.parent_outputs() {
            publisher
                .storage()
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
                origin,
                target,
            })
            .map(Some)
            .map_err(|error| error.to_string())
    }

    #[expect(
        clippy::type_complexity,
        reason = "keeps actual source and checker products together"
    )]
    pub(super) fn native_theorem_source_metadata(
        self,
        module: &mizar_build::module_index::ModuleId,
        prefix: &str,
    ) -> Result<
        Option<
            Box<(
                mizar_resolve::resolved_ast::SurfaceResolvedArena,
                mizar_resolve::env::SymbolEnv,
                mizar_checker::type_checker::TermFormulaInferenceOutput,
                mizar_checker::typed_ast::TypedAst,
                mizar_checker::resolved_typed_ast::ResolvedTypedAst,
            )>,
        >,
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
        Ok(Some(Box::new((
            arena, env, metadata.1, metadata.2, metadata.3,
        ))))
    }

    #[expect(
        clippy::type_complexity,
        reason = "retains genuine selection and complete witness products"
    )]
    pub(super) fn native_theorem_candidate_witness_payload(
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        }))
        .ok_or_else(|| "native theorem candidate witness file requires one VC parent".to_owned())?;
        let kernel_parent = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::KernelCheck)
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
        let Some(source_metadata) =
            self.native_theorem_source_metadata(module, "native theorem candidate witness file")?
        else {
            return Ok(None);
        };
        let (arena, env, _, typed, _) = source_metadata.as_ref();
        let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(arena, env)?;
        let check =
            mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                arena, typed, &scope, env, &labels, &resolved, None,
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
                arena,
                &check,
                env,
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
                .validate_output(
                    input.snapshot,
                    parent.as_output_ref(),
                    OutputAuthorization::Published,
                )
                .map_err(|error| error.to_string())?;
        }
        Ok(Some((selection, (products, file, reference))))
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
                (PipelinePhase::Elaboration, "SourceTheoremCore"),
                (PipelinePhase::VcDischarge, "SourceTheoremDischarge"),
            ]
            .into_iter()
            .any(|(phase, kind)| {
                unique(input.parent_outputs().iter().filter(|parent| {
                    parent.as_output_ref().phase() == &IrPipelinePhase::from(phase)
                        && parent.as_output_ref().output_kind() == &OutputKind::new(kind)
                }))
                .is_none()
            })
        {
            return blocking();
        }
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
                path.push_str(&result.output_refs[0].content_hash().to_lower_hex());
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
    let Some(size) = framing::checked_size(domain, &fields, 32 * 1024 * 1024) else {
        return Ok(None);
    };
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
    let Some(bytes) = framing::encode(domain, &fields, size) else {
        return Ok(None);
    };
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
    let Some(fields) = framing::decode::<6>(
        bytes,
        b"mizar-driver/native-theorem-candidate-storage/v1\0",
        32 * 1024 * 1024,
    ) else {
        return Ok(None);
    };
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
    let Some(size) = framing::checked_size(domain, &fields, 32 * 1024 * 1024) else {
        return Ok(None);
    };
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
    let Some(bytes) = framing::encode(domain, &fields, size) else {
        return Ok(None);
    };
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
    let Some(fields) = framing::decode::<2>(bytes, domain, 32 * 1024 * 1024) else {
        return Ok(None);
    };
    if fields[1].len() != 32 {
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
