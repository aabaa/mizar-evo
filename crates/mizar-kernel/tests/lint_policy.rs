use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn kernel_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_kernel_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let workspace_members = workspace_members(&manifest);

    assert_eq!(
        workspace_members
            .iter()
            .filter(|member| member.as_str() == "crates/mizar-kernel")
            .count(),
        1,
        "{} must list crates/mizar-kernel exactly once in [workspace].members; found {workspace_members:?}",
        manifest_path.display()
    );
}

#[test]
fn kernel_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

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
            .any(|line| assignment_is(line, "name", "mizar_kernel")),
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
fn kernel_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "mizar-core = { path = \"../mizar-core\" }",
                "mizar-session = { path = \"../mizar-session\" }",
            ],
        )],
        "{} must keep the task-1 trusted-kernel dependency boundary exact: \
         production dependencies are mizar-core and mizar-session only, and \
         dev/build/target dependency sections require a later explicit task",
        manifest_path.display()
    );
}

#[test]
fn kernel_lib_exposes_only_current_spec_backed_modules() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);
    let source_files = rust_source_files(&crate_root().join("src"))
        .into_iter()
        .map(|path| {
            path.strip_prefix(crate_root())
                .expect("source path lives in crate root")
                .display()
                .to_string()
        })
        .collect::<Vec<_>>();
    let declarations = public_semantic_declarations(&source);

    assert!(
        source.contains("#![forbid(unsafe_code)]"),
        "{} must forbid unsafe code at the trusted-kernel crate root",
        lib_path.display()
    );
    for marker in [
        "Trusted certificate checking boundary",
        "owns pipeline phase 14",
        "checks evidence only",
        "must not perform proof search",
        "overload resolution",
        "cluster search",
        "ATP search",
        "implicit coercion",
        "fallback inference",
        "global-state",
    ] {
        assert!(
            source.contains(marker),
            "{} must keep the task-1 trusted-kernel statement marker `{marker}`",
            lib_path.display()
        );
    }
    assert_eq!(
        declarations,
        ["12: pub mod clause;", "compact: pubmod"],
        "{} must expose only current spec-backed kernel modules; found:\n{}",
        lib_path.display(),
        declarations.join("\n")
    );
    assert_eq!(
        source_files,
        ["src/clause.rs", "src/lib.rs"],
        "kernel source modules require their \
         paired English/Japanese specs first, found {source_files:?}"
    );
    for spec in [
        workspace_root().join("doc/design/mizar-kernel/en/clause.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/clause.md"),
    ] {
        assert!(
            spec.exists(),
            "{} must exist before its matching source module is exposed",
            spec.display()
        );
    }
}

#[test]
fn kernel_source_stays_off_producer_policy_cache_and_artifact_boundaries() {
    let forbidden = [
        "mizar_atp",
        "mizar_proof",
        "mizar_cache",
        "mizar_artifact",
        "AtpPortfolio",
        "BackendRunner",
        "ProofPolicy",
        "ProofWitnessStore",
        "ArtifactService",
        "Cache",
        "HashMap",
        "SystemTime",
        "thread_rng",
        "rand::",
    ];
    let mut violations = Vec::new();

    for path in rust_source_files(&crate_root().join("src")) {
        let source = read_to_string(&path);
        for token in forbidden {
            if source.contains(token) {
                let display_path = path.strip_prefix(crate_root()).unwrap_or(&path);
                violations.push(format!("{}: {token}", display_path.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "mizar-kernel task-1 source must not couple to ATP/proof/cache/artifact \
         producers or nondeterministic helper surfaces:\n{}",
        violations.join("\n")
    );
}

#[test]
fn public_surface_scanner_catches_common_shapes() {
    let public_samples = [
        "pub mod clause;",
        "pub use clause::Clause;",
        "pub struct Clause;",
        "pub enum RejectionReason { Invalid }",
        "pub fn check() {}",
        "pub const VERSION: u32 = 1;",
        "pub static LIMIT: u32 = 1;",
        "pub type ClauseId = u32;",
        "pub trait Checker {}",
        "pub unsafe fn check() {}",
        "pub async fn check() {}",
        "pub extern crate mizar_core;",
        "pub macro sample() {}",
        "pub union RawClause { value: u32 }",
        "#[macro_export] macro_rules! kernel_macro { () => {}; }",
        "pub\nstruct Clause;",
        "pub(crate)\nfn helper() {}",
        "pub\nunsafe fn check() {}",
        "pub\nasync fn check() {}",
        "pub\nextern crate mizar_core;",
        "pub\nextern \"C\" fn check() {}",
        "pub\nmacro sample() {}",
        "pub\nunion RawClause { value: u32 }",
        "pub(crate)\nunsafe fn helper() {}",
        "pub(super)\nasync fn helper() {}",
        "pub(crate)\nextern \"C\" fn helper() {}",
        "#[macro_export]\nmacro_rules! kernel_macro { () => {}; }",
    ];

    for sample in public_samples {
        assert!(!public_semantic_declarations(sample).is_empty(), "{sample}");
    }

    for sample in [
        "#![forbid(unsafe_code)]",
        "//! task 1 intentionally exposes no public semantic surface",
        "const PRIVATE: u32 = 1;",
    ] {
        assert!(public_semantic_declarations(sample).is_empty(), "{sample}");
    }
}

#[test]
fn dependency_scanner_catches_dependency_subtables() {
    let samples = [
        "[dependencies]",
        "[dependencies.mizar-atp]",
        "[dev-dependencies]",
        "[dev-dependencies.fixture]",
        "[build-dependencies]",
        "[build-dependencies.codegen]",
        "[target.'cfg(unix)'.dependencies]",
        "[target.'cfg(unix)'.dependencies.mizar-atp]",
        "[target.'cfg(unix)'.dev-dependencies.fixture]",
        "[target.'cfg(unix)'.build-dependencies.codegen]",
    ];

    for sample in samples {
        let section = section_name(sample).expect(sample);
        assert!(dependency_section(section), "{sample}");
    }

    for sample in ["[package]", "[lib]", "[lints]", "[workspace]"] {
        let section = section_name(sample).expect(sample);
        assert!(!dependency_section(section), "{sample}");
    }
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(Path::parent)
        .expect("crate lives under crates/<name>")
        .to_path_buf()
}

fn read_to_string(path: &Path) -> String {
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

fn workspace_members(manifest: &str) -> Vec<String> {
    let Some(workspace) = table_body(manifest, "workspace") else {
        return Vec::new();
    };
    let Some(members) = array_value(workspace, "members") else {
        return Vec::new();
    };

    quoted_strings(members)
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
        if let Some((_, lines)) = active.as_mut()
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
    line.strip_prefix('[')
        .and_then(|line| line.strip_suffix(']'))
        .map(str::trim)
}

fn dependency_section(section: &str) -> bool {
    const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    section
        .split('.')
        .any(|segment| DEPENDENCY_TABLES.contains(&segment))
}

fn assignment_is(line: &str, key: &str, expected: &str) -> bool {
    line.split_once('=')
        .map(|(actual_key, actual_value)| {
            actual_key.trim() == key && actual_value.trim().trim_matches('"') == expected
        })
        .unwrap_or(false)
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path.to_path_buf());
        }
        return;
    }

    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())) {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        collect_rust_source_files(&entry.path(), files);
    }
}

