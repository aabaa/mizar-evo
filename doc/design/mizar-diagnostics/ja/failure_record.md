# Failure Records

> 正本は英語です。英語版:
> [../en/failure_record.md](../en/failure_record.md)。

## 目的

この文書は `mizar-diagnostics` が所有する diagnostic draft と diagnostic record
model を定義する。record は stable diagnostic identity、source identity、failure
classification、structured details、小さな projection payload を保持する。record
は proof acceptance、trusted status、kernel acceptance、cache reuse、artifact
publication、LSP protocol shape、CLI formatting、driver session orchestration を
決定しない。

registry は `DiagnosticCode` identity を所有する。record は code と registry
descriptor metadata を参照するだけで、message text から identity を推測しない。

## Scope And Lifecycle

record には 2 つの stage がある。

| Stage | Owner | Meaning |
|---|---|---|
| `DiagnosticDraft` | `DiagnosticSink` を通じた phase producer | snapshot-wide deduplication、ordering、handle assignment の前にある producer-owned draft。 |
| `DiagnosticRecord` | Aggregator | deterministic handle、freshness、normalized metadata を持つ immutable published record。 |

draft は build に coherent publication boundary ができる前に収集されてよい。record
は aggregation が source snapshot freshness を検証し deterministic identifier を
割り当てた後だけ publish される。obsolete snapshot diagnostics を current record と
して publish してはならない。

## Shared Fields

draft と record は次の field を共有する。

| Field | Type | 必須 | Rule |
|---|---|---:|---|
| `code` | `DiagnosticCode` | yes | Stable identity。registry に存在し、新しい current diagnostic では retired であってはならない。 |
| `phase` | `PipelinePhase` | yes | diagnostic を生成した phase。これは ordering/provenance metadata であり、phase-status authority ではない。 |
| `category` | `FailureCategory` | yes | 安定した machine-readable failure class。 |
| `stable_detail_key` | `String` | yes | deduplication と sorting のための deterministic key。localized text を含めてはならない。 |
| `message` | `String` | yes | 人間向け primary message。version 間で変わってよく、identity ではない。 |
| `primary_span` | `DiagnosticSpan` | yes | 主 location。必ず `SourceId` を参照する。 |
| `secondary_spans` | `Vec<DiagnosticSpan>` | yes, may be empty | 補助 location。自然な順序がある場合は producer が並べ、aggregator が normalize する。 |
| `notes` | `Vec<DiagnosticNote>` | yes, may be empty | 人間向け note/help text と任意の source anchor。 |
| `details` | `DiagnosticDetails` | yes, may be empty | machine-readable payload map。 |
| `fixes` | `Vec<FixSuggestionRef>` | yes, may be empty | task 13 が追加する structured fix suggestion。それまでは attachment slot。 |
| `explanation` | `Option<ExplanationRef>` | optional | task 15 が追加する lazy explanation handle。それまでは attachment slot。 |

message、note、summary、rendered label、localized text は presentation payload である。
tools と consumers はこれらの文字列ではなく `DiagnosticCode` と structured field を
key にしなければならない。

## DiagnosticDraft

```rust
struct DiagnosticDraft {
    source_snapshot: BuildSnapshotId,
    code: DiagnosticCode,
    phase: PipelinePhase,
    category: FailureCategory,
    stable_detail_key: String,
    message: String,
    primary_span: DiagnosticSpan,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    details: DiagnosticDetails,
    fixes: Vec<FixSuggestionRef>,
    explanation: Option<ExplanationRef>,
}
```

draft の規則:

- producer はすべての span を既に `SourceId` に grounded にしていなければならない。
- `source_snapshot` は producer が観測した build snapshot である。producer はこれを
  必ず与えるが、draft が publication に対して current であるかは決定しない。
- draft 上の span freshness は将来の publication snapshot ではなく
  `source_snapshot` に対するものである。draft span は通常 `SpanFreshness::Current`
  を使う。producer は古い context から copy した related span に限り stale または
  historical と mark してよいが、publication freshness はなお aggregator が所有する。
- producer は CLI-rendered string、LSP range、JSON-RPC payload、artifact mutation
  instruction、driver event を attach してはならない。
- producer が structured detail を attach できるのは、各 key が人間向け wording から
  独立した stable meaning を持つ場合だけである。

## DiagnosticRecord

```rust
struct DiagnosticRecord {
    handle: DiagnosticHandle,
    code: DiagnosticCode,
    semantic_name: String,
    severity: DiagnosticSeverity,
    phase: PipelinePhase,
    category: FailureCategory,
    stable_detail_key: String,
    message: String,
    primary_span: DiagnosticSpan,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    details: DiagnosticDetails,
    fixes: Vec<FixSuggestionRef>,
    explanation: Option<ExplanationRef>,
    related: Vec<DiagnosticHandle>,
    freshness: DiagnosticFreshness,
}
```

record の規則:

- later spec が safe severity override を明示的に定義しない限り、`semantic_name` と
  `severity` は validated registry descriptor から copy する。
- `handle` は aggregator が割り当て、その snapshot 内でだけ意味を持つ。
- `freshness` は diagnostic が使った source snapshot と publication boundary が一致
  するかを記録する。
