//! Internal build-cache optimization services.
//!
//! See the canonical crate plan at
//! [`doc/design/mizar-cache/en/00.crate_plan.md`](../../../doc/design/mizar-cache/en/00.crate_plan.md).
//!
//! `mizar-cache` owns cache-key, dependency-fingerprint, record/blob-store,
//! proof-reuse validation, and cluster-db work as optimization data. It does
//! not own proof authority, kernel acceptance, proof-witness selection, build
//! scheduling, IR storage, or artifact publication.
