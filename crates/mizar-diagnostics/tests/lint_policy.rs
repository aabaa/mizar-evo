use std::{fs, path::PathBuf};

#[test]
fn diagnostics_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_diagnostics_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let members = workspace_members(&manifest);
    let occurrences = members
        .iter()
        .filter(|member| **member == "crates/mizar-diagnostics")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-diagnostics exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn workspace_consumers_do_not_depend_on_mizar_diagnostics_yet() {
    let root = workspace_root();
    let workspace_manifest_path = root.join("Cargo.toml");
    let workspace_manifest = read_to_string(&workspace_manifest_path);
    let mut reverse_dependencies = Vec::new();

    for member in workspace_members(&workspace_manifest) {
        if member == "crates/mizar-diagnostics" {
            continue;
        }

        let manifest_path = root.join(&member).join("Cargo.toml");
        let manifest = read_to_string(&manifest_path);
        for (section, lines) in dependency_sections(&manifest) {
            if dependency_section_names_mizar_diagnostics(&section) {
                reverse_dependencies.push(format!(
                    "{} [{section}]: dependency table",
                    manifest_path.display()
                ));
            }
            for line in lines {
                if line_depends_on_mizar_diagnostics(line) {
                    reverse_dependencies
                        .push(format!("{} [{section}]: {line}", manifest_path.display()));
                }
            }
        }
    }

    assert!(
        reverse_dependencies.is_empty(),
        "real consumer adoption must remain deferred until the adoption seam is \
         ready; unexpected mizar-diagnostics reverse dependencies:\n{}",
        reverse_dependencies.join("\n")
    );
}

#[test]
fn diagnostics_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-diagnostics")),
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
            .any(|line| assignment_is(line, "name", "mizar_diagnostics")),
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
fn diagnostics_manifest_depends_only_on_mizar_session() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_string(),
            vec!["mizar-session = { path = \"../mizar-session\" }"],
        )],
        "{} must keep mizar-session as the only dependency until a task-scoped \
         spec expands the diagnostics boundary",
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
fn diagnostics_allow_exceptions_are_documented_inline() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in rust_target_files(&root) {
        collect_undocumented_allows(&root, &path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "intentional lint allow exceptions need an adjacent reason:\n{}",
        violations.join("\n")
    );
}

#[test]
fn diagnostics_lib_states_initial_boundary() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);

    for marker in [
        "#![forbid(unsafe_code)]",
        "Stable diagnostic identity",
        "00.crate_plan.md",
        "diagnostic-code registry,",
        "structured failure records",
        "producer-side sinks",
        "deterministic",
        "aggregation; rendering,",
        "driver, LSP, and artifact integration",
        "later tasks.",
        "pub mod aggregator;",
        "pub mod failure_record;",
        "pub mod registry;",
        "pub mod sink;",
    ] {
        assert!(
            source.contains(marker),
            "{} must keep the task-9 diagnostics-boundary marker `{marker}`",
            lib_path.display()
        );
    }

    for forbidden_module in ["render", "fix", "explain", "driver", "lsp", "artifact"] {
        assert!(
            !source.contains(&format!("mod {forbidden_module}")),
            "{} must not add private `{forbidden_module}` wiring before its \
             task-scoped spec and implementation land",
            lib_path.display()
        );
    }

    assert_eq!(
        source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("pub ") || line.starts_with("pub("))
            .collect::<Vec<_>>(),
        vec![
            "pub mod aggregator;",
            "pub mod failure_record;",
            "pub mod registry;",
            "pub mod sink;",
        ],
        "{} must expose only the task-9 aggregator, registry, failure_record, and sink modules at \
         the crate root for now",
        lib_path.display()
    );
}

