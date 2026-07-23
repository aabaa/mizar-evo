# Crate Exit Report: mizar-parser

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## 結果

状態: 現在の `mizar-parser` crate milestone は complete。

品質スコア: closeout 文書同期後の独立監査で 94/100。

適用された score cap: なし。この report が存在する前は、paired exit report の
欠落と stale な status index が `design_drift` であり、hard gate 5 は fail、score
は無効で最大 89 だった。`PARSER-CRATE-CLOSEOUT` はその drift を閉じる。

この結果は parser crate milestone だけを閉じる。global Step 5 を閉じず、parser
Task 46 を昇格せず、Task 49 を作成・昇格せず、Steps 6/7 を許可しない。

## 範囲

Milestone scope:

- parser Tasks 1-45 と 47-48。
- frontend 適合済み token input、source-shaped `SurfaceAst` output、grammar / Pratt
  parsing、syntax recovery、deterministic parser behavior、active parse-only corpus。
- source/spec correspondence、reserved-word coverage、bilingual synchronization、
  public-enum policy、Task-47/48 increment。
- 下記に記録する現在の parser/syntax/corpus/trace/count/hash evidence。

Included:

- public parser transfer objects と syntax-only parser entry point。
- completed parser tasks が表す grammar surface。canonical な3つの
  `reconsider_tail` form と top-level `PropertyImplementation` を含む。
- parser unit/determinism/lint-policy、syntax、frontend、real parse-only runner evidence。
- paired EN/JA parser design documents と本 exit report。

Excluded:

- aliased P-043-01 / P-046 concrete operator-declaration gap。named frontend
  string-required context が存在するまで deferred。
- Chapter 8 と Chapters 4/15 + Appendix A の `reconsider` list form の間に残る
  nonblocking human-owned P-265-47D wording `spec_gap`。
- resolver/checker、semantic property/coherence decision、proof acceptance、
  Core/CFG/VC、artifact、global Step-5 completion。
- authority-approved task がない future grammar growth と Steps 6/7。

current-milestone complete は、考え得るすべての、または future canonical grammar
production の実装完了を意味しない。新しい grammar work は fresh canonical
authority から開始しなければならない。

## Hard Gates

| Protocol gate | 状態 | 根拠 |
|---|---|---|
| 1. Blocking/high specification inconsistency なし | Pass | Canonical EN spec、active `.miz` corpus、exact trace rows、parser plan は completed Tasks 1-45/47-48 で一致する。P-265-47D は nonblocking human-owned。 |
| 2. Undocumented source behavior なし | Pass | [source_spec_audit.md](./source_spec_audit.md) は public surface と promised behavior を source/tests へ trace する。未解決 `source_undocumented_behavior` はない。 |
| 3. Milestone-owned coverage または明示 deferral | Pass | 43 parse-only requirements はすべて covered。P-043-01/P-046 は external trigger、owner、rationale を明示する。 |
| 4. Expectation integrity | Pass | Task 48 は既存 `.miz`/expectation を変更していない。Task 47 の1 sidecar diagnostic は canonical grammar へ向けた stale expectation 修正で、implementation-derived rebaseline ではない。 |
| 5. Design/source synchronization | Pass | Paired parser docs、crate/top indexes、Task-48 completion record、本 paired report は source/current oracle と一致する。 |
| 6. Responsibility boundaries | Pass | Parsing は syntax-only。resolver/checker/proof/Core/CFG/VC/artifact/cache/build responsibility を `mizar-parser` へ移していない。 |
| 7. Coverage-audit synchronization | Pass | Task 48 は実 coverage change を audit に反映済み。本 docs-only closeout は coverage mapping、trace status、owner、follow-up ownership、deferred rationale を変えないため追加 edit 不要。 |
| 8. Verification | Pass | Focused crate tests、format、denied-warning workspace Clippy、full workspace tests、5 CLI、count/hash manifests、diff checks が pass。 |
| 9. Residual-risk classification | Pass | 残る parser item は deferred P-043-01/P-046 と human-owned P-265-47D。`boundary_violation` / `repo_metadata_conflict` はない。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 15/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

減点は明示 deferred operator-declaration surface、human-owned `reconsider` wording
gap、次 parser task authority がないことによる。hard-gate failure ではない。

## Deferred Items

| ID | Classification と理由 | Owner | Unblock condition |
|---|---|---|---|
| P-043-01 / P-046 | `source_drift` / `test_gap`: concrete operator declaration と reserved-word corpus は current milestone 外。2つの ID は同一 gap を指す。 | Deferred `mizar-parser` Task 46 | Future `mizar-frontend` string-required operator-declaration context が canonical に authorized となり Task 46 を scope に入れる。 |
| P-265-47D | Nonblocking human-owned `spec_gap`: Chapter 8 は単一 `reconsider_item`、Chapters 4/15 と Appendix A は list を記す。 | Human specification owner | English canonical wording を明示的に調停する。current parser behavior から spec edit を推測しない。 |

## Human Review Surface

主要 review surface:

- [00.crate_plan.md](./00.crate_plan.md)、[todo.md](./todo.md)、本 report。
- [grammar.md](./grammar.md)、[pratt.md](./pratt.md)、
  [recovery.md](./recovery.md)、[source_spec_audit.md](./source_spec_audit.md)。
