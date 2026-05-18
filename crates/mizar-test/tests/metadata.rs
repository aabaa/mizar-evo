use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use mizar_test::{DiscoveryConfig, TestProfile, ValidationMode, build_test_plan};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn empty_corpus_succeeds() {
    let corpus = Corpus::new();

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0);
    assert_eq!(plan.cases.len(), 0);
    assert_eq!(plan.manifest.requirements.len(), 0);
}

#[test]
fn malformed_toml_fails() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/bad.src", "");
    corpus.write(
        "tests/lexical/pass/bad.expect.toml",
        "schema_version = \"one\"\n",
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");
}

#[test]
fn duplicate_expectation_ids_fail() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.add_case(
        "tests/lexical/pass/dup_one",
        "dup_shared",
        "spec.en.test.basic",
    );
    corpus.add_case(
        "tests/lexical/pass/dup_two",
        "dup_shared",
        "spec.en.test.basic",
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-DUP-ID");
}

#[test]
fn missing_source_fails() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/lexical/pass/missing.expect.toml",
        expectation("missing", "missing.src", "spec.en.test.basic"),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-MISSING-SOURCE");
}

#[test]
fn missing_sidecar_fails_for_payload() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/orphan.src", "");

    let plan = corpus.plan();

    assert_has_code(&plan, "E-LAYOUT-MISSING-SIDECAR");
}

#[test]
fn unknown_spec_refs_fail() {
    let corpus = Corpus::new();
    corpus.add_case(
        "tests/lexical/pass/unknown_spec",
        "unknown_spec",
        "spec.en.test.unknown",
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-TRACE-UNKNOWN-SPEC-REF");
}

#[test]
fn manifest_test_back_reference_succeeds() {
    let corpus = Corpus::new();
    corpus.add_case("tests/lexical/pass/linked", "linked", "spec.en.test.basic");
    corpus.add_requirement(
        "spec.en.test.basic",
        &["tests/lexical/pass/linked.expect.toml"],
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    assert_eq!(plan.cases.len(), 1);
}

#[test]
fn expectation_source_must_be_clean_relative_path() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/pass/escape.src", "");
    corpus.write(
        "tests/lexical/pass/escape.expect.toml",
        expectation("escape", "../escape.src", "spec.en.test.basic"),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SOURCE-PATH");
}

#[test]
fn expectation_source_must_use_payload_extension() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/pass/not_payload.txt", "");
    corpus.write(
        "tests/lexical/pass/not_payload.expect.toml",
        expectation("not_payload", "not_payload.txt", "spec.en.test.basic"),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SOURCE-EXTENSION");
}

#[test]
fn manifest_paths_must_be_clean_relative_paths() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.basic"
source = "../doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = ["../tests/lexical/pass/escape.expect.toml"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-MANIFEST-SOURCE-PATH");
    assert_has_code(&plan, "E-MANIFEST-TEST-PATH");
}

#[test]
fn manifest_duplicate_ids_fail() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.basic"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = []

[[requirement]]
id = "spec.en.test.basic"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = []
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let plan = corpus.plan();

    assert_has_code(&plan, "E-MANIFEST-DUP-ID");
}

#[test]
fn plan_order_is_deterministic_by_expectation_path() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.add_case("tests/lexical/pass/z_case", "z_case", "spec.en.test.basic");
    corpus.add_case("tests/lexical/pass/a_case", "a_case", "spec.en.test.basic");

    let plan = corpus.plan();
    let paths = plan
        .cases
        .iter()
        .map(|case| rel(&corpus.root, &case.expectation_path))
        .collect::<Vec<_>>();

    assert_eq!(
        paths,
        vec![
            PathBuf::from("tests/lexical/pass/a_case.expect.toml"),
            PathBuf::from("tests/lexical/pass/z_case.expect.toml"),
        ]
    );
}

struct Corpus {
    root: PathBuf,
}

impl Corpus {
    fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let root =
            std::env::temp_dir().join(format!("mizar-test-metadata-{}-{id}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let corpus = Self { root };
        corpus.create_standard_roots();
        corpus.write("tests/coverage/spec_trace.toml", "");
        corpus
    }

    fn create_standard_roots(&self) {
        for dir in [
            "tests/miz",
            "tests/lexical",
            "tests/certificates",
            "tests/generated",
            "tests/fuzz",
            "tests/property",
            "tests/snapshots",
            "tests/coverage",
            "doc/spec/en",
        ] {
            fs::create_dir_all(self.root.join(dir)).unwrap();
        }
    }

    fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn add_requirement(&self, id: &str, tests: &[&str]) {
        let tests = tests
            .iter()
            .map(|test| format!("\"{test}\""))
            .collect::<Vec<_>>()
            .join(", ");
        self.write(
            "tests/coverage/spec_trace.toml",
            format!(
                r#"
[[requirement]]
id = "{id}"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = [{tests}]
"#
            ),
        );
        self.write("doc/spec/en/test.md", "# Test\n");
    }

    fn add_case(&self, stem_path: &str, id: &str, spec_ref: &str) {
        self.write(format!("{stem_path}.src"), "");
        let source = Path::new(stem_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
            + ".src";
        self.write(
            format!("{stem_path}.expect.toml"),
            expectation(id, &source, spec_ref),
        );
    }

    fn plan(&self) -> mizar_test::TestPlan {
        let config = DiscoveryConfig {
            workspace_root: self.root.clone(),
            tests_root: self.root.join("tests"),
            manifest_path: self.root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        build_test_plan(&config).unwrap()
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn expectation(id: &str, source: &str, spec_ref: &str) -> String {
    format!(
        r#"schema_version = 1
id = "{id}"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "{source}"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["{spec_ref}"]
"#
    )
}

fn assert_has_code(plan: &mizar_test::TestPlan, code: &str) {
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.0 == code),
        "expected diagnostic {code}, got {:#?}",
        plan.diagnostics
    );
}

fn rel(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap().to_path_buf()
}
