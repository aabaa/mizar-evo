//! Cache-side cluster-db origin records and aggregate indexes.
//!
//! The design is specified in
//! [cluster_db.md](../../../doc/design/mizar-cache/en/cluster_db.md).

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use mizar_session::Hash;

use crate::cache_key::{
    CompatibilityField, NamedHash, NamedSchemaVersion, PolicyFingerprint, SchemaVersion,
};

/// Current cluster-db schema version.
pub const CLUSTER_DB_SCHEMA_VERSION: &str = "mizar-cache/cluster-db-schema/v1";
/// Domain used for cluster-db cache-side hashes.
pub const CLUSTER_DB_HASH_DOMAIN: &str = "mizar-cache/cluster-db/v1";

/// Visibility status exported by the contribution owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterContributionVisibility {
    /// Contribution is visible to importers.
    ImporterVisible,
    /// Contribution is private to its module or package.
    Private,
    /// Contribution is local-only and cannot enter importer-visible indexes.
    LocalOnly,
}

/// Accepted-status class exported by the contribution owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterContributionStatus {
    /// The owner projected the contribution as accepted.
    Accepted,
    /// The contribution is pending proof or validation.
    Pending,
    /// The contribution was rejected.
    Rejected,
    /// The contribution was recovered from malformed source.
    Recovered,
    /// Only external attestation is available.
    ExternallyAttested,
}

/// Cluster-db contribution kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterContributionKind {
    /// Existential registration contribution.
    Existential,
    /// Conditional cluster contribution.
    ConditionalCluster,
    /// Functorial cluster contribution.
    FunctorialCluster,
    /// Reduction rule contribution.
    Reduction,
    /// Struct inheritance edge contribution.
    StructInheritance,
    /// Registration-subsumption DAG node.
    SubsumptionNode,
    /// Registration-subsumption DAG edge.
    SubsumptionEdge,
}

/// Dependency-footprint completeness for one origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterOriginFootprintCompleteness {
    /// Origin metadata covers every required dependency family.
    Complete,
    /// The origin is coarser than ideal but complete for reuse.
    ConservativeComplete,
    /// The origin is incomplete and must not be reused.
    IncompleteUncacheable,
}

/// Aggregate index entry kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterIndexEntryKind {
    /// Conditional-cluster or inheritance graph row.
    Graph,
    /// Registration-subsumption DAG row.
    SubsumptionDag,
    /// Attribute inverted-index row.
    Attribute,
    /// Type or mode inverted-index row.
    Type,
    /// Reduction inverted-index row.
    Reduction,
}

/// Origin metadata for one accepted contribution candidate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClusterContributionOrigin {
    /// Cache-side origin identity.
    pub origin_key: String,
    /// Owning package id.
    pub package_id: String,
    /// Owning module path.
    pub module_path: String,
    /// Producer-owned stable contribution id.
    pub stable_contribution_id: String,
    /// Source label or stable generated label.
    pub label: String,
    /// Contribution kind.
    pub contribution_kind: ClusterContributionKind,
    /// Canonical target/type/term pattern or DAG-node key hash.
    pub target_pattern_hash: Hash,
    /// Canonical guard and side-condition hash.
    pub guard_hash: Hash,
    /// Canonical generated contribution payload hash.
    pub declared_contribution_hash: Hash,
    /// Import visibility projected by the owner.
    pub accepted_visibility: ClusterContributionVisibility,
    /// Status projected by the owner.
    pub accepted_status: ClusterContributionStatus,
    /// Hash of the owner-projected accepted status.
    pub accepted_status_projection_hash: Hash,
    /// Accepted witness or deterministic-discharge identity for proof-backed contributions.
    pub accepted_witness_or_discharge_hash: Option<NamedHash>,
    /// Whether this accepted contribution depends on proof/discharge validation.
    pub proof_backed: bool,
    /// Verifier policy under which the contribution is visible.
    pub verifier_policy_fingerprint: PolicyFingerprint,
    /// Policy compatibility fields. Empty or unknown fields are fail-closed.
    pub policy_compatibility: Vec<CompatibilityField>,
    /// Producer schema compatibility fields. Empty or unknown fields are fail-closed.
    pub schema_compatibility: Vec<CompatibilityField>,
    /// Producer toolchain compatibility fields. Empty or unknown fields are fail-closed.
    pub toolchain_compatibility: Vec<CompatibilityField>,
    /// Producer schemas needed to interpret this origin.
    pub producer_schema_versions: Vec<NamedSchemaVersion>,
    /// Trace replay hashes required by this origin. `None` is incomplete.
    pub trace_replay_hashes: Option<Vec<NamedHash>>,
    /// Dependency-facing interface hashes required by this origin. `None` is incomplete.
    pub dependency_interface_hashes: Option<Vec<NamedHash>>,
    /// Complete footprint hash for stale-origin detection.
    pub origin_footprint_hash: Option<Hash>,
    /// Origin dependency completeness.
    pub footprint_completeness: ClusterOriginFootprintCompleteness,
    /// Explicit uncacheable marker.
    pub uncacheable: bool,
}

/// One producer-supplied aggregate-index row.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClusterIndexEntry {
    /// Aggregate index kind.
    pub entry_kind: ClusterIndexEntryKind,
    /// Primary lookup key, such as attribute, type, graph, or term head.
    pub primary_key: String,
    /// Secondary lookup key, such as generated target, rule FQN, or edge endpoint.
    pub secondary_key: String,
    /// Stable strategy or simplification key when applicable.
    pub strategy_key: String,
    /// Producer-owned contribution fingerprint for this row.
    pub contribution_fingerprint: Hash,
}

/// Validated origin record and its aggregate rows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClusterContributionRecord {
    /// Cluster-db schema version for this cache-side record.
    pub schema_version: SchemaVersion,
    /// Origin metadata.
    pub origin: ClusterContributionOrigin,
    /// Declaration order within the origin module.
    pub declaration_order: u32,
    /// Producer-supplied aggregate index entries.
    pub index_entries: Vec<ClusterIndexEntry>,
    /// Diagnostic-only refs for explaining misses.
    pub diagnostic_refs: Vec<NamedHash>,
}

