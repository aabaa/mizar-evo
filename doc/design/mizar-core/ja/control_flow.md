# Control-Flow Preparation

> 正本は英語です。英語版:
> [../en/control_flow.md](../en/control_flow.md)。

この文書は `mizar-core` が所有する phase 10 の `ControlFlowIr` 構築を仕様化する。
architecture 06 Step 6、architecture 07 Step 1、仕様 20 章を詳細化する。task 14 は
spec-only であり、Rust source は実装しない。

## スコープ

`ControlFlowIr` は phase 9 の `CoreIr` algorithm shell と、後続の VC / extraction
consumer の間にある algorithm 専用の射影である。これは次を所有する。

- 1 つの core algorithm に対する deterministic basic block と control-flow edge。
- parameter、result binder、statement local、site-local `Pick` binder 用の local binding metadata。
- flow diagnostic が使う statement-level assignment / initialization state。
- contract、assertion、loop invariant、decreasing term の CFG site への配置。
- ghost/runtime use classification と ghost-effect metadata。
- core statement、CFG block、loop site、local binding から `CoreSourceRef` への source map。

`ControlFlowIr` は次を所有しない。

- proof acceptance、theorem discharge、kernel checking。
- `VcId` assignment、`ObligationAnchor` construction、canonical VC fingerprint、VC-local
  dependency slice。これらは internal architecture 07 により `mizar-vc` の責務である。
- algorithm payload の source-to-checker extraction。
- code extraction や target-runtime lowering。
- public diagnostic code allocation。

したがって `mizar-vc` との境界は一方向である。`mizar-core` は source-mapped な
control-flow fact を構築・検証し、`mizar-vc` はそれを `CoreIr` と obligation seed と
一緒に消費して canonical `VcIr` を構築する。

## 分類

task 14 の分類:

- `design_drift`: architecture 07 の affected-module list は
  `mizar-vc/control_flow.md` module に触れているが、internal architecture 07 は
  elaboration と control-flow preparation を `mizar-core` に割り当てている。この文書は
  現在の crate 境界を記録する。`mizar-core` が `ControlFlowIr` を所有し、
  `mizar-vc` が VC generation を所有する。
- `design_drift`: architecture 07 は phase 11 が algorithm control-flow fact を準備すると
  も述べている。現在の crate 境界では、`mizar-core` の phase 10 がそれらの fact を準備し、
  `mizar-vc` の phase 11 は `VcId` 割り当てと `VcIr` 構築のときにそれを消費する。
- `external_dependency_gap`: full algorithm payload の source-to-checker extraction と
  parser task 32-34 の source-derived coverage はこの task の外に残る。upstream payload
  bridge ができるまで、実装 task は explicit core/Rust fixture を使う。
- `external_dependency_gap`: 仕様 20 章の snapshot / claim payload は task 13 の
  `CoreAlgorithmStmtKind` surface には存在しない。これらは silently drop してはならない。
  将来 checker-owned shell が explicit snapshot-site metadata を運ぶまで、phase 10 は lower しない。
- `external_dependency_gap`: `mizar-vc`、`mizar-kernel`、`mizar-proof` はこの crate
  task で downstream consumer として実装しない。それらの API を仮実装してはならない。
- `deferred`: Rust data structure、CFG construction、contract/ghost/termination
  attachment、diagnostics、obligation-seed handoff は task 15-18 に残す。

task 14 を止める `spec_gap` はない。20 章が言語挙動を定義し、design document が
所有権境界を定義している。

## 入力

control-flow preparation は valid phase-9 `CoreIr` subset を消費する。

- `CoreAlgorithm` row。
- algorithm の root `statements` から到達可能なすべての `CoreAlgorithmStmt` row。
  nested `If`、`While`、`Match` body を含む。
- statement value、condition、assertion、contract、invariant、decreasing measure が参照する
  lowered term / formula。
- phase 9 からの algorithm diagnostic と source map。
- `CoreIr` に既に存在する generated origin と obligation seed。新しい `VcId` は割り当てない。

constructor は raw `CoreIr` validation により、到達可能な nested statement の `owner` が
containing algorithm と一致していることを信頼してよい。この invariant が壊れている場合、
ownership を推測せず、structured diagnostic として失敗しなければならない。

## データモデル

実装は名前を詳細化してよいが、次の意味的 shape を保つ必要がある。

