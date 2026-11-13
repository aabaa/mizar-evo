//! Sealed source-authenticated pending registration and obligation intake.

use crate::{
    registration_resolution::{
        RegistrationDatabase, RegistrationValidationInput, check_source_registration_sequence,
    },
    type_checker::TermFormulaInferenceOutput,
    typed_ast::{
        InitialObligationId, InitialObligationKind, InitialObligationStatus, TypedAst,
        TypedAstParts, TypedSiteRef,
    },
};
use mizar_resolve::{env::SymbolEnv, resolved_ast::SurfaceResolvedArena};
use mizar_session::SourceAnchor;
use std::{collections::BTreeSet, sync::Arc};

fn invalid() -> String {
    "source.registration.unsupported_source".to_owned()
}

/// Native-authenticated outputs; construction is the only admission path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRegistrationHandoff {
    original: Arc<TypedAst>,
    database: RegistrationDatabase,
    inference: TermFormulaInferenceOutput,
    validations: Vec<RegistrationValidationInput>,
    correctness: Vec<(TypedSiteRef, InitialObligationId, SourceAnchor)>,
}

impl SourceRegistrationHandoff {
    pub fn build(
        original: Arc<TypedAst>,
        native: &SurfaceResolvedArena,
        symbols: &SymbolEnv,
    ) -> Result<Self, String> {
        // Reconstruct the entire syntax-only receiver, including authenticated source rows.
        let mut canonical = TypedAst::try_new(TypedAstParts {
            source_id: native.source_id(),
            module_id: native.module().clone(),
            resolved_root: Some(native.arena().root()),
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: original.nodes().clone(),
            contexts: Default::default(),
            types: Default::default(),
            facts: Default::default(),
            coercions: Default::default(),
            initial_obligations: Default::default(),
            diagnostics: Default::default(),
        })
        .map_err(|_| invalid())?;
        canonical
            .install_source_nodes(native)
            .map_err(|_| invalid())?;
        if *original != canonical {
            return Err(invalid());
        }
        let checked = check_source_registration_sequence(native, original.nodes(), symbols)?;
        let validations = checked.validations().to_vec();
        let correctness = checked.correctness().to_vec();
        let (database, inference) = checked.into_outputs();
        if database
            .pending()
            .iter()
            .zip(&validations)
            .any(|(pending, validation)| {
                pending.resolver_registration() != validation.resolver_registration()
            })
        {
            return Err(invalid());
        }
        Ok(Self {
            original,
            database,
            inference,
            validations,
            correctness,
        })
    }

    pub(crate) fn original(&self) -> &TypedAst {
        &self.original
    }

    pub const fn database(&self) -> &RegistrationDatabase {
        &self.database
    }

    pub const fn inference(&self) -> &TermFormulaInferenceOutput {
        &self.inference
    }

    pub fn validations(&self) -> &[RegistrationValidationInput] {
        &self.validations
    }

    pub fn correctness(
        &self,
    ) -> impl Iterator<Item = (&TypedSiteRef, InitialObligationId, &SourceAnchor)> {
        self.correctness
            .iter()
            .map(|(site, id, anchor)| (site, *id, anchor))
    }

    pub(crate) fn validate_intake(&self) -> Result<(), String> {
        let obligations = self.database.initial_obligations();
        let count = self.correctness.len();
        if count == 0
            || count != self.database.pending().len()
            || count != self.validations.len()
            || count != obligations.len()
            || !self.database.activated().is_empty()
            || !self.database.rejected().is_empty()
            || !self.database.diagnostics().is_empty()
        {
            return Err(invalid());
        }
        let (mut ids, mut sites, mut owners) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        for ((pending, validation), (site, id, anchor)) in self
            .database
            .pending()
            .iter()
            .zip(&self.validations)
            .zip(&self.correctness)
        {
            let obligation = obligations.get(*id).ok_or_else(invalid)?;
            let node = self
                .original
                .nodes()
                .node(site.node())
                .ok_or_else(invalid)?;
            let SourceAnchor::Range(range) = anchor else {
                return Err(invalid());
            };
            let owner_range = validation.source_range();
            if !matches!(site, TypedSiteRef::Node(_))
                || node.anchor != *anchor
                || range.source_id != owner_range.source_id
                || range.start < owner_range.start
                || range.end > owner_range.end
                || site.node() == validation.owner().node()
                || !ids.insert(*id)
                || !sites.insert(site.node())
                || !owners.insert(validation.owner().node())
                || pending.resolver_registration() != validation.resolver_registration()
                || pending.obligations() != [*id]
                || obligation.id != *id
                || obligation.kind != InitialObligationKind::RegistrationCorrectness
                || obligation.status != InitialObligationStatus::Pending
                || obligation.owner != *validation.owner()
                || obligation.source_range != owner_range
                || obligation.assumptions != validation.assumptions()
                || obligation.goal != *validation.correctness_goal()
                || obligation.provenance != *validation.correctness_provenance()
            {
                return Err(invalid());
            }
        }
        Ok(())
    }

