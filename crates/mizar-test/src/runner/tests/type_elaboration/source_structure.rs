use super::{
    ImportedStructureConstructorSurfaceMutation, ImportedStructureConstructorTestMutation,
    ImportedStructureConstructorTestOptions, SourceStructureRouteOutput,
    ImportedStructureSelectorSurfaceMutation, ImportedStructureSelectorTestMutation,
    ImportedStructureSelectorTestOptions, SyntheticSourceStructureDependencies,
    imported_structure_constructor_handoff_for_test, imported_structure_selector_handoff_for_test,
    source_structure_output, source_structure_output_with_mutation,
    synthetic_source_structure_output, synthetic_source_structure_output_with_mutation,
};
use mizar_resolve::env::SourceContributionIndex;

macro_rules! b2p_structure_handoff {
    (
        $ast:expr,
        $module:expr,
        $symbols:expr,
        $binding_env:expr,
        $loaded_source:expr,
        $roots:expr,
        $constructor:expr,
        $context:expr,
        $surface_mutation:expr,
        $handoff_mutation:expr $(,)?
    ) => {
        imported_structure_constructor_handoff_for_test(
            $ast,
            $module,
            $symbols,
            $binding_env,
            $loaded_source,
            $roots,
            ImportedStructureConstructorTestOptions {
                constructor: $constructor,
                context: $context,
                surface_mutation: $surface_mutation,
                handoff_mutation: $handoff_mutation,
            },
        )
    };
}

macro_rules! b2bp_structure_handoff {
    (
        $ast:expr,
        $module:expr,
        $symbols:expr,
        $binding_env:expr,
        $loaded_source:expr,
        $roots:expr,
        $selector:expr,
        $context:expr,
        $surface_mutation:expr,
        $handoff_mutation:expr $(,)?
    ) => {
        imported_structure_selector_handoff_for_test(
            $ast,
            $module,
            $symbols,
            $binding_env,
            $loaded_source,
            $roots,
            ImportedStructureSelectorTestOptions {
                selector: $selector,
                context: $context,
                surface_mutation: $surface_mutation,
                handoff_mutation: $handoff_mutation,
            },
        )
    };
}

const TASK258B3M2B2B2P_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureConstructorWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2);\n",
    "  thus x = x;\n",
    "end;\n",
);

const TASK258B3M2B2B2BP_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementStructureSelectorWitnessSmoke: x = x proof\n",
    "  take TypeCaseStruct(x: 1, y: 2).x;\n",
    "  thus x = x;\n",
    "end;\n",
);

fn task258b3m2b2b2bp_frontend_diagnostic_profile(
    source: &str,
    ordinal: usize,
) -> Vec<(String, usize, usize)> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let package_root = std::env::temp_dir().join(format!(
        "mizar-test-task254-selector-diagnostic-{}-{ordinal}-{unique}",
        std::process::id()
    ));
    let source_path = package_root.join("src").join("task254_selector.miz");
    std::fs::create_dir_all(source_path.parent().expect("source parent"))
        .expect("create Task 254 selector diagnostic package");
    std::fs::write(&source_path, source).expect("write Task 254 selector diagnostic source");
    let package = PackageId::new("mizar-test-task254-selector-diagnostic");
    let module_path = ModulePath::new(format!("tests.task254_selector_diagnostic_{ordinal}"));
    let normalized_path =
        mizar_session::normalize_path(&package_root, &source_path).expect("normalize source path");
    let frontend = mizar_frontend::orchestration::Frontend::new(
        mizar_frontend::source::FrontendSourceLoader::new(
            mizar_session::DiskSourceLoader::new(&package_root),
        ),
        ParseOnlyImportProvider,
        mizar_frontend::parsing::MizarParserSeam,
    );
    let output = frontend
        .run(
            mizar_frontend::source::SourceUnitRequest {
                snapshot: super::shared::snapshot_id(20_000 + ordinal),
                input: mizar_session::SourceInput {
                    package_id: package,
                    module_path,
                    normalized_path,
                    edition: Edition::new("2026"),
                    origin: mizar_session::SourceOriginInput::Disk { path: source_path },
                },
            },
            &InMemorySessionIdAllocator::new(),
        )
        .expect("Task 254 selector diagnostic frontend should run");
    std::fs::remove_dir_all(&package_root)
        .expect("clean Task 254 selector diagnostic package");
    output
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            let code = match diagnostic.code {
                mizar_frontend::orchestration::DiagnosticCode::Syntax(code) => code.to_string(),
                other => format!("{other:?}"),
            };
            let mizar_frontend::orchestration::DiagnosticLocation::SourceRange(range) =
                diagnostic.location
            else {
                panic!("Task 254 selector diagnostic must have a source range");
            };
            (code, range.start, range.end)
        })
        .collect()
}

fn task258b3m2b2b2p_roots() -> [(usize, mizar_checker::binding_env::BindingContextId); 6] {
    let module = mizar_checker::binding_env::BindingContextId::new(0);
    let proof = mizar_checker::binding_env::BindingContextId::new(1);
    [
        (45, module),
        (47, module),
        (54, proof),
        (57, proof),
        (63, proof),
        (65, proof),
    ]
}

