use std::{error::Error, fmt};

use mizar_session::PackageId;

const PACKAGE_ID_PATTERN: &str = "[a-z][a-z0-9]*(?:_[a-z0-9]+)*";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPackageManifest {
    pub package_id: PackageId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestValidationError {
    InvalidPackageId {
        package_id: String,
        expected: &'static str,
    },
}

pub fn validate_package_manifest(
    manifest: &PackageManifest,
) -> Result<ValidatedPackageManifest, ManifestValidationError> {
    validate_package_id_spelling(&manifest.name)?;
    Ok(ValidatedPackageManifest {
        package_id: PackageId::new(manifest.name.clone()),
    })
}

pub fn validate_package_id_spelling(package_id: &str) -> Result<(), ManifestValidationError> {
    if is_lowercase_snake_case_package_id(package_id) {
        Ok(())
    } else {
        Err(ManifestValidationError::InvalidPackageId {
            package_id: package_id.to_owned(),
            expected: PACKAGE_ID_PATTERN,
        })
    }
}

pub fn is_lowercase_snake_case_package_id(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }

    let mut previous_was_underscore = false;
    for ch in chars {
        match ch {
            'a'..='z' | '0'..='9' => previous_was_underscore = false,
            '_' if !previous_was_underscore => previous_was_underscore = true,
            _ => return false,
        }
    }

    !previous_was_underscore
}

impl fmt::Display for ManifestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPackageId {
                package_id,
                expected,
            } => {
                write!(
                    f,
                    "invalid package id `{package_id}`; expected lowercase snake_case matching `{expected}`"
                )
            }
        }
    }
}

impl Error for ManifestValidationError {}

#[cfg(test)]
mod tests {
    use super::{
        ManifestValidationError, PackageManifest, is_lowercase_snake_case_package_id,
        validate_package_id_spelling, validate_package_manifest,
    };

    #[test]
    fn package_ids_accept_lowercase_snake_case() {
        for package_id in ["a", "mml", "mathcomp_mizar", "pkg1", "pkg_1_core2"] {
            assert!(
                is_lowercase_snake_case_package_id(package_id),
                "{package_id:?}"
            );
            validate_package_id_spelling(package_id).expect("valid package id");
        }
    }

    #[test]
    fn package_ids_reject_hyphenated_or_normalized_spellings() {
        for package_id in [
            "mathcomp-mizar",
            "MathComp",
            "mathcomp__mizar",
            "mathcomp_",
            "_mathcomp",
            "mathcomp mizar",
            "mathcomp.mizar",
            "1mathcomp",
            "",
        ] {
            let error = validate_package_id_spelling(package_id).unwrap_err();
            assert!(
                matches!(
                    error,
                    ManifestValidationError::InvalidPackageId {
                        package_id: ref rejected,
                        expected: "[a-z][a-z0-9]*(?:_[a-z0-9]+)*",
                    } if rejected == package_id
                ),
                "{package_id:?}: {error:?}"
            );
        }
    }

    #[test]
    fn package_manifest_validation_preserves_spelling_without_hyphen_normalization() {
        let valid = PackageManifest {
            name: "mathcomp_mizar".to_owned(),
        };
        let validated = validate_package_manifest(&valid).expect("valid manifest");
        assert_eq!(validated.package_id.as_str(), "mathcomp_mizar");

        let hyphenated = PackageManifest {
            name: "mathcomp-mizar".to_owned(),
        };
        let error = validate_package_manifest(&hyphenated).unwrap_err();
        assert!(matches!(
            error,
            ManifestValidationError::InvalidPackageId { package_id, .. }
                if package_id == "mathcomp-mizar"
        ));
    }
}
