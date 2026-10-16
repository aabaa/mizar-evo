//! Read-only published inputs described in
//! [artifact_reader.md](../../../../doc/design/mizar-doc/en/artifact_reader.md).

use std::path::Path;

use mizar_artifact::{
    manifest::{ManifestError, ManifestFileReadOptions, PublishedManifestRead, read_manifest_file},
    store::{
        PublishedArtifactPath, PublishedArtifactReadOptions, SchemaVersion, artifact_hash_domain,
        read_published_artifact,
    },
    verified_artifact::{
        VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifact, VerifiedArtifactReadOptions,
        artifact_hash_excluded_paths, read_builtin_verified_artifact, read_verified_artifact,
    },
};
use mizar_session::Hash;

/// Loads one caller-selected publication, preserving its typed module data and order.
///
/// Validates published references and witness reachability without granting current
/// workspace freshness, proof acceptance, or payload-to-accepted-result authority.
pub fn read_published_artifacts(
    artifact_root: impl AsRef<Path>,
    expected_manifest_hash: Hash,
) -> Result<(PublishedManifestRead, Vec<VerifiedArtifact>), ManifestError> {
    let root = artifact_root.as_ref();
    let manifest = read_manifest_file(
        root,
        ManifestFileReadOptions {
            validate_references: true,
            validate_witness_files: true,
            validate_development_artifacts: false,
            expected_manifest_hash: Some(expected_manifest_hash),
        },
    )?;
    let mut modules = Vec::with_capacity(manifest.manifest.modules.len());
    let exclusions = artifact_hash_excluded_paths();
    for entry in &manifest.manifest.modules {
        let path =
            PublishedArtifactPath::new(&entry.artifact_file).map_err(ManifestError::StoreIo)?;
        let domain = artifact_hash_domain(
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            entry.artifact_hash.schema_version,
        );
        let read = read_published_artifact(
            root,
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &exclusions,
                expected_artifact_hash: Some(entry.artifact_hash.digest),
            },
        )
        .map_err(ManifestError::StoreIo)?;
        let reader = if entry.artifact_hash.schema_version == SchemaVersion::new(2, 0) {
            read_builtin_verified_artifact
        } else {
            read_verified_artifact
        };
        let artifact = reader(
            &read.value,
            VerifiedArtifactReadOptions {
                artifact_path: Some(path.as_str()),
                expected_module: Some(&entry.module),
                expected_interface_hash: Some(entry.interface_hash.digest),
                expected_implementation_hash: Some(entry.implementation_hash.digest),
            },
        )
        .map_err(|error| ManifestError::VerifiedArtifact {
            path: entry.artifact_file.clone(),
            error: Box::new(error),
        })?;
        modules.push(artifact);
    }
    Ok((manifest, modules))
}