- published current record は obsolete source snapshot を使ってはならない。
- `related` は同じ build snapshot 内の record を link する。cross-snapshot 関係は
  direct handle ではなく explanation または artifact reference として表す。

## Construction And Round Trips

task 5 の record round-trip は in-memory structural round-trip であり、durable
serialization ではない。draft constructor は code を `DiagnosticRegistry` に対して
validate し、unknown code と retired code を reject しなければならない。record
constructor は draft code を validated registry で lookup し、すべての shared draft
field を正確に保存し、その lookup で見つかった descriptor から `semantic_name` と
`severity` を copy し、clone、equality、accessor、deterministic debug snapshot を通じて
同じ値を公開しなければならない。`SourceId` と `BuildSnapshotId` は task 5 record API
では session-local identity のままである。task 5 API は JSON、LSP、artifact、
cross-session serialization を約束しない。

draft から record へ変換するとき、aggregator は publication snapshot を与える。
`source_snapshot` が publication snapshot と等しい draft は
`DiagnosticFreshness::Current` を生成してよい。`source_snapshot` が異なる draft は
`Stale` を生成するか、aggregator によって current publication から除外されなければ
ならない。task 5 constructor は test と将来 consumer のために stale/historical
record を表現してよいが、それらを current build output として publish してはならない。

## Handles And Freshness

```rust
struct DiagnosticHandle {
    snapshot: BuildSnapshotId,
    id: DiagnosticId,
}

struct DiagnosticId(u64);

enum DiagnosticFreshness {
    Current {
        source_snapshot: BuildSnapshotId,
    },
    Stale {
        source_snapshot: BuildSnapshotId,
        current_snapshot: BuildSnapshotId,
        reason: StaleDiagnosticReason,
    },
    Historical {
        source_snapshot: BuildSnapshotId,
        artifact_hash: Option<String>,
    },
}

enum StaleDiagnosticReason {
    SourceEdited,
    SourceRemoved,
    SnapshotSuperseded,
    ProducerCacheObsolete,
    HistoricalReplay,
}
```

`DiagnosticId` は 1 つの `BuildSnapshotId` の中で deterministic であり、global な
意味は持たない。aggregator は source identity、diagnostic code、phase、primary
span、stable detail key、normalized details、fix identity、explanation identity、
deduplicated ordinal からこれを導出する。

`Current` record は、owning consumer による CLI output、artifact projection、
semantic LSP publication の対象になりうる。`Stale` record は LSP layer が stale と
mark し unsafe edit を suppress する場合だけ editor overlay として見せてよい。
`Historical` record は artifact/cache/log read 用であり、current build output と
扱ってはならない。

`Current { source_snapshot }` は、`source_snapshot` が record handle に使われた
publication snapshot と等しい場合だけ valid である。`Stale { source_snapshot,
current_snapshot, .. }` は 2 つの snapshot が異なることを要求する。`Historical`
record は current publication snapshot を持たず、current `BuildDiagnosticIndex` に
含めてはならない。

## Source Spans

```rust
struct DiagnosticSpan {
    range: SourceRange,
    role: DiagnosticSpanRole,
    label: Option<String>,
    freshness: SpanFreshness,
    zero_width: Option<ZeroWidthSpanIntent>,
}

enum DiagnosticSpanRole {
    Primary,
    Secondary,
    DefinitionSite,
    Related,
}

enum SpanFreshness {
    Current,
    Stale { reason: StaleDiagnosticReason },
    Historical,
}

enum ZeroWidthSpanIntent {
    Eof,
    InsertionPoint,
}
```

`SourceRange` は `mizar-session` の byte range であり、`SourceId`、`start`、`end` を
持つ。すべての diagnostic span は `SourceId` を含まなければならない。line/column
conversion、UTF-16 offset、context snippet、rendered underline は render または LSP
consumer が所有する projection である。

task 5 の span constructor は `start <= end`、`primary_span.role == Primary`、および
`secondary_spans` に `role == Primary` の entry が無いことを強制しなければならない。
file length や line-map membership は validate しない。それは source-map consumer の
責務である。zero-width range は `zero_width` が `Some(Eof)` または
`Some(InsertionPoint)` の場合だけ許される。non-zero range は `zero_width == None` を
使わなければならない。

## FailureCategory

`FailureCategory` は propagation、ordering、regression test のための安定した
machine-readable classification である。初期 category は次の通り。

| Category | Meaning |
|---|---|
| `parse_error` | Lexical、syntactic、parse-recovery failure。 |
| `resolve_error` | Name、import、namespace、symbol resolution failure。 |
| `type_error` | Type、attribute、mode、registration mismatch。 |
| `overload_ambiguity` | Overload/template ambiguity または no viable overload。 |
| `cluster_loop` | Cluster、attribute、registration cycle。 |
| `atp_timeout` | obligation を unresolved に残す ATP timeout または resource exhaustion。 |
| `certificate_rejection` | Malformed、unsupported、または policy-rejected evidence envelope。 |
| `kernel_rejection` | Kernel-level evidence または replay rejection。 |
| `logic_failure` | 上記に分類されない logical inconsistency または VC failure。 |
| `compatibility_warning` | Compatibility または packaging warning。 |
| `informational` | `I` code 割り当て後の informational display diagnostic。 |
| `internal_invariant` | 通常の user output ではない developer-mode internal diagnostic。 |

