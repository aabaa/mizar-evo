use super::{
    CanonicalHashDomain, CanonicalJson, FieldPath, HashClass, MinorVersionPolicy,
    PublishedArtifactPath, PublishedArtifactReadOptions, PublishedPathError, SchemaVersion,
    SchemaVersionError, SchemaVersionSupport, StoreIoError, StoreIoOperation, TEMP_FILE_PREFIX,
    artifact_hash_domain, canonical_json_string, commit_with_temporary_candidates,
    read_published_artifact, write_published_artifact,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(1);

#[test]
fn canonical_json_sorts_object_keys_and_ends_with_newline() {
    let first = CanonicalJson::object([
        ("zeta", CanonicalJson::integer(2)),
        ("alpha", CanonicalJson::integer(1)),
    ])
    .expect("unique keys");
    let second = CanonicalJson::object([
        ("alpha", CanonicalJson::integer(1)),
        ("zeta", CanonicalJson::integer(2)),
    ])
    .expect("unique keys");

    assert_eq!(
        canonical_json_string(&first),
        canonical_json_string(&second)
    );
    assert_eq!(canonical_json_string(&first), "{\"alpha\":1,\"zeta\":2}\n");
}

#[test]
fn canonical_json_uses_required_string_escapes_only() {
    let value = CanonicalJson::string(
        "quote:\" slash:\\ backspace:\u{0008} tab:\t newline:\n formfeed:\u{000c} \
         carriage:\r ctrl:\u{001f} snow:\u{2603}",
    );

    assert_eq!(
        canonical_json_string(&value),
        "\"quote:\\\" slash:\\\\ backspace:\\b tab:\\t newline:\\n formfeed:\\f \
         carriage:\\r ctrl:\\u001f snow:\u{2603}\"\n"
    );
}

#[test]
fn canonical_json_rejects_duplicate_object_keys() {
    let error = CanonicalJson::object([
        ("alpha", CanonicalJson::integer(1)),
        ("alpha", CanonicalJson::integer(2)),
    ])
    .expect_err("duplicate key must be rejected");

    assert_eq!(
        error.to_string(),
        "duplicate canonical JSON object key `alpha`"
    );
}

#[test]
fn published_paths_reject_non_portable_or_escaping_spelling() {
    let cases = [
        ("", PublishedPathError::Empty),
        ("/absolute.json", PublishedPathError::Absolute),
        ("~/artifact.json", PublishedPathError::HomeRelative),
        ("module\\artifact.json", PublishedPathError::Backslash),
        ("module//artifact.json", PublishedPathError::EmptySegment),
        ("module/./artifact.json", PublishedPathError::CurrentSegment),
        ("module/../artifact.json", PublishedPathError::ParentSegment),
        ("C:/artifact.json", PublishedPathError::DrivePrefix),
        ("module/C:/artifact.json", PublishedPathError::DrivePrefix),
        ("module/artifact:ads.json", PublishedPathError::DrivePrefix),
    ];

    for (raw, reason) in cases {
        let error = PublishedArtifactPath::new(raw).expect_err("path must be rejected");
        assert!(
            matches!(error, StoreIoError::Path { reason: actual, .. } if actual == reason),
            "{raw} should fail with {reason:?}, got {error:?}"
        );
    }
}

#[test]
fn atomic_write_publishes_final_canonical_artifact_and_hash() {
    let root = TestArtifactRoot::new();
    let path = PublishedArtifactPath::new("modules/alpha.mizir.json").expect("valid path");
    let value = CanonicalJson::object([
        ("schema_version", CanonicalJson::string("1.0")),
        ("module", CanonicalJson::string("alpha")),
    ])
    .expect("unique keys");
    let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));

    let written =
        write_published_artifact(root.path(), &path, &value, &domain, &[]).expect("write succeeds");
    assert_eq!(written.path, path);
    assert_eq!(written.artifact_hash, domain.hash(&value, &[]));
    assert_eq!(
        fs::read_to_string(root.path().join(path.as_str())).expect("final artifact exists"),
        canonical_json_string(&value)
    );
    assert!(
        !fs::read_dir(root.path().join("modules"))
            .expect("published directory")
            .any(|entry| entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .starts_with(TEMP_FILE_PREFIX)),
        "successful rename must not leave visible temporary files"
    );

    let read = read_published_artifact(
        root.path(),
        &path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: Some(&domain),
            hash_excluded_paths: &[],
            expected_artifact_hash: Some(written.artifact_hash),
        },
    )
    .expect("read succeeds");
    assert_eq!(read.path, path);
    assert_eq!(read.value, value);
    assert_eq!(read.artifact_hash, Some(written.artifact_hash));
}

