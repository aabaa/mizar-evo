use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy)]
struct PublicEnumDecision {
    relative_path: &'static str,
    enum_name: &'static str,
    spec_name: &'static str,
}

const PUBLIC_ENUM_DECISIONS: &[PublicEnumDecision] = &[
    PublicEnumDecision {
        relative_path: "src/declarations.rs",
        enum_name: "DeclarationShellKind",
        spec_name: "declarations.md",
    },
    PublicEnumDecision {
        relative_path: "src/declarations.rs",
        enum_name: "DeclarationShellVisibilityState",
        spec_name: "declarations.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "SymbolKind",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "Visibility",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "ExportStatus",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "SignatureShell",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "RelationKind",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "DefinitionKind",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "DeclarationConflictClass",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "RegistrationKind",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "LexicalSummaryKind",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "NamespaceNodeKind",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "NamespaceEdgeKind",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "NamespaceTarget",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "DependencyEndpoint",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "DeclarationDependencyKind",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/env.rs",
        enum_name: "ContributionKind",
        spec_name: "env.md",
    },
    PublicEnumDecision {
        relative_path: "src/imports.rs",
        enum_name: "ImportPathPrefix",
        spec_name: "imports.md",
    },
    PublicEnumDecision {
        relative_path: "src/imports.rs",
        enum_name: "ImportPathFailureClass",
        spec_name: "imports.md",
    },
    PublicEnumDecision {
        relative_path: "src/imports.rs",
        enum_name: "ImportGraphBuildError",
        spec_name: "imports.md",
    },
    PublicEnumDecision {
        relative_path: "src/labels.rs",
        enum_name: "LabelProjectionSource",
        spec_name: "labels.md",
    },
    PublicEnumDecision {
        relative_path: "src/labels.rs",
        enum_name: "LabelReferenceScope",
        spec_name: "labels.md",
    },
    PublicEnumDecision {
        relative_path: "src/labels.rs",
        enum_name: "LabelDiagnosticKind",
        spec_name: "labels.md",
    },
    PublicEnumDecision {
        relative_path: "src/names.rs",
        enum_name: "NamespaceResolutionOrigin",
        spec_name: "names.md",
    },
    PublicEnumDecision {
        relative_path: "src/names.rs",
        enum_name: "NamespaceFailureClass",
        spec_name: "names.md",
    },
    PublicEnumDecision {
        relative_path: "src/names.rs",
        enum_name: "NamespacePartialOrigin",
        spec_name: "names.md",
    },
    PublicEnumDecision {
        relative_path: "src/names.rs",
        enum_name: "NameProjectionSource",
        spec_name: "names.md",
    },
    PublicEnumDecision {
        relative_path: "src/names.rs",
        enum_name: "NameReferenceScope",
        spec_name: "names.md",
    },
    PublicEnumDecision {
        relative_path: "src/names.rs",
        enum_name: "NameDiagnosticRole",
        spec_name: "names.md",
    },
    PublicEnumDecision {
        relative_path: "src/names.rs",
        enum_name: "NameDiagnosticKind",
        spec_name: "names.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "RecoveryState",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "NodeResolutionState",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "NodeReferenceKey",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "ResolvedArenaError",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "NameLookupClass",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "NameResolution",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "LabelKind",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "LabelExpectation",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "LabelResolution",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "ImportResolution",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "ImportFailureClass",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "ExportFailureClass",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "ExportTarget",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/resolved_ast.rs",
        enum_name: "ResolvedAstError",
        spec_name: "resolved_ast.md",
    },
    PublicEnumDecision {
        relative_path: "src/symbols.rs",
        enum_name: "SymbolOverloadPolicy",
        spec_name: "symbols.md",
    },
    PublicEnumDecision {
        relative_path: "src/symbols.rs",
        enum_name: "SymbolDiagnosticClass",
        spec_name: "symbols.md",
    },
];

