//! Read-only published inputs described in
//! [artifact_reader.md](../../../../doc/design/mizar-doc/en/artifact_reader.md).

use std::{fs, io, path::Path};

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
use mizar_lexer::{PreprocessedLexicalSource, preprocess_source_for_lexing};
use mizar_session::{
    Hash,
    source::{SourcePathError, hash_text, normalize_source_path, normalize_source_text},
    source_map::LoadingMapSegment,
};

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

/// Reads optional native source text with the caller's expected normalized source hash.
// Reuse the existing normalization tuple without introducing a source wrapper.
#[allow(clippy::type_complexity)]
pub fn read_native_disk_source(
    package_root: impl AsRef<Path>,
    path: impl AsRef<Path>,
    expected_source_hash: Hash,
) -> io::Result<Option<(String, Option<Vec<LoadingMapSegment>>)>> {
    let root = package_root.as_ref();
    let normalized_path = match normalize_source_path(root, path.as_ref()) {
        Ok(path) => path,
        Err(SourcePathError::SourcePathUnavailable {
            kind: io::ErrorKind::NotFound,
            ..
        }) => return Ok(None),
        Err(error) => {
            let kind = match &error {
                SourcePathError::PackageRootUnavailable { kind, .. }
                | SourcePathError::SourcePathUnavailable { kind, .. } => *kind,
                _ => io::ErrorKind::InvalidInput,
            };
            return Err(io::Error::new(kind, error));
        }
    };
    let bytes = match fs::read(root.join(normalized_path.as_str())) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let normalized = normalize_source_text(text);
    if hash_text(&normalized.0) != expected_source_hash {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "native source hash mismatch",
        ));
    }
    Ok(Some(normalized))
}

/// Reads optional native source with all identity-free lexical preprocessing data.
#[allow(clippy::type_complexity)]
pub fn read_native_lexical_source(
    package_root: impl AsRef<Path>,
    path: impl AsRef<Path>,
    expected_source_hash: Hash,
) -> io::Result<
    Option<(
        String,
        Option<Vec<LoadingMapSegment>>,
        PreprocessedLexicalSource,
    )>,
> {
    let Some((text, loading_map)) =
        read_native_disk_source(package_root, path, expected_source_hash)?
    else {
        return Ok(None);
    };
    let lexical = preprocess_source_for_lexing(&text);
    Ok(Some((text, loading_map, lexical)))
}