```rust
struct ControlFlowIr {
    algorithm: CoreAlgorithmId,
    item: CoreItemId,
    symbol: SymbolId,
    entry: BasicBlockId,
    blocks: ControlFlowBlockTable,
    locals: ControlFlowLocalTable,
    contexts: ControlFlowContextTable,
    contracts: ControlFlowContractSet,
    loops: ControlFlowLoopTable,
    exits: Vec<ControlFlowExit>,
    ghost_effects: ControlFlowGhostTable,
    termination: ControlFlowTerminationPlan,
    source_map: ControlFlowSourceMap,
    diagnostics: ControlFlowDiagnosticTable,
}
```

module-level output は、lowering に成功した algorithm ごとに 1 つの `ControlFlowIr` を持ち、
skipped/error algorithm 用の diagnostic record を持つ。`CoreAlgorithmStmtKind::Error` が運ぶ
phase 9 diagnostic は参照として保持し、phase 10 だけの diagnostic は control-flow diagnostic row
として表すため、builder は `CoreIr` を変更しない。

### basic block

```rust
struct ControlFlowBlock {
    id: BasicBlockId,
    algorithm: CoreAlgorithmId,
    statements: Vec<CoreAlgorithmStmtId>,
    terminator: ControlFlowTerminator,
    context_in: ProgramContextId,
    context_out: Vec<ProgramContextId>,
    reachable: Reachability,
    source: CoreSourceRef,
}

enum ControlFlowTerminator {
    Goto(BasicBlockId),
    Branch {
        condition: CoreFormulaId,
        then_block: BasicBlockId,
        else_block: BasicBlockId,
    },
    Switch {
        scrutinee: CoreTermId,
        arms: Vec<ControlFlowSwitchArm>,
        join: Option<BasicBlockId>,
    },
    Return(Option<CoreTermId>),
    Break { loop_id: LoopId, target: BasicBlockId },
    Continue { loop_id: LoopId, target: BasicBlockId },
    Unreachable,
    Error(ControlFlowDiagnosticId),
}
```

block は deterministic source traversal order で割り当てる。synthetic join、loop-header、
exit block は、それを導入する construct の直後に安定した local counter で割り当てる。
hash iteration を使ってはならない。

`statements` は terminator の前に block 内で効果を持つ original source-order
statement-shell id を保持する。`If`、`While`、`Match` のような structured statement は通常、
opaque block statement として残さず、terminator または synthetic loop/switch metadata になる。

### local

```rust
struct ControlFlowLocal {
    id: LocalId,
    algorithm: CoreAlgorithmId,
    binder: CoreBinder,
    kind: LocalKind,
    declaration: LocalDeclaration,
    mutability: LocalMutability,
    ghost: bool,
    initialized_at: Option<CoreAlgorithmStmtId>,
    source: CoreSourceRef,
}

enum LocalKind {
    Parameter,
    Result,
    Let,
    Pick { witness_ty: Option<CoreFormulaId> },
    HiddenLoopValue,
}

enum LocalDeclaration {
    Parameter,
    Result,
    Var,
    Const,
    GhostVar,
    GhostConst,
    PickRuntime,
    PickGhost,
    HiddenLoopValue,
    Unsupported(CoreVarRole),
}

enum LocalMutability {
    Immutable,
    Mutable,
    Unknown,
}
```

parameter は immutable である。`CoreAlgorithm.result` が存在する場合は result local を作る。
`Let` と `Pick` binder は statement order で local を作る。`Pick` local は、別の pick が同じ
witness type を持つ場合でも site-local であり、stable-choice generated origin を再利用しない。
hidden loop value は、後続の statement form が必要にする loop measure、range bound、
collection domain などの old-state value を表してよい。

`LocalDeclaration` は checker-owned declaration metadata だけから導出する。explicit fixture では
`CoreBinder.role` がその semantic role を運んでよく、statement の `ghost` flag は runtime / ghost
state の区別を補う。ただし phase 10 は `var` や `ghost const` のような source spelling を解析しては
ならない。local declaration が mutability と ghostness を識別しない場合、construction は mutable
runtime state を既定値にせず、元の role を `LocalDeclaration::Unsupported` として記録し、
mutability を `Unknown` にして、unsupported local-declaration diagnostic を出す。この diagnostic は
Task 15 が所有する structural construction diagnostic であり、Task 17 の後続 data-flow diagnostic
を置き換えるものではない。parameter と result local は、将来拡張で checker-owned metadata が
ghost-only と明示しない限り immutable runtime local である。

