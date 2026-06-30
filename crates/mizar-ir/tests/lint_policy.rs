use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn ir_manifest_opts_into_workspace_lints() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let lints = section(&manifest, "lints");

    assert!(
        lints
            .iter()
            .any(|line| assignment_is(line, "workspace", "true")),
        "{} must keep [lints] workspace = true so cargo build/test and clippy \
         use the shared lint policy",
        manifest_path.display()
    );
}

#[test]
fn workspace_manifest_includes_mizar_ir_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let occurrences = manifest
        .lines()
        .filter(|line| line.trim() == "\"crates/mizar-ir\",")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-ir exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn ir_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-ir")),
        "{} must keep the package name stable",
        manifest_path.display()
    );
    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "version", "0.1.0")),
        "{} must keep the task-1 crate version explicit until a release-policy \
         task changes it",
        manifest_path.display()
    );
    for key in [
        "edition.workspace",
        "license.workspace",
        "repository.workspace",
    ] {
        assert!(
            package.iter().any(|line| assignment_is(line, key, "true")),
            "{} must inherit {key} from the workspace",
            manifest_path.display()
        );
    }
    assert!(
        lib.iter()
            .any(|line| assignment_is(line, "name", "mizar_ir")),
        "{} must keep the library crate name stable",
        manifest_path.display()
    );
    assert!(
        lib.iter()
            .any(|line| assignment_is(line, "path", "src/lib.rs")),
        "{} must keep the task-1 library entry point at src/lib.rs",
        manifest_path.display()
    );
}

#[test]
fn ir_manifest_dependency_boundary_stays_narrow() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec!["mizar-session = { path = \"../mizar-session\" }"],
        )],
        "{} task 1 must keep production dependencies limited to mizar-session; \
         cache/artifact dependencies arrive only in their later task specs, and \
         mizar-driver/mizar-diagnostics must remain absent",
        manifest_path.display()
    );

    for (section, lines) in &dependency_sections {
        for forbidden in [
            "mizar-artifact",
            "mizar-build",
            "mizar-cache",
            "mizar-diagnostics",
            "mizar-driver",
        ] {
            assert!(
                lines.iter().all(|line| !line.contains(forbidden)),
                "{} must not list premature downstream crate `{forbidden}` in [{section}]",
                manifest_path.display()
            );
        }
    }
}

#[test]
fn ir_lib_states_boundary_and_external_gaps() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);

    for marker in [
        "#![forbid(unsafe_code)]",
        "Compiler-internal IR storage",
        "`mizar-ir` owns sealed internal phase-output storage",
        "consumes snapshot identity from `mizar-session`",
        "cache-key construction",
        "dependency fingerprints",
        "proof-reuse validation",
        "proof acceptance",
        "trusted",
        "kernel acceptance",
        "external dependency gaps",
    ] {
        assert!(
            source.contains(marker),
            "{} must keep the task-1 boundary marker `{marker}`",
            lib_path.display()
        );
    }
}

#[test]
fn public_forward_compatible_enums_are_marked_non_exhaustive() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in rust_source_files(&root.join("src")) {
        let source = read_to_string(&path);
        collect_public_enums_without_non_exhaustive(&root, &path, &source, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "public enum forward-compatibility policy requires #[non_exhaustive] on:\n{}",
        violations.join("\n")
    );
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(|path| path.parent())
        .expect("crate lives under workspace/crates/mizar-ir")
        .to_path_buf()
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        if path.is_dir() {
            let mut entries = fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .map(|entry| {
                    entry
                        .unwrap_or_else(|error| {
                            panic!(
                                "failed to read directory entry under {}: {error}",
                                path.display()
                            )
                        })
                        .path()
                })
                .collect::<Vec<_>>();
            entries.sort();
            stack.extend(entries.into_iter().rev());
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }

    files
}

fn collect_public_enums_without_non_exhaustive(
    root: &Path,
    path: &Path,
    source: &str,
    violations: &mut Vec<String>,
) {
    let lines = source.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("pub enum ")
            && !has_preceding_non_exhaustive_attribute(&lines, index)
        {
            let relative = path.strip_prefix(root).unwrap_or(path);
            violations.push(format!("{}:{}", relative.display(), index + 1));
        }
    }
}

fn has_preceding_non_exhaustive_attribute(lines: &[&str], enum_index: usize) -> bool {
    for line in lines[..enum_index].iter().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("///")
            || trimmed.starts_with("//!")
            || trimmed.starts_with("//")
        {
            continue;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("# [") {
            if trimmed.contains("non_exhaustive") {
                return true;
            }
            continue;
        }
        break;
    }

    false
}

fn section<'a>(manifest: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    let mut in_section = false;
    let mut lines = Vec::new();

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == header;
            continue;
        }
        if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
            lines.push(trimmed);
        }
    }

    lines
}

fn dependency_sections(manifest: &str) -> Vec<(String, Vec<&str>)> {
    let mut sections = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_lines = Vec::new();

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some(name) = current_name.take() {
                sections.push((name, current_lines));
                current_lines = Vec::new();
            }

            let name = trimmed.trim_start_matches('[').trim_end_matches(']');
            if name.ends_with("dependencies") {
                current_name = Some(name.to_owned());
            }
            continue;
        }

        if current_name.is_some() && !trimmed.is_empty() && !trimmed.starts_with('#') {
            current_lines.push(trimmed);
        }
    }

    if let Some(name) = current_name {
        sections.push((name, current_lines));
    }

    sections
}

fn assignment_is(line: &str, key: &str, value: &str) -> bool {
    let Some((actual_key, actual_value)) = line.split_once('=') else {
        return false;
    };
    actual_key.trim() == key && actual_value.trim().trim_matches('"') == value
}
