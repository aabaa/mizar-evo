use mizar_checker::typed_ast::{
    FactProvenance, FactStatus, Polarity, TypeFactDraft, TypeFactId, TypeFactTable,
    TypePredicateRef, TypedArena, TypedAst, TypedAstParts, TypedNode, TypedNodeId, TypedSiteRef,
};
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
    declarations::DeclarationShellCollector,
    env::{NamespacePath, SymbolEnv},
    resolved_ast::{ModuleId, SurfaceResolvedArena},
    symbols::SignatureProjectionExtractor,
};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
    SourceInput, SourceOriginInput,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicUsize, Ordering},
};

static FIXTURE_ID: AtomicUsize = AtomicUsize::new(1);
static IDS: OnceLock<InMemorySessionIdAllocator> = OnceLock::new();

pub struct SourceFixture {
    pub native: SurfaceResolvedArena,
    pub symbols: SymbolEnv,
    pub typed: TypedAst,
}

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

impl SourceFixture {
    pub fn parse(source_text: &str, module_name: &str) -> Self {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("checker278-{id}"));
        std::fs::create_dir_all(root.join("src")).expect("create parser fixture directory");
        let path = root.join("src/fixture.miz");
        std::fs::write(&path, source_text).expect("write parser fixture");
        let module = ModuleId::new(
            PackageId::new(format!("checker278-{id}")),
            ModulePath::new(module_name),
        );
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{id:064x}"
        ))
        .expect("fixture snapshot");
        let output = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            NoImports,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot,
                input: SourceInput {
                    package_id: module.package().clone(),
                    module_path: module.path().clone(),
                    normalized_path: mizar_session::normalize_path(&root, &path)
                        .expect("normalize fixture path"),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path: path.clone() },
                },
            },
            IDS.get_or_init(InMemorySessionIdAllocator::new),
        )
        .expect("parse fixture");
        std::fs::remove_dir_all(&root).expect("remove parser fixture directory");
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let ast = output.ast.expect("fixture AST");
        let shells = DeclarationShellCollector::new(&ast, &module).collect();
        let collected =
            SignatureProjectionExtractor::new(&ast, &shells, NamespacePath::new(module_name))
                .collect(&module);
        // Keep real resolver refusals for the conflicting-return negative case.
        let native = SurfaceResolvedArena::lower(&ast, &module).expect("resolved fixture AST");
        let nodes = TypedArena::try_new(
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
        .expect("canonical typed source rows");
        let mut typed = TypedAst::try_new(TypedAstParts {
            source_id: native.source_id(),
            module_id: module,
            resolved_root: Some(native.arena().root()),
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes,
            contexts: Default::default(),
            types: Default::default(),
            facts: Default::default(),
            coercions: Default::default(),
            initial_obligations: Default::default(),
            diagnostics: Default::default(),
        })
        .expect("canonical typed AST");
        typed
            .install_source_nodes(&native)
            .expect("install authenticated source rows");
        Self {
            native,
            symbols: collected.into_env(),
            typed,
        }
    }

    pub fn with_known_fact(
        &self,
        subject: TypedSiteRef,
        predicate: &str,
    ) -> (TypedAst, TypeFactId) {
        let node = self
            .typed
            .nodes()
            .node(subject.node())
            .expect("fact subject is a canonical typed source node");
        let mizar_session::SourceAnchor::Range(range) = &node.anchor else {
            panic!("fact subject has a source range anchor");
        };
        let mut facts = TypeFactTable::new();
        let id = facts.insert(TypeFactDraft {
            subject,
            predicate: TypePredicateRef::new(predicate),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Declared((*range).into()),
            status: FactStatus::Known,
        });
        let mut typed = TypedAst::try_new(TypedAstParts {
            source_id: self.typed.source_id(),
            module_id: self.typed.module_id().clone(),
            resolved_root: self.typed.resolved_root(),
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: self.typed.nodes().clone(),
            contexts: self.typed.contexts().clone(),
            types: self.typed.types().clone(),
            facts,
            coercions: self.typed.coercions().clone(),
            initial_obligations: self.typed.initial_obligations().clone(),
            diagnostics: self.typed.diagnostics().clone(),
        })
        .expect("typed AST with source-anchored fact");
        typed
            .install_source_nodes(&self.native)
            .expect("install authenticated source rows");
        (typed, id)
    }
}

pub fn ordinary_overloads() -> SourceFixture {
    SourceFixture::parse(
        "definition let x be set; func OvRootSet: ovroot(x) -> set equals x; coherence; end;\n\
         definition let x be object; func OvRootObject: ovroot(x) -> object equals x; coherence; end;\n\
         reserve x for set; theorem OvUse: ovroot(x) = x;\n",
        "ordinary_overloads",
    )
}

pub fn single_ordinary() -> SourceFixture {
    SourceFixture::parse(
        "definition let x be set; func OvRoot: ovroot(x) -> set equals x; coherence; end;\n\
         reserve x for set; theorem OvUse: ovroot(x) = x;\n",
        "single_ordinary",
    )
}

pub fn source_view() -> SourceFixture {
    SourceFixture::parse(
        "definition let x be object; func OvRoot: ovroot(x) -> object equals x; coherence; end;\n\
         reserve x for set; theorem OvUse: ovroot(x qua object) = x;\n",
        "source_view",
    )
}

// reason: Shared by two integration-test binaries; only the positive binary uses this fixture.
#[allow(dead_code)]
pub fn template_overload() -> SourceFixture {
    SourceFixture::parse(
        "definition let T be type; let x be T; func OvTemplate: ovtemplate[T](x) -> T equals x; coherence; end;\n\
         reserve x for set; theorem OvUse: ovtemplate[set](x) = x;\n",
        "template_overload",
    )
}
