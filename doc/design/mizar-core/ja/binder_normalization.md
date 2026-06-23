# binder normalization

> 正本は英語です。英語版:
> [../en/binder_normalization.md](../en/binder_normalization.md)。

状態: `mizar-core` task 4 のモジュール仕様。この文書は task 5 と task 6 に対して
規範的である。core の項と論理式向けに
[architecture 16](../../architecture/ja/16.substitution_and_binding.md) を精緻化し、
data-shape input として [core_ir.md](./core_ir.md) を使う。

## scope

`binder_normalization` は `CoreIr` の項、論理式、binder、証明骨格断片、アルゴリズム
契約論理式上の決定的な束縛挙動を所有する。後続の VC 生成と kernel substitution
checking が再生する表現と操作を提供する。

所有する挙動:

- normalized core term/formula の正準 binder 表現。
- display name ではなく正準形による alpha equivalence。
- free-variable computation と side-condition extraction。
- normalized core term/formula 上の capture-avoiding substitution。
- `set`、`deffunc`、`defpred` の definition-time closure representation と substitution
  mechanics。
- normalized result を source-facing binder に戻す必要がある場合の決定的 fresh variable
  allocation。
- malformed substitution evidence の構造化された rejection。

責務外:

- name resolution、type checking、overload resolution、registration activation、proof
  search。
- source-to-checker payload extraction。
- VC id assignment、kernel certificate format emission、proof acceptance。
- raw source rescanning や text-macro expansion。

## gap classification

| ID | class | evidence | action |
|---|---|---|---|
| BIND-G001 | `spec_gap` | task 4 前に `binder_normalization.md` は存在しない。 | 本 document が task 5 と task 6 用の module-spec gap を閉じる。 |
| BIND-G002 | `test_gap` | task 5 前に binder-normalization source / test は存在しない。 | task 5 が substitution test を追加し、task 6 が alpha-equivalence / normalization test を追加する。 |
| BIND-G003 | `external_dependency_gap` | `mizar-kernel` は workspace crate ではなく、certificate replay も未実装。 | replay 可能な形のみを仕様化し、kernel API を仮実装しない。 |
| BIND-G004 | `external_dependency_gap` | source-derived checker payload extraction は upstream で deferred。 | 実際の source-to-core fixture を作れるまでは explicit Rust fixture を使う。 |
| BIND-G005 | `deferred` | cross-edit proof reuse anchor と `VcId` は downstream concern。 | binder path は決定的にするが、downstream id は割り当てない。 |

## representation decision

`mizar-core` は normalized term/formula に locally nameless representation を使う。

- bound variable は現在の binder stack に相対的な de Bruijn index で表す。
- free variable は周囲の core context 由来の安定した `CoreVarId` を保持する。
- schematic variable と generated fresh variable は、variable class 付きの明示的な安定 id
  を保持する。
- source display name は binder frame と diagnostic のためだけに保持する。

これが task 5 と task 6 の採用表現である。

根拠:

- alpha-equivalent な入力は name-renaming map を持たずに同一の構造的 encoding へ
  normalize される。
- definition-time closure が捕捉する free variable は安定 id のままなので、後続の display
  name shadowing が意味を変えない。
- substitution replay は normalized tree と binder stack の純粋な線形 walk として実行
  できる。
- kernel は resolver、checker、parser、cluster、overload、ATP state を参照せずに
  side condition を再検査できる。
- pure de Bruijn は closure と diagnostic に必要な captured free variable を見えにくくする。
  named-with-alpha は renaming map の再生が必要で、display-name dependency を入れやすい。

## core shapes

task 5 はこれらを具体 Rust type または等価な private structure として実装してよいが、
以下の invariant を保たなければならない。

