# Lazy Explanation Handles

> 正本は英語です。英語版:
> [../en/explain.md](../en/explain.md)。

## 目的

この文書は `mizar-diagnostics` が所有する lazy explanation handle を定義する。
explanation data は、type inference、cluster resolution、overload resolution、proof failure、
verification condition などの diagnostic に bounded context を与えるが、通常の diagnostic
record へ large trace を埋め込まない。

explanation handle は `DiagnosticRecord` に attached される compact で deterministic な
reference である。handle は bounded preview を含んでよい。full explanation trace は artifact、
cache-backed query data、dedicated explanation file、または後続 query service に留まる。

human preview text は presentation である。tool は preview wording ではなく、
`DiagnosticCode`、structured diagnostic field、stable explanation handle identity を key にする。

## Scope

explain module が所有するもの:

- diagnostic record から参照される stable `ExplanationHandleId`。
- explanation kind と subject metadata。
- CLI と editor display に適した bounded preview。
- 後で handle を resolve するために必要な source、snapshot、artifact/hash、query precondition。
- explanation handle の deterministic ordering と debug snapshot。
- available、missing、stale、truncated などの lazy resolution status value。

explain module が所有しないもの:

- proof acceptance、trusted status、kernel acceptance、phase success。
- phase service による type/proof/VC trace の生成。
- artifact creation、artifact mutation、cache validity、cache eviction。
- LSP `mizar/explain` request routing、JSON-RPC payload、editor protocol conversion。
- driver session orchestration、build scheduling、snapshot publication。
- source loading、path normalization、line map、UTF-16 conversion。

## Data Model

task 15 は次と等価な structured explanation payload を実装する。

```rust
struct ExplanationHandle {
    id: ExplanationHandleId,
    kind: ExplanationKind,
    subject: ExplanationSubject,
    source: ExplanationSourceRef,
    required_snapshot: Option<BuildSnapshotId>,
    required_artifact_hash: Option<Hash>,
    summary_hash: Option<Hash>,
    preview: Option<ExplanationPreview>,
}

enum ExplanationSubject {
    Diagnostic { code: DiagnosticCode, stable_detail_key: String },
    Expression(String),
    VerificationCondition(String),
    SourceRange(SourceRange),
    PhaseLocal { phase: PipelinePhase, key: String },
}

enum ExplanationSourceRef {
    PreviewOnly,
    Artifact { path: String, content_hash: Hash },
    CacheRecord { cache_key: String, content_hash: Option<Hash> },
    QueryService { service_key: String, query_key: String },
}

struct ExplanationPreview {
    format: ExplanationPreviewFormat,
    text: String,
    truncated: bool,
    byte_len: usize,
    line_count: usize,
}
```

`Diagnostic` subject は pre-publication stable diagnostic field を使い、`DiagnosticHandle` には
依存しない。handle は aggregation 後にだけ割り当てられるためである。`Expression` と
`VerificationCondition` subject は、この crate boundary では opaque structured key である。意味は
owning phase または後続 query service が定義する。`mizar-diagnostics` は key を保存・比較するが、
semantic authority として解釈しない。
`Diagnostic` subject が diagnostic draft に attach される場合、その code と stable detail key は
containing draft と一致しなければならない。

`ExplanationHandleId` は publication 前に explicit producer key または deterministic
producer-local ordinal から割り当てられる。human preview や localized wording から derived
してはならない。published record は convenience のため、explanation handle の隣に containing
`DiagnosticHandle` を projection してよいが、その back-reference は explanation handle identity
ではない。

`summary_hash` は canonical bounded summary または backing explanation descriptor のための
structured integrity/identity field である。localized preview text の hash ではない。存在する場合、
task 15 はこれを canonical explanation identity に含め、backing explanation data の mismatch 検出に
使わなければならない。

## Explanation Kinds

初期 explanation kind:

| Kind | Meaning |
|---|---|
| `TypeInference` | type、mode、attribute、registration reasoning。 |
| `ClusterResolution` | attribute/cluster search または loop explanation。 |
| `OverloadResolution` | candidate set、selected policy、ambiguity detail。 |
| `ProofFailure` | proof obligation、ATP、evidence rejection context。 |
| `VerificationCondition` | VC generation/checking context。 |
| `AlgorithmTrace` | algorithm-contract checking context。 |
| `DiagnosticContext` | 上記に分類されない general diagnostic context。 |
| `Internal` | developer-only internal explanation data。 |

