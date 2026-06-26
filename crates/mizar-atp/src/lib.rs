//! ATP candidate-evidence production boundary.
//!
//! `mizar-atp` owns pipeline phase 13: translating `NeedsAtp` VC obligations
//! into backend-neutral ATP problems, running untrusted backends, and
//! collecting formula/substitution evidence candidates for `mizar-kernel`.
//!
//! This crate does not accept proofs, select trusted winners, call the kernel
//! as proof authority, or expose backend proof methods as trusted material.

#![forbid(unsafe_code)]

pub mod backend;
pub mod problem;
pub mod property_encoding;
pub mod smtlib_encoder;
pub mod tptp_encoder;
pub mod translator;
