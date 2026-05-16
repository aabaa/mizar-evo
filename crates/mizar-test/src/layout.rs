use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::{expectation_stem, payload_stem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredLayout {
    pub payloads: Vec<PathBuf>,
    pub sidecars: Vec<PathBuf>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

const KNOWN_ROOTS: &[&str] = &[
    "miz",
    "lexical",
    "certificates",
    "generated",
    "fuzz",
    "property",
    "snapshots",
];

pub fn discover(tests_root: &Path) -> Result<DiscoveredLayout, std::io::Error> {
    let mut payloads = Vec::new();
    let mut sidecars = Vec::new();
    let mut diagnostics = Vec::new();

    for root in KNOWN_ROOTS {
        let root_path = tests_root.join(root);
        if !root_path.exists() {
            diagnostics.push(ValidationDiagnostic::warning(
                root_path,
                "layout",
                "W-LAYOUT-MISSING-ROOT",
                format!("layout.root.{root}"),
                "optional test root is missing",
            ));
            continue;
        }
        walk(&root_path, &mut payloads, &mut sidecars)?;
    }

    payloads.sort();
    sidecars.sort();

    for payload in &payloads {
        let Some(stem) = payload_stem(payload) else {
            continue;
        };
        let sidecar = payload.with_file_name(format!("{stem}.expect.toml"));
        if !sidecar.is_file() {
            diagnostics.push(ValidationDiagnostic::error(
                payload,
                "layout",
                "E-LAYOUT-MISSING-SIDECAR",
                "layout.sidecar",
                format!(
                    "payload `{}` is missing adjacent expectation sidecar",
                    payload.display()
                ),
            ));
        }
    }

    for sidecar in &sidecars {
        if expectation_stem(sidecar).is_none() {
            diagnostics.push(ValidationDiagnostic::error(
                sidecar,
                "layout",
                "E-LAYOUT-BAD-SIDECAR-NAME",
                "layout.sidecar_name",
                "expectation sidecar must end in `.expect.toml`",
            ));
        }
    }

    Ok(DiscoveredLayout {
        payloads,
        sidecars,
        diagnostics,
    })
}

fn walk(
    root: &Path,
    payloads: &mut Vec<PathBuf>,
    sidecars: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk(&path, payloads, sidecars)?;
        } else if file_type.is_file() {
            if is_payload(&path) {
                payloads.push(path);
            } else if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".expect.toml"))
            {
                sidecars.push(path);
            }
        }
    }
    Ok(())
}

fn is_payload(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.ends_with(".miz")
        || name.ends_with(".src")
        || name.ends_with(".cert.json")
        || name.ends_with(".fixture.toml")
}
