use std::{fs, path::PathBuf};

#[test]
fn proof_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_proof_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let members = workspace_members(&manifest);
    let occurrences = members
        .iter()
        .filter(|member| **member == "crates/mizar-proof")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-proof exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn proof_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-proof")),
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
            .any(|line| assignment_is(line, "name", "mizar_proof")),
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
fn proof_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "mizar-artifact = { path = \"../mizar-artifact\" }",
                "mizar-atp = { path = \"../mizar-atp\" }",
                "mizar-kernel = { path = \"../mizar-kernel\" }",
                "mizar-session = { path = \"../mizar-session\" }",
                "mizar-vc = { path = \"../mizar-vc\" }",
            ],
        )],
        "{} must keep task-1 production dependencies limited to mizar-session, \
         mizar-kernel, mizar-vc, mizar-atp, and mizar-artifact; \
         dev/build/target dependency sections require a later explicit task",
        manifest_path.display()
    );
}

#[test]
fn proof_lib_states_boundary_and_exposes_modules_after_specs() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);

    for marker in [
        "#![forbid(unsafe_code)]",
        "Proof policy, selection, status projection",
        "does not accept proofs",
        "Trusted acceptance comes only from `mizar-kernel`",
        "not promoted into trusted proof status",
        "used_axioms",
    ] {
        assert!(
            source.contains(marker),
            "{} must keep the task-1 proof-boundary marker `{marker}`",
            lib_path.display()
        );
    }

    let declarations = public_module_declarations(&source);
    assert_eq!(
        declarations,
        ["policy", "selection"],
        "{} may expose only the task-3 policy module and task-6 selection \
         module; status and witness modules require later paired specs; found \
         {declarations:?}",
        lib_path.display()
    );
}

#[test]
fn proof_crate_tree_contains_only_task_six_files() {
    let mut files = crate_files();
    files.sort();

    assert_eq!(
        files,
        [
            "Cargo.toml",
            "src/lib.rs",
            "src/policy.rs",
            "src/selection.rs",
            "tests/lint_policy.rs"
        ],
        "mizar-proof task 6 may contain only the policy and selection modules \
         plus the lint guard; status, witness-store, build scripts, examples, \
         benches, or extra tests require later explicit tasks; found {files:?}"
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

fn section<'a>(document: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    let mut in_section = false;
    let mut lines = Vec::new();

    for line in document.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == header;
        } else if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
            lines.push(line.trim());
        }
    }

    lines
}

fn assignment_is(line: &str, key: &str, value: &str) -> bool {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return false;
    };

    lhs.trim() == key && rhs.trim().trim_matches('"') == value
}

fn workspace_members(manifest: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut in_members = false;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_members = false;
            continue;
        }
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
            continue;
        }
        if in_members {
            if trimmed.starts_with(']') {
                break;
            }
            if let Some(member) = trimmed.trim_end_matches(',').trim().strip_prefix('"') {
                members.push(member.trim_end_matches('"').to_owned());
            }
        }
    }

    members
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

fn public_module_declarations(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            line.trim_start()
                .strip_prefix("pub mod ")
                .map(|module| module.trim_end_matches(';').to_owned())
        })
        .collect()
}

fn crate_files() -> Vec<String> {
    let root = crate_root();
    let mut files = Vec::new();
    collect_files(&root, &root, &mut files);
    files
}

fn collect_files(root: &std::path::Path, dir: &std::path::Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, files);
        } else {
            files.push(
                path.strip_prefix(root)
                    .expect("file lives under crate root")
                    .display()
                    .to_string(),
            );
        }
    }
}
