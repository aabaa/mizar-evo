//! Module and symbol resolution for Mizar Evo.
//!
//! This crate owns phases 4-5 of the pipeline. It currently exposes the
//! resolver-owned `ResolvedAst` and `SymbolEnv` data shapes, the module-index
//! input seam, source-shaped import path and graph resolution,
//! declaration-shell collection, namespace lookup, preliminary symbol-name
//! lookup, crate-local/internal name diagnostics, dot-chain finalization, and
//! executable label resolution plus declaration-symbol collection and
//! parser-backed per-kind signature projection. Recovered syntax is consumed
//! according to the resolver-local policy without changing parser recovery
//! ownership.

/// Source-shaped declaration shell collection.
pub mod declarations;

/// Symbol environment data shapes and deterministic indexes.
pub mod env;

/// Semantic import graph construction and cycle rejection.
pub mod imports;

/// Label declaration and citation resolution.
pub mod labels;

/// Resolver-side module-index phase input seam.
pub mod module_index;

/// Dependency `ModuleSummary` reuse through canonical artifact projections.
pub mod module_summary_reuse;

/// Namespace, preliminary symbol-name resolution, internal diagnostics, and
/// dot-chain finalization.
pub mod names;

/// Resolved AST data shapes and reference tables.
pub mod resolved_ast;

mod recovery;

/// Symbol/signature projection and collection.
pub mod symbols;

