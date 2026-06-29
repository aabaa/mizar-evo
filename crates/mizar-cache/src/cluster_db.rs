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
    /// Required schema or toolchain compatibility is unknown or unsupported.
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

    fn record(
        origin_key: &str,
        declaration_order: u32,
        index_entries: Vec<ClusterIndexEntry>,
    ) -> ClusterContributionRecord {
        ClusterContributionRecord {
            schema_version: SchemaVersion::new(CLUSTER_DB_SCHEMA_VERSION),
            origin: ClusterContributionOrigin {
                origin_key: origin_key.to_owned(),
                package_id: "pkg".to_owned(),
                module_path: "alpha".to_owned(),
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
