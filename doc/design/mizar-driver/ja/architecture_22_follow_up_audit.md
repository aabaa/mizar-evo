# Architecture-22 フォローアップ監査

> 正本は英語です。英語版:
> [../en/architecture_22_follow_up_audit.md](../en/architecture_22_follow_up_audit.md)。

状態: task D-020 で完了。

## 範囲

この監査は、D-016 determinism suite、D-018 source/spec 対応監査、D-019 二言語
documentation sync 監査の後で、実装済みの `mizar-driver` seam を
[architecture 22](../../architecture/ja/22.incremental_verification_contract.md) に
照合し直す。

焦点となる契約:

- driver-owned query boundary と cache-key purity;
- snapshot-scoped result freshness と obsolete-output suppression;
- message text identity ではなく `mizar-diagnostics` record による diagnostics
  publication;
- cache / proof authority が driver の外に残ること;
- real owner seam が current artifact boundary record を供給しない限り、
  artifact-publication serialization が driver の外に残ること;
- 実装済み seam に対する、worker-count control をまたいだ deterministic local event、
  CLI、diagnostics-gap、watch replay behavior。

この監査は、real semantic、proof、cache、artifact producer に対する full clean /
incremental / parallel equivalence を主張しない。それらの性質には、下で分類済みの
owner seam が必要である。

## 結果

- 実装済み driver seam に対する未解決の blocking/high architecture-22 drift は
  見つからなかった。
- driver は snapshot-scoped publication を維持している。obsolete watch/LSP session は
  current diagnostics や artifact-boundary event を publish できない。
- phase registry は driver-owned query boundary のままである。純粋な
  `PhaseService::cache_key` projection を観測するが、cache compatibility、proof reuse、
  artifact freshness decision は所有しない。
- diagnostics は `mizar-diagnostics` record / index または明示的な owner-gap event として
  運ばれる。message text は diagnostic identity として扱わない。
- artifact publication は non-driver authority のままである。driver は producing owner が
  current record を供給した場合だけ protocol-agnostic artifact-boundary event を運べるが、
  publication token を発行せず artifact を serialize しない。
- D-016 は実装済み seam の crate-local determinism coverage を提供している。real
  producer / cache / artifact / proof reuse を伴う full clean / incremental / parallel
  equivalence は、対応する owner seam が存在するまで `deferred` のままである。
- D-020 audit document は英語と日本語で paired であり、二言語 sync 監査はこの file pair を
  含める形で更新済みである。

## Architecture-22 契約 matrix

