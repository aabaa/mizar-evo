//! Opaque identifiers owned by `mizar-session`.
//!
//! ```compile_fail
//! use mizar_session::{BuildRequestId, BuildSessionId};
//!
//! fn takes_session_id(_id: BuildSessionId) {}
//! fn cannot_use_request_id_as_session_id(id: BuildRequestId) {
//!     takes_session_id(id);
//! }
//! ```
//!
//! ```compile_fail
//! use mizar_session::{BuildRequestId, BuildSnapshotId};
//!
//! fn takes_request_id(_id: BuildRequestId) {}
//! fn cannot_use_snapshot_id_as_request_id(id: BuildSnapshotId) {
//!     takes_request_id(id);
//! }
//! ```
//!
//! ```compile_fail
//! use mizar_session::{BuildSnapshotId, SourceId};
//!
//! fn takes_snapshot_id(_id: BuildSnapshotId) {}
//! fn cannot_use_source_id_as_snapshot_id(id: SourceId) {
//!     takes_snapshot_id(id);
//! }
//! ```
//!
//! ```compile_fail
//! use mizar_session::{SourceId, SourceMapId};
//!
//! fn takes_source_id(_id: SourceId) {}
//! fn cannot_use_source_map_id_as_source_id(id: SourceMapId) {
//!     takes_source_id(id);
//! }
//! ```
//!
//! ```compile_fail
//! use mizar_session::{SnapshotLeaseId, SourceMapId};
//!
//! fn takes_source_map_id(_id: SourceMapId) {}
//! fn cannot_use_lease_id_as_source_map_id(id: SnapshotLeaseId) {
//!     takes_source_map_id(id);
//! }
//! ```
//!
//! ```compile_fail
//! use mizar_session::{BuildSessionId, SnapshotLeaseId};
//!
//! fn takes_lease_id(_id: SnapshotLeaseId) {}
//! fn cannot_use_session_id_as_lease_id(id: BuildSessionId) {
//!     takes_lease_id(id);
//! }
//! ```
//!
//! ```compile_fail
//! use mizar_session::BuildSessionId;
//!
//! fn requires_semantic_order<T: Ord>() {}
//! requires_semantic_order::<BuildSessionId>();
//! ```
//!
//! ```compile_fail
//! use mizar_session::BuildRequestId;
//!
//! fn requires_semantic_order<T: Ord>() {}
//! requires_semantic_order::<BuildRequestId>();
//! ```
//!
//! ```compile_fail
//! use mizar_session::BuildSnapshotId;
//!
//! fn requires_semantic_order<T: Ord>() {}
//! requires_semantic_order::<BuildSnapshotId>();
//! ```
//!
//! ```compile_fail
//! use mizar_session::SourceId;
//!
//! fn requires_semantic_order<T: Ord>() {}
//! requires_semantic_order::<SourceId>();
//! ```
//!
//! ```compile_fail
//! use mizar_session::SourceMapId;
//!
//! fn requires_semantic_order<T: Ord>() {}
//! requires_semantic_order::<SourceMapId>();
//! ```
//!
//! ```compile_fail
//! use mizar_session::SnapshotLeaseId;
//!
//! fn requires_semantic_order<T: Ord>() {}
//! requires_semantic_order::<SnapshotLeaseId>();
//! ```
//!
//! ```compile_fail
//! use mizar_session::ids::OpaqueId;
//! ```
//!
//! ```compile_fail
//! use mizar_session::BuildSessionId;
//!
//! let id = BuildSessionId(0);
//! ```
//!
//! ```compile_fail
//! use mizar_session::{BuildSnapshotId, Hash};
//!
//! let id = BuildSnapshotId(Hash::from_bytes([0; Hash::BYTE_LEN]));
//! ```
//!
//! ```compile_fail
//! use mizar_session::SourceId;
//!
//! fn expose_raw_id(id: SourceId) {
//!     let SourceId(raw) = id;
//!     let _ = raw;
//! }
//! ```

