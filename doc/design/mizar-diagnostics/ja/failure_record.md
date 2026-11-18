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
| `primary_location` | `DiagnosticPrimaryLocation` | yes | 主スパン、ソース読込み要求、または Documentation の由来位置。 |
| `secondary_spans` | `Vec<DiagnosticSpan>` | 必須、空を許す | 生成元の順序と重複を保持し、集約は代表payloadを選択する。 |
| `notes` | `Vec<DiagnosticNote>` | yes, may be empty | 人間向け note/help text と任意の source anchor。 |
| `details` | `DiagnosticDetails` | yes, may be empty | machine-readable payload map。 |
| `fixes` | `Vec<FixSuggestion>` | yes, may be empty | structured advisory fix suggestion。canonical payload で normalize され、edit を適用しない。 |
| `explanation` | `Option<ExplanationHandle>` | optional | bounded preview metadata を持つ structured lazy explanation handle。 |

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
    primary_location: DiagnosticPrimaryLocation,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    details: DiagnosticDetails,
    fixes: Vec<FixSuggestion>,
    explanation: Option<ExplanationHandle>,
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
- producer が `required_snapshot` を持つ `ExplanationHandle` を attach する場合、
  その snapshot は draft の `source_snapshot` と一致しなければならない。
- attach された `ExplanationHandle` が diagnostic subject を使う場合、その
  `DiagnosticCode` と stable detail key は containing diagnostic と一致しなければならない。

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
    primary_location: DiagnosticPrimaryLocation,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    details: DiagnosticDetails,
    fixes: Vec<FixSuggestion>,
    explanation: Option<ExplanationHandle>,
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
span、stable detail key、normalized details、canonical fix payload、explanation
handle identity、deduplicated ordinal からこれを導出する。

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
    anchor: SourceAnchor,
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

task 5 の span constructor は `start <= end`、`Span` 主位置の `role == Primary`、および
`secondary_spans` に `role == Primary` の entry が無いことを強制しなければならない。
file length や line-map membership は validate しない。それは source-map consumer の
責務である。既存のrange constructorではzero-width range は `zero_width` が `Some(Eof)` または
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
structured edit は `FixSuggestion` payload と `fix.md` に属する。

## Attachment Slots

`FixSuggestion` value は structured advisory payload であり、placeholder adapter ではない。
`fix.md` が定義する stable suggestion identity、optional producer identity、applicability、
safety、edit、command ref、snapshot/hash precondition を保持するが、edit の適用、LSP code
action の作成、command execution、artifact mutation は行わない。`ExplanationHandle` value は bounded
preview metadata を持つ structured lazy reference である。これらの attachment identity は、
aggregation と debug snapshot が deterministic に比較できるよう `stable_detail_key` と同じ ASCII
grammar を使い、人間向け text を identity として再解釈してはならない。

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

## Public Enum Compatibility

Task 18 はすべての failure-record public enums を downstream forward compatibility の
ため `#[non_exhaustive]` として mark する。

- `PipelinePhase`;
- `FailureCategory`;
- `StaleDiagnosticReason`;
- `DiagnosticFreshness`;
- `DiagnosticPrimaryLocation`;
- `DiagnosticSpanRole`;
- `SpanFreshness`;
- `ZeroWidthSpanIntent`;
- `DiagnosticNoteKind`;
- `DiagnosticDetailValue`;
- `DiagnosticRecordError`。

`FailureCategory` はさらに architecture 19 に従う。category name と phase ownership は
stable machine-readable semantics であり、category の追加または reclassification には
compatibility review が必要である。この marker は external exhaustive match が semver
trap になることを防ぐ。crate 内部の match は deliberate review が必要な箇所では exhaustive の
ままでよい。

## Boundary Rules

- record は proof、kernel、cache、artifact、driver、LSP の fact を記述してよいが、
  それらを決定しない。
- proof または kernel rejection record は proof status を決定しない。proof または
  kernel component がその decision を所有する。
- freshness state は snapshot や artifact を mutate しない。aggregation と consumer
  layer が publication を決定する。
- record は主位置と実在する副 `SourceRange` を保存し、LSP UTF-16 position は保存しない。
- record は compact structured detail を保存し、artifact manifest や cache mutation
  instruction は保存しない。

## Source-loading primary locations

`DiagnosticPrimaryLocation` は non-exhaustive な `Span(DiagnosticSpan)` または `SourceLoad { package_id: PackageId, path: NormalizedPath }`。draft 入力と record は `primary_location` を使用し、accessor はこの列挙型を返す。ソース読込み descriptor（現在 E0600–E0603）は後者と `PipelinePhase::SourceLoad`、`FailureCategory::SourceLoadError`（`source_load`、`source_load_error`）を必須とする。他コードはこの phase/category を使えず、Documentation のみ以下の専用非範囲位置も使用できる。空の package 識別子は不正。どちらの主位置でも副スパンの role 検証を維持する。debug は既存スパン表示を維持し、読込み位置の package/path をエスケープした文字列で表示する。

E0600 の構造化された失敗理由は `stable_detail_key` で表し、重複する reason field は要求しない。読込み主位置の debug 表示は基底文字列に対する `source_load(package={package:?},path={path:?})`。複数フィールドが不正な場合、拒否するが検証エラーの優先順位は規定しない。

## Frontend anchor adoption