#[test]
fn diagnostics_sources_do_not_export_macros() {
    let root = crate_root();
    let mut declarations = Vec::new();

    for path in rust_target_files(&root) {
        let source = read_to_string(&path);
        for declaration in macro_export_declarations(&source) {
            let relative = path.strip_prefix(&root).unwrap_or(&path);
            declarations.push(format!("{}: {declaration}", relative.display()));
        }
    }

    assert!(
        declarations.is_empty(),
        "mizar-diagnostics must not expose exported macros before a task-scoped \
         spec introduces them:\n{}",
        declarations.join("\n")
    );
}

#[test]
fn allow_detector_covers_common_attribute_shapes() {
    let samples = [
        "#[allow(dead_code)]",
        "# [allow(dead_code)]",
        "#[ allow(dead_code)]",
        "#[allow (dead_code)]",
        "#![allow(dead_code)]",
        "#! [allow(dead_code)]",
        "# ! [allow(dead_code)]",
        "#[cfg_attr(test, allow(dead_code))]",
        "#[\n    cfg_attr(test,\n        allow(dead_code)\n    )\n]",
    ];

    for sample in samples {
        assert!(is_allow_attribute(sample), "{sample}");
    }
    assert!(!is_allow_attribute("#[doc = \"allow(dead_code)\"]"));
    assert!(!is_allow_attribute(
        "#[cfg_attr(test, /* allow(dead_code) */ doc = \"not a lint allow\")]"
    ));
}

#[test]
fn allow_scanner_handles_multiline_attributes_and_requires_rationale_text() {
    let source = "#[cfg_attr(\n    test,\n    doc = \"close ] bracket\",\n    allow(dead_code)\n)]\nfn sample() {}\n";
    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);

    let source = "// reason: test-only fixture\n#[cfg_attr(\n    test,\n    allow(dead_code)\n)]\nfn sample() {}\n";
    assert!(undocumented_allow_line_numbers(source).is_empty());

    let source = "// adjacent but not a rationale\n#[allow(dead_code)]\nfn sample() {}\n";
    assert_eq!(undocumented_allow_line_numbers(source), vec![2]);

    let source = "#[cfg_attr(test, doc = r#\"reason: text, ] and allow(dead_code)\"#, allow(dead_code))]\nfn sample() {}\n";
    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);

    let source =
        "#[cfg_attr(test, doc = \"reason: text in a string\", allow(dead_code))]\nfn sample() {}\n";
    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);
}

#[test]
fn rust_file_collector_skips_crate_local_target_directory() {
    assert!(skip_rust_directory(&PathBuf::from("target")));
    assert!(!skip_rust_directory(&PathBuf::from("src")));
}

#[test]
fn macro_export_detector_flags_common_attribute_shapes() {
    let source = "#[macro_export]\nmacro_rules! sample { () => {}; }\n# [macro_export]\nmacro_rules! other { () => {}; }\n#[\n    macro_export\n]\nmacro_rules! third { () => {}; }\n";
    assert_eq!(
        macro_export_declarations(source),
        vec!["#[macro_export]", "# [macro_export]", "#["]
    );
}

