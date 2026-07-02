use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use mizar_session::Hash;
use mizar_test::{
    ParallelismProfile, SchemaVersion, SnapshotBaselineError, SnapshotBaselineStatus, SnapshotBody,
    SnapshotError, SnapshotKind, SnapshotProfile, SnapshotRecord, SnapshotTextDiff,
    SnapshotUpdateMode, SnapshotUpdateReason, TestCaseId, ToolchainInfo, compare_snapshot_records,
    verify_or_update_snapshot_baseline, verify_snapshot_determinism,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn snapshot_records_normalize_line_endings_and_sort_profile_metadata() {
    let left = SnapshotRecord::new(
        TestCaseId("snapshot_case".to_owned()),
        SnapshotKind::SurfaceAst,
        profile_with_metadata([("target", "x86_64"), ("rustc", "1.75")]),
        SnapshotBody::text("root\r\n  child\rleaf\n"),
    )
    .unwrap();
    let right = SnapshotRecord::new(
        TestCaseId("snapshot_case".to_owned()),
        SnapshotKind::SurfaceAst,
        profile_with_metadata([("rustc", "1.75"), ("target", "x86_64")]),
        SnapshotBody::text("root\n  child\nleaf\n"),
    )
    .unwrap();

    assert_eq!(left.body.canonical_text(), "root\n  child\nleaf\n");
    assert_eq!(left.content_hash, right.content_hash);
    assert_eq!(
        left.canonical_text().unwrap(),
        right.canonical_text().unwrap()
    );
    assert!(
        left.canonical_text()
            .unwrap()
            .contains("profile.toolchain.metadata.0.key = rustc\n")
    );
}

#[test]
fn snapshot_hash_includes_identity_kind_profile_and_body() {
    let base = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "same\n",
    );
    let different_id = record("other_case", SnapshotKind::SurfaceAst, profile(), "same\n");
    let different_kind = record("snapshot_case", SnapshotKind::CoreIr, profile(), "same\n");
    let different_toolchain_name = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        SnapshotProfile {
            toolchain: ToolchainInfo::new("other-tool", "task4"),
            ..profile()
        },
        "same\n",
    );
    let different_toolchain_version = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        SnapshotProfile {
            toolchain: ToolchainInfo::new("mizar-test", "task4b"),
            ..profile()
        },
        "same\n",
    );
    let different_metadata = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile_with_metadata([("rustc", "1.76")]),
        "same\n",
    );
    let different_verifier_hash = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        SnapshotProfile {
            verifier_config_hash: hash(0x43),
            ..profile()
        },
        "same\n",
    );
    let different_parallelism = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        SnapshotProfile {
            parallelism: ParallelismProfile::Parallel { workers: 4 },
            ..profile()
        },
        "same\n",
    );
    let different_normalize_paths = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        SnapshotProfile {
            normalize_paths: false,
            ..profile()
        },
        "same\n",
    );
    let different_allow_local_paths = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        SnapshotProfile {
            allow_local_paths: true,
            ..profile()
        },
        "same\n",
    );
    let different_body = record("snapshot_case", SnapshotKind::SurfaceAst, profile(), "same");

    for variant in [
        different_id,
        different_kind,
        different_toolchain_name,
        different_toolchain_version,
        different_metadata,
        different_verifier_hash,
        different_parallelism,
        different_normalize_paths,
        different_allow_local_paths,
        different_body,
    ] {
        assert_ne!(base.content_hash, variant.content_hash);
    }

    let mut different_schema = base.clone();
    different_schema.schema_version = SchemaVersion(2);
    different_schema.content_hash = mizar_session::hash_text(
        &different_schema
            .canonical_hash_input()
            .expect("changed schema should still canonicalize"),
    );
    assert_ne!(base.content_hash, different_schema.content_hash);
}

