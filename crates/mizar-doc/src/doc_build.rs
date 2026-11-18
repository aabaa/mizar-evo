//! Raw published metadata for later documentation build and index planning.

use std::{collections::BTreeMap, path::Path};

use mizar_artifact::{
    manifest::{ManifestError, PublishedManifestRead},
    module_summary::ModuleSummaryIdentity,
    verified_artifact::VerifiedExport,
};
use mizar_session::Hash;

use crate::artifact_reader::read_published_artifacts;

/// Reads a selected publication's complete exports in module/FQN/origin order.
///
/// Preserves stored metadata without granting source freshness or proof acceptance.
pub fn read_published_export_index(
    artifact_root: impl AsRef<Path>,
    expected_manifest_hash: Hash,
) -> Result<
    (
        PublishedManifestRead,
        BTreeMap<ModuleSummaryIdentity, Vec<VerifiedExport>>,
    ),
    ManifestError,
> {
    let (manifest, modules) = read_published_artifacts(artifact_root, expected_manifest_hash)?;
    let index = modules
        .into_iter()
        .map(|mut module| {
            module.exports.sort_by(|a, b| {
                a.fully_qualified_name
                    .cmp(&b.fully_qualified_name)
                    .then_with(|| a.origin_id.cmp(&b.origin_id))
            });
            (module.module, module.exports)
        })
        .collect();
    Ok((manifest, index))
}

/// Reads complete optional summaries from a caller-selected validated publication.
pub fn read_published_module_summary_index(
    artifact_root: impl AsRef<Path>,
    expected_manifest_hash: Hash,
) -> Result<
    (
        PublishedManifestRead,
        BTreeMap<ModuleSummaryIdentity, Option<mizar_artifact::module_summary::ModuleSummary>>,
    ),
    ManifestError,
> {
    use mizar_artifact::{
        module_summary::{
            MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummaryReadOptions, read_module_summary,
        },
        store::{
            PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
            read_published_artifact,
        },
    };

    let root = artifact_root.as_ref();
    let (manifest, _) = read_published_artifacts(root, expected_manifest_hash)?;
    let mut index = BTreeMap::new();
    for entry in &manifest.manifest.modules {
        let summary = if let (Some(file), Some(hash), Some(interface_hash)) = (
            &entry.module_summary_file,
            &entry.module_summary_hash,
            &entry.module_summary_interface_hash,
        ) {
            let path = PublishedArtifactPath::new(file).map_err(ManifestError::StoreIo)?;
            let domain = artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, hash.schema_version);
            let read = read_published_artifact(
                root,
                &path,
                PublishedArtifactReadOptions {
                    artifact_hash_domain: Some(&domain),
                    hash_excluded_paths: &[],
                    expected_artifact_hash: Some(hash.digest),
                },
            )
            .map_err(ManifestError::StoreIo)?;
            Some(
                read_module_summary(
                    &read.value,
                    ModuleSummaryReadOptions {
                        artifact_path: Some(path.as_str()),
                        expected_module: Some(&entry.module),
                        expected_interface_hash: Some(interface_hash.digest),
                    },
                )
                .map_err(|error| ManifestError::ModuleSummary {
                    path: file.clone(),
                    error: Box::new(error),
                })?,
            )
        } else {
            None
        };
        index.insert(entry.module.clone(), summary);
    }
    Ok((manifest, index))
}

/// Reads all stored owner exports for an exact published label record.
pub fn read_published_label_export_candidates(
    artifact_root: impl AsRef<Path>,
    expected_manifest_hash: Hash,
    module: &ModuleSummaryIdentity,
    label: &mizar_artifact::module_summary::ExportedLabelSummary,
) -> Result<(PublishedManifestRead, Option<Vec<VerifiedExport>>), ManifestError> {
    let root = artifact_root.as_ref();
    let (manifest, mut exports) = read_published_export_index(root, expected_manifest_hash)?;
    let (_, summaries) = read_published_module_summary_index(root, expected_manifest_hash)?;
    let candidates = summaries
        .get(module)
        .and_then(Option::as_ref)
        .filter(|summary| summary.exported_labels.contains(label))
        .and_then(|_| exports.remove(module))
        .map(|exports| {
            exports
                .into_iter()
                .filter(|export| export.fully_qualified_name == label.owner_fully_qualified_name)
                .collect()
        });
    Ok((manifest, candidates))
}

use std::{error::Error, fmt, path::PathBuf, sync::Arc};

use mizar_artifact::{
    manifest::{ModuleArtifactEntry, PackageIdentity},
    module_summary::{ExportedLabelSummary, ModuleSummary},
    store::CanonicalJson,
};
use mizar_session::{
    BuildRequestId, BuildSnapshotId, SessionIdAllocator, SnapshotRegistry, WorkspaceRoot,
};

/// Independent phase-16 consumers and their warning policies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase16Profile {
    /// Whether documentation planning is enabled.
    pub render_docs: bool,
    /// Whether the original request is delegated to extraction.
    pub extract_code: bool,
    /// Whether downstream documentation warnings fail documentation.
    pub fail_on_doc_warnings: bool,
    /// Whether downstream extraction warnings fail extraction.
    pub fail_on_extraction_warnings: bool,
    /// Whether downstream rendering includes private documentation.
    pub include_private_docs: bool,
}

