# Module: registry

> 正本は英語です。英語版:
> [../en/registry.md](../en/registry.md)。

## 目的

この module は driver-owned な phase service registry と salsa query 境界を定義する。

`mizar-driver` は phase service を 1 つの deterministic な front door の背後で
組み立て、scheduler work を service call へ写像し、phase crate が salsa に直接
依存しないよう query/database 層を所有する。registry は実 owner seam だけを結線する。
phase adapter、producer output、artifact publication token、LSP bridge が未準備なら、
fake adapter、stub API、provisional token、一時的な仮配線を登録せず、
`external_dependency_gap` または `deferred` として記録する。

## 所有境界

`registry` が所有するもの:

- phase service の deterministic registration、lookup、duplicate-phase rejection;
- architecture phase 0 から 16 までの service table;
- registered service を呼び出す driver-owned salsa database と query adapter;
- `phase`、`cache_key`、`execute` を含む `PhaseService` 契約;
- protocol-agnostic な driver/service 境界としての `PhaseContext` と `PhaseResult`;
- service cache-key projection の純粋性検査;
- driver query outcome から `mizar-build` が消費できる scheduler/cache-seam input への
  変換。

`registry` が所有しないもの:

- source loading、parsing、name resolution、type checking、VC generation、proof
  acceptance、trusted status、kernel acceptance、ATP winner policy、artifact
  semantics;
- `mizar-cache` の cache compatibility decision、dependency-fingerprint construction、
  proof-reuse validation、cache-store lookup、cache promotion;
- `mizar-build` の task graph readiness、worker scheduling、resource admission、
  cancellation-state propagation、scheduler result collation;
- `mizar-ir` の sealed storage internals、IR identity assignment、artifact projection
  schema;
- `mizar-artifact` の manifest transaction、artifact serialization、publication-token
  issuance;
- `mizar-diagnostics` の diagnostic-code allocation、diagnostic identity、record
  aggregation、rendering、explanation/fix resolution;
- `mizar-lsp` の protocol conversion、document-version handling、editor command
  shaping、LSP diagnostic/code-action payload。

## Gap 分類

| Gap | 分類 | registry での扱い |
|---|---|---|
| phase 4-14 の real semantic phase adapter がまだ揃っていない。 | `external_dependency_gap` | owning crate が real service surface を公開するまで、その phase は未登録のままにする。 |
| real phase-15 producer output と artifact publication token が driver から利用できない。 | `external_dependency_gap` | 後続で owner-provided な artifact/proof seam だけを受け取る。publication authority を作らない。 |
| real LSP protocol bridge と event consumption は `mizar-lsp` が所有する。 | `external_dependency_gap` | registry output は protocol-agnostic に保つ。event/LSP conversion はこの module の外に置く。 |
| durable cache lookup と compatibility は `mizar-cache` が所有する。 | 後続の cache integration task が real owner API を結線するまでは `external_dependency_gap` | task 5 は driver query boundary を通じて cache-key intent だけを記録する。compatibility、lookup、proof-reuse decision は `mizar-cache` に残す。 |
| 要求された `mizar-artifact` closeout report がこの checkout に存在しない。 | `repo_metadata_conflict` | 報告のみ。driver task stream では artifact metadata を修復しない。 |
| deterministic registration と cache-key purity test のために test-only registry fixture service が必要。 | 許可される test fixture | fixture は tests 内に限定し、real phase adapter として export したり提示したりしてはならない。 |

## Task D-015 意味論アダプター readiness

Task D-015 は driver core、event stream、CLI batch entry point、watch
orchestration が存在する状態で、semantic / proof / artifact / doc phase
adapter を監査する。この task では production semantic adapter を登録しない。
real adapter は、owning crate が次の seam を同時に公開した場合だけ登録できる:

- scheduler work-unit identity と sealed parent output handle 上の
  driver-callable service input;
- synthetic payload なしで `mizar-ir` に格納できる canonical producer output;
- message-text identity ではなく `mizar-diagnostics` を通る diagnostic record
  または文書化された diagnostics bridge;
