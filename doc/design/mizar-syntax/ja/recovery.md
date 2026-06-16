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
| `DanglingOperator` | Pratt expression parsing が必要な operand を持たない operator を見つけた | infix operator では任意。prefix-operator recovery では挿入された `MissingTerm` operand と組み合わせてよい |
| `NonAssociativeOperatorChain` | parser が non-associative operator contract に反する chain を見つけた | 任意。recovery node は必須ではない |
| `MissingEnd` | parser が同期点に欠落 `end` を挿入した | 挿入後に parsing を継続する場合は設定する |
| `MissingSemicolon` | parser が `;` を必要とする top-level item boundary、definition-content boundary、block boundary、または EOF に到達した | 次の item、次の definition content、block closer、または EOF へ継続する場合は設定する |
| `MissingStringLiteral` | parser が string-required context で欠落 string literal を挿入した | 挿入後に parsing を継続する場合は設定する |
| `MalformedImport` | parser task 6 が現在の import statement boundary で継続できる import 内部の不正構文を見つけた | `as` 後の alias 欠落や branch import の `}` 欠落などで、import item を表現したまま継続する場合は設定する |
| `MalformedExport` | parser task 7 が現在の export statement boundary で継続できる export 内部の不正構文を見つけた | `export` 後または `,` 後の module path 欠落などで、export item を表現したまま継続する場合は設定する |
| `MalformedVisibility` | parser task 7 が duplicate、dangling、または不正な top-level visibility marker を見つけた | visible item wrapper を表現したままにする場合は設定する。malformed tail token があれば、その内部で skip する |
| `MalformedTypeExpression` | parser task 8 が現在の reserve または type-argument boundary で継続できる malformed type syntax を見つけた。parser task 11 が `qua` target type の欠落または malformed target を見つけた。parser task 14 が `be` / `being` 後の明示的 quantifier type 欠落を見つけた。parser task 15 が comprehension `is` 後の明示的 generator type 欠落を見つけた。parser task 16 が statement-level `be` / `being` 後の明示 type 欠落を見つけた。parser task 18 が `consider` variable type または `reconsider ... as` target type の欠落を見つけた。parser task 21 が inline-definition parameter type または `deffunc` return type の欠落を見つけた。parser task 23 は通常 definition parameter で qualified-variable recovery を再利用する。parser task 25 が `->` 後の `func` return type 欠落を見つけた。parser task 26 が `is` 後の `mode` body type 欠落を見つけた。parser task 27 が `->` 後の `redefine func` return type 欠落を見つけた。parser task 29 が `->` 後の structure field / property type または inheritance member narrowed type 欠落を見つけた | reserve/type/`QuaExpression`、quantifier variable-segment、comprehension variable-segment、statement または definition qualified-variable segment、`ReconsiderStatement` target type、`TypedParameter`、`InlineFunctorDefinition` return type、`FunctorDefinition` または `FunctorRedefinition` return type、`ModeDefinition` body type、`StructureField` / `StructureProperty` member type、または inheritance member narrowed type を表現したままにする場合は設定する。malformed tail token、欠落 type operand、欠落 delimiter は syntax recovery として保持する |
| `MalformedTermExpression` | parser task 9 が現在の term-list または delimiter boundary で継続できる malformed primary term syntax を見つけた。parser task 10 が malformed selector/update postfix syntax を見つけた。parser task 15 が missing mapper/generator structure や `}` 欠落などの malformed set-comprehension syntax を見つけた。parser task 16 が `take` witness、`set` equating identifier / RHS、または simple-statement tail の欠落や malformed syntax を見つけた。parser task 18 が `ReconsiderItem` identifier または equated term の欠落や malformed syntax を見つけた。parser task 19 が iterative equality の initial term または `.=` step term の欠落や malformed syntax を見つけた。parser task 21 が inline-definition name または `deffunc equals` body の欠落や malformed syntax を見つけた。parser task 23 が attribute-definition label、subject、pattern name の欠落、または malformed definition-content tail を見つけた。parser task 24 が predicate-definition label 欠落、malformed raw predicate pattern、または malformed predicate-definition tail を見つけた。parser task 25 が functor-definition label 欠落、malformed raw functor pattern、`equals` term body 欠落、malformed term case、`equals ... otherwise` term body 欠落、または malformed functor-definition tail を見つけた。parser task 26 が mode-definition label 欠落、malformed raw mode pattern、または malformed mode-definition tail を見つけた。parser task 27 が redefinition label / subject 欠落、malformed redefinition pattern、`equals` term body 欠落、malformed notation alias pattern、または malformed redefinition / notation tail を見つけた。parser task 29 が空または malformed な structure pattern、field / property name、inheritance target、field / property redefinition name、または malformed structure / inheritance member tail を見つけた | term、statement、definition-content node、redefinition node、notation alias、structure definition、inheritance definition、または structure / inheritance member を表現したままにする場合は設定する。term argument 欠落、malformed raw surface、malformed tail、欠落 delimiter は syntax recovery として保持する |
| `MalformedFormulaExpression` | parser task 14 が prefix `not`、binary connective、quantifier `st`、`holds` の後、少なくとも 1 個の variable segment 後の malformed quantifier-header separator / tail、または unmatched parenthesized-formula opener を見つけた。parser task 15 が set-comprehension `:` 後の condition formula 欠落を見つけた。parser task 16 が `assume`、`assume that`、`let ... such that`、または `given ... such that` 内の proposition formula 欠落を見つけた。parser task 18 が `consider such that` condition の欠落を見つけた。parser task 19 が conclusion proposition 欠落または不正な `then` linkable statement を見つけた。parser task 21 が `defpred means` body 欠落または不正な `then deffunc` / `then defpred` を見つけた。parser task 23 が definition-parameter formula、attribute delimiter、formula definiens body、formula case、または `otherwise` body の欠落や malformed syntax を見つけた。parser task 24 が predicate-definition colon、`means` delimiter、formula-definiens body、formula case、または `otherwise` body の欠落を見つけた。parser task 25 が functor-definition colon、`->` delimiter、body keyword、`means` formula-definiens body、formula case、`means ... otherwise` formula body、または term-case condition の欠落を見つけた。parser task 26 が mode-definition colon または `is` delimiter 欠落を見つけた。parser task 27 が redefinition colon、`is`、`->`、body keyword、`means` formula-definiens body、formula case、`means ... otherwise` formula body、term-case condition、notation alias の `for` delimiter、または redefinition body 後の必須 `coherence` keyword の欠落を見つけた | formula/proposition node が `MissingFormula` または `UnmatchedOpeningDelimiter` recovery を挿入されて表現されたまま残る場合、または recoverable syntax 後に quantifier / set-comprehension / statement / definition / redefinition / notation node が表現されたまま残る場合に設定する |
| `MalformedJustification` | parser task 17 が `by` 後の malformed citation または computation-proof syntax を見つけた。reference 欠落、malformed grouped / bulk citation、deferred template-argument tail、malformed computation option を含む。parser task 18 が `consider` / `reconsider` の mandatory simple justification の欠落または malformed syntax を見つけた。parser task 19 が conclusion または iterative equality の明示 citation tail の malformed syntax を見つけた。parser task 20 が明示的な `per cases by` citation tail の malformed syntax を見つけた。parser task 23 が definition-parameter または correctness-condition justification の malformed syntax を見つけた。parser task 26 が mandatory な `sethood` property justification の欠落または malformed syntax を見つけた。parser task 27 が optional `with` label の malformed syntax を含む mandatory redefinition coherence justification の欠落または malformed syntax を見つけた。parser task 28 が mandatory property-clause justification の欠落または malformed syntax を見つけた。parser task 29 が rejected `coherence with ...` tail を含む inheritance coherence justification の欠落または malformed syntax を見つけた | justification node、justification を要求する statement、definition content、`CoherenceCondition`、`PropertyClause`、または `InheritanceDefinition` が `MissingProofStep`、skipped malformed citation source、または delimiter recovery を挿入されて表現されたまま残る場合に設定する |
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
| `MissingEnd` | parser task 20 が `now`、`hereby`、`case`、`suppose` block に欠落 `end` を挿入する。parser task 22 は `ProofBlock` に挿入する。parser task 23 は `DefinitionBlockItem` に挿入する。parser task 29 は `StructureDefinition` または explicit `InheritanceDefinition` block に挿入する。recovery prepass も legacy / deferred block placeholder に対して挿入してよい | 挿入 placeholder の `SurfaceNodeKind::ErrorRecovery(MissingEnd)` | insertion point の zero-width range。block opener / context child を insertion range の外に保持してよい | `SyntaxDiagnosticCode::MissingEnd`。同じ recovery が source text も skip する場合を除き skipped range はない | `MissingEnd` |
| `MissingStringLiteral` | parser が string-required context で欠落 string literal を挿入した | 挿入 placeholder | insertion point の zero-width range。必須 child はない | `SyntaxDiagnosticCode::MissingStringLiteral`。skipped range はない | `MissingStringLiteral` |
| `MissingItem` | 未生成。将来の module / item parser が top-level item を期待し、次の item boundary または EOF の前で同期する | 挿入 placeholder | insertion point の zero-width range。同期 token または包含 item list の任意 context child を持て、range 外でもよい | 専用 diagnostic code はまだない。producer task が user-facing diagnostic を出す前に code を追加するか既存 code 共有を明記する。skip された source は存在する場合 `SkippedTokenRange` に属する | `MissingItem` |
| `MissingTypeExpression` | parser task 8 が reserve `for` のような declaration binder の後、または bracket `type_arg_list` 内で type expression を期待した。parser task 9 では `ChoiceTerm` の `the` 後に type operand を期待した。parser task 11 では `term qua` 後の target type を期待した。parser task 14 では `be` / `being` 後の明示的 quantifier type を期待した。parser task 15 では `is` 後の明示的 comprehension generator type を期待した。parser task 16 では `be` / `being` 後の明示的 statement variable type を期待した。parser task 18 では `reconsider ... as` target type を期待する。parser task 21 では `be` / `being` 後の inline-definition parameter type と `->` 後の `deffunc` return type を期待する。parser task 23 は通常 definition parameter で qualified-variable recovery を再利用してよい。parser task 25 は `->` 後の `func` return type を期待する。parser task 27 は `->` 後の `redefine func` return type を期待する。parser task 29 は `->` 後の structure member type と inheritance member narrowed type を期待する。`of` / `over` の term argument 欠落は missing type expression ではない | 挿入 placeholder | insertion point の zero-width range。任意の keyword / binder context child を持て、range 外でもよい | `SyntaxDiagnosticCode::MalformedTypeExpression`。純粋な挿入では skipped range はない | `MissingTypeExpression` |
| `MissingTerm` | parser task 9 が term list、parenthesized term、application argument、set enumeration、constructor field value 内で primary term を期待する。parser task 10 が structure-update field value を期待する。parser task 12 が prefix operator 後の operand を期待し、その operator に `DanglingOperator` を報告する。parser task 15 は set-comprehension syntax の recovery 時に mapper / generator placeholder を挿入してよい。parser task 16 は missing `take` witness、`set` equating identifier、equating RHS を挿入してよい。parser task 18 は missing `ReconsiderItem` identifier と equated RHS を挿入してよい。parser task 19 は iterative equality の initial term または step term 欠落を挿入してよい。parser task 21 は inline-definition name または `deffunc equals` body 欠落を挿入してよい。parser task 22 は theorem / lemma label 欠落に挿入してよい。parser task 23 は attribute-definition label、subject、pattern name 欠落に挿入してよい。parser task 24 は predicate-definition label 欠落または malformed predicate-pattern placeholder を挿入してよい。parser task 25 は functor-definition label、malformed functor-pattern placeholder、`equals` term body、term-case value、または `equals ... otherwise` term body 欠落を挿入してよい。parser task 27 は redefinition label、subject、malformed redefinition-pattern placeholder、`equals` term body、term-case value、`equals ... otherwise` term body、または raw notation-pattern placeholder 欠落を挿入してよい。parser task 29 は structure pattern、structure member name、inheritance target、inheritance member name の欠落を挿入してよい | 挿入 placeholder | insertion point の zero-width range。任意の call / delimiter / operator / context child を持て、range 外でもよい | term-list/update/comprehension/simple-statement/iterative-equality/local-definition/theorem-label/attribute-definition/predicate-definition/functor-definition/redefinition/notation/structure/inheritance 挿入では `SyntaxDiagnosticCode::MalformedTermExpression`、task-12 prefix-operator 挿入では `SyntaxDiagnosticCode::DanglingOperator`。純粋な挿入では skipped range はない | `MissingTerm` |
| `MissingFormula` | parser task 14 が `not`、`st`、`holds`、connective などの logical syntax の後に formula を期待する。parser task 15 が set-comprehension `:` 後に formula を期待する。parser task 16 が `Proposition` 内に formula を期待する。parser task 18 が `consider such that` condition を期待する。parser task 19 が conclusion proposition formula を期待する。parser task 20 が case / suppose branch の proposition または condition formula を期待する。parser task 21 が `defpred means` body formula を期待する。parser task 22 は theorem / lemma formula を期待する。parser task 23 は definition-parameter condition formula と formula-definiens value、condition、または `otherwise` formula を期待する。parser task 24 は predicate-definition formula-definiens value、condition、または `otherwise` formula を期待する。parser task 25 は functor-definition formula-definiens value、condition、`means ... otherwise` formula、または term-case condition formula を期待する。parser task 27 は redefinition formula-definiens value、condition、`means ... otherwise` formula、または term-case condition formula を期待する | 挿入 placeholder | insertion point の zero-width range。任意の keyword / operator / context child を持て、range 外でもよい | `SyntaxDiagnosticCode::MalformedFormulaExpression`。純粋な挿入では skipped range はない | `MissingFormula` |
| `MissingStatement` | parser task 19 が `then` 後に linkable statement を期待する。parser task 21 は `then deffunc` / `then defpred` を non-linkable のままにする。将来の statement parser も proof、algorithm、block statement を期待し、次の statement boundary で同期してよい | 挿入 placeholder | insertion point の zero-width range。任意の preceding keyword または block context child を持て、range 外でもよい | task-19 / task-21 の不正な `then` linkable statement では `SyntaxDiagnosticCode::MalformedFormulaExpression`。skip された source は存在する場合 `SkippedTokenRange` に属する | `MissingStatement` |
| `MissingProofStep` | parser task 17 が justification node を保持しながら、欠落した reference、grouped citation item、computation option、または computation option value を挿入する。parser task 18 は `consider` / `reconsider` の mandatory simple justification 欠落にも挿入する。parser task 19 は conclusion / iterative-equality justification の明示 citation reference 欠落にも挿入する。parser task 20 は明示的な `per cases by` tail の citation reference 欠落にも挿入する。parser task 23 は definition parameter と correctness condition で general justification recovery を再利用する。parser task 27 は mandatory redefinition coherence proof と任意の `with` label operand に general justification recovery を再利用する。parser task 28 は mandatory property clause で general justification recovery を再利用する。parser task 29 は inheritance coherence clause で general justification recovery を再利用する。将来の proof parser は missing inference step、case branch、proof-closing step にも使ってよい | 挿入 placeholder | insertion point の zero-width range。任意の proof / block / justification context child を持て、range 外でもよい | task-17 citation / computation-proof insertion、task-18 missing mandatory simple justification、task-19 malformed explicit citation tail、task-20 malformed explicit `per cases by` tail、task-23 malformed definition justification、task-27 malformed coherence justification、task-28 malformed property-clause justification、task-29 malformed inheritance-coherence justification では `SyntaxDiagnosticCode::MalformedJustification`。skip された source は存在する場合 `SkippedTokenRange` に属する | `MissingProofStep` |
| `MissingAnnotationArgument` | 未生成。将来の annotation parser が string literal や bracket argument などの annotation argument を期待する | 挿入 placeholder | insertion point の zero-width range。任意の annotation marker / list context child を持て、range 外でもよい | 専用 diagnostic code はまだない。不正または skip された source は状況に応じて `MalformedAnnotation` または `Recovery` の `SkippedTokenRange` に属する | `MissingAnnotationArgument` |
| `SkippedToken` | parser task 5 が item boundary に到達するため 1 個以上の top-level token を skip する。parser task 16 は malformed simple-statement tail を statement/item boundary まで skip してよい。parser task 17 は malformed justification tail を citation、delimiter、statement、item boundary まで skip してよい。parser task 18 は malformed `consider` / `reconsider` statement tail を statement/item boundary まで skip してよい。parser task 19 は malformed conclusion / iterative-equality tail を statement/item boundary まで skip してよい。parser task 20 は malformed block tail を semicolon、`end`、statement boundary、item boundary、EOF まで skip してよい。parser task 21 は malformed inline-definition parameter list または tail を `,`、`)`、`->`、`equals`、`means`、semicolon、`end`、statement boundary、item boundary、EOF まで skip してよい。parser task 23 から 29 は malformed definition-content または nested structure / inheritance member tail を semicolon、`field`、`property`、`coherence`、`end`、次の definition-content start、item boundary、EOF まで skip してよい | skipped input の marker | skip された source span を覆う range。必須 child はない。root-listed token leaf を重複させない場合だけ任意の synchronization owner child を付けてよい | task 5 の top-level skip では `SyntaxDiagnosticCode::UnexpectedTopLevelToken`。task-16 の skipped simple-statement tail、task-17 の skipped justification tail、task-18 の skipped `consider` / `reconsider` tail、task-19 の skipped conclusion / iterative-equality tail、task-20 の skipped block tail、task-21 の skipped inline-definition tail、task-23〜29 の skipped definition-content または nested member tail は malformed node の原因になった diagnostic を共有する。skip span は `SurfaceTrivia::skipped_token_ranges` に `SkippedTokenReason::Recovery` で必ず記録する | `SkippedToken` |
| `UnmatchedOpeningDelimiter` | parser task 8 が同期点までに対応する `]` を持たない `[` type argument を見つけた、parser task 9 が対応する closer を持たない term delimiter を見つけた、parser task 10 が対応する closer を持たない selector/update delimiter を見つけた、parser task 14 が対応する `)` を持たない parenthesized formula を見つけた、parser task 15 が対応する `}` を持たない set-comprehension `{` を見つけた、または future parser が別の opener に対応する closer を同期点 / EOF までに見つけられなかった | 通常は挿入された missing close と組になる marker | primary marker range は期待される closer または synchronization point の zero-width range。opener / context は、それが別の structural node にまだ所有されていない場合は recovery child として表せる。tree shape が non-root parent の重複を起こす場合は diagnostic secondary anchor として表す | task-8 type argument では `SyntaxDiagnosticCode::MalformedTypeExpression`、task-9 / task-10 / task-15 term delimiter では `MalformedTermExpression`、task-14 formula delimiter では `MalformedFormulaExpression`。opener span は secondary diagnostic anchor にするべきで、skip text があれば trivia に属する | `UnmatchedOpeningDelimiter` |
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