fn task258b3m2b2b2bp_roots() -> [(usize, mizar_checker::binding_env::BindingContextId); 6] {
    let module = mizar_checker::binding_env::BindingContextId::new(0);
    let proof = mizar_checker::binding_env::BindingContextId::new(1);
    [
        (47, module),
        (49, module),
        (56, proof),
        (59, proof),
        (66, proof),
        (68, proof),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task258B3M2B2B2PResolverMutation {
    LocalAndFqn,
    ModulePackage,
    ModulePath,
    SymbolKind,
    PrimarySpelling,
    StructuralPath,
    OriginSource,
    OriginModule,
    OriginAnchor,
    OriginImportEdge,
    OriginRecovery,
    Signature,
    Visibility,
    ExportStatus,
    Namespace,
    Contribution,
    ContributionKind,
    ContributionModule,
    ContributionAnchor,
}

fn task258b3m2b2b2p_rebuild_contributions(
    indexes: &mut SymbolEnvIndexes,
    selected: mizar_resolve::env::SourceContributionId,
    current_module: &ResolverModuleId,
    mutation: Task258B3M2B2B2PResolverMutation,
) {
    if !matches!(
        mutation,
        Task258B3M2B2B2PResolverMutation::ContributionKind
            | Task258B3M2B2B2PResolverMutation::ContributionModule
            | Task258B3M2B2B2PResolverMutation::ContributionAnchor
    ) {
        return;
    }
    let records = indexes.contributions.iter().cloned().collect::<Vec<_>>();
    let mut rebuilt = SourceContributionIndex::new();
    for record in records {
        let selected_record = record.id() == selected;
        let module = if selected_record
            && mutation == Task258B3M2B2B2PResolverMutation::ContributionModule
        {
            current_module.clone()
        } else {
            record.module().clone()
        };
        let kind = if selected_record
            && mutation == Task258B3M2B2B2PResolverMutation::ContributionKind
        {
            ContributionKind::LocalSource {
                source_id: record
                    .kind()
                    .source_id()
                    .expect("B2P selected contribution is source-backed"),
            }
        } else {
            record.kind().clone()
        };
        let anchor = if selected_record
            && mutation == Task258B3M2B2B2PResolverMutation::ContributionAnchor
        {
            let SourceAnchor::Range(range) = record.anchor() else {
                panic!("B2P selected contribution range");
            };
            SourceAnchor::Range(SourceRange {
                source_id: range.source_id,
                start: range.start + 1,
                end: range.end,
            })
        } else {
            record.anchor().clone()
        };
        let id = rebuilt.insert(module, kind, anchor);
        assert_eq!(
            id.index(),
            record.id().index(),
            "B2P contribution rebuild must retain dense identity"
        );
        for symbol in record.effects().symbols() {
            rebuilt.add_symbol(id, symbol.clone());
        }
        for definition in record.effects().definitions() {
            rebuilt.add_definition(id, *definition);
        }
        for group in record.effects().overload_groups() {
            rebuilt.add_overload_group(id, *group);
        }
        for registration in record.effects().registrations() {
            rebuilt.add_registration(id, *registration);
        }
        for summary in record.effects().lexical_summaries() {
            rebuilt.add_lexical_summary(id, *summary);
        }
        for label in record.effects().labels() {
            rebuilt.add_label(id, label.clone());
        }
        for edge in record.effects().namespace_edges() {
            rebuilt.add_namespace_edge(id, *edge);
        }
        for dependency in record.effects().declaration_dependencies() {
            rebuilt.add_declaration_dependency(id, *dependency);
        }
        for import in record.effects().imports() {
            rebuilt.add_import(id, *import);
        }
        for export in record.effects().exports() {
            rebuilt.add_export(id, *export);
        }
        for diagnostic in record.effects().diagnostics() {
            rebuilt.add_diagnostic(id, *diagnostic);
        }
    }
    indexes.contributions = rebuilt;
}

fn task258b3m2b2b2p_substituted_symbol_env(
    symbols: &SymbolEnv,
    mutation: Task258B3M2B2B2PResolverMutation,
) -> SymbolEnv {
    let selected = symbols
        .symbols()
        .iter()
        .find(|entry| {
            entry.symbol().module().path().as_str() == "parser.type_fixtures"
                && entry.kind() == SymbolKind::Structure
                && entry.primary_spelling() == "TypeCaseStruct"
        })
        .expect("B2P imported TypeCaseStruct entry")
        .clone();
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(symbols);
    task258b3m2b2b2p_rebuild_contributions(
        &mut indexes,
        selected.contribution(),
        symbols.module_id(),
        mutation,
    );
    let contribution = if mutation == Task258B3M2B2B2PResolverMutation::Contribution {
        indexes.contributions.insert(
            selected.symbol().module().clone(),
            ContributionKind::ImportedSource {
                source_id: selected.origin().source_id(),
            },
            selected.origin().anchor().clone(),
        )
    } else {
        selected.contribution()
    };
    let symbol_module = if mutation == Task258B3M2B2B2PResolverMutation::ModulePackage {
        ResolverModuleId::new(
            PackageId::new("mizar-test-b2p-substitute"),
            selected.symbol().module().path().clone(),
        )
    } else if mutation == Task258B3M2B2B2PResolverMutation::ModulePath {
        ResolverModuleId::new(
            selected.symbol().module().package().clone(),
            ModulePath::new("parser.other_fixtures"),
        )
    } else {
        selected.symbol().module().clone()
    };
    let symbol = if mutation == Task258B3M2B2B2PResolverMutation::LocalAndFqn {
        ResolverSymbolId::new(
            symbol_module,
            LocalSymbolId::new(
                "summary:parser.type_fixtures#parse-only#TypeCaseStruct:substitute",
            ),
            FullyQualifiedName::new("parser.type_fixtures::TypeCaseStruct#substitute"),
        )
    } else if matches!(
        mutation,
        Task258B3M2B2B2PResolverMutation::ModulePackage
            | Task258B3M2B2B2PResolverMutation::ModulePath
    ) {
        ResolverSymbolId::new(
            symbol_module,
            selected.symbol().local().clone(),
            selected.symbol().fqn().clone(),
        )
    } else {
        selected.symbol().clone()
    };
    let symbol_kind = if mutation == Task258B3M2B2B2PResolverMutation::SymbolKind {
        SymbolKind::Mode
    } else {
        selected.kind()
    };
    let primary_spelling =
        if mutation == Task258B3M2B2B2PResolverMutation::PrimarySpelling {
            "TypeCaseStructDrift"
        } else {
            selected.primary_spelling()
        };
    let structural_path =
        if mutation == Task258B3M2B2B2PResolverMutation::StructuralPath {
            vec![6]
        } else {
            selected.origin().structural_path().to_vec()
        };
    let origin_source = if mutation == Task258B3M2B2B2PResolverMutation::OriginSource {
        let allocator = InMemorySessionIdAllocator::new();
        allocator
            .next_source_id(snapshot_id(252))
            .expect("B2P first drift source allocation");
        allocator
            .next_source_id(snapshot_id(252))
            .expect("B2P distinct drift source allocation")
    } else {
        selected.origin().source_id()
    };
    let origin_module = if mutation == Task258B3M2B2B2PResolverMutation::OriginModule {
        symbols.module_id().clone()
    } else {
        selected.origin().module_id().clone()
    };
    let origin_anchor = if mutation == Task258B3M2B2B2PResolverMutation::OriginAnchor {
        SourceAnchor::Range(SourceRange {
            source_id: origin_source,
            start: 8,
            end: 27,
        })
    } else {
        selected.origin().anchor().clone()
    };
    let mut origin =
        SemanticOrigin::new(origin_source, origin_module, origin_anchor, structural_path);
    if mutation == Task258B3M2B2B2PResolverMutation::OriginImportEdge {
        origin = origin.with_import_edge(
            symbols
                .imports()
                .iter()
                .next()
                .expect("B2P authenticated import")
                .import(),
        );
    }
    if mutation == Task258B3M2B2B2PResolverMutation::OriginRecovery {
        origin = origin.recovered();
    }
    let visibility = if mutation == Task258B3M2B2B2PResolverMutation::Visibility {
        Visibility::Private
    } else {
        selected.visibility()
    };
    let export_status = if mutation == Task258B3M2B2B2PResolverMutation::ExportStatus {
        ExportStatus::ReExported
    } else {
        selected.export_status()
    };
    let namespace = if mutation == Task258B3M2B2B2PResolverMutation::Namespace {
        NamespacePath::new("tests.b2p.substitute")
    } else {
        selected.namespace().clone()
    };
    let mut replacement = SymbolEntry::new(
        symbol.clone(),
        symbol_kind,
        namespace,
        primary_spelling,
        origin,
        contribution,
    )
    .with_visibility(visibility)
    .with_export_status(export_status)
    .with_relations(selected.relations().to_vec());
    if mutation == Task258B3M2B2B2PResolverMutation::Signature {
        replacement = replacement.with_signature(mizar_resolve::env::SignatureShell::Pending);
    } else if let Some(signature) = selected.signature().cloned() {
        replacement = replacement.with_signature(signature);
    }
    indexes.symbols = Default::default();
    for entry in symbols.symbols().iter() {
        indexes.symbols.insert(if entry.symbol() == selected.symbol() {
            replacement.clone()
        } else {
            entry.clone()
        });
    }
    indexes.contributions.add_symbol(contribution, symbol);
    SymbolEnv::new(symbols.module_id().clone(), indexes)
}

#[test]
fn task258b3m2b2b2bp_structure_selector_proof_context_reuse_is_exact() {
    assert_eq!(TASK258B3M2B2B2BP_SOURCE.len(), 171);
    assert_eq!(
        sha256_text(TASK258B3M2B2B2BP_SOURCE),
        "63f4be4d458905ba01f7510798bb87783bb90c9e6f866044be5726ce35429d00"
    );
    let (ast, module, shells, base_symbols, diagnostic_count) =
        task253_ast_from_source_text_with_diagnostic_count(
            TASK258B3M2B2B2BP_SOURCE,
            23_001,
        );
    assert_eq!(diagnostic_count, 0);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|id| id.index())),
        (79, Some(78))
    );
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, base_symbols);
    let bindings = task258b3m2b2b1p_binding_env(&ast, &module, &symbols);
    assert_eq!(
        (
            bindings.contexts().len(),
            bindings.bindings().len(),
            bindings.diagnostics().len(),
        ),
        (2, 1, 0)
    );
    assert_eq!(
        bindings.debug_text(),
        concat!(
            "binding-env-debug-v1\n",
            "module: mizar-test-task253-corruption::tests.task253_local_corruption_23001\n",
            "contexts:\n",
            "  context#0 owner=module parent=none layer=module scope=none bindings=[binding#0] visible=[binding#0] recovery=normal\n",
            "  context#1 owner=source-statement(109..169) parent=context#0 layer=proof scope=[0] bindings=[] visible=[binding#0] recovery=normal\n",
            "bindings:\n",
            "  binding#0 spelling=\"x\" kind=reserved_variable owner=context#0 identity=reserved_variable(spelling=\"x\", range=37..38) range=37..38 visible_after=0 type=source(43..46) status=reserved captured=[] diagnostics=[] recovery=normal\n",
            "diagnostics:\n",
        )
    );
    assert!(
        source_structure_output(&ast, module.clone(), &shells, &symbols).is_none(),
        "B2BP exact source must not activate the legacy Task254 route"
    );
    let roots = task258b3m2b2b2bp_roots();
    let proof = mizar_checker::binding_env::BindingContextId::new(1);
    let first = b2bp_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2BP_SOURCE,
        &roots,
        62,
        proof,
        ImportedStructureSelectorSurfaceMutation::None,
        ImportedStructureSelectorTestMutation::None,
    )
    .expect("B2BP exact selector")
    .expect("B2BP exact handoff");
    let second = b2bp_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2BP_SOURCE,
        &roots,
        62,
        proof,
        ImportedStructureSelectorSurfaceMutation::None,
        ImportedStructureSelectorTestMutation::None,
    )
    .expect("B2BP replay selector")
    .expect("B2BP replay handoff");
    assert_eq!(first.primary_counts, (6, 4, 2));
    assert_eq!(
        first
            .typed_ast
            .source_term()
            .expect("B2BP exact Task252 handoff")
            .debug_text(),
        concat!(
            "source-primary-term-debug-v1\n",
            "module: tests.task253_local_corruption_23001\n",
            "term#0 ordinal=0 kind=variable-reference role=value range=103..104 site=47 context=0 recovery=normal spelling=\"x\" parent=-\n",
            "term#1 ordinal=1 kind=variable-reference role=value range=107..108 site=49 context=0 recovery=normal spelling=\"x\" parent=-\n",
            "term#2 ordinal=2 kind=numeral role=value range=140..141 site=55 context=1 recovery=normal spelling=\"1\" parent=-\n",
            "term#3 ordinal=3 kind=numeral role=value range=146..147 site=58 context=1 recovery=normal spelling=\"2\" parent=-\n",
            "term#4 ordinal=4 kind=variable-reference role=value range=159..160 site=66 context=1 recovery=normal spelling=\"x\" parent=-\n",
            "term#5 ordinal=5 kind=variable-reference role=value range=163..164 site=68 context=1 recovery=normal spelling=\"x\" parent=-\n",
            "reference#0 term=0 binding=0 role=variable use_ordinal=1 scope=-\n",
            "reference#1 term=1 binding=0 role=variable use_ordinal=1 scope=-\n",
            "reference#2 term=4 binding=0 role=variable use_ordinal=1 scope=[0]\n",
            "reference#3 term=5 binding=0 role=variable use_ordinal=1 scope=[0]\n",
            "numeric-request#0 term=2 ordinal=0 owner=55 range=140..141 spelling=\"1\"\n",
            "numeric-request#1 term=3 ordinal=1 owner=58 range=146..147 spelling=\"2\"\n",
        )
    );
    assert_eq!(first.handoff, second.handoff);
    assert_eq!(first.handoff.debug_text(), second.handoff.debug_text());
    assert_eq!(
        (
            first.handoff.terms().len(),
            first.handoff.wrappers().len(),
            first.handoff.roots().len(),
            first.handoff.members().len(),
            first.handoff.field_updates().len(),
            first.handoff.edges().len(),
            first.handoff.requests().len(),
        ),
        (2, 0, 1, 3, 0, 3, 9)
    );
    assert!(first.handoff.application_fingerprint().is_none());
    assert_eq!(
        first
            .handoff
            .terms()
            .iter()
            .map(|(_, term)| {
                (
                    term.site().node().index(),
                    term.source_range(),
                    term.source_ordinal(),
                    term.context(),
                    term.recovery(),
                    term.spelling(),
                    term.kind(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                62,
                range(ast.source_id, 122, 150),
                0,
                proof,
                mizar_checker::source_structure::SourceStructureRecovery::Normal,
                "TypeCaseStruct ( x : 1 , y : 2 ) . x",
                mizar_checker::source_structure::SourceStructureTermKind::SelectorAccess,
            ),
            (
                61,
                range(ast.source_id, 122, 148),
                1,
                proof,
                mizar_checker::source_structure::SourceStructureRecovery::Normal,
                "TypeCaseStruct ( x : 1 , y : 2 )",
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
            ),
        ]
    );

    let root = first
        .handoff
        .roots()
        .get(mizar_checker::source_structure::SourceStructureRootId::new(0))
        .expect("B2BP root 0");
    assert_eq!(root.term().index(), 1);
    assert_eq!(root.symbol().module().package(), module.package());
    assert_eq!(
        root.symbol().module().path().as_str(),
        "parser.type_fixtures"
    );
    assert_eq!(
        root.symbol().local().as_str(),
        "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5"
    );
    assert_eq!(
        root.symbol().fqn().as_str(),
        "parser.type_fixtures::TypeCaseStruct#5"
    );
    assert_eq!(root.contribution().index(), 2);
    assert_eq!(root.origin().source_id(), ast.source_id);
    assert_eq!(root.origin().module_id(), root.symbol().module());
    assert_eq!(
        root.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 7, 27))
    );
    assert_eq!(root.origin().structural_path(), [5]);
    assert!(root.origin().import_edge().is_none());
    assert!(!root.origin().is_recovered());
    assert_eq!(root.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        root.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert!(root.signature().is_none());

    assert_eq!(
        first
            .handoff
            .members()
            .iter()
            .map(|(_, member)| {
                (
                    member.term().index(),
                    member.ordinal(),
                    member.site().node().index(),
                    member.source_range(),
                    member.spelling(),
                    member.role(),
                    member.parent(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                29,
                range(ast.source_id, 149, 150),
                "x",
                mizar_checker::source_structure::SourceStructureMemberRole::Selector,
                None,
            ),
            (
                1,
                0,
                20,
                range(ast.source_id, 137, 138),
                "x",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
                None,
            ),
            (
                1,
                1,
                24,
                range(ast.source_id, 143, 144),
                "y",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
                None,
            ),
        ]
    );
    assert_eq!(
        first
            .handoff
            .edges()
            .iter()
            .map(|(_, edge)| {
                (
                    edge.term().index(),
                    edge.ordinal(),
                    edge.role(),
                    edge.member().map(|member| member.index()),
                    edge.target(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_structure::SourceStructureEdgeRole::SelectorBase,
                None,
                mizar_checker::source_structure::SourceStructureTarget::Structure(
                    mizar_checker::source_structure::SourceStructureTermId::new(1),
                ),
            ),
            (
                1,
                0,
                mizar_checker::source_structure::SourceStructureEdgeRole::ConstructorValue,
                Some(1),
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2),
                ),
            ),
            (
                1,
                1,
                mizar_checker::source_structure::SourceStructureEdgeRole::ConstructorValue,
                Some(2),
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(3),
                ),
            ),
        ]
    );
    assert_eq!(
        first
            .handoff
            .requests()
            .iter()
            .map(|(_, request)| {
                (
                    request.term().index(),
                    request.request_ordinal(),
                    request.member().map(|member| member.index()),
                    request.kind(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                Some(0),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                0,
                1,
                Some(0),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                0,
                2,
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ResultType,
            ),
            (
                1,
                0,
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ConstructorSignature,
            ),
            (
                1,
                1,
                Some(1),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                1,
                2,
                Some(1),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                1,
                3,
                Some(2),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                1,
                4,
                Some(2),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                1,
                5,
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ResultType,
            ),
        ]
    );

    for (node, expected) in [
        (20, "source.term.structure.member.constructor-assignment"),
        (24, "source.term.structure.member.constructor-assignment"),
        (29, "source.term.structure.member.selector"),
        (54, "source.surface.unowned"),
        (55, "source.term.numeral"),
        (56, "source.surface.unowned"),
        (58, "source.term.numeral"),
        (59, "source.surface.unowned"),
        (61, "source.term.structure.constructor"),
        (62, "source.term.structure.selector"),
        (63, "source.surface.unowned"),
        (64, "source.surface.unowned"),
        (65, "source.surface.unowned"),
    ] {
        assert_eq!(
            first
                .typed_ast
                .nodes()
                .iter()
                .find(|(id, _)| id.index() == node)
                .map(|(_, row)| row)
                .expect("B2BP owned-kind site")
                .kind
                .as_str(),
            expected,
            "node {node}"
        );
    }
    assert_eq!(
        first
            .typed_ast
            .nodes()
            .iter()
            .filter(|(_, row)| row.kind.as_str().starts_with("source.term.structure."))
            .map(|(id, row)| (id.index(), row.kind.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                20,
                "source.term.structure.member.constructor-assignment"
            ),
            (
                24,
                "source.term.structure.member.constructor-assignment"
            ),
            (29, "source.term.structure.member.selector"),
            (61, "source.term.structure.constructor"),
            (62, "source.term.structure.selector"),
        ],
        "no other surface node may acquire Task254 ownership"
    );

    assert_eq!(
        first.typed_ast.source_structure(),
        first.resolved.source_structure()
    );
    assert_eq!(first.typed_ast.source_term(), first.resolved.source_term());
    assert!(first.typed_ast.source_context().is_none());
    assert!(first.typed_ast.source_type().is_none());
    assert!(first.typed_ast.source_attribute().is_none());
    assert!(first.typed_ast.source_evidence().is_none());
    assert!(first.typed_ast.source_application().is_none());
    assert!(first.typed_ast.source_set_term().is_none());
    assert!(first.typed_ast.source_atomic_formula().is_none());
    assert!(first.typed_ast.source_composite_formula().is_none());
    assert!(first.typed_ast.source_formula_composition().is_none());
    assert!(
        first
            .typed_ast
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        first
            .typed_ast
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(first.typed_ast.source_statement().is_none());
    assert!(first.typed_ast.source_statement_references().is_none());
    assert!(first.typed_ast.source_statement_witnesses().is_none());
    assert!(first.typed_ast.contexts().is_empty());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.initial_obligations().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.source_context().is_none());
    assert!(first.resolved.source_type().is_none());
    assert!(first.resolved.source_attribute().is_none());
    assert!(first.resolved.source_evidence().is_none());
    assert!(first.resolved.source_application().is_none());
    assert!(first.resolved.source_set_term().is_none());
    assert!(first.resolved.source_atomic_formula().is_none());
    assert!(first.resolved.source_composite_formula().is_none());
    assert!(first.resolved.source_formula_composition().is_none());
    assert!(
        first
            .resolved
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        first
            .resolved
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(first.resolved.source_statement().is_none());
    assert!(first.resolved.source_statement_references().is_none());
    assert!(first.resolved.source_statement_witnesses().is_none());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.collection_candidates().is_empty());
    assert!(first.resolved.expanded_candidates().is_empty());
    assert!(first.resolved.template_expansions().is_empty());
    assert!(first.resolved.viable_candidates().is_empty());
    assert!(first.resolved.viability_decisions().is_empty());
    assert!(first.resolved.specificity_graphs().is_empty());
    assert!(first.resolved.resolved_overloads().is_empty());
    assert!(first.resolved.inserted_coercions().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
}

#[test]
fn task258b3m2b2b2bp_structure_selector_corruption_replay_and_constructor_compatibility_fail_closed(
) {
    let (ast, module, _shells, base_symbols, diagnostic_count) =
        task253_ast_from_source_text_with_diagnostic_count(
            TASK258B3M2B2B2BP_SOURCE,
            23_002,
        );
    assert_eq!(diagnostic_count, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, base_symbols);
    let bindings = task258b3m2b2b1p_binding_env(&ast, &module, &symbols);
    let roots = task258b3m2b2b2bp_roots();
    let proof = mizar_checker::binding_env::BindingContextId::new(1);
    let baseline = b2bp_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2BP_SOURCE,
        &roots,
        62,
        proof,
        ImportedStructureSelectorSurfaceMutation::None,
        ImportedStructureSelectorTestMutation::None,
    )
    .expect("B2BP baseline selector")
    .expect("B2BP baseline");
    let baseline_handoff = baseline.handoff.debug_text();
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let assert_clean_replay = || {
        let replay = b2bp_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2BP_SOURCE,
            &roots,
            62,
            proof,
            ImportedStructureSelectorSurfaceMutation::None,
            ImportedStructureSelectorTestMutation::None,
        )
        .expect("B2BP clean replay selector")
        .expect("B2BP clean replay");
        assert_eq!(replay.handoff.debug_text(), baseline_handoff);
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
        assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    };

    for mutation in [
        Task258B3M2B2B2PResolverMutation::LocalAndFqn,
        Task258B3M2B2B2PResolverMutation::ModulePackage,
        Task258B3M2B2B2PResolverMutation::ModulePath,
        Task258B3M2B2B2PResolverMutation::SymbolKind,
        Task258B3M2B2B2PResolverMutation::PrimarySpelling,
        Task258B3M2B2B2PResolverMutation::StructuralPath,
        Task258B3M2B2B2PResolverMutation::OriginSource,
        Task258B3M2B2B2PResolverMutation::OriginModule,
        Task258B3M2B2B2PResolverMutation::OriginAnchor,
        Task258B3M2B2B2PResolverMutation::OriginImportEdge,
        Task258B3M2B2B2PResolverMutation::OriginRecovery,
        Task258B3M2B2B2PResolverMutation::Signature,
        Task258B3M2B2B2PResolverMutation::Visibility,
        Task258B3M2B2B2PResolverMutation::ExportStatus,
        Task258B3M2B2B2PResolverMutation::Namespace,
        Task258B3M2B2B2PResolverMutation::Contribution,
        Task258B3M2B2B2PResolverMutation::ContributionKind,
        Task258B3M2B2B2PResolverMutation::ContributionModule,
        Task258B3M2B2B2PResolverMutation::ContributionAnchor,
    ] {
        let substituted = task258b3m2b2b2p_substituted_symbol_env(&symbols, mutation);
        assert!(
            b2bp_structure_handoff!(
                &ast,
                &module,
                &substituted,
                &bindings,
                TASK258B3M2B2B2BP_SOURCE,
                &roots,
                62,
                proof,
                ImportedStructureSelectorSurfaceMutation::DirectProductionSeam,
                ImportedStructureSelectorTestMutation::None,
            )
            .is_none(),
            "same-source resolver substitution {mutation:?} selected"
        );
    }
    assert_clean_replay();

    for node in 0..79 {
        for mutation in [
            ImportedStructureSelectorSurfaceMutation::NodeKind(node),
            ImportedStructureSelectorSurfaceMutation::NodeRange(node),
            ImportedStructureSelectorSurfaceMutation::NodeRecovery(node),
            ImportedStructureSelectorSurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                b2bp_structure_handoff!(
                    &ast,
                    &module,
                    &symbols,
                    &bindings,
                    TASK258B3M2B2B2BP_SOURCE,
                    &roots,
                    62,
                    proof,
                    mutation,
                    ImportedStructureSelectorTestMutation::None,
                )
                .is_none(),
                "surface mutation {mutation:?} selected"
            );
        }
    }
    assert!(
        b2bp_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2BP_SOURCE,
            &roots,
            62,
            proof,
            ImportedStructureSelectorSurfaceMutation::RootIdentity,
            ImportedStructureSelectorTestMutation::None,
        )
        .is_none()
    );
    assert_clean_replay();
    for selector in [61, 63, usize::MAX] {
        assert!(
            b2bp_structure_handoff!(
                &ast,
                &module,
                &symbols,
                &bindings,
                TASK258B3M2B2B2BP_SOURCE,
                &roots,
                selector,
                proof,
                ImportedStructureSelectorSurfaceMutation::DirectProductionSeam,
                ImportedStructureSelectorTestMutation::None,
            )
            .is_none(),
            "production selector admitted site {selector}"
        );
    }

    for byte_index in 0..TASK258B3M2B2B2BP_SOURCE.len() {
        let mut bytes = TASK258B3M2B2B2BP_SOURCE.as_bytes().to_vec();
        bytes[byte_index] = if bytes[byte_index] == b'!' { b'?' } else { b'!' };
        let loaded_source = String::from_utf8(bytes).expect("ASCII B2BP mutation");
        assert!(
            b2bp_structure_handoff!(
                &ast,
                &module,
                &symbols,
                &bindings,
                &loaded_source,
                &roots,
                62,
                proof,
                ImportedStructureSelectorSurfaceMutation::None,
                ImportedStructureSelectorTestMutation::None,
            )
            .is_none(),
            "byte mutation {byte_index} selected"
        );
    }
    for loaded_source in [
        TASK258B3M2B2B2BP_SOURCE
            .trim_end_matches('\n')
            .to_owned(),
        format!("{TASK258B3M2B2B2BP_SOURCE}\n"),
    ] {
        assert!(
            b2bp_structure_handoff!(
                &ast,
                &module,
                &symbols,
                &bindings,
                &loaded_source,
                &roots,
                62,
                proof,
                ImportedStructureSelectorSurfaceMutation::None,
                ImportedStructureSelectorTestMutation::None,
            )
            .is_none()
        );
    }

    let missing_name =
        TASK258B3M2B2B2BP_SOURCE.replacen("TypeCaseStruct(x: 1, y: 2).x;", "TypeCaseStruct(x: 1, y: 2).;", 1);
    assert_eq!(missing_name.len(), 170);
    let (missing_ast, missing_module, missing_shells, missing_base_symbols, missing_diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(&missing_name, 23_100);
    assert_eq!(
        (
            missing_diagnostics,
            missing_ast.nodes().len(),
            missing_ast.root().map(|root| root.index()),
        ),
        (1, 78, Some(77))
    );
    assert_eq!(
        task258b3m2b2b2bp_frontend_diagnostic_profile(&missing_name, 23_101),
        [("malformed_term_expression".to_owned(), 149, 150)]
    );
    assert!(
        missing_ast.nodes().iter().all(|node| !node.recovered),
        "missing selector name must not fabricate recovered ownership"
    );
    let missing_symbols = augment_type_elaboration_import_summaries(
        &missing_ast,
        &missing_module,
        missing_base_symbols,
    );
    assert!(
        b2bp_structure_handoff!(
            &missing_ast,
            &missing_module,
            &missing_symbols,
            &bindings,
            TASK258B3M2B2B2BP_SOURCE,
            &roots,
            62,
            proof,
            ImportedStructureSelectorSurfaceMutation::DirectProductionSeam,
            ImportedStructureSelectorTestMutation::None,
        )
        .is_none()
    );
    assert!(
        source_structure_output(
            &missing_ast,
            missing_module,
            &missing_shells,
            &missing_symbols,
        )
        .is_none()
    );

    for (ordinal, source) in [
        TASK258B3M2B2B2BP_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2).x;",
            "TypeCaseStruct(x: 1, y: 2).y;",
            1,
        ),
        TASK258B3M2B2B2BP_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2).x;",
            "TypeCaseStruct(x: 1, y: 2).x();",
            1,
        ),
        TASK258B3M2B2B2BP_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2).x;",
            "TypeCaseStruct(x: 1, y: 2).x(1);",
            1,
        ),
        TASK258B3M2B2B2BP_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2).x;",
            "TypeCaseStruct(x: 1, y: 2).x.x;",
            1,
        ),
        TASK258B3M2B2B2BP_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2).x;",
            "(TypeCaseStruct(x: 1, y: 2)).x;",
            1,
        ),
        TASK258B3M2B2B2BP_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2).x;",
            "TypeCaseStruct(x: 1, y: 2);",
            1,
        ),
        TASK258B3M2B2B2BP_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2).x;",
            "TypeCaseStruct(x: 1, y: 2) with (x := 3);",
            1,
        ),
        TASK258B3M2B2B2BP_SOURCE.replacen(
            "FormulaStatementStructureSelectorWitnessSmoke",
            "FormulaStatementStructureSelectorWitnessNearMiss",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, near_shells, near_base_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 23_200 + ordinal);
        assert_eq!(diagnostics, 0, "valid excluded selector form {ordinal}");
        let near_symbols =
            augment_type_elaboration_import_summaries(&near_ast, &near_module, near_base_symbols);
        assert!(
            b2bp_structure_handoff!(
                &near_ast,
                &near_module,
                &near_symbols,
                &bindings,
                TASK258B3M2B2B2BP_SOURCE,
                &roots,
                62,
                proof,
                ImportedStructureSelectorSurfaceMutation::DirectProductionSeam,
                ImportedStructureSelectorTestMutation::None,
            )
            .is_none(),
            "valid excluded selector form {ordinal} selected"
        );
        assert!(
            source_structure_output(&near_ast, near_module, &near_shells, &near_symbols).is_none(),
            "valid excluded selector form {ordinal} activated the legacy route"
        );
    }

    let invalid_roots = [(56, proof), (56, proof)];
    let task252_error = b2bp_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2BP_SOURCE,
        &invalid_roots,
        62,
        proof,
        ImportedStructureSelectorSurfaceMutation::None,
        ImportedStructureSelectorTestMutation::TermRange(0),
    )
    .expect("Task252 precedence selector")
    .expect_err("Task252 corruption must fail");
    assert!(task252_error.starts_with("Task252:"), "{task252_error}");
    assert!(
        b2bp_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2BP_SOURCE,
            &invalid_roots,
            61,
            proof,
            ImportedStructureSelectorSurfaceMutation::None,
            ImportedStructureSelectorTestMutation::TermRange(0),
        )
        .is_none(),
        "selector rejection must precede Task252 and Task254"
    );

    let context_error = b2bp_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2BP_SOURCE,
        &roots,
        62,
        mizar_checker::binding_env::BindingContextId::new(0),
        ImportedStructureSelectorSurfaceMutation::None,
        ImportedStructureSelectorTestMutation::None,
    )
    .expect("context selector")
    .expect_err("selector and nested constructor contexts must agree");
    assert!(context_error.starts_with("Task254:"), "{context_error}");

    let incomplete_roots = [
        (47, mizar_checker::binding_env::BindingContextId::new(0)),
        (49, mizar_checker::binding_env::BindingContextId::new(0)),
        (56, proof),
        (66, proof),
        (68, proof),
    ];
    let incomplete_error = b2bp_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2BP_SOURCE,
        &incomplete_roots,
        62,
        proof,
        ImportedStructureSelectorSurfaceMutation::None,
        ImportedStructureSelectorTestMutation::None,
    )
    .expect("incomplete Task252 selector")
    .expect_err("incomplete but internally valid Task252 profile must fail");
    assert!(incomplete_error.starts_with("Task254:"), "{incomplete_error}");

    let module_context = mizar_checker::binding_env::BindingContextId::new(0);
    let jointly_substituted_roots = [
        (47, module_context),
        (49, module_context),
        (56, module_context),
        (59, module_context),
        (66, module_context),
        (68, module_context),
    ];
    let jointly_substituted_error = b2bp_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2BP_SOURCE,
        &jointly_substituted_roots,
        62,
        module_context,
        ImportedStructureSelectorSurfaceMutation::None,
        ImportedStructureSelectorTestMutation::None,
    )
    .expect("joint context substitution selector")
    .expect_err("joint Task252/254 context substitution must fail");
    assert!(
        jointly_substituted_error.starts_with("Task254:"),
        "{jointly_substituted_error}"
    );

    for mutation in [
        ImportedStructureSelectorTestMutation::BindingSourceId,
        ImportedStructureSelectorTestMutation::BindingModuleId,
        ImportedStructureSelectorTestMutation::BindingContextCount,
        ImportedStructureSelectorTestMutation::BindingCount,
        ImportedStructureSelectorTestMutation::BindingDiagnosticCount,
        ImportedStructureSelectorTestMutation::ModuleContextOwner,
        ImportedStructureSelectorTestMutation::ModuleContextParent,
        ImportedStructureSelectorTestMutation::ModuleContextLayer,
        ImportedStructureSelectorTestMutation::ModuleContextScope,
        ImportedStructureSelectorTestMutation::ModuleContextBindings,
        ImportedStructureSelectorTestMutation::ModuleContextVisibleBindings,
        ImportedStructureSelectorTestMutation::ModuleContextRecovery,
        ImportedStructureSelectorTestMutation::ProofContextOwner,
        ImportedStructureSelectorTestMutation::ProofContextParent,
        ImportedStructureSelectorTestMutation::ProofContextLayer,
        ImportedStructureSelectorTestMutation::ProofContextScope,
        ImportedStructureSelectorTestMutation::ProofContextBindings,
        ImportedStructureSelectorTestMutation::ProofContextVisibleBindings,
        ImportedStructureSelectorTestMutation::ProofContextRecovery,
        ImportedStructureSelectorTestMutation::BindingSpelling,
        ImportedStructureSelectorTestMutation::BindingKind,
        ImportedStructureSelectorTestMutation::BindingIdentityKind,
        ImportedStructureSelectorTestMutation::BindingIdentitySpelling,
        ImportedStructureSelectorTestMutation::BindingIdentityRange,
        ImportedStructureSelectorTestMutation::BindingOwner,
        ImportedStructureSelectorTestMutation::BindingDeclarationRange,
        ImportedStructureSelectorTestMutation::BindingVisibleOrdinal,
        ImportedStructureSelectorTestMutation::BindingTypeSite,
        ImportedStructureSelectorTestMutation::BindingStatus,
        ImportedStructureSelectorTestMutation::BindingCaptured,
        ImportedStructureSelectorTestMutation::BindingDiagnostics,
        ImportedStructureSelectorTestMutation::BindingRecovery,
    ] {
        let result = b2bp_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2BP_SOURCE,
            &roots,
            62,
            proof,
            ImportedStructureSelectorSurfaceMutation::None,
            mutation,
        )
        .expect("Task48 mutation selector");
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("{mutation:?} must fail closed"),
        };
        assert!(
            error.starts_with("Task48:") || error.starts_with("Task254:"),
            "{mutation:?}: {error}"
        );
        assert_clean_replay();
    }

    let mut primary_mutations = vec![
        ImportedStructureSelectorTestMutation::PrimarySourceId,
        ImportedStructureSelectorTestMutation::PrimaryModuleId,
    ];
    for index in 0..6 {
        primary_mutations.extend([
            ImportedStructureSelectorTestMutation::PrimaryTermSite(index),
            ImportedStructureSelectorTestMutation::PrimaryTermRange(index),
            ImportedStructureSelectorTestMutation::PrimaryTermOrdinal(index),
            ImportedStructureSelectorTestMutation::PrimaryTermContext(index),
            ImportedStructureSelectorTestMutation::PrimaryTermRecovery(index),
            ImportedStructureSelectorTestMutation::PrimaryTermSpelling(index),
            ImportedStructureSelectorTestMutation::PrimaryTermKind(index),
            ImportedStructureSelectorTestMutation::PrimaryTermRole(index),
            ImportedStructureSelectorTestMutation::PrimaryTermParent(index),
        ]);
    }
    for index in 0..4 {
        primary_mutations.extend([
            ImportedStructureSelectorTestMutation::PrimaryReferenceTerm(index),
            ImportedStructureSelectorTestMutation::PrimaryReferenceBinding(index),
            ImportedStructureSelectorTestMutation::PrimaryReferenceRole(index),
            ImportedStructureSelectorTestMutation::PrimaryReferenceUseOrdinal(index),
            ImportedStructureSelectorTestMutation::PrimaryReferenceScope(index),
        ]);
    }
    for index in 0..2 {
        primary_mutations.extend([
            ImportedStructureSelectorTestMutation::NumericRequestTerm(index),
            ImportedStructureSelectorTestMutation::NumericRequestOwner(index),
            ImportedStructureSelectorTestMutation::NumericRequestRange(index),
            ImportedStructureSelectorTestMutation::NumericRequestSpelling(index),
            ImportedStructureSelectorTestMutation::NumericRequestOrdinal(index),
        ]);
    }
    for mutation in primary_mutations {
        let result = b2bp_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2BP_SOURCE,
            &roots,
            62,
            proof,
            ImportedStructureSelectorSurfaceMutation::None,
            mutation,
        )
        .expect("Task252 mutation selector");
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("{mutation:?} must fail closed"),
        };
        assert!(
            error.starts_with("Task252:") || error.starts_with("Task254:"),
            "{mutation:?}: {error}"
        );
        assert_clean_replay();
    }

    let mut mutations = vec![
        ImportedStructureSelectorTestMutation::RootTerm,
        ImportedStructureSelectorTestMutation::RootSymbol,
        ImportedStructureSelectorTestMutation::RootContribution,
        ImportedStructureSelectorTestMutation::FieldUpdateExtra,
        ImportedStructureSelectorTestMutation::TermRangeAndStalePrimaryReplay,
    ];
    for index in 0..2 {
        mutations.extend([
            ImportedStructureSelectorTestMutation::TermSite(index),
            ImportedStructureSelectorTestMutation::TermRange(index),
            ImportedStructureSelectorTestMutation::TermOrdinal(index),
            ImportedStructureSelectorTestMutation::TermContext(index),
            ImportedStructureSelectorTestMutation::TermRecovery(index),
            ImportedStructureSelectorTestMutation::TermSpelling(index),
            ImportedStructureSelectorTestMutation::TermKind(index),
        ]);
    }
    for index in 0..3 {
        mutations.extend([
            ImportedStructureSelectorTestMutation::MemberTerm(index),
            ImportedStructureSelectorTestMutation::MemberOrdinal(index),
            ImportedStructureSelectorTestMutation::MemberSite(index),
            ImportedStructureSelectorTestMutation::MemberRange(index),
            ImportedStructureSelectorTestMutation::MemberSpelling(index),
            ImportedStructureSelectorTestMutation::MemberRole(index),
            ImportedStructureSelectorTestMutation::MemberParent(index),
            ImportedStructureSelectorTestMutation::EdgeTerm(index),
            ImportedStructureSelectorTestMutation::EdgeOrdinal(index),
            ImportedStructureSelectorTestMutation::EdgeRole(index),
            ImportedStructureSelectorTestMutation::EdgeMember(index),
            ImportedStructureSelectorTestMutation::EdgeTarget(index),
        ]);
    }
    for index in 0..9 {
        mutations.extend([
            ImportedStructureSelectorTestMutation::RequestTerm(index),
            ImportedStructureSelectorTestMutation::RequestOrdinal(index),
            ImportedStructureSelectorTestMutation::RequestMember(index),
            ImportedStructureSelectorTestMutation::RequestKind(index),
        ]);
    }
    for mutation in mutations {
        let error = b2bp_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2BP_SOURCE,
            &roots,
            62,
            proof,
            ImportedStructureSelectorSurfaceMutation::None,
            mutation,
        )
        .expect("Task254 mutation selector")
        .expect_err("Task254 mutation must fail");
        assert!(error.starts_with("Task254:"), "{mutation:?}: {error}");
        let replay = b2bp_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2BP_SOURCE,
            &roots,
            62,
            proof,
            ImportedStructureSelectorSurfaceMutation::None,
            ImportedStructureSelectorTestMutation::None,
        )
        .expect("clean replay selector")
        .expect("clean replay");
        assert_eq!(replay.handoff.debug_text(), baseline_handoff);
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
        assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    }

    let stale_error = b2bp_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2BP_SOURCE,
        &roots,
        62,
        proof,
        ImportedStructureSelectorSurfaceMutation::None,
        ImportedStructureSelectorTestMutation::StalePrimaryReplay,
    )
    .expect("stale replay selector")
    .expect_err("stale replay must fail");
    assert!(
        stale_error.starts_with("TypedAst: rejected stale Task252 fingerprint:"),
        "{stale_error}"
    );
    let stale_clean_replay = b2bp_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2BP_SOURCE,
        &roots,
        62,
        proof,
        ImportedStructureSelectorSurfaceMutation::None,
        ImportedStructureSelectorTestMutation::None,
    )
    .expect("stale clean replay selector")
    .expect("stale clean replay");
    assert_eq!(stale_clean_replay.handoff.debug_text(), baseline_handoff);
    assert_eq!(stale_clean_replay.typed_ast.debug_text(), baseline_typed);
    assert_eq!(stale_clean_replay.resolved.debug_text(), baseline_resolved);

    let (
        constructor_ast,
        constructor_module,
        _constructor_shells,
        constructor_base_symbols,
        constructor_diagnostics,
    ) = task253_ast_from_source_text_with_diagnostic_count(
        TASK258B3M2B2B2P_SOURCE,
        23_003,
    );
    assert_eq!(constructor_diagnostics, 0);
    let constructor_symbols = augment_type_elaboration_import_summaries(
        &constructor_ast,
        &constructor_module,
        constructor_base_symbols,
    );
    let constructor_bindings = task258b3m2b2b1p_binding_env(
        &constructor_ast,
        &constructor_module,
        &constructor_symbols,
    );
    let constructor = b2p_structure_handoff!(
        &constructor_ast,
        &constructor_module,
        &constructor_symbols,
        &constructor_bindings,
        TASK258B3M2B2B2P_SOURCE,
        &task258b3m2b2b2p_roots(),
        59,
        proof,
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::None,
    )
    .expect("B2P direct compatibility selector")
    .expect("B2P direct compatibility handoff");
    let constructor_hashes = [
        sha256_text(&constructor.handoff.debug_text()),
        sha256_text(&constructor.typed_ast.debug_text()),
        sha256_text(&constructor.resolved.debug_text()),
    ];
    assert_eq!(
        constructor_hashes,
        [
            "fecf695b295f5a866e061f15e97c9d78ecd987436f595271310b10b2aa3bcda6",
            "29de2a0c245141bd92ee5acf57e9fcab7cdfb821b545c1174ca0bf84897b5c41",
            "1a847a93e601c0107867cc0b3988db13a7164c6928ad94f052029b9a2cee3e34",
        ]
    );

    let (legacy_ast, legacy_module, legacy_shells, legacy_symbols) = task254_real_ast();
    let legacy = source_structure_output(
        &legacy_ast,
        legacy_module,
        &legacy_shells,
        &legacy_symbols,
    )
    .expect("legacy Task254 compatibility selector")
    .expect("legacy Task254 compatibility output");
    assert_eq!(
        sha256_text(
            &legacy
                .typed_ast
                .source_structure()
                .expect("legacy Task254 handoff")
                .debug_text()
        ),
        "0d6af57b89e6156d8e5de6831568c81ec110880bebf1e4aeb4ab00563f4da6c8"
    );
    assert_eq!(
        sha256_text(&legacy.typed_ast.debug_text()),
        "8264d1574faf67e19b6b84d6e11fa7ab6435335238b398fa0966bbfbc63d0599"
    );
    assert_eq!(
        sha256_text(&legacy.resolved.debug_text()),
        "118a998bc5edb770c7818be1d74cbece0f566353bf9d3e6aabb817d994a3db40"
    );
}

