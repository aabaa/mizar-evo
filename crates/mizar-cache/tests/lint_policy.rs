use std::{fs, path::PathBuf};

#[test]
fn cache_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_cache_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let members = workspace_members(&manifest);
    let occurrences = members
        .iter()
        .filter(|member| **member == "crates/mizar-cache")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-cache exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn cache_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-cache")),
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
            .any(|line| assignment_is(line, "name", "mizar_cache")),
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
fn cache_manifest_dependency_boundary_matches_cache_key_builder() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "blake3 = \"1.8.5\"",
                "mizar-artifact = { path = \"../mizar-artifact\" }",
                "mizar-session = { path = \"../mizar-session\" }",
            ],
        )],
        "{} must keep production dependencies limited to the cache-key builder \
         hash implementation plus mizar-session and mizar-artifact; \
         dev/build/target dependency sections require a later explicit task",
        manifest_path.display()
    );
}

#[test]
fn cache_lib_states_boundary_and_cache_key_module() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);

    for marker in [
        "#![forbid(unsafe_code)]",
        "Internal build cache keys",
        "does not accept proofs",
        "Trusted acceptance comes only from `mizar-kernel`",
        "not promoted into kernel-verified status",
        "used_axioms",
    ] {
        assert!(
            source.contains(marker),
            "{} must keep the task-1 cache-boundary marker `{marker}`",
            lib_path.display()
        );
    }

    assert_eq!(
        public_api_declarations(&source),
        ["pub mod cache_key;"],
        "{} must expose only the cache_key API until later module specs land",
        lib_path.display()
    );
}

#[test]
fn cache_key_api_does_not_expose_proof_authority_terms() {
    let cache_key_path = crate_root().join("src/cache_key.rs");
    let source = read_to_string(&cache_key_path);
    let public_surface = public_api_surface_lines(&source);

    for forbidden in [
        "KernelCheckResult",
        "ProofStatus",
        "used_axioms",
        "TrustedAcceptance",
        "Authority",
        "Accept",
    ] {
        assert!(
            public_surface
                .iter()
                .all(|declaration| !declaration.contains(forbidden)),
            "{} must not expose proof-authority projection terms through the \
             cache_key public API; found `{forbidden}` in {public_surface:?}",
            cache_key_path.display()
        );
    }
}

#[test]
fn cache_key_implementation_excludes_mutable_runtime_inputs() {
    let cache_key_path = crate_root().join("src/cache_key.rs");
    let source = read_to_string(&cache_key_path);
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation prefix exists");

    for forbidden in [
        "std::fs",
        "std::time",
        "SystemTime",
        "Instant",
        "read_dir",
        "cache_dir",
        "scheduler",
        "thread::current",
        "process::id",
        "temp_dir",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "{} cache_key implementation must remain a pure projection and \
             exclude mutable runtime input `{forbidden}`",
            cache_key_path.display()
        );
    }
}

#[test]
fn cache_crate_tree_contains_only_task_three_files() {
    let mut files = crate_files();
    files.sort();

    assert_eq!(
        files,
        [
            "Cargo.toml",
            "src/cache_key.rs",
            "src/lib.rs",
            "tests/lint_policy.rs"
        ],
        "mizar-cache task 3 may contain only the crate manifest, root module, \
         cache_key implementation, and lint guard; other behavior modules, \
         build scripts, examples, benches, or extra tests require later \
         explicit tasks; found {files:?}"
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

fn section<'a>(document: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    let mut in_section = false;
    let mut lines = Vec::new();

    for line in document.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == header;
        } else if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
            lines.push(trimmed);
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

fn public_api_declarations(source: &str) -> Vec<String> {
    let public_markers = [
        "pub const ",
        "pub enum ",
        "pub fn ",
        "pub mod ",
        "pub static ",
        "pub struct ",
        "pub trait ",
        "pub type ",
        "pub use ",
    ];

    source
        .lines()
        .map(str::trim_start)
        .filter_map(|line| {
            public_markers
                .iter()
                .find(|marker| line.starts_with(**marker))
                .map(|_| line.to_owned())
        })
        .collect()
}

fn public_api_surface_lines(source: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_public_type = false;
    let mut depth = 0usize;

    for raw_line in source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation prefix exists")
        .lines()
    {
        let line = raw_line.trim_start();
        if line.starts_with("//") || line.starts_with("#[") || line.is_empty() {
            continue;
        }

        if line.starts_with("pub ") {
            lines.push(line.to_owned());
            in_public_type = starts_public_type(line);
        } else if in_public_type {
            lines.push(line.to_owned());
        }

        if in_public_type {
            depth = depth
                .saturating_add(line.matches('{').count())
                .saturating_sub(line.matches('}').count());
            if depth == 0 {
                in_public_type = false;
            }
        }
    }

    lines
}

fn starts_public_type(line: &str) -> bool {
    line.starts_with("pub struct ")
        || line.starts_with("pub enum ")
        || line.starts_with("pub trait ")
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
