//! Validated logical destinations, independent of link lookup and href encoding.

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use mizar_artifact::{
    module_summary::{ExportedLabelSummary, ModuleSummaryIdentity},
    verified_artifact::VerifiedExport,
};
use mizar_session::Hash;

use crate::doc_build::{AuthenticatedDocImports, DocBuildError, DocImportCoverage};

/// An exact exported item and its supplied authoritative label and optional alias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocItemAllocation {
    /// Exact member of the module's authenticated export set.
    pub export: VerifiedExport,
    /// Exact stored label; absence prevents this item's anchor from being exposed.
    pub label: Option<ExportedLabelSummary>,
    /// Requested short anchor, suppressed when unsafe.
    pub alias: Option<String>,
}

/// A logical page with its optional authenticated module binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocPageAllocation {
    /// Slash-separated path relative to the selected output root.
    pub path: String,
    /// Full module identity and publication hash, absent for non-module pages.
    pub module: Option<(ModuleSummaryIdentity, Hash)>,
    /// Documented exported items on this page.
    pub items: Vec<DocItemAllocation>,
    /// Exact declared section DOM ids.
    pub section_ids: Vec<String>,
}

/// An explicit logical specification section and its declared destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocSectionAllocation {
    /// Producer-owned logical section id, without consumer parsing.
    pub id: String,
    /// Logical page path.
    pub path: String,
    /// Exact declared section DOM id.
    pub fragment: String,
}

/// Specification-version-bound section inventory, including explicit coverage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocSectionIndex {
    /// Exact specification version supplied by the producer.
    pub version: String,
    /// Completeness of this section inventory.
    pub coverage: DocImportCoverage,
    /// Known explicit section destinations.
    pub sections: Vec<DocSectionAllocation>,
}

/// An explicitly supplied destination namespace awaiting validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocAllocationTable {
    /// Selected output root to which the logical paths are bound.
    pub output_root: PathBuf,
    /// Completeness of the entire supplied page namespace.
    pub coverage: DocImportCoverage,
    /// Allocated logical pages.
    pub pages: Vec<DocPageAllocation>,
    /// Explicit versioned specification inventory, if available.
    pub sections: Option<DocSectionIndex>,
}

/// Immutable validated destinations; blocked pages and unsafe aliases stay hidden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedDocAllocations {
    table: DocAllocationTable,
    blocked_pages: BTreeSet<String>,
    section_coverage: DocImportCoverage,
}

impl ValidatedDocAllocations {
    /// Returns the exact selected output root.
    pub fn output_root(&self) -> &Path {
        &self.table.output_root
    }

    /// Returns destination-table coverage, independent of import/citation coverage.
    pub fn coverage(&self) -> DocImportCoverage {
        self.table.coverage
    }

    /// Returns a safe logical page for the exact module/publication identity.
    pub fn module_page(&self, module: &ModuleSummaryIdentity, publication: Hash) -> Option<&str> {
        self.table.pages.iter().find_map(|page| {
            (self.page_ready(&page.path)
                && page
                    .module
                    .as_ref()
                    .is_some_and(|(identity, hash)| identity == module && *hash == publication))
            .then_some(page.path.as_str())
        })
    }

    /// Returns a logical page, exact full FQN and safe optional alias.
    pub fn item(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
        export: &VerifiedExport,
    ) -> Option<(&str, &str, Option<&str>)> {
        let path = self.module_page(module, publication)?;
        let page = self.table.pages.iter().find(|page| page.path == path)?;
        let item = page.items.iter().find(|item| &item.export == export)?;
        item.label.as_ref()?;
        Some((
            path,
            &item.export.fully_qualified_name,
            item.alias.as_deref(),
        ))
    }

    /// Returns section-inventory coverage after page-readiness restrictions.
    pub fn section_coverage(&self) -> DocImportCoverage {
        self.section_coverage
    }

    /// Returns a known safe section, retaining partial coverage separately.
    pub fn section(&self, id: &str) -> Option<(&str, &str)> {
        let index = self.table.sections.as_ref()?;
        if index.coverage == DocImportCoverage::Unavailable {
            return None;
        }
        let section = index.sections.iter().find(|section| section.id == id)?;
        self.page_ready(&section.path)
            .then_some((section.path.as_str(), section.fragment.as_str()))
    }

    fn page_ready(&self, path: &str) -> bool {
        self.table.coverage == DocImportCoverage::Complete && !self.blocked_pages.contains(path)
    }
}

