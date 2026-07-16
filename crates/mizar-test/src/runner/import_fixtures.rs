use std::collections::BTreeSet;

use mizar_frontend::lexical_env::{
    ExportRank, ExportedOperatorAssociativity, ExportedOperatorFixity, ExportedOperatorMetadata,
    ExportedSymbolShape, FrontendLexicalEnvironmentError, LexicalEnvironmentRequest,
    LexicalSummaryFingerprint, LexicalSummaryProvider, ModuleId, ModuleLexicalSummary,
    ResolvedImport, ResolvedImportEntry, ResolvedImports, SymbolId, UserSymbolArity,
    UserSymbolKind,
};
use mizar_resolve::env::{
    ContributionKind, ExportStatus, NamespacePath, SymbolEntry, SymbolEnv, SymbolEnvIndexes,
    SymbolKind, Visibility,
};
use mizar_resolve::resolved_ast::{
    FullyQualifiedName, LocalSymbolId, ModuleId as ResolverModuleId, SemanticOrigin,
    SymbolId as ResolverSymbolId,
};
use mizar_session::{ModulePath, SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeKind};

pub(super) fn augment_type_elaboration_import_summaries(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: SymbolEnv,
) -> SymbolEnv {
    let imported_modules = type_elaboration_imported_fixture_modules(ast, module);
    if imported_modules.is_empty() {
        return symbols;
    }
    let mut indexes = clone_symbol_env_indexes(&symbols);
    for (imported_module, anchor) in imported_modules {
        let frontend_module = ModuleId::new(imported_module.path().as_str());
        let exported_symbols = parse_only_fixture_symbols(&frontend_module);
        if exported_symbols.is_empty() {
            continue;
        }
        let contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource {
                source_id: ast.source_id,
            },
            SourceAnchor::Range(anchor),
        );
        for (ordinal, exported) in exported_symbols.iter().enumerate() {
            if !matches!(
                (exported.kind, exported.spelling.as_str()),
                (UserSymbolKind::Attribute, "empty")
                    | (UserSymbolKind::Attribute, "TypeCaseAttr")
                    | (UserSymbolKind::Mode, "TypeCaseMode")
                    | (UserSymbolKind::Structure, "R")
                    | (UserSymbolKind::Structure, "TypeCaseStruct")
                    | (UserSymbolKind::Predicate, "divides")
                    | (UserSymbolKind::Functor, "++")
            ) {
                continue;
            }
            let Some(kind) = resolver_symbol_kind(exported.kind) else {
                continue;
            };
            let symbol = ResolverSymbolId::new(
                imported_module.clone(),
                LocalSymbolId::new(format!("summary:{}:{ordinal}", exported.symbol_id.as_str())),
                FullyQualifiedName::new(format!(
                    "{}::{}#{}",
                    imported_module.path().as_str(),
                    exported.spelling,
                    ordinal
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol.clone(),
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    exported.spelling.clone(),
                    SemanticOrigin::new(
                        ast.source_id,
                        imported_module.clone(),
                        SourceAnchor::Range(anchor),
                        vec![ordinal as u32],
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
            indexes.contributions.add_symbol(contribution, symbol);
        }
    }
    SymbolEnv::new(module.clone(), indexes)
}

fn clone_symbol_env_indexes(symbols: &SymbolEnv) -> SymbolEnvIndexes {
    SymbolEnvIndexes {
        imports: symbols.imports().clone(),
        exports: symbols.exports().clone(),
        symbols: symbols.symbols().clone(),
        labels: symbols.labels().clone(),
        definitions: symbols.definitions().clone(),
        overloads: symbols.overloads().clone(),
        registrations: symbols.registrations().clone(),
        lexical_summaries: symbols.lexical_summaries().clone(),
        namespace_graph: symbols.namespace_graph().clone(),
        declaration_dependencies: symbols.declaration_dependencies().clone(),
        contributions: symbols.contributions().clone(),
        module_summaries: symbols.module_summaries().clone(),
    }
}

fn type_elaboration_imported_fixture_modules(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
) -> Vec<(ResolverModuleId, SourceRange)> {
    let mut modules = Vec::new();
    for node in ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
    {
        let Some(module_path) = node
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .find(|child| matches!(child.kind, SurfaceNodeKind::ModulePath))
        else {
            continue;
        };
        let Ok(spelling) = module_path_spelling(ast, module_path) else {
            continue;
        };
        let frontend_module = ModuleId::new(spelling.as_str());
        if parse_only_fixture_symbols(&frontend_module).is_empty() {
            continue;
        }
        let imported_module =
            ResolverModuleId::new(module.package().clone(), ModulePath::new(spelling.as_str()));
        if modules
            .iter()
            .any(|(existing, _)| existing == &imported_module)
        {
            continue;
        }
        modules.push((imported_module, module_path.range));
    }
    modules
}

pub(super) fn module_path_spelling(ast: &SurfaceAst, node: &SurfaceNode) -> Result<String, ()> {
    if !matches!(node.kind, SurfaceNodeKind::ModulePath) || node.children.is_empty() {
        return Err(());
    }
    let mut segments = Vec::new();
    for child_id in &node.children {
        let child = ast.node(*child_id).ok_or(())?;
        if !matches!(child.kind, SurfaceNodeKind::PathSegment) || child.children.len() != 1 {
            continue;
        }
        let token = ast
            .node(child.children[0])
            .and_then(SurfaceNode::token_text)
            .ok_or(())?;
        segments.push(token.to_owned());
    }
    if segments.is_empty() {
        return Err(());
    }
    Ok(segments.join("."))
}

fn resolver_symbol_kind(kind: UserSymbolKind) -> Option<SymbolKind> {
    match kind {
        UserSymbolKind::Functor => Some(SymbolKind::Functor),
        UserSymbolKind::Predicate => Some(SymbolKind::Predicate),
        UserSymbolKind::Mode => Some(SymbolKind::Mode),
        UserSymbolKind::Attribute => Some(SymbolKind::Attribute),
        UserSymbolKind::Structure => Some(SymbolKind::Structure),
        UserSymbolKind::Selector => Some(SymbolKind::Selector),
        UserSymbolKind::Constructor => None,
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParseOnlyImportProvider;

impl LexicalSummaryProvider for ParseOnlyImportProvider {
    fn resolve_imports(
        &self,
        request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
        let mut imports = Vec::new();
        let mut summaries = Vec::new();
        let mut seen_modules = BTreeSet::new();

        for (stub_ordinal, stub) in request.import_stubs.iter().enumerate() {
            let module_id = ModuleId::new(stub.path.spelling.as_ref());
            imports.push(ResolvedImportEntry {
                stub_ordinal,
                stub_span: stub.span,
                import: ResolvedImport {
                    module_id: module_id.clone(),
                },
            });

            if seen_modules.insert(module_id.clone()) {
                summaries.push(ModuleLexicalSummary {
                    exported_symbols: parse_only_fixture_symbols(&module_id),
                    module_id,
                    fingerprint: LexicalSummaryFingerprint::new((stub_ordinal as u64) + 1),
                });
            }
        }

        Ok(ResolvedImports {
            imports,
            summaries,
            diagnostics: Vec::new(),
        })
    }
}

fn parse_only_fixture_symbols(module_id: &ModuleId) -> Vec<ExportedSymbolShape> {
    if module_id.as_str() != "parser.type_fixtures" {
        return Vec::new();
    }
    [
        (
            "empty",
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(0),
            None,
        ),
        (
            "T",
            UserSymbolKind::Mode,
            UserSymbolArity::at_least(0),
            None,
        ),
        (
            "R",
            UserSymbolKind::Structure,
            UserSymbolArity::at_least(0),
            None,
        ),
        (
            "TypeCaseAttr",
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(0),
            None,
        ),
        (
            "TypeCaseMode",
            UserSymbolKind::Mode,
            UserSymbolArity::at_least(0),
            None,
        ),
        (
            "TypeCaseStruct",
            UserSymbolKind::Structure,
            UserSymbolArity::at_least(0),
            None,
        ),
        (
            "divides",
            UserSymbolKind::Predicate,
            UserSymbolArity::exact(2),
            None,
        ),
        (
            "<=",
            UserSymbolKind::Predicate,
            UserSymbolArity::exact(2),
            None,
        ),
        (
            "~",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(1),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Prefix,
                precedence: 70,
            }),
        ),
        (
            "!",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(1),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Postfix,
                precedence: 90,
            }),
        ),
        (
            "|.",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(1),
            None,
        ),
        (
            ".|",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(1),
            None,
        ),
        (
            "++",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(2),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
                precedence: 10,
            }),
        ),
        (
            "**",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(2),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Right),
                precedence: 20,
            }),
        ),
        (
            "%%",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(2),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Infix(
                    ExportedOperatorAssociativity::NonAssociative,
                ),
                precedence: 10,
            }),
        ),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(rank, (spelling, kind, arity, operator))| ExportedSymbolShape {
            spelling: spelling.to_owned(),
            symbol_id: SymbolId::new(format!("{}#parse-only#{spelling}", module_id.as_str())),
            source_module: module_id.clone(),
            export_rank: ExportRank::new(rank as u32),
            kind,
            arity,
            operator,
        },
    )
    .collect()
}
