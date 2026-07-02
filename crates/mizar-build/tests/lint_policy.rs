use std::{fs, path::PathBuf};

#[test]
fn build_manifest_opts_into_workspace_lints() {
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
fn public_build_enums_are_forward_compatible_and_documented() {
    let mut expected = PUBLIC_ENUM_POLICY
        .iter()
        .map(|entry| (entry.source.to_owned(), entry.name.to_owned()))
        .collect::<Vec<_>>();
    expected.sort();

    let mut actual = Vec::new();
    let mut violations = Vec::new();
    for source in rust_source_files() {
        let text = read_to_string(&crate_root().join(&source));
        for declaration in public_enum_declarations(&source, &text) {
            if !declaration.has_non_exhaustive {
                violations.push(format!(
                    "{}:{}: public enum `{}` must keep #[non_exhaustive]",
                    source, declaration.line_number, declaration.name
                ));
            }
            actual.push((source.clone(), declaration.name));
        }
    }
    actual.sort();

    assert_eq!(
        actual, expected,
        "every public mizar-build enum must be classified by the task-21 \
         forward-compatibility policy"
    );

    for doc in public_enum_policy_docs(Language::English) {
        assert_documented_policy(
            doc,
            "## Public Enum Policy",
            "No exhaustive public enum exceptions are owned by this module",
            public_enum_policy_names_for_doc(doc, Language::English),
        );
    }
    for doc in public_enum_policy_docs(Language::Japanese) {
        assert_documented_policy(
            doc,
            "## 公開 enum policy",
            "この module が所有する exhaustive public enum exception はない",
            public_enum_policy_names_for_doc(doc, Language::Japanese),
        );
    }

    assert!(
        violations.is_empty(),
        "public mizar-build enum forward-compatibility policy drift:\n{}",
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

fn assignment_is(line: &str, key: &str, value: &str) -> bool {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return false;
    };
    lhs.trim() == key && rhs.trim().trim_matches('"') == value
}

struct PublicEnumPolicy {
    source: &'static str,
    name: &'static str,
    en_doc: &'static str,
    ja_doc: &'static str,
}

struct PublicEnumDeclaration {
    name: String,
    line_number: usize,
    has_non_exhaustive: bool,
}

#[derive(Clone, Copy)]
enum Language {
    English,
    Japanese,
}

const PUBLIC_ENUM_POLICY: &[PublicEnumPolicy] = &[
    planner_enum("LockSource"),
    planner_enum("DependencyKind"),
    planner_enum("VersionConstraint"),
    planner_enum("VersionComparison"),
    planner_enum("Solver"),
    planner_enum("DependencySelection"),
    planner_enum("PackagePlanSource"),
    planner_enum("ManifestDiagnosticKind"),
    planner_enum("ManifestValidationError"),
    module_index_enum("PackageIndexSource"),
    module_index_enum("NamespaceRoot"),
    module_index_enum("ModuleIndexLocation"),
    module_index_enum("ModuleIndexDiagnosticKind"),
    module_index_enum("ModuleIndexProviderError"),
    task_graph_enum("TaskKind"),
    task_graph_enum("PipelinePhase"),
    task_graph_enum("WorkUnit"),
    task_graph_enum("DependencyCoverage"),
    task_graph_enum("ResourceClass"),
    task_graph_enum("PriorityClass"),
    task_graph_enum("ModuleDependencyCoverage"),
    task_graph_enum("ModuleDependencyKind"),
    task_graph_enum("DocumentationProfile"),
    task_graph_enum("VcDescriptorPolicy"),
    task_graph_enum("TaskGraphDiagnosticKind"),
    scheduler_enum("TaskState"),
    scheduler_enum("SchedulerMode"),
    scheduler_enum("CacheSchedulingPolicy"),
    scheduler_enum("SyntheticTaskStatus"),
    scheduler_enum("CompletionOrder"),
    scheduler_enum("SchedulerQueue"),
    scheduler_enum("SchedulerEventKind"),
    scheduler_enum("SchedulerDiagnosticKind"),
    scheduler_enum("SchedulerDispatchStatus"),
    resource_enum("ResourceAdmissionStatus"),
    resource_enum("ResourceScope"),
    cancel_enum("CancellationReason"),
    cancel_enum("CancellationDecision"),
    cancel_enum("CancellationCheckpoint"),
    failure_state_enum("FailureCategory"),
    failure_state_enum("BlockReason"),
    cache_seam_enum("CacheSchedulingOutcome"),
    cache_seam_enum("CacheFallbackReason"),
    cache_seam_enum("CacheSchedulingPlanDiagnosticKind"),
    artifact_commit_enum("ArtifactCommitError"),
];

const fn planner_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/planner.rs",
        name,
        en_doc: "doc/design/mizar-build/en/planner.md",
        ja_doc: "doc/design/mizar-build/ja/planner.md",
    }
}