```rust
enum NormalizedVar {
    Bound(BoundVar),
    Free(CoreVarId),
    Schematic(CoreVarId),
    Generated(CoreVarId),
}

enum NormalizedVarClass {
    Free,
    Schematic,
    Generated,
}

enum NormalizedVarSort {
    Term,
    Formula,
}

struct BoundVar {
    index_from_innermost: u32,
}

struct BinderContext {
    frames: Vec<BinderFrame>,
    free_variables: BTreeSet<CoreVarId>,
    variable_classes: BTreeMap<CoreVarId, NormalizedVarClass>,
    variable_roles: BTreeMap<CoreVarId, CoreVarRole>,
    variable_sorts: BTreeMap<CoreVarId, NormalizedVarSort>,
}

struct BinderFrame {
    canonical_index: u32,
    original_var: CoreVarId,
    role: CoreVarRole,
    source_name: Option<String>,
    source: CoreSourceRef,
}

struct NormalizedBinderEntry {
    frame: BinderFrame,
    ty_guard: Option<Box<NormalizedFormula>>,
}

struct GeneratedOriginRecord {
    owner: CoreItemId,
    kind: GeneratedOriginKind,
    key: GeneratedOriginKey,
    params: Vec<NormalizedVar>,
}

enum NormalizedTermKind {
    Var(NormalizedVar),
    Const(SymbolId),
    Apply { functor: SymbolId, args: Vec<NormalizedTerm> },
    Select { selector: SymbolId, base: Box<NormalizedTerm> },
    Tuple(Vec<NormalizedTerm>),
    SetEnum(Vec<NormalizedTerm>),
    Generated { origin: GeneratedOriginRecord, args: Vec<NormalizedTerm> },
    Error(CoreDiagnosticId),
}

struct NormalizedTerm {
    kind: NormalizedTermKind,
    free_variables: BTreeSet<CoreVarId>,
}

enum NormalizedFormulaKind {
    // binder 以外の logical node は `CoreFormulaKind` に従うためここでは省略する。
    Forall { binders: Vec<NormalizedBinderEntry>, body: Box<NormalizedFormula> },
    Exists { binders: Vec<NormalizedBinderEntry>, body: Box<NormalizedFormula> },
    Error(CoreDiagnosticId),
}

struct NormalizedFormula {
    kind: NormalizedFormulaKind,
    free_variables: BTreeSet<CoreVarId>,
}
```

`BoundVar.index_from_innermost = 0` は最も近い binder を指す。binder に入ると外側の
bound variable index は 1 増える。binder から出ると、閉じた本体の外にある variable だけを
戻す。index は意味論的であり、source name は意味論的ではない。

`CoreVarId` は 1 つの `CoreIr` snapshot 内でのみ安定である。cross-snapshot proof reuse
anchor として使ってはならない。

`variable_classes`、`variable_roles`、`variable_sorts` は `Free`、`Schematic`、
`Generated` classification、role-sensitive check、term/formula compatibility の source of
truth である。task 5 はこれらの map または等価な context object を実装しなければならない。
`CoreTermKind::Var(CoreVarId)` occurrence が binder stack にない場合、この context により
分類する。通常の local core variable についてのみ、class entry がない場合の既定を `Free` と
してよいのは raw normalization 中だけである。schematic / generated id は明示的な context
entry を持たなければならない。validation と canonicalization は、すべての non-bound
normalized variable について明示的な class/role/sort metadata を要求する。normalized object
が構築された後は、normalization-only の default を再利用しない。Binder frame は自身の role
と source を持つため、freshened binder frame は fresh id に対応する context metadata がない
場合でも valid のままである。context metadata が存在する場合は complete で frame と一致し
なければならない。

## binder normalization

normalization は binder stack を持って `CoreTerm` / `CoreFormula` を walk する。

1. binder に入ると、lexical binder order で `BinderFrame` を push する。
2. binder は現在の normalized object 内の次の `canonical_index` を受け取る。
3. binder scope 内の `original_var` occurrence は
   `NormalizedVar::Bound(BoundVar { index_from_innermost })` になる。
4. binder stack に見つからない occurrence は、周囲の core context に従って
   `Free`、`Schematic`、または `Generated` のままにする。
5. source name と source range は frame/provenance に保持するが、semantic equality と
   hash から除外する。

複数 binder を持つ construct では、binder を左から右へ処理する。

- binder `i` の guard は prior binders と binder `i` を push した後、binder `i+1..` が
  見える前に normalize する。
- したがって binder 自身の guard は、surface form が self-typed predicate を許す場合には
  その binder を参照してよい。prior binder も参照できるが、later binder は参照しては
  ならない。
- body はその construct の binder をすべて push した後にだけ normalize する。

この規則は quantifier、definition formal、scheme/template formal、proof-introduction
binder、algorithm parameter/result binder list に適用する。typed binder list について
alpha-equivalence と substitution が食い違うことを防ぐ。type guard は display-name lookup に
依存してはならない。

normalization output は platform、worker count、hash-map iteration order、diagnostic display
name に依存せず決定的でなければならない。

task 6 の public normalization API は次の通りである。

```rust
fn normalize_core_term(
    core: &CoreIr,
    term_id: CoreTermId,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm>;

fn normalize_core_formula(
    core: &CoreIr,
    formula_id: CoreFormulaId,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula>;
```