#[test]
fn task258b3m2b2b2p_structure_constructor_proof_context_reuse_is_exact() {
    assert_eq!(TASK258B3M2B2B2P_SOURCE.len(), 172);
    assert_eq!(
        sha256_text(TASK258B3M2B2B2P_SOURCE),
        "24e2ee2332ead5c0d46025df6044450eeab3ebb5733ebe83587ceae3ba129eb6"
    );
    let (ast, module, shells, base_symbols, diagnostic_count) =
        task253_ast_from_source_text_with_diagnostic_count(
            TASK258B3M2B2B2P_SOURCE,
            22_001,
        );
    assert_eq!(diagnostic_count, 0);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|id| id.index())),
        (76, Some(75))
    );
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, base_symbols);
    let bindings = task258b3m2b2b1p_binding_env(&ast, &module, &symbols);
    assert_eq!(
        (
            bindings.contexts().len(),
            bindings.bindings().len(),
            bindings.diagnostics().len(),
        ),
        (2, 1, 0)
    );
    assert_eq!(
        bindings.debug_text(),
        concat!(
            "binding-env-debug-v1\n",
            "module: mizar-test-task253-corruption::tests.task253_local_corruption_22001\n",
            "contexts:\n",
            "  context#0 owner=module parent=none layer=module scope=none bindings=[binding#0] visible=[binding#0] recovery=normal\n",
            "  context#1 owner=source-statement(112..170) parent=context#0 layer=proof scope=[0] bindings=[] visible=[binding#0] recovery=normal\n",
            "bindings:\n",
            "  binding#0 spelling=\"x\" kind=reserved_variable owner=context#0 identity=reserved_variable(spelling=\"x\", range=37..38) range=37..38 visible_after=0 type=source(43..46) status=reserved captured=[] diagnostics=[] recovery=normal\n",
            "diagnostics:\n",
        )
    );
    assert!(
        source_structure_output(&ast, module.clone(), &shells, &symbols).is_none(),
        "B2P exact source must not activate the legacy Task254 route"
    );
    let roots = task258b3m2b2b2p_roots();
    let proof = mizar_checker::binding_env::BindingContextId::new(1);
    let first = b2p_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2P_SOURCE,
        &roots,
        59,
        proof,
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::None,
    )
    .expect("B2P exact selector")
    .expect("B2P exact handoff");
    let second = b2p_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2P_SOURCE,
        &roots,
        59,
        proof,
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::None,
    )
    .expect("B2P replay selector")
    .expect("B2P replay handoff");
    assert_eq!(first.primary_counts, (6, 4, 2));
    assert_eq!(
        first
            .typed_ast
            .source_term()
            .expect("B2P exact Task252 handoff")
            .debug_text(),
        concat!(
            "source-primary-term-debug-v1\n",
            "module: tests.task253_local_corruption_22001\n",
            "term#0 ordinal=0 kind=variable-reference role=value range=106..107 site=45 context=0 recovery=normal spelling=\"x\" parent=-\n",
            "term#1 ordinal=1 kind=variable-reference role=value range=110..111 site=47 context=0 recovery=normal spelling=\"x\" parent=-\n",
            "term#2 ordinal=2 kind=numeral role=value range=143..144 site=53 context=1 recovery=normal spelling=\"1\" parent=-\n",
            "term#3 ordinal=3 kind=numeral role=value range=149..150 site=56 context=1 recovery=normal spelling=\"2\" parent=-\n",
            "term#4 ordinal=4 kind=variable-reference role=value range=160..161 site=63 context=1 recovery=normal spelling=\"x\" parent=-\n",
            "term#5 ordinal=5 kind=variable-reference role=value range=164..165 site=65 context=1 recovery=normal spelling=\"x\" parent=-\n",
            "reference#0 term=0 binding=0 role=variable use_ordinal=1 scope=-\n",
            "reference#1 term=1 binding=0 role=variable use_ordinal=1 scope=-\n",
            "reference#2 term=4 binding=0 role=variable use_ordinal=1 scope=[0]\n",
            "reference#3 term=5 binding=0 role=variable use_ordinal=1 scope=[0]\n",
            "numeric-request#0 term=2 ordinal=0 owner=53 range=143..144 spelling=\"1\"\n",
            "numeric-request#1 term=3 ordinal=1 owner=56 range=149..150 spelling=\"2\"\n",
        )
    );
    assert_eq!(first.handoff, second.handoff);
    assert_eq!(first.handoff.debug_text(), second.handoff.debug_text());
    assert_eq!(
        (
            first.handoff.terms().len(),
            first.handoff.wrappers().len(),
            first.handoff.roots().len(),
            first.handoff.members().len(),
            first.handoff.field_updates().len(),
            first.handoff.edges().len(),
            first.handoff.requests().len(),
        ),
        (1, 0, 1, 2, 0, 2, 6)
    );
    assert!(first.handoff.application_fingerprint().is_none());

    let term = first
        .handoff
        .terms()
        .get(mizar_checker::source_structure::SourceStructureTermId::new(0))
        .expect("B2P term 0");
    assert_eq!(term.site().node().index(), 59);
    assert_eq!(term.source_range(), range(ast.source_id, 125, 151));
    assert_eq!(term.source_ordinal(), 0);
    assert_eq!(term.context(), proof);
    assert_eq!(
        term.recovery(),
        mizar_checker::source_structure::SourceStructureRecovery::Normal
    );
    assert_eq!(term.spelling(), "TypeCaseStruct ( x : 1 , y : 2 )");
    assert_eq!(
        term.kind(),
        mizar_checker::source_structure::SourceStructureTermKind::Constructor
    );

    let root = first
        .handoff
        .roots()
        .get(mizar_checker::source_structure::SourceStructureRootId::new(0))
        .expect("B2P root 0");
    assert_eq!(root.term().index(), 0);
    assert_eq!(root.symbol().module().package(), module.package());
    assert_eq!(
        root.symbol().module().path().as_str(),
        "parser.type_fixtures"
    );
    assert_eq!(
        root.symbol().local().as_str(),
        "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5"
    );
    assert_eq!(
        root.symbol().fqn().as_str(),
        "parser.type_fixtures::TypeCaseStruct#5"
    );
    assert_eq!(root.contribution().index(), 2);
    assert_eq!(root.origin().source_id(), ast.source_id);
    assert_eq!(root.origin().module_id(), root.symbol().module());
    assert_eq!(
        root.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 7, 27))
    );
    assert_eq!(root.origin().structural_path(), [5]);
    assert!(root.origin().import_edge().is_none());
    assert!(!root.origin().is_recovered());
    assert_eq!(root.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        root.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert!(root.signature().is_none());

    assert_eq!(
        first
            .handoff
            .members()
            .iter()
            .map(|(_, member)| {
                (
                    member.term().index(),
                    member.ordinal(),
                    member.site().node().index(),
                    member.source_range(),
                    member.spelling(),
                    member.role(),
                    member.parent(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                20,
                range(ast.source_id, 140, 141),
                "x",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
                None,
            ),
            (
                0,
                1,
                24,
                range(ast.source_id, 146, 147),
                "y",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
                None,
            ),
        ]
    );
    assert_eq!(
        first
            .handoff
            .edges()
            .iter()
            .map(|(_, edge)| {
                (
                    edge.term().index(),
                    edge.ordinal(),
                    edge.role(),
                    edge.member().map(|member| member.index()),
                    edge.target(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_structure::SourceStructureEdgeRole::ConstructorValue,
                Some(0),
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2),
                ),
            ),
            (
                0,
                1,
                mizar_checker::source_structure::SourceStructureEdgeRole::ConstructorValue,
                Some(1),
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(3),
                ),
            ),
        ]
    );
    assert_eq!(
        first
            .handoff
            .requests()
            .iter()
            .map(|(_, request)| {
                (
                    request.term().index(),
                    request.request_ordinal(),
                    request.member().map(|member| member.index()),
                    request.kind(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ConstructorSignature,
            ),
            (
                0,
                1,
                Some(0),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                0,
                2,
                Some(0),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                0,
                3,
                Some(1),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                0,
                4,
                Some(1),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                0,
                5,
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ResultType,
            ),
        ]
    );

    for (node, expected) in [
        (59, "source.term.structure.constructor"),
        (20, "source.term.structure.member.constructor-assignment"),
        (24, "source.term.structure.member.constructor-assignment"),
        (52, "source.surface.unowned"),
        (53, "source.term.numeral"),
        (54, "source.surface.unowned"),
        (55, "source.surface.unowned"),
        (56, "source.term.numeral"),
        (57, "source.surface.unowned"),
        (58, "source.surface.unowned"),
        (60, "source.surface.unowned"),
        (61, "source.surface.unowned"),
        (62, "source.surface.unowned"),
    ] {
        assert_eq!(
            first
                .typed_ast
                .nodes()
                .iter()
                .find(|(id, _)| id.index() == node)
                .map(|(_, row)| row)
                .expect("B2P owned-kind site")
                .kind
                .as_str(),
            expected,
            "node {node}"
        );
    }
    assert_eq!(
        first
            .typed_ast
            .nodes()
            .iter()
            .filter(|(_, row)| row.kind.as_str().starts_with("source.term.structure."))
            .map(|(id, row)| (id.index(), row.kind.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                20,
                "source.term.structure.member.constructor-assignment"
            ),
            (
                24,
                "source.term.structure.member.constructor-assignment"
            ),
            (59, "source.term.structure.constructor"),
        ],
        "no other surface node may acquire Task254 ownership"
    );

    assert_eq!(
        first.typed_ast.source_structure(),
        first.resolved.source_structure()
    );
    assert_eq!(first.typed_ast.source_term(), first.resolved.source_term());
    assert!(first.typed_ast.source_context().is_none());
    assert!(first.typed_ast.source_type().is_none());
    assert!(first.typed_ast.source_attribute().is_none());
    assert!(first.typed_ast.source_evidence().is_none());
    assert!(first.typed_ast.source_application().is_none());
    assert!(first.typed_ast.source_set_term().is_none());
    assert!(first.typed_ast.source_atomic_formula().is_none());
    assert!(first.typed_ast.source_composite_formula().is_none());
    assert!(first.typed_ast.source_formula_composition().is_none());
    assert!(
        first
            .typed_ast
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        first
            .typed_ast
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(first.typed_ast.source_statement().is_none());
    assert!(first.typed_ast.source_statement_references().is_none());
    assert!(first.typed_ast.source_statement_witnesses().is_none());
    assert!(first.typed_ast.contexts().is_empty());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.initial_obligations().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.source_context().is_none());
    assert!(first.resolved.source_type().is_none());
    assert!(first.resolved.source_attribute().is_none());
    assert!(first.resolved.source_evidence().is_none());
    assert!(first.resolved.source_application().is_none());
    assert!(first.resolved.source_set_term().is_none());
    assert!(first.resolved.source_atomic_formula().is_none());
    assert!(first.resolved.source_composite_formula().is_none());
    assert!(first.resolved.source_formula_composition().is_none());
    assert!(
        first
            .resolved
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        first
            .resolved
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(first.resolved.source_statement().is_none());
    assert!(first.resolved.source_statement_references().is_none());
    assert!(first.resolved.source_statement_witnesses().is_none());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.collection_candidates().is_empty());
    assert!(first.resolved.expanded_candidates().is_empty());
    assert!(first.resolved.template_expansions().is_empty());
    assert!(first.resolved.viable_candidates().is_empty());
    assert!(first.resolved.viability_decisions().is_empty());
    assert!(first.resolved.specificity_graphs().is_empty());
    assert!(first.resolved.resolved_overloads().is_empty());
    assert!(first.resolved.inserted_coercions().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
}

#[test]
fn task258b3m2b2b2p_structure_constructor_corruption_replay_and_legacy_output_fail_closed() {
    let (ast, module, _shells, base_symbols, diagnostic_count) =
        task253_ast_from_source_text_with_diagnostic_count(
            TASK258B3M2B2B2P_SOURCE,
            22_002,
        );
    assert_eq!(diagnostic_count, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, base_symbols);
    let bindings = task258b3m2b2b1p_binding_env(&ast, &module, &symbols);
    let roots = task258b3m2b2b2p_roots();
    let proof = mizar_checker::binding_env::BindingContextId::new(1);
    let baseline = b2p_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2P_SOURCE,
        &roots,
        59,
        proof,
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::None,
    )
    .expect("B2P baseline selector")
    .expect("B2P baseline");
    let baseline_handoff = baseline.handoff.debug_text();
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();

    for mutation in [
        Task258B3M2B2B2PResolverMutation::LocalAndFqn,
        Task258B3M2B2B2PResolverMutation::ModulePackage,
        Task258B3M2B2B2PResolverMutation::ModulePath,
        Task258B3M2B2B2PResolverMutation::SymbolKind,
        Task258B3M2B2B2PResolverMutation::PrimarySpelling,
        Task258B3M2B2B2PResolverMutation::StructuralPath,
        Task258B3M2B2B2PResolverMutation::OriginSource,
        Task258B3M2B2B2PResolverMutation::OriginModule,
        Task258B3M2B2B2PResolverMutation::OriginAnchor,
        Task258B3M2B2B2PResolverMutation::OriginImportEdge,
        Task258B3M2B2B2PResolverMutation::OriginRecovery,
        Task258B3M2B2B2PResolverMutation::Signature,
        Task258B3M2B2B2PResolverMutation::Visibility,
        Task258B3M2B2B2PResolverMutation::ExportStatus,
        Task258B3M2B2B2PResolverMutation::Namespace,
        Task258B3M2B2B2PResolverMutation::Contribution,
        Task258B3M2B2B2PResolverMutation::ContributionKind,
        Task258B3M2B2B2PResolverMutation::ContributionModule,
        Task258B3M2B2B2PResolverMutation::ContributionAnchor,
    ] {
        let substituted = task258b3m2b2b2p_substituted_symbol_env(&symbols, mutation);
        assert!(
            b2p_structure_handoff!(
                &ast,
                &module,
                &substituted,
                &bindings,
                TASK258B3M2B2B2P_SOURCE,
                &roots,
                59,
                proof,
                ImportedStructureConstructorSurfaceMutation::DirectProductionSeam,
                ImportedStructureConstructorTestMutation::None,
            )
            .is_none(),
            "same-source resolver substitution {mutation:?} selected"
        );
    }

    for node in 0..76 {
        for mutation in [
            ImportedStructureConstructorSurfaceMutation::NodeKind(node),
            ImportedStructureConstructorSurfaceMutation::NodeRange(node),
            ImportedStructureConstructorSurfaceMutation::NodeRecovery(node),
            ImportedStructureConstructorSurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                b2p_structure_handoff!(
                    &ast,
                    &module,
                    &symbols,
                    &bindings,
                    TASK258B3M2B2B2P_SOURCE,
                    &roots,
                    59,
                    proof,
                    mutation,
                    ImportedStructureConstructorTestMutation::None,
                )
                .is_none(),
                "surface mutation {mutation:?} selected"
            );
        }
    }
    assert!(
        b2p_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2P_SOURCE,
            &roots,
            59,
            proof,
            ImportedStructureConstructorSurfaceMutation::RootIdentity,
            ImportedStructureConstructorTestMutation::None,
        )
        .is_none()
    );
    for constructor in [58, 60, usize::MAX] {
        assert!(
            b2p_structure_handoff!(
                &ast,
                &module,
                &symbols,
                &bindings,
                TASK258B3M2B2B2P_SOURCE,
                &roots,
                constructor,
                proof,
                ImportedStructureConstructorSurfaceMutation::DirectProductionSeam,
                ImportedStructureConstructorTestMutation::None,
            )
            .is_none(),
            "production selector admitted constructor {constructor}"
        );
    }

    for byte_index in 0..TASK258B3M2B2B2P_SOURCE.len() {
        let mut bytes = TASK258B3M2B2B2P_SOURCE.as_bytes().to_vec();
        bytes[byte_index] = if bytes[byte_index] == b'!' { b'?' } else { b'!' };
        let loaded_source = String::from_utf8(bytes).expect("ASCII B2P mutation");
        assert!(
            b2p_structure_handoff!(
                &ast,
                &module,
                &symbols,
                &bindings,
                &loaded_source,
                &roots,
                59,
                proof,
                ImportedStructureConstructorSurfaceMutation::None,
                ImportedStructureConstructorTestMutation::None,
            )
            .is_none(),
            "byte mutation {byte_index} selected"
        );
    }
    for loaded_source in [
        TASK258B3M2B2B2P_SOURCE
            .trim_end_matches('\n')
            .to_owned(),
        format!("{TASK258B3M2B2B2P_SOURCE}\n"),
    ] {
        assert!(
            b2p_structure_handoff!(
                &ast,
                &module,
                &symbols,
                &bindings,
                &loaded_source,
                &roots,
                59,
                proof,
                ImportedStructureConstructorSurfaceMutation::None,
                ImportedStructureConstructorTestMutation::None,
            )
            .is_none()
        );
    }

    for (ordinal, source) in [
        TASK258B3M2B2B2P_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2)",
            "TypeCaseStruct(y: 2, x: 1)",
            1,
        ),
        TASK258B3M2B2B2P_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2)",
            "TypeCaseStruct(x: 1)",
            1,
        ),
        TASK258B3M2B2B2P_SOURCE.replacen(
            "take TypeCaseStruct(x: 1, y: 2);",
            "take (TypeCaseStruct(x: 1, y: 2));",
            1,
        ),
        TASK258B3M2B2B2P_SOURCE.replacen(
            "take TypeCaseStruct(x: 1, y: 2);",
            "take TypeCaseStruct(x: 1, y: 2).x;",
            1,
        ),
        TASK258B3M2B2B2P_SOURCE.replacen(
            "take TypeCaseStruct(x: 1, y: 2);",
            "take TypeCaseStruct(x: 1, y: 2) with (x := 3);",
            1,
        ),
        TASK258B3M2B2B2P_SOURCE.replacen(
            "FormulaStatementStructureConstructorWitnessSmoke",
            "FormulaStatementStructureConstructorWitnessNearMiss",
            1,
        ),
        TASK258B3M2B2B2P_SOURCE.replacen(
            "import parser.type_fixtures;",
            "import parser.other_fixtures;",
            1,
        ),
        TASK258B3M2B2B2P_SOURCE.replacen(
            "TypeCaseStruct(x: 1, y: 2)",
            "TypeCaseStruct(x: , y: 2)",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, near_shells, near_base_symbols, diagnostic_count) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 22_100 + ordinal);
        if ordinal == 7 {
            assert_eq!(
                (
                    diagnostic_count,
                    near_ast.nodes().len(),
                    near_ast.root().map(|root| root.index()),
                ),
                (1, 74, Some(73)),
                "malformed constructor value recovery profile"
            );
            assert_eq!(
                near_ast
                    .nodes()
                    .iter()
                    .enumerate()
                    .filter_map(|(index, node)| node.recovered.then_some(index))
                    .collect::<Vec<_>>(),
                [52],
                "malformed constructor value recovered node"
            );
        }
        let near_symbols =
            augment_type_elaboration_import_summaries(&near_ast, &near_module, near_base_symbols);
        assert!(
            b2p_structure_handoff!(
                &near_ast,
                &near_module,
                &near_symbols,
                &bindings,
                TASK258B3M2B2B2P_SOURCE,
                &roots,
                59,
                proof,
                ImportedStructureConstructorSurfaceMutation::DirectProductionSeam,
                ImportedStructureConstructorTestMutation::None,
            )
            .is_none(),
            "parsed near miss {ordinal} selected"
        );
        assert!(
            source_structure_output(&near_ast, near_module, &near_shells, &near_symbols).is_none(),
            "parsed near miss {ordinal} activated the legacy route"
        );
    }

    let invalid_roots = [(54, proof), (54, proof)];
    let task252_error = b2p_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2P_SOURCE,
        &invalid_roots,
        59,
        proof,
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::TermRange,
    )
    .expect("Task252 precedence selector")
    .expect_err("Task252 corruption must fail");
    assert!(task252_error.starts_with("Task252:"), "{task252_error}");
    assert!(
        b2p_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2P_SOURCE,
            &invalid_roots,
            58,
            proof,
            ImportedStructureConstructorSurfaceMutation::None,
            ImportedStructureConstructorTestMutation::TermRange,
        )
        .is_none(),
        "selector rejection must precede Task252 and Task254"
    );

    let context_error = b2p_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2P_SOURCE,
        &roots,
        59,
        mizar_checker::binding_env::BindingContextId::new(0),
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::None,
    )
    .expect("context selector")
    .expect_err("constructor and primary contexts must agree");
    assert!(context_error.starts_with("Task254:"), "{context_error}");

    let incomplete_roots = [(54, proof), (57, proof)];
    let incomplete_error = b2p_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2P_SOURCE,
        &incomplete_roots,
        59,
        proof,
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::None,
    )
    .expect("incomplete Task252 selector")
    .expect_err("incomplete but internally valid Task252 profile must fail");
    assert!(incomplete_error.starts_with("Task254:"), "{incomplete_error}");

    let module_context = mizar_checker::binding_env::BindingContextId::new(0);
    let jointly_substituted_roots = [
        (45, module_context),
        (47, module_context),
        (54, module_context),
        (57, module_context),
        (63, module_context),
        (65, module_context),
    ];
    let jointly_substituted_error = b2p_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2P_SOURCE,
        &jointly_substituted_roots,
        59,
        module_context,
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::None,
    )
    .expect("joint context substitution selector")
    .expect_err("joint Task252/254 context substitution must fail");
    assert!(
        jointly_substituted_error.starts_with("Task254:"),
        "{jointly_substituted_error}"
    );

    for mutation in [
        ImportedStructureConstructorTestMutation::TermSite,
        ImportedStructureConstructorTestMutation::TermRange,
        ImportedStructureConstructorTestMutation::TermOrdinal,
        ImportedStructureConstructorTestMutation::TermContext,
        ImportedStructureConstructorTestMutation::TermRecovery,
        ImportedStructureConstructorTestMutation::TermSpelling,
        ImportedStructureConstructorTestMutation::TermKind,
        ImportedStructureConstructorTestMutation::RootTerm,
        ImportedStructureConstructorTestMutation::RootSymbol,
        ImportedStructureConstructorTestMutation::RootContribution,
        ImportedStructureConstructorTestMutation::MemberTerm,
        ImportedStructureConstructorTestMutation::MemberOrdinal,
        ImportedStructureConstructorTestMutation::MemberSite,
        ImportedStructureConstructorTestMutation::MemberRange,
        ImportedStructureConstructorTestMutation::MemberSpelling,
        ImportedStructureConstructorTestMutation::MemberRole,
        ImportedStructureConstructorTestMutation::MemberParent,
        ImportedStructureConstructorTestMutation::FieldUpdateExtra,
        ImportedStructureConstructorTestMutation::EdgeTerm,
        ImportedStructureConstructorTestMutation::EdgeOrdinal,
        ImportedStructureConstructorTestMutation::EdgeRole,
        ImportedStructureConstructorTestMutation::EdgeMember,
        ImportedStructureConstructorTestMutation::EdgeTarget,
        ImportedStructureConstructorTestMutation::RequestTerm,
        ImportedStructureConstructorTestMutation::RequestOrdinal,
        ImportedStructureConstructorTestMutation::RequestMember,
        ImportedStructureConstructorTestMutation::RequestKind,
        ImportedStructureConstructorTestMutation::TermRangeAndStalePrimaryReplay,
    ] {
        let error = b2p_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2P_SOURCE,
            &roots,
            59,
            proof,
            ImportedStructureConstructorSurfaceMutation::None,
            mutation,
        )
        .expect("Task254 mutation selector")
        .expect_err("Task254 mutation must fail");
        assert!(error.starts_with("Task254:"), "{mutation:?}: {error}");
        let replay = b2p_structure_handoff!(
            &ast,
            &module,
            &symbols,
            &bindings,
            TASK258B3M2B2B2P_SOURCE,
            &roots,
            59,
            proof,
            ImportedStructureConstructorSurfaceMutation::None,
            ImportedStructureConstructorTestMutation::None,
        )
        .expect("clean replay selector")
        .expect("clean replay");
        assert_eq!(replay.handoff.debug_text(), baseline_handoff);
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
        assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    }

    let stale_error = b2p_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2P_SOURCE,
        &roots,
        59,
        proof,
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::StalePrimaryReplay,
    )
    .expect("stale replay selector")
    .expect_err("stale replay must fail");
    assert!(
        stale_error.starts_with("TypedAst: rejected stale Task252 fingerprint:"),
        "{stale_error}"
    );
    let stale_clean_replay = b2p_structure_handoff!(
        &ast,
        &module,
        &symbols,
        &bindings,
        TASK258B3M2B2B2P_SOURCE,
        &roots,
        59,
        proof,
        ImportedStructureConstructorSurfaceMutation::None,
        ImportedStructureConstructorTestMutation::None,
    )
    .expect("stale clean replay selector")
    .expect("stale clean replay");
    assert_eq!(stale_clean_replay.handoff.debug_text(), baseline_handoff);
    assert_eq!(stale_clean_replay.typed_ast.debug_text(), baseline_typed);
    assert_eq!(stale_clean_replay.resolved.debug_text(), baseline_resolved);

    let (legacy_ast, legacy_module, legacy_shells, legacy_symbols) = task254_real_ast();
    let legacy_first = source_structure_output(
        &legacy_ast,
        legacy_module.clone(),
        &legacy_shells,
        &legacy_symbols,
    )
    .expect("legacy Task254 selector")
    .expect("legacy Task254 output");
    let legacy_second =
        source_structure_output(&legacy_ast, legacy_module, &legacy_shells, &legacy_symbols)
            .expect("legacy Task254 replay selector")
            .expect("legacy Task254 replay");
    assert_eq!(
        legacy_first
            .typed_ast
            .source_structure()
            .expect("legacy Task254 handoff")
            .debug_text(),
        legacy_second
            .typed_ast
            .source_structure()
            .expect("legacy Task254 replay handoff")
            .debug_text()
    );
    assert_eq!(
        sha256_text(
            &legacy_first
                .typed_ast
                .source_structure()
                .expect("legacy Task254 handoff")
                .debug_text()
        ),
        "0d6af57b89e6156d8e5de6831568c81ec110880bebf1e4aeb4ab00563f4da6c8"
    );
    assert_eq!(
        sha256_text(&legacy_first.typed_ast.debug_text()),
        "8264d1574faf67e19b6b84d6e11fa7ab6435335238b398fa0966bbfbc63d0599"
    );
    assert_eq!(
        sha256_text(&legacy_first.resolved.debug_text()),
        "118a998bc5edb770c7818be1d74cbece0f566353bf9d3e6aabb817d994a3db40"
    );
    assert_eq!(
        legacy_first.typed_ast.debug_text(),
        legacy_second.typed_ast.debug_text()
    );
    assert_eq!(
        legacy_first.resolved.debug_text(),
        legacy_second.resolved.debug_text()
    );
}