fn public_semantic_declarations(source: &str) -> Vec<String> {
    let mut declarations = source
        .lines()
        .enumerate()
        .filter(|(_, line)| public_semantic_declaration(line))
        .map(|(index, line)| format!("{}: {}", index + 1, line.trim()))
        .collect::<Vec<_>>();
    let compact_source = compact_non_comment_source(source);
    for marker in [
        "#[macro_export]",
        "pubmod",
        "pubuse",
        "pubstruct",
        "pubenum",
        "pubfn",
        "pubconst",
        "pubstatic",
        "pubtype",
        "pubtrait",
        "pubunsafe",
        "pubasync",
        "pubextern",
        "pubexterncrate",
        "pubmacro",
        "pubunion",
        "pub(crate)mod",
        "pub(crate)use",
        "pub(crate)struct",
        "pub(crate)enum",
        "pub(crate)fn",
        "pub(crate)const",
        "pub(crate)static",
        "pub(crate)type",
        "pub(crate)trait",
        "pub(crate)unsafe",
        "pub(crate)async",
        "pub(crate)extern",
        "pub(crate)externcrate",
        "pub(crate)macro",
        "pub(crate)union",
        "pub(super)mod",
        "pub(super)use",
        "pub(super)struct",
        "pub(super)enum",
        "pub(super)fn",
        "pub(super)const",
        "pub(super)static",
        "pub(super)type",
        "pub(super)trait",
        "pub(super)unsafe",
        "pub(super)async",
        "pub(super)extern",
        "pub(super)externcrate",
        "pub(super)macro",
        "pub(super)union",
        "pub(in",
    ] {
        if compact_source.contains(marker) {
            declarations.push(format!("compact: {marker}"));
        }
    }
    declarations.sort();
    declarations.dedup();
    declarations
}

fn public_semantic_declaration(line: &str) -> bool {
    let trimmed = line.trim();

    trimmed.starts_with("pub ")
        || trimmed.starts_with("pub(")
        || trimmed.starts_with("pub(crate)")
        || trimmed.starts_with("pub(super)")
        || trimmed.starts_with("pub(in ")
        || trimmed.starts_with("#[macro_export]")
}

fn table_body<'a>(document: &'a str, name: &str) -> Option<&'a str> {
    let header = format!("[{name}]");
    let start = document.find(&header)? + header.len();
    let rest = &document[start..];
    let end = rest
        .find("\n[")
        .map(|index| start + index)
        .unwrap_or(document.len());
    Some(&document[start..end])
}

fn array_value<'a>(table: &'a str, key: &str) -> Option<&'a str> {
    let key_offset = table.find(key)?;
    let after_key = &table[key_offset + key.len()..];
    let equals_offset = after_key.find('=')?;
    let after_equals = &after_key[equals_offset + 1..];
    let start_offset = after_equals.find('[')?;
    let after_start = &after_equals[start_offset + 1..];
    let end_offset = after_start.find(']')?;
    Some(&after_start[..end_offset])
}

fn quoted_strings(value: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut rest = value;

    while let Some(start) = rest.find('"') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        strings.push(after_start[..end].to_owned());
        rest = &after_start[end + 1..];
    }

    strings
}

fn compact_non_comment_source(source: &str) -> String {
    source
        .lines()
        .filter_map(|line| {
            line.split_once("//").map_or(Some(line), |(code, _)| {
                (!code.trim().is_empty()).then_some(code)
            })
        })
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}