| 契約点 | Driver status | Evidence | Remaining classification |
|---|---|---|---|
| Clean-build equivalence | driver-local projection のみ実装済み。request/session identity、event ordering、CLI output、diagnostics owner-gap output、unavailable-owner output、watch replay suppression は repeated run と worker control をまたいで deterministic である。 | `tests/determinism.rs`; `todo.md` と `00.crate_plan.md` の D-016 record。 | real cache、producer、artifact、proof、multi-task dispatch owner seam が存在するまで、full real clean / incremental / parallel equivalence は `DRIVER-G-007` の下で `deferred`。 |
| Cache-miss fallback と cache authority | `registry.rs` は phase service に純粋な cache-key intent を問い合わせ、cache observation を報告する。cache compatibility を決定せず、cache hit を proof authority に昇格しない。 | `tests/registry.rs` の cache-key purity / query-boundary test; `tests/lint_policy.rs`、`tests/driver.rs`、`tests/cli.rs` の owner-boundary source guard。 | real cache lookup / reuse validation は driver の外に残る。semantic/proof/artifact adapter の不足は `DRIVER-G-013` の `external_dependency_gap`。 |
| Snapshot-scoped results | request/session の lane と generation metadata が currentness を決める。event は session/snapshot identity を validate し、obsolete publication を suppress する。watch replay は superseded session を suppressed publication に変える。 | `tests/request.rs`、`tests/events.rs`、`tests/watch.rs`、`tests/driver.rs`、`tests/determinism.rs`。 | 実装済み driver seam に新しい gap はない。real LSP bridge publication は `DRIVER-G-012` の `external_dependency_gap` / `deferred` のまま。 |
| Query boundary | `PhaseRegistry`、`DriverQueryBoundary`、`DriverQueryDatabase` が driver-local salsa / query-compatible boundary を所有する。syntax/parser/semantic phase crate は driver dependency や query-engine dependency を得ない。 | `tests/registry.rs` の query-boundary coverage と `tests/lint_policy.rs` の dependency guard。 | 実装済み driver seam に新しい gap はない。non-phase-zero work の scheduler-selected real phase dispatch は `DRIVER-G-011` の `external_dependency_gap`。 |
| Diagnostics freshness と identity | driver event は current session に対してのみ diagnostics owner record を運ぶ。CLI は利用可能なら `mizar-diagnostics` を通じて record を render し、未準備なら diagnostics bridge gap を明示する。message text は diagnostic identity key ではない。 | `tests/events.rs`、`tests/cli.rs`、`tests/determinism.rs`; `source_spec_correspondence.md` の behavior table。 | real frontend / module-index to diagnostic bridge gap は `DRIVER-G-010` と `DRIVER-G-013` の `external_dependency_gap` のまま。 |
| Artifact publication boundary | driver は artifact を completion order で commit せず、artifact を serialize せず、publication token を作らない。artifact-boundary event は current session identity と owner-provided record を要求する。 | fake artifact output を防ぐ `tests/events.rs`、`tests/watch.rs`、`tests/driver.rs`、`tests/cli.rs`、`tests/determinism.rs` の guard。 | artifact publication token と full phase-15 producer emission は `DRIVER-G-005` / `DRIVER-G-013` の `external_dependency_gap`; 欠落する `mizar-artifact` closeout は `DRIVER-G-001` の report-only `repo_metadata_conflict`。 |
| Parallel compatibility | 実装済み driver projection は deterministic order key で event を sort し、`--jobs` control をまたいで CLI output を安定させる。scheduler semantics は `mizar-build` から消費し、driver は scheduler readiness や completion-order semantics を複製しない。 | `tests/events.rs`、`tests/driver.rs`、`tests/determinism.rs`; D-016 の隣接 `mizar-build` verification。 | non-phase-zero work に対する real scheduler callback dispatch は `DRIVER-G-011` の `external_dependency_gap`; full worker-race equivalence は `DRIVER-G-007` の下で deferred。 |
| LSP freshness | driver request/session/event semantics は LSP lane currentness と obsolete-publication suppression を含むが、event payload は protocol-agnostic のままである。 | `tests/request.rs`、`tests/events.rs`、`tests/watch.rs`、`tests/determinism.rs`; D-018 source/spec audit。 | real LSP protocol bridge は `DRIVER-G-012` の下で driver の外に残る。 |

## Follow-up record

この監査は新しい blocking/high task を導入しない。

既存 record が有効な follow-up のままである:

- `DRIVER-G-005`: artifact publication token と full producer emission は
  `external_dependency_gap`。
- `DRIVER-G-007`: real cache、producer、artifact、proof、worker-race seam を伴う full
  clean / incremental / parallel equivalence は `deferred`。
- `DRIVER-G-010`: frontend canonical producer payload と diagnostics bridge readiness は
  `external_dependency_gap`。
- `DRIVER-G-011`: scheduler-selected real phase dispatch callback は
  `external_dependency_gap`。
- `DRIVER-G-012`: real file-watcher / coalescing owner と LSP bridge は
  `external_dependency_gap` / `deferred`。
- `DRIVER-G-013`: semantic/proof/artifact phase adapter は `external_dependency_gap`。
- `DRIVER-G-001` と `DRIVER-G-009`: artifact metadata conflict は、この task stream では
  report-only `repo_metadata_conflict` のまま。

## 検証

D-020 は documentation-only である。必要な local check:

- `git diff --check`
- task 関連 path を stage した後の `git diff --cached --check`

review が source change を要求しない限り、この task に Rust verification は不要である。
監査証跡は、上で名前を挙げた実装済み crate-local test に依存する。final crate closeout では
full workspace verification suite を実行する。