#[test]
fn snapshot_hash_input_frames_metadata_keys_and_values_injectively() {
    let left = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile_with_metadata([("a = b", "c")]),
        "body\n",
    );
    let right = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile_with_metadata([("a", "b = c")]),
        "body\n",
    );

    assert_ne!(left.content_hash, right.content_hash);
    assert!(left.canonical_hash_input().unwrap().contains(
        "profile.toolchain.metadata.0.key.len = 5\nprofile.toolchain.metadata.0.key = a = b"
    ));
    assert!(right.canonical_hash_input().unwrap().contains(
        "profile.toolchain.metadata.0.value.len = 5\nprofile.toolchain.metadata.0.value = b = c"
    ));
}

#[test]
fn snapshot_records_reject_local_absolute_paths_by_default() {
    for path in [
        "/tmp/mizar.snap",
        "file:///tmp/mizar.snap",
        "C:\\tmp\\mizar.snap",
        "C:/tmp/mizar.snap",
        "\\\\srv\\share\\mizar.snap",
        "path=/tmp/mizar.snap",
        "\"path\":\"/tmp/mizar.snap\"",
        "path=C:\\tmp\\mizar.snap",
    ] {
        let error = SnapshotRecord::new(
            TestCaseId("snapshot_case".to_owned()),
            SnapshotKind::FailureRecord,
            profile(),
            SnapshotBody::text(format!("diagnostic at {path}\n")),
        )
        .unwrap_err();

        assert!(
            matches!(error, SnapshotError::LocalPath { .. }),
            "expected local path rejection for {path}, got {error:?}"
        );
    }
}

#[test]
fn snapshot_records_allow_local_paths_when_profile_requests_them() {
    let record = record(
        "snapshot_case",
        SnapshotKind::FailureRecord,
        SnapshotProfile {
            allow_local_paths: true,
            ..profile()
        },
        "diagnostic at /tmp/mizar.snap path=C:/tmp/mizar.snap \\\\srv\\share\\mizar.snap\n",
    );

    assert!(record.canonical_text().unwrap().contains("/tmp/mizar.snap"));
    assert!(
        record
            .canonical_text()
            .unwrap()
            .contains("C:/tmp/mizar.snap")
    );
    assert!(
        record
            .canonical_text()
            .unwrap()
            .contains("\\\\srv\\share\\mizar.snap")
    );
}

#[test]
fn snapshot_comparison_reports_first_text_difference() {
    let expected = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\n  expected\n",
    );
    let actual = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\n  actual\n",
    );

    let mismatch = compare_snapshot_records(&expected, &actual).unwrap_err();

    assert_eq!(mismatch.expected_hash, expected.content_hash);
    assert_eq!(mismatch.actual_hash, actual.content_hash);
    assert_eq!(
        mismatch.first_difference,
        Some(SnapshotTextDiff {
            line: 2,
            expected: Some("  expected".to_owned()),
            actual: Some("  actual".to_owned())
        })
    );
}

#[test]
fn snapshot_comparison_recomputes_stale_public_hashes() {
    let expected = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\nexpected\n",
    );
    let mut actual = expected.clone();
    actual.body = SnapshotBody::text("root\nactual\n");

    assert!(matches!(
        actual.canonical_text().unwrap_err(),
        SnapshotError::StaleContentHash { .. }
    ));
    let mismatch = compare_snapshot_records(&expected, &actual).unwrap_err();
    assert_eq!(
        mismatch.first_difference,
        Some(SnapshotTextDiff {
            line: 2,
            expected: Some("expected".to_owned()),
            actual: Some("actual".to_owned()),
        })
    );
}

