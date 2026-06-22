use std::{fs, path::PathBuf};

#[test]
fn artifact_manifest_opts_into_workspace_lints() {
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
fn artifact_manifest_depends_only_on_mizar_session() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_string(),
            vec!["mizar-session = { path = \"../mizar-session\" }"],
        )],
        "{} must keep mizar-session as its only dependency across normal, dev, \
         build, and target-specific dependency sections until a task-scoped \
         spec expands the crate boundary",
        manifest_path.display()
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

#[test]
fn public_artifact_enums_are_non_exhaustive_and_documented() {
    let expected = public_enum_policy();
    let mut expected_pairs = expected
        .iter()
        .map(|entry| (entry.source.to_string(), entry.name.to_string()))
        .collect::<Vec<_>>();
    expected_pairs.sort_unstable();

    let mut actual_pairs = Vec::new();
    for source in source_files() {
        for declaration in public_enum_declarations(&source) {
            assert!(
                declaration.non_exhaustive,
                "{}::{} must stay #[non_exhaustive] unless the owning module spec \
                 documents an explicit exhaustive public-enum exception",
                declaration.source, declaration.name
            );
            actual_pairs.push((declaration.source, declaration.name));
        }
    }
    actual_pairs.sort_unstable();

    assert_eq!(
        actual_pairs, expected_pairs,
        "every public enum in non-test src/**/*.rs must be recorded in the \
         task-19 forward-compatibility policy table"
    );

    for entry in expected {
        assert_documented_enum(
            entry.en_doc,
            "## Public Enum Forward Compatibility",
            entry.name,
        );
        assert_documented_enum(entry.ja_doc, "## 公開 enum の前方互換性", entry.name);
    }
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
                dependency_section(section_name).then(|| (section_name.to_string(), Vec::new()));
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

#[derive(Debug)]
struct PublicEnumPolicy {
    source: &'static str,
    name: &'static str,
    en_doc: &'static str,
    ja_doc: &'static str,
}

#[derive(Debug)]
struct PublicEnumDeclaration {
    source: String,
    name: String,
    non_exhaustive: bool,
}

fn public_enum_policy() -> &'static [PublicEnumPolicy] {
    &[
        PublicEnumPolicy {
            source: "src/manifest.rs",
            name: "ManifestError",
            en_doc: "doc/design/mizar-artifact/en/manifest.md",
            ja_doc: "doc/design/mizar-artifact/ja/manifest.md",
        },
        PublicEnumPolicy {
            source: "src/module_summary.rs",
            name: "ModuleSummaryError",
            en_doc: "doc/design/mizar-artifact/en/module_summary.md",
            ja_doc: "doc/design/mizar-artifact/ja/module_summary.md",
        },
        PublicEnumPolicy {
            source: "src/module_summary.rs",
            name: "ProofStatusSummary",
            en_doc: "doc/design/mizar-artifact/en/module_summary.md",
            ja_doc: "doc/design/mizar-artifact/ja/module_summary.md",
        },
        PublicEnumPolicy {
            source: "src/proof_witness.rs",
            name: "EvidenceKind",
            en_doc: "doc/design/mizar-artifact/en/proof_witness.md",
            ja_doc: "doc/design/mizar-artifact/ja/proof_witness.md",
        },
        PublicEnumPolicy {
            source: "src/proof_witness.rs",
            name: "ProofStatus",
            en_doc: "doc/design/mizar-artifact/en/proof_witness.md",
            ja_doc: "doc/design/mizar-artifact/ja/proof_witness.md",
        },
        PublicEnumPolicy {
            source: "src/proof_witness.rs",
            name: "ProofWitnessError",
            en_doc: "doc/design/mizar-artifact/en/proof_witness.md",
            ja_doc: "doc/design/mizar-artifact/ja/proof_witness.md",
        },
        PublicEnumPolicy {
            source: "src/registration_summary.rs",
            name: "ArtifactHashClass",
            en_doc: "doc/design/mizar-artifact/en/registration_summary.md",
            ja_doc: "doc/design/mizar-artifact/ja/registration_summary.md",
        },
        PublicEnumPolicy {
            source: "src/registration_summary.rs",
            name: "RegistrationAcceptedStatus",
            en_doc: "doc/design/mizar-artifact/en/registration_summary.md",
            ja_doc: "doc/design/mizar-artifact/ja/registration_summary.md",
        },
        PublicEnumPolicy {
            source: "src/registration_summary.rs",
            name: "RegistrationContributionKind",
            en_doc: "doc/design/mizar-artifact/en/registration_summary.md",
            ja_doc: "doc/design/mizar-artifact/ja/registration_summary.md",
        },
        PublicEnumPolicy {
            source: "src/registration_summary.rs",
            name: "RegistrationKind",
            en_doc: "doc/design/mizar-artifact/en/registration_summary.md",
            ja_doc: "doc/design/mizar-artifact/ja/registration_summary.md",
        },
        PublicEnumPolicy {
            source: "src/registration_summary.rs",
            name: "RegistrationSummaryError",
            en_doc: "doc/design/mizar-artifact/en/registration_summary.md",
            ja_doc: "doc/design/mizar-artifact/ja/registration_summary.md",
        },
        PublicEnumPolicy {
            source: "src/registration_summary.rs",
            name: "RegistrationTraceKind",
            en_doc: "doc/design/mizar-artifact/en/registration_summary.md",
            ja_doc: "doc/design/mizar-artifact/ja/registration_summary.md",
        },
        PublicEnumPolicy {
            source: "src/registration_summary.rs",
            name: "RegistrationVisibility",
            en_doc: "doc/design/mizar-artifact/en/registration_summary.md",
            ja_doc: "doc/design/mizar-artifact/ja/registration_summary.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "CanonicalJson",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "CanonicalJsonError",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "FieldPathError",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "HashClass",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "MinorVersionPolicy",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "PublishedPathError",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "SchemaVersionError",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "SchemaVersionParseError",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "StoreIoError",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/store.rs",
            name: "StoreIoOperation",
            en_doc: "doc/design/mizar-artifact/en/store.md",
            ja_doc: "doc/design/mizar-artifact/ja/store.md",
        },
        PublicEnumPolicy {
            source: "src/verified_artifact.rs",
            name: "DiagnosticSeverity",
            en_doc: "doc/design/mizar-artifact/en/verified_artifact.md",
            ja_doc: "doc/design/mizar-artifact/ja/verified_artifact.md",
        },
        PublicEnumPolicy {
            source: "src/verified_artifact.rs",
            name: "ExportProofStatus",
            en_doc: "doc/design/mizar-artifact/en/verified_artifact.md",
            ja_doc: "doc/design/mizar-artifact/ja/verified_artifact.md",
        },
        PublicEnumPolicy {
            source: "src/verified_artifact.rs",
            name: "ExportVisibility",
            en_doc: "doc/design/mizar-artifact/en/verified_artifact.md",
            ja_doc: "doc/design/mizar-artifact/ja/verified_artifact.md",
        },
        PublicEnumPolicy {
            source: "src/verified_artifact.rs",
            name: "ObligationStatus",
            en_doc: "doc/design/mizar-artifact/en/verified_artifact.md",
            ja_doc: "doc/design/mizar-artifact/ja/verified_artifact.md",
        },
        PublicEnumPolicy {
            source: "src/verified_artifact.rs",
            name: "VerifiedArtifactError",
            en_doc: "doc/design/mizar-artifact/en/verified_artifact.md",
            ja_doc: "doc/design/mizar-artifact/ja/verified_artifact.md",
        },
    ]
}