/// Explicit dependency documentation reference, without resolution authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyDocIndexRef {
    /// Complete owning package identity.
    pub package: PackageIdentity,
    /// Selected publication hash.
    pub publication_hash: Hash,
    /// Caller-supplied index location, not read during planning.
    pub path: PathBuf,
}

/// One explicitly selected publication and its downstream inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocPublicationRequest {
    /// Expected owning package, including version and lock identity.
    pub package: PackageIdentity,
    /// Selected publication root.
    pub artifact_root: PathBuf,
    /// Expected manifest publication hash.
    pub expected_manifest_hash: Hash,
    /// Downstream output destination, not written during planning.
    pub output_dir: PathBuf,
    /// Optional source bundle reference, not read during planning.
    pub source_bundle: Option<PathBuf>,
    /// Explicit dependency references; absence differs from an empty list.
    pub dependency_indexes: Option<Vec<DependencyDocIndexRef>>,
    /// Per-module semantic import order; absent entries and vectors stay absent.
    pub import_order: BTreeMap<ModuleSummaryIdentity, Option<Vec<ModuleSummaryIdentity>>>,
}

/// Consumer request retained by the caller for independent extraction delegation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocBuildRequest {
    /// Independent consumer choices and policies.
    pub profile: Phase16Profile,
    /// Selected publications, never discovered from the filesystem.
    pub publications: Vec<DocPublicationRequest>,
}

/// Complete stored module inputs for later documentation consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocModuleInput {
    /// Original manifest entry, including required source path and artifact hashes.
    pub artifact: ModuleArtifactEntry,
    /// Complete exports with required stored ranges in FQN/origin order.
    pub exports: Vec<VerifiedExport>,
    /// Complete optional summary; absence differs from an empty summary.
    pub summary: Option<ModuleSummary>,
    /// Validated raw comment projection; null is unavailable, including legacy profiles.
    pub comment_projection: CanonicalJson,
}

/// Unresolved package index inputs; no links, anchors or UsedBy are inferred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocIndex {
    /// Required package version copied from stored metadata.
    pub package_version: String,
    /// All modules, including empty modules, keyed by full stored identity.
    pub modules: BTreeMap<ModuleSummaryIdentity, DocModuleInput>,
}

impl DocIndex {
    /// Returns every exact-record owner candidate, preserving absence and ambiguity.
    pub fn label_export_candidates(
        &self,
        module: &ModuleSummaryIdentity,
        label: &ExportedLabelSummary,
    ) -> Option<Vec<&VerifiedExport>> {
        let input = self.modules.get(module)?;
        input
            .summary
            .as_ref()?
            .exported_labels
            .contains(label)
            .then(|| {
                input
                    .exports
                    .iter()
                    .filter(|export| {
                        export.fully_qualified_name == label.owner_fully_qualified_name
                    })
                    .collect()
            })
    }
}

/// One authenticated publication with its unresolved index and retained options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocBuildPlan {
    native_publication: Option<Arc<mizar_driver::events::SnapshotPublication>>,
    /// Original publication request with canonically ordered dependency references.
    pub request: DocPublicationRequest,
    /// Independent consumer choices and policies.
    pub profile: Phase16Profile,
    /// Identity of the registry-retained current snapshot used for binding.
    pub snapshot: BuildSnapshotId,
    /// Request whose current snapshot was authenticated.
    pub build_request: BuildRequestId,
    /// Complete validated manifest, path and publication hash.
    pub manifest: PublishedManifestRead,
    /// Unresolved index owning the sole copy of module exports.
    pub index: DocIndex,
}

/// Refusal to produce any plans, preserving existing publication error layers.
#[derive(Debug)]
pub enum DocBuildError {
    /// Existing publication integrity, schema, path or freshness refusal.
    Publication(ManifestError),
    /// Missing required metadata or a request/snapshot/publication mismatch.
    Binding(&'static str),
}

impl fmt::Display for DocBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Publication(error) => error.fmt(formatter),
            Self::Binding(reason) => formatter.write_str(reason),
        }
    }
}

impl Error for DocBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Publication(error) => Some(error),
            Self::Binding(_) => None,
        }
    }
}

/// Authentication coverage for a finite documentation input scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DocImportCoverage {
    /// No genuine producer view establishes the queried scope.
    Unavailable,
    /// Known inputs remain usable, but some required coverage is missing.
    Incomplete,
    /// The explicit scope is complete, including an authoritative empty scope.
    Complete,
}

