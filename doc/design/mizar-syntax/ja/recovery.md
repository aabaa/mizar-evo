# mizar-syntax: Recovery Nodes

> 正本は英語です。英語版: [../en/recovery.md](../en/recovery.md)。

状態: 欠落した構文要素、スキップされたトークン、対応しない区切り記号、不正な注釈の recovery 語彙は `mizar-syntax` に実装済み。parser による生成は段階的に追加する。

## 目的

このモジュールは、パーサー回復を構文上どう表現するかを定義する。

## 責務

- 欠落した構文要素、スキップされたトークン、対応しない区切り記号、不正なアノテーションを表現する。
- リゾルバとチェッカが明示的にスキップまたは却下できるよう、回復ノードに印を付ける。
- 診断のために元のソース範囲を保持する。

parser は現在、lexer error token の recovered token node と、`end` 欠落、文字列
リテラル欠落、task 5 の top-level skipped token の明示的な recovered node を生成する。
残りの recovery kind は、将来の parser grammar task が producer を追加するときに
syntax snapshot 語彙を変更しなくてよいよう、`mizar-syntax` で構築可能にしておく。

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
| `MissingSemicolon` | parser が `;` を必要とする top-level item boundary または EOF に到達した | 次の item または EOF へ継続する場合は設定する |
| `MissingStringLiteral` | parser が string-required context で欠落 string literal を挿入した | 挿入後に parsing を継続する場合は設定する |
| `MalformedImport` | parser task 6 が現在の import statement boundary で継続できる import 内部の不正構文を見つけた | `as` 後の alias 欠落や branch import の `}` 欠落などで、import item を表現したまま継続する場合は設定する |
| `MalformedExport` | parser task 7 が現在の export statement boundary で継続できる export 内部の不正構文を見つけた | `export` 後または `,` 後の module path 欠落などで、export item を表現したまま継続する場合は設定する |
| `MalformedVisibility` | parser task 7 が duplicate、dangling、または不正な top-level visibility marker を見つけた | visible item wrapper を表現したままにする場合は設定する。malformed tail token があれば、その内部で skip する |
| `MalformedTypeExpression` | parser task 8 が現在の reserve または type-argument boundary で継続できる malformed type syntax を見つけた | reserve/type node を表現したままにする場合は設定する。malformed tail token や欠落 delimiter は syntax recovery として保持する |
| `UnexpectedTopLevelToken` | parser task 5 が top-level item を開始できない source token を skip した | `SkippedToken` recovery node と skipped trivia range を生成する場合は設定する |
| `UnrecoverableInput` | parser が入力に対して信頼できる `SurfaceAst` を構築できない | 任意。parser が source edit を提案できる場合は設定し、parse result は `ast = None` になり得る |

diagnostic code 語彙は syntax-level に限る。名前解決、型検査、証明義務、意味的事実を
encode してはならない。

### recovery 語彙

`SyntaxRecoveryKind` は、consumer 前の syntax phase が約束する recovery category を
カバーする。「未生成」の kind は、対応する parser grammar task が producer を仕様化し
実装するまで、mizar-syntax の語彙としてのみ存在する。