#[test]
fn snapshot_comparison_reports_insertions_deletions_and_final_newline_diffs() {
    let insertion = compare_snapshot_records(
        &record(
            "snapshot_case",
            SnapshotKind::SurfaceAst,
            profile(),
            "root\n",
        ),
        &record(
            "snapshot_case",
            SnapshotKind::SurfaceAst,
            profile(),
            "root\nextra\n",
        ),
    )
    .unwrap_err();
    assert_eq!(
        insertion.first_difference,
        Some(SnapshotTextDiff {
            line: 2,
            expected: Some(String::new()),
            actual: Some("extra".to_owned()),
        })
    );

    let deletion = compare_snapshot_records(
        &record(
            "snapshot_case",
            SnapshotKind::SurfaceAst,
            profile(),
            "root\nextra\n",
        ),
        &record(
            "snapshot_case",
            SnapshotKind::SurfaceAst,
            profile(),
            "root\n",
        ),
    )
    .unwrap_err();
    assert_eq!(
        deletion.first_difference,
        Some(SnapshotTextDiff {
            line: 2,
            expected: Some("extra".to_owned()),
            actual: Some(String::new()),
        })
    );

    let final_newline = compare_snapshot_records(
        &record(
            "snapshot_case",
            SnapshotKind::SurfaceAst,
            profile(),
            "root\n",
        ),
        &record("snapshot_case", SnapshotKind::SurfaceAst, profile(), "root"),
    )
    .unwrap_err();
    assert_eq!(
        final_newline.first_difference,
        Some(SnapshotTextDiff {
            line: 2,
            expected: Some(String::new()),
            actual: None,
        })
    );
}

#[test]
fn snapshot_records_validate_identity_and_profile_fields() {
    let empty_id = SnapshotRecord::new(
        TestCaseId(String::new()),
        SnapshotKind::SurfaceAst,
        profile(),
        SnapshotBody::text("body\n"),
    )
    .unwrap_err();
    let empty_toolchain = SnapshotRecord::new(
        TestCaseId("snapshot_case".to_owned()),
        SnapshotKind::SurfaceAst,
        SnapshotProfile {
            toolchain: ToolchainInfo::new("", "1"),
            ..profile()
        },
        SnapshotBody::text("body\n"),
    )
    .unwrap_err();
    let empty_toolchain_version = SnapshotRecord::new(
        TestCaseId("snapshot_case".to_owned()),
        SnapshotKind::SurfaceAst,
        SnapshotProfile {
            toolchain: ToolchainInfo::new("mizar-test", ""),
            ..profile()
        },
        SnapshotBody::text("body\n"),
    )
    .unwrap_err();
    let empty_metadata_key = SnapshotRecord::new(
        TestCaseId("snapshot_case".to_owned()),
        SnapshotKind::SurfaceAst,
        profile_with_metadata([("", "value")]),
        SnapshotBody::text("body\n"),
    )
    .unwrap_err();
    let zero_workers = SnapshotRecord::new(
        TestCaseId("snapshot_case".to_owned()),
        SnapshotKind::SurfaceAst,
        SnapshotProfile {
            parallelism: ParallelismProfile::Parallel { workers: 0 },
            ..profile()
        },
        SnapshotBody::text("body\n"),
    )
    .unwrap_err();

    assert_eq!(empty_id, SnapshotError::EmptyTestId);
    assert_eq!(empty_toolchain, SnapshotError::EmptyToolchainName);
    assert_eq!(
        empty_toolchain_version,
        SnapshotError::EmptyToolchainVersion
    );
    assert_eq!(empty_metadata_key, SnapshotError::EmptyMetadataKey);
    assert_eq!(zero_workers, SnapshotError::ParallelWorkerCountZero);
}