- proof、cache、artifact、LSP authority が owning crate に残ること。

| Service | D-015 readiness | 分類 |
|---|---|---|
| `ModuleResolver` | `mizar-resolve` は resolver-owned data shape と deterministic な internal diagnostics を公開しているが、public resolver diagnostic code と artifact-backed `ModuleSummary` reuse は未準備であり、driver service envelope や sealed producer payload は公開されていない。 | `external_dependency_gap` |
| `SemanticChecker` | `mizar-checker` は explicit checker-owned payload を公開しているが、source-derived payload extraction、accepted proof / artifact status integration、public diagnostic allocation、artifact emission / reuse、統一された driver service envelope は未準備。 | `external_dependency_gap` |
| `Elaborator` | `mizar-core` は explicit lowering / control-flow data と local diagnostics を公開しているが、source-to-checker extraction、具体的な downstream identity、artifact schema、public diagnostic allocation、downstream VC / kernel / proof / artifact consumer は未準備。 | `external_dependency_gap` |
| `VcService` | `mizar-vc` は explicit payload 上の untrusted VC candidate、handoff hash、reuse input を公開しているが、source-derived `proof_verification` runner、全 upstream payload family、downstream proof / cache / artifact consumer、accepted discharge path は未準備。 | `external_dependency_gap` |
| `AtpService` | `mizar-atp` は untrusted backend-neutral problem / candidate machinery と generic runner behavior を公開しているが、real backend adapter、backend output から kernel-owned formula / substitution candidate への extraction、proof-policy integration、cache reuse、witness publication は未準備。 | `external_dependency_gap` |
| `KernelService` | `mizar-kernel` は explicit normalized input に対する trusted evidence checking を公開しているが、source-derived certificate、ATP proof translation、cluster / reduction producer payload、service-envelope normalization、cancellation plumbing、downstream proof / cache / artifact consumer は未準備。 | `external_dependency_gap` |
| `ArtifactService` | `mizar-artifact` は artifact schema / store primitive を公開しているが、real producer projection と phase-15 emission はまだ external gap であり、driver には publication-token authority を作る権限がない。要求された closeout report の欠落は、gap table に記録した別の report-only `repo_metadata_conflict` である。 | `external_dependency_gap` |
| `DocExtractionService` | `mizar-doc` には design TODO があるが、workspace crate も実装済みの service-facing extraction owner surface も存在しない。 | `deferred` |

D-015 では real adapter を登録しないため、既存の registry tests が適切な coverage である:
missing service が owner gap を報告すること、test-only fixture が tests 内に留まること、
duplicate coverage が拒否されること、driver/query boundary が proof / cache / artifact /
LSP authority を移動しないことを検証している。adapter ごとの fixture test は、それぞれの
real adapter を登録する後続 task で追加しなければならない。

## Phase Service Table

registry は architecture phase または連続した phase group ごとに 1 つの service
descriptor を記録する。descriptor は 1 つ以上の phase を cover する。service 名が
異なっていても、coverage が重複すれば拒否する。各 service は、real owning crate が
adapter seam を公開するまで absent でよい。

| Service | Phases | Owner seam | Registry status |
|---|---:|---|---|
| `WorkspacePlanner` | 0 | `mizar-build` planner | Real bootstrap owner は存在する。driver task 8 が planner semantics を複製せず結線する。 |
| `SourceFrontend` | 1-3 | `mizar-frontend` | D-006 は adapter を `external_dependency_gap` として記録する。real adapter には将来の producer / diagnostic / input seam が必要。 |
| `ModuleResolver` | 4-5 | `mizar-resolve` | resolver が service surface を公開するまで `external_dependency_gap`。 |
| `SemanticChecker` | 6-8 | `mizar-checker` | checker service が real typed output を公開するまで `external_dependency_gap`。 |
| `Elaborator` | 9-10 | `mizar-core` | core/elaboration service が着地するまで `external_dependency_gap`。 |
| `VcService` | 11-12 | `mizar-vc` | VC generation と deterministic discharge service が着地するまで `external_dependency_gap`。 |
| `AtpService` | 13 | `mizar-atp` | `external_dependency_gap`。ATP policy と backend evidence は registry の外に残る。 |
| `KernelService` | 14 | `mizar-kernel`/`mizar-proof` | `external_dependency_gap`。kernel acceptance と proof-status projection は owner decision のまま。 |
| `ArtifactService` | 15 | `mizar-artifact` と producer seam | `external_dependency_gap`。registry は publication token を作らない。 |
| `DocExtractionService` | 16 | `mizar-doc` | documentation/extraction owner surface が着地するまで `deferred`。 |