#[test]
fn published_artifact_write_is_deterministic_for_identical_inputs() {
    let root = TestArtifactRoot::new();
    let path = PublishedArtifactPath::new("modules/alpha.mizir.json").expect("valid path");
    let value = CanonicalJson::object([
        ("schema_version", CanonicalJson::string("1.0")),
        ("module", CanonicalJson::string("alpha")),
        ("exports", CanonicalJson::array([])),
    ])
    .expect("unique keys");
    let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));

    let first_write = write_published_artifact(root.path(), &path, &value, &domain, &[])
        .expect("first write succeeds");
    let first_bytes = fs::read(root.path().join(path.as_str())).expect("first artifact bytes");
    let second_write = write_published_artifact(root.path(), &path, &value, &domain, &[])
        .expect("second write succeeds");
    let second_bytes = fs::read(root.path().join(path.as_str())).expect("second artifact bytes");

    assert_eq!(first_write.path, second_write.path);
    assert_eq!(first_write.artifact_hash, second_write.artifact_hash);
    assert_eq!(first_write.artifact_hash, domain.hash(&value, &[]));
    assert_eq!(first_bytes, second_bytes);
    assert_eq!(first_bytes, canonical_json_string(&value).into_bytes());
}

#[test]
fn atomic_write_replaces_previous_complete_artifact() {
    let root = TestArtifactRoot::new();
    let path = PublishedArtifactPath::new("modules/alpha.mizir.json").expect("valid path");
    let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));
    let first = CanonicalJson::object([
        ("schema_version", CanonicalJson::string("1.0")),
        ("value", CanonicalJson::integer(1)),
    ])
    .expect("unique keys");
    let second = CanonicalJson::object([
        ("schema_version", CanonicalJson::string("1.0")),
        ("value", CanonicalJson::integer(2)),
    ])
    .expect("unique keys");

    let first_write =
        write_published_artifact(root.path(), &path, &first, &domain, &[]).expect("first write");
    assert_eq!(
        read_published_artifact(
            root.path(),
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &[],
                expected_artifact_hash: Some(first_write.artifact_hash),
            },
        )
        .expect("first read")
        .value,
        first
    );

    let second_write = write_published_artifact(root.path(), &path, &second, &domain, &[])
        .expect("replacement write");
    let read = read_published_artifact(
        root.path(),
        &path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: Some(&domain),
            hash_excluded_paths: &[],
            expected_artifact_hash: Some(second_write.artifact_hash),
        },
    )
    .expect("replacement read");

    assert_eq!(read.value, second);
    assert_ne!(first_write.artifact_hash, second_write.artifact_hash);
}

#[test]
fn stale_temporary_name_collision_retries_before_publishing() {
    let root = TestArtifactRoot::new();
    let final_path = root.path().join("alpha.json");
    let stale_temp = root.path().join(".mizar-artifact-tmp-stale-alpha.json");
    let fresh_temp = root.path().join(".mizar-artifact-tmp-fresh-alpha.json");
    fs::write(&stale_temp, b"stale").expect("stale temp fixture");
    let value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.0"))])
        .expect("unique keys");
    let bytes = super::canonical_json_bytes(&value);

    commit_with_temporary_candidates(
        &final_path,
        root.path(),
        &bytes,
        [stale_temp.clone(), fresh_temp.clone()],
    )
    .expect("retry should publish with second temp candidate");

    assert_eq!(
        fs::read_to_string(&final_path).expect("published artifact"),
        canonical_json_string(&value)
    );
    assert!(
        stale_temp.exists(),
        "stale temp files from other sessions must not be removed on collision"
    );
    assert!(
        !fresh_temp.exists(),
        "successful publish should rename away the temp file it created"
    );
}

#[test]
fn write_creates_fresh_artifact_root_before_canonicalizing_it() {
    let root = TestArtifactRoot::new_removed();
    let path = PublishedArtifactPath::new("alpha.json").expect("valid path");
    let value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.0"))])
        .expect("unique keys");
    let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));

    write_published_artifact(root.path(), &path, &value, &domain, &[])
        .expect("write creates artifact root");

    assert!(root.path().join("alpha.json").is_file());
}