#[test]
fn reverse_dependency_detector_covers_dependency_table_aliases() {
    assert!(dependency_section_names_mizar_diagnostics(
        "dependencies.mizar-diagnostics"
    ));
    assert_eq!(
        section_name("[dependencies.mizar-diagnostics] # temporary"),
        Some("dependencies.mizar-diagnostics")
    );
    assert_eq!(
        section_name("[dependencies.diag] # temporary"),
        Some("dependencies.diag")
    );
    assert!(!dependency_section_names_mizar_diagnostics(
        "dependencies.diag"
    ));
    assert!(line_depends_on_mizar_diagnostics(
        "mizar-diagnostics = { path = \"../mizar-diagnostics\" }"
    ));
    assert!(line_depends_on_mizar_diagnostics(
        "diag = { package = \"mizar-diagnostics\", path = \"../mizar-diagnostics\" }"
    ));
    assert!(line_depends_on_mizar_diagnostics(
        "diag = { package=\"mizar-diagnostics\", path=\"../mizar-diagnostics\" }"
    ));
    assert!(line_depends_on_mizar_diagnostics(
        "package = \"mizar-diagnostics\""
    ));
    assert!(line_depends_on_mizar_diagnostics(
        "package = \"mizar-diagnostics\" # renamed dependency table"
    ));
    assert!(line_depends_on_mizar_diagnostics(
        "path = \"../mizar-diagnostics\""
    ));
    assert!(line_depends_on_mizar_diagnostics(
        "path = \"../mizar-diagnostics\" # renamed dependency table"
    ));
    assert!(!line_depends_on_mizar_diagnostics(
        "path = \"../mizar-session\""
    ));
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
    let Some(start) = manifest.find("members = [") else {
        return Vec::new();
    };
    let rest = &manifest[start + "members = [".len()..];
    let Some(end) = rest.find(']') else {
        return Vec::new();
    };

    rest[..end]
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix('"')
                .and_then(|line| line.split_once('"'))
                .map(|(member, _)| member.to_string())
        })
        .collect()
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
    let header = strip_inline_comment(line).trim();
    header.strip_prefix('[')?.strip_suffix(']')
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

fn dependency_section_names_mizar_diagnostics(section_name: &str) -> bool {
    section_name
        .split('.')
        .any(|part| part.trim_matches('"').trim_matches('\'') == "mizar-diagnostics")
}

fn line_depends_on_mizar_diagnostics(line: &str) -> bool {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return false;
    };

    let key = lhs.trim().trim_matches('"');
    let rhs_without_comment = strip_inline_comment(rhs).trim();
    let value = rhs_without_comment.trim_matches('"').trim_matches('\'');
    if (key == "package" && value == "mizar-diagnostics")
        || (key == "path" && value == "../mizar-diagnostics")
    {
        return true;
    }

    let package_name = key;
    let compact_rhs = compact_toml_value(rhs_without_comment);
    package_name == "mizar-diagnostics"
        || package_name.starts_with("mizar-diagnostics.")
        || compact_rhs.contains("package=\"mizar-diagnostics\"")
        || compact_rhs.contains("package='mizar-diagnostics'")
        || compact_rhs.contains("path=\"../mizar-diagnostics\"")
        || compact_rhs.contains("path='../mizar-diagnostics'")
}

fn strip_inline_comment(value: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;

    for (byte_index, character) in value.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active_quote {
                quote = None;
            }
            continue;
        }

        if character == '"' || character == '\'' {
            quote = Some(character);
        } else if character == '#' {
            return &value[..byte_index];
        }
    }

    value
}

fn compact_toml_value(value: &str) -> String {
    let mut compact = String::new();
    let mut quote = None;
    let mut escaped = false;

    for character in value.chars() {
        if let Some(active_quote) = quote {
            compact.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active_quote {
                quote = None;
            }
            continue;
        }

        if character == '"' || character == '\'' {
            quote = Some(character);
            compact.push(character);
        } else if !character.is_whitespace() {
            compact.push(character);
        }
    }

    compact
}

fn assignment_is(line: &str, key: &str, value: &str) -> bool {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return false;
    };
    lhs.trim() == key && rhs.trim().trim_matches('"') == value
}

fn rust_target_files(root: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);
    add_explicit_manifest_target_files(root, &mut files);
    files.sort();
    files.dedup();
    files
}

fn add_explicit_manifest_target_files(root: &std::path::Path, files: &mut Vec<PathBuf>) {
    for target_path in explicit_manifest_target_paths(root) {
        if let Some(parent) = target_path.parent()
            && parent.exists()
        {
            collect_rust_files(parent, files);
        }
        files.push(target_path);
    }
}

