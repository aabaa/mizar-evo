# Crate Exit Report: mizar-syntax

## Parser Task 46 post-exit addendum

Parser Task 46はexit済みsyntax vocabularyを`OperatorDeclaration`とappend-only
`SyntaxKind::OperatorDeclaration = 193`で拡張する。対応するsurface kind、typed
accessor、snapshot/raw/node/rowan contract、testは以前のdiscriminantを保存する。
このsyntax-only incrementはhistorical crate exitを再採点せず、reopenしない。

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## 結果

状態: task 25 の AST refactor follow-up audit 後に refresh した close-out。
task-35 autonomous `mizar-syntax` milestone は下記 score の historical basis として
残す。post-exit task 22 から task 25 は完了済みであり、S-021 rustdoc summary は
policy trigger により明示的に deferred のまま残る。下記の Parser Task 48
vocabulary addendum は post-exit の syntax increment であり、その historical
milestone を再採点せず、再開もしない。

品質スコア: reviewed 94/100。

適用された score cap: historical report 時点ではなし。S-025 再監査時点でも、
`mizar-syntax` scope には新しい未解決の hard gate failure、
`source_undocumented_behavior`、`test_expectation_drift`、boundary violation、
repo metadata conflict は把握していない。現在残る work は policy-triggered
S-021 rustdoc deferral のみである。

## 範囲

Milestone scope:

- `mizar-syntax` が必要とする rowan-backed `SurfaceAst`、syntax vocabulary、
  trivia、recovery、diagnostic、typed accessor、snapshot contract を完了する。
- syntax task completion に必要な paired parser-facing syntax work を parser task
  4-36 まで完了する。
- original historical report の後に parser task 36 / syntax task 22 の
  predicate-label follow-through、task 24 の private AST source split、task 25 の
  follow-up audit が着地し、S-023 / S-025 audit で追跡されていることを記録する。
- syntax representation を source-shaped に保ち、semantic name、type、proof、
  VC behavior を入れない。
- source/spec/test correspondence、bilingual synchronization、本 crate exit evidence
  を記録する。

Included:

- rowan-backed storage、deterministic green-tree / text snapshot、raw-kind
  compatibility、typed compatibility view、task 35 までの現在の syntax vocabulary、
  parser task 36 の predicate redefinition label slot。
- green-tree construction、stable snapshot rendering、AST tests の private AST
  implementation partition。公開 `mizar_syntax` API path は変更していない。
- comment、doc-comment attachment hint、skipped-token range、whitespace hint 用の
  syntax-owned trivia side table。
- recovery kind と syntax diagnostic surface。active parser producer と
  vocabulary-only future producer を含む。
- module、import/export、type、term、formula、statement、theorem/proof、
  definition、registration、template、algorithm、verification clause、annotation
  surface の parser/syntax pairing evidence。
- Rust unit / lint / snapshot tests、active parser `.miz` parse-only coverage、
  expectation sidecar、traceability metadata。
- crate plan、source/spec audit、bilingual audit、TODO、本 report を含む英語・日本語
  design documents。

Excluded:

- current implementation に合わせるためだけの `doc/spec/en`、`doc/spec/ja`、既存
  `.miz` tests、expectation sidecar、syntax snapshot の変更。
- resolver-owned namespace classification、type checking、overload resolution、
  cluster facts、theorem validity、proof obligation、algorithm VC generation、
  package/build semantics。
- lexer / raw comment extraction、frontend cache orchestration、paired task map
  外の parser grammar ownership、`salsa` query integration。
- 現在 vocabulary-only と明示されている recovery kind の producer-backed tests、
  および dotted algorithm `Lvalue` の active `.miz` coverage。これは owning
  frontend/parser dot-role increment が unrelated diagnostics なしに surface を運べるまで
  deferred。
- S-021 re-entry trigger が満たされるまでの rustdoc summary。

## Hard Gates

