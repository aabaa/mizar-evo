# Public Enum Policy

> Canonical language: English. Japanese companion:
> [../ja/public_enum_policy.md](../ja/public_enum_policy.md).

## Purpose

This document is the canonical Task 19 classification for public enums in
`mizar-kernel`. It applies the public-enum forward-compatibility procedure to
the trusted kernel crate after the phase-14 public API surface exists.

## Policy

All public enums in `mizar-kernel` are forward-compatible API surfaces and
must be marked `#[non_exhaustive]` immediately before `pub enum`. Task 19
records no justified exhaustive exceptions.

Adding a public enum requires adding it to the inventory below, adding the
matching Japanese companion entry, and keeping the crate-local lint guard
green. Removing a public enum or changing a variant's meaning requires a
source/documentation consistency review in the same task.

`RejectionCategory` and `RejectionDetail` additionally follow
[rejection.md](./rejection.md) and
[architecture 19](../../architecture/en/19.failure_semantics.md): stable-key
spelling, meaning, phase ownership, ordering, removal, rename, or category
remapping changes require compatibility review. Adding a rejection detail is
allowed only when the owning phase is documented and tests assert category,
detail key, and deterministic location.

## Gap Classification

- `design_drift`: earlier module work established `#[non_exhaustive]` for
  several public enums but did not create one crate-wide inventory.
- `source_drift`: public enums added after the initial rejection baseline were
  missing the explicit forward-compatibility marker before task 19.
- `deferred`: downstream crates that match public kernel enums must use
  wildcard arms; downstream compatibility checks remain outside this crate.

## Inventory

The inventory block is checked exactly by
`crates/mizar-kernel/tests/lint_policy.rs`.

<!-- public-enum-inventory:start -->
```text
certificate_parser::CertificateHashInputAlgorithm
certificate_parser::CertificateRejectionDetail
certificate_parser::ClauseRefNamespace
certificate_parser::ClauseTautologyPolicy
certificate_parser::FailureCategory
certificate_parser::FinalGoalNamespace
certificate_parser::RequiredProofStatus
certificate_parser::SectionTag
checker::AcceptedProofStatus
checker::BaseFactNamespace
checker::CheckedFactRef
checker::ClusterTraceContextError
checker::ImportedFactContextError
checker::ImportedFactNamespace
checker::KernelCheckStatus
clause::ClauseError
clause::ClauseForm
clause::Polarity
clause::SymbolKind
clause::TautologyPolicy
clause::Term
formula_evidence::Formula
formula_evidence::FormulaEvidenceError
formula_evidence::FormulaSource
formula_evidence::FormulaSourceClass
formula_evidence::GoalPolarity
rejection::ClauseRefNamespace
rejection::RejectionCategory
rejection::RejectionDetail
rejection::RejectionRecordError
resolution_trace::ImportedClauseContextError
sat_checker::SatCheckResult
substitution_checker::SubstitutionContextError
```
<!-- public-enum-inventory:end -->