/// Canonical aggregate row stored in an index snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClusterAggregateRow {
    /// Origin identity.
    pub origin_key: String,
    /// Origin package id.
    pub package_id: String,
    /// Origin module path.
    pub module_path: String,
    /// Declaration order within the origin module.
    pub declaration_order: u32,
    /// Primary lookup key.
    pub primary_key: String,
    /// Secondary lookup key.
    pub secondary_key: String,
    /// Stable strategy or simplification key.
    pub strategy_key: String,
    /// Producer-owned contribution fingerprint.
    pub contribution_fingerprint: Hash,
}

impl PartialOrd for ClusterAggregateRow {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClusterAggregateRow {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            &self.primary_key,
            &self.package_id,
            &self.module_path,
            self.declaration_order,
            &self.secondary_key,
            &self.strategy_key,
            &self.origin_key,
            self.contribution_fingerprint.as_bytes(),
        )
            .cmp(&(
                &other.primary_key,
                &other.package_id,
                &other.module_path,
                other.declaration_order,
                &other.secondary_key,
                &other.strategy_key,
                &other.origin_key,
                other.contribution_fingerprint.as_bytes(),
            ))
    }
}

/// Aggregate cluster-db indexes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClusterAggregateIndexes {
    /// Conditional cluster and inheritance graph rows.
    pub graph_rows: Vec<ClusterAggregateRow>,
    /// Registration-subsumption DAG rows.
    pub subsumption_dag_rows: Vec<ClusterAggregateRow>,
    /// Attribute inverted-index rows.
    pub attr_index_rows: Vec<ClusterAggregateRow>,
    /// Type or mode inverted-index rows.
    pub type_index_rows: Vec<ClusterAggregateRow>,
    /// Reduction inverted-index rows.
    pub reduction_index_rows: Vec<ClusterAggregateRow>,
}

/// Complete cluster-db snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClusterIndexSnapshot {
    /// Canonical accepted origins.
    pub origins: Vec<ClusterContributionRecord>,
    /// Aggregate indexes derived from the origins.
    pub indexes: ClusterAggregateIndexes,
}

/// Request for materializing an import-scoped view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportScopedViewRequest {
    /// Importing package id.
    pub importing_package_id: String,
    /// Importing module path.
    pub importing_module_path: String,
    /// Canonical import-closure identity.
    pub import_closure_identity: String,
    /// Visible origin keys for this import closure.
    pub visible_origin_keys: Vec<String>,
    /// Active verifier policy.
    pub verifier_policy_fingerprint: PolicyFingerprint,
    /// Cluster-db schema version expected by the consumer.
    pub cluster_db_schema_version: SchemaVersion,
    /// Producer schemas required by this view.
    pub producer_schema_versions: Vec<NamedSchemaVersion>,
    /// Policy compatibility fields for this view.
    pub policy_compatibility: Vec<CompatibilityField>,
    /// Schema compatibility fields for this view.
    pub schema_compatibility: Vec<CompatibilityField>,
    /// Toolchain compatibility fields for this view.
    pub toolchain_compatibility: Vec<CompatibilityField>,
    /// Traversal/profile settings that affect closure or reduction strategy.
    pub traversal_profile: Vec<CompatibilityField>,
}

/// Canonical import-scoped view key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportScopedViewKey {
    /// Importing package id.
    pub importing_package_id: String,
    /// Importing module path.
    pub importing_module_path: String,
    /// Canonical import-closure identity.
    pub import_closure_identity: String,
    /// Hash of the sorted visible origin set.
    pub visible_origin_set_hash: Hash,
    /// Active verifier policy.
    pub verifier_policy_fingerprint: PolicyFingerprint,
    /// Cluster-db schema version.
    pub cluster_db_schema_version: SchemaVersion,
    /// Producer schemas required by this view.
    pub producer_schema_versions: Vec<NamedSchemaVersion>,
    /// Policy compatibility fields for this view.
    pub policy_compatibility: Vec<CompatibilityField>,
    /// Schema compatibility fields for this view.
    pub schema_compatibility: Vec<CompatibilityField>,
    /// Toolchain compatibility fields for this view.
    pub toolchain_compatibility: Vec<CompatibilityField>,
    /// Traversal/profile settings that affect graph closure or reduction strategy.
    pub traversal_profile: Vec<CompatibilityField>,
}

/// Materialized import-scoped view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportScopedView {
    /// View key.
    pub key: ImportScopedViewKey,
    /// Sorted visible origin keys.
    pub visible_origin_keys: Vec<String>,
    /// Filtered aggregate indexes.
    pub indexes: ClusterAggregateIndexes,
}

/// Fail-closed import-scoped view miss.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClusterDbViewMiss {
    /// A required identity field was empty.
    MissingRequiredIdentity {
        /// Field name.
        field: &'static str,
    },
    /// Requested schema is unsupported.
    UnsupportedSchema {
        /// Actual schema.
        actual: SchemaVersion,
    },
    /// Required schema, policy, toolchain, traversal, or producer compatibility is unknown.
    UnknownCompatibility {
        /// Compatibility family.
        family: &'static str,
    },
    /// A requested visible origin does not exist.
    MissingVisibleOrigin {
        /// Missing origin key.
        origin_key: String,
    },
}

/// In-memory cluster-db index data layer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClusterDbIndex {
    origins: BTreeMap<String, ClusterContributionRecord>,
    graph_rows: BTreeSet<ClusterAggregateRow>,
    subsumption_dag_rows: BTreeSet<ClusterAggregateRow>,
    attr_index_rows: BTreeSet<ClusterAggregateRow>,
    type_index_rows: BTreeSet<ClusterAggregateRow>,
    reduction_index_rows: BTreeSet<ClusterAggregateRow>,
}

