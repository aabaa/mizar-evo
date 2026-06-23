use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn checker_manifest_opts_into_workspace_lints() {
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
fn checker_manifest_keeps_task_one_package_metadata() {
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
            .any(|line| assignment_is(line, "name", "mizar_checker")),
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
fn checker_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "mizar-resolve = { path = \"../mizar-resolve\" }",
                "mizar-session = { path = \"../mizar-session\" }",
            ],
        )],
        "{} must depend only on mizar-resolve and mizar-session until a \
         task-scoped checker spec expands the crate boundary",
        manifest_path.display()
    );
}

#[test]
fn checker_source_does_not_import_syntax_directly() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in checker_src_files(&root) {
        let source = read_to_string(&path);
        if source.contains("mizar_syntax::") || source.contains("extern crate mizar_syntax") {
            let display_path = path.strip_prefix(&root).unwrap_or(&path);
            violations.push(display_path.display().to_string());
        }
    }

    assert!(
        violations.is_empty(),
        "mizar-checker must keep checker-local semantic source-shape and \
         binding-env boundaries instead of importing mizar-syntax directly:\n{}",
        violations.join("\n")
    );
}

#[test]
fn overload_collection_stays_on_explicit_payload_boundary() {
    let root = crate_root();
    let path = root.join("src/overload_resolution.rs");
    let source = read_to_string(&path);
    let forbidden = [
        "mizar_resolve::env",
        "SymbolEnv",
        "ResolvedAst",
        "mizar_syntax::",
        "extern crate mizar_syntax",
    ];
    let violations = forbidden
        .iter()
        .copied()
        .filter(|token| source.contains(token))
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "{} must collect explicit overload payloads without resolver-global \
         scans, resolved-AST walks, or direct syntax parsing:\n{}",
        path.display(),
        violations.join("\n")
    );
}

#[test]
fn resolved_typed_ast_stays_on_checker_output_boundary() {
    let root = crate_root();
    let path = root.join("src/resolved_typed_ast.rs");
    let source = read_to_string(&path);
    let forbidden = [
        "mizar_resolve::env",
        "SymbolEnv",
        "ResolvedAst",
        "mizar_syntax::",
        "extern crate mizar_syntax",
        "std::fs::",
        "File::create",
        "OpenOptions",
        "artifact",
        "Artifact",
        "ClusterTrace::",
        "ClusterClosure",
        "fire_cluster",
        "replay",
    ];
    let violations = forbidden
        .iter()
        .copied()
        .filter(|token| source.contains(token))
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "{} must assemble from checker output tables without resolver-global \
         scans, direct syntax parsing, artifact emission, or cluster firing/replay:\n{}",
        path.display(),
        violations.join("\n")
    );
}

