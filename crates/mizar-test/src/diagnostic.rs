use std::cmp::Ordering;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagnosticCode(pub &'static str);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    pub severity: ValidationSeverity,
    pub path: PathBuf,
    pub record_kind: &'static str,
    pub code: DiagnosticCode,
    pub detail_key: String,
    pub message: String,
}

impl ValidationDiagnostic {
    pub fn error(
        path: impl Into<PathBuf>,
        record_kind: &'static str,
        code: &'static str,
        detail_key: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            ValidationSeverity::Error,
            path,
            record_kind,
            code,
            detail_key,
            message,
        )
    }

    pub fn warning(
        path: impl Into<PathBuf>,
        record_kind: &'static str,
        code: &'static str,
        detail_key: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            ValidationSeverity::Warning,
            path,
            record_kind,
            code,
            detail_key,
            message,
        )
    }

    fn new(
        severity: ValidationSeverity,
        path: impl Into<PathBuf>,
        record_kind: &'static str,
        code: &'static str,
        detail_key: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            path: path.into(),
            record_kind,
            code: DiagnosticCode(code),
            detail_key: detail_key.into(),
            message: message.into(),
        }
    }
}

impl Ord for ValidationDiagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path
            .cmp(&other.path)
            .then(self.record_kind.cmp(other.record_kind))
            .then(self.code.cmp(&other.code))
            .then(self.detail_key.cmp(&other.detail_key))
    }
}

impl PartialOrd for ValidationDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for ValidationDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            ValidationSeverity::Error => "error",
            ValidationSeverity::Warning => "warning",
        };
        write!(
            f,
            "{severity}[{}] {}: {}",
            self.code.0,
            display_path(&self.path),
            self.message
        )
    }
}

fn display_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_owned()
    } else {
        path.display().to_string()
    }
}