/// Result of applying a module-origin update.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClusterDbUpdateReport {
    /// Newly inserted origins.
    pub inserted_origins: Vec<String>,
    /// Existing origins whose payload changed.
    pub replaced_origins: Vec<String>,
    /// Stale origins removed before reuse.
    pub removed_origins: Vec<String>,
    /// Existing origins whose payload was unchanged.
    pub unchanged_origins: Vec<String>,
    /// Origins whose aggregate rows were touched.
    pub rebuilt_origins: Vec<String>,
}

/// Fail-closed write rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClusterDbWriteRejection {
    /// The record used an unsupported cluster-db schema.
    UnsupportedSchema {
        /// Origin key when available.
        origin_key: String,
        /// Actual schema.
        actual: SchemaVersion,
    },
    /// A required identity or metadata field is missing.
    MissingRequiredIdentity {
        /// Origin key when available.
        origin_key: String,
        /// Field name.
        field: &'static str,
    },
    /// A contribution was not accepted by the owner.
    UnacceptedContribution {
        /// Origin key.
        origin_key: String,
        /// Exported status.
        status: ClusterContributionStatus,
    },
    /// A contribution is accepted but not importer-visible.
    NotImporterVisible {
        /// Origin key.
        origin_key: String,
        /// Exported visibility.
        visibility: ClusterContributionVisibility,
    },
    /// Origin metadata or dependency footprint is incomplete.
    IncompleteOrigin {
        /// Origin key.
        origin_key: String,
        /// Missing or incomplete field.
        field: &'static str,
    },
    /// Origin is explicitly uncacheable.
    Uncacheable {
        /// Origin key.
        origin_key: String,
    },
    /// Duplicate origin keys had conflicting payloads.
    ConflictingDuplicateOrigin {
        /// Origin key.
        origin_key: String,
    },
    /// An incoming origin key collides with a different module's existing origin.
    OriginKeyCollision {
        /// Origin key.
        origin_key: String,
        /// Existing owner package.
        existing_package_id: String,
        /// Existing owner module.
        existing_module_path: String,
    },
    /// Required schema, policy, toolchain, or producer compatibility is unknown.
    UnknownCompatibility {
        /// Origin key.
        origin_key: String,
        /// Compatibility family.
        family: &'static str,
    },
}

impl ClusterDbIndex {
    /// Creates an empty cluster-db index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an origin by key.
    pub fn origin(&self, origin_key: &str) -> Option<&ClusterContributionRecord> {
        self.origins.get(origin_key)
    }

