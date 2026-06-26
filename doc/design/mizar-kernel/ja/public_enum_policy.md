# Public Enum Policy

> 正本: [../en/public_enum_policy.md](../en/public_enum_policy.md)。

## 目的

この文書は `mizar-kernel` の public enum に対する task 19 の正規分類である。
phase 14 の public API surface が存在した後で、trusted kernel crate に
public-enum forward-compatibility procedure を適用する。

## 方針

`mizar-kernel` のすべての public enum は forward-compatible API surface
であり、`pub enum` の直前に `#[non_exhaustive]` を付けなければならない。
task 19 では、正当化された exhaustive exception は記録しない。

public enum を追加する task は、下の inventory と対応する英語正本の
inventory を更新し、crate-local lint guard を green に保つ。public enum の
削除、または variant の意味変更は、同じ task で source/documentation
consistency review を要求する。

`RejectionCategory` と `RejectionDetail` は追加で
[rejection.md](./rejection.md) と
[architecture 19](../../architecture/ja/19.failure_semantics.md) に従う:
stable-key の spelling、meaning、phase ownership、ordering、removal、
rename、category remapping の変更には compatibility review が必要である。
rejection detail の追加は、owning phase が文書化され、category、detail key、
deterministic location を assert するテストがある場合にだけ許可される。

## Gap Classification

- `design_drift`: 先行 module work は複数の public enum に
  `#[non_exhaustive]` を確立していたが、crate-wide inventory はなかった。
- `source_drift`: initial rejection baseline 後に追加された public enum に、
  task 19 前は明示的な forward-compatibility marker が欠けていた。
- `deferred`: public kernel enum を match する downstream crate は wildcard
  arm を使う必要がある。downstream compatibility checks はこの crate の外側に残る。

## Inventory

この inventory block は `crates/mizar-kernel/tests/lint_policy.rs` により
exact に検査される。

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
