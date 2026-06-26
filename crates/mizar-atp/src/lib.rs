//! ATP candidate-evidence production boundary.
//!
//! `mizar-atp` owns pipeline phase 13: translating open VC obligations into
//! backend-neutral ATP problems, running untrusted backends, and collecting
//! formula/substitution evidence candidates for `mizar-kernel`.
//!
//! This crate does not accept proofs, select trusted winners, call the kernel
//! as proof authority, or expose backend proof methods as trusted material.
//! Current task 1 intentionally publishes no semantic modules until their
//! English/Japanese module specs are added by later tasks.

#![forbid(unsafe_code)]
