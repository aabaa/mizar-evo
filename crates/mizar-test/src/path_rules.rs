use std::path::{Component, Path, PathBuf};

pub fn clean_relative_path(path: &Path) -> bool {
    let Some(raw_path) = path.to_str() else {
        return false;
    };

    !path.as_os_str().is_empty()
        && raw_path
            .split(['/', '\\'])
            .all(|component| !matches!(component, "" | "." | ".."))
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

pub fn absolute_from(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        normalize_lexically(path)
    } else {
        normalize_lexically(&base.join(path))
    }
}

pub fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

pub fn executable_payload_stem(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    for suffix in [".fixture.toml", ".cert.json", ".miz", ".src"] {
        if let Some(stem) = name.strip_suffix(suffix) {
            return Some(stem.to_owned());
        }
    }
    None
}
