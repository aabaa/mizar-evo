# Spec Traceability

This directory contains the machine-readable mapping between `doc/spec/`
requirements and committed tests.

The specification files do not contain per-test links. `spec_trace.toml` is the
source of truth for coverage tracking.

## Documentation Volume Baseline

`doc_volume_baseline.tsv` is the ratchet behind the `documentation_volume_stays_within_baseline`
lint in `crates/mizar-test/tests/lint_policy.rs`. Each row records the ceiling an
existing document already exceeds: `ceremony_tokens` (evaluation scores, gate
tallies, model names, digests in `doc/design`), `contract_lines` (task contracts
over 60 lines), and `contract_fanout` (module documents linking a contract).
Files without a row are held at the default. Rows may only be lowered or
removed; the rule text is the "Documentation Volume Ledger" section of
`doc/design/autonomous_crate_development.md`.
