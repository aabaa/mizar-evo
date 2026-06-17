# mizar-parser: Recovery

> 正本は英語です。英語版: [../en/recovery.md](../en/recovery.md)。

状態: task 12 の最小回復、task 28 の入れ子 block-end 回復、task 5 の
module-skeleton recovery、task 6 の import recovery、task 7 の export/visibility
recovery、task 8 の type-expression recovery、task 9 の primary-term recovery、
task 13 の atomic-formula recovery、task 14 の formula recovery、S-013 / S-014 の
statement/proof recovery、task 29 までの S-015 definition recovery、task 33 の
algorithm control-flow recovery、および task 34 の algorithm-verification recovery は
実装済み。完全な文法回復は引き続き計画中。

## 目的

このモジュールは、パーサーの同期点と回復方針を定義する。

## 責務

- `;`、`end`、トップレベル項目のキーワード、EOF などの安定した境界で同期する。
- 回復可能な構文構造を保持しながら、構文診断を出力する。
- 意味論的な事実を捏造せず、`mizar-syntax` の回復ノードを生成する。

現在の挙動:

- parser は private な有界先読み token cursor、既存の `SyntaxDiagnosticCode`
  variant を再利用する期待トークン診断 helper、同期集合、recovery node 送出
  helper を持つ。これらは内部基盤であり、crate root の公開 API を変更しない。
- 同期集合は `;`、`end`、EOF、および [grammar.md](./grammar.md) で文書化した
  task 5 の top-level dispatch start で停止する。具体的には `import`、`export`、
  `definition`、`reserve`、`registration`、`claim`、`theorem`、`lemma`、`open`、
  `assumed`、`conditional`、`private`、`public`、`infix_operator`、
  `prefix_operator`、`postfix_operator`、`synonym`、`antonym` である。後続の item
  文法タスクが concrete dispatch を追加するときに、この集合を拡張または絞り込む。
- 利用可能な `end` token を対応付けた後も parser の block stack が開いている場合、
  block 風キーワードに対する `end` 欠落を EOF で診断し、各欠落 close に明示的な
  recovered `MissingEnd` node を作る。diagnostic は block opener を secondary anchor
  として保持するが、recovery node 自体に必須の context child は持たせない。これにより
  後続の module skeleton node が source token を所有しても non-root parent が重複しない。
  現在の stack は top-level block と、それ自身の `end` を持つ algorithm control block を
  含む。parser task 33 は `if`、`while`、`for`、`match` の concrete statement-list
  recovery を所有し、recovery prepass は block-end matching 用の浅い構文 mirror を保つ。
  `for` は `for <identifier> = ...` / `for <identifier> in ...` の loop 風 token shape と、
  次の境界より前に `do` body marker が現れる malformed-head shape で開く。これにより
  formula quantifier が block end を消費しないようにする。`if` は明らかな
  algorithm/proof control introducer の後、または次の境界より前に
  `do` body marker が現れる場合に開く。`otherwise` は open algorithm block 内で完了済み
  match case の surface shape に合わせて `end` または `end;` の後にだけ開く。その
  algorithm prefix を持たない式レベル・definition 側の `otherwise` は開かない。`else if`
  は nested block opener ではなく、1 つの conditional chain として扱う。
- 合成の文字列必須 parser context で文字列リテラルが欠落した場合に診断し、明示的な recovered `MissingStringLiteral` node を作る。
- task 5 の module skeleton parsing は、top-level item semicolon 欠落を
  `MissingSemicolon` で診断し、unexpected top-level token を
  `UnexpectedTopLevelToken`、明示的な recovered `SkippedToken` node、および
  `SkippedTokenReason::Recovery` の `SurfaceTrivia::skipped_token_ranges` entry で
  表現する。
- task 6 の import parsing は、import prelude の後に現れる遅れた import を task 5 の
  skipped-token recovery path に乗せ、import statement の semicolon 欠落を
  `MissingSemicolon` で診断する。`as` の後の alias 欠落や branch import の `}` 欠落
  など、現在の statement boundary で継続できる import 内部の不正構文は
  `MalformedImport` で診断する。
- task 7 の export parsing は、export prelude の後に現れる遅れた export を task 5 の
  skipped-token recovery path に乗せ、export statement の semicolon 欠落を
  `MissingSemicolon` で診断する。`export` の後または comma の後の path 欠落など、
  現在の statement boundary で継続できる export 内部の不正構文は
  `MalformedExport` で診断する。task 7 の visibility parsing は、duplicate marker、
  dangling marker、非 theorem/notation top-level declaration への visibility を
  `MalformedVisibility` で診断する。
- task 8 の reserve / type-expression parsing は、reserve-hosted type syntax の不正を
  `MalformedTypeExpression` で診断する。`reserve ... for` の後、または bracket
  `type_arg_list` 内で純粋に type が欠落した場合は recovered `MissingTypeExpression` を
  作る。空でない malformed tail は、最も近い reserve/type node が所有する
  `SkippedToken` recovery を使う。bracket type-argument list が `]` より前に `;`、
  top-level item boundary、または EOF に到達した場合、`MalformedTypeExpression`、
  `[` への secondary anchor、`TypeArguments` 下の `UnmatchedOpeningDelimiter`
  recovery node を作る。
- task 9 の primary-term parsing は、malformed term-list と primary-term syntax を
  `MalformedTermExpression` で診断する。純粋な term argument 欠落では `MissingTerm` を
  挿入する。空でない malformed tail は、最も近い term node が所有する `SkippedToken`
  recovery を使ってよい。parenthesized、application、set-enumeration、reserved
  bracket-functor delimiter が期待する closer より前に synchronization に到達した場合、
  `MalformedTermExpression`、opener への secondary anchor、nearest term node 下の
  `UnmatchedOpeningDelimiter` recovery を作る。`the` の後に type expression を持たない
  `ChoiceTerm` は、欠落している child が choice term の type operand であるため、
  type-expression recovery（`MalformedTypeExpression` と `MissingTypeExpression`）を使う。