#[test]
fn task254_real_route_publishes_exact_aggregate_and_preserves_final_ownership() {
    let (ast, module, shells, symbols) = task254_real_ast();
    let first = source_structure_output(&ast, module.clone(), &shells, &symbols)
        .expect("Task 254 exact selector")
        .unwrap_or_else(|error| panic!("Task 254 real route failed: {error}"));
    let second = source_structure_output(&ast, module, &shells, &symbols)
        .expect("Task 254 exact selector should be deterministic")
        .unwrap_or_else(|error| panic!("Task 254 repeated route failed: {error}"));
    assert_task254_real_oracle(&first);
    assert_task254_real_binding_contexts(&first);
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task254_real_member_container_and_term_key_corruption_fails_atomically() {
    let (ast, module, shells, symbols) = task254_real_ast();
    let corruptions: [fn(&mut mizar_checker::source_structure::SourceStructureHandoffInput); 5] = [
        |input| input.terms[0].source_range = input.terms[1].source_range,
        |input| input.members[0].site = input.members[6].site.clone(),
        |input| input.members[6].site = input.members[8].site.clone(),
        |input| input.field_updates[0].site = input.members[7].site.clone(),
        |input| input.requests.swap(0, 1),
    ];
    for corrupt in corruptions {
        let error =
            source_structure_output_with_mutation(&ast, module.clone(), &shells, &symbols, corrupt)
                .expect("corruption must not change exact selection")
                .expect_err("corrupt Task 254 transaction must fail atomically");
        assert!(
            error.contains("structure") || error.contains("field update"),
            "{error}"
        );
    }
    assert!(
        source_structure_output(&ast, module, &shells, &symbols)
            .expect("uncorrupted exact selector")
            .is_ok()
    );
}

#[test]
fn task254_exact_selector_excludes_every_other_active_type_case() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 254 isolation plan should build");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if source_structure_output(&ast, resolver.module, &resolver.shells, &symbols).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(
        selected,
        ["fail_type_elaboration_local_structure_term_gap_001"]
    );
}

