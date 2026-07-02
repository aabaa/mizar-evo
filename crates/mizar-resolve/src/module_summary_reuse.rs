//! Dependency `ModuleSummary` reuse through canonical artifact projections.
//!
//! The resolver consumes `mizar-artifact` module summaries after the artifact
//! reader has validated schema version, canonical ordering, identity, and hash
//! policy. This module maps the validated dependency-facing projection into
//! resolver-owned `SymbolEnv` indexes without owning artifact formats or
//! loading dependency source files.

use std::collections::BTreeMap;

use crate::env::{
    ContributionKind, DeclarationDependencyKind, DependencyEndpoint, ExportStatus, LabelEntry,
    LexicalSummaryKind, ModuleSummaryIdentity as ResolverModuleSummaryIdentity, NamespacePath,
    SignatureShell, SourceContributionId, SymbolEntry, SymbolEnv, SymbolEnvIndexes, SymbolKind,
    Visibility,
};
use crate::module_index::{IndexedModuleId, ModuleIndexInput, ModuleIndexProviderError};
use crate::resolved_ast::{
    FullyQualifiedName, LabelKind, LabelOriginPath, LocalSymbolId, ModuleId, SemanticOrigin,
    SymbolId,
};
use mizar_artifact::module_summary::{
    self as artifact_summary, DependencyInterfaceRef, ExportedLabelSummary, ExportedSymbolSummary,
    ModuleReexportSummary, ModuleSummary, ModuleSummaryIdentity, ModuleSummaryReadOptions,
};
use mizar_artifact::store::CanonicalJson;
use mizar_session::{
    GeneratedSpanAnchor, GeneratedSpanOrigin, Hash, ModulePath, PackageId, SourceAnchor, SourceId,
    SourceRange,
};

/// Summary reuse request for one canonical dependency module.
#[derive(Debug, Clone)]
pub struct ModuleSummaryReuseRequest<'a> {
    module: &'a IndexedModuleId,
    anchor: SourceAnchor,
    expected_interface_hash: Option<Hash>,
}

impl<'a> ModuleSummaryReuseRequest<'a> {
    /// Creates a summary reuse request.
    #[must_use]
    pub const fn new(module: &'a IndexedModuleId, anchor: SourceAnchor) -> Self {
        Self {
            module,
            anchor,
            expected_interface_hash: None,
        }
    }

    /// Requires the artifact summary to have the provided interface hash.
    #[must_use]
    pub const fn with_expected_interface_hash(mut self, hash: Hash) -> Self {
        self.expected_interface_hash = Some(hash);
        self
    }

    /// Returns the requested canonical module.
    #[must_use]
    pub const fn module(&self) -> &'a IndexedModuleId {
        self.module
    }

    /// Returns the caller-supplied source or generated anchor.
    #[must_use]
    pub const fn anchor(&self) -> &SourceAnchor {
        &self.anchor
    }
}

/// Result of trying to reuse one dependency module summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSummaryReuseResult {
    env: Option<SymbolEnv>,
    diagnostics: Vec<ModuleSummaryReuseDiagnostic>,
}

impl ModuleSummaryReuseResult {
    fn reused(env: SymbolEnv, diagnostics: Vec<ModuleSummaryReuseDiagnostic>) -> Self {
        Self {
            env: Some(env),
            diagnostics,
        }
    }

    fn fallback(diagnostic: ModuleSummaryReuseDiagnostic) -> Self {
        Self {
            env: None,
            diagnostics: vec![diagnostic],
        }
    }

    /// Returns the summary-backed environment when reuse succeeded.
    #[must_use]
    pub const fn env(&self) -> Option<&SymbolEnv> {
        self.env.as_ref()
    }

    /// Returns crate-local/internal summary reuse diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[ModuleSummaryReuseDiagnostic] {
        &self.diagnostics
    }

    /// Returns whether a summary-backed environment was produced.
    #[must_use]
    pub const fn reused_summary(&self) -> bool {
        self.env.is_some()
    }
}

/// Crate-local/internal summary reuse fallback or warning record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSummaryReuseDiagnostic {
    module: ModuleId,
    artifact: Option<String>,
    reason: ModuleSummaryReuseReason,
    detail: String,
}

impl ModuleSummaryReuseDiagnostic {
    fn new(
        module: ModuleId,
        artifact: Option<String>,
        reason: ModuleSummaryReuseReason,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            module,
            artifact,
            reason,
            detail: detail.into(),
        }
    }

    /// Returns the canonical resolver module.
    #[must_use]
    pub const fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Returns the artifact path when the build provider supplied one.
    #[must_use]
    pub fn artifact(&self) -> Option<&str> {
        self.artifact.as_deref()
    }

    /// Returns the deterministic fallback or warning reason.
    #[must_use]
    pub const fn reason(&self) -> ModuleSummaryReuseReason {
        self.reason
    }

    /// Returns stable crate-local detail text.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Summary reuse fallback or warning reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ModuleSummaryReuseReason {
    /// The module index provider did not expose a usable summary reference.
    SummaryUnavailable,
    /// The artifact reader rejected the summary.
    ArtifactRejected,
    /// The summary contains a resolver projection that would require guessing.
    UnsupportedProjection,
    /// A lexical contribution could not be paired with an exported symbol.
    UnpairedLexicalContribution,
}

/// Resolver-side adapter for canonical `ModuleSummary` values.
#[derive(Clone, Copy)]
pub struct ModuleSummaryReuse<'a> {
    module_index: ModuleIndexInput<'a>,
}

impl<'a> ModuleSummaryReuse<'a> {
    /// Creates a summary reuse adapter over the resolver module-index input.
    #[must_use]
    pub const fn new(module_index: ModuleIndexInput<'a>) -> Self {
        Self { module_index }
    }

    /// Reads a canonical module-summary JSON value through `mizar-artifact` and
    /// projects it into resolver-owned indexes.
    #[must_use]
    pub fn read_and_project(
        self,
        request: ModuleSummaryReuseRequest<'_>,
        value: &CanonicalJson,
    ) -> ModuleSummaryReuseResult {
        let context = match self.context(&request) {
            Ok(context) => context,
            Err(diagnostic) => return ModuleSummaryReuseResult::fallback(diagnostic),
        };
        let read = artifact_summary::read_module_summary(
            value,
            ModuleSummaryReadOptions {
                artifact_path: context.artifact.as_deref(),
                expected_module: None,
                expected_interface_hash: request.expected_interface_hash,
            },
        );
        match read {
            Ok(summary) => {
                if let Some(detail) =
                    known_identity_mismatch_detail(&context.expected_identity, &summary.module)
                {
                    return ModuleSummaryReuseResult::fallback(ModuleSummaryReuseDiagnostic::new(
                        context.module,
                        context.artifact,
                        ModuleSummaryReuseReason::ArtifactRejected,
                        detail,
                    ));
                }
                project_summary(context, &request.anchor, &summary)
            }
            Err(error) => ModuleSummaryReuseResult::fallback(ModuleSummaryReuseDiagnostic::new(
                context.module,
                context.artifact,
                ModuleSummaryReuseReason::ArtifactRejected,
                error.to_string(),
            )),
        }
    }

