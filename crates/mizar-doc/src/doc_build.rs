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

use std::{error::Error, fmt, path::PathBuf};

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
    let mut plans = Vec::new();
    for selected in &request.publications {
        let (manifest, artifacts) = crate::artifact_reader::read_published_artifacts_with_comments(
            &selected.artifact_root,
            selected.expected_manifest_hash,
        )
        .map_err(DocBuildError::Publication)?;
        let stored = &manifest.manifest;
        if stored.package != selected.package
            || !snapshot
                .source_versions
                .iter()
                .any(|source| source.package_id.as_str() == stored.package.package_id)
            || stored.toolchain != snapshot.toolchain.identity()
            || stored.lockfile_hash.digest != snapshot.lockfile_hash
            || stored.verifier_config_hash.digest != snapshot.verifier_config_hash
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
                || artifact.provenance.lockfile_hash.digest != snapshot.lockfile_hash
                || artifact.provenance.verifier_config_hash.digest != snapshot.verifier_config_hash
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
    if !registry.is_current_for_request(snapshot_id, build_request) {
        return Err(DocBuildError::Binding(
            "snapshot became stale during planning",
        ));
    }
    Ok(plans)
}