#[test]
fn task254_synthetic_entry_points_keep_dependency_and_mutation_boundaries_explicit() {
    let (ast, module, _, symbols) = task254_real_ast();
    let binding_env = task254_module_binding_env(&ast, module.clone());
    let roots = task254_outer_structure_roots(&ast);
    let output = synthetic_source_structure_output(
        &ast,
        module.clone(),
        binding_env.clone(),
        &symbols,
        &roots,
        None::<SyntheticSourceStructureDependencies>,
        &BTreeSet::new(),
    )
    .expect("synthetic Task 254 route");
    assert_task254_real_oracle(&output);
    let error = synthetic_source_structure_output_with_mutation(
        &ast,
        module,
        binding_env,
        &symbols,
        &roots,
        None::<SyntheticSourceStructureDependencies>,
        &BTreeSet::new(),
        |input| {
            input.members[0].role =
                mizar_checker::source_structure::SourceStructureMemberRole::Selector
        },
    )
    .expect_err("synthetic key/role substitution must fail");
    assert!(error.contains("structure"), "{error}");
}

#[test]
fn task254_synthetic_application_fingerprint_root_and_unrelated_matrix() {
    let (ast, module, _, symbols) = task254_real_ast();
    let roots = task254_outer_structure_roots(&ast);
    let numeral_three = task254_term_expression_with_spelling(&ast, "3");
    let positive_dependencies =
        task254_bare_application_dependencies(&ast, &module, &symbols, numeral_three, Some("3"));
    let positive = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module.clone()),
        &symbols,
        &roots,
        Some(positive_dependencies.clone()),
        &BTreeSet::new(),
    )
    .expect("Task 254 should consume a Task 253 root application");
    let structure = positive.typed_ast.source_structure().expect("Task 254");
    assert!(structure.application_fingerprint().is_some());
    assert_eq!(
        positive
            .typed_ast
            .source_application()
            .expect("targeted Task 253")
            .debug_text(),
        structure
            .application_fingerprint()
            .expect("Task 254 application fingerprint")
    );
    assert_eq!(
        positive.typed_ast.source_application(),
        positive.resolved.source_application()
    );
    assert_eq!(
        positive.typed_ast.source_structure(),
        positive.resolved.source_structure()
    );
    assert_eq!(
        positive.typed_ast.source_term(),
        positive.resolved.source_term()
    );
    assert_eq!(
        structure
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Application(_)
            ))
            .count(),
        1
    );
    assert_eq!(
        positive
            .typed_ast
            .source_term()
            .expect("Task 252")
            .terms()
            .len(),
        7
    );

    let unrelated_roots = [roots[0], roots[2]];
    let unrelated = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module),
        &symbols,
        &unrelated_roots,
        Some(positive_dependencies),
        &BTreeSet::new(),
    )
    .expect("unrelated Task 253 may coexist with Task 254");
    assert_eq!(
        unrelated
            .typed_ast
            .source_structure()
            .expect("Task 254")
            .application_fingerprint(),
        None
    );
    assert!(unrelated.typed_ast.source_application().is_some());
    assert_eq!(
        unrelated.typed_ast.source_application(),
        unrelated.resolved.source_application()
    );
    assert_eq!(
        unrelated.typed_ast.source_structure(),
        unrelated.resolved.source_structure()
    );
    assert_eq!(
        unrelated.typed_ast.source_term(),
        unrelated.resolved.source_term()
    );
    assert_eq!(
        unrelated
            .typed_ast
            .source_term()
            .expect("Task 252")
            .terms()
            .len(),
        7
    );
}

