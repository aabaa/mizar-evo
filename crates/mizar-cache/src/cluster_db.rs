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
mod tests {
    use super::*;

    #[test]
    fn accepted_contribution_writes_origin_and_indexes() {
        let mut db = ClusterDbIndex::new();
        let report = db
            .apply_module_update(
                "pkg",
                "alpha",
                vec![record(
                    "origin-a",
                    0,
                    vec![entry(ClusterIndexEntryKind::Graph, "T", "A")],
                )],
            )
            .expect("accepted contribution is inserted");

        assert_eq!(report.inserted_origins, ["origin-a"]);
        assert_eq!(report.rebuilt_origins, ["origin-a"]);

        let snapshot = db.snapshot();
        assert_eq!(snapshot.origins.len(), 1);
        assert_eq!(snapshot.indexes.graph_rows.len(), 1);
        assert_eq!(snapshot.indexes.graph_rows[0].origin_key, "origin-a");
        assert!(snapshot.indexes.attr_index_rows.is_empty());
    }

    #[test]
    fn accepted_contribution_populates_every_aggregate_index_kind() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "origin-all",
                0,
                vec![
                    entry(ClusterIndexEntryKind::Graph, "type", "graph"),
                    entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "edge"),
                    entry(ClusterIndexEntryKind::Attribute, "attr", "producer"),
                    entry(ClusterIndexEntryKind::Type, "mode", "trigger"),
                    entry(ClusterIndexEntryKind::Reduction, "lhs", "rule"),
                ],
            )],
        )
        .expect("accepted contribution is inserted into all indexes");

        let snapshot = db.snapshot();
        assert_eq!(snapshot.indexes.graph_rows.len(), 1);
        assert_eq!(snapshot.indexes.subsumption_dag_rows.len(), 1);
        assert_eq!(snapshot.indexes.attr_index_rows.len(), 1);
        assert_eq!(snapshot.indexes.type_index_rows.len(), 1);
        assert_eq!(snapshot.indexes.reduction_index_rows.len(), 1);
    }

    #[test]
    fn non_visible_or_unaccepted_contributions_are_rejected() {
        for (label, mutate) in [
            (
                "private",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.accepted_visibility = ClusterContributionVisibility::Private;
                }) as Box<dyn Fn(&mut ClusterContributionRecord)>,
            ),
            (
                "local-only",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.accepted_visibility = ClusterContributionVisibility::LocalOnly;
                }),
            ),
            (
                "pending",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.accepted_status = ClusterContributionStatus::Pending;
                }),
            ),
            (
                "rejected",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.accepted_status = ClusterContributionStatus::Rejected;
                }),
            ),
            (
                "recovered",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.accepted_status = ClusterContributionStatus::Recovered;
                }),
            ),
            (
                "externally-attested",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.accepted_status = ClusterContributionStatus::ExternallyAttested;
                }),
            ),
            (
                "uncacheable",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.uncacheable = true;
                }),
            ),
        ] {
            let mut db = ClusterDbIndex::new();
            let mut candidate = record(
                &format!("origin-{label}"),
                0,
                vec![entry(ClusterIndexEntryKind::Attribute, "attr", label)],
            );
            mutate(&mut candidate);

            assert!(
                db.apply_module_update("pkg", "alpha", vec![candidate])
                    .is_err(),
                "{label} contribution must not enter visible cluster-db indexes"
            );
            assert!(db.snapshot().origins.is_empty());
        }
    }

    #[test]
    fn incomplete_origin_metadata_forces_rejection() {
        for (label, mutate) in [
            (
                "incomplete-footprint",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.footprint_completeness =
                        ClusterOriginFootprintCompleteness::IncompleteUncacheable;
                }) as Box<dyn Fn(&mut ClusterContributionRecord)>,
            ),
            (
                "missing-footprint-hash",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.origin_footprint_hash = None;
                }),
            ),
            (
                "missing-dependency-interface-hashes",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.dependency_interface_hashes = None;
                }),
            ),
            (
                "missing-trace-replay-hashes",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.trace_replay_hashes = None;
                }),
            ),
            (
                "missing-proof-identity",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.proof_backed = true;
                    record.origin.accepted_witness_or_discharge_hash = None;
                }),
            ),
            (
                "missing-producer-schema",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.producer_schema_versions.clear();
                }),
            ),
            (
                "blank-origin-key",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.origin_key = " ".to_owned();
                }),
            ),
            (
                "blank-package-id",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.package_id = " ".to_owned();
                }),
            ),
            (
                "blank-module-path",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.module_path = " ".to_owned();
                }),
            ),
            (
                "blank-stable-contribution-id",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.stable_contribution_id = " ".to_owned();
                }),
            ),
            (
                "blank-label",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.label = " ".to_owned();
                }),
            ),
            (
                "blank-trace-hash-name",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.trace_replay_hashes = Some(vec![NamedHash {
                        name: String::new(),
                        domain: "test-domain/trace".to_owned(),
                        digest: hash(21),
                    }]);
                }),
            ),
            (
                "blank-schema-family",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.producer_schema_versions = vec![NamedSchemaVersion {
                        schema_family: String::new(),
                        name: "registration-summary".to_owned(),
                        version: SchemaVersion::new("1.0"),
                    }];
                }),
            ),
            (
                "unsupported-producer-schema",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.producer_schema_versions = vec![NamedSchemaVersion {
                        schema_family: "mizar-artifact/registration-summary".to_owned(),
                        name: "registration-summary".to_owned(),
                        version: SchemaVersion::new("unknown"),
                    }];
                }),
            ),
            (
                "blank-proof-identity-name",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.accepted_witness_or_discharge_hash = Some(NamedHash {
                        name: String::new(),
                        domain: "test-domain/witness".to_owned(),
                        digest: hash(22),
                    });
                }),
            ),
        ] {
            let mut db = ClusterDbIndex::new();
            let mut candidate = record(
                &format!("origin-{label}"),
                0,
                vec![entry(ClusterIndexEntryKind::Type, "type", label)],
            );
            mutate(&mut candidate);

            assert!(
                db.apply_module_update("pkg", "alpha", vec![candidate])
                    .is_err(),
                "{label} must be rejected before visible indexing"
            );
            assert!(db.snapshot().origins.is_empty());
        }
    }

    #[test]
    fn unknown_schema_or_toolchain_compatibility_forces_rejection() {
        for (label, mutate) in [
            (
                "missing-policy-compatibility",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.policy_compatibility.clear();
                }) as Box<dyn Fn(&mut ClusterContributionRecord)>,
            ),
            (
                "unknown-policy-compatibility",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.policy_compatibility[0].value = "unknown".to_owned();
                }),
            ),
            (
                "missing-schema-compatibility",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.schema_compatibility.clear();
                }),
            ),
            (
                "unknown-schema-compatibility",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.schema_compatibility[0].value = "unknown".to_owned();
                }),
            ),
            (
                "missing-toolchain-compatibility",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.toolchain_compatibility.clear();
                }),
            ),
            (
                "unsupported-toolchain-compatibility",
                Box::new(|record: &mut ClusterContributionRecord| {
                    record.origin.toolchain_compatibility[0].value = "unsupported".to_owned();
                }),
            ),
        ] {
            let mut db = ClusterDbIndex::new();
            let mut candidate = record(
                &format!("origin-{label}"),
                0,
                vec![entry(ClusterIndexEntryKind::Type, "type", label)],
            );
            mutate(&mut candidate);

            assert!(matches!(
                db.apply_module_update("pkg", "alpha", vec![candidate]),
                Err(ClusterDbWriteRejection::UnknownCompatibility { .. })
            ));
            assert!(db.snapshot().origins.is_empty());
        }
    }

    #[test]
    fn rename_or_removal_cleans_stale_origins_before_reuse() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "old-origin",
                0,
                vec![entry(ClusterIndexEntryKind::Attribute, "attr", "old")],
            )],
        )
        .expect("initial insert");

        let report = db
            .apply_module_update(
                "pkg",
                "alpha",
                vec![record(
                    "new-origin",
                    1,
                    vec![entry(ClusterIndexEntryKind::Attribute, "attr", "new")],
                )],
            )
            .expect("rename replaces module origins");

        assert_eq!(report.removed_origins, ["old-origin"]);
        assert_eq!(report.inserted_origins, ["new-origin"]);
        assert_eq!(report.rebuilt_origins, ["new-origin", "old-origin"]);
        assert!(db.origin("old-origin").is_none());
        assert!(db.origin("new-origin").is_some());

        let rows = db.snapshot().indexes.attr_index_rows;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].origin_key, "new-origin");
        assert_eq!(rows[0].secondary_key, "new");
    }

    #[test]
    fn rebuild_report_touches_only_changed_origins() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![
                record(
                    "stable-origin",
                    0,
                    vec![entry(ClusterIndexEntryKind::Graph, "T", "stable")],
                ),
                record(
                    "changed-origin",
                    1,
                    vec![entry(ClusterIndexEntryKind::Reduction, "f", "old-rule")],
                ),
            ],
        )
        .expect("initial insert");

        let report = db
            .apply_module_update(
                "pkg",
                "alpha",
                vec![
                    record(
                        "stable-origin",
                        0,
                        vec![entry(ClusterIndexEntryKind::Graph, "T", "stable")],
                    ),
                    record(
                        "changed-origin",
                        1,
                        vec![entry(ClusterIndexEntryKind::Reduction, "f", "new-rule")],
                    ),
                ],
            )
            .expect("changed origin replaced");

        assert_eq!(report.unchanged_origins, ["stable-origin"]);
        assert_eq!(report.replaced_origins, ["changed-origin"]);
        assert_eq!(report.rebuilt_origins, ["changed-origin"]);

        let snapshot = db.snapshot();
        assert_eq!(snapshot.indexes.graph_rows[0].origin_key, "stable-origin");
        assert_eq!(snapshot.indexes.reduction_index_rows.len(), 1);
        assert_eq!(
            snapshot.indexes.reduction_index_rows[0].secondary_key,
            "new-rule"
        );
    }

    #[test]
    fn deterministic_ordering_is_independent_of_write_order() {
        let first = record(
            "b-origin",
            1,
            vec![
                entry(ClusterIndexEntryKind::Type, "T", "b"),
                entry(ClusterIndexEntryKind::SubsumptionDag, "root", "b"),
            ],
        );
        let second = record(
            "a-origin",
            0,
            vec![
                entry(ClusterIndexEntryKind::Attribute, "A", "a"),
                entry(ClusterIndexEntryKind::Graph, "T", "a"),
            ],
        );

        let mut left = ClusterDbIndex::new();
        left.apply_module_update("pkg", "alpha", vec![first.clone(), second.clone()])
            .expect("left update");

        let mut right = ClusterDbIndex::new();
        right
            .apply_module_update("pkg", "alpha", vec![second, first])
            .expect("right update");

        assert_eq!(left.snapshot(), right.snapshot());
        assert_eq!(left.snapshot().origins[0].origin.origin_key, "a-origin");
        assert_eq!(left.snapshot().origins[1].origin.origin_key, "b-origin");
    }

    #[test]
    fn deterministic_ordering_sorts_multiple_rows_in_each_index_bucket() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![
                record(
                    "z-origin",
                    2,
                    vec![
                        entry(ClusterIndexEntryKind::Graph, "T", "z"),
                        entry(ClusterIndexEntryKind::SubsumptionDag, "sym", "z"),
                        entry(ClusterIndexEntryKind::Attribute, "attr", "z"),
                        entry(ClusterIndexEntryKind::Type, "type", "z"),
                        entry(ClusterIndexEntryKind::Reduction, "lhs", "z"),
                    ],
                ),
                record(
                    "a-origin",
                    0,
                    vec![
                        entry(ClusterIndexEntryKind::Graph, "T", "a"),
                        entry(ClusterIndexEntryKind::SubsumptionDag, "sym", "a"),
                        entry(ClusterIndexEntryKind::Attribute, "attr", "a"),
                        entry(ClusterIndexEntryKind::Type, "type", "a"),
                        entry(ClusterIndexEntryKind::Reduction, "lhs", "a"),
                    ],
                ),
            ],
        )
        .expect("multi-row insertion succeeds");

        let snapshot = db.snapshot();
        assert_eq!(
            origin_keys(&snapshot.indexes.graph_rows),
            vec!["a-origin", "z-origin"]
        );
        assert_eq!(
            origin_keys(&snapshot.indexes.subsumption_dag_rows),
            vec!["a-origin", "z-origin"]
        );
        assert_eq!(
            origin_keys(&snapshot.indexes.attr_index_rows),
            vec!["a-origin", "z-origin"]
        );
        assert_eq!(
            origin_keys(&snapshot.indexes.type_index_rows),
            vec!["a-origin", "z-origin"]
        );
        assert_eq!(
            origin_keys(&snapshot.indexes.reduction_index_rows),
            vec!["a-origin", "z-origin"]
        );
    }

    #[test]
    fn duplicate_conflicting_origin_rejects_without_mutation() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "existing",
                0,
                vec![entry(ClusterIndexEntryKind::Graph, "T", "old")],
            )],
        )
        .expect("initial insert");
        let before = db.snapshot();

        let first = record(
            "dup",
            1,
            vec![entry(ClusterIndexEntryKind::Attribute, "attr", "one")],
        );
        let second = record(
            "dup",
            1,
            vec![entry(ClusterIndexEntryKind::Attribute, "attr", "two")],
        );

        let result = db.apply_module_update("pkg", "alpha", vec![first, second]);

        assert!(matches!(
            result,
            Err(ClusterDbWriteRejection::ConflictingDuplicateOrigin { .. })
        ));
        assert_eq!(db.snapshot(), before);
    }

    #[test]
    fn cross_module_origin_key_collision_rejects_without_mutation() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "shared-origin",
                0,
                vec![entry(ClusterIndexEntryKind::Graph, "T", "alpha")],
            )],
        )
        .expect("initial insert");
        let before = db.snapshot();

        let mut colliding = record(
            "shared-origin",
            0,
            vec![entry(ClusterIndexEntryKind::Graph, "T", "beta")],
        );
        colliding.origin.module_path = "beta".to_owned();

        let result = db.apply_module_update("pkg", "beta", vec![colliding]);

        assert!(matches!(
            result,
            Err(ClusterDbWriteRejection::OriginKeyCollision { .. })
        ));
        assert_eq!(db.snapshot(), before);
    }

    #[test]
    fn identical_duplicate_origin_is_coalesced() {
        let mut db = ClusterDbIndex::new();
        let candidate = record(
            "dup",
            0,
            vec![entry(ClusterIndexEntryKind::Attribute, "attr", "same")],
        );

        let report = db
            .apply_module_update("pkg", "alpha", vec![candidate.clone(), candidate])
            .expect("identical duplicate coalesces");

        assert_eq!(report.inserted_origins, ["dup"]);
        assert_eq!(db.snapshot().origins.len(), 1);
    }

    #[test]
    fn import_scoped_view_filters_to_visible_origins() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "origin-a",
                0,
                vec![
                    entry(ClusterIndexEntryKind::Graph, "type", "a-graph"),
                    entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "a-edge"),
                    entry(ClusterIndexEntryKind::Attribute, "attr", "a-attr"),
                    entry(ClusterIndexEntryKind::Type, "mode", "a-type"),
                    entry(ClusterIndexEntryKind::Reduction, "lhs", "a-rule"),
                ],
            )],
        )
        .expect("origin a inserted");
        db.apply_module_update(
            "pkg",
            "beta",
            vec![record_for(
                "origin-b",
                "pkg",
                "beta",
                0,
                vec![
                    entry(ClusterIndexEntryKind::Graph, "type", "b-graph"),
                    entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "b-edge"),
                    entry(ClusterIndexEntryKind::Attribute, "attr", "b-attr"),
                    entry(ClusterIndexEntryKind::Type, "mode", "b-type"),
                    entry(ClusterIndexEntryKind::Reduction, "lhs", "b-rule"),
                ],
            )],
        )
        .expect("origin b inserted");

        let view = db
            .import_scoped_view(view_request(&["origin-a"]))
            .expect("visible origin view materializes");

        assert_eq!(view.visible_origin_keys, ["origin-a"]);
        assert_eq!(origin_keys(&view.indexes.graph_rows), ["origin-a"]);
        assert_eq!(
            origin_keys(&view.indexes.subsumption_dag_rows),
            ["origin-a"]
        );
        assert_eq!(origin_keys(&view.indexes.attr_index_rows), ["origin-a"]);
        assert_eq!(origin_keys(&view.indexes.type_index_rows), ["origin-a"]);
        assert_eq!(
            origin_keys(&view.indexes.reduction_index_rows),
            ["origin-a"]
        );
    }

    #[test]
    fn import_scoped_view_reuses_across_unrelated_origin_changes() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "origin-a",
                0,
                vec![entry(ClusterIndexEntryKind::Attribute, "attr", "a")],
            )],
        )
        .expect("origin a inserted");
        db.apply_module_update(
            "pkg",
            "beta",
            vec![record_for(
                "origin-b",
                "pkg",
                "beta",
                0,
                vec![entry(ClusterIndexEntryKind::Reduction, "lhs", "old")],
            )],
        )
        .expect("origin b inserted");
        let before = db
            .import_scoped_view(view_request(&["origin-a"]))
            .expect("origin a view before unrelated change");

        db.apply_module_update(
            "pkg",
            "beta",
            vec![record_for(
                "origin-b",
                "pkg",
                "beta",
                0,
                vec![entry(ClusterIndexEntryKind::Reduction, "lhs", "new")],
            )],
        )
        .expect("unrelated origin b changed");

        let after = db
            .import_scoped_view(view_request(&["origin-a"]))
            .expect("origin a view after unrelated change");
        assert_eq!(after, before);
    }

    #[test]
    fn visible_origin_change_invalidates_exactly_affected_views() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "origin-a",
                0,
                vec![entry(ClusterIndexEntryKind::Graph, "type", "old")],
            )],
        )
        .expect("origin a inserted");
        db.apply_module_update(
            "pkg",
            "beta",
            vec![record_for(
                "origin-b",
                "pkg",
                "beta",
                0,
                vec![entry(ClusterIndexEntryKind::Type, "mode", "stable")],
            )],
        )
        .expect("origin b inserted");

        let before_a = db
            .import_scoped_view(view_request(&["origin-a"]))
            .expect("view a before");
        let before_b = db
            .import_scoped_view(view_request(&["origin-b"]))
            .expect("view b before");
        let before_both = db
            .import_scoped_view(view_request(&["origin-b", "origin-a"]))
            .expect("view both before");

        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "origin-a",
                0,
                vec![entry(ClusterIndexEntryKind::Graph, "type", "new")],
            )],
        )
        .expect("origin a changed");

        let after_a = db
            .import_scoped_view(view_request(&["origin-a"]))
            .expect("view a after");
        let after_b = db
            .import_scoped_view(view_request(&["origin-b"]))
            .expect("view b after");
        let after_both = db
            .import_scoped_view(view_request(&["origin-a", "origin-b"]))
            .expect("view both after");

        assert_ne!(after_a, before_a);
        assert_eq!(after_a.key, before_a.key);
        assert_eq!(after_b, before_b);
        assert_ne!(after_both, before_both);
        assert_ne!(
            before_a.key.visible_origin_set_hash,
            before_both.key.visible_origin_set_hash
        );
    }

    #[test]
    fn view_request_fails_closed_for_missing_origins_and_unknown_compatibility() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "origin-a",
                0,
                vec![entry(ClusterIndexEntryKind::Attribute, "attr", "a")],
            )],
        )
        .expect("origin inserted");

        assert!(matches!(
            db.import_scoped_view(view_request(&["missing-origin"])),
            Err(ClusterDbViewMiss::MissingVisibleOrigin { .. })
        ));

        for (field, mutate) in [
            (
                "importing_package_id",
                Box::new(|request: &mut ImportScopedViewRequest| {
                    request.importing_package_id = " ".to_owned();
                }) as Box<dyn Fn(&mut ImportScopedViewRequest)>,
            ),
            (
                "importing_module_path",
                Box::new(|request: &mut ImportScopedViewRequest| {
                    request.importing_module_path = " ".to_owned();
                }),
            ),
            (
                "import_closure_identity",
                Box::new(|request: &mut ImportScopedViewRequest| {
                    request.import_closure_identity = " ".to_owned();
                }),
            ),
        ] {
            let mut missing_identity = view_request(&["origin-a"]);
            mutate(&mut missing_identity);
            assert!(matches!(
                db.import_scoped_view(missing_identity),
                Err(ClusterDbViewMiss::MissingRequiredIdentity {
                    field: actual_field
                }) if actual_field == field
            ));
        }

        let mut unsupported_schema = view_request(&["origin-a"]);
        unsupported_schema.cluster_db_schema_version = SchemaVersion::new("unsupported");
        assert!(matches!(
            db.import_scoped_view(unsupported_schema),
            Err(ClusterDbViewMiss::UnsupportedSchema { .. })
        ));

        let mut unknown_policy = view_request(&["origin-a"]);
        unknown_policy.policy_compatibility[0].value = "unknown".to_owned();
        assert!(matches!(
            db.import_scoped_view(unknown_policy),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "policy_compatibility"
            })
        ));

        let mut unknown_schema_compatibility = view_request(&["origin-a"]);
        unknown_schema_compatibility.schema_compatibility[0].value = "unknown".to_owned();
        assert!(matches!(
            db.import_scoped_view(unknown_schema_compatibility),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "schema_compatibility"
            })
        ));

        let mut unsupported_producer_schema = view_request(&["origin-a"]);
        unsupported_producer_schema.producer_schema_versions[0].version =
            SchemaVersion::new("unknown");
        assert!(matches!(
            db.import_scoped_view(unsupported_producer_schema),
            Err(ClusterDbViewMiss::UnsupportedSchema { .. })
        ));

        let mut unknown_toolchain = view_request(&["origin-a"]);
        unknown_toolchain.toolchain_compatibility[0].value = "unsupported".to_owned();
        assert!(matches!(
            db.import_scoped_view(unknown_toolchain),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "toolchain_compatibility"
            })
        ));

        let mut missing_schema = view_request(&["origin-a"]);
        missing_schema.producer_schema_versions.clear();
        assert!(matches!(
            db.import_scoped_view(missing_schema),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "producer_schema_versions"
            })
        ));

        let mut missing_schema_compatibility = view_request(&["origin-a"]);
        missing_schema_compatibility.schema_compatibility.clear();
        assert!(matches!(
            db.import_scoped_view(missing_schema_compatibility),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "schema_compatibility"
            })
        ));

        let mut producer_schema_mismatch = view_request(&["origin-a"]);
        producer_schema_mismatch.producer_schema_versions[0].version = SchemaVersion::new("2.0");
        assert!(matches!(
            db.import_scoped_view(producer_schema_mismatch),
            Err(ClusterDbViewMiss::UnsupportedSchema { .. })
        ));

        let mut policy_mismatch = view_request(&["origin-a"]);
        policy_mismatch.policy_compatibility[0].value = "different-known".to_owned();
        assert!(matches!(
            db.import_scoped_view(policy_mismatch),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "policy_compatibility"
            })
        ));

        let mut schema_compatibility_mismatch = view_request(&["origin-a"]);
        schema_compatibility_mismatch.schema_compatibility[0].value = "different-known".to_owned();
        assert!(matches!(
            db.import_scoped_view(schema_compatibility_mismatch),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "schema_compatibility"
            })
        ));

        let mut toolchain_mismatch = view_request(&["origin-a"]);
        toolchain_mismatch.toolchain_compatibility[0].value = "different-known".to_owned();
        assert!(matches!(
            db.import_scoped_view(toolchain_mismatch),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "toolchain_compatibility"
            })
        ));

        let mut missing_traversal = view_request(&["origin-a"]);
        missing_traversal.traversal_profile.clear();
        assert!(matches!(
            db.import_scoped_view(missing_traversal),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "traversal_profile"
            })
        ));

        let mut mismatched_policy = view_request(&["origin-a"]);
        mismatched_policy.verifier_policy_fingerprint = PolicyFingerprint::new(hash(42));
        assert!(matches!(
            db.import_scoped_view(mismatched_policy),
            Err(ClusterDbViewMiss::UnknownCompatibility {
                family: "verifier_policy_fingerprint"
            })
        ));
    }

    #[test]
    fn import_scoped_view_order_independent_of_visible_origin_order_and_write_order() {
        let origin_a = record(
            "origin-a",
            0,
            vec![
                entry(ClusterIndexEntryKind::Graph, "type", "a-graph"),
                entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "a-edge"),
                entry(ClusterIndexEntryKind::Attribute, "attr", "a-attr"),
                entry(ClusterIndexEntryKind::Type, "mode", "a-type"),
                entry(ClusterIndexEntryKind::Reduction, "lhs", "a-rule"),
            ],
        );
        let origin_b = record_for(
            "origin-b",
            "pkg",
            "beta",
            0,
            vec![
                entry(ClusterIndexEntryKind::Graph, "type", "b-graph"),
                entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "b-edge"),
                entry(ClusterIndexEntryKind::Attribute, "attr", "b-attr"),
                entry(ClusterIndexEntryKind::Type, "mode", "b-type"),
                entry(ClusterIndexEntryKind::Reduction, "lhs", "b-rule"),
            ],
        );

        let mut left = ClusterDbIndex::new();
        left.apply_module_update("pkg", "alpha", vec![origin_a.clone()])
            .expect("left origin a");
        left.apply_module_update("pkg", "beta", vec![origin_b.clone()])
            .expect("left origin b");

        let mut right = ClusterDbIndex::new();
        right
            .apply_module_update("pkg", "beta", vec![origin_b])
            .expect("right origin b");
        right
            .apply_module_update("pkg", "alpha", vec![origin_a])
            .expect("right origin a");

        let left_view = left
            .import_scoped_view(view_request(&["origin-b", "origin-a", "origin-a"]))
            .expect("left view");
        let right_view = right
            .import_scoped_view(view_request(&["origin-a", "origin-b"]))
            .expect("right view");

        assert_eq!(left_view, right_view);
        assert_eq!(left_view.visible_origin_keys, ["origin-a", "origin-b"]);
        assert_eq!(
            origin_keys(&left_view.indexes.graph_rows),
            ["origin-a", "origin-b"]
        );
        assert_eq!(
            origin_keys(&left_view.indexes.subsumption_dag_rows),
            ["origin-a", "origin-b"]
        );
        assert_eq!(
            origin_keys(&left_view.indexes.attr_index_rows),
            ["origin-a", "origin-b"]
        );
        assert_eq!(
            origin_keys(&left_view.indexes.type_index_rows),
            ["origin-a", "origin-b"]
        );
        assert_eq!(
            origin_keys(&left_view.indexes.reduction_index_rows),
            ["origin-a", "origin-b"]
        );
    }

    #[test]
    fn import_scoped_view_does_not_infer_hidden_trace_steps() {
        let mut db = ClusterDbIndex::new();
        db.apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "origin-a",
                0,
                vec![entry(ClusterIndexEntryKind::Graph, "type", "graph-only")],
            )],
        )
        .expect("graph-only origin inserted");
        db.apply_module_update(
            "pkg",
            "beta",
            vec![record_for(
                "origin-b",
                "pkg",
                "beta",
                0,
                vec![entry(ClusterIndexEntryKind::Reduction, "lhs", "reduction")],
            )],
        )
        .expect("reduction origin inserted");

        let view = db
            .import_scoped_view(view_request(&["origin-a"]))
            .expect("view materializes");

        assert_eq!(origin_keys(&view.indexes.graph_rows), ["origin-a"]);
        assert!(view.indexes.reduction_index_rows.is_empty());
        assert!(view.indexes.subsumption_dag_rows.is_empty());
        assert!(view.indexes.attr_index_rows.is_empty());
        assert!(view.indexes.type_index_rows.is_empty());

        let reduction_only = db
            .import_scoped_view(view_request(&["origin-b"]))
            .expect("visible reduction-only view materializes");

        assert!(reduction_only.indexes.graph_rows.is_empty());
        assert!(reduction_only.indexes.subsumption_dag_rows.is_empty());
        assert!(reduction_only.indexes.attr_index_rows.is_empty());
        assert!(reduction_only.indexes.type_index_rows.is_empty());
        assert_eq!(
            origin_keys(&reduction_only.indexes.reduction_index_rows),
            ["origin-b"]
        );
    }

    fn record(
        origin_key: &str,
        declaration_order: u32,
        index_entries: Vec<ClusterIndexEntry>,
    ) -> ClusterContributionRecord {
        record_for(origin_key, "pkg", "alpha", declaration_order, index_entries)
    }

    fn record_for(
        origin_key: &str,
        package_id: &str,
        module_path: &str,
        declaration_order: u32,
        index_entries: Vec<ClusterIndexEntry>,
    ) -> ClusterContributionRecord {
        ClusterContributionRecord {
            schema_version: SchemaVersion::new(CLUSTER_DB_SCHEMA_VERSION),
            origin: ClusterContributionOrigin {
                origin_key: origin_key.to_owned(),
                package_id: package_id.to_owned(),
                module_path: module_path.to_owned(),
                stable_contribution_id: format!("{origin_key}-id"),
                label: format!("{origin_key}-label"),
                contribution_kind: ClusterContributionKind::ConditionalCluster,
                target_pattern_hash: hash(1),
                guard_hash: hash(2),
                declared_contribution_hash: hash(3),
                accepted_visibility: ClusterContributionVisibility::ImporterVisible,
                accepted_status: ClusterContributionStatus::Accepted,
                accepted_status_projection_hash: hash(4),
                accepted_witness_or_discharge_hash: Some(named_hash("witness", 5)),
                proof_backed: true,
                verifier_policy_fingerprint: PolicyFingerprint::new(hash(6)),
                policy_compatibility: vec![compat("verifier-policy", "known")],
                schema_compatibility: vec![compat("cluster-db-schema", "known")],
                toolchain_compatibility: vec![compat("producer-toolchain", "known")],
                producer_schema_versions: vec![schema("registration-summary")],
                trace_replay_hashes: Some(vec![named_hash("trace", 7)]),
                dependency_interface_hashes: Some(vec![named_hash("registration-interface", 8)]),
                origin_footprint_hash: Some(hash(9)),
                footprint_completeness: ClusterOriginFootprintCompleteness::Complete,
                uncacheable: false,
            },
            declaration_order,
            index_entries,
            diagnostic_refs: vec![named_hash("diagnostic", 10)],
        }
    }

    fn view_request(visible_origin_keys: &[&str]) -> ImportScopedViewRequest {
        ImportScopedViewRequest {
            importing_package_id: "consumer-pkg".to_owned(),
            importing_module_path: "consumer".to_owned(),
            import_closure_identity: "consumer/import-closure/v1".to_owned(),
            visible_origin_keys: visible_origin_keys
                .iter()
                .map(|origin_key| (*origin_key).to_owned())
                .collect(),
            verifier_policy_fingerprint: PolicyFingerprint::new(hash(6)),
            cluster_db_schema_version: SchemaVersion::new(CLUSTER_DB_SCHEMA_VERSION),
            producer_schema_versions: vec![schema("registration-summary")],
            policy_compatibility: vec![compat("verifier-policy", "known")],
            schema_compatibility: vec![compat("cluster-db-schema", "known")],
            toolchain_compatibility: vec![compat("producer-toolchain", "known")],
            traversal_profile: vec![compat("traversal-profile", "canonical")],
        }
    }

    fn entry(
        entry_kind: ClusterIndexEntryKind,
        primary_key: &str,
        secondary_key: &str,
    ) -> ClusterIndexEntry {
        ClusterIndexEntry {
            entry_kind,
            primary_key: primary_key.to_owned(),
            secondary_key: secondary_key.to_owned(),
            strategy_key: format!("{primary_key}->{secondary_key}"),
            contribution_fingerprint: hash(primary_key.as_bytes()[0] ^ secondary_key.as_bytes()[0]),
        }
    }

    fn schema(name: &str) -> NamedSchemaVersion {
        NamedSchemaVersion {
            schema_family: "mizar-artifact/registration-summary".to_owned(),
            name: name.to_owned(),
            version: SchemaVersion::new("1.0"),
        }
    }

    fn named_hash(name: &str, seed: u8) -> NamedHash {
        NamedHash {
            name: name.to_owned(),
            domain: format!("test-domain/{name}"),
            digest: hash(seed),
        }
    }

    fn compat(field_name: &str, value: &str) -> CompatibilityField {
        CompatibilityField {
            family: "cluster-db-test".to_owned(),
            field_name: field_name.to_owned(),
            value: value.to_owned(),
        }
    }

    fn origin_keys(rows: &[ClusterAggregateRow]) -> Vec<&str> {
        rows.iter().map(|row| row.origin_key.as_str()).collect()
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }
}
