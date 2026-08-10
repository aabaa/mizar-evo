# module 境界リファクタリング gate

> 正本は英語です。英語版:
> [../en/module_boundary_refactor.md](../en/module_boundary_refactor.md)。

状態: task R-029 gate complete。

## 範囲

この gate は、task R-001〜R-028 後の `mizar-resolve` source layout を監査する。
oversized file、混在した責務、resolver module table と module spec 境界に沿って
分割すべき private helper を確認する。

この refactor は behavior-preserving である。public module name、public type、public
method、diagnostic payload ordering、deterministic debug rendering text、`.miz` test、
expectation sidecar、traceability metadata は移動または変更しない。

## 結果

- public module path は `mizar_resolve::{declarations, env, imports, labels,
  module_index, names, resolved_ast, symbols}` のままである。
- review bottleneck になっていた inline unit-test block を、module ごとの private
  `tests.rs` file へ移した。
- private renderer / validation / diagnostic helper block は、独立した review surface に
  なっていた箇所だけ private submodule へ移した:
  `env/snapshot.rs`、`resolved_ast/snapshot.rs`、
  `resolved_ast/validation.rs`、`names/diagnostics.rs`。
- public API、behavior contract、crate responsibility boundary、artifact schema は
  変更していない。
- 移動した API について source/spec 対応と二言語ドキュメント同期の scope を再実行し、
  新しい drift は見つからなかった。

## source layout

| public module | public source | R-029 後の private helper / test | gate result |
|---|---|---|---|
| `declarations` | `src/declarations.rs` | `src/declarations/tests.rs` | public declaration-shell API は module root に残し、test を implementation body から移した。 |
| `env` | `src/env.rs` | `src/env/snapshot.rs`, `src/env/tests.rs` | `SymbolEnv` と index API は module root に残し、deterministic snapshot rendering を private helper module へ移した。 |
| `imports` | `src/imports.rs` | `src/imports/tests.rs` | import path / graph API は module root に残し、test を implementation body から移した。 |
| `labels` | `src/labels.rs` | `src/labels/tests.rs` | label projection / resolution API は module root に残し、test を implementation body から移した。 |
| `module_index` | `src/module_index.rs` | `src/module_index/tests.rs` | resolver-side module-index seam は module root に残し、test を implementation body から移した。 |
| `names` | `src/names.rs` | `src/names/diagnostics.rs`, `src/names/tests.rs` | namespace / name / dot-chain API は module root に残し、crate-local internal diagnostic assembly を private helper module へ移した。 |
| `resolved_ast` | `src/resolved_ast.rs` | `src/resolved_ast/snapshot.rs`, `src/resolved_ast/validation.rs`, `src/resolved_ast/tests.rs` | Resolved AST data shape は module root に残し、deterministic snapshot rendering と validation helper を private module へ移した。 |
| `symbols` | `src/symbols.rs` | `src/symbols/tests.rs` | symbol / signature API は module root に残し、test を implementation body から移した。 |
| private recovery policy | `src/recovery.rs` | none | すでに小さく private なので分割不要。 |

## 再実行した監査

- source/spec 対応: public API source root は引き続き module spec と対応している。
  helper を移動した行は [source_spec_correspondence.md](./source_spec_correspondence.md) で
  private helper path も参照する。
- 二言語ドキュメント同期: この file を両言語 directory に追加し、[todo.md](./todo.md)
  と crate plan の task / status wording は R-029 完了として同期した。
- 境界規律: 分割は `mizar-resolve` 内に閉じている。parser、syntax、frontend、
  build、checker、proof、diagnostics registry、driver、artifact の責務は追加しない。

## verification

この gate は refactor 後に通常の resolver verification を要求する:

```text
cargo fmt --check
cargo test -p mizar-resolve
cargo clippy -p mizar-resolve --all-targets --all-features -- -D warnings
```

crate-wide close-out では full workspace と `mizar-test` plan gate を実行する。

## 計画済み R-032A / R-032B ownership recheck

R-032 は完了済み R-029 refactor gate を再オープンしない。既存 module owner に
split し、public module layout を変えない。

- R-032A: `src/resolved_ast.rs` / `src/resolved_ast/tests.rs` /
  sole R-026 `SurfaceResolvedArenaError` owning-spec decision entry 用
  `tests/lint_policy.rs`。
- R-032B: `src/labels.rs` / `src/labels/tests.rs` / sole R-026
  `ProofLabelSourceCollectionError` / `labels.md` owning-spec decision用
  `tests/lint_policy.rs`。
- paired docs。新 module は計画しない。

