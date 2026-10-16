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
