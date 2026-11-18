//! Validated logical destinations, independent of link lookup and href encoding.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use mizar_artifact::{
    module_summary::{ExportedLabelSummary, ModuleSummaryIdentity},
    verified_artifact::VerifiedExport,
};
use mizar_session::Hash;

use crate::doc_build::{
    AuthenticatedDocIdentities, AuthenticatedDocImports, DocBuildError, DocImportCoverage,
};

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
    specification_version: String,
    inventory: Vec<(ModuleSummaryIdentity, Hash)>,
    inventory_publications: Vec<Hash>,
}

impl ValidatedDocAllocations {
    pub(crate) fn matches_inputs(
        &self,
        inputs: &AuthenticatedDocImports,
        specification_version: &str,
    ) -> bool {
        self.specification_version == specification_version
            && self.inventory_publications == inputs.inventory_publications()
            && self.inventory
                == inputs
                    .inventory()
                    .map(|(module, hash)| (module.clone(), hash))
                    .collect::<Vec<_>>()
            && validate_doc_allocations(
                inputs,
                self.output_root(),
                specification_version,
                self.table.clone(),
            )
            .is_ok_and(|rebound| rebound == *self)
    }

    pub(crate) fn page_causes(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
        export: Option<&VerifiedExport>,
    ) -> Vec<&'static str> {
        let mut causes = Vec::new();
        let page = self.table.pages.iter().find(|page| {
            page.module
                .as_ref()
                .is_some_and(|(identity, hash)| identity == module && *hash == publication)
        });
        if self.table.coverage != DocImportCoverage::Complete || page.is_none() {
            causes.push("page_unavailable");
        }
        if let Some(page) = page {
            if self.blocked_pages.contains(&page.path) {
                causes.push("anchor_collision");
            }
            if let Some(export) = export
                && !page
                    .items
                    .iter()
                    .any(|item| &item.export == export && item.label.is_some())
            {
                causes.push("label_unavailable");
            }
        }
        causes
    }

    pub(crate) fn section_causes(&self) -> Vec<&'static str> {
        let mut causes = match self.section_coverage {
            DocImportCoverage::Complete => Vec::new(),
            DocImportCoverage::Unavailable => vec!["sections_unavailable"],
            _ => vec!["sections_partial"],
        };
        if self.table.coverage != DocImportCoverage::Complete {
            causes.push("page_unavailable");
        }
        if self.table.sections.as_ref().is_some_and(|index| {
            index
                .sections
                .iter()
                .any(|section| self.blocked_pages.contains(&section.path))
        }) {
            causes.push("anchor_collision");
        }
        causes
    }

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

/// Complete generated module-entry and dedicated-item destinations for exact identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocDestinationAllocations {
    allocations: ValidatedDocAllocations,
    dedicated_paths: BTreeMap<(String, String), String>,
}

impl DocDestinationAllocations {
    /// Existing Doc9 references retain their module-page destinations.
    pub fn allocations(&self) -> &ValidatedDocAllocations {
        &self.allocations
    }

    /// Returns the paired dedicated page, exact full FQN and shared safe alias.
    pub fn dedicated_item(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
        export: &VerifiedExport,
    ) -> Option<(&str, &str, Option<&str>)> {
        let (module_path, fqn, alias) = self.allocations.item(module, publication, export)?;
        let path = self
            .dedicated_paths
            .get(&(module_path.to_owned(), fqn.to_owned()))?;
        Some((path, fqn, alias))
    }
}