    /// Applies a complete replacement for one module's origin records.
    ///
    /// Stale origins for the module are removed before new rows are inserted.
    pub fn apply_module_update(
        &mut self,
        package_id: &str,
        module_path: &str,
        records: Vec<ClusterContributionRecord>,
    ) -> Result<ClusterDbUpdateReport, ClusterDbWriteRejection> {
        reject_empty("", "package_id", package_id)?;
        reject_empty("", "module_path", module_path)?;

        let incoming = canonical_records(records)?;
        for record in incoming.values() {
            validate_record(record)?;
            if record.origin.package_id != package_id {
                return Err(ClusterDbWriteRejection::MissingRequiredIdentity {
                    origin_key: record.origin.origin_key.clone(),
                    field: "origin.package_id does not match update package",
                });
            }
            if record.origin.module_path != module_path {
                return Err(ClusterDbWriteRejection::MissingRequiredIdentity {
                    origin_key: record.origin.origin_key.clone(),
                    field: "origin.module_path does not match update module",
                });
            }
            if let Some(existing) = self.origins.get(&record.origin.origin_key)
                && (existing.origin.package_id != package_id
                    || existing.origin.module_path != module_path)
            {
                return Err(ClusterDbWriteRejection::OriginKeyCollision {
                    origin_key: record.origin.origin_key.clone(),
                    existing_package_id: existing.origin.package_id.clone(),
                    existing_module_path: existing.origin.module_path.clone(),
                });
            }
        }

        let mut report = ClusterDbUpdateReport::default();
        let incoming_keys: BTreeSet<&str> = incoming.keys().map(String::as_str).collect();
        let stale_keys: Vec<String> = self
            .origins
            .iter()
            .filter(|(key, record)| {
                record.origin.package_id == package_id
                    && record.origin.module_path == module_path
                    && !incoming_keys.contains(key.as_str())
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in stale_keys {
            if let Some(old) = self.origins.remove(&key) {
                self.remove_rows_for(&old);
                report.removed_origins.push(key);
            }
        }

        for (key, record) in incoming {
            match self.origins.get(&key) {
                None => {
                    self.insert_rows_for(&record);
                    self.origins.insert(key.clone(), record);
                    report.inserted_origins.push(key);
                }
                Some(existing) if existing == &record => {
                    report.unchanged_origins.push(key);
                }
                Some(existing) => {
                    let old = existing.clone();
                    self.remove_rows_for(&old);
                    self.insert_rows_for(&record);
                    self.origins.insert(key.clone(), record);
                    report.replaced_origins.push(key);
                }
            }
        }

        report.finish();
        Ok(report)
    }

    /// Removes one origin and its aggregate rows.
    pub fn remove_origin(&mut self, origin_key: &str) -> Option<ClusterContributionRecord> {
        let removed = self.origins.remove(origin_key)?;
        self.remove_rows_for(&removed);
        Some(removed)
    }

    /// Returns a canonical snapshot.
    pub fn snapshot(&self) -> ClusterIndexSnapshot {
        ClusterIndexSnapshot {
            origins: self.origins.values().cloned().collect(),
            indexes: ClusterAggregateIndexes {
                graph_rows: sorted_rows(&self.graph_rows, compare_graph_rows),
                subsumption_dag_rows: sorted_rows(
                    &self.subsumption_dag_rows,
                    compare_subsumption_dag_rows,
                ),
                attr_index_rows: sorted_rows(&self.attr_index_rows, compare_attr_index_rows),
                type_index_rows: sorted_rows(&self.type_index_rows, compare_type_index_rows),
                reduction_index_rows: sorted_rows(
                    &self.reduction_index_rows,
                    compare_reduction_index_rows,
                ),
            },
        }
    }

    /// Materializes an import-scoped view by filtering aggregate indexes to visible origins.
    pub fn import_scoped_view(
        &self,
        request: ImportScopedViewRequest,
    ) -> Result<ImportScopedView, ClusterDbViewMiss> {
        let request = canonical_view_request(request)?;
        for origin_key in &request.visible_origin_keys {
            let record = self.origins.get(origin_key).ok_or_else(|| {
                ClusterDbViewMiss::MissingVisibleOrigin {
                    origin_key: origin_key.clone(),
                }
            })?;
            validate_view_origin(record, &request)?;
        }

        let visible_origins: BTreeSet<&str> = request
            .visible_origin_keys
            .iter()
            .map(String::as_str)
            .collect();
        let key = ImportScopedViewKey {
            importing_package_id: request.importing_package_id,
            importing_module_path: request.importing_module_path,
            import_closure_identity: request.import_closure_identity,
            visible_origin_set_hash: visible_origin_set_hash(&request.visible_origin_keys),
            verifier_policy_fingerprint: request.verifier_policy_fingerprint,
            cluster_db_schema_version: request.cluster_db_schema_version,
            producer_schema_versions: request.producer_schema_versions,
            policy_compatibility: request.policy_compatibility,
            schema_compatibility: request.schema_compatibility,
            toolchain_compatibility: request.toolchain_compatibility,
            traversal_profile: request.traversal_profile,
        };

        Ok(ImportScopedView {
            visible_origin_keys: request.visible_origin_keys.clone(),
            indexes: ClusterAggregateIndexes {
                graph_rows: filtered_sorted_rows(
                    &self.graph_rows,
                    &visible_origins,
                    compare_graph_rows,
                ),
                subsumption_dag_rows: filtered_sorted_rows(
                    &self.subsumption_dag_rows,
                    &visible_origins,
                    compare_subsumption_dag_rows,
                ),
                attr_index_rows: filtered_sorted_rows(
                    &self.attr_index_rows,
                    &visible_origins,
                    compare_attr_index_rows,
                ),
                type_index_rows: filtered_sorted_rows(
                    &self.type_index_rows,
                    &visible_origins,
                    compare_type_index_rows,
                ),
                reduction_index_rows: filtered_sorted_rows(
                    &self.reduction_index_rows,
                    &visible_origins,
                    compare_reduction_index_rows,
                ),
            },
            key,
        })
    }

    fn insert_rows_for(&mut self, record: &ClusterContributionRecord) {
        for entry in &record.index_entries {
            let row = aggregate_row(record, entry);
            match entry.entry_kind {
                ClusterIndexEntryKind::Graph => {
                    self.graph_rows.insert(row);
                }
                ClusterIndexEntryKind::SubsumptionDag => {
                    self.subsumption_dag_rows.insert(row);
                }
                ClusterIndexEntryKind::Attribute => {
                    self.attr_index_rows.insert(row);
                }
                ClusterIndexEntryKind::Type => {
                    self.type_index_rows.insert(row);
                }
                ClusterIndexEntryKind::Reduction => {
                    self.reduction_index_rows.insert(row);
                }
            }
        }
    }

    fn remove_rows_for(&mut self, record: &ClusterContributionRecord) {
        for entry in &record.index_entries {
            let row = aggregate_row(record, entry);
            match entry.entry_kind {
                ClusterIndexEntryKind::Graph => {
                    self.graph_rows.remove(&row);
                }
                ClusterIndexEntryKind::SubsumptionDag => {
                    self.subsumption_dag_rows.remove(&row);
                }
                ClusterIndexEntryKind::Attribute => {
                    self.attr_index_rows.remove(&row);
                }
                ClusterIndexEntryKind::Type => {
                    self.type_index_rows.remove(&row);
                }
                ClusterIndexEntryKind::Reduction => {
                    self.reduction_index_rows.remove(&row);
                }
            }
        }
    }
}

impl ClusterDbUpdateReport {
    fn finish(&mut self) {
        self.inserted_origins.sort();
        self.replaced_origins.sort();
        self.removed_origins.sort();
        self.unchanged_origins.sort();

        let mut rebuilt = BTreeSet::new();
        rebuilt.extend(self.inserted_origins.iter().cloned());
        rebuilt.extend(self.replaced_origins.iter().cloned());
        rebuilt.extend(self.removed_origins.iter().cloned());
        self.rebuilt_origins = rebuilt.into_iter().collect();
    }
}

impl fmt::Display for ClusterDbWriteRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema { origin_key, actual } => write!(
                formatter,
                "unsupported cluster-db schema `{}` for origin `{origin_key}`",
                actual.as_str()
            ),
            Self::MissingRequiredIdentity { origin_key, field } => {
                write!(formatter, "missing `{field}` for origin `{origin_key}`")
            }
            Self::UnacceptedContribution { origin_key, status } => write!(
                formatter,
                "origin `{origin_key}` is not accepted: {status:?}"
            ),
            Self::NotImporterVisible {
                origin_key,
                visibility,
            } => write!(
                formatter,
                "origin `{origin_key}` is not importer-visible: {visibility:?}"
            ),
            Self::IncompleteOrigin { origin_key, field } => {
                write!(formatter, "incomplete `{field}` for origin `{origin_key}`")
            }
            Self::Uncacheable { origin_key } => {
                write!(formatter, "origin `{origin_key}` is uncacheable")
            }
            Self::ConflictingDuplicateOrigin { origin_key } => {
                write!(formatter, "conflicting duplicate origin `{origin_key}`")
            }
            Self::OriginKeyCollision {
                origin_key,
                existing_package_id,
                existing_module_path,
            } => write!(
                formatter,
                "incoming origin `{origin_key}` collides with existing origin in \
                 `{existing_package_id}:{existing_module_path}`"
            ),
            Self::UnknownCompatibility { origin_key, family } => write!(
                formatter,
                "unknown {family} compatibility for origin `{origin_key}`"
            ),
        }
    }
}