/// Validates the whole supplied namespace without choosing paths or repairing ids.
pub fn validate_doc_allocations(
    inputs: &AuthenticatedDocImports,
    output_root: &Path,
    specification_version: &str,
    mut table: DocAllocationTable,
) -> Result<ValidatedDocAllocations, DocBuildError> {
    if table.output_root.as_os_str() != output_root.as_os_str() {
        return Err(DocBuildError::Binding("documentation output root mismatch"));
    }
    let mut paths = BTreeSet::new();
    let mut modules = BTreeSet::new();
    let mut blocked_pages = BTreeSet::new();
    for page in &mut table.pages {
        let bytes = page.path.as_bytes();
        let drive_prefix =
            bytes.first().is_some_and(u8::is_ascii_alphabetic) && bytes.get(1) == Some(&b':');
        if drive_prefix
            || page.path.contains(['\\', '\0'])
            || page
                .path
                .split('/')
                .any(|part| matches!(part, "" | "." | ".."))
            || !paths.insert(page.path.clone())
        {
            return Err(DocBuildError::Binding(
                "invalid or duplicate documentation page path",
            ));
        }
        let mut section_ids = BTreeSet::new();
        if page
            .section_ids
            .iter()
            .any(|id| !section_ids.insert(id.clone()))
        {
            return Err(DocBuildError::Binding(
                "duplicate documentation section DOM id",
            ));
        }
        let Some((module, publication)) = &page.module else {
            if !page.items.is_empty() {
                return Err(DocBuildError::Binding(
                    "documentation items lack module binding",
                ));
            }
            continue;
        };
        let exports = inputs
            .exports(module, *publication)
            .ok_or(DocBuildError::Binding(
                "documentation page lacks authenticated module publication",
            ))?;
        if !modules.insert(module.clone()) {
            return Err(DocBuildError::Binding(
                "duplicate documentation module allocation",
            ));
        }
        let summary = inputs.summary(module, *publication);
        let mut items: Vec<DocItemAllocation> = Vec::new();
        for item in page.items.drain(..) {
            if !exports.contains(&item.export) {
                return Err(DocBuildError::Binding(
                    "documentation item is not an authenticated export",
                ));
            }
            if let Some(label) = &item.label
                && (label.owner_fully_qualified_name != item.export.fully_qualified_name
                    || !summary.is_some_and(|summary| summary.exported_labels.contains(label)))
            {
                return Err(DocBuildError::Binding(
                    "documentation label is not authoritative for item",
                ));
            }
            if let Some(previous) = items.iter().find(|previous| previous.export == item.export) {
                if previous != &item {
                    return Err(DocBuildError::Binding(
                        "contradictory documentation item allocations",
                    ));
                }
            } else {
                items.push(item);
            }
        }
        let mut full_ids = BTreeSet::new();
        for item in &items {
            let id = &item.export.fully_qualified_name;
            if !full_ids.insert(id.clone()) || section_ids.contains(id) {
                blocked_pages.insert(page.path.clone());
            }
        }
        let labels_complete = items.iter().all(|item| item.label.is_some());
        let mut labels = std::collections::BTreeMap::new();
        for label in items.iter().filter_map(|item| item.label.as_ref()) {
            *labels.entry(label.label.clone()).or_insert(0usize) += 1;
        }
        for item in &mut items {
            let safe = labels_complete
                && item.alias.as_ref().is_some_and(|alias| {
                    item.label
                        .as_ref()
                        .is_some_and(|label| &label.label == alias)
                        && labels.get(alias) == Some(&1)
                        && !full_ids.contains(alias)
                        && !section_ids.contains(alias)
                });
            if !safe {
                item.alias = None;
            }
        }
        page.items = items;
    }
    for path in &paths {
        if path
            .match_indices('/')
            .any(|(offset, _)| paths.contains(&path[..offset]))
        {
            return Err(DocBuildError::Binding(
                "documentation page file/directory collision",
            ));
        }
    }
    let mut section_coverage = DocImportCoverage::Unavailable;
    if let Some(index) = &table.sections {
        if index.version != specification_version {
            return Err(DocBuildError::Binding(
                "documentation specification version mismatch",
            ));
        }
        section_coverage = index.coverage;
        let mut ids = BTreeSet::new();
        for section in &index.sections {
            if !ids.insert(&section.id) {
                return Err(DocBuildError::Binding(
                    "duplicate documentation logical section id",
                ));
            }
            let page = table
                .pages
                .iter()
                .find(|page| page.path == section.path)
                .ok_or(DocBuildError::Binding(
                    "documentation section page is not allocated",
                ))?;
            if !page.section_ids.contains(&section.fragment) {
                return Err(DocBuildError::Binding(
                    "documentation section fragment is not declared",
                ));
            }
            if blocked_pages.contains(&section.path)
                && section_coverage != DocImportCoverage::Unavailable
            {
                section_coverage = DocImportCoverage::Incomplete;
            }
        }
        if table.coverage != DocImportCoverage::Complete
            && section_coverage != DocImportCoverage::Unavailable
        {
            section_coverage = DocImportCoverage::Incomplete;
        }
    }
    Ok(ValidatedDocAllocations {
        table,
        blocked_pages,
        section_coverage,
    })
}

/// Encodes logical path segments and the whole fragment; authenticates nothing.
pub fn encode_doc_href(path: &str, fragment: Option<&str>) -> String {
    let mut href = String::new();
    for (text, is_path) in
        std::iter::once((path, true)).chain(fragment.into_iter().map(|fragment| (fragment, false)))
    {
        if !is_path {
            href.push('#');
        }
        for byte in text.bytes() {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) || (is_path && byte == b'/')
            {
                href.push(char::from(byte));
            } else {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                href.push('%');
                href.push(char::from(HEX[usize::from(byte >> 4)]));
                href.push(char::from(HEX[usize::from(byte & 15)]));
            }
        }
    }
    href
}