fn source_files() -> Vec<String> {
    let source_root = crate_root().join("src");
    let mut sources = Vec::new();
    collect_source_files(&source_root, &source_root, &mut sources);
    sources.sort();
    sources
}

fn collect_source_files(
    root: &std::path::Path,
    directory: &std::path::Path,
    sources: &mut Vec<String>,
) {
    for entry in
        fs::read_dir(directory).unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
    {
        let path = entry.expect("source entry").path();
        if path.is_dir() {
            collect_source_files(root, &path, sources);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("tests.rs") {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .expect("source path is under source root")
            .to_string_lossy()
            .replace('\\', "/");
        sources.push(format!("src/{relative}"));
    }
}

fn public_enum_declarations(source: &str) -> Vec<PublicEnumDeclaration> {
    let path = crate_root().join(source);
    let text = read_to_string(&path);
    let lines = text.lines().collect::<Vec<_>>();
    let mut declarations = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let Some(rest) = line.trim_start().strip_prefix("pub enum ") else {
            continue;
        };
        let name = rest
            .split(|character: char| character == '{' || character.is_whitespace())
            .next()
            .expect("enum name after pub enum")
            .to_string();
        let non_exhaustive = lines[..index]
            .iter()
            .rev()
            .take_while(|previous| {
                let trimmed = previous.trim();
                trimmed.is_empty() || trimmed.starts_with("#[")
            })
            .any(|previous| previous.trim() == "#[non_exhaustive]");
        declarations.push(PublicEnumDeclaration {
            source: source.to_string(),
            name,
            non_exhaustive,
        });
    }

    declarations
}

fn assert_documented_enum(doc: &str, heading: &str, enum_name: &str) {
    let path = workspace_root().join(doc);
    let text = read_to_string(&path);
    assert!(
        text.contains(heading),
        "{} must contain the task-19 public enum policy section",
        path.display()
    );
    assert!(
        text.contains(&format!("`{enum_name}`")),
        "{} must document the forward-compatibility decision for {enum_name}",
        path.display()
    );
}