/// Publication-authenticated inputs for later documentation link consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedDocImports {
    selected_plan: DocBuildPlan,
    publications: Vec<PublishedManifestRead>,
    // Reuse the validating readers' native indexes without another public row type.
    #[allow(clippy::type_complexity)]
    modules: BTreeMap<ModuleSummaryIdentity, (Hash, Vec<VerifiedExport>, Option<ModuleSummary>)>,
    producers: BTreeMap<ModuleSummaryIdentity, mizar_resolve::imports::PublishedImportView>,
    targets: BTreeMap<ModuleSummaryIdentity, Vec<Option<(ModuleSummaryIdentity, Hash)>>>,
    coverage: BTreeMap<ModuleSummaryIdentity, DocImportCoverage>,
}

/// Complete publication-authenticated identities for the declared finite inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedDocIdentities {
    identities: Vec<mizar_artifact::verified_artifact::PublishedIdentityProjection>,
}

impl AuthenticatedDocIdentities {
    /// Full module order; each producer retains complete FQN/origin export order.
    pub fn identities(&self) -> &[mizar_artifact::verified_artifact::PublishedIdentityProjection] {
        &self.identities
    }
}

/// Revalidates current inputs and requires one exact identity producer per module.
pub fn authenticate_doc_identities<A: SessionIdAllocator>(
    registry: &SnapshotRegistry<A>,
    workspace_root: &WorkspaceRoot,
    inputs: &AuthenticatedDocImports,
    inventory: &[(&PackageIdentity, Hash, &Path)],
    producers: &[&mizar_artifact::verified_artifact::PublishedIdentityProjection],
) -> Result<AuthenticatedDocIdentities, DocBuildError> {
    let imports = inputs
        .producers
        .iter()
        .map(|(module, view)| (module.clone(), inputs.modules[module].0, view))
        .collect::<Vec<_>>();
    let current = authenticate_doc_imports(
        registry,
        workspace_root,
        &inputs.selected_plan,
        inventory,
        &imports,
    )?;
    if current != *inputs {
        return Err(DocBuildError::Binding("identity inventory changed"));
    }
    let root = &inputs.selected_plan.request;
    let mut artifacts = BTreeMap::new();
    for (_, hash, path) in std::iter::once((
        &root.package,
        root.expected_manifest_hash,
        root.artifact_root.as_path(),
    ))
    .chain(inventory.iter().copied())
    {
        let (manifest, stored) =
            read_published_artifacts(path, hash).map_err(DocBuildError::Publication)?;
        if !inputs.publications.contains(&manifest) {
            return Err(DocBuildError::Binding("identity publication changed"));
        }
        for mut artifact in stored {
            artifact.exports.sort_by(|left, right| {
                left.fully_qualified_name
                    .cmp(&right.fully_qualified_name)
                    .then_with(|| left.origin_id.cmp(&right.origin_id))
            });
            artifacts.insert(artifact.module.clone(), artifact);
        }
    }
    let mut identities = BTreeMap::new();
    for producer in producers {
        let artifact = producer.artifact();
        let Some((publication, _, _)) = inputs.modules.get(&artifact.module) else {
            return Err(DocBuildError::Binding(
                "identity producer outside inventory",
            ));
        };
        if producer.publication_hash() != *publication
            || artifacts.get(&artifact.module) != Some(artifact)
            || identities
                .insert(artifact.module.clone(), (*producer).clone())
                .is_some()
        {
            return Err(DocBuildError::Binding("identity producer binding mismatch"));
        }
        if *publication == inputs.selected_plan.request.expected_manifest_hash
            && !registry
                .get(inputs.selected_plan.snapshot)
                .is_some_and(|snapshot| {
                    snapshot.source_versions.iter().any(|source| {
                        artifact.provenance.captured_source.as_ref()
                            == Some(&(source.source_id, source.origin.clone(), source.source_hash))
                            && source.package_id.as_str() == artifact.module.package_id
                            && source.module_path.as_str() == artifact.module.module_path
                            && source.edition.as_str() == artifact.module.language_edition
                            && source.normalized_path.as_str() == artifact.source_file
                    })
                })
        {
            return Err(DocBuildError::Binding("identity source snapshot mismatch"));
        }
    }
    if identities.len() != inputs.modules.len() {
        return Err(DocBuildError::Binding(
            "incomplete identity producer inventory",
        ));
    }
    if !snapshot_binding_is_current(
        registry,
        inputs.selected_plan.snapshot,
        inputs.selected_plan.build_request,
        inputs.selected_plan.native_publication.as_deref(),
    ) {
        return Err(DocBuildError::Binding(
            "snapshot became stale during identity authentication",
        ));
    }
    Ok(AuthenticatedDocIdentities {
        identities: identities.into_values().collect(),
    })
}

impl AuthenticatedDocImports {
    pub(crate) fn matches_plan(&self, plan: &DocBuildPlan) -> bool {
        self.selected_plan == *plan
    }

    pub(crate) fn inventory(&self) -> impl Iterator<Item = (&ModuleSummaryIdentity, Hash)> {
        self.modules.iter().map(|(module, row)| (module, row.0))
    }

    pub(crate) fn inventory_publications(&self) -> Vec<Hash> {
        let mut publications = self
            .publications
            .iter()
            .map(|publication| publication.artifact_hash)
            .collect::<Vec<_>>();
        publications.sort_by_key(|hash| hash.to_lower_hex());
        publications
    }