kind は explanation payload を分類するだけである。proof、kernel replay、phase が成功したかを
決めない。

## Preview Bounds

preview は optional かつ bounded である。実装は次の定数を強制する。

- maximum preview bytes。
- maximum preview lines。
- deterministic truncation marker。

preview が bound を超える場合、stored preview は deterministic に truncate され、
`truncated=true` として mark されなければならない。large trace は `DiagnosticRecord`、
`BuildDiagnosticIndex`、CLI output、通常の artifact diagnostic projection に埋め込んではならない。

## Lazy Resolution

task 15 は handle を lazy に resolve し、次と等価な bounded result を返す store を公開する。

```rust
enum ExplanationResolution {
    Available(ExplanationPayload),
    Missing { reason: ExplanationMissingReason },
    Stale { source_snapshot: BuildSnapshotId, current_snapshot: BuildSnapshotId },
    Unavailable { reason: String },
}
```

resolution は、この crate が backing storage を所有せずに検証できる handle precondition を確認する。
snapshot-bound handle は `required_snapshot` の外では stale である。artifact hash と cache/query
precondition は backing owner が検証できるよう handle identity に保存される。その canonical handle に
対する bounded payload が登録されていない場合、resolution は `Missing` へ degrade する。cache-backed
handle は query を高速化してよいが、cache data 欠落は `Missing` または `Unavailable` へ degrade
しなければならず、diagnostic identity や proof acceptance を変えてはならない。`summary_hash` が存在し
registered backing data が comparable summary hash を公開する場合、mismatch は異なる explanation data を
黙って返すのではなく、`Unavailable` へ degrade する。

store は bounded explanation payload だけを返す。LSP response を publish せず、build を
schedule せず、artifact を mutate せず、proof status を validate しない。

## Attachment And Deduplication

explanation handle は compact structured payload または stable handle として diagnostic record に
attach される。次を preserve しなければならない。

- 所属する diagnostic code。
- 存在する場合は、containing published diagnostic handle を projection として扱うこと。
- stable explanation handle id。
- kind と subject。
- source reference。
- snapshot/artifact/hash precondition。
- 存在する場合は summary hash。
- optional bounded preview metadata。

draft construction は、snapshot precondition が draft の source snapshot と異なる explanation
handle、または diagnostic subject が異なる code/stable detail key を指す explanation handle を
拒否する。これにより stale または foreign diagnostic explanation が current explanation data として
publish されることを防ぐ。

aggregation deduplication は canonical explanation identity を diagnostic identity に含める。
canonical explanation identity は handle id、kind、subject、source reference、
snapshot/artifact/hash precondition、存在する場合は `summary_hash` で構成される。preview text、
localized text、rendered `explain:` line、full explanation payload は deduplication key ではない。

## Debug Snapshot

task 15 は deterministic explanation debug snapshot を公開し、field order は次の通り。

1. `kind=explanation`。
2. `id`。
3. `diagnostic`。
4. `explanation_kind`。
5. `subject`。
6. `source`。
7. snapshot/artifact/hash precondition。
8. `summary_hash`。
9. bounded preview metadata。

snapshot は test/debug data であり、CLI rendering でも LSP response でもない。memory address、
map iteration order、localized field name、process-local ordering、unbounded full trace を
含めてはならない。

## Boundary Rules

- explanation handle は reference であり、embedded proof/type/VC trace ではない。
- explanation payload が欠落しても diagnostic record を suppress してはならない。
- stale handle を current explanation data として publish してはならない。
- cache-backed explanation は optional であり、trusted status を確立しない。
- CLI rendering は bounded preview または `explain:` reference を表示してよいが、large trace を
  resolve しない。
- LSP request conversion、stale-handle retry policy、JSON response shaping は `mizar-lsp`
  または driver layer に属する。
- artifact-backed explanation file は artifact/cache component が所有する。`mizar-diagnostics` は
  reference を保存し、shape だけを validate する。