    /// Projects an already validated module summary into resolver-owned
    /// indexes. Callers that start from artifact JSON should prefer
    /// [`Self::read_and_project`].
    #[must_use]
    pub fn project_validated(
        self,
        request: ModuleSummaryReuseRequest<'_>,
        summary: &ModuleSummary,
    ) -> ModuleSummaryReuseResult {
        let context = match self.context(&request) {
            Ok(context) => context,
            Err(diagnostic) => return ModuleSummaryReuseResult::fallback(diagnostic),
        };
        if let Some(detail) =
            known_identity_mismatch_detail(&context.expected_identity, &summary.module)
        {
            return ModuleSummaryReuseResult::fallback(ModuleSummaryReuseDiagnostic::new(
                context.module,
                context.artifact,
                ModuleSummaryReuseReason::ArtifactRejected,
                detail,
            ));
        }
        if let Some(expected) = request.expected_interface_hash
            && summary.interface_hash != expected
        {
            return ModuleSummaryReuseResult::fallback(ModuleSummaryReuseDiagnostic::new(
                context.module,
                context.artifact,
                ModuleSummaryReuseReason::ArtifactRejected,
                "module summary interface hash does not match requested hash",
            ));
        }
        project_summary(context, &request.anchor, summary)
    }

    fn context(
        self,
        request: &ModuleSummaryReuseRequest<'_>,
    ) -> Result<ModuleSummaryReuseContext, ModuleSummaryReuseDiagnostic> {
        let module = crate::module_index::resolver_module_id(request.module);
        let module_entry = self
            .module_index
            .module(request.module)
            .map_err(|error| provider_diagnostic(&module, None, error))?;
        let summary_ref = self
            .module_index
            .dependency_summary(request.module)
            .map_err(|error| provider_diagnostic(&module, None, error))?;
        let package = self
            .module_index
            .package(&request.module.package)
            .map_err(|error| {
                provider_diagnostic(&module, Some(summary_ref.artifact.clone()), error)
            })?;
        let expected_identity = ModuleSummaryIdentity {
            package_id: request.module.package.as_str().to_owned(),
            package_version: Some(package.version.to_string()),
            lockfile_identity: None,
            module_path: request.module.path.as_str().to_owned(),
            language_edition: module_entry.edition.as_str().to_owned(),
        };
        Ok(ModuleSummaryReuseContext {
            module,
            artifact: Some(summary_ref.artifact.clone()),
            expected_identity,
        })
    }
}

#[derive(Debug, Clone)]
struct ModuleSummaryReuseContext {
    module: ModuleId,
    artifact: Option<String>,
    expected_identity: ModuleSummaryIdentity,
}

fn provider_diagnostic(
    module: &ModuleId,
    artifact: Option<String>,
    error: ModuleIndexProviderError,
) -> ModuleSummaryReuseDiagnostic {
    ModuleSummaryReuseDiagnostic::new(
        module.clone(),
        artifact,
        ModuleSummaryReuseReason::SummaryUnavailable,
        error.to_string(),
    )
}

fn known_identity_mismatch_detail(
    expected: &ModuleSummaryIdentity,
    actual: &ModuleSummaryIdentity,
) -> Option<String> {
    let mismatch = expected.package_id != actual.package_id
        || expected.package_version != actual.package_version
        || expected.module_path != actual.module_path
        || expected.language_edition != actual.language_edition;
    mismatch.then(|| {
        format!(
            "module summary identity does not match requested module: expected package={} version={} module={} edition={}, actual package={} version={} module={} edition={}",
            expected.package_id,
            optional_identity_field(expected.package_version.as_deref()),
            expected.module_path,
            expected.language_edition,
            actual.package_id,
            optional_identity_field(actual.package_version.as_deref()),
            actual.module_path,
            actual.language_edition
        )
    })
}

fn optional_identity_field(value: Option<&str>) -> &str {
    value.unwrap_or("<none>")
}

fn project_summary(
    context: ModuleSummaryReuseContext,
    anchor: &SourceAnchor,
    summary: &ModuleSummary,
) -> ModuleSummaryReuseResult {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution_identity = resolver_summary_identity(&summary.module, summary.interface_hash);
    let contribution = indexes.contributions.insert(
        context.module.clone(),
        ContributionKind::Summary {
            identity: contribution_identity.clone(),
        },
        anchor.clone(),
    );
    indexes
        .module_summaries
        .insert(context.module.clone(), contribution_identity, contribution);

    let Some(source_id) = anchor_source_id(anchor) else {
        return ModuleSummaryReuseResult::fallback(ModuleSummaryReuseDiagnostic::new(
            context.module,
            context.artifact,
            ModuleSummaryReuseReason::UnsupportedProjection,
            "unsupported source anchor for module summary projection",
        ));
    };
    let mut diagnostics = Vec::new();
    let mut symbols_by_key = BTreeMap::new();

    for (ordinal, symbol) in summary.exported_symbols.iter().enumerate() {
        let Some(symbol_entry) =
            lower_exported_symbol(&context, anchor, source_id, contribution, ordinal, symbol)
        else {
            return ModuleSummaryReuseResult::fallback(ModuleSummaryReuseDiagnostic::new(
                context.module,
                context.artifact,
                ModuleSummaryReuseReason::UnsupportedProjection,
                format!(
                    "unsupported exported symbol `{}`",
                    symbol.fully_qualified_name
                ),
            ));
        };
        if symbol_entry.export_status() != ExportStatus::LocalOnly {
            symbols_by_key.insert(symbol.origin_id.clone(), symbol_entry.symbol().clone());
            symbols_by_key.insert(
                symbol.fully_qualified_name.clone(),
                symbol_entry.symbol().clone(),
            );
        }
        indexes
            .contributions
            .add_symbol(contribution, symbol_entry.symbol().clone());
        indexes.symbols.insert(symbol_entry);
    }

    for (ordinal, label) in summary.exported_labels.iter().enumerate() {
        let Some(label_entry) =
            lower_exported_label(&context, anchor, source_id, contribution, ordinal, label)
        else {
            return ModuleSummaryReuseResult::fallback(ModuleSummaryReuseDiagnostic::new(
                context.module,
                context.artifact,
                ModuleSummaryReuseReason::UnsupportedProjection,
                format!("unsupported exported label `{}`", label.label),
            ));
        };
        indexes
            .contributions
            .add_label(contribution, label_entry.origin_path().clone());
        indexes.labels.insert(label_entry);
    }

    for lexical in &summary.lexical_summary.contributions {
        let Some(kind) = lexical_kind(&lexical.kind) else {
            diagnostics.push(ModuleSummaryReuseDiagnostic::new(
                context.module.clone(),
                context.artifact.clone(),
                ModuleSummaryReuseReason::UnsupportedProjection,
                format!("unsupported lexical contribution kind `{}`", lexical.kind),
            ));
            continue;
        };
        let Some(symbol) = symbols_by_key.get(&lexical.key).cloned() else {
            diagnostics.push(ModuleSummaryReuseDiagnostic::new(
                context.module.clone(),
                context.artifact.clone(),
                ModuleSummaryReuseReason::UnpairedLexicalContribution,
                format!("unpaired lexical contribution `{}`", lexical.key),
            ));
            continue;
        };
        let summary_id = indexes.lexical_summaries.insert(
            symbol,
            NamespacePath::new(summary.module.module_path.clone()),
            lexical.key.clone(),
            kind,
            None,
            contribution,
        );
        indexes
            .contributions
            .add_lexical_summary(contribution, summary_id);
    }

    for reexport in &summary.reexports {
        insert_reexport_dependency(&mut indexes, &context, anchor, contribution, reexport);
    }
    for dependency in &summary.dependency_interfaces {
        insert_dependency_interface(&mut indexes, &context, anchor, contribution, dependency);
    }

    ModuleSummaryReuseResult::reused(SymbolEnv::new(context.module, indexes), diagnostics)
}