const PUBLIC_ENUM_DECISION_SOURCE_FILES: &[&str] = &[
    "src/declarations.rs",
    "src/env.rs",
    "src/imports.rs",
    "src/labels.rs",
    "src/names.rs",
    "src/resolved_ast.rs",
    "src/symbols.rs",
];

#[test]
fn resolve_manifest_opts_into_workspace_lints() {
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
fn resolve_allow_exceptions_are_documented_inline() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in resolve_rust_target_files(&root) {
        collect_undocumented_allows(&root, &path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "intentional lint allow exceptions need an adjacent reason:\n{}",
        violations.join("\n")
    );
}

#[test]
fn public_resolver_enums_are_marked_non_exhaustive_and_documented() {
    let root = crate_root();
    let specs_root = workspace_root().join("doc/design/mizar-resolve");
    let expected_keys = PUBLIC_ENUM_DECISIONS
        .iter()
        .map(|decision| {
            (
                decision.relative_path.to_owned(),
                decision.enum_name.to_owned(),
            )
        })
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();

    for decision in PUBLIC_ENUM_DECISIONS {
        collect_public_enum_decision_violations(&root, &specs_root, decision, &mut violations);
    }
    collect_unlisted_public_enum_decisions(&root, &expected_keys, &mut violations);

    assert!(
        violations.is_empty(),
        "task 26 public enum forward-compatibility policy requires \
         #[non_exhaustive] and owning-spec decisions for resolver-owned \
         public enums:\n{}",
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

#[test]
fn allow_scanner_keeps_multiline_cfg_attr_open_past_bracket_in_string() {
    let source = "#[\n    cfg_attr(\n        test,\n        doc = \"close ] bracket\",\n        allow(dead_code)\n    )\n]\nfn sample() {}\n";

    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);
}

#[test]
fn allow_scanner_keeps_multiline_cfg_attr_open_past_bracket_in_comment() {
    let source = "#[cfg_attr(\n    test, // ]\n    allow(dead_code)\n)]\nfn sample() {}\n";

    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);
}

#[test]
fn allow_scanner_keeps_multiline_cfg_attr_open_past_bracket_in_raw_string() {
    let source = "#[cfg_attr(\n    test,\n    doc = r#\"close \" ] bracket\"#,\n    allow(dead_code)\n)]\nfn sample() {}\n";

    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);
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

#[test]
fn manifest_target_path_parser_covers_custom_cargo_targets() {
    let root = Path::new("crate");
    let manifest = r#"
[package]
build = "build/custom.rs"

[lib] # library target
path = "lib/custom.rs"

[[bin]] # binary target
name = "resolve-tool"
path = "tools/resolve_tool.rs"

[[test]]
name = "custom-test"
path = "custom/test_entry.rs"
"#;

    assert_eq!(
        explicit_manifest_target_paths_from_manifest(root, manifest),
        vec![
            PathBuf::from("crate/build/custom.rs"),
            PathBuf::from("crate/lib/custom.rs"),
            PathBuf::from("crate/tools/resolve_tool.rs"),
            PathBuf::from("crate/custom/test_entry.rs"),
        ]
    );
}

#[test]
fn default_cargo_target_directories_are_scanned() {
    let root =
        std::env::temp_dir().join(format!("mizar_resolve_lint_policy_{}", std::process::id()));
    remove_dir_if_exists(&root);
    create_dir(&root.join("src"));
    create_dir(&root.join("benches"));
    create_dir(&root.join("examples"));
    write_test_file(&root.join("Cargo.toml"), "[package]\nname = \"fixture\"\n");
    write_test_file(&root.join("src/lib.rs"), "fn lib() {}\n");
    write_test_file(&root.join("build.rs"), "fn main() {}\n");
    write_test_file(&root.join("benches/resolve.rs"), "fn bench() {}\n");
    write_test_file(&root.join("examples/resolve.rs"), "fn example() {}\n");

    let files = resolve_rust_target_files(&root);

    assert!(files.contains(&root.join("src/lib.rs")));
    assert!(files.contains(&root.join("build.rs")));
    assert!(files.contains(&root.join("benches/resolve.rs")));
    assert!(files.contains(&root.join("examples/resolve.rs")));
    remove_dir_if_exists(&root);
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

fn collect_public_enum_decision_violations(
    root: &Path,
    specs_root: &Path,
    decision: &PublicEnumDecision,
    violations: &mut Vec<String>,
) {
    let path = root.join(decision.relative_path);
    let source = read_to_string(&path);
    if !source_contains_public_enum(&source, decision.enum_name) {
        violations.push(format!(
            "{}: missing pub enum {}",
            decision.relative_path, decision.enum_name
        ));
        return;
    }

    if !enum_has_attribute(&source, decision.enum_name, "non_exhaustive") {
        violations.push(format!(
            "{}: pub enum {} lacks #[non_exhaustive]",
            decision.relative_path, decision.enum_name
        ));
    }

    for language in ["en", "ja"] {
        let spec_path = specs_root.join(language).join(decision.spec_name);
        let spec = read_to_string(&spec_path);
        if spec_contains_public_enum_decision(&spec, decision.enum_name) {
            continue;
        }
        violations.push(format!(
            "doc/design/mizar-resolve/{language}/{}: missing R-026 decision for {}",
            decision.spec_name, decision.enum_name
        ));
    }
}

fn collect_unlisted_public_enum_decisions(
    root: &Path,
    expected_keys: &BTreeSet<(String, String)>,
    violations: &mut Vec<String>,
) {
    for relative_source_file in PUBLIC_ENUM_DECISION_SOURCE_FILES {
        let path = root.join(relative_source_file);
        let source = read_to_string(&path);
        let lines = source.lines().collect::<Vec<_>>();
        let relative_path = (*relative_source_file).to_owned();

        for (line_index, line) in lines.iter().enumerate() {
            if declaration_has_preceding_attribute(&lines, line_index, "cfg(test)") {
                continue;
            }
            let Some(enum_name) = public_enum_name(line) else {
                continue;
            };
            if expected_keys.contains(&(relative_path.clone(), enum_name.to_owned())) {
                continue;
            }

            violations.push(format!(
                "{relative_path}:{} pub enum {enum_name} has no R-026 decision",
                line_index + 1
            ));
        }
    }
}

fn public_enum_name(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("pub enum ")?;
    rest.split(|character: char| character != '_' && !character.is_ascii_alphanumeric())
        .next()
        .filter(|name| !name.is_empty())
}

fn source_contains_public_enum(source: &str, enum_name: &str) -> bool {
    source
        .lines()
        .any(|line| public_enum_name(line) == Some(enum_name))
}

fn enum_has_attribute(source: &str, enum_name: &str, attribute_name: &str) -> bool {
    let lines = source.lines().collect::<Vec<_>>();

    for (line_index, line) in lines.iter().enumerate() {
        if public_enum_name(line) == Some(enum_name) {
            return declaration_has_preceding_attribute(&lines, line_index, attribute_name);
        }
    }

    false
}

fn spec_contains_public_enum_decision(spec: &str, enum_name: &str) -> bool {
    let bullet = format!("- `{enum_name}`");
    spec.lines().any(|line| line.trim() == bullet)
}

fn declaration_has_preceding_attribute(
    lines: &[&str],
    declaration_line_index: usize,
    attribute_name: &str,
) -> bool {
    let mut line_index = declaration_line_index;

    while let Some(previous_index) = line_index.checked_sub(1) {
        let previous_line = lines[previous_index].trim();
        if previous_line.is_empty() || previous_line.starts_with("///") {
            line_index = previous_index;
            continue;
        }
        if starts_attribute(lines[previous_index]) {
            let (attribute, _) = attribute_block(lines, previous_index);
            if attribute_is(&attribute, attribute_name) {
                return true;
            }
            line_index = previous_index;
            continue;
        }
        break;
    }

    false
}

fn attribute_is(attribute: &str, attribute_name: &str) -> bool {
    let compact = compact_attribute_tokens(attribute);
    let outer = format!("#[{attribute_name}");
    let inner = format!("#![{attribute_name}");

    compact.starts_with(&outer) || compact.starts_with(&inner)
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

fn has_adjacent_allow_rationale(lines: &[&str], start: usize, end: usize) -> bool {
    let previous_line_has_reason = start
        .checked_sub(1)
        .is_some_and(|index| line_comment_contains_reason(lines[index]));
    let same_line_has_reason =
        (start..=end).any(|index| line_comment_contains_reason(lines[index]));
    let next_line_has_reason = lines
        .get(end + 1)
        .is_some_and(|line| line_comment_contains_reason(line));

    previous_line_has_reason || same_line_has_reason || next_line_has_reason
}

fn line_comment_contains_reason(line: &str) -> bool {
    line_comment_text(line).is_some_and(|comment| {
        let lower = comment.to_ascii_lowercase();
        lower.contains("reason:") || lower.contains("rationale:")
    })
}

fn line_comment_text(line: &str) -> Option<&str> {
    let mut characters = line.char_indices().peekable();
    let mut quote = None;
    let mut escaped = false;
    let mut in_block_comment = false;

    while let Some((index, character)) = characters.next() {
        if in_block_comment {
            if character == '*' && characters.next_if(|(_, next)| *next == '/').is_some() {
                in_block_comment = false;
            }
            continue;
        }

        if let Some(current_quote) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == current_quote {
                quote = None;
            }
            continue;
        }

        if character == '/' {
            if characters.next_if(|(_, next)| *next == '/').is_some() {
                return line.get(index + 2..);
            }
            if characters.next_if(|(_, next)| *next == '*').is_some() {
                in_block_comment = true;
            }
        } else if character == '"' || character == '\'' {
            quote = Some(character);
        }
    }

    None
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

fn resolve_rust_target_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    collect_rust_files(&root.join("src"), &mut files);
    collect_rust_files(&root.join("tests"), &mut files);
    collect_rust_files(&root.join("benches"), &mut files);
    collect_rust_files(&root.join("examples"), &mut files);
    let default_build_script = root.join("build.rs");
    if default_build_script.exists() {
        files.push(default_build_script);
    }
    add_explicit_manifest_target_files(root, &mut files);
    files.sort();
    files.dedup();
    files
}

fn add_explicit_manifest_target_files(root: &Path, files: &mut Vec<PathBuf>) {
    for target_path in explicit_manifest_target_paths(root) {
        if let Some(parent) = target_path.parent()
            && parent.exists()
        {
            collect_rust_files(parent, files);
        }
        files.push(target_path);
    }
}

fn explicit_manifest_target_paths(root: &Path) -> Vec<PathBuf> {
    let manifest = read_to_string(&root.join("Cargo.toml"));
    explicit_manifest_target_paths_from_manifest(root, &manifest)
}

fn explicit_manifest_target_paths_from_manifest(root: &Path, manifest: &str) -> Vec<PathBuf> {
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
    rest.split_once(quote).map(|(quoted, _)| quoted)
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) {
    if !directory.exists() || directory.file_name().and_then(|name| name.to_str()) == Some("target")
    {
        return;
    }

    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()));

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read {} entry: {error}", directory.display())
        });
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

fn create_dir(path: &Path) {
    fs::create_dir_all(path)
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", path.display()));
}

fn write_test_file(path: &Path, contents: &str) {
    fs::write(path, contents)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
}

fn remove_dir_if_exists(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|error| panic!("failed to remove {}: {error}", path.display()));
    }
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(Path::parent)
        .expect("crate must live under workspace/crates")
        .to_path_buf()
}