#[test]
fn interrupted_temporary_file_is_not_visible_as_final_artifact() {
    let root = TestArtifactRoot::new();
    let final_path = PublishedArtifactPath::new("modules/alpha.json").expect("valid path");
    let partial_dir = root.path().join("modules");
    fs::create_dir_all(&partial_dir).expect("partial directory");
    fs::write(
        partial_dir.join(format!("{TEMP_FILE_PREFIX}-partial-alpha.json")),
        b"{\"schema_version\":",
    )
    .expect("partial temporary fixture");

    let error = read_published_artifact(
        root.path(),
        &final_path,
        PublishedArtifactReadOptions::default(),
    )
    .expect_err("reader must not discover temporary files");

    assert!(
        matches!(
            error,
            StoreIoError::Io {
                operation: StoreIoOperation::Read,
                kind: std::io::ErrorKind::NotFound,
                ..
            }
        ),
        "missing final path should be reported instead of reading temp content: {error:?}"
    );
}

#[test]
fn read_reports_corruption_with_artifact_positions() {
    let root = TestArtifactRoot::new();
    let utf8_path = PublishedArtifactPath::new("bad-utf8.json").expect("valid path");
    fs::write(root.path().join(utf8_path.as_str()), [0xff]).expect("bad utf8 fixture");
    let error = read_published_artifact(
        root.path(),
        &utf8_path,
        PublishedArtifactReadOptions::default(),
    )
    .expect_err("invalid UTF-8 must fail");
    assert!(
        matches!(
            error,
            StoreIoError::InvalidUtf8 {
                location: super::ArtifactTextLocation {
                    byte_offset: 0,
                    line: 1,
                    column: 1
                },
                ..
            }
        ),
        "invalid UTF-8 should carry first byte location: {error:?}"
    );

    let malformed_path = PublishedArtifactPath::new("malformed.json").expect("valid path");
    fs::write(root.path().join(malformed_path.as_str()), b"{\"a\":\n").expect("malformed fixture");
    let error = read_published_artifact(
        root.path(),
        &malformed_path,
        PublishedArtifactReadOptions::default(),
    )
    .expect_err("malformed JSON must fail");
    assert!(
        matches!(
            error,
            StoreIoError::CorruptCanonicalJson {
                location: super::ArtifactTextLocation {
                    byte_offset: 5,
                    line: 1,
                    column: 6
                },
                ..
            }
        ),
        "malformed JSON should include byte/line/column: {error:?}"
    );
}

#[test]
fn read_rejects_duplicate_keys_and_noncanonical_spelling() {
    let root = TestArtifactRoot::new();
    let duplicate_path = PublishedArtifactPath::new("duplicate.json").expect("valid path");
    fs::write(
        root.path().join(duplicate_path.as_str()),
        b"{\"a\":1,\"a\":2}\n",
    )
    .expect("duplicate fixture");
    let error = read_published_artifact(
        root.path(),
        &duplicate_path,
        PublishedArtifactReadOptions::default(),
    )
    .expect_err("duplicate object key must fail");
    assert!(
        matches!(error, StoreIoError::CorruptCanonicalJson { ref reason, .. } if reason.contains("duplicate object key")),
        "duplicate key should be a positioned canonical JSON error: {error:?}"
    );

    let unsorted_path = PublishedArtifactPath::new("unsorted.json").expect("valid path");
    fs::write(
        root.path().join(unsorted_path.as_str()),
        b"{\"b\":1,\"a\":2}\n",
    )
    .expect("unsorted fixture");
    let error = read_published_artifact(
        root.path(),
        &unsorted_path,
        PublishedArtifactReadOptions::default(),
    )
    .expect_err("unsorted canonical JSON must fail");
    assert!(
        matches!(error, StoreIoError::NonCanonicalJson { .. }),
        "noncanonical key order should fail after parsing: {error:?}"
    );
}