    /// Returns the validated selected publication, followed by the supplied inventory.
    pub fn publications(&self) -> &[PublishedManifestRead] {
        &self.publications
    }

    /// Returns exact stored exports for a full module/publication identity.
    pub fn exports(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
    ) -> Option<&[VerifiedExport]> {
        self.modules
            .get(module)
            .filter(|row| row.0 == publication)
            .map(|row| row.1.as_slice())
    }

    /// Returns a stored summary only for its authenticated publication.
    pub fn summary(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
    ) -> Option<&ModuleSummary> {
        self.modules
            .get(module)
            .filter(|row| row.0 == publication)?
            .2
            .as_ref()
    }

    /// Returns the genuine source view, retaining semantic order and unresolved occurrences.
    pub fn producer(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
    ) -> Option<&mizar_resolve::imports::PublishedImportView> {
        self.modules
            .get(module)
            .filter(|row| row.0 == publication)?;
        self.producers.get(module)
    }

    /// Pairs every resolved occurrence with its authenticated finite-inventory endpoint.
    pub fn bound_imports(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
    ) -> Option<
        impl Iterator<
            Item = (
                &mizar_resolve::imports::ResolvedImportCandidate,
                Option<(&ModuleSummaryIdentity, Hash)>,
            ),
        >,
    > {
        let view = self.producer(module, publication)?;
        Some(
            view.resolution()
                .resolved()
                .iter()
                .zip(self.targets.get(module)?)
                .map(|(binding, target)| {
                    (
                        binding,
                        target.as_ref().map(|(module, hash)| (module, *hash)),
                    )
                }),
        )
    }

    /// Returns coverage for exactly this source module and publication.
    pub fn coverage(&self, module: &ModuleSummaryIdentity, publication: Hash) -> DocImportCoverage {
        if self
            .modules
            .get(module)
            .is_none_or(|row| row.0 != publication)
        {
            return DocImportCoverage::Unavailable;
        }
        self.coverage
            .get(module)
            .copied()
            .unwrap_or(DocImportCoverage::Unavailable)
    }

    /// Returns completeness of all explicitly inventoried source import scopes.
    pub fn closure_coverage(&self) -> DocImportCoverage {
        if self
            .coverage
            .values()
            .all(|coverage| *coverage == DocImportCoverage::Unavailable)
        {
            DocImportCoverage::Unavailable
        } else if self
            .coverage
            .values()
            .all(|coverage| *coverage == DocImportCoverage::Complete)
        {
            DocImportCoverage::Complete
        } else {
            DocImportCoverage::Incomplete
        }
    }
}

/// Artifact projections authenticated against one finite publication inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedDocDirectBy {
    inventory_publications: Vec<Hash>,
    producers: Vec<mizar_artifact::direct_by::DirectByProjection>,
    coverage: BTreeMap<ModuleSummaryIdentity, (Hash, DocImportCoverage)>,
}

impl AuthenticatedDocDirectBy {
    pub(crate) fn matches_inputs(&self, inputs: &AuthenticatedDocImports) -> bool {
        authenticate_doc_direct_by(inputs, &self.producers.iter().collect::<Vec<_>>())
            .is_ok_and(|rebound| rebound == *self)
    }

    /// Returns original producer occurrences and examined body/proof scopes.
    pub fn producers(&self) -> &[mizar_artifact::direct_by::DirectByProjection] {
        &self.producers
    }

    /// Returns citation examination coverage for exactly this module/publication.
    pub fn coverage(&self, module: &ModuleSummaryIdentity, publication: Hash) -> DocImportCoverage {
        self.coverage
            .get(module)
            .filter(|row| row.0 == publication)
            .map_or(DocImportCoverage::Unavailable, |row| row.1)
    }

    /// Returns examination coverage of the supplied scope, independently of import coverage.
    pub fn citation_coverage(&self) -> DocImportCoverage {
        if self
            .coverage
            .values()
            .all(|row| row.1 == DocImportCoverage::Unavailable)
        {
            DocImportCoverage::Unavailable
        } else if self
            .coverage
            .values()
            .all(|row| row.1 == DocImportCoverage::Complete)
        {
            DocImportCoverage::Complete
        } else {
            DocImportCoverage::Incomplete
        }
    }
}

