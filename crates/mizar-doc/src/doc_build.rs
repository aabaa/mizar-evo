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
