# mizar-parser: ソース／仕様対応監査

> 正本は英語です。英語版:
> [../en/source_spec_audit.md](../en/source_spec_audit.md)。

状態: task 43の監査完了。Task 265 ownershipはcompleted Task 47までrefresh済みで、
Task 48はactive、Task 46はdeferredである。

## 範囲

この監査は、[grammar.md](./grammar.md)、[pratt.md](./pratt.md)、
[recovery.md](./recovery.md) が約束する公開 API と実装向け挙動を、ソースと
テストへ対応付ける。あわせて、task 43 が要求する Appendix A の予約語
カバレッジ確認を記録する。

この監査の権威順は `doc/spec/en/`、実行可能 `.miz` テスト、
`tests/coverage/spec_trace.toml`、expectation sidecar、design doc、source の順で
ある。言語仕様と既存`.miz` sourceは変更していない。Task 47はcanonical grammarと
矛盾するdiagnosticを除くためだけにexisting expectation 1件を変更しており、実装挙動への
rebaselineではない。

## 結果

ブロックする non-deferred の `spec_gap`、`boundary_violation`、
`repo_metadata_conflict`は見つかっていない。task 42までに実装済みのparser
挙動はsource、unit test、active parse-only corpus case、traceability metadataに
表現されている。original task-43 auditは下記deferred operator-declaration gap 1件を
記録し、Task 265はcanonical authorityを持つlater current-state row 2件を追加した。
一方はTask 47でclose済み、もう一方はactiveである。

task 43 で残した follow-up は deferred の 1 件のみである。

| ID | 分類 | フォローアップ | 状態 |
|---|---|---|---|
| P-043-01 | `source_drift` / `test_gap` | concrete な `operator_decl` parsing と、`infix_operator`、`prefix_operator`、`postfix_operator`、および infix associativity word の `left`、`right`、`none` に対する active parser corpus coverage。canonical な Appendix A、第 10 章、第 13 章はこれらの語に grammar position をすでに与えているが、現行 parser は declaration keyword を task-5 top-level notation start として認識するか、`ParseRequest::operator_fixity` 経由の Pratt metadata を消費するだけで、source から concrete operator declaration をまだ parse しない。 | deferred parser task 46 |

Task 265は次のclassified execution ownershipを識別した。Task 47はそのbounded
source/expectation/trace sliceを変更済みで、Task 48は未実装のままである。

| ID | 分類 | 証拠とexact scope | current owner |
|---|---|---|---|
| P-265-47 | `source_drift` / `test_expectation_drift` / `test_gap`（closed） | parserはomitted、explicit-`by`、proof-block `reconsider_tail`を受理し、active corpusはexact row 2件をcoverする。unchanged mixed recovery `.miz`はomitted formのerrorを期待しない。 | completed parser Task 47。semantic reconsiderはchecker authorityが所有しparser creditなし。 |
| P-265-48 | `source_drift` / `test_gap` | Chapter 7は`property_impl` means/equals formとcorrectness/coherence blockを定義するが、parserはexact property-implementation formを公開せずparse-only trace rowはdeferred。 | active parser Task 48。parse-only implementation/corpusだけを所有し、Task-39 semantic activationはgatedのまま。 |

`transitivity` は parser では明示的な future-reserved として残す。task 28 はすでに、
古い TODO 文言が canonical property production とずれていることを design drift として
記録している。現時点で `transitivity` property clause の parser grammar position はない。

## 公開 API 対応

| 公開 surface | 仕様上の約束 | ソース | テスト証拠 | 所見 |
|---|---|---|---|---|
| `parse(ParseRequest) -> ParseOutput` | crate-root の parser entry point は到達可能かつ deterministic に保つ。 | `crates/mizar-parser/src/lib.rs`, `src/grammar.rs` | `crates/mizar-parser/tests/lint_policy.rs`, `crates/mizar-parser/tests/determinism.rs`, active `mizar-test parse-only` corpus | 所見なし |
| `ParseRequest` | `source_id`、`edition`、frontend 適合済み token、source-position-aware operator fixity、string-required context を運び、resolver/build-system state を持たない。 | `src/lib.rs`, `src/grammar.rs`, `src/recovery.rs` | parser unit test、frontend seam test、`mizar-test` parse-only runner | 所見なし |
| `ParserToken` / `ParserTokenKind` | frontend token transfer object を消費し、token kind/text/range を syntax output へ保存する。 | `src/lib.rs`, `src/grammar.rs`, `src/event.rs` | parser token-preservation test、parse-only snapshot、public API 到達性 lint test | 所見なし |
| `OperatorFixityEntry`, `OperatorFixity`, `OperatorAssociativity` | term Pratt parsing は active な spelling-level fixity metadata を使う。associativity/fixity enum は意図的 exhaustive とする。 | `src/lib.rs`, `src/grammar.rs`, `src/module.rs`, `src/pratt.rs` | parser operator test、determinism test、`pass_parser_operator_terms_001`、operator fail corpus | 所見なし |
| `StringRequiredContext` | string-required parser context は forward-compatible で、現状は synthetic test context をサポートする。 | `src/lib.rs`, `src/recovery.rs` | missing-string parser unit test、enum policy lint test | 所見なし |
| `ParseOutput` | optional `SurfaceAst` と syntax diagnostics を返す。recover 不能な stray `end` は `ast = None` を返してよい。 | `src/lib.rs`, `src/recovery.rs`, `src/module.rs` | parser recovery unit test、`fail_parser_stray_end_001`、active parse-only runner | 所見なし |

## 挙動対応