fn lower_exported_symbol(
    context: &ModuleSummaryReuseContext,
    anchor: &SourceAnchor,
    source_id: SourceId,
    contribution: SourceContributionId,
    ordinal: usize,
    symbol: &ExportedSymbolSummary,
) -> Option<SymbolEntry> {
    let visibility = summary_visibility(&symbol.visibility)?;
    let kind = symbol_kind(&symbol.declaration_kind)?;
    let symbol_id = SymbolId::new(
        context.module.clone(),
        LocalSymbolId::new(format!("summary:{}", symbol.origin_id)),
        FullyQualifiedName::new(symbol.fully_qualified_name.clone()),
    );
    let mut entry = SymbolEntry::new(
        symbol_id,
        kind,
        namespace_path(&symbol.namespace_path),
        primary_spelling(&symbol.fully_qualified_name),
        SemanticOrigin::new(
            source_id,
            context.module.clone(),
            anchor.clone(),
            vec![0, ordinal as u32],
        ),
        contribution,
    )
    .with_visibility(visibility.visibility)
    .with_export_status(visibility.export_status)
    .with_signature(SignatureShell::Opaque {
        schema: "mizar-artifact/module-summary-symbol-v1".to_owned(),
        payload: symbol_payload(symbol),
    });
    if symbol.declaration_kind == "selector" {
        entry = entry.with_notation_spelling(symbol.rendered_signature.clone());
    }
    Some(entry)
}

fn lower_exported_label(
    context: &ModuleSummaryReuseContext,
    anchor: &SourceAnchor,
    source_id: SourceId,
    contribution: SourceContributionId,
    ordinal: usize,
    label: &ExportedLabelSummary,
) -> Option<LabelEntry> {
    let visibility = summary_visibility(&label.visibility)?;
    let kind = label_kind(&label.target_kind)?;
    Some(
        LabelEntry::new(
            LabelOriginPath::new(format!(
                "summary:{}:{}",
                label.owner_fully_qualified_name, label.origin_id
            )),
            kind,
            NamespacePath::new(context.module.path().as_str()),
            label.label.clone(),
            SemanticOrigin::new(
                source_id,
                context.module.clone(),
                anchor.clone(),
                vec![1, ordinal as u32],
            ),
            contribution,
        )
        .with_visibility(visibility.visibility)
        .with_export_status(visibility.export_status),
    )
}

fn insert_reexport_dependency(
    indexes: &mut SymbolEnvIndexes,
    context: &ModuleSummaryReuseContext,
    anchor: &SourceAnchor,
    contribution: SourceContributionId,
    reexport: &ModuleReexportSummary,
) {
    let target = module_id_from_summary_identity(&reexport.target_module);
    let dependency = indexes.declaration_dependencies.insert(
        DependencyEndpoint::Module(context.module.clone()),
        DependencyEndpoint::Module(target),
        DeclarationDependencyKind::ReExport,
        generated_summary_anchor(anchor, "module-summary-reexport"),
        contribution,
    );
    indexes
        .contributions
        .add_declaration_dependency(contribution, dependency);
}

fn insert_dependency_interface(
    indexes: &mut SymbolEnvIndexes,
    context: &ModuleSummaryReuseContext,
    anchor: &SourceAnchor,
    contribution: SourceContributionId,
    dependency: &DependencyInterfaceRef,
) {
    let target = module_id_from_summary_identity(&dependency.module);
    let dependency_identity =
        resolver_summary_identity(&dependency.module, dependency.interface_hash);
    indexes
        .module_summaries
        .insert(target.clone(), dependency_identity, contribution);
    let edge = indexes.declaration_dependencies.insert(
        DependencyEndpoint::Module(context.module.clone()),
        DependencyEndpoint::Module(target),
        DeclarationDependencyKind::Import,
        generated_summary_anchor(anchor, "module-summary-dependency-interface"),
        contribution,
    );
    indexes
        .contributions
        .add_declaration_dependency(contribution, edge);
}

#[derive(Debug, Clone, Copy)]
struct LoweredVisibility {
    visibility: Visibility,
    export_status: ExportStatus,
}

fn summary_visibility(value: &str) -> Option<LoweredVisibility> {
    match normalized_identifier(value).as_str() {
        "public" | "pub" | "exported" => Some(LoweredVisibility {
            visibility: Visibility::Public,
            export_status: ExportStatus::Exported,
        }),
        "private" | "local" | "local_only" => Some(LoweredVisibility {
            visibility: Visibility::Private,
            export_status: ExportStatus::LocalOnly,
        }),
        _ => None,
    }
}