#[test]
fn task254_synthetic_structure_syntax_matrix_preserves_written_shape() {
    let (real_ast, module, _, symbols) = task254_real_ast();
    let source = real_ast.source_id;
    let mut syntax = Task254SyntheticSyntax::new(source);

    let zero = syntax.constructor(Vec::new(), false);
    let repeated = {
        let one = syntax.primary("1");
        let two = syntax.primary("2");
        let three = syntax.primary("3");
        syntax.constructor(
            vec![("marker", one), ("carrier", two), ("marker", three)],
            false,
        )
    };
    let nested_value = {
        let child = syntax.constructor(Vec::new(), false);
        syntax.constructor(vec![("carrier", child)], false)
    };
    let selector_chain = {
        let base = syntax.constructor(Vec::new(), false);
        let inner = syntax.selector(base, "carrier", Vec::new());
        let primary = syntax.primary("4");
        let structure = syntax.constructor(Vec::new(), false);
        syntax.selector(inner, "marker", vec![primary, structure])
    };
    let zero_selector_call = {
        let base = syntax.constructor(Vec::new(), false);
        syntax.selector_zero_call(base, "carrier")
    };
    let update = {
        let base = syntax.constructor(Vec::new(), false);
        let primary = syntax.primary("5");
        let replacement = syntax.constructor(Vec::new(), false);
        let repeated_value = syntax.primary("6");
        syntax.update(
            base,
            vec![
                (vec!["start", "x"], primary),
                (vec!["marker"], replacement),
                (vec!["start", "x"], repeated_value),
            ],
        )
    };
    syntax.gap(4);
    let wrapped_core = syntax.constructor(Vec::new(), false);
    let inner_wrapper = syntax.wrapper(wrapped_core);
    let wrapped = syntax.wrapper(inner_wrapper);
    let wrapped_core_range = syntax.range(wrapped_core);
    let root_specs = [
        (syntax.range(zero), SurfaceNodeKind::StructureConstructor),
        (
            syntax.range(repeated),
            SurfaceNodeKind::StructureConstructor,
        ),
        (
            syntax.range(nested_value),
            SurfaceNodeKind::StructureConstructor,
        ),
        (
            syntax.range(selector_chain),
            SurfaceNodeKind::SelectorAccess,
        ),
        (
            syntax.range(zero_selector_call),
            SurfaceNodeKind::SelectorAccess,
        ),
        (syntax.range(update), SurfaceNodeKind::StructureUpdate),
        (syntax.range(wrapped), SurfaceNodeKind::ParenthesizedTerm),
    ];
    let ast = syntax.finish(vec![
        zero,
        repeated,
        nested_value,
        selector_chain,
        zero_selector_call,
        update,
        wrapped,
    ]);
    let roots =
        root_specs.map(|(range, kind)| task254_node_with_range_and_kind(&ast, range, &kind));
    let wrapped_core_index = task254_node_with_range_and_kind(
        &ast,
        wrapped_core_range,
        &SurfaceNodeKind::StructureConstructor,
    );
    let degraded = BTreeSet::from([wrapped_core_index]);
    let output = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module),
        &symbols,
        &roots,
        None,
        &degraded,
    )
    .expect("Task 254 synthetic syntax matrix");
    let handoff = output.typed_ast.source_structure().expect("Task 254");

    assert!(handoff.terms().iter().any(|(_, term)| {
        term.kind() == mizar_checker::source_structure::SourceStructureTermKind::Constructor
            && handoff.members().iter().all(|(_, member)| {
                member.term()
                    != mizar_checker::source_structure::SourceStructureTermId::new(
                        term.source_ordinal(),
                    )
            })
    }));
    assert_eq!(
        handoff
            .members()
            .iter()
            .filter(|(_, member)| member.spelling() == "marker")
            .count(),
        4
    );
    assert!(handoff.members().iter().any(|(_, member)| {
        member.spelling() == "x"
            && member.role()
                == mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment
            && member.parent().is_some()
    }));
    assert_eq!(
        handoff
            .field_updates()
            .iter()
            .filter(|(_, update)| update.spelling().starts_with("start . x :="))
            .count(),
        2
    );
    assert_eq!(
        handoff
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Structure(_)
            ))
            .count(),
        7
    );
    assert!(handoff.edges().iter().any(|(_, edge)| {
        edge.role() == mizar_checker::source_structure::SourceStructureEdgeRole::SelectorArgument
            && matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Primary(_)
            )
    }));
    assert_eq!(handoff.wrappers().len(), 2);
    assert_eq!(
        handoff
            .wrappers()
            .iter()
            .map(|(_, wrapper)| wrapper.ordinal())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(handoff.terms().iter().any(|(_, term)| {
        term.recovery() == mizar_checker::source_structure::SourceStructureRecovery::Degraded
    }));
    assert_eq!(
        output.typed_ast.source_structure(),
        output.resolved.source_structure()
    );
}

#[test]
fn task254_synthetic_task253_targets_cover_selector_argument_and_update_value() {
    let (real_ast, module, _, symbols) = task254_real_ast();
    for (spelling, expected_role, root_kind) in [
        (
            "91",
            mizar_checker::source_structure::SourceStructureEdgeRole::SelectorArgument,
            SurfaceNodeKind::SelectorAccess,
        ),
        (
            "92",
            mizar_checker::source_structure::SourceStructureEdgeRole::UpdateValue,
            SurfaceNodeKind::StructureUpdate,
        ),
    ] {
        let mut syntax = Task254SyntheticSyntax::new(real_ast.source_id);
        let base = syntax.constructor(Vec::new(), false);
        let application = syntax.primary(spelling);
        let application_range = syntax.range(application);
        let root = match root_kind {
            SurfaceNodeKind::SelectorAccess => syntax.selector(base, "carrier", vec![application]),
            SurfaceNodeKind::StructureUpdate => {
                syntax.update(base, vec![(vec!["carrier"], application)])
            }
            _ => unreachable!("Task 254 target matrix root kind"),
        };
        let root_range = syntax.range(root);
        let ast = syntax.finish(vec![root]);
        let application = task254_node_with_range_and_kind(
            &ast,
            application_range,
            &SurfaceNodeKind::TermExpression,
        );
        let root = task254_node_with_range_and_kind(&ast, root_range, &root_kind);
        let dependencies = task254_bare_application_dependencies(
            &ast,
            &module,
            &symbols,
            application,
            Some(spelling),
        );
        let output = synthetic_source_structure_output(
            &ast,
            module.clone(),
            task254_module_binding_env(&ast, module.clone()),
            &symbols,
            &[root],
            Some(dependencies),
            &BTreeSet::new(),
        )
        .expect("Task 254 Task 253 target matrix");
        let handoff = output.typed_ast.source_structure().expect("Task 254");
        assert!(handoff.application_fingerprint().is_some());
        assert!(handoff.edges().iter().any(|(_, edge)| {
            edge.role() == expected_role
                && matches!(
                    edge.target(),
                    mizar_checker::source_structure::SourceStructureTarget::Application(_)
                )
        }));
        assert_eq!(
            output.typed_ast.source_application(),
            output.resolved.source_application()
        );
        assert_eq!(
            output.typed_ast.source_structure(),
            output.resolved.source_structure()
        );
        assert_eq!(
            output.typed_ast.source_term(),
            output.resolved.source_term()
        );
    }
}

#[test]
fn task254_imported_root_provenance_matrix_reaches_source_structure_producer() {
    let (real_ast, module, _, _) = task254_real_ast();
    let mut syntax = Task254SyntheticSyntax::new(real_ast.source_id);
    let constructor = syntax.constructor_named("ImportedPair", Vec::new(), false);
    let constructor_range = syntax.range(constructor);
    let ast = syntax.finish(vec![constructor]);
    let root = task254_node_with_range_and_kind(
        &ast,
        constructor_range,
        &SurfaceNodeKind::StructureConstructor,
    );
    let valid_symbols =
        task254_imported_structure_symbols(&ast, &module, Task254ImportedRootCorruption::None);
    let valid = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module.clone()),
        &valid_symbols,
        &[root],
        None::<SyntheticSourceStructureDependencies>,
        &BTreeSet::new(),
    )
    .expect("valid imported Task 254 root");
    let imported_root = valid
        .typed_ast
        .source_structure()
        .expect("Task 254")
        .roots()
        .iter()
        .next()
        .map(|(_, root)| root)
        .expect("imported Task 254 root row");
    assert_eq!(
        imported_root.symbol().module().path().as_str(),
        "task254.imported"
    );
    assert_eq!(
        valid.typed_ast.source_structure(),
        valid.resolved.source_structure()
    );

    for corruption in [
        Task254ImportedRootCorruption::ContributionKind,
        Task254ImportedRootCorruption::ContributionSource,
        Task254ImportedRootCorruption::ContributionModule,
        Task254ImportedRootCorruption::ContributionRange,
        Task254ImportedRootCorruption::SymbolEffect,
        Task254ImportedRootCorruption::Visibility,
        Task254ImportedRootCorruption::ExportStatus,
        Task254ImportedRootCorruption::Namespace,
        Task254ImportedRootCorruption::AuthenticatedImportModule,
        Task254ImportedRootCorruption::AuthenticatedImportEffect,
    ] {
        let validation_symbols = task254_imported_structure_symbols(&ast, &module, corruption);
        let error =
            super::type_elaboration::synthetic_source_structure_output_with_validation_symbols(
                &ast,
                module.clone(),
                task254_module_binding_env(&ast, module.clone()),
                &valid_symbols,
                &validation_symbols,
                &[root],
                None::<SyntheticSourceStructureDependencies>,
                &BTreeSet::new(),
            )
            .expect_err("corrupt imported Task 254 provenance must fail in the producer");
        assert!(
            error.contains("structure") && error.contains("root"),
            "{corruption:?}: {error}"
        );
    }
}

#[test]
fn task254_synthetic_whole_subtree_exclusion_matrix_is_exact() {
    let (real_ast, module, _, symbols) = task254_real_ast();
    let mut syntax = Task254SyntheticSyntax::new(real_ast.source_id);
    let type_arguments = syntax.constructor(Vec::new(), true);
    let set_enumeration_value = syntax.opaque_term(SurfaceNodeKind::SetEnumeration, "{ 1 }");
    let set_enumeration = syntax.constructor(vec![("carrier", set_enumeration_value)], false);
    let comprehension_value = syntax.opaque_term(SurfaceNodeKind::SetComprehension, "{ x }");
    let comprehension = syntax.constructor(vec![("carrier", comprehension_value)], false);
    let choice_value = syntax.opaque_term(SurfaceNodeKind::ChoiceTerm, "the set");
    let choice = syntax.constructor(vec![("carrier", choice_value)], false);
    let qua_value = syntax.opaque_term(SurfaceNodeKind::QuaExpression, "x qua set");
    let qua = syntax.constructor(vec![("carrier", qua_value)], false);
    let template_value = syntax.opaque_term(SurfaceNodeKind::TemplateArgument, "template");
    let template_descendant = syntax.constructor(vec![("carrier", template_value)], false);
    let template_child = syntax.constructor(Vec::new(), false);
    let template_child_range = syntax.range(template_child);
    let template_ancestor = syntax.template_ancestor(template_child);
    let application_child = syntax.constructor(Vec::new(), false);
    let application_child_range = syntax.range(application_child);
    let reverse_application = syntax.application_ancestor(application_child);
    let root_specs = [
        (
            syntax.range(type_arguments),
            SurfaceNodeKind::StructureConstructor,
        ),
        (
            syntax.range(set_enumeration),
            SurfaceNodeKind::StructureConstructor,
        ),
        (
            syntax.range(comprehension),
            SurfaceNodeKind::StructureConstructor,
        ),
        (syntax.range(choice), SurfaceNodeKind::StructureConstructor),
        (syntax.range(qua), SurfaceNodeKind::StructureConstructor),
        (
            syntax.range(template_descendant),
            SurfaceNodeKind::StructureConstructor,
        ),
        (template_child_range, SurfaceNodeKind::StructureConstructor),
        (
            application_child_range,
            SurfaceNodeKind::StructureConstructor,
        ),
    ];
    let reverse_application_range = syntax.range(reverse_application);
    let ast = syntax.finish(vec![
        type_arguments,
        set_enumeration,
        comprehension,
        choice,
        qua,
        template_descendant,
        template_ancestor,
        reverse_application,
    ]);
    let roots =
        root_specs.map(|(range, kind)| task254_node_with_range_and_kind(&ast, range, &kind));
    let application_site = task254_node_with_range_and_kind(
        &ast,
        reverse_application_range,
        &SurfaceNodeKind::ApplicationTerm,
    );
    let dependencies =
        task254_bare_application_dependencies(&ast, &module, &symbols, application_site, None);
    let output = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module),
        &symbols,
        &roots,
        Some(dependencies),
        &BTreeSet::new(),
    )
    .expect("Task 254 excluded subtree matrix");
    let handoff = output.typed_ast.source_structure().expect("Task 254");
    assert!(handoff.terms().is_empty());
    assert!(handoff.members().is_empty());
    assert!(handoff.field_updates().is_empty());
    assert!(handoff.edges().is_empty());
    assert_eq!(handoff.application_fingerprint(), None);
    assert!(output.typed_ast.source_application().is_some());
    assert_eq!(
        output.typed_ast.source_structure(),
        output.resolved.source_structure()
    );
}