impl Error for ClusterDbWriteRejection {}

impl fmt::Display for ClusterDbViewMiss {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequiredIdentity { field } => {
                write!(formatter, "missing import-scoped view `{field}`")
            }
            Self::UnsupportedSchema { actual } => write!(
                formatter,
                "unsupported import-scoped view schema `{}`",
                actual.as_str()
            ),
            Self::UnknownCompatibility { family } => {
                write!(
                    formatter,
                    "unknown import-scoped view {family} compatibility"
                )
            }
            Self::MissingVisibleOrigin { origin_key } => {
                write!(formatter, "missing visible origin `{origin_key}`")
            }
        }
    }
}

impl Error for ClusterDbViewMiss {}

fn canonical_records(
    records: Vec<ClusterContributionRecord>,
) -> Result<BTreeMap<String, ClusterContributionRecord>, ClusterDbWriteRejection> {
    let mut canonical = BTreeMap::new();
    for record in records {
        let record = canonical_record(record)?;
        match canonical.get(&record.origin.origin_key) {
            None => {
                canonical.insert(record.origin.origin_key.clone(), record);
            }
            Some(existing) if existing == &record => {}
            Some(_) => {
                return Err(ClusterDbWriteRejection::ConflictingDuplicateOrigin {
                    origin_key: record.origin.origin_key,
                });
            }
        }
    }
    Ok(canonical)
}

fn canonical_record(
    mut record: ClusterContributionRecord,
) -> Result<ClusterContributionRecord, ClusterDbWriteRejection> {
    canonicalize_schema_versions(
        &record.origin.origin_key,
        "producer_schema_versions",
        &mut record.origin.producer_schema_versions,
    )?;
    canonicalize_compatibility_fields(
        &record.origin.origin_key,
        "policy_compatibility",
        &mut record.origin.policy_compatibility,
    )?;
    canonicalize_compatibility_fields(
        &record.origin.origin_key,
        "schema_compatibility",
        &mut record.origin.schema_compatibility,
    )?;
    canonicalize_compatibility_fields(
        &record.origin.origin_key,
        "toolchain_compatibility",
        &mut record.origin.toolchain_compatibility,
    )?;
    if let Some(values) = &mut record.origin.trace_replay_hashes {
        canonicalize_named_hashes(&record.origin.origin_key, "trace_replay_hashes", values)?;
    }
    if let Some(values) = &mut record.origin.dependency_interface_hashes {
        canonicalize_named_hashes(
            &record.origin.origin_key,
            "dependency_interface_hashes",
            values,
        )?;
    }
    canonicalize_index_entries(&record.origin.origin_key, &mut record.index_entries)?;
    canonicalize_named_hashes(
        &record.origin.origin_key,
        "diagnostic_refs",
        &mut record.diagnostic_refs,
    )?;
    Ok(record)
}

fn validate_record(record: &ClusterContributionRecord) -> Result<(), ClusterDbWriteRejection> {
    let origin = &record.origin;
    if record.schema_version.as_str() != CLUSTER_DB_SCHEMA_VERSION {
        return Err(ClusterDbWriteRejection::UnsupportedSchema {
            origin_key: origin.origin_key.clone(),
            actual: record.schema_version.clone(),
        });
    }

    reject_empty(&origin.origin_key, "origin_key", &origin.origin_key)?;
    reject_empty(&origin.origin_key, "package_id", &origin.package_id)?;
    reject_empty(&origin.origin_key, "module_path", &origin.module_path)?;
    reject_empty(
        &origin.origin_key,
        "stable_contribution_id",
        &origin.stable_contribution_id,
    )?;
    reject_empty(&origin.origin_key, "label", &origin.label)?;

    if origin.accepted_status != ClusterContributionStatus::Accepted {
        return Err(ClusterDbWriteRejection::UnacceptedContribution {
            origin_key: origin.origin_key.clone(),
            status: origin.accepted_status,
        });
    }
    if origin.accepted_visibility != ClusterContributionVisibility::ImporterVisible {
        return Err(ClusterDbWriteRejection::NotImporterVisible {
            origin_key: origin.origin_key.clone(),
            visibility: origin.accepted_visibility,
        });
    }
    if origin.uncacheable {
        return Err(ClusterDbWriteRejection::Uncacheable {
            origin_key: origin.origin_key.clone(),
        });
    }
    if origin.footprint_completeness == ClusterOriginFootprintCompleteness::IncompleteUncacheable {
        return Err(ClusterDbWriteRejection::IncompleteOrigin {
            origin_key: origin.origin_key.clone(),
            field: "origin_footprint",
        });
    }
    if origin.origin_footprint_hash.is_none() {
        return Err(ClusterDbWriteRejection::IncompleteOrigin {
            origin_key: origin.origin_key.clone(),
            field: "origin_footprint_hash",
        });
    }
    if origin.proof_backed && origin.accepted_witness_or_discharge_hash.is_none() {
        return Err(ClusterDbWriteRejection::IncompleteOrigin {
            origin_key: origin.origin_key.clone(),
            field: "accepted_witness_or_discharge_hash",
        });
    }
    if let Some(value) = &origin.accepted_witness_or_discharge_hash {
        validate_named_hashes(
            &origin.origin_key,
            "accepted_witness_or_discharge_hash",
            std::slice::from_ref(value),
        )?;
    }
    match &origin.trace_replay_hashes {
        Some(values) => validate_named_hashes(&origin.origin_key, "trace_replay_hashes", values)?,
        None => {
            return Err(ClusterDbWriteRejection::IncompleteOrigin {
                origin_key: origin.origin_key.clone(),
                field: "trace_replay_hashes",
            });
        }
    }
    match &origin.dependency_interface_hashes {
        Some(values) => {
            validate_named_hashes(&origin.origin_key, "dependency_interface_hashes", values)?;
        }
        None => {
            return Err(ClusterDbWriteRejection::IncompleteOrigin {
                origin_key: origin.origin_key.clone(),
                field: "dependency_interface_hashes",
            });
        }
    }
    validate_schema_versions(&origin.origin_key, &origin.producer_schema_versions)?;
    validate_compatibility_fields(
        &origin.origin_key,
        "policy_compatibility",
        &origin.policy_compatibility,
    )?;
    validate_compatibility_fields(
        &origin.origin_key,
        "schema_compatibility",
        &origin.schema_compatibility,
    )?;
    validate_compatibility_fields(
        &origin.origin_key,
        "toolchain_compatibility",
        &origin.toolchain_compatibility,
    )?;
    for entry in &record.index_entries {
        reject_empty(
            &origin.origin_key,
            "index_entries.primary_key",
            &entry.primary_key,
        )?;
        reject_empty(
            &origin.origin_key,
            "index_entries.secondary_key",
            &entry.secondary_key,
        )?;
    }
    Ok(())
}