use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[allow(dead_code)]
const BUILD_SNAPSHOT_HASH_DOMAIN: &[u8] = b"mizar-session/build-snapshot-id/v1";
const BUILD_SNAPSHOT_SERIALIZED_PREFIX: &str = "mizar-session-build-snapshot-v1:";

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub(crate) struct OpaqueId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct Hash([u8; Self::BYTE_LEN]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct BuildSessionId(OpaqueId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct BuildRequestId(OpaqueId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct BuildSnapshotId(Hash);

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct SourceId(OpaqueId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct SourceMapId(OpaqueId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct SnapshotLeaseId(OpaqueId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdError {
    MalformedSerializedId,
    WrongIdDomain,
    UnknownSnapshotRegistry,
    AllocatorOverflow,
    NonPersistableSerialization,
}

impl Hash {
    pub const BYTE_LEN: usize = 32;

    pub const fn from_bytes(bytes: [u8; Self::BYTE_LEN]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; Self::BYTE_LEN] {
        &self.0
    }
}

impl BuildSnapshotId {
    pub const SERIALIZED_LEN: usize = BUILD_SNAPSHOT_SERIALIZED_PREFIX.len() + Hash::BYTE_LEN * 2;

    pub fn to_published_schema_string(self) -> Result<String, IdError> {
        let mut encoded = String::with_capacity(Self::SERIALIZED_LEN);
        encoded.push_str(BUILD_SNAPSHOT_SERIALIZED_PREFIX);
        push_lower_hex(&mut encoded, self.0.as_bytes());
        Ok(encoded)
    }

    pub fn from_published_schema_str(serialized: &str) -> Result<Self, IdError> {
        let Some(hex) = serialized.strip_prefix(BUILD_SNAPSHOT_SERIALIZED_PREFIX) else {
            return if has_serialized_id_shape(serialized) {
                Err(IdError::WrongIdDomain)
            } else {
                Err(IdError::MalformedSerializedId)
            };
        };

        decode_lower_hex_hash(hex).map(Self)
    }
}

impl FromStr for BuildSnapshotId {
    type Err = IdError;

    fn from_str(serialized: &str) -> Result<Self, Self::Err> {
        Self::from_published_schema_str(serialized)
    }
}

impl BuildSessionId {
    pub fn to_published_schema_string(self) -> Result<String, IdError> {
        reject_non_persistable_id()
    }
}

impl BuildRequestId {
    pub fn to_published_schema_string(self) -> Result<String, IdError> {
        reject_non_persistable_id()
    }
}

impl SourceId {
    pub fn to_published_schema_string(self) -> Result<String, IdError> {
        reject_non_persistable_id()
    }
}

impl SourceMapId {
    pub fn to_published_schema_string(self) -> Result<String, IdError> {
        reject_non_persistable_id()
    }
}

impl SnapshotLeaseId {
    pub fn to_published_schema_string(self) -> Result<String, IdError> {
        reject_non_persistable_id()
    }
}

#[allow(dead_code)]
pub(crate) fn build_snapshot_id_from_sorted_canonical_bytes(
    schema_identity: &[u8],
    toolchain_identity: &[u8],
    sorted_canonical_snapshot_bytes: &[u8],
) -> BuildSnapshotId {
    build_snapshot_id_from_parts(
        BUILD_SNAPSHOT_HASH_DOMAIN,
        schema_identity,
        toolchain_identity,
        sorted_canonical_snapshot_bytes,
    )
}

#[allow(dead_code)]
fn build_snapshot_id_from_parts(
    domain_separator: &[u8],
    schema_identity: &[u8],
    toolchain_identity: &[u8],
    sorted_canonical_snapshot_bytes: &[u8],
) -> BuildSnapshotId {
    let mut hasher = blake3::Hasher::new();
    update_hash_part(&mut hasher, b"domain", domain_separator);
    update_hash_part(&mut hasher, b"schema", schema_identity);
    update_hash_part(&mut hasher, b"toolchain", toolchain_identity);
    update_hash_part(
        &mut hasher,
        b"sorted-canonical-snapshot",
        sorted_canonical_snapshot_bytes,
    );
    BuildSnapshotId(Hash::from_bytes(*hasher.finalize().as_bytes()))
}

#[allow(dead_code)]
fn update_hash_part(hasher: &mut blake3::Hasher, label: &[u8], bytes: &[u8]) {
    hasher.update(&(label.len() as u64).to_le_bytes());
    hasher.update(label);
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn reject_non_persistable_id() -> Result<String, IdError> {
    Err(IdError::NonPersistableSerialization)
}

fn push_lower_hex(output: &mut String, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
}

fn decode_lower_hex_hash(hex: &str) -> Result<Hash, IdError> {
    if hex.len() != Hash::BYTE_LEN * 2 {
        return Err(IdError::MalformedSerializedId);
    }

    let mut bytes = [0; Hash::BYTE_LEN];
    for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        let high = decode_lower_hex_nibble(pair[0])?;
        let low = decode_lower_hex_nibble(pair[1])?;
        bytes[index] = (high << 4) | low;
    }

    Ok(Hash::from_bytes(bytes))
}

fn decode_lower_hex_nibble(byte: u8) -> Result<u8, IdError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(IdError::MalformedSerializedId),
    }
}

fn has_serialized_id_shape(serialized: &str) -> bool {
    let Some((domain, hex)) = serialized.split_once(':') else {
        return false;
    };

    !domain.is_empty()
        && hex.len() == Hash::BYTE_LEN * 2
        && hex
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedSerializedId => f.write_str("malformed serialized id"),
            Self::WrongIdDomain => f.write_str("serialized id has the wrong domain separator"),
            Self::UnknownSnapshotRegistry => {
                f.write_str("id belongs to an unknown snapshot registry")
            }
            Self::AllocatorOverflow => f.write_str("id allocator overflowed"),
            Self::NonPersistableSerialization => {
                f.write_str("non-persistable id cannot be serialized into a published schema")
            }
        }
    }
}

impl Error for IdError {}

#[cfg(test)]
mod tests {
    use super::{
        BuildRequestId, BuildSessionId, BuildSnapshotId, Hash, IdError, OpaqueId, SnapshotLeaseId,
        SourceId, SourceMapId, build_snapshot_id_from_parts,
        build_snapshot_id_from_sorted_canonical_bytes,
    };
    use std::collections::HashSet;
    use std::error::Error;
    use std::fmt::Debug;
    use std::hash::Hash as HashTrait;
    use std::str::FromStr;

    #[test]
    fn allocator_issued_ids_compare_equal_only_within_their_domain() {
        assert_eq!(BuildSessionId(OpaqueId(7)), BuildSessionId(OpaqueId(7)));
        assert_ne!(BuildSessionId(OpaqueId(7)), BuildSessionId(OpaqueId(8)));

        assert_eq!(BuildRequestId(OpaqueId(7)), BuildRequestId(OpaqueId(7)));
        assert_ne!(BuildRequestId(OpaqueId(7)), BuildRequestId(OpaqueId(8)));

        assert_eq!(SourceId(OpaqueId(7)), SourceId(OpaqueId(7)));
        assert_ne!(SourceId(OpaqueId(7)), SourceId(OpaqueId(8)));

        assert_eq!(SourceMapId(OpaqueId(7)), SourceMapId(OpaqueId(7)));
        assert_ne!(SourceMapId(OpaqueId(7)), SourceMapId(OpaqueId(8)));

        assert_eq!(SnapshotLeaseId(OpaqueId(7)), SnapshotLeaseId(OpaqueId(7)));
        assert_ne!(SnapshotLeaseId(OpaqueId(7)), SnapshotLeaseId(OpaqueId(8)));
    }

    #[test]
    fn build_snapshot_ids_compare_by_hash_identity() {
        let first = BuildSnapshotId(Hash::from_bytes([1; Hash::BYTE_LEN]));
        let same = BuildSnapshotId(Hash::from_bytes([1; Hash::BYTE_LEN]));
        let different = BuildSnapshotId(Hash::from_bytes([2; Hash::BYTE_LEN]));

        assert_eq!(first, same);
        assert_ne!(first, different);
    }

    #[test]
    fn ids_are_copy_clone_debug_and_hashable() {
        assert_copy_clone_debug_hash(BuildSessionId(OpaqueId(1)), "BuildSessionId");
        assert_copy_clone_debug_hash(BuildRequestId(OpaqueId(2)), "BuildRequestId");
        assert_copy_clone_debug_hash(
            BuildSnapshotId(Hash::from_bytes([3; Hash::BYTE_LEN])),
            "BuildSnapshotId",
        );
        assert_copy_clone_debug_hash(SourceId(OpaqueId(4)), "SourceId");
        assert_copy_clone_debug_hash(SourceMapId(OpaqueId(5)), "SourceMapId");
        assert_copy_clone_debug_hash(SnapshotLeaseId(OpaqueId(6)), "SnapshotLeaseId");
    }

    #[test]
    fn hash_newtype_preserves_bytes_without_encoding_policy() {
        let mut bytes = [0; Hash::BYTE_LEN];
        bytes[0] = 0x12;
        bytes[31] = 0xab;

        let hash = Hash::from_bytes(bytes);

        assert_eq!(hash.as_bytes(), &bytes);
        assert_eq!(hash, Hash::from_bytes(bytes));
        assert!(format!("{hash:?}").starts_with("Hash("));
    }

    #[test]
    fn build_snapshot_id_serializes_as_domain_prefixed_lowercase_hex() {
        let mut bytes = [0; Hash::BYTE_LEN];
        bytes[0] = 0x01;
        bytes[1] = 0x23;
        bytes[30] = 0xab;
        bytes[31] = 0xcd;
        let id = BuildSnapshotId(Hash::from_bytes(bytes));

        let serialized = id.to_published_schema_string().unwrap();

        assert_eq!(
            serialized,
            "mizar-session-build-snapshot-v1:012300000000000000000000000000000000000000000000000000000000abcd"
        );
        assert_eq!(serialized.len(), BuildSnapshotId::SERIALIZED_LEN);
        assert_eq!(
            BuildSnapshotId::from_published_schema_str(&serialized),
            Ok(id)
        );
        assert_eq!(BuildSnapshotId::from_str(&serialized), Ok(id));
    }

    #[test]
    fn build_snapshot_id_rejects_non_canonical_or_malformed_hex() {
        let uppercase = "mizar-session-build-snapshot-v1:ABCD000000000000000000000000000000000000000000000000000000000000";
        let short = "mizar-session-build-snapshot-v1:abcd";
        let empty = "mizar-session-build-snapshot-v1:";
        let long = "mizar-session-build-snapshot-v1:012300000000000000000000000000000000000000000000000000000000abcd00";
        let invalid = "mizar-session-build-snapshot-v1:012g000000000000000000000000000000000000000000000000000000000000";
        let missing_domain = "012300000000000000000000000000000000000000000000000000000000abcd";

        assert_eq!(
            BuildSnapshotId::from_published_schema_str(uppercase),
            Err(IdError::MalformedSerializedId)
        );
        assert_eq!(
            BuildSnapshotId::from_published_schema_str(short),
            Err(IdError::MalformedSerializedId)
        );
        assert_eq!(
            BuildSnapshotId::from_published_schema_str(empty),
            Err(IdError::MalformedSerializedId)
        );
        assert_eq!(
            BuildSnapshotId::from_published_schema_str(long),
            Err(IdError::MalformedSerializedId)
        );
        assert_eq!(
            BuildSnapshotId::from_published_schema_str(invalid),
            Err(IdError::MalformedSerializedId)
        );
        assert_eq!(
            BuildSnapshotId::from_published_schema_str(missing_domain),
            Err(IdError::MalformedSerializedId)
        );
    }

    #[test]
    fn build_snapshot_id_rejects_well_formed_ids_from_the_wrong_domain() {
        let wrong_domain = "mizar-session-source-v1:012300000000000000000000000000000000000000000000000000000000abcd";
        let wrong_domain_with_malformed_hash = "mizar-session-source-v1:abcd";

        assert_eq!(
            BuildSnapshotId::from_published_schema_str(wrong_domain),
            Err(IdError::WrongIdDomain)
        );
        assert_eq!(
            BuildSnapshotId::from_published_schema_str(wrong_domain_with_malformed_hash),
            Err(IdError::MalformedSerializedId)
        );
    }

    #[test]
    fn build_snapshot_hash_includes_domain_schema_and_toolchain_parts() {
        let id = build_snapshot_id_from_sorted_canonical_bytes(
            b"snapshot-schema-v1",
            b"toolchain-a",
            b"already sorted canonical bytes",
        );
        let same = build_snapshot_id_from_sorted_canonical_bytes(
            b"snapshot-schema-v1",
            b"toolchain-a",
            b"already sorted canonical bytes",
        );
        let changed_domain = build_snapshot_id_from_parts(
            b"mizar-session/build-snapshot-id/v2",
            b"snapshot-schema-v1",
            b"toolchain-a",
            b"already sorted canonical bytes",
        );
        let changed_schema = build_snapshot_id_from_sorted_canonical_bytes(
            b"snapshot-schema-v2",
            b"toolchain-a",
            b"already sorted canonical bytes",
        );
        let changed_toolchain = build_snapshot_id_from_sorted_canonical_bytes(
            b"snapshot-schema-v1",
            b"toolchain-b",
            b"already sorted canonical bytes",
        );
        let changed_canonical_bytes = build_snapshot_id_from_sorted_canonical_bytes(
            b"snapshot-schema-v1",
            b"toolchain-a",
            b"different sorted canonical bytes",
        );

        assert_eq!(id, same);
        assert_ne!(id, changed_domain);
        assert_ne!(id, changed_schema);
        assert_ne!(id, changed_toolchain);
        assert_ne!(id, changed_canonical_bytes);
    }

    #[test]
    fn allocator_issued_ids_reject_published_schema_serialization() {
        assert_eq!(
            BuildSessionId(OpaqueId(1)).to_published_schema_string(),
            Err(IdError::NonPersistableSerialization)
        );
        assert_eq!(
            BuildRequestId(OpaqueId(2)).to_published_schema_string(),
            Err(IdError::NonPersistableSerialization)
        );
        assert_eq!(
            SourceId(OpaqueId(3)).to_published_schema_string(),
            Err(IdError::NonPersistableSerialization)
        );
        assert_eq!(
            SourceMapId(OpaqueId(4)).to_published_schema_string(),
            Err(IdError::NonPersistableSerialization)
        );
        assert_eq!(
            SnapshotLeaseId(OpaqueId(5)).to_published_schema_string(),
            Err(IdError::NonPersistableSerialization)
        );
    }

    #[test]
    fn id_error_has_the_specified_basic_variants() {
        let cases = [
            (
                IdError::MalformedSerializedId,
                "MalformedSerializedId",
                "malformed",
            ),
            (IdError::WrongIdDomain, "WrongIdDomain", "wrong domain"),
            (
                IdError::UnknownSnapshotRegistry,
                "UnknownSnapshotRegistry",
                "unknown snapshot registry",
            ),
            (
                IdError::AllocatorOverflow,
                "AllocatorOverflow",
                "overflowed",
            ),
            (
                IdError::NonPersistableSerialization,
                "NonPersistableSerialization",
                "non-persistable",
            ),
        ];

        for (error, debug_name, display_fragment) in cases {
            let display = error.to_string();
            assert!(display.contains(display_fragment));
            assert_eq!(format!("{error:?}"), debug_name);
            assert_error_trait(error);
        }
    }

    fn assert_copy_clone_debug_hash<T>(id: T, debug_name: &str)
    where
        T: Copy + Clone + Debug + Eq + HashTrait,
    {
        let copied = id;
        let cloned = clone_id(&id);

        assert_eq!(copied, id);
        assert_eq!(cloned, id);
        assert!(format!("{id:?}").starts_with(debug_name));

        let mut ids = HashSet::new();
        ids.insert(id);
        assert!(ids.contains(&copied));
    }

    fn clone_id<T>(id: &T) -> T
    where
        T: Clone,
    {
        id.clone()
    }

    fn assert_error_trait(error: IdError) {
        let error: &dyn Error = &error;
        assert!(error.source().is_none());
    }
}