fn symbol_kind(value: &str) -> Option<SymbolKind> {
    match normalized_identifier(value).as_str() {
        "predicate" => Some(SymbolKind::Predicate),
        "functor" => Some(SymbolKind::Functor),
        "mode" => Some(SymbolKind::Mode),
        "attribute" => Some(SymbolKind::Attribute),
        "structure" | "struct" => Some(SymbolKind::Structure),
        "selector" => Some(SymbolKind::Selector),
        "registration" => Some(SymbolKind::Registration),
        "theorem" => Some(SymbolKind::Theorem),
        "lemma" => Some(SymbolKind::Lemma),
        "algorithm" => Some(SymbolKind::Algorithm),
        "scheme" => Some(SymbolKind::Scheme),
        "template" => Some(SymbolKind::Template),
        "synonym" => Some(SymbolKind::Synonym),
        "antonym" => Some(SymbolKind::Antonym),
        "redefinition" => Some(SymbolKind::Redefinition),
        "builtin" => Some(SymbolKind::Builtin),
        _ => None,
    }
}

fn label_kind(value: &str) -> Option<LabelKind> {
    match normalized_identifier(value).as_str() {
        "theorem" | "lemma" => Some(LabelKind::Theorem),
        "definition" => Some(LabelKind::Definition),
        "proof_step" | "proof-step" => Some(LabelKind::ProofStep),
        "registration" => Some(LabelKind::Registration),
        _ => None,
    }
}

fn lexical_kind(value: &str) -> Option<LexicalSummaryKind> {
    match normalized_identifier(value).as_str() {
        "notation" => Some(LexicalSummaryKind::Notation),
        "selector" => Some(LexicalSummaryKind::Selector),
        "constructor" => Some(LexicalSummaryKind::Constructor),
        _ => None,
    }
}

fn normalized_identifier(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}

fn namespace_path(components: &[String]) -> NamespacePath {
    NamespacePath::new(components.join("."))
}

fn primary_spelling(fqn: &str) -> String {
    fqn.rsplit("::")
        .next()
        .and_then(|tail| tail.rsplit('.').next())
        .unwrap_or(fqn)
        .to_owned()
}

fn symbol_payload(symbol: &ExportedSymbolSummary) -> String {
    format!(
        "kind={};signature={};fingerprint={}",
        symbol.declaration_kind,
        symbol.rendered_signature,
        hash_hex(&symbol.interface_fingerprint)
    )
}

fn resolver_summary_identity(
    identity: &ModuleSummaryIdentity,
    interface_hash: Hash,
) -> ResolverModuleSummaryIdentity {
    ResolverModuleSummaryIdentity::new(format!(
        "mizar-artifact/module-summary:{}:{}:{}",
        identity.package_id,
        identity.module_path,
        hash_hex(&interface_hash)
    ))
}

fn module_id_from_summary_identity(identity: &ModuleSummaryIdentity) -> ModuleId {
    ModuleId::new(
        PackageId::new(identity.package_id.clone()),
        ModulePath::new(identity.module_path.clone()),
    )
}

fn anchor_source_id(anchor: &SourceAnchor) -> Option<SourceId> {
    match anchor {
        SourceAnchor::Range(range) => Some(range.source_id),
        SourceAnchor::Point { source_id, .. } => Some(*source_id),
        SourceAnchor::Generated(origin) => match origin.anchor() {
            GeneratedSpanAnchor::Range(SourceRange { source_id, .. })
            | GeneratedSpanAnchor::Point { source_id, .. } => Some(source_id),
            _ => None,
        },
        _ => None,
    }
}

fn generated_summary_anchor(anchor: &SourceAnchor, reason: &str) -> SourceAnchor {
    generated_span_anchor(anchor)
        .and_then(|generated| GeneratedSpanOrigin::new(generated, reason).ok())
        .map(SourceAnchor::Generated)
        .unwrap_or_else(|| anchor.clone())
}

fn generated_span_anchor(anchor: &SourceAnchor) -> Option<GeneratedSpanAnchor> {
    match anchor {
        SourceAnchor::Range(range) => Some(GeneratedSpanAnchor::Range(*range)),
        SourceAnchor::Point { source_id, offset } => Some(GeneratedSpanAnchor::Point {
            source_id: *source_id,
            offset: *offset,
        }),
        SourceAnchor::Generated(origin) => Some(origin.anchor()),
        _ => None,
    }
}