/// Binds original Artifact-owned citations to already authenticated finite inputs.
///
/// The caller supplies a current gate-1 handoff; this function neither reads source
/// nor discovers publications. Missing cited modules retain incomplete occurrences.
pub fn authenticate_doc_direct_by(
    inputs: &AuthenticatedDocImports,
    producers: &[&mizar_artifact::direct_by::DirectByProjection],
) -> Result<AuthenticatedDocDirectBy, DocBuildError> {
    use mizar_artifact::{
        direct_by::{DirectByEndpoint, DirectByTarget},
        verified_artifact::ExportVisibility,
    };
    let bound = |endpoint: &DirectByEndpoint| -> Result<bool, DocBuildError> {
        let Some((hash, exports, _)) = inputs.modules.get(&endpoint.module) else {
            return Ok(false);
        };
        if endpoint.publication_hash != *hash
            || endpoint.export.export_kind != "theorem"
            || !exports.contains(&endpoint.export)
        {
            return Err(DocBuildError::Binding(
                "direct-by endpoint binding mismatch",
            ));
        }
        Ok(true)
    };
    let mut result = AuthenticatedDocDirectBy {
        inventory_publications: inputs.inventory_publications(),
        producers: Vec::new(),
        coverage: inputs
            .modules
            .iter()
            .map(|(module, (hash, _, _))| (module.clone(), (*hash, DocImportCoverage::Unavailable)))
            .collect(),
    };
    for view in producers {
        let Some((hash, exports, _)) = inputs.modules.get(view.module()) else {
            return Err(DocBuildError::Binding("direct-by source outside inventory"));
        };
        let entry = inputs
            .publications
            .iter()
            .flat_map(|publication| &publication.manifest.modules)
            .find(|entry| &entry.module == view.module())
            .ok_or(DocBuildError::Binding("direct-by source missing"))?;
        if view.publication_hash() != *hash
            || view.source_hash() != entry.source_hash
            || inputs
                .producer(view.module(), *hash)
                .is_some_and(|producer| producer.source_id() != view.captured_source().0)
            || result
                .producers
                .iter()
                .any(|prior| prior.module() == view.module())
        {
            return Err(DocBuildError::Binding("direct-by source binding mismatch"));
        }
        let mut complete = view.complete();
        let mut examined = Vec::new();
        for examination in view.examinations() {
            complete &= examination.complete;
            let Some(owner) = &examination.owner else {
                complete = false;
                continue;
            };
            if owner.module != *view.module()
                || owner.publication_hash != *hash
                || !bound(owner)?
                || examined.contains(&&owner.export)
            {
                return Err(DocBuildError::Binding(
                    "direct-by examination binding mismatch",
                ));
            }
            examined.push(&owner.export);
        }
        complete &= exports
            .iter()
            .filter(|export| {
                export.export_kind == "theorem" && export.visibility == ExportVisibility::Public
            })
            .all(|export| examined.contains(&export));
        for occurrence in view.occurrences() {
            if let Some(owner) = &occurrence.citing {
                if owner.module != *view.module()
                    || owner.publication_hash != *hash
                    || !bound(owner)?
                    || !examined.contains(&&owner.export)
                {
                    return Err(DocBuildError::Binding("direct-by citing endpoint mismatch"));
                }
            } else {
                complete = false;
            }
            match &occurrence.target {
                DirectByTarget::Theorem(endpoint) => complete &= bound(endpoint)?,
                DirectByTarget::NonTheorem => {}
                _ => complete = false,
            }
        }
        result.coverage.insert(
            view.module().clone(),
            (
                *hash,
                if complete {
                    DocImportCoverage::Complete
                } else {
                    DocImportCoverage::Incomplete
                },
            ),
        );
        result.producers.push((*view).clone());
    }
    Ok(result)
}

