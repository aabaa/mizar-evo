use super::*;

impl<'a> crate::registry::SourceLoadInputs<'a> {
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
            .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
                origin,
                target,
            },
        )
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
            .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
        let Some(content) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
            return Ok(None);
        };
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
                origin,
                target,
            },
        )
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
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
                origin,
                target,
            },
        )
        .map(Some)
        .map_err(|error| error.to_string())
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
    let Some(bytes) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
        return Ok(None);
    };
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
    let Some(bytes) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
        return Ok(None);
    };
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
    let Some(fields) = framing::decode::<4>(bytes, domain, 32 * 1024 * 1024) else {
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
    let Some(bytes) = framing::encode(domain, &fields, 32 * 1024 * 1024) else {
        return Ok(None);
    };
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
    mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope).map(Some)
}