| kind | producer condition | node 形状 | range と child rule | diagnostic / trivia の分担 | snapshot name |
|---|---|---|---|---|---|
| `ErrorToken` | parser が lexer-owned error-recovery token を受け取った | `SurfaceTokenKind::ErrorRecovery` を持つ recovered token node。または parser task が明示的 wrapper を必要とする場合の `SurfaceNodeKind::ErrorRecovery(ErrorToken)` | token 形では元 token range を使う。wrapper 形では同じ source range を使い、必須 child はない | `SyntaxDiagnosticCode::UnexpectedErrorToken`。raw token text は recovered token に残し、trivia には入れない | `ErrorToken` |
| `MissingEnd` | parser が block synchronization point に欠落 `end` を挿入した | 挿入 placeholder の `SurfaceNodeKind::ErrorRecovery(MissingEnd)` | insertion point の zero-width range。block opener / context child を insertion range の外に保持してよい | `SyntaxDiagnosticCode::MissingEnd`。同じ recovery が source text も skip する場合を除き skipped range はない | `MissingEnd` |
| `MissingStringLiteral` | parser が string-required context で欠落 string literal を挿入した | 挿入 placeholder | insertion point の zero-width range。必須 child はない | `SyntaxDiagnosticCode::MissingStringLiteral`。skipped range はない | `MissingStringLiteral` |
| `MissingItem` | 未生成。将来の module / item parser が top-level item を期待し、次の item boundary または EOF の前で同期する | 挿入 placeholder | insertion point の zero-width range。同期 token または包含 item list の任意 context child を持て、range 外でもよい | 専用 diagnostic code はまだない。producer task が user-facing diagnostic を出す前に code を追加するか既存 code 共有を明記する。skip された source は存在する場合 `SkippedTokenRange` に属する | `MissingItem` |
| `MissingTypeExpression` | parser task 8 が reserve `for` のような declaration binder の後、または bracket `type_arg_list` 内で type expression を期待した。`of` / `over` の term argument 欠落は missing type expression ではない | 挿入 placeholder | insertion point の zero-width range。任意の keyword / binder context child を持て、range 外でもよい | `SyntaxDiagnosticCode::MalformedTypeExpression`。純粋な挿入では skipped range はない | `MissingTypeExpression` |
| `MissingTerm` | 未生成。将来の term parser が term operand、argument、selector base、constructor field value を期待する | 挿入 placeholder | insertion point の zero-width range。任意の operator / call / context child を持て、range 外でもよい | 専用 diagnostic code はまだない。純粋な挿入では skipped range はない | `MissingTerm` |
| `MissingFormula` | 未生成。将来の formula parser が `st`、`holds`、connective、theorem / proof keyword などの後に formula を期待する | 挿入 placeholder | insertion point の zero-width range。任意の keyword / operator context child を持て、range 外でもよい | 専用 diagnostic code はまだない。純粋な挿入では skipped range はない | `MissingFormula` |
| `MissingStatement` | 未生成。将来の statement parser が proof、algorithm、block statement を期待し、次の statement boundary で同期する | 挿入 placeholder | insertion point の zero-width range。任意の preceding keyword または block context child を持て、range 外でもよい | 専用 diagnostic code はまだない。skip された source は存在する場合 `SkippedTokenRange` に属する | `MissingStatement` |
| `MissingProofStep` | 未生成。将来の proof parser が justification、inference step、case branch、proof-closing step を期待する | 挿入 placeholder | insertion point の zero-width range。任意の proof / block context child を持て、range 外でもよい | 専用 diagnostic code はまだない。skip された source は存在する場合 `SkippedTokenRange` に属する | `MissingProofStep` |
| `MissingAnnotationArgument` | 未生成。将来の annotation parser が string literal や bracket argument などの annotation argument を期待する | 挿入 placeholder | insertion point の zero-width range。任意の annotation marker / list context child を持て、range 外でもよい | 専用 diagnostic code はまだない。不正または skip された source は状況に応じて `MalformedAnnotation` または `Recovery` の `SkippedTokenRange` に属する | `MissingAnnotationArgument` |
| `SkippedToken` | parser task 5 が item boundary に到達するため 1 個以上の top-level token を skip する。将来の parser task はより狭い grammar boundary で同じ marker を使ってよい | skipped input の marker | skip された source span を覆う range。必須 child はない。root-listed token leaf を重複させない場合だけ任意の synchronization owner child を付けてよい | task 5 の top-level skip では `SyntaxDiagnosticCode::UnexpectedTopLevelToken`。skip span は `SurfaceTrivia::skipped_token_ranges` に `SkippedTokenReason::Recovery` で必ず記録する | `SkippedToken` |
| `UnmatchedOpeningDelimiter` | parser task 8 が同期点までに対応する `]` を持たない `[` type argument を見つけた、または future parser が別の opener に対応する closer を同期点 / EOF までに見つけられなかった | 通常は挿入された missing close と組になる marker | primary marker range は期待される closer または synchronization point の zero-width range。opener / context は、それが別の structural node にまだ所有されていない場合は recovery child として表せる。tree shape が non-root parent の重複を起こす場合は diagnostic secondary anchor として表す | task-8 type argument では `SyntaxDiagnosticCode::MalformedTypeExpression`。opener span は secondary diagnostic anchor にするべきで、skip text があれば trivia に属する | `UnmatchedOpeningDelimiter` |
| `UnmatchedClosingDelimiter` | 未生成。将来の parser が対応する opener を持たない closing delimiter を見つける | source text を囲む marker | unmatched closer token を覆う range。必須 child はない | 専用 diagnostic code はまだない。closer token は token stream に残し、それを超えて skip した token は trivia に属する | `UnmatchedClosingDelimiter` |
| `MalformedAnnotation` | 未生成。将来の annotation parser が valid annotation として parse できない annotation marker または body を認識する | source text を囲む marker | 不正な annotation marker / body span を覆う range。利用できる場合、任意の annotation owner child を付けてよい | 専用 diagnostic code はまだない。不正 source は `SurfaceTrivia::skipped_token_ranges` に `SkippedTokenReason::MalformedAnnotation` で必ず記録する | `MalformedAnnotation` |

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
raw skipped text を string payload として recovery node に encode してはならない。
文法 task は、その ownership を文書化し、rowan rendering が source token を一度だけ
出力できるよう recovery を non-recovery structural owner の内側に nest する場合に限り、
skip された token node を in-range recovery child として追加してよい。この場合でも
trivia は skipped span を記録する。

現在未生成の recovery kind を parser task が生成し始める場合、その task は producer
condition を精密化する、専用 `SyntaxDiagnosticCode` を追加する、または trivia
ownership rule をより具体化するなら、この表を更新しなければならない。

### 公開 enum の互換性

`SyntaxRecoveryKind` と `SyntaxDiagnosticCode` は、文法 recovery の成長に伴う将来の
variant を約束している。[todo.md](./todo.md) の consumer 前ゲートでは、これらを
下流 crate 向けに `#[non_exhaustive]` とし、lint-policy gate が、後続 task が
意図的な exhaustive decision を記録しない限り、これらの属性を固定する。
`mizar-syntax` 内部の match は exhaustive のままにし、recovery kind 追加時に
snapshot と diagnostic のローカル更新が強制されるようにする。`mizar-parser` を含む
下流 crate は、`#[non_exhaustive]` により必要になる箇所で wildcard fallback arm を
含めなければならない。