#[cfg(test)]
mod determinism_tests {
    use super::env::{
        ContributionKind, DependencyEndpoint, ExportIndexEntry, ImportIndexEntry,
        ResolvedExportIndex, ResolvedImportIndex, SourceContributionIndex, SymbolEnv,
        SymbolEnvIndexes,
    };
    use super::imports::{ImportEdgeCandidate, ImportGraphBuilder, ModuleImportCandidates};
    use super::module_index::{
        IndexedModuleId, ModuleIndexEntry, ModuleIndexInput, ModuleIndexLocation,
        NamespaceIndexEntry, NamespaceRoot, PackageIndexEntry, WorkspaceStubModuleIndexProvider,
    };
    use super::names::NameDiagnosticCollector;
    use super::resolved_ast::{
        ExportTarget, FullyQualifiedName, ImportResolution, LocalSymbolId, ModuleId,
        NameLookupClass, NameRefEntry, NameRefTable, NameResolution, NodeResolutionState,
        ReferenceSite, ResolvedArenaBuilder, ResolvedAst, ResolvedExport, ResolvedImport,
        ResolvedImports, ResolvedNode, SemanticOrigin, SymbolId, SymbolRef, UnresolvedNameRef,
    };
    use mizar_build::module_index::PackageIndexSource;
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceAnchor, SourceId, SourceRange,
    };
    use mizar_syntax::SurfaceNodeKind;
    use semver::Version;

    #[test]
    fn resolver_public_seams_are_deterministic_for_equivalent_inputs() {
        let first = determinism_observation();
        let second = determinism_observation();

        assert_eq!(first, second);
        assert!(!first.import_graph.has_cycles());
        let diagnostic_spellings = first
            .name_diagnostics
            .iter()
            .map(super::names::NameDiagnostic::attempted_spelling)
            .collect::<Vec<_>>();
        assert_eq!(diagnostic_spellings, ["AlphaMissing", "Missing"]);
        assert!(
            first
                .resolved_ast_snapshot
                .contains("resolved-ast-debug-v1")
        );
        assert!(first.symbol_env_snapshot.contains("symbol-env-debug-v1"));
        assert!(!first.resolved_ast_snapshot.contains("SourceId"));
        assert!(!first.symbol_env_snapshot.contains("SourceId"));
    }

    #[derive(Debug, PartialEq, Eq)]
    struct DeterminismObservation {
        import_graph: super::imports::ImportGraphResolution,
        resolved_ast_snapshot: String,
        name_diagnostics: super::names::NameDiagnosticReport,
        symbol_env_snapshot: String,
    }

    fn determinism_observation() -> DeterminismObservation {
        let provider = fixture_provider();
        let import_graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
            .build(&graph_candidates())
            .unwrap();
        let source_id = source_id(25);
        let (resolved_ast, import_id, export_id) = resolved_ast_fixture(source_id);
        let name_report = NameDiagnosticCollector::new().collect(resolved_ast.name_refs());
        let symbol_env = symbol_env_fixture(source_id, import_id, export_id);

        DeterminismObservation {
            import_graph,
            resolved_ast_snapshot: resolved_ast.snapshot_text(),
            name_diagnostics: name_report,
            symbol_env_snapshot: symbol_env.snapshot_text(),
        }
    }

    fn graph_candidates() -> Vec<ModuleImportCandidates> {
        let main = module_id("app", "main");
        let util = module_id("app", "util");
        let logic = module_id("dep", "logic");
        vec![
            ModuleImportCandidates::new(
                main.clone(),
                vec![
                    ImportEdgeCandidate::new(util.clone(), range(source_id(26), 10, 20), 1),
                    ImportEdgeCandidate::new(logic.clone(), range(source_id(26), 0, 9), 0),
                ],
            ),
            ModuleImportCandidates::new(
                util,
                vec![ImportEdgeCandidate::new(
                    logic,
                    range(source_id(26), 21, 30),
                    0,
                )],
            ),
        ]
    }

    fn resolved_ast_fixture(
        source_id: SourceId,
    ) -> (
        ResolvedAst,
        super::resolved_ast::ResolvedImportId,
        super::resolved_ast::ResolvedExportId,
    ) {
        let module = module_id("app", "main");
        let dep = module_id("dep", "logic");
        let root_origin = origin(source_id, module.clone(), 0, 1, &[0]);
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new("Pred/0"),
            FullyQualifiedName::new("app::main::Pred/0"),
        );

        let mut builder = ResolvedArenaBuilder::new();
        let name_node = builder
            .push(
                ResolvedNode::new(
                    SurfaceNodeKind::Reference,
                    Vec::new(),
                    origin(source_id, module.clone(), 0, 1, &[1]),
                )
                .with_resolution(NodeResolutionState::Resolved),
            )
            .unwrap();
        let unresolved_node = builder
            .push(
                ResolvedNode::new(
                    SurfaceNodeKind::Reference,
                    Vec::new(),
                    origin(source_id, module.clone(), 2, 3, &[2]),
                )
                .with_resolution(NodeResolutionState::Unresolved),
            )
            .unwrap();
        let early_unresolved_node = builder
            .push(
                ResolvedNode::new(
                    SurfaceNodeKind::Reference,
                    Vec::new(),
                    origin(source_id, module.clone(), 1, 2, &[5]),
                )
                .with_resolution(NodeResolutionState::Unresolved),
            )
            .unwrap();
        let import_node = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::ImportItem,
                Vec::new(),
                origin(source_id, module.clone(), 4, 5, &[3]),
            ))
            .unwrap();
        let export_node = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::ExportItem,
                Vec::new(),
                origin(source_id, module.clone(), 6, 7, &[4]),
            ))
            .unwrap();
        let root = builder
            .push(ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                vec![
                    name_node,
                    early_unresolved_node,
                    unresolved_node,
                    import_node,
                    export_node,
                ],
                root_origin,
            ))
            .unwrap();
        let arena = builder.finish(root).unwrap();

        let mut name_refs = NameRefTable::new();
        name_refs.insert(NameRefEntry::new(
            ReferenceSite::new(name_node, range(source_id, 0, 1), "Pred"),
            NameResolution::Resolved(SymbolRef::new(symbol.clone(), range(source_id, 0, 1))),
            origin(source_id, module.clone(), 0, 1, &[1]),
        ));
        name_refs.insert(NameRefEntry::new(
            ReferenceSite::new(
                early_unresolved_node,
                range(source_id, 1, 2),
                "AlphaMissing",
            ),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "AlphaMissing",
                range(source_id, 1, 2),
                NameLookupClass::Symbol,
            )),
            origin(source_id, module.clone(), 1, 2, &[5]),
        ));
        name_refs.insert(NameRefEntry::new(
            ReferenceSite::new(unresolved_node, range(source_id, 2, 3), "Missing"),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "Missing",
                range(source_id, 2, 3),
                NameLookupClass::Symbol,
            )),
            origin(source_id, module.clone(), 2, 3, &[2]),
        ));

        let mut imports = ResolvedImports::new();
        let import_id = imports.push_import(ResolvedImport::new(
            import_node,
            range(source_id, 4, 5),
            "import dep.logic as Logic;",
            Some("Logic".to_owned()),
            ImportResolution::Resolved(dep.clone()),
            origin(source_id, module.clone(), 4, 5, &[3]),
        ));
        let export_id = imports.push_export(ResolvedExport::new(
            export_node,
            range(source_id, 6, 7),
            "export Pred;",
            ExportTarget::Symbol(symbol),
            origin(source_id, module.clone(), 6, 7, &[4]),
        ));

        (
            ResolvedAst::try_new(
                source_id,
                module,
                arena,
                name_refs,
                super::resolved_ast::LabelRefTable::new(),
                imports,
            )
            .unwrap(),
            import_id,
            export_id,
        )
    }

    fn symbol_env_fixture(
        source_id: SourceId,
        import_id: super::resolved_ast::ResolvedImportId,
        export_id: super::resolved_ast::ResolvedExportId,
    ) -> SymbolEnv {
        let module = module_id("app", "main");
        let dep = module_id("dep", "logic");
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id },
            SourceAnchor::Range(range(source_id, 0, 7)),
        );
        contributions.add_import(contribution, import_id);
        contributions.add_export(contribution, export_id);

        let mut imports = ResolvedImportIndex::new();
        imports.insert(ImportIndexEntry::new(
            import_id,
            Some(dep.clone()),
            Some("Logic".to_owned()),
            contribution,
        ));
        let mut exports = ResolvedExportIndex::new();
        exports.insert(ExportIndexEntry::new(
            export_id,
            Some(DependencyEndpoint::Module(dep)),
            contribution,
        ));

        SymbolEnv::new(
            module,
            SymbolEnvIndexes {
                imports,
                exports,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        )
    }

    fn fixture_provider() -> WorkspaceStubModuleIndexProvider {
        WorkspaceStubModuleIndexProvider::new(
            vec![package_entry("app"), package_entry("dep")],
            vec![
                NamespaceIndexEntry {
                    root: NamespaceRoot::PackageName,
                    prefix: vec!["app".to_owned()],
                    package_id: PackageId::new("app"),
                },
                NamespaceIndexEntry {
                    root: NamespaceRoot::PackageName,
                    prefix: vec!["dep".to_owned()],
                    package_id: PackageId::new("dep"),
                },
            ],
            vec![
                module_entry("app", "main"),
                module_entry("app", "util"),
                module_entry("dep", "logic"),
            ],
            Vec::new(),
        )
    }

    fn package_entry(package: &str) -> PackageIndexEntry {
        PackageIndexEntry {
            package_id: PackageId::new(package),
            version: Version::new(0, 1, 0),
            edition: Edition::new("2026"),
            source: PackageIndexSource::Workspace {
                package_root: format!("packages/{package}"),
                source_root: format!("packages/{package}/src"),
                manifest_path: format!("packages/{package}/mizar.toml"),
            },
            dependencies: Vec::new(),
        }
    }

    fn module_entry(package: &str, path: &str) -> ModuleIndexEntry {
        ModuleIndexEntry {
            module: IndexedModuleId {
                package: PackageId::new(package),
                path: ModulePath::new(path),
            },
            package_id: PackageId::new(package),
            module_path: ModulePath::new(path),
            location: ModuleIndexLocation::WorkspaceFile {
                source_root: format!("packages/{package}/src"),
                normalized_path: format!("packages/{package}/src/{path}.miz"),
                source_relative_path: format!("{path}.miz"),
            },
            edition: Edition::new("2026"),
        }
    }

    fn module_id(package: &str, path: &str) -> ModuleId {
        ModuleId::new(PackageId::new(package), ModulePath::new(path))
    }

    fn origin(
        source_id: SourceId,
        module: ModuleId,
        start: usize,
        end: usize,
        structural_path: &[u32],
    ) -> SemanticOrigin {
        SemanticOrigin::new(
            source_id,
            module,
            SourceAnchor::Range(range(source_id, start, end)),
            structural_path.to_vec(),
        )
    }

    fn source_id(seed: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(seed))
            .unwrap()
    }

    fn snapshot_id(seed: u8) -> BuildSnapshotId {
        let hex = format!("{seed:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }
}
