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