const fn module_index_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/module_index.rs",
        name,
        en_doc: "doc/design/mizar-build/en/module_index.md",
        ja_doc: "doc/design/mizar-build/ja/module_index.md",
    }
}

const fn task_graph_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/task_graph.rs",
        name,
        en_doc: "doc/design/mizar-build/en/task_graph.md",
        ja_doc: "doc/design/mizar-build/ja/task_graph.md",
    }
}

const fn scheduler_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/scheduler.rs",
        name,
        en_doc: "doc/design/mizar-build/en/scheduler.md",
        ja_doc: "doc/design/mizar-build/ja/scheduler.md",
    }
}

const fn resource_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/resource.rs",
        name,
        en_doc: "doc/design/mizar-build/en/resource.md",
        ja_doc: "doc/design/mizar-build/ja/resource.md",
    }
}

const fn cancel_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/cancel.rs",
        name,
        en_doc: "doc/design/mizar-build/en/cancel.md",
        ja_doc: "doc/design/mizar-build/ja/cancel.md",
    }
}

const fn failure_state_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/failure_state.rs",
        name,
        en_doc: "doc/design/mizar-build/en/failure_state.md",
        ja_doc: "doc/design/mizar-build/ja/failure_state.md",
    }
}

const fn cache_seam_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/cache_seam.rs",
        name,
        en_doc: "doc/design/mizar-build/en/cache_seam.md",
        ja_doc: "doc/design/mizar-build/ja/cache_seam.md",
    }
}

const fn artifact_commit_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/artifact_commit.rs",
        name,
        en_doc: "doc/design/mizar-build/en/artifact_commit.md",
        ja_doc: "doc/design/mizar-build/ja/artifact_commit.md",
    }
}

fn rust_source_files() -> Vec<String> {
    let root = crate_root().join("src");
    let mut files = Vec::new();
    collect_source_files(&root, &root, &mut files);
    files.sort();
    files
        .into_iter()
        .map(|file| format!("src/{file}"))
        .collect()
}

fn collect_source_files(root: &std::path::Path, dir: &std::path::Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(root, &path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(
                path.strip_prefix(root)
                    .expect("source file lives under src")
                    .display()
                    .to_string(),
            );
        }
    }
}

fn public_enum_declarations(source: &str, text: &str) -> Vec<PublicEnumDeclaration> {
    let lines = text.lines().collect::<Vec<_>>();
    let cfg_test_lines = cfg_test_item_lines(&lines);
    let mut declarations = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        if cfg_test_lines[index] {
            continue;
        }
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("pub enum ") else {
            continue;
        };
        let name = rest
            .split(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
            .next()
            .expect("pub enum declaration has a name");
        assert!(
            PUBLIC_ENUM_POLICY
                .iter()
                .any(|entry| entry.source == source && entry.name == name),
            "{source}: discovered unclassified public enum `{name}`"
        );
        declarations.push(PublicEnumDeclaration {
            name: name.to_owned(),
            line_number: index + 1,
            has_non_exhaustive: previous_attribute_is_non_exhaustive(&lines, index),
        });
    }

    declarations
}