| Gate | 状態 | 根拠 |
|---|---|---|
| Specification consistency | Pass | [00.crate_plan.md](./00.crate_plan.md) が syntax behavior を `doc/spec/en`、active `.miz` coverage、traceability metadata、module specs へ対応付ける。S-019 は新しい `spec_gap` や blocking inconsistency を見つけなかった。 |
| Test contract | Pass with explicit deferred rationale | Rust tests は builder/accessor、rowan/raw-kind、snapshot、trivia、recovery、diagnostic、lint-policy contract を覆う。Active parser `.miz` cases と expectation sidecars は task 35 までの parser-facing syntax と parser task 36 の predicate redefinition label repair を覆う。Deferred seed rows、vocabulary-only recovery producers、dotted `Lvalue` active `.miz` coverage、rustdoc summaries は owner と unblock condition を明示している。 |
| Traceability | Pass | [source_spec_correspondence.md](./source_spec_correspondence.md) が public API、method-level behavior、enum / diagnostic surface、source files、test evidence を trace する。`tests/coverage/spec_trace.toml` は active / planned parser-facing cases を記録する。 |
| Design/source sync | Pass | S-019 は unimplemented promised public behavior や undocumented implementation-facing public behavior を見つけなかった。S-020 は英語・日本語 design docs を同期した。S-023 と S-025 の再監査では、status / source-layout record の更新後に predicate-label または AST-refactor drift は残っていない。 |
| Boundary discipline | Pass | [README.md](./README.md)、[00.crate_plan.md](./00.crate_plan.md)、module specs は lexer、parser、resolver、type、proof、VC、cache、build-system responsibility を `mizar-syntax` の外に保っている。 |
| Verification | Pass | 現在 branch の verification result は下に記録する。format、clippy、workspace tests、traceability plan はすべて pass。 |
| Residual risk | Pass | 残る item は `MSYN-GAP-001`、`MSYN-GAP-003`、`MSYN-GAP-013`、または deferred S-021 rustdoc work として分類済み。blocking/high syntax-owned finding は残っていない。 |

## スコア内訳

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

deferred の parser/frontend-owned `.miz` activation、vocabulary-only recovery
producer、dotted algorithm `Lvalue` の active-corpus gap、policy-triggered rustdoc
deferral に対して小さな減点を残す。これらは文書化済みの risk であり、hard gate
failure ではない。

## Deferred Items

| ID | 理由 | Owner | Unblock condition |
|---|---|---|---|
| `MSYN-GAP-001` | [parse_only_fixture_seed.md](./parse_only_fixture_seed.md) は future grammar activation point 用の inactive parser-facing seed rows を保持する。 | Owning future parser/syntax task | owning parser grammar task が frontend parser seam 経由で expectation を満たせるときだけ各 row を activate し、意図を変えずに `.miz`、`.expect.toml`、traceability metadata を追加または更新する。 |
| `MSYN-GAP-003` | 一部の recovery kind は `mizar-syntax` で constructible だが、現在の parser producer がまだ emit しないため vocabulary-only として文書化されている。 | Owning parser recovery producer tasks | future parser producer が該当 recovery kind を emit する時点で [recovery.md](./recovery.md)、producer tests、syntax/parser evidence を更新する。 |
| `MSYN-GAP-013` | Dotted algorithm `Lvalue` は parser unit tests で表現・テスト済みだが、active `.miz` coverage は、その dotted assignment surface を unrelated diagnostics なしに保てる frontend/parser dot-role increment を待つ。 | Parser/frontend dot-role integration | owning increment が frontend seam 経由で surface をきれいに運べるようになったら、active `.miz` coverage と traceability を追加する。 |
| S-021 | Rustdoc summaries は、フロントエンドパイプライン外の長命な consumer が `mizar-syntax` に対してコーディングを始めるか、workspace が rustdoc policy を採用するまで deferred。 | Syntax maintenance after policy/consumer trigger | S-021 に再着手し、canonical ではない API documentation として rustdoc summaries を追加し、その時点で存在する workspace rustdoc/lint policy checks を実行する。 |

この milestone で解決した項目:

- `MSYN-GAP-002`、`MSYN-GAP-004`、`MSYN-GAP-005`、`MSYN-GAP-006`、
  `MSYN-GAP-007`、`MSYN-GAP-008`、`MSYN-GAP-011`、`MSYN-GAP-012` は
  [00.crate_plan.md](./00.crate_plan.md) に記録された task sequence で閉じた。
- `MSYN-GAP-009` は process note として残る。parser と syntax の task number が
  食い違う場合、crate plan pairing map が authority である。
- `MSYN-GAP-010` は metadata watch item として残る。現在 repository metadata
  conflict は観測されていない。

## Human Review Surface

human reviewer は主に次を確認する。