    pub(crate) fn validate_installation(&self, ast: &TypedAst) -> Result<(), String> {
        let expected = self
            .original
            .as_ref()
            .clone()
            .with_source_registration(self.clone())
            .map_err(|_| invalid())?;
        if *ast != expected {
            return Err(invalid());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_ast::{TypedArena, TypedNode, TypedNodeId};
    use mizar_frontend::{
        lexical_env::{
            FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider,
            ResolvedImports,
        },
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector, env::NamespacePath, resolved_ast::ModuleId,
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath,
        PackageId, SourceInput, SourceOriginInput,
    };

    struct NoImports;
    impl LexicalSummaryProvider for NoImports {
        fn resolve_imports(
            &self,
            request: &LexicalEnvironmentRequest<'_>,
        ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
            assert!(request.import_stubs.is_empty());
            Ok(ResolvedImports {
                imports: vec![],
                summaries: vec![],
                diagnostics: vec![],
            })
        }
    }

    fn fixture() -> SourceRegistrationHandoff {
        let root = std::env::temp_dir().join(format!("checker273-private-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        let path = root.join("src/fixture.miz");
        let real = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../tests/miz/pass/clusters/pass_advanced_semantics_existential_registration_001.miz",
        );
        std::fs::copy(real, &path).unwrap();
        let module = ModuleId::new(PackageId::new("private-intake"), ModulePath::new("fixture"));
        let output = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            NoImports,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                    "mizar-session-build-snapshot-v1:{:064x}",
                    1
                ))
                .unwrap(),
                input: SourceInput {
                    package_id: module.package().clone(),
                    module_path: module.path().clone(),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();
        std::fs::remove_dir_all(root).unwrap();
        assert!(output.diagnostics.is_empty());
        let ast = output.ast.unwrap();
        let shells = DeclarationShellCollector::new(&ast, &module).collect();
        let collected =
            SignatureProjectionExtractor::new(&ast, &shells, NamespacePath::new("fixture"))
                .collect(&module);
        assert!(collected.diagnostics().is_empty());
        let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
        let mut original = TypedAst::try_new(TypedAstParts {
            source_id: native.source_id(),
            module_id: module,
            resolved_root: Some(native.arena().root()),
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: TypedArena::try_new(
                Some(TypedNodeId::new(native.arena().root().index())),
                native
                    .arena()
                    .iter()
                    .map(|(id, node)| {
                        TypedNode::new(format!("{:?}", node.kind()), node.origin().anchor().clone())
                            .with_resolved_node(id)
                            .with_children(
                                node.children()
                                    .iter()
                                    .map(|id| TypedNodeId::new(id.index()))
                                    .collect(),
                            )
                    })
                    .collect(),
            )
            .unwrap(),
            contexts: Default::default(),
            types: Default::default(),
            facts: Default::default(),
            coercions: Default::default(),
            initial_obligations: Default::default(),
            diagnostics: Default::default(),
        })
        .unwrap();
        original.install_source_nodes(&native).unwrap();
        let original = Arc::new(original);
        let symbols = collected.into_env();
        let handoff =
            SourceRegistrationHandoff::build(original.clone(), &native, &symbols).unwrap();
        assert_eq!(
            handoff,
            SourceRegistrationHandoff::build(original, &native, &symbols).unwrap()
        );
        handoff
    }

    #[test]
    fn registration_intake_rejects_corrupted_correctness_identity() {
        let valid = fixture();
        let installed = valid
            .original()
            .clone()
            .with_source_registration(valid.clone())
            .unwrap();
        valid.validate_installation(&installed).unwrap();
        for mutation in 0..6 {
            let mut bad = valid.clone();
            match mutation {
                0 => bad.correctness.clear(),
                1 => bad.correctness[0].1 = InitialObligationId::new(usize::MAX),
                2 => {
                    bad.correctness[0].2 = SourceAnchor::Point {
                        source_id: valid.original.source_id(),
                        offset: 0,
                    }
                }
                3 => {
                    bad.correctness[0].2 = valid
                        .original
                        .nodes()
                        .node(valid.original.nodes().root().unwrap())
                        .unwrap()
                        .anchor
                        .clone()
                }
                4 => bad.correctness[0].0 = valid.validations[0].owner().clone(),
                _ => bad.correctness.push(bad.correctness[0].clone()),
            }
            assert!(
                valid
                    .original()
                    .clone()
                    .with_source_registration(bad.clone())
                    .is_err(),
                "{mutation}"
            );
            assert!(bad.validate_installation(&installed).is_err(), "{mutation}");
        }
    }
}