/// Authenticates genuine producer imports against an explicit finite publication inventory.
///
/// Inventory paths are artifact roots, independent of raw Doc4 dependency-index paths.
/// Revalidates the selected root and its current snapshot before retaining any inputs.
pub fn authenticate_doc_imports<A: SessionIdAllocator>(
    registry: &SnapshotRegistry<A>,
    workspace_root: &WorkspaceRoot,
    root: &DocBuildPlan,
    inventory: &[(&PackageIdentity, Hash, &Path)],
    producers: &[(
        ModuleSummaryIdentity,
        Hash,
        &mizar_resolve::imports::PublishedImportView,
    )],
) -> Result<AuthenticatedDocImports, DocBuildError> {
    use std::collections::BTreeSet;
    let current = build_doc_plans_inner(
        registry,
        root.build_request,
        root.snapshot,
        workspace_root,
        &DocBuildRequest {
            profile: root.profile.clone(),
            publications: vec![root.request.clone()],
        },
        root.native_publication.clone(),
    )?;
    if current.as_slice() != [root.clone()] {
        return Err(DocBuildError::Binding(
            "selected documentation plan changed",
        ));
    }
    let mut result = AuthenticatedDocImports {
        selected_plan: root.clone(),
        publications: Vec::new(),
        modules: BTreeMap::new(),
        producers: BTreeMap::new(),
        targets: BTreeMap::new(),
        coverage: BTreeMap::new(),
    };
    for (package, hash, path) in std::iter::once((
        &root.request.package,
        root.request.expected_manifest_hash,
        root.request.artifact_root.as_path(),
    ))
    .chain(inventory.iter().copied())
    {
        let (manifest, mut exports) =
            read_published_export_index(path, hash).map_err(DocBuildError::Publication)?;
        let (summary_manifest, mut summaries) =
            read_published_module_summary_index(path, hash).map_err(DocBuildError::Publication)?;
        if manifest.manifest.package != *package || manifest != summary_manifest {
            return Err(DocBuildError::Binding(
                "publication package or index binding mismatch",
            ));
        }
        if let Some(previous) = result
            .publications
            .iter()
            .find(|previous| previous.artifact_hash == hash)
        {
            if previous != &manifest {
                return Err(DocBuildError::Binding("conflicting duplicate publication"));
            }
            continue;
        }
        for entry in &manifest.manifest.modules {
            let identity = &entry.module;
            if identity.package_id != package.package_id
                || identity.package_version != package.package_version
                || identity.lockfile_identity != package.lockfile_identity
                || identity.language_edition != manifest.manifest.language_edition
            {
                return Err(DocBuildError::Binding(
                    "inventory module package binding mismatch",
                ));
            }
            let summary = summaries.remove(identity).flatten();
            if summary
                .as_ref()
                .is_some_and(|summary| summary.source_hash != entry.source_hash)
            {
                return Err(DocBuildError::Binding(
                    "inventory summary source binding mismatch",
                ));
            }
            if result
                .modules
                .insert(
                    identity.clone(),
                    (
                        hash,
                        exports
                            .remove(identity)
                            .ok_or(DocBuildError::Binding("inventory export coverage mismatch"))?,
                        summary,
                    ),
                )
                .is_some()
            {
                return Err(DocBuildError::Binding("ambiguous module publications"));
            }
        }
        result.publications.push(manifest);
    }
    let mut incomplete = BTreeSet::new();
    for (module, hash, view) in producers {
        let Some((publication, _, summary)) = result.modules.get(module) else {
            return Err(DocBuildError::Binding("producer source outside inventory"));
        };
        let entry = result
            .publications
            .iter()
            .flat_map(|publication| &publication.manifest.modules)
            .find(|entry| &entry.module == module)
            .ok_or(DocBuildError::Binding("producer source missing"))?;
        if hash != publication
            || view.module().package().as_str() != module.package_id
            || view.module().path().as_str() != module.module_path
            || view.source_hash() != entry.source_hash
        {
            return Err(DocBuildError::Binding(
                "producer source publication binding mismatch",
            ));
        }
        if *hash == root.request.expected_manifest_hash
            && !registry.get(root.snapshot).is_some_and(|snapshot| {
                snapshot.source_versions.iter().any(|source| {
                    source.source_id == view.source_id()
                        && source.package_id.as_str() == module.package_id
                        && source.module_path.as_str() == module.module_path
                        && source.source_hash == view.source_hash()
                })
            })
        {
            return Err(DocBuildError::Binding("producer source identity mismatch"));
        }
        if view.visible_surfaces().is_some() {
            if let Some(summary) = summary {
                let mut declarations = summary.clone();
                declarations.reexports.clear();
                declarations.dependency_interfaces.clear();
                declarations.refresh_interface_hash().map_err(|_| {
                    DocBuildError::Binding("producer declaration summary binding mismatch")
                })?;
                if view.compose_summary(declarations).as_ref() != Some(summary) {
                    return Err(DocBuildError::Binding(
                        "producer summary composition mismatch",
                    ));
                }
            }
        } else {
            incomplete.insert(module.clone());
        }
        for captured in view.summaries() {
            if let Some((_, _, Some(stored))) = result.modules.get(&captured.module) {
                if captured != stored {
                    return Err(DocBuildError::Binding(
                        "producer dependency summary binding mismatch",
                    ));
                }
            } else {
                incomplete.insert(module.clone());
            }
        }
        let targets = view
            .resolution()
            .resolved()
            .iter()
            .map(|binding| {
                let captured = view.summaries().iter().find(|summary| {
                    summary.module.package_id == binding.target().package().as_str()
                        && summary.module.module_path == binding.target().path().as_str()
                });
                let target = captured.and_then(|summary| {
                    result
                        .modules
                        .get(&summary.module)
                        .map(|row| (summary.module.clone(), row.0))
                });
                if target.is_none() {
                    incomplete.insert(module.clone());
                }
                target
            })
            .collect();
        if let Some(previous) = result.producers.insert(module.clone(), (*view).clone())
            && previous != **view
        {
            return Err(DocBuildError::Binding("conflicting duplicate producer"));
        }
        result.targets.insert(module.clone(), targets);
    }
    for (module, (_, _, summary)) in &result.modules {
        if !result.producers.contains_key(module) {
            incomplete.insert(module.clone());
        }
        let Some(summary) = summary else {
            incomplete.insert(module.clone());
            continue;
        };
        for edge in &summary.dependency_interfaces {
            match result.modules.get(&edge.module) {
                Some((_, _, Some(target))) if target.interface_hash != edge.interface_hash => {
                    return Err(DocBuildError::Binding(
                        "dependency interface binding mismatch",
                    ));
                }
                Some((_, _, Some(_))) => {}
                _ => {
                    incomplete.insert(module.clone());
                }
            }
        }
        for edge in &summary.reexports {
            if !summary
                .dependency_interfaces
                .iter()
                .any(|dependency| dependency.module == edge.target_module)
            {
                return Err(DocBuildError::Binding(
                    "forwarding edge lacks dependency binding",
                ));
            }
        }
    }
    loop {
        let before = incomplete.len();
        for (module, (_, _, summary)) in &result.modules {
            if summary.as_ref().is_some_and(|summary| {
                summary
                    .dependency_interfaces
                    .iter()
                    .any(|edge| incomplete.contains(&edge.module))
            }) || result.targets.get(module).is_some_and(|targets| {
                targets
                    .iter()
                    .flatten()
                    .any(|(target, _)| incomplete.contains(target))
            }) {
                incomplete.insert(module.clone());
            }
        }
        if incomplete.len() == before {
            break;
        }
    }
    for module in result.modules.keys() {
        result.coverage.insert(
            module.clone(),
            if !result.producers.contains_key(module) {
                DocImportCoverage::Unavailable
            } else if incomplete.contains(module) {
                DocImportCoverage::Incomplete
            } else {
                DocImportCoverage::Complete
            },
        );
    }
    if !snapshot_binding_is_current(
        registry,
        root.snapshot,
        root.build_request,
        root.native_publication.as_deref(),
    ) {
        return Err(DocBuildError::Binding(
            "snapshot became stale during import authentication",
        ));
    }
    Ok(result)
}