- [00.crate_plan.md](./00.crate_plan.md)
- 本 report
- [README.md](./README.md)
- [ast.md](./ast.md)
- [trivia.md](./trivia.md)
- [recovery.md](./recovery.md)
- [grammar_audit.md](./grammar_audit.md)
- [parse_only_acceptance_matrix.md](./parse_only_acceptance_matrix.md)
- [parse_only_fixture_seed.md](./parse_only_fixture_seed.md)
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
- [todo.md](./todo.md)
- `crates/mizar-syntax/src/lib.rs`
- `crates/mizar-syntax/src/ast.rs`
- `crates/mizar-syntax/src/ast/green.rs`
- `crates/mizar-syntax/src/ast/snapshot.rs`
- `crates/mizar-syntax/src/ast/tests.rs`
- `crates/mizar-syntax/src/trivia.rs`
- `crates/mizar-syntax/src/recovery.rs`
- `crates/mizar-syntax/tests/lint_policy.rs`
- `tests/snapshots/mizar_syntax_surface_ast_current_vocabulary.snap`
- `tests/coverage/spec_trace.toml`
- [00.crate_plan.md](./00.crate_plan.md) が参照する active parser `.miz` と
  `.expect.toml` files
- 同じ `mizar-syntax` design files の日本語 companion

この report が参照する canonical language/test surface には次が含まれる。

- `doc/spec/en/03.type_system.md`
- `doc/spec/en/05.structures.md`
- `doc/spec/en/06.attributes.md`
- `doc/spec/en/07.modes.md`
- `doc/spec/en/09.predicates.md`
- `doc/spec/en/10.functors.md`
- `doc/spec/en/11.symbol_management.md`
- `doc/spec/en/12.modules_and_namespaces.md`
- `doc/spec/en/13.term_expression.md`
- `doc/spec/en/14.formulas.md`
- `doc/spec/en/15.statements.md`
- `doc/spec/en/16.theorems_and_proofs.md`
- `doc/spec/en/17.clusters_and_registrations.md`
- `doc/spec/en/18.templates.md`
- `doc/spec/en/20.algorithm_and_verification.md`
- `doc/spec/en/21.source_code_annotation_and_atp.md`
- `doc/spec/en/appendix_a.grammar_summary.md`
- `doc/spec/en/appendix_b.operator_precedence.md`

この final report task は、source implementation、`.miz` file、`.expect.toml`
file、syntax snapshot、canonical language specification file を変更しない。

## Test Expectation Summary

この final report task は `.expect.toml` files、snapshots、`.miz` files、Rust source
を変更しない。この milestone は次の expectation group に依存する。

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| `tests/snapshots/mizar_syntax_surface_ast_current_vocabulary.snap` と `src/ast/tests.rs` を含む `crates/mizar-syntax` Rust tests | rowan-backed syntax storage、deterministic snapshot、task 35 までの typed accessor と predicate redefinition label-slot follow-through、trivia attachment、recovery vocabulary、diagnostics、lint policy を guard する。 | Pass | Rust unit/lint/snapshot | 文書化済みの negative invariant test でだけ panic。 | [ast.md](./ast.md)、[trivia.md](./trivia.md)、[recovery.md](./recovery.md) |
| `tests/miz/pass/parser/pass_parser_minimal_token_stream_001.expect.toml` と task-5 module/recovery sidecars | frontend-reachable parser/syntax baseline と module skeleton recovery を guard する。 | sidecar が encode する Pass または fail/recover | Parse-only | expected recovery fixture に限る syntax diagnostics。 | `spec.en.syntax.parser_minimal_token_stream`、`spec.en.syntax.parser_recovery.*`、Chapter 12 |
| parser tasks 6-8 の import/export/type sidecars | import/export/visibility と type-expression syntax surface を guard する。 | sidecar が encode する Pass または fail/recover | Parse-only | malformed import/export/type fixture 用 syntax diagnostics。 | Chapters 3 and 12、Appendix A |
| parser tasks 9-15 の term/formula sidecars | primary term、selector/update、`qua`、operator term、set comprehension、atomic formula、connective、quantifier、constant、formula recovery を guard する。 | sidecar が encode する Pass または fail/recover | Parse-only | missing operand、malformed delimiter、non-associative `iff` などの recovery fixture 用 syntax diagnostics。 | Chapters 13 and 14、Appendix B |
| parser tasks 16-22 の statement/proof sidecars | simple statement、justification、`consider` / `reconsider`、conclusion、iterative equality、block statement、inline definition、theorem/proof item、proof recovery を guard する。 | sidecar が encode する Pass または fail/recover | Parse-only | malformed statement、justification、proof recovery fixture 用 syntax diagnostics。 | Chapters 15、16、parser-facing Chapter 20 hosts |
| parser tasks 23-30 の definition/structure/registration sidecars | definition block、predicate/functor/mode definition、redefinition、notation、property、structure、inheritance、registration、cluster、reduction、recovery を guard する。 | sidecar が encode する Pass または fail/recover | Parse-only | malformed definition-family / registration fixture 用 syntax diagnostics。 | Chapters 5-7、9-11、16、17 |
| parser tasks 31-35 の template/algorithm/annotation sidecars | template arguments/references、algorithm、claim、control flow、verification clause、annotation、malformed annotation/algorithm recovery を guard する。 | sidecar が encode する Pass または fail/recover | Parse-only | chained `iff`、malformed algorithm/claim/control-flow/verification/annotation fixture、expected recovery node 用 syntax diagnostics。 | Chapters 18、20、21、Appendix A |
| Deferred seed rows and `MSYN-GAP-013` | future executable intent を保持しつつ、未対応 producer が今日存在するとは扱わない。 | Planned/deferred | unblock まで Parse-only または parser-unit-only | owner が producer できるまで active runner diagnostics は不要。 | [parse_only_fixture_seed.md](./parse_only_fixture_seed.md)、[00.crate_plan.md](./00.crate_plan.md) |