fn hash_hex(hash: &Hash) -> String {
    hash.as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarations::DeclarationShellCollector;
    use crate::env::DefinitionKind;
    use crate::module_index::{
        DependencyModuleSummaryRef, ModuleIndexEntry, ModuleIndexLocation, NamespaceRoot,
        PackageIndexEntry, WorkspaceStubModuleIndexProvider,
    };
    use crate::resolved_ast::FullyQualifiedName;
    use crate::symbols::{SymbolCollector, SymbolDeclarationProjection};
    use mizar_artifact::module_summary::{
        ExportedLabelSummary, ExportedSymbolSummary, LexicalContributionSummary,
        ModuleLexicalSummary, ProofStatusSummary, SourceRangeSummary, current_schema_version,
        module_summary_json,
    };
    use mizar_artifact::store::CanonicalJson;
    use mizar_build::module_index::PackageIndexSource;
    use mizar_session::{
        BuildSnapshotId, Edition, InMemorySessionIdAllocator, SessionIdAllocator, SourceRange,
    };
    use mizar_syntax::{
        SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceNodeKind, SurfaceTokenKind,
    };
    use semver::Version;

    #[test]
    fn summary_backed_projection_matches_source_backed_exports() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(1);
        let anchor = SourceAnchor::Range(range(source_id, 0, 1));
        let summary = sample_summary();
        let json = module_summary_json(&summary).expect("canonical summary json");
        let result = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(&indexed_module, anchor.clone()),
            &json,
        );

        assert!(result.reused_summary());
        assert!(result.diagnostics().is_empty());
        let summary_env = result.env().expect("summary-backed env");
        let source_env = source_backed_export_env(source_id, anchor, &summary);
        assert_summary_contribution(summary_env);

        assert_eq!(
            exported_symbols(summary_env),
            exported_symbols(&source_env),
            "summary-backed exported symbols should match source-backed facts"
        );
        assert_eq!(exported_labels(summary_env), exported_labels(&source_env));
        assert_eq!(
            lexical_summaries(summary_env),
            lexical_summaries(&source_env)
        );
        assert_eq!(module_summaries(summary_env), module_summaries(&source_env));
        assert_eq!(
            declaration_dependencies(summary_env),
            declaration_dependencies(&source_env)
        );
        assert!(summary_env.declaration_dependencies().iter().all(
            |entry| matches!(entry.anchor(), SourceAnchor::Generated(origin)
                    if origin.reason().starts_with("module-summary-"))
        ));
    }

    #[test]
    fn summary_backed_symbol_surface_matches_source_collector() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(2);
        let anchor = SourceAnchor::Range(range(source_id, 0, 1));
        let source_env = source_collector_env(source_id);
        let summary = summary_from_source_env(&source_env);
        let json = module_summary_json(&summary).expect("canonical summary json");
        let summary_result = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider))
            .read_and_project(
                ModuleSummaryReuseRequest::new(&indexed_module, anchor),
                &json,
            );
        let summary_symbol = summary_result
            .env()
            .unwrap()
            .symbols()
            .iter()
            .next()
            .expect("summary symbol");

        let source_symbol = source_env.symbols().iter().next().expect("source symbol");

        assert_eq!(summary_symbol.kind(), source_symbol.kind());
        assert_eq!(summary_symbol.visibility(), source_symbol.visibility());
        assert_eq!(
            summary_symbol.export_status(),
            source_symbol.export_status()
        );
        assert_eq!(summary_symbol.symbol().fqn(), source_symbol.symbol().fqn());
        assert_eq!(summary_symbol.signature(), source_symbol.signature());
    }

    #[test]
    fn valid_summary_consumption_is_deterministic() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(3);
        let anchor = SourceAnchor::Range(range(source_id, 0, 1));
        let json = module_summary_json(&sample_summary()).expect("canonical summary json");

        let first = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(&indexed_module, anchor.clone()),
            &json,
        );
        let second = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(&indexed_module, anchor),
            &json,
        );

        assert_eq!(first, second);
        assert_eq!(
            first.env().unwrap().snapshot_text(),
            second.env().unwrap().snapshot_text()
        );
        assert!(first.diagnostics().is_empty());
    }

    #[test]
    fn lockfile_identity_is_accepted_when_known_identity_fields_match() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(4);
        let mut summary = sample_summary();
        summary.module.lockfile_identity = Some("lock:fixture".to_owned());
        summary.refresh_interface_hash().expect("refresh hash");
        let json = module_summary_json(&summary).expect("canonical summary json");

        let result = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(
                &indexed_module,
                SourceAnchor::Range(range(source_id, 0, 1)),
            ),
            &json,
        );

        assert!(result.reused_summary());
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn incompatible_schema_falls_back_with_deterministic_record() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(5);
        let mut json = module_summary_json(&sample_summary()).expect("canonical summary json");
        let CanonicalJson::Object(fields) = &mut json else {
            panic!("summary json should be object");
        };
        fields.insert("schema_version".to_owned(), CanonicalJson::string("99.0"));

        let first = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(
                &indexed_module,
                SourceAnchor::Range(range(source_id, 0, 1)),
            ),
            &json,
        );
        let second = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(
                &indexed_module,
                SourceAnchor::Range(range(source_id, 0, 1)),
            ),
            &json,
        );

        assert!(!first.reused_summary());
        assert_eq!(first, second);
        assert_eq!(
            first.diagnostics()[0].reason(),
            ModuleSummaryReuseReason::ArtifactRejected
        );
        assert_eq!(
            first.diagnostics()[0].artifact(),
            Some("dep/core.summary.json")
        );
    }

    #[test]
    fn identity_and_expected_hash_mismatch_fall_back() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(6);
        let mut summary = sample_summary();
        summary.module.module_path = "other".to_owned();
        summary.refresh_interface_hash().expect("refresh hash");
        let identity_mismatch =
            module_summary_json(&summary).expect("canonical identity-mismatch summary json");

        let identity_result = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider))
            .read_and_project(
                ModuleSummaryReuseRequest::new(
                    &indexed_module,
                    SourceAnchor::Range(range(source_id, 0, 1)),
                ),
                &identity_mismatch,
            );
        assert!(!identity_result.reused_summary());
        assert_eq!(
            identity_result.diagnostics()[0].reason(),
            ModuleSummaryReuseReason::ArtifactRejected
        );

        let hash_mismatch = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider))
            .read_and_project(
                ModuleSummaryReuseRequest::new(
                    &indexed_module,
                    SourceAnchor::Range(range(source_id, 0, 1)),
                )
                .with_expected_interface_hash(Hash::from_bytes([99; Hash::BYTE_LEN])),
                &module_summary_json(&sample_summary()).expect("canonical summary json"),
            );
        assert!(!hash_mismatch.reused_summary());
        assert_eq!(
            hash_mismatch.diagnostics()[0].reason(),
            ModuleSummaryReuseReason::ArtifactRejected
        );

        let validated_mismatch = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider))
            .project_validated(
                ModuleSummaryReuseRequest::new(
                    &indexed_module,
                    SourceAnchor::Range(range(source_id, 0, 1)),
                )
                .with_expected_interface_hash(Hash::from_bytes([98; Hash::BYTE_LEN])),
                &sample_summary(),
            );
        assert!(!validated_mismatch.reused_summary());
        assert_eq!(
            validated_mismatch.diagnostics()[0].reason(),
            ModuleSummaryReuseReason::ArtifactRejected
        );
    }

    #[test]
    fn unknown_symbol_visibility_fails_closed() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(7);
        let mut summary = sample_summary();
        summary.exported_symbols[0].visibility = "surprisingly_visible".to_owned();
        summary.refresh_interface_hash().expect("refresh hash");

        let result = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(
                &indexed_module,
                SourceAnchor::Range(range(source_id, 0, 1)),
            ),
            &module_summary_json(&summary).expect("canonical summary json"),
        );

        assert!(!result.reused_summary());
        assert_eq!(
            result.diagnostics()[0].reason(),
            ModuleSummaryReuseReason::UnsupportedProjection
        );
        assert!(
            result.diagnostics()[0]
                .detail()
                .contains("unsupported exported symbol")
        );
    }

    #[test]
    fn unknown_label_visibility_and_target_kind_fail_closed() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(8);

        let mut unknown_visibility = sample_summary();
        unknown_visibility.exported_labels[0].visibility = "ambient".to_owned();
        unknown_visibility
            .refresh_interface_hash()
            .expect("refresh hash");
        let visibility_result = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider))
            .read_and_project(
                ModuleSummaryReuseRequest::new(
                    &indexed_module,
                    SourceAnchor::Range(range(source_id, 0, 1)),
                ),
                &module_summary_json(&unknown_visibility).expect("canonical summary json"),
            );
        assert!(!visibility_result.reused_summary());
        assert_eq!(
            visibility_result.diagnostics()[0].reason(),
            ModuleSummaryReuseReason::UnsupportedProjection
        );
        assert!(
            visibility_result.diagnostics()[0]
                .detail()
                .contains("unsupported exported label")
        );

        let mut unknown_target = sample_summary();
        unknown_target.exported_labels[0].target_kind = "scheme".to_owned();
        unknown_target
            .refresh_interface_hash()
            .expect("refresh hash");
        let target_result = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider))
            .read_and_project(
                ModuleSummaryReuseRequest::new(
                    &indexed_module,
                    SourceAnchor::Range(range(source_id, 0, 1)),
                ),
                &module_summary_json(&unknown_target).expect("canonical summary json"),
            );
        assert!(!target_result.reused_summary());
        assert_eq!(
            target_result.diagnostics()[0].reason(),
            ModuleSummaryReuseReason::UnsupportedProjection
        );
        assert!(
            target_result.diagnostics()[0]
                .detail()
                .contains("unsupported exported label")
        );
    }

    #[test]
    fn known_private_visibility_remains_local_only() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(9);
        let mut summary = sample_summary();
        summary.exported_symbols[0].visibility = "private".to_owned();
        summary.exported_labels[0].visibility = "private".to_owned();
        summary.refresh_interface_hash().expect("refresh hash");

        let result = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(
                &indexed_module,
                SourceAnchor::Range(range(source_id, 0, 1)),
            ),
            &module_summary_json(&summary).expect("canonical summary json"),
        );

        assert!(result.reused_summary());
        let env = result.env().expect("summary-backed env");
        let symbol = env.symbols().iter().next().expect("symbol entry");
        assert_eq!(symbol.visibility(), Visibility::Private);
        assert_eq!(symbol.export_status(), ExportStatus::LocalOnly);
        let label = env.labels().iter().next().expect("label entry");
        assert_eq!(label.visibility(), Visibility::Private);
        assert_eq!(label.export_status(), ExportStatus::LocalOnly);
        assert!(env.lexical_summaries().is_empty());
    }

    #[test]
    fn unpaired_lexical_contribution_is_deterministic_fallback_record() {
        let (provider, indexed_module) = provider_fixture();
        let source_id = source_id(10);
        let mut summary = sample_summary();
        summary.lexical_summary.contributions[0].key = "dep::core::Missing".to_owned();
        summary.refresh_interface_hash().expect("refresh hash");
        let json = module_summary_json(&summary).expect("canonical summary json");

        let first = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(
                &indexed_module,
                SourceAnchor::Range(range(source_id, 0, 1)),
            ),
            &json,
        );
        let second = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(
                &indexed_module,
                SourceAnchor::Range(range(source_id, 0, 1)),
            ),
            &json,
        );

        assert!(first.reused_summary());
        assert_eq!(first, second);
        assert_eq!(
            first.diagnostics()[0].reason(),
            ModuleSummaryReuseReason::UnpairedLexicalContribution
        );
        assert_eq!(
            first.diagnostics()[0].artifact(),
            Some("dep/core.summary.json")
        );
        assert_eq!(first.env().unwrap().lexical_summaries().len(), 0);
    }

    #[test]
    fn missing_dependency_summary_does_not_source_load() {
        let package = PackageId::new("dep");
        let indexed_module = IndexedModuleId::new(package.clone(), ModulePath::new("core"));
        let provider = WorkspaceStubModuleIndexProvider::new(
            vec![package_entry(package.clone())],
            Vec::new(),
            vec![ModuleIndexEntry {
                module: indexed_module.clone(),
                package_id: package,
                module_path: ModulePath::new("core"),
                location: ModuleIndexLocation::WorkspaceFile {
                    source_root: "src".to_owned(),
                    normalized_path: "src/core.miz".to_owned(),
                    source_relative_path: "core.miz".to_owned(),
                },
                edition: Edition::new("2026"),
            }],
            Vec::new(),
        );
        let source_id = source_id(11);

        let result = ModuleSummaryReuse::new(ModuleIndexInput::new(&provider)).read_and_project(
            ModuleSummaryReuseRequest::new(
                &indexed_module,
                SourceAnchor::Range(range(source_id, 0, 1)),
            ),
            &module_summary_json(&sample_summary()).expect("canonical summary json"),
        );

        assert!(!result.reused_summary());
        assert_eq!(
            result.diagnostics()[0].reason(),
            ModuleSummaryReuseReason::SummaryUnavailable
        );
    }

    fn provider_fixture() -> (WorkspaceStubModuleIndexProvider, IndexedModuleId) {
        let package = PackageId::new("dep");
        let module = IndexedModuleId::new(package.clone(), ModulePath::new("core"));
        let summary_hash = Hash::from_bytes([41; Hash::BYTE_LEN]);
        let provider = WorkspaceStubModuleIndexProvider::new(
            vec![package_entry(package.clone())],
            vec![crate::module_index::NamespaceIndexEntry {
                root: NamespaceRoot::PackageName,
                prefix: vec!["dep".to_owned()],
                package_id: package.clone(),
            }],
            vec![ModuleIndexEntry {
                module: module.clone(),
                package_id: package.clone(),
                module_path: ModulePath::new("core"),
                location: ModuleIndexLocation::DependencySummary {
                    artifact: "dep/core.summary.json".to_owned(),
                    content_hash: summary_hash,
                },
                edition: Edition::new("2026"),
            }],
            vec![DependencyModuleSummaryRef {
                module: module.clone(),
                artifact: "dep/core.summary.json".to_owned(),
                content_hash: summary_hash,
            }],
        );
        (provider, module)
    }

    fn package_entry(package_id: PackageId) -> PackageIndexEntry {
        PackageIndexEntry {
            package_id,
            version: Version::new(1, 2, 3),
            edition: Edition::new("2026"),
            source: PackageIndexSource::RegistryArtifact {
                registry: "registry.example".to_owned(),
                checksum: "checksum".to_owned(),
            },
            dependencies: Vec::new(),
        }
    }

    fn sample_summary() -> ModuleSummary {
        let mut summary = ModuleSummary {
            schema_version: current_schema_version(),
            module: ModuleSummaryIdentity {
                package_id: "dep".to_owned(),
                package_version: Some("1.2.3".to_owned()),
                lockfile_identity: None,
                module_path: "core".to_owned(),
                language_edition: "2026".to_owned(),
            },
            source_hash: Hash::from_bytes([1; Hash::BYTE_LEN]),
            interface_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
            exported_symbols: vec![ExportedSymbolSummary {
                origin_id: "thm:T1".to_owned(),
                fully_qualified_name: "dep::core::T1".to_owned(),
                namespace_path: vec!["core".to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "theorem".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 0,
                    end_byte: 10,
                },
                rendered_signature: "theorem T1: x = x".to_owned(),
                interface_fingerprint: Hash::from_bytes([2; Hash::BYTE_LEN]),
                proof_status: Some(ProofStatusSummary::Accepted),
            }],
            exported_labels: vec![ExportedLabelSummary {
                origin_id: "label:T1".to_owned(),
                label: "T1".to_owned(),
                owner_fully_qualified_name: "dep::core::T1".to_owned(),
                visibility: "public".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 0,
                    end_byte: 2,
                },
                target_kind: "theorem".to_owned(),
            }],
            lexical_summary: ModuleLexicalSummary {
                schema_version: "resolver-lexical-summary-v1".to_owned(),
                fingerprint: None,
                contributions: vec![LexicalContributionSummary {
                    kind: "notation".to_owned(),
                    key: "dep::core::T1".to_owned(),
                    payload: "T1".to_owned(),
                }],
            },
            reexports: vec![ModuleReexportSummary {
                target_module: ModuleSummaryIdentity {
                    package_id: "base".to_owned(),
                    package_version: Some("0.1.0".to_owned()),
                    lockfile_identity: None,
                    module_path: "prelude".to_owned(),
                    language_edition: "2026".to_owned(),
                },
                target_item_origin_id: Some("thm:Base".to_owned()),
                exported_name: Some("Base".to_owned()),
                provenance_origin_id: Some("reexport:base".to_owned()),
            }],
            dependency_interfaces: vec![DependencyInterfaceRef {
                module: ModuleSummaryIdentity {
                    package_id: "base".to_owned(),
                    package_version: Some("0.1.0".to_owned()),
                    lockfile_identity: None,
                    module_path: "prelude".to_owned(),
                    language_edition: "2026".to_owned(),
                },
                interface_hash: Hash::from_bytes([3; Hash::BYTE_LEN]),
            }],
        };
        summary.refresh_interface_hash().expect("interface hash");
        summary
    }

    fn source_backed_export_env(
        source_id: SourceId,
        anchor: SourceAnchor,
        summary: &ModuleSummary,
    ) -> SymbolEnv {
        let module = ModuleId::new(PackageId::new("dep"), ModulePath::new("core"));
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::ImportedSource { source_id },
            anchor.clone(),
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new("summary:thm:T1"),
            FullyQualifiedName::new("dep::core::T1"),
        );
        let symbol_entry = SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Theorem,
            NamespacePath::new("core"),
            "T1",
            SemanticOrigin::new(source_id, module.clone(), anchor.clone(), vec![0, 0]),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
        .with_signature(SignatureShell::Opaque {
            schema: "mizar-artifact/module-summary-symbol-v1".to_owned(),
            payload: "kind=theorem;signature=theorem T1: x = x;fingerprint=0202020202020202020202020202020202020202020202020202020202020202".to_owned(),
        });
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        indexes.symbols.insert(symbol_entry);
        let label = LabelEntry::new(
            LabelOriginPath::new("summary:dep::core::T1:label:T1"),
            LabelKind::Theorem,
            NamespacePath::new("core"),
            "T1",
            SemanticOrigin::new(source_id, module.clone(), anchor.clone(), vec![1, 0]),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported);
        indexes
            .contributions
            .add_label(contribution, label.origin_path().clone());
        indexes.labels.insert(label);
        let lexical = indexes.lexical_summaries.insert(
            symbol,
            NamespacePath::new("core"),
            "dep::core::T1",
            LexicalSummaryKind::Notation,
            None,
            contribution,
        );
        indexes
            .contributions
            .add_lexical_summary(contribution, lexical);
        indexes.module_summaries.insert(
            module.clone(),
            resolver_summary_identity(&summary.module, summary.interface_hash),
            contribution,
        );
        let dependency = &summary.dependency_interfaces[0];
        let dependency_module = module_id_from_summary_identity(&dependency.module);
        indexes.module_summaries.insert(
            dependency_module.clone(),
            resolver_summary_identity(&dependency.module, dependency.interface_hash),
            contribution,
        );
        let edge = indexes.declaration_dependencies.insert(
            DependencyEndpoint::Module(module.clone()),
            DependencyEndpoint::Module(dependency_module),
            DeclarationDependencyKind::Import,
            generated_summary_anchor(&anchor, "module-summary-dependency-interface"),
            contribution,
        );
        indexes
            .contributions
            .add_declaration_dependency(contribution, edge);
        let reexport_module = module_id_from_summary_identity(&summary.reexports[0].target_module);
        let reexport_edge = indexes.declaration_dependencies.insert(
            DependencyEndpoint::Module(module.clone()),
            DependencyEndpoint::Module(reexport_module),
            DeclarationDependencyKind::ReExport,
            generated_summary_anchor(&anchor, "module-summary-reexport"),
            contribution,
        );
        indexes
            .contributions
            .add_declaration_dependency(contribution, reexport_edge);
        SymbolEnv::new(module, indexes)
    }

    fn source_collector_env(source_id: SourceId) -> SymbolEnv {
        let module = ModuleId::new(PackageId::new("dep"), ModulePath::new("core"));
        let mut builder = SurfaceAstBuilder::new(source_id);
        let theorem = node(
            &mut builder,
            SurfaceNodeKind::TheoremItem,
            source_id,
            0,
            10,
            Vec::new(),
        );
        let root = finish_module(&mut builder, source_id, vec![theorem]);
        let ast = builder.finish(Some(root), None);
        let shells = DeclarationShellCollector::new(&ast, &module).collect();
        let projection = SymbolDeclarationProjection::new(
            shells.declarations()[0].id(),
            NamespacePath::new("core"),
            "T1",
            SymbolKind::Theorem,
        )
        .with_definition_kind(DefinitionKind::Theorem)
        .with_signature(SignatureShell::Opaque {
            schema: "mizar-artifact/module-summary-symbol-v1".to_owned(),
            payload: "kind=theorem;signature=theorem T1: x = x;fingerprint=0202020202020202020202020202020202020202020202020202020202020202".to_owned(),
        });
        SymbolCollector::new(source_id, &module, &shells, &[projection])
            .collect()
            .env()
            .clone()
    }

    fn summary_from_source_env(source_env: &SymbolEnv) -> ModuleSummary {
        let source_symbol = source_env.symbols().iter().next().expect("source symbol");
        let signature_payload = match source_symbol.signature() {
            Some(SignatureShell::Opaque { payload, .. }) => payload.clone(),
            _ => "theorem T1: x = x".to_owned(),
        };
        let mut summary = ModuleSummary {
            schema_version: current_schema_version(),
            module: ModuleSummaryIdentity {
                package_id: "dep".to_owned(),
                package_version: Some("1.2.3".to_owned()),
                lockfile_identity: None,
                module_path: "core".to_owned(),
                language_edition: "2026".to_owned(),
            },
            source_hash: Hash::from_bytes([11; Hash::BYTE_LEN]),
            interface_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
            exported_symbols: vec![ExportedSymbolSummary {
                origin_id: source_symbol.symbol().local().as_str().to_owned(),
                fully_qualified_name: source_symbol.symbol().fqn().as_str().to_owned(),
                namespace_path: vec!["core".to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "theorem".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 0,
                    end_byte: 10,
                },
                rendered_signature: signature_payload
                    .strip_prefix("kind=theorem;signature=")
                    .and_then(|tail| tail.split(";fingerprint=").next())
                    .unwrap_or("theorem T1: x = x")
                    .to_owned(),
                interface_fingerprint: Hash::from_bytes([2; Hash::BYTE_LEN]),
                proof_status: Some(ProofStatusSummary::Accepted),
            }],
            exported_labels: Vec::new(),
            lexical_summary: ModuleLexicalSummary {
                schema_version: "resolver-lexical-summary-v1".to_owned(),
                fingerprint: None,
                contributions: Vec::new(),
            },
            reexports: Vec::new(),
            dependency_interfaces: Vec::new(),
        };
        summary.refresh_interface_hash().expect("interface hash");
        summary
    }

    fn exported_symbols(
        env: &SymbolEnv,
    ) -> Vec<(
        String,
        String,
        String,
        SymbolKind,
        Visibility,
        ExportStatus,
        String,
    )> {
        env.symbols()
            .iter()
            .filter(|entry| entry.export_status() != ExportStatus::LocalOnly)
            .map(|entry| {
                (
                    format!(
                        "{}::{}",
                        entry.symbol().module().package().as_str(),
                        entry.symbol().module().path().as_str()
                    ),
                    entry.symbol().local().as_str().to_owned(),
                    entry.symbol().fqn().as_str().to_owned(),
                    entry.kind(),
                    entry.visibility(),
                    entry.export_status(),
                    match entry.signature() {
                        Some(SignatureShell::Opaque { payload, .. }) => payload.clone(),
                        Some(SignatureShell::Pending) => "pending".to_owned(),
                        Some(SignatureShell::Malformed { class }) => class.clone(),
                        None => String::new(),
                    },
                )
            })
            .collect()
    }

    fn exported_labels(
        env: &SymbolEnv,
    ) -> Vec<(String, String, LabelKind, Visibility, ExportStatus)> {
        env.labels()
            .iter()
            .filter(|entry| entry.export_status() != ExportStatus::LocalOnly)
            .map(|entry| {
                (
                    entry.origin_path().as_str().to_owned(),
                    entry.primary_spelling().to_owned(),
                    entry.kind(),
                    entry.visibility(),
                    entry.export_status(),
                )
            })
            .collect()
    }

    fn lexical_summaries(env: &SymbolEnv) -> Vec<(String, LexicalSummaryKind, String)> {
        env.lexical_summaries()
            .iter()
            .map(|entry| {
                (
                    entry.spelling().to_owned(),
                    entry.kind(),
                    entry.symbol().fqn().as_str().to_owned(),
                )
            })
            .collect()
    }

    fn module_summaries(env: &SymbolEnv) -> Vec<(String, String)> {
        env.module_summaries()
            .iter()
            .map(|entry| {
                (
                    format!(
                        "{}::{}",
                        entry.module().package().as_str(),
                        entry.module().path().as_str()
                    ),
                    entry.identity().as_str().to_owned(),
                )
            })
            .collect()
    }

    fn assert_summary_contribution(env: &SymbolEnv) {
        let contributions = env.contributions().iter().collect::<Vec<_>>();
        assert_eq!(contributions.len(), 1);
        let contribution = contributions[0];
        let ContributionKind::Summary { identity } = contribution.kind() else {
            panic!("expected summary contribution");
        };
        assert!(identity.as_str().contains("mizar-artifact/module-summary"));
        assert_eq!(contribution.effects().symbols().len(), 1);
        assert_eq!(contribution.effects().labels().len(), 1);
        assert_eq!(contribution.effects().lexical_summaries().len(), 1);
        assert_eq!(contribution.effects().declaration_dependencies().len(), 2);
        assert!(contribution.effects().diagnostics().is_empty());
    }

    fn declaration_dependencies(
        env: &SymbolEnv,
    ) -> Vec<(DeclarationDependencyKind, String, String, String)> {
        env.declaration_dependencies()
            .iter()
            .map(|entry| {
                (
                    entry.kind(),
                    endpoint_key(entry.source()),
                    endpoint_key(entry.target()),
                    anchor_key(entry.anchor()),
                )
            })
            .collect()
    }

    fn endpoint_key(endpoint: &DependencyEndpoint) -> String {
        match endpoint {
            DependencyEndpoint::Module(module) => {
                format!(
                    "module:{}::{}",
                    module.package().as_str(),
                    module.path().as_str()
                )
            }
            DependencyEndpoint::Symbol(symbol) => {
                format!("symbol:{}", symbol.fqn().as_str())
            }
            DependencyEndpoint::Label(label) => {
                format!("label:{}", label.as_str())
            }
            DependencyEndpoint::Import(id) => format!("import:{}", id.index()),
            DependencyEndpoint::Export(id) => format!("export:{}", id.index()),
            DependencyEndpoint::NamespaceEdge(id) => format!("namespace-edge:{}", id.index()),
            DependencyEndpoint::UnresolvedName(id) => format!("unresolved-name:{}", id.index()),
            DependencyEndpoint::UnresolvedLabel(id) => {
                format!("unresolved-label:{}", id.index())
            }
        }
    }

    fn anchor_key(anchor: &SourceAnchor) -> String {
        match anchor {
            SourceAnchor::Generated(origin) => format!("generated:{}", origin.reason()),
            SourceAnchor::Range(_) => "range".to_owned(),
            SourceAnchor::Point { .. } => "point".to_owned(),
            _ => "unknown".to_owned(),
        }
    }

    fn finish_module(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        items: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let item_list = node(builder, SurfaceNodeKind::ItemList, source_id, 0, 200, items);
        let unit = node(
            builder,
            SurfaceNodeKind::CompilationUnit,
            source_id,
            0,
            200,
            vec![item_list],
        );
        node(
            builder,
            SurfaceNodeKind::Root,
            source_id,
            0,
            200,
            vec![unit],
        )
    }

    fn node(
        builder: &mut SurfaceAstBuilder,
        kind: SurfaceNodeKind,
        source_id: SourceId,
        start: usize,
        end: usize,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let token = builder.add_token(
            SurfaceTokenKind::Identifier,
            "T1",
            range(source_id, start, start.saturating_add(2).min(end)),
        );
        let mut children = children;
        if children.is_empty() {
            children.push(token);
        }
        builder.add_node(kind, range(source_id, start, end), children)
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