各 category は descriptive metadata である。failure を success に変えたり、
proof/evidence/kernel rejection を downgrade したりできない。
`malformed_certificate`、`unsupported_certificate_format`、`invalid_substitution`、
`invalid_sat_refutation` などの specific rejection reason は structured details に
保存しなければならない。

## Structured Details

```rust
struct DiagnosticDetails {
    entries: BTreeMap<String, DiagnosticDetailValue>,
}

enum DiagnosticDetailValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Code(DiagnosticCode),
    Source(SourceRange),
    List(Vec<DiagnosticDetailValue>),
}
```

`stable_detail_key` と detail-map key は次の ASCII grammar に一致しなければならない。

```text
detail_key = segment ("." segment)*
segment = [a-z][a-z0-9]* ("_" [a-z0-9]+)*
```

例: `proof.rejection_reason`、
`declaration_symbol.symbol.duplicate_declaration`、`resolve.candidate_count` は valid
である。empty segment、leading/trailing dot、uppercase、leading/trailing/doubled
underscore、hyphen、whitespace、non-ASCII character は invalid である。

detail value は debug/test snapshot のために deterministic に比較・serialize できなければ
ならない。canonical value ordering は variant order `Boolean`、`Integer`、`String`、
`Code`、`Source`、`List` を使い、各 variant 内は自然順、list は lexicographic comparison
を使う。`Source` value は `SourceId` debug identity、`start`、`end` の順に並べる。
details には compact preview や reference を含めてよいが、large trace、LSP payload、
terminal escape sequence、localized prose を埋め込んではならない。

大きな proof trace、solver log、candidate list、explanation body は artifact、
cache-backed query data、または explanation payload file に属する。record はそれらの
copy ではなく handle を保存してよい。

## Notes

```rust
struct DiagnosticNote {
    kind: DiagnosticNoteKind,
    message: String,
    span: Option<DiagnosticSpan>,
}

enum DiagnosticNoteKind {
    Note,
    Help,
    Cause,
    Related,
}
```

note は record に attach される human-facing projection である。`help` note は CLI
output で `help:` として render されてよいが、note 自体は structured edit ではない。
structured edit は `FixSuggestionRef` と task 13 に属する。

## Attachment Slots

`FixSuggestionRef` と `ExplanationRef` は record attachment slot であり、placeholder
adapter ではない。task 13 は structured fix payload と edit applicability を定義する。
task 15 は lazy explanation handle を定義する。それらの task が入るまでは、record
実装は record shape に必要な場合に限り、これらの field を空 attachment vector または
opaque reference として表してよい。provisional LSP code action、CLI formatting、
explanation storage を発明してはならない。
task 5 の opaque attachment identity は、aggregation と debug snapshot が deterministic
に比較できるよう `stable_detail_key` と同じ ASCII grammar を使う。後続の fix/explain
task はこれらの identity をより豊かな payload で wrap してよいが、人間向け text を
identity として再解釈してはならない。

## Deterministic Debug Rendering

task 5 implementation は draft と record の deterministic debug rendering を提供
しなければならない。これは test/debug format であり CLI rendering ではない。実装は
LF line ending、color 無し、localized field name 無しの canonical debug snapshot string
として公開するべきであり、field order は次の通り。

1. `kind`（`draft` または `record`）。
2. `handle`（draft では `none`）。
3. `code`。
4. `semantic_name`（draft では `none`）。
5. `severity`（draft では `none`）。
6. `phase`。
7. `category`。
8. `stable_detail_key`。
9. `message`。
10. `source_snapshot`。
11. `freshness`（draft では `draft`）。
12. `primary`。
13. `secondary`。
14. `notes`。
15. `details`。
16. `fixes`。
17. `explanation`。
18. `related`。

string は Rust debug-string escaping で escape する。span は
`<SourceId debug>:<start>..<end>:<role>:<span freshness>:<zero-width intent>:<label>`
として render する。`SourceId debug` は session-local test/debug identity であり、
published schema string ではない。detail entry は key order で render し、value は
[Structured Details](#structured-details) の canonical ordering と representation で
render する。empty list は `[]`、absent optional field は `none` として render し、
record は complete freshness state と related handle を render する。

これは nondeterministic map iteration order、identity としての localized string、
memory address、process-local ordering を含めてはならない。

## Boundary Rules

- record は proof、kernel、cache、artifact、driver、LSP の fact を記述してよいが、
  それらを決定しない。
- proof または kernel rejection record は proof status を決定しない。proof または
  kernel component がその decision を所有する。
- freshness state は snapshot や artifact を mutate しない。aggregation と consumer
  layer が publication を決定する。
- record は LSP UTF-16 position ではなく `SourceRange` を保存する。
- record は compact structured detail を保存し、artifact manifest や cache mutation
  instruction は保存しない。
