use std::{fs, path::PathBuf};

#[test]
fn atp_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_atp_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let members = workspace_members(&manifest);
    let occurrences = members
        .iter()
        .filter(|member| **member == "crates/mizar-atp")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-atp exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn atp_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-atp")),
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
            .any(|line| assignment_is(line, "name", "mizar_atp")),
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
fn atp_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "mizar-core = { path = \"../mizar-core\" }",
                "mizar-kernel = { path = \"../mizar-kernel\" }",
                "mizar-session = { path = \"../mizar-session\" }",
                "mizar-vc = { path = \"../mizar-vc\" }",
            ],
        )],
        "{} must keep task-1 production dependencies limited to mizar-core, \
         mizar-kernel, mizar-session, and mizar-vc; dev/build/target \
         dependency sections require a later explicit task",
        manifest_path.display()
    );
}

#[test]
fn atp_lib_is_task_one_shell_without_semantic_modules() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);
    let expected_source = r#"//! ATP candidate-evidence production boundary.
//!
//! `mizar-atp` owns pipeline phase 13: translating open VC obligations into
//! backend-neutral ATP problems, running untrusted backends, and collecting
//! formula/substitution evidence candidates for `mizar-kernel`.
//!
//! This crate does not accept proofs, select trusted winners, call the kernel
//! as proof authority, or expose backend proof methods as trusted material.
//! Current task 1 intentionally publishes no semantic modules until their
//! English/Japanese module specs are added by later tasks.

#![forbid(unsafe_code)]
"#;
    let source_files = rust_source_files(&crate_root().join("src"))
        .into_iter()
        .map(|path| {
            path.strip_prefix(crate_root())
                .expect("source path lives in crate root")
                .display()
                .to_string()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        source,
        expected_source,
        "{} must remain exactly the task-1 source-surface allowlist; semantic \
         ATP modules, backend runners, kernel calls, proof acceptance, \
         certificate helpers, and trusted backend proof material require later \
         spec tasks",
        lib_path.display()
    );
    assert_eq!(
        source_files,
        ["src/lib.rs"],
        "task 1 must not add semantic ATP modules before their specs; found \
         {source_files:?}"
    );
}

#[test]
fn atp_crate_tree_is_task_one_allowlist() {
    let mut files = crate_files()
        .into_iter()
        .filter(|file| file != "Cargo.lock")
        .collect::<Vec<_>>();
    files.sort();

    assert_eq!(
        files,
        ["Cargo.toml", "src/lib.rs", "tests/lint_policy.rs"],
        "task 1 must not add build scripts, examples, benches, extra tests, \
         semantic modules, backend runners, kernel/proof behavior, or other \
         crate-root files before their explicit spec tasks; found {files:?}"
    );
}

#[test]
fn workspace_lint_baseline_denies_rustc_warnings_and_clippy_all() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let rust_lints = section(&manifest, "workspace.lints.rust");
    let clippy_lints = section(&manifest, "workspace.lints.clippy");

    assert!(
        rust_lints
            .iter()
            .any(|line| assignment_is(line, "warnings", "deny")),
        "{} must deny rustc warnings in the shared lint baseline",
        manifest_path.display()
    );
    assert!(
        clippy_lints
            .iter()
            .any(|line| assignment_is(line, "all", "deny")),
        "{} must deny clippy::all in the shared lint baseline",
        manifest_path.display()
    );
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(|path| path.parent())
        .expect("crate lives under crates/<name>")
        .to_path_buf()
}

fn read_to_string(path: &std::path::Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn section<'a>(manifest: &'a str, section: &str) -> Vec<&'a str> {
    let header = format!("[{section}]");
    let mut lines = Vec::new();
    let mut active = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            active = trimmed == header;
            continue;
        }
        if active && !trimmed.is_empty() && !trimmed.starts_with('#') {
            lines.push(trimmed);
        }
    }
    lines
}

fn workspace_members(manifest: &str) -> Vec<String> {
    let workspace = section(manifest, "workspace");
    let mut members = Vec::new();
    let mut in_members = false;

    for line in workspace {
        if !in_members {
            if let Some((lhs, rhs)) = line.split_once('=')
                && lhs.trim() == "members"
            {
                in_members = true;
                push_member_entries(rhs, &mut members);
                if rhs.contains(']') {
                    break;
                }
            }
            continue;
        }

        push_member_entries(line, &mut members);
        if line.contains(']') {
            break;
        }
    }

    members
}

fn push_member_entries(line: &str, members: &mut Vec<String>) {
    for entry in line.split(',') {
        let member = entry
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim()
            .trim_matches('"');
        if !member.is_empty() {
            members.push(member.to_owned());
        }
    }
}

fn dependency_sections(manifest: &str) -> Vec<(String, Vec<&str>)> {
    let mut sections = Vec::new();
    let mut active = None;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if let Some(section_name) = section_name(trimmed) {
            if let Some(section) = active.take() {
                sections.push(section);
            }
            active =
                dependency_section(section_name).then(|| (section_name.to_owned(), Vec::new()));
            continue;
        }

        if let Some((_, lines)) = &mut active
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            lines.push(trimmed);
        }
    }

    if let Some(section) = active {
        sections.push(section);
    }

    sections
}

fn section_name(line: &str) -> Option<&str> {
    line.strip_prefix('[')?.strip_suffix(']')
}

fn dependency_section(section_name: &str) -> bool {
    matches!(
        section_name,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    ) || section_name.starts_with("dependencies.")
        || section_name.starts_with("dev-dependencies.")
        || section_name.starts_with("build-dependencies.")
        || section_name.ends_with(".dependencies")
        || section_name.ends_with(".dev-dependencies")
        || section_name.ends_with(".build-dependencies")
        || section_name.contains(".dependencies.")
        || section_name.contains(".dev-dependencies.")
        || section_name.contains(".build-dependencies.")
}

fn assignment_is(line: &str, key: &str, value: &str) -> bool {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return false;
    };
    lhs.trim() == key && rhs.trim().trim_matches('"') == value
}

fn rust_source_files(root: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(path: &std::path::Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn crate_files() -> Vec<String> {
    let root = crate_root();
    let mut files = Vec::new();
    collect_crate_files(&root, &root, &mut files);
    files
}

fn collect_crate_files(root: &std::path::Path, path: &std::path::Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        let name = entry.file_name();
        if name == "target" {
            continue;
        }

        if path.is_dir() {
            collect_crate_files(root, &path, files);
        } else {
            files.push(
                path.strip_prefix(root)
                    .expect("crate file lives in crate root")
                    .display()
                    .to_string(),
            );
        }
    }
}