#[test]
fn expected_hash_mismatch_and_non_artifact_domains_are_rejected() {
    let root = TestArtifactRoot::new();
    let path = PublishedArtifactPath::new("alpha.json").expect("valid path");
    let value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.0"))])
        .expect("unique keys");
    let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));
    write_published_artifact(root.path(), &path, &value, &domain, &[]).expect("write");

    let other_value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.1"))])
        .expect("unique keys");
    let wrong_hash = domain.hash(&other_value, &[]);
    let error = read_published_artifact(
        root.path(),
        &path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: Some(&domain),
            hash_excluded_paths: &[],
            expected_artifact_hash: Some(wrong_hash),
        },
    )
    .expect_err("hash mismatch must fail");
    assert!(matches!(error, StoreIoError::ArtifactHashMismatch { .. }));

    let interface_domain =
        CanonicalHashDomain::new(HashClass::Interface, "store-test", SchemaVersion::new(1, 0));
    let error = write_published_artifact(root.path(), &path, &value, &interface_domain, &[])
        .expect_err("non-artifact domain must fail");
    assert!(matches!(
        error,
        StoreIoError::NonArtifactHashDomain {
            class: HashClass::Interface
        }
    ));

    let error = read_published_artifact(
        root.path(),
        &path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: Some(&interface_domain),
            hash_excluded_paths: &[],
            expected_artifact_hash: None,
        },
    )
    .expect_err("read with non-artifact domain must fail");
    assert!(matches!(
        error,
        StoreIoError::NonArtifactHashDomain {
            class: HashClass::Interface
        }
    ));

    let error = read_published_artifact(
        root.path(),
        &path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: None,
            hash_excluded_paths: &[],
            expected_artifact_hash: Some(domain.hash(&value, &[])),
        },
    )
    .expect_err("expected hash without domain must fail");
    assert!(matches!(error, StoreIoError::MissingHashDomain { .. }));
}

#[cfg(unix)]
#[test]
fn write_rejects_parent_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root = TestArtifactRoot::new();
    let outside = TestArtifactRoot::new();
    symlink(outside.path(), root.path().join("escape")).expect("parent symlink");
    let path = PublishedArtifactPath::new("escape/sub/out.json").expect("valid lexical path");
    let value = CanonicalJson::object([("schema_version", CanonicalJson::string("1.0"))])
        .expect("unique keys");
    let domain = artifact_hash_domain("store-test", SchemaVersion::new(1, 0));

    let error = write_published_artifact(root.path(), &path, &value, &domain, &[])
        .expect_err("parent symlink escape must fail");

    assert!(matches!(
        error,
        StoreIoError::Path {
            reason: PublishedPathError::RootEscape,
            ..
        }
    ));
    assert!(
        !outside.path().join("sub").exists(),
        "write validation must reject ancestor symlink before creating directories outside root"
    );
}

#[cfg(unix)]
#[test]
fn read_rejects_final_symlink_escape_before_following_it() {
    use std::os::unix::fs::symlink;

    let root = TestArtifactRoot::new();
    let outside = TestArtifactRoot::new();
    let target = outside.path().join("secret.json");
    fs::write(&target, b"{\"schema_version\":\"1.0\"}\n").expect("outside target");
    symlink(&target, root.path().join("link.json")).expect("final symlink");

    let path = PublishedArtifactPath::new("link.json").expect("valid lexical path");
    let error =
        read_published_artifact(root.path(), &path, PublishedArtifactReadOptions::default())
            .expect_err("final symlink must be rejected before read");

    assert!(matches!(
        error,
        StoreIoError::Path {
            reason: PublishedPathError::SymlinkEscape,
            ..
        }
    ));
}

#[cfg(unix)]
#[test]
fn read_rejects_ancestor_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root = TestArtifactRoot::new();
    let outside = TestArtifactRoot::new();
    fs::write(
        outside.path().join("out.json"),
        b"{\"schema_version\":\"1.0\"}\n",
    )
    .expect("outside artifact");
    symlink(outside.path(), root.path().join("escape")).expect("ancestor symlink");

    let path = PublishedArtifactPath::new("escape/out.json").expect("valid lexical path");
    let error =
        read_published_artifact(root.path(), &path, PublishedArtifactReadOptions::default())
            .expect_err("ancestor symlink escape must be rejected");

    assert!(matches!(
        error,
        StoreIoError::Path {
            reason: PublishedPathError::RootEscape,
            ..
        }
    ));
}