`CorePlace` への assignment は、checker-owned place metadata が利用できる場合それを通じて
解決する。source-to-checker extraction がより豊かな lvalue identity を提供するまでは、
explicit Rust fixture は canonical `CorePlace` key を使ってよい。CFG は source spelling から
aliasing を推測してはならない。

### context と assignment state

flow construction は、証明ではなく、block ごとの context summary を記録する。

- block entry / exit で definitely initialized な local。
- 代入された可能性がある mutable place。
- `break` / `continue` resolution 用の active loop stack。
- branch と match arm が導入する path condition。
- loop header / exit で利用できる active invariant fact。
- specification context にだけ見える ghost-only state。

```rust
struct ProgramContext {
    id: ProgramContextId,
    definitely_initialized: Vec<LocalId>,
    maybe_assigned: Vec<CorePlace>,
    available_facts: Vec<ContextFactId>,
    assignment_effects: Vec<AssignmentEffectId>,
    call_effects: Vec<CallSiteId>,
    path_conditions: Vec<CoreFormulaId>,
    active_invariants: Vec<CoreFormulaId>,
    loop_stack: Vec<LoopId>,
    ghost_visible: Vec<LocalId>,
}
```

すべての block は input context と、normal / exit output context を参照する。context row は
source-independent な semantic summary である。source link は、その context transition を生じさせた
block、statement placement、local、loop、exit record が保持する。
`available_facts` は entry `requires`、`let` / `const` initializer からの local equality、
checkpoint 後の asserted formula、loop fact、known-terminating call の `ensures` fact を記録する。
`assignment_effects` は後続の Hoare-style VC が必要にする old-state assignment transformer と
hidden value を記録する。`call_effects` は call-site row を参照し、phase 11 が source syntax から
call を再構築せずに unconditional fact と partial-call conditional metadata を区別できるようにする。

Task 15 は CFG construction に必要な最小 context を記録する。parameter は entry で definitely
initialized、result local は存在するが definitely initialized ではない。initializer を持つ `Let` と
すべての `Pick` local は successor context に追加し、`Assign` statement は place write を記録する。
branch / loop condition は対応する successor context の path condition として保持し、assertion、
call fact は checker-owned call payload が存在するまで empty または placement-only のままにする。
Task 16 は contract、ghost、assertion、invariant、termination metadata を取り付ける。
Task 17 はそれらの context の不正利用に対する flow diagnostic を追加する。

join point は definitely initialized set を交差し、path condition を symbolic に join する。
後続 task が精密な join を表現できない場合は、保守的な context を保持し、すべての path で保証されない
fact を主張せず、unsupported precision diagnostic を出す。

### contract と assertion

```rust
struct ControlFlowContractSet {
    requires: Vec<ContractSite>,
    ensures: Vec<ContractSite>,
    calls: Vec<CallSiteId>,
    assertions: Vec<AssertionSite>,
    loop_invariants: Vec<LoopInvariantSite>,
    decreasing: Vec<TerminationMeasureSite>,
}
```

`requires` clause は entry assumption と、後続 `mizar-vc` generation 用の caller-side
obligation になる。`ensures` clause はすべての `Return` terminator に付く。`Assert`
statement は checkpoint site になり、normal successor context の fact になる。
loop invariant は仕様 20.5.1 の要求通り、loop header、normal back edge、`break`、`continue`
exit に付く。

`ContractSite` row は kind (`Requires` または `Ensures`)、lowered formula、exact source、
entry または return placement を運ぶ。`AssertionSite` row は formula、source、
aggregate algorithm-contract payload または block / successor context を持つ statement checkpoint の
placement を運ぶ。`LoopInvariantSite` row は formula、source、aggregate algorithm-contract payload
または loop header / normal backedge / break exit / continue exit の placement を運ぶ。loop placement は
owning `LoopId` を持つ。`TerminationMeasureSite` row は term、source、algorithm header / loop header /
loop continue edge の placement を運ぶ。

`CoreContractSet.assertions` と `CoreContractSet.invariants` は aggregate `AlgorithmContract`
placement として保存する。これらは後続 obligation generation 用の metadata anchor であり、それだけで
unconditional successor fact を追加しない。statement `Assert` と loop `While` annotation が flow-local
fact と edge site を与える。