R-032A は complete validated structural map と state/key mismatch を含む exact
public error table を所有する。R-032B exact `'a` impl は AST/arena borrow だけを
store、namespace/contribution を own、module を store せず `new` で validate、
`collect` で `resolved.module()` を使う。両 operation は exact public error
enum を返す。module-global ordinal と exact `proof-step-v1` identity は label owner。
exhaustive direct-edge table は default-deny で、unlisted/recovered/malformed/
wrapped edge は syntax/semantic traversal を漏らさず row/ordinal を出さない。
upper boundary は exact `Root` -> `CompilationUnit` -> `ItemList` で、direct
item-list child 以外の theorem は unreachable。
callback、unmapped side channel、fabricated id、unchecked conversion、panic はない。
R-032A node origin `[surface_id]` と R-032B richer table-origin path は意図的に
異なり、それぞれ validate する。

parser/frontend production、Cargo/workspace metadata、他 resolver module、public
checker handoff、checker/type/proof/Core/CFG/VC responsibility は除外する。
implementation pressure が別 source owner、public boundary、mapping owner を必要と
する場合は、変更を広げず frozen R-032A/R-032B contract を停止・再 review する。

R-032A preflight はこの stop rule を正しく発火した。dense
`SurfaceNodeId`-bearing iteration は mizar-syntax S-026 が所有し別 commit で
land する。R-032A は frozen resolver files からその accessor を consume するだけ。
unsafe / dummy-AST id fabrication は禁止のまま。

R-032A implementation preflight は、two-Rust-file wording が mandatory
R-026 public-enum decision owner を欠いた時にも stop rule を発火した。この
欠落は High `design_drift` で semantic `spec_gap` はない。existing enum policy
と source/spec correspondence が exact `tests/lint_policy.rs` entry をすでに
authorizeする。したがって別同期docs-only correctionはimplementationをexact
Rust 3 filesへfreezeし、他lint/module-layout changeはauthorizeしない。

## R-032A implementation result

R-032Aは上記でauthorizeしたexact `src/resolved_ast.rs`、private
`src/resolved_ast/tests.rs`、sole `tests/lint_policy.rs` owning-spec
decision entryだけを使用した。existing `resolved_ast` public moduleがownerの
ままで、module split、ownership transfer、callback、parallel map、
syntax/checker/runner source changeはない。R-032A完了時にR-032B streamは
existing `labels` ownerの次workとして残り、下記separate lint-policy
correctionはcommit済み。

## R-032B lint-policy frozen-scope correction（completed prerequisite record）

R-032B implementation inventoryでもstop ruleが発火した。frozen public
`ProofLabelSourceCollectionError`はR-026 guardに必ずscanされる。omitted
policy ownerはHigh `design_drift`であり、semantic `spec_gap`、`test_gap`、
test-intent changeではない。later implementation ownerはexact
`src/labels.rs`、`src/labels/tests.rs`、`tests/lint_policy.rs`で、last fileは
sole `ProofLabelSourceCollectionError` owning-spec decision
`spec_name: "labels.md"`だけを受けられる。module split / ownership transferは
authorizeしない。

completed synchronized docs-only correctionはexact 31 design files、
resolver 16、checker 8、`mizar-test` 6、global ledger 1を対象にする。source、
specification、fixture、sidecar、expectation、trace status/count、Cargo
metadata、semantic contract、test intentは変更しない。
`spec_coverage_audit.md`はdeliberate no-op。independent specification、
test/scope、source/documentation consistency reviewはすべて**NO FINDINGS**で、
docs-only verification/count/hash gateはPASS。independent final read-only
qualityも**NO FINDINGS**で、全9 hard gates PASS、capなし、valid
`100/100`（`20/20/15/15/10/10/5/5`）。そのpre-commit record時点では
task-only staging/cached-diff review、commit、post-commit
invariant/fresh-inventory gateだけがpendingだった。これらはcorrection commit
`f1cf0a5d15f2db51176e9e91a4f5a6447a88ad7a`とそのfresh inventoryで後に完了。

## R-032B implementation result

R-032Bはexisting `labels` owner内に完全に留まる。commit
`b3a7e79a6b60db2974e911c69bb56ff5f4609064`はexact `src/labels.rs`、
private `src/labels/tests.rs`、上でauthorizeしたsole
`tests/lint_policy.rs` decisionを変更する。module split、ownership transfer、
callback、unmapped side channel、fabricated id、syntax/checker/runner source、
active artifact、semantic ownerを追加しない。

collectorはR-032A arenaをconsumeし、label projection、reference candidate、
scope/ordinal/completion、resolver provenance derivationだけを所有する。
historical private `mizar-test` B5C routeがproduction consumerで、public
checker unresolved-reference handoffは除外されたまま。Medium third-childと
unauthorized `Default` / `From` implementation findings、および全
test-sufficiency findingはfixed。final fresh test-sufficiency、
implementation、source/documentation rereviewは**NO FINDINGS**で、全
pre-quality verification gateはPASS。independent final qualityも
**NO FINDINGS**、全9 hard gates PASS、score capなし、valid `100/100`
（`20/20/15/15/10/10/5/5`）。task-only restaging/cached-diff review、
commit、post-commit invariant/fresh inventoryはcomplete。

## Checker Task 258B5C boundary status

