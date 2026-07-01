# Diagnostic Sink

> 正本は英語です。英語版:
> [../en/sink.md](../en/sink.md)。

## 目的

この文書は `mizar-diagnostics` が所有する producer-side diagnostic sink を定義する。
sink は compiler phase が後続の aggregation に向けて validated `DiagnosticDraft` を
submit するための狭い境界である。sink は draft を collect し producer payload を保存
するだけであり、diagnostic rendering、record handle assignment、deduplication、LSP
diagnostic publication、artifact mutation、phase/proof/build status の決定は行わない。

## Scope

sink が所有するもの:

- 既に validated な `DiagnosticDraft` 値の受け取り。
- 後続 aggregation のための producer scope ごとの draft grouping。
- debugging と trace comparison のための producer scope 内の draft order の保存。
- field を変更せず collected draft を aggregator に公開すること。
- sealed 後の emit など sink misuse の rejection。

sink が所有しないもの:

- registry code allocation や message identity。
- `failure_record.md` が既に所有する draft construction rules。
- build-wide deterministic ordering、deduplication、`BuildDiagnosticIndex`。
- CLI formatting、LSP protocol conversion、JSON-RPC payload、code action。
- proof acceptance、trusted status、kernel acceptance、phase success。
- cache write、artifact mutation、driver session orchestration。

## Producer Scope

producer は 1 つの producer scope に bound された sink を受け取る。

```rust
struct DiagnosticProducerScope {
    phase: PipelinePhase,
    source_snapshot: BuildSnapshotId,
    producer_name: &'static str,
}
```

`phase` は provenance metadata であり、emit される各 draft の `phase` と一致しなければ
ならない。`source_snapshot` は producer が観測した snapshot であり、emit される各 draft
の `source_snapshot` と一致しなければならない。`producer_name` は test と trace output
のための stable debug metadata であり、user-facing identity ではない。

producer scope は 1 つの phase invocation に局所的である。cross-phase または build-wide
aggregation は task 9 の挙動であり、sink の挙動ではない。

## API Shape

task 7 implementation は次の conceptual API を公開するべきである。

```rust
trait DiagnosticSink {
    fn scope(&self) -> DiagnosticProducerScope;
    fn emit(&mut self, draft: DiagnosticDraft) -> Result<(), DiagnosticSinkError>;
    fn seal(&mut self) -> DiagnosticBatch;
}

struct DiagnosticBatch {
    scope: DiagnosticProducerScope,
    drafts: Vec<DiagnosticDraft>,
}
```

crate style に合う場合、具体的な Rust API は trait ではなく owned struct を使ってよい。
ただし behavior は同じでなければならない。producer は draft を append し、その後
1 つの immutable batch を aggregation へ渡す。

`emit` は sink-level invariant だけを validate する: sink が open であること、draft
phase が scope と一致すること、draft source snapshot が scope と一致すること。
`DiagnosticDraft` construction が既に強制した registry descriptor validation は再検証
しない。

失敗した `emit` call は non-mutating である。phase または snapshot mismatch により
draft が reject された場合、その draft は collect されず、既に collected な draft は
変更されず、sink が既に sealed されていない限り後続の valid draft を受け入れられる
open 状態のままである。

`seal` は sink を consume または close し、collected draft を変更せず返す。seal 後の
emit は失敗するか、ownership により不可能でなければならない。

## Emission Rules

producer がしなければならないこと:

- `failure_record` API を通じて draft を構築し、`DiagnosticCode`、span、detail、fix、
  explanation、freshness input が validate されるようにする。
- `phase` と `source_snapshot` が sink scope に一致する draft だけを emit する。
- `stable_detail_key` と structured detail を message text から独立させる。
- large trace ではなく compact structured reference を含める。
- sink order を local production order としてだけ扱う。

producer がしてはならないこと:

- CLI-rendered string、terminal style、line excerpt、underline を attach する。
- LSP range、UTF-16 offset、JSON-RPC payload、code-action object を attach する。
- artifact mutation instruction、cache write、driver event、scheduler command を sink
  経由で emit する。
- stale diagnostic を current output として publish する。
- diagnostic severity または category から proof acceptance、trusted status、
  kernel acceptance、phase success を推測する。

## Collection Semantics

`DiagnosticBatch` は seal 後 immutable である。field level で draft を byte-for-byte に
保存する。sink は次を行ってはならない。

- draft の sort。
- draft の deduplication。
- `DiagnosticId` または `DiagnosticHandle` の assignment。
- message、note、detail、fix、explanation handle の rewrite。
- 他の draft と似ているという理由で draft を drop すること。

aggregator は後で normalize、deduplicate、sort、handle assignment、stale publication
rejection、`BuildDiagnosticIndex` production を行ってよい。task 7 の tests は、sink が
input draft を正確に保存し、local production order を保持するが、それを global
publication order として扱わないことを確認するべきである。

## Error Handling

sink error は internal producer-boundary error である。test または developer diagnostic
として報告されてよいが、通常の user diagnostic ではなく、後続 registry task が明示的に
そう定義しない限り public `DiagnosticCode` を allocate してはならない。

初期 error case:

| Error | Rule |
|---|---|
| `SinkSealed` | seal 後の emission は reject されるか不可能である。 |
| `PhaseMismatch` | draft `phase` が scope `phase` と異なる。 |
| `SnapshotMismatch` | draft `source_snapshot` が scope `source_snapshot` と異なる。 |

これらの error は producer boundary だけを保護する。phase が fail したかどうかは決定
しない。phase status は scheduler/driver layer が所有する。
`PhaseMismatch` と `SnapshotMismatch` は recoverable producer-boundary error であり、
sink を seal したり、既に collected な draft を remove したりしてはならない。

## Determinism

sink は tests のために canonical `DiagnosticBatch::debug_snapshot()` string として
deterministic debug data を公開しなければならない。この string は LF line ending、
color 無し、localized field name 無しで、field order は次の通り。

1. `kind=batch`。
2. `phase`。
3. `source_snapshot`。`DiagnosticDraft::debug_snapshot()` と同じ形式で render する。
4. `producer_name`。Rust debug-string escaping で escape する。
5. `draft_count`。
6. local emission order の `draft[0]`、`draft[1]`、...。

各 `draft[n]` entry は、対応する `DiagnosticDraft::debug_snapshot()` から trailing newline
を削除し、内部 newline を `\n` として escape して embed する。empty batch は
`draft_count=0` として render し、`draft[n]` line を持たない。

この debug data は CLI rendering ではない。memory address、thread ID、map iteration
order、localized field name、明示的な local emission ordinal 以外の process-local
ordering を含めてはならない。

## Boundary Rules

- sink は draft だけを collect し、`DiagnosticRecord` は作らない。
- sink の local order は publication order ではない。
- sink は mismatched producer metadata を reject できるが、failure を success に変えたり
  success を failure に変えたりできない。
- sink は `mizar-driver`、`mizar-lsp`、`mizar-artifact`、または既存 resolver/frontend
  diagnostic type に依存しない。
- 既存 lexer/frontend/parser/resolver diagnostic migration は real consumer adoption seam
  が存在するまで deferred のままである。
