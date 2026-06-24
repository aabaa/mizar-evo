use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn vc_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_vc_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let occurrences = manifest.matches("\"crates/mizar-vc\"").count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-vc exactly once as a workspace member",
        manifest_path.display()
    );
}

#[test]
fn vc_manifest_keeps_task_one_package_metadata() {
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
            .any(|line| assignment_is(line, "name", "mizar_vc")),
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
fn vc_manifest_dependency_boundary_is_task_one_minimal() {
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
        "{} must depend only on mizar-core and mizar-session, with no dev/build/target \
         dependency escape hatch, until a task-scoped VC spec expands the crate boundary",
        manifest_path.display()
    );
}

#[test]
fn vc_lib_exposes_only_current_spec_backed_modules() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);
    let declarations = public_semantic_declarations(&source);
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
        declarations,
        ["6: pub mod vc_ir;"],
        "{} must expose only the task-3 `vc_ir` module until later module \
         specs exist; found:\n{}",
        lib_path.display(),
        declarations.join("\n")
    );
    assert_eq!(
        source_files,
        ["src/lib.rs", "src/vc_ir.rs"],
        "task 3 owns exactly the vc_ir source module; later private VC modules \
         must wait for their task-scoped specs, found {source_files:?}"
    );
    for spec in [
        workspace_root().join("doc/design/mizar-vc/en/vc_ir.md"),
        workspace_root().join("doc/design/mizar-vc/ja/vc_ir.md"),
    ] {
        assert!(
            spec.exists(),
            "{} must exist before src/vc_ir.rs is exposed",
            spec.display()
        );
    }
    for forbidden in [
        "pub use ",
        "pub(crate) ",
        "pub(super) ",
        "pub(in ",
        "crate::",
        "mizar_core::",
        "mizar_session::",
    ] {
        assert!(
            !source.contains(forbidden),
            "{} must remain a module-registration library shell; found \
             forbidden token `{forbidden}`",
            lib_path.display()
        );
    }
}

#[test]
fn vc_ir_public_enums_are_forward_compatible() {
    let source_path = crate_root().join("src/vc_ir.rs");
    let source = read_to_string(&source_path);
    let lines = source.lines().collect::<Vec<_>>();
    let mut violations = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("pub enum ")
            && !previous_attribute_is_non_exhaustive(&lines, line_index)
        {
            violations.push(format!("{}:{}", source_path.display(), line_index + 1));
        }
    }

    assert!(
        violations.is_empty(),
        "public vc_ir enums must stay #[non_exhaustive] until task 17 records \
         a different public-enum policy:\n{}",
        violations.join("\n")
    );
}

#[test]
fn vc_source_has_no_undocumented_lint_allows() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in rust_source_files(&root) {
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
        "#![allow(dead_code)]",
        "#![ allow(dead_code)]",
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
    ["dependencies", "dev-dependencies", "build-dependencies"]
        .iter()
        .any(|dependency_kind| {
            section_name == *dependency_kind
                || section_name.starts_with(&format!("{dependency_kind}."))
                || section_name.ends_with(&format!(".{dependency_kind}"))
                || section_name.contains(&format!(".{dependency_kind}."))
        })
}

fn assignment_is(line: &str, key: &str, expected: &str) -> bool {
    let Some((actual_key, actual_value)) = line.split_once('=') else {
        return false;
    };

    actual_key.trim() == key && actual_value.trim().trim_matches('"') == expected
}

fn public_semantic_declarations(source: &str) -> Vec<String> {
    let mut declarations = Vec::new();

    for (line_index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if public_semantic_declaration(trimmed) {
            declarations.push(format!("{}: {trimmed}", line_index + 1));
        }
    }

    declarations
}

fn public_semantic_declaration(line: &str) -> bool {
    if line.starts_with("#[macro_export]") {
        return true;
    }

    if !line.starts_with("pub ") {
        return false;
    }

    [
        "pub mod ",
        "pub use ",
        "pub struct ",
        "pub enum ",
        "pub trait ",
        "pub type ",
        "pub fn ",
        "pub const ",
        "pub static ",
        "pub macro ",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
        || public_function_with_qualifier(line)
}

fn public_function_with_qualifier(line: &str) -> bool {
    let mut words = line.split_whitespace();

    if words.next() != Some("pub") {
        return false;
    }

    words.any(|word| {
        word == "fn"
            || word
                .strip_prefix("fn")
                .is_some_and(|rest| rest.starts_with('('))
    })
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(path: &Path, files: &mut Vec<PathBuf>) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };

    if metadata.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path.to_path_buf());
        }
        return;
    }

    if !metadata.is_dir() {
        return;
    }

    let entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    for entry in entries {
        let entry_path = entry.path();
        if entry_path.file_name().is_some_and(|name| name == "target") {
            continue;
        }
        collect_rust_source_files(&entry_path, files);
    }
}

fn collect_undocumented_allows(root: &Path, path: &Path, violations: &mut Vec<String>) {
    let source = read_to_string(path);

    for line_number in undocumented_allow_line_numbers(&source) {
        let display_path = path.strip_prefix(root).unwrap_or(path);
        violations.push(format!("{}:{line_number}", display_path.display()));
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

fn has_adjacent_allow_rationale(lines: &[&str], start: usize, end: usize) -> bool {
    lines[start..=end]
        .iter()
        .any(|line| line.contains("reason:"))
        || lines
            .get(end + 1)
            .is_some_and(|line| line.trim_start().starts_with("// reason:"))
}

fn previous_attribute_is_non_exhaustive(lines: &[&str], line_index: usize) -> bool {
    lines[..line_index]
        .iter()
        .rev()
        .find(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("///")
        })
        .is_some_and(|line| line.trim() == "#[non_exhaustive]")
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
        if character == '"' {
            let mut closing = characters.clone();
            let mut matched = true;
            for _ in 0..hashes {
                if closing.next_if_eq(&'#').is_none() {
                    matched = false;
                    break;
                }
            }
            if matched {
                for _ in 0..hashes {
                    characters.next();
                }
                break;
            }
        }
    }

    true
}
