//! Proof policy, selection, status projection, and witness-publication boundary.
//!
//! `mizar-proof` owns policy decisions between untrusted evidence producers and
//! trusted kernel validation results. It records, selects, projects, and stages
//! proof evidence, but it does not accept proofs.
//!
//! Trusted acceptance comes only from `mizar-kernel` `KernelCheckResult`
//! values whose status is accepted. Backend proof methods, resolution traces,
//! SMT proof objects, backend logs, externally attested records, diagnostics,
//! and cache records are not promoted into trusted proof status or trusted
//! `used_axioms`.

#![forbid(unsafe_code)]

pub mod policy;
