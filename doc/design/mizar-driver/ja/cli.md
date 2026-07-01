# mizar-driver CLI Surface

> 正本は英語です。英語版: [../en/cli.md](../en/cli.md)。

状態: task D-012 で仕様化。source implementation は task D-013 で完了。

## 目的

`cli` module は `mizar-driver` の batch command-line entry point である。
command-line option と package / workspace file を protocol-agnostic な driver
request へ変換し、progress には driver event stream を購読し、diagnostics は
`mizar-diagnostics` 経由で render し、terminal driver outcome を stable process exit code
へ map する。

CLI は user interface である。manifest syntax、dependency resolution、phase semantics、
type checking、proof acceptance、cache compatibility、artifact serialization、artifact
publication token、LSP protocol conversion は所有しない。

## Command Surface

binary name は `mizar`。task D-013 が実装するのは batch build command のみである:

```text
mizar build [OPTIONS]
```

`mizar explain`、`mizar refine`、`mizar minimize`、`mizar semver-check` などの future
command は spec 23 に仕様があるが、後続 task が CLI module を明示的に拡張するまで D-013
の所有範囲ではない。

`mizar build` は次を受け付ける:

| Option | Meaning |
|---|---|
| `--workspace <path>` | workspace root。既定は current working directory。 |
| `--manifest-path <path>` | single-package build の package manifest path。異なる root を選ぶ `--workspace` とは同時指定不可。 |
| `--package <name>` | workspace build で build 対象 package を制限する。複数回指定可。 |
| `--profile <check|release>` | driver build profile。`release` は owner-provided verifier policy を要求し、trusted / kernel requirement を CLI が弱めてはならない。 |
| `--target <package-or-module>` | build target を制限する。CLI は target identity を記録し、解決は driver / build owner に任せる。 |
| `--jobs <n>` | scheduler input へ渡す worker count hint。1 未満は usage error。 |
| `--locked` | 既存 lock file を要求し、dependency resolution output を更新しない。 |
| `--no-incremental` | この invocation だけ incremental / cache reuse を無効化する。semantic behavior は変えない。 |
| `--message-format <human|json>` | human text または JSON Lines command output。既定は `human`。 |
| `--quiet` | progress event を抑制する。ただし diagnostics と final outcome は抑制しない。 |

source implementation 前にこの文書へ追加された場合に限り、小さな alias を追加してよい。
未文書化 flag は usage error である。

## Request Mapping

`mizar build` は次を構築する:

- `BuildRequestOrigin::Batch` を持つ `BuildRequestDraft`。one-shot build では lane `0`、
  generation `0` を使う。ただし後続の multi-invocation batch mode が別途仕様化された場合を除く。
- plan request、package manifest、lock file、source layout、dependency artifact index、
  dependency overlay、VC descriptor、scheduler control、resource budget、cancellation policy
  について、`mizar-build` の owner API から得た `DriverSubmitInput`。
- request / session layer を通じた source / dependency / verifier snapshot input。CLI が
  ad hoc hash を作ってはならない。

CLI は session 作成に `CompilerDriver::submit` を呼ばなければならない。`BuildSession` を手で
構築したり、phase registry を迂回したり、scheduler submission を成功に見せるため phase service
を先に走らせたり、scheduler synthetic output を phase artifact に変換してはならない。

Manifest parsing と lockfile parsing は `mizar-build` へ委譲する。diagnostic record creation、
ordering、rendering、explanation は `mizar-diagnostics` へ委譲する。artifact writing は、存在する
場合に artifact owner seam へ委譲する。

## Progress And Diagnostics

Progress は `BuildEventStream` から render する:

- `SessionAccepted`、`SnapshotCaptured`、`PlanningReady`、`TaskProgress`、
  `PhaseServiceGap`、`DispatchGap`、`OwnerReadinessGap`、`PublicationSuppressed`、
  `SessionFinished` は progress / status line として表示してよい;
- event order は replay された event stream に従い、worker completion order から再計算してはならない;
- event text は presentation に限り、diagnostic identity になってはならない。

Diagnostics は `mizar-diagnostics` record、index、owner-provided batch からだけ render する。
driver / planning / scheduler record から diagnostics owner record への bridge が存在するまでは、
CLI は diagnostic id、code、severity、message identity を発明せず、classified
`external_dependency_gap` または structured owner-readiness gap を報告しなければならない。

Human output は progress と diagnostics を stderr に書く。machine output は stdout の JSON Lines とし、
`schema_version`、stable `kind`、owner-provided identity を持つ。JSON output でも CLI record と
LSP JSON-RPC payload は分離しなければならない。

## Exit Codes

CLI は terminal outcome を決定的に map する:

| Code | Name | Meaning |
|---|---|---|
| `0` | `Success` | driver session が `Succeeded` で finish し、error-severity diagnostic が残っていない。 |
| `1` | `BuildFailed` | language / build diagnostic が package を reject する、または session が `Failed` で finish した。 |
| `2` | `Usage` | flag 不正、workspace / manifest selection の衝突、numeric value 不正、driver request 受理前の unreadable command input。 |
| `3` | `UnavailableOwner` | missing phase service や scheduler-to-registry dispatch gap を含む `external_dependency_gap`、`deferred`、unavailable owner seam により request が block された。 |
| `4` | `Cancelled` | session が `Cancelled` で finish した、または publication 前に superseded された。 |
| `101` | `InternalError` | driver invariant failure または structured diagnostics に変換されなかった unexpected internal error。 |

Exit-code mapping は rendered message text ではなく、structured driver / session state と
owner diagnostic severity を見る。

## Gap Classification

| Gap | Classification | CLI disposition |
|---|---|---|
| planning / scheduler error 用 driver-to-diagnostics owner record bridge が未完成。 | `external_dependency_gap` | owner-readiness / gap status を報告する。diagnostic id を allocate したり fake diagnostic を render したりしない。 |
| filesystem-backed な `--manifest-path` selection と workspace / member discovery は D-013 library entry point では未完成。 | `external_dependency_gap` | `UnavailableOwner` で exit する。owner-provided batch input なしに manifest path が選択済みであるかのように扱わない。 |
| current single-package input を超える package / module target filtering は未完成。 | `external_dependency_gap` | supplied package と完全一致する package selection だけを受け入れる。それ以外は real target resolver が存在するまで `UnavailableOwner` で exit する。 |
| source layout と request snapshot input が一致しない。 | `external_dependency_gap` | driver submission 前に拒否し、captured snapshot の外側の work について current event を publish しない。 |
| real semantic / proof / artifact phase adapter が unavailable。 | `external_dependency_gap` / `deferred` | `UnavailableOwner` で exit する。build success を報告したり artifact を fabricate したりしない。 |
| real artifact publication token / manifest commit seam が unavailable。 | `external_dependency_gap` | artifact output path が committed されたと claim しない。 |
| LSP protocol conversion は CLI の外。 | out of scope | JSON-RPC、document URI、code action、progress token、LSP severity を emit しない。 |

## Testing Requirements

Task D-013 tests は次を cover しなければならない:

- argument parsing から batch request / profile / target / scheduler control への変換;
- success、failed diagnostics、unavailable owner gap、cancellation、usage error、internal error の
  stable exit-code mapping;
- replayed event order からの human / JSON progress rendering;
- `mizar-diagnostics` owner record 経由の diagnostics rendering、または bridge missing 時の明示的 gap;
- `src/cli.rs` に LSP protocol term、artifact publication token、artifact serialization、
  manifest commit call、committed output-path claim、artifact owner API、phase semantics、
  proof acceptance、cache compatibility decision、fake output ref が入らないこと。