fn explicit_manifest_target_paths(root: &std::path::Path) -> Vec<PathBuf> {
    let manifest = read_to_string(&root.join("Cargo.toml"));
    let mut section = "";
    let mut paths = Vec::new();

    for line in manifest.lines() {
        let trimmed = line.trim();
        if let Some(section_name) = manifest_section_name(trimmed) {
            section = section_name;
            continue;
        }

        if section == "package" {
            if let Some(path) = quoted_assignment(trimmed, "build") {
                paths.push(root.join(path));
            }
        } else if is_rust_target_section(section)
            && let Some(path) = quoted_assignment(trimmed, "path")
        {
            paths.push(root.join(path));
        }
    }

    paths
}

fn manifest_section_name(line: &str) -> Option<&str> {
    let header = line
        .split_once('#')
        .map_or(line, |(before_comment, _)| before_comment)
        .trim();

    if header.starts_with('[') && header.ends_with(']') {
        Some(header.trim_start_matches('[').trim_end_matches(']'))
    } else {
        None
    }
}

fn is_rust_target_section(section: &str) -> bool {
    matches!(section, "lib" | "bin" | "test" | "example" | "bench")
}

fn quoted_assignment<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }

    let value = rhs.trim();
    quoted_value(value, '"').or_else(|| quoted_value(value, '\''))
}

