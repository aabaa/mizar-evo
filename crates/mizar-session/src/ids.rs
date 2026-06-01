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
        SourceId, SourceMapId,
    };
    use std::collections::HashSet;
    use std::error::Error;
    use std::fmt::Debug;
    use std::hash::Hash as HashTrait;

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
        let cloned = id.clone();

        assert_eq!(copied, id);
        assert_eq!(cloned, id);
        assert!(format!("{id:?}").starts_with(debug_name));

        let mut ids = HashSet::new();
        ids.insert(id);
        assert!(ids.contains(&copied));
    }

    fn assert_error_trait(error: IdError) {
        let error: &dyn Error = &error;
        assert!(error.source().is_none());
    }
}