fn cfg_test_item_lines(lines: &[&str]) -> Vec<bool> {
    let mut ignored = vec![false; lines.len()];
    let mut pending_cfg_test = false;
    let mut cfg_test_depth = None;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if let Some(depth) = &mut cfg_test_depth {
            ignored[index] = true;
            update_brace_depth(depth, line);
            if *depth == 0 {
                cfg_test_depth = None;
            }
            continue;
        }

        if pending_cfg_test {
            ignored[index] = true;
            if trimmed.is_empty() || trimmed.starts_with("#[") || trimmed.starts_with("///") {
                continue;
            }
            pending_cfg_test = false;
            let mut depth = 0;
            update_brace_depth(&mut depth, line);
            if depth > 0 {
                cfg_test_depth = Some(depth);
            }
            continue;
        }

        if trimmed == "#[cfg(test)]" {
            ignored[index] = true;
            pending_cfg_test = true;
        }
    }

    ignored
}

fn update_brace_depth(depth: &mut usize, line: &str) {
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;

    while let Some(character) = chars.next() {
        if in_line_comment {
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        if character == '/' && chars.peek() == Some(&'/') {
            in_line_comment = true;
            continue;
        }
        if character == '"' {
            in_string = true;
            continue;
        }
        if character == '{' {
            *depth += 1;
        } else if character == '}' {
            *depth = depth.saturating_sub(1);
        }
    }
}

fn previous_attribute_is_non_exhaustive(lines: &[&str], enum_index: usize) -> bool {
    lines[..enum_index]
        .iter()
        .rev()
        .take_while(|line| {
            let trimmed = line.trim();
            trimmed.is_empty() || trimmed.starts_with("#[") || trimmed.starts_with("///")
        })
        .any(|line| line.trim() == "#[non_exhaustive]")
}

fn public_enum_policy_docs(language: Language) -> Vec<&'static str> {
    let mut docs = PUBLIC_ENUM_POLICY
        .iter()
        .map(|entry| match language {
            Language::English => entry.en_doc,
            Language::Japanese => entry.ja_doc,
        })
        .collect::<Vec<_>>();
    docs.sort_unstable();
    docs.dedup();
    docs
}

fn public_enum_policy_names_for_doc(doc: &str, language: Language) -> Vec<&'static str> {
    let mut names = PUBLIC_ENUM_POLICY
        .iter()
        .filter(|entry| match language {
            Language::English => entry.en_doc == doc,
            Language::Japanese => entry.ja_doc == doc,
        })
        .map(|entry| entry.name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

fn assert_documented_policy(
    doc: &str,
    heading: &str,
    no_exhaustive_exception_text: &str,
    expected_names: Vec<&str>,
) {
    let path = workspace_root().join(doc);
    let text = read_to_string(&path);
    let section = public_enum_policy_section(&text, heading)
        .unwrap_or_else(|| panic!("{} must contain {heading}", path.display()));
    assert!(
        section.contains(no_exhaustive_exception_text),
        "{} must state that there are no exhaustive public enum exceptions",
        path.display()
    );

    let mut rows = public_enum_policy_rows(section);
    for row in &rows {
        assert!(
            row.has_non_exhaustive,
            "{} must document the #[non_exhaustive] decision for {}",
            path.display(),
            row.name
        );
    }
    rows.sort_by(|left, right| left.name.cmp(&right.name));
    let actual_names = rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>();
    assert!(
        actual_names == expected_names,
        "{} Public Enum Policy rows must exactly match the source inventory; \
         expected {:?}, actual {:?}",
        path.display(),
        expected_names,
        actual_names
    );
}

fn public_enum_policy_section<'a>(document: &'a str, heading: &str) -> Option<&'a str> {
    let start = document.find(heading)?;
    let rest = &document[start..];
    let end = rest
        .find("\n## ")
        .map_or(document.len(), |offset| start + offset);
    Some(&document[start..end])
}

#[derive(Debug)]
struct PublicEnumPolicyRow {
    name: String,
    has_non_exhaustive: bool,
}

fn public_enum_policy_rows(section: &str) -> Vec<PublicEnumPolicyRow> {
    section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("| `")?;
            let (name, _) = rest.split_once("` |")?;
            Some(PublicEnumPolicyRow {
                name: name.to_owned(),
                has_non_exhaustive: trimmed.contains("#[non_exhaustive]"),
            })
        })
        .collect()
}