Rust registry は concrete な phase key として
`mizar-build::task_graph::PipelinePhase` を使う。一部の `PipelinePhase` variant は
architecture subphase を意図的にまとめている。`Frontend` は lexing/parsing phases 2-3
を cover し、`BackendRun` は ATP dispatch の backend execution subphase である。
したがって registry の内部 rank は現在の `PipelinePhase` variant に対する deterministic
ordering であり、上の表の architecture phase number を置き換えるものではない。

registry は submitted task に対して missing-service diagnostic を公開してよいが、その
diagnostic は欠けている owner seam を識別しなければならない。phase を emulate したり、
output handle を fabricate したり、phase を complete と mark してはならない。

## データモデル

### PhaseService

registry-level trait は概念上の形である。task 5 は、利用可能な owner API に合う具体的な
Rust type parameter または object-safe adapter を選んでよい。

```rust
trait PhaseService {
    fn phase(&self) -> PhaseDescriptor;
    fn cache_key(&self, input: &PhaseInput, context: &PhaseCacheContext) -> PhaseCacheIntent;
    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext) -> PhaseResult;
}
```

`phase` は次を含む descriptor を返す:

- service name;
- cover する architecture phase または連続 phase range;
- cache と query identity に使う service schema/version;
- owning crate または adapter owner;
- `execute` から期待される output kind family。

descriptor は registration identity であり、semantic authority ではない。phase の意味は
architecture spec と owning phase crate から来る。

`cache_key` は、与えられた input identity と context identity field から cache-key
intent または owner-provided cache-key request への純粋な projection である。cache
lookup は行わず、cache compatibility も決定しない。real cache seam が結線された後、
driver-owned query boundary が canonical `CacheKey` と lookup result の構築/検証を
`mizar-cache` に依頼する。

`execute` は 1 つの immutable input について owner service を実行し、structured output
を返す。scheduler state を mutate したり、event を直接 publish したり、artifact を
write したり、LSP payload へ変換したり、proof/cache/artifact record を trusted status
へ昇格したりしてはならない。

### PhaseContext

```rust
struct PhaseContext {
    snapshot: BuildSnapshotId,
    work_unit: WorkUnit,
}

struct PhaseCacheContext {
    common: PhaseContext,
    input_identities: PhaseInputIdentities,
}

struct PhaseExecutionContext {
    common: PhaseContext,
    cancellation: Option<CancellationToken>,
    diagnostics: Option<DiagnosticSink>,
    output_publisher: Option<PhaseOutputPublisher>,
}
```

`PhaseContext` は immutable な共通 driver/service identity 境界である。
`PhaseCacheContext` は `cache_key` に渡される narrowed view であり、purity contract が
許す immutable identity data だけを含む。`PhaseExecutionContext` は `execute` に渡され、
execution に必要な owner handle を含む。各 field は owner seam への参照である:

- snapshot id は `request` と `mizar-session` から来る;
- session id と request generation は driver query/event/publication layer に残し、
  すべての `PhaseService` context から除外する;
- work-unit identity と cancellation は `mizar-build` から消費する;
- diagnostics は `mizar-diagnostics` producer sink と record を通る;
- output sealing は `mizar-ir` publisher/storage handle を通る。

task 5 では real phase adapter をまだ登録しないため、これらの handle は optional である。
real adapter は registry context が供給する owner handle を使うか、
`external_dependency_gap` を報告しなければならない。private sink、output store、
publication token で context を迂回してはならない。

