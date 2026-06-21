# mizar-resolve: recovered 構文ポリシー

> 正本: 英語版 [../en/recovery.md](../en/recovery.md)。

状態: resolver task R-022 以前について実装済み。この文書は
`mizar-syntax` の recovery contract から導く derived design であり、新しい
parser recovery 挙動を導入しない。

## 目的

`mizar-syntax` は recovery producer、recovery kind、syntax diagnostic、
`SurfaceAst` node/token の `recovered` flag を所有する。`mizar-resolve` は
その marker を消費し、recovered 構文を semantic に有効な source とみなさず、
deterministic に後続処理を継続する。

resolver のポリシーは次のとおり。

- navigation、invalidation、downstream の degraded fact に役立つ表現済み
  source shape は保持する。
- recovery により必須 source 情報が隠れている場合、identity、signature、
  relation edge を創作しない。
- parser diagnostic は syntax diagnostic のまま保持し、public resolver
  diagnostic code へ変換しない。
- recovered syntax root を繰り返すだけの dependent semantic diagnostic は抑制する。

## stage ごとの扱い

| Stage | recovered input の扱い | diagnostic rule |
|---|---|---|
| import path candidates | parser が directive を表現できた場合、candidate を `RecoveredSyntax` の unresolved import として保持する。recovered candidate から graph edge は追加しない。 | public resolver code は割り当てない。後続の name diagnostic が unresolved import に依存する場合はあるが、recovered import candidate を独立した user-facing resolver diagnostic へ昇格しない。 |
| declaration shell と export shell | shell 自身、transparent wrapper、または descendant に recovery があれば、表現済み declaration/export shell を recovered として保持する。deterministic identity のない shell は shell-only のままにする。 | declaration collection 自体は diagnostic を出さない。 |
| namespace paths | `RecoveredSyntax` の unresolved namespace record として保持し、recovered namespace candidate 経由では resolve しない。 | recovered namespace record は internal name report の diagnostic root にしない。 |
| ordinary names と dot chains | 状況に応じて `NameRefTable` entry を unresolved または checker-deferred として保持する。recovered dot-chain は、元 candidate が local recovered flag だけを持つ場合でも reference origin を recovered にする。 | recovered reference origin は internal name diagnostic root / cascade から除外する。 |
| label references | reference origin が recovered、または spelling が空の場合、unresolved label-reference table entry を保持する。 | R-022 では label-reference failure 用の独立 diagnostic report はない。 |
| label declarations | deterministic な origin path が残る場合、label projection と label index fact は保持するが、clean label-reference lookup の candidate には recovered projection を使わない。 | recovered label projection は duplicate/conflicting-label diagnostics から除外する。 |
| symbol declarations | identity data が deterministic に残る場合、symbol/definition は保持する。ただし recovered origin、local-only export status、malformed `recovered-shell` signature を使う。context-only recovered shell は shell-only に留める。 | recovered symbol projection は duplicate / illegal-overload diagnostics から除外する。definition conflict は `RecoveredShell` のみ残す。 |
| lexical summaries | parser-backed extractor が lexer-visible notation として marker を付けた projection のうち、collected shell が recovered でなく export-visible なものだけを seed する。 | lexical summary seed をスキップしても resolver diagnostic は出さない。 |

## 境界ルール

- resolver code は skipped source text を調べて欠けた構文を推測してはならない。
- resolver code は parser recovery node を作成したり、`SyntaxRecoveryKind` を
  変更したりしてはならない。
- R-G001 が未解決の間、resolver diagnostics は crate-local/internal のままにする。
  このポリシーは public resolver diagnostic code を割り当てない。
- downstream の checker/type/proof phase は recovered fact を reject または skip
  してよいが、resolver は degraded input を認識できる provenance を保持する。

## テスト

R-022 の unit coverage は次を検証する。

- recovered declaration / symbol input で collection が panic しない。
- recovered namespace/name/dot-chain input は resolver table に保持されるが、
  internal name diagnostic root にならない。
- recovered label declaration は duplicate/conflict diagnostics を生成しない。
- recovered symbol declaration は duplicate / illegal-overload diagnostics を生成せず、
  `RecoveredShell` conflict metadata は保持される。
