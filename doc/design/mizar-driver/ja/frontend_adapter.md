# SourceFrontend Adapter Readiness

> 正本は英語です。英語版:
> [../en/frontend_adapter.md](../en/frontend_adapter.md)。

状態: task D-006 は `external_dependency_gap` として分類した。この task では
adapter source を実装しない。

## 範囲

Task D-006 は、`mizar-driver` が `mizar-frontend` phase 1-3 の real
`SourceFrontend` `PhaseService` を登録できるかを確認した。

adapter は、次の owner seam がすべて存在する場合にだけ許可される:

- `mizar-frontend` が real source-to-syntax phase behavior を提供している;
- `mizar-ir` が、canonical payload bytes と stored type 用 decoder を含む real
  producer contract を通じて frontend payload を seal できる;
- `mizar-diagnostics` が、diagnostic message text を identity として使わずに、
  frontend diagnostics を validated `DiagnosticDraft` として受け取れる;
- driver が planner や source-loading authority を複製せず、build-plan work slice を
  frontend source request へ変換できる。

## Inventory Result

`mizar-frontend` は real な `Frontend<L, P, PS>::run` API を公開しており、
source、preprocessing、token、optional AST、diagnostic、frontend content-cache-key
data を含む `FrontendOutput<PS::Ast>` を返す。

`mizar-ir::publisher::PhaseOutputPublisher` も real だが、publish には
producer-supplied canonical payload bytes、typed decoder、side table、parent
handle、named input hash、explicit allowed work-unit context が必要である。現在の
frontend surface は、`FrontendOutput` 用の canonical serialization や stable producer
payload schema を定義していない。

`mizar-diagnostics::sink::DiagnosticSink` は already validated な `DiagnosticDraft` を
受け付ける。現在の frontend surface は、`FrontendDiagnostic` を diagnostic registry
code、structured detail field、freshness、draft input へ写像する driver-facing
conversion を提供していない。driver は frontend message text を diagnostic identity
として使ってはならない。

driver core と scheduler submission spec はまだ存在しないため、real adapter invocation
向けの安定した driver-owned build-plan slice から `SourceUnitRequest` への mapping も
まだない。

## Classification

| ID | Class | Disposition | Evidence | Required owner seam |
|---|---|---|---|---|
| DRIVER-G-010 | `design_drift` | `external_dependency_gap` | `SourceFrontend` には real in-memory frontend output があるが、canonical frontend producer payload、diagnostics-draft bridge、driver build-plan input mapping がない。 | Frontend / IR / diagnostics / driver integration が producer payload schema、diagnostic draft conversion、request mapping を定義する必要がある。 |

## Decision

D-006 では `SourceFrontend` adapter を追加しない。registry table は引き続き
`SourceFrontend` を `external_dependency_gap` と分類する。

不足する owner seam が存在するようになったら、将来の実装は少なくとも次を満たす
real adapter を追加できる:

- source、preprocessing、lexing、parsing、parser recovery を再実装せず
  `mizar-frontend` を呼び出す;
- owner-defined canonical frontend payload だけを `PhaseOutputPublisher` 経由で
  publish する;
- validated diagnostics だけを `DiagnosticSink` 経由で emit する;
- payload / diagnostic / publisher context が欠けている場合は、output handle を
  fabricate せず blocking adapter result として報告する;
- LSP conversion、cache compatibility decision、artifact publication は
  `mizar-driver` の外に残す。

## Verification

D-006 は documentation-only である。classification には diff check と review-only
agent を使う。後続変更で source code を追加しない限り Rust verification は不要である。
