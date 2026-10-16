fn assignment(manifest: &str, section: &str, key: &str, value: &str) -> bool {
    manifest
        .split(&format!("[{section}]"))
        .nth(1)
        .unwrap_or_default()
        .split('[')
        .next()
        .unwrap_or_default()
        .lines()
        .any(|line| {
            line.split_once('=')
                .is_some_and(|(left, right)| left.trim() == key && right.trim() == value)
        })
}

#[test]
fn doc_inherits_workspace_lints() {
    assert!(assignment(
        include_str!("../Cargo.toml"),
        "lints",
        "workspace",
        "true"
    ));
}

#[test]
fn workspace_denies_rustc_warnings() {
    assert!(assignment(
        include_str!("../../../Cargo.toml"),
        "workspace.lints.rust",
        "warnings",
        "\"deny\""
    ));
}

#[test]
fn workspace_denies_clippy_all() {
    assert!(assignment(
        include_str!("../../../Cargo.toml"),
        "workspace.lints.clippy",
        "all",
        "\"deny\""
    ));
}