dense id row、generated-origin row、diagnostic row が欠けている場合は malformed binding
evidence であり、`BinderDiagnosticClass::MalformedEvidence` を返さなければならない。
これらの関数は resolver、checker、VC、proof、kernel service を呼ばない。

## alpha-equivalence

2 つの normalized term/formula が alpha-equivalent であるとは、次を除外した意味論的
normalized encoding が構造的に等しいことをいう。

- source display name。
- source range。
- diagnostic と recovery metadata。
- logical node の一部ではない provenance。

alpha-equivalence に含めるもの:

- bound-variable index。
- free/schematic/generated stable id と class。
- functor / predicate `SymbolId`。
- `CoreTermKind::Generated` は generated-origin semantic record、つまり owner、kind、
  semantic key、alpha-normalized payload、normalized argument order で比較する。dense な
  `GeneratedOriginId` は 1 つの `CoreIr` snapshot 内の table lookup としてだけ使ってよい。
  stable-choice term はすでに通常の `Apply` node でなければならない。
- recovery/error node は、両辺が明示的な recovery data である場合に限り diagnostic id で
  比較する。Error node は proof acceptance 用の valid logical term ではない。
- binder role と type guard。
- logical structure と child order。

task 6 は normalized structure の純粋比較として alpha-equivalence API を公開しなければ
ならない。name resolution を再実行したり source text を参照したりしてはならない。

task 6 はさらに次を公開する。

```rust
fn validate_normalized_term(
    term: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()>;

fn validate_normalized_formula(
    formula: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()>;

fn canonical_term(
    term: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<CanonicalTerm>;

fn canonical_formula(
    formula: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<CanonicalFormula>;

fn alpha_equivalent_terms(
    left: &NormalizedTerm,
    right: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<bool>;

fn alpha_equivalent_formulas(
    left: &NormalizedFormula,
    right: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<bool>;
```

validation は `context.frames.len()` から開始するため、ambient binder context の下で
normalize された term/formula は valid のままである。invalid de Bruijn index、malformed
binder metadata、guard 内の later-binder reference、normalized structure と一致しない public
`free_variables` cache を reject しなければならない。

canonical key は source name、source range、provenance、diagnostic display text を除外する。
bound index、free/schematic/generated id と class、symbol id、binder role と guard、
generated semantic origin record、child order、recovery diagnostic id は含める。key shape は
次と等価である。

```rust
enum CanonicalVar {
    Bound(u32),
    Free(CoreVarId),
    Schematic(CoreVarId),
    Generated(CoreVarId),
}

struct CanonicalBinderEntry {
    role: CoreVarRole,
    ty_guard: Option<Box<CanonicalFormula>>,
}

struct CanonicalGeneratedOrigin {
    owner: CoreItemId,
    kind: GeneratedOriginKind,
    key: GeneratedOriginKey,
    params: Vec<CanonicalVar>,
}
```

## free variables

free-variable computation は安定 id の sorted set を返す。binder に入ると、その binder の
`original_var` は body の free-variable set から除かれる。shadowed source name は
`CoreVarId` が異なるため別物のままである。

free-variable constraint は次の形を持つ。

```rust
struct FreeVariableConstraint {
    variable: CoreVarId,
    must_remain_free_in: NormalizedTermOrFormulaPath,
}
```

constraint は substitution replay または後続の kernel checking が、variable が capture されて
いないことを証明する必要がある場合に emitted される。

## capture-avoiding substitution

substitution は normalized term/formula 上で定義する。

```rust
struct Substitution {
    target: SubstitutionTarget,
    replacement: SubstitutionReplacement,
    side_conditions: Vec<FreeVariableConstraint>,
}

enum SubstitutionResult<T> {
    Applied(T),
    Rejected(BinderDiagnostic),
}

enum SubstitutionTarget {
    TermVar(NormalizedVar),
    FormulaVar(NormalizedVar),
}

enum SubstitutionReplacement {
    Term(NormalizedTerm),
    Formula(NormalizedFormula),
}

struct BinderDiagnostic {
    class: BinderDiagnosticClass,
    source: CoreSourceRef,
    owner: Option<CoreNodeRef>,
    message_key: CoreDiagnosticMessageKey,
}
```

規則:

- substitution は normalized source、normalized target、binder context、side condition の
  純粋関数である。