`CallSiteId` は flow-level call-site table への link である。現在の Task 16 surface では、
`CallSite` は明示 checker-owned call payload の statement と source だけを記録する。
resolved callee、argument term、instantiated `requires` / `ensures`、termination availability は、
checker payload がその data を公開するまで deferred である。phase 10 は call contract を spelling から
推測してはならず、明示 payload がなければ call-site table は empty のままにする。後続 task で
その payload が存在するときも、partial または unknown call の `ensures` は後続 `mizar-vc`
obligation generation 用の conditional metadata に留め、unconditional successor fact にしてはならない。
phase 11 が具体的な `VcId` を割り当てる。

task 14 は placement を仕様化する。task 16 は call 以外の具体 table を取り付け、
明示 checker-owned call payload がない限り call row は empty のままにする。task 18 が
最終的な obligation-seed handoff を定義する。

### ghost effect

ghost table は local visibility を記録し、assignment effect と pick local を runtime / ghost-only の
別 list に分ける。assertion、contract、invariant、decreasing site は specification-only として
construction され、ghost table の別 row ではなく contract / termination table に記録される。
ghost value は specification context (`requires`、`ensures`、`assert`、`invariant`、`decreasing`) には
流れてよいが、runtime assignment、runtime return value、non-ghost call へ流れてはならない。
違反は flow diagnostic である。

ghost-only pick も non-emptiness obligation を作るが、その runtime effect は後続 extraction で消去される。
runtime pick は local `Pick` binding として表す。どちらの形も generated stable-choice origin を
emit しない。
Task 16 は assignment effect と pick local を runtime / ghost-only の別 list に記録し、
後続 erasure と diagnostic が statement を再解析せずにこの分離を消費できるようにする。

### termination

`ControlFlowTerminationPlan` は次を記録する。

- recursive / mutually recursive algorithm 用の header-level decreasing term。
- loop header に付く loop-level decreasing term。
- measure が既に減少したことを証明しなければならない `continue` edge。
- decreasing measure が存在しないため partial である site。

algorithm-level decreasing term がない場合、phase 10 は algorithm partial site を記録する。
loop-level decreasing term がない場合、phase 10 は loop partial site を記録する。
明示 payload を持つ decreasing term は algorithm header、loop header、到達可能な continue edge に
取り付ける。

この plan は後続 VC generation 用 metadata である。phase 10 は well-foundedness を証明せず、
`VcId` を割り当てず、algorithm を terminating functor に昇格しない。

### source map

```rust
struct ControlFlowSourceMap {
    algorithm_sources: Map<CoreAlgorithmId, CoreSourceRef>,
    block_sources: Map<BasicBlockId, CoreSourceRef>,
    statement_placements: Map<CoreAlgorithmStmtId, ControlFlowStatementPlacement>,
    local_sources: Map<LocalId, CoreSourceRef>,
    loop_sources: Map<LoopId, CoreSourceRef>,
    exit_sources: Map<ControlFlowExitId, CoreSourceRef>,
}
```

すべての block、local、loop、exit、diagnostic、contract/termination site は source reference を
保持しなければならない。synthetic block は、それを作らせた construct の source と、synthetic role を
説明する generated provenance を使う。diagnostic は可能な限り最も狭い responsible source を指す。
`statement_placements` は各 core statement shell が CFG のどこに寄与したかを記録する。
block statement、terminator、loop header、switch arm、local binding、contract/checkpoint site、
error site のいずれかである。synthetic block に展開される structured statement も、source
statement id から CFG ownership へ追跡できる。

### diagnostic

```rust
struct ControlFlowDiagnosticId(usize);

struct ControlFlowDiagnostic {
    kind: ControlFlowDiagnosticKind,
    algorithm: CoreAlgorithmId,
    statement: Option<CoreAlgorithmStmtId>,
    source: CoreSourceRef,
    carried_core_diagnostic: Option<CoreDiagnosticId>,
}

enum ControlFlowDiagnosticKind {
    UnsupportedLocalDeclaration,
    IllegalBreak,
    IllegalContinue,
    Phase9Error,
    FlowDiagnostic,
}
```

