# mizar-syntax: Recovery Nodes

> 正本は英語です。英語版: [../en/recovery.md](../en/recovery.md)。

状態: task 12 の最小回復ノードは実装済み。完全な回復語彙は計画中。

## 目的

このモジュールは、パーサー回復を構文上どう表現するかを定義する。

## 責務

- 欠落した構文要素、スキップされたトークン、対応しない区切り記号、不正なアノテーションを表現する。
- リゾルバとチェッカが明示的にスキップまたは却下できるよう、回復ノードに印を付ける。
- 診断のために元のソース範囲を保持する。

現在の最小語彙には、lexer error token の recovered token node と、`end` 欠落および文字列リテラル欠落の明示的な recovered node が含まれる。より広い skipped-token、対応しない区切り記号、不正なアノテーションの回復は引き続き計画中である。

## Public API

### Syntax diagnostic

`SyntaxDiagnostic` は、任意の `SurfaceAst` とともに運ばれる parser-facing な
diagnostic record である。

| field | contract |
|---|---|
| `code` | orchestration と test が使う安定した syntax diagnostic category |
| `message` | 人間向けの parser diagnostic text。安定した machine key ではない |
| `primary` | diagnostic の主 highlight を置く source range |
| `secondary` | opener / context / candidate span のための任意の source anchor |
| `recovery_note` | parser が継続できた recovery action の短い説明。任意 |

`SyntaxDiagnostic::new` は secondary anchor と recovery note を持たない diagnostic
を作る。`with_secondary` は既存の secondary anchor を置き換えずに追加する。
`with_recovery_note` は parser recovery advice または action text を記録する。
recover せず parsing を中止する diagnostic では未設定のままでよい。

現在の `SyntaxDiagnosticCode` は次のとおり。

| code | producer condition | recovery note expectation |
|---|---|---|
| `UnexpectedErrorToken` | parser が lexer-owned error-recovery token を受け取った | 任意。recovered token 自体が lexer input を保持する |
| `DanglingOperator` | Pratt expression parsing が必要な operand を持たない operator を見つけた | 任意。recovery node は必須ではない |
| `NonAssociativeOperatorChain` | parser が non-associative operator contract に反する chain を見つけた | 任意。recovery node は必須ではない |
| `MissingEnd` | parser が同期点に欠落 `end` を挿入した | 挿入後に parsing を継続する場合は設定する |
| `MissingStringLiteral` | parser が string-required context で欠落 string literal を挿入した | 挿入後に parsing を継続する場合は設定する |
| `UnrecoverableInput` | parser が入力に対して信頼できる `SurfaceAst` を構築できない | 任意。parser が source edit を提案できる場合は設定し、parse result は `ast = None` になり得る |

diagnostic code 語彙は syntax-level に限る。名前解決、型検査、証明義務、意味的事実を
encode してはならない。

### 現在の recovery 語彙

`SyntaxRecoveryKind` は現在、task 12 の最小語彙を持つ。

| kind | producer 上の意味 | node 形状 | diagnostic code |
|---|---|---|---|
| `ErrorToken` | lexer error-recovery token が syntax stream に保持された | `SurfaceTokenKind::ErrorRecovery` を持つ recovered token node。または parser task が明示的 wrapper を必要とする場合の recovery node | `SyntaxDiagnosticCode::UnexpectedErrorToken` |
| `MissingEnd` | parser が同期点に欠落 `end` を挿入した | zero-width insertion range と任意の opener / context child を持つ `SurfaceNodeKind::ErrorRecovery(MissingEnd)` | `SyntaxDiagnosticCode::MissingEnd` |
| `MissingStringLiteral` | string-required context で parser が欠落 string literal を挿入した | zero-width insertion range を持ち、必須 child を持たない `SurfaceNodeKind::ErrorRecovery(MissingStringLiteral)` | `SyntaxDiagnosticCode::MissingStringLiteral` |

recovered node は `recovered = true` でなければならない。recovered token は元の
token text と source range を保持し、診断、formatter recovery、LSP 機能が捏造
された text ではなくユーザー入力を表示できるようにする。

### range と child の規則

recovery range は、`SurfaceAst` と同じ source に属する source-local byte range
である。

- 挿入された欠落 construct は、挿入位置の zero-width range を使う。
- recovered lexer token は、元の token range を使う。
- recovery node は、自身の range の外にある context child を保持してよい。
  互換 view は診断と parser test のためにそれらの child を保持するが、rowan
  green tree は source-shaped であり続けるため、out-of-range recovery child を
  省略する。
- 通常の non-recovery node は、引き続きすべての child range を包含するべきである。
  将来の recovery 例外は、それを必要とする recovery kind の隣に文書化しなければ
  ならない。

### recovery と trivia の分担

recovery node は、parser consumer が気付くべき構文上の placeholder または marker
を表す。`SurfaceTrivia::skipped_token_ranges` は、診断、整形、code action のための
skip された source span と任意の owner を表す。recovery strategy が placeholder
挿入と source text の skip の両方を行う場合、placeholder は
`SurfaceNodeKind::ErrorRecovery` に属し、skip された span は trivia に属する。
raw skipped text を recovery node に encode してはならない。

### 拡張時の契約

recovery 語彙の拡張は [todo.md](./todo.md) の task 5 である。新しい
`SyntaxRecoveryKind` はそれぞれ次を指定しなければならない。

- それを生成する parser condition。
- node が挿入 placeholder、source text の wrapper、skipped input の marker の
  どれであるか。
- range と child-role rule。out-of-range context child を許すかどうかも含む。
- 対応する `SyntaxDiagnosticCode`、または既存 code を共有する理由。
- skip された source span も `SkippedTokenRange` として記録するかどうか。
- snapshot rendering name と、少なくとも 1 つの構築可能な test fixture。

計画中の category は次のとおり。

| category | 実装前に必要な仕様 |
|---|---|
| missing constructs | downstream phase が区別する必要のある construct family ごとの kind、insertion range、context-child role |
| skipped tokens | owner selection、skipped range ownership、`SkippedTokenReason::Recovery` との相互作用 |
| unmatched delimiters | opener / closer context role と primary diagnostic anchor |
| malformed annotations | annotation range ownership と `SkippedTokenReason::MalformedAnnotation` との相互作用 |

### 公開 enum の互換性

`SyntaxRecoveryKind` と `SyntaxDiagnosticCode` は、文法 recovery の成長に伴う将来の
variant を約束している。[todo.md](./todo.md) の consumer 前ゲートでは、後続 task が
意図的な exhaustive decision を記録しない限り、これらを下流 crate 向けに
`#[non_exhaustive]` とするべきである。`mizar-syntax` と `mizar-parser` 内部の
match は exhaustive のままにし、recovery kind 追加時に snapshot と diagnostic の
ローカル更新が強制されるようにする。