historical B5C source/test deltaはprivate `mizar-test` consumer、exact fail
fixture 2件、sidecar 2件、covered trace row 2件、および
`crates/mizar-test/tests/metadata.rs`のfrozen active-count/CLI assertion 4件
（declaration stage `5`から`7`）だけ。unchanged
R-032A `SurfaceResolvedArena`とR-032B `ProofLabelSourceCollector` /
`LabelResolver` APIをconsumeし、resolver production/APIは変更しない。
plan/pass/failは`421/389`と`228/193`、active
parse/declaration/type/proofは`101/7/198/1`、warnings/errorsは`23/0`。

public codeは空のままで、private key
`declaration_symbol.label.proof_scope_confinement`だけがrouteをauthenticate
する。confinement negative 2件はR-G007のこのsliceだけを閉じ、
import/name/dot-chain/other label-reference workはopenのまま。B5Cのtest、
implementation、source/documentation reviewと全verification gateは完了し、
independent final qualityは**NO FINDINGS**、全9 hard gates PASS、score
capなし、valid `100/100`。task-only cached-diff review、dedicated commit
`33ac57e96f048dc40559565f54369cac854409a7`、post-commit fresh inventoryは
このhistorical checkpointで完了済み。

## Checker Task 263R frozen boundary

prerequisiteとlater repairはexisting `symbols` owner内に留まる。implementationが
edit可能なのは`src/symbols.rs`とprivate `src/symbols/tests.rs`だけ。owner
discriminatorはduplicate classificationだけが使うinternal declaration-shell identityで、
`SymbolId`、`DefinitionShell`、`SignatureShell`、`SymbolEnv`、module summary、
public APIへ追加しない。module split、dependency edge、resolver/checker ownership
transfer、lint-policy decision、fixture、runner、Cargo変更は禁止。Task 263はdedicated
lower commitとfresh inventory後だけcorrected resolver resultをconsumeする。

## Checker Task 263R implemented boundary

implementationはfrozen existing `symbols` owner内に留まり、exact
`src/symbols.rs`とprivate `src/symbols/tests.rs`だけを変更する。new selector-owner
field、parent walk、conflict-key componentはprivateである。module split、public
surface変更、dependency edge、lint-policy変更、resolver/checker ownership transfer、
runner route、corpus artifact、trace metadata、Cargo変更はない。本lower commitと
fresh inventory後もTask 263がsole future production consumerである。

final boundary/consistency reviewは**NO FINDINGS**、全9 quality gateはcapなし
`100/100`でPASS。exact staging/commitが残る。

## Checker Task 264R frozen boundary

lower correctionは既存module内に留める。enum/mapping/shell testは
`declarations.rs` / `declarations/tests.rs`、no-projection、stable sibling/anchor
fingerprint、append-only code/key、symbol testは`symbols.rs` / `symbols/tests.rs`。
new module/dependency/Cargo target/public semantic identity/diagnostic classは追加しない。
public ABI deltaはnon-exhaustive append-only shell variantだけ。Checker Task 248P/264は
separate consumerのままで、module split/line-count threshold decisionを変えない。

## Checker Task 264R implemented boundary

implementationは既存`declarations`/`symbols` ownerとprivate test moduleのexact 4 Rust
files内に留まり、新規module/dependencyはなく、public deltaはfrozen append-only shell
variantだけである。checker/runner/corpus/trace/Cargo/lint inventory/module-split policyは不変。

## Resolver Task 277R1 module boundary

implementation は既存 public `names` owner と private `names/tests.rs` 内に留まる。
`SurfaceResolvedArena` は sole resolver identity authority のまま。collector はこれを
validate し `resolved_node_for` を呼ぶが、`resolved_ast.rs` を変更せず ID を construct しない。
module split、dependency/Cargo change、`SymbolId`、`NameRef`、`ResolvedAst` field、public
diagnostic/error enum、checker handoff、resolver-to-checker ownership transfer はない。mizar-test
2 paths は test-only consumer で、production runner と全 active route は boundary 外である。

final source scopeはresolver 2 filesとmizar-test test-only 2 pathsのexact 4。module、
dependency、Cargo、lint-policy、production route、ownership boundaryは変更しない。

## Resolver Task 277R2 module boundary

[Task 277R2](../../task_contracts/ja/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md) のproduction
ownerは既存`names.rs`、resolver regressionはprivate `names/tests.rs`、public-enum/documentation
policy adjustmentはresolver `tests/lint_policy.rs`に限定する。`SurfaceResolvedArena`はsole
identity authorityのままで、`resolved_ast.rs`/other resolver moduleはeditしない。lower consumerは
mizar-test private leaf 1件とtest registrationだけ。

implemented source scopeはresolver 3 paths+mizar-test test-only 2 pathsのexact 5。module split、
Cargo/dependency、parser/frontend/checker/Core、production runner、public diagnostic/error、active
route、resolver-to-checker ownership transferはない。
independent boundary integration reviewは**NO FINDINGS**。