- paired English/Japanese documents。
- `doc/spec/en/`、`tests/coverage/spec_trace.toml` が参照する active parser
  `.miz`/sidecars、exact Task-47/48 corpus files。
- `crates/mizar-parser/src/`、paired `mizar-syntax` vocabulary、parser
  determinism/lint tests、`crates/mizar-test/tests/metadata.rs`。

`PARSER-CRATE-CLOSEOUT` は source、specification、`.miz`、expectation、snapshot、
traceability file を変更しない。

## Test Expectation Summary

| Evidence group | Intent | Expected phase/outcome | Specification surface |
|---|---|---|---|
| Parser unit/determinism/lint-policy tests | AST ownership、recovery boundary、deterministic output、public policy を守る。 | Rust tests pass。expectation file change なし。 | Completed parser task contract と paired design docs。 |
| `tests/miz/pass/parser/pass_parser_reconsider_tails_001.miz` と `tests/miz/pass/parser/pass_parser_reconsider_tails_001.expect.toml` | Task 47 additive coverage。omitted / proof-block tail を semantic acceptance なしで覆う。 | `parse_only` pass、`diagnostic_codes = []`。 | Chapters 4/8/15、Appendix A。 |
| `tests/miz/fail/parser/fail_parser_consider_reconsider_recovery_001.miz` と `tests/miz/fail/parser/fail_parser_consider_reconsider_recovery_001.expect.toml` | Task 47 は `.miz` を byte-identical に保ち、canonical `reconsider x as set;` 用の obsolete `malformed_justification` だけを削除し、他の recovery diagnostics/intent を維持した。 | Existing fail sidecar は parse-only recovery contract として pass。 | Chapter 15 parser syntax と P-265-47B。 |
| `tests/miz/pass/parser/pass_parser_property_implementations_001.miz` と `tests/miz/pass/parser/pass_parser_property_implementations_001.expect.toml` | Task 48 additive means/equals property coverage。 | `parse_only` pass、`diagnostic_codes = []`。 | Chapters 7/12、Appendix A。 |
| `tests/miz/fail/parser/fail_parser_property_implementations_recovery_001.miz` と `tests/miz/fail/parser/fail_parser_property_implementations_recovery_001.expect.toml` | Task 48 additive malformed parameter/body/correctness と following-item recovery。既存 expectation rebaseline なし。 | exact 13 diagnostics を持つ `parse_only` fail contract。 | Chapters 7/12、Appendix A。 |

## Verification

コマンド:

```text
cargo test -p mizar-parser
cargo test -p mizar-syntax
cargo test -p mizar-frontend
cargo test -p mizar-test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- parse-only --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- declaration-symbol --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- type-elaboration --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- proof-verification --tests-root tests --manifest tests/coverage/spec_trace.toml
git diff --check
```

結果:

- parser 221 unit / 3 determinism / 14 lint-policy、syntax 54 unit / 8 lint-policy
  tests が pass。
- plan 407 cases / 369 requirements、pass/fail 222/185、warnings/errors 23/0、
  parse/declaration/type/proof coverage 43/43、10/5、236/224、4/1。
- active parse/declaration/type/proof admission 99/5/188/1 はすべて pass。
- plan/parse/declaration/type/proof stdout hash は順に
  `2957a40b91a4cf64206301b4bf91d1c42ecdac2a564b70af370d2e52333ab57b`、
  `c9dcbcef79e727f31720d46532febe5a20e02a7710cf691e49d89fcfb69bccfa`、
  `210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`、
  `1dadbeabb219f5853c713ad53aa1cc7cd720a0e80abd7f882e9e0a5ea7802625`、
  `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。
- raw/normalized 276-test-list hash は
  `967495e78e1068f592e64834ea3ffb9eac9c25692ea5cbd4f11006a679c66590` と
  `1be4ae09188b27a40814adc6597de4806dabb13bcac019b294154e1455072adf`。
- `mizar-test` production は 18 paths / 20,088 lines、path/content hash は
  `63e4e770b0d10872415548410d417071c1901f3ffa5aea964a81d2dbbc572ed0` と
  `7e5adca22db2b73f94f04c406f10788f2cd49ba48109bb105a3fd076c339d560`。
- parser production は 12 paths / 38,256 lines、path/content hash は
  `192f9d0b5e6534c4daab010ec51a9356e9e0fd6fb86876bd2600a75844e7566a` と
  `3728e0ac374c11b3ef0553379d2e9affcd861513e004dfee80589b47bcf2130a`。
- format、Clippy、crate/workspace tests、diff checks は pass。

23 plan warnings は既存 cross-workspace soundness/corpus-size warning であり、
parser-closeout error ではない。plan errors は 0。

## Handoff

Next recommended work: current authority 下の `mizar-parser` にはない。future
parser task の前に fresh inventory が必要。P-043-01/P-046 は named frontend
context が authorized になった場合のみ再開できる。

Known constraints: global Step 5 は本 parser milestone 外で active のまま。
Task 49 を推測せず、Steps 6/7 を昇格しない。parser work は syntax-only とし、
authority order と bilingual synchronization を維持する。

推奨 reasoning setting: 次の cross-crate authority inventory または canonical
grammar/semantic decision は `xhigh`。bounded かつ authorized 済み parser-only
implementation は `high`、pure docs refresh は `medium`。