fn aggregate_row(
    record: &ClusterContributionRecord,
    entry: &ClusterIndexEntry,
) -> ClusterAggregateRow {
    ClusterAggregateRow {
        origin_key: record.origin.origin_key.clone(),
        package_id: record.origin.package_id.clone(),
        module_path: record.origin.module_path.clone(),
        declaration_order: record.declaration_order,
        primary_key: entry.primary_key.clone(),
        secondary_key: entry.secondary_key.clone(),
        strategy_key: entry.strategy_key.clone(),
        contribution_fingerprint: entry.contribution_fingerprint,
    }
}

fn sorted_rows(
    rows: &BTreeSet<ClusterAggregateRow>,
    compare: fn(&ClusterAggregateRow, &ClusterAggregateRow) -> Ordering,
) -> Vec<ClusterAggregateRow> {
    let mut rows: Vec<_> = rows.iter().cloned().collect();
    rows.sort_by(compare);
    rows
}

fn filtered_sorted_rows(
    rows: &BTreeSet<ClusterAggregateRow>,
    visible_origins: &BTreeSet<&str>,
    compare: fn(&ClusterAggregateRow, &ClusterAggregateRow) -> Ordering,
) -> Vec<ClusterAggregateRow> {
    let mut rows: Vec<_> = rows
        .iter()
        .filter(|row| visible_origins.contains(row.origin_key.as_str()))
        .cloned()
        .collect();
    rows.sort_by(compare);
    rows
}

fn canonical_view_request(
    mut request: ImportScopedViewRequest,
) -> Result<ImportScopedViewRequest, ClusterDbViewMiss> {
    reject_empty_view("importing_package_id", &request.importing_package_id)?;
    reject_empty_view("importing_module_path", &request.importing_module_path)?;
    reject_empty_view("import_closure_identity", &request.import_closure_identity)?;
    if request.cluster_db_schema_version.as_str() != CLUSTER_DB_SCHEMA_VERSION {
        return Err(ClusterDbViewMiss::UnsupportedSchema {
            actual: request.cluster_db_schema_version,
        });
    }

    request.visible_origin_keys.sort();
    canonicalize_view_by_key(
        "visible_origin_keys",
        &mut request.visible_origin_keys,
        String::clone,
        ClusterDbViewMiss::MissingRequiredIdentity {
            field: "visible_origin_keys",
        },
    )?;
    for origin_key in &request.visible_origin_keys {
        reject_empty_view("visible_origin_keys", origin_key)?;
    }

    canonicalize_view_schema_versions(&mut request.producer_schema_versions)?;
    validate_view_schema_versions(&request.producer_schema_versions)?;
    canonicalize_view_compatibility_fields(
        "policy_compatibility",
        &mut request.policy_compatibility,
    )?;
    validate_view_compatibility_fields("policy_compatibility", &request.policy_compatibility)?;
    canonicalize_view_compatibility_fields(
        "schema_compatibility",
        &mut request.schema_compatibility,
    )?;
    validate_view_compatibility_fields("schema_compatibility", &request.schema_compatibility)?;
    canonicalize_view_compatibility_fields(
        "toolchain_compatibility",
        &mut request.toolchain_compatibility,
    )?;
    validate_view_compatibility_fields(
        "toolchain_compatibility",
        &request.toolchain_compatibility,
    )?;
    canonicalize_view_compatibility_fields("traversal_profile", &mut request.traversal_profile)?;
    validate_view_compatibility_fields("traversal_profile", &request.traversal_profile)?;

    Ok(request)
}

fn validate_view_origin(
    record: &ClusterContributionRecord,
    request: &ImportScopedViewRequest,
) -> Result<(), ClusterDbViewMiss> {
    if record.origin.verifier_policy_fingerprint != request.verifier_policy_fingerprint {
        return Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "verifier_policy_fingerprint",
        });
    }
    for required in &record.origin.producer_schema_versions {
        if !request.producer_schema_versions.contains(required) {
            return Err(ClusterDbViewMiss::UnsupportedSchema {
                actual: required.version.clone(),
            });
        }
    }
    for required in &record.origin.policy_compatibility {
        if !request.policy_compatibility.contains(required) {
            return Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "policy_compatibility",
            });
        }
    }
    for required in &record.origin.schema_compatibility {
        if !request.schema_compatibility.contains(required) {
            return Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "schema_compatibility",
            });
        }
    }
    for required in &record.origin.toolchain_compatibility {
        if !request.toolchain_compatibility.contains(required) {
            return Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "toolchain_compatibility",
            });
        }
    }
    Ok(())
}