#[test]
fn snapshot_update_creates_and_verify_round_trips() {
    let root = SnapshotRoot::new();
    let baseline_path = Path::new("snapshots/general/snapshot_case.snap");
    let snapshot = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\n",
    );

    let missing = verify_or_update_snapshot_baseline(
        root.path(),
        baseline_path,
        &snapshot,
        SnapshotUpdateMode::VerifyOnly,
    )
    .unwrap_err();
    assert!(matches!(
        missing,
        SnapshotBaselineError::MissingBaseline { .. }
    ));
    assert!(!root.path().join(baseline_path).exists());

    let created = verify_or_update_snapshot_baseline(
        root.path(),
        baseline_path,
        &snapshot,
        SnapshotUpdateMode::Update {
            reason: SnapshotUpdateReason::SemanticBehaviorChange,
        },
    )
    .unwrap();

    assert_eq!(created.status, SnapshotBaselineStatus::Created);
    assert_eq!(
        created.update_reason,
        Some(SnapshotUpdateReason::SemanticBehaviorChange)
    );
    assert_eq!(root.read(baseline_path), snapshot.canonical_text().unwrap());

    let matched = verify_or_update_snapshot_baseline(
        root.path(),
        baseline_path,
        &snapshot,
        SnapshotUpdateMode::VerifyOnly,
    )
    .unwrap();
    assert_eq!(matched.status, SnapshotBaselineStatus::Matched);
    assert_eq!(matched.update_reason, None);
}

#[test]
fn snapshot_verify_only_missing_baseline_does_not_create_parent_directories() {
    let root = SnapshotRoot::new();
    let baseline_path = Path::new("snapshots/deep/missing/snapshot_case.snap");
    let snapshot = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\n",
    );

    let missing = verify_or_update_snapshot_baseline(
        root.path(),
        baseline_path,
        &snapshot,
        SnapshotUpdateMode::VerifyOnly,
    )
    .unwrap_err();

    assert!(matches!(
        missing,
        SnapshotBaselineError::MissingBaseline { .. }
    ));
    assert!(!root.path().join("snapshots/deep").exists());
}

#[test]
fn snapshot_verify_only_reports_mismatch_without_rewriting() {
    let root = SnapshotRoot::new();
    let baseline_path = Path::new("snapshots/general/snapshot_case.snap");
    root.write(baseline_path, "old\ncontent_hash = not-a-hash\n");
    let snapshot = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "new\n",
    );

    let mismatch = verify_or_update_snapshot_baseline(
        root.path(),
        baseline_path,
        &snapshot,
        SnapshotUpdateMode::VerifyOnly,
    )
    .unwrap_err();

    let SnapshotBaselineError::Mismatch { mismatch, .. } = mismatch else {
        panic!("expected mismatch");
    };
    assert_eq!(mismatch.expected_hash, None);
    assert_eq!(mismatch.actual_hash, snapshot.content_hash);
    assert_eq!(
        mismatch.first_difference,
        Some(SnapshotTextDiff {
            line: 1,
            expected: Some("old".to_owned()),
            actual: Some("snapshot_record = \"mizar-test-snapshot\"".to_owned()),
        })
    );
    assert_eq!(root.read(baseline_path), "old\ncontent_hash = not-a-hash\n");

    let updated = verify_or_update_snapshot_baseline(
        root.path(),
        baseline_path,
        &snapshot,
        SnapshotUpdateMode::Update {
            reason: SnapshotUpdateReason::DiagnosticContractChange,
        },
    )
    .unwrap();
    assert_eq!(updated.status, SnapshotBaselineStatus::Updated);
    assert_eq!(root.read(baseline_path), snapshot.canonical_text().unwrap());
}

#[test]
fn snapshot_baseline_helper_rejects_stale_records_before_io() {
    let root = SnapshotRoot::new();
    let baseline_path = Path::new("snapshots/general/snapshot_case.snap");
    let mut snapshot = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\n",
    );
    snapshot.body = SnapshotBody::text("changed\n");

    let error = verify_or_update_snapshot_baseline(
        root.path(),
        baseline_path,
        &snapshot,
        SnapshotUpdateMode::Update {
            reason: SnapshotUpdateReason::SchemaChange,
        },
    )
    .unwrap_err();

    assert!(matches!(
        error,
        SnapshotBaselineError::Snapshot(SnapshotError::StaleContentHash { .. })
    ));
    assert!(!root.path().join("snapshots").exists());
}

