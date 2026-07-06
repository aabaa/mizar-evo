use std::{fs, path::PathBuf};

use mizar_proof::policy::{
    ExternalEvidenceMode, PolicyCandidate, PortfolioEarlyStopClass, PortfolioEarlyStopInput,
    PortfolioEarlyStopReason, ProofPolicyEvaluator, VerifierPolicy,
};

#[test]
fn proof_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_proof_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let members = workspace_members(&manifest);
    let occurrences = members
        .iter()
        .filter(|member| **member == "crates/mizar-proof")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-proof exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn proof_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-proof")),
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
            .any(|line| assignment_is(line, "name", "mizar_proof")),
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
fn proof_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "mizar-artifact = { path = \"../mizar-artifact\" }",
                "mizar-atp = { path = \"../mizar-atp\" }",
                "mizar-kernel = { path = \"../mizar-kernel\" }",
                "mizar-session = { path = \"../mizar-session\" }",
                "mizar-vc = { path = \"../mizar-vc\" }",
            ],
        )],
        "{} must keep task-1 production dependencies limited to mizar-session, \
         mizar-kernel, mizar-vc, mizar-atp, and mizar-artifact; \
         dev/build/target dependency sections require a later explicit task",
        manifest_path.display()
    );
}

#[test]
fn proof_lib_states_boundary_and_exposes_modules_after_specs() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);

    for marker in [
        "#![forbid(unsafe_code)]",
        "Proof policy, selection, status projection",
        "does not accept proofs",
        "Trusted acceptance comes only from `mizar-kernel`",
        "not promoted into trusted proof status",
        "used_axioms",
    ] {
        assert!(
            source.contains(marker),
            "{} must keep the task-1 proof-boundary marker `{marker}`",
            lib_path.display()
        );
    }

    let declarations = public_module_declarations(&source);
    assert_eq!(
        declarations,
        ["policy", "selection", "status", "witness_store"],
        "{} may expose only the task-3 policy module, task-6 selection module, \
         task-9 status module, and task-11 witness_store module; found \
         {declarations:?}",
        lib_path.display()
    );
}

#[test]
fn proof_crate_tree_contains_task_nineteen_private_test_modules() {
    let mut files = crate_files();
    files.sort();

    assert_eq!(
        files,
        [
            "Cargo.toml",
            "src/lib.rs",
            "src/policy.rs",
            "src/policy/tests.rs",
            "src/selection.rs",
            "src/selection/tests.rs",
            "src/status.rs",
            "src/status/tests.rs",
            "src/witness_store.rs",
            "src/witness_store/tests.rs",
            "tests/determinism_suite.rs",
            "tests/lint_policy.rs"
        ],
        "mizar-proof task 19 may contain only the policy, selection, status, \
         witness_store modules, their private test submodules, the determinism \
         suite, and the lint guard; build scripts, examples, benches, or extra \
         tests require later explicit tasks; found {files:?}"
    );
}

#[test]
fn early_stop_public_api_rejects_external_when_kernel_certificates_required() {
    let evaluator = ProofPolicyEvaluator::new(
        VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
            .with_require_kernel_certificates(true),
    );

    assert_eq!(
        evaluator.best_possible_early_stop_class(&PolicyCandidate::ExternallyAttested),
        None
    );

    let input =
        PortfolioEarlyStopInput::new(Some(PortfolioEarlyStopClass::PolicyPermittedExternal), []);
    assert_eq!(
        input.observed_best_class(),
        Some(PortfolioEarlyStopClass::PolicyPermittedExternal)
    );
    assert_eq!(input.pending_best_possible_classes(), &[]);

    let decision = evaluator.portfolio_early_stop_decision(&input);
    assert!(!decision.may_stop());
    assert_eq!(
        decision.reason(),
        PortfolioEarlyStopReason::ObservedClassNotSelectable
    );
    assert_eq!(
        decision.observed_best_class(),
        Some(PortfolioEarlyStopClass::PolicyPermittedExternal)
    );
    assert_eq!(decision.blocking_pending_class(), None);
}

