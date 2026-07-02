# mizar-ir dispatch input

> 正本は英語です。英語版:
> [../en/dispatch_input.md](../en/dispatch_input.md)。

## 目的

この文書は、scheduler-selected phase dispatch が消費する `mizar-ir` 境界を
定義する。

`mizar-build` は ready task を選択し、dispatcher を呼ぶ。`mizar-driver` は
その callback を phase registry へ接続する。`mizar-ir` は、scheduler が task
を選択した後に real phase service が受け取ってよい不変の phase input identity
bundle と、seal 済み parent output handle を所有する。

この境界は、直前の build dispatch seam の IR 所有部分を閉じる。scheduling
semantics を `mizar-ir` へ移さず、phase semantics、type checking、proof
acceptance、cache compatibility decision、artifact publication token、LSP
protocol conversion も `mizar-ir` へ移さない。

## 所有境界

`mizar-ir` が所有するもの:

- phase registry query boundary が使う決定的な identity summary である
  `PhaseInputIdentities`。
- identities と seal 済み parent output handles を含む不変 dispatch input package
  である `PhaseDispatchInputBundle`。
- scheduler dispatch 前に snapshot/currentness validation を通過した、
  seal 済み `AnyPhaseOutputRef` の wrapper である `SealedParentOutputHandle`。
- downstream front door が scheduler-selected task type 向けに IR 所有 bundle を
  供給できる generic `PhaseDispatchInputProvider<Task>` trait。これにより
  `mizar-ir` は `mizar-build` や `mizar-driver` に依存しない。
- `PhaseOutputPublisher` 上の snapshot/current-output validation と、
  `IrStorageService` 上の seal 済み handle validation。
- dependency hash と parent output identity の canonical ordering。

対象外:

- ready task selection、dependency ordering、cancellation checkpoint、
  resource admission、cache-hit scheduling、result collation。
- phase semantics、module dependency、type-checking input、proof obligation、
  proof acceptance、trusted proof status の導出。
- `mizar-cache` `CacheKey`、dependency fingerprint、cache compatibility decision、
  proof-reuse validation の構築。
- producer payload schema、fake producer output、semantic adapter、proof adapter、
  artifact publication token、manifest commit、diagnostics rendering、LSP protocol
  payload。

## データモデル

```rust
pub struct PhaseInputIdentities {
    input_hash: Hash,
    dependency_hashes: Vec<Hash>,
    parent_output_hashes: Vec<Hash>,
}

pub struct SealedParentOutputHandle {
    handle: AnyPhaseOutputRef,
}

pub struct PhaseDispatchInputBundle {
    snapshot: BuildSnapshotId,
    identities: PhaseInputIdentities,
    parent_outputs: Vec<SealedParentOutputHandle>,
}

pub struct PhaseDispatchInputRequest<'a, Task: ?Sized> {
    task: &'a Task,
    snapshot: BuildSnapshotId,
}

pub trait PhaseDispatchInputProvider<Task: ?Sized> {
    fn dispatch_input_for_task(
        &self,
        request: PhaseDispatchInputRequest<'_, Task>,
    ) -> Result<Option<PhaseDispatchInputBundle>, DispatchInputError>;
}
```

`snapshot` は bundle を scheduler-selected dispatch snapshot に bind する。
downstream registry/front-door code は phase を実行する前に、返された bundle を
`PhaseDispatchInputRequest` の snapshot と照合しなければならない。これにより
provider が別 snapshot の parent handle を誤って返すことを防ぐ。

`input_hash` は、source text、選択済み module identity、typed signature、
正規化済み VC descriptor、toolchain/configuration summary など、parent 以外の
phase input に対する owner-supplied な安定 identity である。`mizar-ir` はこの
hash を記録し正準化するが、そこから意味論を導出しない。

`dependency_hashes` は、dependency artifact summary など、non-output dependency
について owner seam が供給する安定 hash である。dependency data が欠けている
場合、caller は owner gap として報告しなければならない。owning seam が実際に
依存なしを証明していない限り、空 vector で置き換えてはならない。

`parent_output_hashes` は、seal 済み parent output handle から `mizar-ir` が
導出する。これは `PhaseOutputId` の hash であり、caller が供給する生 hash では
ない。dependent task は対応する `SealedParentOutputHandle` を execution context
から受け取るため、real service は hash を payload と見なさず storage 経由で
parent output を参照できる。

