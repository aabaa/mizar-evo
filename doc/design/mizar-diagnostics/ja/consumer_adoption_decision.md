# Consumer Adoption Decision

> Canonical language: English. English canonical:
> [../en/consumer_adoption_decision.md](../en/consumer_adoption_decision.md).

## 目的

この note は `mizar-diagnostics` task 16 の adoption decision を記録する。この
crate は diagnostic-code registry、structured failure records、producer sink、
deterministic aggregation、CLI rendering、structured fix suggestions、lazy
explanation handles を所有する。consumer migration は real owning-consumer seam
が存在する場合だけ許可される。

## 決定

task 16 は documentation-only である。この task では lexer、frontend、parser、
resolver、driver、LSP、build、artifact consumer を migrate しない。現在の
workspace には、placeholder adapter を発明したり owning crate から authority を
移したりせずに `mizar-diagnostics` record を受け取れる real consumer adoption
seam がまだ存在しない。

既存 diagnostic は、下の trigger が満たされるまで owning crate に残す。将来
adoption する場合、tool と consumer は `DiagnosticCode` と structured field を
key にしなければならない。message text は presentation のままである。

## Dispositions

| 領域 | 根拠 | disposition | 再検討 trigger |
|---|---|---|---|
| `mizar-resolve` | resolver closeout は R-G001 を記録している: public resolver diagnostic code ownership がなく、task R-013/R-015 は name/import/label diagnostics を crate-local/internal に保つ。 | `external_dependency_gap` / `deferred` | spec chapter 22 が resolver diagnostic code を割り当てる、または委譲し、`mizar-resolve` が corpus または `.miz` coverage を伴う real producer-side adoption task を定義する。 |
| lexer/frontend/parser diagnostics | これらの crate は既に crate-local diagnostics と merge ordering を公開しているが、`mizar-diagnostics` 用の cross-crate migration plan または consumer adoption seam は存在しない。 | `external_dependency_gap` / `deferred` | owning crate が既存 corpus expectations、source ranges、deterministic ordering を real sink/aggregation path で保存する migration task を開く。 |
| `mizar-lsp` | LSP diagnostic-publication spec と implementation tasks は未完了である。range conversion、document versions、overlay diagnostics、protocol shaping は `mizar-lsp` に属する。 | `external_dependency_gap` | `mizar-lsp` の snapshot publication と diagnostics conversion の tasks が land し、owned LSP projection として `BuildDiagnosticIndex` を消費する。 |
| `mizar-driver` | `crates/mizar-driver` は現在 scaffold を持ち、diagnostics に依存し得る。ただし driver request/session/event-stream/publication tasks は future work のままである。 | `external_dependency_gap` | `mizar-driver` が diagnostics に driver bridge を所有させずに、phase batches を collect できる real session orchestration と publication boundary を着地させる。 |
| artifact projection | `mizar-artifact` は artifact mutation、manifest publication、durable projected artifact schema を所有する。 | `external_dependency_gap` | artifact-facing emission が real producer outputs と artifact-owned projection task によって駆動される。 |

## Boundary Rules

- placeholder adapter、stub API、fake resolver adoption、provisional conversion
  layer を追加しない。
- `mizar-diagnostics` から `mizar-driver`、`mizar-lsp`、resolver、lexer、
  parser、frontend への dependency を追加しない。
- この crate が adopted に見えるようにするためだけに既存 diagnostic を migrate
  しない。
- 将来 migration する場合も、tool を diagnostic message text で key しない。
- LSP protocol conversion、driver session orchestration、artifact mutation、
  proof acceptance、kernel acceptance、phase success authority を
  `mizar-diagnostics` へ移さない。

## Verification

task 16 の verification は documentation-only である。

- workspace reverse-dependency lint は accidental な `mizar-diagnostics`
  consumer adoption を防ぐ guard のままである。
- source files が変わらない限り、この task では `git diff --check` と
  `git diff --cached --check` で十分である。