#[test]
fn checker_public_semantic_api_matches_documented_modules() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in checker_src_files(&root) {
        let source = read_to_string(&path);
        for (line_index, line) in source.lines().enumerate() {
            if public_declaration_name(line).is_some()
                && !public_checker_api_is_documented(&root, &path, line)
            {
                let display_path = path.strip_prefix(&root).unwrap_or(&path);
                violations.push(format!("{}:{}", display_path.display(), line_index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "public checker APIs require their owning module spec first:\n{}",
        violations.join("\n")
    );
}

#[test]
fn checker_public_enums_are_forward_compatible_and_documented() {
    let root = crate_root();
    let docs_root = workspace_root().join("doc/design/mizar-checker");
    let modules = [
        ("src/typed_ast.rs", "typed_ast.md"),
        ("src/binding_env.rs", "binding_env.md"),
        ("src/type_checker.rs", "type_checker.md"),
        (
            "src/registration_resolution.rs",
            "registration_resolution.md",
        ),
        ("src/cluster_trace.rs", "cluster_trace.md"),
        ("src/overload_resolution.rs", "overload_resolution.md"),
        ("src/resolved_typed_ast.rs", "resolved_typed_ast.md"),
    ];
    let mut violations = Vec::new();

    for (source_path, spec_name) in modules {
        let source_path = root.join(source_path);
        let source = read_to_string(&source_path);
        let public_enums = public_enums(&source);
        let public_enum_names = public_enums
            .iter()
            .map(|public_enum| public_enum.name.as_str())
            .collect::<BTreeSet<_>>();
        assert!(
            !public_enums.is_empty(),
            "{} should have checker-owned public enums for task-31 policy coverage",
            source_path.display()
        );

        let en_path = docs_root.join("en").join(spec_name);
        let ja_path = docs_root.join("ja").join(spec_name);
        let en_doc = read_to_string(&en_path);
        let ja_doc = read_to_string(&ja_path);
        let en_section = public_enum_policy_section(&en_doc).unwrap_or_else(|| {
            panic!(
                "{} must contain a Public Enum Policy section",
                en_path.display()
            )
        });
        let ja_section = public_enum_policy_section(&ja_doc).unwrap_or_else(|| {
            panic!(
                "{} must contain a Public Enum Policy section",
                ja_path.display()
            )
        });
        let en_policy_enums = public_enum_policy_entries(en_section);
        let ja_policy_enums = public_enum_policy_entries(ja_section);
        push_duplicate_policy_entries(&en_path, &en_policy_enums, &mut violations);
        push_duplicate_policy_entries(&ja_path, &ja_policy_enums, &mut violations);

        if !en_section.contains("No exhaustive public enum exceptions are owned by this module.") {
            violations.push(format!(
                "{}: Public Enum Policy must state there are no exhaustive public enum exceptions",
                en_path.display()
            ));
        }
        if !ja_section.contains("この module が所有する exhaustive public enum exception はない。")
        {
            violations.push(format!(
                "{}: Public Enum Policy must state there are no exhaustive public enum exceptions",
                ja_path.display()
            ));
        }

        for documented_enum in &en_policy_enums {
            if !public_enum_names.contains(documented_enum.as_str()) {
                violations.push(format!(
                    "{}: Public Enum Policy table must not include unknown `{}` row",
                    en_path.display(),
                    documented_enum
                ));
            }
        }
        for documented_enum in &ja_policy_enums {
            if !public_enum_names.contains(documented_enum.as_str()) {
                violations.push(format!(
                    "{}: Public Enum Policy table must not include unknown `{}` row",
                    ja_path.display(),
                    documented_enum
                ));
            }
        }

        for public_enum in &public_enums {
            if !public_enum.has_non_exhaustive {
                violations.push(format!(
                    "{}:{} public enum {} must be #[non_exhaustive]",
                    source_path.display(),
                    public_enum.line_number,
                    public_enum.name
                ));
            }
            if !en_policy_enums.contains(&public_enum.name) {
                violations.push(format!(
                    "{}: Public Enum Policy table must include `{}` as a first-column entry",
                    en_path.display(),
                    public_enum.name
                ));
            }
            if !ja_policy_enums.contains(&public_enum.name) {
                violations.push(format!(
                    "{}: Public Enum Policy table must include `{}` as a first-column entry",
                    ja_path.display(),
                    public_enum.name
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "checker public enum policy/source drift:\n{}",
        violations.join("\n")
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
fn checker_allow_exceptions_are_documented_inline() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in checker_rust_target_files(&root) {
        collect_undocumented_allows(&root, &path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "intentional lint allow exceptions need an adjacent reason:\n{}",
        violations.join("\n")
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
            lines.push(line);
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

fn collect_undocumented_allows(root: &Path, path: &Path, violations: &mut Vec<String>) {
    let source = read_to_string(path);

    for line_number in undocumented_allow_line_numbers(&source) {
        let display_path = path.strip_prefix(root).unwrap_or(path);
        violations.push(format!("{}:{line_number}", display_path.display()));
    }
}

fn public_declaration_name(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix("pub ")
        .or_else(|| trimmed.strip_prefix("pub("))?;
    rest.split(|character: char| {
        character.is_whitespace() || matches!(character, '(' | ')' | ':' | '<' | '{')
    })
    .find(|part| !part.is_empty())
}

fn public_checker_api_is_documented(root: &Path, path: &Path, line: &str) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    if matches!(
        relative,
        path if path == Path::new("src/typed_ast.rs")
            || path == Path::new("src/binding_env.rs")
            || path == Path::new("src/type_checker.rs")
            || path == Path::new("src/registration_resolution.rs")
            || path == Path::new("src/cluster_trace.rs")
            || path == Path::new("src/overload_resolution.rs")
            || path == Path::new("src/resolved_typed_ast.rs")
    ) {
        return true;
    }
    relative == Path::new("src/lib.rs")
        && matches!(
            line.trim(),
            "pub mod typed_ast;"
                | "pub mod binding_env;"
                | "pub mod type_checker;"
                | "pub mod registration_resolution;"
                | "pub mod cluster_trace;"
                | "pub mod overload_resolution;"
                | "pub mod resolved_typed_ast;"
        )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublicEnum {
    name: String,
    line_number: usize,
    has_non_exhaustive: bool,
}

fn public_enums(source: &str) -> Vec<PublicEnum> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut public_enums = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        let Some(name) = plain_public_enum_name(line) else {
            continue;
        };
        public_enums.push(PublicEnum {
            name: name.to_owned(),
            line_number: line_index + 1,
            has_non_exhaustive: has_non_exhaustive_attribute_before(&lines, line_index),
        });
    }

    public_enums
}

fn plain_public_enum_name(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("pub enum ")?;
    rest.split(|character: char| character.is_whitespace() || matches!(character, '<' | '{' | '('))
        .find(|part| !part.is_empty())
}

fn has_non_exhaustive_attribute_before(lines: &[&str], enum_line_index: usize) -> bool {
    for line in lines[..enum_line_index].iter().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "#[non_exhaustive]" {
            return true;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("#!") || trimmed.starts_with(']') {
            continue;
        }
        break;
    }

    false
}

fn public_enum_policy_section(document: &str) -> Option<&str> {
    let start = document
        .lines()
        .scan(0, |offset, line| {
            let current = *offset;
            *offset += line.len() + 1;
            Some((current, line))
        })
        .find_map(|(offset, line)| (line.trim() == "## Public Enum Policy").then_some(offset))?;
    let rest = &document[start..];
    let end = rest
        .lines()
        .scan(0, |offset, line| {
            let current = *offset;
            *offset += line.len() + 1;
            Some((current, line))
        })
        .skip(1)
        .find_map(|(offset, line)| {
            (line.starts_with("## ") && line.trim() != "## Public Enum Policy").then_some(offset)
        })
        .unwrap_or(rest.len());

    Some(&rest[..end])
}

fn public_enum_policy_entries(section: &str) -> Vec<String> {
    section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("| `")?;
            let (name, _) = rest.split_once("` |")?;
            Some(name.to_owned())
        })
        .collect()
}

fn push_duplicate_policy_entries(path: &Path, entries: &[String], violations: &mut Vec<String>) {
    let mut seen = BTreeSet::new();

    for entry in entries {
        if !seen.insert(entry) {
            violations.push(format!(
                "{}: Public Enum Policy table must not duplicate `{}` rows",
                path.display(),
                entry
            ));
        }
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

fn checker_rust_target_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for relative_dir in ["src", "tests", "benches", "examples"] {
        collect_rs_files(&root.join(relative_dir), &mut files);
    }
    let build_rs = root.join("build.rs");
    if build_rs.exists() {
        files.push(build_rs);
    }
    files.sort();
    files
}

fn checker_src_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rs_files(&root.join("src"), &mut files);
    files.sort();
    files
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}
