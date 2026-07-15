use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use mizar_test::{ValidationSeverity, layout};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

const KNOWN_ROOTS: &[&str] = &[
    "miz",
    "lexical",
    "certificates",
    "generated",
    "fuzz",
    "property",
    "stress",
    "snapshots",
];

#[test]
fn discover_sorts_raw_payload_and_sidecar_inventories() {
    let corpus = LayoutCorpus::new();
    corpus.create_known_roots_except(None);
    corpus.write("miz/z_case.miz", "");
    corpus.write("miz/z_case.expect.toml", "");
    corpus.write("lexical/a_case.src", "");
    corpus.write("lexical/a_case.expect.toml", "");

    let discovered = layout::discover(corpus.path()).unwrap();

    let mut expected_payloads = vec![
        corpus.join("miz/z_case.miz"),
        corpus.join("lexical/a_case.src"),
    ];
    expected_payloads.sort();
    let mut expected_sidecars = vec![
        corpus.join("miz/z_case.expect.toml"),
        corpus.join("lexical/a_case.expect.toml"),
    ];
    expected_sidecars.sort();

    assert_eq!(discovered.payloads, expected_payloads);
    assert_eq!(discovered.sidecars, expected_sidecars);
    assert!(discovered.diagnostics.is_empty());
}

#[test]
fn discover_reports_missing_known_root_warning() {
    let corpus = LayoutCorpus::new();
    corpus.create_known_roots_except(Some("stress"));

    let discovered = layout::discover(corpus.path()).unwrap();

    assert!(discovered.payloads.is_empty());
    assert!(discovered.sidecars.is_empty());
    assert_eq!(discovered.diagnostics.len(), 1);
    let diagnostic = &discovered.diagnostics[0];
    assert_eq!(diagnostic.severity, ValidationSeverity::Warning);
    assert_eq!(diagnostic.path, corpus.join("stress"));
    assert_eq!(diagnostic.record_kind, "layout");
    assert_eq!(diagnostic.code.0, "W-LAYOUT-MISSING-ROOT");
    assert_eq!(diagnostic.detail_key, "layout.root.stress");
}

#[test]
fn unknown_roots_returns_multiple_directories_sorted() {
    let corpus = LayoutCorpus::new();
    corpus.create_dir("z_unknown");
    corpus.create_dir("alpha_unknown");
    corpus.create_dir("miz");
    corpus.create_dir("coverage");
    corpus.write("ordinary.txt", "not a test root\n");

    let unknown = layout::unknown_roots(corpus.path()).unwrap();

    let mut expected = vec![corpus.join("z_unknown"), corpus.join("alpha_unknown")];
    expected.sort();
    assert_eq!(unknown, expected);
}

struct LayoutCorpus {
    root: PathBuf,
}

impl LayoutCorpus {
    fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let root =
            std::env::temp_dir().join(format!("mizar-test-layout-{}-{id}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn join(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    fn create_known_roots_except(&self, omitted: Option<&str>) {
        for root in KNOWN_ROOTS {
            if Some(*root) != omitted {
                self.create_dir(root);
            }
        }
    }

    fn create_dir(&self, relative: impl AsRef<Path>) {
        fs::create_dir_all(self.join(relative)).unwrap();
    }

    fn write(&self, relative: impl AsRef<Path>, content: impl AsRef<[u8]>) {
        let path = self.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }
}

impl Drop for LayoutCorpus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
