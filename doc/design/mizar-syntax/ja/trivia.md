# mizar-syntax: Trivia

> 正本は英語です。英語版: [../en/trivia.md](../en/trivia.md)。

状態: task 4 の trivia model は実装済み。parser が生成する item への attachment
fixture は、最初の item-node 増分で追加予定。

## 目的

このモジュールは、診断、整形、ドキュメント生成、LSP 機能のために保持する、構文に隣接した trivia を定義する。

`mizar-syntax` は、trivia の構文向けモデルだけを所有し、コメント抽出は所有しない。
`mizar-frontend::PreprocessedSource` は、コメントとドキュメントコメントの抽出、
raw doc-comment body、字句用テキスト、preprocess map の所有者であり続ける。
`SurfaceAst` は、コメントの range / kind、doc-comment attachment target、
skipped-token range、空白依存 hint という、`SourceRange` ベースの構文 hint だけを保持する。

## 責務

- raw comment body をコピーせず、コメント range と構文的な attachment hint を保持する。
- doc-comment attachment target を構文的に表し、意味的な解釈を持ち込まない。
- skipped token range と recovery の所有 hint を保持する。
- formatter と LSP consumer が必要とする空白依存 hint を保持する。
- 必要に応じて trivia を決定的に snapshot rendering できるようにする。

## Public API

### Storage Boundary

`SurfaceTrivia` は `SurfaceAst` が持つ immutable な trivia side table である。
次の要素を含む。

| Field | 意味 |
|---|---|
| `comments` | `CommentKind` 付きの非ドキュメント／ドキュメントコメント range。raw text は frontend が所有する |
| `doc_comment_attachments` | doc-comment の source range から node、token、または detached source anchor への構文 attachment hint |
| `skipped_token_ranges` | recovery 中に skip された source range。任意の構文 owner と reason を持つ |
| `whitespace_hints` | formatting、code action、token separation に影響する空白の source range |

`SurfaceTriviaBuilder` は side table を構築し、`finish` 前に source-local range と
kind で entry を整列する。これにより snapshot は構築順に依存しない。すべての
range は trivia table と同じ `SourceId` に属さなければならない。

各 table 内の sorting は決定的である。

- comment は range start、range end、`CommentKind` の順（`SingleLine`、
  `MultiLine`、`Documentation`、未知 kind は最後）で並ぶ。
- doc-comment attachment は range start、range end、placement（`Leading`、
  `Trailing`）、target key の順で並ぶ。
- skipped range は range start、range end、reason（`Recovery`、
  `MalformedAnnotation`、`UnexpectedToken`）、任意の owner key の順で並ぶ。
  owner がある entry は `None` より前に並ぶ。
- whitespace hint は range start、range end、hint kind（`RequiresSeparation`、
  `LineBreakBefore`、`LineBreakAfter`、`SyntheticBoundary`）の順で並ぶ。

target key は node target、token target、detached anchor の順で並ぶ。node / token
target はその後、互換 id index、range start、range end の順で並ぶ。detached anchor
は描画 key prefix と local data によって、generated anchor、point、range、unknown
anchor の順で並ぶ。generated anchor は anchor range または point の後に
generated-origin reason を含める。target key に生の `SourceId` debug output を
含めてはならない。

### Ownership Split

frontend preprocessing 層は、loaded source からコメントとドキュメントコメントを
抽出し、doc-comment body を保存し、コメントを `lexical_text` から取り除き、
source map を保持する。syntax trivia は、そのデータを `SourceRange` と
attachment target で参照するだけである。この分担により、コメントだけの編集で
token stream が変わらない場合に parser output を再利用でき、後続の attachment
step が frontend 所有の `PreprocessedSource` から trivia hint を再構築できる。

### Attachment Targets

`TriviaAttachmentTarget` には 3 つの形がある。

- `Node(TriviaNodeTarget)`: token ではない構文 node への attachment。
- `Token(TriviaNodeTarget)`: token-local な leading / trailing attachment。
- `Detached(SourceAnchor)`: trivia は保持するが、所有する syntax node がない場合。

`TriviaNodeTarget` は、互換用の `SurfaceNodeId` と target node の `SourceRange` の
両方を保持する。Trivia を `SurfaceAst` に付けるとき、
`SurfaceAst::with_trivia` は、各 node / token target がその AST 内に存在すること、
保存された range が target node と一致すること、node target が token node を
指さないこと、token target が token node を指すことを検証する。Detached source
anchor は、source range や point へ戻る generated anchor も含め、同じ source に
属さなければならない。

`TriviaPlacement` は attachment が leading か trailing かを記録する。doc comment
が直後の item node に付く関係は構文的なものである。documentation generator は
後で comment body を解釈してよいが、その意味は `SurfaceAst` に入らない。

### Skipped Ranges

`SkippedTokenRange` は、skip された source range、任意の owner target、
`SkippedTokenReason`（`Recovery`、`MalformedAnnotation`、`UnexpectedToken`）を
記録する。Recovery node は引き続き自身の `recovered` flag を持つ。trivia side
table は、診断、formatter、LSP code action のために skip された source span を
保持する。

### Whitespace Hints

`WhitespaceHint` は、必須の token separation、構文要素の前後の line break、
preprocessing が導入した synthetic boundary などの range-based hint を記録する。
これらの hint は formatting-sensitive な事実を保持するが、空白を意味論的入力にはしない。

### Snapshot Rendering

`SurfaceAst::snapshot_text` は task 3 の syntax-only baseline format のままにする。
`SurfaceAst::snapshot_text_with_trivia` は、テストや corpus baseline が trivia
ownership と attachment を検査する必要がある場合に、決定的な `trivia:` section
を追加する。Trivia snapshot は source-local byte range、kind name、attachment
target、skipped reason、whitespace hint kind を出力し、raw comment text、file
path、source-id debug output、rowan identity は出力しない。

現在の trivia snapshot section は次のとおり。

```text
trivia:
  <entry-or-none>
```

entry 行は次の形を使う。

```text
Comment kind=<CommentKind> range=<start>..<end>
DocComment range=<start>..<end> placement=<TriviaPlacement> target=<target>
SkippedTokens reason=<SkippedTokenReason> range=<start>..<end> owner=<target-or-none>
WhitespaceHint kind=<WhitespaceHintKind> range=<start>..<end>
```

target は `node:range:<start>..<end>`、`token:range:<start>..<end>`、
`detached:range:<start>..<end>`、`detached:point:<offset>`、
`detached:generated`、または `detached:unknown` として描画する。missing node /
token target は防御的 snapshot rendering では `<missing>` として描画するが、通常の
snapshot が生成される前に `SurfaceAst::with_trivia` がそのような target を拒否
しなければならない。

### 公開 enum の互換性

`TriviaAttachmentTarget`、`TriviaPlacement`、`SkippedTokenReason`、
`WhitespaceHintKind` は、parser、frontend、formatter、LSP 層が trivia ownership を
共有するため公開されている。[todo.md](./todo.md) の consumer 前ゲートでは、成長し得る
enum（`TriviaAttachmentTarget`、`SkippedTokenReason`、`WhitespaceHintKind`）を、
所有タスクが意図的な exhaustive decision を記録しない限り、下流 crate 向けに
`#[non_exhaustive]` とするべきである。`TriviaPlacement` は、leading / trailing
placement が閉じた二分の構文関係であるため、現時点では exhaustive のままと想定する。
具体的な middle / detached placement が設計された場合だけ、この判断を見直す。
内部 match は exhaustive のままにする。