/// Plans all selected publications atomically against a retained current snapshot.
///
/// Disabled documentation performs no registry or publication reads. Extraction
/// remains delegated through the original request, without lowering or publication.
pub fn build_doc_plans<A: SessionIdAllocator>(
    registry: &SnapshotRegistry<A>,
    build_request: BuildRequestId,
    snapshot_id: BuildSnapshotId,
    workspace_root: &WorkspaceRoot,
    request: &DocBuildRequest,
) -> Result<Vec<DocBuildPlan>, DocBuildError> {
    build_doc_plans_inner(
        registry,
        build_request,
        snapshot_id,
        workspace_root,
        request,
        None,
    )
}

/// Plans native selected roots using Driver's retained original capture and publication.
/// Registry currentness and all ordinary publication readers remain authoritative.
pub fn build_native_doc_plans<A: SessionIdAllocator>(
    registry: &SnapshotRegistry<A>,
    build_request: BuildRequestId,
    snapshot_id: BuildSnapshotId,
    workspace_root: &WorkspaceRoot,
    request: &DocBuildRequest,
    publication: Arc<mizar_driver::events::SnapshotPublication>,
) -> Result<Vec<DocBuildPlan>, DocBuildError> {
    build_doc_plans_inner(
        registry,
        build_request,
        snapshot_id,
        workspace_root,
        request,
        Some(publication),
    )
}

fn snapshot_binding_is_current<A: SessionIdAllocator>(
    registry: &SnapshotRegistry<A>,
    snapshot: BuildSnapshotId,
    request: BuildRequestId,
    native_publication: Option<&mizar_driver::events::SnapshotPublication>,
) -> bool {
    registry.is_current_for_request(snapshot, request)
        && native_publication.is_none_or(|publication| {
            registry.get(snapshot).as_ref() == Some(publication.captured().snapshot())
        })
}

