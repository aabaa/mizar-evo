//! Internal build cache keys, records, proof-reuse validation, and cluster-db
//! storage.
//!
//! `mizar-cache` owns cache optimization surfaces described by
//! [`00.crate_plan.md`]. It does not accept proofs.
//! Trusted acceptance comes only from `mizar-kernel` `KernelCheckResult`
//! values selected and projected by their owning proof/status layers; cache
//! records, externally attested evidence, backend diagnostics, backend logs,
//! timing metadata, and cache hit/miss state are optimization metadata.
//! Cache metadata is not promoted into kernel-verified status or trusted
//! `used_axioms`.
//!
//! [`00.crate_plan.md`]: ../../../doc/design/mizar-cache/en/00.crate_plan.md

#![forbid(unsafe_code)]