fn canonicalize_view_schema_versions(
    values: &mut Vec<NamedSchemaVersion>,
) -> Result<(), ClusterDbViewMiss> {
    values.sort_by_key(schema_version_key);
    canonicalize_view_by_key(
        "producer_schema_versions",
        values,
        schema_version_key,
        ClusterDbViewMiss::UnknownCompatibility {
            family: "producer_schema_versions",
        },
    )
}

fn validate_view_schema_versions(values: &[NamedSchemaVersion]) -> Result<(), ClusterDbViewMiss> {
    if values.is_empty() {
        return Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "producer_schema_versions",
        });
    }
    for value in values {
        reject_empty_view(
            "producer_schema_versions.schema_family",
            &value.schema_family,
        )?;
        reject_empty_view("producer_schema_versions.name", &value.name)?;
        reject_empty_view("producer_schema_versions.version", value.version.as_str())?;
        if value.version.as_str().eq_ignore_ascii_case("unknown")
            || value.version.as_str().eq_ignore_ascii_case("unsupported")
        {
            return Err(ClusterDbViewMiss::UnsupportedSchema {
                actual: value.version.clone(),
            });
        }
    }
    Ok(())
}

fn canonicalize_view_compatibility_fields(
    field: &'static str,
    values: &mut Vec<CompatibilityField>,
) -> Result<(), ClusterDbViewMiss> {
    values.sort_by_key(compatibility_field_key);
    canonicalize_view_by_key(
        field,
        values,
        compatibility_field_key,
        ClusterDbViewMiss::UnknownCompatibility { family: field },
    )
}

fn validate_view_compatibility_fields(
    field: &'static str,
    values: &[CompatibilityField],
) -> Result<(), ClusterDbViewMiss> {
    if values.is_empty() {
        return Err(ClusterDbViewMiss::UnknownCompatibility { family: field });
    }
    for value in values {
        reject_empty_view(field, &value.family)?;
        reject_empty_view(field, &value.field_name)?;
        reject_empty_view(field, &value.value)?;
        if value.value.eq_ignore_ascii_case("unknown")
            || value.value.eq_ignore_ascii_case("unsupported")
        {
            return Err(ClusterDbViewMiss::UnknownCompatibility { family: field });
        }
    }
    Ok(())
}

fn canonicalize_view_by_key<T, K, F>(
    _field: &'static str,
    values: &mut Vec<T>,
    key_for: F,
    conflict_error: ClusterDbViewMiss,
) -> Result<(), ClusterDbViewMiss>
where
    T: Clone + PartialEq,
    K: Ord,
    F: Fn(&T) -> K,
{
    let mut output: Vec<T> = Vec::with_capacity(values.len());
    for value in values.iter() {
        if let Some(previous) = output.last()
            && key_for(previous) == key_for(value)
        {
            if previous == value {
                continue;
            }
            return Err(conflict_error);
        }
        output.push(value.clone());
    }
    *values = output;
    Ok(())
}

fn reject_empty_view(field: &'static str, value: &str) -> Result<(), ClusterDbViewMiss> {
    if value.trim().is_empty() {
        return Err(ClusterDbViewMiss::MissingRequiredIdentity { field });
    }
    Ok(())
}

