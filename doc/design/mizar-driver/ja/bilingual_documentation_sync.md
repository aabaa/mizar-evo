# 二言語ドキュメント同期監査

> 正本は英語です。英語版:
> [../en/bilingual_documentation_sync.md](../en/bilingual_documentation_sync.md)。

状態: task D-019 で完了。task D-020 で architecture-22 follow-up audit の file pair を
含めて更新。

## 範囲

この監査は、`doc/design/mizar-driver/en/` 配下の英語正本 design document と、
`doc/design/mizar-driver/ja/` 配下の日本語 companion を比較する。

監査項目:

- すべての英語 driver design document に同名の日本語 companion があること;
- すべての日本語 driver design document に対応する英語正本があること;
- downstream engineering work に必要な粒度で、section structure、task record、
  gap classification、ownership boundary が同期していること;
- 新規 audit document が両言語で対になっていること。

localized heading や wording は、技術内容が同等であれば差分を許容する。この監査は
source behavior や language semantics を変更しない。

## 結果

- 未解決の blocking/high EN/JA documentation drift は見つからなかった。
- 現在の driver design corpus では、英語と日本語の file set はこの audit document を含め
  1 対 1 で対応している。
- D-020 までの task record、既知の `DRIVER-G-*` classification、
  `external_dependency_gap`、`deferred`、report-only `repo_metadata_conflict` record は
  両言語に存在する。
- D-018 source/spec correspondence audit は同期済みのままであり、未解決の blocking、
  high、medium source/spec drift がないことを引き続き報告している。
- D-020 architecture-22 follow-up audit は両言語で paired になり、実装済み driver seam に
  未解決の blocking/high drift がないことを報告している。

## ペア coverage

| 英語正本 file | 日本語 companion | 同期結果 |
|---|---|---|
| `00.crate_plan.md` | `00.crate_plan.md` | paired。Responsibility、preflight、gap table、D-020 までの task decomposition、exit criteria、既知の deferred/external gap は aligned。 |
| `todo.md` | `todo.md` | paired。Module ownership、prerequisite、ordered task、D-018 から D-020 の completion、verification note、non-owner boundary は aligned。 |
| `request.md` | `request.md` | paired。Request/session data model、currentness lane、snapshot capture、publication suppression、supersession、error handling、test、public enum policy は aligned。 |
| `registry.md` | `registry.md` | paired。Phase service table、readiness gap、registration rule、cache-key purity、salsa boundary、scheduler/cache seam、diagnostics/artifact/LSP boundary、test、public enum policy は aligned。 |
| `driver.md` | `driver.md` | paired。Driver front-door ownership、public API、submit flow、scheduler boundary、cancellation、artifact/diagnostics boundary、test、public enum policy は aligned。 |
| `events.md` | `events.md` | paired。Protocol-agnostic event shape、freshness/suppression、deterministic ordering、diagnostics/artifact event、consumer rule、test、public enum policy は aligned。 |
| `cli.md` | `cli.md` | paired。Batch command surface、request mapping、progress/diagnostics rendering、exit code、owner-gap handling、test、public enum policy は aligned。 |
| `frontend_adapter.md` | `frontend_adapter.md` | paired。D-006 `SourceFrontend` readiness inventory と `external_dependency_gap` decision は aligned。 |
| `source_spec_correspondence.md` | `source_spec_correspondence.md` | paired。D-018 public API、public method surface、promised behavior、gap record、docs-only verification path は aligned。 |
| `bilingual_documentation_sync.md` | `bilingual_documentation_sync.md` | この task で paired。 |
| `architecture_22_follow_up_audit.md` | `architecture_22_follow_up_audit.md` | D-020 で paired。Architecture-22 query-boundary、stale-output、diagnostics、artifact-publication、determinism classification は aligned。 |

## Drift と follow-up 記録

新しい blocking/high bilingual documentation drift は見つからなかった。

既存の分類済み record は変更しない:

- `DRIVER-G-001` と `DRIVER-G-009` は artifact metadata に関する report-only
  `repo_metadata_conflict` のまま。この task では `mizar-artifact` metadata を修復しない。
- `DRIVER-G-010` から `DRIVER-G-014` は frontend、scheduler dispatch、watch/LSP
  bridge、semantic/proof/artifact adapter、document extraction に対する現在の owner-seam
  `external_dependency_gap` または `deferred` record のまま。
- real producer/cache/artifact/proof seam を伴う full clean/incremental/parallel equivalence は、
  対応する owner seam が存在するまで deferred のまま。

## 検証

この audit と D-020 refresh は documentation-only である。必要な local check:

- `git diff --check`
- task 関連 path を stage した後の `git diff --cached --check`

後続 review が source change を要求しない限り、この task に Rust verification は不要である。