`PhaseContext` は mutable scheduler internals、wall-clock time、worker id、event
subscriber、LSP protocol payload、artifact manifest transaction、owning crate から
提供されていない cache-store handle、cache lookup handle、publication token を公開しては
ならない。cache lookup は、`cache_key` が pure intent を返した後、`PhaseService` method
の外側にある driver-owned query adapter で行う。

### PhaseResult

```rust
struct PhaseResult {
    status: PhaseStatus,
    diagnostics: Vec<DiagnosticBatch>,
    output_refs: Vec<AnyPhaseOutputRef>,
    cache_observation: Option<PhaseCacheObservation>,
}
```

`PhaseStatus` は driver/scheduler status projection である:

- `Complete` は、その phase が dependent に必要な output をすべて生成したことを表す;
- `Recoverable` は syntax-only または development 機能は継続できるが、semantic
  acceptance と proof status は degraded metadata を使ってはならないことを表す;
- `Blocking` は対象 unit の dependent work だけを停止する;
- `Fatal` は global build state を信頼できないため session を停止する;
- `Cancelled` は current output を公開せず cooperative cancellation を記録する。

status は proof acceptance、trusted status、cache compatibility、artifact publication
ではない。diagnostics は structured record/batch であり、render 済み message text は
決して identity ではない。

## Registration Rules

registration は session の scheduler submission 前に builder を通じて行う。builder は
immutable な `PhaseRegistry` を生成する。

1. 各 service descriptor を normalize する。
2. first covered phase、covered phase span、service name、schema version で descriptor
   を sort する。
3. architecture phase 0 から 16 のいずれかで coverage が重複したら拒否する。
4. phase span が空、非連続、0 から 16 の範囲外、または service table と互換しない
   descriptor を拒否する。
5. 後続の spec update が ownership を変えない限り、この spec に記録された owning-crate
   seam と異なる phase owner を主張する descriptor を拒否する。
6. table を freeze し、lookup order が registration order、hash-map order、plugin
   discovery order に依存しないようにする。

phase lookup は registered service descriptor、または gap classification を持つ
missing-service error のどちらかを返す。missing service は synthetic phase output を
決して生成しない。

## Cache-Key Purity Contract

`PhaseService::cache_key` が依存してよいもの:

- `BuildSnapshotId` と immutable snapshot content identity;
- package、module、work-unit、phase、output-kind identity;
- source hash、dependency artifact hash、lockfile hash、toolchain identity、language
  edition、verifier policy/configuration hash、owner schema version;
- owner-provided dependency slice または fingerprint;
- deterministic phase input hash と sealed parent output identity。

依存してはならないもの:

- `BuildSessionId`、`BuildRequestId`、`BuildLaneId`、request generation、watch/LSP
  supersession state;
- scheduler worker count、worker id、ready-queue timing、cancellation timing、event
  subscriber state、wall-clock time、input identity に記録されていない random seed、
  environment variable、declared source/dependency/artifact identity の外側にある
  filesystem metadata;
- render 済み diagnostic text、CLI progress formatting、LSP document-version payload、
  JSON-RPC id、protocol conversion 後の editor range、code action;
- previous cache hit/miss timing、cache-store directory iteration order、artifact commit
  order。

task 5 implementation は purity harness を含めなければならない。等価な input と context
identity で `cache_key` を 2 回実行したら、同一の intent/hash data を返さなければならない。
禁止された runtime/session field だけを変えても cache-key intent は変わってはならない。
必要な content identity を変えた場合は、intent が変わるか、明示的な uncacheable/no-key
result を返さなければならない。harness は test-only fixture service を使ってよいが、
production code は fake phase adapter を出荷してはならない。

## Salsa Query Boundary

`mizar-driver` は orchestration に使う salsa database を所有する。phase crate は pure
service を公開し、salsa に直接依存してはならない。そうしないと driver/query-engine の
lifetime と invalidation policy が semantic owner に漏れる。