#[test]
fn schema_version_checks_detect_mismatches() {
    let support = SchemaVersionSupport::new("store-test", 1, 2, MinorVersionPolicy::UpToSupported);

    assert_eq!(
        support.check(Some("1.2")).expect("supported version"),
        SchemaVersion::new(1, 2)
    );
    assert!(matches!(
        support.check(None),
        Err(SchemaVersionError::Missing { .. })
    ));
    assert!(matches!(
        support.check(Some("1")),
        Err(SchemaVersionError::Malformed { .. })
    ));
    assert!(matches!(
        support.check(Some("2.0")),
        Err(SchemaVersionError::MajorMismatch { .. })
    ));
    assert!(matches!(
        support.check(Some("0.9")),
        Err(SchemaVersionError::MajorMismatch { .. })
    ));
    assert!(matches!(
        support.check(Some("1.3")),
        Err(SchemaVersionError::MinorTooNew { .. })
    ));
}

#[test]
fn schema_version_policy_can_allow_newer_minor_versions() {
    let support = SchemaVersionSupport::new("store-test", 1, 2, MinorVersionPolicy::AllowNewer);

    assert_eq!(
        support
            .check(Some("1.99"))
            .expect("schema declared newer-minor compatibility"),
        SchemaVersion::new(1, 99)
    );
}

#[test]
fn schema_version_errors_carry_supported_range_and_artifact_path() {
    let support = SchemaVersionSupport::new("store-test", 1, 2, MinorVersionPolicy::UpToSupported);

    assert_eq!(support.supported_range(), "1.0..=1.2");
    let error = support
        .check_at_path(Some("1.3"), "build/alpha.mizir.json")
        .expect_err("newer minor should be rejected");

    let SchemaVersionError::MinorTooNew {
        context,
        supported,
        actual,
        actual_version,
    } = &error
    else {
        panic!("expected minor-too-new error");
    };
    assert_eq!(context.family(), "store-test");
    assert_eq!(context.supported_range(), "1.0..=1.2");
    assert_eq!(context.artifact_path(), Some("build/alpha.mizir.json"));
    assert_eq!(*supported, 2);
    assert_eq!(*actual, 3);
    assert_eq!(*actual_version, SchemaVersion::new(1, 3));
    assert!(error.to_string().contains("build/alpha.mizir.json"));
    assert!(error.to_string().contains("1.3"));
    assert!(error.to_string().contains("1.0..=1.2"));
}

#[test]
fn hash_domains_are_separated() {
    let version = SchemaVersion::new(1, 0);
    let value =
        CanonicalJson::object([("stable", CanonicalJson::string("same"))]).expect("unique keys");
    let classes = [
        HashClass::Interface,
        HashClass::Implementation,
        HashClass::Diagnostic,
        HashClass::Artifact,
    ];

    for (left_index, left_class) in classes.iter().enumerate() {
        let left = CanonicalHashDomain::new(*left_class, "store-test", version);
        for right_class in classes.iter().skip(left_index + 1) {
            let right = CanonicalHashDomain::new(*right_class, "store-test", version);
            assert_ne!(left.hash(&value, &[]), right.hash(&value, &[]));
            assert_ne!(left.hash_input(&value, &[]), right.hash_input(&value, &[]));
        }
    }
}

#[test]
fn hash_frame_includes_schema_family_and_version() {
    let value =
        CanonicalJson::object([("stable", CanonicalJson::string("same"))]).expect("unique keys");
    let baseline =
        CanonicalHashDomain::new(HashClass::Interface, "store-test", SchemaVersion::new(1, 0));
    let other_family = CanonicalHashDomain::new(
        HashClass::Interface,
        "other-store-test",
        SchemaVersion::new(1, 0),
    );
    let other_version =
        CanonicalHashDomain::new(HashClass::Interface, "store-test", SchemaVersion::new(1, 1));

    assert_ne!(
        baseline.hash_input(&value, &[]),
        other_family.hash_input(&value, &[])
    );
    assert_ne!(baseline.hash(&value, &[]), other_family.hash(&value, &[]));
    assert_ne!(
        baseline.hash_input(&value, &[]),
        other_version.hash_input(&value, &[])
    );
    assert_ne!(baseline.hash(&value, &[]), other_version.hash(&value, &[]));
}

#[test]
fn hash_excluded_fields_do_not_affect_hashes() {
    let baseline = CanonicalJson::object([
        ("stable", CanonicalJson::string("same")),
        ("verified_at", CanonicalJson::string("2026-01-01T00:00:00Z")),
    ])
    .expect("unique keys");
    let changed_local = CanonicalJson::object([
        ("stable", CanonicalJson::string("same")),
        ("verified_at", CanonicalJson::string("2026-01-02T00:00:00Z")),
    ])
    .expect("unique keys");
    let domain = CanonicalHashDomain::new(
        HashClass::Implementation,
        "store-test",
        SchemaVersion::new(1, 0),
    );
    let excluded = [FieldPath::new(["verified_at"]).expect("non-empty path")];

    assert_ne!(
        domain.hash(&baseline, &[]),
        domain.hash(&changed_local, &[])
    );
    assert_eq!(
        domain.hash(&baseline, &excluded),
        domain.hash(&changed_local, &excluded)
    );
}

