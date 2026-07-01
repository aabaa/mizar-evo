# Structured Fix Suggestions

> 正本は英語です。英語版:
> [../en/fix.md](../en/fix.md)。

## 目的

この文書は `mizar-diagnostics` が所有する structured fix suggestion を定義する。fix
suggestion は `DiagnosticRecord` に attached される bounded で machine-readable な projection
である。source edit または opaque command reference を記述できるが、それ自身では edit を
適用しない。

fix suggestion は stable suggestion identity と `DiagnosticCode` を key にする。human title、
`help:` text、localized wording、CLI rendering は identity ではない。

## Scope

fix module が所有するもの:

- diagnostic record から参照される stable `FixSuggestionId`。
- title、applicability、safety classification、edit list を持つ structured fix payload。
- text edit 用の compiler-native source range。
- safe validation のための optional expected text と snapshot/hash precondition。
- fix payload の deterministic ordering と debug snapshot。
- CLI rendering と LSP code-action conversion が読める projection data。

fix module が所有しないもの:

- automatic edit application や workspace mutation。
- LSP `CodeAction` または `WorkspaceEdit` protocol object。
- current-buffer validation や command execution。
- source loading、line map、path normalization、UTF-16 conversion。
- proof acceptance、phase status、kernel acceptance、driver orchestration、artifact mutation。

## Data Model

task 13 は diagnostic record 上に次と等価な形の structured fix payload を保存する。

```rust
struct FixSuggestion {
    id: FixSuggestionId,
    producer_key: Option<String>,
    title: String,
    applicability: FixApplicability,
    safety: FixSafety,
    edits: Vec<FixEdit>,
    command: Option<FixCommandRef>,
    required_snapshot: Option<BuildSnapshotId>,
    required_text_hash: Option<Hash>,
}

struct FixEdit {
    range: SourceRange,
    replacement: String,
    expected_text: Option<String>,
}
```

`FixSuggestionId` は diagnostic draft または record 内で stable であり、aggregation 前に
explicit producer key または deterministic producer-local ordinal から割り当てられる。
`DiagnosticHandle` に依存してはならない。handle は aggregation と deduplication の後にだけ
割り当てられるためである。published record は convenience のため、fix payload の隣に containing
`DiagnosticHandle` を projection してよいが、その back-reference は pre-publication fix identity
の一部ではない。

`producer_key` は、deterministic ordinal だけでは記述性が足りない場合に producer が供給する
optional structured identity string である。human-facing text ではない。durable consumer は
`DiagnosticCode`、structured diagnostic detail field、fix identity string を key にしなければ
ならず、title や rendered wording を key にしてはならない。

`title` は human-facing である。変わってよく、identity として使ってはならない。`command` は
後続 owner のための opaque reference であり、`mizar-diagnostics` の中で実行してはならない。

## Applicability And Safety

初期 applicability level:

| Applicability | Meaning |
|---|---|
| `MachineApplicable` | すべての precondition がまだ成り立つ場合、edit は機械的に正しいと期待される。 |
| `MaybeIncorrect` | edit は plausible だが review が必要である。 |
| `HasPlaceholders` | replacement に user が埋める placeholder が含まれる。 |
| `Informational` | direct edit はなく、help text として render される。 |

初期 safety class:

| Safety | Rule |
|---|---|
| `LocalTextEdit` | すべての edit が explicit source range を target にし、expected text を current source text に対して validate できる。 |
| `SnapshotBound` | suggestion は `required_snapshot` に対してだけ valid である。 |
| `ArtifactAssisted` | suggestion は外部 owner が確認する artifact/source hash に依存する。 |
| `CommandOnly` | text edit は提供されない。後続 command owner が command ref を解釈してよい。 |

どの applicability level も、この crate に automatic application を許可しない。
`MachineApplicable` であっても「precondition が成り立つ場合に offer して安全」という意味であり、
「confirmation なしで apply する」という意味ではない。

## Edit Rules

各 `FixEdit` は次を満たさなければならない。

- `SourceId`、start、end byte offset を持つ compiler-native `SourceRange` を使う。
- `start <= end` を validate する。
- replacement text を UTF-8 として carry する。
- current-buffer validation のために optional `expected_text` を carry してよい。
- 後続 spec が deterministic merge rule を定義しない限り、同じ suggestion 内の別 edit と
  overlap しない。

task 13 は、1 つの `SourceId` 内で edit range が overlap する suggestion を reject するべきである。
複数 edit は deterministic snapshot のために source key、start、end、replacement text で order
される。

fix module は byte range を line/column や LSP UTF-16 coordinate に変換しない。CLI rendering は
payload から `help:` text を derive してよい。LSP conversion は `mizar-lsp` が所有する。

## Attachment To Records

fix suggestion は compact structured payload または stable handle として `DiagnosticRecord` に
attach される。次を preserve しなければならない。

- 所属する diagnostic code と handle。
- stable suggestion id。
- applicability と safety。
- edit range と replacement text。
- expected text と snapshot/hash precondition。

aggregation deduplication は canonical fix payload を diagnostic identity に含めなければならない。
canonical fix payload は `FixSuggestionId`、`producer_key`、applicability、safety、
`expected_text` を含む ordered edit、optional command reference、snapshot/hash precondition で
構成される。canonical fix payload field が 1 つでも異なる場合、diagnostic は distinct に保たなければ
ならない。human title、message text、rendered `help:` line、localized wording は deduplication
key ではない。

## Debug Snapshot

task 13 は deterministic fix debug snapshot を公開するべきであり、field order は次の通り。

1. `kind=fix`。
2. `id`。
3. `producer_key`。
4. `diagnostic`。
5. `title`。Rust debug-string escaping で escape する。
6. `applicability`。
7. `safety`。
8. ordered edits。
9. optional command。
10. snapshot/hash precondition。

snapshot は test/debug data であり、CLI rendering でも LSP code action でもない。memory
address、map iteration order、localized field name、process-local ordering を含めてはならない。

## Boundary Rules

- fix suggestion は advisory である。source text や artifact を mutate しない。
- `mizar-diagnostics` は payload shape と deterministic ordering を validate してよいが、
  current editor buffer は validate しない。
- CLI rendering は fix から derived した help text を表示してよいが、それを apply しない。
- LSP code-action conversion、current-buffer revalidation、command execution は `mizar-lsp`
  または driver layer に属する。
- proof/kernel/trusted acceptance は、fix suggestion が存在するか、または user が受け入れたかに
  依存してはならない。