registry は 1 つの driver process について driver database を作成または借用する。各
`BuildSession` について、driver は次の immutable input query を install する:

- captured `BuildSnapshot` identity と snapshot input summary;
- phase registry descriptor table;
- `mizar-build` 由来の phase 0 `BuildPlan` data;
- `mizar-build` が供給する task graph/work-unit identity;
- 各 phase が必要とする sealed parent output handle と diagnostic batch;
- verifier configuration と build profile identity。

derived query は snapshot id、phase descriptor、work unit、service schema version、
input-handle hash、owner dependency fingerprint で key 付けされる。derived query は次を
計算してよい:

- service cache-key intent;
- 後続の `mizar-cache` owner seam が使う cache-key intent と query observation;
- 後続の cache lookup が miss した、または disabled の場合の phase execution result;
- task の scheduler-visible output reference と diagnostic batch。

cancellation は cached query identity の semantic input ではない。cancellation token は
safe checkpoint で execution を停止し `Cancelled` を返してよいが、完了済み semantic
output を変えてはならない。古い snapshot/session の obsolete query result は diagnostics
や cache validation のために retain され得るが、current publication の前には request
module の publication guard を通らなければならず、別 snapshot で reuse する前には
`mizar-cache` validation を通らなければならない。

## Scheduler And Cache Seams

registry/driver layer は work を `mizar-build` へ submit する。scheduler semantics を
複製しない。scheduler は readiness、dependency blocking、resource admission、
cancellation propagation、terminal task state、canonical collation を所有する。
registry が提供するもの:

- 各 task の phase span を満たせる service を説明する registered service descriptor;
- scheduler が task execution を要求した場合だけ service work を実行する query adapter;
- `mizar-cache` が validation した後、`mizar-build` cache seam が受け取る形の
  caller-supplied cache decision。

validated cache hit は execution skip であり、completed scheduler dependency である。
proof evidence、semantic acceptance、trusted status、artifact publication authority では
ない。

## Diagnostics, Artifacts, And LSP Boundaries

phase service は `mizar-diagnostics` producer sink を通じて diagnostics を emit してよい。
registry は structured record と producer identity を保たなければならず、message text を
identity として扱ってはならない。

phase service は `mizar-ir` handle を通じて output を seal してよい。registry は publish
後の raw mutable IR value を公開してはならず、`mizar-ir` lineage、projection、
cache-adapter authority を local に構築してはならない。

artifact commit は artifact/proof/producer owner への handoff のままである。real artifact
publication token または producer output が利用できない場合、registry は
`external_dependency_gap` を記録し、publication の手前で停止する。

LSP protocol conversion は `mizar-lsp` に残る。registry は protocol-agnostic な phase
result と diagnostic readiness を後続の event stream へ渡してよいが、LSP diagnostic、
code action、document edit、JSON-RPC response、editor publication decision を作っては
ならない。

## Implementation で必要なテスト

task 5 は focused test を追加しなければならない:

- input order に依存しない deterministic registration;
- architecture phase ごとの duplicate coverage rejection;
- real owner seam の欠落が synthetic output なしで classified gap として報告されること;
- test-only fixture service による `PhaseService::cache_key` purity;
- registry が driver-local salsa/query boundary を作成または所有し、service の
  `cache_key`/`execute` call が registry query adapter を通じて仲介されることを示す
  positive guard;
- registry を通じて syntax/parser/phase owner crate が driver または salsa dependency を
  得ていないことを確認する salsa/query-boundary source scan。scan 対象 owner set は
  `mizar-lexer`、`mizar-syntax`、`mizar-parser`、`mizar-frontend`、`mizar-resolve`、
  `mizar-checker`、`mizar-core`、`mizar-vc`、`mizar-atp`、`mizar-kernel`、
  `mizar-proof`、`mizar-artifact`、`mizar-doc`;
- registry が cache compatibility、proof acceptance、artifact publication token、LSP
  payload、scheduler readiness semantics を構築しないことを示す boundary guard。