#[test]
fn hash_exclusion_paths_ignore_absent_paths_and_parent_paths_win() {
    let value = CanonicalJson::object([
        (
            "local",
            CanonicalJson::object([
                ("verified_at", CanonicalJson::string("now")),
                ("session", CanonicalJson::string("local")),
            ])
            .expect("unique nested keys"),
        ),
        ("stable", CanonicalJson::string("same")),
    ])
    .expect("unique keys");
    let parent_only = [FieldPath::new(["local"]).expect("non-empty path")];
    let parent_and_child = [
        FieldPath::new(["local"]).expect("non-empty path"),
        FieldPath::new(["local", "verified_at"]).expect("non-empty path"),
        FieldPath::new(["missing"]).expect("non-empty path"),
    ];
    let domain =
        CanonicalHashDomain::new(HashClass::Artifact, "store-test", SchemaVersion::new(1, 0));

    assert_eq!(
        domain.hash_input(&value, &parent_only),
        domain.hash_input(&value, &parent_and_child)
    );
}

#[test]
fn hash_exclusion_paths_remove_nested_child_fields_only() {
    let first = CanonicalJson::object([
        (
            "local",
            CanonicalJson::object([
                ("verified_at", CanonicalJson::string("now")),
                ("session", CanonicalJson::string("same-session")),
            ])
            .expect("unique nested keys"),
        ),
        ("stable", CanonicalJson::string("same")),
    ])
    .expect("unique keys");
    let changed_child = CanonicalJson::object([
        (
            "local",
            CanonicalJson::object([
                ("verified_at", CanonicalJson::string("later")),
                ("session", CanonicalJson::string("same-session")),
            ])
            .expect("unique nested keys"),
        ),
        ("stable", CanonicalJson::string("same")),
    ])
    .expect("unique keys");
    let changed_sibling = CanonicalJson::object([
        (
            "local",
            CanonicalJson::object([
                ("verified_at", CanonicalJson::string("later")),
                ("session", CanonicalJson::string("different-session")),
            ])
            .expect("unique nested keys"),
        ),
        ("stable", CanonicalJson::string("same")),
    ])
    .expect("unique keys");
    let domain =
        CanonicalHashDomain::new(HashClass::Artifact, "store-test", SchemaVersion::new(1, 0));
    let excluded_child = [FieldPath::new(["local", "verified_at"]).expect("non-empty path")];

    assert_eq!(
        domain.hash(&first, &excluded_child),
        domain.hash(&changed_child, &excluded_child)
    );
    assert_ne!(
        domain.hash(&first, &excluded_child),
        domain.hash(&changed_sibling, &excluded_child)
    );
}

#[test]
fn hash_exclusion_paths_do_not_traverse_arrays() {
    let value = CanonicalJson::object([(
        "items",
        CanonicalJson::array([CanonicalJson::object([(
            "verified_at",
            CanonicalJson::string("local"),
        )])
        .expect("unique nested keys")]),
    )])
    .expect("unique keys");
    let domain =
        CanonicalHashDomain::new(HashClass::Artifact, "store-test", SchemaVersion::new(1, 0));
    let attempted_array_path = [FieldPath::new(["items", "verified_at"]).expect("non-empty path")];

    assert_eq!(
        domain.hash_input(&value, &[]),
        domain.hash_input(&value, &attempted_array_path)
    );
}

struct TestArtifactRoot {
    path: PathBuf,
}

impl TestArtifactRoot {
    fn new() -> Self {
        let root = Self::fresh_path();
        fs::create_dir_all(&root).expect("test artifact root");
        Self { path: root }
    }

    fn new_removed() -> Self {
        let root = Self::fresh_path();
        let _ = fs::remove_dir_all(&root);
        Self { path: root }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn fresh_path() -> PathBuf {
        let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "mizar-artifact-store-test-{}-{counter}",
            std::process::id()
        ))
    }
}

impl Drop for TestArtifactRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