`DiagnosticSpan::from_anchor` は [仕様22.1.2](../../../spec/ja/22.error_handling_and_diagnostics.md#2212-sourceスパンとコンテキストの表示) に従い既存のsession SourceAnchorを正本payloadとして保持する。`anchor()`で参照し、`range()`はconstな幾何的射影を返す。Range/Point形状と生成理由原文は任意の明示EOF・挿入意図と独立に保持する。rangeを重複保存せず、射影のためsessionの生成アンカー参照をconstにする。
既存range constructorの検証は維持する。`from_anchor`は生成形式を含むPoint・ゼロ長Rangeの意図未指定を許し、逆転範囲・非ゼロ位置への明示意図・未知アンカー形式を拒否する。GeneratedSpanOriginは既に非空白理由を要求する。ソース結合・本文長・UTF-8検証は後続の変換側・source-map利用側が所有する。
個々の副位置列の順序と重複をdraft/recordに保持する。debugは既存Range表示を維持し、Point・Generatedのpayloadを区別する。[集約](../en/aggregator.md#deduplication-identity) の幾何的識別子と正準代表選択を維持する。描画は幾何位置を使い、点には単一マーカーを付け、意図を推測しない。
テスト対象はアンカー各形式・理由原文・順序付き重複副位置・明示と未指定意図・逆転範囲・既存constructorの拒否・点描画。ソース結合済み変換・registry登録・生成元への採用は別作業とする。

## Documentation primary locations

[spec22 の位置](../../../spec/ja/22.error_handling_and_diagnostics.md#2212-sourceスパンとコンテキストの表示)に従い `DiagnosticPrimaryLocation` に `Documentation { publication_hash: Hash, package_id: String, package_version: Option<String>, lockfile_identity: Option<String>, module_path: String, language_edition: String, block_ordinal: u64 }` を直接追加する。Hash は認証済み publication 識別子で、5つの module フィールドは表示ラベルではなく任意値の区別を含む完全な識別を保持する。Block ordinal は保存されたソース順のゼロ始まり番号であり、byte offset や関連付け判断ではない。必須文字列は空白のみを認めず、識別子の trim・修復を行わない。
Documentation コードは実在する主 Span またはこの variant を許可し、`PipelinePhase::Documentation` と `FailureCategory::DocumentationWarning`（`documentation`、`documentation_warning`）を必須とする。他 family はこの位置・metadata を使用できない。draft/record/accessor/debug に全フィールドを保持し、wrapper・Artifact 依存・新しい SourceId・永続化形式を導入しない。既存の鮮度、副スパン、explanation 検証は維持する。範囲専用 consumer は未対応主位置を引き続き明示的に拒否する。

## Markdown/tag occurrence details
[spec22 §22.6.9](../../../spec/ja/22.error_handling_and_diagnostics.md#2269-documentation-markdown-and-tag-warnings) は両主位置形式の適用を W0405–W0411 に拡張する。問題の marker/slot/delimiter の最初の完全な連続写像区間を主位置とし、欠落 field は marker を使う。他区間は論理出現順の副スパンとする。conversion は独立に保存した有効な raw-block 写像を使い、使用可能な主位置がなければ block provenance を使う。除去した container prefix を架空の範囲で橋渡しせず、失敗した view 自身で位置を決めない。
`stable_detail_key = semantic_name`。`documentation.occurrence` は `List` `[publication, module, target, block, reason, offsets]`。publication は小文字 hash String、module は `[package_id, optional_version, optional_lock, module_path, edition]` とし、scalar はすべて String、option は空 List または String の単要素 List。target は不明なら空 List、検証済み module は `["module"]`、検証済み export は `["item", origin_id, fqn]` とし、外側の publication/module も完全な target identity に含む。block は先行ゼロなしの正規10進 u64 String。reason は descriptor の semantic-name String。offsets は Q3 抽出 block 内の半開 byte offset の正規10進 String 対 `[start, end]` で、除去した container prefix もこの座標系に含む。論理 occurrence がない conversion だけ空 List を許し、i64 への縮小や架空のゼロ offset は禁止する。
`documentation.raw` は math/conversion では空 List、tag は `[marker, post_marker_payload, duplicate_key]` とし、区切り・末尾 W と内部改行を含む Q3 原文を保持する。duplicate_key は未確立なら空、param は `[marker, raw_name]`、singleton は `[marker]`、repeatable は `[marker, post_marker_payload]`。String の `documentation.subreason` は missing_slot `name|reference|version`、missing_body `returns|requires|ensures|deprecated`、duplicate `param|singleton|repeatable`、metadata_mismatch `parameter_absent|no_return`、unmatched_math_delimiter `inline|display`、empty_display_math `display`、conversion_failure `conversion_failed|mapping_failed`。これらは翻訳しない machine token とする。
conversion 専用の `documentation.conversion_stage` は `commonmark|display_math|tag_fields|inline_math|source_map` とし最初の失敗段階を示す。写像合成失敗は source_map。その block の tag/math draft は確定せず、独立に確立した条件を上記と同一形式の `[occurrence, raw, subreason]` tuple の List として `documentation.retained_conditions` に保持する。論理 start/end を数値順、次に reason/subreason/raw を既存 value comparator 順に並べ、重複度を保持する。条件がなければ空 List。conversion occurrence は offset の有無と独立に block と stage/reason を識別し、他 block の警告は保持する。既存 details の producer 規約のみで、新しい共有 validator・serializer・parser 実装は導入しない。

## Shared link provenance

[採用済み link 詳細](./documentation_links.md#structured-identity-and-details) は既存 String/List 値を使い schema validator を追加しない。link reference は実 span または block provenance を使える。`DocumentationScope` は `Documentation` と同じ publication/module field を持ち block ordinal を持たず、同じ必須識別・Documentation metadata 検証を適用する。架空の座標なしに入力 subject を表す。