fn assert_task254_real_oracle(output: &SourceStructureRouteOutput) {
    let handoff = output
        .typed_ast
        .source_structure()
        .expect("Task 254 handoff");
    let primary = output.typed_ast.source_term().expect("Task 252 handoff");
    assert_eq!(
        (
            handoff.terms().len(),
            handoff.wrappers().len(),
            handoff.roots().len(),
            handoff.members().len(),
            handoff.field_updates().len(),
            handoff.edges().len(),
            handoff.requests().len(),
        ),
        (5, 0, 3, 9, 2, 10, 26)
    );
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (8, 0, 8)
    );
    assert_eq!(handoff.application_fingerprint(), None);
    assert_eq!(
        handoff.primary_term_fingerprint(),
        primary.debug_text().as_str()
    );
    assert_eq!(
        handoff
            .terms()
            .iter()
            .map(|(id, term)| (id.index(), term.source_ordinal(), term.kind()))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
            ),
            (
                1,
                1,
                mizar_checker::source_structure::SourceStructureTermKind::SelectorAccess,
            ),
            (
                2,
                2,
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
            ),
            (
                3,
                3,
                mizar_checker::source_structure::SourceStructureTermKind::FunctionalUpdate,
            ),
            (
                4,
                4,
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
            ),
        ]
    );
    assert_eq!(
        handoff
            .members()
            .iter()
            .map(|(_, member)| (member.spelling(), member.role()))
            .collect::<Vec<_>>(),
        [
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "marker",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::Selector,
            ),
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "marker",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment,
            ),
            (
                "marker",
                mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment,
            ),
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "marker",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
        ]
    );
    for (_, member) in handoff.members().iter() {
        let node = output
            .typed_ast
            .nodes()
            .node(member.site().node())
            .expect("member arena node");
        let expected = match member.role() {
            mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment => {
                "source.term.structure.member.constructor-assignment"
            }
            mizar_checker::source_structure::SourceStructureMemberRole::Selector => {
                "source.term.structure.member.selector"
            }
            mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment => {
                "source.term.structure.member.update-path-segment"
            }
            role => panic!("unexpected member role: {role:?}"),
        };
        assert_eq!(node.kind.as_str(), expected);
    }
    for (_, update) in handoff.field_updates().iter() {
        assert_eq!(
            output
                .typed_ast
                .nodes()
                .node(update.site().node())
                .expect("FieldUpdate arena node")
                .kind
                .as_str(),
            "source.term.structure.field-update"
        );
    }
    assert_eq!(
        handoff
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Primary(_)
            ))
            .count(),
        8
    );
    assert_eq!(
        handoff
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Structure(_)
            ))
            .count(),
        2
    );
    assert_eq!(output.typed_ast.source_application(), None);
    assert_eq!(
        output.typed_ast.source_structure(),
        output.resolved.source_structure()
    );
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
}

fn assert_task254_real_binding_contexts(output: &SourceStructureRouteOutput) {
    let handoff = output
        .typed_ast
        .source_structure()
        .expect("Task 254 handoff");
    let primary = output.typed_ast.source_term().expect("Task 252 handoff");
    assert_eq!(output.binding_env.contexts().len(), 2);
    assert_eq!(output.binding_env.bindings().len(), 2);
    assert!(matches!(
        output
            .binding_env
            .contexts()
            .get(mizar_checker::binding_env::BindingContextId::new(0))
            .expect("Task 254 module context")
            .owner,
        mizar_checker::binding_env::BindingContextOwner::Module
    ));
    assert!(matches!(
        output
            .binding_env
            .contexts()
            .get(mizar_checker::binding_env::BindingContextId::new(1))
            .expect("Task 254 definition context")
            .owner,
        mizar_checker::binding_env::BindingContextOwner::DeclarationShell(_)
    ));
    assert!(handoff.terms().iter().all(|(_, term)| term.context()
        == mizar_checker::binding_env::BindingContextId::new(1)));
    assert!(primary.terms().iter().all(|(_, term)| term.context()
        == mizar_checker::binding_env::BindingContextId::new(1)));
}

fn task254_real_ast() -> (
    SurfaceAst,
    ResolverModuleId,
    mizar_resolve::declarations::DeclarationShellSet,
    SymbolEnv,
) {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 254 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == "fail_type_elaboration_local_structure_term_gap_001")
        .expect("Task 254 case should remain active");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .unwrap_or_else(|error| panic!("Task 254 frontend failed: {error}"));
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.expect("Task 254 AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let module = resolver.module;
    let shells = resolver.shells;
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
    (ast, module, shells, symbols)
}

fn task254_outer_structure_roots(ast: &SurfaceAst) -> Vec<usize> {
    let mut parents = vec![None; ast.nodes().len()];
    for (parent, node) in ast.nodes().iter().enumerate() {
        for child in &node.children {
            parents[child.index()] = Some(parent);
        }
    }
    ast.nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            matches!(
                node.kind,
                SurfaceNodeKind::StructureConstructor
                    | SurfaceNodeKind::SelectorAccess
                    | SurfaceNodeKind::StructureUpdate
            )
        })
        .filter(|(index, _)| {
            let mut cursor = parents[*index];
            while let Some(parent) = cursor {
                if matches!(
                    ast.nodes()[parent].kind,
                    SurfaceNodeKind::StructureConstructor
                        | SurfaceNodeKind::SelectorAccess
                        | SurfaceNodeKind::StructureUpdate
                ) {
                    return false;
                }
                cursor = parents[parent];
            }
            true
        })
        .map(|(index, _)| index)
        .collect()
}

fn task254_module_binding_env(
    ast: &SurfaceAst,
    module: ResolverModuleId,
) -> mizar_checker::binding_env::BindingEnv {
    let mut contexts = mizar_checker::binding_env::BindingContextTable::new();
    let context = contexts.insert(mizar_checker::binding_env::BindingContextDraft {
        owner: mizar_checker::binding_env::BindingContextOwner::Module,
        parent: None,
        layer: mizar_checker::binding_env::BindingContextLayer::Module,
        lexical_scope: None,
        bindings: Vec::new(),
        visible_bindings: Vec::new(),
        recovery: mizar_checker::binding_env::BindingContextRecovery::Normal,
    });
    assert_eq!(
        context,
        mizar_checker::binding_env::BindingContextId::new(0)
    );
    mizar_checker::binding_env::BindingEnv::try_new(mizar_checker::binding_env::BindingEnvParts {
        source_id: ast.source_id,
        module_id: module,
        contexts,
        bindings: mizar_checker::binding_env::BindingTable::new(),
        diagnostics: mizar_checker::binding_env::BindingDiagnosticTable::new(),
    })
    .expect("synthetic Task 254 binding env")
}

fn task254_term_expression_with_spelling(ast: &SurfaceAst, spelling: &str) -> usize {
    ast.nodes()
        .iter()
        .enumerate()
        .find(|(_, node)| {
            matches!(node.kind, SurfaceNodeKind::TermExpression)
                && task254_subtree_tokens(ast, node) == [spelling]
        })
        .map(|(index, _)| index)
        .unwrap_or_else(|| panic!("Task 254 term expression `{spelling}`"))
}

fn task254_bare_application_dependencies(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    application_node: usize,
    excluded_primary_spelling: Option<&str>,
) -> SyntheticSourceStructureDependencies {
    use mizar_checker::{
        source_application::{
            SourceFunctorApplicationForm, SourceFunctorApplicationHandoffInput,
            SourceFunctorApplicationId, SourceFunctorApplicationInput,
            SourceFunctorApplicationKind, SourceFunctorApplicationProducer,
            SourceFunctorApplicationRecovery, SourceFunctorCandidateId,
            SourceFunctorCandidateInput, SourceFunctorHeadSite, SourceFunctorTypeRequestInput,
            SourceFunctorTypeRequestKind,
        },
        source_term::{
            SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermId,
            SourcePrimaryTermInput, SourcePrimaryTermKind, SourcePrimaryTermProducer,
            SourcePrimaryTermRecovery, SourcePrimaryTermRole,
        },
        typed_ast::{
            NodeRecoveryState, TypedArena, TypedNode, TypedNodeId, TypedSiteRef, TypingState,
        },
    };
    use mizar_session::SourceAnchor;

    let application = &ast.nodes()[application_node];
    let application_children = application
        .children
        .iter()
        .filter_map(|child| {
            ast.node(*child)
                .filter(|node| !matches!(node.kind, SurfaceNodeKind::Token(_)))
                .map(|_| child.index())
        })
        .collect::<Vec<_>>();
    let [head_node] = application_children.as_slice() else {
        panic!("bare Task 253 site must have one structural head");
    };
    assert_eq!(ast.nodes()[*head_node].range, application.range);
    let application_spelling = task254_subtree_tokens(ast, application).join(" ");

    let mut numeral_nodes = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| matches!(node.kind, SurfaceNodeKind::NumeralTerm))
        .filter(|(_, node)| {
            excluded_primary_spelling
                .is_none_or(|excluded| task254_subtree_tokens(ast, node).as_slice() != [excluded])
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    numeral_nodes.sort_by_key(|index| ast.nodes()[*index].range.start);

    let mut typed_nodes = ast
        .nodes()
        .iter()
        .map(|node| {
            TypedNode::new("source.surface.unowned", SourceAnchor::Range(node.range))
                .with_children(
                    node.children
                        .iter()
                        .map(|child| TypedNodeId::new(child.index()))
                        .collect(),
                )
                .with_typing(TypingState::Unknown)
                .with_recovery(if node.recovered {
                    NodeRecoveryState::Recovered
                } else {
                    NodeRecoveryState::Normal
                })
        })
        .collect::<Vec<_>>();
    for numeral in &numeral_nodes {
        typed_nodes[*numeral].kind = "source.term.numeral".into();
    }
    typed_nodes[application_node].kind = "source.term.functor-application.symbolic".into();
    typed_nodes[*head_node].kind = "source.term.functor-head.single".into();
    let arena = TypedArena::try_new(
        ast.root().map(|root| TypedNodeId::new(root.index())),
        typed_nodes,
    )
    .expect("Task 254 synthetic dependency arena");
    let binding_env = task254_module_binding_env(ast, module.clone());
    let primary_input = SourcePrimaryTermHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms: numeral_nodes
            .iter()
            .enumerate()
            .map(|(source_ordinal, node)| {
                let source = &ast.nodes()[*node];
                SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(*node)),
                    source_range: source.range,
                    source_ordinal,
                    context: mizar_checker::binding_env::BindingContextId::new(0),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: task254_subtree_tokens(ast, source).join(" "),
                    kind: SourcePrimaryTermKind::Numeral,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }
            })
            .collect(),
        references: Vec::new(),
        numeric_type_requests: numeral_nodes
            .iter()
            .enumerate()
            .map(|(request_ordinal, node)| {
                let source = &ast.nodes()[*node];
                SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(request_ordinal),
                    owner: TypedSiteRef::Node(TypedNodeId::new(*node)),
                    source_range: source.range,
                    spelling: task254_subtree_tokens(ast, source).join(" "),
                    request_ordinal,
                }
            })
            .collect(),
    };
    let primary = SourcePrimaryTermProducer::build(primary_input, &binding_env, &arena)
        .expect("Task 254 synthetic Task 252 dependency");
    let candidate = symbols
        .symbols()
        .iter()
        .filter(|entry| entry.kind() == SymbolKind::Functor)
        .filter_map(|entry| {
            let SourceAnchor::Range(range) = entry.origin().anchor() else {
                return None;
            };
            (range.end <= application.range.start).then_some((range.start, entry))
        })
        .min_by_key(|(start, _)| *start)
        .map(|(_, entry)| entry)
        .expect("source-preceding synthetic Task 253 candidate");
    let application_id = SourceFunctorApplicationId::new(0);
    let candidate_id = SourceFunctorCandidateId::new(0);
    let application_input = SourceFunctorApplicationHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        applications: vec![SourceFunctorApplicationInput {
            site: TypedSiteRef::Node(TypedNodeId::new(application_node)),
            source_range: application.range,
            source_ordinal: 0,
            context: mizar_checker::binding_env::BindingContextId::new(0),
            recovery: SourceFunctorApplicationRecovery::Normal,
            spelling: application_spelling.clone(),
            kind: SourceFunctorApplicationKind::Symbolic,
            form: SourceFunctorApplicationForm::Bare,
            head_ordinal: 0,
            head: SourceFunctorHeadSite::Single {
                site: TypedSiteRef::Node(TypedNodeId::new(*head_node)),
                source_range: ast.nodes()[*head_node].range,
                spelling: application_spelling,
            },
        }],
        wrappers: Vec::new(),
        candidates: vec![SourceFunctorCandidateInput {
            application: application_id,
            ordinal: 0,
            symbol: candidate.symbol().clone(),
            contribution: candidate.contribution(),
        }],
        arguments: Vec::new(),
        type_requests: vec![
            SourceFunctorTypeRequestInput {
                application: application_id,
                candidate: Some(candidate_id),
                request_ordinal: 0,
                kind: SourceFunctorTypeRequestKind::CandidateSignature,
            },
            SourceFunctorTypeRequestInput {
                application: application_id,
                candidate: None,
                request_ordinal: 1,
                kind: SourceFunctorTypeRequestKind::ApplicationResultType,
            },
        ],
    };
    let application = SourceFunctorApplicationProducer::build(
        application_input,
        symbols,
        &binding_env,
        &primary,
        &arena,
    )
    .expect("Task 254 synthetic Task 253 dependency");
    SyntheticSourceStructureDependencies {
        arena,
        primary,
        application: Some(application),
    }
}

fn task254_subtree_tokens<'a>(
    ast: &'a SurfaceAst,
    node: &'a mizar_syntax::SurfaceNode,
) -> Vec<&'a str> {
    let mut tokens = Vec::new();
    fn collect<'a>(
        ast: &'a SurfaceAst,
        node: &'a mizar_syntax::SurfaceNode,
        tokens: &mut Vec<&'a str>,
    ) {
        if let Some(token) = node.token_text() {
            tokens.push(token);
            return;
        }
        for child in &node.children {
            if let Some(child) = ast.node(*child) {
                collect(ast, child, tokens);
            }
        }
    }
    collect(ast, node, &mut tokens);
    tokens
}

fn task254_node_with_range_and_kind(
    ast: &SurfaceAst,
    range: SourceRange,
    kind: &SurfaceNodeKind,
) -> usize {
    let matches = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| node.range == range && &node.kind == kind)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [index] = matches.as_slice() else {
        panic!("expected one synthetic {kind:?} at {range:?}");
    };
    *index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task254ImportedRootCorruption {
    None,
    ContributionKind,
    ContributionSource,
    ContributionModule,
    ContributionRange,
    SymbolEffect,
    Visibility,
    ExportStatus,
    Namespace,
    AuthenticatedImportModule,
    AuthenticatedImportEffect,
}