#[test]
fn public_enums_are_forward_compatible_and_documented() {
    let enum_policy = public_enum_policy();
    let mut expected = enum_policy
        .iter()
        .map(|entry| format!("{}::{}", entry.relative_path, entry.enum_name))
        .collect::<Vec<_>>();
    expected.sort();

    let mut discovered = Vec::new();
    for relative_path in rust_source_files() {
        let source = read_to_string(&crate_root().join(&relative_path));
        for enum_name in public_enum_names(&source) {
            discovered.push(format!("{relative_path}::{enum_name}"));
        }
    }
    discovered.sort();

    assert_eq!(
        discovered, expected,
        "each public mizar-proof enum must be classified by the forward-compatibility policy"
    );

    let mut violations = Vec::new();
    for entry in enum_policy {
        let source = read_to_string(&crate_root().join(entry.relative_path));
        if !enum_has_attribute(&source, entry.enum_name, "non_exhaustive") {
            violations.push(format!(
                "{}: pub enum {} must be #[non_exhaustive]",
                entry.relative_path, entry.enum_name
            ));
        }
        for spec_path in [entry.en_spec, entry.ja_spec] {
            let spec = read_to_string(&workspace_root().join(spec_path));
            let row = format!("| `{}` |", entry.enum_name);
            if !spec.contains(&row) {
                violations.push(format!(
                    "{}: missing public enum policy row for {}",
                    spec_path, entry.enum_name
                ));
            }
            if !spec.contains("No exhaustive public enum exceptions") {
                violations.push(format!(
                    "{}: must state there are no exhaustive public enum exceptions",
                    spec_path
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "public enum forward-compatibility policy drift:\n{}",
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

fn section<'a>(document: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    let mut in_section = false;
    let mut lines = Vec::new();

    for line in document.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == header;
        } else if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
            lines.push(line.trim());
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

fn workspace_members(manifest: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut in_members = false;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_members = false;
            continue;
        }
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
            continue;
        }
        if in_members {
            if trimmed.starts_with(']') {
                break;
            }
            if let Some(member) = trimmed.trim_end_matches(',').trim().strip_prefix('"') {
                members.push(member.trim_end_matches('"').to_owned());
            }
        }
    }

    members
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

fn public_module_declarations(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            line.trim_start()
                .strip_prefix("pub mod ")
                .map(|module| module.trim_end_matches(';').to_owned())
        })
        .collect()
}

fn crate_files() -> Vec<String> {
    let root = crate_root();
    let mut files = Vec::new();
    collect_files(&root, &root, &mut files);
    files
}

fn rust_source_files() -> Vec<String> {
    let mut files = crate_files()
        .into_iter()
        .filter(|path| path.starts_with("src/") && path.ends_with(".rs"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn collect_files(root: &std::path::Path, dir: &std::path::Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, files);
        } else {
            files.push(
                path.strip_prefix(root)
                    .expect("file lives under crate root")
                    .display()
                    .to_string(),
            );
        }
    }
}

struct PublicEnumPolicy {
    relative_path: &'static str,
    enum_name: &'static str,
    en_spec: &'static str,
    ja_spec: &'static str,
}

const PUBLIC_ENUM_POLICY: &[PublicEnumPolicy] = &[
    policy_enum("BuildMode"),
    policy_enum("ExternalEvidenceMode"),
    policy_enum("OpenObligationMode"),
    policy_enum("PolicyAssumptionMode"),
    policy_enum("KernelEvidenceFormat"),
    policy_enum("CandidatePolicyClass"),
    policy_enum("PortfolioEarlyStopClass"),
    policy_enum("PortfolioEarlyStopReason"),
    policy_enum("KernelEvidenceOrigin"),
    policy_enum("AcceptedGoalPolarity"),
    policy_enum("PolicyCandidate"),
    policy_enum("BackendProofPayloadKind"),
    policy_enum("ExternalEvidencePublicationStatus"),
    policy_enum("PolicyDiagnosticCategory"),
    policy_enum("PolicyReasonCode"),
    selection_enum("SelectionInputError"),
    selection_enum("ProofWinnerClass"),
    selection_enum("ProofWitnessPublication"),
    selection_enum("ProofSelectionSource"),
    selection_enum("ArtifactProofSelectionError"),
    status_enum("TrustedUsedAxiomsError"),
    status_enum("ProjectedProofStatus"),
    status_enum("CurrentArtifactObligationStatus"),
    status_enum("ArtifactPublicationGap"),
    status_enum("ArtifactStatusPublication"),
    status_enum("StatusProjectionError"),
    witness_store_enum("ProofWitnessStoreError"),
];

fn public_enum_policy() -> &'static [PublicEnumPolicy] {
    PUBLIC_ENUM_POLICY
}

