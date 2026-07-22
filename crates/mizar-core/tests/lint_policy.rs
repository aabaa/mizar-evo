use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[test]
fn core_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_core_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let occurrences = manifest.matches("\"crates/mizar-core\"").count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-core exactly once as a workspace member",
        manifest_path.display()
    );
}

#[test]
fn core_manifest_keeps_task_one_package_metadata() {
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
            .any(|line| assignment_is(line, "name", "mizar_core")),
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
fn core_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "mizar-checker = { path = \"../mizar-checker\" }",
                "mizar-resolve = { path = \"../mizar-resolve\" }",
                "mizar-session = { path = \"../mizar-session\" }",
            ],
        )],
        "{} must depend only on mizar-checker, mizar-resolve, and \
         mizar-session, with no dev/build/target dependency escape hatch, until \
         a task-scoped core spec expands the crate boundary",
        manifest_path.display()
    );
}

#[test]
fn public_semantic_modules_have_owning_specs() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);
    let modules = public_module_exports(&source);
    let mut violations = Vec::new();

    if modules
        != [
            "binder_normalization",
            "control_flow",
            "core_ir",
            "elaborator",
        ]
    {
        violations.push(format!(
            "{} must expose exactly the documented task-3/task-5/task-8/task-15 modules, found {:?}",
            lib_path.display(),
            modules
        ));
    }

    for module in modules {
        for language in ["en", "ja"] {
            let spec_path = workspace_root()
                .join("doc/design/mizar-core")
                .join(language)
                .join(format!("{module}.md"));
            if !spec_path.is_file() {
                violations.push(format!(
                    "{}: public module `{module}` needs owning spec {}",
                    lib_path.display(),
                    spec_path.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "public core APIs require their owning module spec first:\n{}",
        violations.join("\n")
    );
}

#[test]
fn core_source_stays_off_frontend_and_downstream_boundaries() {
    let root = crate_root();
    let forbidden = [
        "mizar_frontend::",
        "mizar_lexer::",
        "mizar_parser::",
        "mizar_syntax::",
        "mizar_vc::",
        "mizar_kernel::",
        "mizar_proof::",
        "mizar_resolve::env",
        "ResolvedAst",
        "SymbolEnv",
        "resolver_env",
        "resolved_ast as",
        "extern crate mizar_frontend",
        "extern crate mizar_syntax",
    ];
    let mut violations = Vec::new();

    for path in rust_source_files(&root.join("src")) {
        let source = read_to_string(&path);
        let compact_source = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let allow_exact_task31_metadata = path
            .strip_prefix(&root)
            .is_ok_and(|relative| relative == Path::new("src/elaborator.rs"));
        let boundary_source =
            strip_exact_task31_metadata_import(&compact_source, allow_exact_task31_metadata);
        let resolver_aliases = resolver_crate_aliases(&boundary_source);
        if resolver_braced_import_contains_forbidden_api(&boundary_source) {
            let display_path = path.strip_prefix(&root).unwrap_or(&path);
            violations.push(format!(
                "{}: mizar_resolve::{{env/resolved_ast}}",
                display_path.display()
            ));
        }
        for token in forbidden {
            let compact_token = token
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>();
            if boundary_source.contains(&compact_token) {
                let display_path = path.strip_prefix(&root).unwrap_or(&path);
                violations.push(format!("{}: {token}", display_path.display()));
            }
        }
        for alias in &resolver_aliases {
            if resolver_alias_uses_forbidden_api(&boundary_source, alias)
                || resolver_alias_imports_forbidden_api(&boundary_source, alias)
            {
                let display_path = path.strip_prefix(&root).unwrap_or(&path);
                violations.push(format!(
                    "{}: {alias}::env/resolved_ast",
                    display_path.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "mizar-core must consume checker/resolver/session payloads without \
         crossing frontend or downstream proof/VC/kernel boundaries:\n{}",
        violations.join("\n")
    );
}

#[test]
fn resolver_boundary_scanner_catches_braced_and_nested_imports() {
    let samples = [
        "use mizar_resolve::{env as resolver_env};",
        "use mizar_resolve::{resolved_ast as ra};",
        "use mizar_resolve::{imports::{ImportGraphBuilder}, env as e};",
        "use mizar_resolve::{imports::{ImportGraphBuilder}, resolved_ast::{ResolvedAst}};",
    ];

    for sample in samples {
        let compact_sample = sample
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        assert!(
            resolver_braced_import_contains_forbidden_api(&compact_sample),
            "{sample}"
        );
    }

    let allowed = "use mizar_resolve::{symbols::{SymbolId}, labels::LabelId};";
    let compact_allowed = allowed
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    assert!(!resolver_braced_import_contains_forbidden_api(
        &compact_allowed,
    ));

    let task31_metadata = "use mizar_resolve::env::{ExportStatus, Visibility};";
    let compact_task31_metadata = task31_metadata
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    assert!(compact_task31_metadata.contains("mizar_resolve::env"));
    assert_eq!(
        strip_exact_task31_metadata_import(&compact_task31_metadata, true),
        ""
    );
    assert_eq!(
        strip_exact_task31_metadata_import(&compact_task31_metadata, false),
        compact_task31_metadata
    );

    for sample in [
        "use mizar_resolve::env::{Visibility};",
        "use mizar_resolve::env::{ExportStatus, SymbolEnv, Visibility};",
        "use mizar_resolve::env::{ExportStatus as Status, Visibility};",
    ] {
        let compact_sample = sample
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        assert!(
            strip_exact_task31_metadata_import(&compact_sample, true)
                .contains("mizar_resolve::env")
        );
    }
}

#[test]
fn resolver_boundary_scanner_catches_crate_alias_access() {
    let samples = [
        (
            "use mizar_resolve as resolve; resolve::env::ContributionKind;",
            "resolve",
        ),
        (
            "use mizar_resolve as resolve; resolve::resolved_ast::ModuleId;",
            "resolve",
        ),
        ("use mizar_resolve as resolve; use resolve::env;", "resolve"),
        (
            "use mizar_resolve as resolve; use resolve::{imports::{ImportGraphBuilder}, env as e};",
            "resolve",
        ),
        (
            "use mizar_resolve as resolve; use resolve::{resolved_ast::{ResolvedAst}};",
            "resolve",
        ),
    ];

    for (sample, expected_alias) in samples {
        let compact_sample = sample
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let aliases = resolver_crate_aliases(&compact_sample);
        assert!(aliases.contains(&expected_alias.to_owned()), "{sample}");
        assert!(
            resolver_alias_uses_forbidden_api(&compact_sample, expected_alias)
                || resolver_alias_imports_forbidden_api(&compact_sample, expected_alias),
            "{sample}"
        );
    }
}

#[test]
fn public_api_scanner_catches_common_public_surface_shapes() {
    let public_samples = [
        "pub const fn sample() {}",
        "pub async fn sample() {}",
        "pub unsafe fn sample() {}",
        "pub const SAMPLE: u8 = 0;",
        "pub static SAMPLE: u8 = 0;",
        "pub macro sample() {}",
        "#[macro_export] macro_rules! sample { () => {}; }",
    ];

    for sample in public_samples {
        assert!(public_semantic_declaration(sample), "{sample}");
    }

    assert!(!public_semantic_declaration(
        "//! task 1 intentionally exposes only the crate boundary"
    ));
}

#[test]
fn public_module_export_scanner_finds_core_ir_module() {
    let source = "//! docs\n\npub mod binder_normalization;\npub mod core_ir;\n";

    assert_eq!(
        public_module_exports(source),
        ["binder_normalization", "core_ir",]
    );
}

#[test]
fn public_core_enums_are_forward_compatible_and_documented() {
    let crate_root = crate_root();
    let modules = [
        "binder_normalization",
        "control_flow",
        "core_ir",
        "elaborator",
    ];
    let mut violations = Vec::new();
    let mut enums_by_module = BTreeMap::new();

    for module in modules {
        let source_path = crate_root.join("src").join(format!("{module}.rs"));
        let source = read_to_string(&source_path);
        for (line_number, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub mod ") || trimmed.starts_with("pub use ") {
                violations.push(format!(
                    "{}:{}: public nested modules or re-exports must update the public enum policy guard before exposing additional enum surfaces",
                    source_path.display(),
                    line_number + 1
                ));
            }
        }
        let public_enums = public_enums(&source);
        if public_enums.is_empty() {
            violations.push(format!(
                "{}: expected at least one public enum to classify",
                source_path.display()
            ));
        }
        for public_enum in &public_enums {
            if !public_enum.has_non_exhaustive {
                violations.push(format!(
                    "{}:{}: public enum `{}` must keep #[non_exhaustive]",
                    source_path.display(),
                    public_enum.line_number,
                    public_enum.name
                ));
            }
        }
        enums_by_module.insert(module, public_enums);
    }

    for (module, public_enums) in &enums_by_module {
        for language in ["en", "ja"] {
            let spec_path = workspace_root()
                .join("doc/design/mizar-core")
                .join(language)
                .join(format!("{module}.md"));
            let spec = read_to_string(&spec_path);
            let Some(policy) = public_enum_policy_section(&spec) else {
                violations.push(format!(
                    "{}: missing public enum policy section",
                    spec_path.display()
                ));
                continue;
            };
            if !policy_contains_no_exhaustive_exception(policy) {
                violations.push(format!(
                    "{}: public enum policy must state that there are no exhaustive public enum exceptions",
                    spec_path.display()
                ));
            }
            if !policy.contains("Internal")
                && !policy.contains("内部 match")
                && !policy.contains("内部の match")
            {
                violations.push(format!(
                    "{}: public enum policy must say internal mizar-core matches may remain exhaustive",
                    spec_path.display()
                ));
            }
            let source_enum_names = public_enums
                .iter()
                .map(|public_enum| public_enum.name.clone())
                .collect::<BTreeSet<_>>();
            let policy_enum_names = policy_enum_names(policy);
            if policy_enum_names != source_enum_names {
                violations.push(format!(
                    "{}: public enum policy rows must exactly match source enums; source={:?}, policy={:?}",
                    spec_path.display(),
                    source_enum_names,
                    policy_enum_names
                ));
            }
            for public_enum in public_enums {
                let enum_name = format!("`{}`", public_enum.name);
                if !policy.contains(&enum_name) {
                    violations.push(format!(
                        "{}: public enum policy must list `{}`",
                        spec_path.display(),
                        public_enum.name
                    ));
                    continue;
                }
                if !policy
                    .lines()
                    .any(|line| line.contains(&enum_name) && line.contains("#[non_exhaustive]"))
                {
                    violations.push(format!(
                        "{}: public enum policy row for `{}` must record #[non_exhaustive]",
                        spec_path.display(),
                        public_enum.name
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "public core enum forward-compatibility policy drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn source_spec_audit_covers_public_modules_and_deferred_gaps() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);
    let modules = public_module_exports(&source);
    let expected_modules = modules.iter().copied().collect::<BTreeSet<_>>();
    let expected_gaps = [
        (
            "CORE-AUDIT-G001".to_owned(),
            "external_dependency_gap".to_owned(),
        ),
        (
            "CORE-AUDIT-G002".to_owned(),
            "external_dependency_gap".to_owned(),
        ),
        (
            "CORE-AUDIT-G003".to_owned(),
            "external_dependency_gap".to_owned(),
        ),
        (
            "CORE-AUDIT-G004".to_owned(),
            "external_dependency_gap".to_owned(),
        ),
        (
            "CORE-AUDIT-G005".to_owned(),
            "external_dependency_gap".to_owned(),
        ),
        ("CORE-AUDIT-G006".to_owned(), "deferred".to_owned()),
    ]
    .into_iter()
    .collect::<BTreeMap<_, _>>();
    let mut gaps_by_language = BTreeMap::new();
    let mut violations = Vec::new();

    if modules
        != [
            "binder_normalization",
            "control_flow",
            "core_ir",
            "elaborator",
        ]
    {
        violations.push(format!(
            "{} must expose exactly the source/spec audited public modules, found {:?}",
            lib_path.display(),
            modules
        ));
    }

    for language in ["en", "ja"] {
        let audit_path = workspace_root()
            .join("doc/design/mizar-core")
            .join(language)
            .join("source_spec_audit.md");
        let audit = read_to_string(&audit_path);
        let audit_modules = audit_public_module_sections(&audit);
        if audit_modules != expected_modules {
            violations.push(format!(
                "{}: source/spec audit module sections must exactly match public modules; source={:?}, audit={:?}",
                audit_path.display(),
                expected_modules,
                audit_modules
            ));
        }
        let audit_gaps = audit_gap_classes(&audit);
        if audit_gaps != expected_gaps {
            violations.push(format!(
                "{}: source/spec audit gap ids/classes must exactly match expected follow-up register; expected={:?}, audit={:?}",
                audit_path.display(),
                expected_gaps,
                audit_gaps
            ));
        }
        gaps_by_language.insert(language, audit_gaps);
        for module in &modules {
            let module_source_path = crate_root().join("src").join(format!("{module}.rs"));
            let module_source = read_to_string(&module_source_path);
            let expected_public_items = audited_public_items(&module_source);
            let module_section = audit_module_section(&audit, module).unwrap_or_else(|| {
                violations.push(format!(
                    "{}: source/spec audit must include a section for `{module}`",
                    audit_path.display()
                ));
                ""
            });
            for marker in [
                format!("`{module}`"),
                format!("src/{module}.rs"),
                format!("{module}.md"),
            ] {
                if !audit.contains(&marker) {
                    violations.push(format!(
                        "{}: source/spec audit must mention {marker}",
                        audit_path.display()
                    ));
                }
            }
            for public_item in expected_public_items {
                let marker = format!("`{public_item}`");
                if !module_section.contains(&marker) {
                    violations.push(format!(
                        "{}: source/spec audit section `{module}` must mention public item {marker} from {}",
                        audit_path.display(),
                        module_source_path.display()
                    ));
                }
            }
        }
        for gap_detail in audit_gap_detail_violations(&audit) {
            violations.push(format!("{}: {gap_detail}", audit_path.display()));
        }
        for marker in [
            "No source/spec drift",
            "source-to-checker extraction",
            "type_elaboration",
            "proof_verification",
            "artifact schema",
            "proof acceptance",
            "VC generation",
            "kernel checking",
            "VcId",
            "ObligationAnchor",
            "source_undocumented_behavior",
        ] {
            if !audit.contains(marker) {
                violations.push(format!(
                    "{}: source/spec audit must record deferred/external marker `{marker}`",
                    audit_path.display()
                ));
            }
        }
    }

    if gaps_by_language.get("en") != gaps_by_language.get("ja") {
        violations.push(format!(
            "EN/JA source/spec audit gap ids/classes must stay synchronized; en={:?}, ja={:?}",
            gaps_by_language.get("en"),
            gaps_by_language.get("ja")
        ));
    }

    assert!(
        violations.is_empty(),
        "mizar-core source/spec audit drift:\n{}",
        violations.join("\n")
    );
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

fn public_module_exports(source: &str) -> Vec<&str> {
    let mut modules = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        let Some(module_name) = trimmed.strip_prefix("pub mod ") else {
            continue;
        };
        let module_name = module_name
            .trim_end_matches(';')
            .split_whitespace()
            .next()
            .unwrap_or_default();
        if !module_name.is_empty() {
            modules.push(module_name);
        }
    }

    modules.sort_unstable();
    modules
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublicEnum {
    name: String,
    line_number: usize,
    has_non_exhaustive: bool,
}

fn public_enums(source: &str) -> Vec<PublicEnum> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut enums = Vec::new();
    let mut brace_depth = 0_i32;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("pub enum ") else {
            brace_depth += brace_delta(line);
            continue;
        };
        if brace_depth != 0 {
            brace_depth += brace_delta(line);
            continue;
        }
        let name = rest
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if name.is_empty() {
            continue;
        }
        let has_non_exhaustive = contiguous_outer_attributes(&lines, index)
            .iter()
            .any(|attribute| attribute.trim() == "#[non_exhaustive]");
        enums.push(PublicEnum {
            name,
            line_number: index + 1,
            has_non_exhaustive,
        });
        brace_depth += brace_delta(line);
    }

    enums
}

fn contiguous_outer_attributes<'a>(lines: &'a [&str], enum_index: usize) -> Vec<&'a str> {
    let mut attributes = Vec::new();
    let mut index = enum_index;
    while index > 0 {
        let previous = lines[index - 1].trim();
        if previous.starts_with("#[") || previous.starts_with("///") {
            attributes.push(previous);
            index -= 1;
        } else {
            break;
        }
    }
    attributes.reverse();
    attributes
}

fn public_enum_policy_section(document: &str) -> Option<&str> {
    let mut start = None;
    for (index, line) in document.match_indices('\n') {
        let next_start = index + line.len();
        let remaining = &document[next_start..];
        if remaining.starts_with("## Public Enum Policy")
            || remaining.starts_with("## public enum policy")
        {
            start = Some(next_start);
            break;
        }
    }
    let start = start.or_else(|| {
        (document.starts_with("## Public Enum Policy")
            || document.starts_with("## public enum policy"))
        .then_some(0)
    })?;
    let rest = &document[start..];
    let end = rest
        .find("\n## ")
        .map_or(document.len(), |offset| start + offset);
    Some(&document[start..end])
}

fn policy_enum_names(policy: &str) -> BTreeSet<String> {
    policy
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with('|') || trimmed.starts_with("|---") {
                return None;
            }
            let first_cell = trimmed.trim_matches('|').split('|').next()?.trim();
            let name = first_cell.strip_prefix('`')?.split('`').next()?;
            if name.is_empty() || name == "Public enum" || name == "public enum" {
                return None;
            }
            Some(name.to_owned())
        })
        .collect()
}

fn policy_contains_no_exhaustive_exception(policy: &str) -> bool {
    policy.contains("No exhaustive public enum exceptions are owned by this module")
        || policy.contains("この module が所有する exhaustive public enum exception はない")
}

fn audit_public_module_sections(document: &str) -> BTreeSet<&str> {
    document
        .lines()
        .filter_map(|line| {
            let module = line.trim().strip_prefix("### `")?.split('`').next()?;
            (!module.is_empty()).then_some(module)
        })
        .collect()
}

fn audit_module_section<'a>(document: &'a str, module: &str) -> Option<&'a str> {
    let header = format!("### `{module}`");
    let start = document.find(&header)?;
    let rest = &document[start..];
    let end = rest
        .find("\n### `")
        .or_else(|| rest.find("\n## "))
        .map_or(document.len(), |offset| start + offset);
    Some(&document[start..end])
}

fn audit_gap_classes(document: &str) -> BTreeMap<String, String> {
    audit_gap_rows(document)
        .into_iter()
        .filter_map(|(id, cells)| {
            let class = cells.get(1)?.trim_matches('`').to_owned();
            Some((id, class))
        })
        .collect()
}

fn audit_gap_detail_violations(document: &str) -> Vec<String> {
    audit_gap_rows(document)
        .into_iter()
        .flat_map(|(id, cells)| {
            let mut violations = Vec::new();
            if cells.len() < 6 {
                violations.push(format!(
                    "follow-up row {id} must include ID, Class, Evidence, Owner, Unblock condition, and Target follow-up / downstream phase"
                ));
                return violations;
            }
            for (index, label) in [
                "Evidence",
                "Owner",
                "Unblock condition",
                "Target follow-up / downstream phase",
            ]
            .iter()
            .enumerate()
            {
                let cell = cells[index + 2].trim();
                if cell.is_empty() || cell == "-" {
                    violations.push(format!(
                        "follow-up row {id} must have a non-empty {label} cell"
                    ));
                }
            }
            violations
        })
        .collect()
}

fn audit_gap_rows(document: &str) -> BTreeMap<String, Vec<String>> {
    document
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("| CORE-AUDIT-") {
                return None;
            }
            let cells = trimmed
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            let id = cells.first()?;
            Some((
                (*id).to_owned(),
                cells.into_iter().map(str::to_owned).collect(),
            ))
        })
        .collect()
}

fn audited_public_items(source: &str) -> BTreeSet<String> {
    let mut items = BTreeSet::new();
    let mut brace_depth = 0_i32;
    let mut pending_macro = None::<String>;

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(invocation) = &mut pending_macro {
            invocation.push(' ');
            invocation.push_str(trimmed);
            if trimmed.ends_with(");") {
                collect_generated_public_item(invocation, &mut items);
                pending_macro = None;
            }
            brace_depth += brace_delta(line);
            continue;
        }

        if brace_depth == 0 {
            if let Some(name) = public_item_name(trimmed) {
                items.insert(name);
            }
            if macro_invocation_starts_public_item(trimmed) {
                if trimmed.ends_with(");") {
                    collect_generated_public_item(trimmed, &mut items);
                } else {
                    pending_macro = Some(trimmed.to_owned());
                }
            }
        }
        brace_depth += brace_delta(line);
    }

    items
}

fn public_item_name(line: &str) -> Option<String> {
    for prefix in [
        "pub struct ",
        "pub enum ",
        "pub type ",
        "pub fn ",
        "pub const ",
        "pub static ",
        "pub trait ",
        "pub macro ",
    ] {
        let Some(rest) = line.strip_prefix(prefix) else {
            continue;
        };
        let name = rest
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn macro_invocation_starts_public_item(line: &str) -> bool {
    ["dense_id!(", "string_key!(", "table!("]
        .iter()
        .any(|prefix| line.starts_with(prefix))
}

fn collect_generated_public_item(invocation: &str, items: &mut BTreeSet<String>) {
    for prefix in ["dense_id!(", "string_key!(", "table!("] {
        let Some(rest) = invocation.trim().strip_prefix(prefix) else {
            continue;
        };
        let name = rest
            .trim_start()
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if !name.is_empty() {
            items.insert(name);
        }
        return;
    }
}

fn brace_delta(line: &str) -> i32 {
    let mut delta = 0_i32;
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(character) = chars.next() {
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
            break;
        }
        match character {
            '"' => in_string = true,
            '{' => delta += 1,
            '}' => delta -= 1,
            _ => {}
        }
    }

    delta
}

fn strip_exact_task31_metadata_import(
    compact_source: &str,
    allow_exact_task31_metadata: bool,
) -> String {
    const EXACT_IMPORT: &str = "usemizar_resolve::env::{ExportStatus,Visibility};";

    if allow_exact_task31_metadata {
        compact_source.replacen(EXACT_IMPORT, "", 1)
    } else {
        compact_source.to_owned()
    }
}

fn resolver_braced_import_contains_forbidden_api(compact_source: &str) -> bool {
    let mut rest = compact_source;
    while let Some(start) = rest.find("mizar_resolve::{") {
        rest = &rest[start + "mizar_resolve::{".len()..];
        let Some(end) = matching_brace_offset(rest) else {
            return true;
        };
        let group = &rest[..end];
        for segment in top_level_segments(group) {
            if segment == "env"
                || segment.starts_with("env::")
                || segment.starts_with("envas")
                || segment == "resolved_ast"
                || segment.starts_with("resolved_ast::")
                || segment.starts_with("resolved_astas")
            {
                return true;
            }
        }
        rest = &rest[end + 1..];
    }

    false
}

fn resolver_crate_aliases(compact_source: &str) -> Vec<String> {
    let mut aliases = Vec::new();
    let mut rest = compact_source;
    let pattern = "usemizar_resolveas";

    while let Some(start) = rest.find(pattern) {
        rest = &rest[start + pattern.len()..];
        let alias = rest
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if !alias.is_empty() {
            aliases.push(alias);
        }
    }

    aliases.sort();
    aliases.dedup();
    aliases
}

fn resolver_alias_uses_forbidden_api(compact_source: &str, alias: &str) -> bool {
    compact_source.contains(&format!("{alias}::env::"))
        || compact_source.contains(&format!("{alias}::resolved_ast::"))
}

fn resolver_alias_imports_forbidden_api(compact_source: &str, alias: &str) -> bool {
    compact_source.contains(&format!("use{alias}::env"))
        || compact_source.contains(&format!("use{alias}::resolved_ast"))
        || alias_braced_import_contains_forbidden_api(compact_source, alias)
}

fn alias_braced_import_contains_forbidden_api(compact_source: &str, alias: &str) -> bool {
    let mut rest = compact_source;
    let pattern = format!("use{alias}::{{");

    while let Some(start) = rest.find(&pattern) {
        rest = &rest[start + pattern.len()..];
        let Some(end) = matching_brace_offset(rest) else {
            return true;
        };
        let group = &rest[..end];
        for segment in top_level_segments(group) {
            if segment == "env"
                || segment.starts_with("env::")
                || segment.starts_with("envas")
                || segment == "resolved_ast"
                || segment.starts_with("resolved_ast::")
                || segment.starts_with("resolved_astas")
            {
                return true;
            }
        }
        rest = &rest[end + 1..];
    }

    false
}

fn matching_brace_offset(text_after_open_brace: &str) -> Option<usize> {
    let mut depth = 0_u32;

    for (index, character) in text_after_open_brace.char_indices() {
        match character {
            '{' => depth += 1,
            '}' if depth == 0 => return Some(index),
            '}' => depth -= 1,
            _ => {}
        }
    }

    None
}

fn top_level_segments(group: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut depth = 0_u32;
    let mut segment_start = 0;

    for (index, character) in group.char_indices() {
        match character {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            ',' if depth == 0 => {
                segments.push(&group[segment_start..index]);
                segment_start = index + 1;
            }
            _ => {}
        }
    }

    segments.push(&group[segment_start..]);
    segments
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap_or_else(|error| panic!("{}: {error}", root.display())) {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", root.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}