fn build_doc_plans_inner<A: SessionIdAllocator>(
    registry: &SnapshotRegistry<A>,
    build_request: BuildRequestId,
    snapshot_id: BuildSnapshotId,
    workspace_root: &WorkspaceRoot,
    request: &DocBuildRequest,
    native_publication: Option<Arc<mizar_driver::events::SnapshotPublication>>,
) -> Result<Vec<DocBuildPlan>, DocBuildError> {
    if !request.profile.render_docs {
        return Ok(Vec::new());
    }
    let snapshot = registry
        .get(snapshot_id)
        .filter(|snapshot| {
            &snapshot.workspace_root == workspace_root
                && registry.is_current_for_request(snapshot_id, build_request)
        })
        .ok_or(DocBuildError::Binding(
            "snapshot is not current for request/workspace",
        ))?;
    use mizar_artifact::registration_summary::ArtifactHashRef;
    let native_hashes = if let Some(publication) = &native_publication {
        if publication.build_request() != build_request
            || publication.captured().snapshot() != &snapshot
            || publication.manifests().len() != publication.artifact_roots().len()
            || publication.manifests().len() != publication.artifacts().len()
            || publication.artifacts().len() != publication.source_metadata().len()
        {
            return Err(DocBuildError::Binding("native capture context mismatch"));
        }
        Some((
            ArtifactHashRef::source_owner_fingerprint(
                publication.lockfile_producer(),
                snapshot.lockfile_hash,
            )
            .ok_or(DocBuildError::Binding("unsupported native lock producer"))?,
            ArtifactHashRef::source_owner_fingerprint(
                "mizar-build/package-config-content/v1",
                snapshot.verifier_config_hash,
            )
            .ok_or(DocBuildError::Binding("unsupported native config producer"))?,
        ))
    } else {
        None
    };
    let hashes_match = |lock: &ArtifactHashRef, config: &ArtifactHashRef| {
        native_hashes.as_ref().map_or_else(
            || {
                lock.digest == snapshot.lockfile_hash
                    && config.digest == snapshot.verifier_config_hash
            },
            |(expected_lock, expected_config)| lock == expected_lock && config == expected_config,
        )
    };
    let mut plans = Vec::new();
    for selected in &request.publications {
        let (manifest, artifacts) = crate::artifact_reader::read_published_artifacts_with_comments(
            &selected.artifact_root,
            selected.expected_manifest_hash,
        )
        .map_err(DocBuildError::Publication)?;
        if let Some(publication) = &native_publication {
            let original = publication
                .artifact_roots()
                .iter()
                .zip(publication.manifests())
                .zip(publication.artifacts())
                .find(|((root, original), _)| {
                    *root == &selected.artifact_root && *original == &manifest
                });
            if !original
                .is_some_and(|(_, original)| artifacts.len() == 1 && artifacts[0].0 == **original)
            {
                return Err(DocBuildError::Binding(
                    "native original publication changed",
                ));
            }
        }
        let stored = &manifest.manifest;
        if stored.package != selected.package
            || !snapshot
                .source_versions
                .iter()
                .any(|source| source.package_id.as_str() == stored.package.package_id)
            || stored.toolchain != snapshot.toolchain.identity()
            || !hashes_match(&stored.lockfile_hash, &stored.verifier_config_hash)
        {
            return Err(DocBuildError::Binding(
                "package or manifest build provenance mismatch",
            ));
        }
        let package_version = stored
            .package
            .package_version
            .clone()
            .ok_or(DocBuildError::Binding("package version is required"))?;
        let (_, mut summaries) = read_published_module_summary_index(
            &selected.artifact_root,
            selected.expected_manifest_hash,
        )
        .map_err(DocBuildError::Publication)?;
        let mut modules = BTreeMap::new();
        for (entry, (mut artifact, comment_projection)) in stored.modules.iter().zip(artifacts) {
            let identity = &entry.module;
            if identity.package_id != stored.package.package_id
                || identity.package_version.as_ref() != Some(&package_version)
                || identity.lockfile_identity != stored.package.lockfile_identity
                || identity.language_edition != stored.language_edition
                || entry.source_file.is_empty()
                || !snapshot.source_versions.iter().any(|source| {
                    source.package_id.as_str() == identity.package_id
                        && source.module_path.as_str() == identity.module_path
                        && source.normalized_path.as_str() == entry.source_file
                        && source.source_hash == entry.source_hash
                        && source.edition.as_str() == identity.language_edition
                })
                || artifact.provenance.toolchain != snapshot.toolchain.identity()
                || !hashes_match(
                    &artifact.provenance.lockfile_hash,
                    &artifact.provenance.verifier_config_hash,
                )
                || artifact.provenance.language_edition != identity.language_edition
            {
                return Err(DocBuildError::Binding(
                    "module source, package or build provenance mismatch",
                ));
            }
            artifact.exports.sort_by(|a, b| {
                a.fully_qualified_name
                    .cmp(&b.fully_qualified_name)
                    .then_with(|| a.origin_id.cmp(&b.origin_id))
            });
            modules.insert(
                identity.clone(),
                DocModuleInput {
                    artifact: entry.clone(),
                    exports: artifact.exports,
                    summary: summaries.remove(identity).flatten(),
                    comment_projection,
                },
            );
        }
        let mut selected = selected.clone();
        if let Some(dependencies) = &mut selected.dependency_indexes {
            dependencies.sort_by(|a, b| {
                (
                    &a.package.package_id,
                    &a.package.package_version,
                    &a.package.lockfile_identity,
                    a.publication_hash.as_bytes(),
                    &a.path,
                )
                    .cmp(&(
                        &b.package.package_id,
                        &b.package.package_version,
                        &b.package.lockfile_identity,
                        b.publication_hash.as_bytes(),
                        &b.path,
                    ))
            });
        }
        plans.push(DocBuildPlan {
            native_publication: native_publication.clone(),
            request: selected,
            profile: request.profile.clone(),
            snapshot: snapshot_id,
            build_request,
            manifest,
            index: DocIndex {
                package_version,
                modules,
            },
        });
    }
    plans.sort_by(|a, b| {
        let a = &a.request;
        let b = &b.request;
        (
            &a.package.package_id,
            &a.package.package_version,
            &a.package.lockfile_identity,
            a.expected_manifest_hash.as_bytes(),
        )
            .cmp(&(
                &b.package.package_id,
                &b.package.package_version,
                &b.package.lockfile_identity,
                b.expected_manifest_hash.as_bytes(),
            ))
    });
    if plans.windows(2).any(|pair| {
        pair[0].request.package == pair[1].request.package
            && pair[0].request.expected_manifest_hash == pair[1].request.expected_manifest_hash
    }) {
        return Err(DocBuildError::Binding(
            "duplicate selected package publication",
        ));
    }
    if !snapshot_binding_is_current(
        registry,
        snapshot_id,
        build_request,
        native_publication.as_deref(),
    ) {
        return Err(DocBuildError::Binding(
            "snapshot became stale during planning",
        ));
    }
    Ok(plans)
}