- substitution は sort-safe である。`TermVar` target は term replacement だけを受け入れ、
  `FormulaVar` target は formula replacement だけを受け入れる。variable class、role、sort
  constraint は target `BinderContext` から取る。free / schematic / generated variable に必要な
  class/role/sort metadata がない場合、推測せず diagnostic payload 付きで reject または defer する。
- bound substitution は binder を越えるときに de Bruijn shifting を使う。
- free-variable substitution は `CoreVarId` を保持し、target context 内の binder に捕獲
  されてはならない。
- replacement が捕獲されるなら、結果を binder-bearing core shape に戻す前に決定的 freshness
  strategy を使わなければならない。
- freshness または side condition を満たせない場合、substitution は rejected になる。
  capture-prone term を黙って生成してはならない。
- substitution は registration を activate せず、無関係な definition を unfold せず、
  proof search も overload resolution も実行しない。

`BinderDiagnostic` は所有された diagnostic payload であり、すでに挿入済みの
`CoreDiagnosticId` ではない。結果を `CoreIr` に統合する caller が diagnostic table へ挿入し、
error node または skipped item をその id に接続する責務を持つ。これにより substitution
replay は純粋に保たれ、dangling diagnostic id を防ぐ。

### de Bruijn operations

task 5 は次の operation を正確に実装するか、等価な helper を公開しなければならない。

```text
shift(term_or_formula, cutoff, delta)
open_rec(term_or_formula, depth, replacement)
open(term_or_formula, replacement) = open_rec(term_or_formula, 0, replacement)
close_rec(term_or_formula, depth, variable)
close(term_or_formula, variable) = close_rec(term_or_formula, 0, variable)
subst_bound(term_or_formula, depth, replacement)
```

定義:

- `shift(cutoff, delta)` は `i >= cutoff` の bound index に `delta` を加え、`i < cutoff` は
  変えない。負の `delta` は index が負にならない場合だけ許される。そうでない場合は
  rejected になる。
- `open_rec(body, depth, replacement)` は bound index `depth` を
  `shift(replacement, 0, depth)` で置き換え、`depth` より大きい index を decrement し、
  `depth` より小さい index は変えない。binder の下へ入ると `depth` は 1 増える。
- `close_rec(body, depth, variable)` は `variable` の free occurrence を bound index `depth` で
  置き換え、`>= depth` の index を increment し、binder の下では `depth + 1` で再帰する。
- `subst_bound(body, depth, replacement)` は bound index `depth` を
  `shift(replacement, 0, depth)` で置換し、binder structure は残す。binder の下では
  `depth + 1` で再帰する。`open_rec` と異なり、周囲の index は decrement しない。

use-site で normalize された actual argument は depth `0` に置かれる。use-site binder stack に
よって事前に shift してはならない。replacement を shift する operation は `open_rec` と
`subst_bound` だけであり、closure body または target binder の挿入地点の depth 分だけ shift
する。

すべての operation は sorted free-variable set を保持し、invalid index を saturate や wrap
せず rejected にしなければならない。

### freshness witnesses

freshening は再生可能な evidence を生成する。

```rust
struct FreshnessWitness {
    source_id: SourceId,
    owner: CoreItemId,
    original: CoreVarId,
    fresh: CoreVarId,
    binder_path: NormalizedTermOrFormulaPath,
    role: CoreVarRole,
    counter: u32,
}
```

`mizar-core` はこれらの witness を後続 phase 用の internal replay evidence として記録する
責務を持つ。kernel certificate schema は定義しないが、downstream kernel checker が
witness または周囲の replay envelope から `source_id`、owner `CoreItemId`、normalized binder
path、role、original/fresh id、counter を得て、deterministic freshness strategy から fresh id を
再計算し、renaming が capture を避けたことを検証できるようにしなければならない。

task 5 の composition law test:

```text
subst(B, y := U) after subst(A, x := T)
```

は、`x` と `y` が互いに capture しないことを side condition が証明する場合、単一の
normalized composition と等しくなければならない。そうでない場合、composition は決定的な
diagnostic で rejected にならなければならない。

## definition-time closures

local `set`、`deffunc`、`defpred` definition は normalized closure として保存する。

```rust
struct DefinitionClosure {
    formals: Vec<BinderFrame>,
    body: NormalizedTermOrFormula,
    captured_free_variables: BTreeSet<CoreVarId>,
    formal_type_guards: Vec<NormalizedFormula>,
    source: CoreSourceRef,
}
```

保存された body は closure formal について閉じている。formal occurrence は `formals` prefix に
相対的な bound index として表現し、definition-site で捕捉された variable は
`captured_free_variables` 内の free `CoreVarId` のままにする。closure body は use-site binder
stack に相対的に open ではない。