`PhaseDispatchInputRequest` は、scheduler-selected downstream task と、scheduler
callback が選択した `BuildSnapshotId` を運ぶ。provider は snapshot を使って current
parent handle を validate または lookup してよいが、scheduler readiness、dependency
ordering、cache compatibility、phase semantics を再実行してはならない。bundle
欠落は `Ok(None)` で表す。sealed-handle、currentness、snapshot check の validation
failure は `Err(DispatchInputError)` で表すため、caller は invalid owner input と
owner bundle unavailable を区別できる。

## 検証

`SealedParentOutputHandle::from_current_output` は以下を検証する:

1. 供給された handle が publisher の storage service でまだ seal 済みである。
2. handle が dispatch snapshot に属する。
3. handle がその snapshot の current/package output として publish 済みである。

`SealedParentOutputHandle::from_validated_rehydrated_output` は以下を検証する:

1. 供給された handle が指定された storage service でまだ seal 済みである。
2. handle が dispatch snapshot に属する。

rehydrated constructor は、owner がすでに `mizar-cache` で検証し current snapshot
へ再水和した handle のためのもの。これは cache compatibility decision でも proof
authority でもない。別 snapshot の obsolete / stale handle は、まず cache adapter
で再水和するか、current dispatch input bundle から除外しなければならない。

すべての bundle constructor は parent handle が bundle snapshot に属することを
検証し、dependency hash を byte order で正準化し、parent handle を `PhaseOutputId`
hash で正準化する。`PhaseInputIdentities` の parent identity hash は canonical
parent handle list から導出される。同じ parent handle の重複は拒否する。phase input
graph は、重複 dependency edge を暗黙に潰してはならないからである。
`PhaseDispatchInputBundle` は `validate_snapshot` を公開し、scheduler/front-door code
が scheduler-selected dispatch snapshot と一致しない bundle を拒否できるようにする。

## gap 分類

| ID | Class | 根拠 | 対応 |
|---|---|---|---|
| DISPATCH-IR-G001 | `source_drift` / `boundary_violation` risk | この task の前は `mizar-driver` が `PhaseInputIdentities` を所有し、生の parent output hash を受け入れていた。 | identity bundle を `mizar-ir` へ移す。driver は front door として消費する。 |
| DISPATCH-IR-G002 | `external_dependency_gap` | real producer output payload と downstream semantic/proof adapter は未準備である。 | bundle は input identity と seal 済み parent handle だけを運ぶ。producer output や adapter を fabricate しない。 |
| DISPATCH-IR-G003 | `external_dependency_gap` | artifact publication token と manifest commit は `mizar-ir` の外に残る。 | publication token placeholder を追加しない。 |
| DISPATCH-IR-G004 | `deferred` | real semantic、proof、artifact、cache、LSP integration 上の full clean/incremental/parallel equivalence には、この task を超える owner seam が必要である。 | crate-local test と driver front-door test を追加し、system equivalence は defer する。 |

## テスト

この task は focused Rust coverage を追加する:

- dependency identity と parent identity の canonical ordering。
- 重複 parent handle の拒否。
- bundle / scheduler snapshot mismatch の拒否。
- current dispatch に対する wrong-snapshot parent handle の拒否。
- current または rehydrated dispatch 前の foreign-storage parent handle の拒否。
- obsolete parent handle を current/package dispatch input として使うことの拒否。
- seal 済みかつ検証済みで current-snapshot に再水和済みの handle を、
  cache/proof authority なしに受理すること。
- driver registry query fingerprint が driver-owned raw parent hash ではなく
  `mizar-ir` identity を消費すること。
- scheduler-selected driver dispatch が IR 所有 identity bundle と seal 済み
  parent handle を phase execution context へ渡すこと。
- source guard が、`mizar-build` が IR authority を得ず、`mizar-ir` が driver、
  diagnostics、artifact-token、proof、cache compatibility、semantic adapter、
  proof adapter、LSP authority を得ないことを示すこと。

## 公開 enum policy

`DispatchInputError` は downstream crate 向けに `#[non_exhaustive]` とする。
将来の fail-closed dispatch-input validation error を、外部の exhaustive match を
壊さずに追加できるようにするためである。この module に意図的な exhaustive
public-enum 例外はない。
