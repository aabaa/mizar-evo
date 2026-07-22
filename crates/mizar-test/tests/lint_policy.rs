use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn mizar_test_manifest_opts_into_workspace_lints() {
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
fn mizar_test_allow_exceptions_are_documented_inline() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in mizar_test_rust_target_files(&root) {
        collect_undocumented_allows(&root, &path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "intentional lint allow exceptions need an adjacent reason:\n{}",
        violations.join("\n")
    );
}

#[test]
fn public_enums_are_non_exhaustive_and_documented() {
    let root = crate_root();
    let workspace = workspace_root();
    let policies = public_enum_policies();
    let expected = policies
        .iter()
        .map(|policy| (policy.module.to_owned(), policy.name.to_owned()))
        .collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();

    let mut source_paths = Vec::new();
    collect_rust_files(&root.join("src"), &mut source_paths);
    source_paths.sort();

    for source_path in source_paths {
        let Some(module) = source_path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let source = read_to_string(&source_path);
        for enum_name in public_enum_names(&source) {
            observed.insert((module.to_owned(), enum_name));
        }
    }

    assert_eq!(
        observed, expected,
        "public enum policy inventory must match crates/mizar-test/src"
    );

    for policy in policies {
        let source_path = root.join("src").join(format!("{}.rs", policy.module));
        let source = read_to_string(&source_path);
        assert!(
            public_enum_has_non_exhaustive(&source, policy.name),
            "{} must mark public enum {} as #[non_exhaustive]",
            source_path.display(),
            policy.name
        );

        for doc_path in [policy.en_doc, policy.ja_doc] {
            let doc_path = workspace.join(doc_path);
            let doc = read_to_string(&doc_path);
            assert!(
                doc_has_public_enum_policy(&doc, policy.name),
                "{} must document {} as #[non_exhaustive]",
                doc_path.display(),
                policy.name
            );
        }
    }
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
fn allow_scanner_does_not_skip_after_non_allow_attribute_with_bracket_in_string() {
    let source = "#[doc = \"open [bracket\"]\n#[allow(dead_code)]\nfn sample() {}\n";

    assert_eq!(undocumented_allow_line_numbers(source), vec![2]);
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
    let source = "#[allow(dead_code)]\nconst reason: &str = \"not a comment\";\n";
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
name = "mizar-test"
path = "tools/mizar_test.rs"

[[test]]
name = "custom-test"
path = "custom/test_entry.rs"
"#;

    assert_eq!(
        explicit_manifest_target_paths_from_manifest(root, manifest),
        vec![
            PathBuf::from("crate/build/custom.rs"),
            PathBuf::from("crate/lib/custom.rs"),
            PathBuf::from("crate/tools/mizar_test.rs"),
            PathBuf::from("crate/custom/test_entry.rs"),
        ]
    );
}

#[test]
fn explicit_manifest_target_parent_directory_is_scanned_for_modules() {
    let root = std::env::temp_dir().join(format!("mizar_test_lint_policy_{}", std::process::id()));
    remove_dir_if_exists(&root);
    create_dir(&root.join("tools"));
    write_test_file(
        &root.join("Cargo.toml"),
        r#"
[[bin]]
name = "mizar-test"
path = "tools/mizar_test.rs"
"#,
    );
    write_test_file(
        &root.join("tools/mizar_test.rs"),
        "mod helper;\nfn main() {}\n",
    );
    write_test_file(
        &root.join("tools/helper.rs"),
        "#[allow(dead_code)]\nfn helper() {}\n",
    );

    let files = mizar_test_rust_target_files(&root);

    assert!(files.contains(&root.join("tools/helper.rs")));
    remove_dir_if_exists(&root);
}

#[test]
fn implicit_cargo_target_locations_are_scanned() {
    let root = std::env::temp_dir().join(format!(
        "mizar_test_lint_policy_implicit_{}",
        std::process::id()
    ));
    remove_dir_if_exists(&root);
    create_dir(&root.join("examples"));
    create_dir(&root.join("benches"));
    write_test_file(&root.join("Cargo.toml"), "[package]\nname = \"sample\"\n");
    write_test_file(&root.join("build.rs"), "fn main() {}\n");
    write_test_file(&root.join("examples/demo.rs"), "fn main() {}\n");
    write_test_file(&root.join("benches/smoke.rs"), "fn main() {}\n");

    let files = mizar_test_rust_target_files(&root);

    assert!(files.contains(&root.join("build.rs")));
    assert!(files.contains(&root.join("examples/demo.rs")));
    assert!(files.contains(&root.join("benches/smoke.rs")));
    remove_dir_if_exists(&root);
}

#[test]
fn allow_scanner_ignores_attribute_text_inside_multiline_strings() {
    let ordinary_string = "const S: &str = \"\n#[allow(dead_code)]\n\";\n";
    assert!(undocumented_allow_line_numbers(ordinary_string).is_empty());

    let lifetime_string = "const S: &'static str = \"\n#[allow(dead_code)]\n\";\n";
    assert!(undocumented_allow_line_numbers(lifetime_string).is_empty());

    let raw_string = r##"const S: &str = r#"
#[allow(dead_code)]
"#;
"##;
    assert!(undocumented_allow_line_numbers(raw_string).is_empty());
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

#[derive(Clone, Copy)]
struct PublicEnumPolicy {
    module: &'static str,
    name: &'static str,
    en_doc: &'static str,
    ja_doc: &'static str,
}

fn public_enum_policies() -> Vec<PublicEnumPolicy> {
    vec![
        PublicEnumPolicy {
            module: "diagnostic",
            name: "ValidationSeverity",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "expectation",
            name: "TestKind",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "expectation",
            name: "ExpectedOutcome",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "expectation",
            name: "PipelinePhase",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "expectation",
            name: "Architecture22Gate",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "harness",
            name: "TestProfile",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "harness",
            name: "ValidationMode",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "harness",
            name: "HarnessError",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "runner",
            name: "ParseOnlyCaseStatus",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "runner",
            name: "DeclarationSymbolCaseStatus",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "runner",
            name: "TypeElaborationCaseStatus",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "runner",
            name: "ProofVerificationCaseStatus",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotKind",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "ParallelismProfile",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotUpdateReason",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotUpdateMode",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotBaselineStatus",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotBaselineError",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotError",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "staged_model",
            name: "Stage",
            en_doc: "doc/design/mizar-test/en/staged_model.md",
            ja_doc: "doc/design/mizar-test/ja/staged_model.md",
        },
        PublicEnumPolicy {
            module: "toml_lite",
            name: "TomlValue",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "traceability",
            name: "RequirementStatus",
            en_doc: "doc/design/mizar-test/en/traceability.md",
            ja_doc: "doc/design/mizar-test/ja/traceability.md",
        },
        PublicEnumPolicy {
            module: "traceability",
            name: "CoverageShape",
            en_doc: "doc/design/mizar-test/en/traceability.md",
            ja_doc: "doc/design/mizar-test/ja/traceability.md",
        },
    ]
}

fn public_enum_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("pub enum "))
        .filter_map(|rest| {
            rest.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                .next()
        })
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

fn public_enum_has_non_exhaustive(source: &str, name: &str) -> bool {
    let Some(position) = source.find(&format!("pub enum {name}")) else {
        return false;
    };
    for line in source[..position].lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with("#[") {
            return false;
        }
        if trimmed == "#[non_exhaustive]" {
            return true;
        }
    }
    false
}

fn doc_has_public_enum_policy(doc: &str, name: &str) -> bool {
    let row_prefix = format!("| `{name}` |");
    doc.lines()
        .any(|line| line.contains(&row_prefix) && line.contains("`#[non_exhaustive]`"))
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
    let line_starts_in_code = line_start_code_states(&lines);
    let mut violations = Vec::new();
    let mut line_index = 0;

    while line_index < lines.len() {
        if !line_starts_in_code[line_index] || !starts_attribute(lines[line_index]) {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LexState {
    Code,
    String { quote: u8, escaped: bool },
    RawString { hashes: usize },
    BlockComment { depth: usize },
}

fn line_start_code_states(lines: &[&str]) -> Vec<bool> {
    let mut state = LexState::Code;
    let mut starts = Vec::with_capacity(lines.len());

    for line in lines {
        starts.push(state == LexState::Code);
        advance_lex_state(line, &mut state);
    }

    starts
}

fn advance_lex_state(line: &str, state: &mut LexState) {
    let bytes = line.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match *state {
            LexState::Code => {
                if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'/') {
                    break;
                }
                if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
                    *state = LexState::BlockComment { depth: 1 };
                    index += 2;
                    continue;
                }
                if bytes[index] == b'r'
                    && let Some((hashes, consumed)) = raw_string_start(bytes, index)
                {
                    *state = LexState::RawString { hashes };
                    index += consumed;
                    continue;
                }
                if bytes[index] == b'"' {
                    *state = LexState::String {
                        quote: bytes[index],
                        escaped: false,
                    };
                }
                index += 1;
            }
            LexState::String { quote, escaped } => {
                if escaped {
                    *state = LexState::String {
                        quote,
                        escaped: false,
                    };
                } else if bytes[index] == b'\\' {
                    *state = LexState::String {
                        quote,
                        escaped: true,
                    };
                } else if bytes[index] == quote {
                    *state = LexState::Code;
                }
                index += 1;
            }
            LexState::RawString { hashes } => {
                if raw_string_end(bytes, index, hashes) {
                    *state = LexState::Code;
                    index += 1 + hashes;
                } else {
                    index += 1;
                }
            }
            LexState::BlockComment { mut depth } => {
                if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
                    depth += 1;
                    *state = LexState::BlockComment { depth };
                    index += 2;
                } else if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                    depth -= 1;
                    *state = if depth == 0 {
                        LexState::Code
                    } else {
                        LexState::BlockComment { depth }
                    };
                    index += 2;
                } else {
                    index += 1;
                }
            }
        }
    }
}

fn raw_string_start(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    if bytes.get(start) != Some(&b'r') {
        return None;
    }

    let mut index = start + 1;
    let mut hashes = 0;
    while bytes.get(index) == Some(&b'#') {
        hashes += 1;
        index += 1;
    }
    if bytes.get(index) != Some(&b'"') {
        return None;
    }

    Some((hashes, index - start + 1))
}

fn raw_string_end(bytes: &[u8], start: usize, hashes: usize) -> bool {
    bytes.get(start) == Some(&b'"')
        && (0..hashes).all(|offset| bytes.get(start + 1 + offset) == Some(&b'#'))
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

fn mizar_test_rust_target_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    collect_rust_files(root, &mut files);
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
    let root = crate_root();
    root.parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("mizar-test crate must live under crates/")
}