`ControlFlowTerminator::Error` は control-flow diagnostic row を参照する。phase 9 error statement
の場合、その row は元の `CoreDiagnosticId` を運ぶ。illegal `break` / `continue` と unsupported
local declaration は、CFG construction が通常の valid edge や local として表せないため Task 15 で
生成する。より広い flow diagnostic は Task 17 に残す。

## Core-to-CFG construction

construction は `CoreAlgorithm.statements` を source order で走査する。statement builder は
`Normal`、`Return`、`Break(loop)`、`Continue(loop)` の exit context を返す。sequential
composition は `Normal` からだけ継続する。

規則:

- `Let` は local を作り、initializer がある場合は normal assignment effect を記録する。
  unsupported declaration metadata は local に保持し、construction diagnostic として報告する。
- `Pick` は `Pick` metadata 付きの site-local local を作り、witness type があれば記録し、
  ghost/runtime に分類し、後続 handoff 用の non-emptiness obligation site を記録する。
- `Assign` は `CorePlace` への write を記録する。use-before-assignment と alias precision は
  diagnostic であり、source-string guess ではない。
- `Assert` は assertion site を作り、後続 VC 生成用に asserted formula を normal successor
  context の fact として記録する。
- `If` は condition block、then/else block、normal exit 用の deterministic join を作る。
  `else` がない branch は false path として表す。
- `While` は loop header、body entry、normal exit、back edge を作る。invariant と decreasing term
  を loop record に取り付ける。`Break` exit は negated loop condition を得ずに loop exit に join する。
- `Match` は scrutinee 上の deterministic switch、source-order arm entry、normal arm exit 用の
  join を作る。pattern exhaustiveness と capture-variable binding metadata は、checker payload が
  explicit pattern semantics を提供するまで external であり、unsupported arm は diagnostic である。
- 仕様 20 章の `For` form は task 13 の `CoreAlgorithmStmtKind` surface にはまだ存在しない。
  checker payload extraction が explicit range / collection loop shell を提供するまで、phase 10 は
  source text から `for` semantics を合成してはならない。checker-owned fixture が将来の
  `ForRange` または `ForEach` shell を提供する場合、それは hidden immutable range、step、
  collection、exit value、必要なら processed-set ghost local、finiteness または
  order-independence contract site、`While` と同じ `Break` / `Continue` exit discipline を持つ
  loop record として表現する。`ForRange` metadata は、direction (`to` または `downto`)、
  hidden positive-`Nat` step value obligation、direction-specific `next(i)` expression、
  仕様 20.13.3 が要求する `past_end(i_exit)` exit predicate を含めなければならない。
- 仕様 20 章の `Snapshot` と claim 関連 algorithm statement は、現在の core statement shell には
  存在しない。checker payload extraction が explicit snapshot shell を提供するまで、phase 10 は
  source text から snapshot を再構築してはならない。将来の `Snapshot` shell は、現在の
  `ProgramContextId`、visible runtime / ghost local、claim block が必要とする hidden loop value を
  capture する source-mapped snapshot site を記録する。missing snapshot payload は diagnostic であり、
  silently erase してはならない。
- `Return` は現在 block を return terminator で閉じ、postcondition site を取り付ける。
- `Break` と `Continue` は innermost active loop に解決する。loop の外では diagnostic になり、
  error terminator を作る。
- `Error` statement は phase 9 diagnostic を保持し、error terminator または error statement site を作り、
  executable flow を仮造しない。

terminating `Return`、`Break`、`Continue`、`Error` の後にある unreachable statement は silently drop
せず、unreachable block と diagnostic として表現する。

## diagnostic

task 17 は、現在の phase-10 core surface から判定できる flow diagnostic を実装する。

- unreachable statement。
- definite assignment 前の use。

`UseBeforeAssignment` は、到達可能な statement-owned expression を対象にする:
`Let` initializer term、assignment right-hand side、`Assert` formula、`If` condition、
`While` condition、`While` invariant formula、`While` decreasing term、`Match` scrutinee、
`Return` value。`Pick` witness formula は、picked binder を locally bound として扱い、
ambient variable だけを検査する。algorithm-level `requires` / `ensures`、aggregate
`CoreContractSet.assertions` と `CoreContractSet.invariants`、final obligation-seed metadata は
Task 17 では検査しない。caller/result substitution や downstream VC context が必要であり、
phase 10 の責務ではないためである。