## Verification

この crate exit task のために実行したコマンド:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

結果:

- `cargo fmt --check`: passed。
- `cargo clippy --all-targets --all-features -- -D warnings`: passed。
- `cargo test`: workspace 全体で passed。`mizar-syntax` unit tests と lint-policy
  tests を含む。
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: 162 test cases、90 requirements、0 errors、`mizar-syntax` scope 外の既存 planned/no-tests warnings 4 件とともに passed。
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- `MSYN-GAP-013` を unblock できる owning frontend/parser dot-role integration、
  または vocabulary-only `SyntaxRecoveryKind` を active producer-backed coverage に
  変える future parser recovery producer task を開始する。

Known constraints:

- resolver、type、proof、VC、lexer、frontend cache、parser-owned grammar decision を
  `mizar-syntax` へ移さない。
- current implementation behavior に合わせるためだけに `.miz`、expectation、spec を
  変更しない。
- syntax identity は deterministic だが non-persistent のまま保つ。future query layer は
  frontend/build-owned content key の背後で syntax output を cache する。
- 英語 design docs と日本語 companion を同じ変更で同期する。

Open questions:

- どの frontend/parser dot-role milestone が最初に dotted algorithm assignment を
  active `.miz` parse-only coverage へ運ぶべきか。
- どの future parser recovery producer が最初に現在 vocabulary-only の recovery kind を
  activate するべきか。
- 長命な resolver または LSP consumer はいつ S-021 rustdoc summaries を trigger するか。

次タスクの推奨 reasoning setting:

- `high`。次の有用な作業は parser/frontend/syntax boundary、active `.miz` coverage、
  traceability metadata、recovery または dot-role diagnostics を横断するため。
  documentation-only follow-up なら `medium` へ下げてよく、canonical grammar または
  semantic language behavior を変更する場合は `high` より上げる。

## Parser Task 48 post-exit addendum

Parser Task 48 は、すでに exit 済みの syntax vocabulary を top-level
`PropertyImplementation` node と append-only な
`SyntaxKind::PropertyImplementation = 192` で拡張する。対応する
`SurfaceNodeKind`、typed accessor、snapshot rendering、raw-kind / node-kind round
trip、rowan boundary は Task 48 の syntax changes と tests で覆う。nested parameter
は source-shaped な
`DefinitionParameter -> TypeHead -> QualifiedSymbol + optional TypeArguments`
として残り、parser の active Task 48 pass / fail corpus は正しい path と bounded
recovery path を覆う。

この addendum は `SPEC-07-PI-PLACEMENT` について syntax-only の完了を記録する。
semantic property validation は主張せず、semantic Task 39 に deferred のまま残す。
また、新しい `mizar-syntax` task ID を導入しない。したがって、上記の historical
94/100 score と S-025 exit determination は変更しない。