定義時、closure は body に現れるすべての free variable id を記録する。使用時:

1. actual argument を use-site context で normalize する。
2. actual は depth `0` に保持する。closure body 内の binder の下へ挿入する地点でだけ
   `open_rec` または `subst_bound` により shift される。
3. 右から左へ formal bound variable に actual を substitute する。これにより、先に行った
   replacement が残りの formal index を変えない。
4. expanded body を use-site context へ open する。
5. captured free variable は definition-site の `CoreVarId` を保持する。
6. formal type guard は body と同じ formal に対して閉じており、同じ actual substitution を
   右から左へ適用して instantiate する。expansion は、instantiate された guard side condition が
   discharge されるまで保持されている場合に限り conditionally valid である。
7. instantiate された formal type guard は normalized guard fact として expansion result と一緒に返す。
8. それらの guard fact を明示的 predicate、assumption、obligation seed、diagnostic の
   いずれにするかは `binder_normalization` ではなく elaborator が決めるが、黙って捨ててはならない。

必須 regression:

```mizar
defpred P(n be Nat) means n < m;
for m being Nat holds P(m)
```

実引数としての quantified `m` と definition body 内に捕捉された外側の `m` は、展開後も
異なる variable id のままでなければならない。

## deterministic freshness

fresh variable id は次から生成する。

- `SourceId`。
- owner `CoreItemId`。
- normalized binder path。
- variable role。
- normalized object に閉じた deterministic counter。

生成 display name は diagnostic 専用である。equality、hash、proof acceptance、
generated-origin key に影響してはならない。

## diagnostics

malformed binding evidence は構造化された core diagnostic を生成する。例:

- supposedly closed normalized term 内の unbound variable id。
- binder stack 外を指す de Bruijn index。
- free-variable side condition violation。
- free variable を capture してしまう substitution。
- deterministic freshness exhaustion または collision。
- closure actual と formal の不一致。

diagnostic は source/core provenance を保持するが、term を heuristic に修復してはならない。
invalid normalized fragment は明示的な error node として残るか、所有 operation が
`Rejected` を返す。

## test obligations

task 5 は Rust test で以下を追加する。

- nested binder 下の capture-avoiding substitution。
- source display name が shadowing されても stable id が異なる case。
- substitution が binder を越えるときの deterministic freshening。fresh id の再計算に使う
  `FreshnessWitness` の `source_id`、owner、original id、fresh id、binder path、role、counter も
  検査する。
- accepted case の substitution composition law。
- side condition または capture check が失敗する rejected substitution composition。
- capture-producing evidence、明示的な side-condition violation、deterministic freshness
  collision / exhaustion、term/formula sort mismatch、role mismatch、free / schematic /
  generated variable の class/role/sort metadata 欠落を含む malformed substitution の rejection。
- `defpred P(n be Nat) means n < m` shadowing regression を含む definition-time closure
  expansion。
- closure arity / formal mismatch rejection と expansion result における actual-instantiated
  formal type guard の preservation。

task 6 は Rust test で以下を追加する。

- alpha-equivalence の reflexivity、symmetry、transitivity。
- normalization idempotence。
- canonical form が等しいことと term/formula が alpha-equivalent であることの同値性。
- repeated run をまたぐ deterministic canonical form。
- free-variable set ordering と binder removal。
- invalid de Bruijn index と malformed binder context の rejection。
- nested binder 下の `shift`、`open_rec`、`open`、`close_rec`、`close`、`subst_bound` edge
  case。非ゼロ depth の `open_rec` / `close_rec` と、`open_rec` は周囲の index を decrement
  するが `subst_bound` は binder structure を保つという対比を含める。
- generated term の alpha-equivalence。semantic origin record が等しい異なる dense id、
  owner/kind/key/alpha-normalized payload が異なる case、argument order が異なる case、
  explicit recovery/error node を含める。

task 4 では `.miz` fixture は不要である。source-derived substitution coverage は、
checker payload extraction と mizar-test stage support が捏造なしで `mizar-core` に入力を
渡せるようになるまで deferred。

## forbidden behavior

`binder_normalization` は以下をしてはならない。

- semantic equality に source display name を使うこと。
- raw source lookup または text macro expansion。
- malformed substitution evidence の heuristic repair。
- replay 中に active resolver、checker、parser、cluster、overload、ATP、kernel state を
  参照すること。
- `VcId`、proof acceptance status、cross-edit reuse anchor の割り当て。