- task 10 の selector / update parsing は、malformed selector postfix、selector-call
  argument、functional update list を `MalformedTermExpression` で診断する。field-update
  value 欠落では `MissingTerm` を挿入する。selector-call と functional-update delimiter が
  期待する closer より前に synchronization に到達した場合、`MalformedTermExpression`、
  opener への secondary anchor、nearest selector/update term node 下の
  `UnmatchedOpeningDelimiter` recovery を作る。
- task 12 の operator parsing は、同一 operator の non-associative chain を
  `NonAssociativeOperatorChain` で報告する。dangling infix operator は
  `DanglingOperator` を報告し、operator を消費して partial left expression を表現したままにし、
  recovery node を必須とはしない。dangling prefix operator は `DanglingOperator` を報告し、
  `MissingTerm` operand を挿入して recoverable な `PrefixExpression` を保持する。
- task 13 の atomic-formula parsing は malformed atomic operand に term/type recovery を再利用する。
  built-in predicate application の right term 欠落は `MissingTerm` を挿入し、
  `MalformedTermExpression` を報告する。`is` assertion の body 欠落は
  `MissingTypeExpression` を挿入し、`MalformedTypeExpression` を報告する。
- task 14 の formula parsing は prefix `not`、binary connective、quantifier `st`、
  `holds` の後で formula が必要な場合に `MissingFormula` を挿入し、
  `MalformedFormulaExpression` を報告する。quantifier header は少なくとも 1 個の
  variable segment を表現した後に保持する。`be` / `being` 後の explicit type 欠落は
  `MissingTypeExpression` と `MalformedTypeExpression` を再利用し、malformed header
  separator や tail は `MalformedFormulaExpression` を報告する。matching `)` が
  synchronization 前にない parenthesized formula は `UnmatchedOpeningDelimiter` を送出し、
  `MalformedFormulaExpression` を報告し、opener を secondary diagnostic anchor とする。
- task 27 の redefinition と notation-alias parsing は definition-content
  synchronization を再利用する。redefinition label、subject、raw pattern、term body、
  raw notation-pattern side の欠落は、挿入 child が必要な場合に
  `MalformedTermExpression` と `MissingTerm` を使う。`redefine func` return type 欠落は
  `MalformedTypeExpression` と `MissingTypeExpression` を使う。delimiter、formula body、
  notation の `for`、必須 `coherence` keyword 欠落は `MalformedFormulaExpression` を
  使う。mandatory coherence justification と任意の `with` label の malformed syntax は、
  必要に応じて `MalformedJustification` と `MissingProofStep` を使う。malformed tail は
  semicolon、`end`、次の definition-content start、top-level item boundary、または EOF まで
  skip する。
- task 28 の property-clause parsing は definition-content synchronization を再利用する。
  mandatory property justification の欠落または malformed syntax は、proof placeholder
  が必要な場合に `MalformedJustification` と `MissingProofStep` を使う。malformed
  property tail は semicolon、`end`、次の definition-content start、top-level item
  boundary、または EOF まで skip する。property semicolon 欠落は `MissingSemicolon` を
  使い、別の property 句を含む後続 definition item を保持する。
- task 29 の structure / inheritance parsing は、`struct ... end` と explicit
  `inherit ... where ... end` block 内で local member synchronization を使い、block
  boundary では definition-content synchronization へ戻る。空または malformed な
  structure pattern、inheritance target、member name、redefinition name、malformed
  member tail は、inserted placeholder が必要な場合に `MalformedTermExpression` と
  `MissingTerm` を使う。member または redefinition type 欠落は
  `MalformedTypeExpression` と `MissingTypeExpression` を使う。inheritance coherence
  justification の欠落または malformed syntax は `MalformedJustification` と
  `MissingProofStep` を使う。inheritance の `coherence with ...` は受理せず recovered
  syntax として扱う。member semicolon と外側 semicolon の欠落は `MissingSemicolon` を
  使い、block closer 欠落は `MissingEnd` を使う。malformed member tail は semicolon、
  `field`、`property`、`coherence`、`end`、次の definition-content start、top-level item
  boundary、EOF まで skip する。frontend-facing な scope skeleton は nested `struct`
  block を認識し、`inherit` は statement semicolon または `end` より前に `where` がある
  場合だけ block-like として扱う。
- task 34 の algorithm verification parsing は、重複または順序違反の header clause を
  algorithm body boundary まで skip して recover する。`requires`、`ensures`、loop
  `invariant`、`assert` の formula 欠落には `MissingFormula` を挿入し、空または
  dangling な `decreasing` measure には `TermList` 内の `MissingTerm` を挿入する。
  `for ... do decreasing ...;` は clause semicolon まで skipped-token recovery して
  reject し、通常の loop-body statement 後の `invariant` / `decreasing` は misplaced
  algorithm statement として clause semicolon で recover する。
- 対応する block opener を持たない裸の `end` は、構文診断とともに `ast = None` を返す。

## 公開 enum の互換性

`StringRequiredContext` は downstream crate 向けに `#[non_exhaustive]` とする。現在の
parser behavior は `None` と合成の `UniformForTest` context だけを区別するが、実際の
grammar growth では operator declaration と annotation argument の parser-facing
string-required position が追加される。downstream match は wildcard fallback arm を
持たなければならない。一方、`mizar-parser` 内部の match は exhaustive のままにし、
新しい context が追加されたときに recovery と token adaptation の更新がローカルに
強制されるようにする。