fn visible_origin_set_hash(origin_keys: &[String]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    write_view_hash_str(&mut hasher, "mizar-cache/cluster-db/visible-origin-set/v1");
    write_view_hash_len(&mut hasher, origin_keys.len());
    for origin_key in origin_keys {
        write_view_hash_str(&mut hasher, origin_key);
    }
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

fn write_view_hash_str(hasher: &mut blake3::Hasher, value: &str) {
    write_view_hash_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

fn write_view_hash_len(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

fn compare_graph_rows(left: &ClusterAggregateRow, right: &ClusterAggregateRow) -> Ordering {
    (
        &left.primary_key,
        &left.package_id,
        &left.module_path,
        left.declaration_order,
        &left.secondary_key,
        &left.origin_key,
        left.contribution_fingerprint.as_bytes(),
        &left.strategy_key,
    )
        .cmp(&(
            &right.primary_key,
            &right.package_id,
            &right.module_path,
            right.declaration_order,
            &right.secondary_key,
            &right.origin_key,
            right.contribution_fingerprint.as_bytes(),
            &right.strategy_key,
        ))
}

fn compare_subsumption_dag_rows(
    left: &ClusterAggregateRow,
    right: &ClusterAggregateRow,
) -> Ordering {
    (
        &left.primary_key,
        &left.secondary_key,
        &left.strategy_key,
        &left.package_id,
        &left.module_path,
        &left.origin_key,
        left.contribution_fingerprint.as_bytes(),
    )
        .cmp(&(
            &right.primary_key,
            &right.secondary_key,
            &right.strategy_key,
            &right.package_id,
            &right.module_path,
            &right.origin_key,
            right.contribution_fingerprint.as_bytes(),
        ))
}

fn compare_attr_index_rows(left: &ClusterAggregateRow, right: &ClusterAggregateRow) -> Ordering {
    (
        &left.primary_key,
        &left.secondary_key,
        &left.package_id,
        &left.origin_key,
        left.contribution_fingerprint.as_bytes(),
        &left.module_path,
        left.declaration_order,
        &left.strategy_key,
    )
        .cmp(&(
            &right.primary_key,
            &right.secondary_key,
            &right.package_id,
            &right.origin_key,
            right.contribution_fingerprint.as_bytes(),
            &right.module_path,
            right.declaration_order,
            &right.strategy_key,
        ))
}

fn compare_type_index_rows(left: &ClusterAggregateRow, right: &ClusterAggregateRow) -> Ordering {
    (
        &left.primary_key,
        &left.package_id,
        &left.origin_key,
        left.contribution_fingerprint.as_bytes(),
        &left.secondary_key,
        &left.module_path,
        left.declaration_order,
        &left.strategy_key,
    )
        .cmp(&(
            &right.primary_key,
            &right.package_id,
            &right.origin_key,
            right.contribution_fingerprint.as_bytes(),
            &right.secondary_key,
            &right.module_path,
            right.declaration_order,
            &right.strategy_key,
        ))
}

fn compare_reduction_index_rows(
    left: &ClusterAggregateRow,
    right: &ClusterAggregateRow,
) -> Ordering {
    (
        &left.primary_key,
        &left.secondary_key,
        &left.strategy_key,
        &left.package_id,
        &left.origin_key,
        left.contribution_fingerprint.as_bytes(),
        &left.module_path,
        left.declaration_order,
    )
        .cmp(&(
            &right.primary_key,
            &right.secondary_key,
            &right.strategy_key,
            &right.package_id,
            &right.origin_key,
            right.contribution_fingerprint.as_bytes(),
            &right.module_path,
            right.declaration_order,
        ))
}

fn validate_named_hashes(
    origin_key: &str,
    field: &'static str,
    values: &[NamedHash],
) -> Result<(), ClusterDbWriteRejection> {
    for value in values {
        reject_empty(origin_key, field, &value.name)?;
        reject_empty(origin_key, field, &value.domain)?;
    }
    Ok(())
}

fn validate_schema_versions(
    origin_key: &str,
    values: &[NamedSchemaVersion],
) -> Result<(), ClusterDbWriteRejection> {
    if values.is_empty() {
        return Err(ClusterDbWriteRejection::IncompleteOrigin {
            origin_key: origin_key.to_owned(),
            field: "producer_schema_versions",
        });
    }
    for value in values {
        reject_empty(
            origin_key,
            "producer_schema_versions.schema_family",
            &value.schema_family,
        )?;
        reject_empty(origin_key, "producer_schema_versions.name", &value.name)?;
        reject_empty(
            origin_key,
            "producer_schema_versions.version",
            value.version.as_str(),
        )?;
        if value.version.as_str().eq_ignore_ascii_case("unknown")
            || value.version.as_str().eq_ignore_ascii_case("unsupported")
        {
            return Err(ClusterDbWriteRejection::UnsupportedSchema {
                origin_key: origin_key.to_owned(),
                actual: value.version.clone(),
            });
        }
    }
    Ok(())
}

fn validate_compatibility_fields(
    origin_key: &str,
    field: &'static str,
    values: &[CompatibilityField],
) -> Result<(), ClusterDbWriteRejection> {
    if values.is_empty() {
        return Err(ClusterDbWriteRejection::UnknownCompatibility {
            origin_key: origin_key.to_owned(),
            family: field,
        });
    }
    for value in values {
        reject_empty(origin_key, field, &value.family)?;
        reject_empty(origin_key, field, &value.field_name)?;
        reject_empty(origin_key, field, &value.value)?;
        if value.value.eq_ignore_ascii_case("unknown")
            || value.value.eq_ignore_ascii_case("unsupported")
        {
            return Err(ClusterDbWriteRejection::UnknownCompatibility {
                origin_key: origin_key.to_owned(),
                family: field,
            });
        }
    }
    Ok(())
}

fn canonicalize_named_hashes(
    origin_key: &str,
    field: &'static str,
    values: &mut Vec<NamedHash>,
) -> Result<(), ClusterDbWriteRejection> {
    values.sort_by_key(named_hash_key);
    canonicalize_by_key(origin_key, field, values, named_hash_key)
}

fn canonicalize_schema_versions(
    origin_key: &str,
    field: &'static str,
    values: &mut Vec<NamedSchemaVersion>,
) -> Result<(), ClusterDbWriteRejection> {
    values.sort_by_key(schema_version_key);
    canonicalize_by_key(origin_key, field, values, schema_version_key)
}

fn canonicalize_compatibility_fields(
    origin_key: &str,
    field: &'static str,
    values: &mut Vec<CompatibilityField>,
) -> Result<(), ClusterDbWriteRejection> {
    values.sort_by_key(compatibility_field_key);
    canonicalize_by_key(origin_key, field, values, compatibility_field_key)
}

fn canonicalize_index_entries(
    origin_key: &str,
    values: &mut Vec<ClusterIndexEntry>,
) -> Result<(), ClusterDbWriteRejection> {
    values.sort_by_key(index_entry_key);
    canonicalize_by_key(origin_key, "index_entries", values, index_entry_key)
}

fn canonicalize_by_key<T, K, F>(
    origin_key: &str,
    field: &'static str,
    values: &mut Vec<T>,
    key_for: F,
) -> Result<(), ClusterDbWriteRejection>
where
    T: Clone + PartialEq,
    K: Ord + fmt::Debug,
    F: Fn(&T) -> K,
{
    let mut output: Vec<T> = Vec::with_capacity(values.len());
    for value in values.iter() {
        if let Some(previous) = output.last()
            && key_for(previous) == key_for(value)
        {
            if previous == value {
                continue;
            }
            return Err(ClusterDbWriteRejection::MissingRequiredIdentity {
                origin_key: origin_key.to_owned(),
                field,
            });
        }
        output.push(value.clone());
    }
    *values = output;
    Ok(())
}

fn named_hash_key(value: &NamedHash) -> (String, String) {
    (value.name.clone(), value.domain.clone())
}

fn schema_version_key(value: &NamedSchemaVersion) -> (String, String) {
    (value.schema_family.clone(), value.name.clone())
}

fn compatibility_field_key(value: &CompatibilityField) -> (String, String) {
    (value.family.clone(), value.field_name.clone())
}

fn index_entry_key(value: &ClusterIndexEntry) -> (ClusterIndexEntryKind, String, String, String) {
    (
        value.entry_kind,
        value.primary_key.clone(),
        value.secondary_key.clone(),
        value.strategy_key.clone(),
    )
}

fn reject_empty(
    origin_key: &str,
    field: &'static str,
    value: &str,
) -> Result<(), ClusterDbWriteRejection> {
    if value.trim().is_empty() {
        return Err(ClusterDbWriteRejection::MissingRequiredIdentity {
            origin_key: origin_key.to_owned(),
            field,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