use-before diagnostic は具体 class `UseBeforeAssignment { local, var }` を持つ。diagnostic source は、
uninitialized local に言及した term occurrence の source であり、`statement` は enclosing
`CoreAlgorithmStmtId` である。`UnreachableStatement { block }` は unreachable statement の source と
unreachable block id を使う。formula traversal は quantifier scope を尊重する。binder は左から右へ
処理し、binder は自分自身の guard、後続 binder、body の中で shadow する。先行 binder は後続 guard と
body で shadow する。unresolved / external variable は source spelling から推測せず無視する。

これ以前の phase-10 task は、loop 外の illegal `break` / `continue`、unsupported
local-declaration metadata、phase 9 から持ち越された malformed / missing algorithm statement の
structural diagnostic をすでに emit している。より広い diagnostic catalog は次の通りである。

- loop 外の illegal `break` / `continue`。
- unsupported local-declaration metadata。
- unreachable statement。
- definite assignment 前の use。
- immutable parameter または const local への assignment。
- ghost value の runtime state への流入。
- unsupported call target または missing contract-instantiation payload。
- unsupported match / pattern payload。
- unsupported snapshot / claim payload。
- phase 9 から持ち越された malformed / missing algorithm statement。
- unsupported aliasing / lvalue metadata。

immutable parameter / const local への assignment、ghost value の runtime state への流入、
call / contract instantiation error、unsupported pattern payload、snapshot / claim payload、
alias / lvalue precision は、現在の `CoreAlgorithmStmtKind` と `CorePlace` surface が公開していない
checker-owned target / payload metadata を必要とする。これらは deferred のままであり、source spelling から
推測してはならない。

diagnostic は source order、algorithm id、block id、diagnostic class の順に sort する。
diagnostic は downstream consumer 向けに algorithm を partial/error と印付けてよいが、
algorithm が verified であると見せかけてはならない。

## 決定性

同じ `CoreIr` input に対し、construction は byte-stable debug rendering と table order を生成する。

- algorithm order は `CoreAlgorithmTable` に従う。
- block order は deterministic traversal に従う。
- local は parameter/result/source statement order に従う。
- loop id は header order に従う。
- diagnostic は上記 ordering に従う。
- map は key order で render する。

hash-map iteration、filesystem order、source spelling fallback が semantic id に影響してはならない。

## validation と test

task 15 の test は次を覆う。

- deterministic block ordering。
- byte-stable control-flow debug rendering と key-ordered source-map output。
- straight-line let/assign/assert/return flow。
- `else` あり/なしの `if`。
- `break` / `continue` を持つ `while`。
- `match` arm ordering。
- local table contents: parameter/result/let/pick/hidden-local kind、source order、
  mutability、ghost flag、initialization site、unsupported declaration metadata。
- unsupported local declaration と illegal `break` / `continue` の structural diagnostic row。
  source ref、error terminator、存在する場合の carried phase-9 diagnostic ref、deterministic
  diagnostic ordering を含む。
- block、local、statement placement の source-map preservation。

現在の `CoreAlgorithmStmtKind` surface には、source-derived hidden loop local を必要とする
checker-owned payload がない。そのため Task 15 は `HiddenLoopValue` を明示的な表現として保持し、
現在の `While` construction が hidden local を仮造しないことを test し、将来の `for`、snapshot、
termination payload から導出される hidden local は deferred とする。

task 16 の test は次を覆う。

- `requires` と `ensures` の配置。
- successor / header / exit context の assertion fact と invariant site。
- algorithm header、loop header、loop `continue` edge の decreasing measure。
- decreasing payload がない場合の algorithm / loop partial site。
- contract、call、assertion、invariant、decreasing、statement-placement site の exact
  source/provenance preservation。
- checker-owned call payload がない間は call-site table が empty であること。
- explicit core shell が存在する場合に限る、将来の range / collection `for` metadata。
- ghost-only state が runtime effect table に現れないこと。
- runtime と ghost の `Pick` distinction。

task 17 の test は次を覆う。

- unreachable statement。
- use before assignment。
- 新しく emit される flow diagnostic の stable diagnostic ordering。
- checker-owned payload が利用できない間、assignment-to-immutable、
  ghost-to-runtime、call / contract、pattern、snapshot / claim、alias / lvalue
  diagnostic を仮造しないこと。

spec-only task 14 は bilingual documentation review と diff check で検証する。Rust 実装と test は
task 15-17 に deferred する。