fn quoted_value(value: &str, quote: char) -> Option<&str> {
    let rest = value.strip_prefix(quote)?;
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn collect_rust_files(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.expect("directory entry is readable").path();
        if path.is_dir() {
            if skip_rust_directory(&path) {
                continue;
            }
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn skip_rust_directory(path: &std::path::Path) -> bool {
    path.file_name()
        .is_some_and(|name| name == std::ffi::OsStr::new("target"))
}

fn collect_undocumented_allows(
    root: &std::path::Path,
    path: &std::path::Path,
    violations: &mut Vec<String>,
) {
    let source = read_to_string(path);

    for line_number in undocumented_allow_line_numbers(&source) {
        let relative = path.strip_prefix(root).unwrap_or(path);
        violations.push(format!("{}:{line_number}", relative.display()));
    }
}

fn undocumented_allow_line_numbers(source: &str) -> Vec<usize> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut violations = Vec::new();
    let mut line_index = 0;

    while line_index < lines.len() {
        if !starts_attribute(lines[line_index]) {
            line_index += 1;
            continue;
        }

        let (attribute, end_line_index) = attribute_block(&lines, line_index);
        if is_allow_attribute(&attribute)
            && !has_adjacent_allow_rationale(&lines, line_index, end_line_index)
        {
            violations.push(line_index + 1);
        }
        line_index += 1;
    }

    violations
}

fn starts_attribute(line: &str) -> bool {
    let compact = line
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    compact.starts_with("#[") || compact.starts_with("#![")
}

fn attribute_block(lines: &[&str], start: usize) -> (String, usize) {
    let mut block = String::new();
    let mut bracket_depth = 0_i32;
    let mut saw_opening_bracket = false;
    let mut string_quote = None;
    let mut escaped = false;
    let mut in_block_comment = false;

    for (line_index, line) in lines.iter().enumerate().skip(start) {
        if !block.is_empty() {
            block.push('\n');
        }
        block.push_str(line);

        let mut characters = line.chars().peekable();
        while let Some(character) = characters.next() {
            if in_block_comment {
                if character == '*' && characters.next_if_eq(&'/').is_some() {
                    in_block_comment = false;
                }
                continue;
            }

            if let Some(quote) = string_quote {
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == quote {
                    string_quote = None;
                }
                continue;
            }

            if character == '/' {
                if characters.next_if_eq(&'/').is_some() {
                    break;
                }
                if characters.next_if_eq(&'*').is_some() {
                    in_block_comment = true;
                    continue;
                }
            }

            if character == 'r' && skip_raw_string(&mut characters) {
                continue;
            }

            if character == '"' || character == '\'' {
                string_quote = Some(character);
                continue;
            }

            if character == '[' {
                bracket_depth += 1;
                saw_opening_bracket = true;
            } else if character == ']' && saw_opening_bracket {
                bracket_depth -= 1;
            }
        }

        if saw_opening_bracket && bracket_depth <= 0 {
            return (block, line_index);
        }
    }

    (block, lines.len().saturating_sub(1))
}

fn is_allow_attribute(attribute: &str) -> bool {
    let compact = compact_attribute_tokens(attribute);

    compact.starts_with("#[allow(")
        || compact.starts_with("#![allow(")
        || (compact.starts_with("#[cfg_attr(") && compact.contains(",allow("))
        || (compact.starts_with("#![cfg_attr(") && compact.contains(",allow("))
}

fn compact_attribute_tokens(attribute: &str) -> String {
    let mut compact = String::new();
    let mut characters = attribute.chars().peekable();
    let mut string_quote = None;
    let mut escaped = false;
    let mut in_block_comment = false;

    while let Some(character) = characters.next() {
        if in_block_comment {
            if character == '*' && characters.next_if_eq(&'/').is_some() {
                in_block_comment = false;
            }
            continue;
        }

        if let Some(quote) = string_quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote {
                string_quote = None;
            }
            continue;
        }

        if character == '/' {
            if characters.next_if_eq(&'/').is_some() {
                for next in characters.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
                continue;
            }
            if characters.next_if_eq(&'*').is_some() {
                in_block_comment = true;
                continue;
            }
        }

        if character == 'r' && skip_raw_string(&mut characters) {
            continue;
        }

        if character == '"' || character == '\'' {
            string_quote = Some(character);
            continue;
        }

        if !character.is_whitespace() {
            compact.push(character);
        }
    }

    compact
}

fn skip_raw_string(characters: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let mut probe = characters.clone();
    let mut hashes = 0;
    while probe.next_if_eq(&'#').is_some() {
        hashes += 1;
    }
    if probe.next_if_eq(&'"').is_none() {
        return false;
    }

    for _ in 0..hashes {
        characters.next();
    }
    characters.next();

    while let Some(character) = characters.next() {
        if character != '"' {
            continue;
        }

        let mut closing_hashes = 0;
        while closing_hashes < hashes && characters.next_if_eq(&'#').is_some() {
            closing_hashes += 1;
        }
        if closing_hashes == hashes {
            break;
        }
    }

    true
}

fn has_adjacent_allow_rationale(lines: &[&str], start: usize, end: usize) -> bool {
    lines[start..=end]
        .iter()
        .any(|line| allow_rationale_in(line))
        || start
            .checked_sub(1)
            .is_some_and(|previous| allow_rationale_in(lines[previous]))
        || lines
            .get(end + 1)
            .is_some_and(|next| allow_rationale_in(next))
}

fn allow_rationale_in(line: &str) -> bool {
    let Some(comment) = comment_text(line) else {
        return false;
    };

    let lower = comment.to_ascii_lowercase();
    lower.contains("reason:") || lower.contains("rationale:")
}

fn comment_text(line: &str) -> Option<&str> {
    if let Some((before_comment, comment)) = line.split_once("//")
        && !before_comment.contains('"')
    {
        return Some(comment);
    }

    let trimmed = line.trim_start();
    if trimmed.starts_with("/*") || trimmed.starts_with('*') {
        Some(trimmed)
    } else {
        None
    }
}

fn macro_export_declarations(source: &str) -> Vec<String> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut declarations = Vec::new();
    let mut line_index = 0;

    while line_index < lines.len() {
        let trimmed = lines[line_index].trim();
        if starts_attribute(trimmed) {
            let (attribute, end_line_index) = attribute_block(&lines, line_index);
            if is_macro_export_attribute(&attribute) {
                declarations.push(trimmed.to_owned());
            }
            line_index = end_line_index + 1;
            continue;
        }

        line_index += 1;
    }

    declarations
}

fn is_macro_export_attribute(attribute: &str) -> bool {
    let compact = compact_attribute_tokens(attribute);
    compact.starts_with("#[macro_export") || compact.starts_with("#![macro_export")
}
