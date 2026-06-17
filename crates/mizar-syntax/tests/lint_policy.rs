use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn syntax_manifest_opts_into_workspace_lints() {
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
fn syntax_allow_exceptions_are_documented_inline() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in syntax_rust_target_files(&root) {
        collect_undocumented_allows(&root, &path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "intentional lint allow exceptions need an adjacent reason:\n{}",
        violations.join("\n")
    );
}

#[test]
fn public_forward_compatible_enums_are_marked_non_exhaustive() {
    let root = crate_root();
    let expected = [
        ("src/ast.rs", "SyntaxKind"),
        ("src/ast.rs", "SurfaceNodeKind"),
        ("src/ast.rs", "SurfaceTokenKind"),
        ("src/recovery.rs", "SyntaxRecoveryKind"),
        ("src/recovery.rs", "SyntaxDiagnosticCode"),
        ("src/trivia.rs", "TriviaAttachmentTarget"),
        ("src/trivia.rs", "SkippedTokenReason"),
        ("src/trivia.rs", "WhitespaceHintKind"),
    ];

    let mut violations = Vec::new();
    for (relative_path, enum_name) in expected {
        let path = root.join(relative_path);
        let source = read_to_string(&path);
        if !enum_has_attribute(&source, enum_name, "non_exhaustive") {
            violations.push(format!("{relative_path}: pub enum {enum_name}"));
        }
    }

    assert!(
        violations.is_empty(),
        "public enum forward-compatibility policy requires \
         #[non_exhaustive] on:\n{}",
        violations.join("\n")
    );
}

#[test]
fn public_enum_exhaustiveness_exceptions_are_documented() {
    let root = crate_root();
    let documented_exceptions = [
        (
            "src/ast.rs",
            "MizarLanguage",
            "../../../../doc/design/mizar-syntax/en/ast.md",
        ),
        (
            "src/ast.rs",
            "SurfaceOperatorAssociativity",
            "../../../../doc/design/mizar-syntax/en/ast.md",
        ),
        (
            "src/ast.rs",
            "SurfaceFormulaPrefixOperator",
            "../../../../doc/design/mizar-syntax/en/ast.md",
        ),
        (
            "src/ast.rs",
            "SurfaceFormulaConnective",
            "../../../../doc/design/mizar-syntax/en/ast.md",
        ),
        (
            "src/ast.rs",
            "SurfaceQuantifierKind",
            "../../../../doc/design/mizar-syntax/en/ast.md",
        ),
        (
            "src/ast.rs",
            "SurfaceFormulaConstant",
            "../../../../doc/design/mizar-syntax/en/ast.md",
        ),
        (
            "src/trivia.rs",
            "TriviaPlacement",
            "../../../../doc/design/mizar-syntax/en/trivia.md",
        ),
    ];

    let mut violations = Vec::new();
    for (relative_path, enum_name, spec_link) in documented_exceptions {
        let source = read_to_string(&root.join(relative_path));
        if enum_has_attribute(&source, enum_name, "non_exhaustive") {
            violations.push(format!(
                "{relative_path}: pub enum {enum_name} is listed as an exhaustive exception"
            ));
        }
        if !source_contains_public_enum(&source, enum_name) {
            violations.push(format!("{relative_path}: missing pub enum {enum_name}"));
        }
        let spec =
            read_to_string(&workspace_root().join(spec_link.trim_start_matches("../../../../")));
        if !spec.contains(enum_name) || !spec.contains("exhaustive") {
            violations.push(format!(
                "{relative_path}: pub enum {enum_name} needs an exhaustive decision in {spec_link}"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "intentional public enum exhaustiveness exceptions must stay explicit:\n{}",
        violations.join("\n")
    );
}

#[test]
fn every_public_enum_has_a_forward_compatibility_decision() {
    let root = crate_root();
    let forward_compatible = [
        "SyntaxKind",
        "SurfaceNodeKind",
        "SurfaceTokenKind",
        "SyntaxRecoveryKind",
        "SyntaxDiagnosticCode",
        "TriviaAttachmentTarget",
        "SkippedTokenReason",
        "WhitespaceHintKind",
    ];
    let exhaustive_exceptions = [
        "MizarLanguage",
        "SurfaceFormulaConnective",
        "SurfaceFormulaConstant",
        "SurfaceFormulaPrefixOperator",
        "SurfaceOperatorAssociativity",
        "SurfaceQuantifierKind",
        "TriviaPlacement",
    ];
    let mut classified = forward_compatible
        .iter()
        .chain(exhaustive_exceptions.iter())
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    classified.sort_unstable();

    let mut discovered = Vec::new();
    for relative_path in ["src/ast.rs", "src/recovery.rs", "src/trivia.rs"] {
        let source = read_to_string(&root.join(relative_path));
        for enum_name in public_enum_names(&source) {
            discovered.push(enum_name);
        }
    }
    discovered.sort_unstable();

    assert_eq!(
        discovered, classified,
        "each public mizar-syntax enum must be classified as #[non_exhaustive] \
         or as a documented exhaustive exception"
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

#[test]
fn allow_rationale_must_be_in_comment_text() {
    let source = "#[allow(dead_code)]\nconst REASON: &str = \"not a comment\";\n";
    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);

    let source = "#[allow(dead_code)]\nconst S: &str = \"// reason: not a comment\";\n";
    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);

    let source = "#[allow(dead_code)] // reason: compatibility fixture\nfn sample() {}\n";
    assert!(undocumented_allow_line_numbers(source).is_empty());
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

fn enum_has_attribute(source: &str, enum_name: &str, attribute_name: &str) -> bool {
    let enum_declaration = format!("pub enum {enum_name}");
    let lines = source.lines().collect::<Vec<_>>();

    for (line_index, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with(&enum_declaration) {
            return preceding_attributes(&lines, line_index)
                .iter()
                .any(|attribute| attribute_is(attribute, attribute_name));
        }
    }

    false
}

fn source_contains_public_enum(source: &str, enum_name: &str) -> bool {
    let enum_declaration = format!("pub enum {enum_name}");
    source
        .lines()
        .any(|line| line.trim_start().starts_with(&enum_declaration))
}

fn public_enum_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("pub enum "))
        .filter_map(|rest| {
            rest.split(|character: char| !character.is_alphanumeric() && character != '_')
                .next()
        })
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

fn preceding_attributes(lines: &[&str], declaration_line_index: usize) -> Vec<String> {
    let mut attributes = Vec::new();
    let mut line_index = declaration_line_index;

    while line_index > 0 {
        let previous_line_index = line_index - 1;
        if !starts_attribute(lines[previous_line_index]) {
            break;
        }
        let (attribute, _) = attribute_block(lines, previous_line_index);
        attributes.push(attribute);
        line_index = previous_line_index;
    }

    attributes
}

fn attribute_is(attribute: &str, attribute_name: &str) -> bool {
    let compact = compact_attribute_tokens(attribute);

    compact == format!("#[{attribute_name}]") || compact == format!("#![{attribute_name}]")
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

fn syntax_rust_target_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    collect_rust_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if dir.file_name().and_then(|name| name.to_str()) == Some("target") {
        return;
    }

    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()));

    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read {} entry: {error}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    let root = crate_root();
    root.parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("mizar-syntax crate must live under crates/")
}