/// Allocates the complete declared Q1/Q2 namespace without emitting files.
pub fn allocate_doc_destinations(
    inputs: &AuthenticatedDocImports,
    identities: &AuthenticatedDocIdentities,
    doc_dir: &Path,
    specification_version: &str,
    mut auxiliary: DocAllocationTable,
) -> Result<DocDestinationAllocations, DocBuildError> {
    if !identities.matches_inputs(inputs) {
        return Err(DocBuildError::Binding(
            "destination identity inventory mismatch",
        ));
    }
    let version = inputs
        .publications()
        .first()
        .and_then(|publication| publication.manifest.package.package_version.as_deref())
        .ok_or(DocBuildError::Binding(
            "destination package version is required",
        ))?;
    let output_root = doc_dir.join(format!("v{}", encode_doc_component(version)?));
    if auxiliary.output_root != output_root
        || auxiliary.coverage != DocImportCoverage::Complete
        || auxiliary
            .pages
            .iter()
            .any(|page| page.module.is_some() || !page.items.is_empty())
    {
        return Err(DocBuildError::Binding(
            "incomplete or mismatched auxiliary namespace",
        ));
    }
    let mut dedicated_paths = BTreeMap::new();
    for identity in identities.identities() {
        let artifact = identity.artifact();
        let components = identity
            .module_components()
            .iter()
            .map(|component| encode_doc_component(component))
            .collect::<Result<Vec<_>, _>>()?;
        if components.is_empty() {
            return Err(DocBuildError::Binding("empty destination module path"));
        }
        let module_path = if components.len() == 1 {
            format!("{}/index.html", components[0])
        } else {
            format!("{}.html", components.join("/"))
        };
        let summary = inputs.summary(&artifact.module, identity.publication_hash());
        let mut full_ids = BTreeSet::new();
        let mut origins = BTreeSet::new();
        let mut items = Vec::new();
        for export in &artifact.exports {
            if !full_ids.insert(&export.fully_qualified_name) || !origins.insert(&export.origin_id)
            {
                return Err(DocBuildError::Binding(
                    "duplicate destination export identity",
                ));
            }
            let labels = summary
                .into_iter()
                .flat_map(|summary| &summary.exported_labels)
                .filter(|label| label.owner_fully_qualified_name == export.fully_qualified_name)
                .collect::<Vec<_>>();
            let [label] = labels.as_slice() else {
                return Err(DocBuildError::Binding(
                    "missing or ambiguous destination label",
                ));
            };
            let path = format!(
                "{}/items/{}.html",
                components.join("/"),
                encode_doc_component(&label.label)?
            );
            dedicated_paths.insert(
                (module_path.clone(), export.fully_qualified_name.clone()),
                path.clone(),
            );
            auxiliary.pages.push(DocPageAllocation {
                path,
                module: None,
                items: Vec::new(),
                section_ids: vec![export.fully_qualified_name.clone()],
            });
            items.push(DocItemAllocation {
                export: export.clone(),
                label: Some((*label).clone()),
                alias: Some(label.label.clone()),
            });
        }
        items.sort_by(|left, right| {
            left.export
                .fully_qualified_name
                .cmp(&right.export.fully_qualified_name)
                .then_with(|| left.export.origin_id.cmp(&right.export.origin_id))
        });
        auxiliary.pages.push(DocPageAllocation {
            path: module_path,
            module: Some((artifact.module.clone(), identity.publication_hash())),
            items,
            section_ids: Vec::new(),
        });
    }
    // Every prefix has one exact spelling and one file/directory role.
    let mut namespace = BTreeMap::new();
    for (path, file) in [
        ("index.html", true),
        ("search.json", true),
        ("cross-references.json", true),
        ("assets", false),
    ]
    .into_iter()
    .chain(
        auxiliary
            .pages
            .iter()
            .map(|page| (page.path.as_str(), true)),
    ) {
        if file
            && path
                .split('/')
                .next()
                .is_some_and(|part| part.eq_ignore_ascii_case("assets"))
        {
            return Err(DocBuildError::Binding(
                "reserved documentation assets namespace",
            ));
        }
        let mut prefix = String::new();
        let mut parts = path.split('/').peekable();
        while let Some(part) = parts.next() {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            let is_file = file && parts.peek().is_none();
            let key = prefix.to_ascii_lowercase();
            if let Some((spelling, was_file)) = namespace.get(&key) {
                if spelling != &prefix || *was_file || is_file {
                    return Err(DocBuildError::Binding("documentation namespace collision"));
                }
            } else {
                namespace.insert(key, (prefix.clone(), is_file));
            }
        }
    }
    for page in &mut auxiliary.pages {
        page.section_ids.sort();
    }
    auxiliary
        .pages
        .sort_by(|left, right| left.path.cmp(&right.path));
    if let Some(index) = &mut auxiliary.sections {
        index.sections.sort_by(|left, right| {
            (&left.id, &left.path, &left.fragment).cmp(&(&right.id, &right.path, &right.fragment))
        });
    }
    let mut allocations =
        validate_doc_allocations(inputs, &output_root, specification_version, auxiliary)?;
    if !allocations.blocked_pages.is_empty() {
        return Err(DocBuildError::Binding("blocked destination full id"));
    }
    for identity in identities.identities() {
        let artifact = identity.artifact();
        if allocations
            .module_page(&artifact.module, identity.publication_hash())
            .is_none()
        {
            return Err(DocBuildError::Binding(
                "missing generated module destination",
            ));
        }
        for export in &artifact.exports {
            let (module_path, fqn, alias) = allocations
                .item(&artifact.module, identity.publication_hash(), export)
                .ok_or(DocBuildError::Binding("missing generated item destination"))?;
            let path = &dedicated_paths[&(module_path.to_owned(), fqn.to_owned())];
            if let Some(alias) = alias.map(str::to_owned) {
                let page = allocations
                    .table
                    .pages
                    .iter_mut()
                    .find(|page| &page.path == path)
                    .ok_or(DocBuildError::Binding(
                        "missing generated dedicated destination",
                    ))?;
                page.section_ids.push(alias);
                page.section_ids.sort();
            }
        }
    }
    Ok(DocDestinationAllocations {
        allocations,
        dedicated_paths,
    })
}

fn encode_doc_component(component: &str) -> Result<String, DocBuildError> {
    if component.is_empty() {
        return Err(DocBuildError::Binding("empty destination component"));
    }
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::new();
    for (index, byte) in component.bytes().enumerate() {
        if byte.is_ascii_alphanumeric()
            || matches!(byte, b'_' | b'-')
            || byte == b'.' && !matches!(component, "." | "..") && index + 1 != component.len()
        {
            encoded.push(char::from(byte));
        } else {
            encoded.push('~');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 15)]));
        }
    }
    Ok(encoded)
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
        specification_version: specification_version.to_owned(),
        inventory: inputs
            .inventory()
            .map(|(module, hash)| (module.clone(), hash))
            .collect(),
        inventory_publications: inputs.inventory_publications(),
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