const fn policy_enum(enum_name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        relative_path: "src/policy.rs",
        enum_name,
        en_spec: "doc/design/mizar-proof/en/policy.md",
        ja_spec: "doc/design/mizar-proof/ja/policy.md",
    }
}

const fn selection_enum(enum_name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        relative_path: "src/selection.rs",
        enum_name,
        en_spec: "doc/design/mizar-proof/en/selection.md",
        ja_spec: "doc/design/mizar-proof/ja/selection.md",
    }
}

const fn status_enum(enum_name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        relative_path: "src/status.rs",
        enum_name,
        en_spec: "doc/design/mizar-proof/en/status.md",
        ja_spec: "doc/design/mizar-proof/ja/status.md",
    }
}

const fn witness_store_enum(enum_name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        relative_path: "src/witness_store.rs",
        enum_name,
        en_spec: "doc/design/mizar-proof/en/witness_store.md",
        ja_spec: "doc/design/mizar-proof/ja/witness_store.md",
    }
}

fn public_enum_names(source: &str) -> Vec<String> {
    public_enum_declarations(source)
        .into_iter()
        .map(|declaration| declaration.name)
        .collect()
}

fn enum_has_attribute(source: &str, enum_name: &str, attribute: &str) -> bool {
    let lines = source.lines().collect::<Vec<_>>();
    let Some(index) = public_enum_declarations(source)
        .into_iter()
        .find(|declaration| declaration.name == enum_name)
        .map(|declaration| declaration.line_index)
    else {
        return false;
    };
    let marker = format!("#[{attribute}]");
    lines[..index]
        .iter()
        .rev()
        .take_while(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && (trimmed.starts_with("#[") || trimmed.starts_with("///"))
        })
        .any(|line| line.trim() == marker)
}

#[derive(Debug, Eq, PartialEq)]
struct PublicEnumDeclaration {
    name: String,
    line_index: usize,
}

fn public_enum_declarations(source: &str) -> Vec<PublicEnumDeclaration> {
    let tokens = rust_tokens(source);
    let mut declarations = Vec::new();

    for index in 0..tokens.len().saturating_sub(2) {
        if tokens[index].text == "pub"
            && tokens[index + 1].text == "enum"
            && is_identifier(&tokens[index + 2].text)
        {
            declarations.push(PublicEnumDeclaration {
                name: tokens[index + 2].text.clone(),
                line_index: tokens[index].line_index,
            });
        }
    }

    declarations
}

#[derive(Debug, Eq, PartialEq)]
struct RustToken {
    text: String,
    line_index: usize,
}

fn rust_tokens(source: &str) -> Vec<RustToken> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut line_index = 0;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '\n' {
            line_index += 1;
            index += 1;
            continue;
        }
        if ch.is_whitespace() {
            index += 1;
            continue;
        }
        if ch == '/' && chars.get(index + 1) == Some(&'/') {
            index += 2;
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
            continue;
        }
        if ch == '/' && chars.get(index + 1) == Some(&'*') {
            index += 2;
            while index < chars.len() {
                if chars[index] == '\n' {
                    line_index += 1;
                }
                if chars[index] == '*' && chars.get(index + 1) == Some(&'/') {
                    index += 2;
                    break;
                }
                index += 1;
            }
            continue;
        }
        if ch == '"' {
            index += 1;
            let mut escaped = false;
            while index < chars.len() {
                let literal_ch = chars[index];
                if literal_ch == '\n' {
                    line_index += 1;
                }
                if escaped {
                    escaped = false;
                } else if literal_ch == '\\' {
                    escaped = true;
                } else if literal_ch == '"' {
                    index += 1;
                    break;
                }
                index += 1;
            }
            continue;
        }
        if is_identifier_start(ch) {
            let start = index;
            index += 1;
            while index < chars.len() && is_identifier_continue(chars[index]) {
                index += 1;
            }
            tokens.push(RustToken {
                text: chars[start..index].iter().collect(),
                line_index,
            });
            continue;
        }

        tokens.push(RustToken {
            text: ch.to_string(),
            line_index,
        });
        index += 1;
    }

    tokens
}

fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_continue)
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
