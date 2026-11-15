use std::{fs, path::Path};

#[test]
fn lsp_manifest_opts_into_workspace_lints() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    assert_setting(&manifest, "lints", "workspace", "true");
}

#[test]
fn workspace_denies_rust_warnings_and_clippy_all() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml")).unwrap();
    assert_setting(&manifest, "workspace.lints.rust", "warnings", "deny");
    assert_setting(&manifest, "workspace.lints.clippy", "all", "deny");
}

#[test]
fn lsp_allow_exceptions_have_adjacent_reasons() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths = ["src", "tests", "examples", "benches"]
        .map(|name| root.join(name))
        .to_vec();
    while let Some(path) = paths.pop() {
        if path.is_dir() {
            paths.extend(
                fs::read_dir(&path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = fs::read_to_string(&path).unwrap();
            let lines = source.lines().collect::<Vec<_>>();
            for (index, line) in lines.iter().enumerate() {
                let compact = line.split_whitespace().collect::<String>();
                if (compact.starts_with("#[") || compact.starts_with("#!["))
                    && compact.contains("allow(")
                {
                    assert!(
                        lines[index.saturating_sub(1)..=(index + 1).min(lines.len() - 1)]
                            .iter()
                            .any(|line| line.split_once("//").is_some_and(|(_, comment)| {
                                let comment = comment.to_ascii_lowercase();
                                comment.contains("reason:") || comment.contains("rationale:")
                            })),
                        "{}:{} needs an adjacent lint reason",
                        path.display(),
                        index + 1
                    );
                }
            }
        }
    }
}

#[test]
fn public_enum_inventory_and_owner_policies_are_explicit() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths = vec![root.join("src")];
    let mut declarations = Vec::new();
    while let Some(path) = paths.pop() {
        if path.is_dir() {
            paths.extend(
                fs::read_dir(&path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = fs::read_to_string(&path).unwrap();
            for line in source.lines() {
                if let Some(name) = line.trim().strip_prefix("pub enum ") {
                    declarations.push(format!(
                        "{}:{}",
                        path.strip_prefix(root).unwrap().display(),
                        name.split_whitespace().next().unwrap()
                    ));
                }
            }
        }
    }
    declarations.sort();
    assert_eq!(
        declarations,
        [
            "src/build_bridge.rs:BuildBridgeError",
            "src/range_mapper.rs:RangeMapError",
            "src/server.rs:LspSnapshotFreshness",
        ]
    );

    let range_source = fs::read_to_string(root.join("src/range_mapper.rs")).unwrap();
    assert!(range_source.contains("#[non_exhaustive]\npub enum RangeMapError"));
    for (source_path, enum_name) in [
        ("src/server.rs", "LspSnapshotFreshness"),
        ("src/build_bridge.rs", "BuildBridgeError"),
    ] {
        let source = fs::read_to_string(root.join(source_path)).unwrap();
        assert!(
            !source.contains(&format!("#[non_exhaustive]\npub enum {enum_name}")),
            "{enum_name} is intentionally exhaustive"
        );
    }
    let docs = root.join("../../doc/design/mizar-lsp");
    let range_en = policy_section(&docs.join("en/range.md"));
    let range_ja = policy_section(&docs.join("ja/range.md"));
    assert!(range_en.contains("`RangeMapError` remains `#[non_exhaustive]`"));
    assert!(range_ja.contains("`RangeMapError` は `#[non_exhaustive]` を維持する"));
    for (language, policy) in [
        ("en", "deliberately remains exhaustive"),
        ("ja", "は意図的に exhaustive を維持する"),
    ] {
        let snapshot = policy_section(&docs.join(format!("{language}/snapshot.md")));
        assert!(snapshot.contains(&format!("`LspSnapshotFreshness` {policy}")));
        let bridge = policy_section(&docs.join(format!("{language}/build_bridge.md")));
        assert!(bridge.contains(&format!("`BuildBridgeError` {policy}")));
        assert!(bridge.contains("Capture/Submit"));
        assert!(bridge.contains("Driver"));
    }
}

fn policy_section(path: &Path) -> String {
    let document = fs::read_to_string(path).unwrap();
    let section = document
        .split_once("## Public enum policy")
        .unwrap_or_else(|| panic!("{} needs a Public enum policy section", path.display()))
        .1;
    section.split("\n## ").next().unwrap().to_owned()
}

#[test]
fn downstream_matches_compile_for_public_enum_policies() {
    let _ = |error: mizar_lsp::RangeMapError| match error {
        mizar_lsp::RangeMapError::SourceMap(_) => {}
        mizar_lsp::RangeMapError::PositionOverflow => {}
        mizar_lsp::RangeMapError::InvalidOpenBufferContext => {}
        _ => {}
    };
    let _ = |freshness: mizar_lsp::server::LspSnapshotFreshness| match freshness {
        mizar_lsp::server::LspSnapshotFreshness::Current => {}
        mizar_lsp::server::LspSnapshotFreshness::StaleArtifact => {}
    };
    let _ = |error: mizar_lsp::server::BuildBridgeError| match error {
        mizar_lsp::server::BuildBridgeError::Capture(_) => {}
        mizar_lsp::server::BuildBridgeError::Submit(_) => {}
    };
}

fn assert_setting(manifest: &str, section: &str, key: &str, value: &str) {
    let header = format!("[{section}]");
    assert!(
        manifest
            .lines()
            .map(str::trim)
            .skip_while(|line| *line != header)
            .skip(1)
            .take_while(|line| !line.starts_with('['))
            .any(|line| line.split_once('=').is_some_and(|(left, right)| {
                left.trim() == key && right.trim().trim_matches('"') == value
            })),
        "[{section}] must set {key} = {value}"
    );
}