#[test]
fn snapshot_update_reports_io_errors() {
    let root = SnapshotRoot::new();
    root.write(Path::new("snapshots/general"), "not a directory\n");
    let baseline_path = Path::new("snapshots/general/snapshot_case.snap");
    let snapshot = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\n",
    );

    let error = verify_or_update_snapshot_baseline(
        root.path(),
        baseline_path,
        &snapshot,
        SnapshotUpdateMode::Update {
            reason: SnapshotUpdateReason::SchemaChange,
        },
    )
    .unwrap_err();

    assert!(matches!(error, SnapshotBaselineError::Io { .. }));
}

#[test]
fn snapshot_update_rejects_unsafe_baseline_paths() {
    let root = SnapshotRoot::new();
    let snapshot = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\n",
    );

    for path in [
        Path::new("../snapshots/out.snap"),
        Path::new("snapshots/out.txt"),
        Path::new("other/out.snap"),
        Path::new("/tmp/out.snap"),
    ] {
        let error = verify_or_update_snapshot_baseline(
            root.path(),
            path,
            &snapshot,
            SnapshotUpdateMode::Update {
                reason: SnapshotUpdateReason::SchemaChange,
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            SnapshotBaselineError::InvalidBaselinePath { .. }
        ));
    }
}

#[test]
fn snapshot_determinism_check_accepts_repeated_canonical_renders() {
    let first = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\r\nchild\r",
    );
    let second = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "root\nchild\n",
    );

    verify_snapshot_determinism(&[first, second]).unwrap();
}

#[test]
fn snapshot_determinism_check_reports_first_injected_difference() {
    let first = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "duration = 10\n",
    );
    let same = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "duration = 10\n",
    );
    let nondeterministic = record(
        "snapshot_case",
        SnapshotKind::SurfaceAst,
        profile(),
        "duration = 11\n",
    );

    let failure = verify_snapshot_determinism(&[first, same, nondeterministic]).unwrap_err();

    assert_eq!(failure.baseline_index, 0);
    assert_eq!(failure.candidate_index, 2);
    assert_eq!(
        failure.mismatch.first_difference,
        Some(SnapshotTextDiff {
            line: 1,
            expected: Some("duration = 10".to_owned()),
            actual: Some("duration = 11".to_owned()),
        })
    );
}

fn record(
    test_id: &str,
    kind: SnapshotKind,
    profile: SnapshotProfile,
    body: &str,
) -> SnapshotRecord {
    SnapshotRecord::new(
        TestCaseId(test_id.to_owned()),
        kind,
        profile,
        SnapshotBody::text(body),
    )
    .expect("snapshot fixture should build")
}

fn profile() -> SnapshotProfile {
    profile_with_metadata([])
}

fn profile_with_metadata<const N: usize>(metadata: [(&str, &str); N]) -> SnapshotProfile {
    SnapshotProfile {
        toolchain: ToolchainInfo {
            name: "mizar-test".to_owned(),
            version: "task4".to_owned(),
            metadata: metadata
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect::<BTreeMap<_, _>>(),
        },
        verifier_config_hash: hash(0x42),
        parallelism: ParallelismProfile::Sequential,
        normalize_paths: true,
        allow_local_paths: false,
    }
}

fn hash(byte: u8) -> Hash {
    Hash::from_bytes([byte; Hash::BYTE_LEN])
}

struct SnapshotRoot {
    path: PathBuf,
}

impl SnapshotRoot {
    fn new() -> Self {
        let id = NEXT_ROOT.fetch_add(1, Ordering::SeqCst);
        let path =
            std::env::temp_dir().join(format!("mizar-test-snapshot-{}-{id}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative_path: &Path, content: &str) {
        let path = self.path.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn read(&self, relative_path: &Path) -> String {
        fs::read_to_string(self.path.join(relative_path)).unwrap()
    }
}

impl Drop for SnapshotRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