| design 上の約束 | ソース | テスト／corpus 証拠 | 所見 |
|---|---|---|---|
| parsing は syntax-only: name resolution、type inference、overload selection、proof obligation、cache authority、owner-origin id を生成しない。 | `crates/mizar-parser/src/` は resolver/build/cache へ依存せず、semantic fact を生成しない。 | parser crate test と active corpus は syntax shape/diagnostics のみを検査する。 | 所見なし |
| grammar code は parser event sink と文書化済み `mizar-syntax` builder/accessor 境界を通して出力し、raw rowan layout に依存しない。 | `src/event.rs`, `src/grammar.rs`, `src/module.rs`, `src/module/annotations.rs` | parser unit test と `mizar-syntax` builder/view test | 所見なし |
| module skeleton、import、export、visibility wrapper、placeholder top-level dispatch は source order と recovery ownership を保つ。 | `src/module.rs`, `src/sync.rs` | `pass_parser_module_skeleton_001`, `pass_parser_import_items_001`, `pass_parser_export_visibility_001`, late import/export fail case | 所見なし |
| type、term、formula、statement、proof、definition、structure、registration、template、algorithm、claim、verification-clause、annotation、predicate redefinition-label surfaceはtask 36までとTask-47 `reconsider_tail` alignmentを実装済み。 | `src/module.rs`, `src/module/annotations.rs`, `src/path.rs` | `tests/coverage/spec_trace.toml`のactive parser pass/fail corpus requirement、parser unit test、parse-only snapshot | broad surfaceは実装済み。exact remaining `property_impl` exceptionはP-265-48。 |
| term Pratt parsing は `active_from`、newest active same-spelling metadata、prefix/postfix/infix binding power、固定最下位 term operator としての `qua`、non-associative diagnostic を尊重する。 | `src/grammar.rs`, `src/module.rs`, `src/pratt.rs` | parser operator unit test、`crates/mizar-parser/tests/determinism.rs`、`pass_parser_operator_terms_001`、operator fail case | 所見なし |
| formula Pratt parsing は固定 connective precedence と外側 quantifier parsing を使う。 | `src/module.rs` | `pass_parser_formula_connectives_001` と formula fail corpus | 所見なし |
| recovery は semicolon、`end`、top-level item start、category-local start、EOF で同期し、recover 可能な syntax には recovery node と diagnostic を出す。 | `src/recovery.rs`, `src/sync.rs`, `src/module.rs`, `src/module/annotations.rs` | fail parser corpus、parser recovery unit test、task-37 consolidation case | 所見なし |
| stray unmatched `end` は意図的に recover 不能で、diagnostics と `ast = None` を返す。 | `src/recovery.rs` | `fail_parser_stray_end_001`、parser recovery unit test | 所見なし |
| parser determinism と frontend cache readiness を維持する: global state、hidden cache、salsa dependency を持たない。 | `src/lib.rs`, `src/grammar.rs`, `src/module.rs` | `crates/mizar-parser/tests/determinism.rs`、parser fuzz target、task 41 frontend passthrough audit | 所見なし |

## 予約語カバレッジ

task 43 では `crates/mizar-test/tests/metadata.rs` に機械的ガード
`repository_parser_reserved_words_are_covered_or_explicitly_deferred` を
追加した。このテストは `doc/spec/en/appendix_a.grammar_summary.md` の予約語ブロックを
読み、active parser `.miz` corpus source を frontend preprocessing と tokenization に
通し、frontend `ReservedWord` token だけを数える。予約語が active parser corpus に
`ReservedWord` token として出現せず、かつ parser-deferred reserved word list にも
ない場合に失敗する。deferred reserved word が Appendix A から消えた場合、または
active parser corpus source に `ReservedWord` token として出現し始めた場合も、
監査更新を促すために失敗する。

現在の parser-deferred reserved word は次のとおり。

| 予約語 | 理由 |
|---|---|
| `infix_operator` | concrete な source-level operator declaration は P-043-01 / parser task 46 に deferred。 |
| `prefix_operator` | concrete な source-level operator declaration は P-043-01 / parser task 46 に deferred。 |
| `postfix_operator` | concrete な source-level operator declaration は P-043-01 / parser task 46 に deferred。 |
| `left` | deferred された concrete `infix_operator` declaration 内でのみ使う infix associativity value。 |
| `right` | deferred された concrete `infix_operator` declaration 内でのみ使う infix associativity value。 |
| `none` | deferred された concrete `infix_operator` declaration 内でのみ使う infix associativity value。 |
| `transitivity` | provisional な Appendix A word list では予約済みだが、canonical な実装済み property production には含まれない。task 28 が design drift を記録済みであり、現時点の parser grammar position はない。 |

その他すべての Appendix A 予約語は、少なくとも 1 つの active parser corpus `.miz`
source に frontend `ReservedWord` token として出現している。

## Task 47 source/specification再確認

Task 47はP-265-47A/B/Cをcloseする。private `parse_reconsider_statement_at`はChapter 4
§4.4.2、8 §8.2.2、15 §§15.5.1/15.8.2/15.12、Appendix A §§A.4/A.15のomitted、
simple-justification、proof-block tailと一致する。active parse-only corpusは新たにcoveredと
したrequirement 2件のexact backlinkを持ち、既存`.miz`を変えずにhistorical omitted-tail
expectation driftを除去した。

P-265-47Dはnonblockingでhuman-ownedの`spec_gap`として残る。Chapter 8のcompact EBNFは
`reconsider_item` 1件と書く一方、Chapters 4/15とAppendix Aはlistを使う。Task 47は既存の
source-order listを維持し、`doc/spec`を編集しない。P-265-48はauthorizedな
property-implementation `source_drift` / `test_gap`、P-046はdeferredのままである。
`source_undocumented_behavior`、`boundary_violation`、`repo_metadata_conflict`はない。