fn task254_imported_structure_symbols(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    corruption: Task254ImportedRootCorruption,
) -> SymbolEnv {
    let dependency = ResolverModuleId::new(
        module.package().clone(),
        ModulePath::new("task254.imported"),
    );
    let constructor_range = ast
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, SurfaceNodeKind::StructureConstructor))
        .expect("Task 254 imported constructor")
        .range;
    let valid_provenance_range = SourceRange {
        source_id: ast.source_id,
        start: 1,
        end: 5,
    };
    let contribution_range = if corruption == Task254ImportedRootCorruption::ContributionRange {
        SourceRange {
            source_id: ast.source_id,
            start: constructor_range.start + 1,
            end: constructor_range.end + 1,
        }
    } else {
        valid_provenance_range
    };
    let contribution_module = if corruption == Task254ImportedRootCorruption::ContributionModule {
        module.clone()
    } else {
        dependency.clone()
    };
    let contribution_source = if corruption == Task254ImportedRootCorruption::ContributionSource {
        let allocator = InMemorySessionIdAllocator::new();
        allocator
            .next_source_id(snapshot_id(254))
            .expect("Task 254 first drift source allocation");
        allocator
            .next_source_id(snapshot_id(253))
            .expect("Task 254 distinct drift source allocation")
    } else {
        ast.source_id
    };
    if corruption == Task254ImportedRootCorruption::ContributionSource {
        assert_ne!(
            contribution_source, ast.source_id,
            "Task 254 drift source must differ"
        );
    }
    let contribution_kind = if corruption == Task254ImportedRootCorruption::ContributionKind {
        ContributionKind::LocalSource {
            source_id: ast.source_id,
        }
    } else {
        ContributionKind::ImportedSource {
            source_id: contribution_source,
        }
    };

    let import_origin = SemanticOrigin::new(
        ast.source_id,
        module.clone(),
        SourceAnchor::Range(valid_provenance_range),
        vec![0],
    );
    let mut import_nodes = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let import_owner = import_nodes
        .push(mizar_resolve::resolved_ast::ResolvedNode::new(
            SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            import_origin.clone(),
        ))
        .expect("Task 254 imported owner");
    let mut resolved_imports = mizar_resolve::resolved_ast::ResolvedImports::new();
    let import = resolved_imports.push_import(mizar_resolve::resolved_ast::ResolvedImport::new(
        import_owner,
        valid_provenance_range,
        "import task254.imported;",
        None,
        mizar_resolve::resolved_ast::ImportResolution::Resolved(dependency.clone()),
        import_origin,
    ));

    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        contribution_module,
        contribution_kind,
        SourceAnchor::Range(contribution_range),
    );
    let symbol = ResolverSymbolId::new(
        dependency.clone(),
        LocalSymbolId::new("Structure/ImportedPair/0"),
        FullyQualifiedName::new("task254.imported::ImportedPair/0"),
    );
    let namespace = if corruption == Task254ImportedRootCorruption::Namespace {
        NamespacePath::new("task254.drift")
    } else {
        NamespacePath::new(module.path().as_str())
    };
    let visibility = if corruption == Task254ImportedRootCorruption::Visibility {
        Visibility::Private
    } else {
        Visibility::Public
    };
    let export_status = if corruption == Task254ImportedRootCorruption::ExportStatus {
        ExportStatus::LocalOnly
    } else {
        ExportStatus::Exported
    };
    indexes.symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Structure,
            namespace,
            "ImportedPair",
            SemanticOrigin::new(
                ast.source_id,
                dependency.clone(),
                SourceAnchor::Range(valid_provenance_range),
                vec![0],
            ),
            contribution,
        )
        .with_visibility(visibility)
        .with_export_status(export_status),
    );
    if corruption != Task254ImportedRootCorruption::SymbolEffect {
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
    }
    let authenticated_module =
        if corruption == Task254ImportedRootCorruption::AuthenticatedImportModule {
            module.clone()
        } else {
            dependency
        };
    indexes
        .imports
        .insert(mizar_resolve::env::ImportIndexEntry::new(
            import,
            Some(authenticated_module),
            None,
            contribution,
        ));
    if corruption != Task254ImportedRootCorruption::AuthenticatedImportEffect {
        indexes.contributions.add_import(contribution, import);
    }
    SymbolEnv::new(module.clone(), indexes)
}

struct Task254SyntheticSyntax {
    source: SourceId,
    next: usize,
    builder: SurfaceAstBuilder,
}

impl Task254SyntheticSyntax {
    fn new(source: SourceId) -> Self {
        Self {
            source,
            next: 1_000,
            builder: SurfaceAstBuilder::new(source),
        }
    }

    fn token(&mut self, kind: SurfaceTokenKind, spelling: &str) -> SurfaceBuilderNodeId {
        let start = self.next;
        let end = start + spelling.len().max(1);
        self.next = end + 128;
        self.builder.add_token(
            kind,
            spelling,
            SourceRange {
                source_id: self.source,
                start,
                end,
            },
        )
    }

    fn token_at(
        &mut self,
        kind: SurfaceTokenKind,
        spelling: &str,
        cursor: &mut usize,
    ) -> SurfaceBuilderNodeId {
        let start = *cursor;
        let end = start + spelling.len().max(1);
        *cursor = end + 1;
        self.next = self.next.max(end + 128);
        self.builder.add_token(
            kind,
            spelling,
            SourceRange {
                source_id: self.source,
                start,
                end,
            },
        )
    }

    fn node(
        &mut self,
        kind: SurfaceNodeKind,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let start = children
            .iter()
            .filter_map(|child| self.builder.node_range(*child))
            .map(|range| range.start)
            .min()
            .expect("synthetic children");
        let end = children
            .iter()
            .filter_map(|child| self.builder.node_range(*child))
            .map(|range| range.end)
            .max()
            .expect("synthetic children");
        self.builder.add_node(
            kind,
            SourceRange {
                source_id: self.source,
                start,
                end,
            },
            children,
        )
    }

    fn primary(&mut self, spelling: &str) -> SurfaceBuilderNodeId {
        let token = self.token(SurfaceTokenKind::Numeral, spelling);
        let numeral = self.node(SurfaceNodeKind::NumeralTerm, vec![token]);
        self.node(SurfaceNodeKind::TermExpression, vec![numeral])
    }

    fn opaque_term(&mut self, kind: SurfaceNodeKind, spelling: &str) -> SurfaceBuilderNodeId {
        let token = self.token(SurfaceTokenKind::Identifier, spelling);
        let opaque = self.node(kind, vec![token]);
        self.node(SurfaceNodeKind::TermExpression, vec![opaque])
    }

    fn range(&self, node: SurfaceBuilderNodeId) -> SourceRange {
        self.builder.node_range(node).expect("synthetic node range")
    }

    fn gap(&mut self, width: usize) {
        self.next += width;
    }

    fn qualified_structure_named(&mut self, spelling: &str) -> SurfaceBuilderNodeId {
        let token = self.token(SurfaceTokenKind::Identifier, spelling);
        let segment = self.node(SurfaceNodeKind::PathSegment, vec![token]);
        self.node(SurfaceNodeKind::QualifiedSymbol, vec![segment])
    }

    fn constructor(
        &mut self,
        fields: Vec<(&str, SurfaceBuilderNodeId)>,
        type_arguments: bool,
    ) -> SurfaceBuilderNodeId {
        self.constructor_named("Task254Pair", fields, type_arguments)
    }

    fn constructor_named(
        &mut self,
        spelling: &str,
        fields: Vec<(&str, SurfaceBuilderNodeId)>,
        type_arguments: bool,
    ) -> SurfaceBuilderNodeId {
        if fields.is_empty() {
            let root = self.qualified_structure_named(spelling);
            let mut children = vec![root];
            if type_arguments {
                let of = self.token(SurfaceTokenKind::ReservedWord, "of");
                let set = self.token(SurfaceTokenKind::Identifier, "set");
                children.push(self.node(SurfaceNodeKind::TypeArguments, vec![of, set]));
            }
            children.push(self.token(SurfaceTokenKind::ReservedSymbol, "("));
            children.push(self.token(SurfaceTokenKind::ReservedSymbol, ")"));
            return self.node(SurfaceNodeKind::StructureConstructor, children);
        }

        let first_value_start = self.range(fields[0].1).start;
        let first_label = fields[0].0;
        let prefix_width = spelling.len().max(1)
            + usize::from(type_arguments) * ("of".len() + "set".len() + 2)
            + "(".len()
            + first_label.len().max(1)
            + ":".len()
            + 6;
        let mut cursor = first_value_start
            .checked_sub(prefix_width)
            .expect("synthetic constructor prefix range");
        let root_token = self.token_at(SurfaceTokenKind::Identifier, spelling, &mut cursor);
        let root_segment = self.node(SurfaceNodeKind::PathSegment, vec![root_token]);
        let root = self.node(SurfaceNodeKind::QualifiedSymbol, vec![root_segment]);
        let mut children = vec![root];
        if type_arguments {
            let of = self.token_at(SurfaceTokenKind::ReservedWord, "of", &mut cursor);
            let set = self.token_at(SurfaceTokenKind::Identifier, "set", &mut cursor);
            children.push(self.node(SurfaceNodeKind::TypeArguments, vec![of, set]));
        }
        children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, "(", &mut cursor));
        for (ordinal, (label, value)) in fields.into_iter().enumerate() {
            let value_range = self.range(value);
            if ordinal > 0 {
                children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ",", &mut cursor));
            }
            let label = self.token_at(SurfaceTokenKind::Identifier, label, &mut cursor);
            let colon = self.token_at(SurfaceTokenKind::ReservedSymbol, ":", &mut cursor);
            assert!(
                cursor <= value_range.start,
                "synthetic constructor label must precede its value"
            );
            let field = self.node(SurfaceNodeKind::FieldArgument, vec![label, colon, value]);
            children.push(field);
            cursor = value_range.end + 1;
        }
        children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ")", &mut cursor));
        self.node(SurfaceNodeKind::StructureConstructor, children)
    }

    fn selector(
        &mut self,
        base: SurfaceBuilderNodeId,
        member: &str,
        arguments: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let base_range = self.range(base);
        let mut cursor = base_range.end + 1;
        let dot = self.token_at(SurfaceTokenKind::ReservedSymbol, ".", &mut cursor);
        let member = self.token_at(SurfaceTokenKind::Identifier, member, &mut cursor);
        let mut children = vec![base, dot, member];
        if !arguments.is_empty() {
            children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, "(", &mut cursor));
            let argument_count = arguments.len();
            for (ordinal, argument) in arguments.into_iter().enumerate() {
                let argument_range = self.range(argument);
                assert!(
                    cursor <= argument_range.start,
                    "synthetic selector member must precede its arguments"
                );
                children.push(argument);
                if ordinal + 1 < argument_count {
                    cursor = argument_range.end + 1;
                    children.push(self.token_at(
                        SurfaceTokenKind::ReservedSymbol,
                        ",",
                        &mut cursor,
                    ));
                } else {
                    cursor = argument_range.end + 1;
                }
            }
            children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ")", &mut cursor));
        }
        self.node(SurfaceNodeKind::SelectorAccess, children)
    }

    fn selector_zero_call(
        &mut self,
        base: SurfaceBuilderNodeId,
        member: &str,
    ) -> SurfaceBuilderNodeId {
        let dot = self.token(SurfaceTokenKind::ReservedSymbol, ".");
        let member = self.token(SurfaceTokenKind::Identifier, member);
        let open = self.token(SurfaceTokenKind::ReservedSymbol, "(");
        let close = self.token(SurfaceTokenKind::ReservedSymbol, ")");
        self.node(
            SurfaceNodeKind::SelectorAccess,
            vec![base, dot, member, open, close],
        )
    }

    fn update(
        &mut self,
        base: SurfaceBuilderNodeId,
        updates: Vec<(Vec<&str>, SurfaceBuilderNodeId)>,
    ) -> SurfaceBuilderNodeId {
        let mut cursor = self.range(base).end + 1;
        let with = self.token_at(SurfaceTokenKind::ReservedWord, "with", &mut cursor);
        let open = self.token_at(SurfaceTokenKind::ReservedSymbol, "(", &mut cursor);
        let mut children = vec![base, with, open];
        for (ordinal, (path, value)) in updates.into_iter().enumerate() {
            if ordinal > 0 {
                children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ",", &mut cursor));
            }
            let mut field_children = Vec::new();
            let path_count = path.len();
            for (segment_ordinal, segment) in path.into_iter().enumerate() {
                field_children.push(self.token_at(
                    SurfaceTokenKind::Identifier,
                    segment,
                    &mut cursor,
                ));
                if segment_ordinal + 1 < path_count {
                    field_children.push(self.token_at(
                        SurfaceTokenKind::ReservedSymbol,
                        ".",
                        &mut cursor,
                    ));
                }
            }
            let assign = self.token_at(SurfaceTokenKind::ReservedSymbol, ":=", &mut cursor);
            field_children.push(assign);
            let value_range = self.range(value);
            assert!(
                cursor <= value_range.start,
                "synthetic update path must precede its replacement"
            );
            field_children.push(value);
            let start = self
                .builder
                .node_range(field_children[0])
                .expect("synthetic update path start")
                .start;
            let end = self
                .builder
                .node_range(value)
                .expect("synthetic update value")
                .end;
            children.push(self.builder.add_node(
                SurfaceNodeKind::FieldUpdate,
                SourceRange {
                    source_id: self.source,
                    start,
                    end,
                },
                field_children,
            ));
            cursor = value_range.end + 1;
        }
        children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ")", &mut cursor));
        self.node(SurfaceNodeKind::StructureUpdate, children)
    }

    fn wrapper(&mut self, child: SurfaceBuilderNodeId) -> SurfaceBuilderNodeId {
        let child_range = self
            .builder
            .node_range(child)
            .expect("synthetic wrapper child");
        let mut open_cursor = child_range.start.saturating_sub(2);
        let open = self.token_at(
            SurfaceTokenKind::ReservedSymbol,
            "(",
            &mut open_cursor,
        );
        let mut close_cursor = child_range.end + 1;
        let close = self.token_at(
            SurfaceTokenKind::ReservedSymbol,
            ")",
            &mut close_cursor,
        );
        self.builder.add_node(
            SurfaceNodeKind::ParenthesizedTerm,
            SourceRange {
                source_id: self.source,
                start: self
                    .builder
                    .node_range(open)
                    .expect("synthetic wrapper open")
                    .start,
                end: self
                    .builder
                    .node_range(close)
                    .expect("synthetic wrapper close")
                    .end,
            },
            vec![open, child, close],
        )
    }

    fn template_ancestor(&mut self, child: SurfaceBuilderNodeId) -> SurfaceBuilderNodeId {
        let range = self.range(child);
        self.builder
            .add_node(SurfaceNodeKind::TemplateArgument, range, vec![child])
    }

    fn application_ancestor(&mut self, child: SurfaceBuilderNodeId) -> SurfaceBuilderNodeId {
        let range = self.range(child);
        self.builder
            .add_node(SurfaceNodeKind::ApplicationTerm, range, vec![child])
    }

    fn finish(mut self, roots: Vec<SurfaceBuilderNodeId>) -> SurfaceAst {
        let end = roots
            .iter()
            .filter_map(|root| self.builder.node_range(*root))
            .map(|range| range.end)
            .max()
            .unwrap_or(self.next);
        let root = self.builder.add_node(
            SurfaceNodeKind::Root,
            SourceRange {
                source_id: self.source,
                start: 0,
                end: end + 1,
            },
            roots,
        );
        self.builder.finish(Some(root), None)
    }
}
